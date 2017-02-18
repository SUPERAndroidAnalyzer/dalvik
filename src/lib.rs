// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate byteorder;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate error_chain;

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

use error::*;
use sizes::*;
pub use header::Header;
use types::{PrototypeIdData, FieldIdData, MethodIdData, ClassDefData, MapItem, ItemType,
            AnnotationItem, Array, AnnotationsDirectory, ClassData, StringReader, DebugInfo,
            CodeItem};
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
        let file = fs::File::open(path).chain_err(|| "could not open file")?;
        let file_size = file.metadata().chain_err(|| "could not read file metadata")?.len();
        if file_size < HEADER_SIZE as u64 || file_size > u32::MAX as u64 {
            return Err(ErrorKind::InvalidFileSize(file_size).into());
        }
        Dex::from_reader(BufReader::new(file), file_size)
    }

    /// Loads a new Dex data structure from the given reader.
    pub fn from_reader<R: BufRead>(mut reader: R, file_size: u64) -> Result<Dex> {
        let header = Header::from_reader(&mut reader)?;
        if header.get_file_size() as u64 != file_size {
            return Err(ErrorKind::HeaderFileSizeMismatch(file_size, header.get_file_size()).into());
        }

        if header.is_little_endian() {
            Dex::read_data::<_, LittleEndian>(reader, header)
        } else {
            Dex::read_data::<_, BigEndian>(reader, header)
        }
    }

    /// Reads the *dex* file data with the given byte order after reading the header.
    #[allow(cyclomatic_complexity)]
    fn read_data<R: BufRead, E: ByteOrder>(mut reader: R, header: Header) -> Result<Dex> {
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
        let mut classes = Vec::new();
        let mut code_segments = Vec::new();
        let mut strings = Vec::new();
        let mut debug_infos = Vec::new();
        let mut annotations = Vec::new();
        let mut arrays = Vec::new();
        let mut annotations_directories = Vec::new();

        let read_end = if let Some(offset) = header.get_link_offset() {
            offset
        } else {
            header.get_file_size()
        };

        // Read all the file.
        while offset < read_end || !offset_map.is_empty() {
            let offset_type = if offset < read_end {
                match offset_map.get_offset(offset) {
                    Ok(offset_type) => offset_type,
                    Err(Some((next_offset, offset_type))) => {
                        if cfg!(feature = "debug") && next_offset >= offset + 4 {
                            println!("{} unknown bytes were found in the offset {:#010x}.",
                                     next_offset - offset,
                                     offset)
                        }
                        let mut unknown_bytes = Vec::with_capacity((next_offset - offset) as usize);
                        reader.by_ref()
                            .take((next_offset - offset) as u64)
                            .read_to_end(&mut unknown_bytes)
                            .chain_err(|| "could not read unknown bytes")?;
                        unknown_data.push((offset, unknown_bytes.into_boxed_slice()));
                        offset = next_offset;
                        offset_type
                    }
                    _ => break,
                }
            } else {
                println!("{} unused offsets found in offset map:", offset_map.len());
                break;
                // unimplemented!()
            };

            match offset_type {
                OffsetType::StringIdList => {
                    // Read all string offsets.
                    for _ in 0..header.get_string_ids_size() {
                        let offset = reader.read_u32::<E>().chain_err(|| {
                            format!("could not read string offset from string ID list at offset \
                                     {:#010x}",
                                    offset)
                        })?;
                        offset_map.insert(offset, OffsetType::StringData);
                        string_ids.push((offset, None::<String>));
                    }
                    offset += STRING_ID_ITEM_SIZE * header.get_string_ids_size() as u32;
                }
                OffsetType::TypeIdList => {
                    // Read all type string indexes.
                    for _ in 0..header.get_type_ids_size() {
                        type_ids.push(reader.read_u32::<E>().chain_err(|| {
                            format!("could not read type ID from type ID list at offset {:#010x}",
                                    offset)
                        })?);
                    }
                    offset += TYPE_ID_ITEM_SIZE * header.get_type_ids_size() as u32;
                }
                OffsetType::PrototypeIdList => {
                    // Read all prototype IDs.
                    for _ in 0..header.get_prototype_ids_size() {
                        let prototype_id =
                            PrototypeIdData::from_reader::<_, E>(&mut reader).chain_err(|| {
                                    format!("could not read prototype ID from prototype ID list at \
                                             offset {:#010x}",
                                            offset)
                                })?;
                        if let Some(offset) = prototype_id.get_parameters_offset() {
                            offset_map.insert(offset, OffsetType::TypeList);
                        }
                        prototype_ids.push(prototype_id);
                    }
                    offset += PROTO_ID_ITEM_SIZE * header.get_prototype_ids_size() as u32;
                }
                OffsetType::FieldIdList => {
                    // Read all field IDs.
                    for _ in 0..header.get_field_ids_size() {
                        field_ids.push(FieldIdData::from_reader::<_, E>(&mut reader).chain_err(|| {
                            format!("could not read field ID from field ID list at offset {:#010x}",
                                    offset)
                        })?);
                    }
                    offset += FIELD_ID_ITEM_SIZE * header.get_field_ids_size() as u32;
                }
                OffsetType::MethodIdList => {
                    // Read all method IDs.
                    for _ in 0..header.get_method_ids_size() {
                        method_ids.push(MethodIdData::from_reader::<_, E>(&mut reader)
                            .chain_err(|| {
                                format!("could not read method ID from method ID list at offset \
                                         {:#010x}", offset)
                            })?);
                    }
                    offset += METHOD_ID_ITEM_SIZE * header.get_method_ids_size() as u32;
                }
                OffsetType::ClassDefList => {
                    // Read all class definitions.
                    for _ in 0..header.get_class_defs_size() {
                        let class_def = ClassDefData::from_reader::<_, E>(&mut reader)
                            .chain_err(|| {
                                format!("could not read class definition data at offset {:#010x}",
                                        offset)
                            })?;
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
                    // Read map.
                    let map = Map::from_reader::<_, E>(&mut reader, &mut offset_map).chain_err(|| {
                            format!("error reading map at offset {:#010x}", offset)
                        })?;
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
                    // Read type list.
                    let size = reader.read_u32::<E>()
                        .chain_err(|| {
                            format!("error reading type list size size at offset {:#010x}",
                                    offset)
                        })?;

                    let mut type_list = Vec::with_capacity(size as usize);
                    for i in 0..size {
                        type_list.push(reader.read_u16::<E>().chain_err(|| {
                            format!("error reading type ID for type item at index {} at type list \
                                     at offset {:#010x}", i, offset)
                        })?);
                    }
                    type_lists.push(type_list);

                    offset += 4 + TYPE_ITEM_SIZE * size;
                    if size & 0b1 != 0 {
                        // Align misaligned section
                        reader.read_exact(&mut [0u8; TYPE_ITEM_SIZE as usize])
                            .chain_err(|| {
                                format!("error aligning misaligned type list at offset {:#010x}",
                                        offset)
                            })?;
                        offset += TYPE_ITEM_SIZE;
                    }
                }
                OffsetType::AnnotationSetList => {
                    // Read anotation set list.
                    let size = reader.read_u32::<E>()
                        .chain_err(|| {
                            format!("error reading anotation set list size at offset {:#010x}",
                                    offset)
                        })?;
                    let mut annotation_set_list = Vec::with_capacity(size as usize);

                    for _ in 0..size {
                        let annotation_set_offset = reader.read_u32::<E>().chain_err(|| {
                            format!("could not read annotation set offset for an anotation set in \
                                     the anotation set list at offset {:#010x}", offset)
                        })?;
                        offset_map.insert(annotation_set_offset, OffsetType::AnnotationSet);
                        annotation_set_list.push(annotation_set_offset);
                    }
                    annotation_set_ref_lists.push(annotation_set_list);

                    offset += 4 + ANNOTATION_SET_REF_SIZE * size;
                }
                OffsetType::AnnotationSet => {
                    // Read annotation set.
                    let size = reader.read_u32::<E>()
                        .chain_err(|| {
                            format!("error reading anotation set size at offset {:#010x}",
                                    offset)
                        })?;
                    let mut annotation_set = Vec::with_capacity(size as usize);

                    for i in 0..size {
                        let annotation_offset = reader.read_u32::<E>()
                            .chain_err(|| {
                                format!("error reading anotation offset at index {} in anotation \
                                         set at offset {:#010x}",
                                        i,
                                        offset)
                            })?;
                        offset_map.insert(annotation_offset, OffsetType::Annotation);
                        annotation_set.push(annotation_offset);
                    }
                    annotation_sets.push(annotation_set);

                    offset += 4 + ANNOTATION_SET_ITEM_SIZE * size;
                }
                OffsetType::ClassData => {
                    // Read class data.
                    let (class_data, read) =
                        ClassData::from_reader(&mut reader, &mut offset_map).chain_err(|| {
                                format!("could not read class data at offset {:#010x}", offset)
                            })?;
                    classes.push((offset, class_data));
                    offset += read;
                }
                OffsetType::Code => {
                    // Read code.
                    let (code_item, read) =
                        CodeItem::from_reader::<_, E>(&mut reader, &mut offset_map).chain_err(|| {
                                format!("could not read code item at offset {:#010x}", offset)
                            })?;
                    code_segments.push((offset, code_item));

                    offset += read;
                }
                OffsetType::StringData => {
                    // Read string data
                    let (string, read) = StringReader::read_string(&mut reader).chain_err(|| {
                            format!("could not read string data at offset {:#010x}", offset)
                        })?;
                    strings.push((offset, string));
                    offset += read;
                }
                OffsetType::DebugInfo => {
                    // Read debug information.
                    let (debug_info, read) = DebugInfo::from_reader(&mut reader).chain_err(|| {
                            format!("could not read debug information at offset {:#010x}",
                                    offset)
                        })?;
                    debug_infos.push((offset, debug_info));
                    offset += read;
                }
                OffsetType::Annotation => {
                    // Read anotation.
                    let (annotation, read) = AnnotationItem::from_reader(&mut reader).chain_err(|| {
                            format!("could not read annotation at offset {:#010x}", offset)
                        })?;
                    annotations.push((offset, annotation));
                    offset += read;
                }
                OffsetType::EncodedArray => {
                    // Read encoded array.
                    let (array, read) = Array::from_reader(&mut reader).chain_err(|| {
                            format!("could not read encoded array at offset {:#010x}", offset)
                        })?;
                    arrays.push(array);
                    offset += read;
                }
                OffsetType::AnnotationsDirectory => {
                    // Read annotations directory.
                    // println!("Anotations directory found at offset {:#010x}", offset);
                    let directory =
                        AnnotationsDirectory::from_reader::<_, E>(&mut reader, &mut offset_map)
                            .chain_err(|| {
                                format!("could not read annotation directory at offset {:#010x}",
                                        offset)
                            })?;
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
        let size = reader.read_u32::<E>().chain_err(|| "could not read map list size")?;
        let mut map_list = Vec::with_capacity(size as usize);
        offset_map.reserve(size as usize);
        for _ in 0..size {
            let map_item =
                MapItem::from_reader::<_, E>(reader).chain_err(|| "could not read map item")?;
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
