#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    if let Err(error) = desktop_pets::windows::run(desktop_pets::edition::Edition::Micro) {
        #[cfg(windows)]
        desktop_pets::windows::show_blocking_error(&error);
        #[cfg(not(windows))]
        eprintln!("DesktopPets Micro: {error}");
    }
}
