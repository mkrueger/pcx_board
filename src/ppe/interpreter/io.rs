use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Result, Seek, SeekFrom, Write},
    os::unix::prelude::MetadataExt,
    time::SystemTime,
};

use substring::Substring;

const O_RD: i32 = 0;
const O_RW: i32 = 2;
const O_WR: i32 = 1;

pub trait PCBoardIO {
    /// Open a file for append access
    /// channel - integer expression with the channel to use for the file
    /// file - file name to open
    /// am - desired access mode for the file
    /// sm - desired share mode for the file
    fn fappend(&mut self, channel: usize, file: &str, am: i32, sm: i32);

    /// Creates a new file
    /// channel - integer expression with the channel to use for the file
    /// file - file name to open
    /// am - desired access mode for the file
    /// sm - desired share mode for the file
    fn fcreate(&mut self, channel: usize, file: &str, am: i32, sm: i32);

    /// Opens a new file
    /// channel - integer expression with the channel to use for the file
    /// file - file name to open
    /// am - desired access mode for the file
    /// sm - desired share mode for the file
    fn fopen(&mut self, channel: usize, file: &str, am: i32, sm: i32);

    /// Dermine if a file error has occured on a channel since last check
    /// channel - integer expression with the channel to use for the file
    /// Returns
    /// True, if an error occured on the specified channel, False otherwise
    fn ferr(&self, channel: usize) -> bool;

    fn fput(&mut self, channel: usize, text: String);

    /// Read a line from an open file
    /// channel - integer expression with the channel to use for the file
    /// # Returns
    /// The line read or "", on error
    ///
    /// # Example
    /// INTEGER i
    /// STRING s
    /// FOPEN 1,"FILE.DAT",ORD,S DW
    /// IF (FERR(1)) THEN
    ///   PRINTLN "Error on opening..."
    ///   END
    /// ENDIF
    ///
    /// FGET 1, s
    /// WHILE (!FERR(1)) DO
    ///   INC i
    ///   PRINTLN "Line ", RIGHT(i, 3), ": ", s
    ///   FGET 1, s
    /// ENDWHILE
    /// FCLOSE 1
    fn fget(&mut self, channel: usize) -> String;

    /// channel - integer expression with the channel to use for the file
    /// #Example
    /// STRING s
    /// FAPPEND `1,"C:\PCB\MAIN\PPE.LOG",O_RW,S_DN`
    /// FPUTLN 1, `U_NAME`()
    /// FREWIND 1
    /// WHILE (!FERR(1)) DO
    /// FGET 1,s
    /// PRINTLN s
    /// ENDWHILE
    /// FCLOSE 1
    fn frewind(&mut self, channel: usize);

    fn fclose(&mut self, channel: usize);

    fn file_exists(&self, file: &str) -> bool;

    fn delete(&mut self, file: &str) -> std::io::Result<()>;
    fn rename(&mut self, old: &str, new: &str) -> std::io::Result<()>;
    fn copy(&mut self, from: &str, to: &str) -> std::io::Result<()>;

    /// .
    ///
    /// # Examples
    ///
    /// ```
    /// // Example template not implemented for trait functions
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn get_file_date(&self, file: &str) -> Result<SystemTime>;
    fn get_file_size(&self, file: &str) -> u64;
}

struct FileChannel {
    file: Option<Box<File>>,
    reader: Option<BufReader<File>>,
    _content: Vec<u8>,
    err: bool,
}

impl FileChannel {
    fn new() -> Self {
        FileChannel {
            file: None,
            reader: None,
            _content: Vec::new(),
            err: false,
        }
    }
}

pub struct DiskIO {
    path: String, // use that as root
    channels: [FileChannel; 8],
}

impl DiskIO {
    #[must_use]
    pub fn new(path: &str) -> Self {
        DiskIO {
            path: path.to_string(),
            channels: [
                FileChannel::new(),
                FileChannel::new(),
                FileChannel::new(),
                FileChannel::new(),
                FileChannel::new(),
                FileChannel::new(),
                FileChannel::new(),
                FileChannel::new(),
            ],
        }
    }

    fn resolve_file_name(&self, file: &str) -> String {
        let s: String = file
            .chars()
            .map(|x| match x {
                '\\' => '/',
                _ => x,
            })
            .collect();

        if file.starts_with("C:\\") {
            let mut result = self.path.to_string();
            result.push('/');
            result.push_str(s.substring(2, file.len() - 2));
            return result;
        }
        s
    }
}

impl PCBoardIO for DiskIO {
    fn fappend(&mut self, channel: usize, file: &str, _am: i32, _sm: i32) {
        let file = OpenOptions::new()
            .append(true)
            .open(self.resolve_file_name(file));
        match file {
            Ok(handle) => {
                self.channels[channel] = FileChannel {
                    file: Some(Box::new(handle)),
                    reader: None,
                    _content: Vec::new(),
                    err: false,
                };
            }
            _ => {
                self.channels[channel] = FileChannel {
                    file: None,
                    reader: None,
                    _content: Vec::new(),
                    err: true,
                };
            }
        }
    }

    fn fcreate(&mut self, channel: usize, file: &str, _am: i32, _sm: i32) {
        let file = File::create(self.resolve_file_name(file));
        match file {
            Ok(handle) => {
                self.channels[channel] = FileChannel {
                    file: Some(Box::new(handle)),
                    reader: None,
                    _content: Vec::new(),
                    err: false,
                };
            }
            _ => {
                self.channels[channel] = FileChannel {
                    file: None,
                    reader: None,
                    _content: Vec::new(),
                    err: true,
                };
            }
        }
    }

    fn delete(&mut self, file: &str) -> std::io::Result<()> {
        let file = self.resolve_file_name(file);
        fs::remove_file(file)
    }

    fn rename(&mut self, old: &str, new: &str) -> std::io::Result<()> {
        let old = self.resolve_file_name(old);
        let new = self.resolve_file_name(new);
        fs::rename(old, new)
    }
    fn copy(&mut self, from: &str, to: &str) -> std::io::Result<()> {
        let old = self.resolve_file_name(from);
        let new = self.resolve_file_name(to);
        fs::copy(old, new)?;
        Ok(())
    }

    fn fopen(&mut self, channel: usize, file: &str, mode: i32, _sm: i32) {
        println!(
            "fopen file: {} -- {} on channel {channel}",
            file,
            self.resolve_file_name(file)
        );

        let file = match mode {
            O_RD => File::open(self.resolve_file_name(file)),
            O_WR => File::create(self.resolve_file_name(file)),
            O_RW => OpenOptions::new()
                .read(true)
                .write(true)
                .open(self.resolve_file_name(file)),
            _ => File::open(self.resolve_file_name(file)),
        };
        match file {
            Ok(handle) => {
                self.channels[channel] = FileChannel {
                    file: Some(Box::new(handle)),
                    reader: None,
                    _content: Vec::new(),
                    err: false,
                };
            }
            Err(err) => {
                self.channels[channel] = FileChannel {
                    file: None,
                    reader: None,
                    _content: Vec::new(),
                    err: true,
                };
            }
        }
    }

    fn ferr(&self, channel: usize) -> bool {
        self.channels[channel].err
    }

    fn fput(&mut self, channel: usize, text: String) {
        match &mut self.channels[channel].file {
            Some(f) => {
                let _ = f.write(text.as_bytes());
                self.channels[channel].err = false;
            }
            _ => {
                log::error!("channel {} not found", channel);
                self.channels[channel].err = true;
            }
        }
    }

    fn fget(&mut self, channel: usize) -> String {
        if let Some(f) = self.channels[channel].file.take() {
            self.channels[channel].reader = Some(BufReader::new(*f));
        }
        if let Some(reader) = &mut self.channels[channel].reader {
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                self.channels[channel].err = true;
                String::new()
            } else {
                self.channels[channel].err = false;
                line.trim_end_matches(|c| c == '\r' || c == '\n')
                    .to_string()
            }
        } else {
            log::error!("no file!");
            self.channels[channel].err = true;
            String::new()
        }
    }

    fn frewind(&mut self, channel: usize) {
        match &mut self.channels[channel].file {
            Some(f) => {
                f.seek(SeekFrom::Start(0)).expect("seek error");
                self.channels[channel].err = false;
            }
            _ => {
                self.channels[channel].err = true;
            }
        }
    }

    fn fclose(&mut self, channel: usize) {
        match &mut self.channels[channel].file {
            Some(_) => {
                self.channels[channel] = FileChannel {
                    file: None,
                    reader: None,
                    _content: Vec::new(),
                    err: false,
                };
            }
            _ => {
                self.channels[channel].err = true;
            }
        }
    }

    fn file_exists(&self, file: &str) -> bool {
        fs::metadata(self.resolve_file_name(file)).is_ok()
    }

    fn get_file_date(&self, file: &str) -> Result<SystemTime> {
        let metadata = fs::metadata(self.resolve_file_name(file))?;
        metadata.accessed()
    }

    fn get_file_size(&self, file: &str) -> u64 {
        let metadata = fs::metadata(self.resolve_file_name(file)).unwrap();
        metadata.size()
    }
}

struct SimulatedFileChannel {
    file: Option<String>,
    filepos: i32,
    err: bool,
}

impl SimulatedFileChannel {
    fn new() -> Self {
        SimulatedFileChannel {
            file: None,
            filepos: 0,
            err: false,
        }
    }
}

pub struct MemoryIO {
    channels: [SimulatedFileChannel; 8],
    pub files: HashMap<String, String>,
}

impl MemoryIO {
    #[must_use]
    pub fn new() -> Self {
        MemoryIO {
            files: HashMap::new(),
            channels: [
                SimulatedFileChannel::new(),
                SimulatedFileChannel::new(),
                SimulatedFileChannel::new(),
                SimulatedFileChannel::new(),
                SimulatedFileChannel::new(),
                SimulatedFileChannel::new(),
                SimulatedFileChannel::new(),
                SimulatedFileChannel::new(),
            ],
        }
    }
}

impl Default for MemoryIO {
    fn default() -> Self {
        Self::new()
    }
}

impl PCBoardIO for MemoryIO {
    fn fappend(&mut self, channel: usize, file: &str, _am: i32, _sm: i32) {
        let f = file.to_string();
        let mut filepos = 0;

        if let Some(v) = self.files.get(&f) {
            filepos = v.len();
        } else {
            self.files.insert(f.clone(), String::new());
        }

        self.channels[channel] = SimulatedFileChannel {
            file: Some(f),
            filepos: filepos as i32,
            err: false,
        };
    }

    fn fcreate(&mut self, channel: usize, file: &str, _am: i32, _sm: i32) {
        let f = file.to_string();
        self.files.insert(f.clone(), String::new());

        self.channels[channel] = SimulatedFileChannel {
            file: Some(f),
            filepos: 0,
            err: false,
        };
    }

    fn fopen(&mut self, channel: usize, file: &str, _am: i32, _sm: i32) {
        let f = file.to_string();

        if self.files.get(&f).is_none() {
            self.channels[channel] = SimulatedFileChannel {
                file: None,
                filepos: 0,
                err: true,
            };
            return;
        }

        self.channels[channel] = SimulatedFileChannel {
            file: Some(f),
            filepos: 0,
            err: false,
        };
    }

    fn delete(&mut self, file: &str) -> std::io::Result<()> {
        let f = file.to_string();
        self.files.remove(&f);
        Ok(())
    }

    fn rename(&mut self, old: &str, new: &str) -> std::io::Result<()> {
        if let Some(removed) = self.files.remove(&old.to_string()) {
            self.files.insert(new.to_string(), removed);
        }
        Ok(())
    }

    fn copy(&mut self, from: &str, to: &str) -> std::io::Result<()> {
        if let Some(removed) = self.files.get(&from.to_string()) {
            self.files.insert(to.to_string(), removed.clone());
        }
        Ok(())
    }

    fn ferr(&self, channel: usize) -> bool {
        self.channels[channel].err
    }

    fn fput(&mut self, channel: usize, text: String) {
        if let Some(v) = &self.channels[channel].file {
            let content = self.files.get_mut(v).unwrap();
            content.insert_str(self.channels[channel].filepos as usize, &text);
            self.channels[channel].filepos += text.len() as i32;
        } else {
            self.channels[channel] = SimulatedFileChannel {
                file: None,
                filepos: 0,
                err: true,
            };
        }
    }

    fn fget(&mut self, channel: usize) -> String {
        if let Some(v) = &mut self.channels[channel].file {
            let f = self.files.get(v).unwrap();
            if self.channels[channel].filepos as usize >= f.len() {
                self.channels[channel].err = true;
                return "".to_string();
            }
            let result: String = f
                .chars()
                .skip(self.channels[channel].filepos as usize)
                .take_while(|c| *c != '\n')
                .collect();
            self.channels[channel].filepos += result.len() as i32 + 1;
            result
        } else {
            self.channels[channel].err = true;
            String::new()
        }
    }

    fn frewind(&mut self, channel: usize) {
        if self.channels[channel].file.is_some() {
            self.channels[channel].filepos = 0;
        } else {
            self.channels[channel] = SimulatedFileChannel {
                file: None,
                filepos: 0,
                err: true,
            };
        }
    }

    fn fclose(&mut self, channel: usize) {
        self.channels[channel] = SimulatedFileChannel {
            file: None,
            filepos: 0,
            err: false,
        };
    }

    fn file_exists(&self, file: &str) -> bool {
        self.files.contains_key(file)
    }

    fn get_file_date(&self, _file: &str) -> Result<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    fn get_file_size(&self, file: &str) -> u64 {
        if let Some(v) = self.files.get(file) {
            v.len() as u64
        } else {
            0
        }
    }
}
