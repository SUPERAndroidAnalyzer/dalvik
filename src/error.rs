//! Errors module

use crate::sizes::HEADER_SIZE;
use std::{error::Error, fmt};

/// Invalid file size.
#[derive(Debug, Copy, Clone)]
pub struct InvalidFileSize {
    /// Size of the dex file.
    pub file_size: u64,
}

impl fmt::Display for InvalidFileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl Error for InvalidFileSize {}

/// Errors coming from header parsing.
#[derive(Debug, Clone)]
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::header::{ENDIAN_CONSTANT, REVERSE_ENDIAN_CONSTANT};

        match self {
            Self::IncorrectMagic { dex_magic } => {
                write!(f, "incorrect dex magic number: {:?}", dex_magic)
            }
            Self::FileSizeMismatch {
                file_size,
                size_in_header,
            } => write!(
                f,
                "file size in the header ({} bytes) is not the same as the actual file size ({} \
                 bytes)",
                size_in_header, file_size
            ),
            Self::InvalidEndianTag { endian_tag } => write!(
                f,
                "invalid dex endian tag: {:#010x}, it can only be `ENDIAN_CONSTANT` ({:#010x}) or \
                 `REVERSE_ENDIAN_CONSTANT` ({:#010x})",
                endian_tag, ENDIAN_CONSTANT, REVERSE_ENDIAN_CONSTANT
            ),
            Self::IncorrectHeaderSize { header_size } => write!(
                f,
                "invalid dex header_size: {} bytes, it can only be {} bytes",
                header_size, HEADER_SIZE
            ),
            Self::Generic { error } => write!(f, "error in dex header: {}", error),
        }
    }
}

impl Error for Header {}

/// Parsing errors.
#[derive(Debug)]
pub enum Parse {
    /// Invalid offset found.
    InvalidOffset {
        /// Description of the offset.
        desc: String,
    },

    /// Mismatched offsets found.
    OffsetMismatch {
        /// Name of the mismatched offset.
        offset_name: &'static str,
        /// Current offset.
        current_offset: u32,
        /// Expected offset.
        expected_offset: u32,
    },

    /// Unknown string index.
    UnknownStringIndex(u32),

    /// Unknown type index.
    UnknownTypeIndex(u32),

    /// Invalid type descriptor.
    InvalidTypeDescriptor(String),

    /// Invalid shorty type.
    InvalidShortyType(char),

    /// Invalid shorty descriptor.
    InvalidShortyDescriptor(String),

    /// Invalid access flags found.
    InvalidAccessFlags(u32),

    /// Invalid item type found.
    InvalidItemType(u16),

    /// Invalid visibility modifier.
    InvalidVisibility(u8),

    /// Invalid value found.
    InvalidValue {
        /// Error string.
        error: String,
    },

    /// String size mismatch.
    StringSizeMismatch {
        /// Expected string size.
        expected_size: u32,
        /// Actual string size.
        actual_size: usize,
    },

    /// Invalid LEB128 number.
    InvalidLeb128,

    /// Generic error in dex map.
    Map {
        /// Error String.
        error: String,
    },
}

impl fmt::Display for Parse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOffset { desc } => write!(f, "invalid offset: {}", desc),
            Self::OffsetMismatch {
                offset_name,
                current_offset,
                expected_offset,
            } => write!(
                f,
                "mismatched `{}` offsets: expected {:#010x}, current offset {:#010x}",
                offset_name, expected_offset, current_offset
            ),
            Self::UnknownStringIndex(index) => write!(f, "there is no string with index {}", index),
            Self::UnknownTypeIndex(index) => write!(f, "there is no type with index {}", index),
            Self::InvalidTypeDescriptor(descriptor) => {
                write!(f, "invalid type descriptor: `{}`", descriptor)
            }
            Self::InvalidShortyType(shorty_type) => {
                write!(f, "invalid shorty type: `{}`", shorty_type)
            }
            Self::InvalidShortyDescriptor(descriptor) => {
                write!(f, "invalid shorty descriptor: `{}`", descriptor)
            }
            Self::InvalidAccessFlags(access_flags) => {
                write!(f, "invalid access flags: {:#010x}", access_flags)
            }
            Self::InvalidItemType(item_type) => write!(f, "invalid item type: {:#06x}", item_type),
            Self::InvalidVisibility(visibility) => {
                write!(f, "invalid visibility modifier: {:#04x}", visibility)
            }
            Self::InvalidValue { error } => write!(f, "invalid value: {}", error),
            Self::StringSizeMismatch {
                expected_size,
                actual_size,
            } => write!(
                f,
                "string size mismatch: expected {} characters, found {}",
                expected_size, actual_size
            ),
            Self::InvalidLeb128 => write!(
                f,
                "invalid leb128: a leb128 with more than 5 bytes was found"
            ),
            Self::Map { error } => write!(f, "error in dex map: {}", error),
        }
    }
}

impl Error for Parse {}
