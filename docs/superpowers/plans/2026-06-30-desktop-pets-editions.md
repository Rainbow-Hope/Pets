# Desktop Pets Normal and Light Editions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the native portable Desktop Pets application and publish independent Normal, Micro, Nano, and Pico packages under `Executar fora do Códex/` on `main`.

**Architecture:** One Rust library contains the shared configuration, atlas, mood, movement, import, and Win32 shell. Four small binary entry points pass immutable `EditionProfile` values into the shell; profile gates control scheduling, instance limits, and menu capabilities. A PowerShell packager builds into a local non-OneDrive target directory, copies Rainbow Hope and plain-text documentation, and creates the four tracked portable folders.

**Tech Stack:** Rust 2024, `windows-sys` Win32 bindings, WebP through `image`, `serde_json`, `zip`, PowerShell packaging, Git/GitHub.

---

### Task 1: Stabilize the native Normal application

**Files:**
- Modify: `desktop-pets/Cargo.toml`
- Modify: `desktop-pets/src/windows/app.rs`
- Modify: `desktop-pets/src/windows/metrics.rs`
- Modify: `desktop-pets/src/windows/mod.rs`
- Modify: `desktop-pets/src/main.rs`
- Test: `desktop-pets/tests/windows_contract.rs`

- [ ] **Step 1: Extend the failing native contract**

Add assertions that CPU deltas reject invalid counters, the directory scope remains stable, all six movement commands map correctly, and the shell exposes a successful compile-time entry point.

- [ ] **Step 2: Verify the contract fails or the Windows shell fails to compile**

Run:

```powershell
$env:CARGO_TARGET_DIR="$env:LOCALAPPDATA\DesktopPetsBuild"
cargo test --manifest-path desktop-pets/Cargo.toml --test windows_contract
cargo check --manifest-path desktop-pets/Cargo.toml --all-targets
```

Expected: the new entry-point assertion or current Win32 shell compilation fails before stabilization.

- [ ] **Step 3: Complete the Normal shell**

Keep one coordinator per executable directory, create layered topmost tool windows, decode/render premultiplied BGRA frames, hit-test transparent pixels, drag with the left button, create another pet on repeated launch, expose pet/size/movement/import/startup/close menus, persist each instance, and sample total CPU/RAM.

- [ ] **Step 4: Verify debug compilation and tests**

Run both commands from Step 2. Expected: PASS with no warnings.

- [ ] **Step 5: Commit**

Stage only `desktop-pets/` source, lockfile, and tests. Preserve unrelated repository files.

### Task 2: Add explicit edition profiles

**Files:**
- Create: `desktop-pets/src/edition.rs`
- Create: `desktop-pets/src/bin/micro.rs`
- Create: `desktop-pets/src/bin/nano.rs`
- Create: `desktop-pets/src/bin/pico.rs`
- Modify: `desktop-pets/src/lib.rs`
- Modify: `desktop-pets/src/main.rs`
- Modify: `desktop-pets/src/windows/mod.rs`
- Modify: `desktop-pets/src/windows/app.rs`
- Test: `desktop-pets/tests/edition_contract.rs`

- [ ] **Step 1: Write a failing profile matrix test**

The test must require:

```rust
assert_eq!(Edition::Normal.profile().max_windows, None);
assert_eq!(Edition::Micro.profile().max_windows, Some(4));
assert_eq!(Edition::Nano.profile().max_windows, Some(1));
assert_eq!(Edition::Pico.profile().max_windows, Some(1));
assert!(Edition::Normal.profile().can_import);
assert!(Edition::Micro.profile().can_import);
assert!(!Edition::Nano.profile().can_import);
assert!(!Edition::Pico.profile().can_import);
assert!(Edition::Nano.profile().can_move);
assert!(!Edition::Pico.profile().can_move);
assert_eq!(Edition::Normal.profile().frame_interval_ms, 100);
assert_eq!(Edition::Micro.profile().frame_interval_ms, 200);
assert_eq!(Edition::Nano.profile().metric_interval_ms, 5_000);
assert_eq!(Edition::Pico.profile().metric_interval_ms, 8_000);
```

- [ ] **Step 2: Run the test and confirm the missing `edition` module failure**

Run:

```powershell
cargo test --manifest-path desktop-pets/Cargo.toml --test edition_contract
```

- [ ] **Step 3: Implement `Edition` and `EditionProfile`**

Define immutable profiles with fields for executable label, coordinator suffix, frame interval, metric interval, maximum windows, import, movement, startup, new-pet, and close-all capabilities.

- [ ] **Step 4: Connect four entry points**

`DesktopPets.exe`, `DesktopPetsMicro.exe`, `DesktopPetsNano.exe`, and `DesktopPetsPico.exe` call the same shell with different profiles. Include the edition suffix in class and mutex names so editions never coordinate with each other.

- [ ] **Step 5: Gate menus and instance behavior**

Absent capabilities must not create menu entries. Micro rejects a fifth window; Nano and Pico keep one window and repeated launch only surfaces the existing window.

- [ ] **Step 6: Verify all profile and existing tests**

Run `cargo test --manifest-path desktop-pets/Cargo.toml`. Expected: every profile and shared contract passes.

- [ ] **Step 7: Commit**

Commit the edition module, binaries, shell integration, and test.

### Task 3: Remove duplicated atlas alpha storage

**Files:**
- Modify: `desktop-pets/src/pet/atlas.rs`
- Modify: `desktop-pets/src/windows/app.rs`
- Modify: `desktop-pets/tests/atlas_contract.rs`

- [ ] **Step 1: Write a failing memory-representation test**

Require `Frame::alpha_at(x, y)` to read the fourth byte of `premultiplied_bgra` and remove the public full-frame `alpha` vector.

- [ ] **Step 2: Run `atlas_contract` and confirm the new API is absent**

- [ ] **Step 3: Implement alpha-byte access and update scaling/hit testing**

Keep only premultiplied BGRA per decoded frame. The current scaled window may retain one compact alpha mask.

- [ ] **Step 4: Run atlas, Windows, and full tests**

- [ ] **Step 5: Commit**

Commit only atlas, renderer, and test changes.

### Task 4: Create user documentation

**Files:**
- Create: `desktop-pets/package-assets/LEIA-ME.txt`
- Create: `desktop-pets/package-assets/DIFERENCAS-ENTRE-EDICOES.txt`
- Test: `desktop-pets/tests/package_docs_contract.rs`

- [ ] **Step 1: Write a failing document contract**

Require UTF-8 files, the opening hierarchy `Normal > Micro > Nano > Pico`, an index, all four detailed edition headings, the words “não é instalador”, autonomy requirements, and resource-measurement labels.

- [ ] **Step 2: Run the test and confirm both files are missing**

- [ ] **Step 3: Write `LEIA-ME.txt`**

Explain direct execution, required adjacent files, supported Windows versions, menu controls, pet-folder copying, and troubleshooting.

- [ ] **Step 4: Write `DIFERENCAS-ENTRE-EDICOES.txt`**

Order content as direct summary, index, compact table, detailed editions, feature differences, measured results, and selection recommendation.

- [ ] **Step 5: Run the document test**

- [ ] **Step 6: Commit**

Commit the two source documents and contract test.

### Task 5: Build the four tracked portable packages

**Files:**
- Create: `desktop-pets/build-portable.ps1`
- Modify: `.gitignore`
- Create/update: `Executar fora do Códex/`
- Test: `desktop-pets/tests/package_layout_contract.rs`

- [ ] **Step 1: Write a failing package-layout test**

Require the Normal folder and `Leves/Micro`, `Leves/Nano`, and `Leves/Pico`; require the correct executable, `config.json`, both text files, `pets/rainbow-hope/pet.json`, and `spritesheet.webp` in every edition.

- [ ] **Step 2: Run the layout test and confirm the output tree is absent**

- [ ] **Step 3: Implement the packager**

The script sets:

```powershell
$env:CARGO_TARGET_DIR = Join-Path $env:LOCALAPPDATA 'DesktopPetsBuild'
```

It runs format, lint, tests, four release binary builds, recreates only the known edition output folders, writes default configuration, and copies Rainbow Hope plus both documents. It never creates an installer.

- [ ] **Step 4: Run the packager**

Expected: all four independent folders exist under `Executar fora do Códex/`.

- [ ] **Step 5: Run the package-layout test**

- [ ] **Step 6: Commit**

Commit the script and complete runnable output tree, including `.exe` files explicitly requested for GitHub.

### Task 6: Smoke test and measure every edition

**Files:**
- Create: `desktop-pets/measure-editions.ps1`
- Modify: `desktop-pets/package-assets/DIFERENCAS-ENTRE-EDICOES.txt`
- Modify copies under: `Executar fora do Códex/`

- [ ] **Step 1: Implement bounded smoke measurement**

For each exact packaged executable, launch from its package directory, wait until its pet window exists, record executable bytes, package bytes, working-set bytes, and CPU-time delta over a fixed idle interval, then close only that exact process.

- [ ] **Step 2: Run Normal, Micro, Nano, and Pico smoke checks**

Expected: each process remains alive, creates its allowed window, and exits after the close action.

- [ ] **Step 3: Record actual measurements**

Replace no values with estimates. Write the observed machine/date/method and values into the source comparison file, then rerun packaging so every copy matches.

- [ ] **Step 4: Verify hashes of copied documents**

All five copies of each text document must have matching SHA-256 hashes.

- [ ] **Step 5: Commit**

Commit measurement tooling and measured documentation.

### Task 7: Final verification and GitHub publication

**Files:**
- Review every changed source and packaged artifact.

- [ ] **Step 1: Run formatting**

```powershell
cargo fmt --manifest-path desktop-pets/Cargo.toml --check
```

- [ ] **Step 2: Run strict lint**

```powershell
cargo clippy --manifest-path desktop-pets/Cargo.toml --all-targets -- -D warnings
```

- [ ] **Step 3: Run all automated tests**

```powershell
cargo test --manifest-path desktop-pets/Cargo.toml
```

- [ ] **Step 4: Rebuild all releases and packages**

Run `desktop-pets/build-portable.ps1` and confirm four successful release builds.

- [ ] **Step 5: Repeat all four smoke tests**

Confirm each exact packaged executable starts directly without installer or external runtime.

- [ ] **Step 6: Inspect Git state and acceptance criteria**

Confirm unrelated user files remain untouched, all required artifacts are tracked under `Executar fora do Códex/`, no temporary target files are tracked, and the branch is `main`.

- [ ] **Step 7: Finish on the approved main branch**

Use `superpowers:finishing-a-development-branch`, retain the verified commits on `main`, and push `main` to `origin` because the user explicitly requested GitHub publication.
