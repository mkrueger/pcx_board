use std::{
    collections::VecDeque,
    fs::File,
    io::Read,
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, SystemTime},
};

mod ppe;
use icy_engine::ansi::constants::COLOR_OFFSETS;
use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Config,
};
pub use ppe::*;
mod raw;
pub use raw::*;
mod pcb_parser;
pub use pcb_parser::*;

use crate::data::{IcyBoardData, Node, PcbDataType};
pub mod data;
pub mod pcb_text;

pub struct Connection {
    com: RawCom,
    vt: VT,
    _pcb: PCBoardParser,
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
            _pcb: PCBoardParser::new(),
        }
    }
}

enum PcbState {
    Default,
    GotAt,
    ReadColor1,
    ReadColor2(u8),
    ReadAtSequence(String),
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
        self.write_raw(str.chars().map(|c| c as u8).collect::<Vec<u8>>().as_slice())
    }

    fn write_raw(&mut self, data: &[u8]) -> Res<()> {
        let mut v = Vec::new();
        let mut state = PcbState::Default;
        for c in data {
            let c = *c;
            if c == 0x1A {
                break;
            }
            match state {
                PcbState::Default => {
                    if c == b'@' {
                        state = PcbState::GotAt;
                    } else {
                        v.push(c);
                    }
                }
                PcbState::GotAt => {
                    if c == b'X' {
                        state = PcbState::ReadColor1;
                    } else {
                        state = PcbState::ReadAtSequence((c as char).to_string());
                    }
                }
                PcbState::ReadAtSequence(s) => {
                    if c == b'@' {
                        state = PcbState::Default;
                        match s.as_str() {
                            "CLS" => {
                                v.extend(b"\x1B[2J");
                            }
                            str => {
                                println!("Unknown pcb sequence: {}", str);
                            }
                        }
                    } else {
                        state = PcbState::ReadAtSequence(s + &(c as char).to_string());
                    }
                }
                PcbState::ReadColor1 => {
                    if c.is_ascii_hexdigit() {
                        state = PcbState::ReadColor2(c);
                    } else {
                        v.push(b'@');
                        v.push(c);
                        state = PcbState::Default;
                    }
                }
                PcbState::ReadColor2(ch1) => {
                    state = PcbState::Default;
                    if !c.is_ascii_hexdigit() {
                        v.push(b'@');
                        v.push(ch1);
                        v.push(c);
                    } else {
                        v.extend(b"\x1B[0;");
                        let color = ((c as char).to_digit(16).unwrap() << 4
                            | (ch1 as char).to_digit(16).unwrap())
                            as u8;

                        let bg_color = COLOR_OFFSETS[color as usize & 0b0111] + 40;
                        let fg_color = COLOR_OFFSETS[(color >> 4) as usize & 0b0111] + 30;

                        if color & 0b1000_0000 != 0 {
                            v.extend(b"1;");
                        }
                        v.extend(fg_color.to_string().as_bytes());
                        v.push(b';');

                        v.extend(bg_color.to_string().as_bytes());

                        v.push(b'm');
                    }
                }
            }
        }

        self.com.write(&v)?;
        Ok(())
    }

    fn set_color(&mut self, color: u8) {
        let mut v = Vec::new();
        v.extend(b"\x1B[0;");
        let fg_color = COLOR_OFFSETS[color as usize & 0b0111] + 30;
        let bg_color = COLOR_OFFSETS[(color >> 4) as usize & 0b0111] + 40;

        if color & 0b1000_0000 != 0 {
            v.extend(b"1;");
        }
        v.extend(fg_color.to_string().as_bytes());
        v.push(b';');

        v.extend(bg_color.to_string().as_bytes());

        v.push(b'm');
        let _ = self.com.write(&v);
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

    fn inbytes(&mut self) -> i32 {
        let _ = self.com.fill_buffer();
        self.com.buf.len() as i32
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
    let level = log::LevelFilter::Info;

    // Build a stderr logger.
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();
    let log_file = "log.txt";

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(log_file)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Info),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config);

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

    let pcb_data = PcbDataType::load(
        "/home/mkrueger/work/PCBoard/C/PCB/PCBOARD.DAT",
        "/home/mkrueger/work/PCBoard/C",
    )?;
    println!("pcb_data: {:?}", pcb_data);
    let users = pcb_data.load_users()?;

    for u in &users {
        println!("{} pw:{}", u.name, u.password);
    }
    for stream in listener.incoming() {
        println!("incoming connection!");
        let stream = stream?;
        let files_copy = files.clone();
        let users = users.clone();
        let pcb_data = pcb_data.clone();
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
                pcb_data,
                pcb_text: Vec::new(),
                yes_char: 'Y',
                no_char: 'N',
            };
            pcb_data.load_data();

            let prg =
                ppl_engine::decompiler::load_file("/home/mkrueger/work/pcx_board/lbmenu/MENU.PPE");

            let mut io = DiskIO::new("/home/mkrueger/work/pcx_board");
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
                }
            }

            loop {
                if st.elapsed().unwrap() > Duration::from_secs(30) {
                    break;
                }

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
                        /*
                        if ch == b'\x1B' {
                            print!("\\x1B");
                        } else {
                            print!(
                                "{} ({}/0x{:02X}),",
                                char::from_u32(ch as u32).unwrap(),
                                ch as u32,
                                ch as u32
                            );
                        }*/
                    }
                    if ch.is_err() {
                        break;
                    }
                }
                if got {
                    println!();
                }
                thread::sleep(Duration::from_millis(20));
            }
        });
    }
    Ok(())
}
