

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

        Ok(dex_reader.into())
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

impl<R> From<DexReader<R>> for Dex
    where R: BufRead
{
    fn from(reader: DexReader<R>) -> Dex {
        unimplemented!()
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
