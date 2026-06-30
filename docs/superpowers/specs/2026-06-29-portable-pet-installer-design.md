# Portable Codex Pet Installer - Design

## Summary

Build a portable Windows 10/11 application that installs pet ZIP packages from
this repository into the current user's Codex pet directory. The distributed
application is a single executable inside a ZIP at the repository root. It
requires no Python installation, administrator rights, network access, or
Windows installer.

The implementation uses Python and Tkinter, packaged into one executable with
PyInstaller. Python, pip, and PyInstaller are build-time requirements only.

## Scope

The installer:

- runs on Windows 10 and Windows 11;
- accepts a downloaded pet `.zip`;
- validates the package before writing files;
- installs into `%USERPROFILE%\.codex\pets\<pet-id>`;
- handles an existing pet through optional size verification, replacement,
  installation as a renamed copy, or cancellation;
- presents all user-facing text in Portuguese;
- does not download pets or modify the repository.

macOS, Linux, automatic downloads, pet creation, atlas editing, and digital
code signing are outside this scope.

## Release Layout

The repository root contains:

```text
Instalador-Pets-Windows.zip
```

The archive contains:

```text
Instalador-Pets-Windows/
  Instalador-Pets.exe
  LEIA-ME.txt
```

The repository also contains auditable build inputs:

```text
installer/
  pet_installer/
  tests/
  build.ps1
  Instalador-Pets.spec
  requirements-build.txt
```

Build environments use an isolated Python virtual environment. Build artifacts
and virtual environments remain ignored by Git.

## User Interface

The main window is a compact Windows utility with:

- a command to select a pet ZIP;
- the selected file name;
- the detected pet display name and ID after validation;
- the destination directory;
- a primary install command;
- a cancel/close command;
- a concise status area.

The file picker accepts `.zip` files. Validation errors are shown before the
install command is enabled.

When the destination ID already exists, the conflict dialog offers:

- **Verificar se e identico**;
- **Atualizar**;
- **Instalar como copia**;
- **Cancelar**.

The application explains that size verification does not compare file content
and can classify different files with equal sizes as identical.

## Package Validation

A valid package contains one top-level pet directory with `pet.json` and the
relative spritesheet referenced by `spritesheetPath`. Current repository
packages reference `spritesheet.webp`.

Before installation, the application:

- rejects empty archives;
- rejects absolute paths, parent traversal, drive-qualified paths, links, and
  duplicate archive entries;
- rejects archives with more than 100 files or more than 100 MiB of
  uncompressed data;
- requires one top-level directory;
- parses `pet.json` as UTF-8 JSON;
- requires `id`, `displayName`, and `spritesheetPath`;
- accepts IDs matching lowercase letters, digits, and internal hyphens, up to
  64 characters;
- requires a relative spritesheet path inside the same pet directory;
- requires the referenced spritesheet to exist;
- copies regular package files only.

Validation never extracts arbitrary archive paths directly into the
destination.

## Installation Flow

### New pet

1. Read and validate the ZIP.
2. Copy accepted files into a staging directory under `.codex/pets`.
3. Atomically rename the staging directory to `<pet-id>`.
4. Report the installed pet name and destination.

### Existing pet

If `%USERPROFILE%\.codex\pets\<pet-id>` exists, no files are changed until the
user chooses an action.

#### Optional size verification

Verification uses exact logical file lengths expressed in bits:

```text
size_in_bits = size_in_bytes * 8
```

It deliberately does not hash or compare file content.

1. Build a relative-path-to-size map for all regular files in the incoming
   package.
2. Build the same map for all regular files in the installed pet directory.
3. Compare the sum of all file sizes in bits.
4. If the totals differ, classify the pets as distinct and stop verification.
5. If the totals match, compare the relative path sets.
6. If the path sets match, compare each corresponding file size in bits.
7. Only matching totals, paths, and per-file sizes classify the pet as already
   present.

When identical, the application reports that the pet is already present and
cancels installation. When distinct, it reports the difference and offers
**Atualizar**, **Instalar como copia**, or **Cancelar**.

#### Update

1. Stage the new package.
2. Rename the existing destination to a temporary backup.
3. Rename the staged package to the final destination.
4. Remove the backup after success.
5. If replacement fails, restore the backup and report the error.

#### Install as copy

1. Ask for a new display name.
2. Normalize the name into a lowercase hyphenated ID.
3. Reject an empty/invalid ID or an ID whose destination already exists.
4. Rewrite `id` and `displayName` in the staged `pet.json`.
5. Install into `.codex/pets/<new-id>`.

The description and spritesheet path remain unchanged.

## Error Handling

- Failed validation leaves existing pets untouched.
- Temporary staging and backup directories use unique names.
- A failed update restores the original pet whenever the original directory
  was successfully moved.
- Incomplete staging directories are removed after handled failures.
- Error dialogs use concise Portuguese messages and include the affected file
  or directory when useful.
- The application does not request elevation. Permission failures are reported
  with the destination path.

## Documentation

The repository provides two separate guides:

- `INSTRUCOES-INSTALACAO-MANUAL.md`: current ZIP extraction process for
  Windows, macOS, and Linux;
- `INSTRUCOES-INSTALADOR.md`: Windows portable installer workflow, conflict
  options, size-verification behavior, SmartScreen warning, and troubleshooting.

`INSTRUCOES.md` becomes a short index linking to both methods. `README.md`
links to the installer ZIP and both guides.

The installer guide distinguishes:

- **Runtime requirements:** Windows 10/11 and Codex; Python is not required;
- **Build requirements:** Python, pip, and PyInstaller;
- **ZIP support:** Windows Explorer can extract the installer archive;
- **SmartScreen:** the unsigned executable may require **More info** followed
  by **Run anyway**.

## Build

`build.ps1`:

1. creates or reuses an isolated build virtual environment;
2. installs the pinned build requirements;
3. runs the automated tests;
4. invokes PyInstaller in one-file, windowed mode;
5. runs the executable self-test;
6. assembles `Instalador-Pets-Windows/`;
7. writes `LEIA-ME.txt`;
8. creates `Instalador-Pets-Windows.zip` at the repository root.

The executable includes a noninteractive `--self-test` command used only by the
build and verification workflow.

## Testing

Automated tests cover:

- valid repository-style ZIP packages;
- missing or invalid `pet.json`;
- missing spritesheets;
- unsafe paths, duplicate entries, links, file-count limits, and size limits;
- exact total-size comparison in bits;
- early distinct classification when totals differ;
- relative-path and per-file bit-size comparison when totals match;
- identical-pet cancellation;
- staged installation of a new pet;
- successful update;
- rollback after a simulated replacement failure;
- renamed copies with updated folder, `id`, and `displayName`;
- invalid or conflicting copy names.

Release verification covers:

- the complete automated test suite;
- PyInstaller build success;
- executable `--self-test`;
- launching the GUI without a console window;
- installation of at least one current repository ZIP into an isolated test
  destination;
- inspection of the release ZIP contents;
- verification that the repository worktree contains only intended changes.

## Acceptance Criteria

The feature is complete when:

- `Instalador-Pets-Windows.zip` exists at the repository root;
- its executable launches on Windows 10/11 without Python or administrator
  rights;
- current repository pet ZIPs validate and install correctly;
- existing pets present all four approved conflict actions;
- optional verification follows the approved total-bits-first algorithm;
- renamed copies update their folder, `id`, and `displayName`;
- failed updates preserve the prior installed pet;
- the two installation guides and README links are present;
- tests, build, self-test, and isolated installation verification pass.
