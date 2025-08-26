//! Component registry for mod components
//!
//! This module provides functionality for registering and managing components
//! that can be accessed by mods.

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
pub use bevy_modtypes::HostModResult;
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
    /// Insert component function
    pub insert_fn: fn(&mut EntityWorldMut, Box<dyn Any>),
}

/// Find a component registration by ID
pub fn find_component_registration(id: &str) -> Option<&'static ComponentRegistration> {
    COMPONENT_REGISTRY.iter().find(|reg| reg.id == id)
}


