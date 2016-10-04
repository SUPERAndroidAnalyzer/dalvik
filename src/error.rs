use std::error::Error as StdError;
use std::result::Result as StdResult;
use std::{fmt, io, usize};
use super::HEADER_SIZE;

/// Dalvik parser result type.
pub type Result<T> = StdResult<T, Error>;

/// Dalvik parser errors.
#[derive(Debug)]
pub enum Error {
    /// Invalid bytecode.
    BytecodeParse(String),
    /// Invalid magic number.
    InvalidMagic(String),
    /// Invalid file size.
    InvalidFileSize(String),
    /// Invalid file size.
    InvalidEndianTag(String),
    /// Invalid header size.
    InvalidHeaderSize(String),
    /// Mismatched offsets.
    MismatchedOffsets(String),
    /// Invalid access flags.
    InvalidAccessFlags(String),
    /// Generic header error.
    Header(String),
    /// IO error.
    IO(io::Error),
}

#[doc(hidden)]
impl Error {
    /// Creates a new bytecode parser error.
    pub fn bytecode_parse(bytecode: [u8; 4]) -> Error {
        Error::BytecodeParse(format!("invalid bytecode: {:?}", bytecode))
    }
    /// Creates a new invalid magic number error.
    pub fn invalid_magic(dex_magic: [u8; 8]) -> Error {
        Error::InvalidMagic(format!("invalid dex magic number: {:?}", dex_magic))
    }
    /// Creates a new invalid file size error.
    pub fn invalid_file_size(file_size: u64, header_size: Option<usize>) -> Error {
        match header_size {
            Some(size) => {
                if size < HEADER_SIZE {
                    Error::InvalidFileSize(format!("the file size in the header file is not \
                                                       valid: {}, the size must be bigger or \
                                                       equal to {} bytes",
                                                   size,
                                                   HEADER_SIZE))
                } else {
                    Error::InvalidFileSize(format!("the file size in the dex file and the \
                                                       actual dex file size do not match - file \
                                                       size in header: {}, real file size: {}",
                                                   size,
                                                   file_size))
                }
            }
            None => {
                Error::InvalidFileSize(format!("invalid dex file size: the size must be \
                                                   between {} and {} bytes, but the size is {}",
                                               HEADER_SIZE,
                                               usize::MAX,
                                               file_size))
            }
        }
    }
    /// Creates a new invalid endian tag error.
    pub fn invalid_endian_tag(endian_tag: u32) -> Error {
        Error::InvalidEndianTag(format!("invalid dex endian tag: {:#010x}, it can only be \
                                            `ENDIAN_CONSTANT` or `REVERSE_ENDIAN_CONSTANT`",
                                        endian_tag))
    }
    /// Creates a new invalid header size error.
    pub fn invalid_header_size(header_size: usize) -> Error {
        Error::InvalidHeaderSize(format!("invalid dex header_size: {}, it can only be {}",
                                         header_size,
                                         HEADER_SIZE))
    }
    /// Creates a new mismatched offset error.
    pub fn mismatched_offsets<S: AsRef<str>>(offset_name: S,
                                             current_offset: usize,
                                             expected_offset: usize)
                                             -> Error {
        Error::MismatchedOffsets(format!("invalid `{}` offset: expected {:#010x}, got {:#010x}",
                                         offset_name.as_ref(),
                                         expected_offset,
                                         current_offset))
    }
    /// Creates a new invalid access flags error.
    pub fn invalid_access_flags(access_flags: u32) -> Error {
        Error::InvalidAccessFlags(format!("invalid access flags: {:#010x}", access_flags))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::BytecodeParse(ref d) |
            &Error::InvalidMagic(ref d) |
            &Error::InvalidFileSize(ref d) |
            &Error::InvalidEndianTag(ref d) |
            &Error::InvalidHeaderSize(ref d) |
            &Error::MismatchedOffsets(ref d) |
            &Error::InvalidAccessFlags(ref d) |
            &Error::Header(ref d) => d,
            &Error::IO(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::BytecodeParse(_) |
            &Error::InvalidMagic(_) |
            &Error::InvalidFileSize(_) |
            &Error::InvalidEndianTag(_) |
            &Error::InvalidHeaderSize(_) |
            &Error::MismatchedOffsets(_) |
            &Error::InvalidAccessFlags(_) |
            &Error::Header(_) => None,
            &Error::IO(ref e) => Some(e),
        }
    }
}
