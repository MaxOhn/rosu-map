use std::{
    io::Error as IoError,
    str::Utf8Error,
    string::{FromUtf16Error, FromUtf8Error},
};

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    InvalidInteger,
    Io { source: IoError },
    UnknownFileFormat,
    Utf8 { source: Utf8Error },
    Utf16 { source: FromUtf16Error },
}

// TODO: remove
impl From<()> for ParseError {
    fn from(_: ()) -> Self {
        unimplemented!()
    }
}

impl From<IoError> for ParseError {
    fn from(source: IoError) -> Self {
        Self::Io { source }
    }
}

impl From<Utf8Error> for ParseError {
    fn from(source: Utf8Error) -> Self {
        Self::Utf8 { source }
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(source: FromUtf8Error) -> Self {
        Self::Utf8 {
            source: source.utf8_error(),
        }
    }
}

impl From<FromUtf16Error> for ParseError {
    fn from(source: FromUtf16Error) -> Self {
        Self::Utf16 { source }
    }
}
