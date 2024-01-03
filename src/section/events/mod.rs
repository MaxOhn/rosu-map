use std::str::FromStr;

pub use self::decode::{Events, EventsState, ParseEventsError};

mod decode;

/// A break section during a [`Beatmap`].
///
/// [`Beatmap`]: crate::beatmap::Beatmap
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct BreakPeriod {
    pub start_time: f64,
    pub end_time: f64,
}

impl BreakPeriod {
    /// The minimum duration required for a break to have any effect.
    pub const MIN_BREAK_DURATION: f64 = 650.0;

    /// The duration of the break.
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Whether the break has any effect.
    pub fn has_effect(&self) -> bool {
        self.duration() >= Self::MIN_BREAK_DURATION
    }
}

/// The type of an event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    Background,
    Video,
    Break,
    Color,
    Sprite,
    Sample,
    Animation,
}

impl FromStr for EventType {
    type Err = ParseEventTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "Background" => Ok(Self::Background),
            "1" | "Video" => Ok(Self::Video),
            "2" | "Break" => Ok(Self::Break),
            "3" | "Colour" => Ok(Self::Color),
            "4" | "Sprite" => Ok(Self::Sprite),
            "5" | "Sample" => Ok(Self::Sample),
            "6" | "Animation" => Ok(Self::Animation),
            _ => Err(ParseEventTypeError),
        }
    }
}

/// Error when failing to parse an [`EventType`].
#[derive(Debug, thiserror::Error)]
#[error("invalid event type")]
pub struct ParseEventTypeError;
