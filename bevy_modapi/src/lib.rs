//! Bevy Mod API
//!
//! This crate provides the API for creating Bevy mods.
//! Mod developers will use this crate to create their mods.

pub mod log;
pub mod query;
pub mod resource;
pub mod spawn;

// Re-export the macros
pub use bevy_modapi_macros::{system, system_def};
pub use bevy_modsdk::Component;
pub use bevy_modtypes::{HostModResult, SystemInfo};

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

    /// Query resources from the host
    /// Returns a pointer to serialized resource data and the length
    pub fn __mod_query_resources(
        resource_id_ptr: *const u8,
        resource_id_len: usize,
        result_ptr: *mut u8,
    ) -> usize;

    /// Spawn entities with components
    pub fn __mod_spawn_entities(components_ptr: *const u8, components_len: usize);

    /// Free memory allocated by the host
    pub fn __mod_free_memory(ptr: *mut u8, len: usize);
}
