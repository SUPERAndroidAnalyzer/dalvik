//! Dalvik executable file format parser.

#![recursion_limit = "1024"]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![forbid(anonymous_parameters)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy_pedantic))]
#![deny(
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
// Allowing these for now.
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        stutter,
        similar_names,
        cast_possible_truncation,
        cast_possible_wrap
    )
)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate bitflags;
extern crate byteorder;

#[cfg(test)]
#[macro_use]
extern crate matches;

use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::{fs, u32};

use failure::{Error, ResultExt};

pub mod bytecode;
pub mod error;
pub mod header;
pub mod types;

mod read;
mod sizes;

pub use header::Header;
use read::DexReader;
use sizes::HEADER_SIZE;

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
            DexReader::new(reader, size.into()).context("could not create reader")?;
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
