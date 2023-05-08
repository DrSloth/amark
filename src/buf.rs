//! A simple buffer wrapper that tracks how many bytes have been processed in the current line

use std::io::{self, BufRead};

/// A specialized buffer to work process incoming text.
#[derive(Debug)]
pub struct Buf {
    /// The internal storage for the buffer
    storage: Vec<u8>,
    /// How many bytes have been processed in the given line
    processed: usize,
}

impl Buf {
    /// Create a buffer that uses the passed vector as an internal storage
    pub fn with_storage(storage: Vec<u8>) -> Self {
        Self {
            storage,
            processed: 0,
        }
    }

    /// Try to clear the buffer and pull a new line from the given reader
    ///
    /// # Errors
    ///
    /// Returns an error when the given readers implementation of `read_until` returns an error.
    pub fn fill_with_line<B: BufRead>(
        &mut self,
        cur_line: &mut usize,
        reader: &mut B,
    ) -> io::Result<()> {
        // Performance Note:
        // This could probably be made more performant by only requiring Read and not going through
        // another buffer, buffering ourselves, calculating the line breaks ONCE and
        // going over those lines

        self.storage.clear();
        self.processed = 0;
        *cur_line = cur_line.wrapping_add(1);
        reader.read_until(b'\n', &mut self.storage)?;
        Ok(())
    }

    // /// Skips forward in the buffer until the pattern is found. Returns None if the Pattern is not
    // /// found or else the subslice from where the pattern was found until the end of the buffer.
    // pub fn skip_until(&mut self, mut pattern: impl FnMut(u8) -> bool) -> Option<&[u8]> {
    //     for (i, b) in self.storage.get(self.processed..)?.iter().enumerate() {
    //         Self::process(&mut self.processed, 1);
    //         if pattern(*b) {
    //             return self.storage.get(i..);
    //         }
    //     }

    //     None
    // }

    // /// Get the input until the specified pattern matches. Returns none if the pattern never matches
    // /// or else the subslice from the start of the buffer until where the pattern was found
    // /// excluding the pattern.
    // pub fn take_until(&mut self, pattern: impl FnMut(u8) -> bool) -> Option<(&[u8], u8)> {
    //     Self::take_until_inner(&self.storage, &mut self.processed, pattern)
    // }

    /// Take as many bytes as the searcher function returns and rewind by as many bytes as the
    /// rewind function returns when called with the last found byte.
    pub fn take_until_rewind(
        &mut self,
        searcher: impl FnMut(&[u8]) -> Option<usize>,
        mut rewind: impl FnMut(u8) -> usize,
    ) -> Option<(&[u8], u8)> {
        match Self::take_until_inner(&self.storage, &mut self.processed, searcher) {
            Some((buf, byte)) => {
                self.processed = self.processed.saturating_sub(rewind(byte));
                Some((buf, byte))
            }
            None => None,
        }
    }

    /// Helper function to make use of the ? operator could be a try block in the future
    #[inline(always)]
    fn take_until_inner<'a>(
        buf: &'a [u8],
        processed: &mut usize,
        mut searcher: impl FnMut(&[u8]) -> Option<usize>,
    ) -> Option<(&'a [u8], u8)> {
        let buf = buf.get(*processed..)?;
        match searcher(buf) {
            Some(pos) => {
                let ret = buf.get(..pos);
                Self::process(processed, pos.wrapping_add(1));
                Some((ret?, *buf.get(pos)?))
            }
            None => {
                *processed = buf.len();
                None
            }
        }
    }

    // pub fn strip_prefix<'buf>(&'buf mut self, prefix: &[u8]) -> Result<&'buf [u8], &'buf [u8]> {
    //     if let Some(stripped) = self.storage.strip_prefix(prefix) {
    //         Self::process(&mut self.processed, prefix.len().wrapping_sub(1));
    //         Ok(stripped)
    //     } else {
    //         Err(self.rest())
    //     }
    // }

    /// Search forward in the buffer and read more lines if need
    pub fn search_forward<B: BufRead>(
        &mut self,
        current_line: &mut usize,
        reader: &mut B,
        mut pattern: impl FnMut(u8) -> bool,
    ) -> io::Result<bool> {
        while !self.storage_empty() {
            match self.next_byte() {
                Some(b) if pattern(b) => return Ok(true),
                Some(_) => (),
                None => self.fill_with_line(current_line, reader)?,
            }
        }

        Ok(false)
    }

    // pub fn rest(&self) -> &[u8] {
    //     self.storage.get(self.processed..).unwrap_or(&[])
    // }

    /// Retrieve the next byte if one is available
    pub fn next_byte(&mut self) -> Option<u8> {
        if let Some(ret) = self.storage.get(self.processed).copied() {
            Self::process(&mut self.processed, 1);
            Some(ret)
        } else {
            None
        }
    }

    /// "Eat up" some of the bytes and mark them as processed by incrementing the processed field.
    fn process(processed: &mut usize, eaten: usize) {
        *processed = processed.saturating_add(eaten);
    }

    /// Rewind ("throw up") some of the processed bytes to make them processible again
    pub fn rewind(&mut self, n: usize) {
        self.processed = self.processed.saturating_sub(n);
    }

    /// Check if the given storage of this buffer is empty. This means no bytes could be read
    /// anymore not. This does not indicate wether there are more bytes to process currently
    pub fn storage_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Take the storage buffer for later reuse
    pub fn take_storage(self) -> Vec<u8> {
        self.storage
    }
}
