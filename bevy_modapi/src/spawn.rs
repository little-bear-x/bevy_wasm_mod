/// Spawn entities with components
///
/// This macro allows mods to spawn entities with specified components.
/// The syntax is `spawn!(component1, component2, ...);`
/// where each component is a tuple of (ComponentType, ComponentValue).
///
/// Example:
/// ```rust
/// spawn!(Square(Vec2 { x: 100.0, y: 120.0 }), Rect(IVec2 { x: 60, y: 64 }));
/// ```
#[macro_export]
macro_rules! spawn {
    ($($component:expr),+ $(,)?) => {
        {
            // Create a vector of components
            let components: Vec<(&'static str, Vec<u8>)> = vec![
                $({
                    let component = $component;
                    // Use the component_id() method to get the component ID
                    let component_id = {
                        fn get_component_id<T: bevy_modapi::Component>(_: &T) -> &'static str {
                            T::component_id()
                        }
                        get_component_id(&component)
                    };
                    let serialized_component = bincode::serde::encode_to_vec(&component, bincode::config::standard())
                        .expect("Failed to serialize component");
                    (component_id, serialized_component)
                }),+
            ];

            // Serialize the components vector
            let serialized_components = bincode::serde::encode_to_vec(&components, bincode::config::standard())
                .expect("Failed to serialize components");

            // Call the host function to spawn entities
            unsafe {
                __mod_spawn_entities(
                    serialized_components.as_ptr(),
                    serialized_components.len(),
                );
            }
        }
    };
}
