//! Dalvik executable file format parser.

#![forbid(anonymous_parameters)]
#![deny(
    clippy::all,
    variant_size_differences,
    unused_results,
    unused_qualifications,
    unused_import_braces,
    unsafe_code,
    trivial_numeric_casts,
    trivial_casts,
    missing_docs,
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]
// Allowing these for now.
#![allow(
    clippy::stutter,
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

use std::{
    fs,
    io::{prelude::BufRead, BufReader},
    path::Path,
    u32,
};

use failure::{Error, ResultExt};

pub mod bytecode;
pub mod error;
pub mod header;
pub mod types;

mod read;
mod sizes;

pub use crate::header::Header;
use crate::{read::DexReader, sizes::HEADER_SIZE};

/// Dex file representation.
#[derive(Debug)]
pub struct Dex {
    header: Header,
    strings: Vec<String>,
    types: Vec<String>,
}

impl Dex {
    /// Reads the Dex data structure from the given path.
    pub fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path).context("could not open file")?;
        let file_size = file
            .metadata()
            .context("could not read file metadata")?
            .len();
        if file_size < u64::from(HEADER_SIZE) || file_size > u64::from(u32::max_value()) {
            return Err(error::InvalidFileSize { file_size }.into());
        }
        Self::from_reader(BufReader::new(file), file_size as usize)
    }

    /// Loads a new Dex data structure from the given reader.
    pub fn from_reader<R, S>(reader: R, size: S) -> Result<Self, Error>
    where
        R: BufRead,
        S: Into<Option<usize>>,
    {
        let mut dex_reader =
            DexReader::from_read(reader, size.into()).context("could not create reader")?;
        dex_reader.read_data().context("could not read dex file")?;

        Ok(dex_reader.into())
    }

    // /// Ads the file in the given path to the current Dex data structure.
    // pub fn add_file<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    //     unimplemented!() // TODO
    // }
    //
    // /// Verifies the file at the given path.
    // pub fn verify_file<P: AsRef<Path>>(&self, path: P) -> bool {
    //     self.header.verify_file(path) // TODO
    // }
    //
    // /// Verifies the file in the given reader.
    // ///
    // /// The reader should be positioned at the start of the file.
    // pub fn verify_reader<R: Read>(&self, reader: R) -> bool {
    //     self.header.verify_reader(reader) // TODO
    // }
}

impl From<DexReader> for Dex {
    fn from(_reader: DexReader) -> Self {
        unimplemented!()
    }
}
