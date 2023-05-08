//! Simple parser/tokenizer for the Amark markup language. This doesn't give any complex data
//! structure like an AST but some context aware tokens with semantic meaning. This is usually
//! enough to generate some other text based on the input like another markup language e.g. HTML.

mod buf;
mod error;

pub use error::AmarkError;
use error::ByteDisp;

use std::{
    fmt::Debug,
    io::{self, BufRead, Write},
};

use crate::buf::Buf;

/// A [`Result`] type that uses [`AmarkError`] as an error type.
pub type AmarkResult<'buf, T> = Result<T, AmarkError<'buf>>;

/// Reader structure for `Amark` markup. This does not hold a reader, the reader needs to be
/// repeatedly passed to [`AmarkReader::parse_next`]
#[derive(Debug)]
pub struct AmarkReader {
    /// The inner actual reader structure
    inner: AmarkReaderInner,
    /// The current line
    cur_line: usize,
}

impl AmarkReader {
    /// Create a new [`AmarkReader`] with an empty buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`AmarkReader`] with a given buffer
    pub fn with_buf(buf: Vec<u8>) -> Self {
        Self {
            inner: AmarkReaderInner::with_buf(buf),
            cur_line: 0,
        }
    }

    /// Parse the next token
    ///
    /// # Errors
    ///
    /// This errors when reading from `reader` fails or the format of the markup is wrong.
    pub fn parse_next<'buf, B: BufRead>(
        &'buf mut self,
        reader: &mut B,
    ) -> AmarkResult<'buf, AmarkToken> {
        self.inner.parse_next_inner(&mut self.cur_line, reader)
    }

    /// Parse the next token and get the current line
    ///
    /// In the future this function may be removed if the borrows are split more.
    pub fn parse_next_get_cur_line<'buf, B: BufRead>(
        &'buf mut self,
        reader: &mut B,
    ) -> (AmarkResult<'buf, AmarkToken>, usize) {
        (
            AmarkReaderInner::parse_next_inner(&mut self.inner, &mut self.cur_line, reader),
            self.cur_line,
        )
    }

    /// Take the inner buffer for later reuse
    pub fn take_buf(self) -> Vec<u8> {
        self.inner.buf.take_storage()
    }

    /// Get the current line
    pub fn cur_line(&self) -> usize {
        self.cur_line
    }
}

impl Default for AmarkReader {
    fn default() -> Self {
        Self::with_buf(Vec::new())
    }
}

/// The inner state structure of the amark reader
///
/// This could be split off later as a parser state structure that has to be passed in or something
/// like that to further split the borrow.
#[derive(Debug)]
struct AmarkReaderInner {
    /// The buffer to proccess data
    buf: Buf,
    /// A stack of [`Context`] items
    context_stack: ContextStack,
}

impl AmarkReaderInner {
    /// Create a new instance with the given storage for the buffer
    fn with_buf(storage: Vec<u8>) -> Self {
        Self {
            buf: Buf::with_storage(storage),
            context_stack: ContextStack::new(),
        }
    }

    /// The actual parsing logic, a PDA using the context and incoming text
    fn parse_next_inner<'buf: 'ret + 'err, 'ret, 'err, B: BufRead>(
        &'buf mut self,
        cur_line: &mut usize,
        reader: &mut B,
    ) -> AmarkResult<'err, AmarkToken<'ret>> {
        loop {
            while let Some(b) = self.buf.next_byte() {
                match self.context_stack.last() {
                    Context::ItemName => match b {
                        b @ (b'[' | b'(' | b'{') => {
                            if b != b'(' {
                                self.context_stack.pop();
                            }
                            let (tok, ctx) = parse_ascii_context_char(b);
                            self.context_stack.push(ctx);
                            self.buf
                                .search_forward(cur_line, reader, |b| !b.is_ascii_whitespace())?;
                            self.buf.rewind(1);
                            return Ok(tok);
                        }
                        b';' => {
                            self.context_stack.pop();
                            // Skip ahead the rest of the whitespace after the ;
                            while let Some(b) = self.buf.next_byte() {
                                if !b.is_ascii_whitespace() {
                                    self.buf.rewind(1);
                                    break;
                                }
                            }
                            return Ok(AmarkToken::ItemEnd);
                        }
                        b if b.is_ascii_whitespace() => (),
                        b => {
                            return Err(AmarkError::UnexpectedInput {
                                expected: b"Start or End of item token {, (, [ or ;"
                                    .as_ref()
                                    .into(),
                                got: vec![b].into(),
                            })
                        }
                    },
                    ctx @ (Context::Container | Context::TopLevel) => match b {
                        b']' => {
                            return if ctx == Context::Container {
                                self.context_stack.pop();
                                Ok(AmarkToken::ContainerEnd)
                            } else {
                                Err(AmarkError::UnexpectedInput {
                                    expected: b"Item or EOF".as_ref().into(),
                                    got: b"]".as_ref().into(),
                                })
                            };
                        }
                        b'}' => {
                            return Err(AmarkError::UnexpectedInput {
                                expected: Context::Container.expected().into(),
                                got: Context::Block.expected().into(),
                            });
                        }
                        b if is_ascii_ident_char(b) => {
                            self.buf.rewind(1);
                            // Polonius could prevent this allocation
                            let item =
                                Self::read_item_name(&mut self.buf).map_err(|e| e.to_owned())?;
                            self.context_stack.push(Context::ItemName);

                            return Ok(AmarkToken::ItemName(item));
                        }
                        _ => (),
                    },
                    Context::Block => {
                        match b {
                            b'\n' => {
                                return Ok(AmarkToken::EmptyLine);
                            }
                            b'\\' => return self.parse_escape_sequence(),
                            b'@' => {
                                let item_name = Self::read_item_name(&mut self.buf)?;
                                self.context_stack.push(Context::ItemName);
                                return Ok(AmarkToken::ItemName(item_name));
                            }
                            b'}' => {
                                self.context_stack.pop();
                                // Skip whitespace ahead
                                loop {
                                    if self
                                        .buf
                                        .next_byte()
                                        .map_or(true, |b| b.is_ascii_whitespace())
                                    {
                                        break;
                                    }
                                }
                                return Ok(AmarkToken::BlockEnd);
                            }
                            b if b.is_ascii_whitespace() => (), // Skip whitespace
                            _ => {
                                self.buf.rewind(1);
                                let (line, _) = Self::try_read_text(&mut self.buf, b'}')
                                    .ok_or_else(|| AmarkError::UnexpectedEof {
                                        expected: b"End of line indicator for text line \
                                                or end of item indicator }"
                                            .as_ref()
                                            .into(),
                                    })?;

                                return Ok(AmarkToken::Text(line));
                            }
                        }
                    }
                    Context::EscapeSequence => {
                        if b == b'(' {
                            self.context_stack.push(Context::Params);
                            self.buf
                                .search_forward(cur_line, reader, |b| !b.is_ascii_whitespace())?;
                            self.buf.rewind(1);
                            return Ok(AmarkToken::ParamsStart);
                        }

                        self.buf.rewind(1);
                        self.context_stack.pop();
                    }
                    Context::Params => {
                        match b {
                            b'\\' => {
                                return self.parse_escape_sequence();
                            }
                            b')' => {
                                while let Some(b) = self.buf.next_byte() {
                                    if !b.is_ascii_whitespace() {
                                        self.buf.rewind(1);
                                        break;
                                    }
                                }
                                self.context_stack.pop();
                                // Pop if we are not an item (we don't need a ; for escapes)
                                if self.context_stack.last() != Context::ItemName {
                                    self.context_stack.pop();
                                }
                                return Ok(AmarkToken::ParamsEnd);
                            }
                            _ => {
                                self.buf.rewind(1);
                                let (line, _) = Self::try_read_text(&mut self.buf, b')')
                                    .ok_or_else(|| AmarkError::UnexpectedEof {
                                        expected: b"End of line indicator for text line \
                                                or end of params indicator )"
                                            .as_ref()
                                            .into(),
                                    })?;

                                return Ok(AmarkToken::Text(line));
                            }
                        }
                    }
                }
            }

            self.buf.fill_with_line(cur_line, reader)?;

            match self.context_stack.last() {
                Context::TopLevel if self.buf.storage_empty() => {
                    return Ok(AmarkToken::End);
                }
                ctx if self.buf.storage_empty() => {
                    return Err(AmarkError::UnexpectedEof {
                        expected: ctx.expected().into(),
                    })
                }
                _ => (),
            }
        }
    }

    /// Try to read a line of text
    fn try_read_text(buf: &mut Buf, end_char: u8) -> Option<(&[u8], u8)> {
        buf.take_until_rewind(
            |haystack| memchr::memchr3(b'\n', end_char, b'\\', haystack),
            |b| if [end_char, b'\\'].contains(&b) { 1 } else { 0 },
        )
    }

    /// Try to parse an escape sequence, escape sequences are always one char long
    ///
    /// # Errors
    ///
    /// Returns an error when EOF is encountered instead of another character
    fn parse_escape_sequence(&mut self) -> AmarkResult<AmarkToken> {
        self.context_stack.push(Context::EscapeSequence);
        if let Some(b) = self.buf.next_byte() {
            Ok(AmarkToken::EscapeSequence(b))
        } else {
            Err(AmarkError::UnexpectedEof {
                expected: Context::EscapeSequence.expected().as_ref().into(),
            })
        }
    }

    /// Try to read an item name. Reads until the next character that isn't valid for item names
    ///
    /// # Errors
    ///
    /// Returns an error when `EOF` is encountered instead of an item name
    fn read_item_name(buf: &mut Buf) -> AmarkResult<&[u8]> {
        let name = buf
            .take_until_rewind(
                |b| {
                    b.iter()
                        .enumerate()
                        .find(|&(_, &b)| !is_ascii_ident_char(b))
                        .map(|(pos, _b)| pos)
                },
                |b| {
                    if is_ascii_context_char(b) || b == b';' {
                        1
                    } else {
                        0
                    }
                },
            )
            .ok_or_else(|| AmarkError::UnexpectedEof {
                expected: b"Any other symbol after item name".as_ref().into(),
            })?;

        Ok(name.0)
    }
}

/// A parsed token from an Amark markup
#[derive(PartialEq, Eq)]
pub enum AmarkToken<'buf> {
    /// Start of a block item '{'
    BlockStart,
    /// Start of a parameter list '('
    ParamsStart,
    /// Start of a container item '['
    ContainerStart,
    /// End of a block item '}'
    BlockEnd,
    /// End of a parameter list ')'
    ParamsEnd,
    /// End of a container element ']'
    ContainerEnd,
    /// End of an item put after param lists or items directly ';'
    ItemEnd,
    /// An empty line
    EmptyLine,
    /// End of input
    End,
    /// An item with the given name
    ItemName(&'buf [u8]),
    /// A line of text with the given content
    Text(&'buf [u8]),
    /// An escape sequence character
    EscapeSequence(u8),
}

impl AmarkToken<'_> {
    /// A more efficient version of the debug implementation which doesn't use `core::fmt`
    ///
    /// # Errors
    ///
    /// Returns an error when the implementation of `write_all` of W errors.
    pub fn dump<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        match *self {
            Self::BlockStart => writer.write_all(b"BlockStart")?,
            Self::ParamsStart => writer.write_all(b"ParamsStart")?,
            Self::ContainerStart => writer.write_all(b"ContainerStart")?,
            Self::BlockEnd => writer.write_all(b"BlockEnd")?,
            Self::ParamsEnd => writer.write_all(b"ParamsEnd")?,
            Self::ContainerEnd => writer.write_all(b"ContainerEnd")?,
            Self::ItemEnd => writer.write_all(b"ItemEnd")?,
            Self::EmptyLine => writer.write_all(b"EmptyLine")?,
            Self::End => writer.write_all(b"End")?,
            Self::ItemName(name) => {
                writer.write_all(b"ItemName(")?;
                writer.write_all(name)?;
                writer.write_all(b")")?;
            }
            Self::Text(text) => {
                writer.write_all(b"Text(")?;
                writer.write_all(text)?;
                writer.write_all(b")")?;
            }
            Self::EscapeSequence(b) => {
                writer.write_all(b"EscapeSequence(")?;
                writer.write_all(&[b])?;
                writer.write_all(b")")?;
            }
        }

        Ok(())
    }
}

impl<'buf> Debug for AmarkToken<'buf> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ItemName(ref name) => f.debug_tuple("ItemName").field(&ByteDisp(name)).finish(),
            Self::BlockStart => write!(f, "BlockStart"),
            Self::ParamsStart => write!(f, "ParamsStart"),
            Self::ContainerStart => write!(f, "ContainerStart"),
            Self::BlockEnd => write!(f, "BlockEnd"),
            Self::ParamsEnd => write!(f, "ParamsEnd"),
            Self::ContainerEnd => write!(f, "ContainerEnd"),
            Self::ItemEnd => write!(f, "ItemEnd"),
            Self::Text(ref t) => f.debug_tuple("Text").field(&ByteDisp(t)).finish(),
            Self::EmptyLine => write!(f, "EmptyLine"),
            Self::End => write!(f, "End"),
            Self::EscapeSequence(ref seq) => {
                if let Some(d) = char::from_u32((*seq).into()) {
                    write!(f, "EscapeSequence({})", d)
                } else {
                    write!(f, "EscapeSequence({})", seq)
                }
            }
        }
    }
}

impl<'buf> AmarkToken<'buf> {
    /// Check if this token indicates the end of a context
    pub fn is_context_end(&self) -> bool {
        matches!(*self, Self::ParamsEnd | Self::BlockEnd | Self::ContainerEnd)
    }
}

/// A stack of [`Context`] items showing where in an Amark file the parser currently is.
///
/// The [`Context`] items are used to know which Tokens have meaning and which tokens are expected.
#[derive(Debug)]
struct ContextStack {
    /// Storage for the stack
    stack: Vec<Context>,
}

impl ContextStack {
    /// Create  a new Empty Context stack
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(5),
        }
    }

    /// Push a [`Context`] on top of the [`ContextStack`]
    pub fn push(&mut self, ctx: Context) {
        self.stack.push(ctx);
    }

    /// Retrieve and remove the last item from the context stack,
    /// if the stack is empty [`Context::TopLevel`] will be returned.
    pub fn pop(&mut self) -> Context {
        self.stack.pop().unwrap_or(Context::TopLevel)
    }

    /// Retrieve the last item from the context stack, if the stack is empty [`Context::TopLevel`]
    /// will be returned.
    pub fn last(&self) -> Context {
        self.stack.last().copied().unwrap_or(Context::TopLevel)
    }
}

/// The context the parser is currently in
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Context {
    /// Inside a block item
    Block,
    /// Inside a parameter list
    Params,
    /// Inside a container item
    Container,
    /// Top level "no" context
    TopLevel,
    /// After an item name
    ItemName,
    /// Parsing an EscapeSequence
    EscapeSequence,
}

impl Context {
    /// String which shows expected token to end the given context. For error reporting on
    /// unexpected EOF.
    pub fn expected(self) -> &'static [u8] {
        match self {
            Self::Block => b"End of Block: }",
            Self::Params => b"End of Parameter List: )",
            Self::Container => b"End of Container: ]",
            Self::TopLevel => b"Any valid Token",
            Self::ItemName => b"An element start indicator: (, [ or {",
            Self::EscapeSequence => b"Escape Sequence after `\\`",
        }
    }
}

/// Check wether a given character is a valid ascii identifier character, used for item names.
fn is_ascii_ident_char(byte: u8) -> bool {
    !byte.is_ascii_whitespace() && !is_ascii_context_char(byte) && byte != b';'
}

/// Wether the given character will trigger a switch into an "item context" (block, container, params)
fn is_ascii_context_char(byte: u8) -> bool {
    [b'{', b'}', b'(', b')', b'[', b']'].contains(&byte)
}

/// Parse the given item context character into the matching token and the fitting next context for
/// the parser to switch to.
fn parse_ascii_context_char(byte: u8) -> (AmarkToken<'static>, Context) {
    match byte {
        b'[' => (AmarkToken::ContainerStart, Context::Container),
        b'{' => (AmarkToken::BlockStart, Context::Block),
        b'(' => (AmarkToken::ParamsStart, Context::Params),
        _ => unreachable!("Only one of these bytes should be passed"),
    }
}
