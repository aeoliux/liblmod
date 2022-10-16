use std::{ffi::CString, os::raw::*};

extern "C" {
	fn syscall(number: c_long, _: ...) -> c_long;
}

#[cfg(target_arch = "x86")]
const INIT_MODULE: c_long = 128;
#[cfg(target_arch = "x86_64")]
const INIT_MODULE: c_long = 175;
#[cfg(target_arch = "arm")]
const INIT_MODULE: c_long = 128;
#[cfg(target_arch = "aarch64")]
const INIT_MODULE: c_long = 105;

#[cfg(target_arch = "x86")]
const DELETE_MODULE: c_long = 129;
#[cfg(target_arch = "x86_64")]
const DELETE_MODULE: c_long = 176;
#[cfg(target_arch = "arm")]
const DELETE_MODULE: c_long = 129;
#[cfg(target_arch = "aarch64")]
const DELETE_MODULE: c_long = 106;

#[allow(temporary_cstring_as_ptr)]
pub fn init_module(image: &[u8], size: c_uint, params: String) -> c_long {
	unsafe {
		syscall(
			INIT_MODULE,
			image.as_ptr(),
			size,
			&CString::new(params).unwrap().as_ptr(),
		)
	}
}

#[allow(temporary_cstring_as_ptr)]
pub fn delete_module(name: String, flags: c_uint) -> c_long {
	unsafe { syscall(DELETE_MODULE, CString::new(name).unwrap(), flags) }
}
