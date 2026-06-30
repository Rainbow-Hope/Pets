#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    if let Err(error) = desktop_pets::windows::run() {
        #[cfg(windows)]
        desktop_pets::windows::show_blocking_error(&error);
        #[cfg(not(windows))]
        eprintln!("DesktopPets: {error}");
    }
}
