use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub struct EffectPoint {
    pub time: f64,
    pub kiai: bool,
    pub scroll_speed: f64,
}

impl EffectPoint {
    pub const DEFAULT_KIAI: bool = false;
    pub const DEFAULT_SCROLL_SPEED: f64 = 1.0;

    pub const fn new(time: f64, kiai: bool) -> Self {
        Self {
            time,
            kiai,
            scroll_speed: Self::DEFAULT_SCROLL_SPEED,
        }
    }

    pub fn is_redundant(&self, existing: &Self) -> bool {
        self.kiai == existing.kiai
            && (self.scroll_speed - existing.scroll_speed).abs() < f64::EPSILON
    }
}

impl Default for EffectPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            kiai: Self::DEFAULT_KIAI,
            scroll_speed: Self::DEFAULT_SCROLL_SPEED,
        }
    }
}

impl PartialOrd for EffectPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
