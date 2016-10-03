use std::path::Path;
use std::{fmt, io, fs, usize};
use std::io::BufReader;

pub mod error;
pub mod bytecode;
mod types;

use error::{Result, Error};

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
    pub fn new<P: AsRef<Path>>(path: P, verify: bool) -> Result<Dex> {
        // TODO
        unimplemented!()
    }
    pub fn add_file<P: AsRef<Path>>(path: P) -> Result<()> {
        unimplemented!()
    }
}

pub const ENDIAN_CONSTANT: u32 = 0x12345678;
pub const REVERSE_ENDIAN_CONSTANT: u32 = 0x78563412;
pub const HEADER_SIZE: usize = 0x70;

/// Dex header representantion structure.
#[derive(Clone)]
pub struct Header {
    magic: [u8; 8],
    checksum: u32,
    signature: [u8; 20],
    file_size: usize,
    header_size: usize,
    endian_tag: u32,
    link_size: usize,
    link_offset: usize,
    map_offset: usize,
    string_ids_size: usize,
    string_ids_offset: usize,
    types_ids_size: usize,
    types_ids_offset: usize,
    proto_ids_size: usize,
    proto_ids_offset: usize,
    field_ids_size: usize,
    field_ids_offset: usize,
    class_defs_size: usize,
    class_defs_offset: usize,
    data_size: usize,
    data_offset: usize,
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "DexHeader {{ magic: {:?} (version: {}), checksum: {} signature: {}, file_size: \
                {}, header_size: {}, endian_tag: {}, link_size: {}, link_offset: {}, map_offset: \
                {}, string_ids_size: {}, string_ids_offset: {}, types_ids_size: {}, \
                types_ids_offset: {}, proto_ids_size: {}, proto_ids_offset: {}, field_ids_size: \
                {}, field_ids_offset: {}, class_defs_size: {}, class_defs_offset: {}, data_size: \
                {}, data_offset: {} }}",
               self.magic,
               self.get_dex_version(),
               self.checksum,
               {
                   let mut signature = String::with_capacity(40);
                   for b in &self.signature {
                       signature.push_str(&format!("{:2x}", b))
                   }
                   signature
               },
               self.file_size,
               self.header_size,
               self.endian_tag,
               self.link_size,
               self.link_offset,
               self.map_offset,
               self.string_ids_size,
               self.string_ids_offset,
               self.types_ids_size,
               self.types_ids_offset,
               self.proto_ids_size,
               self.proto_ids_offset,
               self.field_ids_size,
               self.field_ids_offset,
               self.class_defs_size,
               self.class_defs_offset,
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
            return Err(Error::invalid_dex_file_size(file_size, None));
        }
        let reader = BufReader::new(f);
        let header = try!(Header::from_reader(reader));
        if file_size as usize != header.get_file_size() {
            Err(Error::invalid_dex_file_size(file_size, Some(header.get_file_size())))
        } else {
            Ok(header)
        }
    }

    /// Obtains the header from a Dex file reader.
    pub fn from_reader<R: io::Read>(mut reader: R) -> Result<Header> {
        let mut magic = [0u8; 8];
        try!(reader.read_exact(&mut magic));
        if !Header::is_magic_valid(&magic) {
            return Err(Error::invalid_dex_magic(magic));
        }
        unimplemented!()
    }

    /// Checks if the dex magic number given is valid.
    fn is_magic_valid(magic: &[u8; 8]) -> bool {
        &magic[0..3] == &[0x64, 0x65, 0x78, 0x0a] && magic[7] == 0x00 &&
        magic[4] >= 0x30 && magic[5] >= 0x30 && magic[6] >= 0x30 && magic[4] <= 0x39 &&
        magic[5] <= 0x39 && magic[6] <= 0x39
    }

    /// Gets the magic value.
    pub fn get_magic(&self) -> &[u8; 8] {
        &self.magic
    }

    /// Gets Dex version.
    pub fn get_dex_version(&self) -> u8 {
        (self.magic[5] - 0x30) * 100 + (self.magic[6] - 0x30) * 10 + (self.magic[7] - 0x30)
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
    pub fn get_link_size(&self) -> usize {
        self.link_size
    }

    /// Gets the link section offset.
    pub fn get_link_offset(&self) -> usize {
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
    pub fn get_string_ids_offset(&self) -> usize {
        self.string_ids_offset
    }

    /// Gets the types IDs list size.
    pub fn get_types_ids_size(&self) -> usize {
        self.types_ids_size
    }

    /// Gets the types IDs list offset.
    pub fn get_types_ids_offset(&self) -> usize {
        self.types_ids_offset
    }

    /// Gets the prototype IDs list size.
    pub fn get_proto_ids_size(&self) -> usize {
        self.proto_ids_size
    }

    /// Gets the prototype IDs list offset.
    pub fn get_proto_ids_offset(&self) -> usize {
        self.proto_ids_offset
    }

    /// Gets the field IDs list size.
    pub fn get_field_ids_size(&self) -> usize {
        self.field_ids_size
    }

    /// Gets the field IDs list offset.
    pub fn get_field_ids_offset(&self) -> usize {
        self.field_ids_offset
    }

    /// Gets the class definition list size.
    pub fn get_class_defs_size(&self) -> usize {
        self.class_defs_size
    }

    /// Gets the class definition list offset.
    pub fn get_class_defs_offset(&self) -> usize {
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

pub struct Prototype {
    // TODO;
}

pub struct Field {
    // TODO;
}

pub struct Method {
    // TODO;
}

pub struct ClassDef {
    // TODO;
}
