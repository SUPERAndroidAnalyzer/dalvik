//! Dex file reader module.

use std::io::{BufRead, Read, Cursor};

use byteorder::{ByteOrder, ReadBytesExt, LittleEndian, BigEndian};

use header::Header;
use error::*;
use types::read::*;
use types::*;

/// Structure for reading a Dex file in a fast way.
#[derive(Debug)]
pub struct DexReader {
    /// Reader to use to read data.
    ///
    /// Header must already be read before creating the object.
    file_cursor: Cursor<Box<[u8]>>,
    /// Header of the dex file.
    header: Header,
    /// String list.
    strings: Vec<String>,
    /// Type list.
    types: Vec<Type>,
    /// Prototype ID list.
    prototypes: Vec<Prototype>,
    /// Field ID list.
    ///
    /// This list creates 1:1 relations between field indexes and field information offsets in the
    /// dex file.
    field_ids: Vec<FieldIdData>,
    /// Method ID list.
    method_ids: Vec<MethodIdData>,
    /// List of classes.
    classes: Vec<Class>,

    /// List of lists of references to annotation set offsets.
    annotation_set_ref_list: Vec<Box<[u32]>>,
    /// Set of annotations.
    annotation_sets: Vec<Box<[u32]>>,
    /// Code segment list.
    code_segments: Vec<(u32, CodeItem)>,
    /// Debug information list.
    debug_info: Vec<(u32, DebugInfo)>,
    // /// Annotation list.
    // annotations: Vec<(u32, AnnotationItem)>,
    /// Array list.
    arrays: Vec<(u32, Array)>,
    /// Annotations directories.
    annotations_directories: Vec<(u32, AnnotationsDirectory)>,
}

impl DexReader {
    /// Creates a new reader with the information from the header of the file.
    pub fn new<R: Read>(mut file: R, size: Option<usize>) -> Result<DexReader> {
        let mut file_contents = if let Some(size) = size {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };
        file.read_to_end(&mut file_contents).chain_err(|| "could not read dex file contents")?;
        let mut cursor = Cursor::new(file_contents.into_boxed_slice());
        let header =
            Header::from_reader(&mut cursor).chain_err(|| "could not read dex file header")?;
        let strings = Vec::with_capacity(header.get_string_ids_size() as usize);
        let types = Vec::with_capacity(header.get_type_ids_size() as usize);
        let prototypes = Vec::with_capacity(header.get_prototype_ids_size() as usize);
        let field_ids = Vec::with_capacity(header.get_field_ids_size() as usize);
        let method_ids = Vec::with_capacity(header.get_method_ids_size() as usize);
        Ok(DexReader {
               file_cursor: cursor,
               header: header,
               strings: strings,
               types: types,
               prototypes: prototypes,
               field_ids: field_ids,
               method_ids: method_ids,
               classes: Vec::new(),
               annotation_set_ref_list: Vec::new(),
               annotation_sets: Vec::new(),
               code_segments: Vec::new(),
               debug_info: Vec::new(),
               // annotations: Vec::new(),
               arrays: Vec::new(),
               annotations_directories: Vec::new(),
           })
    }

    /// Reads data from a whole file and stores its information.
    pub fn read_data(&mut self) -> Result<()> {
        if self.header.is_little_endian() {
            self.read_endian_data::<LittleEndian>()
        } else {
            self.read_endian_data::<BigEndian>()
        }
    }

    /// Reads the data in the correct endianness.
    fn read_endian_data<B: ByteOrder>(&mut self) -> Result<()> {
        if let Some(offset) = self.header.get_string_ids_offset() {
            self.file_cursor.set_position(offset as u64);
            self.read_string_list::<B>().chain_err(|| "could not read string list")?;
        }
        if let Some(offset) = self.header.get_type_ids_offset() {
            self.file_cursor.set_position(offset as u64);
            self.read_all_types::<B>().chain_err(|| "could not read type list")?;
        }
        if let Some(offset) = self.header.get_prototype_ids_offset() {
            self.file_cursor.set_position(offset as u64);
            self.read_prototype_list::<B>().chain_err(|| "could not read prototype list")?;
        }
        if let Some(offset) = self.header.get_field_ids_offset() {
            self.file_cursor.set_position(offset as u64);
            self.read_field_id_list::<B>().chain_err(|| "could not read field ID list")?;
        }
        if let Some(offset) = self.header.get_method_ids_offset() {
            self.file_cursor.set_position(offset as u64);
            self.read_method_id_list::<B>().chain_err(|| "could not read method ID list")?;
        }
        if let Some(offset) = self.header.get_class_defs_offset() {
            self.file_cursor.set_position(offset as u64);
            self.read_class_list::<B>().chain_err(|| "could not read class list")?;
        }

        Ok(())
    }

    /// Reads the list of strings.
    fn read_string_list<B: ByteOrder>(&mut self) -> Result<()> {
        for _ in 0..self.header.get_string_ids_size() {
            let current_offset = self.file_cursor.position();
            let offset = self.file_cursor
                .read_u32::<B>()
                .chain_err(|| {
                    format!("could not read string offset from string ID at offset {:#010x}",
                            current_offset)
                })?;
            let current_offset = self.file_cursor.position();
            self.file_cursor.set_position(offset as u64);
            let str_data = self.read_string()?;
            self.strings.push(str_data);
            self.file_cursor.set_position(current_offset);
        }

        Ok(())
    }

    /// Reads an actual string.
    fn read_string(&mut self) -> Result<String> {
        let (size, _) =
            read_uleb128(&mut self.file_cursor).chain_err(|| "could not read string size")?;
        let mut data = Vec::with_capacity(size as usize);
        if size > 0 {
            self.file_cursor.read_until(0, &mut data)?;
            data.pop();
        }

        let string = String::from_utf8(data).chain_err(|| "error decoding UTF-8 from string data")?;
        let char_count = string.chars().count();
        if char_count == size as usize {
            Ok(string)
        } else {
            Err(ErrorKind::StringSizeMismatch(size, char_count).into())
        }
    }

    /// Reads the list of types.
    fn read_all_types<B: ByteOrder>(&mut self) -> Result<()> {
        for _ in 0..self.header.get_type_ids_size() {
            let current_offset = self.file_cursor.position();
            let index =
                self.file_cursor
                    .read_u32::<B>()
                    .chain_err(|| {
                                   format!("could not read type ID at offset {:#010x}",
                                           current_offset)
                               })?;
            let type_str = self.strings
                .get(index as usize)
                .ok_or_else(|| ErrorKind::UnknownStringIndex(index))?;
            self.types.push(type_str.parse()
                .chain_err(|| {
                    format!("could not read type descriptor from string at index {} (`{}`)",
                            index,
                            type_str)
                })?);
        }

        Ok(())
    }

    /// Reads the list of prototype IDs.
    fn read_prototype_list<B: ByteOrder>(&mut self) -> Result<()> {
        for _ in 0..self.header.get_prototype_ids_size() {
            let current_offset = self.file_cursor.position();
            let prototype_id =
                PrototypeIdData::from_reader::<_, B>(&mut self.file_cursor).chain_err(|| {
                               format!("could not read prototype ID at offset {:#010x}",
                                       current_offset)
                           })?;

            let parameters = if let Some(off) = prototype_id.parameters_offset() {
                let current_offset = self.file_cursor.position();
                self.file_cursor.set_position(off as u64);
                let parameters = self.read_type_list::<B>()
                    .chain_err(|| "could not read parameter list")?;
                self.file_cursor.set_position(current_offset);
                Some(parameters)
            } else {
                None
            };
            let shorty_str =
                self.strings
                    .get(prototype_id.shorty_index() as usize)
                    .ok_or_else(|| ErrorKind::UnknownStringIndex(prototype_id.shorty_index()))?;
            let shorty_descriptor = shorty_str.parse()
                .chain_err(|| {
                    format!("could not read shorty descriptor from string at index {} (`{}`)",
                            prototype_id.shorty_index(),
                            shorty_str)
                })?;
            let return_type = self.types
                .get(prototype_id.return_type_index() as usize)
                .ok_or_else(|| {
                                ErrorKind::UnknownTypeIndex(prototype_id.return_type_index() as u16)
                            })?
                .clone();

            self.prototypes.push(Prototype::new(shorty_descriptor, return_type, parameters));
        }
        Ok(())
    }

    /// Reads a list of types.
    fn read_type_list<B: ByteOrder>(&mut self) -> Result<Box<[Type]>> {
        let current_offset = self.file_cursor.position();
        let size = self.file_cursor
            .read_u32::<B>()
            .chain_err(|| {
                           format!("error reading the size of the type list at offset {:#010x}",
                                   current_offset)
                       })?;

        let mut type_list = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let current_offset = self.file_cursor.position();
            let index = self.file_cursor
                .read_u16::<B>()
                .chain_err(|| {
                    format!("error reading type index for type list item at offset {:#010x}",
                            current_offset)
                })?;
            type_list.push(self.types
                               .get(index as usize)
                               .ok_or_else(|| Error::from(ErrorKind::UnknownTypeIndex(index)))?
                               .clone());
        }

        Ok(type_list.into_boxed_slice())
    }

    /// Reads the list of field IDs.
    fn read_field_id_list<B: ByteOrder>(&mut self) -> Result<()> {
        for _ in 0..self.header.get_field_ids_size() {
            let current_offset = self.file_cursor.position();
            self.field_ids
                .push(FieldIdData::from_reader::<_, B>(&mut self.file_cursor).chain_err(|| {
                                         format!("could not read field ID at offset {:#010x}",
                                                 current_offset)
                                     })?);
        }

        Ok(())
    }

    /// Reads the list of method IDs.
    fn read_method_id_list<B: ByteOrder>(&mut self) -> Result<()> {
        for _ in 0..self.header.get_method_ids_size() {
            let current_offset = self.file_cursor.position();
            self.method_ids
                .push(MethodIdData::from_reader::<_, B>(&mut self.file_cursor).chain_err(|| {
                                         format!("could not read method ID at offset {:#010x}",
                                                 current_offset)
                                     })?);
        }

        Ok(())
    }

    /// Reads the list of classes.
    fn read_class_list<B: ByteOrder>(&mut self) -> Result<()> {
        for _ in 0..self.header.get_class_defs_size() {
            let class_offset = self.file_cursor.position();
            let class_def = ClassDefData::from_reader::<_, B>(&mut self.file_cursor).chain_err(|| {
                               format!("could not read class definition data at offset {:#010x}",
                                       class_offset)
                           })?;

            let new_offset = self.file_cursor.position();
            let interfaces = if let Some(offset) = class_def.interfaces_offset() {
                self.file_cursor.set_position(offset as u64);
                self.read_type_list::<B>().chain_err(|| {
                    format!("could not read interfaces list at offset {:#010x} for class at offset \
                             {:#010x}", offset, class_offset)
                    })?
                // TODO check that all are classes (Fully Qualified Names) and no duplicates.
            } else {
                Vec::new().into_boxed_slice()
            };
            let annotations = if let Some(offset) = class_def.annotations_offset() {
                self.file_cursor.set_position(offset as u64);
                Some(self.read_annotations_directory::<B>()
                         .chain_err(|| {
                             format!("could not read annotation list at offset {:#010x} for class \
                                      at offset {:#010x}", offset, class_offset)
                                  })?)
            } else {
                None
            };
            let class_data = if let Some(offset) = class_def.class_data_offset() {
                self.file_cursor.set_position(offset as u64);
                Some(ClassData::from_reader(&mut self.file_cursor).chain_err(|| {
                                        format!("could not read class data at offset {:#010x} for \
                                                 class at offset {:#010x}",
                                                offset, class_offset)
                                    })?)
            } else {
                None
            };
            let static_values = if let Some(offset) = class_def.static_values_offset() {
                self.file_cursor.set_position(offset as u64);
                Some(Array::from_reader(&mut self.file_cursor).chain_err(|| {
                                        format!("could not read encoded array at offset {:#010x}",
                                                offset)
                                    })?)
            } else {
                None
            };
            self.file_cursor.set_position(new_offset);

            self.classes.push(Class::new(class_def.class_index(),
                                         class_def.access_flags(),
                                         class_def.superclass_index(),
                                         interfaces,
                                         class_def.source_file_index(),
                                         annotations,
                                         class_data,
                                         static_values));
        }

        Ok(())
    }

    /// Reads an annotations directory.
    fn read_annotations_directory<B: ByteOrder>(&mut self) -> Result<AnnotationsDirectory> {
        let current_offset = self.file_cursor.position();
        let read = AnnotationsDirectoryOffsets::from_reader::<_, B>(&mut self.file_cursor).chain_err(|| {
                           format!("could not read annotation directory at offset {:#010x}",
                                   current_offset)
                       })?;

        let class_annotations = if let Some(off) = read.class_annotations_offset() {
            self.file_cursor.set_position(off as u64);
            self.read_annotation_set::<B>().chain_err(|| "could not read class annotations set")?
        } else {
            Vec::new().into_boxed_slice()
        };
        let mut field_annotations = Vec::with_capacity(read.field_annotations().len());
        for fa_off in read.field_annotations() {
            self.file_cursor.set_position(fa_off.offset() as u64);
            field_annotations.push(
                FieldAnnotations::new(fa_off.field_index(), self.read_annotation_set::<B>()
                .chain_err(|| "could not read field annotations set")?));
        }
        let mut method_annotations = Vec::with_capacity(read.method_annotations().len());
        for ma_off in read.method_annotations() {
            self.file_cursor.set_position(ma_off.offset() as u64);
            method_annotations.push(
                MethodAnnotations::new(ma_off.method_index(), self.read_annotation_set::<B>()
                .chain_err(|| "could not read method annotations set")?));
        }
        let mut parameter_annotations = Vec::with_capacity(read.parameter_annotations().len());
        for pa_off in read.parameter_annotations() {
            self.file_cursor.set_position(pa_off.offset() as u64);
            parameter_annotations.push(
                ParameterAnnotations::new(pa_off.method_index(), self.read_annotation_set::<B>()
                .chain_err(|| "could not read parameter annotations set")?));
        }

        Ok(AnnotationsDirectory::new(class_annotations,
                                     field_annotations.into_boxed_slice(),
                                     method_annotations.into_boxed_slice(),
                                     parameter_annotations.into_boxed_slice()))
    }

    /// Reads an annotation set.
    fn read_annotation_set<B: ByteOrder>(&mut self) -> Result<Box<[Annotation]>> {
        let current_offset = self.file_cursor.position();
        let size = self.file_cursor
            .read_u32::<B>()
            .chain_err(|| {
                           format!("error reading anotation set size at offset {:#010x}",
                                   current_offset)
                       })?;
        let mut annotation_set = Vec::with_capacity(size as usize);

        for _ in 0..size {
            let current_offset = self.file_cursor.position();
            let annotation_offset = self.file_cursor
                .read_u32::<B>()
                .chain_err(|| {
                               format!("error reading anotation offset at offset {:#010x}",
                                       current_offset)
                           })?;
            let current_offset = self.file_cursor.position();
            self.file_cursor.set_position(annotation_offset as u64);
            annotation_set.push(self.read_annotation()?);
            self.file_cursor.set_position(current_offset);
        }

        Ok(annotation_set.into_boxed_slice())
    }

    /// Reads an annotation.
    fn read_annotation(&mut self) -> Result<Annotation> {
        let current_offset = self.file_cursor.position();
        let annotation = Annotation::from_reader(&mut self.file_cursor).chain_err(|| {
                           format!("could not read annotation at offset {:#010x}", {
                    current_offset
                })
                       })?;

        Ok(annotation)
    }

    // /// Reads the map of the dex file.
    // fn read_map<B: ByteOrder>(&mut self) -> Result<()> {
    //     let map = Map::from_reader::<_, E>(&mut self.reader, &mut self.offset_map).chain_err(|| {
    //             format!("error reading map at offset {:#010x}", self.current_offset)
    //         })?;
    //     if let Some(count) = map.get_num_items_for(ItemType::TypeList) {
    //         self.type_lists.reserve_exact(count);
    //     }
    //     if let Some(count) = map.get_num_items_for(ItemType::AnnotationSetList) {
    //         self.annotation_set_ref_list.reserve_exact(count);
    //     }
    //     if let Some(count) = map.get_num_items_for(ItemType::AnnotationSet) {
    //         self.annotation_sets.reserve_exact(count);
    //     }
    //     if let Some(count) = map.get_num_items_for(ItemType::Annotation) {
    //         self.annotations.reserve_exact(count);
    //     }
    //     if let Some(count) = map.get_num_items_for(ItemType::EncodedArray) {
    //         self.arrays.reserve_exact(count);
    //     }
    //     if let Some(count) = map.get_num_items_for(ItemType::AnnotationsDirectory) {
    //         self.annotations_directories.reserve_exact(count);
    //     }
    //     self.current_offset += 4 + MAP_ITEM_SIZE * map.get_item_list().len() as u32;
    //
    //     Ok(())
    // }

    // /// Reads a list of annotation sets.
    // fn read_annotation_set_list<B: ByteOrder>(&mut self) -> Result<()> {
    //     let size = self.reader
    //         .read_u32::<E>()
    //         .chain_err(|| {
    //             format!("error reading annotation set list size at offset {:#010x}",
    //                     self.current_offset)
    //         })?;
    //     let mut annotation_set_list = Vec::with_capacity(size as usize);
    //
    //     for _ in 0..size {
    //         let annotation_set_offset = self.reader
    //             .read_u32::<E>()
    //             .chain_err(|| {
    //                 format!("could not read annotation set offset for an anotation set in the \
    //                         anotation set list at offset {:#010x}",
    //                         self.current_offset)
    //             })?;
    //         self.offset_map.insert(annotation_set_offset, OffsetType::AnnotationSet);
    //         annotation_set_list.push(annotation_set_offset);
    //     }
    //     self.annotation_set_ref_list.push(annotation_set_list.into_boxed_slice());
    //     self.current_offset += 4 + ANNOTATION_SET_REF_SIZE * size;
    //
    //     Ok(())
    // }

    // /// Reads code information.
    // fn read_code_item<B: ByteOrder>(&mut self) -> Result<()> {
    //     let (code_item, read) = CodeItem::from_reader::<_, E>(&mut self.reader,
    //                                                           &mut self.offset_map)
    // .chain_err(|| {
    //             format!("could not read code item at offset {:#010x}",
    //                     self.current_offset)
    //         })?;
    //     self.code_segments.push((self.current_offset, code_item));
    //     self.current_offset += read;
    //
    //     Ok(())
    // }

    // /// Reads debug information.
    // fn read_debug_info(&mut self) -> Result<()> {
    //     let (debug_info, read) = DebugInfo::from_reader(&mut self.reader).chain_err(|| {
    //             format!("could not read debug information at offset {:#010x}",
    //                     self.current_offset)
    //         })?;
    //     self.debug_info.push((self.current_offset, debug_info));
    //     self.current_offset += read;
    //
    //     Ok(())
    // }
}

/// Reads a uleb128 from a reader.
///
/// Returns the u32 represented by the uleb128 and the number of bytes read.
pub fn read_uleb128<R: Read>(reader: &mut R) -> Result<(u32, u32)> {
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
pub fn read_uleb128p1<R: Read>(reader: &mut R) -> Result<(u32, u32)> {
    let (uleb128, read) = read_uleb128(reader)?;
    Ok((uleb128.wrapping_sub(1), read))
}

/// Reads a sleb128 from a reader.
///
/// Returns the i32 represented by the sleb128 and the number of bytes read.
pub fn read_sleb128<R: Read>(reader: &mut R) -> Result<(i32, u32)> {
    let (uleb128, read) = read_uleb128(reader)?;
    let s_bits = read * 7;
    let mut signed = uleb128 as i32;

    if (signed & 1 << s_bits) != 0 {
        signed |= -1 << s_bits;
    }

    Ok((signed, read))
}
