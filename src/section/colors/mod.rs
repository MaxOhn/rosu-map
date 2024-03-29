use std::{
    ops::{Index, IndexMut},
    str::FromStr,
};

pub use self::decode::{Colors, ColorsKey, ColorsState, ParseColorsError};

pub(crate) mod decode; // pub(crate) for intradoc-links

/// Basic RGBA color.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);

impl Color {
    /// Initialize a new color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    /// Get the red value.
    pub fn red(self) -> u8 {
        self[0]
    }

    /// Get the green value.
    pub fn green(self) -> u8 {
        self[1]
    }

    /// Get the blue value.
    pub fn blue(self) -> u8 {
        self[2]
    }

    /// Get the alpha value.
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

impl From<[u8; 4]> for Color {
    fn from(color: [u8; 4]) -> Self {
        Self(color)
    }
}

impl FromStr for Color {
    type Err = ParseColorsError;

    #[allow(clippy::many_single_char_names)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(',').map(str::trim);

        let r = split.next();
        let g = split.next();
        let b = split.next();
        let none = split.nth(1);

        let (Some(r), Some(g), Some(b), None) = (r, g, b, none) else {
            return Err(ParseColorsError::IncorrectColor);
        };

        Ok(Self::new(r.parse()?, g.parse()?, b.parse()?, 255))
    }
}

/// A combination of a [`Color`] and a name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomColor {
    pub name: String,
    pub color: Color,
}
