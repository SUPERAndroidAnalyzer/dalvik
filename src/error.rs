//! Errors module

use std::fmt;

use crate::sizes::HEADER_SIZE;

/// Invalid file size.
#[derive(Debug, Fail, Copy, Clone)]
pub struct InvalidFileSize {
    /// Size of the dex file.
    pub file_size: u64,
}

impl fmt::Display for InvalidFileSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid dex file size: file size must be between {} and {} bytes, but the size of \
             the file was {} bytes",
            HEADER_SIZE,
            u32::max_value(),
            self.file_size
        )
    }
}

/// Errors comming from header parsing.
#[derive(Debug, Fail)]
pub enum Header {
    /// Incorrect dex magic number.
    IncorrectMagic {
        /// The found dex magic number.
        dex_magic: [u8; 8],
    },

    /// Mismatch between file size in header and real file size.
    FileSizeMismatch {
        /// The real file size.
        file_size: u64,
        /// The file size in the header.
        size_in_header: u32,
    },

    /// Invalid endian tag.
    InvalidEndianTag {
        /// Endian tag found in the header.
        endian_tag: u32,
    },

    /// Incorrect header size.
    IncorrectHeaderSize {
        /// Header size found in the header.
        header_size: u32,
    },

    /// Generic header error.
    Generic {
        /// Error string.
        error: String,
    },
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Header::{
            FileSizeMismatch, Generic, IncorrectHeaderSize, IncorrectMagic, InvalidEndianTag,
        };
        use crate::header::{ENDIAN_CONSTANT, REVERSE_ENDIAN_CONSTANT};

        match self {
            IncorrectMagic { dex_magic } => {
                write!(f, "incorrect dex magic number: {:?}", dex_magic)
            }
            FileSizeMismatch {
                file_size,
                size_in_header,
            } => write!(
                f,
                "file size in the header ({} bytes) is not the same as the actual file size ({} \
                 bytes)",
                size_in_header, file_size
            ),
            InvalidEndianTag { endian_tag } => write!(
                f,
                "invalid dex endian tag: {:#010x}, it can only be `ENDIAN_CONSTANT` ({:#010x}) or \
                 `REVERSE_ENDIAN_CONSTANT` ({:#010x})",
                endian_tag, ENDIAN_CONSTANT, REVERSE_ENDIAN_CONSTANT
            ),
            IncorrectHeaderSize { header_size } => write!(
                f,
                "invalid dex header_size: {} bytes, it can only be {} bytes",
                header_size, HEADER_SIZE
            ),
            Generic { error } => write!(f, "error in dex header: {}", error),
        }
    }
}

/// Parsing errors.
#[derive(Debug, Fail)]
pub enum Parse {
    /// Invalid offset found.
    #[fail(display = "invalid offset: {}", desc)]
    InvalidOffset {
        /// Description of the offset.
        desc: String,
    },

    /// Mismatched offsets found.
    #[fail(
        display = "mismatched `{}` offsets: expected {:#010x}, current offset {:#010x}",
        offset_name, expected_offset, current_offset
    )]
    OffsetMismatch {
        /// Name of the mismatched offset.
        offset_name: &'static str,
        /// Current offset.
        current_offset: u32,
        /// Expected offset.
        expected_offset: u32,
    },

    /// Unknown string index.
    #[fail(display = "there is no string with index {}", index)]
    UnknownStringIndex {
        /// Unknown index.
        index: u32,
    },

    /// Unknown type index.
    #[fail(display = "there is no type with index {}", index)]
    UnknownTypeIndex {
        /// Unknown index.
        index: u32,
    },

    /// Invalid type decriptor.
    #[fail(display = "invalid type descriptor: `{}`", descriptor)]
    InvalidTypeDescriptor {
        /// The invalid descriptor found in the file.
        descriptor: String,
    },

    /// Invalid shorty type.
    #[fail(display = "invalid shorty type: `{}`", shorty_type)]
    InvalidShortyType {
        /// The invalid shorty type found in the file.
        shorty_type: char,
    },

    /// Invalid shorty decriptor.
    #[fail(display = "invalid shorty descriptor: `{}`", descriptor)]
    InvalidShortyDescriptor {
        /// The invalid descriptor found in the file.
        descriptor: String,
    },

    /// Invalid access flags found.
    #[fail(display = "invalid access flags: {:#010x}", access_flags)]
    InvalidAccessFlags {
        /// The invalid access flags integer.
        access_flags: u32,
    },

    /// Invalid item type found.
    #[fail(display = "invalid item type: {:#06x}", item_type)]
    InvalidItemType {
        /// The invalid item type integer.
        item_type: u16,
    },

    /// Invalid visibility modifier.
    #[fail(display = "invalid visibility modifier: {:#04x}", visibility)]
    InvalidVisibility {
        /// The invalid visibility integer.
        visibility: u8,
    },

    /// Invalid value found.
    #[fail(display = "invalid value: {}", error)]
    InvalidValue {
        /// Error string.
        error: String,
    },

    /// String size mismatch.
    #[fail(
        display = "string size mismatch: expected {} characters, found {}",
        expected_size, actual_size
    )]
    StringSizeMismatch {
        /// Expected string size.
        expected_size: u32,
        /// Actual string size.
        actual_size: usize,
    },

    /// Invalid LEB128 number.
    #[fail(display = "invalid leb128: an leb128 with more than 5 bytes was found")]
    InvalidLeb128,

    /// Generic error in dex map.
    #[fail(display = "error in dex map: {}", error)]
    Map {
        /// Error String.
        error: String,
    },
}
