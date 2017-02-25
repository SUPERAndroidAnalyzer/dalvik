use std::u32;
use sizes::HEADER_SIZE;
use header::{ENDIAN_CONSTANT, REVERSE_ENDIAN_CONSTANT};

error_chain!{
    foreign_links {
        Io(::std::io::Error);
        FromUTF8(::std::string::FromUtf8Error);
    }

    errors {
        /// Incorrect dex magic number.
        IncorrectMagic(dex_magic: [u8; 8]) {
            description("incorrect dex magic number")
            display("incorrect dex magic number: {:?}", dex_magic)
        }

        /// Mismatch between file size in header and real file size.
        HeaderFileSizeMismatch(file_size: u64, size_in_header: u32) {
            description("invalid dex file size in header")
            display("invalid dex file size")
        }

        /// Invalid file size.
        InvalidFileSize(file_size: u64) {
            description("invalid dex file size")
            display("invalid dex file size: file size must be between {} and {} bytes, \
                     but the size of the file was {} bytes", HEADER_SIZE, u32::MAX, file_size)
        }

        /// Invalid endian tag.
        InvalidEndianTag(endian_tag: u32) {
            description("invalid dex endian tag")
            display("invalid dex endian tag: {:#010x}, it can only be `ENDIAN_CONSTANT` ({:#010x}) \
                     or `REVERSE_ENDIAN_CONSTANT` ({:#010x})", endian_tag, ENDIAN_CONSTANT,
                    REVERSE_ENDIAN_CONSTANT)
        }

        /// Incorrect header size.
        IncorrectHeaderSize(header_size: u32) {
            description("incorrect header size")
            display("invalid dex header_size: {} bytes, it can only be {} bytes", header_size,
                    HEADER_SIZE)
        }

        /// Invalid offset.
        InvalidOffset(desc: String) {
            description("invalid offset")
            display("invalid offset: {}", desc)
        }

        /// Mismatched offsets.
        MismatchedOffsets(offset_name: &'static str, current_offset: u32, expected_offset: u32) {
            description("mismatched offsets")
            display("mismatched `{}` offsets: expected {:#010x}, current offset {:#010x}",
                    offset_name, expected_offset, current_offset)
        }

        /// Invalid access flags.
        InvalidAccessFlags(access_flags: u32) {
            description("invalid access flags")
            display("invalid access flags: {:#010x}", access_flags)
        }

        /// Invalid item type.
        InvalidItemType(item_type: u16) {
            description("invalid item type")
            display("invalid item type: {:#06x}", item_type)
        }

        /// Invalid visibility modifier.
        InvalidVisibility(visibility: u8) {
            description("invalid visibility modifier")
            display("invalid visibility modifier: {:#04x}", visibility)
        }

        /// Invalid value.
        InvalidValue(error: String) {
            description("invalid value")
            display("invalid value: {}", error)
        }

        /// String size mismatch.
        StringSizeMismatch(expected_size: u32, actual_size: usize) {
            description("string size mismatch")
            display("string size mismatch: expected {} characters, found {}", expected_size,
                    actual_size)
        }

        /// Invalid uleb128.
        InvalidLeb128 {
            description("invalid uleb128")
            display("invalid uleb128: an uleb128 with more than 5 bytes was found")
        }

        /// Generic header error.
        Header(error: String) {
            description("error in dex header")
            display("error in dex header: {}", error)
        }

        /// Generic map error.
        Map(error: String) {
            description("error in dex map")
            display("error in dex map: {}", error)
        }
    }
}
