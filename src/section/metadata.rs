use crate::{
    decode::{DecodeBeatmap, DecodeState},
    util::{KeyValue, ParseNumberError, StrExt},
    FormatVersion,
};

/// Struct containing all data from a `.osu` file's `[Metadata]` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
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
}

impl Default for Metadata {
    #[allow(clippy::default_trait_access)]
    fn default() -> Self {
        Self {
            title: Default::default(),
            title_unicode: Default::default(),
            artist: Default::default(),
            artist_unicode: Default::default(),
            creator: Default::default(),
            version: Default::default(),
            source: Default::default(),
            tags: Default::default(),
            beatmap_id: -1,
            beatmap_set_id: Default::default(),
        }
    }
}

section_keys! {
    /// All valid keys within a `.osu` file's `[Metadata]` section
    pub enum MetadataKey {
        Title,
        TitleUnicode,
        Artist,
        ArtistUnicode,
        Creator,
        Version,
        Source,
        Tags,
        BeatmapID,
        BeatmapSetID,
    }
}

/// All the ways that parsing a `.osu` file into [`Metadata`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseMetadataError {
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

/// The parsing state for [`Metadata`] in [`DecodeBeatmap`].
pub type MetadataState = Metadata;

impl DecodeState for MetadataState {
    fn create(_version: FormatVersion) -> Self {
        Self::default()
    }
}

impl DecodeBeatmap for Metadata {
    type Error = ParseMetadataError;
    type State = Self;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let Ok(KeyValue { key, value }) = KeyValue::parse(line) else {
            return Ok(());
        };

        match key {
            MetadataKey::Title => state.title = value.to_owned(),
            MetadataKey::TitleUnicode => state.title_unicode = value.to_owned(),
            MetadataKey::Artist => state.artist = value.to_owned(),
            MetadataKey::ArtistUnicode => state.artist_unicode = value.to_owned(),
            MetadataKey::Creator => state.creator = value.to_owned(),
            MetadataKey::Version => state.version = value.to_owned(),
            MetadataKey::Source => state.source = value.to_owned(),
            MetadataKey::Tags => state.tags = value.to_owned(),
            MetadataKey::BeatmapID => state.beatmap_id = value.parse_num()?,
            MetadataKey::BeatmapSetID => state.beatmap_set_id = value.parse_num()?,
        }

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
}
