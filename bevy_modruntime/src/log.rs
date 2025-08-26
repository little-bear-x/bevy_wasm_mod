use bevy::log::{debug, error, info, warn};
use crate::ModState;

/// Handle log
pub fn host_handle_log(
    mut caller: wasmtime::Caller<'_, ModState>,
    ptr: i32,
    len: i32,
    level: i32,
) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mem_data = memory.data(&mut caller);
    let bytes = &mem_data[ptr as usize..(ptr + len) as usize];
    let msg = String::from_utf8_lossy(bytes);

    match level {
        0 => {
            debug!("{}", msg);
        }
        1 => {
            info!("{}", msg);
        }
        2 => {
            warn!("{}", msg);
        }
        3 => {
            error!("{}", msg);
        }
        _ => {
            info!("{}", msg);
        }
    }
}


