//! Example Game Mod
//!
//! This is an example mod that demonstrates how to create a Bevy mod.
//! Mods are compiled to WebAssembly and loaded by the host application.

// Remove Bevy prelude import since we're in a WASM context
// use bevy::prelude::*;
use game_sdk::{Player, Rect, Square};
use bevy_modapi::*;
use bevy_math::{Vec2, IVec2};

#[system(schedule = Startup)]
pub fn example_startup_system() {
    log_info!("Startup system running from mod");
    spawn!(Square(Vec2 { x: 100.0, y: 120.6 }), Rect(IVec2 { x: 60, y: 66 }));
    spawn!(Square(Vec2 { x: 100.0, y: 120.3 }), Rect(IVec2 { x: 60, y: 88 }));
}

#[system(schedule = Update)]
pub fn example_update_system() {
    log_info!("Update system running from mod");

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
pub fn spawn_entities_system() {
    // Query for existing (Square, Rect) entities
    let entities: Vec<(Square, Rect)> = query!(Square, Rect).collect();
    
    // If we have less than 3 entities, spawn a new one
    
    if entities.len() < 3 {
        log_info!("Spawning a new entity, current count: {}", entities.len());
        spawn!(Square(Vec2 { x: 100.0, y: 120.0 }), Rect(IVec2 { x: 60, y: 64 }));
    }
}

// Define the mod and list all its systems
system_def!(example_startup_system, example_update_system, spawn_entities_system);
