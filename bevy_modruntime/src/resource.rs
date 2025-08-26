//! Resource registry for mod resources
//!
//! This module provides functionality for registering and managing resources
//! that can be accessed by mods.

use crate::ModState;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use std::any::{Any, TypeId};
pub use bevy_modtypes::QueryResult;

// Resource registry using linkme
#[linkme::distributed_slice]
pub static RESOURCE_REGISTRY: [ResourceRegistration] = [..];

/// Resource registration information
pub struct ResourceRegistration {
    /// The ID of the resource
    pub id: &'static str,
    /// Serialization function
    pub serialize_fn: fn(bevy::ptr::Ptr<'_>) -> Vec<u8>,
    /// Deserialization function
    pub deserialize_fn: fn(&[u8]) -> Box<dyn Any>,
    /// Type id
    pub get_type_id: fn() -> TypeId,
    /// Reg type function
    pub reg_fn: fn(&mut TypeRegistry),
}

/// Find a resource registration by ID
pub fn find_resource_registration(id: &str) -> Option<&'static ResourceRegistration> {
    RESOURCE_REGISTRY.iter().find(|reg| reg.id == id)
}

/// Handle resource query from WASM
pub fn host_handle_query_resources(
    mut caller: wasmtime::Caller<'_, ModState>,
    resource_id_ptr: i32,
    resource_id_len: i32,
    result_ptr: i32,
) -> i32 {
    // Read resource ID from WASM memory
    let memory = match caller.get_export("memory") {
        Some(export) => export.into_memory().unwrap(),
        None => {
            error!(
                "get memory faild while querying resource {}",
                resource_id_ptr
            );
            return 0;
        }
    };

    let mem_data = memory.data(&caller);
    let resource_id_bytes =
        &mem_data[resource_id_ptr as usize..(resource_id_ptr + resource_id_len) as usize];

    // Deserialize resource ID
    let resource_id: String =
        match bincode::serde::decode_from_slice(resource_id_bytes, bincode::config::standard()) {
            Ok((id, _)) => id,
            Err(e) => {
                error!(
                    "deserialize resource id faild while querying resource {}: {}",
                    resource_id_ptr, e
                );
                return 0;
            }
        };

    // Get the Bevy world from the caller's data
    let world = match caller.data().get_world() {
        Some(world) => world,
        None => {
            error!(
                "get bevy world faild while querying resource {}",
                resource_id_ptr
            );
            return 0;
        }
    };

    // Query resource from the world
    let serialized_data = match query_resource_from_world(&world, &resource_id) {
        Some(data) => {
            data
        },
        None => {
            warn!(
                "no resource return while querying resource {}",
                resource_id_ptr
            );
            return 0;
        }
    };

    // If we have data, allocate memory in WASM space and copy the data there
    if !serialized_data.is_empty() {
        let data_len = serialized_data.len();

        // Allocate memory in WASM space for the serialized data
        // We'll use a simple approach: allocate at a fixed offset in the memory
        // In a real implementation, we would need a proper memory allocator
        let data_ptr_offset = 0x100000; // Start at 1MB offset

        // Write the serialized data to WASM memory
        if let Err(e) = memory.write(&mut caller, data_ptr_offset, &serialized_data) {
            error!("Failed to write serialized data to WASM memory: {}", e);
            return 0;
        }

        // Create a QueryResult struct with the correct data pointer and length
        let query_result = QueryResult {
            data_ptr: data_ptr_offset as u32,
            data_len: data_len as u32,
        };

        // Write the QueryResult struct to WASM memory at the provided result_ptr
        let result_bytes = unsafe {
            std::slice::from_raw_parts(
                &query_result as *const QueryResult as *const u8,
                std::mem::size_of::<QueryResult>(),
            )
        };

        if let Err(e) = memory.write(&mut caller, result_ptr as usize, result_bytes) {
            error!("Failed to write QueryResult to WASM memory: {}", e);
            return 0;
        }

        data_len as i32
    } else {
        0
    }
}

/// Query a resource from the Bevy world
pub fn query_resource_from_world(
    world: &UnsafeWorldCell<'_>,
    resource_id: &str,
) -> Option<Vec<u8>> {
    // Find the resource registration
    let registration = find_resource_registration(resource_id)?;
    
    unsafe {
        let world_origin = world.world_mut();
        
        // Get the resource type id
        let type_id = (registration.get_type_id)();
        
        // Get the component id for the resource type
        let component_id = match world_origin.components().get_resource_id(type_id) {
            Some(id) => {
                id
            },
            None => {
                warn!("Failed to get component ID for resource type");
                return None;
            }
        };
        
        // Try to get the resource from the world
        match world_origin.get_resource_by_id(component_id) {
            Some(resource_ptr) => {
                // Serialize the resource
                let serialized_resource = (registration.serialize_fn)(resource_ptr);
                Some(serialized_resource)
            },
            None => {
                warn!("Resource not found in world");
                None
            }
        }
    }
}
