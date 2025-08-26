/// Query macro for querying a resource from the host
#[macro_export]
macro_rules! res {
    ($resource:ty) => {
        {
            // Get resource ID
            let resource_id: &str = <$resource>::resource_id();

            // Serialize resource ID
            let serialized_id = bincode::serde::encode_to_vec(&resource_id, bincode::config::standard())
                .expect("Failed to serialize resource ID");

            // Call host function to query resource
            let mut result = HostModResult {
                data_ptr: 0,
                data_len: 0,
            };

            let result_ptr = &mut result as *mut HostModResult as *mut u8;

            let data_len = unsafe {
                __mod_query_resources(
                    serialized_id.as_ptr(),
                    serialized_id.len(),
                    result_ptr
                )
            };

            // If we got data, deserialize it
            if data_len > 0 && result.data_ptr != 0 {
                // Deserialize the data
                let data_slice = unsafe {
                    std::slice::from_raw_parts(result.data_ptr as *const u8, result.data_len as usize)
                };

                // Deserialize the resource
                match bincode::serde::decode_from_slice::<$resource, _>(data_slice, bincode::config::standard()) {
                    Ok((resource, _)) => {
                        // Free the memory allocated by the host
                        unsafe {
                            __mod_free_memory(result.data_ptr as *mut u8, result.data_len as usize);
                        }
                        
                        Some(resource)
                    }
                    Err(e) => {
                        log_error!("Failed to deserialize resource {}: {}", stringify!($resource), e);
                        // Free the memory allocated by the host
                        unsafe {
                            __mod_free_memory(result.data_ptr as *mut u8, result.data_len as usize);
                        }
                        None
                    }
                }
            } else {
                None
            }
        }
    };
}
