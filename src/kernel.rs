use std::os::raw::*;

#[repr(C)]
pub struct Utsname {
    pub sysname: [c_char; 65],
    pub nodename: [c_char; 65],
    pub release: [c_char; 65],
    pub version: [c_char; 65],
    pub machine: [c_char; 65],
}

impl Utsname {
    pub fn new() -> Utsname {
        Utsname { sysname: [0; 65], nodename: [0; 65], release: [0; 65], version: [0; 65], machine: [0; 65] }
    }
}

extern "C" {
    pub fn uname(utsname: *mut Utsname) -> c_int;
}