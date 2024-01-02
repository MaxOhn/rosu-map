use std::{path::Path, str::FromStr};

use crate::{
    decode::{DecodeBeatmap, DecodeState},
    reader::DecoderError,
    section::{
        colors::{Color, Colors, ColorsState, CustomColor, ParseColorsError},
        difficulty::{Difficulty, DifficultyState, ParseDifficultyError},
        editor::{Editor, EditorState, ParseEditorError},
        events::{BreakPeriod, Events, EventsState, ParseEventsError},
        general::{CountdownType, GameMode, General, GeneralState, ParseGeneralError},
        hit_objects::{HitObject, HitObjects, HitObjectsState, ParseHitObjectsError},
        metadata::{Metadata, MetadataState, ParseMetadataError},
        timing_points::{ControlPoints, ControlPointsState, ParseControlPointsError},
    },
    FormatVersion, ParseVersionError,
};

/// Fully parsed content of a `.osu` file.
#[derive(Clone, Debug, PartialEq)]
pub struct Beatmap {
    pub format_version: FormatVersion,

    // General
    pub audio_file: String,
    pub audio_lead_in: f64,
    pub preview_time: i32,
    pub stack_leniency: f32,
    pub mode: GameMode,
    pub letterbox_in_breaks: bool,
    pub special_style: bool,
    pub widescreen_storyboard: bool,
    pub epilepsy_warning: bool,
    pub samples_match_playback_rate: bool,
    pub countdown: CountdownType,
    pub countdown_offset: i32,

    // Editor
    pub bookmarks: Vec<i32>,
    pub distance_spacing: f64,
    pub beat_divisor: i32,
    pub grid_size: i32,
    pub timeline_zoom: f64,

    // Metadata
    pub title: String,
    pub title_unicode: String,
    pub artist: String,
    pub artist_unicode: String,
    pub creator: String,
    pub version: String,
    pub source: String,
    pub tags: String,
    pub beatmap_id: i32,
    pub beatmap_set_id: i32,

    // Difficulty
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,

    // Events
    pub background_file: String,
    pub breaks: Vec<BreakPeriod>,

    // TimingPoints
    pub control_points: ControlPoints,

    // Colors
    pub custom_combo_colors: Vec<Color>,
    pub custom_colors: Vec<CustomColor>,

    // HitObjects
    pub hit_objects: Vec<HitObject>,
}

impl Beatmap {
    /// Parse a [`Beatmap`] by providing a path to a `.osu` file.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseBeatmapError> {
        crate::from_path(path)
    }

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// slice of bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseBeatmapError> {
        crate::from_bytes(bytes)
    }
}

impl FromStr for Beatmap {
    type Err = ParseBeatmapError;

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::from_str(s)
    }
}

impl Default for Beatmap {
    fn default() -> Self {
        let general = General::default();
        let editor = Editor::default();
        let metadata = Metadata::default();
        let difficulty = Difficulty::default();
        let events = Events::default();
        let control_points = ControlPoints::default();
        let colors = Colors::default();
        let hit_objects = HitObjects::default();

        Self {
            format_version: FormatVersion(FormatVersion::LATEST),
            audio_file: general.audio_file,
            audio_lead_in: general.audio_lead_in,
            preview_time: general.preview_time,
            stack_leniency: general.stack_leniency,
            mode: general.mode,
            letterbox_in_breaks: general.letterbox_in_breaks,
            special_style: general.special_style,
            widescreen_storyboard: general.widescreen_storyboard,
            epilepsy_warning: general.epilepsy_warning,
            samples_match_playback_rate: general.samples_match_playback_rate,
            countdown: general.countdown,
            countdown_offset: general.countdown_offset,
            bookmarks: editor.bookmarks,
            distance_spacing: editor.distance_spacing,
            beat_divisor: editor.beat_divisor,
            grid_size: editor.grid_size,
            timeline_zoom: editor.timeline_zoom,
            title: metadata.title,
            title_unicode: metadata.title_unicode,
            artist: metadata.artist,
            artist_unicode: metadata.artist_unicode,
            creator: metadata.creator,
            version: metadata.version,
            source: metadata.source,
            tags: metadata.tags,
            beatmap_id: metadata.beatmap_id,
            beatmap_set_id: metadata.beatmap_set_id,
            hp_drain_rate: difficulty.hp_drain_rate,
            circle_size: difficulty.circle_size,
            overall_difficulty: difficulty.overall_difficulty,
            approach_rate: difficulty.approach_rate,
            slider_multiplier: difficulty.slider_multiplier,
            slider_tick_rate: difficulty.slider_tick_rate,
            background_file: events.background_file,
            breaks: events.breaks,
            control_points,
            custom_combo_colors: colors.custom_combo_colors,
            custom_colors: colors.custom_colors,
            hit_objects: hit_objects.hit_objects,
        }
    }
}

/// All the ways that parsing a `.osu` file into [`Beatmap`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseBeatmapError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse general section")]
    General(#[from] ParseGeneralError),
    #[error("failed to parse editor section")]
    Editor(#[from] ParseEditorError),
    #[error("failed to parse metadata section")]
    Metadata(#[from] ParseMetadataError),
    #[error("failed to parse difficulty section")]
    Difficulty(#[from] ParseDifficultyError),
    #[error("failed to parse events section")]
    Events(#[from] ParseEventsError),
    #[error("failed to parse timing points section")]
    TimingPoints(#[from] ParseControlPointsError),
    #[error("failed to parse colors section")]
    Colors(#[from] ParseColorsError),
    #[error("failed to parse hit objects section")]
    HitOjects(#[from] ParseHitObjectsError),
}

/// The parsing state for [`Beatmap`] in [`DecodeBeatmap`].
pub struct BeatmapState {
    version: FormatVersion,
    general: GeneralState,
    editor: EditorState,
    metadata: MetadataState,
    difficulty: DifficultyState,
    events: EventsState,
    control_points: ControlPointsState,
    colors: ColorsState,
    hit_objects: HitObjectsState,
}

impl DecodeState for BeatmapState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            general: GeneralState::create(version),
            editor: EditorState::create(version),
            metadata: MetadataState::create(version),
            difficulty: DifficultyState::create(version),
            events: EventsState::create(version),
            control_points: ControlPointsState::create(version),
            colors: ColorsState::create(version),
            hit_objects: HitObjectsState::create(version),
        }
    }
}

impl From<BeatmapState> for Beatmap {
    #[allow(clippy::useless_conversion)]
    fn from(state: BeatmapState) -> Self {
        let general: General = state.general.into();
        let editor: Editor = state.editor.into();
        let metadata: Metadata = state.metadata.into();
        let difficulty: Difficulty = state.difficulty.into();
        let events: Events = state.events.into();
        let control_points: ControlPoints = state.control_points.into();
        let colors: Colors = state.colors.into();
        let hit_objects: HitObjects = state.hit_objects.into();

        Beatmap {
            format_version: state.version,
            audio_file: general.audio_file,
            audio_lead_in: general.audio_lead_in,
            preview_time: general.preview_time,
            stack_leniency: general.stack_leniency,
            mode: general.mode,
            letterbox_in_breaks: general.letterbox_in_breaks,
            special_style: general.special_style,
            widescreen_storyboard: general.widescreen_storyboard,
            epilepsy_warning: general.epilepsy_warning,
            samples_match_playback_rate: general.samples_match_playback_rate,
            countdown: general.countdown,
            countdown_offset: general.countdown_offset,
            bookmarks: editor.bookmarks,
            distance_spacing: editor.distance_spacing,
            beat_divisor: editor.beat_divisor,
            grid_size: editor.grid_size,
            timeline_zoom: editor.timeline_zoom,
            title: metadata.title,
            title_unicode: metadata.title_unicode,
            artist: metadata.artist,
            artist_unicode: metadata.artist_unicode,
            creator: metadata.creator,
            version: metadata.version,
            source: metadata.source,
            tags: metadata.tags,
            beatmap_id: metadata.beatmap_id,
            beatmap_set_id: metadata.beatmap_set_id,
            hp_drain_rate: difficulty.hp_drain_rate,
            circle_size: difficulty.circle_size,
            overall_difficulty: difficulty.overall_difficulty,
            approach_rate: difficulty.approach_rate,
            slider_multiplier: difficulty.slider_multiplier,
            slider_tick_rate: difficulty.slider_tick_rate,
            background_file: events.background_file,
            breaks: events.breaks,
            control_points,
            custom_combo_colors: colors.custom_combo_colors,
            custom_colors: colors.custom_colors,
            hit_objects: hit_objects.hit_objects,
        }
    }
}

impl DecodeBeatmap for Beatmap {
    type Error = ParseBeatmapError;
    type State = BeatmapState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        General::parse_general(&mut state.general, line)?;
        ControlPoints::parse_general(&mut state.control_points, line)?;

        Ok(())
    }

    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Editor::parse_editor(&mut state.editor, line).map_err(ParseBeatmapError::Editor)
    }

    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Metadata::parse_metadata(&mut state.metadata, line).map_err(ParseBeatmapError::Metadata)
    }

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Difficulty::parse_difficulty(&mut state.difficulty, line)
            .map_err(ParseBeatmapError::Difficulty)
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_events(&mut state.events, line).map_err(ParseBeatmapError::Events)
    }

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        ControlPoints::parse_timing_points(&mut state.control_points, line)
            .map_err(ParseBeatmapError::TimingPoints)
    }

    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Colors::parse_colors(&mut state.colors, line).map_err(ParseBeatmapError::Colors)
    }

    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        HitObjects::parse_hit_objects(&mut state.hit_objects, line)
            .map_err(ParseBeatmapError::HitOjects)
    }
}
