use crate::pet::{Atlas, PetError, PetManifest};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

pub const MAX_ARCHIVE_ENTRIES: usize = 128;
pub const MAX_PACKAGE_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacePolicy {
    Reject,
    Replace,
}

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("pet library I/O error: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Pet(#[from] PetError),
    #[error("could not read ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("unsupported import source: {0}")]
    UnsupportedSource(PathBuf),
    #[error("unsafe path in archive: {0}")]
    UnsafeArchivePath(String),
    #[error("archive contains duplicate entry: {0}")]
    DuplicateArchiveEntry(String),
    #[error("archive contains too many entries")]
    TooManyEntries,
    #[error("pet package exceeds the 32 MiB safety limit")]
    PackageTooLarge,
    #[error("pet package must contain exactly one pet.json")]
    InvalidPackageLayout,
    #[error("pet package contains files outside its pet directory")]
    FilesOutsidePackage,
    #[error("pet already exists: {0}")]
    AlreadyExists(String),
    #[error("pet package contains a symbolic link: {0}")]
    SymbolicLink(PathBuf),
    #[error("pet spritesheet is missing: {0}")]
    MissingSpritesheet(PathBuf),
}

#[derive(Debug, Clone)]
pub struct PetLibrary {
    root: PathBuf,
}

impl PetLibrary {
    pub fn open(root: &Path) -> Result<Self, LibraryError> {
        fs::create_dir_all(root)?;
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn discover(&self) -> Result<Vec<PetManifest>, LibraryError> {
        let mut pets = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() || entry.file_name().to_string_lossy().starts_with('.')
            {
                continue;
            }
            let manifest_path = entry.path().join("pet.json");
            if manifest_path.is_file() {
                let manifest = PetManifest::load(&manifest_path)?;
                validate_package(entry.path().as_path(), &manifest)?;
                pets.push(manifest);
            }
        }
        pets.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(pets)
    }

    pub fn import(
        &self,
        source: &Path,
        replace: ReplacePolicy,
    ) -> Result<PetManifest, LibraryError> {
        let staging = self.root.join(format!(".import-{}", Uuid::new_v4()));
        fs::create_dir(&staging)?;
        let result = self.import_staged(source, replace, &staging);
        if staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        result
    }

    fn import_staged(
        &self,
        source: &Path,
        replace: ReplacePolicy,
        staging: &Path,
    ) -> Result<PetManifest, LibraryError> {
        if source.is_dir() {
            let mut total_bytes = 0;
            copy_directory(source, staging, &mut total_bytes)?;
        } else if source
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
        {
            extract_zip(source, staging)?;
        } else {
            return Err(LibraryError::UnsupportedSource(source.to_path_buf()));
        }

        let package = locate_package(staging)?;
        ensure_no_files_outside(staging, &package)?;
        let manifest = PetManifest::load(&package.join("pet.json"))?;
        validate_package(&package, &manifest)?;

        let destination = self.root.join(&manifest.id);
        if destination.exists() && replace == ReplacePolicy::Reject {
            return Err(LibraryError::AlreadyExists(manifest.id));
        }

        let backup = self
            .root
            .join(format!(".backup-{}-{}", manifest.id, Uuid::new_v4()));
        if destination.exists() {
            fs::rename(&destination, &backup)?;
        }
        if let Err(error) = fs::rename(&package, &destination) {
            if backup.exists() {
                let _ = fs::rename(&backup, &destination);
            }
            return Err(error.into());
        }
        if backup.exists() {
            fs::remove_dir_all(backup)?;
        }
        Ok(manifest)
    }
}

fn validate_package(directory: &Path, manifest: &PetManifest) -> Result<(), LibraryError> {
    manifest.validate()?;
    let spritesheet = directory.join(&manifest.spritesheet_path);
    if !spritesheet.is_file() {
        return Err(LibraryError::MissingSpritesheet(spritesheet));
    }
    Atlas::load(&spritesheet)?;
    Ok(())
}

fn copy_directory(
    source: &Path,
    destination: &Path,
    total_bytes: &mut u64,
) -> Result<(), LibraryError> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if file_type.is_symlink() {
            return Err(LibraryError::SymbolicLink(source_path));
        }
        if file_type.is_dir() {
            copy_directory(&source_path, &destination_path, total_bytes)?;
        } else if file_type.is_file() {
            *total_bytes = total_bytes.saturating_add(entry.metadata()?.len());
            if *total_bytes > MAX_PACKAGE_BYTES {
                return Err(LibraryError::PackageTooLarge);
            }
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}

fn extract_zip(source: &Path, staging: &Path) -> Result<(), LibraryError> {
    let file = fs::File::open(source)?;
    let mut archive = zip::ZipArchive::new(file)?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(LibraryError::TooManyEntries);
    }

    let mut names = HashSet::new();
    let mut total_bytes = 0_u64;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let raw_name = entry.name().to_owned();
        let Some(relative) = entry.enclosed_name() else {
            return Err(LibraryError::UnsafeArchivePath(raw_name));
        };
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(LibraryError::SymbolicLink(relative.to_path_buf()));
        }
        let normalized = relative.to_string_lossy().replace('\\', "/").to_lowercase();
        if !names.insert(normalized) {
            return Err(LibraryError::DuplicateArchiveEntry(raw_name));
        }
        total_bytes = total_bytes.saturating_add(entry.size());
        if total_bytes > MAX_PACKAGE_BYTES {
            return Err(LibraryError::PackageTooLarge);
        }

        let output = staging.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(output)?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output_file = fs::File::create(output)?;
        io::copy(&mut entry.take(MAX_PACKAGE_BYTES + 1), &mut output_file)?;
    }
    Ok(())
}

fn locate_package(staging: &Path) -> Result<PathBuf, LibraryError> {
    let mut manifests = Vec::new();
    collect_manifests(staging, 0, &mut manifests)?;
    if manifests.len() != 1 {
        return Err(LibraryError::InvalidPackageLayout);
    }
    manifests[0]
        .parent()
        .map(Path::to_path_buf)
        .ok_or(LibraryError::InvalidPackageLayout)
}

fn collect_manifests(
    directory: &Path,
    depth: usize,
    manifests: &mut Vec<PathBuf>,
) -> Result<(), LibraryError> {
    if depth > 3 {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_manifests(&entry.path(), depth + 1, manifests)?;
        } else if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case("pet.json")
        {
            manifests.push(entry.path());
        }
    }
    Ok(())
}

fn ensure_no_files_outside(staging: &Path, package: &Path) -> Result<(), LibraryError> {
    fn visit(directory: &Path, package: &Path) -> Result<(), LibraryError> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                visit(&entry.path(), package)?;
            } else if !entry.path().starts_with(package) {
                return Err(LibraryError::FilesOutsidePackage);
            }
        }
        Ok(())
    }
    visit(staging, package)
}
