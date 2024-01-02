use std::io::BufRead;

use self::decoder::Decoder;

pub use self::error::DecoderError;

mod decoder;
mod encoding;
mod error;
mod u16_iter;

pub struct Reader<R> {
    decoder: Decoder<R>,
}

impl<R: BufRead> Reader<R> {
    pub fn new(inner: R) -> Result<Self, DecoderError> {
        Decoder::new(inner).map(|decoder| Self { decoder })
    }

    pub fn curr_line(&mut self) -> Result<&str, DecoderError> {
        self.decoder.curr_line()
    }

    pub fn next_line<F, O>(&mut self, f: F) -> Result<Option<O>, DecoderError>
    where
        F: FnOnce(&str) -> O,
    {
        loop {
            match self.decoder.read_line()? {
                Some(line) if should_skip_line(line) => {}
                Some(line) => return Ok(Some(f(line))),
                None => return Ok(None),
            }
        }
    }
}

fn should_skip_line(line: &str) -> bool {
    line.is_empty() || line.starts_with("//")
}
