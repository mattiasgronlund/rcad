use std::collections::HashMap;

use crate::DEEPEST_NESTING;
use crate::error::{Error, Problem};
use crate::graph::{BrokenReference, EntityId, Header, Instance, Record, StepFile, Value};
use crate::token::{Token, Tokens, error_at};

const OPENING: &[u8] = b"ISO-10303-21";
const CLOSING: &[u8] = b"END-ISO-10303-21";

/// The three header entities ISO 10303-21 requires of every file.
const REQUIRED_HEADER_ENTITIES: [&str; 3] = ["FILE_DESCRIPTION", "FILE_NAME", "FILE_SCHEMA"];

/// Which section a value is being read in.
///
/// Only one thing turns on it — a `#123` is a reference in a data section and
/// has nothing to point at in a header — but that one thing is a hole if it is
/// not checked where the values are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Header,
    Data,
}

pub(crate) fn read(bytes: &[u8]) -> Result<StepFile, Error> {
    let mut tokens = Tokens::over(bytes);
    opening_line(&mut tokens)?;

    let mut header = None;
    let mut instances = Vec::new();
    loop {
        let (_, left) = tokens.rest()?;
        if left.starts_with(CLOSING) {
            tokens.step_over(CLOSING.len());
            tokens.take_exactly(Token::Semicolon, "; after END-ISO-10303-21")?;
            break;
        }
        let (at, opener) = tokens.take()?;
        let Token::Keyword(section) = opener else {
            return Err(tokens.fault(
                at,
                Problem::NotWhatWasExpected {
                    wanted: "HEADER, DATA or END-ISO-10303-21",
                    found: opener.describe(),
                },
            ));
        };
        match section {
            "HEADER" => {
                tokens.take_exactly(Token::Semicolon, "; after HEADER")?;
                header = Some(header_section(&mut tokens, at)?);
            }
            "DATA" => {
                tokens.take_exactly(Token::Semicolon, "; after DATA")?;
                data_section(&mut tokens, &mut instances)?;
            }
            other => {
                return Err(tokens.fault(
                    at,
                    Problem::SectionThisReaderDoesNotRead {
                        keyword: other.into(),
                    },
                ));
            }
        }
    }
    let (at, left) = tokens.rest()?;
    if !left.is_empty() {
        return Err(tokens.fault(
            at,
            Problem::NotWhatWasExpected {
                wanted: "nothing after END-ISO-10303-21;",
                found: String::from_utf8_lossy(&left[..left.len().min(8)]).into_owned(),
            },
        ));
    }

    let Some(header) = header else {
        return Err(error_at(
            bytes,
            0,
            Problem::HeaderEntityMissing {
                keyword: "HEADER section",
            },
        ));
    };
    let slot_of_id = index(bytes, &instances)?;
    let broken = references_that_point_nowhere(bytes, &instances, &slot_of_id);
    Ok(StepFile {
        header,
        instances,
        slot_of_id,
        broken,
    })
}

fn opening_line(tokens: &mut Tokens<'_>) -> Result<(), Error> {
    let (at, left) = tokens.rest()?;
    if !left.starts_with(OPENING) {
        return Err(tokens.fault(at, Problem::NotAPart21File));
    }
    tokens.step_over(OPENING.len());
    tokens.take_exactly(Token::Semicolon, "; after ISO-10303-21")
}

fn header_section(tokens: &mut Tokens<'_>, at: usize) -> Result<Header, Error> {
    let mut records = Vec::new();
    while !section_ends(tokens)? {
        records.push(one_record(tokens, Section::Header)?);
        tokens.take_exactly(Token::Semicolon, "; after a header entity")?;
    }
    for keyword in REQUIRED_HEADER_ENTITIES {
        if !records.iter().any(|record| record.keyword == keyword) {
            return Err(tokens.fault(at, Problem::HeaderEntityMissing { keyword }));
        }
    }
    let schemas = schema_names(tokens, at, &records)?;
    Ok(Header { records, schemas })
}

fn schema_names(tokens: &Tokens<'_>, at: usize, records: &[Record]) -> Result<Vec<String>, Error> {
    let file_schema = records
        .iter()
        .find(|record| record.keyword == "FILE_SCHEMA")
        .expect("FILE_SCHEMA was just checked to be there");
    let wrong = || tokens.fault(at, Problem::SchemaNamesAreNotAListOfStrings);
    let [Value::List(names)] = file_schema.attributes.as_slice() else {
        return Err(wrong());
    };
    names
        .iter()
        .map(|name| match name.as_text() {
            Some(text) => Ok(text.to_string()),
            None => Err(wrong()),
        })
        .collect()
}

fn data_section(tokens: &mut Tokens<'_>, instances: &mut Vec<Instance>) -> Result<(), Error> {
    while !section_ends(tokens)? {
        instances.push(one_instance(tokens)?);
        tokens.take_exactly(Token::Semicolon, "; after an instance")?;
    }
    Ok(())
}

/// Takes `ENDSEC;` if that is what stands here, and says whether it did.
fn section_ends(tokens: &mut Tokens<'_>) -> Result<bool, Error> {
    let (at, next) = tokens.peek()?;
    let at = *at;
    let ends = matches!(next, Token::Keyword(word) if *word == "ENDSEC");
    let ran_out = matches!(next, Token::End);
    if ran_out {
        return Err(tokens.fault(
            at,
            Problem::FileStopsEarly {
                wanted: "section's ENDSEC;",
            },
        ));
    }
    if ends {
        tokens.take()?;
        tokens.take_exactly(Token::Semicolon, "; after ENDSEC")?;
    }
    Ok(ends)
}

fn one_instance(tokens: &mut Tokens<'_>) -> Result<Instance, Error> {
    let (at, first) = tokens.take()?;
    let Token::Reference(id) = first else {
        return Err(tokens.fault(
            at,
            Problem::NotWhatWasExpected {
                wanted: "#id at the start of an instance",
                found: first.describe(),
            },
        ));
    };
    tokens.take_exactly(Token::Equals, "= after an instance's #id")?;

    let mut parts = Vec::new();
    if tokens.peek()?.1 == Token::Open {
        tokens.take()?;
        while tokens.peek()?.1 != Token::Close {
            parts.push(one_record(tokens, Section::Data)?);
        }
        tokens.take()?;
        if parts.is_empty() {
            return Err(tokens.fault(at, Problem::ComplexInstanceHasNoParts { id }));
        }
        if let Some(twice) = a_type_written_twice(&parts) {
            return Err(tokens.fault(
                at,
                Problem::ComplexInstanceRepeatsAType {
                    id,
                    keyword: twice.into(),
                },
            ));
        }
    } else {
        parts.push(one_record(tokens, Section::Data)?);
    }
    Ok(Instance { id, parts, at })
}

/// The first type name a complex instance gives twice, if it gives one twice.
///
/// Quadratic in the number of parts, which is the number of types one entity
/// inherits from — a handful, and bounded by the schema rather than by the
/// file's size.
fn a_type_written_twice(parts: &[Record]) -> Option<&str> {
    parts.iter().enumerate().find_map(|(position, part)| {
        parts[..position]
            .iter()
            .any(|earlier| earlier.keyword == part.keyword)
            .then_some(part.keyword.as_str())
    })
}

fn one_record(tokens: &mut Tokens<'_>, section: Section) -> Result<Record, Error> {
    let (at, first) = tokens.take()?;
    let Token::Keyword(keyword) = first else {
        return Err(tokens.fault(
            at,
            Problem::NotWhatWasExpected {
                wanted: "an entity type name",
                found: first.describe(),
            },
        ));
    };
    tokens.take_exactly(Token::Open, "( after an entity type name")?;
    let attributes = items(tokens, 1, section)?;
    Ok(Record {
        keyword: keyword.into(),
        attributes,
    })
}

/// The values between a `(` already taken and the `)` this takes.
fn items(tokens: &mut Tokens<'_>, depth: usize, section: Section) -> Result<Vec<Value>, Error> {
    if tokens.peek()?.1 == Token::Close {
        tokens.take()?;
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    loop {
        items.push(one_value(tokens, depth, section)?);
        let (at, next) = tokens.take()?;
        if next == Token::Close {
            return Ok(items);
        }
        if next != Token::Comma {
            return Err(tokens.fault(
                at,
                Problem::NotWhatWasExpected {
                    wanted: ", or ) after a value",
                    found: next.describe(),
                },
            ));
        }
    }
}

fn one_value(tokens: &mut Tokens<'_>, depth: usize, section: Section) -> Result<Value, Error> {
    if depth > DEEPEST_NESTING {
        let at = tokens.peek()?.0;
        return Err(tokens.fault(
            at,
            Problem::NestedTooDeeply {
                limit: DEEPEST_NESTING,
            },
        ));
    }
    let (at, token) = tokens.take()?;
    match token {
        Token::Integer(number) => Ok(Value::Integer(number)),
        Token::Real(number) => Ok(Value::Real(number)),
        Token::Text(text) => Ok(Value::Text(text)),
        Token::Enumeration(name) => Ok(Value::Enumeration(name.into())),
        Token::Nothing => Ok(Value::Nothing),
        Token::Derived => Ok(Value::Derived),
        Token::Reference(id) => match section {
            Section::Data => Ok(Value::Reference(id)),
            Section::Header => Err(tokens.fault(at, Problem::ReferenceInTheHeader { id })),
        },
        Token::Open => Ok(Value::List(items(tokens, depth + 1, section)?)),
        Token::Keyword(keyword) => {
            tokens.take_exactly(Token::Open, "( after a type name wrapping a value")?;
            let inside = one_value(tokens, depth + 1, section)?;
            tokens.take_exactly(Token::Close, ") after a type name wrapping a value")?;
            Ok(Value::Typed {
                keyword: keyword.into(),
                value: Box::new(inside),
            })
        }
        found @ (Token::Close | Token::Comma | Token::Semicolon | Token::Equals | Token::End) => {
            Err(tokens.fault(
                at,
                Problem::NotWhatWasExpected {
                    wanted: "a value",
                    found: found.describe(),
                },
            ))
        }
    }
}

fn index(bytes: &[u8], instances: &[Instance]) -> Result<HashMap<EntityId, usize>, Error> {
    let mut slot_of_id = HashMap::with_capacity(instances.len());
    for (slot, instance) in instances.iter().enumerate() {
        if slot_of_id.insert(instance.id, slot).is_some() {
            return Err(error_at(
                bytes,
                instance.at,
                Problem::EntityIdUsedTwice { id: instance.id },
            ));
        }
    }
    Ok(slot_of_id)
}

fn references_that_point_nowhere(
    bytes: &[u8],
    instances: &[Instance],
    slot_of_id: &HashMap<EntityId, usize>,
) -> Vec<BrokenReference> {
    let mut broken = Vec::new();
    let mut place = Place::over(bytes);
    for instance in instances {
        let mut missing = Vec::new();
        for part in &instance.parts {
            for attribute in &part.attributes {
                gather_missing(attribute, slot_of_id, &mut missing);
            }
        }
        if missing.is_empty() {
            continue;
        }
        let (line, column) = place.of(instance.at);
        broken.extend(missing.into_iter().map(|to| BrokenReference {
            from: instance.id,
            to,
            line,
            column,
        }));
    }
    broken
}

fn gather_missing(
    value: &Value,
    slot_of_id: &HashMap<EntityId, usize>,
    missing: &mut Vec<EntityId>,
) {
    match value {
        Value::Reference(id) if !slot_of_id.contains_key(id) => missing.push(*id),
        Value::List(items) => {
            for item in items {
                gather_missing(item, slot_of_id, missing);
            }
        }
        Value::Typed { value, .. } => gather_missing(value, slot_of_id, missing),
        Value::Reference(_)
        | Value::Integer(_)
        | Value::Real(_)
        | Value::Text(_)
        | Value::Enumeration(_)
        | Value::Nothing
        | Value::Derived => {}
    }
}

/// Turns byte offsets into line and column numbers, walking the file once.
///
/// Offsets arrive in the order the file wrote the instances, which is
/// increasing, so this is linear where asking for each one separately would be
/// quadratic in a file with a lot of broken references.
struct Place<'a> {
    bytes: &'a [u8],
    at: usize,
    line: usize,
    start_of_line: usize,
}

impl<'a> Place<'a> {
    fn over(bytes: &'a [u8]) -> Self {
        Place {
            bytes,
            at: 0,
            line: 1,
            start_of_line: 0,
        }
    }

    fn of(&mut self, wanted: usize) -> (usize, usize) {
        assert!(wanted >= self.at, "offsets must arrive in increasing order");
        while self.at < wanted {
            if self.bytes[self.at] == b'\n' {
                self.line += 1;
                self.start_of_line = self.at + 1;
            }
            self.at += 1;
        }
        (self.line, 1 + wanted - self.start_of_line)
    }
}
