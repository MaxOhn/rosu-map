use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{KeyValue, ParseNumber, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Difficulty {
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseDifficultyError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

pub struct DifficultyState {
    has_approach_rate: bool,
    difficulty: Difficulty,
}

impl ParseState for DifficultyState {
    fn create(_version: FormatVersion) -> Self {
        Self {
            has_approach_rate: false,
            difficulty: Difficulty::default(),
        }
    }
}

impl From<DifficultyState> for Difficulty {
    fn from(state: DifficultyState) -> Self {
        state.difficulty
    }
}

impl ParseBeatmap for Difficulty {
    type ParseError = ParseDifficultyError;
    type State = DifficultyState;

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let KeyValue { key, value } = KeyValue::new(line);

        match key {
            "HPDrainRate" => state.difficulty.hp_drain_rate = value.parse_num()?,
            "CircleSize" => state.difficulty.circle_size = value.parse_num()?,
            "OverallDifficulty" => {
                state.difficulty.overall_difficulty = value.parse_num()?;

                if !state.has_approach_rate {
                    state.difficulty.approach_rate = state.difficulty.overall_difficulty;
                }
            }
            "ApproachRate" => {
                state.difficulty.approach_rate = value.parse_num()?;
                state.has_approach_rate = true;
            }
            "SliderMultiplier" => {
                state.difficulty.slider_multiplier = f32::parse(value)?.clamp(0.4, 3.6)
            }
            "SliderTickRate" => {
                state.difficulty.slider_tick_rate = f32::parse(value)?.clamp(0.5, 8.0)
            }
            _ => {}
        }

        Ok(())
    }
}
