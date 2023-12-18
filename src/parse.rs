use std::{error::Error, io::BufRead, ops::ControlFlow};

use crate::{
    format_version::{FormatVersion, ParseVersionError},
    reader::Reader,
    section::Section,
};

pub use crate::reader::DecoderError;

pub trait ParseState: Sized {
    fn create(version: FormatVersion) -> Self;
}

pub trait ParseBeatmap: Sized {
    type ParseError: Error + From<DecoderError> + From<ParseVersionError>;
    type State: ParseState + Into<Self>;

    fn parse<R: BufRead>(src: R) -> Result<Self, Self::ParseError> {
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

    #[allow(unused_variables)]
    fn parse_general(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;

    #[allow(unused_variables)]
    fn parse_editor(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;

    #[allow(unused_variables)]
    fn parse_metadata(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;

    #[allow(unused_variables)]
    fn parse_difficulty(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;

    #[allow(unused_variables)]
    fn parse_events(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;

    #[allow(unused_variables)]
    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;

    #[allow(unused_variables)]
    fn parse_colors(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;

    #[allow(unused_variables)]
    fn parse_hit_objects(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError>;
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
