//! Bevy Mod API
//!
//! This crate provides the API for creating Bevy mods.
//! Mod developers will use this crate to create their mods.

// Re-export the macros
pub use modapi_macros::{mod_def, system};
pub use modtypes::{QueryResult, SystemInfo};

// Host function declarations
unsafe extern "C" {
    /// Mod load function
    ///
    /// level: 0-debug 1-info 2-warn 3-error else-info
    pub fn __mod_log(ptr: *const u8, len: usize, level: u8);

    /// Query components from the host
    /// Returns a pointer to serialized component data and the length
    pub fn __mod_query_components(
        component_ids_ptr: *const u8,
        component_ids_len: usize,
        result_ptr: *mut u8,
    ) -> usize;

    /// Free memory allocated by the host
    pub fn __mod_free_memory(ptr: *mut u8, len: usize);
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
            let component_ids: Vec<&str> = vec![$(<$component>::component_id()),+];

            // Serialize component IDs
            let serialized_ids = bincode::serde::encode_to_vec(&component_ids, bincode::config::standard())
                .expect("Failed to serialize component IDs");

            // Call host function to query components
            let mut result = QueryResult {
                data_ptr: 0,
                data_len: 0,
            };

            let result_ptr = &mut result as *mut QueryResult as *mut u8;

            let data_len = unsafe {
                __mod_query_components(
                    serialized_ids.as_ptr(),
                    serialized_ids.len(),
                    result_ptr
                )
            };

            // Create an empty vector to hold components
            let mut components: Vec<($($component),+)> = Vec::new();

            // If we got data, deserialize it
            if data_len > 0 && result.data_ptr != 0 {
                // Deserialize the data
                let data_slice = unsafe {
                    std::slice::from_raw_parts(result.data_ptr as *const u8, result.data_len as usize)
                };

                // First, deserialize into Vec<Vec<Vec<u8>>> - entities with their component data
                match bincode::serde::decode_from_slice::<Vec<Vec<Vec<u8>>>, _>(data_slice, bincode::config::standard()) {
                    Ok((serialized_entities, _)) => {
                        // For each entity, deserialize its components
                        for serialized_components in serialized_entities {
                            // Check we have the right number of components
                            if serialized_components.len() != {let mut count = 0; $(let _ = stringify!($component); count += 1;)+ count} {
                                log_error!("Mismatch in component count for entity");
                                continue;
                            }

                            // Deserialize each component
                            let mut index = 0;
                            let mut success = true;
                            let mut deserialized: Vec<std::result::Result<Box<dyn std::any::Any>, ()>> = Vec::new();
                            $(
                                let component_data = &serialized_components[index];
                                index += 1;
                                match bincode::serde::decode_from_slice::<$component, _>(component_data, bincode::config::standard()) {
                                    Ok((component, _)) => {
                                        deserialized.push(Ok(Box::new(component)));
                                    }
                                    Err(e) => {
                                        log_error!("Failed to deserialize component {}: {}", stringify!($component), e);
                                        deserialized.push(Err(()));
                                        success = false;
                                    }
                                }
                            )+

                            // Only create the tuple if all components were successfully deserialized
                            if success {
                                let mut i = 0;
                                let tuple = ($({
                                    let result: std::result::Result<Box<dyn std::any::Any>, ()> = deserialized.remove(0);
                                    *result.unwrap().downcast::<$component>().unwrap()
                                }),+);
                                components.push(tuple);
                            }
                        }
                    }
                    Err(e) => {
                        log_error!("Failed to deserialize components: {}", e);
                    }
                }

                // Free the memory allocated by the host
                unsafe {
                    __mod_free_memory(result.data_ptr as *mut u8, result.data_len as usize);
                }
            }

            components.into_iter()
        }
    };
}

/// Query mutable macro for querying components mutably from the host
#[macro_export]
macro_rules! query_mut {
    ($($component:ty),+) => {
        {
            // For now, this is just a placeholder that works the same as query!
            // In a full implementation, this would need to handle mutability
            query!($($component),+)
        }
    };
}

