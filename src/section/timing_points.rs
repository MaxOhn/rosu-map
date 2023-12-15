use crate::{
    format_version::{FormatVersion, ParseVersionError},
    parse::{ParseBeatmap, ParseState},
    reader::DecoderError,
};

#[derive(Default)]
pub struct TimingPoints {}

#[derive(Debug, thiserror::Error)]
pub enum ParseTimingPointsError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse format version")]
    FormatVersion(#[from] ParseVersionError),
}

pub struct TimingPointsState {
    timing_points: TimingPoints,
}

impl ParseState for TimingPointsState {
    fn create(version: FormatVersion) -> Self {
        Self {
            timing_points: TimingPoints::default(),
        }
    }
}

impl From<TimingPointsState> for TimingPoints {
    fn from(state: TimingPointsState) -> Self {
        state.timing_points
    }
}

impl ParseBeatmap for TimingPoints {
    type ParseError = ParseTimingPointsError;
    type State = TimingPointsState;

    fn parse_timing_points(state: &mut Self::State, line: &str) -> Result<(), Self::ParseError> {
        todo!()
    }
}
