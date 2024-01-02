use crate::util::Pos;

use self::path::{PathControlPoint, SliderPath};

use super::hit_samples::HitSampleInfo;

pub mod curve;
pub mod path;
pub mod path_type;

/// A slider [`HitObject`].
///
/// [`HitObject`]: crate::section::hit_objects::HitObject
#[derive(Clone, Debug, PartialEq)]
pub struct HitObjectSlider {
    pub pos: Pos,
    pub new_combo: bool,
    pub combo_offset: i32,
    pub path: SliderPath,
    pub node_samples: Vec<Vec<HitSampleInfo>>,
    pub repeat_count: i32,
    pub velocity: f64,
}

impl HitObjectSlider {
    pub const fn span_count(&self) -> i32 {
        self.repeat_count + 1
    }

    pub fn duration(&mut self) -> f64 {
        self.span_count() as f64 * self.path.curve().dist() / self.velocity
    }
}
