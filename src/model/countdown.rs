use std::str::FromStr;

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

#[derive(Debug, thiserror::Error)]
#[error("invalid countdown type")]
pub struct ParseCountdownTypeError;
