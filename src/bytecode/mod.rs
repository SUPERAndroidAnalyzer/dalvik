use std::io::Cursor;
use byteorder::{ByteOrder, ReadBytesExt, LittleEndian, BigEndian};
use std::io::Read;
use error::*;
use std::fmt::Debug;

#[derive(Debug)]
pub enum ByteCode {
    Nop,
    Move(u8, u8),
    MoveFrom16(u8, u16),
    ReturnVoid,
}

impl ToString for ByteCode {
    fn to_string(&self) -> String {
        match *self {
            ByteCode::Nop => format!("nop"),
            ByteCode::ReturnVoid => format!("return-void"),
            ByteCode::Move(dest, source) => {
                format!("move v{}, v{}", dest, source)
            },
            ByteCode::MoveFrom16(dest, source) => {
                format!("move/from16 v{}, v{}", dest, source)
            }
        }
    }
}

pub struct ByteCodeDecoder<R: Read> {
    cursor: R,
}

impl<R: Read> ByteCodeDecoder<R> {
    pub fn new(buffer: R) -> Self {
        ByteCodeDecoder {
            cursor: buffer,
        }
    }

    fn format10x(&mut self) -> Result<()> {
        let current_byte = self.cursor.read_u8()?;

        Ok(())
    }

    fn format12x(&mut self) -> Result<(u8, u8)> {
        let current_byte = self.cursor.read_u8()?;

        let high = (current_byte & 0xF0) >> 4;
        let low = current_byte & 0xF;

        Ok((high, low))
    }

    fn format22x(&mut self) -> Result<(u8, u16)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let source = self.cursor.read_u16::<LittleEndian>()?;

        Ok((dest, source))
    }
}

impl<R: Read> Iterator for ByteCodeDecoder<R> {
    type Item = ByteCode;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.cursor.read_u8();

        match byte {
            Ok(0x00) => {
                self.format10x().ok().map(|_| ByteCode::Nop)
            },
            Ok(0x01) => {
                self.format12x().ok().map(|(d, s)| ByteCode::Move(d, s))
            },
            Ok(0x02) => {
                self.format22x().ok().map(|(d, s)| ByteCode::MoveFrom16(d, s))
            },
            Ok(0x0e) => {
                self.format10x().ok().map(|_| ByteCode::ReturnVoid)
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_decode_noop() {
        let raw_opcode:&[u8] = &[0x00, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        matches!(opcode, ByteCode::Nop);
        assert_eq!("nop", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return_void() {
        let raw_opcode:&[u8] = &[0x0e, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        matches!(opcode, ByteCode::Nop);
        assert_eq!("return-void", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move() {
        let raw_opcode:&[u8] = &[0x01, 0x3B];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        matches!(opcode, ByteCode::Move(d, s) if d == 0xB && s == 0x3);
        assert_eq!("move v3, v11", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_from_16() {
        let raw_opcode:&[u8] = &[0x02, 0xAA, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        matches!(opcode, ByteCode::MoveFrom16(d, s) if d == 0xAA && s == 0x3412);
        assert_eq!("move/from16 v170, v13330", opcode.to_string());
    }
}