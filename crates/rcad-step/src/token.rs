use crate::error::{Error, Problem};
use crate::graph::EntityId;
use crate::text;

/// One lexical item, with everything that could be worked out about it worked
/// out: numbers parsed, strings decoded, ids held.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token<'a> {
    /// A type name or a section word, upper case. A user-defined keyword keeps
    /// its leading `!`.
    Keyword(&'a str),
    /// The name between the dots of `.MILLI.`, without them.
    Enumeration(&'a str),
    /// A quoted string, decoded.
    Text(String),
    /// `#123`.
    Reference(EntityId),
    /// `1`, `-4`.
    Integer(i64),
    /// `1.`, `1.E+03`. Always finite. With the place of the last digit the
    /// file wrote it to, which the `f64` does not keep.
    Real(f64, Option<LastDigit>),
    /// `(`.
    Open,
    /// `)`.
    Close,
    /// `,`.
    Comma,
    /// `;`.
    Semicolon,
    /// `=`.
    Equals,
    /// `$`.
    Nothing,
    /// `*`.
    Derived,
    /// There are no more bytes.
    End,
}

impl Token<'_> {
    /// How to name this in a refusal, in the file's own spelling where there is
    /// one.
    pub(crate) fn describe(&self) -> String {
        match self {
            Token::Keyword(word) => (*word).to_string(),
            Token::Enumeration(name) => format!(".{name}."),
            Token::Text(text) => format!("the string {text:?}"),
            Token::Reference(id) => format!("{id}"),
            Token::Integer(number) => format!("{number}"),
            Token::Real(number, _) => format!("{number}"),
            Token::Open => "(".to_string(),
            Token::Close => ")".to_string(),
            Token::Comma => ",".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::Equals => "=".to_string(),
            Token::Nothing => "$".to_string(),
            Token::Derived => "*".to_string(),
            Token::End => "the end of the file".to_string(),
        }
    }
}

/// The power of ten of the last digit a real was written to: `-12` for
/// `0.463647609001` and for `4.63647609001E-01`, `0` for `1.`, `3` for
/// `1.E+03`, and `-13` for `0.4636476090010`, whose last zero is a digit the
/// file wrote.
///
/// **An exporter that cuts a number to a fixed count of digits says so only
/// here.** The `f64` the text reads as cannot tell `0.4636476090010` from
/// `0.463647609001`, and its shortest spelling drops the zero. A cone's slope
/// read from a half-angle cut that way is put back from the cone's corners, and
/// this is how far the angle may have been cut (`docs/decisions.md` §366.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LastDigit(pub(crate) i32);

/// Walks a Part 21 file's bytes one lexical item at a time.
///
/// Holds one item of lookahead, because every place the grammar branches — is
/// this instance simple or complex, is this value a plain keyword or a typed
/// one — branches on the item after the one just taken.
#[derive(Debug)]
pub(crate) struct Tokens<'a> {
    bytes: &'a [u8],
    at: usize,
    ahead: Option<(usize, Token<'a>)>,
}

impl<'a> Tokens<'a> {
    pub(crate) fn over(bytes: &'a [u8]) -> Self {
        Tokens {
            bytes,
            at: 0,
            ahead: None,
        }
    }

    /// Where the reader has got to, for the caller that wants to match raw
    /// bytes rather than a token — which the two hyphenated lines
    /// `ISO-10303-21;` and `END-ISO-10303-21;` need, since no keyword may hold
    /// a hyphen.
    pub(crate) fn rest(&mut self) -> Result<(usize, &'a [u8]), Error> {
        assert!(
            self.ahead.is_none(),
            "rest() after a peek would hand back bytes already read"
        );
        self.skip_blanks()?;
        Ok((self.at, &self.bytes[self.at..]))
    }

    /// Steps past raw bytes the caller matched itself through [`rest`](Self::rest).
    pub(crate) fn step_over(&mut self, count: usize) {
        assert!(self.ahead.is_none(), "step_over after a peek");
        self.at += count;
    }

    /// The next item without taking it.
    pub(crate) fn peek(&mut self) -> Result<&(usize, Token<'a>), Error> {
        if self.ahead.is_none() {
            self.ahead = Some(self.scan()?);
        }
        Ok(self.ahead.as_ref().expect("the item was just put there"))
    }

    /// The next item, taken.
    pub(crate) fn take(&mut self) -> Result<(usize, Token<'a>), Error> {
        match self.ahead.take() {
            Some(ready) => Ok(ready),
            None => self.scan(),
        }
    }

    /// Takes the next item and insists it is the one the grammar wants here.
    pub(crate) fn take_exactly(
        &mut self,
        wanted: Token<'a>,
        called: &'static str,
    ) -> Result<(), Error> {
        let (at, found) = self.take()?;
        if found == wanted {
            return Ok(());
        }
        Err(self.fault(
            at,
            Problem::NotWhatWasExpected {
                wanted: called,
                found: found.describe(),
            },
        ))
    }

    pub(crate) fn fault(&self, at: usize, problem: Problem) -> Error {
        error_at(self.bytes, at, problem)
    }

    fn scan(&mut self) -> Result<(usize, Token<'a>), Error> {
        self.skip_blanks()?;
        let at = self.at;
        let Some(&first) = self.bytes.get(at) else {
            return Ok((at, Token::End));
        };
        let token = match first {
            b'(' => self.one_byte(Token::Open),
            b')' => self.one_byte(Token::Close),
            b',' => self.one_byte(Token::Comma),
            b';' => self.one_byte(Token::Semicolon),
            b'=' => self.one_byte(Token::Equals),
            b'$' => self.one_byte(Token::Nothing),
            b'*' => self.one_byte(Token::Derived),
            b'\'' => self.string(at)?,
            b'#' => self.reference(at)?,
            b'.' => self.enumeration(at)?,
            b'0'..=b'9' | b'+' | b'-' => self.number(at)?,
            b'A'..=b'Z' | b'_' | b'!' | b'a'..=b'z' => self.keyword(at)?,
            b'"' => {
                return Err(self.fault(at, Problem::BinaryValueThisReaderDoesNotRead));
            }
            other => {
                return Err(self.fault(
                    at,
                    Problem::NotWhatWasExpected {
                        wanted: "a value, a keyword or a punctuation mark",
                        found: describe_byte(other),
                    },
                ));
            }
        };
        Ok((at, token))
    }

    fn one_byte(&mut self, token: Token<'a>) -> Token<'a> {
        self.at += 1;
        token
    }

    fn string(&mut self, at: usize) -> Result<Token<'a>, Error> {
        let mut end = at + 1;
        loop {
            match self.bytes.get(end) {
                None => return Err(self.fault(at, Problem::StringNeverEnds)),
                // Two in a row is one apostrophe in the text, not the end.
                Some(b'\'') if self.bytes.get(end + 1) == Some(&b'\'') => end += 2,
                Some(b'\'') => break,
                Some(_) => end += 1,
            }
        }
        let decoded = text::decode(&self.bytes[at + 1..end], at + 1, self.bytes)?;
        self.at = end + 1;
        Ok(Token::Text(decoded))
    }

    fn reference(&mut self, at: usize) -> Result<Token<'a>, Error> {
        let digits = run_of(self.bytes, at + 1, |byte| byte.is_ascii_digit());
        if digits == at + 1 {
            return Err(self.fault(
                at,
                Problem::NotWhatWasExpected {
                    wanted: "digits after #",
                    found: self.what_follows(at + 1),
                },
            ));
        }
        let text = as_ascii(&self.bytes[at + 1..digits]);
        let id = text
            .parse::<u64>()
            .map_err(|_| self.fault(at, Problem::EntityIdTooLarge { text: text.into() }))?;
        self.at = digits;
        Ok(Token::Reference(EntityId(id)))
    }

    fn enumeration(&mut self, at: usize) -> Result<Token<'a>, Error> {
        let start = at + 1;
        let mut end = start;
        if matches!(self.bytes.get(end), Some(b'A'..=b'Z' | b'_')) {
            end = run_of(self.bytes, end, |byte| {
                byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'
            });
        }
        if end == start || self.bytes.get(end) != Some(&b'.') {
            return Err(self.fault(
                at,
                Problem::NotWhatWasExpected {
                    wanted: "an enumeration name between two dots",
                    found: self.what_follows(at),
                },
            ));
        }
        self.at = end + 1;
        Ok(Token::Enumeration(as_ascii(&self.bytes[start..end])))
    }

    fn number(&mut self, at: usize) -> Result<Token<'a>, Error> {
        let after_sign = match self.bytes[at] {
            b'+' | b'-' => at + 1,
            _ => at,
        };
        let whole = run_of(self.bytes, after_sign, |byte| byte.is_ascii_digit());
        if whole == after_sign {
            return Err(self.fault(
                at,
                Problem::NotWhatWasExpected {
                    wanted: "digits after a sign",
                    found: self.what_follows(after_sign),
                },
            ));
        }
        if self.bytes.get(whole) != Some(&b'.') {
            if matches!(self.bytes.get(whole), Some(b'E' | b'e')) {
                let end = run_of(self.bytes, whole + 1, |byte| {
                    byte.is_ascii_digit() || byte == b'+' || byte == b'-'
                });
                let text = as_ascii(&self.bytes[at..end]).into();
                return Err(self.fault(at, Problem::ExponentWithoutADecimalPoint { text }));
            }
            let text = as_ascii(&self.bytes[at..whole]);
            let number = text
                .parse::<i64>()
                .map_err(|_| self.fault(at, Problem::IntegerTooLarge { text: text.into() }))?;
            self.at = whole;
            return Ok(Token::Integer(number));
        }

        let fraction = run_of(self.bytes, whole + 1, |byte| byte.is_ascii_digit());
        let mut end = fraction;
        let mut exponent_written = None;
        if self.bytes.get(end) == Some(&b'E') {
            let exponent = match self.bytes.get(end + 1) {
                Some(b'+' | b'-') => end + 2,
                _ => end + 1,
            };
            let digits = run_of(self.bytes, exponent, |byte| byte.is_ascii_digit());
            if digits == exponent {
                return Err(self.fault(
                    at,
                    Problem::NotWhatWasExpected {
                        wanted: "digits after E",
                        found: self.what_follows(exponent),
                    },
                ));
            }
            exponent_written = Some(as_ascii(&self.bytes[fraction + 1..digits]));
            end = digits;
        }
        let text = as_ascii(&self.bytes[at..end]);
        let number: f64 = text
            .parse()
            .expect("the bytes just scanned are a number Rust reads");
        if !number.is_finite() {
            return Err(self.fault(at, Problem::RealTooLarge { text: text.into() }));
        }
        self.at = end;
        Ok(Token::Real(
            number,
            last_digit(fraction - (whole + 1), exponent_written),
        ))
    }

    fn keyword(&mut self, at: usize) -> Result<Token<'a>, Error> {
        let start = match self.bytes[at] {
            b'!' => at + 1,
            _ => at,
        };
        let end = run_of(self.bytes, start, |byte| {
            byte.is_ascii_alphanumeric() || byte == b'_'
        });
        let word = as_ascii(&self.bytes[at..end]);
        if word.bytes().any(|byte| byte.is_ascii_lowercase()) {
            return Err(self.fault(
                at,
                Problem::KeywordIsNotUpperCase {
                    keyword: word.into(),
                },
            ));
        }
        self.at = end;
        Ok(Token::Keyword(word))
    }

    fn skip_blanks(&mut self) -> Result<(), Error> {
        loop {
            self.at = run_of(self.bytes, self.at, |byte| {
                matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
            });
            if self.bytes.get(self.at) != Some(&b'/') {
                return Ok(());
            }
            if self.bytes.get(self.at + 1) != Some(&b'*') {
                return Err(self.fault(
                    self.at,
                    Problem::NotWhatWasExpected {
                        wanted: "/* to open a comment",
                        found: self.what_follows(self.at),
                    },
                ));
            }
            let opened = self.at;
            let mut end = self.at + 2;
            loop {
                match self.bytes.get(end) {
                    None => return Err(self.fault(opened, Problem::CommentNeverEnds)),
                    Some(b'*') if self.bytes.get(end + 1) == Some(&b'/') => break,
                    Some(_) => end += 1,
                }
            }
            self.at = end + 2;
        }
    }

    /// The bytes at a spot, named for a refusal, cut short so a hostile file
    /// cannot make the message the size of itself.
    fn what_follows(&self, at: usize) -> String {
        match self.bytes.get(at) {
            None => "the end of the file".to_string(),
            Some(&byte) => describe_byte(byte),
        }
    }
}

/// The place of the last digit of a real written with `decimals` digits after
/// its point and `exponent` after its `E`, if it has one.
///
/// `None` where that place is not an `i32`, which takes an exponent of ten
/// digits on a real that is still finite — `0.E+9999999999` is nought.
fn last_digit(decimals: usize, exponent: Option<&str>) -> Option<LastDigit> {
    let exponent: i32 = match exponent {
        None => 0,
        Some(written) => written.parse().ok()?,
    };
    exponent
        .checked_sub(i32::try_from(decimals).ok()?)
        .map(LastDigit)
}

/// Where in the file a byte offset falls, counting from one.
///
/// Walking the file to find out is fine because it happens once, on the way out
/// with a refusal in hand, and never on the path that succeeds.
pub(crate) fn error_at(bytes: &[u8], at: usize, problem: Problem) -> Error {
    let before = &bytes[..at.min(bytes.len())];
    let line = 1 + before.iter().filter(|&&byte| byte == b'\n').count();
    let start_of_line = match before.iter().rposition(|&byte| byte == b'\n') {
        Some(newline) => newline + 1,
        None => 0,
    };
    Error {
        problem,
        line,
        column: 1 + (before.len() - start_of_line),
    }
}

fn run_of(bytes: &[u8], from: usize, wanted: impl Fn(u8) -> bool) -> usize {
    let mut at = from;
    while at < bytes.len() && wanted(bytes[at]) {
        at += 1;
    }
    at
}

/// A slice the scanner has already checked is ASCII.
fn as_ascii(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("the scanner only takes ASCII into a keyword or a number")
}

fn describe_byte(byte: u8) -> String {
    if byte.is_ascii_graphic() {
        format!("{:?}", byte as char)
    } else {
        format!("the byte {byte:#04x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn last_digit_of(text: &str) -> Option<LastDigit> {
        match Tokens::over(text.as_bytes()).take() {
            Ok((_, Token::Real(_, last))) => last,
            other => panic!("{text} is a real, and was read as {other:?}"),
        }
    }

    #[test]
    fn a_real_keeps_the_place_of_the_last_digit_it_was_written_to() {
        assert_eq!(last_digit_of("0.463647609001"), Some(LastDigit(-12)));
        assert_eq!(last_digit_of("4.63647609001E-01"), Some(LastDigit(-12)));
        assert_eq!(last_digit_of("0.4636476090010"), Some(LastDigit(-13)));
        assert_eq!(last_digit_of("-2.5E-10"), Some(LastDigit(-11)));
        assert_eq!(last_digit_of("1."), Some(LastDigit(0)));
        assert_eq!(last_digit_of("1.E+03"), Some(LastDigit(3)));
        assert_eq!(last_digit_of("100."), Some(LastDigit(0)));
    }

    #[test]
    fn a_real_whose_last_digit_is_past_an_i32_keeps_no_place() {
        assert_eq!(last_digit_of("0.E+9999999999"), None);
        assert_eq!(last_digit_of("0.E-2147483648"), Some(LastDigit(i32::MIN)));
        assert_eq!(last_digit_of("0.0E-2147483648"), None);
    }
}
