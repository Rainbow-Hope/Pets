# All-Pets Helper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `AdicionarTodosOsPets.exe`, an optional portable helper that imports every embedded repository pet and valid Codex pet into the edition folder where the helper is run.

**Architecture:** A Rust build script generates a deterministic embedded pet catalog from the repository `pets/` tree. A shared helper module detects the adjacent Desktop Pets edition, stages embedded/Codex sources, imports them through the existing `PetLibrary`, and returns a count-only report. A small Windows binary displays the report with a native message box, while the package script places the helper in Normal and in one optional light-edition helper folder.

**Tech Stack:** Rust 2024, `serde_json`, existing `PetLibrary`, `windows-sys` message boxes, PowerShell packaging, Cargo integration tests.

---

## File map

- Create `desktop-pets/build.rs`: scans `../pets`, validates manifests/spritesheets, and generates `OUT_DIR/embedded_pets.rs`.
- Modify `desktop-pets/Cargo.toml`: register `AdicionarTodosOsPets` binary.
- Modify `desktop-pets/src/lib.rs`: export `all_pets_helper` and `embedded_pets`.
- Create `desktop-pets/src/embedded_pets.rs`: includes generated static catalog.
- Create `desktop-pets/src/all_pets_helper.rs`: destination detection, import/report logic, Codex-source scanning.
- Create `desktop-pets/src/bin/add_all_pets.rs`: executable entrypoint and Windows message box.
- Modify `desktop-pets/build-portable.ps1`: build/copy helper into Normal and optional light helper folder.
- Create `desktop-pets/package-assets/LEIA-ME-AUXILIAR.txt`: optional-helper instructions.
- Modify `desktop-pets/package-assets/LEIA-ME.txt`: document helper behavior and placement.
- Modify `desktop-pets/package-assets/DIFERENCAS-ENTRE-EDICOES.txt`: document helper distribution and size impact.
- Modify `desktop-pets/tests/package_layout_contract.rs`: assert helper package layout.
- Modify `desktop-pets/tests/package_docs_contract.rs`: assert helper docs.
- Create `desktop-pets/tests/all_pets_helper_contract.rs`: helper behavior contract.
- Modify generated output under `Executar fora do Códex/` by running `desktop-pets/build-portable.ps1`.

---

### Task 1: Embedded catalog contract

**Files:**
- Create: `desktop-pets/tests/all_pets_helper_contract.rs`
- Create later: `desktop-pets/build.rs`
- Create later: `desktop-pets/src/embedded_pets.rs`

- [ ] **Step 1: Write the failing test**

Add this test module to `desktop-pets/tests/all_pets_helper_contract.rs`:

```rust
use desktop_pets::embedded_pets::embedded_pets;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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
    assert_eq!(embedded_ids.into_iter().collect::<BTreeSet<_>>(), repository_pet_ids());
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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml embedded_catalog_contains_each_repository_pet_once_in_id_order
```

Expected: compile failure because `desktop_pets::embedded_pets` does not exist.

- [ ] **Step 3: Implement the minimal generated catalog**

Create `desktop-pets/build.rs` with deterministic scanning of `../pets`, duplicate-id rejection, safe `spritesheetPath` validation, and generated `include_bytes!` entries.

Create `desktop-pets/src/embedded_pets.rs`:

```rust
pub struct EmbeddedPet {
    pub id: &'static str,
    pub manifest_name: &'static str,
    pub manifest: &'static [u8],
    pub spritesheet_name: &'static str,
    pub spritesheet: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/embedded_pets.rs"));
```

Modify `desktop-pets/src/lib.rs`:

```rust
pub mod embedded_pets;
```

The generated file must expose:

```rust
pub fn embedded_pets() -> &'static [EmbeddedPet] {
    EMBEDDED_PETS
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml embedded_catalog
```

Expected: the two embedded-catalog tests pass.

- [ ] **Step 5: Commit**

Run:

```powershell
git add desktop-pets\build.rs desktop-pets\src\embedded_pets.rs desktop-pets\src\lib.rs desktop-pets\tests\all_pets_helper_contract.rs
git commit -m "Add embedded pet catalog"
```

---

### Task 2: Destination detection contract

**Files:**
- Modify: `desktop-pets/tests/all_pets_helper_contract.rs`
- Create later: `desktop-pets/src/all_pets_helper.rs`
- Modify later: `desktop-pets/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Append these tests to `desktop-pets/tests/all_pets_helper_contract.rs`:

```rust
use desktop_pets::all_pets_helper::{detect_install_target, HelperError};
use tempfile::TempDir;

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
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml detect_install_target
```

Expected: compile failure because `all_pets_helper` does not exist.

- [ ] **Step 3: Implement minimal detection**

Create `desktop-pets/src/all_pets_helper.rs` with:

```rust
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
```

Modify `desktop-pets/src/lib.rs`:

```rust
pub mod all_pets_helper;
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml detect_install_target
```

Expected: destination-detection tests pass.

- [ ] **Step 5: Commit**

Run:

```powershell
git add desktop-pets\src\all_pets_helper.rs desktop-pets\src\lib.rs desktop-pets\tests\all_pets_helper_contract.rs
git commit -m "Detect all-pets helper destination"
```

---

### Task 3: Import/report contract

**Files:**
- Modify: `desktop-pets/tests/all_pets_helper_contract.rs`
- Modify later: `desktop-pets/src/all_pets_helper.rs`

- [ ] **Step 1: Write the failing test**

Append these tests to `desktop-pets/tests/all_pets_helper_contract.rs`:

```rust
use desktop_pets::all_pets_helper::{
    install_all_pets, scan_codex_pet_sources, EmbeddedPetSource, InstallReport,
};

fn repository_package(package: &str) -> PathBuf {
    repository_root().join("pets").join(package)
}

fn copy_package(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination package");
    fs::copy(source.join("pet.json"), destination.join("pet.json")).expect("copy manifest");
    fs::copy(source.join("spritesheet.webp"), destination.join("spritesheet.webp"))
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
        manifest: Box::leak(fs::read(package.join("pet.json")).expect("manifest").into_boxed_slice()),
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
    assert!(target.pets_dir.join("rainbow-hope").join("pet.json").is_file());
    assert!(target.pets_dir.join("zuko-chibi").join("pet.json").is_file());
    assert!(target.pets_dir.join("codex-only-test").join("pet.json").is_file());
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
    write_file(&codex_root.join("invalid-pet").join("pet.json"), br#"{"id":"invalid-pet"}"#);
    let codex_sources = scan_codex_pet_sources(&codex_root).expect("codex sources");
    let embedded = [test_embedded_source("rainbow-hope", &repository_package("rainbow-hope"))];

    let report = install_all_pets(&target, &embedded, &codex_sources).expect("report");

    assert_eq!(report.added_embedded, 0);
    assert_eq!(report.added_codex, 0);
    assert_eq!(report.skipped_existing, 2);
    assert_eq!(report.skipped_invalid, 1);
    assert!(fs::read_dir(&target.pets_dir)
        .expect("read pets")
        .all(|entry| !entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .starts_with(".helper-")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml install_all_pets
```

Expected: compile failure because import/report functions do not exist.

- [ ] **Step 3: Implement import/report logic**

Extend `desktop-pets/src/all_pets_helper.rs` with:

- `EmbeddedPetSource` carrying `id`, `manifest_name`, `manifest`, `spritesheet_name`, and `spritesheet`.
- `CodexPetSource { path: PathBuf }`.
- `InstallReport` with the exact fields in the tests.
- `scan_codex_pet_sources(root)` that returns `codex_absent` through an empty source list only when the caller supplies a missing path; it ignores non-directories and symlinks.
- `install_all_pets(target, embedded, codex_sources)` that stages embedded packages under `pets/.helper-<uuid>`, imports with `PetLibrary::import(..., ReplacePolicy::Reject)`, counts `LibraryError::AlreadyExists(_)` as existing, counts other per-pet errors as invalid, and removes helper staging directories after success and failure.

- [ ] **Step 4: Run test to verify it passes**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml all_pets_helper_contract
```

Expected: all helper contract tests pass.

- [ ] **Step 5: Commit**

Run:

```powershell
git add desktop-pets\src\all_pets_helper.rs desktop-pets\tests\all_pets_helper_contract.rs
git commit -m "Import all embedded and Codex pets"
```

---

### Task 4: Helper executable and package layout contract

**Files:**
- Modify: `desktop-pets/Cargo.toml`
- Create: `desktop-pets/src/bin/add_all_pets.rs`
- Modify: `desktop-pets/tests/package_layout_contract.rs`
- Modify later: `desktop-pets/build-portable.ps1`

- [ ] **Step 1: Write the failing layout test**

Modify `desktop-pets/tests/package_layout_contract.rs` so `all_four_portable_packages_have_executable_config_docs_and_rainbow_hope` also asserts:

```rust
assert!(
    output.join("Normal").join("AdicionarTodosOsPets.exe").is_file(),
    "Normal must include the optional all-pets helper"
);
assert!(
    output
        .join("Leves")
        .join("Auxiliar opcional - Todos os Pets")
        .join("AdicionarTodosOsPets.exe")
        .is_file(),
    "light editions must have one separately downloadable helper"
);
assert!(
    output
        .join("Leves")
        .join("Auxiliar opcional - Todos os Pets")
        .join("LEIA-ME-AUXILIAR.txt")
        .is_file(),
    "optional helper folder must include its own instructions"
);
for relative in ["Leves/Micro", "Leves/Nano", "Leves/Pico"] {
    assert!(
        !output.join(relative).join("AdicionarTodosOsPets.exe").exists(),
        "light packages stay small by not bundling the helper directly: {relative}"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml all_four_portable_packages_have_executable_config_docs_and_rainbow_hope
```

Expected: test fails because the helper executable and optional folder are absent.

- [ ] **Step 3: Add binary and packaging**

Modify `desktop-pets/Cargo.toml`:

```toml
[[bin]]
name = "AdicionarTodosOsPets"
path = "src/bin/add_all_pets.rs"
```

Create `desktop-pets/src/bin/add_all_pets.rs` that:

- gets `std::env::current_exe()`;
- calls `detect_install_target(exe_dir)`;
- uses `embedded_pets()` converted to `EmbeddedPetSource`;
- reads Codex source from `%USERPROFILE%\.codex\pets`;
- calls `install_all_pets`;
- displays either an error or report using `MessageBoxW`.

Modify `desktop-pets/build-portable.ps1` to copy:

- `release\AdicionarTodosOsPets.exe` into `Executar fora do Códex\Normal\AdicionarTodosOsPets.exe`;
- `release\AdicionarTodosOsPets.exe` into `Executar fora do Códex\Leves\Auxiliar opcional - Todos os Pets\AdicionarTodosOsPets.exe`;
- `desktop-pets\package-assets\LEIA-ME-AUXILIAR.txt` into the same optional folder.

- [ ] **Step 4: Run package script to verify layout passes**

Run:

```powershell
desktop-pets\build-portable.ps1
```

Expected: release build, fmt, clippy, tests all pass; output packages contain the helper layout.

- [ ] **Step 5: Commit**

Run:

```powershell
git add desktop-pets\Cargo.toml desktop-pets\Cargo.lock desktop-pets\src\bin\add_all_pets.rs desktop-pets\build-portable.ps1 desktop-pets\tests\package_layout_contract.rs "Executar fora do Códex"
git commit -m "Package optional all-pets helper"
```

---

### Task 5: Documentation contract

**Files:**
- Modify: `desktop-pets/tests/package_docs_contract.rs`
- Create: `desktop-pets/package-assets/LEIA-ME-AUXILIAR.txt`
- Modify: `desktop-pets/package-assets/LEIA-ME.txt`
- Modify: `desktop-pets/package-assets/DIFERENCAS-ENTRE-EDICOES.txt`
- Modify generated docs under `Executar fora do Códex/` by rerunning packaging.

- [ ] **Step 1: Write the failing docs test**

Append this test to `desktop-pets/tests/package_docs_contract.rs`:

```rust
#[test]
fn docs_explain_optional_all_pets_helper() {
    let readme = fs::read_to_string(assets().join("LEIA-ME.txt")).expect("readme");
    let differences =
        fs::read_to_string(assets().join("DIFERENCAS-ENTRE-EDICOES.txt")).expect("differences");
    let helper = fs::read_to_string(assets().join("LEIA-ME-AUXILIAR.txt")).expect("helper docs");
    let combined = format!("{readme}\n{differences}\n{helper}").to_lowercase();

    for required in [
        "adicionartodosospets.exe",
        "opcional",
        "mesma pasta",
        ".codex\\pets",
        "não modifica",
        "não substitui",
        "pode apagar",
        "normal já contém",
        "auxiliar opcional - todos os pets",
    ] {
        assert!(combined.contains(required), "missing helper doc phrase: {required}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml docs_explain_optional_all_pets_helper
```

Expected: failure because `LEIA-ME-AUXILIAR.txt` and helper phrases are absent.

- [ ] **Step 3: Write docs**

Create `desktop-pets/package-assets/LEIA-ME-AUXILIAR.txt` with same-folder usage, non-installer behavior, Codex-source read-only behavior, duplicate-skip behavior, and deletion-after-import behavior.

Update `desktop-pets/package-assets/LEIA-ME.txt` with an “AUXILIAR OPCIONAL” section.

Update `desktop-pets/package-assets/DIFERENCAS-ENTRE-EDICOES.txt` with a note that Normal includes the helper and light editions provide it separately so Micro/Nano/Pico remain smaller by default.

- [ ] **Step 4: Run docs test**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path desktop-pets\Cargo.toml docs_explain_optional_all_pets_helper
```

Expected: docs test passes.

- [ ] **Step 5: Rebuild packages and commit docs**

Run:

```powershell
desktop-pets\build-portable.ps1
git add desktop-pets\package-assets\LEIA-ME.txt desktop-pets\package-assets\DIFERENCAS-ENTRE-EDICOES.txt desktop-pets\package-assets\LEIA-ME-AUXILIAR.txt desktop-pets\tests\package_docs_contract.rs "Executar fora do Códex"
git commit -m "Document optional all-pets helper"
```

---

### Task 6: Final verification and publish

**Files:**
- Read/verify: full repository status.
- Modify if measurements change materially: `desktop-pets/package-assets/edition-measurements.json`, `desktop-pets/package-assets/DIFERENCAS-ENTRE-EDICOES.txt`, generated copies under `Executar fora do Códex/`.

- [ ] **Step 1: Run full build and tests**

Run:

```powershell
desktop-pets\build-portable.ps1
```

Expected: release build, `cargo fmt --check`, clippy `-D warnings`, and full `cargo test` pass.

- [ ] **Step 2: Run helper smoke test manually from a temporary package**

Run a temporary smoke script that copies one edition folder, copies the optional helper beside it when using a light edition, sets a synthetic Codex source through the test-only environment override if implemented, runs `AdicionarTodosOsPets.exe`, and verifies new pet folders exist without modifying the source.

Expected: helper exits successfully and imports embedded repository pets plus the synthetic Codex-only pet.

- [ ] **Step 3: Run edition smoke measurement**

Run:

```powershell
desktop-pets\measure-editions.ps1 -WarmupSeconds 3 -SampleSeconds 3 -OutputPath "$env:TEMP\DesktopPets-helper-smoke.json"
```

Expected: all four editions launch and report nonzero window handles.

- [ ] **Step 4: Inspect git status and avoid unrelated files**

Run:

```powershell
git status --short --branch
```

Expected: only intentional helper, docs, package, and plan files are staged/committed. Do not stage `05-Vídeos e gifs/` or `Sem título.png`.

- [ ] **Step 5: Use finishing-a-development-branch**

Read and follow `superpowers:finishing-a-development-branch`. Because the user already approved committing to `main`, verify `origin/main...main`, fetch first, then push only if `origin/main` has not advanced unexpectedly.

Run:

```powershell
git fetch origin main
git rev-list --left-right --count origin/main...main
git push origin main
```

Expected: push succeeds to `main`.

