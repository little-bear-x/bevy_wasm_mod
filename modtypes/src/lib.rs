//! Shared type

/// System info
#[repr(C)]
#[derive(Debug)]
pub struct SystemInfo {
    /// The name of the system as it will be exported.
    pub export_name: [u8; 64],
}

/// Log level
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// Log message
#[repr(C)]
#[derive(Debug)]
pub struct LogMessage {
    /// Log level
    pub level: LogLevel,
    /// Message content
    pub message: [u8; 256],
}

