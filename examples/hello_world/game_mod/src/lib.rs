//! Example Game Mod
//!
//! This is an example mod that demonstrates how to create a Bevy mod.
//! Mods are compiled to WebAssembly and loaded by the host application.

// Remove Bevy prelude import since we're in a WASM context
// use bevy::prelude::*;
use game_sdk::{Player, Rect, Square};
use bevy_modapi::*;

#[system]
pub fn example_system() {
    log_info!("A info message from mod");

    for (square, rect) in query!(Square, Rect) {
        log_info!("From Mod: Found square: {:?} and rect: {:?}", square.0, rect.0);
    }
    
    if let Some(player) = res!(Player) {
        log_info!("From Mod: Found player: {:?}", player.0);
    } else {
        log_warn!("From Mod: Player resource not found");
    }
}

#[system]
pub fn example_system_two() {
    log_warn!("A info message from example system two");
}

// Define the mod and list all its systems
system_def!(example_system, example_system_two);
