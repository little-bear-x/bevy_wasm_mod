//! Spawn functionality for mods
//!
//! This module provides the host-side implementation for spawning entities from mods.

use crate::ModState;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use std::any::Any;

/// Handle entity spawn request from WASM
pub fn host_handle_spawn_entities(
    mut caller: wasmtime::Caller<'_, ModState>,
    components_ptr: i32,
    components_len: i32,
) {
    // Read components data from WASM memory
    let memory = match caller.get_export("memory") {
        Some(export) => export.into_memory().unwrap(),
        None => {
            error!("Failed to get memory while spawning entities");
            return;
        }
    };

    let mem_data = memory.data(&caller);
    let components_bytes =
        &mem_data[components_ptr as usize..(components_ptr + components_len) as usize];

    // Deserialize components data
    let components_data: Vec<(String, Vec<u8>)> =
        match bincode::serde::decode_from_slice(components_bytes, bincode::config::standard()) {
            Ok((data, _)) => data,
            Err(e) => {
                error!(
                    "Failed to deserialize components data while spawning entities: {}",
                    e
                );
                return;
            }
        };

    // Get the Bevy world from the caller's data
    let world = match caller.data().get_world() {
        Some(world) => world,
        None => {
            error!("Failed to get Bevy world while spawning entities");
            return;
        }
    };

    // Spawn the entity with components
    spawn_entity_with_components(&world, &components_data);
}

/// Spawn an entity with the specified components
fn spawn_entity_with_components(
    world: &UnsafeWorldCell<'_>,
    components_data: &Vec<(String, Vec<u8>)>,
) {
    unsafe {
        let world_mut = world.world_mut();
        let mut entity_commands = world_mut.spawn_empty();

        for (component_id, component_data) in components_data {
            // Find the component registration
            let registration = match crate::component::find_component_registration(component_id) {
                Some(reg) => reg,
                None => {
                    error!("Component registration not found for ID: {}", component_id);
                    continue;
                }
            };

            // Deserialize the component
            let component_any: Box<dyn Any> = (registration.deserialize_fn)(component_data);

            // Use the insert function to add the component to the entity
            (registration.insert_fn)(&mut entity_commands, component_any);
        }

        // Actually spawn the entity
        let entity = entity_commands.id();
        info!("Spawned entity with ID: {:?}", entity);
    }
}
