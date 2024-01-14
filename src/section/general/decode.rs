use crate::{
    decode::{DecodeBeatmap, DecodeState},
    section::hit_objects::hit_samples::{ParseSampleBankError, SampleBank},
    util::{KeyValue, ParseNumber, ParseNumberError, StrExt},
    Beatmap,
};

use super::{CountdownType, GameMode, ParseCountdownTypeError, ParseGameModeError};

/// Struct containing all data from a `.osu` file's `[General]` section.
#[derive(Clone, Debug, PartialEq)]
pub struct General {
    pub audio_file: String,
    pub audio_lead_in: f64,
    pub preview_time: i32,
    pub default_sample_bank: SampleBank,
    pub default_sample_volume: i32,
    pub stack_leniency: f32,
    pub mode: GameMode,
    pub letterbox_in_breaks: bool,
    pub special_style: bool,
    pub widescreen_storyboard: bool,
    pub epilepsy_warning: bool,
    pub samples_match_playback_rate: bool,
    pub countdown: CountdownType,
    pub countdown_offset: i32,
}

impl Default for General {
    #[allow(clippy::default_trait_access)]
    fn default() -> Self {
        Self {
            audio_file: Default::default(),
            audio_lead_in: Default::default(),
            preview_time: -1,
            default_sample_bank: Default::default(),
            default_sample_volume: 100,
            stack_leniency: 0.7,
            mode: Default::default(),
            letterbox_in_breaks: Default::default(),
            special_style: Default::default(),
            widescreen_storyboard: false,
            epilepsy_warning: Default::default(),
            samples_match_playback_rate: false,
            countdown: CountdownType::Normal,
            countdown_offset: Default::default(),
        }
    }
}

impl From<General> for Beatmap {
    fn from(general: General) -> Self {
        Self {
            audio_file: general.audio_file,
            audio_lead_in: general.audio_lead_in,
            preview_time: general.preview_time,
            default_sample_bank: general.default_sample_bank,
            default_sample_volume: general.default_sample_volume,
            stack_leniency: general.stack_leniency,
            mode: general.mode,
            letterbox_in_breaks: general.letterbox_in_breaks,
            special_style: general.special_style,
            widescreen_storyboard: general.widescreen_storyboard,
            epilepsy_warning: general.epilepsy_warning,
            samples_match_playback_rate: general.samples_match_playback_rate,
            countdown: general.countdown,
            countdown_offset: general.countdown_offset,
            ..Self::default()
        }
    }
}

section_keys! {
    /// All valid keys within a `.osu` file's `[General]` section
    pub enum GeneralKey {
        AudioFilename,
        AudioLeadIn,
        PreviewTime,
        SampleSet,
        SampleVolume,
        StackLeniency,
        Mode,
        LetterboxInBreaks,
        SpecialStyle,
        WidescreenStoryboard,
        EpilepsyWarning,
        SamplesMatchPlaybackRate,
        Countdown,
        CountdownOffset,
    }
}

/// All the ways that parsing a `.osu` file into [`General`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseGeneralError {
    #[error("failed to parse countdown type")]
    CountdownType(#[from] ParseCountdownTypeError),
    #[error("failed to parse mode")]
    Mode(#[from] ParseGameModeError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("failed to parse sample bank")]
    SampleBank(#[from] ParseSampleBankError),
}

/// The parsing state for [`General`] in [`DecodeBeatmap`].
pub type GeneralState = General;

impl DecodeState for GeneralState {
    fn create(_: i32) -> Self {
        Self::default()
    }
}

impl DecodeBeatmap for General {
    type Error = ParseGeneralError;
    type State = GeneralState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let Ok(KeyValue { key, value }) = KeyValue::parse(line.trim_comment()) else {
            return Ok(());
        };

        match key {
            GeneralKey::AudioFilename => state.audio_file = value.to_standardized_path(),
            GeneralKey::AudioLeadIn => state.audio_lead_in = f64::from(i32::parse(value)?),
            GeneralKey::PreviewTime => state.preview_time = i32::parse(value)?,
            GeneralKey::SampleSet => state.default_sample_bank = value.parse()?,
            GeneralKey::SampleVolume => state.default_sample_volume = value.parse_num()?,
            GeneralKey::StackLeniency => state.stack_leniency = value.parse_num()?,
            GeneralKey::Mode => state.mode = value.parse()?,
            GeneralKey::LetterboxInBreaks => state.letterbox_in_breaks = i32::parse(value)? == 1,
            GeneralKey::SpecialStyle => state.special_style = i32::parse(value)? == 1,
            GeneralKey::WidescreenStoryboard => {
                state.widescreen_storyboard = i32::parse(value)? == 1;
            }
            GeneralKey::EpilepsyWarning => state.epilepsy_warning = i32::parse(value)? == 1,
            GeneralKey::SamplesMatchPlaybackRate => {
                state.samples_match_playback_rate = i32::parse(value)? == 1;
            }
            GeneralKey::Countdown => state.countdown = value.parse()?,
            GeneralKey::CountdownOffset => state.countdown_offset = value.parse_num()?,
        }

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

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
