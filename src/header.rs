//! Module containing the Dex file header.

use std::path::Path;
use std::{fmt, fs, u32};
use std::io::{BufReader, Read};

use byteorder::{LittleEndian, BigEndian, ByteOrder, ReadBytesExt};

use error::*;
use sizes::*;

/// Endianness constant representing little endian file.
pub const ENDIAN_CONSTANT: u32 = 0x12345678;
/// Endianness constant representing big endian file.
pub const REVERSE_ENDIAN_CONSTANT: u32 = 0x78563412;

/// Dex header representantion structure.
pub struct Header {
    magic: [u8; 8],
    checksum: u32,
    signature: [u8; 20],
    file_size: u32,
    header_size: u32,
    endian_tag: u32,
    link_size: u32,
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

impl Header {
    /// Obtains the header from a Dex file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Header> {
        let f = fs::File::open(path).chain_err(|| "could not open file")?;
        let file_size = f.metadata()
            .chain_err(|| "could not read file metadata")?
            .len();
        if file_size < HEADER_SIZE as u64 || file_size > (u32::MAX as u64) {
            return Err(ErrorKind::InvalidFileSize(file_size).into());
        }
        let header = Header::from_reader(BufReader::new(f)).chain_err(|| {
                           ErrorKind::Header("there was an error reading the header of the dex file"
                                                 .to_owned())
                       })?;
        if file_size == header.get_file_size() as u64 {
            Ok(header)
        } else {
            Err(ErrorKind::HeaderFileSizeMismatch(file_size, header.get_file_size()).into())
        }
    }

    /// Obtains the header from a Dex file reader.
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Header> {
        // Magic number
        let mut magic = [0_u8; 8];
        reader.read_exact(&mut magic).chain_err(|| "could not read dex magic number")?;
        if !Header::is_magic_valid(&magic) {
            return Err(ErrorKind::IncorrectMagic(magic).into());
        }
        // Checksum
        let mut checksum = reader.read_u32::<LittleEndian>()
            .chain_err(|| "could not read file checksum")?;
        // Signature
        let mut signature = [0_u8; 20];
        reader.read_exact(&mut signature).chain_err(|| "could not read file signature")?;
        // File size
        let mut file_size = reader.read_u32::<LittleEndian>()
            .chain_err(|| "could not read file size")?;
        // Header size
        let mut header_size = reader.read_u32::<LittleEndian>()
            .chain_err(|| "could not read header size")?;
        // Endian tag
        let endian_tag = reader.read_u32::<LittleEndian>()
            .chain_err(|| "could not read endian tag")?;

        // Check endianness
        if endian_tag == REVERSE_ENDIAN_CONSTANT {
            // The file is in big endian instead of little endian.
            checksum = checksum.swap_bytes();
            file_size = file_size.swap_bytes();
            header_size = header_size.swap_bytes();
        } else if endian_tag != ENDIAN_CONSTANT {
            return Err(ErrorKind::InvalidEndianTag(endian_tag).into());
        }

        // Check header size
        if header_size != HEADER_SIZE {
            return Err(ErrorKind::IncorrectHeaderSize(header_size).into());
        }

        if endian_tag == ENDIAN_CONSTANT {
            Header::read_data::<_, LittleEndian>(reader,
                                                 magic,
                                                 checksum,
                                                 signature,
                                                 file_size,
                                                 header_size,
                                                 ENDIAN_CONSTANT)
        } else {
            Header::read_data::<_, BigEndian>(reader,
                                              magic,
                                              checksum,
                                              signature,
                                              file_size,
                                              header_size,
                                              REVERSE_ENDIAN_CONSTANT)
        }
    }

    fn read_data<R: Read, E: ByteOrder>(mut reader: R,
                                        magic: [u8; 8],
                                        checksum: u32,
                                        signature: [u8; 20],
                                        file_size: u32,
                                        header_size: u32,
                                        endian_tag: u32)
                                        -> Result<Header> {
        #[inline]
        fn some_if(x: u32, b: bool) -> Option<u32> {
            if b { Some(x) } else { None }
        }
        let mut current_offset = HEADER_SIZE;

        // Link size
        let link_size =
            reader.read_u32::<E>().chain_err(|| "could not read the link section size")?;
        // Link offset
        let link_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the link section offset")?;
        if link_size == 0 && link_offset != 0 {
            return Err(ErrorKind::MismatchedOffsets("link_offset", link_offset, 0).into());
        }

        // Map offset
        let map_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the map section offset")?;
        if map_offset == 0x00000000 {
            return Err(ErrorKind::InvalidOffset("`map_offset` was 0x00000000, and it can never \
                                                 be zero"
                                                        .to_owned())
                               .into());
        }

        // String IDs size
        let string_ids_size = reader.read_u32::<E>()
            .chain_err(|| "could not read the string IDs list size")?;
        // String IDs offset
        let string_ids_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the string IDs list offset")?;
        if string_ids_size > 0 && string_ids_offset != current_offset {
            return Err(ErrorKind::MismatchedOffsets("string_ids_offset",
                                                    string_ids_offset,
                                                    HEADER_SIZE)
                               .into());
        }
        if string_ids_size == 0 && string_ids_offset != 0 {
            return Err(ErrorKind::MismatchedOffsets("string_ids_offset", string_ids_offset, 0)
                           .into());
        }
        current_offset += string_ids_size * STRING_ID_ITEM_SIZE;

        // Types IDs size
        let type_ids_size = reader.read_u32::<E>()
            .chain_err(|| "could not read the type IDs list size")?;
        // Types IDs offset
        let type_ids_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the type IDs list offset")?;
        if type_ids_size > 0 && type_ids_offset != current_offset {
            return Err(ErrorKind::MismatchedOffsets("type_ids_offset",
                                                    type_ids_offset,
                                                    current_offset)
                               .into());
        }
        if type_ids_size == 0 && type_ids_offset != 0 {
            return Err(ErrorKind::MismatchedOffsets("type_ids_offset", type_ids_offset, 0).into());
        }
        current_offset += type_ids_size * TYPE_ID_ITEM_SIZE;

        // Prototype IDs size
        let prototype_ids_size = reader.read_u32::<E>()
            .chain_err(|| "could not read the prototype IDs list size")?;
        // Prototype IDs offset
        let prototype_ids_offset =
            reader.read_u32::<E>().chain_err(|| "could not read the prototype IDs list offset")?;
        if prototype_ids_size > 0 && prototype_ids_offset != current_offset {
            return Err(ErrorKind::MismatchedOffsets("prototype_ids_offset",
                                                    prototype_ids_offset,
                                                    current_offset)
                               .into());
        }
        if prototype_ids_size == 0 && prototype_ids_offset != 0 {
            return Err(ErrorKind::MismatchedOffsets("prototype_ids_offset",
                                                    prototype_ids_offset,
                                                    0)
                               .into());
        }
        current_offset += prototype_ids_size * PROTO_ID_ITEM_SIZE;

        // Field IDs size
        let field_ids_size = reader.read_u32::<E>()
            .chain_err(|| "could not read the field IDs list size")?;
        // Field IDs offset
        let field_ids_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the field IDs list offset")?;
        if field_ids_size > 0 && field_ids_offset != current_offset {
            return Err(ErrorKind::MismatchedOffsets("field_ids_offset",
                                                    field_ids_offset,
                                                    current_offset)
                               .into());
        }
        if field_ids_size == 0 && field_ids_offset != 0 {
            return Err(ErrorKind::MismatchedOffsets("field_ids_offset", field_ids_offset, 0)
                           .into());
        }
        current_offset += field_ids_size * FIELD_ID_ITEM_SIZE;

        // Method IDs size
        let method_ids_size = reader.read_u32::<E>()
            .chain_err(|| "could not read the method IDs list size")?;
        // Method IDs offset
        let method_ids_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the method IDs list offset")?;
        if method_ids_size > 0 && method_ids_offset != current_offset {
            return Err(ErrorKind::MismatchedOffsets("method_ids_offset",
                                                    method_ids_offset,
                                                    current_offset)
                               .into());
        }
        if method_ids_size == 0 && method_ids_offset != 0 {
            return Err(ErrorKind::MismatchedOffsets("method_ids_offset", method_ids_offset, 0)
                           .into());
        }
        current_offset += method_ids_size * METHOD_ID_ITEM_SIZE;

        // Class defs size
        let class_defs_size = reader.read_u32::<E>()
            .chain_err(|| "could not read the class definitions list size")?;
        // Class defs offset
        let class_defs_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the class definitions list offset")?;
        if class_defs_size > 0 && class_defs_offset != current_offset {
            return Err(ErrorKind::MismatchedOffsets("class_defs_offset",
                                                    class_defs_offset,
                                                    current_offset)
                               .into());
        }
        if class_defs_size == 0 && class_defs_offset != 0 {
            return Err(ErrorKind::MismatchedOffsets("class_defs_offset", class_defs_offset, 0)
                           .into());
        }
        current_offset += class_defs_size * CLASS_DEF_ITEM_SIZE;

        // Data size
        let data_size =
            reader.read_u32::<E>().chain_err(|| "could not read the data section size")?;
        if data_size & 0b11 != 0 {
            return Err(ErrorKind::Header(format!("`data_size` must be a 4-byte multiple, but \
                                                  it was {:#010x}",
                                                 data_size))
                               .into());
        }

        // Data offset
        let data_offset = reader.read_u32::<E>()
            .chain_err(|| "could not read the data section offset")?;
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
            return Err(ErrorKind::InvalidOffset(format!("`map_offset` section must be in the \
                                                         `data` section (between {:#010x} and \
                                                         {:#010x}) but it was at {:#010x}",
                                                        data_offset,
                                                        current_offset,
                                                        map_offset))
                               .into());
        }
        if link_size == 0 && current_offset != file_size {
            return Err(ErrorKind::Header(format!("`data` section must end at the EOF if there \
                                                  are no links in the file. Data end: \
                                                  {:#010x}, `file_size`: {:#010x}",
                                                 current_offset,
                                                 file_size))
                               .into());

        }
        if link_size != 0 && link_offset == 0 {
            return Err(ErrorKind::MismatchedOffsets("link_offset", 0, current_offset).into());
        }
        if link_size != 0 && link_offset != 0 {
            if link_offset != current_offset {
                return Err(ErrorKind::MismatchedOffsets("link_offset",
                                                        link_offset,
                                                        current_offset)
                                   .into());
            }
            if link_offset + link_size != file_size {
                return Err(ErrorKind::Header("`link_data` section must end at the end \
                                                       of file"
                                                     .to_owned())
                                   .into());
            }
        }

        Ok(Header {
               magic: magic,
               checksum: checksum,
               signature: signature,
               file_size: file_size,
               header_size: header_size,
               endian_tag: endian_tag,
               link_size: link_size,
               link_offset: some_if(link_offset, link_offset != 0),
               map_offset: map_offset,
               string_ids_size: string_ids_size,
               string_ids_offset: some_if(string_ids_offset, string_ids_offset > 0),
               type_ids_size: type_ids_size,
               type_ids_offset: some_if(type_ids_offset, type_ids_offset > 0),
               prototype_ids_size: prototype_ids_size,
               prototype_ids_offset: some_if(prototype_ids_offset, prototype_ids_size > 0),
               field_ids_size: field_ids_size,
               field_ids_offset: some_if(field_ids_offset, field_ids_size > 0),
               method_ids_size: method_ids_size,
               method_ids_offset: some_if(method_ids_offset, method_ids_size > 0),
               class_defs_size: class_defs_size,
               class_defs_offset: some_if(class_defs_offset, class_defs_size > 0),
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
    pub fn get_link_size(&self) -> u32 {
        self.link_size
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
    pub fn get_string_ids_size(&self) -> u32 {
        self.string_ids_size
    }

    /// Gets the string IDs list offset.
    pub fn get_string_ids_offset(&self) -> Option<u32> {
        self.string_ids_offset
    }

    /// Gets the type IDs list size.
    pub fn get_type_ids_size(&self) -> u32 {
        self.type_ids_size
    }

    /// Gets the type IDs list offset.
    pub fn get_type_ids_offset(&self) -> Option<u32> {
        self.type_ids_offset
    }

    /// Gets the prototype IDs list size.
    pub fn get_prototype_ids_size(&self) -> u32 {
        self.prototype_ids_size
    }

    /// Gets the prototype IDs list offset.
    pub fn get_prototype_ids_offset(&self) -> Option<u32> {
        self.prototype_ids_offset
    }

    /// Gets the field IDs list size.
    pub fn get_field_ids_size(&self) -> u32 {
        self.field_ids_size
    }

    /// Gets the field IDs list offset.
    pub fn get_field_ids_offset(&self) -> Option<u32> {
        self.field_ids_offset
    }

    /// Gets the method IDs list size.
    pub fn get_method_ids_size(&self) -> u32 {
        self.method_ids_size
    }

    /// Gets the method IDs list offset.
    pub fn get_method_ids_offset(&self) -> Option<u32> {
        self.method_ids_offset
    }

    /// Gets the class definition list size.
    pub fn get_class_defs_size(&self) -> u32 {
        self.class_defs_size
    }

    /// Gets the class definition list offset.
    pub fn get_class_defs_offset(&self) -> Option<u32> {
        self.class_defs_offset
    }

    /// Gets the data section size.
    pub fn get_data_size(&self) -> u32 {
        self.data_size
    }

    /// Gets the data secrion offset.
    pub fn get_data_offset(&self) -> u32 {
        self.data_offset
    }

    // /// Verifies the file at the given path.
    // pub fn verify_file<P: AsRef<Path>>(&self, path: P) -> bool {
    //     unimplemented!() // TODO
    // }
    //
    // /// Verifies the file in the given reader.
    // ///
    // /// The reader should be positioned at the start of the file.
    // pub fn verify_reader<R: Read>(&self, mut reader: R) -> bool {
    //     unimplemented!() // TODO
    // }
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
               if let Some(off) = self.link_offset {
                   format!("link_size: {} bytes, link_offset: {:#x}",
                           self.link_size,
                           off)
               } else {
                   String::from("no link section")
               },
               self.map_offset,
               if let Some(off) = self.string_ids_offset {
                   format!("string_ids_size: {} strings, string_ids_offset: {:#x}",
                           self.string_ids_size,
                           off)
               } else {
                   String::from("no strings")
               },
               if let Some(off) = self.type_ids_offset {
                   format!("type_ids_size: {} types, type_ids_offset: {:#x}",
                           self.type_ids_size,
                           off)
               } else {
                   String::from("no types")
               },
               if let Some(off) = self.prototype_ids_offset {
                   format!("prototype_ids_size: {} types, prototype_ids_offset: {:#x}",
                           self.prototype_ids_size,
                           off)
               } else {
                   String::from("no prototypes")
               },
               if let Some(off) = self.field_ids_offset {
                   format!("field_ids_size: {} types, field_ids_offset: {:#x}",
                           self.field_ids_size,
                           off)
               } else {
                   String::from("no fields")
               },
               if let Some(off) = self.method_ids_offset {
                   format!("method_ids_size: {} types, method_ids_offset: {:#x}",
                           self.method_ids_size,
                           off)
               } else {
                   String::from("no methods")
               },
               if let Some(off) = self.class_defs_offset {
                   format!("class_defs_size: {} classes, class_defs_offset: {:#x}",
                           self.class_defs_size,
                           off)
               } else {
                   String::from("no classes")
               },
               self.data_size,
               self.data_offset)
    }
}
