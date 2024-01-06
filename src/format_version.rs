use std::ops::ControlFlow;

use crate::util::{ParseNumber, ParseNumberError};

const VERSION_PREFIX: &str = "osu file format v";

/// The version format of an `.osu` file's content.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormatVersion(pub i32);

impl FormatVersion {
    /// The currently latest version.
    pub const LATEST: i32 = 14;

    pub(crate) fn try_from_line(line: &str) -> ControlFlow<Result<Self, ParseVersionError>, ()> {
        if !line.starts_with(VERSION_PREFIX) {
            return if line.is_empty() {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(Err(ParseVersionError::UnknownFileFormat))
            };
        }

        let res = match line.rsplit('v').next().map(i32::parse) {
            Some(Ok(version)) => Ok(Self(version)),
            Some(Err(err)) => Err(err.into()),
            None => Ok(Self::default()),
        };

        ControlFlow::Break(res)
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

/// All the ways that parsing a [`FormatVersion`] can fail.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseVersionError {
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
    #[error("unknown file format")]
    UnknownFileFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_version() {
        let line = "osu file format v42";
        assert!(matches!(
            FormatVersion::try_from_line(line),
            ControlFlow::Break(Ok(FormatVersion(42)))
        ));
    }

    #[test]
    fn fails_on_comment() {
        let line = "osu file format v42 // comment";
        assert!(matches!(
            FormatVersion::try_from_line(line),
            ControlFlow::Break(Err(ParseVersionError::Number(_)))
        ));
    }

    #[test]
    fn fails_on_wrong_prefix() {
        let line = "file format v42 // comment";
        assert!(matches!(
            FormatVersion::try_from_line(line),
            ControlFlow::Break(Err(ParseVersionError::UnknownFileFormat))
        ));
    }
}
