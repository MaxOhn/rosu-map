use std::{cmp, slice};

use crate::{
    decode::{DecodeBeatmap, DecodeState},
    section::{
        difficulty::{Difficulty, DifficultyState, ParseDifficultyError},
        events::{BreakPeriod, Events, EventsState, ParseEventsError},
        general::{CountdownType, GameMode},
        hit_objects::{slider::path_type::PathType, CurveBuffers},
        timing_points::{
            ControlPoints, DifficultyPoint, ParseTimingPointsError, SamplePoint, TimingPoint,
            TimingPoints, TimingPointsState,
        },
    },
    util::{ParseNumber, ParseNumberError, Pos, StrExt},
    FormatVersion,
};

use super::{
    hit_samples::{
        HitSoundType, ParseHitSoundTypeError, ParseSampleBankInfoError, SampleBank, SampleBankInfo,
    },
    HitObject, HitObjectCircle, HitObjectHold, HitObjectKind, HitObjectSlider, HitObjectSpinner,
    HitObjectType, ParseHitObjectTypeError, PathControlPoint, SliderPath,
};

/// Struct containing all data from a `.osu` file's `[HitObjects]`, `[Events]`,
/// `[Difficulty]`, `[TimingPoints]` and `[General]` section.
#[derive(Clone, Debug, PartialEq)]
pub struct HitObjects {
    // General
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

    // Difficulty
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,

    // Events
    pub background_file: String,
    pub breaks: Vec<BreakPeriod>,

    // TimingPoints
    pub control_points: ControlPoints,

    // HitObjects
    pub hit_objects: Vec<HitObject>,
}

impl Default for HitObjects {
    fn default() -> Self {
        let difficulty = Difficulty::default();
        let events = Events::default();
        let timing_points = TimingPoints::default();

        Self {
            audio_file: timing_points.audio_file,
            audio_lead_in: timing_points.audio_lead_in,
            preview_time: timing_points.preview_time,
            default_sample_bank: timing_points.default_sample_bank,
            default_sample_volume: timing_points.default_sample_volume,
            stack_leniency: timing_points.stack_leniency,
            mode: timing_points.mode,
            letterbox_in_breaks: timing_points.letterbox_in_breaks,
            special_style: timing_points.special_style,
            widescreen_storyboard: timing_points.widescreen_storyboard,
            epilepsy_warning: timing_points.epilepsy_warning,
            samples_match_playback_rate: timing_points.samples_match_playback_rate,
            countdown: timing_points.countdown,
            countdown_offset: timing_points.countdown_offset,
            hp_drain_rate: difficulty.hp_drain_rate,
            circle_size: difficulty.circle_size,
            overall_difficulty: difficulty.overall_difficulty,
            approach_rate: difficulty.approach_rate,
            slider_multiplier: difficulty.slider_multiplier,
            slider_tick_rate: difficulty.slider_tick_rate,
            background_file: events.background_file,
            breaks: events.breaks,
            control_points: timing_points.control_points,
            hit_objects: Vec::default(),
        }
    }
}

/// All the ways that parsing a `.osu` file into [`HitObjects`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseHitObjectsError {
    #[error("failed to parse difficulty section")]
    Difficulty(ParseDifficultyError),
    #[error("failed to parse events section")]
    Events(#[from] ParseEventsError),
    #[error("failed to parse hit object type")]
    HitObjectType(#[from] ParseHitObjectTypeError),
    #[error("failed to parse hit sound type")]
    HitSoundType(#[from] ParseHitSoundTypeError),
    #[error("invalid line")]
    InvalidLine,
    #[error("repeat count is way too high")]
    InvalidRepeatCount(i32),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("invalid sample bank")]
    SampleBankInfo(#[from] ParseSampleBankInfoError),
    #[error("failed to parse timing points")]
    TimingPoints(#[from] ParseTimingPointsError),
    #[error("unknown hit object type")]
    UnknownHitObjectType(HitObjectType),
}

/// The parsing state for [`HitObjects`] in [`DecodeBeatmap`].
pub struct HitObjectsState {
    pub first_object: bool,
    pub last_object: Option<HitObjectType>,
    pub curve_points: Vec<PathControlPoint>,
    pub vertices: Vec<PathControlPoint>,
    pub events: EventsState,
    pub timing_points: TimingPointsState,
    pub difficulty: DifficultyState,
    pub hit_objects: Vec<HitObject>,
    point_split: Vec<*const str>,
}

impl HitObjectsState {
    pub const fn difficulty(&self) -> &Difficulty {
        &self.difficulty.difficulty
    }

    /// Processes the point string of a slider hit object.
    fn convert_path_str(
        &mut self,
        point_str: &str,
        offset: Pos,
    ) -> Result<(), ParseHitObjectsError> {
        let f = |this: &mut Self, point_split: &[&str]| {
            let mut start_idx = 0;
            let mut end_idx = 0;
            let mut first = true;

            while {
                end_idx += 1;

                end_idx < point_split.len()
            } {
                let is_letter = point_split[end_idx]
                    .chars()
                    .next()
                    .ok_or(ParseHitObjectsError::InvalidLine)?
                    .is_ascii_alphabetic();

                if !is_letter {
                    continue;
                }

                let end_point = point_split.get(end_idx + 1).copied();
                this.convert_points(&point_split[start_idx..end_idx], end_point, first, offset)?;

                start_idx = end_idx;
                first = false;
            }

            if end_idx > start_idx {
                this.convert_points(&point_split[start_idx..end_idx], None, first, offset)?;
            }

            Ok(())
        };

        self.point_split(point_str.split('|'), f)
    }

    /// Process a slice of points and store them in internal buffers.
    fn convert_points(
        &mut self,
        points: &[&str],
        end_point: Option<&str>,
        first: bool,
        offset: Pos,
    ) -> Result<(), ParseHitObjectsError> {
        fn read_point(
            value: &str,
            start_pos: Pos,
        ) -> Result<PathControlPoint, ParseHitObjectsError> {
            let mut v = value
                .split(':')
                .map(|s| s.parse_with_limits(f64::from(MAX_COORDINATE_VALUE)));

            let (x, y) = v
                .next()
                .zip(v.next())
                .ok_or(ParseHitObjectsError::InvalidLine)?;

            let pos = Pos::new(x? as i32 as f32, y? as i32 as f32);

            Ok(PathControlPoint::new(pos - start_pos))
        }

        fn is_linear(p0: Pos, p1: Pos, p2: Pos) -> bool {
            ((p1.y - p0.y) * (p2.x - p0.x) - (p1.x - p0.x) * (p2.y - p0.y)).abs() < f32::EPSILON
        }

        let mut path_type = points
            .first()
            .copied()
            .map(PathType::new_from_str)
            .ok_or(ParseHitObjectsError::InvalidLine)?;

        let read_offset = usize::from(first);
        let readable_points = points.len() - 1;
        let end_point_len = usize::from(end_point.is_some());

        self.vertices.clear();
        self.vertices
            .reserve(read_offset + readable_points + end_point_len);

        if first {
            self.vertices.push(PathControlPoint::default());
        }

        for &point in points.iter().skip(1) {
            self.vertices.push(read_point(point, offset)?);
        }

        if let Some(end_point) = end_point {
            self.vertices.push(read_point(end_point, offset)?);
        }

        if path_type == PathType::PERFECT_CURVE {
            if let [a, b, c] = self.vertices.as_slice() {
                if is_linear(a.pos, b.pos, c.pos) {
                    path_type = PathType::LINEAR;
                }
            } else {
                path_type = PathType::BEZIER;
            }
        }

        self.vertices[0].path_type = Some(path_type);

        let mut start_idx = 0;
        let mut end_idx = 0;

        while {
            end_idx += 1;

            end_idx < self.vertices.len() - end_point_len
        } {
            if self.vertices[end_idx].pos != self.vertices[end_idx - 1].pos {
                continue;
            }

            if path_type == PathType::CATMULL && end_idx > 1 {
                continue;
            }

            if end_idx == self.vertices.len() - end_point_len - 1 {
                continue;
            }

            self.vertices[end_idx - 1].path_type = Some(path_type);

            self.curve_points.extend(&self.vertices[start_idx..end_idx]);

            start_idx = end_idx + 1;
        }

        if end_idx > start_idx {
            self.curve_points.extend(&self.vertices[start_idx..end_idx]);
        }

        Ok(())
    }

    /// Whether the last object was a spinner.
    fn last_object_was_spinner(&self) -> bool {
        self.last_object
            .is_some_and(|kind| kind.has_flag(HitObjectType::SPINNER))
    }

    /// Given a `&str` iterator, this method prepares a slice and provides
    /// that slice to the given function `f`.
    ///
    /// Instead of collecting a `&str` iterator into a new vec each time,
    /// this method re-uses the same buffer to avoid allocations.
    ///
    /// It is a safe abstraction around transmuting the `point_split` field.
    fn point_split<'a, I, F, O>(&mut self, point_split: I, f: F) -> O
    where
        I: Iterator<Item = &'a str>,
        F: FnOnce(&mut Self, &[&'a str]) -> O,
    {
        self.point_split.extend(point_split.map(|s| s as *const _));
        let ptr = self.point_split.as_ptr();
        let len = self.point_split.len();

        // SAFETY:
        // - *const str and &str have the same layout.
        // - `self.point_split` is cleared after every use, ensuring that it
        //   does not contain any invalid pointers.
        let point_split = unsafe { slice::from_raw_parts(ptr.cast(), len) };
        let res = f(self, point_split);
        self.point_split.clear();

        res
    }

    fn post_process_breaks(hit_objects: &mut [HitObject], events: &Events) {
        let mut curr_break = 0;
        let mut force_new_combo = false;

        for h in hit_objects.iter_mut() {
            while curr_break < events.breaks.len()
                && events.breaks[curr_break].end_time < h.start_time
            {
                force_new_combo = true;
                curr_break += 1;
            }

            match h.kind {
                HitObjectKind::Circle(ref mut h) => h.new_combo |= force_new_combo,
                HitObjectKind::Slider(ref mut h) => h.new_combo |= force_new_combo,
                HitObjectKind::Spinner(ref mut h) => h.new_combo |= force_new_combo,
                HitObjectKind::Hold(_) => {}
            }

            force_new_combo = false;
        }
    }
}

impl DecodeState for HitObjectsState {
    fn create(version: FormatVersion) -> Self {
        Self {
            first_object: true,
            last_object: None,
            curve_points: Vec::new(),
            vertices: Vec::new(),
            point_split: Vec::new(),
            events: EventsState::create(version),
            timing_points: TimingPointsState::create(version),
            difficulty: DifficultyState::create(version),
            hit_objects: Vec::new(),
        }
    }
}

impl From<HitObjectsState> for HitObjects {
    fn from(state: HitObjectsState) -> Self {
        const CONTROL_POINT_LENIENCY: f64 = 5.0;

        let difficulty: Difficulty = state.difficulty.into();
        let events: Events = state.events.into();
        let timing_points: TimingPoints = state.timing_points.into();

        let mut hit_objects = state.hit_objects;
        hit_objects.sort_by(|a, b| a.start_time.total_cmp(&b.start_time));

        HitObjectsState::post_process_breaks(&mut hit_objects, &events);
        let mut bufs = CurveBuffers::default();

        for h in hit_objects.iter_mut() {
            if let HitObjectKind::Slider(ref mut slider) = h.kind {
                const BASE_SCORING_DIST: f32 = 100.0;

                let beat_len = timing_points
                    .control_points
                    .timing_point_at(h.start_time)
                    .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len);

                let slider_velocity = timing_points
                    .control_points
                    .difficulty_point_at(h.start_time)
                    .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                        point.slider_velocity
                    });

                let scoring_dist = f64::from(BASE_SCORING_DIST)
                    * f64::from(difficulty.slider_multiplier)
                    * slider_velocity;

                slider.velocity = scoring_dist / beat_len;

                let span_count = f64::from(slider.span_count());
                let duration = slider.duration_with_bufs(&mut bufs);

                for i in 0..slider.node_samples.len() {
                    let time =
                        h.start_time + i as f64 * duration / span_count + CONTROL_POINT_LENIENCY;

                    let node_sample_point = timing_points
                        .control_points
                        .sample_point_at(time)
                        .map_or_else(SamplePoint::default, SamplePoint::clone);

                    for sample in slider.node_samples[i].iter_mut() {
                        node_sample_point.apply(sample);
                    }
                }
            }

            let end_time = h.end_time_with_bufs(&mut bufs);

            let sample_point = timing_points
                .control_points
                .sample_point_at(end_time + CONTROL_POINT_LENIENCY)
                .map_or_else(SamplePoint::default, SamplePoint::clone);

            for sample in h.samples.iter_mut() {
                sample_point.apply(sample);
            }
        }

        Self {
            audio_file: timing_points.audio_file,
            audio_lead_in: timing_points.audio_lead_in,
            preview_time: timing_points.preview_time,
            default_sample_bank: timing_points.default_sample_bank,
            default_sample_volume: timing_points.default_sample_volume,
            stack_leniency: timing_points.stack_leniency,
            mode: timing_points.mode,
            letterbox_in_breaks: timing_points.letterbox_in_breaks,
            special_style: timing_points.special_style,
            widescreen_storyboard: timing_points.widescreen_storyboard,
            epilepsy_warning: timing_points.epilepsy_warning,
            samples_match_playback_rate: timing_points.samples_match_playback_rate,
            countdown: timing_points.countdown,
            countdown_offset: timing_points.countdown_offset,
            hp_drain_rate: difficulty.hp_drain_rate,
            circle_size: difficulty.circle_size,
            overall_difficulty: difficulty.overall_difficulty,
            approach_rate: difficulty.approach_rate,
            slider_multiplier: difficulty.slider_multiplier,
            slider_tick_rate: difficulty.slider_tick_rate,
            background_file: events.background_file,
            breaks: events.breaks,
            control_points: timing_points.control_points,
            hit_objects,
        }
    }
}

const MAX_COORDINATE_VALUE: i32 = 131_072;

impl DecodeBeatmap for HitObjects {
    type Error = ParseHitObjectsError;
    type State = HitObjectsState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        TimingPoints::parse_general(&mut state.timing_points, line)
            .map_err(ParseHitObjectsError::TimingPoints)
    }

    fn parse_editor(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_metadata(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Difficulty::parse_difficulty(&mut state.difficulty, line)
            .map_err(ParseHitObjectsError::Difficulty)
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_events(&mut state.events, line).map_err(ParseHitObjectsError::Events)
    }

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        TimingPoints::parse_timing_points(&mut state.timing_points, line)
            .map_err(ParseHitObjectsError::TimingPoints)
    }

    fn parse_colors(_: &mut Self::State, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    // It's preferred to keep the code in-sync with osu!lazer without refactoring.
    #[allow(clippy::too_many_lines)]
    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        let mut split = line.trim_comment().split(',');

        let (Some(x), Some(y), Some(start_time), Some(kind), Some(sound_type)) = (
            split.next(),
            split.next(),
            split.next(),
            split.next(),
            split.next(),
        ) else {
            return Err(ParseHitObjectsError::InvalidLine);
        };

        let pos = Pos {
            x: x.parse_with_limits(MAX_COORDINATE_VALUE as f32)? as i32 as f32,
            y: y.parse_with_limits(MAX_COORDINATE_VALUE as f32)? as i32 as f32,
        };

        let start_time_raw = f64::parse(start_time)?;
        let start_time = start_time_raw;
        let mut hit_object_type: HitObjectType = kind.parse()?;

        let combo_offset = (hit_object_type & HitObjectType::COMBO_OFFSET) >> 4;
        hit_object_type &= !HitObjectType::COMBO_OFFSET;

        let new_combo = hit_object_type.has_flag(HitObjectType::NEW_COMBO);
        hit_object_type &= !HitObjectType::NEW_COMBO;

        let sound_type: HitSoundType = sound_type.parse()?;
        let mut bank_info = SampleBankInfo::default();

        let kind = if hit_object_type.has_flag(HitObjectType::CIRCLE) {
            if let Some(s) = split.next() {
                bank_info.read_custom_sample_banks(s.split(':'))?;
            }

            let circle = HitObjectCircle {
                pos,
                new_combo: state.first_object || state.last_object_was_spinner() || new_combo,
                combo_offset: if new_combo { combo_offset } else { 0 },
            };

            HitObjectKind::Circle(circle)
        } else if hit_object_type.has_flag(HitObjectType::SLIDER) {
            let (point_str, repeat_count) = split
                .next()
                .zip(split.next())
                .ok_or(ParseHitObjectsError::InvalidLine)?;

            let mut len = None;

            let mut repeat_count = repeat_count.parse_num::<i32>()?;

            if repeat_count > 9000 {
                return Err(ParseHitObjectsError::InvalidRepeatCount(repeat_count));
            }

            repeat_count = cmp::max(0, repeat_count - 1);

            if let Some(next) = split.next() {
                let new_len = next
                    .parse_with_limits(f64::from(MAX_COORDINATE_VALUE))?
                    .max(0.0);

                if new_len.abs() >= f64::EPSILON {
                    len = Some(new_len);
                }
            }

            let next_8 = split.next();
            let next_9 = split.next();

            if let Some(next) = split.next() {
                bank_info.read_custom_sample_banks(next.split(':'))?;
            }

            let nodes = repeat_count as usize + 2;

            let mut node_bank_infos = vec![bank_info.clone(); nodes];

            if let Some(next) = next_9.filter(|s| !s.is_empty()) {
                for (bank_info, set) in node_bank_infos.iter_mut().zip(next.split('|')) {
                    bank_info.read_custom_sample_banks(set.split(':'))?;
                }
            }

            let mut node_sound_types = vec![sound_type; nodes];

            if let Some(next) = next_8.filter(|s| !s.is_empty()) {
                for (sound_type, s) in node_sound_types.iter_mut().zip(next.split('|')) {
                    *sound_type = s.parse().unwrap_or_default();
                }
            }

            let node_samples: Vec<_> = node_bank_infos
                .into_iter()
                .zip(node_sound_types)
                .map(|(bank_info, sound_type)| bank_info.convert_sound_type(sound_type))
                .collect();

            state.convert_path_str(point_str, pos)?;
            let mut control_points = Vec::with_capacity(state.curve_points.len());
            control_points.append(&mut state.curve_points);

            let slider = HitObjectSlider {
                pos,
                new_combo: state.first_object || state.last_object_was_spinner() || new_combo,
                combo_offset: if new_combo { combo_offset } else { 0 },
                path: SliderPath::new(control_points, len),
                node_samples,
                repeat_count,
                velocity: 1.0,
            };

            HitObjectKind::Slider(slider)
        } else if hit_object_type.has_flag(HitObjectType::SPINNER) {
            let duration = split
                .next()
                .ok_or(ParseHitObjectsError::InvalidLine)?
                .parse_num::<f64>()?;

            let duration = (duration - start_time).max(0.0);

            if let Some(s) = split.next() {
                bank_info.read_custom_sample_banks(s.split(':'))?;
            }

            let spinner = HitObjectSpinner {
                pos: Pos::new(512.0 / 2.0, 384.0 / 2.0),
                duration,
                new_combo,
            };

            HitObjectKind::Spinner(spinner)
        } else if hit_object_type.has_flag(HitObjectType::HOLD) {
            let mut end_time = start_time.max(start_time_raw);

            if let Some(s) = split.next().filter(|s| !s.is_empty()) {
                let mut ss = s.split(':');

                let new_end_time = ss
                    .next()
                    .ok_or(ParseHitObjectsError::InvalidLine)?
                    .parse_num::<f64>()?;

                end_time = start_time.max(new_end_time);

                bank_info.read_custom_sample_banks(ss)?;
            }

            let hold = HitObjectHold {
                pos_x: pos.x,
                duration: end_time - start_time,
            };

            HitObjectKind::Hold(hold)
        } else {
            return Err(ParseHitObjectsError::UnknownHitObjectType(hit_object_type));
        };

        let result = HitObject {
            start_time,
            kind,
            samples: bank_info.convert_sound_type(sound_type),
        };

        state.first_object = false;
        state.last_object = Some(hit_object_type);
        state.hit_objects.push(result);

        Ok(())
    }
}
