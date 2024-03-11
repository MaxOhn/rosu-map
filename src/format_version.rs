use std::ops::ControlFlow;

use crate::util::{ParseNumber, ParseNumberError};

const VERSION_PREFIX: &str = "osu file format v";

/// The currently latest format version.
pub const LATEST_FORMAT_VERSION: i32 = 14;

pub(crate) fn try_version_from_line(line: &str) -> ControlFlow<Result<i32, ParseVersionError>, ()> {
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
        None => Ok(LATEST_FORMAT_VERSION),
    };

    ControlFlow::Break(res)
}

thiserror! {
    /// All the ways that parsing the format version can fail.
    #[derive(Debug)]
    pub(crate) enum ParseVersionError {
        #[error("failed to parse number")]
        Number(#[from] ParseNumberError),
        #[error("unknown file format")]
        UnknownFileFormat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_version() {
        let line = "osu file format v42";
        assert!(matches!(
            try_version_from_line(line),
            ControlFlow::Break(Ok(42))
        ));
    }

    #[test]
    fn fails_on_comment() {
        let line = "osu file format v42 // comment";
        assert!(matches!(
            try_version_from_line(line),
            ControlFlow::Break(Err(ParseVersionError::Number(_)))
        ));
    }

    #[test]
    fn fails_on_wrong_prefix() {
        let line = "file format v42 // comment";
        assert!(matches!(
            try_version_from_line(line),
            ControlFlow::Break(Err(ParseVersionError::UnknownFileFormat))
        ));
    }
}
