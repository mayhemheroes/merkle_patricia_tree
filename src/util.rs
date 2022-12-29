pub struct KeySegmentIterator<'a> {
    data: &'a [u8; 32],
    pos: usize,
    half: bool,
}

impl<'a> KeySegmentIterator<'a> {
    /// Create a new nibble iterator.
    pub fn new(data: &'a [u8; 32]) -> Self {
        Self {
            data,
            pos: 0,
            half: false,
        }
    }

    /// Shortcut to the `nth()` method of a new iterator.
    ///
    /// Panics when n is out of the range [0, 64).
    pub fn nth(data: &'a [u8; 32], n: usize) -> u8 {
        KeySegmentIterator::new(data)
            .nth(n)
            .expect("Key index out of range, value should be in [0, 64).")
    }
}

impl<'a> Iterator for KeySegmentIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= 32 {
            return None;
        }

        let mut value = self.data[self.pos];

        if self.half {
            self.pos += 1;
            value &= 0xF;
        } else {
            value >>= 4;
        }

        self.half = !self.half;
        Some(value)
    }
}
