use desktop_pets::library::{LibraryError, PetLibrary, ReplacePolicy};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root")
        .to_path_buf()
}

fn copy_rainbow_hope(destination: &Path) {
    let source = repo_root().join("pets").join("rainbow-hope");
    fs::create_dir_all(destination).expect("create package");
    fs::copy(source.join("pet.json"), destination.join("pet.json")).expect("copy manifest");
    fs::copy(
        source.join("spritesheet.webp"),
        destination.join("spritesheet.webp"),
    )
    .expect("copy atlas");
}

#[test]
fn folder_import_installs_a_valid_pet_and_discovery_finds_it() {
    let temp = tempfile::tempdir().expect("temp");
    let source = temp.path().join("source");
    let pets = temp.path().join("portable-pets");
    copy_rainbow_hope(&source);
    let library = PetLibrary::open(&pets).expect("library");

    let imported = library
        .import(&source, ReplacePolicy::Reject)
        .expect("import");
    let discovered = library.discover().expect("discover");

    assert_eq!(imported.id, "rainbow-hope");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].id, "rainbow-hope");
    assert!(pets.join("rainbow-hope").join("spritesheet.webp").exists());
}

#[test]
fn repository_zip_imports_without_installation_tools() {
    let temp = tempfile::tempdir().expect("temp");
    let library = PetLibrary::open(&temp.path().join("pets")).expect("library");
    let archive = repo_root().join("downloads").join("rainbow-hope.zip");

    let imported = library
        .import(&archive, ReplacePolicy::Reject)
        .expect("zip import");

    assert_eq!(imported.display_name, "Rainbow Hope");
}

#[test]
fn zip_parent_traversal_is_rejected_without_writing_outside_library() {
    let temp = tempfile::tempdir().expect("temp");
    let archive_path = temp.path().join("malicious.zip");
    let file = fs::File::create(&archive_path).expect("create archive");
    let mut archive = zip::ZipWriter::new(file);
    archive
        .start_file("../escape.txt", SimpleFileOptions::default())
        .expect("start entry");
    archive.write_all(b"escape").expect("write entry");
    archive.finish().expect("finish archive");
    let library_root = temp.path().join("pets");
    let library = PetLibrary::open(&library_root).expect("library");

    let error = library
        .import(&archive_path, ReplacePolicy::Reject)
        .expect_err("must reject traversal");

    assert!(matches!(error, LibraryError::UnsafeArchivePath(_)));
    assert!(!temp.path().join("escape.txt").exists());
}

#[test]
fn invalid_replacement_leaves_the_installed_pet_untouched() {
    let temp = tempfile::tempdir().expect("temp");
    let source = temp.path().join("source");
    let replacement = temp.path().join("replacement");
    let pets = temp.path().join("pets");
    copy_rainbow_hope(&source);
    copy_rainbow_hope(&replacement);
    fs::write(replacement.join("spritesheet.webp"), b"not-webp").expect("break atlas");
    let library = PetLibrary::open(&pets).expect("library");
    library
        .import(&source, ReplacePolicy::Reject)
        .expect("initial import");
    let before = fs::read(pets.join("rainbow-hope").join("spritesheet.webp")).expect("before");

    assert!(
        library
            .import(&replacement, ReplacePolicy::Replace)
            .is_err()
    );
    let after = fs::read(pets.join("rainbow-hope").join("spritesheet.webp")).expect("after");
    assert_eq!(after, before);
}

#[test]
fn duplicate_id_requires_explicit_replacement() {
    let temp = tempfile::tempdir().expect("temp");
    let source = temp.path().join("source");
    copy_rainbow_hope(&source);
    let library = PetLibrary::open(&temp.path().join("pets")).expect("library");
    library
        .import(&source, ReplacePolicy::Reject)
        .expect("initial import");

    let error = library
        .import(&source, ReplacePolicy::Reject)
        .expect_err("duplicate");

    assert!(matches!(error, LibraryError::AlreadyExists(id) if id == "rainbow-hope"));
}
