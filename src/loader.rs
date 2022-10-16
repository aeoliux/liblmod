use std::io;

/// Load kernel module by byte array.
/// 
/// Example:
/// ```rust
/// extern crate liblmod;
///
/// use std::io::Read;
/// let mut file = std::fs::File::open(std::path::Path::new("./module.ko"))?;
/// let mut image = Vec::new();
/// file.read_to_end(&mut image)?;
///
/// if let Err(e) = liblmod::loader::load(&image, "module.param=0".to_string()) {
///     eprintln!("Failed to insert module by image: {e}");
/// }
/// ```
pub fn load(image: &[u8], params: String) -> io::Result<()> {
	// Count size of image
	let size = image.len() as std::os::raw::c_uint;

	// Call kernel to load module
	if crate::module_libc::init_module(image, size, params) != 0 {
		return Err(io::Error::last_os_error());
	}

	Ok(())
}