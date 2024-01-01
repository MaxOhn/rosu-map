use crate::{
    decode::{DecodeBeatmap, DecodeState},
    reader::DecoderError,
    section::{
        general::{GameMode, GeneralKey, ParseGameModeError},
        hit_objects::hit_samples::{ParseSampleBankError, SampleBank},
    },
    util::{KeyValue, ParseNumber, ParseNumberError, SortedVec, StrExt, MAX_PARSE_VALUE},
    {FormatVersion, ParseVersionError},
};

use super::{
    difficulty_point_at, effect_point_at, sample_point_at, timing_point_at, DifficultyPoint,
    EffectFlags, EffectPoint, ParseEffectFlagsError, SamplePoint, TimeSignature,
    TimeSignatureError, TimingPoint,
};

/// Struct containing all data from a `.osu` file's `[TimingPoints]` section.
#[derive(Default)]
pub struct TimingPoints {
    pub timing_points: SortedVec<TimingPoint>,
    pub difficulty_points: SortedVec<DifficultyPoint>,
    pub effect_points: SortedVec<EffectPoint>,
    pub sample_points: SortedVec<SamplePoint>,
}

/// All the ways that parsing a `.osu` file into [`TimingPoints`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseTimingPointsError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
    #[error("failed to parse effect flags")]
    EffectFlags(#[from] ParseEffectFlagsError),
    #[error("failed to parse mode")]
    Mode(#[from] ParseGameModeError),
    #[error("invalid line")]
    InvalidLine,
    #[error("the numerator of a time signature must be positive")]
    InvalidTimeSignature,
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("failed to parse sample bank")]
    SampleBank(#[from] ParseSampleBankError),
    #[error("time signature error")]
    TimeSignature(#[from] TimeSignatureError),
    #[error("beat length cannot be NaN in a timing control point")]
    TimingControlPointNaN,
}

impl TimingPoints {
    /// Finds the [`DifficultyPoint`] that is active at the given time.
    pub fn difficulty_point_at(&self, time: f64) -> Option<&DifficultyPoint> {
        difficulty_point_at(&self.difficulty_points, time)
    }

    /// Finds the [`EffectPoint`] that is active at the given time.
    pub fn effect_point_at(&self, time: f64) -> Option<&EffectPoint> {
        effect_point_at(&self.effect_points, time)
    }

    /// Finds the [`SamplePoint`] that is active at the given time.
    pub fn sample_point_at(&self, time: f64) -> Option<&SamplePoint> {
        sample_point_at(&self.sample_points, time)
    }

    /// Finds the [`TimingPoint`] that is active at the given time.
    pub fn timing_point_at(&self, time: f64) -> Option<&TimingPoint> {
        timing_point_at(&self.timing_points, time)
    }
}

/// The parsing state for [`TimingPoints`] in [`DecodeBeatmap`].
pub struct TimingPointsState {
    version: FormatVersion,
    default_sample_bank: SampleBank,
    default_sample_volume: i32,
    mode: GameMode,
    last_time: f64,
    pending_difficiulty_point: Option<DifficultyPoint>,
    timing_points: TimingPoints,
}

impl DecodeState for TimingPointsState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            default_sample_bank: SampleBank::default(),
            default_sample_volume: i32::default(),
            mode: GameMode::default(),
            last_time: 0.0,
            pending_difficiulty_point: None,
            timing_points: TimingPoints::default(),
        }
    }
}

impl From<TimingPointsState> for TimingPoints {
    fn from(mut state: TimingPointsState) -> Self {
        if let Some(point) = state.pending_difficiulty_point {
            state.timing_points.difficulty_points.push(point);
        }

        state.timing_points
    }
}

impl DecodeBeatmap for TimingPoints {
    type Error = ParseTimingPointsError;
    type State = TimingPointsState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let Ok(KeyValue { key, value }) = KeyValue::parse(line.trim_comment()) else {
            return Ok(());
        };

        match key {
            GeneralKey::SampleSet => state.default_sample_bank = value.parse()?,
            GeneralKey::SampleVolume => state.default_sample_volume = value.parse_num()?,
            GeneralKey::Mode => state.mode = value.parse()?,
            _ => {}
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

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let mut split = line.trim_comment().split(',');

        let (time, beat_len) = split
            .next()
            .zip(split.next())
            .ok_or(ParseTimingPointsError::InvalidLine)?;

        let time = time.trim().parse_num::<f64>()? + f64::from(state.version.offset());

        // Manual `str::parse_num::<f64>` so that NaN does not cause an error
        let beat_len = beat_len
            .trim()
            .parse::<f64>()
            .map_err(ParseNumberError::InvalidFloat)?;

        if beat_len < f64::from(-MAX_PARSE_VALUE) {
            return Err(ParseNumberError::NumberUnderflow.into());
        } else if beat_len > f64::from(MAX_PARSE_VALUE) {
            return Err(ParseNumberError::NumberOverflow.into());
        }

        let speed_multiplier = if beat_len < 0.0 {
            100.0 / -beat_len
        } else {
            1.0
        };

        let mut time_signature = TimeSignature::new_simple_quadruple();

        if let Some(next) = split.next() {
            if !matches!(next.chars().next(), Some('0')) {
                time_signature = TimeSignature::new(next.parse_num()?)?;
            }
        }

        let mut sample_set = split
            .next()
            .map(i32::parse)
            .transpose()?
            .map(SampleBank::try_from)
            .and_then(Result::ok)
            .unwrap_or(state.default_sample_bank);

        let custom_sample_bank = split.next().map(i32::parse).transpose()?.unwrap_or(0);

        let sample_volume = split
            .next()
            .map(i32::parse)
            .transpose()?
            .unwrap_or(state.default_sample_volume);

        let timing_change = split
            .next()
            .map_or(true, |next| matches!(next.chars().next(), Some('1')));

        let mut kiai_mode = false;
        let mut omit_first_bar_signature = false;

        if let Some(next) = split.next() {
            let effect_flags: EffectFlags = next.parse()?;
            kiai_mode = effect_flags.has_flag(EffectFlags::KIAI);
            omit_first_bar_signature = effect_flags.has_flag(EffectFlags::OMIT_FIRST_BAR_LINE);
        }

        if sample_set == SampleBank::None {
            sample_set = SampleBank::Normal;
        }

        if (time - state.last_time).abs() < f64::EPSILON {
            if let Some(point) = state.pending_difficiulty_point.take() {
                state.timing_points.difficulty_points.push(point);
            }
        }

        if timing_change {
            if beat_len.is_nan() {
                return Err(ParseTimingPointsError::TimingControlPointNaN);
            }

            let timing_point =
                TimingPoint::new(time, beat_len, omit_first_bar_signature, time_signature);

            state.timing_points.timing_points.push(timing_point);
        }

        if !timing_change || state.pending_difficiulty_point.is_none() {
            state.pending_difficiulty_point =
                Some(DifficultyPoint::new(time, beat_len, speed_multiplier));
        }

        let mut effect_point = EffectPoint::new(time, kiai_mode);

        if matches!(state.mode, GameMode::Taiko | GameMode::Mania) {
            effect_point.scroll_speed = speed_multiplier;
        }

        state.timing_points.effect_points.push(effect_point);

        let sample_point = SamplePoint::new(time, sample_set, sample_volume, custom_sample_bank);

        state.timing_points.sample_points.push(sample_point);

        state.last_time = time;

        Ok(())
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
