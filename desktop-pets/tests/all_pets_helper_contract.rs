use desktop_pets::all_pets_helper::{
    EmbeddedPetSource, HelperError, InstallReport, detect_install_target, install_all_pets,
    scan_codex_pet_sources,
};
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
        assert!(
            ids.insert(id.to_string()),
            "duplicate repository pet id {id}"
        );
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

fn repository_package(package: &str) -> PathBuf {
    repository_root().join("pets").join(package)
}

fn copy_package(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination package");
    fs::copy(source.join("pet.json"), destination.join("pet.json")).expect("copy manifest");
    fs::copy(
        source.join("spritesheet.webp"),
        destination.join("spritesheet.webp"),
    )
    .expect("copy spritesheet");
}

fn rename_pet_id(package: &Path, new_id: &str, new_name: &str) {
    let manifest = package.join("pet.json");
    let text = fs::read_to_string(&manifest).expect("read manifest");
    let mut json: Value = serde_json::from_str(&text).expect("manifest json");
    json["id"] = Value::String(new_id.to_string());
    json["displayName"] = Value::String(new_name.to_string());
    fs::write(
        manifest,
        serde_json::to_string_pretty(&json).expect("serialize manifest"),
    )
    .expect("write renamed manifest");
}

fn test_embedded_source(id: &'static str, package: &Path) -> EmbeddedPetSource {
    EmbeddedPetSource {
        id,
        manifest_name: "pet.json",
        manifest: Box::leak(
            fs::read(package.join("pet.json"))
                .expect("manifest")
                .into_boxed_slice(),
        ),
        spritesheet_name: "spritesheet.webp",
        spritesheet: Box::leak(
            fs::read(package.join("spritesheet.webp"))
                .expect("spritesheet")
                .into_boxed_slice(),
        ),
    }
}

#[test]
fn install_all_pets_merges_embedded_and_codex_without_overwriting_existing_ids() {
    let temp = TempDir::new().expect("temp");
    write_file(&temp.path().join("DesktopPets.exe"), b"exe");
    write_file(&temp.path().join("config.json"), b"{}");
    let target = detect_install_target(temp.path()).expect("target");

    let existing = target.pets_dir.join("rainbow-hope");
    copy_package(&repository_package("rainbow-hope"), &existing);

    let codex_root = temp.path().join("codex-pets");
    let codex_package = codex_root.join("codex-only-test");
    copy_package(&repository_package("rainbow-hope"), &codex_package);
    rename_pet_id(&codex_package, "codex-only-test", "Codex Only Test");
    let codex_sources = scan_codex_pet_sources(&codex_root).expect("codex sources");

    let embedded_package = repository_package("zuko-chibi");
    let embedded = [test_embedded_source("zuko-chibi", &embedded_package)];

    let report = install_all_pets(&target, &embedded, &codex_sources).expect("report");

    assert_eq!(
        report,
        InstallReport {
            added_embedded: 1,
            added_codex: 1,
            skipped_existing: 0,
            skipped_invalid: 0,
            codex_absent: false,
            destination: target.pets_dir.clone(),
        }
    );
    assert!(
        target
            .pets_dir
            .join("rainbow-hope")
            .join("pet.json")
            .is_file()
    );
    assert!(
        target
            .pets_dir
            .join("zuko-chibi")
            .join("pet.json")
            .is_file()
    );
    assert!(
        target
            .pets_dir
            .join("codex-only-test")
            .join("pet.json")
            .is_file()
    );
    assert!(codex_package.join("pet.json").is_file());
}

#[test]
fn install_all_pets_counts_existing_and_invalid_sources_and_cleans_staging() {
    let temp = TempDir::new().expect("temp");
    write_file(&temp.path().join("DesktopPets.exe"), b"exe");
    write_file(&temp.path().join("config.json"), b"{}");
    let target = detect_install_target(temp.path()).expect("target");
    copy_package(
        &repository_package("rainbow-hope"),
        &target.pets_dir.join("rainbow-hope"),
    );

    let codex_root = temp.path().join("codex-pets");
    copy_package(
        &repository_package("rainbow-hope"),
        &codex_root.join("rainbow-hope"),
    );
    write_file(
        &codex_root.join("invalid-pet").join("pet.json"),
        br#"{"id":"invalid-pet"}"#,
    );
    let codex_sources = scan_codex_pet_sources(&codex_root).expect("codex sources");
    let embedded = [test_embedded_source(
        "rainbow-hope",
        &repository_package("rainbow-hope"),
    )];

    let report = install_all_pets(&target, &embedded, &codex_sources).expect("report");

    assert_eq!(report.added_embedded, 0);
    assert_eq!(report.added_codex, 0);
    assert_eq!(report.skipped_existing, 2);
    assert_eq!(report.skipped_invalid, 1);
    assert!(
        fs::read_dir(&target.pets_dir)
            .expect("read pets")
            .all(|entry| !entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".helper-"))
    );
}
