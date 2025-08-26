//! Shared type

/// System info
#[repr(C)]
#[derive(Debug)]
pub struct SystemInfo {
    /// The name of the system as it will be exported.
    pub export_name: [u8; 64],
    /// The schedule of the system.
    /// 0 = Update, 1 = Startup
    pub schedule: u8,
}

/// Memory result structure for passing data to WASM
/// 
/// This struct is used to pass data between the host and WASM modules.
/// It must be compatible with the WASM32 target platform, where pointers and usize are 32-bit.
#[repr(C)]
#[derive(Debug)]
pub struct HostModResult {
    /// Pointer to the serialized data
    pub data_ptr: u32,
    /// Length of the serialized data
    pub data_len: u32,
}
