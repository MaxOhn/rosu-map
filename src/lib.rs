//! Library to de- and encode `.osu` files from [osu!].
//!
//! ## What
//!
//! At it's core, `rosu-map` provides the [`DecodeBeatmap`] trait. The trait is responsible for
//! decoding the file itself, error handling, and section parsing. All that's left to do for
//! implementators of the trait is to keep a state of parsed data and, given a section and a line
//! of text, to modify that state.
//!
//! `rosu-map` also provides multiple types that already implement this trait, namely one for each
//! section (see [`Editor`], [`TimingPoints`], ...) and one for the (almost) full content, [`Beatmap`].
//!
//! ## Why
//!
//! Exposing functionality through the trait allows for flexibility when deciding which content to
//! parse and thus make it more efficient when not all data is needed.
//!
//! If only the difficulty attributes are required, parsing via the [`Difficulty`] struct will discard
//! everything except for the few lines within the `[Difficulty]` section of the `.osu` file.
//! Similarly, if only the artist, title, and version is of interest, the [`Metadata`] struct can be
//! used.
//!
//! Additionally, it's worth noting that [`Beatmap`] parses (almost) *everything* which might be
//! overkill for many use-cases. The work-around would be to define a new custom type, copy-paste
//! [`Beatmap`]'s [`DecodeBeatmap`] implementation, and then throw out everything that's not needed.
//!
//! ## How
//!
//! The simplest way to make use of a type's [`DecodeBeatmap`] implementation is by using
//! `rosu-map`s functions [`from_bytes`], [`from_path`], and [`from_str`].
//!
//! ```
//! # use rosu_map::Beatmap;
//! use rosu_map::section::difficulty::Difficulty;
//!
//! let content = "[Difficulty]
//! ApproachRate: 9.2
//! SliderMultiplier: 1.9
//!
//! [Metadata]
//! Creator: peppy";
//!
//! let difficulty = rosu_map::from_str::<Difficulty>(content).unwrap();
//! assert_eq!(difficulty.approach_rate, 9.2);
//!
//! let path = "./resources/Soleily - Renatus (Gamu) [Insane].osu";
//! let map = rosu_map::from_path::<Beatmap>(path).unwrap();
//! assert_eq!(map.audio_file, "03. Renatus - Soleily 192kbps.mp3");
//! ```
//!
//! For information on implementing the [`DecodeBeatmap`] trait on a new type, check out the
//! trait's documentation. For examples, check how types like [`General`] or [`HitObjects`]
//! implement the trait.
//!
//! ## Encoding
//!
//! The [`Beatmap`] struct provides a built-in way to turn itself into the content of a `.osu` file
//! through its `encode*` methods.
//!
//! ```no_run
//! # use rosu_map::Beatmap;
//! let path = "./resources/Within Temptation - The Unforgiving (Armin) [Marathon].osu";
//! let mut map: Beatmap = rosu_map::from_path(path).unwrap();
//!
//! map.approach_rate = 10.0;
//!
//! map.encode_to_path("./new_file.osu").unwrap();
//!
//! let metadata = rosu_map::section::metadata::Metadata {
//!     title: "song title".to_string(),
//!     artist: "artist name".to_string(),
//!     ..Default::default()
//! };
//!
//! let content = Beatmap::from(metadata).encode_to_string().unwrap();
//! assert!(content.contains("Title: song title"));
//! ```
//!
//! ## Features
//!
//! | Flag | Description | Dependencies
//! | - | - | -
//! | `default` | No features |
//! | `tracing` | Any error encountered during decoding will be logged through `tracing::error`. If this features is not enabled, errors will be ignored. | [`tracing`]
//!
//! ## Misc
//!
//! #### Internals
//!
//! A sizable section of `rosu-map` is a port of [osu!lazer]'s beatmap
//! {de/en}coding. Not only does its functionality mirror osu!, but many test cases were
//! translated too, providing a solid degree of correctness even on fringe edge cases.
//!
//! Lazer commit on last port: `f08134f443b2cf255fd19c8bc3ef517b6a3bb8e3`
//!
//! #### Async
//!
//! After some testing and benchmarking, it turns out that async IO does not provide any improvements
//! or performance gains even in a concurrent context. In fact, regular sequential IO consistently
//! outperformed its async counterpart. As such `rosu-map` does not provide an async interface.
//!
//! #### Storyboard
//!
//! `rosu-map` does not provide types that parse storyboards, but the crate [`rosu-storyboard`] does.
//!
//! [osu!]: https://osu.ppy.sh/
//! [osu!lazer]: https://github.com/ppy/osu
//! [`DecodeBeatmap`]: crate::decode::DecodeBeatmap
//! [`Beatmap`]: crate::beatmap::Beatmap
//! [`from_bytes`]: crate::decode::from_bytes
//! [`from_str`]: crate::decode::from_str
//! [`from_path`]: crate::decode::from_path
//! [`General`]: crate::section::general::decode::General
//! [`Editor`]: crate::section::editor::Editor
//! [`Metadata`]: crate::section::metadata::Metadata
//! [`Difficulty`]: crate::section::difficulty::Difficulty
//! [`TimingPoints`]: crate::section::timing_points::decode::TimingPoints
//! [`HitObjects`]: crate::section::hit_objects::decode::HitObjects
//! [`tracing`]: https://docs.rs/tracing
//! [`rosu-storyboard`]: https://github.com/MaxOhn/rosu-storyboard/

#![deny(rustdoc::broken_intra_doc_links, rustdoc::missing_crate_level_docs)]
#![warn(clippy::missing_const_for_fn, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools,
    clippy::match_same_arms,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::explicit_iter_loop
)]

#[macro_use]
mod macros;

mod beatmap;
mod decode;
mod encode;
mod format_version;
mod reader;

/// Section-specific types.
pub mod section;

/// Various utility types for usage in and around this library.
pub mod util;

pub use crate::{
    beatmap::{Beatmap, BeatmapState, ParseBeatmapError},
    decode::{from_bytes, from_path, from_str, DecodeBeatmap, DecodeState},
    format_version::LATEST_FORMAT_VERSION,
};
