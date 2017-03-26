extern crate dalvik;

use std::fs;

#[test]
fn it_header_read() {
    let header = dalvik::Header::from_file("test.dex").unwrap();
    assert_eq!(&[0x64, 0x65, 0x78, 0xa, 0x30, 0x33, 0x35, 0x0],
               header.get_magic());
    assert_eq!(35, header.get_dex_version());
    assert_eq!(0xa0576d4c, header.get_checksum());
    assert_eq!(&[0x91, 0x02, 0x73, 0x72, 0x0d, 0xda, 0xf0, 0x75, 0x5b, 0x48, 0xd8, 0x09, 0xfd,
                 0x6a, 0x6f, 0x01, 0x8f, 0xc9, 0x29, 0x15],
               header.get_signature());
    let file_size = fs::metadata("test.dex").unwrap().len();
    assert_eq!(file_size, header.get_file_size() as u64);
    assert_eq!(0x70, header.get_header_size());
    assert_eq!(0x12345678, header.get_endian_tag());
    assert_eq!(0, header.get_link_size());
    assert!(header.get_link_offset().is_none());
    assert_eq!(0x79ff8, header.get_map_offset());
    assert_eq!(19939, header.get_string_ids_size());
    assert_eq!(0x70, header.get_string_ids_offset().unwrap());
    assert_eq!(2419, header.get_type_ids_size());
    assert_eq!(0x137fc, header.get_type_ids_offset().unwrap());
    assert_eq!(3522, header.get_prototype_ids_size());
    assert_eq!(0x15dc8, header.get_prototype_ids_offset().unwrap());
    assert_eq!(9942, header.get_field_ids_size());
    assert_eq!(0x202e0, header.get_field_ids_offset().unwrap());
    assert_eq!(19282, header.get_method_ids_size());
    assert_eq!(0x33990, header.get_method_ids_offset().unwrap());
    assert_eq!(1791, header.get_class_defs_size());
    assert_eq!(0x59420, header.get_class_defs_offset().unwrap());
    assert_eq!(2420664, header.get_data_size());
    assert_eq!(0x79ff8, header.get_data_offset());
}

// #[test]
// fn it_header_verify() {
//     let header = dalvik::Header::from_file("test.dex").unwrap();
//     assert!(header.verify_file("text.dex"));
// }

#[test]
#[should_panic]
fn it_file_read() {
    dalvik::Dex::from_file("test.dex").unwrap();
}

// #[test]
// fn it_file_verify() {
//     let file = dalvik::Dex::from_file("test.dex").unwrap();
//     assert!(file.verify_file("test.dex"));
// }
