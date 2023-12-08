use std::io::Error as IoError;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    InvalidInteger,
    Io { source: IoError },
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
