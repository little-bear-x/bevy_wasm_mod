//! Bevy Mod Runtime
//!
//! This crate provides the runtime for loading and executing Bevy mods.
//! It handles WebAssembly sandboxing and communication between mods and the host application.

mod utils;

use bevy::prelude::*;
use std::collections::HashMap;
use utils::*;
use wasmtime::{Engine, Instance, Module, Store};

/// Plugin for mod
#[derive(Debug, Resource, Clone)]
pub struct WasmModPlugin {
    /// All mod we will load.
    pub mod_paths: Vec<String>,
}

impl Default for WasmModPlugin {
    fn default() -> Self {
        Self {
            mod_paths: Vec::new(),
        }
    }
}

impl WasmModPlugin {
    /// Adds a mod path to be loaded
    pub fn add_mod_path(mut self, path: impl Into<String>) -> Self {
        self.mod_paths.push(path.into());
        self
    }

    /// Sets the list of mod paths to load
    pub fn set_mod_paths(mut self, paths: Vec<String>) -> Self {
        self.mod_paths = paths;
        self
    }
}

impl Plugin for WasmModPlugin {
    fn build(&self, app: &mut App) {
        // Insert mod resource
        app.insert_resource(self.clone());

        app.add_systems(Startup, load_all_mod);
    }
}

/// Represents a loaded mod with its systems
#[derive(Debug, Resource)]
pub struct LoadedMod {
    /// The name of the mod
    pub name: String,
    /// Systems in the mod, mapping system name to export function name
    pub systems: HashMap<String, String>,
    /// The WASM instance
    pub instance: Instance,
}

/// load all mod from mod paths
fn load_all_mod(r_mod: Res<WasmModPlugin>, mut commands: Commands) {
    info!("loading mods: {:?}", r_mod.mod_paths);

    let engine = Engine::default();

    for mod_path in &r_mod.mod_paths {
        // Load the WASM module
        let module = match Module::from_file(&engine, mod_path) {
            Ok(module) => module,
            Err(e) => {
                error!("Failed to load mod '{}': {}", mod_path, e);
                continue;
            }
        };

        // Create a store
        let mut store = Store::new(&engine, ());

        // Instantiate the module
        let instance = match Instance::new(&mut store, &module, &[]) {
            Ok(instance) => instance,
            Err(e) => {
                error!("Failed to instantiate mod '{}': {}", mod_path, e);
                continue;
            }
        };

        // Get the MOD_SYSTEMS array from the instance
        let systems_map = HashMap::new(); // Try to get the systems count
        match get_systems(&mut store, &instance) {
            Ok(systems) => {
                info!("Get systems: {:?}", systems);
            }
            Err(e) => {
                error!("Failed to get systems: {}", e);
                continue;
            }
        }

        // Try to get the mod name
        let mod_name = match get_mod_name(&mut store, &instance) {
            Ok(name) => {
                info!("Mod name: '{}'", name);
                name
            }
            Err(e) => {
                error!("Failed to get mod name: {}", e);
                "unnamed_mod".to_string()
            }
        };

        // Create a LoadedMod resource
        let loaded_mod = LoadedMod {
            name: mod_name,
            systems: systems_map,
            instance,
        };

        // Add the loaded mod as a resource
        commands.insert_resource(loaded_mod);
    }
}
