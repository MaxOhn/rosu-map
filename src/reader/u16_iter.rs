pub struct U16BeIterator<'a> {
    inner: DoubleByteIterator<'a>,
}

impl<'a> U16BeIterator<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            inner: DoubleByteIterator::new(bytes),
        }
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
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            inner: DoubleByteIterator::new(bytes),
        }
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
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_len() {
        let mut iter = DoubleByteIterator::new(&[1, 2, 3]);
        assert_eq!(iter.next(), Some([1, 2]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn le_works() {
        let mut iter = U16LeIterator::new(&[b'1', 0, b'Z', 0]);
        assert_eq!(iter.next(), Some(b'1' as u16));
        assert_eq!(iter.next(), Some(b'Z' as u16));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn be_works() {
        let mut iter = U16BeIterator::new(&[0, b'1', 0, b'Z']);
        assert_eq!(iter.next(), Some(b'1' as u16));
        assert_eq!(iter.next(), Some(b'Z' as u16));
        assert_eq!(iter.next(), None);
    }
}
