use std::str::FromStr;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Osu,
    Taiko,
    Catch,
    Mania,
}

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
