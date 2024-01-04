use crate::util::Pos;

use self::path::{PathControlPoint, SliderPath};

use super::{hit_samples::HitSampleInfo, CurveBuffers};

pub mod curve;
pub mod event;
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

    /// Returns the duration of the slider.
    ///
    /// If the curve has not yet been accessed, it needs to be calculated
    /// first.
    ///
    /// In case curves of multiple slider paths are being calculated, it is
    /// recommended to initialize [`CurveBuffers`] and pass a mutable reference
    /// of it to [`HitObjectSlider::duration_with_bufs`] so the buffers are
    /// re-used for all sliders.
    pub fn duration(&mut self) -> f64 {
        self.duration_with_bufs(&mut CurveBuffers::default())
    }

    /// Returns the duration of the slider.
    ///
    /// If the slider's curve has not yet been accessed, it needs to be
    /// calculated first for which the given [`CurveBuffers`] are used.
    pub fn duration_with_bufs(&mut self, bufs: &mut CurveBuffers) -> f64 {
        f64::from(self.span_count()) * self.path.curve_with_bufs(bufs).dist() / self.velocity
    }
}
