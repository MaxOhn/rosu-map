use std::{
    io::{BufRead, ErrorKind},
    slice,
};

use super::{encoding::Encoding, error::DecoderError};

pub struct Decoder<R> {
    inner: R,
    read_buf: Vec<u8>,
    // only used for UTF-16 encoded data
    decode_buf: String,
    encoding: Encoding,
}

impl<R: BufRead> Decoder<R> {
    pub fn new(mut inner: R) -> Result<Self, DecoderError> {
        Ok(Self {
            encoding: Self::read_bom(&mut inner)?,
            read_buf: Vec::new(),
            decode_buf: String::new(),
            inner,
        })
    }

    fn read_bom(reader: &mut R) -> Result<Encoding, DecoderError> {
        let buf = loop {
            let available = match reader.fill_buf() {
                Ok(n) => n,
                Err(ref err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) => return Err(err.into()),
            };

            let len = available.len();

            if len >= 3 || len == 0 {
                break available;
            }

            reader.consume(len);
        };

        let (encoding, consumed) = Encoding::from_bom(buf);
        reader.consume(consumed);

        Ok(encoding)
    }

    pub fn read_line(&mut self) -> Result<Option<&str>, DecoderError> {
        self.read_buf.clear();

        if self.inner.read_until(b'\n', &mut self.read_buf)? == 0 {
            return Ok(None);
        }

        // Reading up to b'\n' will miss the final b'\0' for an UTF-16LE encoded
        // string so we need to read an additional byte.
        if self.encoding == Encoding::Utf16LE && self.read_buf.ends_with(b"\n") {
            let mut byte = 0;
            self.inner.read_exact(slice::from_mut(&mut byte))?;
            self.read_buf.push(byte);
        }

        self.encoding
            .decode(&mut self.read_buf, &mut self.decode_buf)
            .map(str::trim)
            .map(Some)
    }
}
