//! # liblmod - Library for loading Linux kernel modules
//!
//! ### Features:
//! - Loading modules (modprobe)
//! - Unloading modules (rmmod)
//!
//! ### Example code:
//! ```rust
//! extern crate liblmod;
//!
//! fn main() -> std::io::Result<()> {
//!     println!("Unloading module kvm");
//!     liblmod::rmmod("kvm".to_string(), liblmod::Flags::Force)?;
//!
//!     println!("Loading module kvm");
//!     liblmod::modprobe("kvm".to_string(), "".to_string(), liblmod::Selection::Current)
//! }
//! ```

pub mod loader;
mod module_libc;

use std::io::ErrorKind;
use std::{
	ffi::CStr,
	fs,
	io::{self, BufRead, BufReader, Read},
	os::raw::*,
	path::Path,
};

/// Loads module by path
///
/// Example
/// ```rust
/// extern crate liblmod;
///
/// if let Err(e) = liblmod::load("./example_module.ko", "example.param=0".to_string()) {
///     eprintln!("Failed to load module: {e}");
/// }
/// ```
pub fn load(path_str: &str, params: String) -> io::Result<()> {
	let path = Path::new(path_str);

	// Read data from file
	let mut file = fs::File::open(path)?;
	let mut image = Vec::new();
	file.read_to_end(&mut image)?;

	// Call a loader
	loader::load(&image, params)
}

mod kernel;

/// Enum for modprobe function
pub enum Selection {
	/// Use current kernel
	Current,

	/// Select kernel manually
	Other(String),
}

/// Loads module for selected or current running kernel
///
/// Example:
/// ```rust
/// extern crate liblmod;
///
/// println!("Loading for current running");
/// if let Err(e) = liblmod::modprobe("kvm".to_string(), "".to_string(), liblmod::Selection::Current) {
///     eprintln!("Failed to load module kvm for current running kernel: {e}");
/// }
///
/// println!("Loading for 5.4-x86_64");
/// if let Err(e) = liblmod::modprobe("kvm".to_string(), "".to_string(), liblmod::Selection::Other("5.4-x86_64".to_string())) {
///     eprintln!("Failed to load module kvm for kernel 5.4-x86_64");
/// }
/// ```
pub fn modprobe(name: String, params: String, kernel: Selection) -> io::Result<()> {
	// Get kernel version
	let kernelname = match kernel {
		Selection::Other(a) => a,
		Selection::Current => {
			let mut uname = kernel::Utsname::new();
			unsafe {
				if kernel::uname(&mut uname) != 0 {
					return Err(io::Error::last_os_error());
				}

				CStr::from_ptr(uname.release.as_ptr())
					.to_str()
					.unwrap()
					.to_string()
			}
		}
	};

	// Construct modules manifests paths
	let basepath = format!("/lib/modules/{}", &kernelname);
	let modulespath = format!("{}/modules.order", &basepath);
	let depspath = format!("{}/modules.dep", &basepath);

	let mut module: String = String::new();
	let mut path: String = String::new();

	// Get path for specified module from modules.order
	{
		let fd = fs::File::open(&modulespath)?;
		let br = BufReader::new(fd);
		for line in br.lines() {
			let unwrapped = match line {
				Ok(o) => o,
				Err(e) => return Err(e),
			};
			if unwrapped.contains(format!("/{}.ko", &name).as_str()) {
				module = unwrapped.clone();
				path = format!("{}/{}", &basepath, unwrapped.clone());
			}
		}

		if path.eq("") {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				format!("Module is not provided by {kernelname} kernel"),
			));
		}
	}

	// Load dependencies for module
	if !module.eq("") {
		let fd = fs::File::open(&depspath)?;
		let br = BufReader::new(fd);
		for line in br.lines() {
			let unwrapped = match line {
				Ok(o) => o,
				Err(e) => return Err(e),
			};
			if unwrapped.starts_with(&module) {
				let split: Vec<&str> = unwrapped.split(" ").collect();
				let length = split.len();
				if length > 1 {
					for dep in &split[1..] {
						let modpath = format!("{}/{}", &basepath, dep);

						match load(modpath.as_str(), String::new()) {
							Err(e) => {
								if e.kind() != ErrorKind::AlreadyExists {
									return Err(e);
								}
							}
							Ok(_) => (),
						}
					}
				}
			}
		}
	}

	// Load final module
	load(path.as_str(), params)
}

/// Flags for rmmod
pub enum Flags {
	/// Module unloading without any flags
	None,

	/// Force module unloading
	Force,

	/// Module unloading with O_NONBLOCK flag
	Casual,
}

/// Removes kernel module from current running kernel
///
/// Example:
/// ```rust
/// extern crate liblmod;
///
/// if let Err(e) = liblmod::rmmod("kvm".to_string(), liblmod::Flags::None) {
///     eprintln!("Failed to unload kernel module kvm: {e}");
/// }
/// ```
pub fn rmmod(name: String, flags: Flags) -> io::Result<()> {
	let mut flags_raw: c_uint = 0;

	// Construct flags for module unloading (Linux 6.0 API: https://github.com/torvalds/linux/blob/v6.0/include/uapi/asm-generic/fcntl.h)
	match flags {
		Flags::None => (),
		Flags::Force => {
			flags_raw =
				u32::from_str_radix("4000", 8).unwrap() | u32::from_str_radix("1000", 8).unwrap()
		}
		Flags::Casual => flags_raw = u32::from_str_radix("4000", 8).unwrap(),
	}

	// Call kernel to unload module
	if module_libc::delete_module(name, flags_raw) == -1 {
		return Err(io::Error::last_os_error());
	}

	Ok(())
}
