//! Asset API for mods.
//!
//! This module provides the `asset_def!` macro for defining assets in mods.
//! The macro embeds the asset data into the WASM binary and communicates with the host
//! to register the asset. The host returns an asset ID.

/// Asset definition macro.
///
/// This macro embeds the asset data into the WASM binary and communicates with the host
/// to register the asset. The host returns an asset ID.
///
/// # Example
/// ```rust
/// let asset_id = asset_def!(type=image, src="path/to/image.png");
/// ```
#[macro_export]
macro_rules! asset_def {
    (type=$type:ident, src=$src:literal) => {{
        // Read the asset file at compile time
        let data = include_bytes!($src);
        let asset_type = stringify!($type);

        // Get the mod name
        let mod_name_ptr = unsafe { $crate::__mod_get_name() };
        // For now, we'll assume the mod name is null-terminated
        // In a more robust implementation, we would get the length from the host
        let mod_name =
            unsafe { core::ffi::CStr::from_ptr(mod_name_ptr as *const core::ffi::c_char) };
        let mod_name_bytes = mod_name.to_bytes();

        // Buffer to hold the asset ID
        let mut asset_id_buffer = [0u8; 256]; // Assuming max 256 characters for asset ID

        // Call the host function to define the asset
        let asset_id_len = unsafe {
            $crate::__mod_define_asset(
                mod_name_bytes.as_ptr(),
                mod_name_bytes.len(),
                asset_type.as_ptr(),
                asset_type.len(),
                data.as_ptr(),
                data.len(),
                asset_id_buffer.as_mut_ptr(),
                asset_id_buffer.len(),
            )
        };

        // Convert the asset ID buffer to a string
        let asset_id = if asset_id_len > 0 && asset_id_len <= asset_id_buffer.len() {
            // Find the null terminator or use the full length
            let actual_len = asset_id_buffer[..asset_id_len]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(asset_id_len);
            
            // Convert to string
            String::from_utf8_lossy(&asset_id_buffer[..actual_len]).to_string()
        } else {
            // Return an empty string if there was an error
            String::new()
        };
        
        // Return the asset ID
        asset_id
    }};
}
