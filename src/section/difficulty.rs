use crate::{
    decode::{DecodeBeatmap, DecodeState},
    util::{KeyValue, ParseNumber, ParseNumberError, StrExt},
    Beatmap,
};

/// Struct containing all data from a `.osu` file's `[Difficulty]` section.
#[derive(Clone, Debug, PartialEq)]
pub struct Difficulty {
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f64,
    pub slider_tick_rate: f64,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self {
            hp_drain_rate: 5.0,
            circle_size: 5.0,
            overall_difficulty: 5.0,
            approach_rate: 5.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,
        }
    }
}

impl From<Difficulty> for Beatmap {
    fn from(difficulty: Difficulty) -> Self {
        Self {
            hp_drain_rate: difficulty.hp_drain_rate,
            circle_size: difficulty.circle_size,
            overall_difficulty: difficulty.overall_difficulty,
            approach_rate: difficulty.approach_rate,
            slider_multiplier: difficulty.slider_multiplier,
            slider_tick_rate: difficulty.slider_tick_rate,
            ..Self::default()
        }
    }
}

section_keys! {
    /// All valid keys within a `.osu` file's `[Difficulty]` section
    pub enum DifficultyKey {
        HPDrainRate,
        CircleSize,
        OverallDifficulty,
        ApproachRate,
        SliderMultiplier,
        SliderTickRate,
    }
}

/// All the ways that parsing a `.osu` file into [`Difficulty`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseDifficultyError {
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}

/// The parsing state for [`Difficulty`] in [`DecodeBeatmap`].
pub struct DifficultyState {
    pub has_approach_rate: bool,
    pub difficulty: Difficulty,
}

impl DecodeState for DifficultyState {
    fn create(_: i32) -> Self {
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

impl DecodeBeatmap for Difficulty {
    type Error = ParseDifficultyError;
    type State = DifficultyState;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let Ok(KeyValue { key, value }) = KeyValue::parse(line.trim_comment()) else {
            return Ok(());
        };

        match key {
            DifficultyKey::HPDrainRate => state.difficulty.hp_drain_rate = value.parse_num()?,
            DifficultyKey::CircleSize => state.difficulty.circle_size = value.parse_num()?,
            DifficultyKey::OverallDifficulty => {
                state.difficulty.overall_difficulty = value.parse_num()?;

                if !state.has_approach_rate {
                    state.difficulty.approach_rate = state.difficulty.overall_difficulty;
                }
            }
            DifficultyKey::ApproachRate => {
                state.difficulty.approach_rate = value.parse_num()?;
                state.has_approach_rate = true;
            }
            DifficultyKey::SliderMultiplier => {
                state.difficulty.slider_multiplier = f64::parse(value)?.clamp(0.4, 3.6);
            }
            DifficultyKey::SliderTickRate => {
                state.difficulty.slider_tick_rate = f64::parse(value)?.clamp(0.5, 8.0);
            }
        }

        Ok(())
    }

    fn parse_events(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_timing_points(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_variables(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_catch_the_beat(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_mania(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
