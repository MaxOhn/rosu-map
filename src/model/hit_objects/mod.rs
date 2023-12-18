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

#[derive(Clone, Debug, PartialEq)]
pub enum HitObjectKind {
    Circle(HitObjectCircle),
    Slider(HitObjectSlider),
    Spinner(HitObjectSpinner),
    Hold(HitObjectHold),
}
