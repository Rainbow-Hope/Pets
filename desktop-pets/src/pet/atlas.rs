use super::PetError;
use image::GenericImageView;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub const CELL_WIDTH: u32 = 192;
pub const CELL_HEIGHT: u32 = 208;
pub const ATLAS_WIDTH: u32 = CELL_WIDTH * 8;
pub const ATLAS_HEIGHT: u32 = CELL_HEIGHT * 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PetState {
    Idle,
    RunningRight,
    RunningLeft,
    Waving,
    Jumping,
    Failed,
    Waiting,
    Running,
    Review,
}

impl PetState {
    pub const ALL: [Self; 9] = [
        Self::Idle,
        Self::RunningRight,
        Self::RunningLeft,
        Self::Waving,
        Self::Jumping,
        Self::Failed,
        Self::Waiting,
        Self::Running,
        Self::Review,
    ];

    pub const fn row(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::RunningRight => 1,
            Self::RunningLeft => 2,
            Self::Waving => 3,
            Self::Jumping => 4,
            Self::Failed => 5,
            Self::Waiting => 6,
            Self::Running => 7,
            Self::Review => 8,
        }
    }

    pub const fn frame_count(self) -> usize {
        match self {
            Self::Idle => 6,
            Self::RunningRight | Self::RunningLeft | Self::Failed => 8,
            Self::Waving => 4,
            Self::Jumping => 5,
            Self::Waiting | Self::Running | Self::Review => 6,
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub premultiplied_bgra: Vec<u8>,
    pub alpha: Vec<u8>,
}

#[derive(Debug)]
pub struct Atlas {
    rows: [Vec<Arc<Frame>>; 9],
}

impl Atlas {
    pub fn load(path: &Path) -> Result<Self, PetError> {
        let bytes = fs::read(path)?;
        let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::WebP)?;
        let (width, height) = decoded.dimensions();
        if (width, height) != (ATLAS_WIDTH, ATLAS_HEIGHT) {
            return Err(PetError::InvalidAtlasSize { width, height });
        }
        if !decoded.color().has_alpha() {
            return Err(PetError::MissingAlpha);
        }

        let rgba = decoded.into_rgba8();
        let mut rows: [Vec<Arc<Frame>>; 9] = std::array::from_fn(|_| Vec::new());
        for state in PetState::ALL {
            let row = state.row();
            for column in 0..state.frame_count() {
                let view = rgba
                    .view(
                        column as u32 * CELL_WIDTH,
                        row as u32 * CELL_HEIGHT,
                        CELL_WIDTH,
                        CELL_HEIGHT,
                    )
                    .to_image();
                let mut bgra = Vec::with_capacity((CELL_WIDTH * CELL_HEIGHT * 4) as usize);
                let mut alpha = Vec::with_capacity((CELL_WIDTH * CELL_HEIGHT) as usize);
                for pixel in view.pixels() {
                    let [red, green, blue, opacity] = pixel.0;
                    let premultiply =
                        |channel: u8| ((u16::from(channel) * u16::from(opacity) + 127) / 255) as u8;
                    bgra.extend_from_slice(&[
                        premultiply(blue),
                        premultiply(green),
                        premultiply(red),
                        opacity,
                    ]);
                    alpha.push(opacity);
                }
                rows[row].push(Arc::new(Frame {
                    width: CELL_WIDTH,
                    height: CELL_HEIGHT,
                    premultiplied_bgra: bgra,
                    alpha,
                }));
            }
        }
        Ok(Self { rows })
    }

    pub fn frames(&self, state: PetState) -> &[Arc<Frame>] {
        &self.rows[state.row()]
    }

    pub fn frame(&self, state: PetState, index: usize) -> &Arc<Frame> {
        let frames = self.frames(state);
        &frames[index % frames.len()]
    }
}
