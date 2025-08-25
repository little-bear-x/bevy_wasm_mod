//! Component registry for mod components
//!
//! This module provides functionality for registering and managing components
//! that can be accessed by mods.

use crate::ModState;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
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

/// Query result structure for passing data to WASM
/// 
/// This struct is used to pass data between the host and WASM modules.
/// It must be compatible with the WASM32 target platform, where pointers and usize are 32-bit.
#[repr(C)]
#[derive(Debug)]
pub struct QueryResult {
    /// Pointer to the serialized data
    pub data_ptr: u32,
    /// Length of the serialized data
    pub data_len: u32,
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
pub fn host_handle_free_memory(mut _caller: wasmtime::Caller<'_, ModState>, _ptr: i32, _len: i32) {
    // In a full implementation, this would free memory allocated in the WASM linear memory
    // For now, this is a placeholder
    // The actual implementation would depend on how we allocate memory in host_handle_query_components
}

/// Query components from the Bevy world
pub fn query_components_from_world(
    world: &UnsafeWorldCell<'_>,
    component_ids: &[String],
) -> Option<Vec<u8>> {
    info!("querying components: {:?}", component_ids);

    if component_ids.is_empty() {
        return None;
    }

    // For now, only support querying single components
    if component_ids.len() != 1 {
        return None;
    }

    let component_id = &component_ids[0];

    // Find the component registration
    let registration = find_component_registration(component_id)?;
    let type_id = (registration.get_type_id)();

    // Get components
    let mut results: Vec<bevy::ptr::Ptr<'_>> = Vec::new();
    unsafe {
        let world_origin = world.world_mut();
        let Some(component_id) = world_origin.components().get_id(type_id) else {
            return None;
        };

        // Iterate through all entities in the world
        for entity_ref in world_origin.iter_entities() {
            let entity = entity_ref.id();
            // Try to get the component from the entity
            if let Some(component_ptr) = world_origin.get_by_id(entity, component_id) {
                results.push(component_ptr);
            }
        }
    }

    info!("Found {} components", results.len());

    // Serialize the components
    let mut serialized_components = Vec::new();
    for component_ptr in results {
        // Convert the component pointer to a reference to Any
        let serialized_component = (registration.serialize_fn)(component_ptr);
        info!("Serialized component size: {}", serialized_component.len());
        serialized_components.push(serialized_component);
    }

    // Serialize the vector of serialized components
    let serialized_data =
        bincode::serde::encode_to_vec(&serialized_components, bincode::config::standard()).ok()?;

    info!("serialized_data length: {}", serialized_data.len());
    info!("serialized_data: {:?}", serialized_data);

    Some(serialized_data)
}
