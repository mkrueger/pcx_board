use std::{
    io::{self, ErrorKind, Read, Write},
    net::{SocketAddr, TcpStream},
    thread,
    time::Duration,
};

pub struct RawCom {
    pub tcp_stream: TcpStream,
    pub buf: std::collections::VecDeque<u8>,
}

impl RawCom {
    pub fn push_str(&mut self, data: &str) {
        self.buf.extend(data.as_bytes().iter());
    }

    pub fn connect(addr: &SocketAddr, timeout: Duration) -> io::Result<Self> {
        let tcp_stream = std::net::TcpStream::connect_timeout(addr, timeout)?;
        tcp_stream.set_nonblocking(true)?;

        Ok(Self {
            tcp_stream,
            buf: std::collections::VecDeque::new(),
        })
    }

    pub fn fill_buffer(&mut self) -> io::Result<()> {
        let mut buf = [0; 1024 * 8];
        match self.tcp_stream.read(&mut buf) {
            Ok(size) => {
                self.buf.extend(buf[0..size].iter());
            }
            Err(ref e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(());
                }
                return Err(io::Error::new(
                    ErrorKind::ConnectionAborted,
                    format!("{}", e),
                ));
            }
        };
        Ok(())
    }

    fn fill_buffer_wait(&mut self, _timeout: Duration) -> io::Result<()> {
        self.tcp_stream.set_nonblocking(false)?;
        self.fill_buffer()?;
        while self.buf.is_empty() {
            self.fill_buffer()?;
            thread::sleep(Duration::from_millis(10));
        }
        self.tcp_stream.set_nonblocking(true)?;
        Ok(())
    }

    pub fn read_char(&mut self, timeout: Duration) -> io::Result<u8> {
        if let Some(b) = self.buf.pop_front() {
            return Ok(b);
        }
        self.fill_buffer_wait(timeout)?;
        if let Some(b) = self.buf.pop_front() {
            return Ok(b);
        }
        Err(io::Error::new(ErrorKind::TimedOut, "timed out"))
    }

    pub fn read_char_nonblocking(&mut self) -> io::Result<u8> {
        if let Some(b) = self.buf.pop_front() {
            return Ok(b);
        }
        Err(io::Error::new(ErrorKind::TimedOut, "no data avaliable"))
    }

    pub fn read_exact(&mut self, duration: Duration, bytes: usize) -> io::Result<Vec<u8>> {
        while self.buf.len() < bytes {
            self.fill_buffer_wait(duration)?;
        }
        Ok(self.buf.drain(0..bytes).collect())
    }

    pub fn is_data_available(&mut self) -> io::Result<bool> {
        self.fill_buffer()?;
        Ok(!self.buf.is_empty())
    }

    pub fn disconnect(&mut self) -> io::Result<()> {
        self.tcp_stream.shutdown(std::net::Shutdown::Both)
    }

    pub fn write(&mut self, buf: &[u8]) -> io::Result<()> {
        /*let e: Vec<u8> = buf.iter().map(|c| if *c == 27 { b'x' } else { *c }).collect();
        println!("write_raw: {} {:?}", &String::from_utf8_lossy(e.as_slice()), buf);
        println!("{}", std::backtrace::Backtrace::force_capture());*/
        self.tcp_stream.write_all(buf)
    }
}
