use crate::library::{LibraryError, PetLibrary, ReplacePolicy};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

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
    #[error(transparent)]
    Library(#[from] LibraryError),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedPetSource {
    pub id: &'static str,
    pub manifest_name: &'static str,
    pub manifest: &'static [u8],
    pub spritesheet_name: &'static str,
    pub spritesheet: &'static [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexPetSource {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexPetSources {
    pub sources: Vec<CodexPetSource>,
    pub absent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallReport {
    pub added_embedded: usize,
    pub added_codex: usize,
    pub skipped_existing: usize,
    pub skipped_invalid: usize,
    pub codex_absent: bool,
    pub destination: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportOutcome {
    Added,
    Existing,
    Invalid,
}

pub fn scan_codex_pet_sources(root: &Path) -> Result<CodexPetSources, HelperError> {
    if !root.exists() {
        return Ok(CodexPetSources {
            sources: Vec::new(),
            absent: true,
        });
    }

    let mut sources = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || !file_type.is_dir() {
            continue;
        }
        let path = entry.path();
        if path.join("pet.json").is_file() {
            sources.push(CodexPetSource { path });
        }
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(CodexPetSources {
        sources,
        absent: false,
    })
}

pub fn install_all_pets(
    target: &InstallTarget,
    embedded: &[EmbeddedPetSource],
    codex: &CodexPetSources,
) -> Result<InstallReport, HelperError> {
    let library = PetLibrary::open(&target.pets_dir)?;
    let mut report = InstallReport {
        added_embedded: 0,
        added_codex: 0,
        skipped_existing: 0,
        skipped_invalid: 0,
        codex_absent: codex.absent,
        destination: target.pets_dir.clone(),
    };

    for pet in embedded {
        match import_embedded_pet(&library, target, pet)? {
            ImportOutcome::Added => report.added_embedded += 1,
            ImportOutcome::Existing => report.skipped_existing += 1,
            ImportOutcome::Invalid => report.skipped_invalid += 1,
        }
    }

    for source in &codex.sources {
        match import_directory_source(&library, &source.path) {
            ImportOutcome::Added => report.added_codex += 1,
            ImportOutcome::Existing => report.skipped_existing += 1,
            ImportOutcome::Invalid => report.skipped_invalid += 1,
        }
    }

    Ok(report)
}

fn import_embedded_pet(
    library: &PetLibrary,
    target: &InstallTarget,
    pet: &EmbeddedPetSource,
) -> Result<ImportOutcome, HelperError> {
    let staging = target.pets_dir.join(format!(".helper-{}", Uuid::new_v4()));
    let package = staging.join("package");
    let result =
        write_embedded_package(&package, pet).map(|()| import_directory_source(library, &package));

    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }

    result
}

fn write_embedded_package(package: &Path, pet: &EmbeddedPetSource) -> Result<(), HelperError> {
    if pet.manifest_name != "pet.json" || !is_simple_file_name(pet.spritesheet_name) {
        return Ok(());
    }
    fs::create_dir_all(package)?;
    fs::write(package.join(pet.manifest_name), pet.manifest)?;
    fs::write(package.join(pet.spritesheet_name), pet.spritesheet)?;
    Ok(())
}

fn import_directory_source(library: &PetLibrary, source: &Path) -> ImportOutcome {
    match library.import(source, ReplacePolicy::Reject) {
        Ok(_) => ImportOutcome::Added,
        Err(LibraryError::AlreadyExists(_)) => ImportOutcome::Existing,
        Err(_) => ImportOutcome::Invalid,
    }
}

fn is_simple_file_name(name: &str) -> bool {
    Path::new(name)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .is_some_and(|file_name| file_name == name)
}
