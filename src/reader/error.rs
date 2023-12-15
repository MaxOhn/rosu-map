use std::{char, io, str};

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    #[error("line did not match encoding")]
    IncorrectEncoding,
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("failed to decode line as UTF-8")]
    Utf8(#[from] str::Utf8Error),
    #[error("failed to decode line as UTF-16")]
    Utf16(#[from] char::DecodeUtf16Error),
}
