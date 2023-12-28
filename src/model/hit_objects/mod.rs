use std::{
    num::ParseIntError,
    ops::{BitAnd, BitAndAssign},
    str::FromStr,
};

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

/// The type of hit object.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct HitObjectType(i32);

impl HitObjectType {
    pub const CIRCLE: i32 = 1;
    pub const SLIDER: i32 = 1 << 1;
    pub const NEW_COMBO: i32 = 1 << 2;
    pub const SPINNER: i32 = 1 << 3;
    pub const COMBO_OFFSET: i32 = (1 << 4) | (1 << 5) | (1 << 6);
    pub const HOLD: i32 = 1 << 7;

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

/// Error type for a failed parsing of [`HitObjectType`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid hit object type")]
pub struct ParseHitObjectTypeError(#[source] ParseIntError);

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
