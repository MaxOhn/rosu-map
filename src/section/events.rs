use crate::{
    format_version::{FormatVersion, ParseVersionError},
    model::events::{BreakPeriod, EventType, ParseEventTypeError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{ParseNumber, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Events {
    pub background_file: String,
    pub breaks: Vec<BreakPeriod>,
}

/// All the ways that parsing a `.osu` file into [`Events`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseEventsError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse event type")]
    EventType(#[from] ParseEventTypeError),
    #[error("invalid line")]
    InvalidLine,
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

/// The parsing state for [`Events`] in [`ParseBeatmap`].
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

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_difficulty(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let mut split = line.trim_comment().split(',');

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
                let offset = f64::from(state.version.offset());
                let start_time = f64::parse(start_time)? + offset;
                let end_time = start_time.max(f64::parse(event_params)? + offset);

                state.events.breaks.push(BreakPeriod {
                    start_time,
                    end_time,
                });
            }
            EventType::Color | EventType::Sample | EventType::Animation => {}
        }

        Ok(())
    }

    fn parse_timing_points(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }
}
