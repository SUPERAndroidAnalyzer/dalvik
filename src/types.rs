use std::fmt;
use error::{Result, Error};
use sizes::*;

/// Data structure representing the `string_id_item` type.
#[derive(Debug, Clone)]
pub struct StringIdData {
    string_offset: usize,
}

impl StringIdData {
    /// Creates a new `StringIdData` from the `u32` representing the `string_data_off` of the
    /// `string_id_item` type.
    pub fn new(string_offset: u32) -> StringIdData {
        StringIdData { string_offset: string_offset as usize }
    }

    /// Gets the string offset in the `data` section.
    ///
    /// Gets the offset from the start of the file to the string data for this item. The offset
    /// should be to a location in the `data` section, and the data should be in the format
    /// specified by `string_data_item`. There is no alignment requirement for the offset.
    pub fn get_string_offset(&self) -> usize {
        self.string_offset
    }
}

/// Data structure representing the `type_id_item` type.
#[derive(Debug, Clone)]
pub struct TypeIdData {
    descriptor_index: usize,
}

impl TypeIdData {
    /// Creates a new `TypeIdData` from the `u32` representing the `descriptor_idx` of the
    /// `type_id_item` type.
    pub fn new(descriptor_index: u32) -> TypeIdData {
        TypeIdData { descriptor_index: descriptor_index as usize }
    }

    /// Gets the descriptor index in the `string_ids` section.
    ///
    /// Gets the index into the `string_ids` list for the descriptor string of this type. The
    /// string must conform to the syntax for `TypeDescriptor`.
    pub fn get_descriptor_index(&self) -> usize {
        self.descriptor_index
    }
}

/// Data structure representing the `proto_id_item` type.
#[derive(Debug, Clone)]
pub struct PrototypeIdData {
    shorty_index: usize,
    return_type_index: usize,
    parameters_offset: Option<usize>,
}

impl PrototypeIdData {
    /// Creates a new `PrototypeIdData` from the three `u32` that conform the `proto_id_item` type.
    pub fn new(shorty_index: u32,
               return_type_index: u32,
               parameters_offset: u32)
               -> PrototypeIdData {
        PrototypeIdData {
            shorty_index: shorty_index as usize,
            return_type_index: return_type_index as usize,
            parameters_offset: if parameters_offset != 0 {
                Some(parameters_offset as usize)
            } else {
                None
            },
        }
    }

    /// Gets the shorty index.
    ///
    /// Gets the index into the `string_ids` list for the short-form descriptor string of this
    /// prototype. The string must conform to the syntax for `ShortyDescriptor`, and must
    /// correspond to the return type and parameters of this item.
    pub fn get_shorty_index(&self) -> usize {
        self.shorty_index
    }

    /// Gets the return type index.
    ///
    /// Gets the index into the `type_ids` list for the return type of this prototype.
    pub fn get_return_type_index(&self) -> usize {
        self.return_type_index
    }

    /// Gets the parameter list offset, if the prototype has parameters.
    ///
    /// Gets the offset from the start of the file to the list of parameter types for this
    /// prototype, or `None` if this prototype has no parameters. This offset, should be in the
    /// data section, and the `data` there should be in the format specified by `type_list`.
    /// Additionally, there should be no reference to the type `void` in the list.
    pub fn get_parameters_offset(&self) -> Option<usize> {
        self.parameters_offset
    }
}

/// Structure representing the `field_id_item` type.
#[derive(Debug, Clone)]
pub struct FieldIdData {
    class_index: usize,
    type_index: usize,
    name_index: usize,
}

impl FieldIdData {
    /// Creates a new `FieldIdData` from the two `u16` and the `u32` that conform the
    /// `field_id_item` type.
    pub fn new(class_index: u16, type_index: u16, name_index: u32) -> FieldIdData {
        FieldIdData {
            class_index: class_index as usize,
            type_index: type_index as usize,
            name_index: name_index as usize,
        }
    }

    /// Gets the index of the class of the field.
    ///
    /// Gets the index into the `type_ids` list for the definer of this field. This must be a class
    /// type, and not an array or primitive type.
    pub fn get_class_index(&self) -> usize {
        self.class_index
    }

    /// Gets the index of the type of the class.
    ///
    /// Gets the index into the `type_ids` list for the type of this field.
    pub fn get_type_index(&self) -> usize {
        self.type_index
    }

    /// Gets the index to the name of this field.
    ///
    /// Gets the index into the `string_ids` list for the name of this field. The string must
    /// conform to the syntax for `MemberName`.
    pub fn get_name_index(&self) -> usize {
        self.name_index
    }
}

/// Structure representing the `method_id_item` type.
#[derive(Debug, Clone)]
pub struct MethodIdData {
    class_index: usize,
    prototype_index: usize,
    name_index: usize,
}

impl MethodIdData {
    /// Creates a new `MethodIdData` from the two `u16` and the `u32` that conform the
    /// `method_id_item` type.
    pub fn new(class_index: u16, prototype_index: u16, name_index: u32) -> MethodIdData {
        MethodIdData {
            class_index: class_index as usize,
            prototype_index: prototype_index as usize,
            name_index: name_index as usize,
        }
    }

    /// Gets the index of the class of the field.
    ///
    /// Gets the index into the `type_ids` list for the definer of this method. This must be a
    /// class or array type, and not a primitive type.
    pub fn get_class_index(&self) -> usize {
        self.class_index
    }

    /// Gets the index of the prototype of the class.
    ///
    /// Gets the index into the `prototype_ids` list for the prototype of this method.
    pub fn get_prototype_index(&self) -> usize {
        self.prototype_index
    }

    /// Gets the index to the name of this field.
    ///
    /// Gets the index into the `string_ids` list for the name of this field. The string must
    /// conform to the syntax for `MemberName`.
    pub fn get_name_index(&self) -> usize {
        self.name_index
    }
}

const NO_INDEX: u32 = 0xFFFFFFFF;

bitflags! {
    flags AccessFlags: u32 {
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

pub struct ClassDefData {
    class_id: usize,
    access_flags: AccessFlags,
    superclass_id: Option<usize>,
    interfaces_offset: Option<usize>,
    source_file_id: Option<usize>,
    annotations_offset: Option<usize>,
    class_data_offset: Option<usize>,
    static_values_offset: Option<usize>,
}

impl ClassDefData {
    pub fn new(class_id: u32,
               access_flags: u32,
               superclass_id: u32,
               interfaces_offset: u32,
               source_file_id: u32,
               annotations_offset: u32,
               class_data_offset: u32,
               static_values_offset: u32)
               -> Result<ClassDefData> {
        Ok(ClassDefData {
            class_id: class_id as usize,
            access_flags: try!(AccessFlags::from_bits(access_flags)
                .ok_or(Error::invalid_access_flags(access_flags))),
            superclass_id: if superclass_id != NO_INDEX {
                Some(superclass_id as usize)
            } else {
                None
            },
            interfaces_offset: if interfaces_offset != 0 {
                Some(interfaces_offset as usize)
            } else {
                None
            },
            source_file_id: if source_file_id != NO_INDEX {
                Some(source_file_id as usize)
            } else {
                None
            },
            annotations_offset: if annotations_offset != 0 {
                Some(annotations_offset as usize)
            } else {
                None
            },
            class_data_offset: if class_data_offset != 0 {
                Some(class_data_offset as usize)
            } else {
                None
            },
            static_values_offset: if static_values_offset != 0 {
                Some(static_values_offset as usize)
            } else {
                None
            },
        })
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

#[derive(Debug, Copy, Clone)]
pub enum ItemType {
    HeaderItem,
    StringIdItem,
    TypeIdItem,
    ProtoIdItem,
    FieldIdItem,
    MethodIdItem,
    ClassDefItem,
    MapList,
    TypeList,
    AnnotationSetRefList,
    AnnotationSetItem,
    ClassDataItem,
    CodeItem,
    StringDataItem,
    DebugInfoItem,
    AnnotationItem,
    EncodedArrayItem,
    AnnotationsDirectoryItem,
}

impl ItemType {
    fn from_u16(value: u16) -> Result<ItemType> {
        match value {
            TYPE_HEADER_ITEM => Ok(ItemType::HeaderItem),
            TYPE_STRING_ID_ITEM => Ok(ItemType::StringIdItem),
            TYPE_TYPE_ID_ITEM => Ok(ItemType::TypeIdItem),
            TYPE_PROTO_ID_ITEM => Ok(ItemType::ProtoIdItem),
            TYPE_FIELD_ID_ITEM => Ok(ItemType::FieldIdItem),
            TYPE_METHOD_ID_ITEM => Ok(ItemType::MethodIdItem),
            TYPE_CLASS_DEF_ITEM => Ok(ItemType::ClassDefItem),
            TYPE_MAP_LIST => Ok(ItemType::MapList),
            TYPE_TYPE_LIST => Ok(ItemType::TypeList),
            TYPE_ANNOTATION_SET_REF_LIST => Ok(ItemType::AnnotationSetRefList),
            TYPE_ANNOTATION_SET_ITEM => Ok(ItemType::AnnotationSetItem),
            TYPE_CLASS_DATA_ITEM => Ok(ItemType::ClassDataItem),
            TYPE_CODE_ITEM => Ok(ItemType::CodeItem),
            TYPE_STRING_DATA_ITEM => Ok(ItemType::StringDataItem),
            TYPE_DEBUG_INFO_ITEM => Ok(ItemType::DebugInfoItem),
            TYPE_ANNOTATION_ITEM => Ok(ItemType::AnnotationItem),
            TYPE_ENCODED_ARRAY_ITEM => Ok(ItemType::EncodedArrayItem),
            TYPE_ANNOTATIONS_DIRECTORY_ITEM => Ok(ItemType::AnnotationsDirectoryItem),
            v => Err(Error::invalid_item_type(v)),
        }
    }
}

impl From<ItemType> for u16 {
    fn from(item_type: ItemType) -> u16 {
        match item_type {
            ItemType::HeaderItem => TYPE_HEADER_ITEM,
            ItemType::StringIdItem => TYPE_STRING_ID_ITEM,
            ItemType::TypeIdItem => TYPE_TYPE_ID_ITEM,
            ItemType::ProtoIdItem => TYPE_PROTO_ID_ITEM,
            ItemType::FieldIdItem => TYPE_FIELD_ID_ITEM,
            ItemType::MethodIdItem => TYPE_METHOD_ID_ITEM,
            ItemType::ClassDefItem => TYPE_CLASS_DEF_ITEM,
            ItemType::MapList => TYPE_MAP_LIST,
            ItemType::TypeList => TYPE_TYPE_LIST,
            ItemType::AnnotationSetRefList => TYPE_ANNOTATION_SET_REF_LIST,
            ItemType::AnnotationSetItem => TYPE_ANNOTATION_SET_ITEM,
            ItemType::ClassDataItem => TYPE_CLASS_DATA_ITEM,
            ItemType::CodeItem => TYPE_CODE_ITEM,
            ItemType::StringDataItem => TYPE_STRING_DATA_ITEM,
            ItemType::DebugInfoItem => TYPE_DEBUG_INFO_ITEM,
            ItemType::AnnotationItem => TYPE_ANNOTATION_ITEM,
            ItemType::EncodedArrayItem => TYPE_ENCODED_ARRAY_ITEM,
            ItemType::AnnotationsDirectoryItem => TYPE_ANNOTATIONS_DIRECTORY_ITEM,
        }
    }
}

pub struct MapItem {
    item_type: ItemType,
    size: usize,
    offset: usize,
}

impl fmt::Debug for MapItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "MapItem {{ item_type: {:?} ({:#06x}), num_items: {} items, size: {}, \
                offset: {:#010x} }}",
               self.item_type,
               u16::from(self.item_type),
               self.size,
               match self.get_size() {
                   Some(size) => format!("{} bytes", size),
                   None => String::from("unknown"),
               },
               self.offset)
    }
}

impl MapItem {
    pub fn new(item_type: u16, size: u32, offset: u32) -> Result<MapItem> {
        Ok(MapItem {
            item_type: try!(ItemType::from_u16(item_type)),
            size: size as usize,
            offset: offset as usize,
        })
    }

    pub fn get_item_type(&self) -> ItemType {
        self.item_type
    }

    pub fn get_num_items(&self) -> usize {
        self.size
    }

    pub fn get_size(&self) -> Option<usize> {
        match self.item_type {
            ItemType::HeaderItem => Some(HEADER_SIZE),
            ItemType::StringIdItem => Some(STRING_ID_ITEM_SIZE),
            ItemType::TypeIdItem => Some(TYPE_ID_ITEM_SIZE),
            ItemType::ProtoIdItem => Some(PROTO_ID_ITEM_SIZE),
            ItemType::FieldIdItem => Some(FIELD_ID_ITEM_SIZE),
            ItemType::MethodIdItem => Some(METHOD_ID_ITEM_SIZE),
            ItemType::ClassDefItem => Some(CLASS_DEF_ITEM_SIZE),
            ItemType::MapList => Some(4 + self.size * MAP_ITEM_SIZE),
            ItemType::TypeList => Some(4 + self.size * TYPE_ITEM_SIZE),
            ItemType::AnnotationSetRefList => Some(4 + self.size * ANNOTATION_SET_REF_SIZE),
            ItemType::AnnotationSetItem => Some(4 + self.size * ANNOTATION_SET_ITEM_SIZE),
            _ => None,
        }
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }
}

#[allow(unused)]
pub enum LEB128 {
    B1(u8),
    B2(u8, u8),
    B3(u8, u8, u8),
    B4(u8, u8, u8, u8),
    B5(u8, u8, u8, u8, u8),
}

#[allow(dead_code)]
impl LEB128 {
    pub fn parse(bytes: &[u8]) -> LEB128 {
        let leb128 = match bytes.len() {
            1 => LEB128::B1(*unsafe { bytes.get_unchecked(0) }),
            2 => {
                LEB128::B2(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) })
            }
            3 => {
                LEB128::B3(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) },
                           *unsafe { bytes.get_unchecked(2) })
            }
            4 => {
                LEB128::B4(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) },
                           *unsafe { bytes.get_unchecked(2) },
                           *unsafe { bytes.get_unchecked(3) })
            }
            5 => {
                LEB128::B5(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) },
                           *unsafe { bytes.get_unchecked(2) },
                           *unsafe { bytes.get_unchecked(3) },
                           *unsafe { bytes.get_unchecked(4) })
            }
            l => {
                panic!("LEB128 slice length is {}, it must be between 1 and 5 bytes",
                       l)
            }
        };
        unimplemented!()
    }
}
