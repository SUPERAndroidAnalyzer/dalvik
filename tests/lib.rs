extern crate dalvik;

#[test]
fn it_header_read() {
    dalvik::Header::from_file("test.dex", false).unwrap();
}

#[test]
fn it_header_verify() {
    // TODO
}
