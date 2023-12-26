use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{KeyValue, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Editor {
    pub bookmarks: Vec<i32>,
    pub distance_spacing: f64,
    pub beat_divisor: i32,
    pub grid_size: i32,
    pub timeline_zoom: f64,
}

section_keys! {
    pub enum EditorKey {
        Bookmarks,
        DistanceSpacing,
        BeatDivisor,
        GridSize,
        TimelineZoom,
    }
}

/// All the ways that parsing a `.osu` file into [`Editor`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseEditorError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

/// The parsing state for [`Editor`] in [`ParseBeatmap`].
pub type EditorState = Editor;

impl ParseState for EditorState {
    fn create(_version: FormatVersion) -> Self {
        Self::default()
    }
}

impl ParseBeatmap for Editor {
    type ParseError = ParseEditorError;
    type State = EditorState;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let Ok(KeyValue { key, value }) = KeyValue::parse(line.trim_comment()) else {
            return Ok(());
        };

        match key {
            EditorKey::Bookmarks => {
                state.bookmarks = value
                    .split(',')
                    .map(str::parse)
                    .filter_map(Result::ok)
                    .collect();
            }
            EditorKey::DistanceSpacing => state.distance_spacing = value.parse_num()?,
            EditorKey::BeatDivisor => state.beat_divisor = value.parse_num()?,
            EditorKey::GridSize => state.grid_size = value.parse_num()?,
            EditorKey::TimelineZoom => state.timeline_zoom = value.parse_num()?,
        }

        Ok(())
    }

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_difficulty(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_events(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
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
