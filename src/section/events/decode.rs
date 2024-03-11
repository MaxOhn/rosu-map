use crate::{
    decode::{DecodeBeatmap, DecodeState},
    util::{ParseNumber, ParseNumberError, StrExt},
    Beatmap,
};

use super::{BreakPeriod, EventType, ParseEventTypeError};

/// Struct containing all data from a `.osu` file's `[Events]` section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Events {
    pub background_file: String,
    pub breaks: Vec<BreakPeriod>,
}

impl From<Events> for Beatmap {
    fn from(events: Events) -> Self {
        Self {
            background_file: events.background_file,
            breaks: events.breaks,
            ..Self::default()
        }
    }
}

thiserror! {
    /// All the ways that parsing a `.osu` file into [`Events`] can fail.
    #[derive(Debug)]
    pub enum ParseEventsError {
        #[error("failed to parse event type")]
        EventType(#[from] ParseEventTypeError),
        #[error("invalid line")]
        InvalidLine,
        #[error("failed to parse number")]
        Number(#[from] ParseNumberError),
    }
}

/// The parsing state for [`Events`] in [`DecodeBeatmap`].
pub type EventsState = Events;

impl DecodeState for EventsState {
    fn create(_: i32) -> Self {
        Self::default()
    }
}

impl DecodeBeatmap for Events {
    type Error = ParseEventsError;
    type State = EventsState;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_difficulty(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let mut split = line.trim_comment().split(',');

        let (Some(event_type), Some(start_time), Some(event_params)) =
            (split.next(), split.next(), split.next())
        else {
            return Err(ParseEventsError::InvalidLine);
        };

        match event_type.parse()? {
            EventType::Sprite => {
                if state.background_file.is_empty() {
                    state.background_file = split
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
                        state.background_file = filename;
                    }
                }
            }
            EventType::Background => state.background_file = event_params.clean_filename(),
            EventType::Break => {
                let start_time = f64::parse(start_time)?;
                let end_time = start_time.max(f64::parse(event_params)?);

                state.breaks.push(BreakPeriod {
                    start_time,
                    end_time,
                });
            }
            EventType::Color | EventType::Sample | EventType::Animation => {}
        }

        Ok(())
    }

    fn parse_timing_points(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_variables(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_catch_the_beat(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_mania(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
