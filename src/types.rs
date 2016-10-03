#[allow(unused)]
pub enum LEB128 {
    B1(u8),
    B2(u8, u8),
    B3(u8, u8, u8),
    B4(u8, u8, u8, u8),
    B5(u8, u8, u8, u8, u8),
}

#[allow(dead_code)]
impl LEB128 {
    pub fn parse(bytes: &[u8]) -> LEB128 {
        let leb128 = match bytes.len() {
            1 => LEB128::B1(*unsafe { bytes.get_unchecked(0) }),
            2 => {
                LEB128::B2(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) })
            }
            3 => {
                LEB128::B3(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) },
                           *unsafe { bytes.get_unchecked(2) })
            }
            4 => {
                LEB128::B4(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) },
                           *unsafe { bytes.get_unchecked(2) },
                           *unsafe { bytes.get_unchecked(3) })
            }
            5 => {
                LEB128::B5(*unsafe { bytes.get_unchecked(0) },
                           *unsafe { bytes.get_unchecked(1) },
                           *unsafe { bytes.get_unchecked(2) },
                           *unsafe { bytes.get_unchecked(3) },
                           *unsafe { bytes.get_unchecked(4) })
            }
            l => {
                panic!("LEB128 slice length is {}, it must be between 1 and 5 bytes",
                       l)
            }
        };
        unimplemented!()
    }
}
