//! Bevy Mod Runtime
//!
//! This crate provides the runtime for loading and executing Bevy mods.
//! It handles WebAssembly sandboxing and communication between mods and the host application.

mod utils;

use bevy::prelude::*;
use modtypes::SystemInfo;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use utils::*;
use wasmtime::Linker;
use wasmtime::TypedFunc;
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
        app.insert_resource(self.clone())
            .insert_resource(LoadedMods(HashMap::new()))
            .insert_resource(ModSystems(Vec::new()));

        app.add_systems(Startup, load_all_mod);
        app.add_systems(PostUpdate, execute_mod_systems);
    }
}

/// Resource all loaded mod list
#[derive(Debug, Resource)]
pub struct LoadedMods(pub HashMap<String, LoadedMod>);

/// loaded mod with its systems
#[derive(Debug)]
pub struct LoadedMod {
    /// System information for each system
    pub system_infos: HashMap<String, SystemInfo>,
    /// The WASM instance
    pub instance: Instance,
    /// The WASM store
    pub store: Arc<RwLock<Store<()>>>,
}

/// Resource to store mod systems as closures(mod name, system func)
#[derive(Resource)]
pub struct ModSystems(pub Vec<(String, TypedFunc<(), ()>)>);

/// load all mod from mod paths
fn load_all_mod(
    r_mod: Res<WasmModPlugin>,
    mut r_loaded_mods: ResMut<LoadedMods>,
    mut r_mod_systems: ResMut<ModSystems>,
) {
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

        let mut linker = Linker::new(&engine);
        match linker.func_wrap("env", "__mod_log", host_handle_log) {
            Ok(_) => {}
            Err(e) => {
                error!("Error in link mod '{}' __mod_log: {}", mod_path, e);
            }
        };

        // Create a store
        let store = Store::new(&engine, ());
        let store_arc = Arc::new(RwLock::new(store));

        // Instantiate the module
        let instance = {
            let mut store = store_arc.write().unwrap();
            match linker.instantiate(&mut *store, &module) {
                Ok(instance) => instance,
                Err(e) => {
                    error!("Failed to instantiate mod '{}': {}", mod_path, e);
                    continue;
                }
            }
        };

        // Try to get the mod name
        let mod_name = {
            let mut store = store_arc.write().unwrap();
            match get_mod_name(&mut *store, &instance) {
                Ok(name) => {
                    info!("Mod name: '{}'", name);
                    name
                }
                Err(e) => {
                    error!("Failed to get mod name: {}", e);
                    "unnamed_mod".to_string()
                }
            }
        };

        // Get the systems names from the instance
        let systems = {
            let mut store = store_arc.write().unwrap();
            match get_systems(&mut *store, &instance) {
                Ok(systems) => {
                    info!("Get systems: {:?}", systems);
                    systems
                }
                Err(e) => {
                    error!("Failed to get systems: {}", e);
                    continue;
                }
            }
        };

        // Get system info for each system
        let mut system_infos = HashMap::new();
        for system_name in &systems {
            let info = {
                let mut store = store_arc.write().unwrap();
                match get_mod_system_info(&mut *store, &instance, system_name) {
                    Ok(info) => info,
                    Err(e) => {
                        error!("Failed to get system info for '{}': {}", system_name, e);
                        continue;
                    }
                }
            };

            let export_name = system_info_export_name_str(&info);
            info!(
                "System info for '{}': export_name = '{}'",
                system_name, &export_name
            );

            let func = {
                let mut store = store_arc.write().unwrap();
                match instance.get_typed_func::<(), ()>(&mut *store, &export_name) {
                    Ok(func) => func,
                    Err(e) => {
                        error!(
                            "Failed to get function '{}' for system '{}': {}",
                            export_name, system_name, e
                        );
                        continue;
                    }
                }
            };

            r_mod_systems.0.push((mod_name.clone(), func));
            system_infos.insert(system_name.clone(), info);
        }

        // Create a LoadedMod resource
        let loaded_mod = LoadedMod {
            system_infos,
            instance,
            store: store_arc,
        };

        // Add the loaded mod
        r_loaded_mods.0.insert(mod_name, loaded_mod);
    }
}

fn execute_mod_systems(mut mod_systems: ResMut<ModSystems>, r_loaded_mods: Res<LoadedMods>) {
    // Execute each mod system
    for (mod_name, system_fn) in &mut mod_systems.0 {
        let store_arc = match r_loaded_mods.0.get(mod_name) {
            Some(loaded_mod) => loaded_mod.store.clone(),
            None => {
                error!("executing system err: mod {} not found", &mod_name);
                continue;
            }
        };
        let mut store = store_arc.write().unwrap();
        match system_fn.call(&mut *store, ()) {
            Ok(_) => {}
            Err(e) => error!("Failed to execute system: {}", e),
        }
    }
}
