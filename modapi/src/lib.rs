//! Bevy Mod API
//!
//! This crate provides the API for creating Bevy mods.
//! Mod developers will use this crate to create their mods.

// Re-export the macros
pub use modapi_macros::{mod_def, system};
pub use modtypes::SystemInfo;

unsafe extern "C" {
    /// Mod load function
    ///
    /// level: 0-debug 1-info 2-warn 3-error else-info
    pub fn __mod_log(ptr: *const u8, len: usize, level: u8);
}

/// Log macro debug
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            use std::ffi::CString;
            let c_str = CString::new(msg).expect("CString::new failed");
            unsafe {
                __mod_log(c_str.as_ptr() as *const u8, c_str.as_bytes().len(), 0);
            }
        }
    };
}

/// Log macro info
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            use std::ffi::CString;
            let c_str = CString::new(msg).expect("CString::new failed");
            unsafe {
                __mod_log(c_str.as_ptr() as *const u8, c_str.as_bytes().len(), 1);
            }
        }
    };
}

/// Log macro warn
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            use std::ffi::CString;
            let c_str = CString::new(msg).expect("CString::new failed");
            unsafe {
                __mod_log(c_str.as_ptr() as *const u8, c_str.as_bytes().len(), 2);
            }
        }
    };
}

/// Log macro error
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            use std::ffi::CString;
            let c_str = CString::new(msg).expect("CString::new failed");
            unsafe {
                __mod_log(c_str.as_ptr() as *const u8, c_str.as_bytes().len(), 3);
            }
        }
    };
}

/// Query macro for querying components from the host
#[macro_export]
macro_rules! query {
    ($($component:ty),+) => {
        {
            // Get component IDs
            let component_ids: &[&str] = &[$(<$component>::component_id()),+];
            // In a real implementation, this would communicate with the host to query components
            // For now, we'll return an empty iterator as a placeholder
            // TODO: Implement actual query functionality
            std::iter::empty::<($($component),+)>()
        }
    };
}
