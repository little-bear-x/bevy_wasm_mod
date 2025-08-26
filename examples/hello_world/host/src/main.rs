//! Example Host Application
//!
//! This is an example host application that demonstrates how to load and run Bevy mods.
//! The host is responsible for loading mods, setting up the WebAssembly runtime,
//! and coordinating communication between mods and the main game.

use bevy::{log::LogPlugin, prelude::*};
use bevy_modruntime::{COMPONENT_REGISTRY, RESOURCE_REGISTRY, WasmModPlugin, mod_component, mod_resource};

#[mod_component(id = "square")]
#[derive(Component, Debug)]
pub struct Square(pub Vec2);

#[mod_component(id = "rect")]
#[derive(Component, Debug)]
pub struct Rect(pub IVec2);

#[mod_resource(id = "player")]
#[derive(Resource, Debug)]
pub struct Player(pub Vec2);

fn print_component_registry() {
    info!("Component registry:");
    for registration in COMPONENT_REGISTRY {
        info!("  Component ID: {}", registration.id);
    }
}

fn print_resource_registry() {
    info!("Resource registry:");
    for registration in RESOURCE_REGISTRY {
        info!("  Resource ID: {}", registration.id);
    }
}

fn spawn_example_component(mut commands: Commands) {
    commands.spawn((Square(Vec2 { x: 0.0, y: 1.0 }), Rect(IVec2 { x: 3, y: 4 })));
    commands.spawn((Square(Vec2 { x: 2.0, y: 3.6 }), Rect(IVec2 { x: 32, y: 48 })));
}

fn insert_example_resource(mut commands: Commands) {
    commands.insert_resource(Player(Vec2 { x: 10.0, y: 20.0 }));
}

fn main() {
    App::new()
        // .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(WasmModPlugin::default().add_mod_path(
            // replace this to your path
            "/home/PulseX/Projects/bevy_wasm_mod/target/wasm32-wasip1/debug/game_mod.wasm",
        ))
        .add_systems(Startup, print_component_registry)
        .add_systems(Startup, print_resource_registry)
        .add_systems(Startup, spawn_example_component)
        .add_systems(Startup, insert_example_resource)
        .run();
}
