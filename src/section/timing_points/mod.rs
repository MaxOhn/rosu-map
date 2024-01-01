use std::{num::ParseIntError, str::FromStr};

use crate::util::SortedVec;

pub use self::{
    control_points::{
        difficulty::DifficultyPoint,
        effect::EffectPoint,
        sample::SamplePoint,
        timing::{TimeSignature, TimeSignatureError, TimingPoint},
    },
    decode::{ParseTimingPointsError, TimingPoints, TimingPointsState},
};

mod control_points;
mod decode;

/// Finds the [`DifficultyPoint`] that is active at the given time.
pub fn difficulty_point_at(
    difficulty_points: &SortedVec<DifficultyPoint>,
    time: f64,
) -> Option<&DifficultyPoint> {
    difficulty_points
        .binary_search_by(|probe| probe.time.total_cmp(&time))
        .map_or(None, |i| Some(&difficulty_points[i]))
}

/// Finds the [`EffectPoint`] that is active at the given time.
pub fn effect_point_at(effect_points: &SortedVec<EffectPoint>, time: f64) -> Option<&EffectPoint> {
    effect_points
        .binary_search_by(|probe| probe.time.total_cmp(&time))
        .map_or(None, |i| Some(&effect_points[i]))
}

/// Finds the [`SamplePoint`] that is active at the given time.
pub fn sample_point_at(sample_points: &SortedVec<SamplePoint>, time: f64) -> Option<&SamplePoint> {
    sample_points
        .binary_search_by(|probe| probe.time.total_cmp(&time))
        .map_or_else(|_| sample_points.get(0), |i| Some(&sample_points[i]))
}

/// Finds the [`TimingPoint`] that is active at the given time.
pub fn timing_point_at(timing_points: &SortedVec<TimingPoint>, time: f64) -> Option<&TimingPoint> {
    timing_points
        .binary_search_by(|probe| probe.time.total_cmp(&time))
        .map_or_else(|_| timing_points.get(0), |i| Some(&timing_points[i]))
}

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
