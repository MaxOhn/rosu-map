use std::{cmp::Ordering, num::NonZeroU32};

#[derive(Clone, Debug, PartialEq)]
/// The time signature at this control point.
pub struct TimingPoint {
    pub time: f64,
    pub beat_len: f64,
    pub omit_first_bar_line: bool,
    pub time_signature: TimeSignature,
}

impl TimingPoint {
    pub const DEFAULT_BEAT_LEN: f64 = 60_000.0 / 60.0;
    pub const DEFAULT_OMIT_FIRST_BAR_LINE: bool = false;
    pub const DEFAULT_TIME_SIGNATURE: TimeSignature = TimeSignature::new_simple_quadruple();

    pub fn new(
        time: f64,
        beat_len: f64,
        omit_first_bar_line: bool,
        time_signature: TimeSignature,
    ) -> Self {
        Self {
            time,
            beat_len: beat_len.clamp(6.0, 60_000.0),
            omit_first_bar_line,
            time_signature,
        }
    }
}

impl Default for TimingPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            beat_len: Self::DEFAULT_BEAT_LEN,
            omit_first_bar_line: Self::DEFAULT_OMIT_FIRST_BAR_LINE,
            time_signature: Self::DEFAULT_TIME_SIGNATURE,
        }
    }
}

impl PartialOrd for TimingPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// The time signature of a track.
pub struct TimeSignature {
    pub numerator: NonZeroU32,
}

impl TimeSignature {
    /// Create a new [`TimeSignature`].
    ///
    /// # Errors
    ///
    /// Returns an error if `numerator` is non-positive.
    pub fn new(numerator: i32) -> Result<Self, TimeSignatureError> {
        u32::try_from(numerator)
            .ok()
            .and_then(NonZeroU32::new)
            .map(|numerator| Self { numerator })
            .ok_or(TimeSignatureError)
    }

    pub const fn new_simple_triple() -> Self {
        Self {
            // SAFETY: 3 != 0
            numerator: unsafe { NonZeroU32::new_unchecked(3) },
        }
    }

    pub const fn new_simple_quadruple() -> Self {
        Self {
            // SAFETY: 4 != 0
            numerator: unsafe { NonZeroU32::new_unchecked(4) },
        }
    }
}

impl TryFrom<i32> for TimeSignature {
    type Error = TimeSignatureError;

    fn try_from(numerator: i32) -> Result<Self, Self::Error> {
        Self::new(numerator)
    }
}

thiserror! {
    #[error("invalid time signature, must be positive integer")]
    /// Error when failing to parse a [`TimeSignature`].
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct TimeSignatureError;
}
