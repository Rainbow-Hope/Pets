use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const EDITION_EXECUTABLES: &[&str] = &[
    "DesktopPets.exe",
    "DesktopPetsMicro.exe",
    "DesktopPetsNano.exe",
    "DesktopPetsPico.exe",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTarget {
    pub root: PathBuf,
    pub pets_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum HelperError {
    #[error("coloque AdicionarTodosOsPets.exe na mesma pasta de uma edição Desktop Pets: {0}")]
    MissingEditionExecutable(PathBuf),
    #[error("config.json ausente na pasta da edição: {0}")]
    MissingConfig(PathBuf),
    #[error("erro de arquivo: {0}")]
    Io(#[from] io::Error),
}

pub fn detect_install_target(exe_dir: &Path) -> Result<InstallTarget, HelperError> {
    let has_edition = EDITION_EXECUTABLES
        .iter()
        .any(|name| exe_dir.join(name).is_file());
    if !has_edition {
        return Err(HelperError::MissingEditionExecutable(exe_dir.to_path_buf()));
    }
    if !exe_dir.join("config.json").is_file() {
        return Err(HelperError::MissingConfig(exe_dir.to_path_buf()));
    }

    let pets_dir = exe_dir.join("pets");
    fs::create_dir_all(&pets_dir)?;
    Ok(InstallTarget {
        root: exe_dir.to_path_buf(),
        pets_dir,
    })
}
