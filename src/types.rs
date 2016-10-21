use std::fmt;
use error::{Result, Error};
use std::io::Read;

use byteorder::{LittleEndian, ByteOrder, ReadBytesExt};

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
        let shorty_index = try!(reader.read_u32::<E>());
        let return_type_index = try!(reader.read_u32::<E>());
        let parameters_offset = try!(reader.read_u32::<E>());
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
        let class_index = try!(reader.read_u16::<E>());
        let type_index = try!(reader.read_u16::<E>());
        let name_index = try!(reader.read_u32::<E>());
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
        let class_index = try!(reader.read_u16::<E>());
        let prototype_index = try!(reader.read_u16::<E>());
        let name_index = try!(reader.read_u32::<E>());
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
        let class_id = try!(reader.read_u32::<E>());
        let access_flags = try!(reader.read_u32::<E>());
        let superclass_id = try!(reader.read_u32::<E>());
        let interfaces_offset = try!(reader.read_u32::<E>());
        let source_file_id = try!(reader.read_u32::<E>());
        let annotations_offset = try!(reader.read_u32::<E>());
        let class_data_offset = try!(reader.read_u32::<E>());
        let static_values_offset = try!(reader.read_u32::<E>());

        #[inline]
        fn some_if(value: u32, condition: bool) -> Option<u32> {
            if condition { Some(value) } else { None }
        }

        Ok(ClassDefData {
            class_id: class_id,
            access_flags: try!(AccessFlags::from_bits(access_flags)
                .ok_or(Error::invalid_access_flags(access_flags))),
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

const VISIBILITY_BUILD: u8 = 0x00;
const VISIBILITY_RUNTIME: u8 = 0x01;
const VISIBILITY_SYSTEM: u8 = 0x02;

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
            b => Err(Error::invalid_visibility(b)),
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
        try!(reader.read_exact(&mut value_type));
        let arg = value_type[0] >> 5;
        let value_type = value_type[0] & 0b00011111;

        fn read_u32<R: Read>(reader: &mut R, arg: u8) -> Result<(u32, u32)> {
            match arg {
                0 => Ok((try!(reader.read_u8()) as u32, 2)),
                1 => Ok((try!(reader.read_u16::<LittleEndian>()) as u32, 3)),
                2 => {
                    let mut bytes = [0u8; 3];
                    try!(reader.read_exact(&mut bytes));
                    // Reading in little endian
                    Ok((bytes[0] as u32 | (bytes[1] as u32) << 8 | (bytes[2] as u32) << 16, 4))

                }
                3 => Ok((try!(reader.read_u32::<LittleEndian>()), 5)),
                a => Err(Error::invalid_value(format!("invalid arg ({}) for u32 value", a))),
            }
        }

        match value_type {
            VALUE_BYTE => {
                if arg == 0 {
                    Ok((Value::Byte(try!(reader.read_i8())), 2))
                } else {
                    Err(Error::invalid_value(format!("invalid arg ({}) for Byte value", arg)))
                }
            }
            VALUE_SHORT => {
                match arg {
                    0 => Ok((Value::Short(try!(reader.read_i8()) as i16), 2)),
                    1 => Ok((Value::Short(try!(reader.read_i16::<LittleEndian>())), 3)),
                    a => Err(Error::invalid_value(format!("invalid arg ({}) for Short value", a))),
                }
            }
            VALUE_CHAR => {
                match arg {
                    0 => Ok((Value::Char(try!(reader.read_u8()) as u16), 2)),
                    1 => Ok((Value::Char(try!(reader.read_u16::<LittleEndian>())), 3)),
                    a => Err(Error::invalid_value(format!("invalid arg ({}) for Char value", a))),
                }
            }
            VALUE_INT => {
                match arg {
                    0 => Ok((Value::Int(try!(reader.read_i8()) as i32), 2)),
                    1 => Ok((Value::Int(try!(reader.read_i16::<LittleEndian>()) as i32), 3)),
                    2 => {
                        let mut bytes = [0u8; 3];
                        try!(reader.read_exact(&mut bytes));
                        // Reading in little endian
                        Ok((Value::Int(bytes[0] as i32 | (bytes[1] as i32) << 8 |
                                       (bytes[2] as i8 as i32) << 16),
                            4))

                    }
                    3 => Ok((Value::Int(try!(reader.read_i32::<LittleEndian>())), 5)),
                    a => Err(Error::invalid_value(format!("invalid arg ({}) for Int value", a))),
                }
            }
            VALUE_LONG => {
                match arg {
                    0 => Ok((Value::Long(try!(reader.read_i8()) as i64), 2)),
                    1 => Ok((Value::Long(try!(reader.read_i16::<LittleEndian>()) as i64), 3)),
                    2 => {
                        let mut bytes = [0u8; 3];
                        try!(reader.read_exact(&mut bytes));
                        // Reading in little endian
                        Ok((Value::Long(bytes[0] as i64 | (bytes[1] as i64) << 8 |
                                        (bytes[2] as i8 as i64) << 16),
                            4))

                    }
                    3 => Ok((Value::Long(try!(reader.read_i32::<LittleEndian>()) as i64), 5)),
                    4 => {
                        let mut bytes = [0u8; 5];
                        try!(reader.read_exact(&mut bytes));
                        // Reading in little endian
                        Ok((Value::Long(bytes[0] as i64 | (bytes[1] as i64) << 8 |
                                        (bytes[2] as i64) << 16 |
                                        (bytes[3] as i64) << 24 |
                                        (bytes[4] as i8 as i64) << 32),
                            6))

                    }
                    5 => {
                        let mut bytes = [0u8; 6];
                        try!(reader.read_exact(&mut bytes));
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
                        try!(reader.read_exact(&mut bytes));
                        // Reading in little endian
                        Ok((Value::Long(bytes[0] as i64 | (bytes[1] as i64) << 8 |
                                        (bytes[2] as i64) << 16 |
                                        (bytes[3] as i64) << 24 |
                                        (bytes[4] as i64) << 32 |
                                        (bytes[5] as i64) << 40 |
                                        (bytes[6] as i8 as i64) << 48),
                            8))

                    }
                    7 => Ok((Value::Long(try!(reader.read_i64::<LittleEndian>())), 9)),
                    _ => unreachable!(),
                }
            }
            VALUE_FLOAT => {
                match arg {
                    c @ 0...3 => {
                        let mut bytes = [0u8; 4];
                        try!(reader.read_exact(&mut bytes[..c as usize + 1]));
                        Ok((Value::Float(LittleEndian::read_f32(&bytes)), c as u32 + 2))
                    }
                    a => Err(Error::invalid_value(format!("invalid arg ({}) for Float value", a))),
                }
            }
            VALUE_DOUBLE => {
                match arg {
                    c @ 0...7 => {
                        let mut bytes = [0u8; 8];
                        try!(reader.read_exact(&mut bytes[..c as usize + 1]));
                        Ok((Value::Double(LittleEndian::read_f64(&bytes)), c as u32 + 2))
                    }
                    _ => unreachable!(),
                }
            }
            VALUE_STRING => {
                let (string_index, read) = try!(read_u32(reader, arg));
                Ok((Value::String(string_index), read))
            }
            VALUE_TYPE => {
                let (type_index, read) = try!(read_u32(reader, arg));
                Ok((Value::Type(type_index), read))
            }
            VALUE_FIELD => {
                let (field_index, read) = try!(read_u32(reader, arg));
                Ok((Value::Field(field_index), read))
            }
            VALUE_METHOD => {
                let (method_index, read) = try!(read_u32(reader, arg));
                Ok((Value::Method(method_index), read))
            }
            VALUE_ENUM => {
                let (enum_index, read) = try!(read_u32(reader, arg));
                Ok((Value::Enum(enum_index), read))
            }
            VALUE_ARRAY => {
                let (array, read) = try!(Array::from_reader(reader));
                Ok((Value::Array(array), read + 1))
            }
            VALUE_ANNOTATION => {
                let (annotation, read) = try!(Annotation::from_reader(reader));
                Ok((Value::Annotation(annotation), read + 1))
            }
            VALUE_NULL => Ok((Value::Null, 1)),
            VALUE_BOOLEAN => {
                match arg {
                    0 => Ok((Value::Boolean(false), 1)),
                    1 => Ok((Value::Boolean(true), 1)),
                    _ => {
                        Err(Error::invalid_value(format!("invalid arg ({}) for Boolean value",
                                                         arg)))
                    }
                }
            }
            v => Err(Error::invalid_value(format!("invalid value type {:#04x}", v))),
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
        let (size, mut read) = try!(read_uleb128(reader));
        let mut array = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let (value, value_read) = try!(Value::from_reader(reader));
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
        let (type_id, mut read) = try!(read_uleb128(reader));
        let (size, size_read) = try!(read_uleb128(reader));
        read += size_read;
        let mut elements = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let (name_id, name_read) = try!(read_uleb128(reader));
            let (value, value_read) = try!(Value::from_reader(reader));
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
        try!(reader.read_exact(&mut visibility));
        let visibility = try!(Visibility::from_u8(visibility[0]));
        let (annotation, read) = try!(Annotation::from_reader(reader));
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
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R) -> Result<AnnotationsDirectory> {
        let class_annotations_offset = try!(reader.read_u32::<E>());
        let field_annotations_size = try!(reader.read_u32::<E>()) as usize;
        let method_annotations_size = try!(reader.read_u32::<E>()) as usize;
        let parameter_annotations_size = try!(reader.read_u32::<E>()) as usize;

        let mut field_annotations = Vec::with_capacity(field_annotations_size);
        for _ in 0..field_annotations_size {
            let field_id = try!(reader.read_u32::<E>());
            let offset = try!(reader.read_u32::<E>());
            field_annotations.push(FieldAnnotations {
                field_id: field_id,
                offset: offset,
            });
        }
        let mut method_annotations = Vec::with_capacity(method_annotations_size);
        for _ in 0..method_annotations_size {
            let method_id = try!(reader.read_u32::<E>());
            let offset = try!(reader.read_u32::<E>());
            method_annotations.push(MethodAnnotations {
                method_id: method_id,
                offset: offset,
            });
        }
        let mut parameter_annotations = Vec::with_capacity(parameter_annotations_size);
        for _ in 0..parameter_annotations_size {
            let method_id = try!(reader.read_u32::<E>());
            let offset = try!(reader.read_u32::<E>());
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
            v => Err(Error::invalid_item_type(v)),
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
        let item_type = try!(reader.read_u16::<E>());
        try!(reader.read_exact(&mut [0u8; 2]));
        let size = try!(reader.read_u32::<E>());
        let offset = try!(reader.read_u32::<E>());
        Ok(MapItem {
            item_type: try!(ItemType::from_u16(item_type)),
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

/// Reads a uleb128 from a reader.
///
/// Returns the u32 represented by the uleb128 and the number of bytes read.
fn read_uleb128<R: Read>(reader: &mut R) -> Result<(u32, u32)> {
    let mut result = 0;
    let mut read = 0;
    for (i, byte) in reader.bytes().enumerate() {
        let byte = try!(byte);
        let payload = (byte & 0b01111111) as u32;
        match i {
            0 => result |= payload,
            1 => result |= payload << 7,
            2 => result |= payload << 14,
            3 => result |= payload << 21,
            4 => result |= payload << 28,
            _ => return Err(Error::invalid_uleb128()),
        }

        if byte & 0b10000000 == 0x00 {
            read = i + 1;
            break;
        }
    }
    Ok((result, read as u32))
}
