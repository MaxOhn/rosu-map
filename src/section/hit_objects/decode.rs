use std::{cmp, slice};

use crate::{
    decode::{DecodeBeatmap, DecodeState},
    reader::DecoderError,
    section::{
        difficulty::DifficultyKey,
        events::{Events, EventsState, ParseEventsError},
        hit_objects::{slider::path_type::PathType, CurveBuffers},
        timing_points::{
            ControlPoints, ControlPointsState, DifficultyPoint, ParseControlPointsError,
            SamplePoint, TimingPoint,
        },
    },
    util::{KeyValue, ParseNumber, ParseNumberError, Pos, StrExt},
    {FormatVersion, ParseVersionError},
};

use super::{
    hit_samples::{HitSoundType, ParseHitSoundTypeError, ParseSampleBankInfoError, SampleBankInfo},
    HitObject, HitObjectCircle, HitObjectHold, HitObjectKind, HitObjectSlider, HitObjectSpinner,
    HitObjectType, ParseHitObjectTypeError, PathControlPoint, SliderPath,
};

/// Struct containing all data from a `.osu` file's `[HitObjects]` section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HitObjects {
    pub hit_objects: Vec<HitObject>,
}

/// All the ways that parsing a `.osu` file into [`HitObjects`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseHitObjectsError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
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
    #[error("failed to parse timing points section")]
    TimingPoints(#[from] ParseControlPointsError),
    #[error("unknown hit object type")]
    UnknownHitObjectType(HitObjectType),
}

/// The parsing state for [`HitObjects`] in [`DecodeBeatmap`].
pub struct HitObjectsState {
    version: FormatVersion,
    first_object: bool,
    last_object: Option<HitObjectType>,
    curve_points: Vec<PathControlPoint>,
    vertices: Vec<PathControlPoint>,
    point_split: Vec<*const str>,
    // TODO: decoding HitObjects for Beatmap currently parses events & control
    // points twice which is not ideal
    events: EventsState,
    control_points: ControlPointsState,
    slider_multiplier: f32,
    hit_objects: HitObjects,
}

impl HitObjectsState {
    /// Processes the point string of a slider hit object.
    fn convert_path_str(
        &mut self,
        point_str: &str,
        offset: Pos,
    ) -> Result<(), ParseHitObjectsError> {
        let f = |this: &mut Self, point_split: &[&str]| {
            let mut start_idx = 0;
            let mut end_idx = 1;
            let mut first = true;

            while end_idx < point_split.len() {
                let is_letter = point_split[end_idx]
                    .chars()
                    .next()
                    .ok_or(ParseHitObjectsError::InvalidLine)?
                    .is_ascii_alphabetic();

                if !is_letter {
                    end_idx += 1;
                    continue;
                }

                let end_point = point_split.get(end_idx + 1).copied();
                this.convert_points(&point_split[start_idx..end_idx], end_point, first, offset)?;

                start_idx = end_idx;
                first = false;

                end_idx += 1;
            }

            if end_idx > start_idx {
                this.convert_points(&point_split[start_idx..end_idx], None, first, offset)?;
            }

            this.point_split.clear();

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
        let mut end_idx = 1;

        while end_idx < self.vertices.len() - end_point_len {
            if self.vertices[end_idx].pos != self.vertices[end_idx - 1].pos {
                end_idx += 1;
                continue;
            }

            if path_type == PathType::CATMULL && end_idx > 1 {
                end_idx += 1;
                continue;
            }

            if end_idx == self.vertices.len() - end_point_len - 1 {
                end_idx += 1;
                continue;
            }

            self.vertices[end_idx - 1].path_type = Some(path_type);

            self.curve_points.extend(&self.vertices[start_idx..end_idx]);

            start_idx = end_idx + 1;
            end_idx += 1;
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
            version,
            first_object: true,
            last_object: None,
            curve_points: Vec::new(),
            vertices: Vec::new(),
            point_split: Vec::new(),
            events: EventsState::create(version),
            control_points: ControlPointsState::create(version),
            slider_multiplier: 0.0,
            hit_objects: HitObjects::default(),
        }
    }
}

impl From<HitObjectsState> for HitObjects {
    fn from(state: HitObjectsState) -> Self {
        const CONTROL_POINT_LENIENCY: f64 = 5.0;

        let mut hit_objects = state.hit_objects;

        hit_objects
            .hit_objects
            .sort_by(|a, b| a.start_time.total_cmp(&b.start_time));

        let events: Events = state.events.into();
        let control_points: ControlPoints = state.control_points.into();

        HitObjectsState::post_process_breaks(&mut hit_objects.hit_objects, &events);
        let mut bufs = CurveBuffers::default();

        for h in hit_objects.hit_objects.iter_mut() {
            if let HitObjectKind::Slider(ref mut slider) = h.kind {
                const BASE_SCORING_DIST: f32 = 100.0;

                let beat_len = control_points
                    .timing_point_at(h.start_time)
                    .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len);

                let slider_velocity = control_points
                    .difficulty_point_at(h.start_time)
                    .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                        point.slider_velocity
                    });

                let scoring_dist = f64::from(BASE_SCORING_DIST)
                    * f64::from(state.slider_multiplier)
                    * slider_velocity;

                slider.velocity = scoring_dist / beat_len;

                let span_count = f64::from(slider.span_count());
                let duration = slider.duration_with_bufs(&mut bufs);

                for i in 0..slider.node_samples.len() {
                    let time =
                        h.start_time + i as f64 * duration / span_count + CONTROL_POINT_LENIENCY;

                    let node_sample_point = control_points
                        .sample_point_at(time)
                        .map_or_else(SamplePoint::default, SamplePoint::clone);

                    for sample in slider.node_samples[i].iter_mut() {
                        node_sample_point.apply(sample);
                    }
                }
            }

            let end_time = h.end_time_with_bufs(&mut bufs);

            let sample_point = control_points
                .sample_point_at(end_time + CONTROL_POINT_LENIENCY)
                .map_or_else(SamplePoint::default, SamplePoint::clone);

            for sample in h.samples.iter_mut() {
                sample_point.apply(sample);
            }
        }

        hit_objects
    }
}

const MAX_COORDINATE_VALUE: i32 = 131_072;

impl DecodeBeatmap for HitObjects {
    type Error = ParseHitObjectsError;
    type State = HitObjectsState;

    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_general(&mut state.events, line)?;
        ControlPoints::parse_general(&mut state.control_points, line)?;

        Ok(())
    }

    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_editor(&mut state.events, line)?;
        ControlPoints::parse_editor(&mut state.control_points, line)?;

        Ok(())
    }

    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_metadata(&mut state.events, line)?;
        ControlPoints::parse_metadata(&mut state.control_points, line)?;

        Ok(())
    }

    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_difficulty(&mut state.events, line)?;
        ControlPoints::parse_difficulty(&mut state.control_points, line)?;

        let Ok(KeyValue { key, value }) = KeyValue::parse(line.trim_comment()) else {
            return Ok(());
        };

        match key {
            DifficultyKey::SliderMultiplier => {
                state.slider_multiplier = f32::parse(value)?.clamp(0.4, 3.6);
            }
            _ => {}
        }

        Ok(())
    }

    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_events(&mut state.events, line)?;
        ControlPoints::parse_events(&mut state.control_points, line)?;

        Ok(())
    }

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_timing_points(&mut state.events, line)?;
        ControlPoints::parse_timing_points(&mut state.control_points, line)?;

        Ok(())
    }

    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_colors(&mut state.events, line)?;
        ControlPoints::parse_colors(&mut state.control_points, line)?;

        Ok(())
    }

    // It's preferred to keep the code in-sync with osu!lazer without refactoring.
    #[allow(clippy::too_many_lines)]
    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
        Events::parse_hit_objects(&mut state.events, line)?;
        ControlPoints::parse_hit_objects(&mut state.control_points, line)?;

        let offset = f64::from(state.version.offset());

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
        let start_time = start_time_raw + offset;
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

            let slider = HitObjectSlider {
                pos,
                new_combo: state.first_object || state.last_object_was_spinner() || new_combo,
                combo_offset: if new_combo { combo_offset } else { 0 },
                path: SliderPath::new(state.curve_points.clone(), len),
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

            let duration = (duration + offset - start_time).max(0.0);

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
                duration: end_time + offset - start_time,
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
        state.hit_objects.hit_objects.push(result);

        Ok(())
    }
}
