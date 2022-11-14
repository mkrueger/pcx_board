use std::{fmt::{Display}, error::Error};

use icy_engine::{Buffer, Caret, AvatarParser};

mod interpreter;
pub use interpreter::*;
use ppl_engine::tables::OpCode;

#[derive(Debug, Clone, Copy)]
pub enum InterpreterError {
    UnsupportedConst(&'static str),
    UnsupportedOpCode(OpCode),
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::UnsupportedConst(c) => write!(f, "unsupported const {:?}", c),
            InterpreterError::UnsupportedOpCode(c) => write!(f, "unsupported op code {:?}", c),
        }
    }
}

impl Error for InterpreterError {
    fn description(&self) -> &str {
        "use std::display"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

pub struct VT {
    pub buf: Buffer,
    pub buffer_parser: AvatarParser,
    pub caret: Caret,
}

impl VT {
    pub fn new() -> Self {
        let mut buf = Buffer::create(80, 25);
        buf.layers[0].is_transparent = false;
        buf.is_terminal_buffer = true;
        
        Self {
            buf,
            buffer_parser: AvatarParser::new(true),
            caret: Caret::new()
        }
    }
}