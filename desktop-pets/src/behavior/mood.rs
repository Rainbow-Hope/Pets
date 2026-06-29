use crate::pet::PetState;
use std::collections::VecDeque;

pub const SAMPLE_INTERVAL_MS: u64 = 2_000;
pub const BASELINE_WINDOW_MS: u64 = 15 * 60 * 1_000;
pub const MOOD_DWELL_MS: u64 = 4_000;
pub const EXHAUSTED_DWELL_MS: u64 = 10_000;
pub const CPU_WEIGHT: f32 = 0.65;
pub const MEMORY_WEIGHT: f32 = 0.35;
pub const ABSOLUTE_CPU_OVERLOAD: f32 = 92.0;
pub const ABSOLUTE_MEMORY_OVERLOAD: f32 = 94.0;
pub const RANDOM_EVENT_MIN_MS: u64 = 30_000;
pub const RANDOM_EVENT_MAX_MS: u64 = 90_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceSample {
    pub cpu_percent: f32,
    pub memory_percent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Calm,
    Focused,
    Occupied,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoodDecision {
    pub mood: Mood,
    pub animation: PetState,
}

pub fn combined_pressure(sample: ResourceSample) -> f32 {
    (sample.cpu_percent.clamp(0.0, 100.0) / 100.0) * CPU_WEIGHT
        + (sample.memory_percent.clamp(0.0, 100.0) / 100.0) * MEMORY_WEIGHT
}

#[derive(Debug)]
pub struct MoodEngine {
    baseline: VecDeque<(u64, ResourceSample)>,
    current: Mood,
    candidate: Option<(Mood, u64)>,
    rng: XorShift64,
    next_random_event_at: u64,
}

impl MoodEngine {
    pub fn with_seed(seed: u64) -> Self {
        let mut rng = XorShift64::new(seed);
        let next_random_event_at = next_event_delay(&mut rng);
        Self {
            baseline: VecDeque::new(),
            current: Mood::Calm,
            candidate: None,
            rng,
            next_random_event_at,
        }
    }

    pub fn update(&mut self, now_ms: u64, sample: Result<ResourceSample, ()>) -> MoodDecision {
        let Ok(sample) = sample else {
            self.current = Mood::Calm;
            self.candidate = None;
            return MoodDecision {
                mood: Mood::Calm,
                animation: calm_animation(now_ms),
            };
        };

        self.prune_baseline(now_ms);
        let target = self.target_mood(sample);
        self.baseline.push_back((now_ms, sample));
        self.stabilize(target, now_ms);

        let mut animation = animation_for(self.current, now_ms);
        if now_ms >= self.next_random_event_at {
            self.next_random_event_at = now_ms.saturating_add(next_event_delay(&mut self.rng));
            if matches!(self.current, Mood::Calm | Mood::Focused) {
                animation = if self.rng.next().is_multiple_of(2) {
                    PetState::Waving
                } else {
                    PetState::Jumping
                };
            }
        }
        MoodDecision {
            mood: self.current,
            animation,
        }
    }

    pub fn baseline_len(&self) -> usize {
        self.baseline.len()
    }

    pub fn next_random_event_at(&self) -> u64 {
        self.next_random_event_at
    }

    fn prune_baseline(&mut self, now_ms: u64) {
        let cutoff = now_ms.saturating_sub(BASELINE_WINDOW_MS);
        while self
            .baseline
            .front()
            .is_some_and(|(timestamp, _)| *timestamp < cutoff)
        {
            self.baseline.pop_front();
        }
    }

    fn target_mood(&self, sample: ResourceSample) -> Mood {
        if sample.cpu_percent >= ABSOLUTE_CPU_OVERLOAD
            || sample.memory_percent >= ABSOLUTE_MEMORY_OVERLOAD
        {
            return Mood::Exhausted;
        }

        let absolute = combined_pressure(sample);
        let adaptive = self.adaptive_pressure(sample);
        let score = absolute.max(adaptive);
        let occupied_threshold = if self.current == Mood::Occupied {
            0.57
        } else {
            0.65
        };
        let focused_threshold = if self.current == Mood::Focused {
            0.30
        } else {
            0.38
        };

        if score >= occupied_threshold {
            Mood::Occupied
        } else if score >= focused_threshold {
            Mood::Focused
        } else {
            Mood::Calm
        }
    }

    fn adaptive_pressure(&self, sample: ResourceSample) -> f32 {
        if self.baseline.len() < 3 {
            return 0.0;
        }
        let count = self.baseline.len() as f32;
        let (cpu_sum, memory_sum) =
            self.baseline
                .iter()
                .fold((0.0, 0.0), |(cpu, memory), (_, item)| {
                    (
                        cpu + item.cpu_percent.clamp(0.0, 100.0),
                        memory + item.memory_percent.clamp(0.0, 100.0),
                    )
                });
        let cpu_rise = ((sample.cpu_percent - cpu_sum / count) / 55.0).clamp(0.0, 1.0);
        let memory_rise = ((sample.memory_percent - memory_sum / count) / 45.0).clamp(0.0, 1.0);
        cpu_rise * CPU_WEIGHT + memory_rise * MEMORY_WEIGHT
    }

    fn stabilize(&mut self, target: Mood, now_ms: u64) {
        if target == self.current {
            self.candidate = None;
            return;
        }

        match self.candidate {
            Some((candidate, since)) if candidate == target => {
                let dwell = if target == Mood::Exhausted {
                    EXHAUSTED_DWELL_MS
                } else {
                    MOOD_DWELL_MS
                };
                if now_ms.saturating_sub(since) >= dwell {
                    self.current = target;
                    self.candidate = None;
                }
            }
            _ => self.candidate = Some((target, now_ms)),
        }
    }
}

impl Default for MoodEngine {
    fn default() -> Self {
        Self::with_seed(0xD35C_70A5)
    }
}

fn animation_for(mood: Mood, now_ms: u64) -> PetState {
    match mood {
        Mood::Calm => calm_animation(now_ms),
        Mood::Focused => PetState::Review,
        Mood::Occupied => PetState::Running,
        Mood::Exhausted => PetState::Failed,
    }
}

fn calm_animation(now_ms: u64) -> PetState {
    if (now_ms / 8_000).is_multiple_of(2) {
        PetState::Idle
    } else {
        PetState::Waiting
    }
}

fn next_event_delay(rng: &mut XorShift64) -> u64 {
    RANDOM_EVENT_MIN_MS + rng.next() % (RANDOM_EVENT_MAX_MS - RANDOM_EVENT_MIN_MS + 1)
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
