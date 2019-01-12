//! Representation of the Dalvik bytecodes and utilities to decode them

use std::{
    fmt::Debug,
    io::{self, Read},
    marker::PhantomData,
};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};

#[derive(Debug)]
#[allow(missing_docs)]
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
    Invoke(InvokeKind, Vec<u8>, MethodReference),
    InvokeRange(InvokeKind, u16, u8, MethodReference),
    Unary(UnaryOperation, u8, u8),
    Binary(BinaryOperation, u8, u8, u8),
    Binary2Addr(BinaryOperation, u8, u8),
    BinaryLit16(BinaryOperation, u8, u8, i16),
    BinaryLit8(BinaryOperation, u8, u8, i8),
    InvokePolymorphic(Vec<u8>, MethodReference, PrototypeReference),
    InvokePolymorphicRange(u16, u8, MethodReference, PrototypeReference),
    InvokeCustom(Vec<u8>, CallSiteReference),
    InvokeCustomRange(u16, u8, CallSiteReference),
}

#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
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
        match self {
            CompareType::LittleThanFloat => "cmpl-float".to_string(),
            CompareType::GreaterThanFloat => "cmpg-float".to_string(),
            CompareType::LittleThanDouble => "cmpl-double".to_string(),
            CompareType::GreaterThanDouble => "cmpg-double".to_string(),
            CompareType::Long => "cmp-long".to_string(),
            CompareType::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
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
        match self {
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

#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
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
        match self {
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

#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
pub enum InvokeKind {
    Virtual,
    Super,
    Direct,
    Static,
    Interface,
    Unknown,
}

impl From<u8> for InvokeKind {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x6e | 0x74 => InvokeKind::Virtual,
            0x6f | 0x75 => InvokeKind::Super,
            0x70 | 0x76 => InvokeKind::Direct,
            0x71 | 0x77 => InvokeKind::Static,
            0x72 | 0x78 => InvokeKind::Interface,
            _ => InvokeKind::Unknown,
        }
    }
}

impl ToString for InvokeKind {
    fn to_string(&self) -> String {
        match self {
            InvokeKind::Virtual => "invoke-virtual".to_string(),
            InvokeKind::Super => "invoke-super".to_string(),
            InvokeKind::Direct => "invoke-direct".to_string(),
            InvokeKind::Static => "invoke-static".to_string(),
            InvokeKind::Interface => "invoke-interface".to_string(),
            InvokeKind::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
pub enum UnaryOperation {
    NegateInt,
    NotInt,
    NegateLong,
    NotLong,
    NegateFloat,
    NegateDouble,
    IntToLong,
    IntToFloat,
    IntToDouble,
    LongToInt,
    LongToFloat,
    LongToDouble,
    FloatToInt,
    FloatToLong,
    FloatToDouble,
    DoubleToInt,
    DoubleToLong,
    DoubleToFloat,
    IntToByte,
    IntToChar,
    IntToShort,
    Unknown,
}

impl From<u8> for UnaryOperation {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x7b => UnaryOperation::NegateInt,
            0x7c => UnaryOperation::NotInt,
            0x7d => UnaryOperation::NegateLong,
            0x7e => UnaryOperation::NotLong,
            0x7f => UnaryOperation::NegateFloat,
            0x80 => UnaryOperation::NegateDouble,
            0x81 => UnaryOperation::IntToLong,
            0x82 => UnaryOperation::IntToFloat,
            0x83 => UnaryOperation::IntToDouble,
            0x84 => UnaryOperation::LongToInt,
            0x85 => UnaryOperation::LongToFloat,
            0x86 => UnaryOperation::LongToDouble,
            0x87 => UnaryOperation::FloatToInt,
            0x88 => UnaryOperation::FloatToLong,
            0x89 => UnaryOperation::FloatToDouble,
            0x8a => UnaryOperation::DoubleToInt,
            0x8b => UnaryOperation::DoubleToLong,
            0x8c => UnaryOperation::DoubleToFloat,
            0x8d => UnaryOperation::IntToByte,
            0x8e => UnaryOperation::IntToChar,
            0x8f => UnaryOperation::IntToShort,
            _ => UnaryOperation::Unknown,
        }
    }
}

impl ToString for UnaryOperation {
    fn to_string(&self) -> String {
        match self {
            UnaryOperation::NegateInt => "neg-int".to_string(),
            UnaryOperation::NotInt => "not-int".to_string(),
            UnaryOperation::NegateLong => "neg-long".to_string(),
            UnaryOperation::NotLong => "not-long".to_string(),
            UnaryOperation::NegateFloat => "neg-float".to_string(),
            UnaryOperation::NegateDouble => "neg-double".to_string(),
            UnaryOperation::IntToLong => "int-to-long".to_string(),
            UnaryOperation::IntToFloat => "int-to-float".to_string(),
            UnaryOperation::IntToDouble => "int-to-double".to_string(),
            UnaryOperation::LongToInt => "long-to-int".to_string(),
            UnaryOperation::LongToFloat => "long-to-float".to_string(),
            UnaryOperation::LongToDouble => "long-to-double".to_string(),
            UnaryOperation::FloatToInt => "float-to-int".to_string(),
            UnaryOperation::FloatToLong => "float-to-long".to_string(),
            UnaryOperation::FloatToDouble => "float-to-double".to_string(),
            UnaryOperation::DoubleToInt => "double-to-int".to_string(),
            UnaryOperation::DoubleToLong => "double-to-long".to_string(),
            UnaryOperation::DoubleToFloat => "double-to-float".to_string(),
            UnaryOperation::IntToByte => "int-to-byte".to_string(),
            UnaryOperation::IntToChar => "int-to-char".to_string(),
            UnaryOperation::IntToShort => "int-to-short".to_string(),
            UnaryOperation::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
pub enum BinaryOperation {
    AddInt,
    SubInt,
    MulInt,
    DivInt,
    RemInt,
    AndInt,
    OrInt,
    XorInt,
    ShlInt,
    ShrInt,
    UshrInt,
    AddLong,
    SubLong,
    MulLong,
    DivLong,
    RemLong,
    AndLong,
    OrLong,
    XorLong,
    ShlLong,
    ShrLong,
    UshrLong,
    AddFloat,
    SubFloat,
    MulFloat,
    DivFloat,
    RemFloat,
    AddDouble,
    SubDouble,
    MulDouble,
    DivDouble,
    RemDouble,
    Unknown,
}

impl From<u8> for BinaryOperation {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x90 | 0xb0 | 0xd0 | 0xd8 => BinaryOperation::AddInt,
            0x91 | 0xb1 | 0xd1 | 0xd9 => BinaryOperation::SubInt,
            0x92 | 0xb2 | 0xd2 | 0xda => BinaryOperation::MulInt,
            0x93 | 0xb3 | 0xd3 | 0xdb => BinaryOperation::DivInt,
            0x94 | 0xb4 | 0xd4 | 0xdc => BinaryOperation::RemInt,
            0x95 | 0xb5 | 0xd5 | 0xdd => BinaryOperation::AndInt,
            0x96 | 0xb6 | 0xd6 | 0xde => BinaryOperation::OrInt,
            0x97 | 0xb7 | 0xd7 | 0xdf => BinaryOperation::XorInt,
            0x98 | 0xb8 | 0xe0 => BinaryOperation::ShlInt,
            0x99 | 0xb9 | 0xe1 => BinaryOperation::ShrInt,
            0x9a | 0xba | 0xe2 => BinaryOperation::UshrInt,
            0x9b | 0xbb => BinaryOperation::AddLong,
            0x9c | 0xbc => BinaryOperation::SubLong,
            0x9d | 0xbd => BinaryOperation::MulLong,
            0x9e | 0xbe => BinaryOperation::DivLong,
            0x9f | 0xbf => BinaryOperation::RemLong,
            0xa0 | 0xc0 => BinaryOperation::AndLong,
            0xa1 | 0xc1 => BinaryOperation::OrLong,
            0xa2 | 0xc2 => BinaryOperation::XorLong,
            0xa3 | 0xc3 => BinaryOperation::ShlLong,
            0xa4 | 0xc4 => BinaryOperation::ShrLong,
            0xa5 | 0xc5 => BinaryOperation::UshrLong,
            0xa6 | 0xc6 => BinaryOperation::AddFloat,
            0xa7 | 0xc7 => BinaryOperation::SubFloat,
            0xa8 | 0xc8 => BinaryOperation::MulFloat,
            0xa9 | 0xc9 => BinaryOperation::DivFloat,
            0xaa | 0xca => BinaryOperation::RemFloat,
            0xab | 0xcb => BinaryOperation::AddDouble,
            0xac | 0xcc => BinaryOperation::SubDouble,
            0xad | 0xcd => BinaryOperation::MulDouble,
            0xae | 0xce => BinaryOperation::DivDouble,
            0xaf | 0xcf => BinaryOperation::RemDouble,
            _ => BinaryOperation::Unknown,
        }
    }
}

impl ToString for BinaryOperation {
    fn to_string(&self) -> String {
        match self {
            BinaryOperation::AddInt => "add-int".to_string(),
            BinaryOperation::SubInt => "sub-int".to_string(),
            BinaryOperation::MulInt => "mul-int".to_string(),
            BinaryOperation::DivInt => "div-int".to_string(),
            BinaryOperation::RemInt => "rem-int".to_string(),
            BinaryOperation::AndInt => "and-int".to_string(),
            BinaryOperation::OrInt => "or-int".to_string(),
            BinaryOperation::XorInt => "xor-int".to_string(),
            BinaryOperation::ShlInt => "shl-int".to_string(),
            BinaryOperation::ShrInt => "shr-int".to_string(),
            BinaryOperation::UshrInt => "ushr-int".to_string(),
            BinaryOperation::AddLong => "add-long".to_string(),
            BinaryOperation::SubLong => "sub-long".to_string(),
            BinaryOperation::MulLong => "mul-long".to_string(),
            BinaryOperation::DivLong => "div-long".to_string(),
            BinaryOperation::RemLong => "rem-long".to_string(),
            BinaryOperation::AndLong => "and-long".to_string(),
            BinaryOperation::OrLong => "or-long".to_string(),
            BinaryOperation::XorLong => "xor-long".to_string(),
            BinaryOperation::ShlLong => "shl-long".to_string(),
            BinaryOperation::ShrLong => "shr-long".to_string(),
            BinaryOperation::UshrLong => "ushr-long".to_string(),
            BinaryOperation::AddFloat => "add-float".to_string(),
            BinaryOperation::SubFloat => "sub-float".to_string(),
            BinaryOperation::MulFloat => "mul-float".to_string(),
            BinaryOperation::DivFloat => "div-float".to_string(),
            BinaryOperation::RemFloat => "rem-float".to_string(),
            BinaryOperation::AddDouble => "add-double".to_string(),
            BinaryOperation::SubDouble => "sub-double".to_string(),
            BinaryOperation::MulDouble => "mul-double".to_string(),
            BinaryOperation::DivDouble => "div-double".to_string(),
            BinaryOperation::RemDouble => "rem-double".to_string(),
            BinaryOperation::Unknown => "unknown".to_string(),
        }
    }
}

/// String index on the Dex string table
pub type StringReference = u32;
/// Class index on the Dex class table
pub type ClassReference = u32;
/// Type index on the Dex type table
pub type TypeReference = u32;
/// Field index on the Dex field table
pub type FieldReference = u32;
/// Method index on the Dex method table
pub type MethodReference = u32;
/// Prototype index on the Dex prototype table
pub type PrototypeReference = u32;
/// Call site index on the Dex call site table
pub type CallSiteReference = u32;

impl ToString for ByteCode {
    fn to_string(&self) -> String {
        match self {
            ByteCode::Nop => "nop".to_string(),
            ByteCode::Move(dest, source) => format!("move v{}, v{}", dest, source),
            ByteCode::MoveFrom16(dest, source) => format!("move/from16 v{}, v{}", dest, source),
            ByteCode::Move16(dest, source) => format!("move/16 v{}, v{}", dest, source),
            ByteCode::MoveWide(dest, source) => format!("move-wide v{}, v{}", dest, source),
            ByteCode::MoveWideFrom16(dest, source) => {
                format!("move-wide/from16 v{}, v{}", dest, source)
            }
            ByteCode::MoveWide16(dest, source) => format!("move-wide/16 v{}, v{}", dest, source),
            ByteCode::MoveObject(dest, source) => format!("move-object v{}, v{}", dest, source),
            ByteCode::MoveObjectFrom16(dest, source) => {
                format!("move-object/from16 v{}, v{}", dest, source)
            }
            ByteCode::MoveObject16(dest, source) => {
                format!("move-object/16 v{}, v{}", dest, source)
            }
            ByteCode::MoveResult(dest) => format!("move-result v{}", dest),
            ByteCode::MoveResultWide(dest) => format!("move-result-wide v{}", dest),
            ByteCode::MoveResultObject(dest) => format!("move-result-object v{}", dest),
            ByteCode::MoveException(dest) => format!("move-exception v{}", dest),
            ByteCode::ReturnVoid => "return-void".to_string(),
            ByteCode::Return(dest) => format!("return v{}", dest),
            ByteCode::ReturnWide(dest) => format!("return-wide v{}", dest),
            ByteCode::ReturnObject(dest) => format!("return-object v{}", dest),
            ByteCode::Const4(dest, literal) => format!("const/4 v{}, #{}", dest, literal),
            ByteCode::Const16(dest, literal) => format!("const/16 v{}, #{}", dest, literal),
            ByteCode::Const(dest, literal) => format!("const v{}, #{}", dest, literal),
            ByteCode::ConstHigh16(dest, literal) => format!("const/high16 v{}, #{}", dest, literal),
            ByteCode::ConstWide16(dest, literal) => {
                format!("const-wide/16 v{}, #{}", dest, literal)
            }
            ByteCode::ConstWide32(dest, literal) => {
                format!("const-wide/32 v{}, #{}", dest, literal)
            }
            ByteCode::ConstWide(dest, literal) => format!("const-wide v{}, #{}", dest, literal),
            ByteCode::ConstWideHigh16(dest, literal) => {
                format!("const-wide/high16 v{}, #{}", dest, literal)
            }
            ByteCode::ConstString(dest, reference) => {
                format!("const-string v{}, string@{}", dest, reference)
            }
            ByteCode::ConstStringJumbo(dest, reference) => {
                format!("const-string/jumbo v{}, string@{}", dest, reference)
            }
            ByteCode::ConstClass(dest, reference) => {
                format!("const-class v{}, class@{}", dest, reference)
            }
            ByteCode::MonitorEnter(reg) => format!("monitor-enter v{}", reg),
            ByteCode::MonitorExit(reg) => format!("monitor-exit v{}", reg),
            ByteCode::CheckCast(reg, reference) => {
                format!("check-cast v{}, type@{}", reg, reference)
            }
            ByteCode::InstanceOf(dest, src, reference) => {
                format!("instance-of v{}, v{}, type@{}", dest, src, reference)
            }
            ByteCode::ArrayLength(dest, src) => format!("array-length v{}, v{}", dest, src),
            ByteCode::NewInstance(dest, reference) => {
                format!("new-instance v{}, type@{}", dest, reference)
            }
            ByteCode::NewArray(dest, src, reference) => {
                format!("new-array v{}, v{}, type@{}", dest, src, reference)
            }
            ByteCode::FilledNewArray(ref registers, reference) => {
                let str_register: Vec<String> =
                    registers.iter().map(|r| format!("v{}", r)).collect();
                format!(
                    "filled-new-array {{{}}}, type@{}",
                    str_register.join(", "),
                    reference
                )
            }
            ByteCode::FilledNewArrayRange(first_reg, amount, reference) => {
                let str_register: Vec<String> = (*first_reg..=(*first_reg + u16::from(*amount)))
                    .map(|r| format!("v{}", r))
                    .collect();
                format!(
                    "filled-new-array/range {{{}}}, type@{}",
                    str_register.join(", "),
                    reference
                )
            }
            ByteCode::FillArrayData(reg, offset) => format!("fill-array-data v{}, {}", reg, offset),
            ByteCode::Throw(reg) => format!("throw v{}", reg),
            ByteCode::Goto(offset) => format!("goto {}", offset),
            ByteCode::Goto16(offset) => format!("goto/16 {}", offset),
            ByteCode::Goto32(offset) => format!("goto/32 {}", offset),
            ByteCode::PackedSwitch(reg, offset) => format!("packed-switch v{}, {}", reg, offset),
            ByteCode::SparseSwitch(reg, offset) => format!("sparse-switch v{}, {}", reg, offset),
            ByteCode::Compare(ref ct, dest, op1, op2) => {
                format!("{} v{}, v{}, v{}", ct.to_string(), dest, op1, op2)
            }
            ByteCode::If(ref tt, dest, src, offset) => {
                format!("{} v{}, v{}, {}", tt.to_string(), dest, src, offset)
            }
            ByteCode::If0(ref tt, dest, offset) => {
                format!("{}z v{}, {}", tt.to_string(), dest, offset)
            }
            ByteCode::Array(ref array_op, dest, op1, op2) => {
                format!("a{} v{}, v{}, v{}", array_op.to_string(), dest, op1, op2)
            }
            ByteCode::Instance(ref array_op, dest, op1, field) => format!(
                "i{} v{}, v{}, field@{}",
                array_op.to_string(),
                dest,
                op1,
                field
            ),
            ByteCode::Static(ref array_op, dest, field) => {
                format!("s{} v{}, field@{}", array_op.to_string(), dest, field)
            }
            ByteCode::Invoke(ref invoke_kind, ref registers, method) => {
                let str_register: Vec<String> =
                    registers.iter().map(|r| format!("v{}", r)).collect();
                format!(
                    "{} {{{}}}, method@{}",
                    invoke_kind.to_string(),
                    str_register.join(", "),
                    method
                )
            }
            ByteCode::InvokeRange(ref invoke_kind, first_reg, amount, reference) => {
                let str_register: Vec<String> = (*first_reg..(*first_reg + u16::from(*amount)))
                    .map(|r| format!("v{}", r))
                    .collect();
                format!(
                    "{}/range {{{}}}, method@{}",
                    invoke_kind.to_string(),
                    str_register.join(", "),
                    reference
                )
            }
            ByteCode::Unary(ref operation, dest, src) => {
                format!("{} v{}, v{}", operation.to_string(), dest, src)
            }
            ByteCode::Binary(ref operation, dest, op1, op2) => {
                format!("{} v{}, v{}, v{}", operation.to_string(), dest, op1, op2)
            }
            ByteCode::Binary2Addr(ref operation, src1, src2) => {
                format!("{}/2addr v{}, v{}", operation.to_string(), src1, src2)
            }
            ByteCode::BinaryLit16(ref operation, dest, src, literal) => match operation {
                BinaryOperation::SubInt => format!("rsub-int v{}, v{}, #{}", dest, src, literal),
                _ => format!(
                    "{}/lit16 v{}, v{}, #{}",
                    operation.to_string(),
                    dest,
                    src,
                    literal
                ),
            },
            ByteCode::BinaryLit8(ref operation, dest, src, literal) => format!(
                "{}/lit8 v{}, v{}, #{}",
                operation.to_string(),
                dest,
                src,
                literal
            ),
            ByteCode::InvokePolymorphic(ref registers, method, proto) => {
                let str_register: Vec<String> =
                    registers.iter().map(|r| format!("v{}", r)).collect();
                format!(
                    "invoke-polymorphic {{{}}}, method@{} proto@{}",
                    str_register.join(", "),
                    method,
                    proto
                )
            }
            ByteCode::InvokePolymorphicRange(first_reg, amount, method, proto) => {
                let str_register: Vec<String> = (*first_reg..(*first_reg + u16::from(*amount)))
                    .map(|r| format!("v{}", r))
                    .collect();
                format!(
                    "invoke-polymorphic/range {{{}}}, method@{} proto@{}",
                    str_register.join(", "),
                    method,
                    proto
                )
            }
            ByteCode::InvokeCustom(ref registers, call_site) => {
                let str_register: Vec<String> =
                    registers.iter().map(|r| format!("v{}", r)).collect();
                format!(
                    "invoke-custom {{{}}}, call_site@{}",
                    str_register.join(", "),
                    call_site
                )
            }
            ByteCode::InvokeCustomRange(first_reg, amount, call_site) => {
                let str_register: Vec<String> = (*first_reg..(*first_reg + u16::from(*amount)))
                    .map(|r| format!("v{}", r))
                    .collect();
                format!(
                    "invoke-custom/range {{{}}}, call_site@{}",
                    str_register.join(", "),
                    call_site
                )
            }
        }
    }
}

/// Implementations of the distinct bytecodes data layouts.
///
/// It will read from the source and return the data de-structured.
#[derive(Debug)]
pub struct ByteCodeDecoder<R: Read + Debug, B: ByteOrder = LittleEndian> {
    cursor: R,
    byte_order: PhantomData<B>,
}

impl<R: Read + Debug, B: ByteOrder> ByteCodeDecoder<R, B> {
    /// Creates a new ByteCodeDecoder given a `Read` input
    pub fn new(cursor: R) -> Self {
        Self {
            cursor,
            byte_order: PhantomData,
        }
    }

    fn format10x(&mut self) -> Result<(), io::Error> {
        let _ = self.cursor.read_u8()?;

        Ok(())
    }

    fn format10t(&mut self) -> Result<i8, io::Error> {
        Ok(self.cursor.read_i8()?)
    }

    fn format11x(&mut self) -> Result<u8, io::Error> {
        Ok(self.cursor.read_u8()?)
    }

    fn format11n(&mut self) -> Result<(u8, i32), io::Error> {
        let current_byte = self.cursor.read_u8()?;

        let literal = i32::from((current_byte & 0xF0) as i8 >> 4);
        let register = current_byte & 0xF;

        Ok((register, literal))
    }

    fn format12x(&mut self) -> Result<(u8, u8), io::Error> {
        let current_byte = self.cursor.read_u8()?;

        let source = (current_byte & 0xF0) >> 4;
        let dest = current_byte & 0xF;

        Ok((dest, source))
    }

    fn format20t(&mut self) -> Result<i16, io::Error>
    where
        B: ByteOrder,
    {
        let _ = self.cursor.read_u8()?;
        let literal = self.cursor.read_i16::<B>()?;

        Ok(literal)
    }
    fn format21t(&mut self) -> Result<(u8, i16), io::Error>
    where
        B: ByteOrder,
    {
        let dest = self.cursor.read_u8()?;
        let offset = self.cursor.read_i16::<B>()?;

        Ok((dest, offset))
    }

    fn format21s(&mut self) -> Result<(u8, i32), io::Error> {
        let dest = self.cursor.read_u8()?;
        let literal = self.cursor.read_i16::<B>()?;

        Ok((dest, i32::from(literal)))
    }

    fn format21hw(&mut self) -> Result<(u8, i32), io::Error> {
        let dest = self.cursor.read_u8()?;
        let literal = (i32::from(self.cursor.read_i16::<B>()?)) << 16;

        Ok((dest, literal))
    }

    fn format21hd(&mut self) -> Result<(u8, i64), io::Error> {
        let dest = self.cursor.read_u8()?;
        let literal = (i64::from(self.cursor.read_i16::<B>()?)) << 48;

        Ok((dest, literal))
    }

    fn format21c(&mut self) -> Result<(u8, u16), io::Error> {
        let dest = self.cursor.read_u8()?;
        let literal = self.cursor.read_u16::<B>()?;

        Ok((dest, literal))
    }

    fn format22c(&mut self) -> Result<(u8, u8, u16), io::Error> {
        let current_byte = self.cursor.read_u8()?;

        let source = (current_byte & 0xF0) >> 4;
        let dest = current_byte & 0xF;
        let reference = self.cursor.read_u16::<B>()?;

        Ok((dest, source, reference))
    }

    fn format22x(&mut self) -> Result<(u8, u16), io::Error> {
        let dest = self.cursor.read_u8()?;
        let source = self.cursor.read_u16::<B>()?;

        Ok((dest, source))
    }

    fn format22t(&mut self) -> Result<(u8, u8, i16), io::Error> {
        let current_byte = self.cursor.read_u8()?;

        let source = (current_byte & 0xF0) >> 4;
        let dest = current_byte & 0xF;
        let offset = self.cursor.read_i16::<B>()?;

        Ok((dest, source, offset))
    }

    fn format22s(&mut self) -> Result<(u8, u8, i16), io::Error> {
        self.format22t()
    }

    fn format22b(&mut self) -> Result<(u8, u8, i8), io::Error> {
        let dest = self.cursor.read_u8()?;
        let operand1 = self.cursor.read_u8()?;
        let literal = self.cursor.read_i8()?;

        Ok((dest, operand1, literal))
    }

    fn format23x(&mut self) -> Result<(u8, u8, u8), io::Error> {
        let dest = self.cursor.read_u8()?;
        let operand1 = self.cursor.read_u8()?;
        let operand2 = self.cursor.read_u8()?;

        Ok((dest, operand1, operand2))
    }

    fn format30t(&mut self) -> Result<i32, io::Error> {
        let _ = self.cursor.read_u8()?;
        let literal = self.cursor.read_i32::<B>()?;

        Ok(literal)
    }

    fn format31i(&mut self) -> Result<(u8, i32), io::Error> {
        let dest = self.cursor.read_u8()?;
        let literal = self.cursor.read_i32::<B>()?;

        Ok((dest, literal))
    }

    fn format31t(&mut self) -> Result<(u8, i32), io::Error> {
        let dest = self.cursor.read_u8()?;
        let literal = self.cursor.read_i32::<B>()?;

        Ok((dest, literal))
    }

    fn format31c(&mut self) -> Result<(u8, u32), io::Error> {
        let dest = self.cursor.read_u8()?;
        let reference = self.cursor.read_u32::<B>()?;

        Ok((dest, reference))
    }

    fn format32x(&mut self) -> Result<(u16, u16), io::Error> {
        let dest = self.cursor.read_u16::<B>()?;
        let source = self.cursor.read_u16::<B>()?;

        Ok((dest, source))
    }

    fn format35c(&mut self) -> Result<(Vec<u8>, u16), io::Error> {
        let mut arguments = Vec::new();
        let first_byte = self.cursor.read_u8()?;

        let count = (first_byte & 0xF0) >> 4;
        let last_register = first_byte & 0xF;

        let reference = self.cursor.read_u16::<B>()?;

        let reg_array = self.read_4bit_array(4)?;
        arguments.extend(reg_array);
        arguments.push(last_register);

        let final_arguments = arguments.into_iter().take(count as usize).collect();

        Ok((final_arguments, reference))
    }

    fn format3rc(&mut self) -> Result<(u16, u8, u16), io::Error> {
        let amount = self.cursor.read_u8()?;
        let reference = self.cursor.read_u16::<LittleEndian>()?;
        let first = self.cursor.read_u16::<LittleEndian>()?;

        Ok((first, amount - 1, reference))
    }

    fn format45cc(&mut self) -> Result<(Vec<u8>, u16, u16), io::Error> {
        let (registers, method_ref) = self.format35c()?;
        let proto_ref = self.cursor.read_u16::<B>()?;

        Ok((registers, method_ref, proto_ref))
    }

    fn format4rcc(&mut self) -> Result<(u16, u8, u16, u16), io::Error> {
        let (first, amount, method_ref) = self.format3rc()?;
        let proto_ref = self.cursor.read_u16::<B>()?;

        Ok((first, amount, method_ref, proto_ref))
    }

    fn format51l(&mut self) -> Result<(u8, i64), io::Error> {
        let dest = self.cursor.read_u8()?;
        let source = self.cursor.read_i64::<B>()?;

        Ok((dest, source))
    }

    fn read_4bit_array(&mut self, amount: u8) -> Result<Vec<u8>, io::Error> {
        let mut values = Vec::new();

        for _ in 0..(amount / 2) {
            let current = self.cursor.read_u8()?;

            let high = (current & 0xF0) >> 4;
            let low = current & 0xF;

            values.push(low);
            values.push(high);
        }

        Ok(values)
    }
}

impl<R: Read + Debug, B: ByteOrder> Iterator for ByteCodeDecoder<R, B> {
    type Item = ByteCode;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.cursor.read_u8();

        match byte {
            Ok(0x00) => self.format10x().ok().map(|_| ByteCode::Nop),
            Ok(0x01) => self.format12x().ok().map(|(d, s)| ByteCode::Move(d, s)),
            Ok(0x02) => self
                .format22x()
                .ok()
                .map(|(d, s)| ByteCode::MoveFrom16(d, s)),
            Ok(0x03) => self.format32x().ok().map(|(d, s)| ByteCode::Move16(d, s)),
            Ok(0x04) => self.format12x().ok().map(|(d, s)| ByteCode::MoveWide(d, s)),
            Ok(0x05) => self
                .format22x()
                .ok()
                .map(|(d, s)| ByteCode::MoveWideFrom16(d, s)),
            Ok(0x06) => self
                .format32x()
                .ok()
                .map(|(d, s)| ByteCode::MoveWide16(d, s)),
            Ok(0x07) => self
                .format12x()
                .ok()
                .map(|(d, s)| ByteCode::MoveObject(d, s)),
            Ok(0x08) => self
                .format22x()
                .ok()
                .map(|(d, s)| ByteCode::MoveObjectFrom16(d, s)),
            Ok(0x09) => self
                .format32x()
                .ok()
                .map(|(d, s)| ByteCode::MoveObject16(d, s)),
            Ok(0x0A) => self.format11x().ok().map(ByteCode::MoveResult),
            Ok(0x0B) => self.format11x().ok().map(ByteCode::MoveResultWide),
            Ok(0x0C) => self.format11x().ok().map(ByteCode::MoveResultObject),
            Ok(0x0D) => self.format11x().ok().map(ByteCode::MoveException),
            Ok(0x0E) => self.format10x().ok().map(|_| ByteCode::ReturnVoid),
            Ok(0x0F) => self.format11x().ok().map(ByteCode::Return),
            Ok(0x10) => self.format11x().ok().map(ByteCode::ReturnWide),
            Ok(0x11) => self.format11x().ok().map(ByteCode::ReturnObject),
            Ok(0x12) => self
                .format11n()
                .ok()
                .map(|(reg, lit)| ByteCode::Const4(reg, lit)),
            Ok(0x13) => self
                .format21s()
                .ok()
                .map(|(reg, lit)| ByteCode::Const16(reg, lit)),
            Ok(0x14) => self
                .format31i()
                .ok()
                .map(|(reg, lit)| ByteCode::Const(reg, lit)),
            Ok(0x15) => self
                .format21hw()
                .ok()
                .map(|(reg, lit)| ByteCode::ConstHigh16(reg, lit)),
            Ok(0x16) => self
                .format21s()
                .ok()
                .map(|(reg, lit)| ByteCode::ConstWide16(reg, i64::from(lit))),
            Ok(0x17) => self
                .format31i()
                .ok()
                .map(|(reg, lit)| ByteCode::ConstWide32(reg, i64::from(lit))),
            Ok(0x18) => self
                .format51l()
                .ok()
                .map(|(reg, lit)| ByteCode::ConstWide(reg, lit)),
            Ok(0x19) => self
                .format21hd()
                .ok()
                .map(|(reg, lit)| ByteCode::ConstWideHigh16(reg, lit)),
            Ok(0x1A) => self.format21c().ok().map(|(reg, reference)| {
                ByteCode::ConstString(reg, StringReference::from(reference))
            }),
            Ok(0x1B) => self
                .format31c()
                .ok()
                .map(|(reg, reference)| ByteCode::ConstStringJumbo(reg, reference)),
            Ok(0x1C) => self
                .format21c()
                .ok()
                .map(|(reg, reference)| ByteCode::ConstClass(reg, ClassReference::from(reference))),
            Ok(0x1D) => self.format11x().ok().map(ByteCode::MonitorEnter),
            Ok(0x1E) => self.format11x().ok().map(ByteCode::MonitorExit),
            Ok(0x1F) => self
                .format21c()
                .ok()
                .map(|(reg, reference)| ByteCode::CheckCast(reg, TypeReference::from(reference))),
            Ok(0x20) => self.format22c().ok().map(|(dest, src, reference)| {
                ByteCode::InstanceOf(dest, src, TypeReference::from(reference))
            }),
            Ok(0x21) => self
                .format12x()
                .ok()
                .map(|(dest, src)| ByteCode::ArrayLength(dest, src)),
            Ok(0x22) => self.format21c().ok().map(|(dest, reference)| {
                ByteCode::NewInstance(dest, TypeReference::from(reference))
            }),
            Ok(0x23) => self.format22c().ok().map(|(dest, size, reference)| {
                ByteCode::NewArray(dest, size, TypeReference::from(reference))
            }),
            Ok(0x24) => self.format35c().ok().map(|(registers, reference)| {
                ByteCode::FilledNewArray(registers, TypeReference::from(reference))
            }),
            Ok(0x25) => self.format3rc().ok().map(|(first, amount, reference)| {
                ByteCode::FilledNewArrayRange(first, amount, TypeReference::from(reference))
            }),
            Ok(0x26) => self
                .format31t()
                .ok()
                .map(|(reg, offset)| ByteCode::FillArrayData(reg, offset)),
            Ok(0x27) => self.format11x().ok().map(ByteCode::Throw),
            Ok(0x28) => self.format10t().ok().map(ByteCode::Goto),
            Ok(0x29) => self.format20t().ok().map(ByteCode::Goto16),
            Ok(0x2A) => self.format30t().ok().map(ByteCode::Goto32),
            Ok(0x2B) => self
                .format31t()
                .ok()
                .map(|(reg, offset)| ByteCode::PackedSwitch(reg, offset)),
            Ok(0x2C) => self
                .format31t()
                .ok()
                .map(|(reg, offset)| ByteCode::SparseSwitch(reg, offset)),
            Ok(a @ 0x2D...0x31) => self
                .format23x()
                .ok()
                .map(|(dest, op1, op2)| ByteCode::Compare(CompareType::from(a), dest, op1, op2)),
            Ok(a @ 0x32...0x37) => self
                .format22t()
                .ok()
                .map(|(dest, src, offset)| ByteCode::If(TestType::from(a), dest, src, offset)),
            Ok(a @ 0x38...0x3D) => self
                .format21t()
                .ok()
                .map(|(dest, offset)| ByteCode::If0(TestType::from(a), dest, offset)),
            Ok(a @ 0x44...0x51) => self
                .format23x()
                .ok()
                .map(|(dest, op1, op2)| ByteCode::Array(ArrayOperation::from(a), dest, op1, op2)),
            Ok(a @ 0x52...0x5f) => self.format22c().ok().map(|(dest, op1, reference)| {
                ByteCode::Instance(
                    ArrayOperation::from(a),
                    dest,
                    op1,
                    FieldReference::from(reference),
                )
            }),
            Ok(a @ 0x60...0x6d) => self.format21c().ok().map(|(dest, reference)| {
                ByteCode::Static(
                    ArrayOperation::from(a),
                    dest,
                    FieldReference::from(reference),
                )
            }),
            Ok(a @ 0x6e...0x72) => self.format35c().ok().map(|(registers, reference)| {
                ByteCode::Invoke(
                    InvokeKind::from(a),
                    registers,
                    MethodReference::from(reference),
                )
            }),
            Ok(a @ 0x74...0x78) => self.format3rc().ok().map(|(first, amount, reference)| {
                ByteCode::InvokeRange(
                    InvokeKind::from(a),
                    first,
                    amount,
                    FieldReference::from(reference),
                )
            }),
            Ok(op @ 0x7b...0x8f) => self
                .format12x()
                .ok()
                .map(|(dest, src)| ByteCode::Unary(UnaryOperation::from(op), dest, src)),
            Ok(op @ 0x90...0xaf) => self.format23x().ok().map(|(dest, src1, src2)| {
                ByteCode::Binary(BinaryOperation::from(op), dest, src1, src2)
            }),
            Ok(op @ 0xb0...0xcf) => self.format12x().ok().map(|(src_dest, src)| {
                ByteCode::Binary2Addr(BinaryOperation::from(op), src_dest, src)
            }),
            Ok(op @ 0xd0...0xd7) => self.format22s().ok().map(|(dest, src, literal)| {
                ByteCode::BinaryLit16(BinaryOperation::from(op), dest, src, literal)
            }),
            Ok(op @ 0xd8...0xe2) => self.format22b().ok().map(|(dest, src, literal)| {
                ByteCode::BinaryLit8(BinaryOperation::from(op), dest, src, literal)
            }),
            Ok(0xfa) => self.format45cc().ok().map(|(registers, method, proto)| {
                ByteCode::InvokePolymorphic(registers, u32::from(method), u32::from(proto))
            }),
            Ok(0xfb) => self
                .format4rcc()
                .ok()
                .map(|(first, amount, method, proto)| {
                    ByteCode::InvokePolymorphicRange(
                        first,
                        amount,
                        u32::from(method),
                        u32::from(proto),
                    )
                }),
            Ok(0xfc) => self.format35c().ok().map(|(registers, call_site)| {
                ByteCode::InvokeCustom(registers, u32::from(call_site))
            }),
            Ok(0xfd) => self.format3rc().ok().map(|(first, amount, call_site)| {
                ByteCode::InvokeCustomRange(first, amount, u32::from(call_site))
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteCode, ByteCodeDecoder, LittleEndian};
    use matches::matches;

    #[test]
    fn it_can_decode_noop() {
        let raw_opcode: &[u8] = &[0x00, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Nop));
        assert_eq!("nop", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return_void() {
        let raw_opcode: &[u8] = &[0x0e, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::ReturnVoid));
        assert_eq!("return-void", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move() {
        let raw_opcode: &[u8] = &[0x01, 0x3B];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Move(d, s) if d == 0xB && s == 0x3));
        assert_eq!("move v11, v3", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_from_16() {
        let raw_opcode: &[u8] = &[0x02, 0xAA, 0x12, 0x34];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveFrom16(d, s) if d == 0xAA && s == 0x3412));
        assert_eq!("move/from16 v170, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_16() {
        let raw_opcode: &[u8] = &[0x03, 0xAA, 0x01, 0x12, 0x34];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Move16(d, s) if d == 0x01AA && s == 0x3412));
        assert_eq!("move/16 v426, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_wide() {
        let raw_opcode: &[u8] = &[0x04, 0x3B];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveWide(d, s) if d == 0xB && s == 0x3));
        assert_eq!("move-wide v11, v3", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_wide_from_16() {
        let raw_opcode: &[u8] = &[0x05, 0xAA, 0x12, 0x34];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveWideFrom16(d, s) if d == 0xAA && s == 0x3412));
        assert_eq!("move-wide/from16 v170, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_wide_16() {
        let raw_opcode: &[u8] = &[0x06, 0xAA, 0x01, 0x12, 0x34];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveWide16(d, s) if d == 0x01AA && s == 0x3412));
        assert_eq!("move-wide/16 v426, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_object() {
        let raw_opcode: &[u8] = &[0x07, 0x3B];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveObject(d, s) if d == 0xB && s == 0x3));
        assert_eq!("move-object v11, v3", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_object_from_16() {
        let raw_opcode: &[u8] = &[0x08, 0xAA, 0x12, 0x34];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveObjectFrom16(d, s) if d == 0xAA && s == 0x3412));
        assert_eq!("move-object/from16 v170, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_object_16() {
        let raw_opcode: &[u8] = &[0x09, 0xAA, 0x01, 0x12, 0x34];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveObject16(d, s) if d == 0x01AA && s == 0x3412));
        assert_eq!("move-object/16 v426, v13330", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_result() {
        let raw_opcode: &[u8] = &[0x0A, 0x3B];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveResult(d) if d == 0x3B));
        assert_eq!("move-result v59", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_result_wide() {
        let raw_opcode: &[u8] = &[0x0B, 0x12];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveResultWide(d) if d == 0x12));
        assert_eq!("move-result-wide v18", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_result_object() {
        let raw_opcode: &[u8] = &[0x0C, 0xFF];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveResultObject(d) if d == 0xFF));
        assert_eq!("move-result-object v255", opcode.to_string());
    }

    #[test]
    fn it_can_decode_move_exception() {
        let raw_opcode: &[u8] = &[0x0D, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::MoveException(d) if d == 0x00));
        assert_eq!("move-exception v0", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return() {
        let raw_opcode: &[u8] = &[0x0F, 0x23];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Return(d) if d == 0x23));
        assert_eq!("return v35", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return_wide() {
        let raw_opcode: &[u8] = &[0x10, 0x23];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::ReturnWide(d) if d == 0x23));
        assert_eq!("return-wide v35", opcode.to_string());
    }

    #[test]
    fn it_can_decode_return_object() {
        let raw_opcode: &[u8] = &[0x11, 0x23];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::ReturnObject(d) if d == 0x23));
        assert_eq!("return-object v35", opcode.to_string());
    }

    #[test]
    fn it_can_decode_const_4_neg() {
        let raw_opcode: &[u8] = &[0x12, 0xF1];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Const4(r, i) if r == 0x1 && i == -1));
        assert_eq!("const/4 v1, #-1", opcode.to_string());
    }

    #[test]
    fn it_can_decode_const_4_pos() {
        let raw_opcode: &[u8] = &[0x12, 0x71];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert!(matches!(opcode, ByteCode::Const4(r, i) if r == 0x1 && i == 7));
        assert_eq!("const/4 v1, #7", opcode.to_string());
    }

    #[test]
    fn it_can_decode_const_16_neg() {
        let raw_opcode: &[u8] = &[0x13, 0xF1, 0xFA, 0xFB];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const/16 v241, #-1030", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Const16(r, i) if r == 0xF1 && i == -1030));
    }

    #[test]
    fn it_can_decode_const() {
        let raw_opcode: &[u8] = &[0x14, 0x44, 0xFA, 0xFB, 0x00, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const v68, #64506", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Const(r, i) if r == 0x44 && i == 64506));
    }

    #[test]
    fn it_can_decode_const_high_16() {
        let raw_opcode: &[u8] = &[0x15, 0x44, 0xFF, 0xFF];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const/high16 v68, #-65536", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstHigh16(r, i) if r == 0x44 && i == -65536));
    }

    #[test]
    fn it_can_decode_const_wide_16() {
        let raw_opcode: &[u8] = &[0x16, 0x44, 0xFF, 0xFF];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-wide/16 v68, #-1", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstWide16(r, i) if r == 0x44 && i == -1));
    }

    #[test]
    fn it_can_decode_const_wide_32() {
        let raw_opcode: &[u8] = &[0x17, 0x44, 0xFF, 0xFF, 0x00, 0x11];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-wide/32 v68, #285278207", opcode.to_string());
        assert!(matches!(opcode, ByteCode::ConstWide32(r, i) if r == 0x44 && i == 285_278_207));
    }

    #[test]
    fn it_can_decode_const_wide() {
        let raw_opcode: &[u8] = &[0x18, 0x01, 0x44, 0xFF, 0xFF, 0x00, 0x44, 0xFF, 0xFF, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-wide v1, #72056786600853316", opcode.to_string());
        assert!(
            matches!(opcode, ByteCode::ConstWide(r, i) if r == 1 && i == 72_056_786_600_853_316)
        );
    }

    #[test]
    fn it_can_decode_const_wide_high16() {
        let raw_opcode: &[u8] = &[0x19, 0x01, 0xFF, 0xFF];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "const-wide/high16 v1, #-281474976710656",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::ConstWideHigh16(r, i) if r == 1 && i == -281_474_976_710_656));
    }

    #[test]
    fn it_can_decode_const_string() {
        let raw_opcode: &[u8] = &[0x1A, 0x01, 0xFF, 0xFF];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-string v1, string@65535", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::ConstString(r, i) if r == 1 && i == 65535));
    }

    #[test]
    fn it_can_decode_const_string_jumbo() {
        let raw_opcode: &[u8] = &[0x1B, 0x01, 0xFF, 0xFF, 0x00, 0x10];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "const-string/jumbo v1, string@268500991",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::ConstStringJumbo(r, i) if r == 1 && i == 268_500_991));
    }

    #[test]
    fn it_can_decode_const_class() {
        let raw_opcode: &[u8] = &[0x1C, 0x01, 0x11, 0x11];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("const-class v1, class@4369", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::ConstClass(r, i) if r == 1 && i == 4369));
    }

    #[test]
    fn it_can_decode_monitor_enter() {
        let raw_opcode: &[u8] = &[0x1D, 0x01];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("monitor-enter v1", opcode.to_string());
        assert!(matches!(opcode, ByteCode::MonitorEnter(r) if r == 1));
    }

    #[test]
    fn it_can_decode_monitor_exit() {
        let raw_opcode: &[u8] = &[0x1E, 0x9];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("monitor-exit v9", opcode.to_string());
        assert!(matches!(opcode, ByteCode::MonitorExit(r) if r == 9));
    }

    #[test]
    fn it_can_decode_check_cast() {
        let raw_opcode: &[u8] = &[0x1F, 0x01, 0x11, 0x11];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("check-cast v1, type@4369", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::CheckCast(r, i) if r == 1 && i == 4369));
    }

    #[test]
    fn it_can_decode_instance_of() {
        let raw_opcode: &[u8] = &[0x20, 0xA2, 0x11, 0x11];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("instance-of v2, v10, type@4369", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::InstanceOf(d, s, i) if d == 2 && s == 10 && i == 4369));
    }

    #[test]
    fn it_can_decode_array_length() {
        let raw_opcode: &[u8] = &[0x21, 0x2A];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("array-length v10, v2", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::ArrayLength(d, s) if d == 10 && s == 2));
    }

    #[test]
    fn it_can_decode_new_instance() {
        let raw_opcode: &[u8] = &[0x22, 0x00, 0x20, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("new-instance v0, type@32", opcode.to_string());
        assert!(matches!(opcode, ByteCode::NewInstance(d, reference) if d == 0 && reference == 32));
    }

    #[test]
    fn it_can_decode_new_array() {
        let raw_opcode: &[u8] = &[0x23, 0xA9, 0x20, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("new-array v9, v10, type@32", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::NewArray(d, s, reference) if d == 9 && s == 10 && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array() {
        let raw_opcode: &[u8] = &[0x24, 0x04, 0x20, 0x00, 0x12, 0x34];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("filled-new-array {}, type@32", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::FilledNewArray(
                ref registers,
                reference
            ) if registers.is_empty() && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_three_elements() {
        let raw_opcode: &[u8] = &[0x24, 0x35, 0x20, 0x00, 0x21, 0x43];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("filled-new-array {v1, v2, v3}, type@32", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::FilledNewArray(
                ref registers,
                reference
            ) if registers.as_ref() == [1, 2, 3] && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_five_elements() {
        let raw_opcode: &[u8] = &[0x24, 0x55, 0x20, 0x00, 0x21, 0x43];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "filled-new-array {v1, v2, v3, v4, v5}, type@32",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::FilledNewArray(
                ref registers,
                reference
            ) if registers.as_ref() == [1, 2, 3, 4, 5] && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_more_than_five_elements() {
        let raw_opcode: &[u8] = &[0x24, 0x85, 0x20, 0x00, 0x21, 0x43];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "filled-new-array {v1, v2, v3, v4, v5}, type@32",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::FilledNewArray(
                ref registers,
                reference
            ) if registers.as_ref() == [1, 2, 3, 4, 5] && reference == 32));
    }

    #[test]
    fn it_can_decode_filled_new_array_range() {
        let raw_opcode: &[u8] = &[0x25, 0x03, 0x22, 0x22, 0x01, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "filled-new-array/range {v1, v2, v3}, type@8738",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::FilledNewArrayRange(
                start,
                amount,
                reference
            ) if start == 1 && amount == 2 && reference == 8738));
    }

    #[test]
    fn it_can_decode_fill_array_data() {
        let raw_opcode: &[u8] = &[0x26, 0x12, 0x11, 0x22, 0x33, 0xFF];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("fill-array-data v18, -13426159", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::FillArrayData(reg, offset) if reg == 18 && offset == -13_426_159));
    }

    #[test]
    fn it_can_decode_throw() {
        let raw_opcode: &[u8] = &[0x27, 0x12];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("throw v18", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Throw(reg) if reg == 18));
    }

    #[test]
    fn it_can_decode_goto() {
        let raw_opcode: &[u8] = &[0x28, 0x03];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("goto 3", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Goto(offset) if offset == 3));
    }

    #[test]
    fn it_can_decode_goto16() {
        let raw_opcode: &[u8] = &[0x29, 0x00, 0x03, 0x04];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("goto/16 1027", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Goto16(offset) if offset == 1027));
    }

    #[test]
    fn it_can_decode_goto32() {
        let raw_opcode: &[u8] = &[0x2A, 0x00, 0x03, 0x04, 0x05, 0x06];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("goto/32 100992003", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Goto32(offset) if offset == 100_992_003));
    }

    #[test]
    fn it_can_decode_packed_switch() {
        let raw_opcode: &[u8] = &[0x2B, 0x04, 0x03, 0x04, 0x05, 0x06];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("packed-switch v4, 100992003", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::PackedSwitch(reg, offset) if reg == 4 && offset == 100_992_003));
    }

    #[test]
    fn it_can_decode_sparse_switch() {
        let raw_opcode: &[u8] = &[0x2C, 0x04, 0x03, 0x04, 0x05, 0x06];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("sparse-switch v4, 100992003", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::SparseSwitch(reg, offset) if reg == 4 && offset == 100_992_003));
    }

    #[test]
    fn it_can_decode_cmp_little_than_float() {
        let raw_opcode: &[u8] = &[0x2D, 0x04, 0x03, 0x02];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("cmpl-float v4, v3, v2", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::Compare(_, dest, op1, op2) if dest == 4 && op1 == 3 && op2 == 2));
    }

    #[test]
    fn it_can_decode_if_ne() {
        let raw_opcode: &[u8] = &[0x33, 0x24, 0x03, 0x02];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("if-ne v4, v2, 515", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::If(_, dest, op1, offset) if dest == 4 && op1 == 2 && offset == 515));
    }

    #[test]
    fn it_can_decode_if0_ge() {
        let raw_opcode: &[u8] = &[0x3B, 0x04, 0x03, 0x02];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("if-gez v4, 515", opcode.to_string());
        assert!(matches!(
            opcode,
             ByteCode::If0(_, dest, offset) if dest == 4 && offset == 515));
    }

    #[test]
    fn it_can_decode_array_operation() {
        let raw_opcode: &[u8] = &[0x4D, 0x04, 0x03, 0x02];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("aput-object v4, v3, v2", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::Array(_, dest, op1, op2) if dest == 4 && op1 == 3 && op2 == 2));
    }

    #[test]
    fn it_can_decode_instance_operation() {
        let raw_opcode: &[u8] = &[0x55, 0x34, 0x03, 0x02];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("iget-boolean v4, v3, field@515", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::Instance(
                _,
                dest,
                op1,
                reference
            ) if dest == 4 && op1 == 3 && reference == 515));
    }

    #[test]
    fn it_can_decode_static_operation() {
        let raw_opcode: &[u8] = &[0x6d, 0x04, 0x03, 0x02];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("sput-short v4, field@515", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::Static(_, dest, reference) if dest == 4 && reference == 515));
    }

    #[test]
    fn it_can_decode_invoke_operation() {
        let raw_opcode: &[u8] = &[0x6f, 0x00, 0x00, 0x01, 0x01, 0x23];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("invoke-super {}, method@256", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::Invoke(
                _,
               ref registers,
               reference
           ) if registers.is_empty() && reference == 256));
    }

    #[test]
    fn it_can_decode_invoke_range_operation() {
        let raw_opcode: &[u8] = &[0x78, 0x09, 0x00, 0x01, 0x00, 0x02];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "invoke-interface/range {v512, v513, v514, v515, v516, v517, v518, v519}, method@256",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::InvokeRange(
                _,
                first_reg,
                amount,
                reference
            ) if first_reg == 512 && amount == 8 && reference == 256));
    }

    #[test]
    fn it_can_decode_unary_operation() {
        let raw_opcode: &[u8] = &[0x84, 0x83];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("long-to-int v3, v8", opcode.to_string());
        assert!(matches!(opcode, ByteCode::Unary(_, dest, src) if dest == 3 &&  src == 8));
    }

    #[test]
    fn it_can_decode_binary_operation() {
        let raw_opcode: &[u8] = &[0xa0, 0x0f, 0x20, 0x13];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("and-long v15, v32, v19", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::Binary(_, dest, src1, src2) if dest == 15 &&  src1 == 32 && src2 == 19));
    }

    #[test]
    fn it_can_decode_binary_2addr_operation() {
        let raw_opcode: &[u8] = &[0xb9, 0x2f];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("shr-int/2addr v15, v2", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::Binary2Addr(_, dest_src, src) if dest_src == 15 &&  src == 2));
    }

    #[test]
    fn it_can_decode_binary_literal_16_operation() {
        let raw_opcode: &[u8] = &[0xd4, 0x2f, 0xFF, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("rem-int/lit16 v15, v2, #255", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::BinaryLit16(_, dest, src, lit) if dest == 15 &&  src == 2 && lit == 255));
    }

    #[test]
    fn it_can_decode_binary_literal_16_rsub_operation() {
        let raw_opcode: &[u8] = &[0xd1, 0x2f, 0xFF, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("rsub-int v15, v2, #255", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::BinaryLit16(_, dest, src, lit) if dest == 15 &&  src == 2 && lit == 255));
    }

    #[test]
    fn it_can_decode_binary_literal_8_operation() {
        let raw_opcode: &[u8] = &[0xe2, 0x10, 0x43, 0x01];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!("ushr-int/lit8 v16, v67, #1", opcode.to_string());
        assert!(matches!(
            opcode,
            ByteCode::BinaryLit8(_, dest, src, lit) if dest == 16 &&  src == 67 && lit == 1));
    }

    #[test]
    fn it_can_decode_invoke_polymorphic() {
        let raw_opcode: &[u8] = &[0xfa, 0x50, 0x00, 0x01, 0x21, 0x43, 0x10, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "invoke-polymorphic {v1, v2, v3, v4, v0}, method@256 proto@16",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::InvokePolymorphic(ref registers, method, proto
        ) if method == 256 && proto == 16 && registers.as_ref() == [1, 2, 3, 4, 0]));
    }

    #[test]
    fn it_can_decode_invoke_polymorphic_range() {
        let raw_opcode: &[u8] = &[0xfb, 0x04, 0x10, 0x00, 0x01, 0x00, 0x01, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "invoke-polymorphic/range {v1, v2, v3}, method@16 proto@1",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::InvokePolymorphicRange(start, amount, method, proto
        ) if method == 16 && proto == 1 && start == 1 && amount == 3));
    }

    #[test]
    fn it_can_decode_invoke_custom() {
        let raw_opcode: &[u8] = &[0xfc, 0x50, 0x00, 0x01, 0x21, 0x43];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "invoke-custom {v1, v2, v3, v4, v0}, call_site@256",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::InvokeCustom(ref registers, call_site
        ) if call_site == 256 && registers.as_ref() == [1, 2, 3, 4, 0]));
    }

    #[test]
    fn it_can_decode_invoke_custom_range() {
        let raw_opcode: &[u8] = &[0xfd, 0x04, 0x10, 0x00, 0x01, 0x00];
        let mut d: ByteCodeDecoder<_, LittleEndian> = ByteCodeDecoder::new(raw_opcode);

        let opcode = d.nth(0).unwrap();

        assert_eq!(
            "invoke-custom/range {v1, v2, v3}, call_site@16",
            opcode.to_string()
        );
        assert!(matches!(
            opcode,
            ByteCode::InvokeCustomRange(first, amount, call_site
        ) if first == 1 && amount == 3 && call_site == 16));
    }
}
