use std::{
    cmp::Ordering,
    io::{BufWriter, Result as IoResult, Write},
};

use crate::{
    beatmap::Beatmap,
    model::{
        control_points::{DifficultyPoint, EffectPoint, SamplePoint, TimingPoint},
        events::EventType,
        hit_objects::{
            slider::{HitObjectSlider, PathType, SplineType},
            HitObject, HitObjectKind,
        },
        hit_samples::{HitSampleInfo, HitSampleInfoName, SampleBank},
        mode::GameMode,
    },
    section::{
        difficulty::DifficultyKey,
        editor::EditorKey,
        general::GeneralKey,
        hit_objects::{HitObjectType, HitSoundType},
        metadata::MetadataKey,
        timing_points::EffectFlags,
    },
    util::{Pos, Sortable, SortedVec},
};

impl Beatmap {
    pub fn encode<W: Write>(&self, dst: W) -> IoResult<()> {
        let mut writer = BufWriter::new(dst);

        writeln!(writer, "osu file format v{}", self.format_version.0)?;

        writer.write_all(b"\n")?;
        self.encode_general(&mut writer)?;

        writer.write_all(b"\n")?;
        self.encode_editor(&mut writer)?;

        writer.write_all(b"\n")?;
        self.encode_metadata(&mut writer)?;

        writer.write_all(b"\n")?;
        self.encode_difficulty(&mut writer)?;

        writer.write_all(b"\n")?;
        self.encode_events(&mut writer)?;

        writer.write_all(b"\n")?;
        self.encode_timing_points(&mut writer)?;

        writer.write_all(b"\n")?;
        self.encode_colors(&mut writer)?;

        writer.write_all(b"\n")?;
        self.encode_hit_objects(&mut writer)?;

        writer.flush()
    }

    fn encode_general<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        writeln!(
            writer,
            "[General]
{}: {}
{}: {}
{}: {}
{}: {}",
            GeneralKey::AudioFilename,
            self.audio_file,
            GeneralKey::AudioLeadIn,
            self.audio_lead_in,
            GeneralKey::PreviewTime,
            self.preview_time,
            GeneralKey::Countdown,
            self.countdown as i32
        )?;

        let sample_set = self
            .sample_points
            .first()
            .map_or(SamplePoint::DEFAULT_SAMPLE_BANK, |sample_point| {
                sample_point.sample_bank
            });

        let sample_set = match sample_set {
            HitSampleInfo::BANK_NORMAL => SampleBank::Normal,
            HitSampleInfo::BANK_SOFT => SampleBank::Soft,
            HitSampleInfo::BANK_DRUM => SampleBank::Drum,
            _ => SampleBank::None,
        };

        writeln!(
            writer,
            "{}: {}
{}: {}
{}: {}
{}: {}",
            GeneralKey::SampleSet,
            sample_set as i32,
            GeneralKey::StackLeniency,
            self.stack_leniency,
            GeneralKey::Mode,
            self.mode as i32,
            GeneralKey::LetterboxInBreaks,
            i32::from(self.letterbox_in_breaks),
        )?;

        if self.epilepsy_warning {
            writeln!(writer, "{}: {}", GeneralKey::EpilepsyWarning, 1)?;
        }

        if self.countdown_offset > 0 {
            writeln!(
                writer,
                "{}: {}",
                GeneralKey::CountdownOffset,
                self.countdown_offset
            )?;
        }

        if self.mode == GameMode::Mania {
            writeln!(
                writer,
                "{}: {}",
                GeneralKey::SpecialStyle,
                i32::from(self.special_style)
            )?;
        }

        writeln!(
            writer,
            "{}: {}",
            GeneralKey::WidescreenStoryboard,
            i32::from(self.widescreen_storyboard)
        )?;

        if self.samples_match_playback_rate {
            writeln!(writer, "{}: {}", GeneralKey::SamplesMatchPlaybackRate, 1)?;
        }

        Ok(())
    }

    fn encode_editor<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        writer.write_all(b"[Editor]")?;

        let mut bookmarks = self.bookmarks.iter();

        if let Some(bookmark) = bookmarks.next() {
            write!(writer, "Bookmarks: {bookmark}")?;

            for bookmark in bookmarks {
                write!(writer, ",{bookmark}")?;
            }

            writer.write_all(b"\n")?;
        }

        writeln!(
            writer,
            "{}: {}
            {}: {}
            {}: {}
            {}: {}",
            EditorKey::DistanceSpacing,
            self.distance_spacing,
            EditorKey::BeatDivisor,
            self.beat_divisor,
            EditorKey::GridSize,
            self.grid_size,
            EditorKey::TimelineZoom,
            self.timeline_zoom
        )
    }

    fn encode_metadata<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        writer.write_all(b"[Metadata]")?;

        writeln!(writer, "{}: {}", MetadataKey::Title, &self.title)?;

        if !self.title_unicode.is_empty() {
            writeln!(
                writer,
                "{}: {}",
                MetadataKey::TitleUnicode,
                &self.title_unicode
            )?;
        }

        writeln!(writer, "{}: {}", MetadataKey::Artist, self.artist)?;

        if !self.artist_unicode.is_empty() {
            writeln!(
                writer,
                "{}: {}",
                MetadataKey::ArtistUnicode,
                &self.artist_unicode
            )?;
        }

        writeln!(writer, "{}: {}", MetadataKey::Creator, &self.creator)?;
        writeln!(writer, "{}: {}", MetadataKey::Version, &self.version)?;

        if !self.source.is_empty() {
            writeln!(writer, "{}: {}", MetadataKey::Source, &self.source)?;
        }

        Ok(())
    }

    fn encode_difficulty<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        writeln!(
            writer,
            "[Difficulty]
{}: {}
{}: {}
{}: {}
{}: {}
{}: {}
{}: {}",
            DifficultyKey::HPDrainRate,
            self.hp_drain_rate,
            DifficultyKey::CircleSize,
            self.circle_size,
            DifficultyKey::OverallDifficulty,
            self.overall_difficulty,
            DifficultyKey::ApproachRate,
            self.approach_rate,
            DifficultyKey::SliderMultiplier,
            self.slider_multiplier,
            DifficultyKey::SliderTickRate,
            self.slider_tick_rate
        )
    }

    fn encode_events<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        if self.background_file.is_empty() && self.breaks.is_empty() {
            return Ok(());
        }

        writer.write_all(b"[Events]")?;

        if !self.background_file.is_empty() {
            writeln!(
                writer,
                "{},0,\"{}\",0,0",
                EventType::Background as i32,
                self.background_file
            )?;
        }

        for b in self.breaks.iter() {
            writeln!(
                writer,
                "{},{},{}",
                EventType::Break as i32,
                b.start_time,
                b.end_time
            )?;
        }

        Ok(())
    }

    fn encode_timing_points<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        fn collect_difficulty_points<'a>(
            map: &'a Beatmap,
        ) -> impl Iterator<Item = DifficultyPoint> + 'a {
            // FIXME: consider scroll_speed_encoded_as_slider_velocity

            map.hit_objects.iter().filter_map(|h| match h.kind {
                HitObjectKind::Slider(_) => Some(DifficultyPoint {
                    time: h.start_time,
                    bpm_multiplier: DifficultyPoint::DEFAULT_BPM_MULTIPLIER,
                    slider_velocity: map
                        .difficulty_point_at(h.start_time)
                        .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                            point.slider_velocity
                        }),
                    generate_ticks: DifficultyPoint::DEFAULT_GENERATE_TICKS,
                }),
                HitObjectKind::Circle(_) | HitObjectKind::Spinner(_) | HitObjectKind::Hold(_) => {
                    None
                }
            })
        }

        fn extra_difficulty_points(
            map: &Beatmap,
            control_points: &mut ControlPointInfo,
            last_relevant_difficulty_point: &mut Option<DifficultyPoint>,
        ) {
            // FIXME: order collected points by time
            for point in collect_difficulty_points(map) {
                let is_redundant = last_relevant_difficulty_point
                    .as_ref()
                    .is_some_and(|p| point.is_redundant(p));

                if !is_redundant {
                    control_points.add(point.clone());
                    *last_relevant_difficulty_point = Some(point);
                }
            }
        }

        fn collect_sample_points<'a>(map: &'a Beatmap) -> impl Iterator<Item = SamplePoint> + 'a {
            map.hit_objects.iter().filter_map(|h| {
                let volume = h.samples.iter().map(|sample| sample.volume).max();
                let custom_idx = h
                    .samples
                    .iter()
                    .map(|sample| sample.custom_sample_bank)
                    .max();

                if let Some((volume, custom_idx)) = volume.zip(custom_idx) {
                    let point = SamplePoint {
                        time: h.end_time(),
                        sample_bank: SamplePoint::DEFAULT_SAMPLE_BANK,
                        sample_volume: volume,
                        custom_sample_bank: custom_idx,
                    };

                    // TODO: yield point
                }

                // TODO: nested objects

                todo!()
            })
        }

        fn extra_sample_points(
            map: &Beatmap,
            control_points: &mut ControlPointInfo,
            last_relevant_sample_point: &mut Option<SamplePoint>,
        ) {
            // FIXME: order collected points by time
            for point in collect_sample_points(map) {
                let is_redundant = last_relevant_sample_point
                    .as_ref()
                    .is_some_and(|p| point.is_redundant(p));

                if !is_redundant {
                    control_points.add(point.clone());
                    *last_relevant_sample_point = Some(point);
                }
            }
        }

        let mut control_points = ControlPointInfo::default();

        for point in self.timing_points.iter() {
            control_points.add(point.clone()); // TODO: work with references?
        }

        for point in self.effect_points.iter() {
            control_points.add(point.clone()); // TODO: work with references?
        }

        writer.write_all(b"[TimingPoints]")?;

        let mut last_relevant_sample_point = None;
        let mut last_relevant_difficulty_point = None;

        let scroll_speed_encoded_as_slider_velocity =
            matches!(self.mode, GameMode::Taiko | GameMode::Mania);

        extra_difficulty_points(
            self,
            &mut control_points,
            &mut last_relevant_difficulty_point,
        );
        extra_sample_points(self, &mut control_points, &mut last_relevant_sample_point);

        if scroll_speed_encoded_as_slider_velocity {
            for i in 0..control_points.effect.len() {
                let effect = &control_points.effect[i];

                let point = DifficultyPoint {
                    time: effect.time,
                    bpm_multiplier: DifficultyPoint::DEFAULT_BPM_MULTIPLIER,
                    slider_velocity: effect.scroll_speed,
                    generate_ticks: DifficultyPoint::DEFAULT_GENERATE_TICKS,
                };

                control_points.add(point);
            }
        }

        // let mut last_control_point_props = None;

        for group in control_points.groups {
            let timing = group.timing.as_ref();

            // var controlPointProperties = getLegacyControlPointProperties(group, groupTimingPoint != null);

            if let Some(timing) = group.timing {
                write!(writer, "{},{},", timing.time, timing.beat_len)?;

                /*
                    outputControlPointAt(controlPointProperties, true);
                    lastControlPointProperties = controlPointProperties;
                    lastControlPointProperties.SliderVelocity = 1;
                */
            }

            // if (controlPointProperties.IsRedundant(lastControlPointProperties))
            //     continue;

            // writer.Write(FormattableString.Invariant($"{group.Time},"));
            // writer.Write(FormattableString.Invariant($"{-100 / controlPointProperties.SliderVelocity},"));

            // outputControlPointAt(controlPointProperties, false);
            // lastControlPointProperties = controlPointProperties;
        }

        Ok(())
    }

    fn encode_colors<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        if self.custom_combo_colors.is_empty() {
            return Ok(());
        }

        writer.write_all(b"[Colours]")?;

        for (color, i) in self.custom_combo_colors.iter().zip(1..) {
            writeln!(
                writer,
                "Combo{i}: {},{},{},{}",
                color.red(),
                color.green(),
                color.blue(),
                color.alpha(),
            )?;
        }

        Ok(())
    }

    fn encode_hit_objects<W: Write>(&self, writer: &mut BufWriter<W>) -> IoResult<()> {
        if self.hit_objects.is_empty() {
            return Ok(());
        }

        writer.write_all(b"[HitObjects]")?;

        for hit_object in self.hit_objects.iter() {
            let mut pos = Pos::new(256.0, 192.0);

            match self.mode {
                GameMode::Osu | GameMode::Catch => match hit_object.kind {
                    HitObjectKind::Circle(ref h) => pos = h.pos,
                    HitObjectKind::Slider(ref h) => pos = h.pos,
                    HitObjectKind::Spinner(ref h) => pos = h.pos,
                    HitObjectKind::Hold(ref h) => pos.x = h.pos_x,
                },
                GameMode::Taiko => {}
                GameMode::Mania => {
                    let total_columns = self.circle_size.max(1.0) as i32;

                    let pos_x = match hit_object.kind {
                        HitObjectKind::Circle(ref h) => h.pos.x,
                        HitObjectKind::Slider(ref h) => h.pos.x,
                        HitObjectKind::Spinner(ref h) => h.pos.x,
                        HitObjectKind::Hold(ref h) => h.pos_x,
                    };

                    pos.x = (pos_x * (512.0 / total_columns as f32)).ceil() as i32 as f32;
                }
            };

            write!(
                writer,
                "{x},{y},{start_time},{kind},{sound},",
                x = pos.x,
                y = pos.y,
                start_time = hit_object.start_time,
                kind = i32::from(HitObjectType::from(hit_object)),
                sound = u8::from(HitSoundType::from(hit_object.samples.as_slice())),
            )?;

            match hit_object.kind {
                HitObjectKind::Circle(_) => {}
                HitObjectKind::Slider(ref h) => add_path_data(writer, h, pos, self.mode)?,
                HitObjectKind::Spinner(ref h) => {
                    write!(writer, "{},", hit_object.start_time + h.duration)?;
                }
                HitObjectKind::Hold(ref h) => {
                    write!(writer, "{}:", hit_object.start_time + h.duration)?;
                }
            }

            get_sample_bank(writer, &hit_object.samples, false, self.mode)?;

            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}

fn add_path_data<W: Write>(
    writer: &mut BufWriter<W>,
    slider: &HitObjectSlider,
    pos: Pos,
    mode: GameMode,
) -> IoResult<()> {
    let mut last_type = None;

    for i in 0..slider.path.control_points().len() {
        let point = slider.path.control_points()[i];

        if let Some(path_type) = point.path_type {
            let mut needs_explicit_segment =
                point.path_type != last_type || point.path_type == Some(PathType::PERFECT_CURVE);

            if i > 1 {
                let p1 = pos + slider.path.control_points()[i - 1].pos;
                let p2 = pos + slider.path.control_points()[i - 2].pos;

                if p1.x as i32 == p2.x as i32 && p1.y as i32 == p2.y as i32 {
                    needs_explicit_segment = true;
                }
            }

            if needs_explicit_segment {
                match path_type.kind {
                    SplineType::BSpline => {
                        if let Some(degree) = path_type.degree {
                            write!(writer, "B{degree}|")?;
                        } else {
                            write!(writer, "B|")?;
                        }
                    }
                    SplineType::Catmull => writer.write_all(b"C|")?,
                    SplineType::PerfectCurve => writer.write_all(b"P|")?,
                    SplineType::Linear => writer.write_all(b"L|")?,
                }

                last_type = Some(path_type);
            } else {
                write!(
                    writer,
                    "{x}:{y}|",
                    x = pos.x + point.pos.x,
                    y = pos.y + point.pos.y
                )?;
            }
        }

        if i != 0 {
            write!(
                writer,
                "{x}:{y}{count}",
                x = pos.x + point.pos.x,
                y = pos.y + point.pos.y,
                count = if i == slider.path.control_points().len() - 1 {
                    ','
                } else {
                    '|'
                }
            )?;
        }
    }

    write!(
        writer,
        "{span_count},{dist},",
        span_count = slider.span_count(),
        dist = slider
            .path
            .expected_dist()
            // .unwrap_or_else(|| slider.path.dist) // TODO: calculate distance
            .unwrap_or(0.0)
    )?;

    for i in 0..=slider.span_count() as usize {
        write!(
            writer,
            "{sound_type}{suffix}",
            sound_type = if i < slider.node_samples.len() {
                u8::from(HitSoundType::from(slider.node_samples[i].as_slice()))
            } else {
                0
            },
            suffix = if i == slider.span_count() as usize {
                ','
            } else {
                '|'
            }
        )?;
    }

    for i in 0..=slider.span_count() as usize {
        if i < slider.node_samples.len() {
            get_sample_bank(writer, &slider.node_samples[i], true, mode)?;
        } else {
            writer.write_all(b"0:0")?;
        }

        let suffix = if i == slider.span_count() as usize {
            b","
        } else {
            b"|"
        };

        writer.write_all(suffix)?;
    }

    Ok(())
}

fn get_sample_bank<W: Write>(
    writer: &mut BufWriter<W>,
    samples: &[HitSampleInfo],
    banks_only: bool,
    mode: GameMode,
) -> IoResult<()> {
    // osu!lazer throws an error if multiple sample's match the filter but
    // we'll just take the first and assume it's the only one.
    let normal_bank = samples
        .iter()
        .find(|sample| {
            matches!(
                sample.name,
                HitSampleInfoName::Name(HitSampleInfo::HIT_NORMAL)
            )
        })
        .map_or(SamplePoint::DEFAULT_SAMPLE_BANK, |sample| sample.bank);

    let normal_bank = SampleBank::from_lowercase(normal_bank);

    let add_bank = samples
        .iter()
        .find(|sample| match sample.name {
            // an empty name implies a filename which we handle through an enum variant so we
            // don't need to check for it anymore
            HitSampleInfoName::Name(n) => n != HitSampleInfo::HIT_NORMAL,
            HitSampleInfoName::File(_) => false,
        })
        .map_or(SamplePoint::DEFAULT_SAMPLE_BANK, |sample| sample.bank);

    let add_bank = SampleBank::from_lowercase(add_bank);

    write!(writer, "{}:{}", normal_bank as i32, add_bank as i32)?;

    if banks_only {
        return Ok(());
    }

    let mut custom_sample_bank = samples
        .iter()
        .find(|sample| matches!(sample.name, HitSampleInfoName::Name(_)))
        .map_or(0, |sample| sample.custom_sample_bank);

    let sample_filename = samples
        .iter()
        .find(|sample| matches!(sample.name, HitSampleInfoName::Name(_)))
        .map(HitSampleInfo::lookup_name)
        .unwrap_or_default();

    let mut volume = samples.first().map_or(100, |sample| sample.volume);

    if mode == GameMode::Mania {
        custom_sample_bank = 0;
        volume = 0;
    }

    write!(writer, ":{custom_sample_bank}:{volume}:{sample_filename}")
}

#[derive(Default)]
struct ControlPointInfo {
    timing: SortedVec<TimingPoint>,
    effect: SortedVec<EffectPoint>,
    difficulty: SortedVec<DifficultyPoint>,
    sample: SortedVec<SamplePoint>,
    groups: Vec<ControlPointGroup>,
}

impl ControlPointInfo {
    fn add<P: ControlPoint>(&mut self, point: P) -> bool {
        if point.check_already_existing(self) {
            false
        } else {
            point.add(self);

            true
        }
    }

    fn group_at(&mut self, time: f64) -> &mut ControlPointGroup {
        let cmp =
            |probe: &ControlPointGroup| probe.time.partial_cmp(&time).unwrap_or(Ordering::Less);

        match self.groups.binary_search_by(cmp) {
            Ok(i) => &mut self.groups[i],
            Err(i) => {
                self.groups.insert(i, ControlPointGroup::new(time));

                &mut self.groups[i]
            }
        }
    }
}

struct ControlPointGroup {
    time: f64,
    sample: Option<SamplePoint>,
    difficulty: Option<DifficultyPoint>,
    timing: Option<TimingPoint>,
    effect: Option<EffectPoint>,
}

trait ControlPoint {
    fn check_already_existing(&self, info: &ControlPointInfo) -> bool;
    fn add(self, info: &mut ControlPointInfo);
}

impl ControlPoint for SamplePoint {
    fn check_already_existing(&self, info: &ControlPointInfo) -> bool {
        let cmp =
            |probe: &SamplePoint| probe.time.partial_cmp(&self.time).unwrap_or(Ordering::Less);

        match info.sample.binary_search_by(cmp).map(|i| &info.sample[i]) {
            Ok(existing) => self.is_redundant(existing),
            Err(_) => false,
        }
    }

    fn add(self, info: &mut ControlPointInfo) {
        info.sample.push(self.clone());
        let time = self.time;
        info.group_at(time).sample = Some(self);
    }
}

impl ControlPoint for DifficultyPoint {
    fn check_already_existing(&self, info: &ControlPointInfo) -> bool {
        info.difficulty
            .binary_search_by(|probe| probe.time.partial_cmp(&self.time).unwrap_or(Ordering::Less))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map_or_else(
                || self.is_redundant(&Self::default()),
                |i| self.is_redundant(&info.difficulty[i]),
            )
    }

    fn add(self, info: &mut ControlPointInfo) {
        info.difficulty.push(self.clone());
        let time = self.time;
        info.group_at(time).difficulty = Some(self);
    }
}

impl ControlPoint for TimingPoint {
    fn check_already_existing(&self, info: &ControlPointInfo) -> bool {
        let cmp =
            |probe: &TimingPoint| probe.time.partial_cmp(&self.time).unwrap_or(Ordering::Less);

        match info.timing.binary_search_by(cmp).map(|i| &info.timing[i]) {
            Ok(existing) => self.is_redundant(existing),
            Err(_) => false,
        }
    }

    fn add(self, info: &mut ControlPointInfo) {
        info.timing.push(self.clone());
        let time = self.time;
        info.group_at(time).timing = Some(self);
    }
}

impl ControlPoint for EffectPoint {
    fn check_already_existing(&self, info: &ControlPointInfo) -> bool {
        let existing = info
            .effect
            .binary_search_by(|probe| probe.time.partial_cmp(&self.time).unwrap_or(Ordering::Less))
            .map_or_else(|i| i.checked_sub(1), Some)
            .map(|i| &info.effect[i]);

        match existing {
            Some(existing) => self.is_redundant(existing),
            None => false,
        }
    }

    fn add(self, info: &mut ControlPointInfo) {
        info.effect.push(self.clone());
        let time = self.time;
        info.group_at(time).effect = Some(self);
    }
}

impl ControlPointGroup {
    const fn new(time: f64) -> Self {
        Self {
            time,
            sample: None,
            difficulty: None,
            timing: None,
            effect: None,
        }
    }
}

struct ControlPointProps {
    slider_velocity: f64,
    timing_signature: i32,
    sample_bank: i32,
    custom_sample_bank: i32,
    sample_volume: i32,
    effect_flags: EffectFlags,
}

impl ControlPointProps {
    fn is_redundant(&self, other: &Self) -> bool {
        (self.slider_velocity - other.slider_velocity).abs() < f64::EPSILON
            && self.timing_signature == other.timing_signature
            && self.sample_bank == other.sample_bank
            && self.custom_sample_bank == other.custom_sample_bank
            && self.sample_volume == other.sample_volume
            && self.effect_flags == other.effect_flags
    }
}
