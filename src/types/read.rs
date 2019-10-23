//! Types used for reading Dex files.

use std::io::Read;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use failure::{Error, ResultExt};

use super::{
    AccessFlags, Annotation, AnnotationElement, Array, EncodedAnnotation, Value, Visibility,
};
use crate::{
    error,
    read::{sleb128, uleb128, uleb128p1},
};

/// Data structure representing the `proto_id_item` type.
#[derive(Debug, Copy, Clone)]
pub struct PrototypeIdData {
    shorty_index: u32,
    return_type_index: u32,
    parameters_offset: Option<u32>,
}

impl PrototypeIdData {
    /// Creates a new `PrototypeIdData` from a reader.
    pub fn from_reader<R, B>(reader: &mut R) -> Result<Self, Error>
    where
        R: ReadBytesExt,
        B: ByteOrder,
    {
        let shorty_index = reader
            .read_u32::<B>()
            .context("could not read the shorty_index field")?;
        let return_type_index = reader
            .read_u32::<B>()
            .context("could not read the return_type_index field")?;
        let parameters_offset = reader
            .read_u32::<B>()
            .context("could not read the parameters_offset field")?;
        Ok(Self {
            shorty_index,
            return_type_index,
            parameters_offset: if parameters_offset == 0 {
                None
            } else {
                Some(parameters_offset)
            },
        })
    }

    /// Gets the shorty index.
    ///
    /// Gets the index into the `string_ids` list for the short-form descriptor string of this
    /// prototype. The string must conform to the syntax for `ShortyDescriptor`, and must
    /// correspond to the return type and parameters of this item.
    pub fn shorty_index(&self) -> u32 {
        self.shorty_index
    }

    /// Gets the return type index.
    ///
    /// Gets the index into the `type_ids` list for the return type of this prototype.
    pub fn return_type_index(&self) -> u32 {
        self.return_type_index
    }

    /// Gets the parameter list offset, if the prototype has parameters.
    ///
    /// Gets the offset from the start of the file to the list of parameter types for this
    /// prototype, or `None` if this prototype has no parameters. This offset, should be in the
    /// data section, and the `data` there should be in the format specified by `type_list`.
    /// Additionally, there should be no reference to the type `void` in the list.
    pub fn parameters_offset(&self) -> Option<u32> {
        self.parameters_offset
    }
}

/// Structure representing the `field_id_item` type.
#[derive(Debug, Copy, Clone)]
pub struct FieldIdData {
    class_index: u16,
    type_index: u16,
    name_index: u32,
}

impl FieldIdData {
    /// Creates a new `FieldIdData` from a reader.
    pub fn from_reader<R, B>(reader: &mut R) -> Result<Self, Error>
    where
        R: ReadBytesExt,
        B: ByteOrder,
    {
        let class_index = reader
            .read_u16::<B>()
            .context("could not read the class_index field")?;
        let type_index = reader
            .read_u16::<B>()
            .context("could not read the type_index field")?;
        let name_index = reader
            .read_u32::<B>()
            .context("could not read the name_index field")?;
        Ok(Self {
            class_index,
            type_index,
            name_index,
        })
    }

    /// Gets the index of the class of the field.
    ///
    /// Gets the index into the `type_ids` list for the definer of this field. This must be a class
    /// type, and not an array or primitive type.
    pub fn class_index(self) -> usize {
        self.class_index as usize
    }

    /// Gets the index of the type of the class.
    ///
    /// Gets the index into the `type_ids` list for the type of this field.
    pub fn type_index(self) -> usize {
        self.type_index as usize
    }

    /// Gets the index to the name of this field.
    ///
    /// Gets the index into the `string_ids` list for the name of this field. The string must
    /// conform to the syntax for `MemberName`.
    pub fn name_index(self) -> usize {
        self.name_index as usize
    }
}

/// Structure representing the `method_id_item` type.
#[derive(Debug, Copy, Clone)]
pub struct MethodIdData {
    class_index: u16,
    prototype_index: u16,
    name_index: u32,
}

impl MethodIdData {
    /// Creates a new `MethodIdData` from a reader.
    pub fn from_reader<R, B>(reader: &mut R) -> Result<Self, Error>
    where
        R: ReadBytesExt,
        B: ByteOrder,
    {
        let class_index = reader
            .read_u16::<B>()
            .context("could not read the class_index field")?;
        let prototype_index = reader
            .read_u16::<B>()
            .context("could not read the prototype_index field")?;
        let name_index = reader
            .read_u32::<B>()
            .context("could not read the name_index field")?;
        Ok(Self {
            class_index,
            prototype_index,
            name_index,
        })
    }

    /// Gets the index of the class of the field.
    ///
    /// Gets the index into the `type_ids` list for the definer of this method. This must be a
    /// class or array type, and not a primitive type.
    pub fn class_index(self) -> usize {
        self.class_index as usize
    }

    /// Gets the index of the prototype of the class.
    ///
    /// Gets the index into the `prototype_ids` list for the prototype of this method.
    pub fn prototype_index(self) -> usize {
        self.prototype_index as usize
    }

    /// Gets the index to the name of this field.
    ///
    /// Gets the index into the `string_ids` list for the name of this field. The string must
    /// conform to the syntax for `MemberName`.
    pub fn name_index(self) -> usize {
        self.name_index as usize
    }
}

const NO_INDEX: u32 = 0xFFFF_FFFF;

/// Data of a class definition.
#[derive(Debug, Copy, Clone)]
pub struct ClassDefData {
    class_index: u32,
    access_flags: AccessFlags,
    superclass_index: Option<u32>,
    interfaces_offset: Option<u32>,
    source_file_index: Option<u32>,
    annotations_offset: Option<u32>,
    class_data_offset: Option<u32>,
    static_values_offset: Option<u32>,
}

impl ClassDefData {
    /// Creates a new `ClassDefData` from a reader.
    pub fn from_reader<R, B>(reader: &mut R) -> Result<Self, Error>
    where
        R: ReadBytesExt,
        B: ByteOrder,
    {
        /// Returns `Some(value)` if `condition` is `true` and `None` otherwise.
        #[inline]
        fn some_if(value: u32, condition: bool) -> Option<u32> {
            if condition {
                Some(value)
            } else {
                None
            }
        }

        let class_index = reader
            .read_u32::<B>()
            .context("could not read the class_index field")?;
        let access_flags = reader
            .read_u32::<B>()
            .context("could not read the access_flags field")?;
        let superclass_index = reader
            .read_u32::<B>()
            .context("could not read the superclass_id field")?;
        let interfaces_offset = reader
            .read_u32::<B>()
            .context("could not read the interfaces_offset field")?;
        let source_file_index = reader
            .read_u32::<B>()
            .context("could not read the source_file_id field")?;
        let annotations_offset = reader
            .read_u32::<B>()
            .context("could not read the annotations_offset field")?;
        let class_data_offset = reader
            .read_u32::<B>()
            .context("could not read the class_data_offset field")?;
        let static_values_offset = reader
            .read_u32::<B>()
            .context("could not read the static_values_offset field")?;

        Ok(Self {
            class_index,
            access_flags: AccessFlags::from_bits(access_flags)
                .ok_or_else(|| error::Parse::InvalidAccessFlags { access_flags })?,
            superclass_index: some_if(superclass_index, superclass_index != NO_INDEX),
            interfaces_offset: some_if(interfaces_offset, interfaces_offset != 0),
            source_file_index: some_if(source_file_index, source_file_index != NO_INDEX),
            annotations_offset: some_if(annotations_offset, annotations_offset != 0),
            class_data_offset: some_if(class_data_offset, class_data_offset != 0),
            static_values_offset: some_if(static_values_offset, static_values_offset != 0),
        })
    }

    /// Gets the index in the *Type IDs* list of the class.
    pub fn class_index(&self) -> u32 {
        self.class_index
    }

    /// Gets the access flags of the class definition.
    pub fn access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    /// Gets the index in the *Type IDs* list of the superclass of this class, if it exists.
    pub fn superclass_index(&self) -> Option<u32> {
        self.superclass_index
    }

    /// Gets the offset of the list of interfaces implemented by the class, if it has any.
    pub fn interfaces_offset(&self) -> Option<u32> {
        self.interfaces_offset
    }

    /// Gets the index in the *String IDs* list of the file name where most of the class was, if it
    /// exists.
    pub fn source_file_index(&self) -> Option<u32> {
        self.source_file_index
    }

    /// Gets the offset of the annotations of the class, if it has any.
    pub fn annotations_offset(&self) -> Option<u32> {
        self.annotations_offset
    }

    /// Gets the offset for the data of the class, if it has any.
    pub fn class_data_offset(&self) -> Option<u32> {
        self.class_data_offset
    }

    /// Gets the offset for the static values of the class, if it has any.
    pub fn static_values_offset(&self) -> Option<u32> {
        self.static_values_offset
    }
}

/// Build visibility.
const VISIBILITY_BUILD: u8 = 0x00;
/// Runtime visibility.
const VISIBILITY_RUNTIME: u8 = 0x01;
/// System visibility.
const VISIBILITY_SYSTEM: u8 = 0x02;

impl Visibility {
    // TODO change it for TryFrom once it becomes available.
    fn from_u8(byte: u8) -> Result<Self, error::Parse> {
        match byte {
            VISIBILITY_BUILD => Ok(Self::Build),
            VISIBILITY_RUNTIME => Ok(Self::Runtime),
            VISIBILITY_SYSTEM => Ok(Self::System),
            b => Err(error::Parse::InvalidVisibility { visibility: b }),
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
const VALUE_METHOD_TYPE: u8 = 0x15;
const VALUE_METHOD_HANDLE: u8 = 0x16;
const VALUE_STRING: u8 = 0x17;
const VALUE_TYPE: u8 = 0x18;
const VALUE_FIELD: u8 = 0x19;
const VALUE_METHOD: u8 = 0x1a;
const VALUE_ENUM: u8 = 0x1b;
const VALUE_ARRAY: u8 = 0x1c;
const VALUE_ANNOTATION: u8 = 0x1d;
const VALUE_NULL: u8 = 0x1e;
const VALUE_BOOLEAN: u8 = 0x1f;

impl Value {
    fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read + ReadBytesExt,
    {
        let mut value_type = [0_u8];
        reader
            .read_exact(&mut value_type)
            .context("could not read the value_type field")?;
        let arg = value_type[0] >> 5;
        let value_type = value_type[0] & 0b0001_1111;

        match value_type {
            VALUE_BYTE => {
                if arg == 0 {
                    Ok(Self::Byte(reader.read_i8().context("could not read Byte")?))
                } else {
                    Err(error::Parse::InvalidValue {
                        error: format!("invalid arg ({}) for Byte value", arg),
                    }
                    .into())
                }
            }
            VALUE_SHORT => match arg {
                0 => Ok(Self::Short(i16::from(
                    reader.read_i8().context("could not read Short")?,
                ))),
                1 => Ok(Self::Short(
                    reader
                        .read_i16::<LittleEndian>()
                        .context("could not read Short")?,
                )),
                a => Err(error::Parse::InvalidValue {
                    error: format!("invalid arg ({}) for Short value", a),
                }
                .into()),
            },
            VALUE_CHAR => match arg {
                0 => Ok(Self::Char(u16::from(
                    reader.read_u8().context("could not read Char")?,
                ))),
                1 => Ok(Self::Char(
                    reader
                        .read_u16::<LittleEndian>()
                        .context("could not read Char")?,
                )),
                a => Err(error::Parse::InvalidValue {
                    error: format!("invalid arg ({}) for Char value", a),
                }
                .into()),
            },
            VALUE_INT => {
                match arg {
                    0 => Ok(Self::Int(i32::from(
                        reader.read_i8().context("could not read Int")?,
                    ))),
                    1 => Ok(Self::Int(i32::from(
                        reader
                            .read_i16::<LittleEndian>()
                            .context("could not read Int")?,
                    ))),
                    2 => {
                        let mut bytes = [0_u8; 3];
                        reader
                            .read_exact(&mut bytes)
                            .context("could not read Int")?;
                        // Reading in little endian
                        Ok(Self::Int(
                            i32::from(bytes[0])
                                | i32::from(bytes[1]) << 8
                                | i32::from(bytes[2] as i8) << 16,
                        ))
                    }
                    3 => Ok(Self::Int(
                        reader
                            .read_i32::<LittleEndian>()
                            .context("could not read Int")?,
                    )),
                    a => Err(error::Parse::InvalidValue {
                        error: format!("invalid arg ({}) for Int value", a),
                    }
                    .into()),
                }
            }
            VALUE_LONG => {
                match arg {
                    0 => Ok(Self::Long(i64::from(reader.read_i8()?))),
                    1 => Ok(Self::Long(i64::from(
                        reader
                            .read_i16::<LittleEndian>()
                            .context("could not read Long")?,
                    ))),
                    2 => {
                        let mut bytes = [0_u8; 3];
                        reader
                            .read_exact(&mut bytes)
                            .context("could not read Long")?;
                        // Reading in little endian
                        Ok(Self::Long(
                            i64::from(bytes[0])
                                | i64::from(bytes[1]) << 8
                                | i64::from(bytes[2] as i8) << 16,
                        ))
                    }
                    3 => Ok(Self::Long(i64::from(
                        reader
                            .read_i32::<LittleEndian>()
                            .context("could not read Long")?,
                    ))),
                    4 => {
                        let mut bytes = [0_u8; 5];
                        reader
                            .read_exact(&mut bytes)
                            .context("could not read Long")?;
                        // Reading in little endian
                        Ok(Self::Long(
                            i64::from(bytes[0])
                                | i64::from(bytes[1]) << 8
                                | i64::from(bytes[2]) << 16
                                | i64::from(bytes[3]) << 24
                                | i64::from(bytes[4] as i8) << 32,
                        ))
                    }
                    5 => {
                        let mut bytes = [0_u8; 6];
                        reader
                            .read_exact(&mut bytes)
                            .context("could not read Long")?;
                        // Reading in little endian
                        Ok(Self::Long(
                            i64::from(bytes[0])
                                | i64::from(bytes[1]) << 8
                                | i64::from(bytes[2]) << 16
                                | i64::from(bytes[3]) << 24
                                | i64::from(bytes[4]) << 32
                                | i64::from(bytes[5] as i8) << 40,
                        ))
                    }
                    6 => {
                        let mut bytes = [0_u8; 7];
                        reader
                            .read_exact(&mut bytes)
                            .context("could not read Long")?;
                        // Reading in little endian
                        Ok(Self::Long(
                            i64::from(bytes[0])
                                | i64::from(bytes[1]) << 8
                                | i64::from(bytes[2]) << 16
                                | i64::from(bytes[3]) << 24
                                | i64::from(bytes[4]) << 32
                                | i64::from(bytes[5]) << 40
                                | i64::from(bytes[6] as i8) << 48,
                        ))
                    }
                    7 => Ok(Self::Long(
                        reader
                            .read_i64::<LittleEndian>()
                            .context("could not read Long")?,
                    )),
                    _ => unreachable!(),
                }
            }
            VALUE_FLOAT => match arg {
                c @ 0..=3 => {
                    let mut bytes = [0_u8; 4];
                    reader
                        .read_exact(&mut bytes[..=c as usize])
                        .context("could not read Float")?;
                    Ok(Self::Float(LittleEndian::read_f32(&bytes)))
                }
                a => Err(error::Parse::InvalidValue {
                    error: format!("invalid arg ({}) for Float value", a),
                }
                .into()),
            },
            VALUE_DOUBLE => match arg {
                c @ 0..=7 => {
                    let mut bytes = [0_u8; 8];
                    reader
                        .read_exact(&mut bytes[..=c as usize])
                        .context("could not read Double")?;
                    Ok(Self::Double(LittleEndian::read_f64(&bytes)))
                }
                _ => unreachable!(),
            },
            VALUE_METHOD_TYPE => unimplemented!(),
            VALUE_METHOD_HANDLE => unimplemented!(),
            VALUE_STRING => {
                let string_index =
                    Self::read_u32(reader, arg).context("could not read String index")?;
                Ok(Self::String(string_index))
            }
            VALUE_TYPE => {
                let type_index =
                    Self::read_u32(reader, arg).context("could not read Type index")?;
                Ok(Self::Type(type_index))
            }
            VALUE_FIELD => {
                let field_index =
                    Self::read_u32(reader, arg).context("could not read Field index")?;
                Ok(Self::Field(field_index))
            }
            VALUE_METHOD => {
                let method_index =
                    Self::read_u32(reader, arg).context("could not read Method index")?;
                Ok(Self::Method(method_index))
            }
            VALUE_ENUM => {
                let enum_index =
                    Self::read_u32(reader, arg).context("could not read Enum index")?;
                Ok(Self::Enum(enum_index))
            }
            VALUE_ARRAY => {
                let array = Array::from_reader(reader).context("could not read Array")?;
                Ok(Self::Array(array))
            }
            VALUE_ANNOTATION => {
                let annotation = EncodedAnnotation::from_reader(reader)
                    .context("could not read Annotation value")?;
                Ok(Self::Annotation(annotation))
            }
            VALUE_NULL => Ok(Self::Null),
            VALUE_BOOLEAN => match arg {
                0 => Ok(Self::Boolean(false)),
                1 => Ok(Self::Boolean(true)),
                _ => Err(error::Parse::InvalidValue {
                    error: format!("invalid arg ({}) for Boolean value", arg),
                }
                .into()),
            },
            v => Err(error::Parse::InvalidValue {
                error: format!("invalid value type {:#04x}", v),
            }
            .into()),
        }
    }

    fn read_u32<R>(reader: &mut R, arg: u8) -> Result<u32, Error>
    where
        R: Read + ReadBytesExt,
    {
        match arg {
            0 => Ok(u32::from(reader.read_u8()?)),
            1 => Ok(u32::from(reader.read_u16::<LittleEndian>()?)),
            2 => {
                let mut bytes = [0_u8; 3];
                reader.read_exact(&mut bytes)?;
                // Reading in little endian
                Ok(u32::from(bytes[0]) | u32::from(bytes[1]) << 8 | u32::from(bytes[2]) << 16)
            }
            3 => Ok(reader.read_u32::<LittleEndian>()?),
            a => Err(error::Parse::InvalidValue {
                error: format!("invalid arg ({}) for u32 value", a),
            }
            .into()),
        }
    }
}

impl Array {
    /// Creates an array from a reader.
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let (size, _) = uleb128(reader).context("could not read array size")?;
        let mut array = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let value = Value::from_reader(reader).context("could not read value")?;
            array.push(value);
        }
        Ok(Self {
            inner: array.into_boxed_slice(),
        })
    }
}

impl EncodedAnnotation {
    /// Creates an annotation from a reader.
    #[doc(hidden)]
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let (type_id, _) = uleb128(reader).context("could not read type ID")?;
        let (size, _) = uleb128(reader).context("could not read size")?;
        let mut elements = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let (name, _) = uleb128(reader).context("could not read element's name_id")?;
            let value = Value::from_reader(reader).context("could not read element's value")?;
            elements.push(AnnotationElement { name, value });
        }
        Ok(Self {
            type_id,
            elements: elements.into_boxed_slice(),
        })
    }
}

impl Annotation {
    /// Creates a new annotation item from a reader.
    #[doc(hidden)]
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let mut visibility = [0_u8];
        reader
            .read_exact(&mut visibility)
            .context("could not read visibility")?;
        let visibility = Visibility::from_u8(visibility[0])?;
        let annotation =
            EncodedAnnotation::from_reader(reader).context("could not read annotation")?;
        Ok(Self {
            visibility,
            annotation,
        })
    }
}

/// List of offsets to field annotations.
#[derive(Debug, Copy, Clone)]
pub struct FieldAnnotationsOffset {
    field_id: u32,
    offset: u32,
}

impl FieldAnnotationsOffset {
    /// Gets the index of the field with annotations in the *Field IDs* list.
    pub fn field_index(self) -> u32 {
        self.field_id
    }

    /// Gets the offset of the annotations of the field.
    pub fn offset(self) -> u32 {
        self.offset
    }
}

/// List of offsets to method annotations.
#[derive(Debug, Copy, Clone)]
pub struct MethodAnnotationsOffset {
    method_id: u32,
    offset: u32,
}

impl MethodAnnotationsOffset {
    /// Gets the index of the method with annotations in the *Method IDs* list.
    pub fn method_index(self) -> u32 {
        self.method_id
    }

    /// Gets the offset of the annotations of the method.
    pub fn offset(self) -> u32 {
        self.offset
    }
}

/// List of offset to parameter annotations.
#[derive(Debug, Copy, Clone)]
pub struct ParameterAnnotationsOffset {
    method_id: u32,
    offset: u32,
}

impl ParameterAnnotationsOffset {
    /// Gets the index of the method with annotations in the *Method IDs* list.
    pub fn method_index(self) -> u32 {
        self.method_id
    }

    /// Gets the offset of the annotations of the method.
    pub fn offset(self) -> u32 {
        self.offset
    }
}

/// Offsets of the annotations in the annotations directory.
#[derive(Debug)]
pub struct AnnotationsDirectoryOffsets {
    class_annotations_offset: Option<u32>,
    field_annotations: Box<[FieldAnnotationsOffset]>,
    method_annotations: Box<[MethodAnnotationsOffset]>,
    parameter_annotations: Box<[ParameterAnnotationsOffset]>,
}

impl AnnotationsDirectoryOffsets {
    /// Creates a new annotations directory from a reader.
    pub fn from_reader<R, B>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read + ReadBytesExt,
        B: ByteOrder,
    {
        let class_annotations_offset = reader
            .read_u32::<B>()
            .context("could not read class annotations offset")?;
        let field_annotations_size = reader
            .read_u32::<B>()
            .context("could not read field annotations size")?
            as usize;
        let method_annotations_size = reader
            .read_u32::<B>()
            .context("could not read method annotations size")?
            as usize;
        let parameter_annotations_size = reader
            .read_u32::<B>()
            .context("could not read parameter annotations size")?
            as usize;

        let mut field_annotations = Vec::with_capacity(field_annotations_size);
        for _ in 0..field_annotations_size {
            let field_id = reader
                .read_u32::<B>()
                .context("could not read field ID for field annotation")?;
            let offset = reader
                .read_u32::<B>()
                .context("could not read field annotation offset")?;
            field_annotations.push(FieldAnnotationsOffset { field_id, offset });
        }
        let mut method_annotations = Vec::with_capacity(method_annotations_size);
        for _ in 0..method_annotations_size {
            let method_id = reader
                .read_u32::<B>()
                .context("could not read method ID for method annotation")?;
            let offset = reader
                .read_u32::<B>()
                .context("could not read method annotation offset")?;
            method_annotations.push(MethodAnnotationsOffset { method_id, offset });
        }
        let mut parameter_annotations = Vec::with_capacity(parameter_annotations_size);
        for _ in 0..parameter_annotations_size {
            let method_id = reader
                .read_u32::<B>()
                .context("could not read method ID for parameter annotation")?;
            let offset = reader
                .read_u32::<B>()
                .context("could not read annotation offset")?;
            parameter_annotations.push(ParameterAnnotationsOffset { method_id, offset });
        }
        Ok(Self {
            class_annotations_offset: if class_annotations_offset == 0 {
                None
            } else {
                Some(class_annotations_offset)
            },
            field_annotations: field_annotations.into_boxed_slice(),
            method_annotations: method_annotations.into_boxed_slice(),
            parameter_annotations: parameter_annotations.into_boxed_slice(),
        })
    }

    /// Gets the class annotations offset, if they exist.
    pub fn class_annotations_offset(&self) -> Option<u32> {
        self.class_annotations_offset
    }

    /// Gets the field annotations.
    pub fn field_annotations(&self) -> &[FieldAnnotationsOffset] {
        &self.field_annotations
    }

    /// Gets the method annotations.
    pub fn method_annotations(&self) -> &[MethodAnnotationsOffset] {
        &self.method_annotations
    }

    /// Gets the parameter annotations.
    pub fn parameter_annotations(&self) -> &[ParameterAnnotationsOffset] {
        &self.parameter_annotations
    }
}

#[derive(Debug, Clone, Copy)]
struct Field {
    field_id: u32,
    access_flags: AccessFlags,
}

#[derive(Debug, Clone, Copy)]
struct Method {
    method_id: u32,
    access_flags: AccessFlags,
    code_offset: Option<u32>,
}

/// Class data structure.
#[derive(Debug)]
pub struct ClassData {
    static_fields: Vec<Field>,
    instance_fields: Vec<Field>,
    direct_methods: Vec<Method>,
    virtual_methods: Vec<Method>,
}

impl ClassData {
    /// Creates a new class data structure from a reader.
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let (static_fields_size, _) =
            uleb128(reader).context("could not read static_fields_size field")?;
        let (instance_fields_size, _) =
            uleb128(reader).context("could not read instance_fields_size field")?;
        let (direct_methods_size, _) =
            uleb128(reader).context("could not read direct_methods_size field")?;
        let (virtual_methods_size, _) =
            uleb128(reader).context("could not read virtual_methods_size field")?;

        let mut static_fields = Vec::with_capacity(static_fields_size as usize);
        Self::read_fields(reader, static_fields_size, &mut static_fields)
            .context("could not read class static fields")?;

        let mut instance_fields = Vec::with_capacity(instance_fields_size as usize);
        Self::read_fields(reader, instance_fields_size, &mut instance_fields)
            .context("could not read class instance fields")?;

        let mut direct_methods = Vec::with_capacity(direct_methods_size as usize);
        Self::read_methods(reader, direct_methods_size, &mut direct_methods)
            .context("could not read class direct methods")?;

        let mut virtual_methods = Vec::with_capacity(virtual_methods_size as usize);
        Self::read_methods(reader, virtual_methods_size, &mut virtual_methods)
            .context("could not read class virtual methods")?;

        Ok(Self {
            static_fields,
            instance_fields,
            direct_methods,
            virtual_methods,
        })
    }

    fn read_fields<R>(
        reader: &mut R,
        field_count: u32,
        field_vec: &mut Vec<Field>,
    ) -> Result<(), Error>
    where
        R: Read,
    {
        if field_count > 0 {
            // First field's ID is given directly.
            let (field_id, _) = uleb128(reader).context("could not read field ID")?;
            let (access_flags, _) = uleb128(reader).context("could not read field access flags")?;

            field_vec.push(Field {
                field_id,
                access_flags: AccessFlags::from_bits(access_flags)
                    .ok_or_else(|| error::Parse::InvalidAccessFlags { access_flags })?,
            });

            let mut last_field_id = field_id;
            for _ in 1..field_count {
                let (field_id_diff, _) = uleb128(reader).context("could not read field ID")?;
                let (access_flags, _) =
                    uleb128(reader).context("could not read field access flags")?;

                // Field IDs other than the first one are given by difference.
                last_field_id += field_id_diff;

                field_vec.push(Field {
                    field_id: last_field_id,
                    access_flags: AccessFlags::from_bits(access_flags)
                        .ok_or_else(|| error::Parse::InvalidAccessFlags { access_flags })?,
                });
            }
        }
        Ok(())
    }

    fn read_methods<R>(
        reader: &mut R,
        method_count: u32,
        method_vec: &mut Vec<Method>,
    ) -> Result<(), Error>
    where
        R: Read,
    {
        if method_count > 0 {
            // First method's ID is given directly.
            let (method_id, _) = uleb128(reader).context("could not read method ID")?;
            let (access_flags, _) =
                uleb128(reader).context("could not read method access flags")?;
            let (code_offset, _) = uleb128(reader).context("could not read method code offset")?;

            let code_offset = if code_offset == 0 {
                None
            } else {
                Some(code_offset)
            };

            method_vec.push(Method {
                method_id,
                access_flags: AccessFlags::from_bits(access_flags)
                    .ok_or_else(|| error::Parse::InvalidAccessFlags { access_flags })?,
                code_offset,
            });

            let mut last_method_id = method_id;
            for _ in 1..method_count {
                let (method_id_diff, _) = uleb128(reader).context("could not read method ID")?;
                let (access_flags, _) =
                    uleb128(reader).context("could not read method access flags")?;
                let (code_offset, _) =
                    uleb128(reader).context("could not read method code offset")?;

                let code_offset = if code_offset == 0 {
                    None
                } else {
                    Some(code_offset)
                };

                // Method IDs other than the first one are given by difference.
                last_method_id += method_id_diff;

                method_vec.push(Method {
                    method_id: last_method_id,
                    access_flags: AccessFlags::from_bits(access_flags)
                        .ok_or_else(|| error::Parse::InvalidAccessFlags { access_flags })?,
                    code_offset,
                });
            }
        }
        Ok(())
    }
}

/// Debug information structure.
#[derive(Debug)]
pub struct DebugInfo {
    line_start: u32,
    parameter_names: Vec<Option<u32>>,
    bytecode: DebugBytecode,
}

impl DebugInfo {
    /// Creates a new debug information structure from a reader.
    pub fn from_reader<R>(reader: &mut R) -> Result<(Self, u32), Error>
    where
        R: Read,
    {
        let (line_start, mut read) = uleb128(reader).context("could not read line_start field")?;
        let (parameters_size, read_p) =
            uleb128(reader).context("could not read parameters_size field")?;
        read += read_p;

        let mut parameter_names = Vec::with_capacity(parameters_size as usize);
        for _ in 0..parameters_size {
            let (name_index, read_i) =
                uleb128p1(reader).context("could not read parameter name index")?;
            read += read_i;
            parameter_names.push(name_index.into());
        }

        let (bytecode, read_b) =
            DebugBytecode::from_reader(reader).context("could not read debug bytecode")?;
        read += read_b;

        Ok((
            Self {
                line_start,
                parameter_names,
                bytecode,
            },
            read,
        ))
    }

    /// Gets the starting line of the debug information.
    pub fn line_start(&self) -> u32 {
        self.line_start
    }

    /// Gets the list of IDs of parameter names affected by the debug information structure in the
    /// string list.
    pub fn parameter_names(&self) -> &[Option<u32>] {
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
    fn from_reader<R>(reader: &mut R) -> Result<(Self, u32), Error>
    where
        R: Read,
    {
        let mut bytecode = Vec::new();
        let mut read = 0;
        loop {
            let (instruction, read_i) =
                DebugInstruction::from_reader(reader).context("could not read instruction")?;
            read += read_i;
            bytecode.push(instruction);

            if instruction == DebugInstruction::EndSequence {
                break;
            }
        }
        Ok((Self { bytecode }, read))
    }
}

/// Debug state machine instruction.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DebugInstruction {
    EndSequence,
    AdvancePc {
        addr_diff: u32,
    },
    AdvanceLine {
        line_diff: i32,
    },
    StartLocal {
        register_num: u32,
        name_id: Option<u32>,
        type_id: Option<u32>,
    },
    StartLocalExtended {
        register_num: u32,
        name_id: Option<u32>,
        type_id: Option<u32>,
        sig_id: Option<u32>,
    },
    EndLocal {
        register_num: u32,
    },
    RestartLocal {
        register_num: u32,
    },
    SetPrologueEnd,
    SetEpilogueBegin,
    SetFile {
        name_id: u32,
    },
    SpecialOpcode {
        opcode: u8,
    },
}

impl DebugInstruction {
    fn from_reader<R>(reader: &mut R) -> Result<(Self, u32), Error>
    where
        R: Read,
    {
        let mut opcode = [0_u8];
        reader
            .read_exact(&mut opcode)
            .context("could not read opcode")?;
        let mut read = 1;
        let instruction = match opcode[0] {
            0x00_u8 => Self::EndSequence,
            0x01_u8 => {
                let (addr_diff, read_ad) = uleb128(reader)
                    .context("could not read `addr_diff` for the DBG_ADVANCE_PC instruction")?;
                read += read_ad;
                Self::AdvancePc { addr_diff }
            }
            0x02_u8 => {
                let (line_diff, read_ld) = sleb128(reader).context({
                    "could not read `line_diff` for the DBG_ADVANCE_LINE instruction"
                })?;
                read += read_ld;
                Self::AdvanceLine { line_diff }
            }
            0x03_u8 => {
                let (register_num, read_rn) = uleb128(reader).context({
                    "could not read `register_num` for the DBG_START_LOCAL instruction"
                })?;
                let (name_id, read_ni) = uleb128p1(reader)
                    .context("could not read `name_id` for the DBG_START_LOCAL instruction")?;
                let (type_id, read_ti) = uleb128p1(reader)
                    .context("could not read `type_id` for the DBG_START_LOCAL instruction")?;
                read += read_rn + read_ni + read_ti;

                Self::StartLocal {
                    register_num,
                    name_id: name_id.into(),
                    type_id: type_id.into(),
                }
            }
            0x04_u8 => {
                let (register_num, read_rn) = uleb128(reader).context({
                    "could not read `register_num` for the DBG_START_LOCAL_EXTENDED instruction"
                })?;
                let (name_id, read_ni) = uleb128p1(reader).context({
                    "could not read `name_id` for the DBG_START_LOCAL_EXTENDED instruction"
                })?;
                let (type_id, read_ti) = uleb128p1(reader).context({
                    "could not read `type_id` for the DBG_START_LOCAL_EXTENDED instruction"
                })?;
                let (sig_id, read_si) = uleb128p1(reader).context({
                    "could not read `sig_id` for the DBG_START_LOCAL_EXTENDED instruction"
                })?;
                read += read_rn + read_ni + read_ti + read_si;

                Self::StartLocalExtended {
                    register_num,
                    name_id: name_id.into(),
                    type_id: type_id.into(),
                    sig_id: sig_id.into(),
                }
            }
            0x05_u8 => {
                let (register_num, read_rn) = uleb128(reader)
                    .context("could not read `register_num` for the DBG_END_LOCAL instruction")?;
                read += read_rn;
                Self::EndLocal { register_num }
            }
            0x06_u8 => {
                let (register_num, read_rn) = uleb128(reader).context(
                    "could not read `register_num` for the DBG_RESTART_LOCAL instruction",
                )?;
                read += read_rn;
                Self::RestartLocal { register_num }
            }
            0x07_u8 => Self::SetPrologueEnd,
            0x08_u8 => Self::SetEpilogueBegin,
            0x09_u8 => {
                let (name_id, read_ni) = uleb128(reader)
                    .context("could not read `name_id` for the DBG_SET_FILE instruction")?;
                read += read_ni;
                Self::SetFile { name_id }
            }
            opcode @ 0x0a_u8..=0xff_u8 => Self::SpecialOpcode { opcode },
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
    pub fn from_reader<R, B>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read + ReadBytesExt,
        B: ByteOrder,
    {
        let registers_size = reader
            .read_u16::<B>()
            .context("could not read registers size")?;
        let ins_size = reader
            .read_u16::<B>()
            .context("could not read incoming words size")?;
        let outs_size = reader
            .read_u16::<B>()
            .context("could not read outgoing words size")?;
        let tries_size = reader
            .read_u16::<B>()
            .context("could not read tries size")?;
        let debug_info_offset = reader
            .read_u32::<B>()
            .context("could not read debug information offset")?;
        let insns_size = reader
            .read_u32::<B>()
            .context("could not read the size of the bytecode array")?;

        let mut insns = Vec::with_capacity(insns_size as usize);
        for _ in 0..insns_size {
            insns.push(reader.read_u16::<B>().context("could not read bytecode")?);
        }

        if tries_size != 0 && (insns_size & 0b1 != 0) {
            let mut padding = [0_u8; 2];
            reader
                .read_exact(&mut padding)
                .context("could not read padding before tries")?;
        }

        let mut tries = Vec::with_capacity(tries_size as usize);
        for _ in 0..tries_size {
            tries.push(TryItem::from_reader::<_, B>(reader).context("could not read try item")?);
        }

        let mut handlers = Vec::new();
        if tries_size > 0 {
            let (handlers_size, _) =
                uleb128(reader).context("could not read catch handlers size")?;

            handlers.reserve_exact(handlers_size as usize);
            for _ in 0..handlers_size {
                let (handler, _) =
                    CatchHandler::from_reader(reader).context("could not read catch handler")?;
                handlers.push(handler);
            }
        }

        Ok(Self {
            registers_size,
            ins_size,
            outs_size,
            debug_info_offset,
            insns,
            tries,
            handlers,
        })
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
    fn from_reader<R, B>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read + ReadBytesExt,
        B: ByteOrder,
    {
        let start_address = reader
            .read_u32::<B>()
            .context("could not read start address")?;
        let insn_count = reader
            .read_u16::<B>()
            .context("could not read instruction count")?;
        let handler_offset = reader
            .read_u16::<B>()
            .context("could not read catch handler offset")?;

        Ok(Self {
            start_address,
            insn_count,
            handler_offset,
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
    #[allow(clippy::cast_sign_loss)]
    fn from_reader<R>(reader: &mut R) -> Result<(Self, u32), Error>
    where
        R: Read,
    {
        let (size, mut read) = sleb128(reader).context("could not read the catch handler size")?;

        let abs_size = size.abs() as usize;
        let mut handlers = Vec::with_capacity(abs_size);
        for _ in 0..abs_size {
            let (handler_info, read_hi) =
                HandlerInfo::from_reader(reader).context("could not read handler information")?;
            handlers.push(handler_info);
            read += read_hi;
        }

        let catch_all_addr = if size < 1 {
            let (addr, read_ca) =
                uleb128(reader).context("could not read the catch all address")?;
            read += read_ca;
            Some(addr)
        } else {
            None
        };

        Ok((
            Self {
                handlers,
                catch_all_addr,
            },
            read,
        ))
    }
}

#[derive(Debug)]
struct HandlerInfo {
    type_id: u32,
    addr: u32,
}

impl HandlerInfo {
    /// Creates a handler information structure from a reader object.
    fn from_reader<R>(reader: &mut R) -> Result<(Self, u32), Error>
    where
        R: Read,
    {
        let (type_id, read_t) = uleb128(reader).context("could not read type ID")?;
        let (addr, read_a) = uleb128(reader).context("could not read address")?;

        Ok((Self { type_id, addr }, read_t + read_a))
    }
}

#[cfg(test)]
mod tests {
    use std::{f32, f64, io::Cursor};

    use super::{
        VALUE_BYTE, VALUE_CHAR, VALUE_DOUBLE, VALUE_FLOAT, VALUE_INT, VALUE_LONG, VALUE_SHORT,
    };
    use crate::types::Value;

    #[test]
    fn it_returns_error_if_value_type_is_not_valid() {
        let raw = [0x01];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        assert!(value_result.is_err());
        assert!(format!("{}", value_result.err().unwrap()).starts_with("invalid value:"));
    }

    #[test]
    fn it_can_decoder_a_byte_value() {
        let raw = [VALUE_BYTE, 255];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Byte(res) => assert_eq!(res, -1),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_not_decode_a_byte_value_if_arg_is_not_zero() {
        let raw = [VALUE_BYTE | 1 << 6, 45];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        assert!(value_result.is_err());
        assert!(format!("{}", value_result.err().unwrap()).starts_with("invalid value:"));
    }

    #[test]
    fn it_can_decode_a_short_value_one_byte() {
        let raw = [VALUE_SHORT, 255];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Short(res) => assert_eq!(res, -1),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_short_value_two_byte() {
        let raw = [VALUE_SHORT | 1 << 5, 255, 45];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Short(res) => assert_eq!(res, 11775),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_not_decode_a_short_value_if_arg_is_more_than_1() {
        let raw = [VALUE_SHORT | 1 << 6, 45, 67];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        assert!(value_result.is_err());
        assert!(format!("{}", value_result.err().unwrap()).starts_with("invalid value:"));
    }

    #[test]
    fn it_can_decode_a_char_value_one_byte() {
        let raw = [VALUE_CHAR, 255];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Char(res) => assert_eq!(res, 255),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_char_value_two_byte() {
        let raw = [VALUE_CHAR | 1 << 5, 255, 45];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Char(res) => assert_eq!(res, 11775),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_not_decode_a_char_value_if_arg_is_more_than_1() {
        let raw = [VALUE_CHAR | 1 << 6, 45, 67];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        assert!(value_result.is_err());
        assert!(format!("{}", value_result.err().unwrap()).starts_with("invalid value:"));
    }

    #[test]
    fn it_can_decode_a_int_value_one_byte() {
        let raw = [VALUE_INT, 127];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Int(res) => assert_eq!(res, 127),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_int_value_two_bytes() {
        let raw = [VALUE_INT | 1 << 5, 254, 255];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Int(res) => assert_eq!(res, -2),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_int_value_three_bytes() {
        let raw = [VALUE_INT | 2 << 5, 255, 45, 23];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Int(res) => assert_eq!(res, 1_519_103),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_int_value_four_byte() {
        let raw = [VALUE_INT | 3 << 5, 255, 45, 255, 34];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Int(res) => assert_eq!(res, 587_148_799),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_not_decode_a_int_value_if_arg_is_more_than_3() {
        let raw = [VALUE_INT | 4 << 5, 45, 67];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        assert!(value_result.is_err());
        assert!(format!("{}", value_result.err().unwrap()).starts_with("invalid value:"));
    }

    #[test]
    fn it_can_decode_a_long_value_one_byte() {
        let raw = [VALUE_LONG, 128];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, -128),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_long_value_two_bytes() {
        let raw = [VALUE_LONG | 1 << 5, 254, 255];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, -2),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_long_value_three_bytes() {
        let raw = [VALUE_LONG | 2 << 5, 1, 1, 1];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, 65793),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_long_value_four_byte() {
        let raw = [VALUE_LONG | 3 << 5, 1, 2, 3, 4];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, 67_305_985),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_long_value_five_byte() {
        let raw = [VALUE_LONG | 4 << 5, 255, 254, 253, 252, 251];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, -17_230_332_161),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_long_value_six_byte() {
        let raw = [VALUE_LONG | 5 << 5, 255, 1, 254, 2, 253, 3];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, 4_385_211_810_303),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_long_value_seven_byte() {
        let raw = [VALUE_LONG | 6 << 5, 255, 0, 0, 0, 0, 0, 0];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, 255),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_long_value_eight_byte() {
        let raw = [VALUE_LONG | 7 << 5, 0, 0, 0, 0, 1, 1, 1, 1];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Long(res) => assert_eq!(res, 72_340_172_821_233_664),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_decode_a_float_value() {
        let raw = [VALUE_FLOAT, 0, 0, 0, 0];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Float(res) => assert!((res - 0.0).abs() < f32::EPSILON),
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_can_not_decode_a_float_value_if_arg_is_more_than_4() {
        let raw = [VALUE_FLOAT | 5 << 5, 0, 0, 0, 0];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        assert!(value_result.is_err());
        assert!(format!("{}", value_result.err().unwrap()).starts_with("invalid value:"));
    }

    #[test]
    fn it_can_decode_a_double_value() {
        let raw = [VALUE_DOUBLE | 7 << 5, 0, 0, 0, 0, 0, 0, 0, 0];
        let value_result = Value::from_reader(&mut Cursor::new(raw));

        match value_result.unwrap() {
            Value::Double(res) => assert!((res - 0.0).abs() < f64::EPSILON),
            _ => unreachable!(),
        }
    }
}
