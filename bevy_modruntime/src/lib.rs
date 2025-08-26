//! Bevy Mod Runtime
//!
//! This crate provides the runtime for loading and executing Bevy mods.
//! It handles WebAssembly sandboxing and communication between mods and the host application.

pub mod component;
pub mod log;
pub mod query;
pub mod resource;
pub mod spawn;
pub mod system;
mod utils;

use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy_modtypes::SystemInfo;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use utils::*;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::preview1::WasiP1Ctx;

// Re-export log handle
pub use log::host_handle_log;

// Re-export component registry and registration
pub use component::{COMPONENT_REGISTRY, ComponentRegistration, HostModResult};

// Re-export query
pub use query::host_handle_query_components;

// Re-export resource registry and registration
pub use resource::{RESOURCE_REGISTRY, ResourceRegistration, host_handle_query_resources};

// Re-export system handle
pub use system::{ModSystems, execute_mod_update_systems, execute_mod_startup_systems};

// Re-export spawn functionality
pub use spawn::host_handle_spawn_entities;

// Re-export the mod_component macro
pub use bevy_modruntime_macros::{mod_component, mod_resource};

use crate::system::ModSystemInfo;
use crate::system::ModSystemSchedule;

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

        app.add_systems(PreStartup, load_all_mod);
        app.add_systems(Startup, load_world);
        app.add_systems(PostStartup, execute_mod_startup_systems);
        app.add_systems(PostUpdate, execute_mod_update_systems);
    }
}

/// Resource all loaded mod list
#[derive(Resource)]
pub struct LoadedMods(pub HashMap<String, LoadedMod>);

/// loaded mod with its systems
pub struct LoadedMod {
    /// System information for each system
    pub system_infos: HashMap<String, SystemInfo>,
    /// The WASM instance
    pub instance: Instance,
    /// The WASM store
    pub store: Arc<RwLock<Store<ModState>>>,
}

/// Wasm state of mod
pub struct ModState {
    wasi_ctx: Arc<Mutex<UnsafeCell<WasiP1Ctx>>>,
    /// Reference to the Bevy world for component queries
    world: Option<Arc<UnsafeWorldCell<'static>>>,
}

impl ModState {
    pub fn new(wasi_ctx: WasiP1Ctx) -> Self {
        Self {
            wasi_ctx: Arc::new(Mutex::new(UnsafeCell::new(wasi_ctx))),
            world: None,
        }
    }

    // Safe
    /// Get wasi ctx
    pub fn get_wasi_ctx_mut(&mut self) -> &mut WasiP1Ctx {
        unsafe { &mut *self.wasi_ctx.lock().unwrap().get() }
    }

    /// Set the Bevy world reference
    pub fn set_world(&mut self, world: Arc<UnsafeWorldCell<'static>>) {
        self.world = Some(world);
    }

    /// Get the Bevy world reference
    pub fn get_world(&self) -> Option<UnsafeWorldCell<'static>> {
        match self.world.clone() {
            Some(world) => Some(*world),
            None => None,
        }
    }
}

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

        let mut linker: Linker<ModState> = Linker::new(&engine);
        match wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |state| {
            state.get_wasi_ctx_mut()
        }) {
            Ok(_) => {}
            Err(e) => {
                error!("Link wasi for mod '{}' faild: {}", mod_path, e);
                continue;
            }
        }
        match linker.func_wrap("env", "__mod_log", host_handle_log) {
            Ok(_) => {}
            Err(e) => {
                error!("Error in link mod '{}' __mod_log: {}", mod_path, e);
            }
        };

        // Add query components function
        match linker.func_wrap(
            "env",
            "__mod_query_components",
            host_handle_query_components,
        ) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "Error in link mod '{}' __mod_query_components: {}",
                    mod_path, e
                );
            }
        };

        // Add query resources function
        match linker.func_wrap("env", "__mod_query_resources", host_handle_query_resources) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "Error in link mod '{}' __mod_query_resources: {}",
                    mod_path, e
                );
            }
        };

        // Add free memory function
        match linker.func_wrap("env", "__mod_free_memory", host_handle_free_memory) {
            Ok(_) => {}
            Err(e) => {
                error!("Error in link mod '{}' __mod_free_memory: {}", mod_path, e);
            }
        };

        // Add spawn entities function
        match linker.func_wrap("env", "__mod_spawn_entities", host_handle_spawn_entities) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "Error in link mod '{}' __mod_spawn_entities: {}",
                    mod_path, e
                );
            }
        };

        // wasi ctx
        let mod_state = ModState::new(
            WasiCtxBuilder::new()
                .inherit_env()
                .args(&[mod_path])
                .build_p1(),
        );

        // Create a store
        let store = Store::new(&engine, mod_state);
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
                "System info for '{}': export_name = '{}', schedule = {}",
                system_name, &export_name, info.schedule
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

            r_mod_systems.0.push(ModSystemInfo {
                mod_name: mod_name.clone(),
                schedule: ModSystemSchedule::from(info.schedule),
                run_func: func,
            });
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

/// Update the world reference in all mod states
fn load_world(world: &mut World) {
    // reg types
    {
        let app_type_registry = world.resource_mut::<AppTypeRegistry>();
        let mut registry = app_type_registry.write();
        for registration in COMPONENT_REGISTRY {
            (registration.reg_fn)(&mut registry)
        }
        for registration in RESOURCE_REGISTRY {
            (registration.reg_fn)(&mut registry)
        }
    }

    // Get the LoadedMods resource
    let loaded_mod_keys: Vec<String> = {
        let r_loaded_mods = world.get_resource::<LoadedMods>();
        if r_loaded_mods.is_none() {
            return;
        }
        let r_loaded_mods = r_loaded_mods.unwrap();
        r_loaded_mods.0.keys().cloned().collect()
    };

    info!("loading world for mods {:?}", loaded_mod_keys);

    // Create a static reference to the world
    // This is safe because the world reference is valid for the duration of the application
    let world_cell = unsafe {
        std::mem::transmute::<UnsafeWorldCell<'_>, UnsafeWorldCell<'static>>(
            world.as_unsafe_world_cell(),
        )
    };

    // Update each loaded mod
    let r_loaded_mods = world.get_resource::<LoadedMods>().unwrap();
    for mod_name in &loaded_mod_keys {
        if let Some(loaded_mod) = r_loaded_mods.0.get(mod_name) {
            let store_arc = loaded_mod.store.clone();
            let mut store = store_arc.write().unwrap();
            store.data_mut().set_world(Arc::new(world_cell));
        } else {
            error!("Faild to load world {}", mod_name);
        }
    }
}
