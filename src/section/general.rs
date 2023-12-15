use std::str::FromStr;

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{KeyValue, ParseNumber, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq)]
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

#[derive(Debug, thiserror::Error)]
pub enum ParseGeneralError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("unknown countdown type")]
    UnknownCountdownType,
    #[error("unknown game mode")]
    UnknownGameMode,
    #[error("unknown sample bank")]
    UnknownSampleBank,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SampleBank {
    #[default]
    None,
    Normal,
    Soft,
    Drum,
}

impl FromStr for SampleBank {
    type Err = ParseGeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "None" => Ok(Self::None),
            "1" | "Normal" => Ok(Self::Normal),
            "2" | "Soft" => Ok(Self::Soft),
            "3" | "Drum" => Ok(Self::Drum),
            _ => Err(ParseGeneralError::UnknownSampleBank),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Osu,
    Taiko,
    Catch,
    Mania,
}

impl FromStr for GameMode {
    type Err = ParseGeneralError;

    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "0" => Ok(Self::Osu),
            "1" => Ok(Self::Taiko),
            "2" => Ok(Self::Catch),
            "3" => Ok(Self::Mania),
            _ => Err(ParseGeneralError::UnknownGameMode),
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CountdownType {
    #[default]
    None,
    Normal,
    HalfSpeed,
    DoubleSpeed,
}

impl FromStr for CountdownType {
    type Err = ParseGeneralError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "None" => Ok(Self::None),
            "1" | "Normal" => Ok(Self::Normal),
            "2" | "Half speed" => Ok(Self::HalfSpeed),
            "3" | "Double speed" => Ok(Self::DoubleSpeed),
            _ => Err(ParseGeneralError::UnknownCountdownType),
        }
    }
}

pub struct GeneralState {
    version: FormatVersion,
    general: General,
}

impl ParseState for GeneralState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            general: General::default(),
        }
    }
}

impl From<GeneralState> for General {
    fn from(state: GeneralState) -> Self {
        state.general
    }
}

impl ParseBeatmap for General {
    type ParseError = ParseGeneralError;
    type State = GeneralState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let KeyValue { key, value } = KeyValue::new(line);

        match key {
            "AudioFilename" => state.general.audio_file = value.to_standardized_path(),
            "AudioLeadIn" => state.general.audio_lead_in = i32::parse(value)? as f64,
            "PreviewTime" => {
                let time = i32::parse(value)?;

                state.general.preview_time = if time == -1 {
                    time
                } else {
                    time + state.version.offset()
                };
            }
            "SampleSet" => state.general.default_sample_bank = value.parse()?,
            "SampleVolume" => state.general.default_sample_volume = value.parse_num()?,
            "StackLeniency" => state.general.stack_leniency = value.parse_num()?,
            "Mode" => state.general.mode = value.parse()?,
            "LetterboxInBreaks" => state.general.letterbox_in_breaks = i32::parse(value)? == 1,
            "SpecialStyle" => state.general.special_style = i32::parse(value)? == 1,
            "WidescreenStoryboard" => state.general.widescreen_storyboard = i32::parse(value)? == 1,
            "EpilepsyWarning" => state.general.epilepsy_warning = i32::parse(value)? == 1,
            "SamplesMatchPlaybackRate" => {
                state.general.samples_match_playback_rate = i32::parse(value)? == 1
            }
            "Countdown" => state.general.countdown = value.parse()?,
            "CountdownOffset" => state.general.countdown_offset = value.parse_num()?,
            _ => {}
        }

        Ok(())
    }
}
