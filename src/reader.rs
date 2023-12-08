use std::io::{BufRead, BufReader, Read};

use crate::error::ParseResult;

pub struct Reader<R> {
    inner: BufReader<R>,
}

impl<R: Read> Reader<R> {
    pub fn new(src: R) -> Self {
        Self {
            inner: BufReader::new(src),
        }
    }

    pub fn read_line<'buf>(&mut self, buf: &'buf mut String) -> ParseResult<Option<&'buf str>> {
        loop {
            buf.clear();

            if self.inner.read_line(buf)? == 0 {
                return Ok(None);
            }

            if let Some(idx) = buf.find("//") {
                buf.truncate(idx);
            }

            if not_empty_after_trim(buf) {
                return Ok(Some(buf.as_str()));
            }
        }
    }

    pub fn read_line_with_comments<'buf>(
        &mut self,
        buf: &'buf mut String,
    ) -> ParseResult<Option<&'buf str>> {
        loop {
            buf.clear();

            if self.inner.read_line(buf)? == 0 {
                return Ok(None);
            }

            if not_empty_after_trim(buf) {
                return Ok(Some(buf.as_str()));
            }
        }
    }
}

fn not_empty_after_trim(buf: &mut String) -> bool {
    if let Some(idx) = buf.rfind(|c: char| !c.is_whitespace()) {
        buf.truncate(idx + 1);

        true
    } else {
        false
    }
}
