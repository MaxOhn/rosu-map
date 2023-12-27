use std::io::{Result as IoResult, Write};

use crate::{
    model::{
        beatmap::Beatmap,
        control_points::{DifficultyPoint, EffectPoint, SamplePoint, TimingPoint},
        events::EventType,
        hit_objects::{
            slider::{HitObjectSlider, PathType, SplineType},
            HitObjectKind,
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
    util::Pos,
};

impl Beatmap {
    /// Encode a [`Beatmap`] into content of a `.osu` file.
    ///
    /// In case of writing directly to a file, it is recommended to pass the
    /// file wrapped in a [`BufWriter`].
    ///
    /// [`BufWriter`]: std::io::BufWriter
    pub fn encode<W: Write>(&self, mut writer: W) -> IoResult<()> {
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
        if self.background_file.is_empty() && self.breaks.is_empty() {
            return Ok(());
        }

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

    fn encode_timing_points<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        #[derive(Debug)]
        struct ControlPointGroup<'a> {
            time: f64,
            effect: &'a EffectPoint,
            sample: &'a SamplePoint,
            control: ControlPoint<'a>,
        }

        #[derive(Debug)]
        enum ControlPoint<'a> {
            Timing(&'a TimingPoint),
            Difficulty(&'a DifficultyPoint),
            None,
        }

        // Decoding adds a sample point and an effect point for each line.
        // Unless the control points were modified manually, each point in
        // time will have both a sample and an effect point.
        // FIXME: Handle the case of modified control points.
        let mut groups: Vec<_> = self
            .sample_points
            .iter()
            .zip(self.effect_points.iter())
            .map(|(sample, effect)| ControlPointGroup {
                time: sample.time,
                effect,
                sample,
                control: ControlPoint::None,
            })
            .collect();

        if groups.is_empty() {
            return Ok(());
        }

        groups.sort_unstable_by(|a, b| a.time.total_cmp(&b.time));

        for timing in self.timing_points.iter() {
            // `groups` should have an item for each point in time so this
            // is always `Ok` unless control points were modified manually.
            // FIXME: Handle the case of modified control points.
            if let Ok(i) = groups.binary_search_by(|probe| probe.time.total_cmp(&timing.time)) {
                groups[i].control = ControlPoint::Timing(timing);
            } else {
                eprintln!(
                    "[WARN] Missing timing timestamp {} in groups {groups:?}",
                    timing.time
                );
            }
        }

        for difficulty in self.difficulty_points.iter() {
            // `groups` should have an item for each point in time so this
            // is always `Ok` unless control points were modified manually.
            // FIXME: Handle the case of modified control points.
            if let Ok(i) = groups.binary_search_by(|probe| probe.time.total_cmp(&difficulty.time)) {
                groups[i].control = ControlPoint::Difficulty(difficulty);
            } else {
                eprintln!(
                    "[WARN] Missing difficulty timestamp {} in groups {groups:?}",
                    difficulty.time
                );
            }
        }

        writer.write_all(b"[TimingPoints]\n")?;

        for group in groups {
            let time = group.time;

            let beat_len = match group.control {
                ControlPoint::Timing(point) => point.beat_len,
                ControlPoint::Difficulty(point) => -100.0 * point.bpm_multiplier,
                ControlPoint::None => 1.0,
            };

            let meter = match group.control {
                ControlPoint::Timing(point) => point.time_signature.numerator.get(),
                ControlPoint::Difficulty(_) | ControlPoint::None => 0,
            };

            let sample_set = SampleBank::from_lowercase(group.sample.sample_bank) as i32;
            let sample_idx = group.sample.custom_sample_bank;
            let volume = group.sample.sample_volume;

            let uninherited = match group.control {
                ControlPoint::Timing(_) => 1,
                ControlPoint::Difficulty(_) | ControlPoint::None => 0,
            };

            let mut effects = EffectFlags::NONE;

            if group.effect.kiai {
                effects |= EffectFlags::KIAI;
            }

            if let ControlPoint::Timing(point) = group.control {
                if point.omit_first_bar_line {
                    effects |= EffectFlags::OMIT_FIRST_BAR_LINE;
                }
            }

            writeln!(
                writer,
                "{time},{beat_len},{meter},{sample_set},\
                {sample_idx},{volume},{uninherited},{effects}"
            )?;
        }

        Ok(())
    }

    fn encode_colors<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        if self.custom_combo_colors.is_empty() {
            return Ok(());
        }

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

    fn encode_hit_objects<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        if self.hit_objects.is_empty() {
            return Ok(());
        }

        writer.write_all(b"[HitObjects]\n")?;

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
    writer: &mut W,
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

    let Some(dist) = slider.path.expected_dist() else {
        return Ok(());
    };

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
