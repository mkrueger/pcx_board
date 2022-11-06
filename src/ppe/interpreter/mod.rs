use std::string::String;
use std::collections::HashMap;

pub mod expressions;
use ppl_engine::ast::*;
use ppl_engine::tables::PPL_TRUE;

use crate::VT;

pub use self::expressions::*;

pub mod statements;
pub use self::statements::*;

pub mod io;
pub use self::io::*;

mod tests;

pub trait ExecutionContext
{
    fn vt(&mut self) -> &mut VT;

    fn gotoxy(&mut self, x: i32, y: i32);
    fn print(&mut self, str: &str);
    fn read(&mut self) -> String;
    fn get_char(&mut self) -> Option<char>;

    /// simulate user input for later processing
    fn send_to_com(&mut self, data: &str);
}

pub struct StackFrame {
    values: HashMap<String, VariableValue>,
    cur_ptr: usize,
    label_table: i32
}

pub struct Interpreter<'a> {
    prg: &'a Program,
    ctx: &'a mut dyn ExecutionContext,
    // lookup: HashMap<&'a Block, i32>,
    label_tables: Vec<HashMap<String, usize>>,
    cur_frame: Vec<StackFrame>,
    io: &'a mut dyn PCBoardIO,
    pub is_running: bool
   //  stack_frames: Vec<StackFrame>
}


pub fn create_array(interpreter: &mut Interpreter, var_type: VariableType, var_info: &VarInfo) -> VariableValue
{
    match var_info {
        VarInfo::Var0(_) => panic!(""),
        VarInfo::Var1(_, vec) => { 
            let dim = get_int(&evaluate_exp(interpreter, vec));
            let mut v = Vec::new();
            v.resize(dim as usize, VariableValue::Integer(0));
            VariableValue::Dim1(var_type, v)
        },
        VarInfo::Var2(_, vec, mat) => VariableValue::Dim2(var_type, Vec::new()),
        VarInfo::Var3(_, vec, mat, cub) => VariableValue::Dim3(var_type, Vec::new()),
    }
}

pub fn set_array_value(arr: &mut VariableValue, var_info: &VarInfo, val: VariableValue, dim1: usize)
{
    match var_info {
        VarInfo::Var0(_) => panic!(""),
        VarInfo::Var1(_, v) => {
            if let VariableValue::Dim1(_, data) = arr {
                data[dim1] = val;
            }
        },
        VarInfo::Var2(_, v, m) => todo!(),
        VarInfo::Var3(_, v, m, c) => todo!(),
    }
}


pub fn get_first_index(var_info: &VarInfo) -> &Expression
{
    match var_info {
        VarInfo::Var1(_, v) => { v }
        VarInfo::Var2(_, v, m) => { v }
        VarInfo::Var3(_, v, m, c) => { v }
        _ => panic!(""),
    }
}


fn execute_statement(interpreter: &mut Interpreter, stmt: &Statement)
{
    // println!("execute {:?}", &stmt);

    match stmt {
        Statement::Let(variable, expr) => {
            let value = evaluate_exp(interpreter, expr);
            let var_name = variable.get_name().clone();
            let var_type = interpreter.prg.get_var_type(&var_name);
            let var_info = interpreter.prg.get_var_info(&var_name).unwrap();

            if var_info.is_array() {
                let dim1 = &evaluate_exp(interpreter, get_first_index(variable));
                let val  = match interpreter.cur_frame.last_mut().unwrap().values.get_mut(&var_name) {
                    Some(val) => {
                        val
                    }
                    None => {
                        let arr = create_array(interpreter, var_type, var_info);
                        interpreter.cur_frame.last_mut().unwrap().values.insert(var_name.clone(), arr);
                        interpreter.cur_frame.last_mut().unwrap().values.get_mut(&var_name).unwrap()
                    }
                };

                set_array_value(val, var_info, value, get_int(dim1) as usize - 1);

            } else {
                interpreter.cur_frame.last_mut().unwrap().values.insert(var_name, convert_to(var_type, &value));
            }
        }
        Statement::Goto(label) => {
            let table = &interpreter.label_tables[interpreter.cur_frame.last().unwrap().label_table as usize];
            interpreter.cur_frame.last_mut().unwrap().cur_ptr = *table.get(label).unwrap();
        }
        Statement::Call(def, params) => {
            call_predefined_procedure(interpreter, def, params);
        }
        Statement::ProcedureCall(name, parameters) => {
            let mut found = false; 
            for f in &interpreter.prg.procedure_implementations {
                if let Declaration::Procedure(pname, params) =  &f.declaration {
                    if name != pname { continue; }
                    let mut prg_frame = StackFrame { 
                        values: HashMap::new(),
                        cur_ptr:0,
                        label_table:0 
                    };

                    for i in 0..parameters.len() {
                        if let Declaration::Variable(var_type, infos) = &params[i] {
                            let value = evaluate_exp(interpreter, &parameters[i]);
                            prg_frame.values.insert(infos[0].get_name().clone(), convert_to(*var_type, &value));
                        } else  {
                            panic!("invalid parameter declaration {:?}", params[i]);
                        }
                    }

                    interpreter.cur_frame.push(prg_frame);

                    while interpreter.cur_frame.last().unwrap().cur_ptr < f.block.statements.len() {
                        let stmt = &f.block.statements[interpreter.cur_frame.last().unwrap().cur_ptr as usize];
                        execute_statement(interpreter, stmt);
                        interpreter.cur_frame.last_mut().unwrap().cur_ptr += 1;
                    }
                    interpreter.cur_frame.pop();
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("procedure not found {}", name);
            }
        },

        Statement::If(cond, statement) => {
            let value = evaluate_exp(interpreter, cond);
            if let VariableValue::Integer(x) = value {
                if x == PPL_TRUE {
                    execute_statement(interpreter, statement);
                }
            } else if let VariableValue::Boolean(x) = value {
                if x {
                    execute_statement(interpreter, statement);
                }
            } else {
                panic!("no bool value {:?}", value);
            }

        }

        Statement::Inc(expr) => {
            let new_value = evaluate_exp(interpreter, &Expression::Identifier(expr.clone())) + VariableValue::Integer(1);
            interpreter.cur_frame.last_mut().unwrap().values.insert(expr.to_string(), new_value);
        }

        Statement::Dec(expr) => {
            let new_value = evaluate_exp(interpreter, &Expression::Identifier(expr.clone())) + VariableValue::Integer(-1);
            interpreter.cur_frame.last_mut().unwrap().values.insert(expr.to_string(), new_value);
        }

        /* unsupported - the compiler does not generate them and ast transformation should remove them */
        Statement::Continue => { panic!("unsupported statement Continue")},
        Statement::Break => { panic!("unsupported statement Break")},
        Statement::For(_, _, _, _, _) => { panic!("unsupported statement For")},
        Statement::DoWhile(_, _) => { panic!("unsupported statement DoWhile")},
        Statement::IfThen(_, _, _, _) => { panic!("unsupported statement IfThen")},
        Statement::Block(_) => { panic!("unsupported statement Begin…End block")},

        // nop statements
        Statement::Label(_) |
        Statement::Comment(_) => { /* skip */ },
        Statement::End => {
            interpreter.is_running = true;
        }

        _ => { panic!("unsupported statement {:?}", stmt); }
    }
}

fn calc_table(blk : &Block) -> HashMap<String, usize>
{
    let mut res = HashMap::new();

    for i in 0..blk.statements.len() {
        if let Statement::Label(label) = &blk.statements[i] {
            res.insert(label.clone(), i);
        }
    }

    res
}

pub fn run(prg : &Program, ctx: &mut dyn ExecutionContext, io: &mut dyn PCBoardIO)
{
    let cur_frame = StackFrame { 
        values: HashMap::new(),
        cur_ptr:0,
        label_table:0
    };

    let mut interpreter =Interpreter {
        prg,
        ctx,
       // lookup: HashMap::new(),
        label_tables: Vec::new(),
        cur_frame: vec![cur_frame],
        io,
        is_running: true
        //  stack_frames: vec![]
    };

    interpreter.label_tables.push(calc_table(&prg.main_block));
    //nterpreter.lookup.insert(&prg.main_block, 0);

    interpreter.ctx.print("Run program...\n");
    while interpreter.is_running && interpreter.cur_frame.last().unwrap().cur_ptr < prg.main_block.statements.len() {
        let stmt = &prg.main_block.statements[interpreter.cur_frame.last().unwrap().cur_ptr as usize];
        execute_statement(&mut interpreter, stmt);
        interpreter.cur_frame.last_mut().unwrap().cur_ptr += 1;
    }
}
