<!-- cargo-rdme start -->

# rosu-map

Library to de- and encode `.osu` files from [osu!](https://osu.ppy.sh/).

## What

At it's core, `rosu-map` provides the [`DecodeBeatmap`] trait. The trait is responsible for 
decoding the file itself, error handling, and section parsing. All that's left to do for
implementators of the trait is to keep a state of parsed data and, given a section and a line of
text, to modify that state.

`rosu-map` also provides multiple types that already implement this trait, namely one for each
section (see [`Editor`], [`TimingPoints`], ...) and one for the (almost) full content, [`Beatmap`].

## Why

Exposing functionality through the trait allows for flexibility when deciding which content to
parse and thus make it more efficient when not all data is needed.

If only the difficulty attributes are required, parsing via the [`Difficulty`] struct will discard
everything except for the few lines within the `[Difficulty]` section of the `.osu` file.
Similarly, if only the artist, title, and version is of interest, the [`Metadata`] struct can be
used.

Additionally, it's worth noting that [`Beatmap`] parses (almost) *everything* which might be
overkill for many use-cases. The work-around would be to define a new custom type, copy-paste
[`Beatmap`]'s [`DecodeBeatmap`] implementation, and then throw out everything that's not needed.

## How

The simplest way to make use of a type's [`DecodeBeatmap`] implementation is by using `rosu-map`s
functions [`from_bytes`], [`from_path`], and [`from_str`].

```rs
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
assert_eq!(map.audio_filename, "03. Renatus - Soleily 192kbps.mp3");
```

For information on implementing the [`DecodeBeatmap`] trait on a new type, check out the trait's
documentation. For examples, one can check how types like [`General`] or [`HitObjects`] implement
the trait.

## Encoding

The [`Beatmap`] struct provides a built-in way to turn itself into the content of a `.osu` file
through its `encode*` methods.

```rs
let path = "./resources/Within Temptation - The Unforgiving (Armin) [Marathon].osu";
let mut map = rosu_map::from_path(path).unwrap();

map.approach_rate = 10.0;

map.encode_to_path("./new_file.osu").unwrap();

let metadata = rosu_map::section::metadata::Metadata {
    title: "song title",
    artist: "artist name",
    ..Default::default()
};

let content = Beatmap::from(metadata).encode_to_string().unwrap();
assert!(content.contains("Title: song title"));
```

## Features

| Flag | Description | Dependencies
| - | - | -
| `default` | No features |
| `tracing` | Any error encountered during decoding will be logged through `tracing::error`. If this features is not enabled, errors will be ignored. | [`tracing`](https://docs.rs/tracing)

## Misc

### Internals

A sizable section of `rosu-map` is a port of [osu!lazer](https://github.com/ppy/osu)'s beatmap
{de/en}coding. With that, not only the functionality mirrors osu!, but also many test cases were
translated, providing a certain degree of correctness even on fringe edge cases.

### Async

After some testing and benchmarking, it turns out that async IO does not provide any improvements
or performance gains even in a concurrent context. In fact, regular sequential IO consistently
outperformed its async counterpart. As such `rosu-map` does not provide an async interface.

<!-- cargo-rdme end -->