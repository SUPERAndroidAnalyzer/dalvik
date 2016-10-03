use std::path::Path;
use std::{fmt, io, fs, usize};
use std::io::{Read, BufReader};

extern crate byteorder;

pub mod error;
pub mod bytecode;
mod types;

use error::{Result, Error};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

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
    /// Loads a new Dex data structure from the file at the given path.
    pub fn new<P: AsRef<Path>>(path: P, verify: bool) -> Result<Dex> {
        let path = path.as_ref();
        let (mut reader, header) = if verify {
            let header = try!(Header::from_file(path, true));
            let f = try!(fs::File::open(path.clone()));
            let reader = BufReader::new(f).bytes().skip(HEADER_SIZE);
            (reader, header)
        } else {
            let f = try!(fs::File::open(path.clone()));
            let mut reader = BufReader::new(f);
            let header = try!(Header::from_reader(&mut reader));
            (reader.bytes().skip(0), header)
        };
        unimplemented!()
    }

    /// Ads the file in the given path to the current Dex data structure.
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
    type_ids_size: usize,
    type_ids_offset: usize,
    prototype_ids_size: usize,
    prototype_ids_offset: usize,
    field_ids_size: usize,
    field_ids_offset: usize,
    method_ids_size: usize,
    method_ids_offset: usize,
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
                {}, string_ids_size: {}, string_ids_offset: {}, type_ids_size: {}, \
                type_ids_offset: {}, proto_ids_size: {}, proto_ids_offset: {}, field_ids_size: \
                {}, field_ids_offset: {}, method_defs_size: {}, method_defs_offset: {}, \
                class_defs_size: {}, class_defs_offset: {}, data_size: {}, data_offset: {} }}",
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
               self.type_ids_size,
               self.type_ids_offset,
               self.prototype_ids_size,
               self.prototype_ids_offset,
               self.field_ids_size,
               self.field_ids_offset,
               self.method_ids_size,
               self.method_ids_offset,
               self.class_defs_size,
               self.class_defs_offset,
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
            return Err(Error::invalid_dex_file_size(file_size, None));
        }
        let header = try!(Header::from_reader(BufReader::new(f)));
        if file_size as usize != header.get_file_size() {
            Err(Error::invalid_dex_file_size(file_size, Some(header.get_file_size())))
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
            return Err(Error::invalid_dex_magic(magic));
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
            return Err(Error::invalid_dex_endian_tag(endian_tag));
        }
        let header_size = header_size as usize;
        let file_size = file_size as usize;
        // Check header size
        if header_size != HEADER_SIZE {
            return Err(Error::invalid_dex_header_size(header_size));
        }
        // Check file size
        if file_size < HEADER_SIZE {
            return Err(Error::invalid_dex_file_size(0, Some(file_size)));
        }

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
        // Map offset
        let map_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
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
        // Data size
        let data_size = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });
        // Data offset
        let data_offset = try!(if endian_tag == ENDIAN_CONSTANT {
            reader.read_u32::<LittleEndian>()
        } else {
            reader.read_u32::<BigEndian>()
        });

        Ok(Header {
            magic: magic,
            checksum: checksum,
            signature: signature,
            file_size: file_size,
            header_size: header_size,
            endian_tag: endian_tag,
            link_size: link_size as usize,
            link_offset: link_offset as usize,
            map_offset: map_offset as usize,
            string_ids_size: string_ids_size as usize,
            string_ids_offset: string_ids_offset as usize,
            type_ids_size: type_ids_size as usize,
            type_ids_offset: type_ids_offset as usize,
            prototype_ids_size: prototype_ids_size as usize,
            prototype_ids_offset: prototype_ids_offset as usize,
            field_ids_size: field_ids_size as usize,
            field_ids_offset: field_ids_offset as usize,
            method_ids_size: method_ids_size as usize,
            method_ids_offset: method_ids_offset as usize,
            class_defs_size: class_defs_size as usize,
            class_defs_offset: class_defs_offset as usize,
            data_size: data_size as usize,
            data_offset: data_offset as usize,
        })
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

    /// Gets the type IDs list size.
    pub fn get_type_ids_size(&self) -> usize {
        self.type_ids_size
    }

    /// Gets the type IDs list offset.
    pub fn get_type_ids_offset(&self) -> usize {
        self.type_ids_offset
    }

    /// Gets the prototype IDs list size.
    pub fn get_prototype_ids_size(&self) -> usize {
        self.prototype_ids_size
    }

    /// Gets the prototype IDs list offset.
    pub fn get_prototype_ids_offset(&self) -> usize {
        self.prototype_ids_offset
    }

    /// Gets the field IDs list size.
    pub fn get_field_ids_size(&self) -> usize {
        self.field_ids_size
    }

    /// Gets the field IDs list offset.
    pub fn get_field_ids_offset(&self) -> usize {
        self.field_ids_offset
    }

    /// Gets the method IDs list size.
    pub fn get_method_ids_size(&self) -> usize {
        self.method_ids_size
    }

    /// Gets the method IDs list offset.
    pub fn get_method_ids_offset(&self) -> usize {
        self.method_ids_offset
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
