use std::{io::BufRead, ops::ControlFlow};

use crate::{
    reader::{DecoderError, Reader},
    util::{ParseNumber, ParseNumberError},
};

const VERSION_PREFIX: &str = "osu file format v";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FormatVersion(pub i32);

impl FormatVersion {
    pub const LATEST: i32 = 14;

    pub(crate) fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self, ParseVersionError> {
        let f = |line: &str| {
            if !line.starts_with(VERSION_PREFIX) {
                return if line.is_empty() {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(Err(ParseVersionError::UnknownFileFormat))
                };
            }

            let res = match line.rsplit('v').next().map(i32::parse) {
                Some(Ok(version)) => Ok(version),
                Some(Err(err)) => Err(err.into()),
                None => Ok(Self::LATEST),
            };

            ControlFlow::Break(res)
        };

        loop {
            match reader.next_line(f)? {
                Some(ControlFlow::Continue(_)) => {}
                Some(ControlFlow::Break(Ok(version))) => return Ok(Self(version)),
                Some(ControlFlow::Break(Err(err))) => return Err(err),
                None => return Err(ParseVersionError::UnknownFileFormat),
            }
        }
    }

    pub fn offset(self) -> i32 {
        const EARLY_VERSION_TIMING_OFFSET: i32 = 24;

        if self.0 < 5 {
            EARLY_VERSION_TIMING_OFFSET
        } else {
            0
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseVersionError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("unknown file format")]
    UnknownFileFormat,
}
