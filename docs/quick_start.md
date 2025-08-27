# Quick Start

English | [中文简体](zh/快速开始.md)

## Process Explanation

| Name | Project Type | Developer | Explanation | Dependencies |
| -- | -- | -- | -- | -- |
| Game Binary | bin | Game Developer | The game binary, the host for WASM | bevy_modruntime |
| Game SDK | lib | Game Developer | SDK provided for mod developers to interact with the game binary (must be open source) | bevy_modsdk |
| Game Mod | lib | Mod Developer | Game plugin, loaded by the game binary | bevy_modapi |

## Quickly Develop a Mod

### Game Binary (bin)
The game binary needs to depend on some crates. Example Cargo.toml configuration:
```toml
[package]
# ...package configuration
[dependencies]
bevy = { version = "0.16", features = ["serialize"] }
serde = { version = "1.0", features = ["derive"] }
bincode = { version = "2.0", features = ["serde"] }
linkme = "0.3"
bevy_modruntime = "0.16"
# ...other dependencies
```
Add mod support to the game binary:
```rs
fn main() {
    App::new()
        .add_plugins(
            WasmModPlugin::default()
                .add_mod_path(
                    // Replace this path with the real mod path
                    "path/to/your/mod.wasm",
                ),
        )
        // Add other plugins here
        // Note: The LogPlugin is necessary
        .run();
}
```

### Game SDK (lib)
For now, we'll just show how to add a system in the mod, which doesn't require using the SDK. We'll just create a placeholder project and expand on it later.
Example Cargo.toml configuration:
```toml
[package]
# ...package configuration
[dependencies]
bevy_modsdk = "0.16"
serde = { version = "1.0", features = ["derive"] }
# ...other dependencies
```

### Game Mod (lib)
Now, let's try writing a mod that adds a system and outputs `hello_world`.

Configure Cargo.toml:
```toml
[package]
# ...package settings
[lib]
crate-type = ["cdylib"]  # Mods will be compiled to WASM
[dependencies]
bevy_modapi = "0.16"
game_sdk = { path = "path/to/gamesdk" }  # Replace with the real SDK path
bincode = { version = "2.0", features = ["serde"] }
# ...other dependencies
```
Then add a system in the mod:
```rs
#[system(schedule = Update)] // Runs in the PostUpdate stage
pub fn example_update_system() {
    log_info!("Hello world from mod"); // Do not use the print! macro or Bevy's log macros
}

system_def!(example_update_system); // Add the system
```

### Compile & Run
#### Compile the Mod
In the terminal, within the mod directory, run:
```bash
cargo build --target wasm32-wasip1
```
This will compile the mod into an executable for the `wasm32-wasip1` target.

*Note: Due to potentially unstable Rust support for `wasm32-wasip1`, the latest stable version of rustc might not be able to compile for this target. First try using the latest stable rustc. If you cannot add this target, try installing rustc version 1.86, adding this target for it, and compiling with `cargo +1.86 build --target wasm32-wasip1`.*

*Note: It is recommended to compile for release when publishing officially.*

#### Compile the Game Binary
Ensure the mod file path in the mod plugin loading settings is correct.

In the terminal, within the game binary directory, run:
```bash
cargo run
```

*Note: Dynamic linking can be used.*

#### Expected Outcome
After running the game binary, the terminal should output the log `Hello world from mod`.

If `MinimalPlugins` or `DefaultPlugins` are enabled in the game binary, this log will output continuously because the system runs in the Update stage.

## More About the `system` and `system_def` Macros
In the example above, we used the `system` procedural macro to create a system in the mod. The `system` macro allows us to set the system's run schedule. Currently, two run schedules are supported:
| Declared Schedule in Mod | Actual Run Schedule in Game |
| -- | -- |
| Startup | PostStartup |
| Update | PostUpdate |

Mod developers can set the schedule using the `schedule` attribute. If it is not set or the value is invalid, the default Update schedule will be used.

---

The `system_def` macro defines all systems in the mod. A mod has one and only one `system_def` macro.

## Querying Components in a Mod
### Define a Component and Spawn an Entity in the Game Binary
```rs
#[mod_component(id = "square")] // Mark the component and set a unique id
#[derive(Component, Debug)]
pub struct Square(pub Vec2);
#[mod_component(id = "rect")]
#[derive(Component, Debug)]
pub struct Rect(pub IVec2);

fn spawn_example_component(mut commands: Commands) {
    commands.spawn((Square(Vec2 { x: 0.0, y: 1.0 }), Rect(IVec2 { x: 3, y: 4 })));
    commands.spawn((
        Square(Vec2 { x: 2.0, y: 3.6 }),
        Rect(IVec2 { x: 32, y: 48 }),
    ));
}

fn main() {
    App::new()
        // ...
        .add_systems(Startup, spawn_example_component)
		 // ...
        .run();
}
```

### Modify the SDK
We need to write a struct in the open-source SDK that has an identical signature to the component defined in the game binary that the mod can access.

Before that, we need to add necessary dependencies to Cargo.toml:
```toml
# ...
[dependencies]
bevy_math = { version = "0.16", features = ['serialize'] }
# ...
```

*Note: Here we do not depend on Bevy to prevent unexpected errors when compiling for wasm32-wasip1.*

Then, we add the structs in lib.rs:
```rs
#[component(id = "square")]
pub struct Square(pub bevy_math::Vec2);
#[component(id = "rect")]
pub struct Rect(pub bevy_math::IVec2);
```

*Note: The signature of these structs must be completely consistent with those in the game binary, and the component id must also be completely consistent with the game binary. Use the component macro provided by bevy_modsdk for marking.*

### Query Components in the Mod
Finally, we query the components in the mod. We will modify the `example_update_system` directly:
```rs
#[system(schedule = Update)]
pub fn example_update_system() {
    log_info!("Update system running from mod");

    for (square, rect) in query!(Square, Rect) {
        log_info!("From Mod: Found square: {:?} and rect: {:?}", square.0, rect.0);
    }
}
```

Finally, recompile the mod and run the game binary to see the result.

## Spawning Entities in a Mod
We can easily spawn entities in the mod. Let's say we want to spawn an entity with `(Square, Rect)`. We only need to modify the mod's code. We'll add a new Startup system to spawn the entity.
```rs
#[system(schedule = Startup)]
pub fn example_startup_system() {
    log_info!("Startup system running from mod");;
    
    spawn!(Square(Vec2 { x: 100.0, y: 120.6 }), Rect(IVec2 { x: 60, y: 66 }));
    spawn!(Square(Vec2 { x: 100.0, y: 120.3 }), Rect(IVec2 { x: 60, y: 88 }));
}

system_def!(example_startup_system, example_update_system); // Modify system_def to add this system
```

Then, we recompile the mod and run the game binary.

## Accessing Game Resources in a Mod
Accessing Resources is similar to querying components.

### Define and Add a Resource in the Game Binary
```rs
#[mod_resource(id = "player")] // Mark and set the resource's unique id
#[derive(Resource, Debug)]
pub struct Player(pub Vec2);

fn main() {
    App::new()
        // ...
		 .insert_resource(Player(Vec2 { x: 10.0, y: 20.0 }))
        .run();
}
```

### Modify the Game SDK
```rs
#[resource(id = "player")] // Ensure the id and signature are consistent with the game binary
pub struct Player(pub bevy_math::Vec2);
```

### Query the Resource in the Mod
We modify the `example_update_system` directly:
```rs
#[system(schedule = Update)]
pub fn example_update_system() {
    log_info!("Update system running from mod");

    for (square, rect) in query!(Square, Rect) {
        log_info!("From Mod: Found square: {:?} and rect: {:?}", square.0, rect.0);
    }
    
	 // Add resource query functionality
    if let Some(player) = res!(Player) {
        log_info!("From Mod: Found player: {:?}", player.0);
    } else {
        log_warn!("From Mod: Player resource not found");
    }
}
```

Finally, recompile the mod and run the game binary to see the result.

## Adding Assets in a Mod
We can add assets required by the game in the mod, such as images, audio, etc. For demonstration, we'll use a simple text file as an example.

### Create a Text File in the Mod's src Directory
Create a text file `example_asset.txt` and write:
```
This is an example asset file for the Bevy WASM mod example.
```

### Add the Asset in the Mod
We modify the `example_startup_system` directly:
```rs
#[system(schedule = Startup)]
pub fn example_startup_system() {
    log_info!("Startup system running from mod");
    
    // Define an asset and get its ID
    let asset_id = asset_def!(type=text, src="example_asset.txt");
    log_info!("Defined asset with ID: {}", asset_id);
    
    spawn!(Square(Vec2 { x: 100.0, y: 120.6 }), Rect(IVec2 { x: 60, y: 66 }));
    spawn!(Square(Vec2 { x: 100.0, y: 120.3 }), Rect(IVec2 { x: 60, y: 88 }));
}
```

*Note: The asset_def macro uses the include_bytes macro to directly pack the asset binary.*

### Add and Set the Asset Handling Function in the Game Binary
First, we define an asset receiving function. Note, the parameter and return signatures of this function are fixed.
```rs
fn handle_new_asset(_world: &mut World, asset_info: AssetInfo) -> String {
    info!("New asset from mod:");
    info!("  Mod name: {}", asset_info.mod_name);
    info!("  Asset type: {}", asset_info.asset_type);
    info!(
        "  Asset data: {}",
        String::from_utf8_lossy(&asset_info.asset_data)
    );

    // Should return the asset id for the mod here.
	 // The asset id can be considered the unique id for all assets added by mods, handled by the game developer. It is necessary if you want the mod to be able to access the asset. We leave it empty here.
    String::new()
}
```

Then, we modify the settings of our `WasmModPlugin`:
```rs
fn main() {
    App::new()
        // ...
        .add_plugins(
            WasmModPlugin::default()
                .add_mod_path(
                    "path/to/your/mod.wasm",
                )
                .set_new_asset_fn(handle_new_asset),
        )
        .run();
}
```

Finally, recompile the mod, run the program, and check the result.

## Example Project
All the above demonstrations can be found in the [hello_world](../examples/hello_world/README.md) example.