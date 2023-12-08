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

            if let Some(line) = trim_end(buf) {
                return Ok(Some(line));
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

            if let Some(line) = trim_end(buf) {
                return Ok(Some(line));
            }
        }
    }
}

fn trim_end(buf: &mut String) -> Option<&str> {
    if let Some(idx) = buf.rfind(|c: char| !c.is_whitespace()) {
        buf.truncate(idx + 1);

        Some(buf)
    } else {
        None
    }
}
