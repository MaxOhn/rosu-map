use std::{error::Error, io::BufRead, ops::ControlFlow};

use crate::{
    model::format_version::{FormatVersion, ParseVersionError},
    reader::Reader,
    section::Section,
};

pub use crate::reader::DecoderError;

/// Intermediate state while parsing via [`DecodeBeatmap`].
pub trait DecodeState: Sized {
    /// Given the [`FormatVersion`], create an instance.
    ///
    /// If the version is not of interest, this is basically
    /// `Default::default()`.
    fn create(version: FormatVersion) -> Self;
}

/// Trait to handle reading and parsing of `.osu` files.
///
/// Generally, the only way to use this trait is to call the [`decode`] method.
///
/// Each section has its own `parse_[section]` method in which, given the next
/// line, the state can be updated.
///
/// # Example
///
/// [`DecodeBeatmap`] is implemented for structs like [`HitObjects`] or
/// [`Beatmap`] so it can be used out the box.
///
/// ```
/// use std::io::Cursor;
/// use rosu_map::{Beatmap, DecodeBeatmap};
/// use rosu_map::model::mode::GameMode;
/// use rosu_map::section::hit_objects::HitObjects;
///
/// let content = "osu file format v14
///
/// [General]
/// Mode: 1    // Some comment
///
/// [Metadata]
/// Title: Some song title";
///
/// let mut reader = Cursor::new(content);
/// let decoded = HitObjects::decode(&mut reader).unwrap();
/// assert!(decoded.hit_objects.is_empty());
///
/// let mut reader = Cursor::new(content);
/// let decoded = Beatmap::decode(&mut reader).unwrap();
/// assert_eq!(decoded.mode, GameMode::Taiko);
/// ```
///
/// Let's assume only the beatmap title and hitobjects are of interest. Using
/// [`Beatmap`] will parse **everything** which will be slower than
/// implementing this trait on a custom type:
///
/// ```
/// use rosu_map::{DecodeBeatmap, DecodeState};
/// use rosu_map::model::{format_version::FormatVersion, hit_objects::HitObject};
/// use rosu_map::section::hit_objects::{HitObjects, HitObjectsState, ParseHitObjectsError};
/// use rosu_map::section::metadata::MetadataKey;
/// use rosu_map::util::KeyValue;
///
/// // Our final struct that we want to parse into.
/// struct CustomBeatmap {
///     title: String,
///     hit_objects: Vec<HitObject>,
/// }
///
/// // The struct that will be built gradually while parsing.
/// struct CustomBeatmapState {
///     title: String,
///     // Built-in way to handle hitobject parsing.
///     hit_objects: HitObjectsState,
/// }
///
/// // Required to implement for the `DecodeBeatmap` trait.
/// impl DecodeState for CustomBeatmapState {
///     fn create(version: FormatVersion) -> Self {
///         Self {
///             title: String::new(),
///             hit_objects: HitObjectsState::create(version),
///         }
///     }
/// }
///
/// // Also required for the `DecodeBeatmap` trait
/// impl From<CustomBeatmapState> for CustomBeatmap {
///     fn from(state: CustomBeatmapState) -> Self {
///         Self {
///             title: state.title,
///             hit_objects: HitObjects::from(state.hit_objects).hit_objects,
///         }
///     }
/// }
///
/// impl DecodeBeatmap for CustomBeatmap {
///     type State = CustomBeatmapState;
///
///     // In our case, only parsing the hitobjects can fail so we can just use
///     // their error type.
///     type Error = ParseHitObjectsError;
///
///     fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
///         let Ok(KeyValue { key, value }) = KeyValue::parse(line) else {
///             return Ok(());
///         };
///
///         match key {
///             MetadataKey::Title => state.title = value.to_owned(),
///             _ => {}
///         }
///
///         Ok(())
///     }
///
///     fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
///         HitObjects::parse_hit_objects(&mut state.hit_objects, line)
///     }
///
///     // Technically, it's recommended to call `HitObjects::parse_[section]`
///     // for each of these in case the hitobjects rely on data from another
///     // section. However, looking at `HitObjects`' implementation of
///     // `DecodeBeatmap`, one can see that only `parse_hit_objects` is
///     // used so we don't need to use any other method either.
///     fn parse_general(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     fn parse_editor(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     fn parse_difficulty(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     fn parse_events(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     fn parse_timing_points(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     fn parse_colors(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
/// }
/// ```
///
/// For more examples, check out how structs like [`TimingPoints`] or
/// [`Beatmap`] implement the [`DecodeBeatmap`] trait.
///
/// [`decode`]: DecodeBeatmap::decode
/// [`Beatmap`]: crate::model::beatmap::Beatmap
/// [`HitObjects`]: crate::section::hit_objects::HitObjects
/// [`TimingPoints`]: crate::section::timing_points::TimingPoints
pub trait DecodeBeatmap: Sized {
    /// Returned error type in case something goes wrong while reading or
    /// parsing.
    type Error: Error + From<DecoderError> + From<ParseVersionError>;

    /// The parsing state which will be updated on each line and turned into
    /// `Self` at the end.
    type State: DecodeState + Into<Self>;

    /// The key method to read and parse content of a `.osu` file into `Self`.
    ///
    /// This method should not be implemented manually.
    fn decode<R: BufRead>(src: R) -> Result<Self, Self::Error> {
        let mut reader = Reader::new(src)?;

        let version = FormatVersion::parse(&mut reader)?;
        let mut state = Self::State::create(version);

        let Some(mut section) = parse_first_section(&mut reader)? else {
            return Ok(state.into());
        };

        loop {
            let flow = match section {
                Section::General => parse_section(&mut reader, &mut state, Self::parse_general)?,
                Section::Editor => parse_section(&mut reader, &mut state, Self::parse_editor)?,
                Section::Metadata => parse_section(&mut reader, &mut state, Self::parse_metadata)?,
                Section::Difficulty => {
                    parse_section(&mut reader, &mut state, Self::parse_difficulty)?
                }
                Section::Events => parse_section(&mut reader, &mut state, Self::parse_events)?,
                Section::TimingPoints => {
                    parse_section(&mut reader, &mut state, Self::parse_timing_points)?
                }
                Section::Colors => parse_section(&mut reader, &mut state, Self::parse_colors)?,
                Section::HitObjects => {
                    parse_section(&mut reader, &mut state, Self::parse_hit_objects)?
                }
            };

            match flow {
                SectionFlow::Continue(next) => section = next,
                SectionFlow::Break(()) => break,
            }
        }

        Ok(state.into())
    }

    /// Update the state based on a line of the `[General]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Editor]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Metadata]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Difficulty]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Events]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[TimingPoints]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Colours]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[HitObjects]` section.
    ///
    /// The line will be non-empty but comments (text starting with `//`) are
    /// **not** trimmed.
    #[allow(unused_variables)]
    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;
}

fn parse_first_section<R: BufRead>(
    reader: &mut Reader<R>,
) -> Result<Option<Section>, DecoderError> {
    loop {
        match reader.next_line(Section::try_from_line) {
            Ok(Some(Some(section))) => return Ok(Some(section)),
            Ok(Some(None)) => {}
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        }
    }
}

type SectionFlow = ControlFlow<(), Section>;

fn parse_section<R: BufRead, S, E>(
    reader: &mut Reader<R>,
    state: &mut S,
    f: fn(&mut S, &str) -> Result<(), E>,
) -> Result<SectionFlow, E>
where
    E: From<DecoderError>,
{
    let mut f = |line: &str| {
        if let Some(next) = Section::try_from_line(line) {
            return Ok(ControlFlow::Break(SectionFlow::Continue(next)));
        }

        f(state, line).map(ControlFlow::Continue)
    };

    loop {
        match reader.next_line(&mut f) {
            Ok(Some(Ok(ControlFlow::Continue(())))) => {}
            Ok(Some(Ok(ControlFlow::Break(flow)))) => return Ok(flow),
            Ok(Some(Err(err))) => return Err(err),
            Ok(None) => return Ok(SectionFlow::Break(())),
            Err(err) => return Err(err.into()),
        }
    }
}
