use std::{
    fs::File,
    io::{BufReader, Cursor, Error as IoError},
    path::Path,
    str::FromStr,
};

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    section::{
        colors::{Color, Colors, ColorsState, CustomColors, ParseColorsError},
        difficulty::{Difficulty, DifficultyState, ParseDifficultyError},
        editor::{Editor, EditorState, ParseEditorError},
        events::{BreakPeriod, Events, EventsState, ParseEventsError},
        general::{CountdownType, GameMode, General, GeneralState, ParseGeneralError},
        hit_objects::{HitObject, HitObjects, HitObjectsState, ParseHitObjectsError},
        metadata::{Metadata, MetadataState, ParseMetadataError},
        timing_points::{ParseTimingPointsError, TimingPoints, TimingPointsState},
    },
};

#[derive(Debug, Default)]
pub struct Beatmap {
    pub format_version: i32,

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
    _a: (),

    // Colours
    pub custom_combo_colors: Vec<Color>,
    pub custom_colors: CustomColors,

    // HitObjects
    pub hit_objects: Vec<HitObject>,
}

impl Beatmap {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ParseBeatmapError> {
        let file = File::open(path).map_err(ParseBeatmapError::OpenFile)?;

        Self::parse(BufReader::new(file))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseBeatmapError> {
        Self::parse(Cursor::new(bytes))
    }
}

impl FromStr for Beatmap {
    type Err = ParseBeatmapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(Cursor::new(s))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseBeatmapError {
    #[error("failed to open file")]
    OpenFile(#[source] IoError),
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
    TimingPoints(#[from] ParseTimingPointsError),
    #[error("failed to parse colors section")]
    Colors(#[from] ParseColorsError),
    #[error("failed to parse hit objects section")]
    HitOjects(#[from] ParseHitObjectsError),
}

pub struct BeatmapState {
    version: FormatVersion,
    general: GeneralState,
    editor: EditorState,
    metadata: MetadataState,
    difficulty: DifficultyState,
    events: EventsState,
    timing_points: TimingPointsState,
    colors: ColorsState,
    hit_objects: HitObjectsState,
}

impl ParseState for BeatmapState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            general: GeneralState::create(version),
            editor: EditorState::create(version),
            metadata: MetadataState::create(version),
            difficulty: DifficultyState::create(version),
            events: EventsState::create(version),
            timing_points: TimingPointsState::create(version),
            colors: ColorsState::create(version),
            hit_objects: HitObjectsState::create(version),
        }
    }
}

impl From<BeatmapState> for Beatmap {
    fn from(state: BeatmapState) -> Self {
        let general: General = state.general.into();
        let editor: Editor = state.editor.into();
        let metadata: Metadata = state.metadata.into();
        let difficulty: Difficulty = state.difficulty.into();
        let events: Events = state.events.into();
        let timing_points: TimingPoints = state.timing_points.into();
        let colors: Colors = state.colors.into();
        let hit_objects: HitObjects = state.hit_objects.into();

        Beatmap {
            format_version: state.version.0,
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
            _a: todo!(),
            custom_combo_colors: colors.custom_combo_colors,
            custom_colors: colors.custom_colors,
            hit_objects: hit_objects.hit_objects,
        }
    }
}

impl ParseBeatmap for Beatmap {
    type ParseError = ParseBeatmapError;
    type State = BeatmapState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        General::parse_general(&mut state.general, line).map_err(ParseBeatmapError::General)
    }

    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        Editor::parse_editor(&mut state.editor, line).map_err(ParseBeatmapError::Editor)
    }

    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        Metadata::parse_metadata(&mut state.metadata, line).map_err(ParseBeatmapError::Metadata)
    }

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        Difficulty::parse_difficulty(&mut state.difficulty, line)
            .map_err(ParseBeatmapError::Difficulty)
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        Events::parse_events(&mut state.events, line).map_err(ParseBeatmapError::Events)
    }

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        TimingPoints::parse_timing_points(&mut state.timing_points, line)
            .map_err(ParseBeatmapError::TimingPoints)
    }

    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        Colors::parse_colors(&mut state.colors, line).map_err(ParseBeatmapError::Colors)
    }

    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        HitObjects::parse_hit_objects(&mut state.hit_objects, line)
            .map_err(ParseBeatmapError::HitOjects)
    }
}
