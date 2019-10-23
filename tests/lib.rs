#![allow(clippy::cyclomatic_complexity)]

extern crate dalvik;

use std::{fs, path::Path};

use dalvik::types::AccessFlags;

#[test]
fn it_header_read() {
    let header = dalvik::Header::from_file("test.dex").unwrap();
    assert_eq!(
        &[0x64, 0x65, 0x78, 0xa, 0x30, 0x33, 0x35, 0x0],
        header.get_magic()
    );
    assert_eq!(35, header.get_dex_version());
    assert_eq!(0xa057_6d4c, header.get_checksum());
    assert_eq!(
        &[
            0x91, 0x02, 0x73, 0x72, 0x0d, 0xda, 0xf0, 0x75, 0x5b, 0x48, 0xd8, 0x09, 0xfd, 0x6a,
            0x6f, 0x01, 0x8f, 0xc9, 0x29, 0x15,
        ],
        header.get_signature()
    );
    let file_size = fs::metadata("test.dex").unwrap().len();
    assert_eq!(file_size, u64::from(header.get_file_size()));
    assert_eq!(0x70, header.get_header_size());
    assert_eq!(0x1234_5678, header.get_endian_tag());
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
    assert_eq!(2_420_664, header.get_data_size());
    assert_eq!(0x79ff8, header.get_data_offset());
}

#[test]
fn it_header_verify() {
    let header = dalvik::Header::from_file("test.dex").unwrap();
    assert!(header.verify_file("text.dex"));
}

#[test]
// #[should_panic]
fn it_file_read() {
    let dex = dalvik::Dex::from_file("test.dex").unwrap();
    for (i, t) in dex.types().iter().enumerate() {
        let path = Path::new(t.name());
        let mut name = path
            .file_name()
            .expect("no class name :(")
            .to_string_lossy()
            .into_owned();
        name.pop();

        let file_str = if let Some(source) = t.source_file() {
            format!("// file: {}\n\n", path.with_file_name(source).display())
        } else {
            String::new()
        };

        let mut imports = Vec::new();
        let superclass_str = if let Some(superclass) = t.superclass() {
            let mut superclass_full_path = superclass.clone();
            superclass_full_path.pop();

            let superclass_obj_str = Path::new(&superclass_full_path)
                .file_name()
                .expect("no superclass name :(")
                .to_string_lossy()
                .into_owned();

            if superclass_obj_str == "Object" {
                String::new()
            } else {
                imports.push(superclass_full_path.replace('/', "."));
                format!(" extends {}", superclass_obj_str)
            }
        } else {
            String::new()
        };

        let mut interfaces_str = if t.interfaces().is_empty() {
            String::new()
        } else {
            String::from(" implements ")
        };
        let mut interfaces = Vec::with_capacity(t.interfaces().len());
        for interface in t.interfaces() {
            let mut interface_full_path = interface.clone();
            interface_full_path.pop();

            imports.push(interface_full_path.replace('/', "."));
            interfaces.push(
                Path::new(&interface_full_path)
                    .file_name()
                    .expect("no interface name :(")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        interfaces_str.push_str(&interfaces.join(", "));
        let mut imports_str = imports
            .into_iter()
            .map(|import| format!("import {};\n", import))
            .collect::<String>();
        imports_str.push('\n');

        eprintln!(
            "{}{}{}{} {}{}{} {{\n\t// TODO\n}}",
            file_str,
            imports_str,
            t.access_flags(),
            if t.access_flags().contains(AccessFlags::ACC_INTERFACE) {
                ""
            } else {
                " class"
            },
            name,
            superclass_str,
            interfaces_str
        );
        if i == 5 {
            break;
        }
    }
    panic!();
}

// #[test]
// fn it_file_verify() {
//     let file = dalvik::Dex::from_file("test.dex").unwrap();
//     assert!(file.verify_file("test.dex"));
// }
