//! Types module.

pub mod read;

use std::str::FromStr;
use std::ops::Deref;

use error::*;
use self::read::ClassData;

#[derive(Debug, Clone)]
/// Basic built-in types.
pub enum Type {
    /// Void type.
    Void,
    /// Boolean.
    Boolean,
    /// Byte (8 bits).
    Byte,
    /// Short (16 bits).
    Short,
    /// Char (16 bits).
    Char,
    /// Int (32 bits).
    Int,
    /// Long (64 bits).
    Long,
    /// Float (32 bits).
    Float,
    /// Double (64 bits).
    Double,
    /// Fully qualified named type.
    ///
    /// Example: an object.
    FullyQualifiedName(String),
    /// Array.
    Array {
        /// Array dimensions.
        dimensions: u8,
        /// Type of the array.
        array_type: Box<Type>,
    },
}

impl FromStr for Type {
    type Err = Error;
    fn from_str(s: &str) -> Result<Type> {
        let mut chars = s.chars();
        match chars.next() {
            Some('V') => Ok(Type::Void),
            Some('Z') => Ok(Type::Boolean),
            Some('B') => Ok(Type::Byte),
            Some('S') => Ok(Type::Short),
            Some('C') => Ok(Type::Char),
            Some('I') => Ok(Type::Int),
            Some('J') => Ok(Type::Long),
            Some('F') => Ok(Type::Float),
            Some('D') => Ok(Type::Double),
            Some('[') => {
                let mut dimensions = 1;
                loop {
                    match chars.next() {
                        Some('[') => dimensions += 1,
                        Some(t) => {
                            let mut type_str = String::with_capacity(s.len() - dimensions as usize);
                            type_str.push(t);
                            type_str.push_str(chars.as_str());
                            return Ok(Type::Array {
                                          dimensions: dimensions,
                                          array_type: Box::new(type_str.parse()?),
                                      });
                        }
                        None => return Err(ErrorKind::InvalidTypeDescriptor(s.to_owned()).into()),
                    }
                }
            }
            Some('L') => Ok(Type::FullyQualifiedName(chars.as_str().to_owned())),
            _ => Err(ErrorKind::InvalidTypeDescriptor(s.to_owned()).into()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ShortyReturnType {
    Void,
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Reference,
}

impl ShortyReturnType {
    fn from_char(c: char) -> Result<ShortyReturnType> {
        match c {
            'V' => Ok(ShortyReturnType::Void),
            'Z' => Ok(ShortyReturnType::Boolean),
            'B' => Ok(ShortyReturnType::Byte),
            'S' => Ok(ShortyReturnType::Short),
            'C' => Ok(ShortyReturnType::Char),
            'I' => Ok(ShortyReturnType::Int),
            'J' => Ok(ShortyReturnType::Long),
            'F' => Ok(ShortyReturnType::Float),
            'D' => Ok(ShortyReturnType::Double),
            'L' => Ok(ShortyReturnType::Reference),
            _ => Err(ErrorKind::InvalidShortyType(c).into()),
        }
    }
}

impl From<Type> for ShortyReturnType {
    fn from(t: Type) -> ShortyReturnType {
        match t {
            Type::Void => ShortyReturnType::Void,
            Type::Boolean => ShortyReturnType::Boolean,
            Type::Byte => ShortyReturnType::Byte,
            Type::Short => ShortyReturnType::Short,
            Type::Char => ShortyReturnType::Char,
            Type::Int => ShortyReturnType::Int,
            Type::Long => ShortyReturnType::Long,
            Type::Float => ShortyReturnType::Float,
            Type::Double => ShortyReturnType::Double,
            Type::FullyQualifiedName(_) => ShortyReturnType::Reference,
            Type::Array { .. } => ShortyReturnType::Reference,
        }
    }
}

impl From<ShortyFieldType> for ShortyReturnType {
    fn from(ft: ShortyFieldType) -> ShortyReturnType {
        match ft {
            ShortyFieldType::Boolean => ShortyReturnType::Boolean,
            ShortyFieldType::Byte => ShortyReturnType::Byte,
            ShortyFieldType::Short => ShortyReturnType::Short,
            ShortyFieldType::Char => ShortyReturnType::Char,
            ShortyFieldType::Int => ShortyReturnType::Int,
            ShortyFieldType::Long => ShortyReturnType::Long,
            ShortyFieldType::Float => ShortyReturnType::Float,
            ShortyFieldType::Double => ShortyReturnType::Double,
            ShortyFieldType::Reference => ShortyReturnType::Reference,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ShortyFieldType {
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Reference,
}

impl ShortyFieldType {
    fn from_char(c: char) -> Result<ShortyFieldType> {
        match c {
            'Z' => Ok(ShortyFieldType::Boolean),
            'B' => Ok(ShortyFieldType::Byte),
            'S' => Ok(ShortyFieldType::Short),
            'C' => Ok(ShortyFieldType::Char),
            'I' => Ok(ShortyFieldType::Int),
            'J' => Ok(ShortyFieldType::Long),
            'F' => Ok(ShortyFieldType::Float),
            'D' => Ok(ShortyFieldType::Double),
            'L' => Ok(ShortyFieldType::Reference),
            _ => Err(ErrorKind::InvalidShortyType(c).into()),
        }
    }
}

/// Short form of type descriptor.
#[derive(Debug)]
pub struct ShortyDescriptor {
    return_type: ShortyReturnType,
    field_types: Box<[ShortyFieldType]>,
}

impl FromStr for ShortyDescriptor {
    type Err = Error;
    fn from_str(s: &str) -> Result<ShortyDescriptor> {
        let mut chars = s.chars();
        let return_type = if let Some(c) = chars.next() {
            ShortyReturnType::from_char(c)?
        } else {
            return Err(ErrorKind::InvalidShortyDescriptor(s.to_owned()).into());
        };
        let mut field_types = Vec::with_capacity(s.len() - 1);
        for c in chars {
            field_types.push(ShortyFieldType::from_char(c)?);
        }
        Ok(ShortyDescriptor {
               return_type: return_type,
               field_types: field_types.into_boxed_slice(),
           })
    }
}

/// Prototype implementation.
#[derive(Debug)]
pub struct Prototype {
    descriptor: ShortyDescriptor,
    return_type: Type,
    parameters: Option<Box<[Type]>>,
}

impl Prototype {
    /// Creates a new prototype.
    pub fn new<TA: Into<Option<Box<[Type]>>>>(descriptor: ShortyDescriptor,
                                              return_type: Type,
                                              parameters: TA)
                                              -> Prototype {
        Prototype {
            descriptor: descriptor,
            return_type: return_type,
            parameters: parameters.into(),
        }
    }
}

/// Annotation visibility.
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    /// Build time visibility.
    Build,
    /// Runtime visibility.
    Runtime,
    /// System visibility.
    System,
}

/// Value of a variable.
#[derive(Debug)]
pub enum Value {
    /// Byte.
    Byte(i8),
    /// Short (16 bits).
    Short(i16),
    /// Char (16 bts).
    Char(u16),
    /// Int (32 bits).
    Int(i32),
    /// Long (64 bits).
    Long(i64),
    /// Float (32 bits).
    Float(f32),
    /// Double (64 bits).
    Double(f64),
    /// String, with the index into the string IDs list.
    String(u32),
    /// Type, with the index into the type IDs list.
    Type(u32),
    /// Field, with the index into the field IDs list.
    Field(u32),
    /// Method with the index into the prototype IDs list.
    Method(u32),
    /// Enum with the index into the fiels IDs list.
    Enum(u32),
    /// An array of values.
    Array(Array),
    /// Annotation.
    Annotation(EncodedAnnotation),
    /// Null.
    Null,
    /// Boolean.
    Boolean(bool),
}

/// Array.
#[derive(Debug)]
pub struct Array {
    inner: Box<[Value]>,
}

/// Annotation element.
#[derive(Debug)]
pub struct AnnotationElement {
    name: u32,
    value: Value,
}

impl AnnotationElement {
    /// Gets the index of the name string.
    pub fn name_index(&self) -> u32 {
        self.name
    }
}

impl Deref for AnnotationElement {
    type Target = Value;

    fn deref(&self) -> &Value {
        &self.value
    }
}

/// Annotation.
#[derive(Debug)]
pub struct EncodedAnnotation {
    type_id: u32,
    elements: Box<[AnnotationElement]>,
}

impl EncodedAnnotation {
    /// Gets the index of the type of the annotation.
    pub fn type_index(&self) -> u32 {
        self.type_id
    }

    /// Gets the elements of the annotation.
    pub fn elements(&self) -> &[AnnotationElement] {
        &self.elements
    }
}

/// Annotation item
#[derive(Debug)]
pub struct Annotation {
    visibility: Visibility,
    annotation: EncodedAnnotation,
}

impl Annotation {
    /// Gets the visibility of the annotation item.
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
}

impl Deref for Annotation {
    type Target = EncodedAnnotation;

    fn deref(&self) -> &EncodedAnnotation {
        &self.annotation
    }
}

/// Annotations directory.
#[derive(Debug)]
pub struct AnnotationsDirectory {
    class_annotations: Box<[Annotation]>,
    field_annotations: Box<[FieldAnnotations]>,
    method_annotations: Box<[MethodAnnotations]>,
    parameter_annotations: Box<[ParameterAnnotations]>,
}

impl AnnotationsDirectory {
    /// Creates a new annotations directory.
    pub fn new<CA: Into<Box<[Annotation]>>>(class_annotations: CA,
                                            field_annotations: Box<[FieldAnnotations]>,
                                            method_annotations: Box<[MethodAnnotations]>,
                                            parameter_annotations: Box<[ParameterAnnotations]>)
                                            -> AnnotationsDirectory {
        AnnotationsDirectory {
            class_annotations: class_annotations.into(),
            field_annotations: field_annotations,
            method_annotations: method_annotations,
            parameter_annotations: parameter_annotations,
        }
    }

    /// Gets the list of class annotations.
    pub fn class_annotations(&self) -> &[Annotation] {
        &self.class_annotations
    }

    /// Gets the list of field annotations.
    pub fn field_annotations(&self) -> &[FieldAnnotations] {
        &self.field_annotations
    }

    /// Gets the list of method annotations.
    pub fn method_annotations(&self) -> &[MethodAnnotations] {
        &self.method_annotations
    }

    /// Gets the list of parameter annotations.
    pub fn parameter_annotations(&self) -> &[ParameterAnnotations] {
        &self.parameter_annotations
    }
}

/// Field annotations.
#[derive(Debug)]
pub struct FieldAnnotations {
    field_id: u32,
    annotations: Box<[Annotation]>,
}

impl FieldAnnotations {
    /// Creates a new list of field annotations.
    pub fn new(field_id: u32, annotations: Box<[Annotation]>) -> FieldAnnotations {
        FieldAnnotations {
            field_id: field_id,
            annotations: annotations,
        }
    }

    /// Gets the index of the annotated field.
    pub fn field_index(&self) -> u32 {
        self.field_id
    }

    /// Gets the list of annotations.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// Method annotations.
#[derive(Debug)]
pub struct MethodAnnotations {
    method_id: u32,
    annotations: Box<[Annotation]>,
}

impl MethodAnnotations {
    /// Creates a new list of method annotations.
    pub fn new(method_id: u32, annotations: Box<[Annotation]>) -> MethodAnnotations {
        MethodAnnotations {
            method_id: method_id,
            annotations: annotations,
        }
    }

    /// Gets the index of the annotated method.
    pub fn method_index(&self) -> u32 {
        self.method_id
    }

    /// Gets the list of annotations.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// Parameter annotations.
#[derive(Debug)]
pub struct ParameterAnnotations {
    method_id: u32,
    annotations: Box<[Annotation]>,
}

impl ParameterAnnotations {
    /// Creates a new list of method annotations.
    pub fn new(method_id: u32, annotations: Box<[Annotation]>) -> ParameterAnnotations {
        ParameterAnnotations {
            method_id: method_id,
            annotations: annotations,
        }
    }

    /// Gets the index of the annotated method.
    pub fn method_index(&self) -> u32 {
        self.method_id
    }

    /// Gets the list of annotations.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

bitflags! {
    /// Access flags.
    pub flags AccessFlags: u32 {
        /// Public access.
        const ACC_PUBLIC = 0x1,
        /// Private access.
        const ACC_PRIVATE = 0x2,
        /// Protected access.
        const ACC_PROTECTED = 0x4,
        /// Static access.
        const ACC_STATIC = 0x8,
        /// Final element (non modifiable).
        const ACC_FINAL = 0x10,
        /// Thread - synchronized element.
        const ACC_SYNCHRONIZED = 0x20,
        /// Volatile element.
        const ACC_VOLATILE = 0x40,
        /// Bridge.
        const ACC_BRIDGE = 0x40,
        /// Transient.
        const ACC_TRANSIENT = 0x80,
        /// Varargs.
        const ACC_VARARGS = 0x80,
        /// Native element.
        const ACC_NATIVE = 0x100,
        /// Interface.
        const ACC_INTERFACE = 0x200,
        /// Abstract element.
        const ACC_ABSTRACT = 0x400,
        /// Strict.
        const ACC_STRICT = 0x800,
        /// Syntetic.
        const ACC_SYNTHETIC = 0x1000,
        /// Annotation.
        const ACC_ANNOTATION = 0x2000,
        /// Enum.
        const ACC_ENUM = 0x4000,
        /// Constructor.
        const ACC_CONSTRUCTOR = 0x10000,
        /// Declared as synchronized element.
        const ACC_DECLARED_SYNCHRONIZED = 0x20000,
    }
}

/// Structure representing a class.
#[derive(Debug)]
pub struct Class {
    class_index: u32,
    access_flags: AccessFlags,
    superclass_index: Option<u32>,
    interfaces: Box<[Type]>,
    source_file_index: Option<u32>,
    annotations: Option<AnnotationsDirectory>,
    class_data: Option<ClassData>,
    static_values: Option<Array>,
}

impl Class {
    /// Creates a new class.
    pub fn new(class_index: u32,
               access_flags: AccessFlags,
               superclass_index: Option<u32>,
               interfaces: Box<[Type]>,
               source_file_index: Option<u32>,
               annotations: Option<AnnotationsDirectory>,
               class_data: Option<ClassData>,
               static_values: Option<Array>)
               -> Class {
        Class {
            class_index: class_index,
            access_flags: access_flags,
            superclass_index: superclass_index,
            interfaces: interfaces,
            source_file_index: source_file_index,
            annotations: annotations,
            class_data: class_data,
            static_values: static_values,
        }
    }

    /// Gets the index of the class in the type IDs list.
    pub fn class_index(&self) -> u32 {
        self.class_index
    }

    /// Gets the access flags of the class.
    pub fn access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    /// Gets the index in the type IDs list of the superclass for this class, ifd it exists.
    pub fn superclass_index(&self) -> Option<u32> {
        self.superclass_index
    }

    /// Gets the list of interfaces implemented by the class.
    pub fn interfaces(&self) -> &[Type] {
        &self.interfaces
    }

    /// Gets the index of the source file in the string list if it's known.
    pub fn source_file_index(&self) -> Option<u32> {
        self.source_file_index
    }

    /// Gets the annotations for the class, if there are any.
    pub fn annotations(&self) -> Option<&AnnotationsDirectory> {
        self.annotations.as_ref()
    }

    /// Gets the data associated with the class.
    pub fn class_data(&self) -> Option<&ClassData> {
        self.class_data.as_ref()
    }

    /// Gets the arrays with the values for the static files in this class.
    ///
    /// The values are in the same order as the static_field_ids in the class data of the class. If
    /// a value is not found, it is considered `0` or `NULL` depending on the type of the variable.
    pub fn static_values(&self) -> Option<&Array> {
        self.static_values.as_ref()
    }


    // static_values: static_values,
}
