use std::{num::ParseIntError, str::FromStr};

use crate::{
    decode::{DecodeBeatmap, DecodeState},
    section::UnknownKeyError,
    util::{KeyValue, ParseNumberError, StrExt},
    Beatmap, FormatVersion,
};

use super::{Color, CustomColor};

/// Struct containing all data from a `.osu` file's `[Colours]` section.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Colors {
    pub custom_combo_colors: Vec<Color>,
    pub custom_colors: Vec<CustomColor>,
}

impl From<Colors> for Beatmap {
    fn from(colors: Colors) -> Self {
        Self {
            custom_combo_colors: colors.custom_combo_colors,
            custom_colors: colors.custom_colors,
            ..Self::default()
        }
    }
}

impl Colors {
    pub const DEFAULT_COMBO_COLORS: [Color; 4] = [
        Color([255, 192, 0, 255]),
        Color([0, 202, 0, 255]),
        Color([18, 124, 255, 255]),
        Color([242, 24, 57, 255]),
    ];
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
        if s.starts_with("Combo") {
            Ok(Self::Combo)
        } else {
            Ok(Self::Name(s.to_owned()))
        }
    }
}

/// All the ways that parsing a `.osu` file into [`Colors`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseColorsError {
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

        let color: Color = value.parse()?;

        match key {
            ColorsKey::Combo => state.custom_combo_colors.push(color),
            ColorsKey::Name(name) => {
                match state.custom_colors.iter_mut().find(|c| c.name == name) {
                    Some(old) => old.color = color,
                    None => state.custom_colors.push(CustomColor { name, color }),
                }
            }
        }

        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
