use super::metrics::SystemMetrics;
use super::{FRAME_INTERVAL_MS, command_to_movement, directory_scope_id};
use crate::behavior::mood::{MoodEngine, SAMPLE_INTERVAL_MS};
use crate::behavior::movement::{Motion, MovementPlanner, Rect as WorkRect, Size};
use crate::config::{
    AppConfig, LoadOutcome, MovementMode, Point as SavedPoint, SavedInstance, StartupPolicy,
    load_or_recover, save_atomic,
};
use crate::library::{PetLibrary, ReplacePolicy};
use crate::pet::{Atlas, Frame, PetState};
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, POINT, RECT,
    SIZE as WinSize, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
    CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject,
    EnumDisplayMonitors, GetDC, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST,
    MONITORINFO, MonitorFromWindow, ReleaseDC, SelectObject,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW, FindWindowW, GWLP_USERDATA,
    GetCursorPos, GetMessageW, GetWindowLongPtrW, GetWindowRect, HMENU, HTCAPTION, HTCLIENT,
    HTTRANSPARENT, HWND_TOPMOST, IDC_ARROW, KillTimer, LoadCursorW, MB_ICONERROR, MB_OK,
    MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, MessageBoxW, PostMessageW, PostQuitMessage,
    RegisterClassW, SWP_NOACTIVATE, SendMessageW, SetTimer, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage, ULW_ALPHA,
    WM_APP, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE, WM_EXITSIZEMOVE, WM_LBUTTONDOWN, WM_NCCREATE,
    WM_NCDESTROY, WM_NCHITTEST, WM_RBUTTONUP, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

const WM_APP_SPAWN: u32 = WM_APP + 1;
const TIMER_ID: usize = 1;
const COMMAND_PET_BASE: u32 = 1_000;
const COMMAND_SIZE_BASE: u32 = 300;
const COMMAND_ADD_PET: u32 = 400;
const COMMAND_NEW_PET: u32 = 401;
const COMMAND_CLOSE: u32 = 500;
const COMMAND_CLOSE_ALL: u32 = 501;
const COMMAND_STARTUP_NONE: u32 = 600;
const COMMAND_STARTUP_LAST: u32 = 601;
const COMMAND_STARTUP_THIS: u32 = 602;
const COMMAND_STARTUP_ALL: u32 = 603;
const COMMAND_SET_POINT_A: u32 = 700;
const COMMAND_SET_POINT_B: u32 = 701;
const SIZE_PRESETS: [u16; 8] = [20, 50, 75, 100, 125, 150, 175, 200];

thread_local! {
    static APP: RefCell<Option<AppContext>> = const { RefCell::new(None) };
}

struct AppContext {
    config_path: PathBuf,
    config: AppConfig,
    library: PetLibrary,
    windows: Vec<HWND>,
    class_name: Vec<u16>,
}

struct ActiveMotion {
    motion: Motion,
    started_ms: u64,
    duration_ms: u64,
}

struct PetWindow {
    instance_id: Uuid,
    pet_id: String,
    atlas: Arc<Atlas>,
    size_percent: u16,
    movement: MovementMode,
    semi_fixed: crate::config::SemiFixedPoints,
    position: SavedPoint,
    mood: MoodEngine,
    metrics: SystemMetrics,
    planner: MovementPlanner,
    animation: PetState,
    frame_index: usize,
    current_alpha: Vec<u8>,
    pixel_width: i32,
    pixel_height: i32,
    started: Instant,
    last_metric_ms: u64,
    last_motion_ms: u64,
    motion: Option<ActiveMotion>,
}

pub fn run() -> Result<(), String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("executable path: {error}"))?;
    let exe_dir = executable
        .parent()
        .ok_or_else(|| "the executable has no parent directory".to_owned())?
        .to_path_buf();
    let scope = directory_scope_id(&exe_dir);
    let class_name = wide(&format!("DesktopPets_{scope}"));
    let mutex_name = wide(&format!("Local\\DesktopPets_{scope}"));

    // SAFETY: name is a valid zero-terminated UTF-16 string and no security descriptor is used.
    let mutex = unsafe { CreateMutexW(null(), 0, mutex_name.as_ptr()) };
    if mutex.is_null() {
        return Err(last_error("create coordinator mutex"));
    }
    // SAFETY: GetLastError reads the calling thread's last-error value.
    let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    if already_running {
        // SAFETY: class name is zero-terminated; null title accepts any matching window.
        let existing = unsafe { FindWindowW(class_name.as_ptr(), null()) };
        if !existing.is_null() {
            // SAFETY: existing is a discovered top-level window in the coordinator process.
            unsafe {
                PostMessageW(existing, WM_APP_SPAWN, 0, 0);
                CloseHandle(mutex);
            }
            return Ok(());
        }
    }

    let result = run_coordinator(exe_dir, class_name);
    // SAFETY: mutex was returned by CreateMutexW and remains owned by this process.
    unsafe { CloseHandle(mutex) };
    result
}

fn run_coordinator(exe_dir: PathBuf, class_name: Vec<u16>) -> Result<(), String> {
    register_window_class(&class_name)?;
    let config_path = exe_dir.join("config.json");
    let config = match load_or_recover(&config_path).map_err(|error| error.to_string())? {
        LoadOutcome::Loaded(config)
        | LoadOutcome::CreatedDefault(config)
        | LoadOutcome::Recovered { config, .. } => config,
    };
    let library = PetLibrary::open(&exe_dir.join("pets")).map_err(|error| error.to_string())?;
    if !library
        .root()
        .join("rainbow-hope")
        .join("pet.json")
        .is_file()
    {
        return Err(format!(
            "Rainbow Hope is missing. Expected:\n{}\n{}",
            library
                .root()
                .join("rainbow-hope")
                .join("pet.json")
                .display(),
            library
                .root()
                .join("rainbow-hope")
                .join("spritesheet.webp")
                .display()
        ));
    }

    APP.with(|app| {
        *app.borrow_mut() = Some(AppContext {
            config_path,
            config,
            library,
            windows: Vec::new(),
            class_name,
        });
    });
    spawn_initial_pets()?;

    // SAFETY: MSG points to writable storage for the standard thread message loop.
    unsafe {
        let mut message: MSG = zeroed();
        loop {
            let status = GetMessageW(&mut message, null_mut(), 0, 0);
            if status == -1 {
                APP.with(|app| *app.borrow_mut() = None);
                return Err(last_error("read window message"));
            }
            if status == 0 {
                break;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    APP.with(|app| *app.borrow_mut() = None);
    Ok(())
}

fn register_window_class(class_name: &[u16]) -> Result<(), String> {
    // SAFETY: null asks for the current executable module.
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        return Err(last_error("get executable module"));
    }
    // SAFETY: loading the shared system arrow cursor requires no cleanup.
    let cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: cursor,
        lpszClassName: class_name.as_ptr(),
        ..unsafe { zeroed() }
    };
    // SAFETY: WNDCLASSW fields remain valid for the process lifetime.
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(last_error("register pet window class"));
    }
    Ok(())
}

fn spawn_initial_pets() -> Result<(), String> {
    let instances = APP.with(|app| {
        let app = app.borrow();
        let context = app.as_ref().expect("app initialized");
        if context.config.startup == StartupPolicy::All && !context.config.instances.is_empty() {
            return context.config.instances.clone();
        }
        let selected = match context.config.startup {
            StartupPolicy::Specific(id) => context
                .config
                .instances
                .iter()
                .find(|instance| instance.id == id),
            StartupPolicy::Last => context.config.last_active.and_then(|id| {
                context
                    .config
                    .instances
                    .iter()
                    .find(|instance| instance.id == id)
            }),
            _ => context.config.instances.first(),
        };
        vec![selected.cloned().unwrap_or_default()]
    });
    for instance in instances {
        create_window_for(instance)?;
    }
    Ok(())
}

fn spawn_new_pet() -> Result<(), String> {
    let instance = APP.with(|app| {
        let mut app = app.borrow_mut();
        let context = app.as_mut().expect("app initialized");
        let mut instance = SavedInstance::new("rainbow-hope");
        let offset = context.windows.len() as i32 * 32;
        instance.position = SavedPoint {
            x: 80 + offset,
            y: 80 + offset,
        };
        context.config.last_active = Some(instance.id);
        context.config.instances.push(instance.clone());
        let _ = save_atomic(&context.config_path, &context.config);
        instance
    });
    create_window_for(instance)
}

fn create_window_for(mut instance: SavedInstance) -> Result<(), String> {
    let (class_name, atlas, pet_id) = APP.with(|app| {
        let app = app.borrow();
        let context = app.as_ref().expect("app initialized");
        let requested = context.library.root().join(&instance.pet_id);
        let (directory, pet_id) = if requested.join("pet.json").is_file() {
            (requested, instance.pet_id.clone())
        } else {
            (
                context.library.root().join("rainbow-hope"),
                "rainbow-hope".to_owned(),
            )
        };
        let manifest = crate::pet::PetManifest::load(&directory.join("pet.json"))
            .map_err(|e| e.to_string())?;
        let atlas = Atlas::load(&directory.join(manifest.spritesheet_path))
            .map(Arc::new)
            .map_err(|e| e.to_string())?;
        Ok::<_, String>((context.class_name.clone(), atlas, pet_id))
    })?;
    instance.pet_id.clone_from(&pet_id);
    let pixel_width = scale_dimension(crate::pet::CELL_WIDTH, instance.size_percent);
    let pixel_height = scale_dimension(crate::pet::CELL_HEIGHT, instance.size_percent);
    let window = Box::new(PetWindow {
        instance_id: instance.id,
        pet_id,
        atlas,
        size_percent: instance.size_percent,
        movement: instance.movement,
        semi_fixed: instance.semi_fixed,
        position: instance.position,
        mood: MoodEngine::with_seed(instance.id.as_u128() as u64),
        metrics: SystemMetrics::default(),
        planner: MovementPlanner::with_seed((instance.id.as_u128() >> 64) as u64),
        animation: PetState::Idle,
        frame_index: 0,
        current_alpha: Vec::new(),
        pixel_width,
        pixel_height,
        started: Instant::now(),
        last_metric_ms: 0,
        last_motion_ms: 0,
        motion: None,
    });
    let raw = Box::into_raw(window);
    // SAFETY: null asks for the current executable module.
    let module = unsafe { GetModuleHandleW(null()) };
    // SAFETY: class is registered; raw remains owned until WM_NCDESTROY.
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            wide("DesktopPets").as_ptr(),
            WS_POPUP,
            instance.position.x,
            instance.position.y,
            pixel_width,
            pixel_height,
            null_mut(),
            null_mut(),
            module,
            raw.cast(),
        )
    };
    if hwnd.is_null() {
        // SAFETY: CreateWindowExW failed before taking ownership of raw.
        unsafe { drop(Box::from_raw(raw)) };
        return Err(last_error("create pet window"));
    }
    APP.with(|app| {
        if let Some(context) = app.borrow_mut().as_mut() {
            context.windows.push(hwnd);
        }
    });
    // SAFETY: hwnd is a valid newly created top-level window.
    unsafe {
        ShowWindow(hwnd, 8);
        SetTimer(hwnd, TIMER_ID, FRAME_INTERVAL_MS, None);
    }
    render(hwnd)?;
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: WM_NCCREATE lparam points to CREATESTRUCTW for this window.
        let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };
        return 1;
    }
    let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PetWindow };

    match message {
        WM_APP_SPAWN => {
            if let Err(error) = spawn_new_pet() {
                show_error(hwnd, &error);
            }
            0
        }
        WM_TIMER if wparam == TIMER_ID => {
            if !raw.is_null()
                && let Err(error) = tick(hwnd, unsafe { &mut *raw })
            {
                show_error(hwnd, &error);
            }
            0
        }
        WM_LBUTTONDOWN => {
            unsafe {
                ReleaseCapture();
                SendMessageW(hwnd, 0x00A1, HTCAPTION as usize, 0);
            }
            0
        }
        WM_RBUTTONUP => {
            if !raw.is_null() {
                show_context_menu(hwnd, unsafe { &mut *raw });
            }
            0
        }
        WM_NCHITTEST => {
            if raw.is_null() {
                return HTTRANSPARENT as LRESULT;
            }
            hit_test(hwnd, unsafe { &*raw }, lparam)
        }
        WM_EXITSIZEMOVE => {
            if !raw.is_null() {
                update_saved_position(hwnd, unsafe { &mut *raw }, true);
            }
            0
        }
        WM_DISPLAYCHANGE => {
            if !raw.is_null() {
                clamp_window(hwnd, unsafe { &mut *raw });
                let _ = render(hwnd);
            }
            0
        }
        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) };
            0
        }
        WM_DESTROY => {
            unsafe { KillTimer(hwnd, TIMER_ID) };
            let empty = APP.with(|app| {
                let mut app = app.borrow_mut();
                let Some(context) = app.as_mut() else {
                    return true;
                };
                context.windows.retain(|candidate| *candidate != hwnd);
                if !raw.is_null() {
                    let pet = unsafe { &*raw };
                    if let Some(instance) = context
                        .config
                        .instances
                        .iter_mut()
                        .find(|instance| instance.id == pet.instance_id)
                    {
                        instance.position = pet.position;
                        instance.size_percent = pet.size_percent;
                        instance.pet_id.clone_from(&pet.pet_id);
                        instance.movement = pet.movement;
                        instance.semi_fixed = pet.semi_fixed;
                        context.config.last_active = Some(instance.id);
                    }
                    let _ = save_atomic(&context.config_path, &context.config);
                }
                context.windows.is_empty()
            });
            if empty {
                unsafe { PostQuitMessage(0) };
            }
            0
        }
        WM_NCDESTROY => {
            if !raw.is_null() {
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    drop(Box::from_raw(raw));
                }
            }
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn tick(hwnd: HWND, pet: &mut PetWindow) -> Result<(), String> {
    let now_ms = pet.started.elapsed().as_millis() as u64;
    if now_ms.saturating_sub(pet.last_metric_ms) >= SAMPLE_INTERVAL_MS {
        let sample = pet.metrics.sample().ok_or(());
        pet.animation = pet.mood.update(now_ms, sample).animation;
        pet.last_metric_ms = now_ms;
    }
    update_motion(hwnd, pet, now_ms);
    pet.frame_index = pet.frame_index.wrapping_add(1);
    render(hwnd)
}

fn update_motion(hwnd: HWND, pet: &mut PetWindow, now_ms: u64) {
    if let Some(active) = &pet.motion {
        let progress =
            (now_ms.saturating_sub(active.started_ms) as f64 / active.duration_ms as f64).min(1.0);
        let interpolate = |start: i32, end: i32| {
            (f64::from(start) + f64::from(end - start) * progress).round() as i32
        };
        pet.position = SavedPoint {
            x: interpolate(active.motion.from.x, active.motion.to.x),
            y: interpolate(active.motion.from.y, active.motion.to.y),
        };
        pet.animation = active.motion.animation;
        // SAFETY: hwnd is valid and SetWindowPos only changes position.
        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                pet.position.x,
                pet.position.y,
                0,
                0,
                SWP_NOACTIVATE | 0x0001,
            );
        }
        if progress >= 1.0 {
            pet.motion = None;
            pet.last_motion_ms = now_ms;
            persist_pet(pet, true);
        }
        return;
    }
    if pet.movement == MovementMode::Fixed || now_ms.saturating_sub(pet.last_motion_ms) < 3_000 {
        return;
    }
    if pet.movement == MovementMode::SemiFixed {
        let mut cursor = POINT::default();
        // SAFETY: cursor points to writable storage.
        if unsafe { GetCursorPos(&mut cursor) } == 0 {
            return;
        }
        let dx = i64::from(cursor.x - pet.position.x);
        let dy = i64::from(cursor.y - pet.position.y);
        if dx * dx + dy * dy > 180_i64.pow(2) {
            return;
        }
    }
    let areas = monitor_work_areas();
    let size = Size {
        width: pet.pixel_width,
        height: pet.pixel_height,
    };
    if let Some(motion) = pet
        .planner
        .plan(pet.movement, pet.position, size, &areas, pet.semi_fixed)
    {
        let dx = i64::from(motion.to.x - motion.from.x);
        let dy = i64::from(motion.to.y - motion.from.y);
        let distance = ((dx * dx + dy * dy) as f64).sqrt();
        let duration_ms = (distance / 120.0 * 1_000.0).clamp(800.0, 8_000.0) as u64;
        pet.motion = Some(ActiveMotion {
            motion,
            started_ms: now_ms,
            duration_ms,
        });
    } else {
        pet.last_motion_ms = now_ms;
    }
}

fn render(hwnd: HWND) -> Result<(), String> {
    let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PetWindow };
    if raw.is_null() {
        return Ok(());
    }
    // SAFETY: the pointer is stored by WM_NCCREATE and owned until WM_NCDESTROY.
    let pet = unsafe { &mut *raw };
    let frame = pet.atlas.frame(pet.animation, pet.frame_index);
    let (pixels, alpha) = scale_frame(frame, pet.pixel_width as u32, pet.pixel_height as u32);
    pet.current_alpha = alpha;

    // SAFETY: GDI resources are paired with their corresponding cleanup calls below.
    unsafe {
        let screen = GetDC(null_mut());
        if screen.is_null() {
            return Err(last_error("get screen device context"));
        }
        let memory = CreateCompatibleDC(screen);
        if memory.is_null() {
            ReleaseDC(null_mut(), screen);
            return Err(last_error("create memory device context"));
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: pet.pixel_width,
                biHeight: -pet.pixel_height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..zeroed()
            },
            ..zeroed()
        };
        let mut bits: *mut c_void = null_mut();
        let bitmap = CreateDIBSection(screen, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if bitmap.is_null() || bits.is_null() {
            DeleteDC(memory);
            ReleaseDC(null_mut(), screen);
            return Err(last_error("create transparent bitmap"));
        }
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits.cast::<u8>(), pixels.len());
        let previous = SelectObject(memory, bitmap);
        let destination = POINT {
            x: pet.position.x,
            y: pet.position.y,
        };
        let source = POINT { x: 0, y: 0 };
        let size = WinSize {
            cx: pet.pixel_width,
            cy: pet.pixel_height,
        };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let updated = windows_sys::Win32::UI::WindowsAndMessaging::UpdateLayeredWindow(
            hwnd,
            screen,
            &destination,
            &size,
            memory,
            &source,
            0,
            &blend,
            ULW_ALPHA,
        );
        SelectObject(memory, previous);
        DeleteObject(bitmap);
        DeleteDC(memory);
        ReleaseDC(null_mut(), screen);
        if updated == 0 {
            return Err(last_error("render transparent pet"));
        }
    }
    Ok(())
}

fn scale_frame(frame: &Frame, width: u32, height: u32) -> (Vec<u8>, Vec<u8>) {
    let mut pixels = vec![0; (width * height * 4) as usize];
    let mut alpha = vec![0; (width * height) as usize];
    for y in 0..height {
        let source_y = y * frame.height / height;
        for x in 0..width {
            let source_x = x * frame.width / width;
            let source = (source_y * frame.width + source_x) as usize;
            let destination = (y * width + x) as usize;
            pixels[destination * 4..destination * 4 + 4]
                .copy_from_slice(&frame.premultiplied_bgra[source * 4..source * 4 + 4]);
            alpha[destination] = frame.alpha[source];
        }
    }
    (pixels, alpha)
}

fn show_context_menu(hwnd: HWND, pet: &mut PetWindow) {
    let pets = APP.with(|app| {
        app.borrow()
            .as_ref()
            .and_then(|context| context.library.discover().ok())
            .unwrap_or_default()
    });
    // SAFETY: popup menus are owned and destroyed in this function.
    unsafe {
        let root = CreatePopupMenu();
        let pet_menu = CreatePopupMenu();
        for (index, manifest) in pets.iter().enumerate() {
            append_item(
                pet_menu,
                COMMAND_PET_BASE + index as u32,
                &manifest.display_name,
                manifest.id == pet.pet_id,
            );
        }
        AppendMenuW(root, MF_POPUP, pet_menu as usize, wide("Pet").as_ptr());
        append_item(root, COMMAND_ADD_PET, "Adicionar pet ZIP...", false);
        append_item(root, COMMAND_NEW_PET, "Novo pet", false);

        let size_menu = CreatePopupMenu();
        for (index, size) in SIZE_PRESETS.iter().enumerate() {
            append_item(
                size_menu,
                COMMAND_SIZE_BASE + index as u32,
                &format!("{size}%"),
                *size == pet.size_percent,
            );
        }
        AppendMenuW(root, MF_POPUP, size_menu as usize, wide("Tamanho").as_ptr());

        let movement_menu = CreatePopupMenu();
        let movement_names = [
            "Fixo",
            "Faixa inferior",
            "Linha",
            "Tela inteira",
            "Entre monitores",
            "Semi-fixo A/B",
        ];
        for (index, name) in movement_names.iter().enumerate() {
            let mode = command_to_movement(200 + index as u32).unwrap_or(MovementMode::Fixed);
            append_item(
                movement_menu,
                200 + index as u32,
                name,
                pet.movement == mode,
            );
        }
        AppendMenuW(movement_menu, MF_SEPARATOR, 0, null());
        append_item(
            movement_menu,
            COMMAND_SET_POINT_A,
            "Definir ponto A aqui",
            false,
        );
        append_item(
            movement_menu,
            COMMAND_SET_POINT_B,
            "Definir ponto B aqui",
            false,
        );
        AppendMenuW(
            root,
            MF_POPUP,
            movement_menu as usize,
            wide("Movimento").as_ptr(),
        );

        let startup_menu = CreatePopupMenu();
        let startup = APP.with(|app| {
            app.borrow()
                .as_ref()
                .map(|context| context.config.startup.clone())
                .unwrap_or_default()
        });
        append_item(
            startup_menu,
            COMMAND_STARTUP_NONE,
            "Não restaurar",
            startup == StartupPolicy::None,
        );
        append_item(
            startup_menu,
            COMMAND_STARTUP_LAST,
            "Último ativo",
            startup == StartupPolicy::Last,
        );
        append_item(
            startup_menu,
            COMMAND_STARTUP_THIS,
            "Este pet",
            startup == StartupPolicy::Specific(pet.instance_id),
        );
        append_item(
            startup_menu,
            COMMAND_STARTUP_ALL,
            "Todos",
            startup == StartupPolicy::All,
        );
        AppendMenuW(
            root,
            MF_POPUP,
            startup_menu as usize,
            wide("Ao iniciar").as_ptr(),
        );

        AppendMenuW(root, MF_SEPARATOR, 0, null());
        append_item(root, COMMAND_CLOSE, "Fechar este pet", false);
        append_item(root, COMMAND_CLOSE_ALL, "Fechar todos", false);

        let mut cursor = POINT::default();
        GetCursorPos(&mut cursor);
        let command = TrackPopupMenu(
            root,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            cursor.x,
            cursor.y,
            0,
            hwnd,
            null(),
        );
        DestroyMenu(root);
        if command != 0 {
            handle_command(hwnd, pet, command as u32, &pets);
        }
    }
}

unsafe fn append_item(menu: HMENU, id: u32, label: &str, checked: bool) {
    let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
    unsafe { AppendMenuW(menu, flags, id as usize, wide(label).as_ptr()) };
}

fn handle_command(hwnd: HWND, pet: &mut PetWindow, command: u32, pets: &[crate::pet::PetManifest]) {
    if let Some(mode) = command_to_movement(command) {
        pet.movement = mode;
        pet.motion = None;
        persist_pet(pet, true);
        return;
    }
    if let Some(index) = command
        .checked_sub(COMMAND_SIZE_BASE)
        .filter(|index| (*index as usize) < SIZE_PRESETS.len())
    {
        pet.size_percent = SIZE_PRESETS[index as usize];
        pet.pixel_width = scale_dimension(crate::pet::CELL_WIDTH, pet.size_percent);
        pet.pixel_height = scale_dimension(crate::pet::CELL_HEIGHT, pet.size_percent);
        clamp_window(hwnd, pet);
        persist_pet(pet, true);
        let _ = render(hwnd);
        return;
    }
    if let Some(index) = command
        .checked_sub(COMMAND_PET_BASE)
        .filter(|index| (*index as usize) < pets.len())
    {
        let selected = &pets[index as usize];
        let result = APP.with(|app| {
            let app = app.borrow();
            let context = app.as_ref().expect("app initialized");
            Atlas::load(
                &context
                    .library
                    .root()
                    .join(&selected.id)
                    .join(&selected.spritesheet_path),
            )
            .map(Arc::new)
            .map_err(|error| error.to_string())
        });
        match result {
            Ok(atlas) => {
                pet.atlas = atlas;
                pet.pet_id.clone_from(&selected.id);
                pet.frame_index = 0;
                persist_pet(pet, true);
                let _ = render(hwnd);
            }
            Err(error) => show_error(hwnd, &error),
        }
        return;
    }

    match command {
        COMMAND_ADD_PET => {
            if let Some(path) = choose_zip(hwnd) {
                let result = APP.with(|app| {
                    let app = app.borrow();
                    let context = app.as_ref().expect("app initialized");
                    context
                        .library
                        .import(&path, ReplacePolicy::Replace)
                        .map_err(|error| error.to_string())
                });
                match result {
                    Ok(manifest) => show_information(
                        hwnd,
                        &format!("Pet \"{}\" importado.", manifest.display_name),
                    ),
                    Err(error) => show_error(hwnd, &error),
                }
            }
        }
        COMMAND_NEW_PET => {
            if let Err(error) = spawn_new_pet() {
                show_error(hwnd, &error);
            }
        }
        COMMAND_SET_POINT_A => {
            pet.semi_fixed.a = Some(pet.position);
            persist_pet(pet, true);
        }
        COMMAND_SET_POINT_B => {
            pet.semi_fixed.b = Some(pet.position);
            persist_pet(pet, true);
        }
        COMMAND_STARTUP_NONE => set_startup(StartupPolicy::None),
        COMMAND_STARTUP_LAST => set_startup(StartupPolicy::Last),
        COMMAND_STARTUP_THIS => set_startup(StartupPolicy::Specific(pet.instance_id)),
        COMMAND_STARTUP_ALL => set_startup(StartupPolicy::All),
        COMMAND_CLOSE => unsafe {
            DestroyWindow(hwnd);
        },
        COMMAND_CLOSE_ALL => {
            let windows = APP.with(|app| {
                app.borrow()
                    .as_ref()
                    .map(|context| context.windows.clone())
                    .unwrap_or_default()
            });
            for window in windows {
                unsafe { DestroyWindow(window) };
            }
        }
        _ => {}
    }
}

fn choose_zip(owner: HWND) -> Option<PathBuf> {
    let mut path = [0_u16; 32_768];
    let filter: Vec<u16> = "Pacotes ZIP\0*.zip\0Todos os arquivos\0*.*\0\0"
        .encode_utf16()
        .collect();
    let mut dialog = OPENFILENAMEW {
        lStructSize: size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: filter.as_ptr(),
        lpstrFile: path.as_mut_ptr(),
        nMaxFile: path.len() as u32,
        Flags: OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
        ..unsafe { zeroed() }
    };
    // SAFETY: OPENFILENAMEW references writable path storage for the duration of the call.
    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        return None;
    }
    let length = path.iter().position(|value| *value == 0)?;
    Some(PathBuf::from(String::from_utf16_lossy(&path[..length])))
}

fn set_startup(policy: StartupPolicy) {
    APP.with(|app| {
        let mut app = app.borrow_mut();
        if let Some(context) = app.as_mut() {
            context.config.startup = policy;
            let _ = save_atomic(&context.config_path, &context.config);
        }
    });
}

fn persist_pet(pet: &PetWindow, write: bool) {
    APP.with(|app| {
        let mut app = app.borrow_mut();
        let Some(context) = app.as_mut() else {
            return;
        };
        if let Some(instance) = context
            .config
            .instances
            .iter_mut()
            .find(|instance| instance.id == pet.instance_id)
        {
            instance.pet_id.clone_from(&pet.pet_id);
            instance.size_percent = pet.size_percent;
            instance.position = pet.position;
            instance.movement = pet.movement;
            instance.semi_fixed = pet.semi_fixed;
            context.config.last_active = Some(instance.id);
        }
        if write {
            let _ = save_atomic(&context.config_path, &context.config);
        }
    });
}

fn update_saved_position(hwnd: HWND, pet: &mut PetWindow, write: bool) {
    let mut rectangle = RECT::default();
    // SAFETY: rectangle points to writable storage and hwnd is valid during message dispatch.
    if unsafe { GetWindowRect(hwnd, &mut rectangle) } != 0 {
        pet.position = SavedPoint {
            x: rectangle.left,
            y: rectangle.top,
        };
        persist_pet(pet, write);
    }
}

fn hit_test(hwnd: HWND, pet: &PetWindow, lparam: LPARAM) -> LRESULT {
    let screen_x = (lparam as u32 & 0xffff) as u16 as i16 as i32;
    let screen_y = ((lparam as u32 >> 16) & 0xffff) as u16 as i16 as i32;
    let mut rectangle = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rectangle) } == 0 {
        return HTTRANSPARENT as LRESULT;
    }
    let x = screen_x - rectangle.left;
    let y = screen_y - rectangle.top;
    if x < 0 || y < 0 || x >= pet.pixel_width || y >= pet.pixel_height {
        return HTTRANSPARENT as LRESULT;
    }
    let index = (y * pet.pixel_width + x) as usize;
    if pet.current_alpha.get(index).copied().unwrap_or(0) < 16 {
        HTTRANSPARENT as LRESULT
    } else {
        HTCLIENT as LRESULT
    }
}

fn clamp_window(hwnd: HWND, pet: &mut PetWindow) {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..unsafe { zeroed() }
    };
    if monitor.is_null() || unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return;
    }
    pet.position.x = pet.position.x.clamp(
        info.rcWork.left,
        (info.rcWork.right - pet.pixel_width).max(info.rcWork.left),
    );
    pet.position.y = pet.position.y.clamp(
        info.rcWork.top,
        (info.rcWork.bottom - pet.pixel_height).max(info.rcWork.top),
    );
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            pet.position.x,
            pet.position.y,
            0,
            0,
            SWP_NOACTIVATE | 0x0001,
        );
    }
}

fn monitor_work_areas() -> Vec<WorkRect> {
    unsafe extern "system" fn callback(
        monitor: HMONITOR,
        _dc: HDC,
        _rect: *mut RECT,
        data: LPARAM,
    ) -> i32 {
        let areas = unsafe { &mut *(data as *mut Vec<WorkRect>) };
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..unsafe { zeroed() }
        };
        if unsafe { GetMonitorInfoW(monitor, &mut info) } != 0 {
            areas.push(WorkRect {
                left: info.rcWork.left,
                top: info.rcWork.top,
                right: info.rcWork.right,
                bottom: info.rcWork.bottom,
            });
        }
        1
    }
    let mut areas = Vec::new();
    // SAFETY: callback appends to areas synchronously and data remains valid until return.
    unsafe {
        EnumDisplayMonitors(
            null_mut(),
            null(),
            Some(callback),
            (&mut areas as *mut Vec<WorkRect>) as LPARAM,
        );
    }
    areas
}

fn scale_dimension(base: u32, percent: u16) -> i32 {
    ((u64::from(base) * u64::from(percent) + 50) / 100) as i32
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn last_error(operation: &str) -> String {
    format!("{operation}: {}", std::io::Error::last_os_error())
}

fn show_error(owner: HWND, message: &str) {
    let message = wide(message);
    let title = wide("DesktopPets");
    // SAFETY: both strings are zero-terminated and valid for the call duration.
    unsafe {
        MessageBoxW(
            owner,
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn show_information(owner: HWND, message: &str) {
    let message = wide(message);
    let title = wide("DesktopPets");
    // SAFETY: both strings are zero-terminated and valid for the call duration.
    unsafe {
        MessageBoxW(owner, message.as_ptr(), title.as_ptr(), MB_OK);
    }
}
