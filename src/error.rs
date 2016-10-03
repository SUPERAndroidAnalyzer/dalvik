use std::error::Error as StdError;
use std::result::Result as StdResult;
use std::{fmt, io, usize};
use super::HEADER_SIZE;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    BytecodeParse(String),
    InvalidDexMagic(String),
    InvalidDexHeaderSize(String),
    InvalidDexFileSize(String),
    IO(io::Error),
}

impl Error {
    pub fn bytecode_parse(bytecode: [u8; 4]) -> Error {
        Error::BytecodeParse(format!("invalid bytecode: {:?}", bytecode))
    }
    pub fn invalid_dex_magic(dex_magic: [u8; 8]) -> Error {
        Error::InvalidDexMagic(format!("invalid dex magic number: {:?}", dex_magic))
    }
    pub fn invalid_dex_file_size(file_size: u64, header_size: Option<usize>) -> Error {
        match header_size {
            Some(size) => {
                if size < HEADER_SIZE {
                    Error::InvalidDexFileSize(format!("the file size in the header file is not \
                                                       valid: {}, the size must be bigger than \
                                                       {} bytes",
                                                      size,
                                                      HEADER_SIZE))
                } else {
                    Error::InvalidDexFileSize(format!("the file size in the dex file and the \
                                                       actual dex file size do not match - file \
                                                       size in header: {}, real file size: {}",
                                                      size,
                                                      file_size))
                }
            }
            None => {
                Error::InvalidDexFileSize(format!("invalid dex file size: the size must be \
                                                   between {} and {} bytes, but the size is {}",
                                                  HEADER_SIZE,
                                                  usize::MAX,
                                                  file_size))
            }
        }
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
            &Error::InvalidDexMagic(ref d) |
            &Error::InvalidDexHeaderSize(ref d) |
            &Error::InvalidDexFileSize(ref d) => d,
            &Error::IO(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::BytecodeParse(_) |
            &Error::InvalidDexMagic(_) |
            &Error::InvalidDexHeaderSize(_) |
            &Error::InvalidDexFileSize(_) => None,
            &Error::IO(ref e) => Some(e),
        }
    }
}
