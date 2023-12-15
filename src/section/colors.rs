use std::{
    borrow::Borrow,
    num::ParseIntError,
    ops::{Deref, DerefMut, Index, IndexMut},
    str::FromStr,
};

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{KeyValue, ParseNumberError},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Colors {
    pub custom_combo_colors: Vec<Color>,
    pub custom_colors: CustomColors,
}

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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color([u8; 4]);

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    pub fn red(self) -> u8 {
        self[0]
    }

    pub fn green(self) -> u8 {
        self[1]
    }

    pub fn blue(self) -> u8 {
        self[2]
    }

    pub fn alpha(self) -> u8 {
        self[3]
    }
}

impl Index<usize> for Color {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Color {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl FromStr for Color {
    type Err = ParseColorsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(',');

        let r = split.next();
        let g = split.next();
        let b = split.next();
        let a = split.next();
        let none = split.next();

        let (Some(r), Some(g), Some(b), _, None) = (r, g, b, a, none) else {
            return Err(ParseColorsError::IncorrectColor);
        };

        Ok(Self::new(r.parse()?, g.parse()?, b.parse()?, 255))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
// Likely small enough so a Vec should be faster than a HashMap
pub struct CustomColors(Vec<CustomColor>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomColor {
    pub name: String,
    pub color: Color,
}

impl CustomColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<Q>(&self, name: &Q) -> Option<Color>
    where
        String: Borrow<Q>,
        Q: Eq,
    {
        self.iter()
            .find_map(|color| (color.name.borrow() == name).then_some(color.color))
    }

    pub fn insert(&mut self, color: CustomColor) {
        match self.iter().position(|item| item.name == color.name) {
            Some(idx) => self[idx] = color,
            None => self.push(color),
        }
    }
}

impl Deref for CustomColors {
    type Target = Vec<CustomColor>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CustomColors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type ColorsState = Colors;

impl ParseState for ColorsState {
    fn create(_version: FormatVersion) -> Self {
        Self::default()
    }
}

impl ParseBeatmap for Colors {
    type ParseError = ParseColorsError;
    type State = ColorsState;

    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let KeyValue { key, value } = KeyValue::new(line);
        let mut split = value.split(',');

        let r = split.next();
        let g = split.next();
        let b = split.next();
        let a = split.next();
        let none = split.next();

        let (r, g, b) = match (r, g, b, a, none) {
            (Some(r), Some(g), Some(b), Some(_), None)
            | (Some(r), Some(g), Some(b), None, None) => (r.parse()?, g.parse()?, b.parse()?),
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
}
