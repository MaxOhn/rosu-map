use std::str::FromStr;

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{ParseNumber, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Events {
    pub background_file: String,
    pub breaks: Vec<BreakPeriod>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseEventsError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("invalid line")]
    InvalidLine,
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("unknown event type")]
    UnknownEventType,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct BreakPeriod {
    pub start_time: f64,
    pub end_time: f64,
}

enum EventType {
    Background,
    Video,
    Break,
    Color,
    Sprite,
    Sample,
    Animation,
}

impl FromStr for EventType {
    type Err = ParseEventsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "Background" => Ok(Self::Background),
            "1" | "Video" => Ok(Self::Video),
            "2" | "Break" => Ok(Self::Break),
            "3" | "Colour" => Ok(Self::Color),
            "4" | "Sprite" => Ok(Self::Sprite),
            "5" | "Sample" => Ok(Self::Sample),
            "6" | "Animation" => Ok(Self::Animation),
            _ => Err(ParseEventsError::UnknownEventType),
        }
    }
}

pub struct EventsState {
    version: FormatVersion,
    events: Events,
}

impl ParseState for EventsState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            events: Events::default(),
        }
    }
}

impl From<EventsState> for Events {
    fn from(state: EventsState) -> Self {
        state.events
    }
}

impl ParseBeatmap for Events {
    type ParseError = ParseEventsError;
    type State = EventsState;

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let mut split = line.split(',');

        let (Some(event_type), Some(start_time), Some(event_params)) =
            (split.next(), split.next(), split.next())
        else {
            return Err(ParseEventsError::InvalidLine);
        };

        match event_type.parse()? {
            EventType::Sprite => {
                if state.events.background_file.is_empty() {
                    state.events.background_file = split
                        .next()
                        .ok_or(ParseEventsError::InvalidLine)?
                        .clean_filename();
                }
            }
            EventType::Video => {
                const VIDEO_EXTENSIONS: &[[u8; 3]] = &[
                    *b"mp4", *b"mov", *b"avi", *b"flv", *b"mpg", *b"wmv", *b"m4v",
                ];

                let filename = event_params.clean_filename();

                if let [.., a, b, c] = filename.as_bytes() {
                    let extension = [
                        a.to_ascii_lowercase(),
                        b.to_ascii_lowercase(),
                        c.to_ascii_lowercase(),
                    ];

                    if !VIDEO_EXTENSIONS.contains(&extension) {
                        state.events.background_file = filename;
                    }
                }
            }
            EventType::Background => state.events.background_file = event_params.clean_filename(),
            EventType::Break => {
                let offset = state.version.offset() as f64;
                let start_time = f64::parse(start_time)? + offset;
                let end_time = start_time.max(f64::parse(event_params)? + offset);

                state.events.breaks.push(BreakPeriod {
                    start_time,
                    end_time,
                });
            }
            EventType::Color => {}
            EventType::Sample => {}
            EventType::Animation => {}
        }

        Ok(())
    }
}
