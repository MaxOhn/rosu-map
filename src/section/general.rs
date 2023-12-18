use crate::{
    format_version::{FormatVersion, ParseVersionError},
    model::{
        countdown::{CountdownType, ParseCountdownTypeError},
        hit_samples::{ParseSampleBankError, SampleBank},
        mode::{GameMode, ParseGameModeError},
    },
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

/// All the ways that parsing a `.osu` file into [`General`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseGeneralError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse countdown type")]
    CountdownType(#[from] ParseCountdownTypeError),
    #[error("failed to parse mode")]
    Mode(#[from] ParseGameModeError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("failed to parse sample bank")]
    SampleBank(#[from] ParseSampleBankError),
}

/// The parsing state for [`General`] in [`ParseBeatmap`].
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
        let KeyValue { key, value } = KeyValue::new(line.trim_comment());

        match key {
            "AudioFilename" => state.general.audio_file = value.to_standardized_path(),
            "AudioLeadIn" => state.general.audio_lead_in = f64::from(i32::parse(value)?),
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
                state.general.samples_match_playback_rate = i32::parse(value)? == 1;
            }
            "Countdown" => state.general.countdown = value.parse()?,
            "CountdownOffset" => state.general.countdown_offset = value.parse_num()?,
            _ => {}
        }

        Ok(())
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_difficulty(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_events(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_timing_points(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
        Ok(())
    }
}
