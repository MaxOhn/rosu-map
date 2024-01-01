use rosu_map::{
    beatmap::ParseBeatmapError,
    section::{
        colors::{Color, Colors},
        difficulty::Difficulty,
        editor::Editor,
        events::{BreakPeriod, Events},
        general::{CountdownType, GameMode, General},
        hit_objects::{
            hit_samples::{HitSampleInfoName, SampleBank},
            CurveBuffers, HitObject, HitObjectKind, HitObjects, PathType,
        },
        metadata::Metadata,
        timing_points::{
            DifficultyPoint, EffectPoint, SamplePoint, TimeSignature, TimingPoint, TimingPoints,
        },
    },
    util::Pos,
    Beatmap, ParseVersionError,
};

const RENATUS: &str = include_str!("../resources/Soleily - Renatus (Gamu) [Insane].osu");

#[test]
fn format_version() {
    let map: Beatmap = rosu_map::from_path("../resources/beatmap-version-4.osu").unwrap();

    assert_eq!(map.format_version, 4);
    assert_eq!(map.preview_time, -1);
}

#[test]
fn general() {
    let general: General = rosu_map::from_str(RENATUS).unwrap();

    assert_eq!(general.audio_file, "03. Renatus - Soleily 192kbps.mp3");
    assert_eq!(general.audio_lead_in, 0.0);
    assert_eq!(general.preview_time, 164471);
    assert_eq!(general.stack_leniency, 0.7);
    assert_eq!(general.mode, GameMode::Osu);
    assert_eq!(general.letterbox_in_breaks, false);
    assert_eq!(general.special_style, false);
    assert_eq!(general.widescreen_storyboard, false);
    assert_eq!(general.samples_match_playback_rate, false);
    assert_eq!(general.countdown, CountdownType::None);
    assert_eq!(general.countdown_offset, 0);
}

#[test]
fn editor() {
    let editor: Editor = rosu_map::from_str(RENATUS).unwrap();

    let expected_bookmarks = [
        11505, 22054, 32604, 43153, 53703, 64252, 74802, 85351, 95901, 106450, 116999, 119637,
        130186, 140735, 151285, 161834, 164471, 175020, 185570, 196119, 206669, 209306,
    ];

    assert_eq!(editor.bookmarks, expected_bookmarks);
    assert_eq!(editor.distance_spacing, 1.8);
    assert_eq!(editor.beat_divisor, 4);
    assert_eq!(editor.grid_size, 4);
    assert_eq!(editor.timeline_zoom, 2.0);
}

#[test]
fn metadata() {
    let metadata: Metadata = rosu_map::from_str(RENATUS).unwrap();

    assert_eq!(metadata.title, "Renatus");
    assert_eq!(metadata.title_unicode, "Renatus");
    assert_eq!(metadata.artist, "Soleily");
    assert_eq!(metadata.artist_unicode, "Soleily");
    assert_eq!(metadata.creator, "Gamu");
    assert_eq!(metadata.version, "Insane");
    assert_eq!(metadata.source, "");
    assert_eq!(metadata.tags, "MBC7 Unisphere 地球ヤバイEP Chikyu Yabai");
    assert_eq!(metadata.beatmap_id, 557821);
    assert_eq!(metadata.beatmap_set_id, 241526);
}

#[test]
fn difficulty() {
    let difficulty: Difficulty = rosu_map::from_str(RENATUS).unwrap();

    assert_eq!(difficulty.hp_drain_rate, 6.5);
    assert_eq!(difficulty.circle_size, 4.0);
    assert_eq!(difficulty.overall_difficulty, 8.0);
    assert_eq!(difficulty.approach_rate, 9.0);
    assert_eq!(difficulty.slider_multiplier, 1.8);
    assert_eq!(difficulty.slider_tick_rate, 2.0);
}

#[test]
fn events() {
    let events: Events = rosu_map::from_str(RENATUS).unwrap();

    let break_point = BreakPeriod {
        start_time: 122474.0,
        end_time: 140135.0,
    };

    assert_eq!(events.background_file, "machinetop_background.jpg");
    assert_eq!(events.breaks[0], break_point);
    assert!(events.breaks[0].has_effect());
}

#[test]
fn video_lowercase_ext() {
    let events: Events =
        rosu_map::from_path("../resources/video-with-lowercase-extension.osb").unwrap();

    assert_eq!(events.background_file, "BG.jpg");
}

#[test]
fn video_uppercase_ext() {
    let events: Events =
        rosu_map::from_path("../resources/video-with-uppercase-extension.osb").unwrap();

    assert_eq!(events.background_file, "BG.jpg");
}

#[test]
fn image_as_video() {
    let events: Events = rosu_map::from_path("../resources/image-specified-as-video.osb").unwrap();

    assert_eq!(events.background_file, "BG.jpg");
}

#[test]
fn timing_points() {
    let control_points: TimingPoints = rosu_map::from_str(RENATUS).unwrap();

    assert_eq!(control_points.timing_points.len(), 4);
    assert_eq!(control_points.difficulty_points.len(), 5);
    assert_eq!(control_points.sample_points.len(), 34);
    assert_eq!(control_points.effect_points.len(), 8);

    let timing_point = control_points
        .timing_point_at(0.0)
        .map_or_else(TimingPoint::default, TimingPoint::clone);
    assert_eq!(timing_point.time, 956.0);
    assert_eq!(timing_point.beat_len, 329.67032967033);
    assert_eq!(
        timing_point.time_signature,
        TimeSignature::new_simple_quadruple()
    );
    assert_eq!(timing_point.omit_first_bar_line, false);

    let timing_point = control_points
        .timing_point_at(48428.0)
        .map_or_else(TimingPoint::default, TimingPoint::clone);
    assert_eq!(timing_point.time, 956.0);
    assert_eq!(timing_point.beat_len, 329.67032967033);
    assert_eq!(
        timing_point.time_signature,
        TimeSignature::new_simple_quadruple()
    );
    assert_eq!(timing_point.omit_first_bar_line, false);

    let timing_point = control_points
        .timing_point_at(119637.0)
        .map_or_else(TimingPoint::default, TimingPoint::clone);
    assert_eq!(timing_point.time, 119637.0);
    assert_eq!(timing_point.beat_len, 659.340659340659);
    assert_eq!(
        timing_point.time_signature,
        TimeSignature::new_simple_quadruple()
    );
    assert_eq!(timing_point.omit_first_bar_line, false);

    let difficulty_point = control_points
        .difficulty_point_at(0.0)
        .map_or_else(DifficultyPoint::default, DifficultyPoint::clone);
    assert_eq!(difficulty_point.time, 0.0);
    assert_eq!(difficulty_point.slider_velocity, 1.0);

    let difficulty_point = control_points
        .difficulty_point_at(48428.0)
        .map_or_else(DifficultyPoint::default, DifficultyPoint::clone);
    assert_eq!(difficulty_point.time, 0.0);
    assert_eq!(difficulty_point.slider_velocity, 1.0);

    let difficulty_point = control_points
        .difficulty_point_at(116999.0)
        .map_or_else(DifficultyPoint::default, DifficultyPoint::clone);
    assert_eq!(difficulty_point.time, 116999.0);
    assert!((difficulty_point.slider_velocity - 0.75).abs() <= 0.1);

    let sound_point = control_points
        .sample_point_at(0.0)
        .map_or_else(SamplePoint::default, SamplePoint::clone);
    assert_eq!(sound_point.time, 956.0);
    assert_eq!(sound_point.sample_bank, SampleBank::Soft);
    assert_eq!(sound_point.sample_volume, 60);

    let sound_point = control_points
        .sample_point_at(53373.0)
        .map_or_else(SamplePoint::default, SamplePoint::clone);
    assert_eq!(sound_point.time, 53373.0);
    assert_eq!(sound_point.sample_bank, SampleBank::Soft);
    assert_eq!(sound_point.sample_volume, 60);

    let sound_point = control_points
        .sample_point_at(119637.0)
        .map_or_else(SamplePoint::default, SamplePoint::clone);
    assert_eq!(sound_point.time, 119637.0);
    assert_eq!(sound_point.sample_bank, SampleBank::Soft);
    assert_eq!(sound_point.sample_volume, 80);

    let effect_point = control_points
        .effect_point_at(0.0)
        .map_or_else(EffectPoint::default, EffectPoint::clone);
    assert_eq!(effect_point.time, 0.0);
    assert_eq!(effect_point.kiai, false);

    let effect_point = control_points
        .effect_point_at(53703.0)
        .map_or_else(EffectPoint::default, EffectPoint::clone);
    assert_eq!(effect_point.time, 53703.0);
    assert_eq!(effect_point.kiai, true);

    let effect_point = control_points
        .effect_point_at(116637.0)
        .map_or_else(EffectPoint::default, EffectPoint::clone);
    assert_eq!(effect_point.time, 95901.0);
    assert_eq!(effect_point.kiai, false);
}

#[test]
fn overlapping_timing_points() {
    fn slider_velocity_at(control_points: &TimingPoints, time: f64) -> f64 {
        control_points
            .difficulty_point_at(time)
            .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            })
    }

    fn kiai_at(control_points: &TimingPoints, time: f64) -> bool {
        control_points
            .effect_point_at(time)
            .map_or(EffectPoint::DEFAULT_KIAI, |point| point.kiai)
    }

    fn sample_bank_at(control_points: &TimingPoints, time: f64) -> SampleBank {
        control_points
            .sample_point_at(time)
            .map_or(SamplePoint::DEFAULT_SAMPLE_BANK, |point| point.sample_bank)
    }

    fn beat_len_at(control_points: &TimingPoints, time: f64) -> f64 {
        control_points
            .timing_point_at(time)
            .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len)
    }

    let control_points: TimingPoints =
        rosu_map::from_path("../resources/overlapping-control-points.osu").unwrap();

    assert_eq!(control_points.timing_points.len(), 4);
    assert_eq!(control_points.difficulty_points.len(), 3);
    assert_eq!(control_points.effect_points.len(), 3);
    assert_eq!(control_points.sample_points.len(), 3);

    assert!((slider_velocity_at(&control_points, 500.0) - 1.5).abs() <= 0.1);
    assert!((slider_velocity_at(&control_points, 1500.0) - 1.5).abs() <= 0.1);
    assert!((slider_velocity_at(&control_points, 2500.0) - 0.75).abs() <= 0.1);
    assert!((slider_velocity_at(&control_points, 3500.0) - 1.5).abs() <= 0.1);

    assert_eq!(kiai_at(&control_points, 500.0), true);
    assert_eq!(kiai_at(&control_points, 1500.0), true);
    assert_eq!(kiai_at(&control_points, 2500.0), false);
    assert_eq!(kiai_at(&control_points, 3500.0), true);

    assert_eq!(sample_bank_at(&control_points, 500.0), SampleBank::Drum);
    assert_eq!(sample_bank_at(&control_points, 1500.0), SampleBank::Drum);
    assert_eq!(sample_bank_at(&control_points, 2500.0), SampleBank::Normal);
    assert_eq!(sample_bank_at(&control_points, 3500.0), SampleBank::Drum);

    assert!((beat_len_at(&control_points, 500.0) - 500.0).abs() <= 0.1);
    assert!((beat_len_at(&control_points, 1500.0) - 500.0).abs() <= 0.1);
    assert!((beat_len_at(&control_points, 2500.0) - 250.0).abs() <= 0.1);
    assert!((beat_len_at(&control_points, 3500.0) - 500.0).abs() <= 0.1);
}

#[test]
fn omit_bar_line_effect() {
    fn omit_first_bar_line_at(control_points: &TimingPoints, time: f64) -> bool {
        control_points
            .timing_point_at(time)
            .map_or(TimingPoint::DEFAULT_OMIT_FIRST_BAR_LINE, |point| {
                point.omit_first_bar_line
            })
    }

    let control_points: TimingPoints =
        rosu_map::from_path("../resources/omit-barline-control-points.osu").unwrap();

    assert_eq!(control_points.timing_points.len(), 6);
    assert_eq!(control_points.effect_points.len(), 0);

    assert_eq!(omit_first_bar_line_at(&control_points, 500.0), false);
    assert_eq!(omit_first_bar_line_at(&control_points, 1500.0), true);
    assert_eq!(omit_first_bar_line_at(&control_points, 2500.0), false);
    assert_eq!(omit_first_bar_line_at(&control_points, 3500.0), false);
    assert_eq!(omit_first_bar_line_at(&control_points, 4500.0), false);
    assert_eq!(omit_first_bar_line_at(&control_points, 5500.0), true);
}

#[test]
fn timing_point_resets_speed_multiplier() {
    fn slider_velocity_at(control_points: &TimingPoints, time: f64) -> f64 {
        control_points
            .difficulty_point_at(time)
            .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            })
    }

    let control_points: TimingPoints =
        rosu_map::from_path("../resources/timingpoint-speedmultiplier-reset.osu").unwrap();

    assert!((slider_velocity_at(&control_points, 0.0) - 0.5).abs() <= 0.1);
    assert!((slider_velocity_at(&control_points, 2000.0) - 1.0).abs() <= 0.1);
}

#[test]
fn colors() {
    let colors: Colors = rosu_map::from_str(RENATUS).unwrap();

    let expected_colors = vec![
        Color::new(142, 199, 255, 255),
        Color::new(255, 128, 128, 255),
        Color::new(128, 255, 255, 255),
        Color::new(128, 255, 128, 255),
        Color::new(255, 187, 255, 255),
        Color::new(255, 177, 140, 255),
        Color::new(100, 100, 100, 255),
    ];

    assert_eq!(colors.custom_combo_colors, expected_colors);
}

#[test]
fn get_last_object_time() {
    let mut hit_objects = rosu_map::from_path::<HitObjects>("mania-last-object-not-latest.osu")
        .unwrap()
        .hit_objects;

    let last_object = hit_objects.last_mut().unwrap();

    assert_eq!(last_object.start_time, 2494.0);
    assert_eq!(last_object.end_time(), 2494.0);
    assert_eq!(
        hit_objects
            .iter_mut()
            .fold(0.0, |max, h| h.end_time().max(max)),
        2582.0
    );
}

#[test]
fn combo_offset_osu() {
    fn combo_offset(hit_object: &HitObject) -> i32 {
        match hit_object.kind {
            HitObjectKind::Circle(ref h) => h.combo_offset,
            HitObjectKind::Slider(ref h) => h.combo_offset,
            HitObjectKind::Spinner(_) | HitObjectKind::Hold(_) => {
                panic!("expected circle or slider")
            }
        }
    }

    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/hitobject-combo-offset.osu")
        .unwrap()
        .hit_objects;

    assert_eq!(combo_offset(&hit_objects[0]), 0);
    assert_eq!(combo_offset(&hit_objects[2]), 2);
    assert_eq!(combo_offset(&hit_objects[3]), 4);
    assert_eq!(combo_offset(&hit_objects[4]), 6);
    assert_eq!(combo_offset(&hit_objects[8]), 8);
    assert_eq!(combo_offset(&hit_objects[9]), 11);
}

#[test]
fn hit_objects() {
    let hit_objects = rosu_map::from_str::<HitObjects>(RENATUS)
        .unwrap()
        .hit_objects;

    assert_eq!(hit_objects[0].start_time, 956.0);

    assert!(hit_objects[0]
        .samples
        .iter()
        .any(|sample| sample.name == Some(HitSampleInfoName::Normal)));

    let HitObjectKind::Slider(ref slider) = hit_objects[0].kind else {
        panic!("Expected slider")
    };

    assert_eq!(slider.pos, Pos::new(192.0, 168.0));
    assert_eq!(hit_objects[1].start_time, 1285.0);

    assert!(hit_objects[1]
        .samples
        .iter()
        .any(|sample| sample.name == Some(HitSampleInfoName::Clap)));

    let HitObjectKind::Circle(ref circle) = hit_objects[1].kind else {
        panic!("Expected circle")
    };

    assert_eq!(circle.pos, Pos::new(304.0, 56.0));
}

#[test]
fn control_point_difficulty_change() {
    fn slider_velocity_at(control_points: &TimingPoints, time: f64) -> f64 {
        control_points
            .difficulty_point_at(time)
            .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            })
    }

    let control_points: TimingPoints =
        rosu_map::from_path("../resources/controlpoint-difficulty-multiplier.osu").unwrap();

    assert_eq!(slider_velocity_at(&control_points, 5.0), 1.0);
    assert_eq!(slider_velocity_at(&control_points, 1000.0), 10.0);
    assert_eq!(
        slider_velocity_at(&control_points, 2000.0),
        1.8518518518518519
    );
    assert_eq!(slider_velocity_at(&control_points, 3000.0), 0.5);
}

#[test]
fn control_point_custom_sample_bank() {
    fn assert_lookup_name(hit_object: &HitObject, name: &str) {
        assert_eq!(hit_object.samples[0].lookup_name().as_deref(), Some(name));
    }

    let hit_objects =
        rosu_map::from_path::<HitObjects>("../resources/controlpoint-custom-samplebank.osu")
            .unwrap()
            .hit_objects;

    assert_lookup_name(&hit_objects[0], "Gameplay/normal-hitnormal");
    assert_lookup_name(&hit_objects[1], "Gameplay/normal-hitnormal");
    assert_lookup_name(&hit_objects[2], "Gameplay/normal-hitnormal2");
    assert_lookup_name(&hit_objects[3], "Gameplay/normal-hitnormal");
    assert_lookup_name(&hit_objects[4], "Gameplay/soft-hitnormal8");
}

#[test]
fn hit_object_custom_sample_bank() {
    fn assert_lookup_name(hit_object: &HitObject, name: &str) {
        assert_eq!(hit_object.samples[0].lookup_name().as_deref(), Some(name));
    }

    let hit_objects =
        rosu_map::from_path::<HitObjects>("../resources/hitobject-custom-samplebank.osu")
            .unwrap()
            .hit_objects;

    assert_lookup_name(&hit_objects[0], "Gameplay/normal-hitnormal");
    assert_lookup_name(&hit_objects[1], "Gameplay/normal-hitnormal2");
    assert_lookup_name(&hit_objects[2], "Gameplay/normal-hitnormal3");
}

#[test]
fn hit_object_file_samples() {
    fn assert_lookup_name(hit_object: &HitObject, name: &str) {
        assert_eq!(hit_object.samples[0].lookup_name().as_deref(), Some(name));
    }

    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/hitobject-file-samples.osu")
        .unwrap()
        .hit_objects;

    assert_lookup_name(&hit_objects[0], "hit_1.wav");
    assert_lookup_name(&hit_objects[1], "hit_2.wav");
    assert_lookup_name(&hit_objects[2], "Gameplay/normal-hitnormal2");
    assert_lookup_name(&hit_objects[3], "hit_1.wav");
    assert_eq!(hit_objects[3].samples[0].volume, 70);
}

#[test]
fn slider_samples() {
    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/slider-samples.osu")
        .unwrap()
        .hit_objects;

    let (
        HitObjectKind::Slider(slider1),
        HitObjectKind::Slider(slider2),
        HitObjectKind::Slider(slider3),
    ) = (
        &hit_objects[0].kind,
        &hit_objects[1].kind,
        &hit_objects[2].kind,
    )
    else {
        panic!("Expected three sliders")
    };

    assert_eq!(slider1.node_samples[0].len(), 1);
    assert_eq!(
        slider1.node_samples[0][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(slider1.node_samples[1].len(), 1);
    assert_eq!(
        slider1.node_samples[1][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(slider1.node_samples[2].len(), 1);
    assert_eq!(
        slider1.node_samples[2][0].name,
        Some(HitSampleInfoName::Normal)
    );

    assert_eq!(slider2.node_samples[0].len(), 2);
    assert_eq!(
        slider2.node_samples[0][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(
        slider2.node_samples[0][1].name,
        Some(HitSampleInfoName::Clap)
    );
    assert_eq!(slider2.node_samples[1].len(), 2);
    assert_eq!(
        slider2.node_samples[1][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(
        slider2.node_samples[1][1].name,
        Some(HitSampleInfoName::Clap)
    );
    assert_eq!(slider2.node_samples[2].len(), 2);
    assert_eq!(
        slider2.node_samples[2][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(
        slider2.node_samples[2][1].name,
        Some(HitSampleInfoName::Clap)
    );

    assert_eq!(slider3.node_samples[0].len(), 2);
    assert_eq!(
        slider3.node_samples[0][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(
        slider3.node_samples[0][1].name,
        Some(HitSampleInfoName::Whistle)
    );
    assert_eq!(slider3.node_samples[1].len(), 1);
    assert_eq!(
        slider3.node_samples[1][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(slider3.node_samples[2].len(), 2);
    assert_eq!(
        slider3.node_samples[2][0].name,
        Some(HitSampleInfoName::Normal)
    );
    assert_eq!(
        slider3.node_samples[2][1].name,
        Some(HitSampleInfoName::Clap)
    );
}

#[test]
fn hit_object_no_addition_bank() {
    let hit_objects: HitObjects =
        rosu_map::from_path("../resources/hitobject-no-addition-bank.osu").unwrap();

    assert_eq!(
        hit_objects.hit_objects[0].samples[0].bank,
        hit_objects.hit_objects[0].samples[1].bank
    );
}

#[test]
fn invalid_event_pass() {
    let _events: Events = rosu_map::from_path("../resources/invalid-events.osu").unwrap();
}

#[test]
fn invalid_bank_defaults_to_normal() {
    fn assert_object_has_banks(
        h: &HitObject,
        normal_bank: SampleBank,
        additions_bank: Option<SampleBank>,
    ) {
        assert_eq!(normal_bank, h.samples[0].bank);

        if let Some(additions_bank) = additions_bank {
            assert_eq!(additions_bank, h.samples[1].bank);
        }
    }

    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/invalid-bank.osu")
        .unwrap()
        .hit_objects;

    assert_object_has_banks(&hit_objects[0], SampleBank::Drum, None);
    assert_object_has_banks(&hit_objects[1], SampleBank::Normal, None);
    assert_object_has_banks(&hit_objects[2], SampleBank::Soft, None);
    assert_object_has_banks(&hit_objects[3], SampleBank::Drum, None);
    assert_object_has_banks(&hit_objects[4], SampleBank::Normal, None);

    assert_object_has_banks(&hit_objects[5], SampleBank::Drum, Some(SampleBank::Drum));
    assert_object_has_banks(&hit_objects[6], SampleBank::Drum, Some(SampleBank::Normal));
    assert_object_has_banks(&hit_objects[7], SampleBank::Drum, Some(SampleBank::Soft));
    assert_object_has_banks(&hit_objects[8], SampleBank::Drum, Some(SampleBank::Drum));
    assert_object_has_banks(&hit_objects[9], SampleBank::Drum, Some(SampleBank::Normal));
}

#[test]
fn corrupted_header() {
    let metadata: Metadata = rosu_map::from_path("../resources/corrupted-header.osu").unwrap();

    assert_eq!(metadata.title, "Beatmap with corrupted header");
    assert_eq!(metadata.creator, "Evil Hacker");
}

#[test]
fn missing_header() {
    let metadata: Metadata = rosu_map::from_path("../resources/missing-header.osu").unwrap();

    assert_eq!(metadata.title, "Beatmap with no header");
    assert_eq!(metadata.creator, "Incredibly Evil Hacker");
}

#[test]
fn empty_lines_at_start() {
    let metadata: Metadata = rosu_map::from_path("../resources/empty-lines-at-start.osu").unwrap();

    assert_eq!(metadata.title, "Empty lines at start");
    assert_eq!(metadata.creator, "Edge Case Hunter");
}

#[test]
fn empty_lines_without_header() {
    let metadata: Metadata =
        rosu_map::from_path("../resources/empty-line-instead-of-header.osu").unwrap();

    assert_eq!(metadata.title, "The dog ate the file header");
    assert_eq!(metadata.creator, "Why does this keep happening");
}

#[test]
fn no_blank_after_header() {
    let metadata: Metadata =
        rosu_map::from_path("../resources/no-empty-line-after-header.osu").unwrap();

    assert_eq!(
        metadata.title,
        "No empty line delimiting header from contents"
    );
    assert_eq!(metadata.creator, "Edge Case Hunter");
}

#[test]
fn empty_file() {
    assert!(matches!(
        rosu_map::from_bytes::<Beatmap>(&[]),
        Err(ParseBeatmapError::FormatVersion(
            ParseVersionError::UnknownFileFormat
        ))
    ));
}

#[test]
fn multi_segment_sliders() {
    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/multi-segment-slider.osu")
        .unwrap()
        .hit_objects;

    let HitObjectKind::Slider(ref slider) = hit_objects[0].kind else {
        panic!("Expected slider")
    };

    let first = slider.path.control_points();

    assert_eq!(first[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(first[0].path_type, Some(PathType::PERFECT_CURVE));
    assert_eq!(first[1].pos, Pos::new(161.0, -244.0));
    assert_eq!(first[1].path_type, None);

    assert_eq!(first[2].pos, Pos::new(376.0, -3.0));
    assert_eq!(first[2].path_type, Some(PathType::BEZIER));
    assert_eq!(first[3].pos, Pos::new(68.0, 15.0));
    assert_eq!(first[3].path_type, None);
    assert_eq!(first[4].pos, Pos::new(259.0, -132.0));
    assert_eq!(first[4].path_type, None);
    assert_eq!(first[5].pos, Pos::new(92.0, -107.0));
    assert_eq!(first[5].path_type, None);

    let HitObjectKind::Slider(ref slider) = hit_objects[1].kind else {
        panic!("Expected slider")
    };

    let second = slider.path.control_points();

    assert_eq!(second[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(second[0].path_type, Some(PathType::PERFECT_CURVE));
    assert_eq!(second[1].pos, Pos::new(161.0, -244.0));
    assert_eq!(second[1].path_type, None);
    assert_eq!(second[2].pos, Pos::new(376.0, -3.0));
    assert_eq!(second[2].path_type, None);

    let HitObjectKind::Slider(ref slider) = hit_objects[2].kind else {
        panic!("Expected slider")
    };

    let third = slider.path.control_points();

    assert_eq!(third[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(third[0].path_type, Some(PathType::BEZIER));
    assert_eq!(third[1].pos, Pos::new(0.0, 192.0));
    assert_eq!(third[1].path_type, None);
    assert_eq!(third[2].pos, Pos::new(224.0, 192.0));
    assert_eq!(third[2].path_type, None);

    assert_eq!(third[3].pos, Pos::new(224.0, 0.0));
    assert_eq!(third[3].path_type, Some(PathType::BEZIER));
    assert_eq!(third[4].pos, Pos::new(224.0, -192.0));
    assert_eq!(third[4].path_type, None);
    assert_eq!(third[5].pos, Pos::new(480.0, -192.0));
    assert_eq!(third[5].path_type, None);
    assert_eq!(third[6].pos, Pos::new(480.0, 0.0));
    assert_eq!(third[6].path_type, None);

    let HitObjectKind::Slider(ref slider) = hit_objects[3].kind else {
        panic!("Expected slider")
    };

    let fourth = slider.path.control_points();

    assert_eq!(fourth[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(fourth[0].path_type, Some(PathType::BEZIER));
    assert_eq!(fourth[1].pos, Pos::new(1.0, 1.0));
    assert_eq!(fourth[1].path_type, None);
    assert_eq!(fourth[2].pos, Pos::new(2.0, 2.0));
    assert_eq!(fourth[2].path_type, None);
    assert_eq!(fourth[3].pos, Pos::new(3.0, 3.0));
    assert_eq!(fourth[3].path_type, None);
    assert_eq!(fourth[4].pos, Pos::new(3.0, 3.0));
    assert_eq!(fourth[4].path_type, None);

    let HitObjectKind::Slider(ref slider) = hit_objects[4].kind else {
        panic!("Expected slider")
    };

    let fifth = slider.path.control_points();

    assert_eq!(fifth[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(fifth[0].path_type, Some(PathType::BEZIER));
    assert_eq!(fifth[1].pos, Pos::new(1.0, 1.0));
    assert_eq!(fifth[1].path_type, None);
    assert_eq!(fifth[2].pos, Pos::new(2.0, 2.0));
    assert_eq!(fifth[2].path_type, None);
    assert_eq!(fifth[3].pos, Pos::new(3.0, 3.0));
    assert_eq!(fifth[3].path_type, None);
    assert_eq!(fifth[4].pos, Pos::new(3.0, 3.0));
    assert_eq!(fifth[4].path_type, None);

    assert_eq!(fifth[5].pos, Pos::new(4.0, 4.0));
    assert_eq!(fifth[5].path_type, Some(PathType::BEZIER));
    assert_eq!(fifth[6].pos, Pos::new(5.0, 5.0));
    assert_eq!(fifth[6].path_type, None);

    let HitObjectKind::Slider(ref slider) = hit_objects[5].kind else {
        panic!("Expected slider")
    };

    let sixth = slider.path.control_points();

    assert_eq!(sixth[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(sixth[0].path_type, Some(PathType::BEZIER));
    assert_eq!(sixth[1].pos, Pos::new(75.0, 145.0));
    assert_eq!(sixth[1].path_type, None);

    assert_eq!(sixth[2].pos, Pos::new(170.0, 75.0));
    assert_eq!(sixth[2].path_type, Some(PathType::BEZIER));
    assert_eq!(sixth[3].pos, Pos::new(300.0, 145.0));
    assert_eq!(sixth[3].path_type, None);
    assert_eq!(sixth[4].pos, Pos::new(410.0, 20.0));
    assert_eq!(sixth[4].path_type, None);

    let HitObjectKind::Slider(ref slider) = hit_objects[6].kind else {
        panic!("Expected slider")
    };

    let seventh = slider.path.control_points();

    assert_eq!(seventh[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(seventh[0].path_type, Some(PathType::PERFECT_CURVE));
    assert_eq!(seventh[1].pos, Pos::new(75.0, 145.0));
    assert_eq!(seventh[1].path_type, None);

    assert_eq!(seventh[2].pos, Pos::new(170.0, 75.0));
    assert_eq!(seventh[2].path_type, Some(PathType::PERFECT_CURVE));
    assert_eq!(seventh[3].pos, Pos::new(300.0, 145.0));
    assert_eq!(seventh[3].path_type, None);
    assert_eq!(seventh[4].pos, Pos::new(410.0, 20.0));
    assert_eq!(seventh[4].path_type, None);
}

#[test]
fn slider_len_extension_edge_case() {
    let hit_objects =
        rosu_map::from_path::<HitObjects>("../resources/duplicate-last_position-slider.osu")
            .unwrap()
            .hit_objects;

    let HitObjectKind::Slider(ref slider) = hit_objects[0].kind else {
        panic!("Expected slider")
    };

    assert_eq!(slider.path.expected_dist(), Some(2.0));

    let mut bufs = CurveBuffers::default();
    assert_eq!(slider.path.borrowed_curve(&mut bufs).dist(), 1.0);
}

#[test]
fn undefined_ar_inherits_od() {
    let difficulty: Difficulty =
        rosu_map::from_path("../resources/undefined-approach-rate.osu").unwrap();

    assert_eq!(difficulty.approach_rate, 1.0);
    assert_eq!(difficulty.overall_difficulty, 1.0);
}

#[test]
fn ar_before_od() {
    let difficulty: Difficulty =
        rosu_map::from_path("../resources/approach-rate-before-overall-difficulty.osu").unwrap();

    assert_eq!(difficulty.approach_rate, 9.0);
    assert_eq!(difficulty.overall_difficulty, 1.0);
}

#[test]
fn ar_after_od() {
    let difficulty: Difficulty =
        rosu_map::from_path("../resources/approach-rate-after-overall-difficulty.osu").unwrap();

    assert_eq!(difficulty.approach_rate, 9.0);
    assert_eq!(difficulty.overall_difficulty, 1.0);
}

#[test]
fn adjacent_implicit_catmull_segments_merged() {
    let mut hit_objects =
        rosu_map::from_path::<HitObjects>("../resources/adjacent-catmull-segments.osu")
            .unwrap()
            .hit_objects;

    let HitObjectKind::Slider(ref mut slider) = hit_objects[0].kind else {
        panic!("Expected slider")
    };

    let control_points = slider.path.control_points_mut();

    assert_eq!(control_points.len(), 6);

    control_points.retain(|point| point.path_type.is_some());
    assert_eq!(control_points.len(), 1);
    assert_eq!(control_points[0].path_type, Some(PathType::CATMULL));
}

#[test]
fn duplicate_initial_catmull_point_merged() {
    let hit_objects = rosu_map::from_path::<HitObjects>(
        "../resources/catmull-duplicate-initial-controlpoint.osu",
    )
    .unwrap()
    .hit_objects;

    let HitObjectKind::Slider(ref slider) = hit_objects[0].kind else {
        panic!("Expected slider")
    };

    let control_points = slider.path.control_points();

    assert_eq!(control_points.len(), 4);
    assert_eq!(control_points[0].path_type, Some(PathType::CATMULL));
    assert_eq!(control_points[0].pos, Pos::new(0.0, 0.0));
    assert_eq!(control_points[1].path_type, None);
    assert_ne!(control_points[1].pos, Pos::new(0.0, 0.0));
}

#[test]
fn nan_control_points() {
    fn beat_len_at(control_points: &TimingPoints, time: f64) -> f64 {
        control_points
            .timing_point_at(time)
            .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len)
    }

    fn slider_velocity_at(control_points: &TimingPoints, time: f64) -> f64 {
        control_points
            .difficulty_point_at(time)
            .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            })
    }

    fn generate_ticks_at(control_points: &TimingPoints, time: f64) -> bool {
        control_points
            .difficulty_point_at(time)
            .map_or(DifficultyPoint::DEFAULT_GENERATE_TICKS, |point| {
                point.generate_ticks
            })
    }

    let control_points: TimingPoints =
        rosu_map::from_path("../resources/nan-control-points.osu").unwrap();

    assert_eq!(control_points.timing_points.len(), 1);
    assert_eq!(control_points.difficulty_points.len(), 2);

    assert_eq!(beat_len_at(&control_points, 1000.0), 500.0);

    assert_eq!(slider_velocity_at(&control_points, 2000.0), 1.0);
    assert_eq!(slider_velocity_at(&control_points, 3000.0), 1.0);

    assert_eq!(generate_ticks_at(&control_points, 2000.0), false);
    assert_eq!(generate_ticks_at(&control_points, 3000.0), true);
}

#[test]
fn sample_point_leniency() {
    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/sample-point-leniency.osu")
        .unwrap()
        .hit_objects;

    let [hit_object] = hit_objects.as_slice() else {
        panic!("Expected a single hitobject")
    };

    assert!(hit_object.samples.iter().all(|sample| sample.volume == 70));
}

#[test]
fn new_combo_after_break() {
    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/break-between-objects.osu")
        .unwrap()
        .hit_objects;

    assert!(hit_objects[0].new_combo());
    assert!(hit_objects[1].new_combo());
    assert!(!hit_objects[2].new_combo());
}

#[test]
fn spinner_new_combo_between_objects() {
    fn combo_offset(hit_object: &HitObject) -> i32 {
        match hit_object.kind {
            HitObjectKind::Circle(ref h) => h.combo_offset,
            HitObjectKind::Slider(ref h) => h.combo_offset,
            HitObjectKind::Spinner(_) | HitObjectKind::Hold(_) => {
                panic!("expected circle or slider")
            }
        }
    }

    let hit_objects = rosu_map::from_path::<HitObjects>("../resources/spinner-between-objects.osu")
        .unwrap()
        .hit_objects;

    assert_eq!(combo_offset(&hit_objects[0]), 1);
    assert_eq!(combo_offset(&hit_objects[2]), 2);
    assert_eq!(combo_offset(&hit_objects[3]), 2);
    assert_eq!(combo_offset(&hit_objects[5]), 3);
    assert_eq!(combo_offset(&hit_objects[6]), 3);
    assert_eq!(combo_offset(&hit_objects[8]), 4);
    assert_eq!(combo_offset(&hit_objects[9]), 4);
    assert_eq!(combo_offset(&hit_objects[11]), 5);
    assert_eq!(combo_offset(&hit_objects[12]), 6);
    assert_eq!(combo_offset(&hit_objects[14]), 7);
    assert_eq!(combo_offset(&hit_objects[15]), 8);
    assert_eq!(combo_offset(&hit_objects[17]), 9);
}

#[test]
fn slider_conversion_with_custom_dist() {
    let mut hit_objects =
        rosu_map::from_path::<HitObjects>("../resources/custom-slider-length.osu")
            .unwrap()
            .hit_objects;

    let first = hit_objects.first_mut().unwrap();

    assert_eq!(first.end_time(), 3153.0);
}
