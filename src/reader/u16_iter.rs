use super::error::DecoderError;

pub struct U16BeIterator<'a> {
    inner: DoubleByteIterator<'a>,
}

impl<'a> U16BeIterator<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecoderError> {
        let inner = DoubleByteIterator::new(bytes)?;

        Ok(Self { inner })
    }
}

impl Iterator for U16BeIterator<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(u16::from_be_bytes)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct U16LeIterator<'a> {
    inner: DoubleByteIterator<'a>,
}

impl<'a> U16LeIterator<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecoderError> {
        let inner = DoubleByteIterator::new(bytes)?;

        Ok(Self { inner })
    }
}

impl Iterator for U16LeIterator<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(u16::from_le_bytes)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

struct DoubleByteIterator<'a> {
    bytes: &'a [u8],
}

impl<'a> DoubleByteIterator<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self, DecoderError> {
        if bytes.len() % 2 != 0 {
            return Err(DecoderError::IncorrectEncoding);
        }

        Ok(Self { bytes })
    }
}

impl Iterator for DoubleByteIterator<'_> {
    type Item = [u8; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let [a, b, ..] = self.bytes else { return None };
        self.bytes = &self.bytes[2..];

        Some([*a, *b])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.bytes.len() / 2;

        (len, Some(len))
    }
}
