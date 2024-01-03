use std::{fs, num::NonZeroI32};

use rosu_map::{
    section::hit_objects::{
        HitObject, HitObjectKind, HitObjectSlider, PathControlPoint, PathType, SliderPath,
    },
    util::Pos,
    Beatmap,
};
use test_log::test;

#[test]
fn stability() {
    for entry in fs::read_dir("./resources").unwrap() {
        let entry = entry.unwrap();
        let filename = entry.file_name();

        if !filename.to_string_lossy().ends_with(".osu") {
            continue;
        }

        let decoded = Beatmap::from_path(entry.path())
            .unwrap_or_else(|e| panic!("Failed to decode beatmap {filename:?}: {e:?}"));

        let mut bytes = Vec::with_capacity(512);

        decoded
            .encode(&mut bytes)
            .unwrap_or_else(|e| panic!("Failed to encode beatmap {filename:?}: {e:?}"));

        let decoded_after_encode = Beatmap::from_bytes(&bytes).unwrap_or_else(|e| {
            panic!("Failed to decode beatmap after encoding {filename:?}: {e:?}")
        });

        assert_eq!(decoded, decoded_after_encode, "{filename:?}");
    }
}

#[test]
fn bspline_curve_type() {
    let control_points = vec![
        PathControlPoint {
            pos: Pos::new(0.0, 0.0),
            path_type: Some(PathType::new_b_spline(NonZeroI32::new(3).unwrap())),
        },
        PathControlPoint {
            pos: Pos::new(50.0, 50.0),
            path_type: None,
        },
        PathControlPoint {
            pos: Pos::new(100.0, 100.0),
            path_type: Some(PathType::new_b_spline(NonZeroI32::new(3).unwrap())),
        },
        PathControlPoint {
            pos: Pos::new(150.0, 150.0),
            path_type: None,
        },
    ];

    let path = SliderPath::new(control_points, None);

    let slider = HitObjectSlider {
        pos: Pos::new(0.0, 0.0),
        new_combo: false,
        combo_offset: 0,
        path,
        node_samples: Vec::new(),
        repeat_count: 0,
        velocity: 0.0,
    };

    let hit_object = HitObject {
        start_time: 0.0,
        kind: HitObjectKind::Slider(slider),
        samples: Vec::new(),
    };

    let map = Beatmap {
        hit_objects: vec![hit_object],
        ..Default::default()
    };

    let mut bytes = Vec::with_capacity(512);

    map.encode(&mut bytes).unwrap();
    let decoded_after_encode = Beatmap::from_bytes(&bytes).unwrap();

    let HitObjectKind::Slider(ref expected) = map.hit_objects[0].kind else {
        unreachable!()
    };

    let HitObjectKind::Slider(ref actual) = decoded_after_encode.hit_objects[0].kind else {
        unreachable!()
    };

    assert_eq!(actual.path.control_points().len(), 4);
    assert_eq!(expected.path.control_points(), actual.path.control_points());
}

#[test]
fn multi_segment_slider_with_floating_point_error() {
    let control_points = vec![
        PathControlPoint {
            pos: Pos::new(0.0, 0.0),
            path_type: Some(PathType::BEZIER),
        },
        PathControlPoint {
            pos: Pos::new(0.5, 0.5),
            path_type: None,
        },
        PathControlPoint {
            pos: Pos::new(0.51, 0.51),
            path_type: None,
        },
        PathControlPoint {
            pos: Pos::new(1.0, 1.0),
            path_type: Some(PathType::BEZIER),
        },
        PathControlPoint {
            pos: Pos::new(2.0, 2.0),
            path_type: None,
        },
    ];

    let path = SliderPath::new(control_points, None);

    let slider = HitObjectSlider {
        pos: Pos::new(0.6, 0.6),
        new_combo: false,
        combo_offset: 0,
        path,
        node_samples: Vec::new(),
        repeat_count: 0,
        velocity: 0.0,
    };

    let hit_object = HitObject {
        start_time: 0.0,
        kind: HitObjectKind::Slider(slider),
        samples: Vec::new(),
    };

    let map = Beatmap {
        hit_objects: vec![hit_object],
        ..Default::default()
    };

    let mut bytes = Vec::with_capacity(512);

    map.encode(&mut bytes).unwrap();
    let decoded_after_encode = Beatmap::from_bytes(&bytes).unwrap();

    let HitObjectKind::Slider(ref expected) = map.hit_objects[0].kind else {
        unreachable!()
    };

    let HitObjectKind::Slider(ref actual) = decoded_after_encode.hit_objects[0].kind else {
        unreachable!()
    };

    assert_eq!(actual.path.control_points().len(), 5);
    assert_eq!(expected.path.control_points(), actual.path.control_points());
}
