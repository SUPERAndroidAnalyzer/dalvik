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
    Const4(u8, i32),
    Const16(u8, i32),
    Const(u8, i32),
    ConstHigh16(u8, i32),
    ConstWide16(u8, i64),
    ConstWide32(u8, i64),
    ConstWide(u8, i64),
    ConstWideHigh16(u8, i64),
    ConstString(u8, StringReference),
    ConstStringJumbo(u8, StringReference),
    ConstClass(u8, ClassReference),
    MonitorEnter(u8),
    MonitorExit(u8),
    CheckCast(u8, TypeReference),
    InstanceOf(u8, u8, TypeReference),
    ArrayLength(u8, u8),
    NewInstance(u8, TypeReference),
    NewArray(u8, u8, TypeReference),
    FilledNewArray(Vec<u8>, TypeReference),
    FilledNewArrayRange(u16, u8, TypeReference),
    FillArrayData(u8, i32),
    Throw(u8),
    Goto(i8),
    Goto16(i16),
    Goto32(i32),
    PackedSwitch(u8, i32),
    SparseSwitch(u8, i32),
    Compare(CompareType, u8, u8, u8),
    If(TestType, u8, u8, i16),
    If0(TestType, u8, i16),
    Array(ArrayOperation, u8, u8, u8),
    Instance(ArrayOperation, u8, u8, FieldReference),
    Static(ArrayOperation, u8, FieldReference),
}

#[derive(Debug)]
pub enum CompareType {
    LittleThanFloat,
    GreaterThanFloat,
    LittleThanDouble,
    GreaterThanDouble,
    Long,
    Unknown,
}

impl From<u8> for CompareType {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x2D => CompareType::LittleThanFloat,
            0x2E => CompareType::GreaterThanFloat,
            0x2F => CompareType::LittleThanDouble,
            0x30 => CompareType::GreaterThanDouble,
            0x31 => CompareType::Long,
            _ => CompareType::Unknown,
        }
    }
}

impl ToString for CompareType {
    fn to_string(&self) -> String {
        match *self {
            CompareType::LittleThanFloat => "cmpl-float".to_string(),
            CompareType::GreaterThanFloat => "cmpg-float".to_string(),
            CompareType::LittleThanDouble => "cmpl-double".to_string(),
            CompareType::GreaterThanDouble => "cmpg-double".to_string(),
            CompareType::Long => "cmp-long".to_string(),
            CompareType::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum TestType {
    Equal,
    NonEqual,
    LittleThan,
    GreaterThanOrEqual,
    GreaterThan,
    LittleThanOrEqual,
    Unknown,
}

impl From<u8> for TestType {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x32 | 0x38 => TestType::Equal,
            0x33 | 0x39 => TestType::NonEqual,
            0x34 | 0x3A => TestType::LittleThan,
            0x35 | 0x3B => TestType::GreaterThanOrEqual,
            0x36 | 0x3C => TestType::GreaterThan,
            0x37 | 0x3D => TestType::LittleThanOrEqual,
            _ => TestType::Unknown,
        }
    }
}

impl ToString for TestType {
    fn to_string(&self) -> String {
        match *self {
            TestType::Equal => "if-eq".to_string(),
            TestType::NonEqual => "if-ne".to_string(),
            TestType::LittleThan => "if-lt".to_string(),
            TestType::GreaterThanOrEqual => "if-ge".to_string(),
            TestType::GreaterThan => "if-gt".to_string(),
            TestType::LittleThanOrEqual => "if-le".to_string(),
            TestType::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ArrayOperation {
    Get,
    GetWide,
    GetObject,
    GetBoolean,
    GetByte,
    GetChar,
    GetShort,
    Put,
    PutWide,
    PutObject,
    PutBoolean,
    PutByte,
    PutChar,
    PutShort,
    Unknown,
}

impl From<u8> for ArrayOperation {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x44 | 0x52 | 0x60 => ArrayOperation::Get,
            0x45 | 0x53 | 0x61 => ArrayOperation::GetWide,
            0x46 | 0x54 | 0x62 => ArrayOperation::GetObject,
            0x47 | 0x55 | 0x63 => ArrayOperation::GetBoolean,
            0x48 | 0x56 | 0x64 => ArrayOperation::GetByte,
            0x49 | 0x57 | 0x65 => ArrayOperation::GetChar,
            0x4A | 0x58 | 0x66 => ArrayOperation::GetShort,
            0x4B | 0x59 | 0x67 => ArrayOperation::Put,
            0x4C | 0x5A | 0x68 => ArrayOperation::PutWide,
            0x4D | 0x5B | 0x69 => ArrayOperation::PutObject,
            0x4E | 0x5C | 0x6A => ArrayOperation::PutBoolean,
            0x4F | 0x5D | 0x6B => ArrayOperation::PutByte,
            0x50 | 0x5E | 0x6C => ArrayOperation::PutChar,
            0x51 | 0x5F | 0x6D => ArrayOperation::PutShort,
            _ => ArrayOperation::Unknown,
        }
    }
}

impl ToString for ArrayOperation {
    fn to_string(&self) -> String {
        match *self {
            ArrayOperation::Get => "get".to_string(),
            ArrayOperation::GetWide => "get-wide".to_string(),
            ArrayOperation::GetObject => "get-object".to_string(),
            ArrayOperation::GetBoolean => "get-boolean".to_string(),
            ArrayOperation::GetByte => "get-byte".to_string(),
            ArrayOperation::GetChar => "get-char".to_string(),
            ArrayOperation::GetShort => "get-short".to_string(),
            ArrayOperation::Put => "put".to_string(),
            ArrayOperation::PutWide => "put-wide".to_string(),
            ArrayOperation::PutObject => "put-object".to_string(),
            ArrayOperation::PutBoolean => "put-boolean".to_string(),
            ArrayOperation::PutByte => "put-byte".to_string(),
            ArrayOperation::PutChar => "put-char".to_string(),
            ArrayOperation::PutShort => "put-short".to_string(),
            ArrayOperation::Unknown => "unknown".to_string(),
        }
    }
}

pub type StringReference = u32;
pub type ClassReference = u32;
pub type TypeReference = u32;
pub type FieldReference = u32;

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
            },
            ByteCode::Const16(dest, literal) => {
                format!("const/16 v{}, #{}", dest, literal)
            },
            ByteCode::Const(dest, literal) => {
                format!("const v{}, #{}", dest, literal)
            },
            ByteCode::ConstHigh16(dest, literal) => {
                format!("const/high16 v{}, #{}", dest, literal)
            },
            ByteCode::ConstWide16(dest, literal) => {
                format!("const-wide/16 v{}, #{}", dest, literal)
            },
            ByteCode::ConstWide32(dest, literal) => {
                format!("const-wide/32 v{}, #{}", dest, literal)
            },
            ByteCode::ConstWide(dest, literal) => {
                format!("const-wide v{}, #{}", dest, literal)
            },
            ByteCode::ConstWideHigh16(dest, literal) => {
                format!("const-wide/high16 v{}, #{}", dest, literal)
            },
            ByteCode::ConstString(dest, reference) => {
                format!("const-string v{}, string@{}", dest, reference)
            },
            ByteCode::ConstStringJumbo(dest, reference) => {
                format!("const-string/jumbo v{}, string@{}", dest, reference)
            },
            ByteCode::ConstClass(dest, reference) => {
                format!("const-class v{}, class@{}", dest, reference)
            },
            ByteCode::MonitorEnter(reg) => {
                format!("monitor-enter v{}", reg)
            },
            ByteCode::MonitorExit(reg) => {
                format!("monitor-exit v{}", reg)
            },
            ByteCode::CheckCast(reg, reference) => {
                format!("check-cast v{}, type@{}", reg, reference)
            },
            ByteCode::InstanceOf(dest, src, reference) => {
                format!("instance-of v{}, v{}, type@{}", dest, src, reference)
            },
            ByteCode::ArrayLength(dest, src) => {
                format!("array-length v{}, v{}", dest, src)
            },
            ByteCode::NewInstance(dest, reference) => {
                format!("new-instance v{}, type@{}", dest, reference)
            },
            ByteCode::NewArray(dest, src, reference) => {
                format!("new-array v{}, v{}, type@{}", dest, src, reference)
            },
            ByteCode::FilledNewArray(ref registers, reference) => {
                let str_register: Vec<String> = registers.iter().map(|r| format!("v{}", r)).collect();
                format!("filled-new-array {{{}}}, type@{}", str_register.join(", "), reference)
            },
            ByteCode::FilledNewArrayRange(first_reg, amount, reference) => {
                let str_register: Vec<String> = (first_reg..(amount + 1) as u16).map(|r| format!("v{}", r)).collect();
                format!("filled-new-array/range {{{}}}, type@{}", str_register.join(", "), reference)
            },
            ByteCode::FillArrayData(reg, offset) => {
                format!("fill-array-data v{}, {}", reg, offset)
            },
            ByteCode::Throw(reg) => {
                format!("throw v{}", reg)
            },
            ByteCode::Goto(offset) => {
                format!("goto {}", offset)
            },
            ByteCode::Goto16(offset) => {
                format!("goto/16 {}", offset)
            },
            ByteCode::Goto32(offset) => {
                format!("goto/32 {}", offset)
            },
            ByteCode::PackedSwitch(reg, offset) => {
                format!("packed-switch v{}, {}", reg, offset)
            },
            ByteCode::SparseSwitch(reg, offset) => {
                format!("sparse-switch v{}, {}", reg, offset)
            },
            ByteCode::Compare(ref ct, dest, op1, op2) => {
                format!("{} v{}, v{}, v{}", ct.to_string(), dest, op1, op2)
            },
            ByteCode::If(ref tt, dest, src, offset) => {
                format!("{} v{}, v{}, {}", tt.to_string(), dest, src, offset)
            },
            ByteCode::If0(ref tt, dest, offset) => {
                format!("{}z v{}, {}", tt.to_string(), dest, offset)
            },
            ByteCode::Array(ref ao, dest, op1, op2) => {
                format!("a{} v{}, v{}, v{}", ao.to_string(), dest, op1, op2)
            },
            ByteCode::Instance(ref ao, dest, op1, field) => {
                format!("i{} v{}, v{}, field@{}", ao.to_string(), dest, op1, field)
            },
            ByteCode::Static(ref ao, dest, field) => {
                format!("s{} v{}, field@{}", ao.to_string(), dest, field)
            },
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

    fn format10t(&mut self) -> Result<i8> {
        Ok(self.cursor.read_i8()?)
    }

    fn format11x(&mut self) -> Result<u8> {
        Ok(self.cursor.read_u8()?)
    }

    fn format11n(&mut self) -> Result<(u8, i32)> {
        let current_byte = self.cursor.read_u8()?;

        let literal = ((current_byte & 0xF0) as i8 >> 4) as i32;
        let register = current_byte & 0xF;

        Ok((register, literal))
    }

    fn format12x(&mut self) -> Result<(u8, u8)> {
        let current_byte = self.cursor.read_u8()?;

        let source = (current_byte & 0xF0) >> 4;
        let dest = current_byte & 0xF;

        Ok((dest, source))
    }

    fn format20t(&mut self) -> Result<i16> {
        let _ = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let literal = self.cursor.read_i16::<LittleEndian>()?;

        Ok(literal)
    }

    fn format21t(&mut self) -> Result<(u8, i16)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let offset = self.cursor.read_i16::<LittleEndian>()?;

        Ok((dest, offset))
    }

    fn format21s(&mut self) -> Result<(u8, i32)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let literal = self.cursor.read_i16::<LittleEndian>()?;

        Ok((dest, literal as i32))
    }

    fn format21hw(&mut self) -> Result<(u8, i32)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let literal = (self.cursor.read_i16::<LittleEndian>()? as i32) << 16;

        Ok((dest, literal))
    }

    fn format21hd(&mut self) -> Result<(u8, i64)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let literal = (self.cursor.read_i16::<LittleEndian>()? as i64) << 48;

        Ok((dest, literal))
    }

    fn format21c(&mut self) -> Result<(u8, u16)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let literal = self.cursor.read_u16::<LittleEndian>()?;

        Ok((dest, literal))
    }

    fn format22c(&mut self) -> Result<(u8, u8, u16)> {
        let current_byte = self.cursor.read_u8()?;

        let source = ((current_byte & 0xF0) as u8 >> 4) as u8;
        let dest = current_byte & 0xF;

        // TODO: Make byteorder generic
        let reference = self.cursor.read_u16::<LittleEndian>()?;

        Ok((dest, source, reference))
    }

    fn format22x(&mut self) -> Result<(u8, u16)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let source = self.cursor.read_u16::<LittleEndian>()?;

        Ok((dest, source))
    }

    fn format22t(&mut self) -> Result<(u8, u8, i16)> {
        let current_byte = self.cursor.read_u8()?;

        let source = ((current_byte & 0xF0) as u8 >> 4) as u8;
        let dest = current_byte & 0xF;
        // TODO: Make byteorder generic
        let offset = self.cursor.read_i16::<LittleEndian>()?;

        Ok((dest, source, offset))
    }

    fn format23x(&mut self) -> Result<(u8, u8, u8)> {
        let dest = self.cursor.read_u8()?;
        // TODO: Make byteorder generic
        let operand1 = self.cursor.read_u8()?;
        let operand2 = self.cursor.read_u8()?;

        Ok((dest, operand1, operand2))
    }

    fn format30t(&mut self) -> Result<i32> {
        let _ = self.cursor.read_u8()?;
        let literal = self.cursor.read_i32::<LittleEndian>()?;

        Ok(literal)
    }

    fn format31i(&mut self) -> Result<(u8, i32)> {
        let dest = self.cursor.read_u8()?;
        let literal = self.cursor.read_i32::<LittleEndian>()?;

        Ok((dest, literal))
    }

    fn format31t(&mut self) -> Result<(u8, i32)> {
        let dest = self.cursor.read_u8()?;
        let literal = self.cursor.read_i32::<LittleEndian>()?;

        Ok((dest, literal))
    }

    fn format31c(&mut self) -> Result<(u8, u32)> {
        let dest = self.cursor.read_u8()?;
        let reference = self.cursor.read_u32::<LittleEndian>()?;

        Ok((dest, reference))
    }

    fn format32x(&mut self) -> Result<(u16, u16)> {
        // TODO: Make byteorder generic
        let dest = self.cursor.read_u16::<LittleEndian>()?;
        let source = self.cursor.read_u16::<LittleEndian>()?;

        Ok((dest, source))
    }

    fn format35c(&mut self) -> Result<(Vec<u8>, u16)> {
        let mut arguments = Vec::new();
        let first_byte = self.cursor.read_u8()?;

        let count = (first_byte & 0xF0) >> 4;
        let last_register = first_byte & 0xF;
        arguments.push(last_register);

        let reference = self.cursor.read_u16::<LittleEndian>()?;

        let reg_array = self.read_4bit_array(4)?;
        arguments.extend(reg_array);

        let final_arguments = arguments.into_iter().rev().take(count as usize).collect();

        Ok((final_arguments, reference))
    }

    fn format3rc(&mut self) -> Result<(u16, u8, u16)> {
        let amount = self.cursor.read_u8()?;
        let reference = self.cursor.read_u16::<LittleEndian>()?;
        let first = self.cursor.read_u16::<LittleEndian>()?;

        Ok((first, amount - 1, reference))
    }

    fn format51l(&mut self) -> Result<(u8, i64)> {
        // TODO: Make byteorder generic
        let dest = self.cursor.read_u8()?;
        let source = self.cursor.read_i64::<LittleEndian>()?;

        Ok((dest, source))
    }

    fn read_4bit_array(&mut self, amount: u8) -> Result<Vec<u8>> {
        let mut values = Vec::new();

        for _ in 0..(amount/2) {
            let current = self.cursor.read_u8()?;

            let high = (current & 0xF0) >> 4;
            let low = current & 0xF;

            values.push(high);
            values.push(low);
        }

        Ok(values)
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
            Ok(0x13) => {
                self.format21s().ok().map(|(reg, lit)| ByteCode::Const16(reg, lit))
            },
            Ok(0x14) => {
                self.format31i().ok().map(|(reg, lit)| ByteCode::Const(reg, lit))
            },
            Ok(0x15) => {
                self.format21hw().ok().map(|(reg, lit)| ByteCode::ConstHigh16(reg, lit))
            },
            Ok(0x16) => {
                self.format21s().ok().map(|(reg, lit)| ByteCode::ConstWide16(reg, lit as i64))
            },
            Ok(0x17) => {
                self.format31i().ok().map(|(reg, lit)| ByteCode::ConstWide32(reg, lit as i64))
            },
            Ok(0x18) => {
                self.format51l().ok().map(|(reg, lit)| ByteCode::ConstWide(reg, lit))
            },
            Ok(0x19) => {
                self.format21hd().ok().map(|(reg, lit)| ByteCode::ConstWideHigh16(reg, lit))
            },
            Ok(0x1A) => {
                self.format21c().ok().map(|(reg, reference)| ByteCode::ConstString(reg, reference as StringReference))
            },
            Ok(0x1B) => {
                self.format31c().ok().map(|(reg, reference)| ByteCode::ConstStringJumbo(reg, reference as StringReference))
            },
            Ok(0x1C) => {
                self.format21c().ok().map(|(reg, reference)| ByteCode::ConstClass(reg, reference as ClassReference))
            },
            Ok(0x1D) => {
                self.format11x().ok().map(|reg| ByteCode::MonitorEnter(reg))
            },
            Ok(0x1E) => {
                self.format11x().ok().map(|reg| ByteCode::MonitorExit(reg))
            },
            Ok(0x1F) => {
                self.format21c().ok().map(|(reg, reference)| ByteCode::CheckCast(reg, reference as TypeReference))
            },
            Ok(0x20) => {
                self.format22c().ok().map(|(dest, src, reference)| ByteCode::InstanceOf(dest, src, reference as TypeReference))
            },
            Ok(0x21) => {
                self.format12x().ok().map(|(dest, src)| ByteCode::ArrayLength(dest, src))
            },
            Ok(0x22) => {
                self.format21c().ok().map(|(dest, reference)| ByteCode::NewInstance(dest, reference as TypeReference))
            },
            Ok(0x23) => {
                self.format22c().ok().map(|(dest, size, reference)| ByteCode::NewArray(dest, size, reference as TypeReference))
            },
            Ok(0x24) => {
                self.format35c()
                    .ok()
                    .map(|(registers, reference)| {
                        ByteCode::FilledNewArray(registers, reference as TypeReference)
                    })
            },
            Ok(0x25) => {
                self.format3rc()
                    .ok()
                    .map(|(first, amount, reference)| {
                        ByteCode::FilledNewArrayRange(first, amount, reference as TypeReference)
                    })
            },
            Ok(0x26) => {
                self.format31t().ok().map(|(reg, offset)| ByteCode::FillArrayData(reg, offset))
            },
            Ok(0x27) => {
                self.format11x().ok().map(|reg| ByteCode::Throw(reg))
            },
            Ok(0x28) => {
                self.format10t().ok().map(|offset| ByteCode::Goto(offset))
            },
            Ok(0x29) => {
                self.format20t().ok().map(|offset| ByteCode::Goto16(offset))
            },
            Ok(0x2A) => {
                self.format30t().ok().map(|offset| ByteCode::Goto32(offset))
            },
            Ok(0x2B) => {
                self.format31t().ok().map(|(reg, offset)| ByteCode::PackedSwitch(reg, offset))
            },
            Ok(0x2C) => {
                self.format31t().ok().map(|(reg, offset)| ByteCode::SparseSwitch(reg, offset))
            },
            Ok(a @ 0x2D ... 0x31) => {
                self.format23x().ok().map(|(dest, op1, op2)| ByteCode::Compare(CompareType::from(a), dest, op1, op2))
            },
            Ok(a @ 0x32 ... 0x37) => {
                self.format22t().ok().map(|(dest, src, offset)| ByteCode::If(TestType::from(a), dest, src, offset))
            },
            Ok(a @ 0x38 ... 0x3D) => {
                self.format21t().ok().map(|(dest, offset)| ByteCode::If0(TestType::from(a), dest, offset))
            },
            Ok(a @ 0x44 ... 0x51) => {
                self.format23x().ok().map(|(dest, op1, op2)| ByteCode::Array(ArrayOperation::from(a), dest, op1, op2))
            },
            Ok(a @ 0x52 ... 0x5f) => {
                self.format22c().ok().map(|(dest, op1, reference)| ByteCode::Instance(ArrayOperation::from(a), dest, op1, reference as FieldReference))
            },
            Ok(a @ 0x60 ... 0x6d) => {
                self.format21c().ok().map(|(dest, reference)| ByteCode::Static(ArrayOperation::from(a), dest, reference as FieldReference))
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

        assert!(matches!(opcode, ByteCode::Move(d, s) if d == 0xB && s == 0x3));
        assert_eq!("move v11, v3", opcode.to_string());
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

        assert!(matches!(opcode, ByteCode::MoveWide(d, s) if d == 0xB && s == 0x3));
        assert_eq!("move-wide v11, v3", opcode.to_string());
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

        assert!(matches!(opcode, ByteCode::MoveObject(d, s) if d == 0xB && s == 0x3));
        assert_eq!("move-object v11, v3", opcode.to_string());
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

    #[test]
    fn it_can_decode_const_16_neg() {
        let raw_opcode:&[u8] = &[0x13, 0xF1, 0xFA, 0xFB];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const/16 v241, #-1030", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Const16(r, i) if r == 0xF1 && i == -1030));
    }

    #[test]
    fn it_can_decode_const() {
        let raw_opcode:&[u8] = &[0x14, 0x44, 0xFA, 0xFB, 0x00, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const v68, #64506", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Const(r, i) if r == 0x44 && i == 64506));
    }

    #[test]
    fn it_can_decode_const_high_16() {
        let raw_opcode:&[u8] = &[0x15, 0x44, 0xFF, 0xFF];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const/high16 v68, #-65536", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstHigh16(r, i) if r == 0x44 && i == -65536));
    }

    #[test]
    fn it_can_decode_const_wide_16() {
        let raw_opcode:&[u8] = &[0x16, 0x44, 0xFF, 0xFF];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-wide/16 v68, #-1", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstWide16(r, i) if r == 0x44 && i == -1));
    }

    #[test]
    fn it_can_decode_const_wide_32() {
        let raw_opcode:&[u8] = &[0x17, 0x44, 0xFF, 0xFF, 0x00, 0x11];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-wide/32 v68, #285278207", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstWide32(r, i) if r == 0x44 && i == 285278207));
    }

    #[test]
    fn it_can_decode_const_wide() {
        let raw_opcode:&[u8] = &[0x18, 0x01, 0x44, 0xFF, 0xFF, 0x00, 0x44, 0xFF, 0xFF, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-wide v1, #72056786600853316", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstWide(r, i) if r == 1 && i == 72056786600853316));
    }

    #[test]
    fn it_can_decode_const_wide_high16() {
        let raw_opcode:&[u8] = &[0x19, 0x01, 0xFF, 0xFF];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-wide/high16 v1, #-281474976710656", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstWideHigh16(r, i) if r == 1 && i == -281474976710656));
    }

    #[test]
    fn it_can_decode_const_string() {
        let raw_opcode:&[u8] = &[0x1A, 0x01, 0xFF, 0xFF];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-string v1, string@65535", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstString(r, i) if r == 1 && i == 65535 as StringReference));
    }

    #[test]
    fn it_can_decode_const_string_jumbo() {
        let raw_opcode:&[u8] = &[0x1B, 0x01, 0xFF, 0xFF, 0x00, 0x10];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-string/jumbo v1, string@268500991", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstStringJumbo(r, i) if r == 1 && i == 268500991 as StringReference));
    }

    #[test]
    fn it_can_decode_const_class() {
        let raw_opcode:&[u8] = &[0x1C, 0x01, 0x11, 0x11];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-class v1, class@4369", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstClass(r, i) if r == 1 && i == 4369 as ClassReference));
    }

    #[test]
    fn it_can_decode_monitor_enter() {
        let raw_opcode:&[u8] = &[0x1D, 0x01];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("monitor-enter v1", opcode.to_string());
        assert!(matches!(opcode, ByteCode::MonitorEnter(r) if r == 1));
    }

    #[test]
    fn it_can_decode_monitor_exit() {
        let raw_opcode:&[u8] = &[0x1E, 0x9];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("monitor-exit v9", opcode.to_string());
        assert!(matches!(opcode, ByteCode::MonitorExit(r) if r == 9));
    }

    #[test]
    fn it_can_decode_check_cast() {
        let raw_opcode:&[u8] = &[0x1F, 0x01, 0x11, 0x11];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("check-cast v1, type@4369", opcode.to_string());
        assert!(matches!(opcode, ByteCode::CheckCast(r, i) if r == 1 && i == 4369 as TypeReference));
    }

    #[test]
    fn it_can_decode_instance_of() {
        let raw_opcode:&[u8] = &[0x20, 0xA2, 0x11, 0x11];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("instance-of v2, v10, type@4369", opcode.to_string());
        assert!(matches!(opcode, ByteCode::InstanceOf(d, s, i) if d == 2 && s == 10 && i == 4369 as TypeReference));
    }

    #[test]
    fn it_can_decode_array_length() {
        let raw_opcode:&[u8] = &[0x21, 0x2A];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("array-length v10, v2", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ArrayLength(d, s) if d == 10 && s == 2));
    }

    #[test]
    fn it_can_decode_new_instance() {
        let raw_opcode:&[u8] = &[0x22, 0x00, 0x20, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("new-instance v0, type@32", opcode.to_string());
        assert!(matches!(opcode, ByteCode::NewInstance(d, reference) if d == 0 && reference == 32));
    }

    #[test]
    fn it_can_decode_new_array() {
        let raw_opcode:&[u8] = &[0x23, 0xA9, 0x20, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("new-array v9, v10, type@32", opcode.to_string());
        assert!(matches!(opcode, ByteCode::NewArray(d, s, reference) if d == 9 && s == 10 && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array() {
        let raw_opcode:&[u8] = &[0x24, 0x04, 0x20, 0x00, 0x12, 0x34];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("filled-new-array {}, type@32", opcode.to_string());
        assert!(matches!(opcode, ByteCode::FilledNewArray(ref registers, reference) if registers.len() == 0 && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_three_elements() {
        let raw_opcode:&[u8] = &[0x24, 0x35, 0x20, 0x00, 0x43, 0x21];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("filled-new-array {v1, v2, v3}, type@32", opcode.to_string());
        assert!(matches!(opcode, ByteCode::FilledNewArray(ref registers, reference) if registers.as_ref() == [1, 2, 3] && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_five_elements() {
        let raw_opcode:&[u8] = &[0x24, 0x55, 0x20, 0x00, 0x43, 0x21];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("filled-new-array {v1, v2, v3, v4, v5}, type@32", opcode.to_string());
        assert!(matches!(opcode, ByteCode::FilledNewArray(ref registers, reference) if registers.as_ref() == [1, 2, 3, 4, 5] && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_more_than_five_elements() {
        let raw_opcode:&[u8] = &[0x24, 0x85, 0x20, 0x00, 0x43, 0x21];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("filled-new-array {v1, v2, v3, v4, v5}, type@32", opcode.to_string());
        assert!(matches!(opcode, ByteCode::FilledNewArray(ref registers, reference) if registers.as_ref() == [1, 2, 3, 4, 5] && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_range() {
        let raw_opcode:&[u8] = &[0x25, 0x03, 0x22, 0x22, 0x01, 0x00];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("filled-new-array/range {v1, v2}, type@8738", opcode.to_string());
        assert!(matches!(opcode, ByteCode::FilledNewArrayRange(start, amount, reference) if start == 1 && amount == 2 && reference == 8738));
    }

    #[test]
    fn it_can_decode_fill_array_data() {
        let raw_opcode:&[u8] = &[0x26, 0x12, 0x11, 0x22, 0x33, 0xFF];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("fill-array-data v18, -13426159", opcode.to_string());
        assert!(matches!(opcode, ByteCode::FillArrayData(reg, offset) if reg == 18 && offset == -13426159));
    }

    #[test]
    fn it_can_decode_throw() {
        let raw_opcode:&[u8] = &[0x27, 0x12];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("throw v18", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Throw(reg) if reg == 18));
    }

    #[test]
    fn it_can_decode_goto() {
        let raw_opcode:&[u8] = &[0x28, 0x03];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("goto 3", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Goto(offset) if offset == 3));
    }

    #[test]
    fn it_can_decode_goto16() {
        let raw_opcode:&[u8] = &[0x29, 0x00, 0x03, 0x04];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("goto/16 1027", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Goto16(offset) if offset == 1027));
    }

    #[test]
    fn it_can_decode_goto32() {
        let raw_opcode:&[u8] = &[0x2A, 0x00, 0x03, 0x04, 0x05, 0x06];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("goto/32 100992003", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Goto32(offset) if offset == 100992003));
    }

    #[test]
    fn it_can_decode_packed_switch() {
        let raw_opcode:&[u8] = &[0x2B, 0x04, 0x03, 0x04, 0x05, 0x06];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("packed-switch v4, 100992003", opcode.to_string());
        assert!(matches!(opcode, ByteCode::PackedSwitch(reg, offset) if reg == 4 && offset == 100992003));
    }

    #[test]
    fn it_can_decode_sparse_switch() {
        let raw_opcode:&[u8] = &[0x2C, 0x04, 0x03, 0x04, 0x05, 0x06];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("sparse-switch v4, 100992003", opcode.to_string());
        assert!(matches!(opcode, ByteCode::SparseSwitch(reg, offset) if reg == 4 && offset == 100992003));
    }

    #[test]
    fn it_can_decode_cmp_little_than_float() {
        let raw_opcode:&[u8] = &[0x2D, 0x04, 0x03, 0x02];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("cmpl-float v4, v3, v2", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Compare(_, dest, op1, op2) if dest == 4 && op1 == 3 && op2 == 2));
    }

    #[test]
    fn it_can_decode_if_ne() {
        let raw_opcode:&[u8] = &[0x33, 0x24, 0x03, 0x02];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("if-ne v4, v2, 515", opcode.to_string());
        assert!(matches!(opcode, ByteCode::If(_, dest, op1, offset) if dest == 4 && op1 == 2 && offset == 515));
    }

    #[test]
    fn it_can_decode_if0_ge() {
        let raw_opcode:&[u8] = &[0x3B, 0x04, 0x03, 0x02];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("if-gez v4, 515", opcode.to_string());
        assert!(matches!(opcode, ByteCode::If0(_, dest, offset) if dest == 4 && offset == 515));
    }

    #[test]
    fn it_can_decode_array_operation() {
        let raw_opcode:&[u8] = &[0x4D, 0x04, 0x03, 0x02];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("aput-object v4, v3, v2", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Array(_, dest, op1, op2) if dest == 4 && op1 == 3 && op2 == 2));
    }

    #[test]
    fn it_can_decode_instance_operation() {
        let raw_opcode:&[u8] = &[0x55, 0x34, 0x03, 0x02];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("iget-boolean v4, v3, field@515", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Instance(_, dest, op1, reference) if dest == 4 && op1 == 3 && reference == 515));
    }

    #[test]
    fn it_can_decode_static_operation() {
        let raw_opcode:&[u8] = &[0x6d, 0x04, 0x03, 0x02];
        let mut d = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("sput-short v4, field@515", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Static(_, dest, reference) if dest == 4 && reference == 515));
    }
}