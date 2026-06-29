use crate::config::{MovementMode, Point, SemiFixedPoints};
use crate::pet::PetState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn contains_top_left(self, point: Point, size: Size) -> bool {
        point.x >= self.left
            && point.y >= self.top
            && point.x <= self.right.saturating_sub(size.width)
            && point.y <= self.bottom.saturating_sub(size.height)
    }

    fn clamp(self, point: Point, size: Size) -> Point {
        Point {
            x: point.x.clamp(
                self.left,
                self.right.saturating_sub(size.width).max(self.left),
            ),
            y: point.y.clamp(
                self.top,
                self.bottom.saturating_sub(size.height).max(self.top),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Motion {
    pub from: Point,
    pub to: Point,
    pub direction: Direction,
    pub animation: PetState,
}

impl Motion {
    pub fn new(from: Point, to: Point) -> Self {
        let direction = if to.x < from.x {
            Direction::Left
        } else {
            Direction::Right
        };
        let animation = match direction {
            Direction::Left => PetState::RunningLeft,
            Direction::Right => PetState::RunningRight,
        };
        Self {
            from,
            to,
            direction,
            animation,
        }
    }
}

#[derive(Debug)]
pub struct MovementPlanner {
    rng: XorShift64,
}

impl MovementPlanner {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: XorShift64::new(seed),
        }
    }

    pub fn plan(
        &mut self,
        mode: MovementMode,
        current: Point,
        size: Size,
        work_areas: &[Rect],
        semi_fixed: SemiFixedPoints,
    ) -> Option<Motion> {
        if mode == MovementMode::Fixed || work_areas.is_empty() {
            return None;
        }
        let current_area = nearest_work_area(current, size, work_areas)?;
        let target = match mode {
            MovementMode::Fixed => return None,
            MovementMode::BottomStrip => Point {
                x: self.random_between(
                    current_area.left,
                    current_area.right.saturating_sub(size.width),
                ),
                y: current_area.bottom.saturating_sub(size.height),
            },
            MovementMode::Line => Point {
                x: self.random_between(
                    current_area.left,
                    current_area.right.saturating_sub(size.width),
                ),
                y: current.y.clamp(
                    current_area.top,
                    current_area
                        .bottom
                        .saturating_sub(size.height)
                        .max(current_area.top),
                ),
            },
            MovementMode::WholeScreen => self.random_point(current_area, size),
            MovementMode::BetweenMonitors => {
                let index = self.rng.next() as usize % work_areas.len();
                self.random_point(work_areas[index], size)
            }
            MovementMode::SemiFixed => {
                let (a, b) = (semi_fixed.a?, semi_fixed.b?);
                let destination = if squared_distance(current, a) <= squared_distance(current, b) {
                    b
                } else {
                    a
                };
                clamp_to_work_areas(destination, size, work_areas)
            }
        };
        Some(Motion::new(current, target))
    }

    fn random_point(&mut self, area: Rect, size: Size) -> Point {
        Point {
            x: self.random_between(area.left, area.right.saturating_sub(size.width)),
            y: self.random_between(area.top, area.bottom.saturating_sub(size.height)),
        }
    }

    fn random_between(&mut self, start: i32, end: i32) -> i32 {
        let end = end.max(start);
        let width = i64::from(end) - i64::from(start) + 1;
        i64::from(start).saturating_add((self.rng.next() % width as u64) as i64) as i32
    }
}

impl Default for MovementPlanner {
    fn default() -> Self {
        Self::with_seed(0x50E7_CAFE)
    }
}

pub fn clamp_to_work_areas(point: Point, size: Size, work_areas: &[Rect]) -> Point {
    nearest_work_area(point, size, work_areas)
        .map(|area| area.clamp(point, size))
        .unwrap_or(point)
}

fn nearest_work_area(point: Point, size: Size, work_areas: &[Rect]) -> Option<Rect> {
    if let Some(area) = work_areas
        .iter()
        .copied()
        .find(|area| area.contains_top_left(point, size))
    {
        return Some(area);
    }
    work_areas.iter().copied().min_by_key(|area| {
        let clamped = area.clamp(point, size);
        squared_distance(point, clamped)
    })
}

fn squared_distance(a: Point, b: Point) -> i64 {
    let dx = i64::from(a.x) - i64::from(b.x);
    let dy = i64::from(a.y) - i64::from(b.y);
    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
}

#[derive(Debug)]
struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }
}
