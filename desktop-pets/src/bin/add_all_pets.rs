use desktop_pets::all_pets_helper::{
    EmbeddedPetSource, InstallReport, detect_install_target, install_all_pets,
    scan_codex_pet_sources,
};
use desktop_pets::embedded_pets::embedded_pets;
use std::env;
use std::path::PathBuf;

fn main() {
    match run() {
        Ok(report) => show_message("Adicionar Todos os Pets", &format_report(&report), false),
        Err(error) => {
            show_message("Adicionar Todos os Pets", &error, true);
            std::process::exit(1);
        }
    }
}

fn run() -> Result<InstallReport, String> {
    let current_exe = env::current_exe()
        .map_err(|error| format!("Não foi possível localizar este executável: {error}"))?;
    let exe_dir = current_exe
        .parent()
        .ok_or_else(|| "Não foi possível localizar a pasta deste executável.".to_owned())?;
    let target = detect_install_target(exe_dir).map_err(|error| error.to_string())?;

    let embedded = embedded_pets()
        .iter()
        .map(|pet| EmbeddedPetSource {
            id: pet.id,
            manifest_name: pet.manifest_name,
            manifest: pet.manifest,
            spritesheet_name: pet.spritesheet_name,
            spritesheet: pet.spritesheet,
        })
        .collect::<Vec<_>>();

    let codex_root = codex_pets_root();
    let codex_sources = scan_codex_pet_sources(&codex_root).map_err(|error| error.to_string())?;
    install_all_pets(&target, &embedded, &codex_sources).map_err(|error| error.to_string())
}

fn codex_pets_root() -> PathBuf {
    if let Some(override_root) = env::var_os("DESKTOP_PETS_CODEX_PETS_DIR") {
        return PathBuf::from(override_root);
    }
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
        .join("pets")
}

fn format_report(report: &InstallReport) -> String {
    let codex_status = if report.codex_absent {
        "A pasta .codex\\pets não foi encontrada."
    } else {
        "A pasta .codex\\pets foi verificada sem ser modificada."
    };
    format!(
        "Pets adicionados do catálogo embutido: {}\n\
         Pets adicionados do Codex: {}\n\
         Pets já existentes ignorados: {}\n\
         Pets inválidos ignorados: {}\n\
         {}\n\n\
         Destino:\n{}",
        report.added_embedded,
        report.added_codex,
        report.skipped_existing,
        report.skipped_invalid,
        codex_status,
        report.destination.display()
    )
}

#[cfg(windows)]
fn show_message(title: &str, message: &str, is_error: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MessageBoxW,
    };

    let title = wide(title);
    let message = wide(message);
    let icon = if is_error {
        MB_ICONERROR
    } else {
        MB_ICONINFORMATION
    };

    // SAFETY: both UTF-16 strings are zero-terminated and valid during the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | icon,
        );
    }
}

#[cfg(not(windows))]
fn show_message(_title: &str, message: &str, is_error: bool) {
    if is_error {
        eprintln!("{message}");
    } else {
        println!("{message}");
    }
}

#[cfg(windows)]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
