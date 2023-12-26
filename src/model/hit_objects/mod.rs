use super::hit_samples::HitSampleInfo;

use self::slider::HitObjectSlider;

pub use self::{circle::HitObjectCircle, hold::HitObjectHold, spinner::HitObjectSpinner};

mod circle;
mod hold;
pub mod slider;
mod spinner;

#[derive(Clone, Debug, PartialEq)]
pub struct HitObject {
    pub start_time: f64,
    pub kind: HitObjectKind,
    pub samples: Vec<HitSampleInfo>,
}

impl HitObject {
    pub fn end_time(&self) -> f64 {
        match self.kind {
            HitObjectKind::Circle(_) => self.start_time,
            HitObjectKind::Slider(ref h) => todo!(),
            HitObjectKind::Spinner(ref h) => self.start_time + h.duration,
            HitObjectKind::Hold(ref h) => self.start_time + h.duration,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HitObjectKind {
    Circle(HitObjectCircle),
    Slider(HitObjectSlider),
    Spinner(HitObjectSpinner),
    Hold(HitObjectHold),
}
