# Desktop Pets — Normal and Light Editions Design

## Summary

Desktop Pets will be distributed in two product types:

1. **Normal**, the complete edition.
2. **Light**, a family containing the **Micro**, **Nano**, and **Pico** variations.

The capability and expected resource hierarchy is:

```text
Normal > Micro > Nano > Pico
```

Normal has the most features and the highest expected CPU, memory, and storage use. Pico has the smallest feature set and the lowest expected resource use. All four remain portable Windows 10/11 applications, require no installer or external runtime, include Rainbow Hope, and preserve the core CPU/RAM-driven pet behavior.

## Shared architecture

All editions are built from one Rust codebase. Pure configuration, pet-manifest validation, atlas decoding, CPU/RAM mood logic, and native layered-window rendering remain shared. A compile-time `EditionProfile` selects the visible features and timing constants for each executable.

Compile-time selection is required instead of a runtime switch. Code and dependencies that an edition cannot use are excluded from that executable where practical. This keeps the light editions smaller and prevents disabled functionality from appearing in their menus.

Each edition has:

- its own executable name;
- its own directory-scoped coordinator identity;
- its own portable package;
- its own `config.json` inside that package;
- its own startup identity when startup support exists.

Running one edition never sends commands to, reads configuration from, or changes another edition.

There is no installer, setup wizard, bootstrapper, or first-run dependency download. The `.exe` runs directly beside its own `config.json` and `pets/` directory. It does not require Codex, Rust, Python, WebView, internet access, administrator rights, or a separately installed runtime.

## Edition profiles

### Normal

Executable: `DesktopPets.exe`

Normal implements the complete approved Portable Desktop Pets design:

- any number of independent pet windows;
- repeated launches create another pet in the coordinator process;
- ZIP and extracted-folder import;
- all six movement modes;
- semi-fixed points A and B;
- all startup and restoration policies;
- pet selection, size from 20–200%, closing one, and closing all;
- 10 animation updates per second;
- CPU/RAM sampling every 2 seconds.

### Micro

Executable: `DesktopPetsMicro.exe`

Micro is the nearly complete light edition. It retains the same user-facing feature categories as Normal, including multiple pets, import, movement, and restoration. Its savings come from conservative scheduling and bounded concurrency:

- 5 animation updates per second;
- CPU/RAM sampling every 5 seconds;
- no catch-up rendering after the application is delayed, locked, or suspended;
- maximum of four simultaneous pet windows;
- longer idle intervals before autonomous movement and random events;
- decoded atlases are shared between windows using the same pet.

When the four-window limit is reached, another launch focuses the existing coordinator and shows a plain-language limit message rather than creating a fifth pet.

### Nano

Executable: `DesktopPetsNano.exe`

Nano is the balanced light edition:

- one pet window;
- CPU/RAM mood and occasional calm/focused random events;
- all six autonomous movement modes;
- pet selection from folders already present in its local `pets/` directory;
- size selection from 20–200%;
- close command;
- 5 animation updates per second;
- CPU/RAM sampling every 5 seconds.

Nano excludes:

- ZIP/folder import through the interface;
- multiple simultaneous pets;
- repeated-launch spawning;
- Windows startup and restoration controls;
- “close all”, because only one pet can exist.

A repeated launch does not create another process or pet; it only brings the existing Nano pet to the foreground.

### Pico

Executable: `DesktopPetsPico.exe`

Pico is the essential light edition:

- one fixed pet window;
- CPU/RAM mood;
- occasional waving and jumping while calm or focused;
- pet selection from folders already present in its local `pets/` directory;
- size selection from 20–200%;
- close command;
- 4 animation updates per second;
- CPU/RAM sampling every 8 seconds.

Pico excludes:

- autonomous movement and movement menus;
- ZIP/folder import through the interface;
- multiple pets and repeated-launch spawning;
- Windows startup and restoration controls;
- semi-fixed points;
- “close all”.

Dragging remains available so the user can manually position the fixed pet.

## Resource reductions

The frame representation will stop storing a second full alpha array. Per-pixel hit testing reads the alpha byte already present in each premultiplied BGRA pixel. Only the currently scaled frame keeps a small alpha mask for the active window.

Every edition will:

- decode each active pet atlas once per process;
- share decoded atlases where multiple windows are allowed;
- avoid rendering an unchanged invisible or suspended window;
- use release LTO, one code-generation unit, size optimization, panic abort, and symbol stripping;
- avoid network, WebView, administrator, and registry dependencies.

The build report will record the actual packaged size, executable size, idle CPU, and working-set memory measured on the development machine. It will not claim unmeasured percentage savings.

## Menus

Menus are generated from the active edition profile. Disabled capabilities are absent, not merely greyed out.

| Menu capability | Normal | Micro | Nano | Pico |
| --- | --- | --- | --- | --- |
| Choose installed pet | Yes | Yes | Yes | Yes |
| Import pet | Yes | Yes | No | No |
| Size | Yes | Yes | Yes | Yes |
| Movement modes | Yes | Yes | Yes | No |
| Semi-fixed A/B | Yes | Yes | Yes | No |
| New pet | Yes | Yes, up to four | No | No |
| Startup/restore | Yes | Yes | No | No |
| Close this pet | Yes | Yes | Yes | Yes |
| Close all | Yes | Yes | No | No |

## Repository and portable packages

The final deliverables are committed to the `main` branch of the GitHub repository named `Pets`. All runnable artifacts live under the tracked repository directory `Executar fora do Códex/`.

The build produces four independent folders:

```text
Executar fora do Códex/
├─ LEIA-ME.txt
├─ DIFERENCAS-ENTRE-EDICOES.txt
├─ Normal/
│  ├─ DesktopPets.exe
│  ├─ config.json
│  ├─ LEIA-ME.txt
│  ├─ DIFERENCAS-ENTRE-EDICOES.txt
│  └─ pets/rainbow-hope/...
└─ Leves/
   ├─ Micro/
   │  ├─ DesktopPetsMicro.exe
   │  ├─ config.json
   │  ├─ LEIA-ME.txt
   │  ├─ DIFERENCAS-ENTRE-EDICOES.txt
   │  └─ pets/rainbow-hope/...
   ├─ Nano/
   │  ├─ DesktopPetsNano.exe
   │  ├─ config.json
   │  ├─ LEIA-ME.txt
   │  ├─ DIFERENCAS-ENTRE-EDICOES.txt
   │  └─ pets/rainbow-hope/...
   └─ Pico/
      ├─ DesktopPetsPico.exe
      ├─ config.json
      ├─ LEIA-ME.txt
      ├─ DIFERENCAS-ENTRE-EDICOES.txt
      └─ pets/rainbow-hope/...
```

`LEIA-ME.txt` states explicitly that the files are portable applications, not installers. Copying or downloading one complete edition folder preserves only that edition’s settings and pet library. Moving only the `.exe` without its adjacent files is unsupported because those files are part of the self-contained portable application.

## Edition comparison document

`DIFERENCAS-ENTRE-EDICOES.txt` is maintained at `Executar fora do Códex/DIFERENCAS-ENTRE-EDICOES.txt` and copied unchanged into every edition folder. Its required order is:

1. a direct opening summary containing `Normal > Micro > Nano > Pico`;
2. a table of contents/index;
3. a compact comparison table;
4. detailed descriptions of Normal, Micro, Nano, and Pico;
5. feature-by-feature differences;
6. measured executable, package, CPU, and memory results;
7. a short recommendation explaining which edition suits each use case.

The document uses plain UTF-8 text so it opens in Notepad without Markdown rendering.

## Error handling

- All editions show a blocking message with expected paths when Rainbow Hope is unavailable.
- Invalid installed pet folders are skipped from the picker and logged.
- Normal and Micro keep the existing staged import and rollback protections.
- Nano and Pico never expose import code paths.
- Unsupported configuration data is preserved with an `.invalid.json` suffix before safe recovery.
- An edition limit is reported without starting an extra coordinator or modifying saved instances.

## Tests and measurements

Automated tests cover:

- the exact capabilities and timing constants of all four profiles;
- distinct executable/coordinator identities;
- per-edition instance limits;
- profile-driven menu visibility;
- absence of import/startup/movement commands where excluded;
- the shared alpha-byte hit-testing representation;
- all existing configuration, atlas, mood, movement, and import contracts;
- generation of all four portable folder layouts;
- required headings and hierarchy in `DIFERENCAS-ENTRE-EDICOES.txt`.

Windows smoke tests launch each packaged executable, confirm only the allowed number of windows, exercise its available menu commands, and close it normally. Resource measurements use the same machine, pet, size, idle duration, and foreground conditions for all editions.

Windows 10 compatibility remains a manual check on a separate compatible machine; the development machine verifies Windows 11 behavior.

## Acceptance criteria

The edition work is complete when:

- four independently runnable portable packages are produced;
- every package starts with Rainbow Hope and no installation;
- no installer or dependency bootstrapper is produced;
- Normal retains the complete approved behavior;
- Micro, Nano, and Pico expose exactly their documented capability sets;
- coordinator and configuration state never cross edition boundaries;
- lighter editions use lower update/sample frequencies as documented;
- the duplicated atlas alpha storage is removed;
- `DIFERENCAS-ENTRE-EDICOES.txt` appears in the repository and all four packages;
- all deliverables appear under `Executar fora do Códex/` on the `main` branch of the GitHub `Pets` repository;
- tests, lint, release builds, packaging checks, and Windows smoke tests pass;
- measured resource values are written into the comparison document without invented estimates.
