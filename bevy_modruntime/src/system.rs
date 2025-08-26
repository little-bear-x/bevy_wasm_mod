use crate::LoadedMods;
use bevy::prelude::*;
use wasmtime::TypedFunc;

/// Schedule of mod system
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ModSystemSchedule {
    /// In PostStartup
    Startup,
    /// In PostUpdate
    Update,
}

impl From<u8> for ModSystemSchedule {
    fn from(schedule: u8) -> Self {
        match schedule {
            1 => ModSystemSchedule::Startup,
            _ => ModSystemSchedule::Update,
        }
    }
}

/// Mod system info
pub struct ModSystemInfo {
    pub mod_name: String,
    pub schedule: ModSystemSchedule,
    pub run_func: TypedFunc<(), ()>,
}

/// Resource to store mod systems info
#[derive(Resource)]
pub struct ModSystems(pub Vec<ModSystemInfo>);

/// System to execute startup mod systems
pub fn execute_mod_startup_systems(
    mut mod_systems: ResMut<ModSystems>,
    r_loaded_mods: Res<LoadedMods>,
) {
    // Execute each mod system
    for mod_info in &mut mod_systems.0 {
        if mod_info.schedule != ModSystemSchedule::Startup {
            continue;
        }
        let store_arc = match r_loaded_mods.0.get(&mod_info.mod_name) {
            Some(loaded_mod) => loaded_mod.store.clone(),
            None => {
                error!("executing system err: mod {} not found", &mod_info.mod_name);
                continue;
            }
        };
        let mut store = store_arc.write().unwrap();
        match mod_info.run_func.call(&mut *store, ()) {
            Ok(_) => {}
            Err(e) => error!("Failed to execute system: {}", e),
        }
    }
}

/// System to execute update mod systems
pub fn execute_mod_update_systems(
    mut mod_systems: ResMut<ModSystems>,
    r_loaded_mods: Res<LoadedMods>,
) {
    // Execute each mod system
    for mod_info in &mut mod_systems.0 {
        if mod_info.schedule != ModSystemSchedule::Update {
            continue;
        }
        let store_arc = match r_loaded_mods.0.get(&mod_info.mod_name) {
            Some(loaded_mod) => loaded_mod.store.clone(),
            None => {
                error!("executing system err: mod {} not found", &mod_info.mod_name);
                continue;
            }
        };
        let mut store = store_arc.write().unwrap();
        match mod_info.run_func.call(&mut *store, ()) {
            Ok(_) => {}
            Err(e) => error!("Failed to execute system: {}", e),
        }
    }
}
