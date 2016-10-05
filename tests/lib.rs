extern crate dalvik;

#[test]
fn it_header_read() {
    dalvik::Header::from_file("test.dex", false).unwrap();
}

#[test]
#[should_panic]
fn it_header_verify() {
    dalvik::Header::from_file("test.dex", true).unwrap();
}

#[test]
#[should_panic]
fn it_file_read() {
    dalvik::Dex::from_file("test.dex", false).unwrap();
}

#[test]
#[should_panic]
fn it_file_verify() {
    dalvik::Dex::from_file("test.dex", true).unwrap();
}
