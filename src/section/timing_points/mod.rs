use std::{num::ParseIntError, str::FromStr};

pub use self::{
    control_points::{
        difficulty::DifficultyPoint,
        effect::EffectPoint,
        sample::SamplePoint,
        timing::{TimeSignature, TimeSignatureError, TimingPoint},
    },
    decode::{ControlPoints, ParseTimingPointsError, TimingPoints, TimingPointsState},
};

mod control_points;
mod decode;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EffectFlags(u8);

impl EffectFlags {
    pub const NONE: u8 = 0;
    pub const KIAI: u8 = 1 << 0;
    pub const OMIT_FIRST_BAR_LINE: u8 = 1 << 3;

    /// Check whether any of the given bitflags are set.
    pub const fn has_flag(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }
}

impl From<EffectFlags> for u8 {
    fn from(kind: EffectFlags) -> Self {
        kind.0
    }
}

impl FromStr for EffectFlags {
    type Err = ParseEffectFlagsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self).map_err(ParseEffectFlagsError)
    }
}

/// Error type for a failed parsing of [`EffectFlags`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid effect flags")]
pub struct ParseEffectFlagsError(#[source] ParseIntError);
