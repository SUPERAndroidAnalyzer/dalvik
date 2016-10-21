extern crate dalvik;

#[test]
fn it_header_read() {
    dalvik::Header::from_file("test.dex").unwrap();
}

#[test]
#[should_panic]
fn it_header_verify() {
    let header = dalvik::Header::from_file("test.dex").unwrap();
    assert!(header.verify_file("text.dex"));
}

#[test]
// #[should_panic]
fn it_file_read() {
    dalvik::Dex::from_file("test.dex").unwrap();
}

#[test]
#[should_panic]
fn it_file_verify() {
    let file = dalvik::Dex::from_file("test.dex").unwrap();
    assert!(file.verify_file("test.dex"));
}
