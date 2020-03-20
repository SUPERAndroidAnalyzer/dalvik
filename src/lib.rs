//! Dalvik executable file format parser.

#![forbid(anonymous_parameters, unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(
    clippy::all,
    variant_size_differences,
    unused_results,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    //unreachable_pub,
    trivial_numeric_casts,
    trivial_casts,
    missing_docs,
    missing_doc_code_examples,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    macro_use_extern_crate,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]
#![warn(unused)]
#![allow(clippy::must_use_candidate, rustdoc)]

pub use crate::header::Header;
use crate::{
    read::DexReader,
    sizes::HEADER_SIZE,
    types::{AccessFlags, Type},
};
use anyhow::{Context, Result};
use std::{
    fs,
    io::{prelude::BufRead, BufReader},
    path::Path,
    u32,
};

pub mod bytecode;
pub mod error;
pub mod header;
mod read;
mod sizes;
pub mod types;

/// Dex file representation.
#[derive(Debug)]
pub struct Dex {
    header: Header,
    strings: Vec<String>,
    types: Vec<Class>,
}

impl Dex {
    /// Reads the Dex data structure from the given path.
    pub fn from_file<P>(path: P) -> Result<Self>
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
    pub fn from_reader<R, S>(reader: R, size: S) -> Result<Self>
    where
        R: BufRead,
        S: Into<Option<usize>>,
    {
        let mut dex_reader =
            DexReader::from_read(reader, size.into()).context("could not create reader")?;
        dex_reader.read_data().context("could not read dex file")?;

        Ok(dex_reader.into())
    }

    /// Gets the list of types in the Dalvik information structure.
    pub fn types(&self) -> &[Class] {
        &self.types
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
    fn from(reader: DexReader) -> Self {
        let types = reader
            .classes
            .iter()
            .map(|class| {
                Class {
                    name: match reader
                        .types
                        .get(class.class_index() as usize)
                        .expect("class name not found")
                    {
                        Type::FullyQualifiedName(s) => s.clone(),
                        _ => unreachable!("class name should be a fully qualified name"),
                    },
                    access_flags: class.access_flags(),
                    superclass: if let Some(i) = class.superclass_index() {
                        match reader.types.get(i as usize).expect("superclass not found") {
                            Type::FullyQualifiedName(s) => Some(s.clone()),
                            _ => unreachable!("superclass name should be a fully qualified name"),
                        }
                    } else {
                        None
                    },
                    interfaces: class
                        .interfaces()
                        .iter()
                        .map(|t| match t {
                            Type::FullyQualifiedName(s) => s.clone(),
                            _ => unreachable!("it should be a class name"),
                        })
                        .collect(),
                    source_file: if let Some(i) = class.source_file_index() {
                        Some(
                            reader
                                .strings
                                .get(i as usize)
                                .expect("source file not found")
                                .clone(),
                        )
                    } else {
                        None
                    },
                    // annotations: Option<AnnotationsDirectory>,
                    // static_fields: Vec<Field>,
                    // instance_fields: Vec<Field>,
                    // direct_methods: Vec<Method>,
                    // virtual_methods: Vec<Method>,
                    // static_values: Option<Box<[Value]>>,
                }
            })
            .collect();
        //eprintln!("{:#X?}", types);
        // unimplemented!();
        Self {
            header: reader.header,
            strings: reader.strings,
            types,
        }
    }
}

/// Java class representation.
#[derive(Debug, Clone)]
pub struct Class {
    name: String,
    access_flags: AccessFlags,
    superclass: Option<String>,
    interfaces: Box<[String]>,
    source_file: Option<String>,
    // annotations: Option<AnnotationsDirectory>,
    // static_fields: Vec<Field>,
    // instance_fields: Vec<Field>,
    // direct_methods: Vec<Method>,
    // virtual_methods: Vec<Method>,
    // static_values: Option<Box<[Value]>>,
}

impl Class {
    /// Gets the name of the class.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Gets the access flags of the class.
    pub fn access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    /// Gets the superclass of the class, if any.
    pub fn superclass(&self) -> Option<&String> {
        self.superclass.as_ref()
    }

    /// Gets the list of interfaces implemented by the class.
    pub fn interfaces(&self) -> &[String] {
        &self.interfaces
    }

    /// Gets the name of the source file where the class was implemented.
    pub fn source_file(&self) -> Option<&String> {
        self.source_file.as_ref()
    }
}

/// Class field structure.
#[derive(Debug, Clone)]
pub struct Field {
    access_flags: AccessFlags,
    field_type: String,
    name: String,
}

/// Class method structure.
#[derive(Debug, Clone)]
pub struct Method {
    access_flags: AccessFlags,
    name: String,
    return_type: String,
    parameters: Box<[String]>,
    // TODO: code
}
