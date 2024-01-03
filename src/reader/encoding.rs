use std::str::from_utf8 as str_from_utf8;

use super::{
    error::DecoderError,
    u16_iter::{U16BeIterator, U16LeIterator},
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Encoding {
    #[default]
    Utf8,
    Utf16BE,
    Utf16LE,
}

impl Encoding {
    pub const fn from_bom(bom: &[u8]) -> (Self, usize) {
        match bom {
            [0xEF, 0xBB, 0xBF, ..] => (Self::Utf8, 3),
            [0xFF, 0xFE, ..] => (Self::Utf16LE, 2),
            [0xFE, 0xFF, ..] => (Self::Utf16BE, 2),
            _ => (Self::Utf8, 0),
        }
    }

    /// Decodes the given `src` and returns it as a `&str`.
    ///
    /// In case of UTF-16, the result will be stored in `dst`.
    pub fn decode<'a>(self, src: &'a [u8], dst: &'a mut String) -> Result<&'a str, DecoderError> {
        match self {
            Self::Utf8 => Ok(str_from_utf8(src)?),
            Self::Utf16LE => Self::decode_utf16(U16LeIterator::new(src)?, dst),
            Self::Utf16BE => Self::decode_utf16(U16BeIterator::new(src)?, dst),
        }
    }

    fn decode_utf16<S>(src: S, dst: &mut String) -> Result<&str, DecoderError>
    where
        S: Iterator<Item = u16>,
    {
        dst.clear();

        for res in char::decode_utf16(src) {
            dst.push(res?);
        }

        Ok(dst)
    }
}
