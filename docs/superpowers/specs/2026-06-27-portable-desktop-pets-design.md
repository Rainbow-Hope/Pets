# Portable Desktop Pets — Design

## Summary

Build a portable Windows 10/11 desktop-pet application that reuses the existing Codex pet format. The application runs without installation, displays one or more independent transparent pet windows, and chooses animation states from autonomous movement, occasional random events, and an adaptive CPU/RAM mood.

The distributed folder contains the executable, one configuration file, and the pet library. Copying this folder to another computer or removable drive preserves the pets and saved settings. Rainbow Hope is included and selected for the first pet.

## Product behavior

### Windows and instances

- The application runs as a borderless, transparent, always-on-top overlay.
- A single coordinator process owns any number of independent pet windows.
- Opening the executable while its coordinator is already running asks that process to create another pet window and then exits.
- Coordination is scoped by a hash of the executable directory, so copied application folders remain independent.
- Each pet window has its own selected pet, size, position, movement mode, and semi-fixed points.
- Left-dragging repositions a pet. Right-clicking opens its control menu.
- Only visible sprite pixels capture mouse input; transparent pixels pass input through to the application below.

### Animation atlas

Each compatible pet contains `pet.json` and `spritesheet.webp`. The atlas is a transparent 1536×1872 WebP arranged as eight 192×208 cells across nine rows:

| Row | State | Frames |
| --- | --- | ---: |
| 0 | `idle` | 6 |
| 1 | `running-right` | 8 |
| 2 | `running-left` | 8 |
| 3 | `waving` | 4 |
| 4 | `jumping` | 5 |
| 5 | `failed` | 8 |
| 6 | `waiting` | 6 |
| 7 | `running` | 6 |
| 8 | `review` | 6 |

Unused cells are ignored. Animation timing is defined by the application and remains consistent across pets.

### Adaptive mood

- Sample total system CPU and physical memory use every two seconds.
- Maintain a 15-minute adaptive baseline for each metric, with conservative absolute thresholds during the initial warm-up.
- Calculate a combined pressure score weighted 65% toward CPU and 35% toward memory.
- Absolute high-load safeguards prevent the adaptive baseline from normalizing sustained overload.
- Apply hysteresis and minimum dwell times so the pet does not flicker between states.
- Map the stabilized score to:
  - calm: `idle` and `waiting`;
  - focused: `review`;
  - occupied: `running`;
  - exhausted: `failed`, only after sustained overload.
- Trigger `waving` and `jumping` as occasional random events only while calm or focused.
- A movement animation temporarily overrides the mood animation. The current mood resumes when movement ends.
- If metric collection fails, retain a calm state rather than showing `failed`.

Exact score thresholds, dwell times, random intervals, and animation frame duration are named constants covered by deterministic tests, not user-facing configuration.

### Movement modes

The right-click menu exposes:

1. **Fixed** — no autonomous movement.
2. **Bottom strip** — horizontal movement inside the current monitor work area near its lower edge.
3. **Line** — horizontal movement at the pet's current vertical position.
4. **Whole screen** — movement between random valid points in the current monitor work area.
5. **Between monitors** — movement between valid points across all connected monitor work areas.
6. **Semi-fixed** — remain at user-defined point A or B and move to the other point when the pointer approaches.

Horizontal direction selects `running-left` or `running-right`. Whole-screen vertical displacement is interpolated while the pet faces the dominant horizontal direction; the existing atlas does not introduce vertical walking states.

The semi-fixed menu provides commands to capture the current location as point A or point B. Pointer proximity has a cooldown so the pet cannot oscillate continuously. If either point becomes unavailable after a monitor change, clamp it to the nearest valid work area.

## Interface

No permanent controls, window chrome, or taskbar entries are shown for pet windows. The right-click menu contains:

- **Pet** — select a pet from the local library.
- **Add pet…** — import a ZIP file or extracted folder.
- **Size…** — open a small numeric dialog accepting 20–200 percent.
- **Movement** — select one of the six movement modes.
- **Semi-fixed points** — set point A or point B when relevant.
- **Start with Windows**:
  - do not restore;
  - restore the last active pet;
  - restore one selected saved instance;
  - restore all saved instances.
- **Close this pet**.
- **Close all**.

Changing the selected pet preserves the window's size, position, and movement settings. Closing the final pet terminates the coordinator process.

The pet picker identifies multiple saved instances with the pet display name and a stable short instance number. Selecting “restore one” stores that instance's stable ID rather than only its pet type.

## Portable storage and import

The release layout is:

```text
DesktopPets/
├─ DesktopPets.exe
├─ config.json
└─ pets/
   └─ rainbow-hope/
      ├─ pet.json
      └─ spritesheet.webp
```

- `config.json` stores a schema version, global startup policy, saved instances, positions, sizes, movement modes, semi-fixed points, and the last active instance.
- Paths in configuration are relative to the executable directory wherever possible.
- Configuration writes use a temporary file followed by an atomic replacement.
- On invalid JSON or an unsupported schema, preserve the original with a timestamped `.invalid.json` suffix and create a safe default configuration.
- Rainbow Hope is the bundled default. If its files are missing or invalid, show a clear blocking error containing the expected paths.
- Import accepts a ZIP or a folder containing `pet.json` and its referenced `spritesheet.webp`.
- Validate the identifier, display name, relative spritesheet path, WebP dimensions, alpha support, and expected atlas grid before copying.
- Reject absolute paths, parent traversal, links, oversized archives, duplicate archive entries, and files outside the selected pet package.
- Copy accepted pets into `pets/<id>/` using a staging directory and atomic rename.
- Importing an existing ID asks for confirmation before replacement. A successful replacement refreshes windows currently using that pet.
- The application never reads from or writes to `.codex/pets`; it owns its portable library.

## Windows implementation

Implement the application as a native Rust executable using Win32 APIs:

- layered windows with premultiplied BGRA buffers for per-pixel alpha;
- custom hit testing against the current frame's alpha mask;
- `GetSystemTimes` for total CPU measurement;
- `GlobalMemoryStatusEx` for physical memory measurement;
- Win32 monitor/work-area and DPI APIs for geometry;
- native popup menus, file/folder dialogs, and a small modal numeric input;
- WebP decoding and ZIP/JSON support compiled into the executable;
- a directory-scoped named mutex and local IPC endpoint for launch coordination.

Rendering and monitoring pause while Windows is locked or suspended and resume with a fresh timing baseline. Frame scheduling avoids catch-up bursts after resume.

Enabling Windows startup creates a shortcut in the current user's Startup folder pointing to the executable's current absolute path and restore arguments. Disabling startup removes only the shortcut owned by this application. Moving the portable folder invalidates the old shortcut; the user must enable startup again from the new location.

No network access, installer, administrator rights, registry storage, WebView, or separately installed runtime is required.

## Error handling

- Show import validation failures in plain language without modifying the existing library.
- Keep the previous pet active if a replacement or pet switch fails.
- Clamp saved windows back into a valid work area after display topology or resolution changes.
- Fall back to Rainbow Hope when a non-default saved pet is missing; if Rainbow Hope is also unavailable, show the blocking asset error.
- Recover from stale coordinator locks by verifying the IPC endpoint before treating another process as active.
- Log concise diagnostics to `DesktopPets.log` beside the executable, rotate at a bounded size, and never include personal paths beyond the portable application directory.

## Performance targets

- Average idle CPU use below 1% on a typical Windows 10/11 system.
- Approximately 50 MB or less working memory for the coordinator and first pet.
- Additional pet windows reuse decoded atlases and add minimal incremental memory.
- System metrics are sampled every two seconds; animation frames are rendered only for visible, active windows.

## Test plan

### Automated tests

- Atlas row/frame lookup and animation wrapping for all nine states.
- Adaptive baseline, weighted score, hysteresis, dwell times, overload safeguards, and deterministic random-event scheduling.
- Configuration round-trip, schema handling, atomic recovery, and stable instance restoration.
- ZIP/folder validation, duplicate IDs, replacement rollback, archive size limits, and path-traversal defenses.
- Work-area clamping and geometry for all movement modes, DPI values, and monitor layouts.
- Directory-scoped mutex/IPC behavior and stale-lock recovery.

### Windows integration and manual tests

- Windows 10 and Windows 11.
- Per-pixel transparency, click-through, dragging, native menus, and always-on-top behavior.
- DPI scales of 100%, 125%, 150%, and 200%.
- Pet sizes of 20%, 100%, and 200%.
- Single-monitor, mixed-DPI multi-monitor, monitor removal, and resolution changes.
- Every movement mode, including semi-fixed A/B and cross-monitor movement.
- Multiple independent pets and repeated executable launches.
- All four startup/restore policies.
- Sleep, lock, resume, and temporary metric-read failure.
- Copying the complete folder to another local directory, removable drive, and second compatible computer.
- Idle CPU and memory measurements with one and multiple pets.

## Acceptance criteria

The feature is complete when:

- `DesktopPets.exe` launches directly without installation or an external runtime;
- a fresh release folder opens one Rainbow Hope pet;
- repeated launches create independent pet windows managed by one coordinator;
- current repository pet ZIPs and extracted folders import successfully;
- all six movement modes and all nine animation states operate as specified;
- CPU/RAM mood changes are stable and observable without rapid state flicker;
- size, pet, position, movement, semi-fixed points, and restoration settings survive restart;
- copying the complete folder preserves the library and configuration;
- the automated suite passes and Windows 10/11 manual checks meet the performance targets.
