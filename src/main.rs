use std::{
    collections::VecDeque,
    fs::File,
    io::Read,
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, SystemTime},
};

mod ppe;
use icy_engine::BufferParser;
pub use ppe::*;
mod raw;
pub use raw::*;
mod pcb_parser;
pub use pcb_parser::*;

use crate::data::PcbDataType;
pub mod data;

pub struct Connection {
    com: RawCom,
    vt: VT,
    pcb: PCBoardParser,
}

pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self {
            com: RawCom {
                tcp_stream: stream,
                buf: VecDeque::new(),
            },
            vt: VT::new(),
            pcb: PCBoardParser::new(),
        }
    }
}
impl ExecutionContext for Connection {
    fn vt(&mut self) -> &mut VT {
        &mut self.vt
    }

    fn gotoxy(&mut self, x: i32, y: i32) -> Res<()> {
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

    fn print(&mut self, str: &str) -> Res<()> {
        let mut v = Vec::new();

        for c in str.chars() {
            self.vt
                .buffer_parser
                .print_char(&mut self.vt.buf, 0, &mut self.vt.caret, c);
            self.pcb.print_char(&mut v, &mut self.vt.caret, c as u8);
        }

        self.com.write(&v)?;
        Ok(())
    }

    fn write_raw(&mut self, data: &[u8]) -> Res<()> {
        self.com.write(data)?;
        Ok(())
    }

    fn read(&mut self) -> Res<String> {
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

    fn get_char(&mut self) -> Res<Option<char>> {
        if self.com.is_data_available().unwrap() {
            let u = self.com.read_char_nonblocking().unwrap();
            Ok(Some(char::from_u32(u as u32).unwrap()))
        } else {
            Ok(None)
        }
    }
    fn send_to_com(&mut self, data: &str) -> Res<()> {
        self.com.push_str(data);
        Ok(())
    }
}

fn main() -> Res<()> {
    let listener = TcpListener::bind("127.0.0.1:4321")?;
    println!("listen...");

    let names = [
        "dragon.ans",
        "CUBES.IG",
        "guardian2.ans",
        "test.ans",
        "TG-HARL.ANS",
        "xibalba.ans",
        "20beersmenu1.ans",
        "cards.ans",
        "sixel.ans",
        "resize_terminal.ans",
        "size_back.ans",
        "music.ans",
    ];

    let mut files = Vec::new();
    for name in names {
        let mut fs = File::open(format!("./manual_tests/{}", name)).unwrap();
        let mut data = Vec::new();
        fs.read_to_end(&mut data).unwrap();
        files.push(data);
    }

    for stream in listener.incoming() {
        println!("incoming connection!");
        let stream = stream?;
        let files_copy = files.clone();
        thread::spawn(move || {
            let mut i = 1;
            let mut connection = Connection::new(stream);
            // connection.write_raw(b"\x1BP0pS(E)(C1)P[100,440]V(B),[+100,+0],[+0,-10],[-100,+0],(E)P[500,300],F(C[+100])\x1B\\".to_vec());
            //connection.write_raw(&files_copy[0]).unwrap();

            connection.write_raw(b"Press enter").unwrap();

            let mut st = SystemTime::now();
            let mut pcb_data = PcbDataType::default();
            loop {
                if st.elapsed().unwrap() > Duration::from_secs(30) {
                    break;
                }

                let prg = ppl_engine::decompiler::load_file(
                    &"/home/mkrueger/work/pcx_board/AGSLOG23/AGSLOG.PPE",
                );
                let mut got = false;
                while connection.com.is_data_available().unwrap() {
                    let ch = connection.com.read_char_nonblocking();
                    if let Ok(ch) = ch {
                        connection.write_raw(&[ch]).unwrap();
                        st = SystemTime::now();
                        got = true;
                        if ch == b'\r' {
                            connection.write_raw(&files_copy[i]).unwrap();
                            i = (i + 1) % files_copy.len();
                        }
                        if ch == b'\x1B' {
                            print!("\\x1B");
                        } else {
                            print!(
                                "{} ({}/0x{:02X}),",
                                char::from_u32(ch as u32).unwrap(),
                                ch as u32,
                                ch as u32
                            );
                        }
                    }
                    if ch.is_err() {
                        break;
                    }
                }
                if got {
                    println!();
                }
                thread::sleep(Duration::from_millis(20));

                let mut io = MemoryIO::new();
                println!("run  {:?}", prg);
                match run(&prg, &mut connection, &mut io, &pcb_data) {
                    Ok(_) => {
                        while connection.com.is_data_available().unwrap() {
                            let ch = connection.com.read_char_nonblocking();
                            if let Ok(ch) = ch {
                                println!("{}", char::from_u32(ch as u32).unwrap());
                            }
                            if ch.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}
