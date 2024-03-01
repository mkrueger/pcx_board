use std::{
    backtrace::Backtrace,
    collections::VecDeque,
    fs::File,
    io::Read,
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
    time::{Duration, SystemTime},
};

mod ppe;
use icy_engine::{ansi::constants::COLOR_OFFSETS, BufferParser};
pub use ppe::*;
mod raw;
pub use raw::*;
mod pcb_parser;
pub use pcb_parser::*;

use crate::data::{IcyBoardData, Node, PcbDataType, UserRecord};
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
        let mut v: Vec<u8> = Vec::new();

        let mut state = 0;
        let mut ch1 = 'A';
        for c in str.chars() {
            match state {
                0 => {
                    if c == '@' {
                        state = 1;
                    } else {
                        v.push(c as u8);
                    }
                }
                1 => {
                    if c == 'X' {
                        state = 2;
                    } else {
                        v.push(b'@');
                        v.push(c as u8);
                        state = 0;
                    }
                }
                2 => {
                    if c.is_ascii_hexdigit() {
                        state = 3;
                    } else {
                        v.push(b'@');
                        v.push(c as u8);
                        ch1 = c;
                        state = 0;
                    }
                }
                3 => {
                    state = 0;
                    if !c.is_ascii_hexdigit() {
                        v.push(b'@');
                        v.push(ch1 as u8);
                        v.push(c as u8);
                    } else {
                        v.extend(b"\x1B[0;");
                        let color =
                            (c.to_digit(16).unwrap() << 4 | ch1.to_digit(16).unwrap()) as u8;

                        let fg_color = COLOR_OFFSETS[color as usize & 0b0111] + 30;
                        let bg_color = COLOR_OFFSETS[(color >> 4) as usize & 0b0111] + 40;

                        if color & 0b1000 != 0 {
                            v.extend(b"1;");
                        }
                        v.extend(fg_color.to_string().as_bytes());
                        v.push(b';');

                        v.extend(bg_color.to_string().as_bytes());

                        v.push(b'm');
                    }
                }
                _ => {
                    state = 0;
                }
            }
        }
        self.com.write(&v)?;
        Ok(())
    }

    fn write_raw(&mut self, data: &[u8]) -> Res<()> {
        let mut v: Vec<u8> = Vec::new();

        let mut state = 0;
        let mut ch1 = b'A';
        for c in data {
            let c = *c;
            match state {
                0 => {
                    if c == b'@' {
                        state = 1;
                    } else {
                        v.push(c);
                    }
                }
                1 => {
                    if c == b'X' {
                        state = 2;
                    } else {
                        v.push(b'@');
                        v.push(c);
                        state = 0;
                    }
                }
                2 => {
                    if c.is_ascii_hexdigit() {
                        state = 3;
                    } else {
                        v.push(b'@');
                        v.push(c);
                        ch1 = c;
                        state = 0;
                    }
                }
                3 => {
                    state = 0;
                    if !c.is_ascii_hexdigit() {
                        v.push(b'@');
                        v.push(ch1);
                        v.push(c);
                    } else {
                        v.extend(b"\x1B[0;");
                        let color = ((c as char).to_digit(16).unwrap() << 4
                            | (ch1 as char).to_digit(16).unwrap())
                            as u8;

                        let fg_color = COLOR_OFFSETS[color as usize & 0b0111] + 30;
                        let bg_color = COLOR_OFFSETS[(color >> 4) as usize & 0b0111] + 40;

                        if color & 0b1000 != 0 {
                            v.extend(b"1;");
                        }
                        v.extend(fg_color.to_string().as_bytes());
                        v.push(b';');

                        v.extend(bg_color.to_string().as_bytes());

                        v.push(b'm');
                    }
                }
                _ => {
                    state = 0;
                }
            }
        }
        self.com.write(&v)?;
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

    let users = UserRecord::read_users(Path::new("/home/mkrueger/work/PCBoard/C/PCB/MAIN/USERS"))?;

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
        let users = users.clone();
        thread::spawn(move || {
            let mut i = 1;
            let mut connection = Connection::new(stream);
            // connection.write_raw(b"\x1BP0pS(E)(C1)P[100,440]V(B),[+100,+0],[+0,-10],[-100,+0],(E)P[500,300],F(C[+100])\x1B\\".to_vec());
            //connection.write_raw(&files_copy[0]).unwrap();

            connection.write_raw(b"Press enter").unwrap();

            let mut st = SystemTime::now();
            let mut pcb_data = IcyBoardData {
                users,
                nodes: vec![Node::default()],
                pcb_data: PcbDataType::default(),
            };
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
