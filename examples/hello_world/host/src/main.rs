//! Example Host Application
//!
//! This is an example host application that demonstrates how to load and run Bevy mods.
//! The host is responsible for loading mods, setting up the WebAssembly runtime,
//! and coordinating communication between mods and the main game.

use bevy::{log::LogPlugin, prelude::*};
use modruntime::WasmModPlugin;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(WasmModPlugin::default().add_mod_path(
            "/home/PulseX/Projects/bevy_mod/target/wasm32-unknown-unknown/debug/game_mod.wasm",
        ))
        .run();
}
