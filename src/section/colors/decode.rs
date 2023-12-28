use std::{num::ParseIntError, str::FromStr};

use crate::{
    decode::{DecodeBeatmap, DecodeState},
    reader::DecoderError,
    section::UnknownKeyError,
    util::{KeyValue, ParseNumberError, StrExt},
    {FormatVersion, ParseVersionError},
};

use super::{Color, CustomColor, CustomColors};

/// Struct containing all data from a `.osu` file's `[Colours]` section.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Colors {
    pub custom_combo_colors: Vec<Color>,
    pub custom_colors: CustomColors,
}

/// All valid keys within a `.osu` file's `[Colours]` section
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorsKey {
    Combo,
    Name(String),
}

impl FromStr for ColorsKey {
    type Err = UnknownKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Combo" => Ok(Self::Combo),
            name => Ok(Self::Name(name.to_owned())),
        }
    }
}

/// All the ways that parsing a `.osu` file into [`Colors`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseColorsError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("color specified in incorrect format (should be R,G,B or R,G,B,A)")]
    IncorrectColor,
    #[error("failed to parse number")]
    Number(#[source] ParseNumberError),
}

impl From<ParseIntError> for ParseColorsError {
    fn from(err: ParseIntError) -> Self {
        Self::Number(ParseNumberError::InvalidInteger(err))
    }
}

/// The parsing state for [`Colors`] in [`DecodeBeatmap`].
pub type ColorsState = Colors;

impl DecodeState for ColorsState {
    fn create(_version: FormatVersion) -> Self {
        Self::default()
    }
}

impl DecodeBeatmap for Colors {
    type Error = ParseColorsError;
    type State = ColorsState;

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

    fn parse_events(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_timing_points(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let Ok(KeyValue { key, value }) = KeyValue::parse(line.trim_comment()) else {
            return Ok(());
        };

        let mut split = value.split(',');

        let r = split.next();
        let g = split.next();
        let b = split.next();
        let a = split.next();
        let none = split.next();

        let (r, g, b) = match (r, g, b, a, none) {
            (Some(r), Some(g), Some(b), _, None) => (r.parse()?, g.parse()?, b.parse()?),
            _ => return Err(ParseColorsError::IncorrectColor),
        };

        let color = Color::new(r, g, b, 255);

        match key {
            ColorsKey::Combo => state.custom_combo_colors.push(color),
            ColorsKey::Name(name) => state.custom_colors.insert(CustomColor { name, color }),
        }

        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
