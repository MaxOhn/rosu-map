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

#[derive(Debug, thiserror::Error)]
pub enum ParseEditorError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

pub type EditorState = Editor;

impl ParseState for EditorState {
    fn create(_version: FormatVersion) -> Self {
        Self::default()
    }
}

impl ParseBeatmap for Editor {
    type ParseError = ParseEditorError;
    type State = EditorState;

    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let KeyValue { key, value } = KeyValue::new(line);

        match key {
            "Bookmarks" => {
                state.bookmarks = value
                    .split(',')
                    .map(str::parse)
                    .flat_map(Result::ok)
                    .collect()
            }
            "DistanceSpacing" => state.distance_spacing = value.parse_num()?,
            "BeatDivisor" => state.beat_divisor = value.parse_num()?,
            "GridSize" => state.grid_size = value.parse_num()?,
            "TimelineZoom" => state.timeline_zoom = value.parse_num()?,
            _ => {}
        }

        Ok(())
    }
}
