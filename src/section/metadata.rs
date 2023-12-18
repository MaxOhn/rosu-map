use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{KeyValue, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

/// All the ways that parsing a `.osu` file into [`Metadata`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseMetadataError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

/// The parsing state for [`Metadata`] in [`ParseBeatmap`].
pub type MetadataState = Metadata;

impl ParseState for MetadataState {
    fn create(_version: FormatVersion) -> Self {
        Self::default()
    }
}

impl ParseBeatmap for Metadata {
    type ParseError = ParseMetadataError;
    type State = Self;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let KeyValue { key, value } = KeyValue::new(line);

        match key {
            "Title" => state.title = value.to_owned(),
            "TitleUnicode" => state.title_unicode = value.to_owned(),
            "Artist" => state.artist = value.to_owned(),
            "ArtistUnicode" => state.artist_unicode = value.to_owned(),
            "Creator" => state.creator = value.to_owned(),
            "Version" => state.version = value.to_owned(),
            "Source" => state.source = value.to_owned(),
            "Tags" => state.tags = value.to_owned(),
            "BeatmapID" => state.beatmap_id = value.parse_num()?,
            "BeatmapSetID" => state.beatmap_set_id = value.parse_num()?,
            _ => {}
        }

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
