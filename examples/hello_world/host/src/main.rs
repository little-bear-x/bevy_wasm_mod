//! Example Host Application
//!
//! This is an example host application that demonstrates how to load and run Bevy mods.
//! The host is responsible for loading mods, setting up the WebAssembly runtime,
//! and coordinating communication between mods and the main game.

use bevy::{log::LogPlugin, prelude::*};
use modruntime::{COMPONENT_REGISTRY, WasmModPlugin, mod_component};

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

fn spawn_example_component(mut commands: Commands) {
    commands.spawn((Square(Vec2 { x: 0.0, y: 1.0 }), Rect(IVec2 { x: 3, y: 4 })));
}

fn main() {
    App::new()
        // .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(WasmModPlugin::default().add_mod_path(
            "/home/PulseX/Projects/bevy_wasm_mod/target/wasm32-wasip1/debug/game_mod.wasm",
        ))
        .add_systems(Startup, print_component_registry)
        .add_systems(Startup, spawn_example_component)
        .run();
}
