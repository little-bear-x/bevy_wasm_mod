use anyhow::anyhow;
use wasmtime::{Instance, Result, Store, TypedFunc};

/// Get system names in a mod
pub(crate) fn get_systems(mut store: &mut Store<()>, instance: &Instance) -> Result<Vec<String>> {
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

/// Get the name of a mod
pub(crate) fn get_mod_name(mut store: &mut Store<()>, instance: &Instance) -> Result<String> {
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
