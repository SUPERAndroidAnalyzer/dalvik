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

use byteorder::{BigEndian, LittleEndian};

pub mod error;
pub mod header;
mod sizes;
mod types; // TODO: Should not be public
mod offset_map;
mod read;

use error::*;
pub use header::Header;
use read::DexReader;
use sizes::HEADER_SIZE;

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

        let little_endian = header.is_little_endian();
        let mut dex_reader = DexReader::new(header, reader);
        if little_endian {
                dex_reader.read_data::<LittleEndian>()
            } else {
                dex_reader.read_data::<BigEndian>()
            }.chain_err(|| "could not read dex file")?;
        unimplemented!()
        // Ok(dex_reader.into())
    }

    // /// Ads the file in the given path to the current Dex data structure.
    // pub fn add_file<P: AsRef<Path>>(_path: P) -> Result<()> {
    //     unimplemented!()
    // }

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
