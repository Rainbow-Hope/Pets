use desktop_pets::all_pets_helper::{detect_install_target, HelperError};
use desktop_pets::embedded_pets::embedded_pets;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root")
        .to_path_buf()
}

fn repository_pet_ids() -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let pets_root = repository_root().join("pets");
    collect_pet_ids(&pets_root, &mut ids);
    ids
}

fn collect_pet_ids(directory: &Path, ids: &mut BTreeSet<String>) {
    let manifest = directory.join("pet.json");
    if manifest.is_file() {
        let text = fs::read_to_string(&manifest).expect("manifest text");
        let json: Value = serde_json::from_str(&text).expect("manifest json");
        let id = json["id"].as_str().expect("manifest id");
        assert!(ids.insert(id.to_string()), "duplicate repository pet id {id}");
        return;
    }

    let mut children = fs::read_dir(directory)
        .expect("read pet directory")
        .map(|entry| entry.expect("pet directory entry").path())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        if child.is_dir() {
            collect_pet_ids(&child, ids);
        }
    }
}

#[test]
fn embedded_catalog_contains_each_repository_pet_once_in_id_order() {
    let embedded = embedded_pets();
    let embedded_ids = embedded
        .iter()
        .map(|pet| pet.id.to_string())
        .collect::<Vec<_>>();
    let mut sorted = embedded_ids.clone();
    sorted.sort();
    assert_eq!(embedded_ids, sorted);
    assert_eq!(
        embedded_ids.into_iter().collect::<BTreeSet<_>>(),
        repository_pet_ids()
    );
}

#[test]
fn embedded_catalog_contains_only_manifest_and_referenced_spritesheet_bytes() {
    for pet in embedded_pets() {
        assert_eq!(pet.manifest_name, "pet.json");
        assert!(pet.manifest.starts_with(b"{"));
        assert_eq!(pet.spritesheet_name, "spritesheet.webp");
        assert!(pet.spritesheet.starts_with(b"RIFF"));
        assert!(pet.spritesheet.len() > 1024);
    }
}

fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, bytes).expect("write test file");
}

#[test]
fn detect_install_target_accepts_all_four_edition_executables() {
    for executable in [
        "DesktopPets.exe",
        "DesktopPetsMicro.exe",
        "DesktopPetsNano.exe",
        "DesktopPetsPico.exe",
    ] {
        let temp = TempDir::new().expect("temp dir");
        write_file(&temp.path().join(executable), b"exe");
        write_file(&temp.path().join("config.json"), b"{}");

        let target = detect_install_target(temp.path()).expect("target");

        assert_eq!(target.root, temp.path());
        assert_eq!(target.pets_dir, temp.path().join("pets"));
        assert!(target.pets_dir.is_dir());
    }
}

#[test]
fn detect_install_target_rejects_optional_download_folder_without_edition() {
    let temp = TempDir::new().expect("temp dir");
    write_file(&temp.path().join("AdicionarTodosOsPets.exe"), b"helper");
    write_file(&temp.path().join("LEIA-ME-AUXILIAR.txt"), b"docs");

    let error = detect_install_target(temp.path()).expect_err("not an edition folder");

    assert!(matches!(error, HelperError::MissingEditionExecutable(_)));
    assert!(!temp.path().join("pets").exists());
}
