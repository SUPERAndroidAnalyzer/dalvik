//! Dex file reader module.

mod types;

use std::io::{BufRead, Read};
use byteorder::{ByteOrder, ReadBytesExt};

use header::Header;
use error::*;
use offset_map::{OffsetMap, OffsetType};
use self::types::*;
use sizes::*;

/// Structure for reading a Dex file in a fast way.
#[derive(Debug)]
pub struct DexReader<R: BufRead> {
    /// Reader to use to read data.
    ///
    /// Header must already be read before creating the object.
    reader: R,
    /// Current reading offset.
    current_offset: u32,
    /// Header of the dex file.
    header: Header,
    /// Map to store all known offsets in the Dex file.
    offset_map: OffsetMap,
    /// Vector of chunks of binary data that were not understood when reading sequencially.
    unknown_data: Vec<(u32, Box<[u8]>)>,
    /// String ID list.
    ///
    /// This list creates 1:1 relations between String indexes and string offsets in the dex file.
    string_ids: Vec<u32>,
    /// Type ID list.
    ///
    /// This list creates 1:1 relations between type indexes and type descriptor string index.
    type_ids: Vec<u32>,
    /// Prototype ID list.
    prototype_ids: Vec<PrototypeIdData>,
    /// Field ID list.
    ///
    /// This list creates 1:1 relations between field indexes and field information offsets in the
    /// dex file.
    field_ids: Vec<FieldIdData>,
    /// Method ID list.
    method_ids: Vec<MethodIdData>,
    /// Class definition list
    class_defs: Vec<ClassDefData>,
    /// Type list.
    ///
    /// Each element is the index in the type ID list for the type.
    type_lists: Vec<Box<[u16]>>,
    /// List of lists of references to annotation set offsets.
    annotation_set_ref_list: Vec<Box<[u32]>>,
    /// Set of annotations.
    annotation_sets: Vec<Box<[u32]>>,
    /// Class data list.
    classes: Vec<(u32, ClassData)>,
    /// Code segment list.
    code_segments: Vec<(u32, CodeItem)>,
    /// String list.
    strings: Vec<(u32, String)>,
    /// Debug information list.
    debug_info: Vec<(u32, DebugInfo)>,
    /// Annotation list.
    annotations: Vec<(u32, AnnotationItem)>,
    /// Array list.
    arrays: Vec<(u32, Array)>,
    /// Annotations directories.
    annotations_directories: Vec<(u32, AnnotationsDirectory)>,
}

impl<R> DexReader<R>
    where R: BufRead
{
    /// Creates a new reader with the information from the header of the file.
    pub fn new(header: Header, reader: R) -> DexReader<R> {
        let offset_map = header.generate_offset_map();
        let string_ids = Vec::with_capacity(header.get_string_ids_size());
        let type_ids = Vec::with_capacity(header.get_type_ids_size());
        let prototype_ids = Vec::with_capacity(header.get_prototype_ids_size());
        let field_ids = Vec::with_capacity(header.get_field_ids_size());
        let method_ids = Vec::with_capacity(header.get_method_ids_size());
        let class_defs = Vec::with_capacity(header.get_class_defs_size());
        DexReader {
            reader: reader,
            current_offset: HEADER_SIZE,
            header: header,
            offset_map: offset_map,
            unknown_data: Vec::new(),
            string_ids: string_ids,
            type_ids: type_ids,
            prototype_ids: prototype_ids,
            field_ids: field_ids,
            method_ids: method_ids,
            class_defs: class_defs,
            type_lists: Vec::new(),
            annotation_set_ref_list: Vec::new(),
            annotation_sets: Vec::new(),
            classes: Vec::new(),
            code_segments: Vec::new(),
            strings: Vec::new(),
            debug_info: Vec::new(),
            annotations: Vec::new(),
            arrays: Vec::new(),
            annotations_directories: Vec::new(),
        }
    }

    /// Reades data from a whole file and stores its information.
    pub fn read_data<E: ByteOrder>(&mut self) -> Result<()> {
        let read_end = if let Some(offset) = self.header.get_link_offset() {
            offset
        } else {
            self.header.get_file_size()
        };

        while self.current_offset < read_end || !self.offset_map.is_empty() {
            let offset_type = if self.current_offset < read_end {
                match self.offset_map.get_offset(self.current_offset) {
                    Ok(offset_type) => offset_type,
                    Err(Some((next_offset, offset_type))) => {
                        let byte_count = next_offset - self.current_offset;
                        if cfg!(feature = "debug") {
                            println!("{} unknown bytes were found in the offset {:#010x}.",
                                     byte_count,
                                     self.current_offset)
                        }
                        let mut unknown_bytes = Vec::with_capacity(byte_count as usize);
                        self.reader
                            .by_ref()
                            .take(byte_count as u64)
                            .read_to_end(&mut unknown_bytes)
                            .chain_err(|| "could not read unknown bytes")?;
                        self.unknown_data
                            .push((self.current_offset, unknown_bytes.into_boxed_slice()));
                        self.current_offset = next_offset;
                        offset_type
                    }
                    _ => break,
                }
            } else {
                if cfg!(feature = "debug") {
                    println!("{} unused offsets found in offset map:",
                             self.offset_map.len());
                }
                break;
                // unimplemented!()
            };

            self.read_next::<E>(offset_type)?;
        }
        Ok(())
    }

    /// Reads the next element
    fn read_next<E: ByteOrder>(&mut self, offset_type: OffsetType) -> Result<()> {
        match offset_type {
            OffsetType::StringIdList => self.read_string_id_list::<E>()?,
            OffsetType::TypeIdList => self.read_type_id_list::<E>()?,
            OffsetType::PrototypeIdList => self.read_prototype_id_list::<E>()?,
            OffsetType::FieldIdList => self.read_field_id_list::<E>()?,
            OffsetType::MethodIdList => self.read_method_id_list::<E>()?,
            OffsetType::ClassDefList => self.read_class_def_list::<E>()?,
            OffsetType::Map => self.read_map::<E>()?,
            OffsetType::TypeList => self.read_type_lists::<E>()?,
            OffsetType::AnnotationSetList => self.read_annotation_set_list::<E>()?,
            OffsetType::AnnotationSet => self.read_annotation_set::<E>()?,
            OffsetType::Annotation => self.read_annotation()?,
            OffsetType::AnnotationsDirectory => self.read_annotation_directory::<E>()?,
            OffsetType::EncodedArray => self.read_encoded_array()?,
            OffsetType::ClassData => self.read_class_data()?,
            OffsetType::Code => self.read_code_item::<E>()?,
            OffsetType::StringData => self.read_string_data()?,
            OffsetType::DebugInfo => self.read_debug_info()?,
            OffsetType::Link => unreachable!(),
        }

        Ok(())
    }

    /// Reads the list of string IDs.
    fn read_string_id_list<E: ByteOrder>(&mut self) -> Result<()> {
        // Read all string offsets.
        let offset = self.current_offset;
        for i in 0..self.header.get_string_ids_size() {
            let offset = self.reader
                .read_u32::<E>()
                .chain_err(|| {
                    format!("could not read string offset from string ID with index {} at list at \
                            offset {:#010x}",
                            i,
                            offset)
                })?;
            self.offset_map.insert(offset, OffsetType::StringData);
            self.string_ids.push(offset);
        }
        self.current_offset += STRING_ID_ITEM_SIZE * self.header.get_string_ids_size() as u32;

        Ok(())
    }

    /// Reads the list of type IDs.
    fn read_type_id_list<E: ByteOrder>(&mut self) -> Result<()> {
        // Read all type string indexes.
        let offset = self.current_offset;
        for i in 0..self.header.get_type_ids_size() {
            self.type_ids.push(self.reader
                .read_u32::<E>()
                .chain_err(|| {
                    format!("could not read type ID with index {} from type ID list at offset \
                            {:#010x}",
                            i,
                            offset)
                })?);
        }
        self.current_offset += TYPE_ID_ITEM_SIZE * self.header.get_type_ids_size() as u32;

        Ok(())
    }

    /// Reads the list of prototype IDs.
    fn read_prototype_id_list<E: ByteOrder>(&mut self) -> Result<()> {
        let offset = self.current_offset;
        for i in 0..self.header.get_prototype_ids_size() {
            let prototype_id = PrototypeIdData::from_reader::<_, E>(&mut self.reader).chain_err(|| {
                    format!("could not read prototype ID with index {} from prototype ID list at \
                            offset {:#010x}",
                            i,
                            offset)
                })?;
            if let Some(offset) = prototype_id.get_parameters_offset() {
                self.offset_map.insert(offset, OffsetType::TypeList);
            }
            self.prototype_ids.push(prototype_id);
        }
        self.current_offset += PROTO_ID_ITEM_SIZE * self.header.get_prototype_ids_size() as u32;

        Ok(())
    }

    /// Reads the list of field IDs.
    fn read_field_id_list<E: ByteOrder>(&mut self) -> Result<()> {
        let offset = self.current_offset;
        for i in 0..self.header.get_field_ids_size() {
            self.field_ids.push(FieldIdData::from_reader::<_, E>(&mut self.reader).chain_err(|| {
                    format!("could not read field ID with index {} from field ID list at offset \
                            {:#010x}", i, offset)
                })?);
        }
        self.current_offset += FIELD_ID_ITEM_SIZE * self.header.get_field_ids_size() as u32;

        Ok(())
    }

    /// Reads the list of method IDs.
    fn read_method_id_list<E: ByteOrder>(&mut self) -> Result<()> {
        let offset = self.current_offset;
        for i in 0..self.header.get_method_ids_size() {
            self.method_ids.push(MethodIdData::from_reader::<_, E>(&mut self.reader).chain_err(|| {
                    format!("could not read method ID with index {} from method ID list at offset \
                            {:#010x}",
                            i, offset)
                })?);
        }
        self.current_offset += METHOD_ID_ITEM_SIZE * self.header.get_method_ids_size() as u32;

        Ok(())
    }

    /// Reads the list of class definitions.
    fn read_class_def_list<E: ByteOrder>(&mut self) -> Result<()> {
        let offset = self.current_offset;
        for _ in 0..self.header.get_class_defs_size() {
            let class_def = ClassDefData::from_reader::<_, E>(&mut self.reader).chain_err(|| {
                    format!("could not read class definition data in list at offset {:#010x}",
                            offset)
                })?;
            if let Some(offset) = class_def.get_interfaces_offset() {
                self.offset_map.insert(offset, OffsetType::TypeList);
            }
            if let Some(offset) = class_def.get_annotations_offset() {
                self.offset_map.insert(offset, OffsetType::AnnotationsDirectory);
            }
            if let Some(offset) = class_def.get_class_data_offset() {
                self.offset_map.insert(offset, OffsetType::ClassData);
            }
            if let Some(offset) = class_def.get_static_values_offset() {
                self.offset_map.insert(offset, OffsetType::EncodedArray);
            }
            self.class_defs.push(class_def);
        }
        self.current_offset += CLASS_DEF_ITEM_SIZE * self.header.get_class_defs_size() as u32;

        Ok(())
    }

    /// Reads the map of the dex file.
    fn read_map<E: ByteOrder>(&mut self) -> Result<()> {
        let map = Map::from_reader::<_, E>(&mut self.reader, &mut self.offset_map).chain_err(|| {
                format!("error reading map at offset {:#010x}", self.current_offset)
            })?;
        if let Some(count) = map.get_num_items_for(ItemType::TypeList) {
            self.type_lists.reserve_exact(count);
        }
        if let Some(count) = map.get_num_items_for(ItemType::AnnotationSetList) {
            self.annotation_set_ref_list.reserve_exact(count);
        }
        if let Some(count) = map.get_num_items_for(ItemType::AnnotationSet) {
            self.annotation_sets.reserve_exact(count);
        }
        if let Some(count) = map.get_num_items_for(ItemType::Annotation) {
            self.annotations.reserve_exact(count);
        }
        if let Some(count) = map.get_num_items_for(ItemType::EncodedArray) {
            self.arrays.reserve_exact(count);
        }
        if let Some(count) = map.get_num_items_for(ItemType::AnnotationsDirectory) {
            self.annotations_directories.reserve_exact(count);
        }
        self.current_offset += 4 + MAP_ITEM_SIZE * map.get_item_list().len() as u32;

        Ok(())
    }

    /// Reads the list of types.
    fn read_type_lists<E: ByteOrder>(&mut self) -> Result<()> {
        let size = self.reader
            .read_u32::<E>()
            .chain_err(|| {
                format!("error reading the size of the type list at offset {:#010x}",
                        self.current_offset)
            })?;

        let mut type_list = Vec::with_capacity(size as usize);
        for i in 0..size {
            type_list.push(self.reader.read_u16::<E>().chain_err(|| {
                format!("error reading type ID for type item with index {} at type list at offset \
                        {:#010x}", i, self.current_offset)
            })?);
        }
        self.type_lists.push(type_list.into_boxed_slice());

        self.current_offset += 4 + TYPE_ITEM_SIZE * size;
        if size & 0b1 != 0 {
            // Align misaligned section
            self.reader
                .read_exact(&mut [0u8; TYPE_ITEM_SIZE as usize])
                .chain_err(|| {
                    format!("error aligning misaligned type list at offset {:#010x}",
                            self.current_offset)
                })?;
            self.current_offset += TYPE_ITEM_SIZE;
        }

        Ok(())
    }

    /// Reads a list of annotation sets.
    fn read_annotation_set_list<E: ByteOrder>(&mut self) -> Result<()> {
        let size = self.reader
            .read_u32::<E>()
            .chain_err(|| {
                format!("error reading annotation set list size at offset {:#010x}",
                        self.current_offset)
            })?;
        let mut annotation_set_list = Vec::with_capacity(size as usize);

        for _ in 0..size {
            let annotation_set_offset = self.reader
                .read_u32::<E>()
                .chain_err(|| {
                    format!("could not read annotation set offset for an anotation set in the \
                            anotation set list at offset {:#010x}",
                            self.current_offset)
                })?;
            self.offset_map.insert(annotation_set_offset, OffsetType::AnnotationSet);
            annotation_set_list.push(annotation_set_offset);
        }
        self.annotation_set_ref_list.push(annotation_set_list.into_boxed_slice());
        self.current_offset += 4 + ANNOTATION_SET_REF_SIZE * size;

        Ok(())
    }

    /// Reads an annotation set.
    fn read_annotation_set<E: ByteOrder>(&mut self) -> Result<()> {
        let size = self.reader
            .read_u32::<E>()
            .chain_err(|| {
                format!("error reading anotation set size at offset {:#010x}",
                        self.current_offset)
            })?;
        let mut annotation_set = Vec::with_capacity(size as usize);

        for i in 0..size {
            let annotation_offset = self.reader
                .read_u32::<E>()
                .chain_err(|| {
                    format!("error reading anotation offset at index {} in anotation \
                             set at offset {:#010x}",
                            i,
                            self.current_offset)
                })?;
            self.offset_map.insert(annotation_offset, OffsetType::Annotation);
            annotation_set.push(annotation_offset);
        }
        self.annotation_sets.push(annotation_set.into_boxed_slice());
        self.current_offset += 4 + ANNOTATION_SET_ITEM_SIZE * size;

        Ok(())
    }

    /// Reads the class information.
    fn read_class_data(&mut self) -> Result<()> {
        let (class_data, read) =
            ClassData::from_reader(&mut self.reader, &mut self.offset_map).chain_err(|| {
                    format!("could not read class data at offset {:#010x}",
                            self.current_offset)
                })?;
        self.classes.push((self.current_offset, class_data));
        self.current_offset += read;

        Ok(())
    }

    /// Reads code information.
    fn read_code_item<E: ByteOrder>(&mut self) -> Result<()> {
        let (code_item, read) = CodeItem::from_reader::<_, E>(&mut self.reader,
                                                              &mut self.offset_map).chain_err(|| {
                format!("could not read code item at offset {:#010x}",
                        self.current_offset)
            })?;
        self.code_segments.push((self.current_offset, code_item));
        self.current_offset += read;

        Ok(())
    }

    /// Reads an actual string.
    fn read_string_data(&mut self) -> Result<()> {
        let (string, read) = StringReader::read_string(&mut self.reader).chain_err(|| {
                format!("could not read string data at offset {:#010x}",
                        self.current_offset)
            })?;
        self.strings.push((self.current_offset, string));
        self.current_offset += read;

        Ok(())
    }

    /// Reads debug information.
    fn read_debug_info(&mut self) -> Result<()> {
        let (debug_info, read) = DebugInfo::from_reader(&mut self.reader).chain_err(|| {
                format!("could not read debug information at offset {:#010x}",
                        self.current_offset)
            })?;
        self.debug_info.push((self.current_offset, debug_info));
        self.current_offset += read;

        Ok(())
    }

    /// Reads an annotation.
    fn read_annotation(&mut self) -> Result<()> {
        let (annotation, read) = AnnotationItem::from_reader(&mut self.reader).chain_err(|| {
                format!("could not read annotation at offset {:#010x}", {
                    self.current_offset
                })
            })?;
        self.annotations.push((self.current_offset, annotation));
        self.current_offset += read;

        Ok(())
    }

    /// Reads an encoded array.
    fn read_encoded_array(&mut self) -> Result<()> {
        let (array, read) = Array::from_reader(&mut self.reader).chain_err(|| {
                format!("could not read encoded array at offset {:#010x}",
                        self.current_offset)
            })?;
        self.arrays.push((self.current_offset, array));
        self.current_offset += read;

        Ok(())
    }

    fn read_annotation_directory<E: ByteOrder>(&mut self) -> Result<()> {
        // Read annotations directory.
        // println!("Anotations directory found at offset {:#010x}", offset);
        let directory =
            AnnotationsDirectory::from_reader::<_, E>(&mut self.reader, &mut self.offset_map)
                .chain_err(|| {
                    format!("could not read annotation directory at offset {:#010x}",
                            self.current_offset)
                })?;
        let read = 4 * 4 + directory.get_field_annotations().len() * 8 +
                   directory.get_method_annotations().len() * 8 +
                   directory.get_parameter_annotations().len() * 8;
        self.annotations_directories.push((self.current_offset, directory));
        self.current_offset += read as u32;

        Ok(())
    }
}

/// The struct representing the *dex* file Map.
#[derive(Debug)]
struct Map {
    map_list: Box<[MapItem]>,
}

impl Map {
    /// Reads a map object from a reader.
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
        Ok(Map { map_list: map_list.into_boxed_slice() })
    }

    /// Gets the list of items.
    pub fn get_item_list(&self) -> &[MapItem] {
        &self.map_list
    }

    /// Gets the number of items for a particular item type.
    pub fn get_num_items_for(&self, item_type: ItemType) -> Option<usize> {
        for item in self.map_list.iter() {
            if item.get_item_type() == item_type {
                return Some(item.get_num_items());
            }
        }
        None
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
            0...4 => result |= payload << (i * 7),
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
