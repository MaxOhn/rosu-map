use std::{
    io::{BufReader, Read},
    slice,
};

use crate::error::{ParseError, ParseResult};

const LATEST_VERSION: i32 = 14;
const VERSION_PREFIX: &str = "osu file format v";

#[derive(Copy, Clone)]
pub enum Encoding {
    Utf8,
    Utf16LE,
    Utf16BE,
}

impl Encoding {
    fn convert(self, buf: Vec<u8>) -> ParseResult<String> {
        match self {
            Encoding::Utf8 => Ok(String::from_utf8(buf)?),
            Encoding::Utf16LE | Encoding::Utf16BE => {
                if buf.len() % 2 != 0 {
                    panic!("len={}", buf.len());
                    // return Err(todo!());
                }

                let ptr = buf.as_ptr().cast();
                let len = buf.len() / 2;

                let slice = unsafe { slice::from_raw_parts(ptr, len) };

                Ok(String::from_utf16(slice)?)
            }
        }
    }
}

pub struct Reader<R> {
    inner: BufReader<R>,
    encoding: Encoding,
}

impl<R: Read> Reader<R> {
    pub fn new(src: R) -> ParseResult<(Self, i32)> {
        let mut inner = BufReader::new(src);
        let (encoding, format_version) = Self::parse_header(&mut inner)?;

        Ok((Self { inner, encoding }, format_version))
    }

    fn parse_header(inner: &mut BufReader<R>) -> ParseResult<(Encoding, i32)> {
        let mut lines = ReadLines::new(inner);

        let mut buf = lines.next().ok_or(ParseError::UnknownFileFormat)??;

        print_buf(&buf);

        let (encoding, shift) = match &buf[..] {
            [0xEF, 0xBB, 0xBF, ..] => (Encoding::Utf8, 3),
            [0xFF, 0xFE, ..] => (Encoding::Utf16LE, 2),
            [0xFE, 0xFF, ..] => (Encoding::Utf16BE, 2),
            _ => (Encoding::Utf8, 0),
        };

        buf.rotate_left(shift);
        buf.truncate(buf.len() - shift);

        let line = encoding.convert(buf)?;

        let line = line.trim();

        if !line.is_empty() {
            return Self::parse_format_version(line).map(|version| (encoding, version));
        }

        for res in lines {
            let buf = res?;

            print_buf(&buf);

            let line = encoding.convert(buf)?;

            let line = line.trim();

            if !line.is_empty() {
                return Self::parse_format_version(line).map(|version| (encoding, version));
            }
        }

        Err(ParseError::UnknownFileFormat)
    }

    fn parse_format_version(line: &str) -> ParseResult<i32> {
        if line.starts_with(VERSION_PREFIX) {
            if let Some(res) = line.rsplit('v').next().map(str::parse) {
                return res.map_err(|_| ParseError::InvalidInteger);
            }
        }

        Ok(LATEST_VERSION)
    }

    pub fn read_line(&mut self) -> ParseResult<Option<String>> {
        for res in ReadLines::new(&mut self.inner) {
            let buf = res?;

            print_buf(&buf);

            let mut line = self.encoding.convert(buf)?;

            if let Some(idx) = line.find("//") {
                line.truncate(idx);
            }

            if not_empty_after_trim(&mut line) {
                return Ok(Some(line));
            }
        }

        Ok(None)
    }

    pub fn read_line_with_comments(&mut self) -> ParseResult<Option<String>> {
        for res in ReadLines::new(&mut self.inner) {
            let buf = res?;

            print_buf(&buf);

            let mut line = self.encoding.convert(buf)?;

            if not_empty_after_trim(&mut line) {
                return Ok(Some(line));
            }
        }

        Ok(None)
    }
}

fn not_empty_after_trim(line: &mut String) -> bool {
    if let Some(idx) = line.rfind(|c: char| !c.is_whitespace()) {
        line.truncate(idx + 1);

        true
    } else {
        false
    }
}

fn print_buf(buf: &[u8]) {
    print!("{buf:?}");

    match std::str::from_utf8(buf) {
        Ok(s) => print!(" - {s:?}"),
        Err(_) => print!(" - invalid utf8"),
    }

    println!();
}

trait Number: Sized {
    fn parse(bytes: &[u8]) -> ParseResult<Self>;
}

impl Number for i32 {
    fn parse(bytes: &[u8]) -> ParseResult<Self> {
        let mut bytes = bytes.iter();

        let (sign, init) = match bytes.next().ok_or(())? {
            b'-' => (-1, 0),
            byte @ b'0'..=b'9' => (1, byte - b'0'),
            _ => return Err(ParseError::InvalidInteger),
        };

        bytes
            .try_fold(init as i32, |n, &byte| {
                let digit = match byte {
                    b'0'..=b'9' => sign * (byte - b'0') as i32,
                    _ => return None,
                };

                n.checked_mul(10).and_then(|n| n.checked_add(digit))
            })
            .ok_or(ParseError::InvalidInteger)
    }
}

struct ReadLines<B> {
    buf: B,
}

impl<B> ReadLines<B> {
    fn new(buf: B) -> Self {
        Self { buf }
    }
}

mod tmp {
    use std::io;

    impl<B: io::BufRead> Iterator for super::ReadLines<B> {
        type Item = io::Result<Vec<u8>>;

        fn next(&mut self) -> Option<io::Result<Vec<u8>>> {
            let mut bytes = vec![];
            match self.buf.read_until(b'\n', &mut bytes) {
                Err(e) => Some(Err(e)),
                Ok(0) => None,
                Ok(_) => Some(Ok(bytes)),
            }
        }
    }
}
