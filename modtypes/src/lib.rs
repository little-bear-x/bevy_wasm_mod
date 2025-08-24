//! Shared type

/// System info
#[repr(C)]
#[derive(Debug)]
pub struct SystemInfo {
    /// The name of the system as it will be exported.
    pub export_name: [u8; 64],
}
