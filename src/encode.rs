use std::{
    fs::File,
    io::{BufWriter, Error as IoError, ErrorKind, Result as IoResult, Write},
    path::Path,
};

use crate::{
    beatmap::Beatmap,
    section::{
        difficulty::DifficultyKey,
        editor::EditorKey,
        events::EventType,
        general::{GameMode, GeneralKey},
        hit_objects::{
            hit_samples::{HitSampleInfo, HitSampleInfoName, HitSoundType},
            CurveBuffers, HitObjectKind, HitObjectSlider, HitObjectType, PathType, SplineType,
        },
        metadata::MetadataKey,
        timing_points::{
            ControlPoints, DifficultyPoint, EffectFlags, EffectPoint, SamplePoint, TimingPoint,
        },
    },
    util::Pos,
};

impl Beatmap {
    /// Encode a [`Beatmap`] into content of a `.osu` file and store it at the
    /// given path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rosu_map::Beatmap;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let map: Beatmap = /* ... */
    /// # Beatmap::default();
    /// let path = "./maps/123456.osu";
    /// map.encode_to_path(path)?;
    /// # Ok(()) }
    /// ```
    pub fn encode_to_path<P: AsRef<Path>>(&mut self, path: P) -> IoResult<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        self.encode(writer)
    }

    /// Encode a [`Beatmap`] into content of a `.osu` file and store it into a
    /// [`String`].
    ///
    /// # Example
    ///
    /// ```
    /// # use rosu_map::Beatmap;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let map: Beatmap = /* ... */
    /// # Beatmap::default();
    /// let content: String = map.encode_to_string()?;
    /// # Ok(()) }
    /// ```
    pub fn encode_to_string(&mut self) -> IoResult<String> {
        let mut writer = Vec::with_capacity(4096);
        self.encode(&mut writer)?;

        String::from_utf8(writer).map_err(|e| IoError::new(ErrorKind::Other, e))
    }

    /// Encode a [`Beatmap`] into content of a `.osu` file.
    ///
    /// # Example
    ///
    /// In case of writing directly to a file, it is recommended to pass the
    /// file wrapped in a [`BufWriter`] or just use
    /// [`encode_to_path`].
    ///
    /// ```no_run
    /// use std::{fs::File, io::BufWriter};
    /// # use rosu_map::Beatmap;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let map: Beatmap = /* ... */
    /// # Beatmap::default();
    /// let path = "./maps/123456.osu";
    /// let file = File::create(path)?;
    /// let writer = BufWriter::new(file);
    ///
    /// map.encode(writer)?;
    /// # Ok(()) }
    /// ```
    ///
    /// Encoding into a [`Vec<u8>`] can be done by passing a mutable reference.
    ///
    /// ```
    /// # use rosu_map::Beatmap;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let map: Beatmap = /* ... */
    /// # Beatmap::default();
    /// let mut bytes: Vec<u8> = Vec::with_capacity(2048);
    ///
    /// map.encode(&mut bytes)?;
    ///
    /// // Or just use `Beatmap::encode_to_string`
    /// let content = String::from_utf8(bytes)?;
    ///
    /// # Ok(()) }
    /// ```
    ///
    /// [`encode_to_path`]: Beatmap::encode_to_path
    pub fn encode<W: Write>(&mut self, mut writer: W) -> IoResult<()> {
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

    fn encode_general<W: Write>(&self, writer: &mut W) -> IoResult<()> {
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
            .control_points
            .sample_points
            .first()
            .map_or(SamplePoint::DEFAULT_SAMPLE_BANK, |sample_point| {
                sample_point.sample_bank
            });

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

    fn encode_editor<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        writer.write_all(b"[Editor]\n")?;

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

    fn encode_metadata<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        writer.write_all(b"[Metadata]\n")?;

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

    fn encode_difficulty<W: Write>(&self, writer: &mut W) -> IoResult<()> {
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

    fn encode_events<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        writer.write_all(b"[Events]\n")?;

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

    fn encode_timing_points<W: Write>(&mut self, writer: &mut W) -> IoResult<()> {
        fn output_control_point_at<W: Write>(
            writer: &mut W,
            props: &ControlPointProperties,
            is_timing: bool,
        ) -> IoResult<()> {
            writeln!(
                writer,
                "{},{},{},{},{},{}",
                props.timing_signature,
                props.sample_bank,
                props.custom_sample_bank,
                props.sample_volume,
                if is_timing { "1" } else { "0" },
                props.effect_flags
            )
        }

        let mut control_points = self.control_points.clone();
        let mut bufs = CurveBuffers::default();
        let mut last_sample = None;

        let mut handle_samples = |samples: &[HitSampleInfo], end_time: f64| {
            if samples.is_empty() {
                return;
            }

            // We know the samples aren't empty so we can unwrap
            let volume = samples.iter().map(|sample| sample.volume).max().unwrap();

            let custom_idx = samples
                .iter()
                .map(|sample| sample.custom_sample_bank)
                .max()
                .unwrap();

            let sample = SamplePoint {
                time: end_time,
                sample_bank: SamplePoint::DEFAULT_SAMPLE_BANK,
                sample_volume: volume,
                custom_sample_bank: custom_idx,
            };

            if !last_sample
                .as_ref()
                .is_some_and(|last| sample.is_redundant(last))
            {
                control_points.add(sample.clone());
                last_sample = Some(sample);
            }
        };

        for h in self.hit_objects.iter_mut() {
            let end_time = h.end_time_with_bufs(&mut bufs);
            // FIXME: respect order with samples coming from nested objects
            handle_samples(&h.samples, end_time);

            if let HitObjectKind::Slider(ref mut slider) = h.kind {
                let _curve = slider.path.curve_with_bufs(&mut bufs);

                for _nested_samples in slider.node_samples.iter() {
                    // TODO
                }
            }
        }

        let mut groups: Vec<_> = self
            .control_points
            .timing_points
            .iter()
            .map(ControlPointGroup::from)
            .collect();

        groups.sort_unstable_by(|a, b| a.time.total_cmp(&b.time));

        let times = self
            .control_points
            .difficulty_points
            .iter()
            .map(|point| point.time)
            .chain(control_points.effect_points.iter().map(|point| point.time))
            .chain(control_points.sample_points.iter().map(|point| point.time));

        for time in times {
            if let Err(i) = groups.binary_search_by(|probe| probe.time.total_cmp(&time)) {
                groups.insert(i, ControlPointGroup::new(time));
            }
        }

        writer.write_all(b"[TimingPoints]\n")?;
        let mut last_props = ControlPointProperties::default();

        for group in groups {
            let props = ControlPointProperties::new(
                group.time,
                &control_points,
                &last_props,
                group.timing.is_some(),
            );

            if let Some(timing) = group.timing {
                write!(writer, "{},{},", timing.time, timing.beat_len)?;
                output_control_point_at(writer, &props, true)?;
                last_props = ControlPointProperties {
                    slider_velocity: 1.0,
                    ..props
                };
            }

            if props.is_redundant(&last_props) {
                continue;
            }

            write!(writer, "{},{},", group.time, -100.0 / props.slider_velocity)?;
            output_control_point_at(writer, &props, false)?;
            last_props = props;
        }

        Ok(())
    }

    fn encode_colors<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        writer.write_all(b"[Colours]\n")?;

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

    fn encode_hit_objects<W: Write>(&mut self, writer: &mut W) -> IoResult<()> {
        writer.write_all(b"[HitObjects]\n")?;
        let mut bufs = CurveBuffers::default();

        for hit_object in self.hit_objects.iter_mut() {
            let pos = match hit_object.kind {
                HitObjectKind::Circle(ref h) => h.pos,
                HitObjectKind::Slider(ref h) => h.pos,
                HitObjectKind::Spinner(ref h) => h.pos,
                HitObjectKind::Hold(ref h) => Pos::new(h.pos_x, 192.0),
            };

            write!(
                writer,
                "{x},{y},{start_time},{kind},{sound},",
                x = pos.x,
                y = pos.y,
                start_time = hit_object.start_time,
                kind = i32::from(HitObjectType::from(&*hit_object)),
                sound = u8::from(HitSoundType::from(hit_object.samples.as_slice())),
            )?;

            match hit_object.kind {
                HitObjectKind::Circle(_) => {}
                HitObjectKind::Slider(ref mut h) => {
                    add_path_data(writer, h, pos, self.mode, &mut bufs)?;
                }
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

#[derive(Clone, Default)]
struct ControlPointProperties {
    slider_velocity: f64,
    timing_signature: u32,
    sample_bank: i32,
    custom_sample_bank: i32,
    sample_volume: i32,
    effect_flags: u8,
}

impl ControlPointProperties {
    fn new(
        time: f64,
        control_points: &ControlPoints,
        last_props: &Self,
        update_sample_bank: bool,
    ) -> Self {
        let timing = control_points.timing_point_at(time);
        let difficulty = control_points.difficulty_point_at(time);
        let sample = control_points
            .sample_point_at(time)
            .map_or_else(SamplePoint::default, SamplePoint::clone);
        let effect = control_points.effect_point_at(time);

        let mut tmp_hit_sample =
            HitSampleInfo::new(HitSampleInfoName::File(String::new()), None, 0, 0);
        sample.apply(&mut tmp_hit_sample);

        let mut effect_flags = EffectFlags::NONE;

        if effect.map_or(EffectPoint::DEFAULT_KIAI, |point| point.kiai) {
            effect_flags |= EffectFlags::KIAI;
        }

        if timing.map_or(TimingPoint::DEFAULT_OMIT_FIRST_BAR_LINE, |point| {
            point.omit_first_bar_line
        }) {
            effect_flags |= EffectFlags::OMIT_FIRST_BAR_LINE;
        }

        Self {
            slider_velocity: difficulty.map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            }),
            timing_signature: timing
                .map_or(TimingPoint::DEFAULT_TIME_SIGNATURE, |point| {
                    point.time_signature
                })
                .numerator
                .get(),
            sample_bank: if update_sample_bank {
                tmp_hit_sample.bank as i32
            } else {
                last_props.sample_bank
            },
            custom_sample_bank: if tmp_hit_sample.custom_sample_bank >= 0 {
                tmp_hit_sample.custom_sample_bank
            } else {
                last_props.custom_sample_bank
            },
            sample_volume: tmp_hit_sample.volume,
            effect_flags,
        }
    }

    fn is_redundant(&self, other: &Self) -> bool {
        (self.slider_velocity - other.slider_velocity).abs() < f64::EPSILON
            && self.timing_signature == other.timing_signature
            && self.sample_bank == other.sample_bank
            && self.custom_sample_bank == other.custom_sample_bank
            && self.sample_volume == other.sample_volume
            && self.effect_flags == other.effect_flags
    }
}

struct ControlPointGroup<'a> {
    time: f64,
    timing: Option<&'a TimingPoint>,
}

impl<'a> ControlPointGroup<'a> {
    const fn new(time: f64) -> Self {
        Self { time, timing: None }
    }
}

impl<'a> From<&'a TimingPoint> for ControlPointGroup<'a> {
    fn from(point: &'a TimingPoint) -> Self {
        Self {
            time: point.time,
            timing: Some(point),
        }
    }
}

fn add_path_data<W: Write>(
    writer: &mut W,
    slider: &mut HitObjectSlider,
    pos: Pos,
    mode: GameMode,
    bufs: &mut CurveBuffers,
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

    let dist = slider
        .path
        .expected_dist()
        .unwrap_or_else(|| slider.path.curve_with_bufs(bufs).dist());

    write!(
        writer,
        "{span_count},{dist},",
        span_count = slider.span_count(),
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
    writer: &mut W,
    samples: &[HitSampleInfo],
    banks_only: bool,
    mode: GameMode,
) -> IoResult<()> {
    // osu!lazer throws an error if multiple samples match the filter but
    // we'll just take the first and assume it's the only one.
    let normal_bank = samples
        .iter()
        .find(|sample| sample.name == HitSampleInfo::HIT_NORMAL)
        .map_or(SamplePoint::DEFAULT_SAMPLE_BANK, |sample| sample.bank);

    let add_bank = samples
        .iter()
        .find(|sample| {
            !matches!(
                sample.name,
                HitSampleInfo::HIT_NORMAL | HitSampleInfoName::File(_)
            )
        })
        .map(|sample| sample.bank)
        .unwrap_or_default();

    write!(writer, "{}:{}", normal_bank as i32, add_bank as i32)?;

    if banks_only {
        return Ok(());
    }

    let mut custom_sample_bank = samples
        .iter()
        .find(|sample| matches!(sample.name, HitSampleInfoName::Default(_)))
        .map_or(0, |sample| sample.custom_sample_bank);

    let sample_filename = samples
        .iter()
        .find(|sample| matches!(sample.name, HitSampleInfoName::File(ref filename) if !filename.is_empty()))
        .map(HitSampleInfo::lookup_name);

    let mut volume = samples.first().map_or(100, |sample| sample.volume);

    if mode != GameMode::Mania {
        custom_sample_bank = 0;
        volume = 0;
    }

    write!(writer, ":{custom_sample_bank}:{volume}:")?;

    if let Some(filename) = sample_filename {
        write!(writer, "{filename}")?;
    }

    Ok(())
}
