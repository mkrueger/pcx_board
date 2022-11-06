use std::{net::{TcpListener, TcpStream}, io::{Write, BufReader, BufRead, Read}, collections::VecDeque, time::Duration};

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

    fn gotoxy(&mut self, x: i32, y: i32) {
        println!("gotoxy {}.{}", x,y);
        self.vt.caret.set_position_xy(x, y);
        let mut b = Vec::new();
        b.extend_from_slice(b"\x1B[");
        b.extend_from_slice((1 + y).to_string().as_bytes());
        b.extend_from_slice(b";");
        b.extend_from_slice((1 + x).to_string().as_bytes());
        b.extend_from_slice(b"H");
        self.com.write(&b);
    }

    fn print(&mut self, str: &str)
    {
        let mut v = Vec::new();

        for c in str.chars() {
            self.vt.buffer_parser.print_char(&mut self.vt.buf, &mut self.vt.caret, c as u8);
            self.pcb.print_char(&mut v, &mut self.vt.caret, c as u8);
        }

        self.com.write(&v);
    }

    fn read(&mut self) -> String
    {
        let mut result = String::new();

        loop {
            let ch = self.com.read_char(Duration::from_secs(600)).unwrap();
            if ch == b'\r' || ch == b'\n' {
                break;
            }
            result.push(char::from_u32(ch as u32).unwrap());
        }
        result
    }

    fn get_char(&mut self) -> Option<char>
    {
        if self.com.is_data_available().unwrap() {
            let u = self.com.read_char_nonblocking().unwrap();
            Some(char::from_u32(u as u32).unwrap()) 
        } else {
            None
        }
    }
    fn send_to_com(&mut self, data: &str)
    {
        self.com.push_str(data);
    }

}

fn main() -> std::io::Result<()> {

    let listener = TcpListener::bind("127.0.0.1:4321")?;
    println!("listen...");
    for stream in listener.incoming() {
        println!("incoming connection!");

        let mut connection = Connection::new(stream?);
        
        loop {
            let prg = load_file(&"/home/mkrueger/work/pcx_board/AGSENTR1/AGSENTR.PPE");
            let mut io = MemoryIO::new();
            run(&prg, &mut connection, &mut io);
            while connection.com.is_data_available().unwrap() {
                connection.com.read_char_nonblocking();
            }
        }
    }
    Ok(())
}