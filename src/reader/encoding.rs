use std::str::{from_utf8 as str_from_utf8, from_utf8_unchecked as str_from_utf8_unchecked};

use super::u16_iter::{U16BeIterator, U16LeIterator};

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
    /// In case of UTF-16 or invalid UTF-8, the result will be stored in `dst`.
    pub fn decode<'a>(self, mut src: &'a [u8], dst: &'a mut String) -> &'a str {
        match self {
            Self::Utf8 => match str_from_utf8(src) {
                Ok(s) => s,
                // Replace invalid UTF-8 characters with U+FFFD.
                // XXX: Use `std::str::Utf8Chunks` when stabilized.
                //      See <https://github.com/rust-lang/rust/issues/99543>
                Err(mut err) => {
                    dst.clear();

                    loop {
                        let valid_up_to = err.valid_up_to();
                        // SAFETY: The previous UTF-8 conversion succeeded up
                        // until `valid_up_to`.
                        let valid = unsafe { str_from_utf8_unchecked(&src[..valid_up_to]) };
                        dst.push_str(valid);
                        dst.push(char::REPLACEMENT_CHARACTER);

                        if let Some(error_len) = err.error_len() {
                            src = &src[valid_up_to + error_len..];
                        } else {
                            return dst;
                        }

                        match str_from_utf8(src) {
                            Ok(s) => {
                                dst.push_str(s);

                                return dst;
                            }
                            Err(e) => err = e,
                        }
                    }
                }
            },
            Self::Utf16LE => Self::decode_utf16(U16LeIterator::new(src), dst),
            Self::Utf16BE => Self::decode_utf16(U16BeIterator::new(src), dst),
        }
    }

    fn decode_utf16<S: Iterator<Item = u16>>(src: S, dst: &mut String) -> &str {
        dst.clear();

        let chars = char::decode_utf16(src).map(|ch| ch.unwrap_or(char::REPLACEMENT_CHARACTER));
        dst.extend(chars);

        dst
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_utf8() {
        let src = b"hello world o/";
        let mut dst = String::new();
        let res = Encoding::Utf8.decode(src, &mut dst);

        assert_eq!(res, "hello world o/");
        assert!(dst.is_empty());
    }

    #[test]
    fn invalid_utf8() {
        // From beatmap /b/49374
        let src = &[
            32, 209, 44, 49, 44, 55, 56, 50, 52, 53, 44, 57, 48, 50, 52, 53, 44, 48, 44, 48, 44,
            48, 44, 50, 53, 53, 44, 50, 53, 53, 44, 50, 53, 53,
        ];
        let mut dst = String::new();
        Encoding::Utf8.decode(src, &mut dst);

        assert_eq!(dst, " �,1,78245,90245,0,0,0,255,255,255");
    }
}
