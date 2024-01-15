use std::{
    error::Error,
    fs::File,
    io,
    io::{BufRead, BufReader, Cursor},
    ops::ControlFlow,
    path::Path,
};

use crate::{format_version, reader::Reader, section::Section};

/// Parse a type that implements [`DecodeBeatmap`] by providing a path to a
/// `.osu` file.
///
/// # Example
///
/// ```rust,no_run
/// use rosu_map::section::hit_objects::HitObjects;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = "/path/to/file.osu";
/// let content: HitObjects = rosu_map::from_path(path)?;
/// # Ok(()) }
/// ```
pub fn from_path<D: DecodeBeatmap>(path: impl AsRef<Path>) -> Result<D, io::Error> {
    File::open(path).map(BufReader::new).and_then(D::decode)
}

/// Parse a type that implements [`DecodeBeatmap`] by providing the content of
/// a `.osu` file as a slice of bytes.
///
/// # Example
///
/// ```rust
/// use rosu_map::section::metadata::Metadata;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let bytes: &[u8] = b"[General]
/// Mode: 2
///
/// [Metadata]
/// Creator: pishifat";
///
/// let metadata: Metadata = rosu_map::from_bytes(bytes)?;
/// assert_eq!(metadata.creator, "pishifat");
/// # Ok(()) }
/// ```
pub fn from_bytes<D: DecodeBeatmap>(bytes: &[u8]) -> Result<D, io::Error> {
    D::decode(Cursor::new(bytes))
}

/// Parse a type that implements [`DecodeBeatmap`] by providing the content of
/// a `.osu` file as a string.
///
/// # Example
///
/// ```rust
/// use rosu_map::section::difficulty::Difficulty;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let s: &str = "[Difficulty]
/// SliderMultiplier: 3
///
/// [Editor]
/// BeatDivisor: 4";
///
/// let difficulty: Difficulty = rosu_map::from_str(s)?;
/// assert_eq!(difficulty.slider_multiplier, 3.0);
/// # Ok(()) }
/// ```
pub fn from_str<D: DecodeBeatmap>(s: &str) -> Result<D, io::Error> {
    D::decode(Cursor::new(s))
}

/// Intermediate state while parsing via [`DecodeBeatmap`].
pub trait DecodeState: Sized {
    /// Given the format version, create an instance.
    ///
    /// If the version is not of interest, this is basically
    /// `Default::default()`.
    fn create(version: i32) -> Self;
}

/// Trait to handle reading and parsing content of `.osu` files.
///
/// Generally, the only way to interact with this trait should be calling the
/// [`decode`] method.
///
/// Each section has its own `parse_[section]` method in which, given the next
/// line, the state should be updated. Note that the given lines will be
/// non-empty but comments (text starting with `//`) are **not trimmed**.
///
/// # Example
///
/// [`DecodeBeatmap`] is implemented for structs like [`HitObjects`] or
/// [`Beatmap`] so it can be used out the box.
///
/// ```
/// use std::io::Cursor;
/// use rosu_map::{Beatmap, DecodeBeatmap};
/// use rosu_map::section::general::GameMode;
/// use rosu_map::section::hit_objects::HitObjects;
///
/// let content: &str = "osu file format v14
///
/// [General]
/// Mode: 1 // Some comment
///
/// [Metadata]
/// Title: Some song title";
///
/// // Converting &str to &[u8] so that io::BufRead is satisfied
/// let mut reader = content.as_bytes();
/// let decoded = HitObjects::decode(&mut reader).unwrap();
/// assert_eq!(decoded.mode, GameMode::Taiko);
/// assert!(decoded.hit_objects.is_empty());
///
/// let mut reader = content.as_bytes();
/// let decoded = Beatmap::decode(&mut reader).unwrap();
/// assert_eq!(decoded.mode, GameMode::Taiko);
/// assert_eq!(decoded.title, "Some song title");
/// ```
///
/// Let's assume only the beatmap title and difficulty attributes are of
/// interest. Using [`Beatmap`] will parse **everything** which will be much
/// slower than implementing this trait on a custom type:
///
/// ```
/// use rosu_map::{DecodeBeatmap, DecodeState};
/// use rosu_map::section::difficulty::{Difficulty, DifficultyState, ParseDifficultyError};
/// use rosu_map::section::metadata::MetadataKey;
/// use rosu_map::util::KeyValue;
///
/// // Our final struct that we want to parse into.
/// struct CustomBeatmap {
///     title: String,
///     ar: f32,
///     cs: f32,
///     hp: f32,
///     od: f32,
/// }
///
/// // The struct that will be built gradually while parsing.
/// struct CustomBeatmapState {
///     title: String,
///     // Built-in way to handle difficulty parsing.
///     difficulty: DifficultyState,
/// }
///
/// // Required to implement for the `DecodeBeatmap` trait.
/// impl DecodeState for CustomBeatmapState {
///     fn create(version: i32) -> Self {
///         Self {
///             title: String::new(),
///             difficulty: DifficultyState::create(version),
///         }
///     }
/// }
///
/// // Also required for the `DecodeBeatmap` trait
/// impl From<CustomBeatmapState> for CustomBeatmap {
///     fn from(state: CustomBeatmapState) -> Self {
///         let difficulty = Difficulty::from(state.difficulty);
///
///         Self {
///             title: state.title,
///             ar: difficulty.approach_rate,
///             cs: difficulty.circle_size,
///             hp: difficulty.hp_drain_rate,
///             od: difficulty.overall_difficulty,
///         }
///     }
/// }
///
/// impl DecodeBeatmap for CustomBeatmap {
///     type State = CustomBeatmapState;
///
///     // In our case, only parsing the difficulty can fail so we can just use
///     // its error type.
///     type Error = ParseDifficultyError;
///
///     fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
///         // Note that comments are *not* trimmed at this point.
///         // To do that, one can use the `rosu_map::util::StrExt` trait and
///         // its `trim_comment` method.
///         let Ok(KeyValue { key, value }) = KeyValue::parse(line) else {
///             // Unknown key, discard line
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
///     fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error> {
///         // Let `Difficulty` and its state handle the difficulty parsing.
///         Difficulty::parse_difficulty(&mut state.difficulty, line)
///     }
///
///     // None of the other sections are of interest.
///     fn parse_general(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     fn parse_editor(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
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
///     fn parse_hit_objects(_state: &mut Self::State, _line: &str) -> Result<(), Self::Error> {
///         Ok(())
///     }
/// }
/// ```
///
/// For more examples, check out how structs like [`TimingPoints`] or
/// [`Beatmap`] implement the [`DecodeBeatmap`] trait.
///
/// [`decode`]: DecodeBeatmap::decode
/// [`Beatmap`]: crate::beatmap::Beatmap
/// [`HitObjects`]: crate::section::hit_objects::HitObjects
/// [`TimingPoints`]: crate::section::timing_points::TimingPoints
pub trait DecodeBeatmap: Sized {
    /// Error type in case something goes wrong while parsing.
    ///
    /// Note that this error is not thrown by the [`decode`] method. Instead,
    /// when a `parse_[section]` method returns such an error, it will be
    /// handled silently. That means, if the `tracing` feature is enabled, the
    /// error and its causes will be logged on the `ERROR` level. If `tracing`
    /// is not enabled, the error will be ignored entirely.
    ///
    /// [`decode`]: DecodeBeatmap::decode
    type Error: Error;

    /// The parsing state which will be updated on each line and turned into
    /// `Self` at the end.
    type State: DecodeState + Into<Self>;

    /// The key method to read and parse content of a `.osu` file into `Self`.
    ///
    /// This method should not be implemented manually.
    fn decode<R: BufRead>(src: R) -> Result<Self, io::Error> {
        let mut reader = Reader::new(src)?;

        let (version, use_curr_line) = parse_version(&mut reader)?;
        let mut state = Self::State::create(version);

        let Some(mut section) = parse_first_section(&mut reader, use_curr_line)? else {
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
                Section::Variables => {
                    parse_section(&mut reader, &mut state, Self::parse_variables)?
                }
                Section::CatchTheBeat => {
                    parse_section(&mut reader, &mut state, Self::parse_catch_the_beat)?
                }
                Section::Mania => parse_section(&mut reader, &mut state, Self::parse_mania)?,
            };

            match flow {
                SectionFlow::Continue(next) => section = next,
                SectionFlow::Break(()) => break,
            }
        }

        Ok(state.into())
    }

    /// Update the state based on a line of the `[General]` section.
    #[allow(unused_variables)]
    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Editor]` section.
    #[allow(unused_variables)]
    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Metadata]` section.
    #[allow(unused_variables)]
    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Difficulty]` section.
    #[allow(unused_variables)]
    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Events]` section.
    #[allow(unused_variables)]
    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[TimingPoints]` section.
    #[allow(unused_variables)]
    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Colours]` section.
    #[allow(unused_variables)]
    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[HitObjects]` section.
    #[allow(unused_variables)]
    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Variables]` section.
    #[allow(unused_variables)]
    fn parse_variables(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[CatchTheBeat]` section.
    #[allow(unused_variables)]
    fn parse_catch_the_beat(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;

    /// Update the state based on a line of the `[Mania]` section.
    #[allow(unused_variables)]
    fn parse_mania(state: &mut Self::State, line: &str) -> Result<(), Self::Error>;
}

struct UseCurrentLine(bool);

fn parse_version<R: BufRead>(reader: &mut Reader<R>) -> Result<(i32, UseCurrentLine), io::Error> {
    loop {
        let (version, use_curr_line) =
            match reader.next_line(format_version::try_version_from_line)? {
                Some(ControlFlow::Continue(())) => continue,
                Some(ControlFlow::Break(Ok(version))) => (version, false),
                // Only used when `tracing` feature is enabled
                #[allow(unused)]
                Some(ControlFlow::Break(Err(err))) => {
                    #[cfg(feature = "tracing")]
                    {
                        tracing::error!("Failed to parse format version: {err}");
                        log_error_cause(&err);
                    }

                    (format_version::LATEST_FORMAT_VERSION, true)
                }
                None => (format_version::LATEST_FORMAT_VERSION, false),
            };

        return Ok((version, UseCurrentLine(use_curr_line)));
    }
}

fn parse_first_section<R: BufRead>(
    reader: &mut Reader<R>,
    UseCurrentLine(use_curr_line): UseCurrentLine,
) -> Result<Option<Section>, io::Error> {
    if use_curr_line {
        if let opt @ Some(_) = Section::try_from_line(reader.curr_line()) {
            return Ok(opt);
        }
    }

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
) -> Result<SectionFlow, io::Error>
where
    E: Error,
{
    let mut f = |line: &str| {
        if let Some(next) = Section::try_from_line(line) {
            return ControlFlow::Break(SectionFlow::Continue(next));
        }

        // Only used when `tracing` feature is enabled
        #[allow(unused)]
        let res = f(state, line);

        #[cfg(feature = "tracing")]
        if let Err(err) = res {
            tracing::error!("Failed to process line {line:?}: {err}");
            log_error_cause(&err);
        }

        ControlFlow::Continue(())
    };

    loop {
        match reader.next_line(&mut f) {
            Ok(Some(ControlFlow::Continue(()))) => {}
            Ok(Some(ControlFlow::Break(flow))) => return Ok(flow),
            Ok(None) => return Ok(SectionFlow::Break(())),
            Err(err) => return Err(err),
        }
    }
}

#[cfg(feature = "tracing")]
fn log_error_cause(mut err: &dyn Error) {
    while let Some(src) = err.source() {
        tracing::error!("  - caused by: {src}");
        err = src;
    }
}
