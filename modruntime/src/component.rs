//! Component registry for mod components
//!
//! This module provides functionality for registering and managing components
//! that can be accessed by mods.

use std::any::Any;

// Component registry using linkme
#[linkme::distributed_slice]
pub static COMPONENT_REGISTRY: [ComponentRegistration] = [..];

/// Component registration information
pub struct ComponentRegistration {
    /// The ID of the component
    pub id: &'static str,
    /// Serialization function
    pub serialize_fn: fn(&dyn Any) -> Vec<u8>,
    /// Deserialization function
    pub deserialize_fn: fn(&[u8]) -> Box<dyn Any>,
}