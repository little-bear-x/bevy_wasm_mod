//! Mod sdk apis

pub use bevy_modsdk_macros::*;

pub trait Component {
    fn component_id() -> &'static str;
}
