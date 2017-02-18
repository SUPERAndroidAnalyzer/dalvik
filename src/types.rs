use std::fmt;
use std::io::{Read, BufRead};

use byteorder::{LittleEndian, ByteOrder, ReadBytesExt};

use error::*;
use offset_map::{OffsetMap, OffsetType};

/// Data structure representing the `proto_id_item` type.
#[derive(Debug, Clone)]
pub struct PrototypeIdData {
    shorty_index: u32,
    return_type_index: u32,
    parameters_offset: Option<u32>,
}

impl PrototypeIdData {
    /// Creates a new `PrototypeIdData` from a reader.
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R) -> Result<PrototypeIdData> {
        let shorty_index = reader.read_u32::<E>()
            .chain_err(|| "could not read the sorty_index field")?;
        let return_type_index = reader.read_u32::<E>()
            .chain_err(|| "could not read the return_type_index field")?;
        let parameters_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the parameters_offset field")?;
        Ok(PrototypeIdData {
            shorty_index: shorty_index,
            return_type_index: return_type_index,
            parameters_offset: if parameters_offset != 0 {
                Some(parameters_offset)
            } else {
                None
            },
        })
    }

    /// Gets the shorty index.
    ///
    /// Gets the index into the `string_ids` list for the short-form descriptor string of this
    /// prototype. The string must conform to the syntax for `ShortyDescriptor`, and must
    /// correspond to the return type and parameters of this item.
    pub fn get_shorty_index(&self) -> usize {
        self.shorty_index as usize
    }

    /// Gets the return type index.
    ///
    /// Gets the index into the `type_ids` list for the return type of this prototype.
    pub fn get_return_type_index(&self) -> usize {
        self.return_type_index as usize
    }

    /// Gets the parameter list offset, if the prototype has parameters.
    ///
    /// Gets the offset from the start of the file to the list of parameter types for this
    /// prototype, or `None` if this prototype has no parameters. This offset, should be in the
    /// data section, and the `data` there should be in the format specified by `type_list`.
    /// Additionally, there should be no reference to the type `void` in the list.
    pub fn get_parameters_offset(&self) -> Option<u32> {
        self.parameters_offset
    }
}

/// Structure representing the `field_id_item` type.
#[derive(Debug, Clone)]
pub struct FieldIdData {
    class_index: u16,
    type_index: u16,
    name_index: u32,
}

impl FieldIdData {
    /// Creates a new `FieldIdData` from a reader.
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R) -> Result<FieldIdData> {
        let class_index = reader.read_u16::<E>()
            .chain_err(|| "could not read the class_index field")?;
        let type_index = reader.read_u16::<E>()
            .chain_err(|| "could not read the type_index field")?;
        let name_index = reader.read_u32::<E>()
            .chain_err(|| "could not read the name_index field")?;
        Ok(FieldIdData {
            class_index: class_index,
            type_index: type_index,
            name_index: name_index,
        })
    }

    /// Gets the index of the class of the field.
    ///
    /// Gets the index into the `type_ids` list for the definer of this field. This must be a class
    /// type, and not an array or primitive type.
    pub fn get_class_index(&self) -> usize {
        self.class_index as usize
    }

    /// Gets the index of the type of the class.
    ///
    /// Gets the index into the `type_ids` list for the type of this field.
    pub fn get_type_index(&self) -> usize {
        self.type_index as usize
    }

    /// Gets the index to the name of this field.
    ///
    /// Gets the index into the `string_ids` list for the name of this field. The string must
    /// conform to the syntax for `MemberName`.
    pub fn get_name_index(&self) -> usize {
        self.name_index as usize
    }
}

/// Structure representing the `method_id_item` type.
#[derive(Debug, Clone)]
pub struct MethodIdData {
    class_index: u16,
    prototype_index: u16,
    name_index: u32,
}

impl MethodIdData {
    /// Creates a new `MethodIdData` from a reader.
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R) -> Result<MethodIdData> {
        let class_index = reader.read_u16::<E>()
            .chain_err(|| "could not read the class_index field")?;
        let prototype_index = reader.read_u16::<E>()
            .chain_err(|| "could not read the prototype_index field")?;
        let name_index = reader.read_u32::<E>()
            .chain_err(|| "could not read the name_index field")?;
        Ok(MethodIdData {
            class_index: class_index,
            prototype_index: prototype_index,
            name_index: name_index,
        })
    }

    /// Gets the index of the class of the field.
    ///
    /// Gets the index into the `type_ids` list for the definer of this method. This must be a
    /// class or array type, and not a primitive type.
    pub fn get_class_index(&self) -> usize {
        self.class_index as usize
    }

    /// Gets the index of the prototype of the class.
    ///
    /// Gets the index into the `prototype_ids` list for the prototype of this method.
    pub fn get_prototype_index(&self) -> usize {
        self.prototype_index as usize
    }

    /// Gets the index to the name of this field.
    ///
    /// Gets the index into the `string_ids` list for the name of this field. The string must
    /// conform to the syntax for `MemberName`.
    pub fn get_name_index(&self) -> usize {
        self.name_index as usize
    }
}

const NO_INDEX: u32 = 0xFFFFFFFF;

bitflags! {
    pub flags AccessFlags: u32 {
        const ACC_PUBLIC = 0x1,
        const ACC_PRIVATE = 0x2,
        const ACC_PROTECTED = 0x4,
        const ACC_STATIC = 0x8,
        const ACC_FINAL = 0x10,
        const ACC_SYNCHRONIZED = 0x20,
        const ACC_VOLATILE = 0x40,
        const ACC_BRIDGE = 0x40,
        const ACC_TRANSIENT = 0x80,
        const ACC_VARARGS = 0x80,
        const ACC_NATIVE = 0x100,
        const ACC_INTERFACE = 0x200,
        const ACC_ABSTRACT = 0x400,
        const ACC_STRICT = 0x800,
        const ACC_SYNTHETIC = 0x1000,
        const ACC_ANNOTATION = 0x2000,
        const ACC_ENUM = 0x4000,
        const ACC_CONSTRUCTOR = 0x10000,
        const ACC_DECLARED_SYNCHRONIZED = 0x20000,
    }
}

#[derive(Debug, Clone)]
pub struct ClassDefData {
    class_id: u32,
    access_flags: AccessFlags,
    superclass_id: Option<u32>,
    interfaces_offset: Option<u32>,
    source_file_id: Option<u32>,
    annotations_offset: Option<u32>,
    class_data_offset: Option<u32>,
    static_values_offset: Option<u32>,
}

impl ClassDefData {
    /// Creates a new `ClassDefData` from a reader.
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R) -> Result<ClassDefData> {
        let class_id = reader.read_u32::<E>().chain_err(|| "could not read the class_id field")?;
        let access_flags = reader.read_u32::<E>()
            .chain_err(|| "could not read the access_flags field")?;
        let superclass_id = reader.read_u32::<E>()
            .chain_err(|| "could not read the superclass_id field")?;
        let interfaces_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the interfaces_offset field")?;
        let source_file_id = reader.read_u32::<E>()
            .chain_err(|| "could not read the source_file_id field")?;
        let annotations_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the annotations_offset field")?;
        let class_data_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the class_data_offset field")?;
        let static_values_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the static_values_offset field")?;

        #[inline]
        fn some_if(value: u32, condition: bool) -> Option<u32> {
            if condition { Some(value) } else { None }
        }

        Ok(ClassDefData {
            class_id: class_id,
            access_flags: AccessFlags::from_bits(access_flags)
                .ok_or_else(|| Error::from(ErrorKind::InvalidAccessFlags(access_flags)))?,
            superclass_id: some_if(superclass_id, superclass_id != NO_INDEX),
            interfaces_offset: some_if(interfaces_offset, interfaces_offset != 0),
            source_file_id: some_if(source_file_id, source_file_id != NO_INDEX),
            annotations_offset: some_if(annotations_offset, annotations_offset != 0),
            class_data_offset: some_if(class_data_offset, class_data_offset != 0),
            static_values_offset: some_if(static_values_offset, static_values_offset != 0),
        })
    }

    /// Gets the class ID (index in the *Type IDs* list) of the class definition.
    pub fn get_class_id(&self) -> usize {
        self.class_id as usize
    }

    /// Gets the access flags of the class definition.
    pub fn get_access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    /// Gets the class ID (index in the *Type IDs* list) of the superclass, if it exists.
    pub fn get_superclass_id(&self) -> Option<usize> {
        match self.superclass_id {
            Some(i) => Some(i as usize),
            None => None,
        }
    }

    /// Gets the offset of the list of interfaces implemented by the class, if it has any.
    pub fn get_interfaces_offset(&self) -> Option<u32> {
        self.interfaces_offset
    }

    /// Gets the index in the *String IDs* list of the file name where most of the class was, if it
    /// exists.
    pub fn get_source_file_id(&self) -> Option<usize> {
        match self.source_file_id {
            Some(i) => Some(i as usize),
            None => None,
        }
    }

    /// Gets the offset of the annotations of the class, if it has any.
    pub fn get_annotations_offset(&self) -> Option<u32> {
        self.annotations_offset
    }

    /// Gets the offset for the data of the class, if it has any.
    pub fn get_class_data_offset(&self) -> Option<u32> {
        self.class_data_offset
    }

    /// Gets the offset for the static values of the class, if it has any.
    pub fn get_static_values_offset(&self) -> Option<u32> {
        self.static_values_offset
    }
}

pub const VISIBILITY_BUILD: u8 = 0x00;
pub const VISIBILITY_RUNTIME: u8 = 0x01;
pub const VISIBILITY_SYSTEM: u8 = 0x02;

/// Annotation visibility
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Build,
    Runtime,
    System,
}

impl Visibility {
    fn from_u8(byte: u8) -> Result<Visibility> {
        match byte {
            VISIBILITY_BUILD => Ok(Visibility::Build),
            VISIBILITY_RUNTIME => Ok(Visibility::Runtime),
            VISIBILITY_SYSTEM => Ok(Visibility::System),
            b => Err(ErrorKind::InvalidVisibility(b).into()),
        }
    }
}

const VALUE_BYTE: u8 = 0x00;
const VALUE_SHORT: u8 = 0x02;
const VALUE_CHAR: u8 = 0x03;
const VALUE_INT: u8 = 0x04;
const VALUE_LONG: u8 = 0x06;
const VALUE_FLOAT: u8 = 0x10;
const VALUE_DOUBLE: u8 = 0x11;
const VALUE_STRING: u8 = 0x17;
const VALUE_TYPE: u8 = 0x18;
const VALUE_FIELD: u8 = 0x19;
const VALUE_METHOD: u8 = 0x1a;
const VALUE_ENUM: u8 = 0x1b;
const VALUE_ARRAY: u8 = 0x1c;
const VALUE_ANNOTATION: u8 = 0x1d;
const VALUE_NULL: u8 = 0x1e;
const VALUE_BOOLEAN: u8 = 0x1f;

/// Value.
#[derive(Debug, Clone)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Char(u16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(u32),
    Type(u32),
    Field(u32),
    Method(u32),
    Enum(u32),
    Array(Array),
    Annotation(Annotation),
    Null,
    Boolean(bool),
}

impl Value {
    fn from_reader<R: Read>(reader: &mut R) -> Result<(Value, u32)> {
        let mut value_type = [0u8];
        reader.read_exact(&mut value_type).chain_err(|| "could not read the value_type field")?;
        let arg = value_type[0] >> 5;
        let value_type = value_type[0] & 0b00011111;

        fn read_u32<R: Read>(reader: &mut R, arg: u8) -> Result<(u32, u32)> {
            match arg {
                0 => Ok((reader.read_u8()? as u32, 2)),
                1 => Ok((reader.read_u16::<LittleEndian>()? as u32, 3)),
                2 => {
                    let mut bytes = [0u8; 3];
                    reader.read_exact(&mut bytes)?;
                    // Reading in little endian
                    Ok((bytes[0] as u32 | (bytes[1] as u32) << 8 | (bytes[2] as u32) << 16, 4))

                }
                3 => Ok((reader.read_u32::<LittleEndian>()?, 5)),
                a => {
                    Err(ErrorKind::InvalidValue(format!("invalid arg ({}) for u32 value", a))
                        .into())
                }
            }
        }

        match value_type {
            VALUE_BYTE => {
                if arg == 0 {
                    Ok((Value::Byte(reader.read_i8().chain_err(|| "could not read Byte")?), 2))
                } else {
                    Err(ErrorKind::InvalidValue(format!("invalid arg ({}) for Byte value", arg))
                        .into())
                }
            }
            VALUE_SHORT => {
                match arg {
                    0 => {
                        Ok((Value::Short(reader.read_i8().chain_err(|| "could not read Short")? as
                                         i16),
                            2))
                    }
                    1 => {
                        Ok((Value::Short(reader.read_i16::<LittleEndian>()
                                .chain_err(|| "could not read Short")?),
                            3))
                    }
                    a => {
                        Err(ErrorKind::InvalidValue(format!("invalid arg ({}) for Short value", a))
                            .into())
                    }
                }
            }
            VALUE_CHAR => {
                match arg {
                    0 => {
                        Ok((Value::Char(reader.read_u8().chain_err(|| "could not read Char")? as
                                        u16),
                            2))
                    }
                    1 => {
                        Ok((Value::Char(reader.read_u16::<LittleEndian>()
                                .chain_err(|| "could not read Char")?),
                            3))
                    }
                    a => {
                        Err(ErrorKind::InvalidValue(format!("invalid arg ({}) for Char value", a))
                            .into())
                    }
                }
            }
            VALUE_INT => {
                match arg {
                    0 => {
                        Ok((Value::Int(reader.read_i8().chain_err(|| "could not read Int")? as
                                       i32),
                            2))
                    }
                    1 => {
                        Ok((Value::Int(reader.read_i16::<LittleEndian>()
                                .chain_err(|| "could not read Int")? as
                                       i32),
                            3))
                    }
                    2 => {
                        let mut bytes = [0u8; 3];
                        reader.read_exact(&mut bytes).chain_err(|| "could not read Int")?;
                        // Reading in little endian
                        Ok((Value::Int(bytes[0] as i32 | (bytes[1] as i32) << 8 |
                                       (bytes[2] as i8 as i32) << 16),
                            4))

                    }
                    3 => {
                        Ok((Value::Int(reader.read_i32::<LittleEndian>()
                                .chain_err(|| "could not read Int")?),
                            5))
                    }
                    a => {
                        Err(ErrorKind::InvalidValue(format!("invalid arg ({}) for Int value", a))
                            .into())
                    }
                }
            }
            VALUE_LONG => {
                match arg {
                    0 => Ok((Value::Long(reader.read_i8()? as i64), 2)),
                    1 => {
                        Ok((Value::Long(reader.read_i16::<LittleEndian>()
                                .chain_err(|| "could not read Long")? as
                                        i64),
                            3))
                    }
                    2 => {
                        let mut bytes = [0u8; 3];
                        reader.read_exact(&mut bytes).chain_err(|| "could not read Long")?;
                        // Reading in little endian
                        Ok((Value::Long(bytes[0] as i64 | (bytes[1] as i64) << 8 |
                                        (bytes[2] as i8 as i64) << 16),
                            4))

                    }
                    3 => {
                        Ok((Value::Long(reader.read_i32::<LittleEndian>()
                                .chain_err(|| "could not read Long")? as
                                        i64),
                            5))
                    }
                    4 => {
                        let mut bytes = [0u8; 5];
                        reader.read_exact(&mut bytes).chain_err(|| "could not read Long")?;
                        // Reading in little endian
                        Ok((Value::Long(bytes[0] as i64 | (bytes[1] as i64) << 8 |
                                        (bytes[2] as i64) << 16 |
                                        (bytes[3] as i64) << 24 |
                                        (bytes[4] as i8 as i64) << 32),
                            6))

                    }
                    5 => {
                        let mut bytes = [0u8; 6];
                        reader.read_exact(&mut bytes).chain_err(|| "could not read Long")?;
                        // Reading in little endian
                        Ok((Value::Long(bytes[0] as i64 | (bytes[1] as i64) << 8 |
                                        (bytes[2] as i64) << 16 |
                                        (bytes[3] as i64) << 24 |
                                        (bytes[4] as i64) << 32 |
                                        (bytes[5] as i8 as i64) << 40),
                            7))

                    }
                    6 => {
                        let mut bytes = [0u8; 7];
                        reader.read_exact(&mut bytes).chain_err(|| "could not read Long")?;
                        // Reading in little endian
                        Ok((Value::Long(bytes[0] as i64 | (bytes[1] as i64) << 8 |
                                        (bytes[2] as i64) << 16 |
                                        (bytes[3] as i64) << 24 |
                                        (bytes[4] as i64) << 32 |
                                        (bytes[5] as i64) << 40 |
                                        (bytes[6] as i8 as i64) << 48),
                            8))

                    }
                    7 => {
                        Ok((Value::Long(reader.read_i64::<LittleEndian>()
                                .chain_err(|| "could not read Long")?),
                            9))
                    }
                    _ => unreachable!(),
                }
            }
            VALUE_FLOAT => {
                match arg {
                    c @ 0...3 => {
                        let mut bytes = [0u8; 4];
                        reader.read_exact(&mut bytes[..c as usize + 1])
                            .chain_err(|| "could not read Float")?;
                        Ok((Value::Float(LittleEndian::read_f32(&bytes)), c as u32 + 2))
                    }
                    a => {
                        Err(ErrorKind::InvalidValue(format!("invalid arg ({}) for Float value", a))
                            .into())
                    }
                }
            }
            VALUE_DOUBLE => {
                match arg {
                    c @ 0...7 => {
                        let mut bytes = [0u8; 8];
                        reader.read_exact(&mut bytes[..c as usize + 1])
                            .chain_err(|| "could not read Double")?;
                        Ok((Value::Double(LittleEndian::read_f64(&bytes)), c as u32 + 2))
                    }
                    _ => unreachable!(),
                }
            }
            VALUE_STRING => {
                let (string_index, read) =
                    read_u32(reader, arg).chain_err(|| "could not read String index")?;
                Ok((Value::String(string_index), read))
            }
            VALUE_TYPE => {
                let (type_index, read) =
                    read_u32(reader, arg).chain_err(|| "could not read Type index")?;
                Ok((Value::Type(type_index), read))
            }
            VALUE_FIELD => {
                let (field_index, read) =
                    read_u32(reader, arg).chain_err(|| "could not read Field index")?;
                Ok((Value::Field(field_index), read))
            }
            VALUE_METHOD => {
                let (method_index, read) =
                    read_u32(reader, arg).chain_err(|| "could not read Method index")?;
                Ok((Value::Method(method_index), read))
            }
            VALUE_ENUM => {
                let (enum_index, read) =
                    read_u32(reader, arg).chain_err(|| "could not read Enum index")?;
                Ok((Value::Enum(enum_index), read))
            }
            VALUE_ARRAY => {
                let (array, read) =
                    Array::from_reader(reader).chain_err(|| "could not read Array")?;
                Ok((Value::Array(array), read + 1))
            }
            VALUE_ANNOTATION => {
                let (annotation, read) =
                    Annotation::from_reader(reader).chain_err(|| "could not read Annotation")?;
                Ok((Value::Annotation(annotation), read + 1))
            }
            VALUE_NULL => Ok((Value::Null, 1)),
            VALUE_BOOLEAN => {
                match arg {
                    0 => Ok((Value::Boolean(false), 1)),
                    1 => Ok((Value::Boolean(true), 1)),
                    _ => {
                        Err(ErrorKind::InvalidValue(format!("invalid arg ({}) for Boolean value",
                                                            arg))
                            .into())
                    }
                }
            }
            v => Err(ErrorKind::InvalidValue(format!("invalid value type {:#04x}", v)).into()),
        }
    }
}

/// Array
#[derive(Debug, Clone)]
pub struct Array {
    inner: Vec<Value>,
}

impl Array {
    /// Creates an array from a reader.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<(Array, u32)> {
        let (size, mut read) = read_uleb128(reader).chain_err(|| "could not read array size")?;
        let mut array = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let (value, value_read) =
                Value::from_reader(reader).chain_err(|| "could not read value")?;
            read += value_read;
            array.push(value);
        }
        Ok((Array { inner: array }, read))
    }
}

/// Annotation element.
#[derive(Debug, Clone)]
pub struct AnnotationElement {
    name: u32,
    value: Value,
}

impl AnnotationElement {
    /// Gets the index of the name string.
    pub fn get_name_index(&self) -> u32 {
        self.name
    }

    /// Gets the value of the annotation element.
    pub fn get_value(&self) -> &Value {
        &self.value
    }
}

/// Annotation.
#[derive(Debug, Clone)]
pub struct Annotation {
    type_id: u32,
    elements: Vec<AnnotationElement>,
}

impl Annotation {
    /// Creates an annotation from a reader.
    fn from_reader<R: Read>(reader: &mut R) -> Result<(Annotation, u32)> {
        let (type_id, mut read) = read_uleb128(reader).chain_err(|| "could not read type ID")?;
        let (size, size_read) = read_uleb128(reader).chain_err(|| "could not read size")?;
        read += size_read;
        let mut elements = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let (name_id, name_read) =
                read_uleb128(reader).chain_err(|| "could not read element's name_id")?;
            let (value, value_read) =
                Value::from_reader(reader).chain_err(|| "could not read element's value")?;
            read += name_read + value_read;
            elements.push(AnnotationElement {
                name: name_id,
                value: value,
            });
        }
        Ok((Annotation {
                type_id: type_id,
                elements: elements,
            },
            read))
    }

    /// Gets the index of the type of the annotation.
    pub fn get_type_index(&self) -> u32 {
        self.type_id
    }

    /// Gets the elements of the annotation.
    pub fn get_elements(&self) -> &[AnnotationElement] {
        &self.elements
    }
}

/// Annotation item
#[derive(Debug, Clone)]
pub struct AnnotationItem {
    visibility: Visibility,
    annotation: Annotation,
}

impl AnnotationItem {
    /// Creates a new annotation item from a reader.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<(AnnotationItem, u32)> {
        let mut visibility = [0u8];
        reader.read_exact(&mut visibility).chain_err(|| "could not read visibility")?;
        let visibility = Visibility::from_u8(visibility[0])?;
        let (annotation, read) =
            Annotation::from_reader(reader).chain_err(|| "could not read annotation")?;
        Ok((AnnotationItem {
                visibility: visibility,
                annotation: annotation,
            },
            read + 1))
    }

    /// Gets the visibility of the annotation item.
    pub fn get_visibility(&self) -> Visibility {
        self.visibility
    }

    /// Gets the annotation of the annotation item.
    pub fn get_annotation(&self) -> &Annotation {
        &self.annotation
    }
}

pub struct FieldAnnotations {
    field_id: u32,
    offset: u32,
}

impl FieldAnnotations {
    /// Gets the index of the field with annotations in the *Field IDs* list.
    pub fn get_field_index(&self) -> usize {
        self.field_id as usize
    }

    /// Gets the offset of the annotations of the field.
    pub fn get_annotations_offset(&self) -> u32 {
        self.offset
    }
}

pub struct MethodAnnotations {
    method_id: u32,
    offset: u32,
}

impl MethodAnnotations {
    /// Gets the index of the method with annotations in the *Method IDs* list.
    pub fn get_method_index(&self) -> usize {
        self.method_id as usize
    }

    /// Gets the offset of the annotations of the method.
    pub fn get_annotations_offset(&self) -> u32 {
        self.offset
    }
}

pub struct ParameterAnnotations {
    method_id: u32,
    offset: u32,
}

impl ParameterAnnotations {
    /// Gets the index of the method with annotations in the *Method IDs* list.
    pub fn get_method_index(&self) -> usize {
        self.method_id as usize
    }

    /// Gets the offset of the annotations of the method.
    pub fn get_annotations_offset(&self) -> u32 {
        self.offset
    }
}

pub struct AnnotationsDirectory {
    class_annotations_offset: Option<u32>,
    field_annotations: Vec<FieldAnnotations>,
    method_annotations: Vec<MethodAnnotations>,
    parameter_annotations: Vec<ParameterAnnotations>,
}

impl AnnotationsDirectory {
    /// Creates a new annotations directory from a reader.
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R,
                                              offset_map: &mut OffsetMap)
                                              -> Result<AnnotationsDirectory> {
        let class_annotations_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read class annotations offset")?;
        offset_map.insert(class_annotations_offset, OffsetType::AnnotationSet);
        let field_annotations_size =
            reader.read_u32::<E>().chain_err(|| "could not read field annotations size")? as usize;
        let method_annotations_size =
            reader.read_u32::<E>().chain_err(|| "could not read method annotations size")? as usize;
        let parameter_annotations_size = reader.read_u32::<E>()
            .chain_err(|| "could not read parameter annotations size")? as
                                         usize;

        let mut field_annotations = Vec::with_capacity(field_annotations_size);
        for _ in 0..field_annotations_size {
            let field_id = reader.read_u32::<E>()
                .chain_err(|| "could not read field ID for field annotation")?;
            let offset = reader.read_u32::<E>()
                .chain_err(|| "could not read field annotation offset")?;
            field_annotations.push(FieldAnnotations {
                field_id: field_id,
                offset: offset,
            });
        }
        let mut method_annotations = Vec::with_capacity(method_annotations_size);
        for _ in 0..method_annotations_size {
            let method_id = reader.read_u32::<E>()
                .chain_err(|| "could not read method ID for method annotation")?;
            let offset = reader.read_u32::<E>()
                .chain_err(|| "could not read method annotation offset")?;
            method_annotations.push(MethodAnnotations {
                method_id: method_id,
                offset: offset,
            });
        }
        let mut parameter_annotations = Vec::with_capacity(parameter_annotations_size);
        for _ in 0..parameter_annotations_size {
            let method_id = reader.read_u32::<E>()
                .chain_err(|| "could not read method ID for parameter annotation")?;
            let offset = reader.read_u32::<E>().chain_err(|| "could not read annotation offset")?;
            parameter_annotations.push(ParameterAnnotations {
                method_id: method_id,
                offset: offset,
            });
        }
        Ok(AnnotationsDirectory {
            class_annotations_offset: if class_annotations_offset != 0 {
                Some(class_annotations_offset)
            } else {
                None
            },
            field_annotations: field_annotations,
            method_annotations: method_annotations,
            parameter_annotations: parameter_annotations,
        })
    }

    /// Gets the class annotations offset, if they exist.
    pub fn get_class_annotations_offset(&self) -> Option<u32> {
        self.class_annotations_offset
    }

    /// Gets the field annotations.
    pub fn get_field_annotations(&self) -> &Vec<FieldAnnotations> {
        &self.field_annotations
    }

    /// Gets the method annotations.
    pub fn get_method_annotations(&self) -> &Vec<MethodAnnotations> {
        &self.method_annotations
    }

    /// Gets the parameter annotations.
    pub fn get_parameter_annotations(&self) -> &Vec<ParameterAnnotations> {
        &self.parameter_annotations
    }
}

const TYPE_HEADER_ITEM: u16 = 0x0000;
const TYPE_STRING_ID_ITEM: u16 = 0x0001;
const TYPE_TYPE_ID_ITEM: u16 = 0x0002;
const TYPE_PROTO_ID_ITEM: u16 = 0x0003;
const TYPE_FIELD_ID_ITEM: u16 = 0x0004;
const TYPE_METHOD_ID_ITEM: u16 = 0x0005;
const TYPE_CLASS_DEF_ITEM: u16 = 0x0006;
const TYPE_MAP_LIST: u16 = 0x1000;
const TYPE_TYPE_LIST: u16 = 0x1001;
const TYPE_ANNOTATION_SET_REF_LIST: u16 = 0x1002;
const TYPE_ANNOTATION_SET_ITEM: u16 = 0x1003;
const TYPE_CLASS_DATA_ITEM: u16 = 0x2000;
const TYPE_CODE_ITEM: u16 = 0x2001;
const TYPE_STRING_DATA_ITEM: u16 = 0x2002;
const TYPE_DEBUG_INFO_ITEM: u16 = 0x2003;
const TYPE_ANNOTATION_ITEM: u16 = 0x2004;
const TYPE_ENCODED_ARRAY_ITEM: u16 = 0x2005;
const TYPE_ANNOTATIONS_DIRECTORY_ITEM: u16 = 0x2006;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItemType {
    Header,
    StringId,
    TypeId,
    ProtoId,
    FieldId,
    MethodId,
    ClassDef,
    Map,
    TypeList,
    AnnotationSetList,
    AnnotationSet,
    ClassData,
    Code,
    StringData,
    DebugInfo,
    Annotation,
    EncodedArray,
    AnnotationsDirectory,
}

impl ItemType {
    fn from_u16(value: u16) -> Result<ItemType> {
        match value {
            TYPE_HEADER_ITEM => Ok(ItemType::Header),
            TYPE_STRING_ID_ITEM => Ok(ItemType::StringId),
            TYPE_TYPE_ID_ITEM => Ok(ItemType::TypeId),
            TYPE_PROTO_ID_ITEM => Ok(ItemType::ProtoId),
            TYPE_FIELD_ID_ITEM => Ok(ItemType::FieldId),
            TYPE_METHOD_ID_ITEM => Ok(ItemType::MethodId),
            TYPE_CLASS_DEF_ITEM => Ok(ItemType::ClassDef),
            TYPE_MAP_LIST => Ok(ItemType::Map),
            TYPE_TYPE_LIST => Ok(ItemType::TypeList),
            TYPE_ANNOTATION_SET_REF_LIST => Ok(ItemType::AnnotationSetList),
            TYPE_ANNOTATION_SET_ITEM => Ok(ItemType::AnnotationSet),
            TYPE_CLASS_DATA_ITEM => Ok(ItemType::ClassData),
            TYPE_CODE_ITEM => Ok(ItemType::Code),
            TYPE_STRING_DATA_ITEM => Ok(ItemType::StringData),
            TYPE_DEBUG_INFO_ITEM => Ok(ItemType::DebugInfo),
            TYPE_ANNOTATION_ITEM => Ok(ItemType::Annotation),
            TYPE_ENCODED_ARRAY_ITEM => Ok(ItemType::EncodedArray),
            TYPE_ANNOTATIONS_DIRECTORY_ITEM => Ok(ItemType::AnnotationsDirectory),
            v => Err(ErrorKind::InvalidItemType(v).into()),
        }
    }
}

impl From<ItemType> for u16 {
    fn from(item_type: ItemType) -> u16 {
        match item_type {
            ItemType::Header => TYPE_HEADER_ITEM,
            ItemType::StringId => TYPE_STRING_ID_ITEM,
            ItemType::TypeId => TYPE_TYPE_ID_ITEM,
            ItemType::ProtoId => TYPE_PROTO_ID_ITEM,
            ItemType::FieldId => TYPE_FIELD_ID_ITEM,
            ItemType::MethodId => TYPE_METHOD_ID_ITEM,
            ItemType::ClassDef => TYPE_CLASS_DEF_ITEM,
            ItemType::Map => TYPE_MAP_LIST,
            ItemType::TypeList => TYPE_TYPE_LIST,
            ItemType::AnnotationSetList => TYPE_ANNOTATION_SET_REF_LIST,
            ItemType::AnnotationSet => TYPE_ANNOTATION_SET_ITEM,
            ItemType::ClassData => TYPE_CLASS_DATA_ITEM,
            ItemType::Code => TYPE_CODE_ITEM,
            ItemType::StringData => TYPE_STRING_DATA_ITEM,
            ItemType::DebugInfo => TYPE_DEBUG_INFO_ITEM,
            ItemType::Annotation => TYPE_ANNOTATION_ITEM,
            ItemType::EncodedArray => TYPE_ENCODED_ARRAY_ITEM,
            ItemType::AnnotationsDirectory => TYPE_ANNOTATIONS_DIRECTORY_ITEM,
        }
    }
}

pub struct MapItem {
    item_type: ItemType,
    size: u32,
    offset: u32,
}

impl MapItem {
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R) -> Result<MapItem> {
        let item_type = reader.read_u16::<E>().chain_err(|| "could not read item type")?;
        reader.read_exact(&mut [0u8; 2]).chain_err(|| "could not read item type padding")?;
        let size = reader.read_u32::<E>().chain_err(|| "could not read size")?;
        let offset = reader.read_u32::<E>().chain_err(|| "could not read offset")?;
        Ok(MapItem {
            item_type: ItemType::from_u16(item_type)?,
            size: size,
            offset: offset,
        })
    }

    pub fn get_item_type(&self) -> ItemType {
        self.item_type
    }

    pub fn get_num_items(&self) -> usize {
        self.size as usize
    }

    pub fn get_offset(&self) -> u32 {
        self.offset
    }
}

impl fmt::Debug for MapItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "MapItem {{ item_type: {:?} ({:#06x}), size: {} items, offset: {:#010x} }}",
               self.item_type,
               u16::from(self.item_type),
               self.size,
               self.offset)
    }
}

struct Field {
    field_id: u32,
    access_flags: AccessFlags,
}

struct Method {
    method_id: u32,
    access_flags: AccessFlags,
    code_offset: Option<u32>,
}

/// Class data structure.
pub struct ClassData {
    static_fields: Vec<Field>,
    instance_fields: Vec<Field>,
    direct_methods: Vec<Method>,
    virtual_methods: Vec<Method>,
}

impl ClassData {
    pub fn from_reader<R: Read>(reader: &mut R,
                                offset_map: &mut OffsetMap)
                                -> Result<(ClassData, u32)> {
        let (static_fields_size, mut read) =
            read_uleb128(reader).chain_err(|| "could not read static_fields_size field")?;
        let (instance_fields_size, read_ifs) =
            read_uleb128(reader).chain_err(|| "could not read instance_fields_size field")?;
        let (direct_methods_size, read_dms) =
            read_uleb128(reader).chain_err(|| "could not read direct_methods_size field")?;
        let (virtual_methods_size, read_vms) =
            read_uleb128(reader).chain_err(|| "could not read virtual_methods_size field")?;
        read += read_ifs + read_dms + read_vms;

        let mut static_fields = Vec::with_capacity(static_fields_size as usize);
        read += ClassData::read_fields(reader, static_fields_size, &mut static_fields)
            .chain_err(|| "could not read class static fields")?;

        let mut instance_fields = Vec::with_capacity(instance_fields_size as usize);
        read += ClassData::read_fields(reader, instance_fields_size, &mut instance_fields)
            .chain_err(|| "could not read class instance fields")?;

        let mut direct_methods = Vec::with_capacity(direct_methods_size as usize);
        read += ClassData::read_methods(reader,
                                        direct_methods_size,
                                        &mut direct_methods,
                                        offset_map)
            .chain_err(|| "could not read class direct methods")?;

        let mut virtual_methods = Vec::with_capacity(virtual_methods_size as usize);
        read += ClassData::read_methods(reader,
                                        virtual_methods_size,
                                        &mut virtual_methods,
                                        offset_map)
            .chain_err(|| "could not read class virtual methods")?;

        Ok((ClassData {
                static_fields: static_fields,
                instance_fields: instance_fields,
                direct_methods: direct_methods,
                virtual_methods: virtual_methods,
            },
            read))
    }

    fn read_fields<R: Read>(reader: &mut R,
                            field_count: u32,
                            field_vec: &mut Vec<Field>)
                            -> Result<u32> {
        let mut read = 0;
        if field_count > 0 {
            // First field's ID is given directly.
            let (field_id, read_fi) = read_uleb128(reader).chain_err(|| "could not read field ID")?;
            let (access_flags, read_af) =
                read_uleb128(reader).chain_err(|| "could not read field access flags")?;
            read += read_fi + read_af;

            field_vec.push(Field {
                field_id: field_id,
                access_flags: AccessFlags::from_bits(access_flags)
                    .ok_or_else(|| Error::from(ErrorKind::InvalidAccessFlags(access_flags)))?,
            });

            let mut last_field_id = field_id;
            for _ in 1..field_count {
                let (field_id_diff, read_fi) =
                    read_uleb128(reader).chain_err(|| "could not read field ID")?;
                let (access_flags_u32, read_af) =
                    read_uleb128(reader).chain_err(|| "could not read field access flags")?;
                read += read_fi + read_af;

                // Field IDs other than the first one are given by difference.
                last_field_id += field_id_diff;

                field_vec.push(Field {
                        field_id: last_field_id,
                        access_flags: AccessFlags::from_bits(access_flags).ok_or_else(|| {
                                Error::from(ErrorKind::InvalidAccessFlags(access_flags))
                            })?,
                    });
            }
        }
        Ok(read)
    }

    fn read_methods<R: Read>(reader: &mut R,
                             method_count: u32,
                             method_vec: &mut Vec<Method>,
                             offset_map: &mut OffsetMap)
                             -> Result<u32> {
        let mut read = 0;
        if method_count > 0 {
            // First method's ID is given directly.
            let (method_id, read_mi) =
                read_uleb128(reader).chain_err(|| "could not read method ID")?;
            let (access_flags, read_af) =
                read_uleb128(reader).chain_err(|| "could not read method access flags")?;
            let (code_offset, read_co) =
                read_uleb128(reader).chain_err(|| "could not read method code offset")?;

            let code_offset = if code_offset != 0 {
                offset_map.insert(code_offset, OffsetType::Code);
                Some(code_offset)
            } else {
                None
            };
            read += read_mi + read_af + read_co;

            method_vec.push(Method {
                method_id: method_id,
                access_flags: AccessFlags::from_bits(access_flags)
                    .ok_or_else(|| Error::from(ErrorKind::InvalidAccessFlags(access_flags)))?,
                code_offset: code_offset,
            });

            let mut last_method_id = method_id;
            for _ in 1..method_count {
                let (method_id_diff, read_mi) =
                    read_uleb128(reader).chain_err(|| "could not read method ID")?;
                let (access_flags, read_af) =
                    read_uleb128(reader).chain_err(|| "could not read method access flags")?;
                let (code_offset, read_co) =
                    read_uleb128(reader).chain_err(|| "could not read method code offset")?;

                let code_offset = if code_offset != 0 {
                    offset_map.insert(code_offset, OffsetType::Code);
                    Some(code_offset)
                } else {
                    None
                };
                read += read_mi + read_af + read_co;

                // Method IDs other than the first one are given by difference.
                last_method_id += method_id_diff;

                method_vec.push(Method {
                        method_id: last_method_id,
                        access_flags: AccessFlags::from_bits(access_flags).ok_or_else(|| {
                                Error::from(ErrorKind::InvalidAccessFlags(access_flags))
                            })?,
                        code_offset: code_offset,
                    });
            }
        }
        Ok(read)
    }
}

/// Dex string reader.
pub struct StringReader;

impl StringReader {
    /// Reads a string from a reader.
    pub fn read_string<R: BufRead>(reader: &mut R) -> Result<(String, u32)> {
        let (size, mut read) = read_uleb128(reader).chain_err(|| "could not read string size")?;
        let mut data = Vec::with_capacity(size as usize);
        if size > 0 {
            read += reader.read_until(0, &mut data)? as u32;
            data.pop();
        }

        let string = String::from_utf8(data).chain_err(|| "error decoding UTF-8 from string data")?;
        let char_count = string.chars().count();
        if char_count != size as usize {
            Err(ErrorKind::StringSizeMismatch(size, char_count).into())
        } else {
            Ok((string, read))
        }
    }
}

/// Debug information structure.
#[derive(Debug)]
pub struct DebugInfo {
    line_start: u32,
    parameter_names: Vec<u32>,
    bytecode: DebugBytecode,
}

impl DebugInfo {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<(DebugInfo, u32)> {
        let (line_start, mut read) =
            read_uleb128(reader).chain_err(|| "could not read line_start field")?;
        let (parameters_size, read_p) =
            read_uleb128(reader).chain_err(|| "could not read parameters_size field")?;
        read += read_p;

        let mut parameter_names = Vec::with_capacity(parameters_size as usize);
        for _ in 0..parameters_size {
            let (name_index, read_i) =
                read_uleb128p1(reader).chain_err(|| "could not read parameter name index")?;
            read += read_i;
            parameter_names.push(name_index);
        }

        let (bytecode, read_b) =
            DebugBytecode::from_reader(reader).chain_err(|| "could not read debug bytecode")?;
        read += read_b;

        Ok((DebugInfo {
                line_start: line_start,
                parameter_names: parameter_names,
                bytecode: bytecode,
            },
            read))
    }

    pub fn get_line_start(&self) -> u32 {
        self.line_start
    }

    pub fn get_parameter_names(&self) -> &[u32] {
        &self.parameter_names
    }
}

/// Debug bytecode.
#[derive(Debug)]
struct DebugBytecode {
    bytecode: Vec<DebugInstruction>,
}

impl DebugBytecode {
    /// Reads the debug bytecode from a reader.
    fn from_reader<R: Read>(reader: &mut R) -> Result<(DebugBytecode, u32)> {
        let mut bytecode = Vec::new();
        let mut read = 0;
        loop {
            let (instruction, read_i) =
                DebugInstruction::from_reader(reader).chain_err(|| "could not read instruction")?;
            read += read_i;
            bytecode.push(instruction);

            if instruction == DebugInstruction::EndSequence {
                break;
            }
        }
        Ok((DebugBytecode { bytecode: bytecode }, read))
    }
}

/// Debug state machine instruction.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DebugInstruction {
    EndSequence,
    AdvancePc { addr_diff: u32 },
    AdvanceLine { line_diff: i32 },
    StartLocal {
        register_num: u32,
        name_id: u32,
        type_id: u32,
    },
    StartLocalExtended {
        register_num: u32,
        name_id: u32,
        type_id: u32,
        sig_id: u32,
    },
    EndLocal { register_num: u32 },
    RestartLocal { register_num: u32 },
    SetPrologueEnd,
    SetEpilogueBegin,
    SetFile { name_id: u32 },
    SpecialOpcode { opcode: u8 },
}

impl DebugInstruction {
    fn from_reader<R: Read>(reader: &mut R) -> Result<(DebugInstruction, u32)> {
        let mut opcode = [0u8];
        reader.read_exact(&mut opcode);
        let mut read = 1;
        let instruction = match opcode[0] {
            0x00u8 => DebugInstruction::EndSequence,
            0x01u8 => {
                let (addr_diff, read_ad) = read_uleb128(reader).chain_err(||{
                        "could not read `addr_diff` for the DBG_ADVANCE_PC instruction"
                    })?;
                read += read_ad;
                DebugInstruction::AdvancePc { addr_diff: addr_diff }
            }
            0x02u8 => {
                let (line_diff, read_ld) = read_sleb128(reader).chain_err(||{
                        "could not read `line_diff` for the DBG_ADVANCE_LINE instruction"
                    })?;
                read += read_ld;
                DebugInstruction::AdvanceLine { line_diff: line_diff }
            }
            0x03u8 => {
                let (register_num, read_rn) = read_uleb128(reader).chain_err(||{
                        "could not read `register_num` for the DBG_START_LOCAL instruction"
                    })?;
                let (name_id, read_ni) = read_uleb128p1(reader).chain_err(||{
                            "could not read `name_id` for the DBG_START_LOCAL instruction"
                        })?;
                let (type_id, read_ti) = read_uleb128p1(reader).chain_err(||{
                                "could not read `type_id` for the DBG_START_LOCAL instruction"
                            })?;
                read += read_rn + read_ni + read_ti;

                DebugInstruction::StartLocal {
                    register_num: register_num,
                    name_id: name_id,
                    type_id: type_id,
                }
            }
            0x04u8 => {
                let (register_num, read_rn) = read_uleb128(reader).chain_err(||{
                        "could not read `register_num` for the DBG_START_LOCAL_EXTENDED instruction"
                    })?;
                let (name_id, read_ni) = read_uleb128p1(reader).chain_err(||{
                        "could not read `name_id` for the DBG_START_LOCAL_EXTENDED instruction"
                    })?;
                let (type_id, read_ti) = read_uleb128p1(reader).chain_err(||{
                        "could not read `type_id` for the DBG_START_LOCAL_EXTENDED instruction"
                    })?;
                let (sig_id, read_si) = read_uleb128p1(reader).chain_err(||{
                        "could not read `sig_id` for the DBG_START_LOCAL_EXTENDED instruction"
                    })?;
                read += read_rn + read_ni + read_ti + read_si;

                DebugInstruction::StartLocalExtended {
                    register_num: register_num,
                    name_id: name_id,
                    type_id: type_id,
                    sig_id: sig_id,
                }
            }
            0x05u8 => {
                let (register_num, read_rn) = read_uleb128(reader).chain_err(|| {
                        "could not read `register_num` for the DBG_END_LOCAL instruction"
                    })?;
                read += read_rn;
                DebugInstruction::EndLocal { register_num: register_num }
            }
            0x06u8 => {
                let (register_num, read_rn) = read_uleb128(reader).chain_err(|| {
                        "could not read `register_num` for the DBG_RESTART_LOCAL instruction"
                    })?;
                read += read_rn;
                DebugInstruction::RestartLocal { register_num: register_num }
            }
            0x07u8 => DebugInstruction::SetPrologueEnd,
            0x08u8 => DebugInstruction::SetEpilogueBegin,
            0x09u8 => {
                let (name_id, read_ni) = read_uleb128(reader).chain_err(|| {
                        "could not read `name_id` for the DBG_SET_FILE instruction"
                    })?;
                read += read_ni;
                DebugInstruction::SetFile { name_id: name_id }
            }
            oc @ 0x0au8...0xffu8 => DebugInstruction::SpecialOpcode { opcode: oc },
            _ => unreachable!(),
        };

        Ok((instruction, read))
    }
}

/// Code Item structure
#[derive(Debug)]
pub struct CodeItem {
    registers_size: u16,
    ins_size: u16,
    outs_size: u16,
    debug_info_offset: u32,
    insns: Vec<u16>,
    tries: Vec<TryItem>,
    handlers: Vec<CatchHandler>,
}

impl CodeItem {
    /// Reads a code item from the given reader.
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R,
                                              offset_map: &mut OffsetMap)
                                              -> Result<(CodeItem, u32)> {
        let registers_size = reader.read_u16::<E>().chain_err(|| "could not read registers size")?;
        let ins_size = reader.read_u16::<E>().chain_err(|| "could not read incoming words size")?;
        let outs_size = reader.read_u16::<E>().chain_err(|| "could not read outgoing words size")?;
        let tries_size = reader.read_u16::<E>().chain_err(|| "could not read tries size")?;
        let debug_info_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read debug information offset")?;
        offset_map.insert(debug_info_offset, OffsetType::DebugInfo);
        let insns_size = reader.read_u32::<E>()
            .chain_err(|| "could not read the size of the bytecode array")?;

        let mut insns = Vec::with_capacity(insns_size as usize);
        for _ in 0..insns_size {
            insns.push(reader.read_u16::<E>().chain_err(|| "could not read bytecode")?);
        }

        let mut read = 16 + 2 * insns_size;

        if tries_size != 0 && (insns_size & 0b1 != 0) {
            let mut padding = [0u8; 2];
            reader.read_exact(&mut padding).chain_err(|| "could not read padding before tries")?;
            read += 2;
        }

        let mut tries = Vec::with_capacity(tries_size as usize);
        for _ in 0..tries_size {
            tries.push(TryItem::from_reader::<_, E>(reader).chain_err(||{
                    "could not read try item"
                })?);
        }

        read += tries_size as u32 * 8;

        let mut handlers = Vec::new();
        if tries_size > 0 {
            let (handlers_size, read_hs) =
                read_uleb128(reader).chain_err(|| "could not read catch handlers size")?;
            read += read_hs;

            handlers.reserve_exact(handlers_size as usize);
            for _ in 0..handlers_size {
                let (handler, read_h) =
                    CatchHandler::from_reader(reader).chain_err(|| "could not read catch handler")?;
                read += read_h;
                handlers.push(handler);
            }
        }

        Ok((CodeItem {
                registers_size: registers_size,
                ins_size: ins_size,
                outs_size: outs_size,
                debug_info_offset: debug_info_offset,
                insns: insns,
                tries: tries,
                handlers: handlers,
            },
            read))
    }
}

/// Try item structure.
#[derive(Debug)]
struct TryItem {
    start_address: u32,
    insn_count: u16,
    handler_offset: u16,
}

impl TryItem {
    /// Creates a try item structure from a reader.
    fn from_reader<R: Read, E: ByteOrder>(reader: &mut R) -> Result<TryItem> {
        let start_address = reader.read_u32::<E>().chain_err(|| "could not read start address")?;
        let insn_count = reader.read_u16::<E>().chain_err(|| "could not read instruction count")?;
        let handler_offset = reader.read_u16::<E>()
            .chain_err(|| "could not read catch handler offset")?;

        Ok(TryItem {
            start_address: start_address,
            insn_count: insn_count,
            handler_offset: handler_offset,
        })
    }
}

/// Struct representing a catch handler.
#[derive(Debug)]
struct CatchHandler {
    handlers: Vec<HandlerInfo>,
    catch_all_addr: Option<u32>,
}

impl CatchHandler {
    /// Reads a catch handler from a reader.
    fn from_reader<R: Read>(reader: &mut R) -> Result<(CatchHandler, u32)> {
        let (size, mut read) =
            read_sleb128(reader).chain_err(|| "could not read the catch handler size")?;

        let abs_size = size.abs() as usize;
        let mut handlers = Vec::with_capacity(abs_size);
        for _ in 0..abs_size {
            let (handler_info, read_hi) = HandlerInfo::from_reader(reader).chain_err(|| {
                    "could not read handler information"
                })?;
            handlers.push(handler_info);
            read += read_hi;
        }

        let catch_all_addr = if size < 1 {
            let (addr, read_ca) =
                read_uleb128(reader).chain_err(|| "could not read the catch all address")?;
            read += read_ca;
            Some(addr)
        } else {
            None
        };

        Ok((CatchHandler {
                handlers: handlers,
                catch_all_addr: catch_all_addr,
            },
            read))
    }
}

#[derive(Debug)]
struct HandlerInfo {
    type_id: u32,
    addr: u32,
}

impl HandlerInfo {
    fn from_reader<R: Read>(reader: &mut R) -> Result<(HandlerInfo, u32)> {
        let (type_id, read_t) = read_uleb128(reader).chain_err(|| "could not read type ID")?;
        let (addr, read_a) = read_uleb128(reader).chain_err(|| "could not read address")?;

        Ok((HandlerInfo {
                type_id: type_id,
                addr: addr,
            },
            read_t + read_a))
    }
}

/// Reads a uleb128 from a reader.
///
/// Returns the u32 represented by the uleb128 and the number of bytes read.
fn read_uleb128<R: Read>(reader: &mut R) -> Result<(u32, u32)> {
    let mut result = 0;
    let mut read = 0;
    for (i, byte) in reader.bytes().enumerate() {
        let byte = byte.chain_err(|| format!("could not read byte {}", i))?;
        let payload = (byte & 0b01111111) as u32;
        match i {
            0...4 => result |= payload << i * 7,
            _ => return Err(ErrorKind::InvalidLeb128.into()),
        }

        if byte & 0b10000000 == 0x00 {
            read = i + 1;
            break;
        }
    }
    Ok((result, read as u32))
}

/// Reads a uleb128p1 from a reader.
///
/// Returns the u32 represented by the uleb128p1 and the number of bytes read.
fn read_uleb128p1<R: Read>(reader: &mut R) -> Result<(u32, u32)> {
    let (uleb128, read) = read_uleb128(reader)?;
    Ok((uleb128.wrapping_sub(1), read))
}

/// Reads a sleb128 from a reader.
///
/// Returns the i32 represented by the sleb128 and the number of bytes read.
fn read_sleb128<R: Read>(reader: &mut R) -> Result<(i32, u32)> {
    let (uleb128, read) = read_uleb128(reader)?;
    let s_bits = read * 7;
    let mut sleb128 = uleb128 as i32;

    if (sleb128 & 1 << s_bits) != 0 {
        sleb128 |= -1 << s_bits;
    }

    Ok((sleb128, read))
}
