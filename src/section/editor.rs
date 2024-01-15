use crate::{
    decode::{DecodeBeatmap, DecodeState},
    util::{KeyValue, ParseNumberError, StrExt},
    Beatmap,
};

/// Struct containing all data from a `.osu` file's `[Editor]` section.
#[derive(Clone, Debug, PartialEq)]
pub struct Editor {
    pub bookmarks: Vec<i32>,
    pub distance_spacing: f64,
    pub beat_divisor: i32,
    pub grid_size: i32,
    pub timeline_zoom: f64,
}

impl Default for Editor {
    #[allow(clippy::default_trait_access)]
    fn default() -> Self {
        Self {
            bookmarks: Default::default(),
            distance_spacing: 1.0,
            beat_divisor: 4,
            grid_size: Default::default(),
            timeline_zoom: 1.0,
        }
    }
}

impl From<Editor> for Beatmap {
    fn from(editor: Editor) -> Self {
        Self {
            bookmarks: editor.bookmarks,
            distance_spacing: editor.distance_spacing,
            beat_divisor: editor.beat_divisor,
            grid_size: editor.grid_size,
            timeline_zoom: editor.timeline_zoom,
            ..Self::default()
        }
    }
}

section_keys! {
    /// All valid keys within a `.osu` file's `[Editor]` section
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
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

/// The parsing state for [`Editor`] in [`DecodeBeatmap`].
pub type EditorState = Editor;

impl DecodeState for EditorState {
    fn create(_: i32) -> Self {
        Self::default()
    }
}

impl DecodeBeatmap for Editor {
    type Error = ParseEditorError;
    type State = EditorState;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
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

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_difficulty(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_events(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
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
