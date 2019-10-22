use std::{
    fs::{create_dir_all, remove_dir_all, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

use dalvik::{types::AccessFlags, Dex};

const OUT_FOLDER: &str = "target/java_out";

fn main() {
    remove_dir_all(OUT_FOLDER).ok();
    create_dir_all(OUT_FOLDER).expect("could not create output folder");

    let dex = Dex::from_file("test.dex").unwrap();
    for (i, t) in dex.types().iter().enumerate() {
        let path = Path::new(t.name());
        let mut name = path
            .file_name()
            .expect("no class name :(")
            .to_string_lossy()
            .into_owned();
        name.pop();

        if let Some(source) = t.source_file() {
            let file_path = Path::new(OUT_FOLDER).join(path).with_file_name(source);
            let file_exists = file_path.exists();
            create_dir_all(file_path.parent().expect("could not get parent dir"))
                .expect("could not create folder");

            let mut writer = BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)
                    .expect("could not open file for writing"),
            );
            let file_str = if file_exists {
                String::new()
            } else {
                format!("// file: {}\n\n", path.with_file_name(source).display())
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

            writeln!(
                writer,
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
            )
            .expect("could not write to the file");
        };
    }
}
