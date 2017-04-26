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
    Move16(u16, u16),
    MoveWide(u8, u8),
    MoveWideFrom16(u8, u16),
    MoveWide16(u16, u16),
    MoveObject(u8, u8),
    MoveObjectFrom16(u8, u16),
    MoveObject16(u16, u16),
    MoveResult(u8),
    MoveResultWide(u8),
    MoveResultObject(u8),
    MoveException(u8),
    ReturnVoid,
    Return(u8),
    ReturnWide(u8),
    ReturnObject(u8),
    Const4(u8, i8),
}

impl ToString for ByteCode {
    fn to_string(&self) -> String {
        match *self {
            ByteCode::Nop => format!("nop"),
            ByteCode::Move(dest, source) => {
                format!("move v{}, v{}", dest, source)
            },
            ByteCode::MoveFrom16(dest, source) => {
                format!("move/from16 v{}, v{}", dest, source)
            },
            ByteCode::Move16(dest, source) => {
                format!("move/16 v{}, v{}", dest, source)
            },
            ByteCode::MoveWide(dest, source) => {
                format!("move-wide v{}, v{}", dest, source)
            },
            ByteCode::MoveWideFrom16(dest, source) => {
                format!("move-wide/from16 v{}, v{}", dest, source)
            },
            ByteCode::MoveWide16(dest, source) => {
                format!("move-wide/16 v{}, v{}", dest, source)
            },
            ByteCode::MoveObject(dest, source) => {
                format!("move-object v{}, v{}", dest, source)
            },
            ByteCode::MoveObjectFrom16(dest, source) => {
                format!("move-object/from16 v{}, v{}", dest, source)
            },
            ByteCode::MoveObject16(dest, source) => {
                format!("move-object/16 v{}, v{}", dest, source)
            },
            ByteCode::MoveResult(dest) => {
                format!("move-result v{}", dest)
            },
            ByteCode::MoveResultWide(dest) => {
                format!("move-result-wide v{}", dest)
            },
            ByteCode::MoveResultObject(dest) => {
                format!("move-result-object v{}", dest)
            },
            ByteCode::MoveException(dest) => {
                format!("move-exception v{}", dest)
            },
            ByteCode::ReturnVoid => format!("return-void"),
            ByteCode::Return(dest) => {
                format!("return v{}", dest)
            },
            ByteCode::ReturnWide(dest) => {
                format!("return-wide v{}", dest)
            },
            ByteCode::ReturnObject(dest) => {
                format!("return-object v{}", dest)
            },
            ByteCode::Const4(dest, literal) => {
                format!("const/4 v{}, #{}", dest, literal)
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

        // TODO: Invert order on the callees
        Ok((high, low))
    }

    fn format11x(&mut self) -> Result<u8> {
        Ok(self.cursor.read_u8()?)
    }

    fn format11n(&mut self) -> Result<(u8, i8)> {
        let current_byte = self.cursor.read_u8()?;

        let literal = ((current_byte & 0xF0) as i8 >> 4) as i8;
        let register = current_byte & 0xF;

        Ok((register, literal))
    }

    fn format22x(&mut self) -> Result<(u8, u16)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let source = self.cursor.read_u16::<LittleEndian>()?;

        Ok((dest, source))
    }

    fn format32x(&mut self) -> Result<(u16, u16)> {
        // TODO: Make byteorder generic
        let dest = self.cursor.read_u16::<LittleEndian>()?;
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
            Ok(0x03) => {
                self.format32x().ok().map(|(d, s)| ByteCode::Move16(d, s))
            },
            Ok(0x04) => {
                self.format12x().ok().map(|(d, s)| ByteCode::MoveWide(d, s))
            },
            Ok(0x05) => {
                self.format22x().ok().map(|(d, s)| ByteCode::MoveWideFrom16(d, s))
            },
            Ok(0x06) => {
                self.format32x().ok().map(|(d, s)| ByteCode::MoveWide16(d, s))
            },
            Ok(0x07) => {
                self.format12x().ok().map(|(d, s)| ByteCode::MoveObject(d, s))
            },
            Ok(0x08) => {
                self.format22x().ok().map(|(d, s)| ByteCode::MoveObjectFrom16(d, s))
            },
            Ok(0x09) => {
                self.format32x().ok().map(|(d, s)| ByteCode::MoveObject16(d, s))
            },
            Ok(0x0A) => {
                self.format11x().ok().map(|d| ByteCode::MoveResult(d))
            },
            Ok(0x0B) => {
                self.format11x().ok().map(|d| ByteCode::MoveResultWide(d))
            },
            Ok(0x0C) => {
                self.format11x().ok().map(|d| ByteCode::MoveResultObject(d))
            },
            Ok(0x0D) => {
                self.format11x().ok().map(|d| ByteCode::MoveException(d))
            },
            Ok(0x0E) => {
                self.format10x().ok().map(|_| ByteCode::ReturnVoid)
            },
            Ok(0x0F) => {
                self.format11x().ok().map(|d| ByteCode::Return(d))
            },
            Ok(0x10) => {
                self.format11x().ok().map(|d| ByteCode::ReturnWide(d))
            },
            Ok(0x11) => {
                self.format11x().ok().map(|d| ByteCode::ReturnObject(d))
            },
            Ok(0x12) => {
                self.format11n().ok().map(|(reg, lit)| ByteCode::Const4(reg, lit))
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

        assert!(matches!(opcode, ByteCode::Nop));
        assert_eq!("nop", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return_void() {
        let raw_opcode:&[u8] = &[0x0e, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::ReturnVoid));
        assert_eq!("return-void", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move() {
        let raw_opcode:&[u8] = &[0x01, 0x3B];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Move(d, s) if d == 0x3 && s == 0xB));
        assert_eq!("move v3, v11", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_from_16() {
        let raw_opcode:&[u8] = &[0x02, 0xAA, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveFrom16(d, s) if d == 0xAA && s == 0x3412));
        assert_eq!("move/from16 v170, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_16() {
        let raw_opcode:&[u8] = &[0x03, 0xAA, 0x01, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Move16(d, s) if d == 0x01AA && s == 0x3412));
        assert_eq!("move/16 v426, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_wide() {
        let raw_opcode:&[u8] = &[0x04, 0x3B];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveWide(d, s) if d == 0x3 && s == 0xB));
        assert_eq!("move-wide v3, v11", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_wide_from_16() {
        let raw_opcode:&[u8] = &[0x05, 0xAA, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveWideFrom16(d, s) if d == 0xAA && s == 0x3412));
        assert_eq!("move-wide/from16 v170, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_wide_16() {
        let raw_opcode:&[u8] = &[0x06, 0xAA, 0x01, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveWide16(d, s) if d == 0x01AA && s == 0x3412));
        assert_eq!("move-wide/16 v426, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_object() {
        let raw_opcode:&[u8] = &[0x07, 0x3B];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveObject(d, s) if d == 0x3 && s == 0xB));
        assert_eq!("move-object v3, v11", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_object_from_16() {
        let raw_opcode:&[u8] = &[0x08, 0xAA, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveObjectFrom16(d, s) if d == 0xAA && s == 0x3412));
        assert_eq!("move-object/from16 v170, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_object_16() {
        let raw_opcode:&[u8] = &[0x09, 0xAA, 0x01, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveObject16(d, s) if d == 0x01AA && s == 0x3412));
        assert_eq!("move-object/16 v426, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_result() {
        let raw_opcode:&[u8] = &[0x0A, 0x3B];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveResult(d) if d == 0x3B));
        assert_eq!("move-result v59", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_result_wide() {
        let raw_opcode:&[u8] = &[0x0B, 0x12];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveResultWide(d) if d == 0x12));
        assert_eq!("move-result-wide v18", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_result_object() {
        let raw_opcode:&[u8] = &[0x0C, 0xFF];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveResultObject(d) if d == 0xFF));
        assert_eq!("move-result-object v255", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_exception() {
        let raw_opcode:&[u8] = &[0x0D, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveException(d) if d == 0x00));
        assert_eq!("move-exception v0", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return() {
        let raw_opcode:&[u8] = &[0x0F, 0x23];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Return(d) if d == 0x23));
        assert_eq!("return v35", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return_wide() {
        let raw_opcode:&[u8] = &[0x10, 0x23];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::ReturnWide(d) if d == 0x23));
        assert_eq!("return-wide v35", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return_object() {
        let raw_opcode:&[u8] = &[0x11, 0x23];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::ReturnObject(d) if d == 0x23));
        assert_eq!("return-object v35", opcode.to_string());
    }

    #[test]
    fn it_can_decode_const_4_neg() {
        let raw_opcode:&[u8] = &[0x12, 0xF1];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Const4(r, i) if r == 0x1 && i == -1));
        assert_eq!("const/4 v1, #-1", opcode.to_string());
    }

    #[test]
    fn it_can_decode_const_4_pos() {
        let raw_opcode:&[u8] = &[0x12, 0x71];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Const4(r, i) if r == 0x1 && i == 7));
        assert_eq!("const/4 v1, #7", opcode.to_string());
    }
}