# Optional All-Pets Helper — Design

## Summary

Create an optional, self-contained Windows executable named
`AdicionarTodosOsPets.exe`. When placed beside one Desktop Pets edition
executable and run, it adds every valid pet from two sources:

1. the current `pets/` catalog embedded in the helper at build time;
2. the current user’s `%USERPROFILE%\.codex\pets` directory, when present.

The helper is not an installer for Desktop Pets. It does not install services,
modify the registry, require administrator rights, access the network, or
change the Codex library. It copies validated pet packages only into the
adjacent portable `pets/` directory.

## Distribution

The Normal package includes the helper directly:

```text
Executar fora do Códex/
└─ Normal/
   ├─ DesktopPets.exe
   ├─ AdicionarTodosOsPets.exe
   ├─ config.json
   └─ pets/...
```

The light editions remain small. One separately downloadable copy is provided:

```text
Executar fora do Códex/
└─ Leves/
   ├─ Auxiliar opcional - Todos os Pets/
   │  ├─ AdicionarTodosOsPets.exe
   │  └─ LEIA-ME-AUXILIAR.txt
   ├─ Micro/...
   ├─ Nano/...
   └─ Pico/...
```

The light-edition instructions state that both files from the optional folder
must be copied into the selected `Micro`, `Nano`, or `Pico` directory. The
helper must then remain in the same directory as that edition’s `.exe` while it
runs. Running it from the optional download directory itself is rejected
because that directory is not an edition.

## Embedded catalog

A Rust build script scans the repository’s tracked `pets/` tree. It recognizes
packages by `pet.json`, parses each manifest, resolves only its safe relative
`spritesheetPath`, and generates compile-time entries using `include_bytes!`.
Only `pet.json` and the referenced spritesheet are embedded. Preview GIFs,
validation reports, downloads, and unrelated files are excluded.

The generated catalog is deterministic:

- entries are sorted by pet identifier;
- duplicate identifiers fail the build;
- unsafe paths fail the build;
- missing spritesheets fail the build;
- each source file has a Cargo rebuild trigger.

The WebP files are already compressed, so there is no adjacent archive that can
be separated accidentally. The helper remains one autonomous `.exe`.

## Destination detection

The helper uses its own executable directory as the destination root. Before
copying, that directory must contain:

- one recognized edition executable:
  `DesktopPets.exe`, `DesktopPetsMicro.exe`, `DesktopPetsNano.exe`, or
  `DesktopPetsPico.exe`;
- `config.json`;
- a writable `pets/` directory, which may be created when absent.

The helper never accepts a destination argument and never writes outside its
own edition directory.

## Import flow

For every embedded package:

1. write the embedded manifest and spritesheet to a uniquely named staging
   directory inside the destination;
2. validate the manifest identifier, display name, relative path, atlas
   geometry, and alpha support using the shared Desktop Pets library;
3. import with `ReplacePolicy::Reject`;
4. remove staging regardless of success or failure.

For `%USERPROFILE%\.codex\pets`:

1. inspect immediate child directories only;
2. ignore links and non-directories;
3. require `pet.json`;
4. validate and import through the same staging/atomic library path.

The source priority is embedded catalog first, Codex directory second.
Existing destination identifiers are never overwritten. This protects user
customizations and naturally eliminates duplicates between the embedded,
Codex, and destination libraries.

## Results and interface

The helper has no permanent window or console. After processing, one native
message box reports:

- pets added from the embedded catalog;
- pets added from Codex;
- pets already present and skipped;
- invalid pets skipped;
- whether the Codex source directory was absent;
- the destination `pets/` directory.

A blocking error is shown before copying when the helper is not beside a
recognized edition or the destination is not writable. Individual invalid pets
do not abort valid imports; they appear in the final count.

## Documentation

`LEIA-ME-AUXILIAR.txt`, the main `LEIA-ME.txt`, and
`DIFERENCAS-ENTRE-EDICOES.txt` explain:

- the helper is optional;
- it adds pets and does not install Desktop Pets;
- Normal already contains it;
- light-edition users copy the helper and its instructions beside their chosen
  edition executable;
- it must remain in that same folder while running;
- it reads `.codex\pets` but never modifies it;
- existing destination pets are not replaced;
- after import, the helper may be deleted without affecting imported pets.

## Testing

Automated tests cover:

- deterministic embedded-catalog generation;
- every current repository pet appears exactly once;
- only manifest and referenced spritesheet bytes are embedded;
- destination detection for all four edition names;
- rejection outside an edition directory;
- existing-ID skip behavior;
- embedded plus Codex merge without duplicates;
- invalid Codex package isolation;
- staging cleanup after success and failure;
- no writes to the Codex source;
- package layout and documentation requirements.

Windows smoke tests copy a temporary edition folder, run the packaged helper,
verify that all embedded repository pet identifiers are present, verify that a
synthetic valid Codex-only pet is added without changing its source, and then
launch the edition picker against the expanded library.

## Acceptance criteria

The helper is complete when:

- `AdicionarTodosOsPets.exe` runs directly without installation or external
  runtime;
- the executable contains every valid tracked repository pet;
- valid `.codex\pets` entries are added when available;
- duplicates and existing destination IDs are skipped without overwrite;
- invalid pets cannot modify the destination;
- Normal contains the helper;
- the light-edition optional folder contains a separately downloadable helper
  and instructions;
- documentation explains same-folder placement and optional deletion;
- automated tests, strict lint, release build, package checks, and Windows
  smoke tests pass;
- all artifacts are committed and pushed to `main` in the GitHub `Pets`
  repository.
