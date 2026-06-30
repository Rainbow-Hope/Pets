use serde_json::Value;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
struct PetEntry {
    id: String,
    manifest: PathBuf,
    spritesheet: PathBuf,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let repository_root = manifest_dir.parent().expect("repository root");
    let pets_root = repository_root.join("pets");

    println!("cargo:rerun-if-changed={}", pets_root.display());

    let mut entries = Vec::new();
    collect_pet_entries(&pets_root, &mut entries);
    entries.sort_by(|left, right| left.id.cmp(&right.id));

    let mut seen = BTreeSet::new();
    for entry in &entries {
        if !seen.insert(entry.id.clone()) {
            panic!("duplicate embedded pet id: {}", entry.id);
        }
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("out dir"));
    fs::write(out_dir.join("embedded_pets.rs"), render_catalog(&entries))
        .expect("write embedded pet catalog");
}

fn collect_pet_entries(directory: &Path, entries: &mut Vec<PetEntry>) {
    let manifest = directory.join("pet.json");
    if manifest.is_file() {
        entries.push(read_entry(directory, &manifest));
        return;
    }

    let mut children = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read pets directory {}: {error}", directory.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("read pets directory entry: {error}"))
                .path()
        })
        .collect::<Vec<_>>();
    children.sort();

    for child in children {
        if child.is_dir() {
            collect_pet_entries(&child, entries);
        }
    }
}

fn read_entry(package_dir: &Path, manifest: &Path) -> PetEntry {
    println!("cargo:rerun-if-changed={}", manifest.display());

    let text = fs::read_to_string(manifest)
        .unwrap_or_else(|error| panic!("read manifest {}: {error}", manifest.display()));
    let json: Value = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("parse manifest {}: {error}", manifest.display()));

    let id = json["id"]
        .as_str()
        .unwrap_or_else(|| panic!("manifest {} is missing string id", manifest.display()))
        .to_string();
    let spritesheet_path = json["spritesheetPath"].as_str().unwrap_or_else(|| {
        panic!(
            "manifest {} is missing string spritesheetPath",
            manifest.display()
        )
    });
    if !is_safe_relative_path(spritesheet_path) {
        panic!(
            "manifest {} has unsafe spritesheetPath: {}",
            manifest.display(),
            spritesheet_path
        );
    }

    let spritesheet = package_dir.join(spritesheet_path);
    if !spritesheet.is_file() {
        panic!("missing spritesheet for {id}: {}", spritesheet.display());
    }
    println!("cargo:rerun-if-changed={}", spritesheet.display());

    PetEntry {
        id,
        manifest: manifest.to_path_buf(),
        spritesheet,
    }
}

fn is_safe_relative_path(path: &str) -> bool {
    let mut has_component = false;
    for component in Path::new(path).components() {
        match component {
            Component::Normal(_) => has_component = true,
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => return false,
        }
    }
    has_component
}

fn render_catalog(entries: &[PetEntry]) -> String {
    let mut output = String::from(
        "static EMBEDDED_PETS: &[EmbeddedPet] = &[\n",
    );
    for entry in entries {
        output.push_str("    EmbeddedPet {\n");
        output.push_str(&format!("        id: {:?},\n", entry.id));
        output.push_str("        manifest_name: \"pet.json\",\n");
        output.push_str(&format!(
            "        manifest: include_bytes!(r#\"{}\"#),\n",
            entry.manifest.display()
        ));
        output.push_str("        spritesheet_name: \"spritesheet.webp\",\n");
        output.push_str(&format!(
            "        spritesheet: include_bytes!(r#\"{}\"#),\n",
            entry.spritesheet.display()
        ));
        output.push_str("    },\n");
    }
    output.push_str("];\n\n");
    output.push_str(
        "pub fn embedded_pets() -> &'static [EmbeddedPet] {\n    EMBEDDED_PETS\n}\n",
    );
    output
}
