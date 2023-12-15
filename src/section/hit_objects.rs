use std::{
    cmp,
    num::{NonZeroI32, ParseIntError},
    ops::{Add, BitAnd, BitAndAssign, Sub},
    slice,
    str::{FromStr, Split},
};

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
    util::{ParseNumber, ParseNumberError, StrExt},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HitObjects {
    pub hit_objects: Vec<HitObject>,
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct HitObject {
    pub start_time: f64,
    pub kind: HitObjectKind,
    pub samples: Vec<HitSampleInfo>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HitObjectKind {
    Circle(HitObjectCircle),
    Slider(HitObjectSlider),
    Spinner(HitObjectSpinner),
    Hold(HitObjectHold),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectCircle {
    pub pos: Pos,
    pub new_combo: bool,
    pub combo_offset: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HitObjectSlider {
    pub pos: Pos,
    pub new_combo: bool,
    pub combo_offset: i32,
    pub path: SliderPath,
    pub node_samples: Vec<Vec<HitSampleInfo>>,
    pub repeat_count: i32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectSpinner {
    pub pos: Pos,
    pub duration: f64,
    pub new_combo: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectHold {
    pub pos_x: f32,
    pub duration: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderPath {
    control_points: Box<[PathControlPoint]>,
    expected_dist: Option<f64>,
    curve: Option<Curve>,
}

impl SliderPath {
    fn new(control_points: &[PathControlPoint], expected_dist: Option<f64>) -> Self {
        Self {
            control_points: control_points.into(),
            expected_dist,
            curve: None,
        }
    }

    pub fn control_points(&self) -> &[PathControlPoint] {
        self.control_points.as_ref()
    }

    pub fn expected_dist(&self) -> Option<f64> {
        self.expected_dist
    }

    pub fn curve(&mut self) -> &Curve {
        self.curve
            .get_or_insert_with(|| Curve::new(&self.control_points, self.expected_dist))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pub path: Vec<Pos>,
    pub lengths: Vec<f64>,
}

impl Curve {
    pub fn new(curve_points: &[PathControlPoint], expected_len: Option<f64>) -> Self {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PathType {
    pub kind: SplineType,
    pub degree: Option<NonZeroI32>,
}

impl PathType {
    pub const CATMULL: Self = Self::new(SplineType::Catmull);
    pub const BEZIER: Self = Self::new(SplineType::BSpline);
    pub const LINEAR: Self = Self::new(SplineType::Linear);
    pub const PERFECT_CURVE: Self = Self::new(SplineType::PerfectCurve);

    const fn new(kind: SplineType) -> Self {
        Self { kind, degree: None }
    }

    const fn new_b_spline(degree: NonZeroI32) -> Self {
        Self {
            kind: SplineType::BSpline,
            degree: Some(degree),
        }
    }

    fn convert(input: &str) -> Self {
        match input.chars().next() {
            Some('B') => {
                if let Ok(Some(degree)) = input[1..].parse().map(NonZeroI32::new) {
                    if degree.is_positive() {
                        return Self::new_b_spline(degree);
                    }
                }

                Self::BEZIER
            }
            Some('L') => Self::LINEAR,
            Some('P') => Self::PERFECT_CURVE,
            _ => Self::CATMULL,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SplineType {
    #[default]
    Catmull,
    BSpline,
    Linear,
    PerfectCurve,
}

#[derive(Copy, Clone)]
struct HitObjectType(u8);

impl HitObjectType {
    const CIRCLE: u8 = 1;
    const SLIDER: u8 = 1 << 1;
    const NEW_COMBO: u8 = 1 << 2;
    const SPINNER: u8 = 1 << 3;
    const COMBO_OFFSET: u8 = (1 << 4) | (1 << 5) | (1 << 6);
    const HOLD: u8 = 1 << 7;

    fn has_flag(self, flag: u8) -> bool {
        (self.0 & flag) != 0
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

impl BitAnd<u8> for HitObjectType {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        self.0 & rhs
    }
}

impl BitAndAssign<u8> for HitObjectType {
    fn bitand_assign(&mut self, rhs: u8) {
        self.0 &= rhs;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PathControlPoint {
    pub pos: Pos,
    pub path_type: Option<PathType>,
}

impl Default for PathControlPoint {
    fn default() -> Self {
        Self {
            pos: Pos::new(0.0, 0.0),
            path_type: Default::default(),
        }
    }
}

impl From<Pos> for PathControlPoint {
    fn from(pos: Pos) -> Self {
        Self {
            pos,
            path_type: None,
        }
    }
}

#[derive(Copy, Clone, Default)]
struct HitSoundType(u8);

impl HitSoundType {
    const NONE: u8 = 0;
    const NORMAL: u8 = 1;
    const WHISTLE: u8 = 2;
    const FINISH: u8 = 4;
    const CLAP: u8 = 5;

    fn has_flag(self, flag: u8) -> bool {
        (self.0 & flag) != 0
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitSampleInfo {
    pub name: HitSampleInfoName,
    pub bank: &'static str,
    pub suffix: Option<String>,
    pub volume: i32,
    pub custom_sample_bank: i32,
    pub bank_specified: bool,
    pub is_layered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HitSampleInfoName {
    Name(&'static str),
    File(Option<String>),
}

impl From<&'static str> for HitSampleInfoName {
    fn from(name: &'static str) -> Self {
        Self::Name(name)
    }
}

impl From<Option<String>> for HitSampleInfoName {
    fn from(filename: Option<String>) -> Self {
        Self::File(filename)
    }
}

impl HitSampleInfo {
    pub const HIT_NORMAL: &'static str = "hitnormal";
    pub const HIT_WHISTLE: &'static str = "hitwhistle";
    pub const HIT_FINISH: &'static str = "hitfinish";
    pub const HIT_CLAP: &'static str = "hitclap";

    pub const BANK_NORMAL: &'static str = "normal";
    pub const BANK_SOFT: &'static str = "soft";
    pub const BANK_DRUM: &'static str = "drum";

    fn new(
        name: impl Into<HitSampleInfoName>,
        bank: Option<&'static str>,
        custom_sample_bank: i32,
        volume: i32,
    ) -> Self {
        Self {
            name: name.into(),
            bank: bank.unwrap_or(Self::BANK_NORMAL),
            suffix: (custom_sample_bank >= 2).then(|| custom_sample_bank.to_string()),
            volume,
            custom_sample_bank,
            bank_specified: bank.is_some(),
            is_layered: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum SampleBank {
    None,
    Normal,
    Soft,
    Drum,
}

impl SampleBank {
    fn to_lowercase_str(self) -> &'static str {
        match self {
            SampleBank::None => "none",
            SampleBank::Normal => "normal",
            SampleBank::Soft => "soft",
            SampleBank::Drum => "drum",
        }
    }
}

impl TryFrom<i32> for SampleBank {
    type Error = ();

    fn try_from(bank: i32) -> Result<Self, Self::Error> {
        match bank {
            0 => Ok(Self::None),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Soft),
            3 => Ok(Self::Drum),
            _ => Err(()),
        }
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
                .map(|s| s.parse_with_limits(MAX_COORDINATE_VALUE as f64));

            let (x, y) = v
                .next()
                .zip(v.next())
                .ok_or(ParseHitObjectsError::InvalidLine)?;

            let pos = Pos::new(x? as i32 as f32, y? as i32 as f32);

            Ok(PathControlPoint::from(pos - start_pos))
        }

        fn is_linear(p0: Pos, p1: Pos, p2: Pos) -> bool {
            ((p1.y - p0.y) * (p2.x - p0.x) - (p1.x - p0.x) * (p2.y - p0.y)).abs() < f32::EPSILON
        }

        let mut path_type = points
            .get(0)
            .copied()
            .map(PathType::convert)
            .ok_or(ParseHitObjectsError::InvalidLine)?;

        let read_offset = first as usize;
        let readable_points = points.len() - 1;
        let end_point_len = end_point.is_some() as usize;

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

    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        let offset = state.version.offset() as f64;

        let mut split = line.split(',');

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

        let combo_offset = ((hit_object_type & HitObjectType::COMBO_OFFSET) >> 4) as i32;
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
                    .parse_with_limits(MAX_COORDINATE_VALUE as f64)?
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
                path: SliderPath::new(&state.curve_points, len),
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

    if !bank_info.filename.as_ref().is_some_and(|s| !s.is_empty()) {
        let mut sample = HitSampleInfo::new(
            HitSampleInfo::HIT_NORMAL,
            bank_info.bank_for_normal,
            bank_info.custom_sample_bank,
            bank_info.volume,
        );

        sample.is_layered =
            sound_type != HitSoundType::NONE && !sound_type.has_flag(HitSoundType::NORMAL);

        sound_types.push(sample);
    } else {
        sound_types.push(HitSampleInfo::new(
            bank_info.filename,
            None,
            1,
            bank_info.volume,
        ));
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
