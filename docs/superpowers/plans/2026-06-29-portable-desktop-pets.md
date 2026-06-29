# Portable Desktop Pets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce a self-contained Windows 10/11 desktop-pet executable with Rainbow Hope bundled by default, autonomous animation driven by CPU/RAM pressure, multiple movement modes, local pet import, and portable persistence.

**Architecture:** A native Rust binary owns one coordinator process and multiple layered Win32 pet windows. Pure modules cover configuration, atlas decoding, mood selection, movement geometry, and import validation; a Windows-only shell handles rendering, menus, metrics, launch coordination, and startup integration. Release packaging copies the executable and validated Rainbow Hope files into a portable folder.

**Tech Stack:** Rust 2024, `windows` Win32 bindings, `serde_json`, `image` WebP decoding, `zip`, and Rust integration tests.

---

### Task 1: Project scaffold and portable configuration

**Files:**
- Create: `desktop-pets/Cargo.toml`
- Create: `desktop-pets/src/lib.rs`
- Create: `desktop-pets/src/main.rs`
- Create: `desktop-pets/src/config.rs`
- Test: `desktop-pets/tests/config_contract.rs`
- Test: `desktop-pets/tests/config_recovery.rs`

- [ ] Write tests for the Rainbow Hope default, size limits, JSON round-trip, missing-file creation, invalid-file backup, and temporary-file cleanup.
- [ ] Run the tests and confirm failure because the API does not exist.
- [ ] Implement the typed schema, validation, recovery, and replacement write.
- [ ] Run the configuration tests and the complete suite.
- [ ] Commit only the task files.

### Task 2: Atlas and animation contract

**Files:**
- Create: `desktop-pets/src/pet/mod.rs`
- Create: `desktop-pets/src/pet/atlas.rs`
- Test: `desktop-pets/tests/atlas_contract.rs`

- [ ] Write tests for the 1536x1872 grid, all nine row/frame counts, wrapping, manifest safety, and the repository Rainbow Hope atlas.
- [ ] Run the tests and confirm failure because the pet module does not exist.
- [ ] Implement manifest loading, path validation, WebP decoding, cropping, premultiplied BGRA conversion, and alpha masks.
- [ ] Run the atlas tests and complete suite.
- [ ] Commit only the task files.

### Task 3: Adaptive mood engine

**Files:**
- Create: `desktop-pets/src/behavior/mod.rs`
- Create: `desktop-pets/src/behavior/mood.rs`
- Test: `desktop-pets/tests/mood_contract.rs`

- [ ] Write deterministic tests for CPU/RAM weighting, warm-up thresholds, rolling baseline, overload safeguards, hysteresis, dwell, calm fallback, and random waving/jumping.
- [ ] Run the tests and confirm the missing module failure.
- [ ] Implement a pure state machine with named timing and score constants plus a seeded PRNG.
- [ ] Run mood tests and the complete suite.
- [ ] Commit only the task files.

### Task 4: Movement geometry

**Files:**
- Create: `desktop-pets/src/behavior/movement.rs`
- Test: `desktop-pets/tests/movement_contract.rs`

- [ ] Write tests for fixed, bottom strip, horizontal line, whole-screen, cross-monitor, semi-fixed A/B, and clamping after monitor changes.
- [ ] Run the tests and confirm the missing movement API failure.
- [ ] Implement monitor-independent geometry and direction selection.
- [ ] Run movement tests and the complete suite.
- [ ] Commit only the task files.

### Task 5: Portable pet library and import

**Files:**
- Create: `desktop-pets/src/library.rs`
- Test: `desktop-pets/tests/library_contract.rs`

- [ ] Write tests for discovery, folder import, ZIP import, traversal, duplicate entries, archive limits, staged replacement, and rollback.
- [ ] Run the tests and confirm the missing library API failure.
- [ ] Implement validation and atomic staged installation.
- [ ] Run library tests and the complete suite.
- [ ] Commit only the task files.

### Task 6: Native Windows application

**Files:**
- Create: `desktop-pets/src/windows/mod.rs`
- Create: `desktop-pets/src/windows/metrics.rs`
- Create: `desktop-pets/src/windows/render.rs`
- Create: `desktop-pets/src/windows/app.rs`
- Modify: `desktop-pets/src/main.rs`
- Test: `desktop-pets/tests/windows_contract.rs`

- [ ] Write pure contract tests for metric deltas, frame timing, command routing, and launch scope IDs.
- [ ] Run the tests and confirm missing Windows-shell APIs.
- [ ] Implement total CPU/RAM sampling, a borderless layered always-on-top window, animation timer, alpha hit testing, dragging, right-click menus, pet/size/movement commands, config saves, and one coordinator mutex.
- [ ] Run contract tests, complete suite, and a debug Windows build.
- [ ] Commit only the task files.

### Task 7: Portable package and documentation

**Files:**
- Create: `desktop-pets/build-portable.ps1`
- Create: `desktop-pets/README.md`
- Modify: `.gitignore`
- Output: `dist/DesktopPets/`

- [ ] Add a packaging script that runs release checks, builds a statically linked executable, initializes `config.json`, and copies Rainbow Hope.
- [ ] Run the script and inspect the exact release tree.
- [ ] Launch the packaged executable and verify that it remains running with a visible window, then close it normally.
- [ ] Copy the release directory to a second path and repeat the smoke launch.
- [ ] Commit only source and documentation files; do not commit `dist`.

### Task 8: Final verification

**Files:**
- Review all files changed above.

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo clippy --all-targets -- -D warnings`.
- [ ] Run `cargo test`.
- [ ] Run `cargo build --release`.
- [ ] Re-run the portable packaging and smoke checks.
- [ ] Compare every acceptance criterion in the design against implementation and recorded evidence.
- [ ] Use `superpowers:finishing-a-development-branch`; because the user explicitly approved `main`, retain commits there and report any manual Windows 10/11 checks that still require a second machine.
