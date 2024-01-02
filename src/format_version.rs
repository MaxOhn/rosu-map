use std::{io::BufRead, ops::ControlFlow};

use crate::{
    reader::{DecoderError, Reader},
    util::{ParseNumber, ParseNumberError},
};

const VERSION_PREFIX: &str = "osu file format v";

/// The version format of an `.osu` file's content.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormatVersion(pub i32);

impl FormatVersion {
    /// The currently latest version.
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
                Some(ControlFlow::Continue(())) => {}
                Some(ControlFlow::Break(Ok(version))) => return Ok(Self(version)),
                Some(ControlFlow::Break(Err(err))) => return Err(err),
                None => return Err(ParseVersionError::UnknownFileFormat),
            }
        }
    }

    pub(crate) const fn offset(self) -> i32 {
        const EARLY_VERSION_TIMING_OFFSET: i32 = 24;

        if self.0 < 5 {
            EARLY_VERSION_TIMING_OFFSET
        } else {
            0
        }
    }
}

impl Default for FormatVersion {
    fn default() -> Self {
        Self(Self::LATEST)
    }
}

impl PartialEq<i32> for FormatVersion {
    fn eq(&self, other: &i32) -> bool {
        self.0.eq(other)
    }
}

/// All the ways that parsing a `.osu` file into [`FormatVersion`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseVersionError {
    #[error("decoder error")]
    Decoder(#[from] DecoderError),
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("unknown file format")]
    UnknownFileFormat,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn finds_version() {
        let mut reader = Reader::new(Cursor::new("osu file format v42")).unwrap();
        assert_eq!(
            FormatVersion::parse(&mut reader).unwrap(),
            FormatVersion(42)
        );
    }

    #[test]
    fn fails_on_comment() {
        let mut reader = Reader::new(Cursor::new("osu file format v42 // comment")).unwrap();
        assert!(matches!(
            FormatVersion::parse(&mut reader),
            Err(ParseVersionError::Number(_))
        ));
    }

    #[test]
    fn skips_whitespace() {
        let mut reader = Reader::new(Cursor::new(
            "
  
       
            osu file format v42",
        ))
        .unwrap();
        assert!(matches!(
            FormatVersion::parse(&mut reader),
            Ok(FormatVersion(42))
        ));
    }

    #[test]
    fn fails_on_wrong_prefix() {
        let mut reader = Reader::new(Cursor::new("file format v42 // comment")).unwrap();
        assert!(matches!(
            FormatVersion::parse(&mut reader),
            Err(ParseVersionError::UnknownFileFormat)
        ));

        let mut reader = Reader::new(Cursor::new(
            "

  a
        
        osu file format v42",
        ))
        .unwrap();
        assert!(matches!(
            FormatVersion::parse(&mut reader),
            Err(ParseVersionError::UnknownFileFormat)
        ));
    }
}
