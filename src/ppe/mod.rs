use icy_engine::{Buffer, Caret, PCBoardParser, AvatarParser};

mod interpreter;
pub use interpreter::*;

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