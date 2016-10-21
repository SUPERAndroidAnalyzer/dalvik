extern crate byteorder;
#[macro_use]
extern crate bitflags;

use std::path::Path;
use std::{fs, u32};
use std::io::prelude::*;
use std::io::BufReader;

use byteorder::{BigEndian, LittleEndian, ByteOrder, ReadBytesExt};

pub mod error;
// pub mod bytecode; // TODO: not in use
pub mod types; // TODO: Should not be public
pub mod sizes; // TODO: Should not be public
pub mod offset_map; // TODO: Should not be public
pub mod header;

use error::{Result, Error};
use sizes::*;
pub use header::Header;
use types::{PrototypeIdData, FieldIdData, MethodIdData, ClassDefData, MapItem, ItemType,
            AnnotationItem, Array, AnnotationsDirectory};
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

        let mut type_lists = Vec::new();
        let mut annotation_set_ref_lists = Vec::new();
        let mut annotation_sets = Vec::new();
        let mut annotations = Vec::new();
        let mut arrays = Vec::new();
        let mut annotations_directories = Vec::new();

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
                        type_ids.push(try!(reader.read_u32::<E>()));
                    }
                    offset += TYPE_ID_ITEM_SIZE * header.get_type_ids_size() as u32;
                }
                OffsetType::PrototypeIdList => {
                    // Read all prototype IDs
                    for _ in 0..header.get_prototype_ids_size() {
                        let prototype_id = try!(PrototypeIdData::from_reader::<_, E>(&mut reader));
                        if let Some(offset) = prototype_id.get_parameters_offset() {
                            offset_map.insert(offset, OffsetType::TypeList);
                        }
                        prototype_ids.push(prototype_id);
                    }
                    offset += PROTO_ID_ITEM_SIZE * header.get_prototype_ids_size() as u32;
                }
                OffsetType::FieldIdList => {
                    // Read all field IDs
                    for _ in 0..header.get_field_ids_size() {
                        field_ids.push(try!(FieldIdData::from_reader::<_, E>(&mut reader)));
                    }
                    offset += FIELD_ID_ITEM_SIZE * header.get_field_ids_size() as u32;
                }
                OffsetType::MethodIdList => {
                    // Read all method IDs
                    for _ in 0..header.get_method_ids_size() {
                        method_ids.push(try!(MethodIdData::from_reader::<_, E>(&mut reader)));
                    }
                    offset += METHOD_ID_ITEM_SIZE * header.get_method_ids_size() as u32;
                }
                OffsetType::ClassDefList => {
                    // Read all class definitions
                    for _ in 0..header.get_class_defs_size() {
                        let class_def = try!(ClassDefData::from_reader::<_, E>(&mut reader));
                        if let Some(offset) = class_def.get_interfaces_offset() {
                            offset_map.insert(offset, OffsetType::TypeList);
                        }
                        if let Some(offset) = class_def.get_annotations_offset() {
                            offset_map.insert(offset, OffsetType::AnnotationsDirectory);
                        }
                        if let Some(offset) = class_def.get_class_data_offset() {
                            offset_map.insert(offset, OffsetType::ClassData);
                        }
                        if let Some(offset) = class_def.get_static_values_offset() {
                            offset_map.insert(offset, OffsetType::EncodedArray);
                        }
                        class_defs.push(class_def);
                    }
                    offset += CLASS_DEF_ITEM_SIZE * header.get_class_defs_size() as u32;
                }
                OffsetType::Map => {
                    // Read map
                    let map = try!(Map::from_reader::<_, E>(&mut reader, &mut offset_map));
                    if let Some(count) = map.get_num_items_for(ItemType::TypeList) {
                        type_lists.reserve_exact(count);
                    }
                    if let Some(count) = map.get_num_items_for(ItemType::AnnotationSetList) {
                        annotation_set_ref_lists.reserve_exact(count);
                    }
                    if let Some(count) = map.get_num_items_for(ItemType::AnnotationSet) {
                        annotation_sets.reserve_exact(count);
                    }
                    if let Some(count) = map.get_num_items_for(ItemType::Annotation) {
                        annotations.reserve_exact(count);
                    }
                    if let Some(count) = map.get_num_items_for(ItemType::EncodedArray) {
                        arrays.reserve_exact(count);
                    }
                    if let Some(count) = map.get_num_items_for(ItemType::AnnotationsDirectory) {
                        annotations_directories.reserve_exact(count);
                    }
                    offset += 4 + MAP_ITEM_SIZE * map.get_item_list().len() as u32;
                }
                OffsetType::TypeList => {
                    let size = try!(reader.read_u32::<E>());

                    let mut type_list = Vec::with_capacity(size as usize);
                    for _ in 0..size {
                        type_list.push(try!(reader.read_u16::<E>()));
                    }
                    type_lists.push(type_list);

                    offset += 4 + TYPE_ITEM_SIZE * size;
                    if size & 0b1 != 0 {
                        // Align misaligned section
                        try!(reader.read_exact(&mut [0u8; TYPE_ITEM_SIZE as usize]));
                        offset += TYPE_ITEM_SIZE;
                    }
                }
                OffsetType::AnnotationSetList => {
                    let size = try!(reader.read_u32::<E>());
                    let mut annotation_set_list = Vec::with_capacity(size as usize);

                    for _ in 0..size {
                        let annotation_set_offset = try!(reader.read_u32::<E>());
                        offset_map.insert(annotation_set_offset, OffsetType::AnnotationSet);
                        annotation_set_list.push(annotation_set_offset);
                    }
                    annotation_set_ref_lists.push(annotation_set_list);

                    offset += 4 + ANNOTATION_SET_REF_SIZE * size;
                }
                OffsetType::AnnotationSet => {
                    let size = try!(reader.read_u32::<E>());
                    let mut annotation_set = Vec::with_capacity(size as usize);

                    for _ in 0..size {
                        let annotation_offset = try!(reader.read_u32::<E>());
                        offset_map.insert(annotation_offset, OffsetType::Annotation);
                        annotation_set.push(annotation_offset);
                    }
                    annotation_sets.push(annotation_set);

                    offset += 4 + ANNOTATION_SET_ITEM_SIZE * size;
                }
                OffsetType::ClassData => {
                    let mut byte = [0];
                    try!(reader.read_exact(&mut byte));
                    offset += 1;
                }//unimplemented!(),
                OffsetType::Code => {
                    let mut byte = [0];
                    try!(reader.read_exact(&mut byte));
                    offset += 1;
                }//unimplemented!(),
                OffsetType::StringData => {
                    let mut byte = [0];
                    try!(reader.read_exact(&mut byte));
                    offset += 1;
                }//unimplemented!(),
                OffsetType::DebugInfo => {
                    let mut byte = [0];
                    try!(reader.read_exact(&mut byte));
                    offset += 1;
                }//unimplemented!(),
                OffsetType::Annotation => {
                    let (annotation, read) = try!(AnnotationItem::from_reader(&mut reader));
                    annotations.push((offset, annotation));
                    offset += read;
                }
                OffsetType::EncodedArray => {
                    let (array, read) = try!(Array::from_reader(&mut reader));
                    arrays.push(array);
                    offset += read;
                }
                OffsetType::AnnotationsDirectory => {
                    let directory = try!(AnnotationsDirectory::from_reader::<_, E>(&mut reader));
                    let read = 4 * 4 + directory.get_field_annotations().len() * 8 +
                               directory.get_method_annotations().len() * 8 +
                               directory.get_parameter_annotations().len() * 8;
                    annotations_directories.push((offset, directory));
                    offset += read as u32;
                }
                OffsetType::Link => unreachable!(),
            }
        }
        println!("Read OK!");
        // TODO search unknown data for offsets. Maybe an iterator with bounds.
        // That would only require 2 binary searches and one slicing, and then, an iterator.

        // TODO generate final data structure.

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

/// The struct representing the *dex* file Map.
#[derive(Debug)]
struct Map {
    map_list: Vec<MapItem>,
}

impl Map {
    pub fn from_reader<R: Read, E: ByteOrder>(reader: &mut R,
                                              offset_map: &mut OffsetMap)
                                              -> Result<Map> {
        let size = try!(reader.read_u32::<E>());
        let mut map_list = Vec::with_capacity(size as usize);
        offset_map.reserve_exact(size as usize);
        for _ in 0..size {
            let map_item = try!(MapItem::from_reader::<_, E>(reader));
            match map_item.get_item_type() {
                ItemType::Header | ItemType::StringId | ItemType::TypeId | ItemType::ProtoId |
                ItemType::FieldId | ItemType::MethodId | ItemType::ClassDef | ItemType::Map => {}
                ItemType::TypeList => {
                    offset_map.insert(map_item.get_offset(), OffsetType::TypeList);
                }
                ItemType::AnnotationSetList => {
                    offset_map.insert(map_item.get_offset(), OffsetType::AnnotationSetList);
                }
                ItemType::AnnotationSet => {
                    offset_map.insert(map_item.get_offset(), OffsetType::AnnotationSet);
                }
                ItemType::ClassData => {
                    offset_map.insert(map_item.get_offset(), OffsetType::ClassData);
                }
                ItemType::Code => {
                    offset_map.insert(map_item.get_offset(), OffsetType::Code);
                }
                ItemType::StringData => {
                    offset_map.insert(map_item.get_offset(), OffsetType::StringData);
                }
                ItemType::DebugInfo => {
                    offset_map.insert(map_item.get_offset(), OffsetType::DebugInfo);
                }
                ItemType::Annotation => {
                    offset_map.insert(map_item.get_offset(), OffsetType::Annotation);
                }
                ItemType::EncodedArray => {
                    offset_map.insert(map_item.get_offset(), OffsetType::EncodedArray);
                }
                ItemType::AnnotationsDirectory => {
                    offset_map.insert(map_item.get_offset(), OffsetType::AnnotationsDirectory);
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
