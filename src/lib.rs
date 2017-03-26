//! Dalvik executable file format parser.

// #![forbid(deprecated, overflowing_literals, stable_features, trivial_casts,
// unconditional_recursion,
//     plugin_as_library, unused_allocation, trivial_numeric_casts, unused_features, while_truem,
//     unused_parens, unused_comparisons, unused_extern_crates, unused_import_braces,
// unused_results,
//     improper_ctypes, non_shorthand_field_patterns, private_no_mangle_fns,
// private_no_mangle_statics,
//     filter_map, used_underscore_binding, option_map_unwrap_or, option_map_unwrap_or_else,
//     mutex_integer, mut_mut, mem_forget, print_stdout)]
// #![deny(unused_qualifications, unused, unused_attributes)]
#![warn(missing_docs, variant_size_differences, enum_glob_use, if_not_else,
    invalid_upcast_comparisons, items_after_statements, non_ascii_literal, nonminimal_bool,
    pub_enum_variant_names, shadow_reuse, shadow_same, shadow_unrelated, similar_names,
    single_match_else, string_add, string_add_assign, unicode_not_nfc, unseparated_literal_suffix,
    use_debug, wrong_pub_self_convention)]
// Allowing these at least for now.
#![allow(missing_docs_in_private_items, unknown_lints, stutter, option_unwrap_used,
    result_unwrap_used, integer_arithmetic, cast_possible_truncation, cast_possible_wrap,
    indexing_slicing, cast_precision_loss, cast_sign_loss)]

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


pub mod error;
pub mod header;
pub mod types;

mod sizes;
mod read;

use error::*;
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Dex> {
        let file = fs::File::open(path).chain_err(|| "could not open file")?;
        let file_size = file.metadata()
            .chain_err(|| "could not read file metadata")?
            .len();
        if file_size < HEADER_SIZE as u64 || file_size > u32::MAX as u64 {
            return Err(ErrorKind::InvalidFileSize(file_size).into());
        }
        Dex::from_reader(BufReader::new(file), Some(file_size as usize))
    }

    /// Loads a new Dex data structure from the given reader.
    pub fn from_reader<R: BufRead>(reader: R, size: Option<usize>) -> Result<Dex> {
        let mut dex_reader = DexReader::new(reader, size).chain_err(|| "could not create reader")?;
        dex_reader.read_data().chain_err(|| "could not read dex file")?;

        Ok(dex_reader.into())
    }

    // /// Ads the file in the given path to the current Dex data structure.
    // pub fn add_file<P: AsRef<Path>>(path: P) -> Result<()> {
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
    fn from(reader: DexReader) -> Dex {
        unimplemented!()
    }
}
