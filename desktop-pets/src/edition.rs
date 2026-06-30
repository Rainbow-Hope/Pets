#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Normal,
    Micro,
    Nano,
    Pico,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditionProfile {
    pub display_name: &'static str,
    pub executable_name: &'static str,
    pub coordinator_suffix: &'static str,
    pub frame_interval_ms: u32,
    pub metric_interval_ms: u64,
    pub max_windows: Option<usize>,
    pub can_import: bool,
    pub can_move: bool,
    pub can_startup: bool,
    pub can_spawn: bool,
    pub can_close_all: bool,
}

impl Edition {
    pub const fn profile(self) -> EditionProfile {
        match self {
            Self::Normal => EditionProfile {
                display_name: "Desktop Pets Normal",
                executable_name: "DesktopPets.exe",
                coordinator_suffix: "normal",
                frame_interval_ms: 100,
                metric_interval_ms: 2_000,
                max_windows: None,
                can_import: true,
                can_move: true,
                can_startup: true,
                can_spawn: true,
                can_close_all: true,
            },
            Self::Micro => EditionProfile {
                display_name: "Desktop Pets Micro",
                executable_name: "DesktopPetsMicro.exe",
                coordinator_suffix: "micro",
                frame_interval_ms: 200,
                metric_interval_ms: 5_000,
                max_windows: Some(4),
                can_import: true,
                can_move: true,
                can_startup: true,
                can_spawn: true,
                can_close_all: true,
            },
            Self::Nano => EditionProfile {
                display_name: "Desktop Pets Nano",
                executable_name: "DesktopPetsNano.exe",
                coordinator_suffix: "nano",
                frame_interval_ms: 200,
                metric_interval_ms: 5_000,
                max_windows: Some(1),
                can_import: false,
                can_move: true,
                can_startup: false,
                can_spawn: false,
                can_close_all: false,
            },
            Self::Pico => EditionProfile {
                display_name: "Desktop Pets Pico",
                executable_name: "DesktopPetsPico.exe",
                coordinator_suffix: "pico",
                frame_interval_ms: 250,
                metric_interval_ms: 8_000,
                max_windows: Some(1),
                can_import: false,
                can_move: false,
                can_startup: false,
                can_spawn: false,
                can_close_all: false,
            },
        }
    }
}
