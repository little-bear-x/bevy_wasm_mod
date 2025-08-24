//! Example of game sdk.
//!
//! Used for complex structure interaction with the Host. Ensure that the definition is the same as the Host

use modsdk::component;

#[component(id = "square")]
pub struct Square(pub bevy_math::Vec2);

#[component(id = "rect")]
pub struct Rect(pub bevy_math::IVec2);
