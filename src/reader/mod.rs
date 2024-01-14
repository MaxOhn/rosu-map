use std::io::{BufRead, Result as IoResult};

use self::decoder::Decoder;

mod decoder;
mod encoding;
mod u16_iter;

pub struct Reader<R> {
    decoder: Decoder<R>,
}

impl<R: BufRead> Reader<R> {
    pub fn new(inner: R) -> IoResult<Self> {
        Decoder::new(inner).map(|decoder| Self { decoder })
    }

    pub fn curr_line(&mut self) -> &str {
        self.decoder.curr_line()
    }

    pub fn next_line<O, F: FnOnce(&str) -> O>(&mut self, f: F) -> IoResult<Option<O>> {
        loop {
            match self.decoder.read_line() {
                Ok(Some(line)) if should_skip_line(line) => {}
                Ok(Some(line)) => return Ok(Some(f(line))),
                Ok(None) => return Ok(None),
                Err(err) => return Err(err),
            }
        }
    }
}

fn should_skip_line(line: &str) -> bool {
    line.is_empty() || line.starts_with("//") || line.starts_with(' ') || line.starts_with('_')
}
