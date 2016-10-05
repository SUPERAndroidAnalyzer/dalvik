extern crate byteorder;
#[macro_use]
extern crate bitflags;

use std::path::Path;
use std::{fmt, fs, usize};
use std::iter::SkipWhile;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};

use byteorder::{BigEndian, LittleEndian, ByteOrder, ReadBytesExt};

pub mod error;
pub mod bytecode; // TODO: not in use
pub mod types; // TODO: Should not be public
pub mod sizes; // TODO: Should not be public

use error::{Result, Error};
use sizes::*;
use types::{StringIdData, TypeIdData, PrototypeIdData, FieldIdData, MethodIdData, ClassDefData,
            MapItem, ItemType};

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
    /// Loads a new Dex data structure from the file at the given path.
    pub fn from_file<P: AsRef<Path>>(path: P, verify: bool) -> Result<Dex> {
        let path = path.as_ref();
        // Read header (and verify file if requested)
        let (mut reader, header) = if verify {
            let header = try!(Header::from_file(path, true));
            let f = try!(fs::File::open(path.clone()));
            let mut reader = BufReader::new(f);
            try!(reader.seek(SeekFrom::Start(HEADER_SIZE as u64)));
            (reader, header)
        } else {
            let f = try!(fs::File::open(path.clone()));
            let mut reader = BufReader::new(f);
            let header = try!(Header::from_reader(&mut reader));
            (reader, header)
        };
        let mut offset = HEADER_SIZE;
        let mut string_ids = Vec::with_capacity(header.get_string_ids_size());
        // Read all string offsets
        for _ in 0..string_ids.capacity() {
            string_ids.push(StringIdData::new(try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            })));
            offset += STRING_ID_ITEM_SIZE;
        }

        let mut type_ids = Vec::with_capacity(header.get_type_ids_size());
        // Read all type string indexes
        for _ in 0..type_ids.capacity() {
            type_ids.push(TypeIdData::new(try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            })));
            offset += TYPE_ID_ITEM_SIZE;
        }

        let mut prototype_ids = Vec::with_capacity(header.get_prototype_ids_size());
        // Read all prototype IDs
        for _ in 0..header.get_prototype_ids_size() {
            let shorty_id = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let return_type_id = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let parameters_offset = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            prototype_ids.push(PrototypeIdData::new(shorty_id, return_type_id, parameters_offset));
            offset += PROTO_ID_ITEM_SIZE;
        }

        let mut field_ids = Vec::with_capacity(header.get_field_ids_size());
        // Read all field IDs
        for _ in 0..header.get_field_ids_size() {
            let class_id = try!(if header.is_little_endian() {
                reader.read_u16::<LittleEndian>()
            } else {
                reader.read_u16::<BigEndian>()
            });
            let type_id = try!(if header.is_little_endian() {
                reader.read_u16::<LittleEndian>()
            } else {
                reader.read_u16::<BigEndian>()
            });
            let name_id = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            field_ids.push(FieldIdData::new(class_id, type_id, name_id));
            offset += FIELD_ID_ITEM_SIZE;
        }

        let mut method_ids = Vec::with_capacity(header.get_method_ids_size());
        // Read all method IDs
        for _ in 0..header.get_method_ids_size() {
            let class_id = try!(if header.is_little_endian() {
                reader.read_u16::<LittleEndian>()
            } else {
                reader.read_u16::<BigEndian>()
            });
            let prototype_id = try!(if header.is_little_endian() {
                reader.read_u16::<LittleEndian>()
            } else {
                reader.read_u16::<BigEndian>()
            });
            let name_id = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            method_ids.push(MethodIdData::new(class_id, prototype_id, name_id));
            offset += METHOD_ID_ITEM_SIZE;
        }

        let mut class_defs = Vec::with_capacity(header.get_class_defs_size());
        // Read all class definitions
        for _ in 0..header.get_class_defs_size() {
            let class_id = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let access_flags = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let superclass_id = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let interfaces_offset = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let source_file_id = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let annotations_offset = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let class_data_offset = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            let static_values_offset = try!(if header.is_little_endian() {
                reader.read_u32::<LittleEndian>()
            } else {
                reader.read_u32::<BigEndian>()
            });
            class_defs.push(try!(ClassDefData::new(class_id,
                                                   access_flags,
                                                   superclass_id,
                                                   interfaces_offset,
                                                   source_file_id,
                                                   annotations_offset,
                                                   class_data_offset,
                                                   static_values_offset)));
            offset += CLASS_DEF_ITEM_SIZE;
        }
        debug_assert!(offset <= header.get_data_offset());

        if offset < header.get_data_offset() {
            if cfg!(feature = "debug") {
                println!("Should have reached data offset at {:#010x}, but still in {:#010x}. {} \
                          bytes of unknown data were found.",
                         header.get_data_offset(),
                         offset,
                         header.get_data_offset() - offset);
            }
            try!(reader.seek(SeekFrom::Current((header.get_data_offset() - offset) as i64)));
            offset = header.get_data_offset();
        }

        let map = try!(if header.is_little_endian() {
            Map::from_reader::<_, LittleEndian>(&mut reader)
        } else {
            Map::from_reader::<_, LittleEndian>(&mut reader)
        });
        offset += 4 + MAP_ITEM_SIZE * map.get_item_list().len();

        for item in map.get_item_list().iter().skip_while(|i| i.get_offset() < offset) {
            if item.get_offset() != offset {
                return Err(Error::Map(format!("there should be an item at the current offset \
                                               ({:#010x}), but next item returned offset \
                                               {:#010x}",
                                              offset,
                                              item.get_offset())));
            }
            match item.get_item_type() {
                ItemType::TypeList => {}
                ItemType::AnnotationSetRefList => {}
                _ => unimplemented!(),
            }
        }

        println!("Map offset: {:#010x}", header.get_map_offset());
        println!("Current offset: {:#010x}", offset);
        println!("{:?}", map);
        panic!();


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

        // TODO search links?

        unimplemented!()
    }

    /// Ads the file in the given path to the current Dex data structure.
    pub fn add_file<P: AsRef<Path>>(_path: P) -> Result<()> {
        unimplemented!()
    }
}

pub const ENDIAN_CONSTANT: u32 = 0x12345678;
pub const REVERSE_ENDIAN_CONSTANT: u32 = 0x78563412;

/// Dex header representantion structure.
pub struct Header {
    magic: [u8; 8],
    checksum: u32,
    signature: [u8; 20],
    file_size: usize,
    header_size: usize,
    endian_tag: u32,
    link_size: Option<usize>,
    link_offset: Option<usize>,
    map_offset: usize,
    string_ids_size: usize,
    string_ids_offset: Option<usize>,
    type_ids_size: usize,
    type_ids_offset: Option<usize>,
    prototype_ids_size: usize,
    prototype_ids_offset: Option<usize>,
    field_ids_size: usize,
    field_ids_offset: Option<usize>,
    method_ids_size: usize,
    method_ids_offset: Option<usize>,
    class_defs_size: usize,
    class_defs_offset: Option<usize>,
    data_size: usize,
    data_offset: usize,
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Header {{ magic: [ {} ] (version: {}), checksum: {:#x}, SHA-1 signature: {}, \
                file_size: {} bytes, header_size: {} bytes, endian_tag: {:#x} ({} endian), {}, \
                map_offset: {:#x}, {}, {}, {}, {}, {}, {}, data_size: {} bytes, data_offset: \
                {:#x} }}",
               {
                   let mut magic_vec = Vec::with_capacity(8);
                   for b in &self.magic {
                       magic_vec.push(format!("{:#02x}", b))
                   }
                   magic_vec.join(", ")
               },
               self.get_dex_version(),
               self.checksum,
               {
                   let mut signature = String::with_capacity(40);
                   for b in &self.signature {
                       signature.push_str(&format!("{:02x}", b))
                   }
                   signature
               },
               self.file_size,
               self.header_size,
               self.endian_tag,
               if self.is_little_endian() {
                   "little"
               } else {
                   "big"
               },
               if self.link_size.is_some() {
                   format!("link_size: {} bytes, link_offset: {:#x}",
                           self.link_size.unwrap(),
                           self.link_offset.unwrap())
               } else {
                   String::from("no link section")
               },
               self.map_offset,
               if self.string_ids_size > 0 {
                   format!("string_ids_size: {} strings, string_ids_offset: {:#x}",
                           self.string_ids_size,
                           self.string_ids_offset.unwrap())
               } else {
                   String::from("no strings")
               },
               if self.type_ids_size > 0 {
                   format!("type_ids_size: {} types, type_ids_offset: {:#x}",
                           self.type_ids_size,
                           self.type_ids_offset.unwrap())
               } else {
                   String::from("no types")
               },
               if self.prototype_ids_size > 0 {
                   format!("prototype_ids_size: {} types, prototype_ids_offset: {:#x}",
                           self.prototype_ids_size,
                           self.prototype_ids_offset.unwrap())
               } else {
                   String::from("no prototypes")
               },
               if self.field_ids_size > 0 {
                   format!("field_ids_size: {} types, field_ids_offset: {:#x}",
                           self.field_ids_size,
                           self.field_ids_offset.unwrap())
               } else {
                   String::from("no fields")
               },
               if self.method_ids_size > 0 {
                   format!("method_ids_size: {} types, method_ids_offset: {:#x}",
                           self.method_ids_size,
                           self.method_ids_offset.unwrap())
               } else {
                   String::from("no methods")
               },
               if self.class_defs_size > 0 {
                   format!("class_defs_size: {} types, class_defs_offset: {:#x}",
                           self.class_defs_size,
                           self.class_defs_offset.unwrap())
               } else {
                   String::from("no classes")
               },
               self.data_size,
               self.data_offset)
    }
}

impl Header {
    /// Obtains the header from a Dex file.
    pub fn from_file<P: AsRef<Path>>(path: P, verify: bool) -> Result<Header> {
        let f = try!(fs::File::open(path));
        let file_size = try!(f.metadata()).len();
        if file_size < HEADER_SIZE as u64 || file_size > usize::MAX as u64 {
            return Err(Error::invalid_file_size(file_size, None));
        }
        let header = try!(Header::from_reader(BufReader::new(f)));
        if file_size as usize != header.get_file_size() {
            Err(Error::invalid_file_size(file_size, Some(header.get_file_size())))
        } else if verify {
            unimplemented!()
        } else {
            Ok(header)
        }
    }

    /// Obtains the header from a Dex file reader.
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Header> {
        // Magic number
        let mut magic = [0u8; 8];
        try!(reader.read_exact(&mut magic));
        if !Header::is_magic_valid(&magic) {
            return Err(Error::invalid_magic(magic));
        }
        // Checksum
        let mut checksum = try!(reader.read_u32::<LittleEndian>());
        // Signature
        let mut signature = [0u8; 20];
        try!(reader.read_exact(&mut signature));
        // File size
        let mut file_size = try!(reader.read_u32::<LittleEndian>());
        // Header size
        let mut header_size = try!(reader.read_u32::<LittleEndian>());
        // Endian tag
        let endian_tag = try!(reader.read_u32::<LittleEndian>());

        // Check endianness
        if endian_tag == REVERSE_ENDIAN_CONSTANT {
            // The file is in big endian instead of little endian.
            checksum = checksum.swap_bytes();
            file_size = file_size.swap_bytes();
            header_size = header_size.swap_bytes();
        } else if endian_tag != ENDIAN_CONSTANT {
            return Err(Error::invalid_endian_tag(endian_tag));
        }
        let header_size = header_size as usize;
        let file_size = file_size as usize;
        // Check header size
        if header_size != HEADER_SIZE {
            return Err(Error::invalid_header_size(header_size));
        }
        // Check file size
        if file_size < HEADER_SIZE {
            return Err(Error::invalid_file_size(0, Some(file_size)));
        }

        let mut current_offset = HEADER_SIZE;

        // Link size
        let link_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        // Link offset
        let link_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if link_size == 0 && link_offset != 0 {
            return Err(Error::mismatched_offsets("link_offset", link_offset, 0));
        }

        // Map offset
        let map_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if map_offset == 0 {
            return Err(Error::MismatchedOffsets(String::from("`map_offset` was 0x00, and it \
                                                              can never be zero")));
        }

        // String IDs size
        let string_ids_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        // String IDs offset
        let string_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if string_ids_size > 0 && string_ids_offset != current_offset {
            return Err(Error::mismatched_offsets("string_ids_offset",
                                                 string_ids_offset,
                                                 HEADER_SIZE));
        }
        if string_ids_size == 0 && string_ids_offset != 0 {
            return Err(Error::mismatched_offsets("string_ids_offset", string_ids_offset, 0));
        }
        current_offset += string_ids_size * 4;

        // Types IDs size
        let type_ids_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        // Types IDs offset
        let type_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if type_ids_size > 0 && type_ids_offset != current_offset {
            return Err(Error::mismatched_offsets("type_ids_offset",
                                                 type_ids_offset,
                                                 current_offset));
        }
        if type_ids_size == 0 && type_ids_offset != 0 {
            return Err(Error::mismatched_offsets("type_ids_offset", type_ids_offset, 0));
        }
        current_offset += type_ids_size * 4;

        // Prototype IDs size
        let prototype_ids_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        // Prototype IDs offset
        let prototype_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if prototype_ids_size > 0 && prototype_ids_offset != current_offset {
            return Err(Error::mismatched_offsets("prototype_ids_offset",
                                                 prototype_ids_offset,
                                                 current_offset));
        }
        if prototype_ids_size == 0 && prototype_ids_offset != 0 {
            return Err(Error::mismatched_offsets("prototype_ids_offset", prototype_ids_offset, 0));
        }
        current_offset += prototype_ids_size * 3 * 4;

        // Field IDs size
        let field_ids_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        // Field IDs offset
        let field_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if field_ids_size > 0 && field_ids_offset != current_offset {
            return Err(Error::mismatched_offsets("field_ids_offset",
                                                 field_ids_offset,
                                                 current_offset));
        }
        if field_ids_size == 0 && field_ids_offset != 0 {
            return Err(Error::mismatched_offsets("field_ids_offset", field_ids_offset, 0));
        }
        current_offset += field_ids_size * (2 * 2 + 4);

        // Method IDs size
        let method_ids_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        // Method IDs offset
        let method_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if method_ids_size > 0 && method_ids_offset != current_offset {
            return Err(Error::mismatched_offsets("method_ids_offset",
                                                 method_ids_offset,
                                                 current_offset));
        }
        if method_ids_size == 0 && method_ids_offset != 0 {
            return Err(Error::mismatched_offsets("method_ids_offset", method_ids_offset, 0));
        }
        current_offset += method_ids_size * (2 * 2 + 4);

        // Class defs size
        let class_defs_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        // Class defs offset
        let class_defs_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if class_defs_size > 0 && class_defs_offset != current_offset {
            return Err(Error::mismatched_offsets("class_defs_offset",
                                                 class_defs_offset,
                                                 current_offset));
        }
        if class_defs_size == 0 && class_defs_offset != 0 {
            return Err(Error::mismatched_offsets("class_defs_offset", class_defs_offset, 0));
        }
        current_offset += class_defs_size * 8 * 4;

        // Data size
        let data_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if data_size & 0b11 != 0 {
            return Err(Error::Header(format!("`data_size` must be a 4-byte multiple, but it \
                                              was {:#010x}",
                                             data_size)));
        }

        // Data offset
        let data_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        }) as usize;
        if data_offset != current_offset {
            // return Err(Error::mismatched_offsets("data_offset", data_offset, current_offset));
            // TODO seems that there is more information after the class definitions.
            if cfg!(feature = "debug") {
                println!("{} bytes of unknown data were found.",
                         data_offset - current_offset);
            }
            current_offset = data_offset;
        }
        current_offset += data_size;
        if map_offset < data_offset || map_offset > data_offset + data_size {
            return Err(Error::MismatchedOffsets(format!("`map_offset` section must be in the \
                                                         `data` section (between {:#010x} and \
                                                         {:#010x}) but it was at {:#010x}",
                                                        data_offset,
                                                        current_offset,
                                                        map_offset)));
        }
        if link_size == 0 && current_offset != file_size {
            return Err(Error::Header(format!("`data` section must end at the EOF if there \
                                                   are no links in the file. Data end: \
                                                   {:#010x}, `file_size`: {:#010x}",
                                             current_offset,
                                             file_size)));

        }
        if link_size != 0 && link_offset == 0 {
            return Err(Error::mismatched_offsets("link_offset", 0, current_offset));
        }
        if link_size != 0 && link_offset != 0 {
            if link_offset != current_offset {
                return Err(Error::mismatched_offsets("link_offset", link_offset, current_offset));
            }
            if link_offset + link_size != file_size {
                return Err(Error::Header(String::from("`link_data` section must end at the end \
                                                       of file")));
            }
        }

        Ok(Header {
            magic: magic,
            checksum: checksum,
            signature: signature,
            file_size: file_size,
            header_size: header_size,
            endian_tag: endian_tag,
            link_size: if link_size == 0 {
                None
            } else {
                Some(link_size as usize)
            },
            link_offset: if link_offset == 0 {
                None
            } else {
                Some(link_offset as usize)
            },
            map_offset: map_offset,
            string_ids_size: string_ids_size,
            string_ids_offset: if string_ids_offset > 0 {
                Some(string_ids_offset)
            } else {
                None
            },
            type_ids_size: type_ids_size,
            type_ids_offset: if type_ids_offset > 0 {
                Some(type_ids_offset)
            } else {
                None
            },
            prototype_ids_size: prototype_ids_size,
            prototype_ids_offset: if prototype_ids_size > 0 {
                Some(prototype_ids_offset)
            } else {
                None
            },
            field_ids_size: field_ids_size,
            field_ids_offset: if field_ids_size > 0 {
                Some(field_ids_offset)
            } else {
                None
            },
            method_ids_size: method_ids_size,
            method_ids_offset: if method_ids_size > 0 {
                Some(method_ids_offset)
            } else {
                None
            },
            class_defs_size: class_defs_size,
            class_defs_offset: if class_defs_size > 0 {
                Some(class_defs_offset)
            } else {
                None
            },
            data_size: data_size,
            data_offset: data_offset,
        })
    }

    /// Checks if the dex magic number given is valid.
    fn is_magic_valid(magic: &[u8; 8]) -> bool {
        &magic[0..4] == &[0x64, 0x65, 0x78, 0x0a] && magic[7] == 0x00 &&
        magic[4] >= 0x30 && magic[5] >= 0x30 && magic[6] >= 0x30 && magic[4] <= 0x39 &&
        magic[5] <= 0x39 && magic[6] <= 0x39
    }

    /// Gets the magic value.
    pub fn get_magic(&self) -> &[u8; 8] {
        &self.magic
    }

    /// Gets Dex version.
    pub fn get_dex_version(&self) -> u8 {
        (self.magic[4] - 0x30) * 100 + (self.magic[5] - 0x30) * 10 + (self.magic[6] - 0x30)
    }

    /// Gets file checksum.
    pub fn get_checksum(&self) -> u32 {
        self.checksum
    }

    /// Gets file SHA-1 signature.
    pub fn get_signature(&self) -> &[u8; 20] {
        &self.signature
    }

    /// Gets file size.
    pub fn get_file_size(&self) -> usize {
        self.file_size
    }

    /// Gets header size, in bytes.
    ///
    /// This must be 0x70.
    pub fn get_header_size(&self) -> usize {
        self.header_size
    }

    /// Gets the endian tag.
    ///
    /// This must be `ENDIAN_CONSTANT` or `REVERSE_ENDIAN_CONSTANT`.
    pub fn get_endian_tag(&self) -> u32 {
        self.endian_tag
    }

    /// Gets wether the file is in little endian or not.
    pub fn is_little_endian(&self) -> bool {
        self.endian_tag == ENDIAN_CONSTANT
    }

    /// Gets wether the file is in big endian or not.
    pub fn is_big_endian(&self) -> bool {
        self.endian_tag == REVERSE_ENDIAN_CONSTANT
    }

    /// Gets the link section size
    pub fn get_link_size(&self) -> Option<usize> {
        self.link_size
    }

    /// Gets the link section offset.
    pub fn get_link_offset(&self) -> Option<usize> {
        self.link_offset
    }

    /// Gets the map section offset.
    pub fn get_map_offset(&self) -> usize {
        self.map_offset
    }

    /// Gets the string IDs list size.
    pub fn get_string_ids_size(&self) -> usize {
        self.string_ids_size
    }

    /// Gets the string IDs list offset.
    pub fn get_string_ids_offset(&self) -> Option<usize> {
        self.string_ids_offset
    }

    /// Gets the type IDs list size.
    pub fn get_type_ids_size(&self) -> usize {
        self.type_ids_size
    }

    /// Gets the type IDs list offset.
    pub fn get_type_ids_offset(&self) -> Option<usize> {
        self.type_ids_offset
    }

    /// Gets the prototype IDs list size.
    pub fn get_prototype_ids_size(&self) -> usize {
        self.prototype_ids_size
    }

    /// Gets the prototype IDs list offset.
    pub fn get_prototype_ids_offset(&self) -> Option<usize> {
        self.prototype_ids_offset
    }

    /// Gets the field IDs list size.
    pub fn get_field_ids_size(&self) -> usize {
        self.field_ids_size
    }

    /// Gets the field IDs list offset.
    pub fn get_field_ids_offset(&self) -> Option<usize> {
        self.field_ids_offset
    }

    /// Gets the method IDs list size.
    pub fn get_method_ids_size(&self) -> usize {
        self.method_ids_size
    }

    /// Gets the method IDs list offset.
    pub fn get_method_ids_offset(&self) -> Option<usize> {
        self.method_ids_offset
    }

    /// Gets the class definition list size.
    pub fn get_class_defs_size(&self) -> usize {
        self.class_defs_size
    }

    /// Gets the class definition list offset.
    pub fn get_class_defs_offset(&self) -> Option<usize> {
        self.class_defs_offset
    }

    /// Gets the data section size.
    pub fn get_data_size(&self) -> usize {
        self.data_size
    }

    /// Gets the data secrion offset.
    pub fn get_data_offset(&self) -> usize {
        self.data_offset
    }
}

#[derive(Debug)]
struct Map {
    map_list: Vec<MapItem>,
}

impl Map {
    fn from_reader<R: Read + Seek, B: ByteOrder>(reader: &mut R) -> Result<Map> {
        let size = try!(reader.read_u32::<B>()) as usize;
        let mut map_list = Vec::with_capacity(size);
        for _ in 0..size {
            let item_type = try!(reader.read_u16::<B>());
            try!(reader.seek(SeekFrom::Current(2)));
            let size = try!(reader.read_u32::<B>());
            let offset = try!(reader.read_u32::<B>());
            map_list.push(try!(MapItem::new(item_type, size, offset)));
        }
        map_list.sort_by_key(|i| i.get_offset());
        Ok(Map { map_list: map_list })
    }

    fn get_item_list(&self) -> &[MapItem] {
        &self.map_list
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
