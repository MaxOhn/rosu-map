use std::{
    num::ParseIntError,
    ops::{BitAnd, BitAndAssign},
    str::FromStr,
};

use self::hit_samples::HitSampleInfo;
pub use self::{
    circle::HitObjectCircle,
    decode::{HitObjects, HitObjectsState, ParseHitObjectsError},
    hold::HitObjectHold,
    slider::{
        curve::{BorrowedCurve, Curve, CurveBuffers},
        event::{SliderEvent, SliderEventType, SliderEventsIter},
        path::{PathControlPoint, SliderPath},
        path_type::{PathType, SplineType},
        HitObjectSlider,
    },
    spinner::HitObjectSpinner,
};

mod circle;
pub(crate) mod decode; // pub(crate) for intradoc-links
mod hold;
mod slider;
mod spinner;

/// Audio-related types.
pub mod hit_samples;

pub(crate) const BASE_SCORING_DIST: f32 = 100.0;

/// A hit object of a [`Beatmap`].
///
/// [`Beatmap`]: crate::beatmap::Beatmap
#[derive(Clone, Debug, PartialEq)]
pub struct HitObject {
    pub start_time: f64,
    pub kind: HitObjectKind,
    pub samples: Vec<HitSampleInfo>,
}

impl HitObject {
    /// Whether the [`HitObject`] starts a new combo.
    pub const fn new_combo(&self) -> bool {
        self.kind.new_combo()
    }

    /// Returns the end time of the [`HitObject`].
    ///
    /// If the curve has not yet been accessed, it needs to be calculated
    /// first.
    ///
    /// In case curves of multiple slider paths are being calculated, it is
    /// recommended to initialize [`CurveBuffers`] and pass a mutable reference
    /// of it to [`HitObject::end_time_with_bufs`] so the buffers are re-used
    /// for all sliders.
    pub fn end_time(&mut self) -> f64 {
        self.end_time_with_bufs(&mut CurveBuffers::default())
    }

    /// Returns the end time of the [`HitObject`].
    ///
    /// If the slider's curve has not yet been accessed, it needs to be
    /// calculated first for which the given [`CurveBuffers`] are used.
    pub fn end_time_with_bufs(&mut self, bufs: &mut CurveBuffers) -> f64 {
        match self.kind {
            HitObjectKind::Circle(_) => self.start_time,
            HitObjectKind::Slider(ref mut h) => self.start_time + h.duration_with_bufs(bufs),
            HitObjectKind::Spinner(ref h) => self.start_time + h.duration,
            HitObjectKind::Hold(ref h) => self.start_time + h.duration,
        }
    }
}

/// Additional data for a [`HitObject`] depending on its type.
#[derive(Clone, Debug, PartialEq)]
pub enum HitObjectKind {
    Circle(HitObjectCircle),
    Slider(HitObjectSlider),
    Spinner(HitObjectSpinner),
    Hold(HitObjectHold),
}

impl HitObjectKind {
    /// Whether the [`HitObjectKind`] starts a new combo.
    pub const fn new_combo(&self) -> bool {
        match self {
            Self::Circle(h) => h.new_combo,
            Self::Slider(h) => h.new_combo,
            Self::Spinner(h) => h.new_combo,
            Self::Hold(_) => false,
        }
    }
}

/// The type of a [`HitObject`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct HitObjectType(i32);

impl HitObjectType {
    pub const CIRCLE: i32 = 1;
    pub const SLIDER: i32 = 1 << 1;
    pub const NEW_COMBO: i32 = 1 << 2;
    pub const SPINNER: i32 = 1 << 3;
    pub const COMBO_OFFSET: i32 = (1 << 4) | (1 << 5) | (1 << 6);
    pub const HOLD: i32 = 1 << 7;

    /// Check whether any of the given bitflags are set.
    pub const fn has_flag(self, flag: i32) -> bool {
        (self.0 & flag) != 0
    }
}

impl From<&HitObject> for HitObjectType {
    fn from(hit_object: &HitObject) -> Self {
        let mut kind = 0;

        match hit_object.kind {
            HitObjectKind::Circle(ref h) => {
                kind |= h.combo_offset << 4;

                if h.new_combo {
                    kind |= Self::NEW_COMBO;
                }

                kind |= Self::CIRCLE;
            }
            HitObjectKind::Slider(ref h) => {
                kind |= h.combo_offset << 4;

                if h.new_combo {
                    kind |= Self::NEW_COMBO;
                }

                kind |= Self::SLIDER;
            }
            HitObjectKind::Spinner(ref h) => {
                if h.new_combo {
                    kind |= Self::NEW_COMBO;
                }

                kind |= Self::SPINNER;
            }
            HitObjectKind::Hold(_) => kind |= Self::HOLD,
        }

        Self(kind)
    }
}

impl FromStr for HitObjectType {
    type Err = ParseHitObjectTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self).map_err(ParseHitObjectTypeError)
    }
}

thiserror! {
    #[error("invalid hit object type")]
    /// Error when failing to parse a [`HitObjectType`].
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ParseHitObjectTypeError(ParseIntError);
}

impl From<HitObjectType> for i32 {
    fn from(kind: HitObjectType) -> Self {
        kind.0
    }
}

impl BitAnd<i32> for HitObjectType {
    type Output = i32;

    fn bitand(self, rhs: i32) -> Self::Output {
        self.0 & rhs
    }
}

impl BitAndAssign<i32> for HitObjectType {
    fn bitand_assign(&mut self, rhs: i32) {
        self.0 &= rhs;
    }
}
