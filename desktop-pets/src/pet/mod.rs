mod atlas;

pub use atlas::{ATLAS_HEIGHT, ATLAS_WIDTH, Atlas, CELL_HEIGHT, CELL_WIDTH, Frame, PetState};

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetManifest {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub spritesheet_path: PathBuf,
}

impl PetManifest {
    pub fn load(path: &Path) -> Result<Self, PetError> {
        let json = fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    pub fn from_json(json: &str) -> Result<Self, PetError> {
        let manifest: Self = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), PetError> {
        let valid_id = !self.id.is_empty()
            && self.id.len() <= 64
            && self
                .id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && !self.id.starts_with('-')
            && !self.id.ends_with('-');
        if !valid_id {
            return Err(PetError::InvalidId(self.id.clone()));
        }
        if self.display_name.trim().is_empty() || self.display_name.len() > 120 {
            return Err(PetError::InvalidDisplayName(self.display_name.clone()));
        }
        if self.spritesheet_path.as_os_str().is_empty()
            || self.spritesheet_path.is_absolute()
            || self.spritesheet_path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(PetError::UnsafeSpritesheetPath(
                self.spritesheet_path.clone(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum PetError {
    #[error("pet file I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid pet manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid pet identifier: {0}")]
    InvalidId(String),
    #[error("invalid pet display name: {0}")]
    InvalidDisplayName(String),
    #[error("unsafe spritesheet path: {0}")]
    UnsafeSpritesheetPath(PathBuf),
    #[error("could not decode pet image: {0}")]
    Image(#[from] image::ImageError),
    #[error("atlas must be 1536x1872 pixels, got {width}x{height}")]
    InvalidAtlasSize { width: u32, height: u32 },
    #[error("atlas does not contain an alpha channel")]
    MissingAlpha,
}
