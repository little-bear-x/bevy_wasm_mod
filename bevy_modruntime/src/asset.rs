use crate::ModState;
use anyhow::anyhow;
use bevy::log::*;
use wasmtime::{Caller, Memory, Result};

pub struct AssetInfo {
    pub mod_name: String,
    pub asset_type: String,
    pub asset_data: Vec<u8>,
}

/// Host function to handle asset definition from mods
///
/// This function receives the mod name, asset type, and asset data from the mod
/// and stores them for later use.
pub fn host_handle_define_asset(
    mut caller: Caller<'_, ModState>,
    mod_name_ptr: i32,
    mod_name_len: i32,
    asset_type_ptr: i32,
    asset_type_len: i32,
    asset_data_ptr: i32,
    asset_data_len: i32,
    asset_id_ptr: i32,
    asset_id_len: i32,
) -> Result<u32> {
    // Validate the parameters
    if mod_name_ptr <= 0 || mod_name_len <= 0 {
        return Err(anyhow!(
            "Invalid mod name parameters: ptr={}, len={}",
            mod_name_ptr,
            mod_name_len
        ));
    }

    if asset_type_ptr <= 0 || asset_type_len <= 0 {
        return Err(anyhow!(
            "Invalid asset type parameters: ptr={}, len={}",
            asset_type_ptr,
            asset_type_len
        ));
    }

    if asset_data_ptr <= 0 || asset_data_len < 0 {
        return Err(anyhow!(
            "Invalid asset data parameters: ptr={}, len={}",
            asset_data_ptr,
            asset_data_len
        ));
    }

    // Get the memory export
    let memory = match caller.get_export("memory") {
        Some(export) => match export.into_memory() {
            Some(memory) => memory,
            None => return Err(anyhow!("Failed to get memory export")),
        },
        None => return Err(anyhow!("Missing memory export")),
    };

    // Read the mod name from memory
    let mod_name = read_string_from_memory(
        &memory,
        &mut caller,
        mod_name_ptr as u32,
        mod_name_len as u32,
    )?;

    // Read the asset type from memory
    let asset_type = read_string_from_memory(
        &memory,
        &mut caller,
        asset_type_ptr as u32,
        asset_type_len as u32,
    )?;

    // Read the asset data from memory
    let asset_data = read_bytes_from_memory(
        &memory,
        &mut caller,
        asset_data_ptr as u32,
        asset_data_len as u32,
    )?;

    let asset_info = AssetInfo {
        mod_name,
        asset_type,
        asset_data,
    };

    let world = match caller.data().get_world() {
        Some(world) => world,
        None => {
            error!("get bevy world faild while insert new asset",);
            return Err(anyhow::anyhow!(
                "get bevy world faild while insert new asset"
            ));
        }
    };

    let world_origin = unsafe { world.world_mut() };

    // Get the asset ID from the new asset function
    let asset_id = caller.data().run_new_asset_fn(world_origin, asset_info)?;

    // Write the asset ID to the WASM memory
    if asset_id_ptr > 0 && asset_id_len > 0 {
        let memory = match caller.get_export("memory") {
            Some(export) => match export.into_memory() {
                Some(memory) => memory,
                None => return Err(anyhow!("Failed to get memory export")),
            },
            None => return Err(anyhow!("Missing memory export")),
        };

        // Convert the asset ID to bytes
        let asset_id_bytes = asset_id.as_bytes();
        let write_len = std::cmp::min(asset_id_bytes.len(), asset_id_len as usize);

        // Write the asset ID to WASM memory
        memory.write(&mut caller, asset_id_ptr as usize, &asset_id_bytes[..write_len])?;
    }

    // Return the length of the asset ID
    Ok(asset_id.len() as u32)
}

/// Helper function to read a string from WASM memory
fn read_string_from_memory<T>(
    memory: &Memory,
    caller: &mut Caller<'_, T>,
    ptr: u32,
    len: u32,
) -> Result<String> {
    let mut buffer = vec![0u8; len as usize];
    memory.read(&caller, ptr as usize, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

/// Helper function to read bytes from WASM memory
fn read_bytes_from_memory<T>(
    memory: &Memory,
    caller: &mut Caller<'_, T>,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; len as usize];
    memory.read(&caller, ptr as usize, &mut buffer)?;
    Ok(buffer)
}
