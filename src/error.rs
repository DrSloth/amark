//! Module containing error definitions

use std::{
    borrow::Cow,
    fmt::{self, Debug, Display, Formatter},
    io,
    ops::Deref,
    str,
};

/// An error that occured while parsing or rendering an `aml` file.
#[derive(Debug)]
pub enum AmarkError<'buf> {
    /// An io Error occured
    IoError(io::Error),
    /// Got i nput that wasn't expected in this context
    UnexpectedInput {
        /// Description of the expected input
        expected: Cow<'static, [u8]>,
        /// The input we actually got
        got: Cow<'buf, [u8]>,
    },
    /// Unexpected end of line
    UnexpectedEol {
        /// Description of what was expected before the end of the line
        expected: Cow<'static, [u8]>,
    },
    /// Unexpected end of File
    UnexpectedEof {
        /// Description of what was expected before the end of the file
        expected: Cow<'buf, [u8]>,
    },
}

impl<'buf> AmarkError<'buf> {
    /// Convert the given error into an owned structure
    pub fn to_owned(self) -> AmarkError<'static> {
        match self {
            Self::IoError(e) => AmarkError::IoError(e),
            Self::UnexpectedInput { expected, got } => AmarkError::UnexpectedInput {
                expected: expected.into_owned().into(),
                got: got.into_owned().into(),
            },
            Self::UnexpectedEof { expected } => AmarkError::UnexpectedEof {
                expected: expected.into_owned().into(),
            },
            Self::UnexpectedEol { expected } => AmarkError::UnexpectedEol {
                expected: expected.into_owned().into(),
            },
        }
    }
}

impl<'buf> From<io::Error> for AmarkError<'buf> {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl<'buf> Display for AmarkError<'buf> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::IoError(ref e) => write!(f, "An io operation failed: {}", e),
            Self::UnexpectedInput {
                ref expected,
                ref got,
            } => {
                write!(
                    f,
                    "Unexpected input:\nexpected: {}\ngot: {}",
                    ByteDisp(expected),
                    ByteDisp(got)
                )
            }
            Self::UnexpectedEol { ref expected } => {
                write!(
                    f,
                    "Unexpected end of line, expected {} before end of line",
                    ByteDisp(expected)
                )
            }
            Self::UnexpectedEof { ref expected } => {
                write!(
                    f,
                    "Unexpected end of file:\nexpected: {}\ngot: End of File",
                    ByteDisp(expected)
                )
            }
        }
    }
}

/// Helper structure to display bytes as string if possible
pub struct ByteDisp<'a, T>(pub &'a T);

impl<'a, T: Deref<Target = [u8]>> Debug for ByteDisp<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a, T: Deref<Target = [u8]>> Display for ByteDisp<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Ok(s) = str::from_utf8(self.0.deref()) {
            write!(f, "{:?}", s)
        } else {
            write!(f, "{:?}", self.0.deref())
        }
    }
}
