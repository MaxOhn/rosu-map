use std::str::FromStr;

pub use self::decode::{Events, EventsState, ParseEventsError};

mod decode;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct BreakPeriod {
    pub start_time: f64,
    pub end_time: f64,
}

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

#[derive(Debug, thiserror::Error)]
#[error("invalid event type")]
pub struct ParseEventTypeError;
