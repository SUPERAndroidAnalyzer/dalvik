use std::path::Path;
use std::{fmt, fs, usize};
use std::io::{BufReader, Read};

use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

use error::{Error, Result};
use sizes::*;
use offset_map::{OffsetMap, OffsetType};

pub const ENDIAN_CONSTANT: u32 = 0x12345678;
pub const REVERSE_ENDIAN_CONSTANT: u32 = 0x78563412;

/// Dex header representantion structure.
pub struct Header {
    magic: [u8; 8],
    checksum: u32,
    signature: [u8; 20],
    file_size: u32,
    header_size: u32,
    endian_tag: u32,
    link_size: Option<u32>,
    link_offset: Option<u32>,
    map_offset: u32,
    string_ids_size: u32,
    string_ids_offset: Option<u32>,
    type_ids_size: u32,
    type_ids_offset: Option<u32>,
    prototype_ids_size: u32,
    prototype_ids_offset: Option<u32>,
    field_ids_size: u32,
    field_ids_offset: Option<u32>,
    method_ids_size: u32,
    method_ids_offset: Option<u32>,
    class_defs_size: u32,
    class_defs_offset: Option<u32>,
    data_size: u32,
    data_offset: u32,
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Header> {
        let f = try!(fs::File::open(path));
        let file_size = try!(f.metadata()).len();
        if file_size < HEADER_SIZE as u64 || file_size > usize::MAX as u64 {
            return Err(Error::invalid_file_size(file_size, None));
        }
        let header = try!(Header::from_reader(BufReader::new(f)));
        if file_size != header.get_file_size() as u64 {
            Err(Error::invalid_file_size(file_size, Some(header.get_file_size())))
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
        });
        // Link offset
        let link_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
        if link_size == 0 && link_offset != 0 {
            return Err(Error::mismatched_offsets("link_offset", link_offset, 0));
        }

        // Map offset
        let map_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
        if map_offset == 0 {
            return Err(Error::MismatchedOffsets(String::from("`map_offset` was 0x00, and it \
                                                              can never be zero")));
        }

        // String IDs size
        let string_ids_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
        // String IDs offset
        let string_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
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
        });
        // Types IDs offset
        let type_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
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
        });
        // Prototype IDs offset
        let prototype_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
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
        });
        // Field IDs offset
        let field_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
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
        });
        // Method IDs offset
        let method_ids_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
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
        });
        // Class defs offset
        let class_defs_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
        if class_defs_size > 0 && class_defs_offset != current_offset {
            return Err(Error::mismatched_offsets("class_defs_offset",
                                                 class_defs_offset,
                                                 current_offset));
        }
        if class_defs_size == 0 && class_defs_offset != 0 {
            return Err(Error::mismatched_offsets("class_defs_offset", class_defs_offset, 0));
        }
        current_offset += class_defs_size * CLASS_DEF_ITEM_SIZE;

        // Data size
        let data_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
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
        });
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
                Some(link_size)
            },
            link_offset: if link_offset == 0 {
                None
            } else {
                Some(link_offset)
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
    pub fn get_file_size(&self) -> u32 {
        self.file_size
    }

    /// Gets header size, in bytes.
    ///
    /// This must be 0x70.
    pub fn get_header_size(&self) -> u32 {
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
        match self.link_size {
            Some(s) => Some(s as usize),
            None => None,
        }
    }

    /// Gets the link section offset.
    pub fn get_link_offset(&self) -> Option<u32> {
        self.link_offset
    }

    /// Gets the map section offset.
    pub fn get_map_offset(&self) -> u32 {
        self.map_offset
    }

    /// Gets the string IDs list size.
    pub fn get_string_ids_size(&self) -> usize {
        self.string_ids_size as usize
    }

    /// Gets the string IDs list offset.
    pub fn get_string_ids_offset(&self) -> Option<u32> {
        self.string_ids_offset
    }

    /// Gets the type IDs list size.
    pub fn get_type_ids_size(&self) -> usize {
        self.type_ids_size as usize
    }

    /// Gets the type IDs list offset.
    pub fn get_type_ids_offset(&self) -> Option<u32> {
        self.type_ids_offset
    }

    /// Gets the prototype IDs list size.
    pub fn get_prototype_ids_size(&self) -> usize {
        self.prototype_ids_size as usize
    }

    /// Gets the prototype IDs list offset.
    pub fn get_prototype_ids_offset(&self) -> Option<u32> {
        self.prototype_ids_offset
    }

    /// Gets the field IDs list size.
    pub fn get_field_ids_size(&self) -> usize {
        self.field_ids_size as usize
    }

    /// Gets the field IDs list offset.
    pub fn get_field_ids_offset(&self) -> Option<u32> {
        self.field_ids_offset
    }

    /// Gets the method IDs list size.
    pub fn get_method_ids_size(&self) -> usize {
        self.method_ids_size as usize
    }

    /// Gets the method IDs list offset.
    pub fn get_method_ids_offset(&self) -> Option<u32> {
        self.method_ids_offset
    }

    /// Gets the class definition list size.
    pub fn get_class_defs_size(&self) -> usize {
        self.class_defs_size as usize
    }

    /// Gets the class definition list offset.
    pub fn get_class_defs_offset(&self) -> Option<u32> {
        self.class_defs_offset
    }

    /// Gets the data section size.
    pub fn get_data_size(&self) -> usize {
        self.data_size as usize
    }

    /// Gets the data secrion offset.
    pub fn get_data_offset(&self) -> u32 {
        self.data_offset
    }

    pub fn generate_offset_map(&self) -> OffsetMap {
        let mut offset_map = OffsetMap::with_capacity(7 + self.string_ids_size as usize +
                                                      self.prototype_ids_size as usize +
                                                      self.class_defs_size as usize * 4);
        if let Some(offset) = self.string_ids_offset {
            offset_map.insert(offset, OffsetType::StringIdList);
        }
        if let Some(offset) = self.type_ids_offset {
            offset_map.insert(offset, OffsetType::TypeIdList);
        }
        if let Some(offset) = self.prototype_ids_offset {
            offset_map.insert(offset, OffsetType::PrototypeIdList);
        }
        if let Some(offset) = self.field_ids_offset {
            offset_map.insert(offset, OffsetType::FieldIdList);
        }
        if let Some(offset) = self.method_ids_offset {
            offset_map.insert(offset, OffsetType::MethodIdList);
        }
        if let Some(offset) = self.class_defs_offset {
            offset_map.insert(offset, OffsetType::ClassDefList);
        }
        offset_map.insert(self.map_offset, OffsetType::Map);

        offset_map
    }

    /// Verifies the file at the given path.
    pub fn verify_file<P: AsRef<Path>>(&self, path: P) -> bool {
        unimplemented!()
    }

    /// Verifies the file in the given reader.
    ///
    /// The reader should be positioned at the start of the file.
    pub fn verify_reader<R: Read>(&self, reader: R) -> bool {
        unimplemented!()
    }
}
