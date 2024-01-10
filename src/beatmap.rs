use std::{io, path::Path, str::FromStr};

use crate::{
    decode::{DecodeBeatmap, DecodeState},
    section::{
        colors::{Color, Colors, ColorsState, CustomColor, ParseColorsError},
        editor::{Editor, EditorState, ParseEditorError},
        events::BreakPeriod,
        general::{CountdownType, GameMode},
        hit_objects::{HitObject, HitObjects, HitObjectsState, ParseHitObjectsError},
        metadata::{Metadata, MetadataState, ParseMetadataError},
        timing_points::ControlPoints,
    },
    FormatVersion,
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
    pub slider_multiplier: f64,
    pub slider_tick_rate: f64,

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
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rosu_map::Beatmap;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/path/to/file.osu";
    /// let map: Beatmap = Beatmap::from_path(path)?;
    /// # Ok(()) }
    /// ```
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        crate::from_path(path)
    }

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// slice of bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rosu_map::Beatmap;
    /// use rosu_map::section::general::GameMode;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bytes: &[u8] = b"[General]
    /// Mode: 2
    ///
    /// [Metadata]
    /// Creator: pishifat";
    ///
    /// let map: Beatmap = Beatmap::from_bytes(bytes)?;
    /// assert_eq!(map.mode, GameMode::Catch);
    /// assert_eq!(map.creator, "pishifat");
    /// # Ok(()) }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, io::Error> {
        crate::from_bytes(bytes)
    }
}

impl FromStr for Beatmap {
    type Err = io::Error;

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// string.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rosu_map::Beatmap;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let s: &str = "[Difficulty]
    /// SliderMultiplier: 3
    ///
    /// [Editor]
    /// BeatDivisor: 4";
    ///
    /// let map: Beatmap = s.parse()?; // same as `Beatmap::from_str(s)`
    /// # let _ = <Beatmap as std::str::FromStr>::from_str(s).unwrap();
    /// assert_eq!(map.slider_multiplier, 3.0);
    /// assert_eq!(map.beat_divisor, 4);
    /// # Ok(()) }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::from_str(s)
    }
}

impl Default for Beatmap {
    fn default() -> Self {
        let editor = Editor::default();
        let metadata = Metadata::default();
        let colors = Colors::default();
        let hit_objects = HitObjects::default();

        Self {
            format_version: FormatVersion::default(),
            audio_file: hit_objects.audio_file,
            audio_lead_in: hit_objects.audio_lead_in,
            preview_time: hit_objects.preview_time,
            stack_leniency: hit_objects.stack_leniency,
            mode: hit_objects.mode,
            letterbox_in_breaks: hit_objects.letterbox_in_breaks,
            special_style: hit_objects.special_style,
            widescreen_storyboard: hit_objects.widescreen_storyboard,
            epilepsy_warning: hit_objects.epilepsy_warning,
            samples_match_playback_rate: hit_objects.samples_match_playback_rate,
            countdown: hit_objects.countdown,
            countdown_offset: hit_objects.countdown_offset,
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
            hp_drain_rate: hit_objects.hp_drain_rate,
            circle_size: hit_objects.circle_size,
            overall_difficulty: hit_objects.overall_difficulty,
            approach_rate: hit_objects.approach_rate,
            slider_multiplier: hit_objects.slider_multiplier,
            slider_tick_rate: hit_objects.slider_tick_rate,
            background_file: hit_objects.background_file,
            breaks: hit_objects.breaks,
            control_points: hit_objects.control_points,
            custom_combo_colors: colors.custom_combo_colors,
            custom_colors: colors.custom_colors,
            hit_objects: hit_objects.hit_objects,
        }
    }
}

/// All the ways that parsing a `.osu` file into [`Beatmap`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseBeatmapError {
    #[error("failed to parse colors section")]
    Colors(#[from] ParseColorsError),
    #[error("failed to parse editor section")]
    Editor(#[from] ParseEditorError),
    #[error("failed to parse hit objects")]
    HitOjects(#[from] ParseHitObjectsError),
    #[error("failed to parse metadata section")]
    Metadata(#[from] ParseMetadataError),
}

/// The parsing state for [`Beatmap`] in [`DecodeBeatmap`].
pub struct BeatmapState {
    pub version: FormatVersion,
    pub editor: EditorState,
    pub metadata: MetadataState,
    pub colors: ColorsState,
    pub hit_objects: HitObjectsState,
}

impl DecodeState for BeatmapState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            editor: EditorState::create(version),
            metadata: MetadataState::create(version),
            colors: ColorsState::create(version),
            hit_objects: HitObjectsState::create(version),
        }
    }
}

impl From<BeatmapState> for Beatmap {
    #[allow(clippy::useless_conversion)]
    fn from(state: BeatmapState) -> Self {
        let editor: Editor = state.editor.into();
        let metadata: Metadata = state.metadata.into();
        let colors: Colors = state.colors.into();
        let hit_objects: HitObjects = state.hit_objects.into();

        Beatmap {
            format_version: state.version,
            audio_file: hit_objects.audio_file,
            audio_lead_in: hit_objects.audio_lead_in,
            preview_time: hit_objects.preview_time,
            stack_leniency: hit_objects.stack_leniency,
            mode: hit_objects.mode,
            letterbox_in_breaks: hit_objects.letterbox_in_breaks,
            special_style: hit_objects.special_style,
            widescreen_storyboard: hit_objects.widescreen_storyboard,
            epilepsy_warning: hit_objects.epilepsy_warning,
            samples_match_playback_rate: hit_objects.samples_match_playback_rate,
            countdown: hit_objects.countdown,
            countdown_offset: hit_objects.countdown_offset,
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
            hp_drain_rate: hit_objects.hp_drain_rate,
            circle_size: hit_objects.circle_size,
            overall_difficulty: hit_objects.overall_difficulty,
            approach_rate: hit_objects.approach_rate,
            slider_multiplier: hit_objects.slider_multiplier,
            slider_tick_rate: hit_objects.slider_tick_rate,
            background_file: hit_objects.background_file,
            breaks: hit_objects.breaks,
            control_points: hit_objects.control_points,
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
        HitObjects::parse_general(&mut state.hit_objects, line)
            .map_err(ParseBeatmapError::HitOjects)
    }

    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Editor::parse_editor(&mut state.editor, line).map_err(ParseBeatmapError::Editor)
    }

    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Metadata::parse_metadata(&mut state.metadata, line).map_err(ParseBeatmapError::Metadata)
    }

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        HitObjects::parse_difficulty(&mut state.hit_objects, line)
            .map_err(ParseBeatmapError::HitOjects)
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        HitObjects::parse_events(&mut state.hit_objects, line).map_err(ParseBeatmapError::HitOjects)
    }

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        HitObjects::parse_timing_points(&mut state.hit_objects, line)
            .map_err(ParseBeatmapError::HitOjects)
    }

    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Colors::parse_colors(&mut state.colors, line).map_err(ParseBeatmapError::Colors)
    }

    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        HitObjects::parse_hit_objects(&mut state.hit_objects, line)
            .map_err(ParseBeatmapError::HitOjects)
    }
}
