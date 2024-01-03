use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
/// Difficulty-related info about this control point.
pub struct DifficultyPoint {
    pub time: f64,
    pub bpm_multiplier: f64,
    pub slider_velocity: f64,
    pub generate_ticks: bool,
}

impl DifficultyPoint {
    pub const DEFAULT_SLIDER_VELOCITY: f64 = 1.0;
    pub const DEFAULT_BPM_MULTIPLIER: f64 = 1.0;
    pub const DEFAULT_GENERATE_TICKS: bool = true;

    pub fn new(time: f64, beat_len: f64, speed_multiplier: f64) -> Self {
        let bpm_multiplier = if beat_len < 0.0 {
            f64::from(((-beat_len) as f32).clamp(10.0, 10_000.0)) / 100.0
        } else {
            1.0
        };

        Self {
            time,
            slider_velocity: speed_multiplier.clamp(0.1, 10.0),
            bpm_multiplier,
            generate_ticks: !beat_len.is_nan(),
        }
    }

    pub fn is_redundant(&self, existing: &Self) -> bool {
        self.generate_ticks == existing.generate_ticks
            && (self.slider_velocity - existing.slider_velocity).abs() < f64::EPSILON
    }
}

impl Default for DifficultyPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            bpm_multiplier: Self::DEFAULT_BPM_MULTIPLIER,
            slider_velocity: Self::DEFAULT_SLIDER_VELOCITY,
            generate_ticks: Self::DEFAULT_GENERATE_TICKS,
        }
    }
}

impl PartialOrd for DifficultyPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
