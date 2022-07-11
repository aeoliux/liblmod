mod module_libc;
pub mod loader;

use std::{io::{self, BufReader, BufRead, Read}, path::Path, fs, ffi::CStr, os::raw::*};

/// Loads module by path
/// 
/// Example
/// ```rust
/// extern crate liblmod;
/// 
/// if let Err(e) = load("./example_module.ko", "example.param=0".to_string()) {
///     eprintln!("Failed to load module: {e}");
/// }
/// ```
pub fn load(path_str: &str, params: String) -> io::Result<()> {
    let path = Path::new(path_str);

    let mut file = fs::File::open(path)?;
    let mut image = Vec::new();
    file.read_to_end(&mut image)?;

    loader::load(&image, params)
}

mod kernel;

/// Enum for modprobe function
pub enum Selection {
    /// Use current kernel
    Current, 

    /// Select kernel manually
    Other(String)
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
    let kernelname = match kernel {
        Selection::Other(a) => a,
        Selection::Current => {
            let mut uname = kernel::Utsname::new();
            unsafe {
                if kernel::uname(&mut uname) != 0 {
                    return Err(io::Error::last_os_error());
                }

                CStr::from_ptr(uname.release.as_ptr()).to_str().unwrap().to_string()
            }
        }
    };
    
    let modulespath = format!("/lib/modules/{}/modules.order", &kernelname);
    let mut path = String::from("");
    {
        let fd = fs::File::open(Path::new(&modulespath.as_str()))?;
        let br = BufReader::new(fd);
        for line in br.lines() {
            let unwrapped = match line {
                Ok(o) => o,
                Err(e) => return Err(e),
            };
            if unwrapped.contains(format!("/{}.ko", &name).as_str()) {
                path = format!("/lib/modules/{}/{}", &kernelname, unwrapped.clone());
            }
        }

        if path.eq(&String::from("")) {
            return Err(
                io::Error::new(io::ErrorKind::Other, format!("Module is not provided by {kernelname} kernel"))
            )
        }
    }

    load(path.as_str(), params)
}

/// Flags for rmmod
pub enum Flags {
    /// Module unloading without any flags
    None,

    /// Force module unloading
    Force, 

    /// Module unloading with O_NONBLOCK flag
    Usuall
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
    match flags {
        Flags::None => (),
        Flags::Force => flags_raw = u32::from_str_radix("4000", 8).unwrap() | u32::from_str_radix("1000", 8).unwrap(),
        Flags::Usuall => flags_raw = u32::from_str_radix("4000", 8).unwrap(),
    }

    if module_libc::delete_module(name, flags_raw) == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}