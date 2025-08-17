use anyhow::anyhow;
use bevy::log::{debug, error, info, warn};
use modtypes::SystemInfo;
use std::mem;
use wasmtime::{Instance, Result, Store, TypedFunc};

/// Get system names in a mod
pub(crate) fn get_systems<T>(mut store: &mut Store<T>, instance: &Instance) -> Result<Vec<String>> {
    let get_count: TypedFunc<(), u32> =
        instance.get_typed_func(&mut store, "__mod_get_systems_count")?;
    let get_ptr: TypedFunc<(), i32> =
        instance.get_typed_func(&mut store, "__mod_get_systems_names_ptr")?;

    let count = get_count.call(&mut store, ())? as usize;
    let ptr = get_ptr.call(&mut store, ())? as usize;

    if count == 0 {
        return Ok(vec![]);
    }

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow!("missing memory export"))?;

    let size_of_entry = 8;
    let mut buffer = vec![0u8; count * size_of_entry];
    memory.read(&mut store, ptr, &mut buffer)?;

    let mut result = Vec::with_capacity(count);

    for i in 0..count {
        let base = i * size_of_entry;

        let name_ptr_bytes: [u8; 4] = buffer[base..base + 4]
            .try_into()
            .map_err(|_| anyhow!("Failed to convert name_ptr slice to array for entry {}", i))?;
        let name_len_bytes: [u8; 4] = buffer[base + 4..base + 8]
            .try_into()
            .map_err(|_| anyhow!("Failed to convert name_len slice to array for entry {}", i))?;

        let name_ptr = u32::from_le_bytes(name_ptr_bytes) as u64;
        let name_len = u32::from_le_bytes(name_len_bytes) as u64;

        // 辅助函数：从 WASM 内存读字符串
        let mut read_str = |ptr: u64, len: u64| -> Result<String> {
            if len == 0 {
                return Ok("".to_string());
            }
            let mut bytes = vec![0u8; len as usize];
            memory.read(&mut store, ptr as usize, &mut bytes)?;
            Ok(String::from_utf8(bytes)?)
        };

        result.push(read_str(name_ptr, name_len)?);
    }

    Ok(result)
}

/// Convert SystemInfo export_name to a String
pub(crate) fn system_info_export_name_str(system_info: &SystemInfo) -> String {
    // Find the null terminator or use the full length
    let len = system_info
        .export_name
        .iter()
        .position(|&x| x == 0)
        .unwrap_or(64);
    String::from_utf8_lossy(&system_info.export_name[..len]).to_string()
}

/// Get info of a system
pub(crate) fn get_mod_system_info<T>(
    mut store: &mut Store<T>,
    instance: &Instance,
    system_name: &str,
) -> Result<SystemInfo> {
    // Construct the info function name based on the system name
    let info_fn_name = format!("__mod_info_system_{}", system_name);

    // Get the typed function from the instance that returns a pointer to SystemInfo
    let info_func: TypedFunc<(), i32> = instance.get_typed_func(&mut store, &info_fn_name)?;

    // Call the function and get the pointer to SystemInfo
    let info_ptr = info_func.call(&mut store, ())? as usize;

    // Get the memory export
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow!("missing memory export"))?;

    // Read the SystemInfo struct from memory
    let mut buffer = vec![0u8; mem::size_of::<SystemInfo>()];
    memory.read(&mut store, info_ptr, &mut buffer)?;

    // Convert the buffer to a SystemInfo struct
    // This is safe because we know the buffer is the correct size and the data is valid
    let system_info: SystemInfo = unsafe { std::ptr::read(buffer.as_ptr() as *const SystemInfo) };

    Ok(system_info)
}

/// Get the name of a mod
pub(crate) fn get_mod_name<T>(mut store: &mut Store<T>, instance: &Instance) -> Result<String> {
    let get_name_func: TypedFunc<(), i32> =
        instance.get_typed_func(&mut store, "__mod_get_name")?;

    let name_ptr = get_name_func.call(&mut store, ())? as usize;

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow!("missing memory export"))?;

    // Read a null-terminated string from memory
    let mut buffer = Vec::new();
    let mut offset = 0;
    let max_len = 256; // Maximum length to prevent infinite loops

    loop {
        let mut byte = [0u8; 1];
        memory.read(&store, name_ptr + offset, &mut byte)?;

        if byte[0] == 0 || offset >= max_len {
            break; // Found null terminator or reached max length
        }
        buffer.push(byte[0]);
        offset += 1;
    }

    Ok(String::from_utf8(buffer)?)
}

/// Handle log
pub fn host_handle_log(mut caller: wasmtime::Caller<'_, ()>, ptr: i32, len: i32, level: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mem_data = memory.data(&mut caller);
    let bytes = &mem_data[ptr as usize..(ptr + len) as usize];
    let msg = String::from_utf8_lossy(bytes);

    match level {
        0 => {
            debug!("{}", msg);
        }
        1 => {
            info!("{}", msg);
        }
        2 => {
            warn!("{}", msg);
        }
        3 => {
            error!("{}", msg);
        }
        _ => {
            info!("{}", msg);
        }
    }
}
