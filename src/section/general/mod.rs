use std::str::FromStr;

pub use self::decode::{General, GeneralKey, GeneralState, ParseGeneralError};

pub(crate) mod decode; // pub(crate) for intradoc-links

/// An osu! gamemode.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Osu,
    Taiko,
    Catch,
    Mania,
}

/// Error when failing to parse a [`GameMode`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid game mode")]
pub struct ParseGameModeError;

impl FromStr for GameMode {
    type Err = ParseGameModeError;

    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "0" => Ok(Self::Osu),
            "1" => Ok(Self::Taiko),
            "2" => Ok(Self::Catch),
            "3" => Ok(Self::Mania),
            _ => Err(ParseGameModeError),
        }
    }
}

impl From<u8> for GameMode {
    fn from(mode: u8) -> Self {
        match mode {
            0 => Self::Osu,
            1 => Self::Taiko,
            2 => Self::Catch,
            3 => Self::Mania,
            _ => Self::Osu,
        }
    }
}

/// The countdown type of a [`Beatmap`].
///
/// [`Beatmap`]: crate::beatmap::Beatmap
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CountdownType {
    #[default]
    None,
    Normal,
    HalfSpeed,
    DoubleSpeed,
}

impl FromStr for CountdownType {
    type Err = ParseCountdownTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "None" => Ok(Self::None),
            "1" | "Normal" => Ok(Self::Normal),
            "2" | "Half speed" => Ok(Self::HalfSpeed),
            "3" | "Double speed" => Ok(Self::DoubleSpeed),
            _ => Err(ParseCountdownTypeError),
        }
    }
}

/// Error when failing to parse a [`CountdownType`].
#[derive(Debug, thiserror::Error)]
#[error("invalid countdown type")]
pub struct ParseCountdownTypeError;
