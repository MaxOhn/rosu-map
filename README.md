[![crates.io](https://img.shields.io/crates/v/rosu-map.svg)](https://crates.io/crates/rosu-map) [![docs](https://docs.rs/rosu-map/badge.svg)](https://docs.rs/rosu-map)

# rosu-map

<!-- cargo-rdme start -->

Library to de- and encode `.osu` files from [osu!].

### What

At it's core, `rosu-map` provides the [`DecodeBeatmap`] trait. The trait is responsible for
decoding the file itself, error handling, and section parsing. All that's left to do for
implementators of the trait is to keep a state of parsed data and, given a section and a line
of text, to modify that state.

`rosu-map` also provides multiple types that already implement this trait, namely one for each
section (see [`Editor`], [`TimingPoints`], ...) and one for the (almost) full content, [`Beatmap`].

### Why

Exposing functionality through the trait allows for flexibility when deciding which content to
parse and thus make it more efficient when not all data is needed.

If only the difficulty attributes are required, parsing via the [`Difficulty`] struct will discard
everything except for the few lines within the `[Difficulty]` section of the `.osu` file.
Similarly, if only the artist, title, and version is of interest, the [`Metadata`] struct can be
used.

Additionally, it's worth noting that [`Beatmap`] parses (almost) *everything* which might be
overkill for many use-cases. The work-around would be to define a new custom type, copy-paste
[`Beatmap`]'s [`DecodeBeatmap`] implementation, and then throw out everything that's not needed.

### How

The simplest way to make use of a type's [`DecodeBeatmap`] implementation is by using
`rosu-map`s functions [`from_bytes`], [`from_path`], and [`from_str`].

```rust
use rosu_map::section::difficulty::Difficulty;

let content = "[Difficulty]
ApproachRate: 9.2
SliderMultiplier: 1.9

[Metadata]
Creator: peppy";

let difficulty = rosu_map::from_str::<Difficulty>(content).unwrap();
assert_eq!(difficulty.approach_rate, 9.2);

let path = "./resources/Soleily - Renatus (Gamu) [Insane].osu";
let map = rosu_map::from_path::<Beatmap>(path).unwrap();
assert_eq!(map.audio_file, "03. Renatus - Soleily 192kbps.mp3");
```

For information on implementing the [`DecodeBeatmap`] trait on a new type, check out the
trait's documentation. For examples, check how types like [`General`] or [`HitObjects`]
implement the trait.

### Encoding

The [`Beatmap`] struct provides a built-in way to turn itself into the content of a `.osu` file
through its `encode*` methods.

```rust
let path = "./resources/Within Temptation - The Unforgiving (Armin) [Marathon].osu";
let mut map: Beatmap = rosu_map::from_path(path).unwrap();

map.approach_rate = 10.0;

map.encode_to_path("./new_file.osu").unwrap();

let metadata = rosu_map::section::metadata::Metadata {
    title: "song title".to_string(),
    artist: "artist name".to_string(),
    ..Default::default()
};

let content = Beatmap::from(metadata).encode_to_string().unwrap();
assert!(content.contains("Title: song title"));
```

### Features

| Flag | Description | Dependencies
| - | - | -
| `default` | No features |
| `tracing` | Any error encountered during decoding will be logged through `tracing::error`. If this features is not enabled, errors will be ignored. | [`tracing`]

### Misc

##### Internals

A sizable section of `rosu-map` is a port of [osu!lazer]'s beatmap
{de/en}coding. Not only does its functionality mirror osu!, but many test cases were
translated too, providing a solid degree of correctness even on fringe edge cases.

Lazer commit on last port: `8bd65d9938a10fc42e6409501b0282f0fa4a25ef`

##### Async

After some testing and benchmarking, it turns out that async IO does not provide any improvements
or performance gains even in a concurrent context. In fact, regular sequential IO consistently
outperformed its async counterpart. As such `rosu-map` does not provide an async interface.

##### Storyboard

`rosu-map` does not provide types that parse storyboards, but the crate [`rosu-storyboard`] does.

[osu!]: https://osu.ppy.sh/
[osu!lazer]: https://github.com/ppy/osu
[`DecodeBeatmap`]: https://docs.rs/rosu-map/latest/rosu_map/decode/trait.DecodeBeatmap.html
[`Beatmap`]: https://docs.rs/rosu-map/latest/rosu_map/beatmap/struct.Beatmap.html
[`from_bytes`]: https://docs.rs/rosu-map/latest/rosu_map/decode/fn.from_bytes.html
[`from_str`]: https://docs.rs/rosu-map/latest/rosu_map/decode/fn.from_str.html
[`from_path`]: https://docs.rs/rosu-map/latest/rosu_map/decode/fn.from_path.html
[`General`]: https://docs.rs/rosu-map/latest/rosu_map/section/general/decode/struct.General.html
[`Editor`]: https://docs.rs/rosu-map/latest/rosu_map/section/editor/struct.Editor.html
[`Metadata`]: https://docs.rs/rosu-map/latest/rosu_map/section/metadata/struct.Metadata.html
[`Difficulty`]: https://docs.rs/rosu-map/latest/rosu_map/section/difficulty/struct.Difficulty.html
[`TimingPoints`]: https://docs.rs/rosu-map/latest/rosu_map/section/timing_points/decode/struct.TimingPoints.html
[`HitObjects`]: https://docs.rs/rosu-map/latest/rosu_map/section/hit_objects/decode/struct.HitObjects.html
[`tracing`]: https://docs.rs/tracing
[`rosu-storyboard`]: https://github.com/MaxOhn/rosu-storyboard/

<!-- cargo-rdme end -->
