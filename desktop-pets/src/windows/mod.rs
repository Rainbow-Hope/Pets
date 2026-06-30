pub mod metrics;

#[cfg(windows)]
mod app;

use crate::config::MovementMode;
use std::path::Path;

pub const FRAME_INTERVAL_MS: u32 = 100;
pub const COMMAND_MOVEMENT_BASE: u32 = 200;

pub fn command_to_movement(command: u32) -> Option<MovementMode> {
    match command.checked_sub(COMMAND_MOVEMENT_BASE)? {
        0 => Some(MovementMode::Fixed),
        1 => Some(MovementMode::BottomStrip),
        2 => Some(MovementMode::Line),
        3 => Some(MovementMode::WholeScreen),
        4 => Some(MovementMode::BetweenMonitors),
        5 => Some(MovementMode::SemiFixed),
        _ => None,
    }
}

pub fn directory_scope_id(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('/', "\\").to_lowercase();
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in normalized.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(windows)]
pub fn run() -> Result<(), String> {
    app::run()
}

#[cfg(windows)]
pub fn show_blocking_error(message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    let message: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "DesktopPets"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: both UTF-16 strings are zero-terminated and valid during the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
pub fn run() -> Result<(), String> {
    Err("DesktopPets is available only on Windows 10/11".to_owned())
}
