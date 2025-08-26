//! Component registry for mod components
//!
//! This module provides functionality for registering and managing components
//! that can be accessed by mods.

use crate::ModState;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
pub use bevy_modtypes::QueryResult;
use std::any::{Any, TypeId};

// Component registry using linkme
#[linkme::distributed_slice]
pub static COMPONENT_REGISTRY: [ComponentRegistration] = [..];

/// Component registration information
pub struct ComponentRegistration {
    /// The ID of the component
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

/// Find a component registration by ID
pub fn find_component_registration(id: &str) -> Option<&'static ComponentRegistration> {
    COMPONENT_REGISTRY.iter().find(|reg| reg.id == id)
}

/// Handle component query from WASM
pub fn host_handle_query_components(
    mut caller: wasmtime::Caller<'_, ModState>,
    component_ids_ptr: i32,
    component_ids_len: i32,
    result_ptr: i32,
) -> i32 {
    // Read component IDs from WASM memory
    let memory = match caller.get_export("memory") {
        Some(export) => export.into_memory().unwrap(),
        None => {
            error!(
                "get memory faild while querying components {}",
                component_ids_ptr
            );
            return 0;
        }
    };

    let mem_data = memory.data(&caller);
    let component_ids_bytes =
        &mem_data[component_ids_ptr as usize..(component_ids_ptr + component_ids_len) as usize];

    // Deserialize component IDs
    let component_ids: Vec<String> =
        match bincode::serde::decode_from_slice(component_ids_bytes, bincode::config::standard()) {
            Ok((ids, _)) => ids,
            Err(e) => {
                error!(
                    "deserialize component id faild while querying components {}: {}",
                    component_ids_ptr, e
                );
                return 0;
            }
        };

    // Get the Bevy world from the caller's data
    let world = match caller.data().get_world() {
        Some(world) => world,
        None => {
            error!(
                "get bevy world faild while querying components {}",
                component_ids_ptr
            );
            return 0;
        }
    };

    // Query components from the world
    let serialized_data = match query_components_from_world(&world, &component_ids) {
        Some(data) => data,
        None => {
            warn!(
                "no components return while querying components {}",
                component_ids_ptr
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

/// Free memory allocated for query results
pub fn host_handle_free_memory(mut caller: wasmtime::Caller<'_, ModState>, ptr: i32, len: i32) {
    // Validate the pointer and length parameters
    if ptr <= 0 || len <= 0 {
        warn!(
            "Invalid parameters for memory free: ptr={}, len={}",
            ptr, len
        );
        return;
    }

    // In the current implementation, we're using a fixed offset in WASM memory (0x100000)
    // We're not actually allocating dynamic memory, so we don't need to free anything
    // However, for safety, we'll zero out the memory to prevent potential data leaks
    // or use-after-free issues if the implementation changes in the future

    let ptr = ptr as u32;
    let len = len as usize;

    // Check if the pointer matches our fixed allocation offset
    if ptr == 0x100000 {
        // Zero out the memory region that was used for the allocation
        if let Some(export) = caller.get_export("memory") {
            if let Some(memory) = export.into_memory() {
                let zero_data = vec![0u8; len];
                if let Err(e) = memory.write(&mut caller, ptr as usize, &zero_data) {
                    warn!("Failed to zero out memory region: {}", e);
                } else {
                    info!("Memory region zeroed out: ptr={}, len={}", ptr, len);
                }
            }
        }
    } else {
        // If the pointer doesn't match our expected offset, log a warning
        // This could indicate a bug or a change in the allocation strategy
        warn!(
            "Attempted to free memory at unexpected location: ptr={}, len={}",
            ptr, len
        );
    }
}

/// Query components from the Bevy world
pub fn query_components_from_world(
    world: &UnsafeWorldCell<'_>,
    component_ids: &[String],
) -> Option<Vec<u8>> {
    if component_ids.is_empty() {
        return None;
    }

    // Find all component registrations
    let mut registrations = Vec::new();
    let mut type_ids = Vec::new();
    let mut component_db_ids = Vec::new();

    unsafe {
        let world_origin = world.world_mut();

        // Get the component registrations and their IDs
        for component_id in component_ids {
            let registration = find_component_registration(component_id)?;
            let type_id = (registration.get_type_id)();
            let component_db_id = world_origin.components().get_id(type_id)?;

            registrations.push(registration);
            type_ids.push(type_id);
            component_db_ids.push(component_db_id);
        }

        // Get components
        let mut entity_results: Vec<Vec<bevy::ptr::Ptr<'_>>> = Vec::new();

        // Iterate through all entities in the world
        for entity_ref in world_origin.iter_entities() {
            let entity = entity_ref.id();
            let mut entity_components = Vec::new();

            'a: for &component_id in &component_db_ids {
                if let Some(component_ptr) = world_origin.get_by_id(entity, component_id) {
                    entity_components.push(component_ptr);
                } else {
                    continue 'a;
                }
            }
            entity_results.push(entity_components);
        }

        // Serialize the components
        let mut serialized_entities = Vec::new();
        for entity_components in entity_results {
            let mut serialized_components = Vec::new();
            for (i, component_ptr) in entity_components.iter().enumerate() {
                let serialized_component = (registrations[i].serialize_fn)(*component_ptr);
                serialized_components.push(serialized_component);
            }
            serialized_entities.push(serialized_components);
        }

        // Serialize the vector of serialized entities
        let serialized_data =
            bincode::serde::encode_to_vec(&serialized_entities, bincode::config::standard())
                .ok()?;

        Some(serialized_data)
    }
}
