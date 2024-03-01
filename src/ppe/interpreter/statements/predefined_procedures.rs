use std::{fs, thread, time::Duration};

use super::super::errors::Error;
use crate::{evaluate_exp, get_int, get_string, Interpreter, Res};
use ppl_engine::ast::*;

pub fn cls(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    interpreter.ctx.print("\x1B[2J")
}

pub fn clreol(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    interpreter.ctx.print("\x1B[K")
}
pub fn more(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn wait(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn color(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn goto(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn confflag(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn confunflag(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}

pub fn dispfile(interpreter: &mut Interpreter, file: String, flags: i32) -> Res<()> {
    let content = fs::read(&file);
    match content {
        Ok(content) => interpreter.ctx.write_raw(&content),
        Err(err) => interpreter
            .ctx
            .print(format!("{} error {}", file, err).as_str()),
    }
}

pub fn input(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fcreate(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let channel = get_int(&evaluate_exp(interpreter, &params[0])?)? as usize;
    let file = &evaluate_exp(interpreter, &params[1])?.to_string();
    let am = get_int(&evaluate_exp(interpreter, &params[2])?)?;
    let sm = get_int(&evaluate_exp(interpreter, &params[3])?)?;
    interpreter.io.fcreate(channel, file, am, sm);
    Ok(())
}

pub fn fopen(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let channel = get_int(&evaluate_exp(interpreter, &params[0])?)? as usize;
    let file = &evaluate_exp(interpreter, &params[1])?.to_string();
    let am = get_int(&evaluate_exp(interpreter, &params[2])?)?;
    let sm = get_int(&evaluate_exp(interpreter, &params[3])?)?;
    interpreter.io.fopen(channel, file, am, sm);
    Ok(())
}

pub fn fappend(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let channel = get_int(&evaluate_exp(interpreter, &params[0])?)? as usize;
    let file = &evaluate_exp(interpreter, &params[1])?.to_string();
    let am = get_int(&evaluate_exp(interpreter, &params[2])?)?;
    let sm = get_int(&evaluate_exp(interpreter, &params[3])?)?;
    interpreter.io.fappend(channel, file, am, sm);
    Ok(())
}

pub fn fclose(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let channel = get_int(&evaluate_exp(interpreter, &params[0])?)?;
    if channel == -1 {
        // READLINE uses -1 as a special value
        return Ok(());
    }
    if !(0..=7).contains(&channel) {
        return Err(Box::new(Error::FileChannelOutOfBounds(channel)));
    }
    interpreter.io.fclose(channel as usize);
    Ok(())
}

pub fn fget(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let channel = get_int(&evaluate_exp(interpreter, &params[0])?)? as usize;
    let value = VariableValue::String(interpreter.io.fget(channel));
    let var_name = get_var_name(&params[1]);
    let var_type = interpreter.prg.get_var_type(&var_name);
    interpreter
        .cur_frame
        .last_mut()
        .unwrap()
        .values
        .insert(var_name, convert_to(var_type, &value));
    Ok(())
}

pub fn fput(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let channel = get_int(&evaluate_exp(interpreter, &params[0])?)? as usize;

    for expr in &params[1..] {
        let value = evaluate_exp(interpreter, expr)?;
        interpreter.io.fput(channel, value.to_string());
    }
    Ok(())
}

pub fn fputln(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let channel = get_int(&evaluate_exp(interpreter, &params[0])?)? as usize;

    for expr in &params[1..] {
        let value = evaluate_exp(interpreter, expr)?;
        interpreter.io.fput(channel, value.to_string());
    }
    interpreter.io.fput(channel, "\n".to_string());
    Ok(())
}

pub fn resetdisp(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn startdisp(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fputpad(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn hangup(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}

pub fn getuser(interpreter: &mut Interpreter) -> Res<()> {
    interpreter.current_user = Some(interpreter.pcb_data.users[interpreter.cur_user].clone());
    interpreter.cur_frame.last_mut().unwrap().values.insert(
        "U_PAGELEN".to_string(),
        VariableValue::Integer(interpreter.pcb_data.users[interpreter.cur_user].page_len),
    );
    interpreter.cur_frame.last_mut().unwrap().values.insert(
        "U_PWD".to_string(),
        VariableValue::String(
            interpreter.pcb_data.users[interpreter.cur_user]
                .password
                .clone(),
        ),
    );
    interpreter.cur_frame.last_mut().unwrap().values.insert(
        "U_PWDEXP".to_string(),
        VariableValue::Date(0000), // TODO
    );
    interpreter.cur_frame.last_mut().unwrap().values.insert(
        "U_SCROLL".to_string(),
        VariableValue::Boolean(interpreter.pcb_data.users[interpreter.cur_user].scroll_flag),
    );
    interpreter.cur_frame.last_mut().unwrap().values.insert(
        "U_SEC".to_string(),
        VariableValue::Integer(interpreter.pcb_data.users[interpreter.cur_user].security_level),
    );

    Ok(())
}

pub fn putuser(interpreter: &mut Interpreter) -> Res<()> {
    if let Some(user) = interpreter.current_user.take() {
        interpreter.pcb_data.users[interpreter.cur_user] = user;
    }
    Ok(())
}

pub fn defcolor(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn delete(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn deluser(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn adjtime(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn log(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}

pub fn inputstr(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO inputstr")
}

pub fn inputyn(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn inputmoney(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn inputint(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn inputcc(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn inputdate(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn inputtime(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn promptstr(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dtron(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dtroff(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn cdchkon(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn cdchkoff(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}

pub fn delay(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    // 1 tick is ~1/18.2s
    let ticks = get_int(&evaluate_exp(interpreter, &params[0])?)?;
    if ticks > 0 {
        thread::sleep(Duration::from_millis((ticks as f32 * 1000.0 / 18.2) as u64));
    }
    Ok(())
}

pub fn sendmodem(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn inc(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dec(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn newline(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn newlines(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}

pub fn tokenize(interpreter: &mut Interpreter, str: String) -> Res<()> {
    let split = str.split(&[' ', ';'][..]).map(|s| s.to_string());
    interpreter.cur_tokens = split.collect();
    Ok(())
}

pub fn gettoken(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn shell(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn disptext(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn stop(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn inputtext(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn beep(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn push(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn pop(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn kbdstuff(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let value = evaluate_exp(interpreter, &params[0])?;
    interpreter.ctx.print(&get_string(&value))
}
pub fn call(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn join(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn quest(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn blt(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dir(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn kbdfile(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn bye(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn goodbye(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn broadcast(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn waitfor(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn kbdchkon(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn kbdchkoff(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn optext(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dispstr(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let value = evaluate_exp(interpreter, &params[0])?;
    interpreter.ctx.print(&get_string(&value))
}

pub fn rdunet(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let value = evaluate_exp(interpreter, &params[0])?;

    if let VariableValue::Integer(value) = value {
        if let Some(node) = interpreter.pcb_data.nodes.get(value as usize) {
            interpreter.pcb_node = Some(node.clone());
        }
    }
    Ok(())
}

pub fn wrunet(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let node = get_int(&evaluate_exp(interpreter, &params[0])?)?;
    let stat = get_string(&evaluate_exp(interpreter, &params[1])?);
    let name = get_string(&evaluate_exp(interpreter, &params[2])?);
    let city = get_string(&evaluate_exp(interpreter, &params[3])?);
    let operation = get_string(&evaluate_exp(interpreter, &params[4])?);
    let broadcast = get_string(&evaluate_exp(interpreter, &params[5])?);

    // Todo: Broadcast

    if !stat.is_empty() {
        interpreter.pcb_data.nodes[node as usize].status = stat.as_bytes()[0] as char;
    }
    interpreter.pcb_data.nodes[node as usize].name = name;
    interpreter.pcb_data.nodes[node as usize].city = city;
    interpreter.pcb_data.nodes[node as usize].operation = operation;

    Ok(())
}

pub fn dointr(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn varseg(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn varoff(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn pokeb(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn pokew(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn varaddr(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}

pub fn ansipos(interpreter: &mut Interpreter, params: &[Expression]) -> Res<()> {
    let x = get_int(&evaluate_exp(interpreter, &params[0])?)? - 1;
    let y = get_int(&evaluate_exp(interpreter, &params[1])?)? - 1;

    interpreter.ctx.gotoxy(x, y)
}

pub fn backup(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn forward(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn freshline(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn wrusys(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn rdusys(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn newpwd(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn opencap(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn closecap(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn message(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn savescrn(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn restscrn(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn sound(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn chat(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn sprint(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn sprintln(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn mprint(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn mprintln(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn rename(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn frewind(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn pokedw(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dbglevel(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn showon(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn showoff(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn pageon(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn pageoff(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fseek(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fflush(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fread(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fwrite(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdefin(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdefout(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdget(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdput(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdputln(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdputpad(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdread(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdwrite(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn adjbytes(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn kbdstring(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn alias(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn redim(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn append(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn copy(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn kbdflush(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    // TODO?
    Ok(())
}
pub fn mdmflush(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    // TODO?
    Ok(())
}
pub fn keyflush(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    // TODO?
    Ok(())
}
pub fn lastin(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn flag(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn download(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn wrusysdoor(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn getaltuser(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn adjdbytes(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn adjtbytes(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn adjtfiles(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn lang(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn sort(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn mousereg(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn scrfile(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn searchinit(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn searchfind(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn searchstop(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn prfound(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn prfoundln(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tpaget(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tpaput(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tpacget(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tpacput(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tparead(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tpawrite(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tpacread(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn tpacwrite(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn bitset(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn bitclear(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn brag(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn frealtuser(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn setlmr(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn setenv(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fcloseall(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn declare(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn function(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn procedure(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn pcall(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fpclr(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn begin(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fend(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn stackabort(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dcreate(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dopen(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dclose(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dsetalias(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dpack(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dcloseall(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dlock(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dlockr(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dlockg(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dunlock(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dncreate(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dnopen(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dnclose(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dncloseall(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dnew(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dadd(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dappend(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dtop(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dgo(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dbottom(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dskip(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dblank(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn ddelete(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn drecall(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dtag(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dseek(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dfblank(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dget(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dput(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn dfcopy(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}

pub fn eval(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn account(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn recordusage(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn msgtofile(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn qwklimits(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn command(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn uselmrs(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn confinfo(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn adjtubytes(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn grafmode(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn adduser(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn killmsg(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn chdir(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn mkdir(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn redir(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdowraka(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdoaddaka(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdowrorg(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdoaddorg(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdoqmod(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdoqadd(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn fdoqdel(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
pub fn sounddelay(interpreter: &Interpreter, params: &[Expression]) -> Res<()> {
    panic!("TODO")
}
