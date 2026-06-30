# Portable Pet Installer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a portable Windows executable that safely installs repository pet ZIPs into the current user's Codex pet directory.

**Architecture:** A standard-library Python package separates ZIP validation, bit-size comparison, transactional installation, and Tkinter presentation. PyInstaller bundles the package and Tcl/Tk into one windowed executable; a PowerShell build script runs tests, builds, self-tests, and assembles the root release ZIP.

**Tech Stack:** Python 3.14, standard-library `zipfile`/`json`/`pathlib`/`tkinter`, `unittest`, PyInstaller, PowerShell, GitHub Markdown.

---

## File Map

- `installer/pet_installer/package.py`: immutable package metadata and safe ZIP validation/extraction.
- `installer/pet_installer/comparison.py`: exact total-first and per-file bit-size comparison.
- `installer/pet_installer/installation.py`: new install, transactional update, rollback, slug creation, and renamed copy.
- `installer/pet_installer/ui.py`: Tkinter main window and four-action conflict dialog.
- `installer/pet_installer/__main__.py`: GUI entry point and noninteractive self-test.
- `installer/tests/helpers.py`: deterministic pet ZIP fixtures.
- `installer/tests/test_package.py`: archive validation and extraction tests.
- `installer/tests/test_comparison.py`: approved bit-size algorithm tests.
- `installer/tests/test_installation.py`: new, update, rollback, and copy tests.
- `installer/tests/test_entrypoint.py`: self-test process behavior.
- `installer/Instalador-Pets.spec`: one-file, windowed PyInstaller configuration.
- `installer/build.ps1`: isolated repeatable test/build/package pipeline.
- `installer/requirements-build.txt`: pinned build dependency.
- `.gitignore`: build virtual environment and generated artifact exclusions.
- `INSTRUCOES-INSTALACAO-MANUAL.md`: existing manual procedure.
- `INSTRUCOES-INSTALADOR.md`: portable installer procedure and requirements.
- `INSTRUCOES.md`: method index.
- `README.md`: installer download and guide links.
- `Instalador-Pets-Windows.zip`: root release artifact.

### Task 1: Package Model And Safe ZIP Validation

**Files:**
- Create: `installer/pet_installer/__init__.py`
- Create: `installer/pet_installer/package.py`
- Create: `installer/tests/__init__.py`
- Create: `installer/tests/helpers.py`
- Create: `installer/tests/test_package.py`

- [ ] **Step 1: Create fixture helpers and failing validation tests**

`helpers.py` must expose `write_pet_zip(path, *, pet_id="rainbow-hope", display_name="Rainbow Hope", files=None)` and write one top-level directory containing UTF-8 `pet.json` plus `spritesheet.webp`.

Tests must assert:

```python
package = validate_pet_zip(zip_path)
self.assertEqual(package.pet_id, "rainbow-hope")
self.assertEqual(package.display_name, "Rainbow Hope")
self.assertEqual(package.root_name, "rainbow-hope")
self.assertEqual(package.spritesheet_path, "spritesheet.webp")
self.assertEqual(set(package.files), {"pet.json", "spritesheet.webp"})
```

Add separate tests that reject missing `pet.json`, invalid JSON, missing spritesheet, `../escape`, absolute paths, duplicate entries, multiple top-level roots, more than 100 files, and more than 100 MiB declared uncompressed data.

- [ ] **Step 2: Run package tests and verify RED**

Run:

```powershell
python -m unittest installer.tests.test_package -v
```

Expected: import failure because `pet_installer.package` does not exist.

- [ ] **Step 3: Implement package validation**

Implement the exact public API:

```text
PetPackage(archive_path, root_name, pet_id, display_name, spritesheet_path, files)
PackageError(ValueError)
validate_pet_zip(path: str | Path) -> PetPackage
extract_package(package: PetPackage, destination: Path) -> None
```

Validation must normalize ZIP names to POSIX paths, reject drive-qualified,
absolute, parent-traversal, duplicate, encrypted, directory-link, and non-file
entries, enforce the limits, parse the manifest, and retain only paths below
the one top-level directory. Extraction must stream each accepted file to a
caller-provided empty staging directory and never call `ZipFile.extract`.

- [ ] **Step 4: Run package tests and verify GREEN**

Run:

```powershell
python -m unittest installer.tests.test_package -v
```

Expected: all package tests pass.

- [ ] **Step 5: Commit package validation**

```powershell
git add installer/pet_installer installer/tests
git commit -m "Add safe pet package validation"
```

### Task 2: Exact Bit-Size Comparison

**Files:**
- Create: `installer/pet_installer/comparison.py`
- Create: `installer/tests/test_comparison.py`

- [ ] **Step 1: Write failing total-first comparison tests**

Define expected behavior through:

```python
result = compare_size_maps(
    {"pet.json": 80, "spritesheet.webp": 160},
    {"pet.json": 80, "spritesheet.webp": 160},
)
self.assertTrue(result.identical)
self.assertEqual(result.incoming_total_bits, 240)
self.assertEqual(result.installed_total_bits, 240)
self.assertEqual(result.stage, "files")
```

Use bit values in the public API. Add tests proving:

- unequal totals stop at `stage == "total"`;
- equal totals with different path sets are distinct;
- equal totals and paths with one redistributed size are distinct;
- directory maps multiply every byte length by eight;
- nested regular files use forward-slash relative paths.

- [ ] **Step 2: Run comparison tests and verify RED**

```powershell
python -m unittest installer.tests.test_comparison -v
```

Expected: import failure for `pet_installer.comparison`.

- [ ] **Step 3: Implement the comparison API**

Implement the exact public API:

```text
SizeComparison(identical, stage, incoming_total_bits, installed_total_bits, differing_paths)
package_size_map(package: PetPackage) -> dict[str, int]
directory_size_map(directory: Path) -> dict[str, int]
compare_size_maps(incoming_bits, installed_bits) -> SizeComparison
```

`package_size_map` returns
`{path: byte_size * 8 for path, byte_size in package.files.items()}`.

The implementation must return immediately after unequal total sums, compare
path sets only when totals match, and compare individual sizes only when path
sets match.

- [ ] **Step 4: Run comparison tests and verify GREEN**

```powershell
python -m unittest installer.tests.test_comparison -v
```

Expected: all comparison tests pass.

- [ ] **Step 5: Commit bit-size comparison**

```powershell
git add installer/pet_installer/comparison.py installer/tests/test_comparison.py
git commit -m "Add exact pet size comparison"
```

### Task 3: Transactional Installation And Renamed Copies

**Files:**
- Create: `installer/pet_installer/installation.py`
- Create: `installer/tests/test_installation.py`

- [ ] **Step 1: Write failing installation tests**

Tests must use temporary destination roots and assert:

```python
installed = install_new(package, pets_root)
self.assertEqual(installed, pets_root / "rainbow-hope")
self.assertTrue((installed / "pet.json").is_file())
self.assertTrue((installed / "spritesheet.webp").is_file())
```

Add tests for existing-destination rejection, successful update, simulated
rename failure with original restoration, copy name normalization
(`"Rainbow Hope Azul"` to `"rainbow-hope-azul"`), rewritten `id` and
`displayName`, invalid copy names, and conflicting copy IDs.

- [ ] **Step 2: Run installation tests and verify RED**

```powershell
python -m unittest installer.tests.test_installation -v
```

Expected: import failure for `pet_installer.installation`.

- [ ] **Step 3: Implement installation operations**

Implement the exact public API:

```text
InstallationError(RuntimeError)
default_pets_root() -> Path
slugify_display_name(display_name: str) -> str
install_new(package: PetPackage, pets_root: Path) -> Path
update_existing(package: PetPackage, pets_root: Path) -> Path
install_copy(package: PetPackage, pets_root: Path, new_display_name: str) -> Path
```

`default_pets_root` returns `Path.home() / ".codex" / "pets"`.

Use unique staging and backup names inside `pets_root`. `update_existing` must
restore the backup if moving the staged directory into place fails.
`install_copy` must rewrite `pet.json` with UTF-8, two-space indentation, and a
trailing newline after extraction but before the final atomic rename.

- [ ] **Step 4: Run installation tests and verify GREEN**

```powershell
python -m unittest installer.tests.test_installation -v
```

Expected: all installation tests pass.

- [ ] **Step 5: Run the full suite**

```powershell
python -m unittest discover -s installer/tests -v
```

Expected: all tests pass with no tracebacks.

- [ ] **Step 6: Commit transactional installation**

```powershell
git add installer/pet_installer/installation.py installer/tests/test_installation.py
git commit -m "Add transactional pet installation"
```

### Task 4: Tkinter Interface And Conflict Flow

**Files:**
- Create: `installer/pet_installer/ui.py`
- Create: `installer/pet_installer/__main__.py`
- Create: `installer/tests/test_entrypoint.py`

- [ ] **Step 1: Write failing entry-point tests**

Use subprocess tests for:

```python
result = subprocess.run(
    [sys.executable, "-m", "pet_installer", "--self-test"],
    cwd=installer_root,
    env={**os.environ, "PYTHONPATH": str(installer_root)},
    capture_output=True,
    text=True,
)
self.assertEqual(result.returncode, 0)
self.assertIn("SELF_TEST_OK", result.stdout)
```

Also assert an invalid argument returns exit code 2 without opening Tk.

- [ ] **Step 2: Run entry-point tests and verify RED**

```powershell
python -m unittest installer.tests.test_entrypoint -v
```

Expected: module execution failure because `pet_installer.__main__` is absent.

- [ ] **Step 3: Implement the entry point**

`__main__.py` must parse only `--self-test`, print `SELF_TEST_OK` for that
mode, return 2 for unsupported arguments, and otherwise invoke `run_app()`.

- [ ] **Step 4: Implement the GUI**

`ui.py` must define this interface:

```text
ConflictAction(StrEnum): VERIFY, UPDATE, COPY, CANCEL
InstallerApp(root: tk.Tk, pets_root: Path | None = None)
run_app() -> None
```

`run_app` creates `tk.Tk()`, constructs `InstallerApp(root)`, and enters
`root.mainloop()`.

The main window must select ZIPs, validate immediately, display package and
destination metadata, and enable installation only for a valid selection.
The modal conflict dialog must expose **Verificar se e identico**,
**Atualizar**, **Instalar como copia**, and **Cancelar**. Verification
must show the size-only limitation, call the comparison module, stop when
identical, and offer update/copy/cancel when distinct. Copy must request a new
display name and surface validation errors without closing the main window.

- [ ] **Step 5: Run entry-point and full tests**

```powershell
python -m unittest installer.tests.test_entrypoint -v
python -m unittest discover -s installer/tests -v
```

Expected: all tests pass.

- [ ] **Step 6: Manually smoke-test the source GUI**

```powershell
$process = Start-Process python -ArgumentList '-m','pet_installer' -WorkingDirectory '.\installer' -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 3
if ($process.HasExited) { throw "GUI encerrou durante o smoke test" }
Stop-Process -Id $process.Id
```

Expected: process remains alive until intentionally stopped.

- [ ] **Step 7: Commit the GUI**

```powershell
git add installer/pet_installer/ui.py installer/pet_installer/__main__.py installer/tests/test_entrypoint.py
git commit -m "Add portable installer interface"
```

### Task 5: Build And Release Pipeline

**Files:**
- Create: `installer/requirements-build.txt`
- Create: `installer/Instalador-Pets.spec`
- Create: `installer/build.ps1`
- Modify: `.gitignore`

- [ ] **Step 1: Add build exclusions**

Append exact entries:

```gitignore
/installer/.venv-build/
/installer/build/
/installer/dist/
/installer/release/
```

- [ ] **Step 2: Pin and install PyInstaller**

Record the currently installable PyInstaller version in
`requirements-build.txt` using an exact `==` pin. Create
`installer/.venv-build`, install the pin, and record the successful version:

```powershell
python -m venv installer/.venv-build
.\installer\.venv-build\Scripts\python.exe -m pip install --upgrade pip
.\installer\.venv-build\Scripts\python.exe -m pip install -r installer\requirements-build.txt
.\installer\.venv-build\Scripts\pyinstaller.exe --version
```

- [ ] **Step 3: Create the PyInstaller spec**

Configure `Instalador-Pets.exe` with:

```python
exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="Instalador-Pets",
    console=False,
    onefile=True,
    upx=False,
)
```

Include Tcl/Tk through PyInstaller's standard hooks and set no administrator
manifest.

- [ ] **Step 4: Create the reproducible build script**

`build.ps1` must stop on errors, resolve the repository root, create the
virtual environment when absent, install the pinned requirements, run
`unittest`, run PyInstaller with clean output directories, execute
`Instalador-Pets.exe --self-test`, assemble the release folder with
`LEIA-ME.txt`, and create root `Instalador-Pets-Windows.zip`.

- [ ] **Step 5: Run the build**

```powershell
powershell -ExecutionPolicy Bypass -File .\installer\build.ps1
```

Expected: tests pass, PyInstaller exits 0, self-test exits 0, and the root ZIP
is created.

- [ ] **Step 6: Inspect the release archive**

```powershell
$zip = [IO.Compression.ZipFile]::OpenRead((Resolve-Path '.\Instalador-Pets-Windows.zip'))
try { $zip.Entries | Select-Object FullName, Length } finally { $zip.Dispose() }
```

Expected entries:

```text
Instalador-Pets-Windows/Instalador-Pets.exe
Instalador-Pets-Windows/LEIA-ME.txt
```

- [ ] **Step 7: Commit build inputs and artifact**

```powershell
git add .gitignore installer Instalador-Pets-Windows.zip
git commit -m "Build portable Windows pet installer"
```

Use the actual ASCII path `Instalador-Pets-Windows.zip` when staging.

### Task 6: Installation Documentation

**Files:**
- Create: `INSTRUCOES-INSTALACAO-MANUAL.md`
- Create: `INSTRUCOES-INSTALADOR.md`
- Modify: `INSTRUCOES.md`
- Modify: `README.md`

- [ ] **Step 1: Move the current manual content**

Copy the complete existing manual procedure into
`INSTRUCOES-INSTALACAO-MANUAL.md`. Preserve Windows, macOS, Linux, manual-copy,
application, and validation sections.

- [ ] **Step 2: Write the installer guide**

Document:

- Windows 10/11 and Codex runtime requirements;
- explicit statement that Python is not required to run the EXE;
- Python, pip, and PyInstaller as source-build requirements;
- root ZIP download, Windows extraction, and EXE launch;
- SmartScreen **Mais informacoes** / **Executar assim mesmo** path;
- ZIP selection and normal installation;
- all four conflict actions;
- exact total-bits-first, then per-file-bits algorithm;
- warning that equal sizes do not prove equal contents;
- update rollback behavior;
- copy renaming of directory, `id`, and `displayName`;
- permission and malformed-package troubleshooting.

- [ ] **Step 3: Convert the original guide into an index**

`INSTRUCOES.md` must link clearly to:

```markdown
- [Instalacao manual](INSTRUCOES-INSTALACAO-MANUAL.md)
- [Instalacao com o instalador para Windows](INSTRUCOES-INSTALADOR.md)
```

- [ ] **Step 4: Update README**

Add a prominent installer download link near `Como instalar`, retain the quick
manual summary, and link both guides. Add `installer/` and the root release ZIP
to the structure section.

- [ ] **Step 5: Validate documentation links**

Run a PowerShell check that extracts every relative Markdown target from the
four documents and asserts each local path exists.

- [ ] **Step 6: Commit documentation**

```powershell
git add README.md INSTRUCOES.md INSTRUCOES-INSTALACAO-MANUAL.md INSTRUCOES-INSTALADOR.md
git commit -m "Document manual and installer workflows"
```

### Task 7: Final Verification And Publication

**Files:**
- Verify all changed files.

- [ ] **Step 1: Run fresh automated tests**

```powershell
python -m unittest discover -s installer/tests -v
```

Expected: all tests pass.

- [ ] **Step 2: Rebuild from the pinned environment**

```powershell
powershell -ExecutionPolicy Bypass -File .\installer\build.ps1
```

Expected: build, EXE self-test, and release packaging pass.

- [ ] **Step 3: Test a real repository pet in isolation**

Run the Python core against `downloads/rainbow-hope.zip` with a temporary pets
root, assert the installed `pet.json` and `spritesheet.webp` byte-for-byte
match the archive, then remove only that verified temporary root.

- [ ] **Step 4: Smoke-test the built EXE**

```powershell
$exe = Resolve-Path '.\installer\dist\Instalador-Pets.exe'
$process = Start-Process $exe -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 4
if ($process.HasExited) { throw "Executavel encerrou durante o smoke test" }
Stop-Process -Id $process.Id
```

Expected: the GUI process remains alive.

- [ ] **Step 5: Verify repository state**

```powershell
git diff --check
git status --short
git log --oneline origin/main..HEAD
```

Expected: no unstaged changes after the final artifact commit and only
installer-related commits ahead of `origin/main`.

- [ ] **Step 6: Push and open a pull request**

Push `codex/pet-installer`, open a ready pull request targeting `main`, verify
the remote artifact and documentation paths through the GitHub API, then merge
the pull request so the root ZIP is publicly available.
