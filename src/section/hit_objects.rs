use std::{
    cmp,
    num::ParseIntError,
    ops::{BitAnd, BitAndAssign},
    slice,
    str::{FromStr, Split},
};

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    model::{
        hit_objects::{
            slider::{HitObjectSlider, PathControlPoint, PathType, SliderPath},
            HitObject, HitObjectCircle, HitObjectHold, HitObjectKind, HitObjectSpinner,
        },
        hit_samples::{HitSampleInfo, HitSampleInfoName, SampleBank},
    },
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{ParseNumber, ParseNumberError, Pos, StrExt},
};

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
    #[error("failed to parse hit object type")]
    HitObjectType(#[source] ParseIntError),
    #[error("failed to parse hit sound type")]
    HitSoundType(#[source] ParseIntError),
    #[error("invalid line")]
    InvalidLine,
    #[error("repeat count is way too high")]
    InvalidRepeatCount,
    #[error("invalid sample bank")]
    InvalidSampleBank,
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("unknown hit object type")]
    UnknownHitObjectType,
}

#[derive(Copy, Clone, Default)]
pub(crate) struct HitObjectType(i32);

impl HitObjectType {
    const CIRCLE: i32 = 1;
    const SLIDER: i32 = 1 << 1;
    const NEW_COMBO: i32 = 1 << 2;
    const SPINNER: i32 = 1 << 3;
    const COMBO_OFFSET: i32 = (1 << 4) | (1 << 5) | (1 << 6);
    const HOLD: i32 = 1 << 7;

    const fn has_flag(self, flag: i32) -> bool {
        (self.0 & flag) != 0
    }
}

impl From<&HitObject> for HitObjectType {
    fn from(hit_object: &HitObject) -> Self {
        let mut kind = 0;

        match hit_object.kind {
            HitObjectKind::Circle(ref h) => {
                kind |= h.combo_offset << 4;

                if h.new_combo {
                    kind |= Self::NEW_COMBO;
                }

                kind |= Self::CIRCLE;
            }
            HitObjectKind::Slider(ref h) => {
                kind |= h.combo_offset << 4;

                if h.new_combo {
                    kind |= Self::NEW_COMBO;
                }

                kind |= Self::SLIDER;
            }
            HitObjectKind::Spinner(ref h) => {
                if h.new_combo {
                    kind |= Self::NEW_COMBO;
                }

                kind |= Self::SPINNER;
            }
            HitObjectKind::Hold(_) => kind |= Self::HOLD,
        }

        Self(kind)
    }
}

impl FromStr for HitObjectType {
    type Err = ParseHitObjectsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self)
            .map_err(ParseHitObjectsError::HitObjectType)
    }
}

impl From<HitObjectType> for i32 {
    fn from(kind: HitObjectType) -> Self {
        kind.0
    }
}

impl BitAnd<i32> for HitObjectType {
    type Output = i32;

    fn bitand(self, rhs: i32) -> Self::Output {
        self.0 & rhs
    }
}

impl BitAndAssign<i32> for HitObjectType {
    fn bitand_assign(&mut self, rhs: i32) {
        self.0 &= rhs;
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct HitSoundType(u8);

impl HitSoundType {
    const NONE: u8 = 0;
    const NORMAL: u8 = 1;
    const WHISTLE: u8 = 2;
    const FINISH: u8 = 4;
    const CLAP: u8 = 5;

    const fn has_flag(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }
}

impl From<&[HitSampleInfo]> for HitSoundType {
    fn from(samples: &[HitSampleInfo]) -> Self {
        let mut kind = Self::NONE;

        for sample in samples.iter() {
            // false positive
            #[allow(clippy::match_wildcard_for_single_variants)]
            match sample.name {
                HitSampleInfoName::Name(HitSampleInfo::HIT_WHISTLE) => kind |= Self::WHISTLE,
                HitSampleInfoName::Name(HitSampleInfo::HIT_FINISH) => kind |= Self::FINISH,
                HitSampleInfoName::Name(HitSampleInfo::HIT_CLAP) => kind |= Self::CLAP,
                _ => {}
            }
        }

        Self(kind)
    }
}

impl From<HitSoundType> for u8 {
    fn from(kind: HitSoundType) -> Self {
        kind.0
    }
}

impl FromStr for HitSoundType {
    type Err = ParseHitObjectsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self)
            .map_err(ParseHitObjectsError::HitSoundType)
    }
}

impl PartialEq<u8> for HitSoundType {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

#[derive(Clone, Default)]
struct SampleBankInfo {
    filename: Option<String>,
    bank_for_normal: Option<&'static str>,
    bank_for_addition: Option<&'static str>,
    volume: i32,
    custom_sample_bank: i32,
}

impl SampleBankInfo {
    fn read_custom_sample_banks(
        &mut self,
        mut split: Split<'_, char>,
    ) -> Result<(), ParseHitObjectsError> {
        let Some(first) = split.next() else {
            return Ok(());
        };

        let bank = i32::parse(first)?.try_into().unwrap_or(SampleBank::Normal);

        let add_bank = split
            .next()
            .ok_or(ParseHitObjectsError::InvalidSampleBank)?
            .parse_num::<i32>()?
            .try_into()
            .unwrap_or(SampleBank::Normal);

        let string_bank = (bank != SampleBank::None).then_some(bank.to_lowercase_str());
        let string_add_bank = (bank != SampleBank::None).then_some(add_bank.to_lowercase_str());

        self.bank_for_normal = string_bank;
        self.bank_for_addition = string_add_bank.or(string_bank);

        if let Some(next) = split.next() {
            self.custom_sample_bank = next.parse_num()?;
        }

        if let Some(next) = split.next() {
            self.volume = cmp::max(0, next.parse_num()?);
        }

        self.filename = split.next().map(str::to_owned);

        Ok(())
    }
}

/// The parsing state for [`HitObjects`] in [`ParseBeatmap`].
pub struct HitObjectsState {
    version: FormatVersion,
    first_object: bool,
    last_object: Option<HitObjectType>,
    curve_points: Vec<PathControlPoint>,
    vertices: Vec<PathControlPoint>,
    point_split: Vec<*const str>,
    hit_objects: HitObjects,
}

impl HitObjectsState {
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
            end_idx += 1;
        }

        if end_idx > start_idx {
            self.curve_points.extend(&self.vertices[start_idx..end_idx]);
        }

        Ok(())
    }

    fn last_object_was_spinner(&self) -> bool {
        self.last_object
            .is_some_and(|kind| kind.has_flag(HitObjectType::SPINNER))
    }
}

impl ParseState for HitObjectsState {
    fn create(version: FormatVersion) -> Self {
        Self {
            version,
            first_object: true,
            last_object: None,
            curve_points: Vec::new(),
            vertices: Vec::new(),
            point_split: Vec::new(),
            hit_objects: HitObjects::default(),
        }
    }
}

impl From<HitObjectsState> for HitObjects {
    fn from(state: HitObjectsState) -> Self {
        state.hit_objects
    }
}

const MAX_COORDINATE_VALUE: i32 = 131_072;

impl ParseBeatmap for HitObjects {
    type ParseError = ParseHitObjectsError;
    type State = HitObjectsState;

    fn parse_general(_: &mut Self::State, _: &str) -> Result<(), Self::ParseError> {
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

    // It's preferred to keep the code in-sync with osu!lazer without refactoring.
    #[allow(clippy::too_many_lines)]
    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
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
                return Err(ParseHitObjectsError::InvalidRepeatCount);
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
                .map(|(bank_info, sound_type)| convert_sound_type(sound_type, bank_info))
                .collect();

            state.convert_path_str(point_str, pos)?;

            let slider = HitObjectSlider {
                pos,
                new_combo: state.first_object || state.last_object_was_spinner() || new_combo,
                combo_offset: if new_combo { combo_offset } else { 0 },
                path: SliderPath::new(state.curve_points.clone(), len),
                node_samples,
                repeat_count,
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
            return Err(ParseHitObjectsError::UnknownHitObjectType);
        };

        let result = HitObject {
            start_time,
            kind,
            samples: convert_sound_type(sound_type, bank_info),
        };

        state.first_object = false;
        state.last_object = Some(hit_object_type);
        state.hit_objects.hit_objects.push(result);

        Ok(())
    }
}

fn convert_sound_type(sound_type: HitSoundType, bank_info: SampleBankInfo) -> Vec<HitSampleInfo> {
    let mut sound_types = Vec::new();

    if bank_info.filename.as_ref().is_some_and(|s| !s.is_empty()) {
        sound_types.push(HitSampleInfo::new(
            bank_info.filename,
            None,
            1,
            bank_info.volume,
        ));
    } else {
        let mut sample = HitSampleInfo::new(
            HitSampleInfo::HIT_NORMAL,
            bank_info.bank_for_normal,
            bank_info.custom_sample_bank,
            bank_info.volume,
        );

        sample.is_layered =
            sound_type != HitSoundType::NONE && !sound_type.has_flag(HitSoundType::NORMAL);

        sound_types.push(sample);
    }

    if sound_type.has_flag(HitSoundType::FINISH) {
        sound_types.push(HitSampleInfo::new(
            HitSampleInfo::HIT_FINISH,
            bank_info.bank_for_addition,
            bank_info.custom_sample_bank,
            bank_info.volume,
        ));
    }

    if sound_type.has_flag(HitSoundType::WHISTLE) {
        sound_types.push(HitSampleInfo::new(
            HitSampleInfo::HIT_WHISTLE,
            bank_info.bank_for_addition,
            bank_info.custom_sample_bank,
            bank_info.volume,
        ));
    }

    if sound_type.has_flag(HitSoundType::CLAP) {
        sound_types.push(HitSampleInfo::new(
            HitSampleInfo::HIT_CLAP,
            bank_info.bank_for_addition,
            bank_info.custom_sample_bank,
            bank_info.volume,
        ));
    }

    sound_types
}
