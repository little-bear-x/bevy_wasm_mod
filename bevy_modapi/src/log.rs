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
