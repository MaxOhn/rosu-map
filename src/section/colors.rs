use std::num::ParseIntError;

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    model::colors::{Color, CustomColor, CustomColors},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{KeyValue, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Colors {
    pub custom_combo_colors: Vec<Color>,
    pub custom_colors: CustomColors,
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

/// The parsing state for [`Colors`] in [`ParseBeatmap`].
pub type ColorsState = Colors;

impl ParseState for ColorsState {
    fn create(_version: FormatVersion) -> Self {
        Self::default()
    }
}

impl ParseBeatmap for Colors {
    type ParseError = ParseColorsError;
    type State = ColorsState;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
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

    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let KeyValue { key, value } = KeyValue::new(line.trim_comment());
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
        let is_combo = key.starts_with("Combo");

        if is_combo {
            state.custom_combo_colors.push(color);
        } else {
            let color = CustomColor {
                name: key.to_owned(),
                color,
            };

            state.custom_colors.insert(color);
        }

        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }
}
