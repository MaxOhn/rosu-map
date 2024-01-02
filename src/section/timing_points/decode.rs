use crate::{
    decode::{DecodeBeatmap, DecodeState},
    reader::DecoderError,
    section::{
        general::{GameMode, GeneralKey, ParseGameModeError},
        hit_objects::hit_samples::{ParseSampleBankError, SampleBank},
    },
    util::{KeyValue, ParseNumber, ParseNumberError, StrExt, MAX_PARSE_VALUE},
    {FormatVersion, ParseVersionError},
};

use super::{
    DifficultyPoint, EffectFlags, EffectPoint, ParseEffectFlagsError, SamplePoint, TimeSignature,
    TimeSignatureError, TimingPoint,
};

/// Struct containing all data from a `.osu` file's `[TimingPoints]` section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControlPoints {
    pub timing_points: Vec<TimingPoint>,
    pub difficulty_points: Vec<DifficultyPoint>,
    pub effect_points: Vec<EffectPoint>,
    pub sample_points: Vec<SamplePoint>,
}

impl ControlPoints {
    /// Finds the [`DifficultyPoint`] that is active at the given time.
    pub fn difficulty_point_at(&self, time: f64) -> Option<&DifficultyPoint> {
        self.difficulty_points
            .binary_search_by(|probe| probe.time.total_cmp(&time))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map(|i| &self.difficulty_points[i])
    }

    /// Finds the [`EffectPoint`] that is active at the given time.
    pub fn effect_point_at(&self, time: f64) -> Option<&EffectPoint> {
        self.effect_points
            .binary_search_by(|probe| probe.time.total_cmp(&time))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map(|i| &self.effect_points[i])
    }

    /// Finds the [`SamplePoint`] that is active at the given time.
    pub fn sample_point_at(&self, time: f64) -> Option<&SamplePoint> {
        self.sample_points
            .binary_search_by(|probe| probe.time.total_cmp(&time))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map(|i| &self.sample_points[i])
            .or_else(|| self.sample_points.first())
    }

    /// Finds the [`TimingPoint`] that is active at the given time.
    pub fn timing_point_at(&self, time: f64) -> Option<&TimingPoint> {
        self.timing_points
            .binary_search_by(|probe| probe.time.total_cmp(&time))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map(|i| &self.timing_points[i])
            .or_else(|| self.timing_points.first())
    }

    /// Add a [`ControlPoint`] into its corresponding list.
    pub fn add<P: ControlPoint>(&mut self, point: P) {
        if point.check_already_existing(self) {
            return;
        }

        point.add(self);
    }
}

/// A [`ControlPoint`] to be added into [`ControlPoints`].
pub trait ControlPoint {
    /// Whether `self` is redundant w.r.t. an already existing control point.
    fn check_already_existing(&self, control_points: &ControlPoints) -> bool;

    /// Adding the control point into [`ControlPoints`].
    ///
    /// Note that control points should be inserted in order by time.
    fn add(self, control_points: &mut ControlPoints);
}

impl ControlPoint for TimingPoint {
    fn check_already_existing(&self, _: &ControlPoints) -> bool {
        false
    }

    fn add(self, control_points: &mut ControlPoints) {
        match control_points
            .timing_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
        {
            Err(i) => control_points.timing_points.insert(i, self),
            Ok(i) => control_points.timing_points[i] = self,
        }
    }
}

impl ControlPoint for DifficultyPoint {
    fn check_already_existing(&self, control_points: &ControlPoints) -> bool {
        match control_points.difficulty_point_at(self.time) {
            Some(existing) => self.is_redundant(existing),
            None => self.is_redundant(&DifficultyPoint::default()),
        }
    }

    fn add(self, control_points: &mut ControlPoints) {
        match control_points
            .difficulty_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
        {
            Err(i) => control_points.difficulty_points.insert(i, self),
            Ok(i) => control_points.difficulty_points[i] = self,
        }
    }
}

impl ControlPoint for EffectPoint {
    fn check_already_existing(&self, control_points: &ControlPoints) -> bool {
        match control_points.effect_point_at(self.time) {
            Some(existing) => self.is_redundant(existing),
            None => self.is_redundant(&EffectPoint::default()),
        }
    }

    fn add(self, control_points: &mut ControlPoints) {
        match control_points
            .effect_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
        {
            Err(i) => control_points.effect_points.insert(i, self),
            Ok(i) => control_points.effect_points[i] = self,
        }
    }
}

impl ControlPoint for SamplePoint {
    fn check_already_existing(&self, control_points: &ControlPoints) -> bool {
        control_points
            .sample_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map_or(false, |i| {
                self.is_redundant(&control_points.sample_points[i])
            })
    }

    fn add(self, control_points: &mut ControlPoints) {
        match control_points
            .sample_points
            .binary_search_by(|probe| probe.time.total_cmp(&self.time))
        {
            Err(i) => control_points.sample_points.insert(i, self),
            Ok(i) => control_points.sample_points[i] = self,
        }
    }
}

/// All the ways that parsing a `.osu` file into [`ControlPoints`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseControlPointsError {
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
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("failed to parse sample bank")]
    SampleBank(#[from] ParseSampleBankError),
    #[error("time signature error")]
    TimeSignature(#[from] TimeSignatureError),
    #[error("beat length cannot be NaN in a timing control point")]
    TimingControlPointNaN,
}

/// The parsing state for [`ControlPoints`] in [`DecodeBeatmap`].
pub struct ControlPointsState {
    version: FormatVersion,
    default_sample_bank: SampleBank,
    default_sample_volume: i32,
    mode: GameMode,
    pending_control_points_time: f64,
    pending_timing_point: Option<TimingPoint>,
    pending_difficulty_point: Option<DifficultyPoint>,
    pending_effect_point: Option<EffectPoint>,
    pending_sample_point: Option<SamplePoint>,
    control_points: ControlPoints,
}

trait Pending: Sized {
    fn pending(state: &mut ControlPointsState) -> &mut Option<Self>;

    fn push_front(self, state: &mut ControlPointsState) {
        let pending = Self::pending(state);

        if pending.is_none() {
            *pending = Some(self);
        }
    }

    fn push_back(self, state: &mut ControlPointsState) {
        *Self::pending(state) = Some(self);
    }
}

impl Pending for TimingPoint {
    fn pending(state: &mut ControlPointsState) -> &mut Option<Self> {
        &mut state.pending_timing_point
    }
}

impl Pending for DifficultyPoint {
    fn pending(state: &mut ControlPointsState) -> &mut Option<Self> {
        &mut state.pending_difficulty_point
    }
}

impl Pending for EffectPoint {
    fn pending(state: &mut ControlPointsState) -> &mut Option<Self> {
        &mut state.pending_effect_point
    }
}

impl Pending for SamplePoint {
    fn pending(state: &mut ControlPointsState) -> &mut Option<Self> {
        &mut state.pending_sample_point
    }
}

impl ControlPointsState {
    fn add_control_point<P: Pending>(&mut self, time: f64, point: P, timing_change: bool) {
        if time != self.pending_control_points_time {
            self.flush_pending_points();
        }

        if timing_change {
            point.push_front(self);
        } else {
            point.push_back(self);
        }

        self.pending_control_points_time = time;
    }

    fn flush_pending_points(&mut self) {
        if let Some(point) = self.pending_timing_point.take() {
            self.control_points.add(point);
        }

        if let Some(point) = self.pending_difficulty_point.take() {
            self.control_points.add(point);
        }

        if let Some(point) = self.pending_effect_point.take() {
            self.control_points.add(point);
        }

        if let Some(point) = self.pending_sample_point.take() {
            self.control_points.add(point);
        }
    }
}

impl DecodeState for ControlPointsState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            default_sample_bank: SampleBank::default(),
            default_sample_volume: i32::default(),
            mode: GameMode::default(),
            pending_control_points_time: 0.0,
            pending_timing_point: None,
            pending_difficulty_point: None,
            pending_effect_point: None,
            pending_sample_point: None,
            control_points: ControlPoints::default(),
        }
    }
}

impl From<ControlPointsState> for ControlPoints {
    fn from(mut state: ControlPointsState) -> Self {
        state.flush_pending_points();

        state.control_points
    }
}

impl DecodeBeatmap for ControlPoints {
    type Error = ParseControlPointsError;
    type State = ControlPointsState;

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
            .ok_or(ParseControlPointsError::InvalidLine)?;

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

        if timing_change {
            if beat_len.is_nan() {
                return Err(ParseControlPointsError::TimingControlPointNaN);
            }

            let timing = TimingPoint::new(time, beat_len, omit_first_bar_signature, time_signature);
            state.add_control_point(time, timing, timing_change)
        }

        let difficulty = DifficultyPoint::new(time, beat_len, speed_multiplier);
        state.add_control_point(time, difficulty, timing_change);

        let sample = SamplePoint::new(time, sample_set, sample_volume, custom_sample_bank);
        state.add_control_point(time, sample, timing_change);

        let mut effect = EffectPoint::new(time, kiai_mode);

        if matches!(state.mode, GameMode::Taiko | GameMode::Mania) {
            effect.scroll_speed = speed_multiplier;
        }

        state.add_control_point(time, effect, timing_change);

        state.pending_control_points_time = time;

        Ok(())
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_hit_objects(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
