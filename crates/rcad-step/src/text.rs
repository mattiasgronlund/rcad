use crate::error::{Error, Problem};
use crate::token::error_at;

/// Turns the inside of a `'...'` into the characters it stands for.
///
/// `body` is the bytes between the quotes, `body_at` is where they start in
/// `whole`, which is the file, and is only there so a refusal can say where.
///
/// Part 21 writes anything outside plain ASCII as an escape: `\X\` for one
/// octet, `\X2\` and `\X4\` for runs of wider code points, `\S\` for the upper
/// half of an 8-bit page. Edition 3 also allows UTF-8 straight through, so that
/// is accepted too. Nothing else is, and nothing is guessed at.
pub(crate) fn decode(body: &[u8], body_at: usize, whole: &[u8]) -> Result<String, Error> {
    let mut out = String::with_capacity(body.len());
    let mut at = 0;
    while at < body.len() {
        let byte = body[at];
        match byte {
            // The scanner ended the string at the first apostrophe not
            // doubled, so every one it left inside is the first of a pair —
            // provided nothing before it stepped over only half of one. Of the
            // directives, `\S\` is the only one that swallows a character it
            // does not otherwise constrain, and it counts the pair.
            b'\'' => {
                assert_eq!(
                    body.get(at + 1),
                    Some(&b'\''),
                    "a lone apostrophe inside a string the scanner delimited"
                );
                out.push('\'');
                at += 2;
            }
            b'\\' => at = escape(body, at, body_at, whole, &mut out)?,
            0x00..=0x7f => {
                out.push(char::from(byte));
                at += 1;
            }
            _ => at = utf8(body, at, body_at, whole, &mut out)?,
        }
    }
    Ok(out)
}

/// Reads one `\` directive, appends what it stands for, and says where it ended.
fn escape(
    body: &[u8],
    at: usize,
    body_at: usize,
    whole: &[u8],
    out: &mut String,
) -> Result<usize, Error> {
    match body.get(at + 1) {
        Some(b'\\') => {
            out.push('\\');
            Ok(at + 2)
        }
        // `\S\c` is the character c moved into the upper half of the current
        // 8-bit page, which is ISO 8859-1 unless `\P` said otherwise, and `\P`
        // saying otherwise is refused below.
        //
        // c is a character of the string and not a raw byte, so a doubled
        // apostrophe there is one apostrophe and both bytes belong to this
        // directive. Reading only the first is how the fuzzer found a panic:
        // the rest of the decoder then met the second on its own, where the
        // string's own grammar says one cannot stand.
        Some(b'S') if body.get(at + 2) == Some(&b'\\') => match body.get(at + 3) {
            Some(&letter) if letter.is_ascii() => {
                out.push(char::from(letter | 0x80));
                Ok(at + 3 + how_wide_one_character_is(body, at + 3))
            }
            _ => Err(unknown_escape(body, at, body_at, whole)),
        },
        Some(b'P') if body.get(at + 3) == Some(&b'\\') => match body.get(at + 2) {
            Some(b'A') => Ok(at + 4),
            Some(&page) if page.is_ascii_uppercase() => Err(error_at(
                whole,
                body_at + at,
                Problem::AlphabetThisReaderDoesNotHave {
                    page: char::from(page),
                },
            )),
            _ => Err(unknown_escape(body, at, body_at, whole)),
        },
        Some(b'X') => match body.get(at + 2) {
            Some(b'\\') => one_octet(body, at, body_at, whole, out),
            Some(b'2') if body.get(at + 3) == Some(&b'\\') => {
                wide(body, at, body_at, whole, out, 4)
            }
            Some(b'4') if body.get(at + 3) == Some(&b'\\') => {
                wide(body, at, body_at, whole, out, 8)
            }
            _ => Err(unknown_escape(body, at, body_at, whole)),
        },
        _ => Err(unknown_escape(body, at, body_at, whole)),
    }
}

/// `\X\hh` — one octet, read as ISO 8859-1, which is what a Part 21 file
/// written without a `\P` page switch means by it.
fn one_octet(
    body: &[u8],
    at: usize,
    body_at: usize,
    whole: &[u8],
    out: &mut String,
) -> Result<usize, Error> {
    let Some(value) = hex(body, at + 3, 2) else {
        return Err(unknown_escape(body, at, body_at, whole));
    };
    out.push(char::from(value as u8));
    Ok(at + 5)
}

/// `\X2\hhhh…\X0\` and `\X4\hhhhhhhh…\X0\` — runs of wider code points.
fn wide(
    body: &[u8],
    at: usize,
    body_at: usize,
    whole: &[u8],
    out: &mut String,
    digits: usize,
) -> Result<usize, Error> {
    let mut read = at + 4;
    let mut units = Vec::new();
    loop {
        let Some(left) = body.get(read..) else {
            return Err(unknown_escape(body, at, body_at, whole));
        };
        if left.starts_with(br"\X0\") {
            read += 4;
            break;
        }
        let Some(value) = hex(body, read, digits) else {
            return Err(unknown_escape(body, at, body_at, whole));
        };
        units.push(value);
        read += digits;
    }
    if digits == 4 {
        let sixteens = units.iter().map(|&value| value as u16);
        for character in char::decode_utf16(sixteens) {
            let Ok(character) = character else {
                return Err(unknown_escape(body, at, body_at, whole));
            };
            out.push(character);
        }
    } else {
        for value in units {
            let Some(character) = char::from_u32(value) else {
                return Err(unknown_escape(body, at, body_at, whole));
            };
            out.push(character);
        }
    }
    Ok(read)
}

/// One UTF-8 character written straight into the file, which Part 21 edition 3
/// allows.
fn utf8(
    body: &[u8],
    at: usize,
    body_at: usize,
    whole: &[u8],
    out: &mut String,
) -> Result<usize, Error> {
    let length = match body[at] {
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => return Err(error_at(whole, body_at + at, Problem::StringIsNotText)),
    };
    let Some(run) = body.get(at..at + length) else {
        return Err(error_at(whole, body_at + at, Problem::StringIsNotText));
    };
    let Ok(text) = std::str::from_utf8(run) else {
        return Err(error_at(whole, body_at + at, Problem::StringIsNotText));
    };
    out.push_str(text);
    Ok(at + length)
}

/// How many bytes one character of a string takes at this spot: two for the
/// doubled apostrophe Part 21 writes one apostrophe as, one for anything else.
fn how_wide_one_character_is(body: &[u8], at: usize) -> usize {
    match body.get(at) {
        Some(b'\'') if body.get(at + 1) == Some(&b'\'') => 2,
        _ => 1,
    }
}

fn hex(body: &[u8], at: usize, digits: usize) -> Option<u32> {
    let run = body.get(at..at + digits)?;
    let mut value = 0u32;
    for &digit in run {
        value = value * 16 + char::from(digit).to_digit(16)?;
    }
    Some(value)
}

/// Names the escape as far as it could be read, cut short so a hostile file
/// cannot make the refusal the size of itself.
fn unknown_escape(body: &[u8], at: usize, body_at: usize, whole: &[u8]) -> Error {
    let end = (at + 8).min(body.len());
    let seen = String::from_utf8_lossy(&body[at..end]).into_owned();
    error_at(
        whole,
        body_at + at,
        Problem::EscapeThisReaderDoesNotKnow { escape: seen },
    )
}
