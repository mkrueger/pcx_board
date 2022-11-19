use std::{net::{TcpListener, TcpStream}, collections::VecDeque, time::Duration, thread};

mod ppe;
use icy_engine::BufferParser;
pub use ppe::*;
mod raw;
pub use raw::*;
mod pcb_parser;
pub use pcb_parser::*;

use ppl_engine::decompiler::load_file;

pub struct Connection
{
    com: RawCom,
    vt: VT,
    pcb: PCBoardParser
}

pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self {
            com: RawCom { tcp_stream: stream, buf: VecDeque::new() },
            vt: VT::new(),
            pcb: PCBoardParser::new()
        }
    }
}
impl ExecutionContext for Connection
{
    fn vt(&mut self) -> &mut VT {
        &mut self.vt
    }

    fn gotoxy(&mut self, x: i32, y: i32) -> Res<()> {
        println!("gotoxy {}.{}", x,y);
        self.vt.caret.set_position_xy(x, y);
        let mut b = Vec::new();
        b.extend_from_slice(b"\x1B[");
        b.extend_from_slice((1 + y).to_string().as_bytes());
        b.extend_from_slice(b";");
        b.extend_from_slice((1 + x).to_string().as_bytes());
        b.extend_from_slice(b"H");
        self.com.write(&b)?;
        Ok(())
    }

    fn print(&mut self, str: &str)-> Res<()> {
    
        let mut v = Vec::new();

        for c in str.chars() {
            self.vt.buffer_parser.print_char(&mut self.vt.buf, &mut self.vt.caret, c)?;
            self.pcb.print_char(&mut v, &mut self.vt.caret, c as u8);
        }

        self.com.write(&v)?;
        Ok(())
    }

    fn read(&mut self) -> Res<String>
    {
        let mut result = String::new();

        loop {
            let ch = self.com.read_char(Duration::from_secs(600)).unwrap();
            if ch == b'\r' || ch == b'\n' {
                break;
            }
            result.push(char::from_u32(ch as u32).unwrap());
        }
        Ok(result)
    }

    fn get_char(&mut self) -> Res<Option<char>>
    {
        if self.com.is_data_available().unwrap() {
            let u = self.com.read_char_nonblocking().unwrap();
            Ok(Some(char::from_u32(u as u32).unwrap()))
        } else {
            Ok(None)
        }
    }
    fn send_to_com(&mut self, data: &str)-> Res<()>  {
        self.com.push_str(data);
        Ok(())
    }

}

fn main() -> Res<()> {

    let listener = TcpListener::bind("127.0.0.1:4321")?;
    println!("listen...");
    for stream in listener.incoming() {
        println!("incoming connection!");
        let stream = stream?;
        thread::spawn(move ||  {
            let mut connection = Connection::new(stream);
            loop {
                let prg = load_file(&"/home/mkrueger/work/pcx_board/AGSENTR1/AGSENTR.PPE");
                let mut io = MemoryIO::new();
                match run(&prg, &mut connection, &mut io) {
                    Ok(_) => {
                        while connection.com.is_data_available().unwrap() {
                            if connection.com.read_char_nonblocking().is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}",e);
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}