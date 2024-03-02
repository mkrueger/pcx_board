use icy_engine::{Caret, TextAttribute};

#[allow(clippy::struct_excessive_bools)]
pub struct PCBoardParser {
    // PCB
    pub pcb_code: bool,
    pub pcb_color: bool,
    pub pcb_value: u8,
    pub pcb_pos: i32,
}

impl PCBoardParser {
    pub fn new() -> Self {
        PCBoardParser {
            pcb_code: false,
            pcb_color: false,
            pcb_value: 0,
            pcb_pos: 0,
        }
    }
}

impl Default for PCBoardParser {
    fn default() -> Self {
        Self::new()
    }
}

const FG_TABLE: [&[u8; 2]; 8] = [b"30", b"34", b"32", b"36", b"31", b"35", b"33", b"37"];
const BG_TABLE: [&[u8; 2]; 8] = [b"40", b"44", b"42", b"46", b"41", b"45", b"43", b"47"];

impl PCBoardParser {
    pub fn print_char(&mut self, buf: &mut Vec<u8>, caret: &mut Caret, ch: u8) {
        if self.pcb_color {
            self.pcb_pos += 1;
            if self.pcb_pos < 3 {
                match self.pcb_pos {
                    1 => {
                        self.pcb_value = conv_ch(ch);
                        return;
                    }
                    2 => {
                        self.pcb_value = (self.pcb_value << 4) + conv_ch(ch);
                        caret.set_attr(TextAttribute::from_u8(
                            self.pcb_value,
                            icy_engine::IceMode::Ice,
                        ));
                        buf.extend_from_slice(b"\x1B[");
                        if caret.get_attribute().is_bold() {
                            buf.extend_from_slice(b"1;");
                        } else {
                            buf.extend_from_slice(b"0;");
                        }
                        buf.extend_from_slice(
                            FG_TABLE[caret.get_attribute().get_foreground() as usize],
                        );
                        buf.extend_from_slice(b";");
                        buf.extend_from_slice(
                            BG_TABLE[caret.get_attribute().get_background() as usize],
                        );
                        buf.extend_from_slice(b"m");
                    }
                    _ => {}
                }
            }
            self.pcb_color = false;
            self.pcb_code = false;
            return;
        }

        if self.pcb_code {
            match ch {
                b'@' => {
                    self.pcb_code = false;
                }
                b'X' => {
                    self.pcb_color = true;
                    self.pcb_pos = 0;
                }
                _ => {}
            }
            return;
        }
        match ch {
            b'@' => {
                self.pcb_code = true;
            }
            _ => buf.push(ch),
        }
    }
}

fn conv_ch(ch: u8) -> u8 {
    if ch.is_ascii_digit() {
        return ch - b'0';
    }
    if (b'a'..=b'f').contains(&ch) {
        return 10 + ch - b'a';
    }
    if (b'A'..=b'F').contains(&ch) {
        return 10 + ch - b'A';
    }
    0
}
