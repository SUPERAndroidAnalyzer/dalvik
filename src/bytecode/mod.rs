use std::io::Cursor;
use byteorder::{ByteOrder, ReadBytesExt, LittleEndian, BigEndian};
use std::io::Read;
use error::*;
use std::fmt::Debug;

#[derive(Debug)]
pub enum OpCode {
    Nop,
    ReturnVoid,

    Unknown,
}

impl From<u8> for OpCode {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x00 => OpCode::Nop,
            0x0e => OpCode::ReturnVoid,
            _ => OpCode::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct ByteCode {
    opcode: OpCode,
    format: Box<Format>,
}

trait Format: Debug {}

// 10x
#[derive(Debug)]
struct Format10x;
impl Format for Format10x {}

pub struct ByteCodeDecoder<R: Read> {
    cursor: R,
}

impl<R: Read> ByteCodeDecoder<R> {
    pub fn new(buffer: R) -> Self {
        ByteCodeDecoder {
            cursor: buffer,
        }
    }

    fn decode_instruction(&mut self, opcode: u8) -> Result<ByteCode> {
        match opcode {
            a @ 0x00 | a @ 0x0e => {
                let oc = OpCode::from(a);

                Ok(ByteCode {
                    opcode: oc,
                    format: Box::new(Format10x),
                })
            },
            _ => Err("Opcode not registered".into()),
        }
    }
}

impl<R: Read> Iterator for ByteCodeDecoder<R> {
    type Item = ByteCode;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.cursor.read_u8();

        match byte {
            Ok(b) => {
                self.decode_instruction(b).ok()
            }
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

        let opcode = d.nth(0);
        println!("{:?}", opcode);

        panic!("AAA");
    }
}