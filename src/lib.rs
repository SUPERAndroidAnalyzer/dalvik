extern crate byteorder;
#[macro_use]
extern crate bitflags;

use std::path::Path;
use std::{fs, u32};
use std::io::prelude::*;
use std::io::BufReader;

use byteorder::{BigEndian, LittleEndian, ByteOrder, ReadBytesExt};

pub mod error;
pub mod bytecode; // TODO: not in use
pub mod types; // TODO: Should not be public
pub mod sizes; // TODO: Should not be public
pub mod offset_map; // TODO: Should not be public
pub mod header;

use error::{Result, Error};
use sizes::*;
pub use header::Header;
use types::{PrototypeIdData, FieldIdData, MethodIdData, ClassDefData, MapItem, ItemType};
use offset_map::{OffsetMap, OffsetType};

#[derive(Debug)]
pub struct Dex {
    header: Header,
    strings: Vec<String>,
    types: Vec<String>,
    prototypes: Vec<Prototype>,
    fields: Vec<Field>,
    methods: Vec<Method>,
    classes: Vec<ClassDef>,
}

// TODO check alignments
impl Dex {
    /// Reads the Dex data structure from the given path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Dex> {
        let file = try!(fs::File::open(path));
        let file_size = try!(file.metadata()).len();
        if file_size < HEADER_SIZE as u64 || file_size > u32::MAX as u64 {
            return Err(Error::invalid_file_size(file_size, None));
        }
        Dex::from_reader(BufReader::new(file), file_size)
    }

    /// Loads a new Dex data structure from the given reader.
    pub fn from_reader<R: Read>(mut reader: R, file_size: u64) -> Result<Dex> {
        let header = try!(Header::from_reader(&mut reader));
        if header.get_file_size() as u64 != file_size {
            return Err(Error::invalid_file_size(file_size as u64, Some(header.get_file_size())));
        }

        if header.is_little_endian() {
            Dex::read_data::<_, LittleEndian>(reader, header)
        } else {
            Dex::read_data::<_, BigEndian>(reader, header)
        }
    }

    /// Reads the *dex* file data with the given byte order after reading the header.
    fn read_data<R: Read, E: ByteOrder>(mut reader: R, header: Header) -> Result<Dex> {
        let mut offset = HEADER_SIZE;
        // We will store all offsets in one map. This enables us to do sequencial reading even if
        // offsets are not in the correct order.
        // It could in any case happen that the ofset we are currently reading is not found in the
        // offsets, which would mean that we have unknown data, which will be saved in a byte vector
        // for later use.
        let mut offset_map = header.generate_offset_map();
        // Here we will store the unknown data we find:
        let mut unknown_data = Vec::new();
        // Initialize lists:
        // String data offsets
        let mut string_ids = Vec::with_capacity(header.get_string_ids_size());
        // Indexes of type names in string ids.
        let mut type_ids = Vec::with_capacity(header.get_type_ids_size());
        let mut prototype_ids = Vec::with_capacity(header.get_prototype_ids_size());
        let mut field_ids = Vec::with_capacity(header.get_field_ids_size());
        let mut method_ids = Vec::with_capacity(header.get_method_ids_size());
        let mut class_defs = Vec::with_capacity(header.get_class_defs_size());

        let mut map = None;
        let mut type_lists = Vec::new();
        let mut annotation_sets = Vec::new();

        let read_end = if let Some(offset) = header.get_link_offset() {
            offset
        } else {
            header.get_file_size()
        };

        // Read all the file.
        while offset < read_end {
            let offset_type = match offset_map.get_offset(offset) {
                Ok(offset_type) => offset_type,
                Err(Some((next_offset, offset_type))) => {
                    if next_offset >= offset + 4 && cfg!(feature = "debug") {
                        println!("{} unknown bytes were found in the offset {:#010x}.",
                                 next_offset - offset,
                                 offset)
                    }
                    let mut unknown_bytes = Vec::with_capacity((next_offset - offset) as usize);
                    try!(reader.by_ref()
                        .take((next_offset - offset) as u64)
                        .read_to_end(&mut unknown_bytes));
                    unknown_data.push((offset, unknown_bytes.into_boxed_slice()));
                    offset = next_offset;
                    offset_type
                }
                _ => break,
            };
            match offset_type {
                OffsetType::StringIdList => {
                    // Read all string offsets
                    for _ in 0..header.get_string_ids_size() {
                        let offset = try!(reader.read_u32::<E>());
                        offset_map.insert(offset, OffsetType::StringData);
                        string_ids.push((offset, None::<String>));
                    }
                    offset += STRING_ID_ITEM_SIZE * header.get_string_ids_size() as u32;
                }
                OffsetType::TypeIdList => {
                    // Read all type string indexes
                    for _ in 0..header.get_type_ids_size() {
                        type_ids.push(try!(reader.read_u32::<E>()) as usize);
                    }
                    offset += TYPE_ID_ITEM_SIZE * header.get_type_ids_size() as u32;
                }
                OffsetType::PrototypeIdList => {
                    // Read all prototype IDs
                    for _ in 0..header.get_prototype_ids_size() {
                        let shorty_id = try!(reader.read_u32::<E>());
                        let return_type_id = try!(reader.read_u32::<E>());
                        let parameters_offset = try!(reader.read_u32::<E>());
                        offset_map.insert(parameters_offset, OffsetType::TypeList);
                        prototype_ids.push(PrototypeIdData::new(shorty_id,
                                                                return_type_id,
                                                                parameters_offset));
                    }
                    offset += PROTO_ID_ITEM_SIZE * header.get_prototype_ids_size() as u32;
                }
                OffsetType::FieldIdList => {
                    // Read all field IDs
                    for _ in 0..header.get_field_ids_size() {
                        let class_id = try!(reader.read_u16::<E>());
                        let type_id = try!(reader.read_u16::<E>());
                        let name_id = try!(reader.read_u32::<E>());
                        field_ids.push(FieldIdData::new(class_id, type_id, name_id));
                    }
                    offset += FIELD_ID_ITEM_SIZE * header.get_field_ids_size() as u32;
                }
                OffsetType::MethodIdList => {
                    // Read all method IDs
                    for _ in 0..header.get_method_ids_size() {
                        let class_id = try!(reader.read_u16::<E>());
                        let prototype_id = try!(reader.read_u16::<E>());
                        let name_id = try!(reader.read_u32::<E>());
                        method_ids.push(MethodIdData::new(class_id, prototype_id, name_id));
                    }
                    offset += METHOD_ID_ITEM_SIZE * header.get_method_ids_size() as u32;
                }
                OffsetType::ClassDefList => {
                    // Read all class definitions
                    for _ in 0..header.get_class_defs_size() {
                        let class_id = try!(reader.read_u32::<E>());
                        let access_flags = try!(reader.read_u32::<E>());
                        let superclass_id = try!(reader.read_u32::<E>());
                        let interfaces_offset = try!(reader.read_u32::<E>());
                        let source_file_id = try!(reader.read_u32::<E>());
                        let annotations_offset = try!(reader.read_u32::<E>());
                        let class_data_offset = try!(reader.read_u32::<E>());
                        let static_values_offset = try!(reader.read_u32::<E>());
                        class_defs.push(try!(ClassDefData::new(class_id,
                                                               access_flags,
                                                               superclass_id,
                                                               interfaces_offset,
                                                               source_file_id,
                                                               annotations_offset,
                                                               class_data_offset,
                                                               static_values_offset)));
                        if interfaces_offset != 0 {
                            offset_map.insert(interfaces_offset, OffsetType::TypeList);
                        }
                        if annotations_offset != 0 {
                            offset_map.insert(annotations_offset, OffsetType::AnnotationsDirectory);
                        }
                        if class_data_offset != 0 {
                            offset_map.insert(class_data_offset, OffsetType::ClassData);
                        }
                        if static_values_offset != 0 {
                            offset_map.insert(class_data_offset, OffsetType::EncodedArray);
                        }
                    }
                    offset += CLASS_DEF_ITEM_SIZE * header.get_class_defs_size() as u32;
                }
                OffsetType::Map => {
                    // Read map
                    map = Some(try!(Map::from_reader::<_, E>(&mut reader, &mut offset_map)));
                    offset += 4 +
                              MAP_ITEM_SIZE * map.as_ref().unwrap().get_item_list().len() as u32;
                }
                OffsetType::TypeList => {
                    let num_type_lists =
                        map.as_ref().unwrap().get_num_items_for(ItemType::TypeList).unwrap();
                    type_lists.reserve_exact(num_type_lists);
                    for _ in 0..num_type_lists {
                        let size = try!(reader.read_u32::<E>());
                        let mut type_list = Vec::with_capacity(size as usize);
                        for _ in 0..size {
                            type_list.push(try!(reader.read_u16::<E>()));
                        }
                        type_lists.push(type_list);
                        offset += 4 + TYPE_ITEM_SIZE * size;
                        if size & 0b1 != 0 {
                            try!(reader.read_exact(&mut [0u8; TYPE_ITEM_SIZE as usize]));
                            offset += TYPE_ITEM_SIZE;
                        }
                    }
                }
                OffsetType::AnnotationSetList => {
                    let num_anotation_sets = map.as_ref()
                        .unwrap()
                        .get_num_items_for(ItemType::AnnotationSetList)
                        .unwrap();
                    annotation_sets.reserve_exact(num_anotation_sets);
                    for _ in 0..num_anotation_sets {
                        let size = try!(reader.read_u32::<E>());
                        let mut annotation_set = Vec::with_capacity(size as usize);
                        for _ in 0..size {
                            annotation_set.push((try!(reader.read_u32::<E>()), None::<u32>));
                        }
                        annotation_sets.push(annotation_set);
                        offset += 4 + ANNOTATION_SET_REF_SIZE * size;
                    }
                }
                OffsetType::AnnotationSet => unimplemented!(),
                OffsetType::Annotation => unimplemented!(),
                OffsetType::AnnotationsDirectory => unimplemented!(),
                OffsetType::ClassData => unimplemented!(),
                OffsetType::Code => unimplemented!(),
                OffsetType::StringData => unimplemented!(),
                OffsetType::DebugInfo => unimplemented!(),
                OffsetType::EncodedArray => unimplemented!(),
                OffsetType::Link => unimplemented!(),
            }
        }
        // TODO search unknown data for offsets. Maybe an iterator with bounds.
        // That would only require 2 binary searches and one slicing, and then, an iterator.

        // let mut annotation_set_refs = Vec::with_capacity(0);
        // let mut annotation_sets = Vec::with_capacity(0);
        // for map_item in map.get_map_list() {
        //     match map_item.get_item_type() {
        //         ItemType::HeaderItem | ItemType::StringIdItem | ItemType::TypeIdItem |
        //         ItemType::ProtoIdItem | ItemType::FieldIdItem | ItemType::MethodIdItem |
        //         ItemType::ClassDefItem | ItemType::MapList | ItemType::TypeList => {}
        //         ItemType::AnnotationSetRefList => {
        //             if map_item.get_num_items() > 0 {
        //                 annotation_set_refs.reserve_exact(map_item.get_num_items());
        //                 for _ in 0..map_item.get_num_items() {
        //                     try!(reader.seek(SeekFrom::Start(map_item.get_offset() as u64)));
        //                     let list_size = try!(if header.is_little_endian() {
        //                         reader.read_u32::<LittleEndian>()
        //                     } else {
        //                         reader.read_u32::<BigEndian>()
        //                     }) as usize;
        //                     let mut annotations_offset = Vec::with_capacity(list_size);
        //                     for _ in 0..list_size {
        //                         let offset = try!(if header.is_little_endian() {
        //                             reader.read_u32::<LittleEndian>()
        //                         } else {
        //                             reader.read_u32::<BigEndian>()
        //                         }) as usize;
        //                         annotations_offset.push(if offset != 0 {Some(offset)} else
        // {None});
        //                     }
        //                 }
        //             }
        //         }
        //         ItemType::AnnotationSetItem => {
        //             if map_item.get_num_items() > 0 {
        //                 annotation_set_list.reserve_exact(map_item.get_num_items());
        //                 for _ in 0..map_item.get_num_items() {
        //                     try!(reader.seek(SeekFrom::Start(map_item.get_offset() as u64)));
        //                     let list_size = try!(if header.is_little_endian() {
        //                         reader.read_u32::<LittleEndian>()
        //                     } else {
        //                         reader.read_u32::<BigEndian>()
        //                     }) as usize;
        //                     let mut annotations_offset = Vec::with_capacity(list_size);
        //                     for _ in 0..list_size {
        //                         let offset = try!(if header.is_little_endian() {
        //                             reader.read_u32::<LittleEndian>()
        //                         } else {
        //                             reader.read_u32::<BigEndian>()
        //                         }) as usize;
        //                         annotations_offset.push(if offset != 0 {Some(offset)} else
        // {None});
        //                     }
        //                 }
        //             }
        //         }
        //         ItemType::ClassDataItem => unimplemented!(),
        //         ItemType::CodeItem => unimplemented!(),
        //         ItemType::StringDataItem => unimplemented!(),
        //         ItemType::DebugInfoItem => unimplemented!(),
        //         ItemType::AnnotationItem => unimplemented!(),
        //         ItemType::EncodedArrayItem => unimplemented!(),
        //         ItemType::AnnotationsDirectoryItem => unimplemented!(),
        //     }
        // }

        unimplemented!()
    }

    /// Ads the file in the given path to the current Dex data structure.
    pub fn add_file<P: AsRef<Path>>(_path: P) -> Result<()> {
        unimplemented!()
    }

    /// Verifies the file at the given path.
    pub fn verify_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.header.verify_file(path)
    }

    /// Verifies the file in the given reader.
    ///
    /// The reader should be positioned at the start of the file.
    pub fn verify_reader<R: Read>(&self, reader: R) -> bool {
        self.header.verify_reader(reader)
    }
}

#[derive(Debug)]
struct Map {
    map_list: Vec<MapItem>,
}

impl Map {
    pub fn from_reader<R: Read, B: ByteOrder>(reader: &mut R,
                                              offset_map: &mut OffsetMap)
                                              -> Result<Map> {
        let size = try!(reader.read_u32::<B>());
        let mut map_list = Vec::with_capacity(size as usize);
        offset_map.reserve_exact(size as usize);
        for _ in 0..size {
            let item_type = try!(reader.read_u16::<B>());
            try!(reader.read_exact(&mut [0u8; 2]));
            let size = try!(reader.read_u32::<B>());
            let offset = try!(reader.read_u32::<B>());
            let map_item = try!(MapItem::new(item_type, size, offset));
            match map_item.get_item_type() {
                ItemType::Header | ItemType::StringId | ItemType::TypeId | ItemType::ProtoId |
                ItemType::FieldId | ItemType::MethodId | ItemType::ClassDef | ItemType::Map => {}
                ItemType::TypeList => {
                    offset_map.insert(offset, OffsetType::TypeList);
                }
                ItemType::AnnotationSetList => {
                    offset_map.insert(offset, OffsetType::AnnotationSetList);
                }
                ItemType::AnnotationSet => {
                    offset_map.insert(offset, OffsetType::AnnotationSet);
                }
                ItemType::ClassData => {
                    offset_map.insert(offset, OffsetType::ClassData);
                }
                ItemType::Code => {
                    offset_map.insert(offset, OffsetType::Code);
                }
                ItemType::StringData => {
                    offset_map.insert(offset, OffsetType::StringData);
                }
                ItemType::DebugInfo => {
                    offset_map.insert(offset, OffsetType::DebugInfo);
                }
                ItemType::Annotation => {
                    offset_map.insert(offset, OffsetType::Annotation);
                }
                ItemType::EncodedArray => {
                    offset_map.insert(offset, OffsetType::EncodedArray);
                }
                ItemType::AnnotationsDirectory => {
                    offset_map.insert(offset, OffsetType::AnnotationsDirectory);
                }
            }
            map_list.push(map_item);
        }
        map_list.sort_by_key(|i| i.get_offset());
        Ok(Map { map_list: map_list })
    }

    pub fn get_item_list(&self) -> &[MapItem] {
        &self.map_list
    }

    pub fn get_num_items_for(&self, item_type: ItemType) -> Option<usize> {
        for item in &self.map_list {
            if item.get_item_type() == item_type {
                return Some(item.get_num_items());
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Prototype {
    // TODO;
}

#[derive(Debug)]
pub struct Field {
    // TODO;
}

#[derive(Debug)]
pub struct Method {
    // TODO;
}

#[derive(Debug)]
pub struct ClassDef {
    // TODO;
}
