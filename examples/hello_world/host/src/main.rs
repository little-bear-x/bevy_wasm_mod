//! Example Host Application
//!
//! This is an example host application that demonstrates how to load and run Bevy mods.
//! The host is responsible for loading mods, setting up the WebAssembly runtime,
//! and coordinating communication between mods and the main game.

use bevy::{log::LogPlugin, prelude::*};
use modruntime::{WasmModPlugin, mod_component, COMPONENT_REGISTRY};

#[mod_component(id = "square")]
#[derive(Component, Debug)]
pub struct Square(pub Vec2);

#[mod_component(id = "rect")]
#[derive(Component, Debug)]
pub struct Rect(pub IVec2);

fn print_component_registry() {
    info!("Component registry:");
    for registration in COMPONENT_REGISTRY {
        info!("  Component ID: {}", registration.id);
    }
}

fn main() {
    App::new()
        // .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(WasmModPlugin::default().add_mod_path(
            "/home/PulseX/Projects/bevy_mod/target/wasm32-unknown-unknown/debug/game_mod.wasm",
        ))
        .add_systems(Startup, print_component_registry)
        .run();
}
