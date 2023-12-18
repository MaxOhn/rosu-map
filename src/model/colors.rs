use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut, Index, IndexMut},
    str::FromStr,
};

use crate::section::colors::ParseColorsError;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color([u8; 4]);

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
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

    #[allow(clippy::many_single_char_names)]
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
        match self.iter_mut().find(|item| item.name == color.name) {
            Some(old) => *old = color,
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
