//! TODO: docs

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

mod decode;
mod encode;
mod reader;

/// All parsed types contained in a `.osu` file.
pub mod model;

/// Decode-related types for all sections.
pub mod section;

/// Various utility types for usage in and around this library.
pub mod util;

pub use crate::{
    decode::{DecodeBeatmap, DecodeState, DecoderError},
    model::beatmap::Beatmap,
};
