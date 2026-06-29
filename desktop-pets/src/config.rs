use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

pub const CONFIG_SCHEMA_VERSION: u32 = 1;
pub const MIN_SIZE_PERCENT: u16 = 20;
pub const MAX_SIZE_PERCENT: u16 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SemiFixedPoints {
    pub a: Option<Point>,
    pub b: Option<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MovementMode {
    #[default]
    Fixed,
    BottomStrip,
    Line,
    WholeScreen,
    BetweenMonitors,
    SemiFixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StartupPolicy {
    #[default]
    None,
    Last,
    Specific(Uuid),
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedInstance {
    pub id: Uuid,
    pub pet_id: String,
    pub size_percent: u16,
    pub position: Point,
    pub movement: MovementMode,
    pub semi_fixed: SemiFixedPoints,
}

impl Default for SavedInstance {
    fn default() -> Self {
        Self {
            id: Uuid::from_u128(1),
            pet_id: "rainbow-hope".to_owned(),
            size_percent: 100,
            position: Point { x: 80, y: 80 },
            movement: MovementMode::Fixed,
            semi_fixed: SemiFixedPoints::default(),
        }
    }
}

impl SavedInstance {
    pub fn new(pet_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            pet_id: pet_id.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub schema_version: u32,
    pub startup: StartupPolicy,
    pub startup_shortcut_name: String,
    pub instances: Vec<SavedInstance>,
    pub last_active: Option<Uuid>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let instance = SavedInstance::default();
        Self {
            schema_version: CONFIG_SCHEMA_VERSION,
            startup: StartupPolicy::None,
            startup_shortcut_name: "DesktopPets.lnk".to_owned(),
            last_active: Some(instance.id),
            instances: vec![instance],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("pet size must be between 20 and 200 percent, got {0}")]
    InvalidSize(u16),
    #[error("configuration I/O error: {0}")]
    Io(String),
    #[error("invalid configuration JSON: {0}")]
    Json(String),
    #[error("unsupported configuration schema {0}")]
    UnsupportedSchema(u32),
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    Loaded(AppConfig),
    CreatedDefault(AppConfig),
    Recovered { backup: PathBuf, config: AppConfig },
}

pub fn validate_size(size_percent: u16) -> Result<u16, ConfigError> {
    if (MIN_SIZE_PERCENT..=MAX_SIZE_PERCENT).contains(&size_percent) {
        Ok(size_percent)
    } else {
        Err(ConfigError::InvalidSize(size_percent))
    }
}

pub fn save_atomic(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temporary = temporary_path(path);
    let bytes =
        serde_json::to_vec_pretty(config).map_err(|error| ConfigError::Json(error.to_string()))?;
    fs::write(&temporary, bytes)?;

    if path.exists() {
        fs::remove_file(path)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

pub fn load_or_recover(path: &Path) -> Result<LoadOutcome, ConfigError> {
    if !path.exists() {
        let config = AppConfig::default();
        save_atomic(path, &config)?;
        return Ok(LoadOutcome::CreatedDefault(config));
    }

    let bytes = fs::read(path)?;
    match serde_json::from_slice::<AppConfig>(&bytes) {
        Ok(config) if config.schema_version == CONFIG_SCHEMA_VERSION => {
            Ok(LoadOutcome::Loaded(config))
        }
        Ok(config) => recover_invalid(path, ConfigError::UnsupportedSchema(config.schema_version)),
        Err(error) => recover_invalid(path, ConfigError::Json(error.to_string())),
    }
}

fn recover_invalid(path: &Path, _reason: ConfigError) -> Result<LoadOutcome, ConfigError> {
    let backup = invalid_backup_path(path);
    fs::rename(path, &backup)?;
    let config = AppConfig::default();
    save_atomic(path, &config)?;
    Ok(LoadOutcome::Recovered { backup, config })
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

fn invalid_backup_path(path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("config");
    path.with_file_name(format!("{stem}.{timestamp}.invalid.json"))
}
