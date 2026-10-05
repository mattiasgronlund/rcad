use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

use crate::error::{Error, Problem};
use crate::read;
use crate::token::LastDigit;

/// A whole Part 21 file: its header, its instances, and where every `#id`
/// points.
#[derive(Debug, Clone)]
pub struct StepFile {
    pub(crate) header: Header,
    pub(crate) instances: Vec<Instance>,
    pub(crate) slot_of_id: HashMap<EntityId, usize>,
    pub(crate) broken: Vec<BrokenReference>,
}

impl StepFile {
    /// Read a file's bytes.
    ///
    /// Takes bytes rather than a `&str` because a Part 21 file is somebody
    /// else's, and "it is valid UTF-8" is not something a caller can promise on
    /// its behalf. Only the inside of a quoted string may be non-ASCII, and
    /// there it must be UTF-8 or one of Part 21's own escapes.
    ///
    /// Succeeding here means the file's *syntax* was read whole. It does not
    /// mean every reference resolves — see
    /// [`references_that_point_nowhere`](Self::references_that_point_nowhere).
    pub fn read(bytes: &[u8]) -> Result<Self, Error> {
        read::read(bytes)
    }

    /// The header section.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Every instance, in the order the file wrote them.
    ///
    /// A plain [`std::slice::Iter`], not `impl Iterator` — the standard
    /// library's own type, so a caller can write it down: hold it in a
    /// struct field, pass it on, or return it from a function of their own.
    pub fn instances(&self) -> std::slice::Iter<'_, Instance> {
        self.instances.iter()
    }

    /// What a `#id` points at, or `None` if the file never defines it.
    ///
    /// `None` happens only for an id in
    /// [`references_that_point_nowhere`](Self::references_that_point_nowhere).
    pub fn instance(&self, id: EntityId) -> Option<&Instance> {
        self.slot_of_id.get(&id).map(|&slot| &self.instances[slot])
    }

    /// Every instance the file wrote under a given type name.
    ///
    /// Each matching instance is yielded once, including a complex one that
    /// wears the name among several others. [`InstancesNamed`]'s own doc
    /// says why it is a struct and not one of two cheaper-looking
    /// alternatives.
    pub fn instances_named<'a>(&'a self, keyword: &'a str) -> InstancesNamed<'a> {
        InstancesNamed {
            keyword,
            remaining: self.instances.iter(),
        }
    }

    /// How many instances the file holds under each type name.
    ///
    /// This is the census the mapping stage subtracts from: everything it does
    /// not map is an entity type nobody read, and this says how many of them
    /// there were. A complex instance counts once under each of its parts.
    pub fn how_many_of_each_type(&self) -> BTreeMap<&str, usize> {
        let mut counts = BTreeMap::new();
        for instance in &self.instances {
            for part in &instance.parts {
                *counts.entry(part.keyword.as_str()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Every `#id` an entity mentions that the file never defines.
    ///
    /// Listed rather than repaired. An id that appears in two entities appears
    /// twice here, once for each, because which entity was reading it is half
    /// of what a reader needs to be told.
    pub fn references_that_point_nowhere(&self) -> &[BrokenReference] {
        &self.broken
    }

    /// Refuse the file if any reference points nowhere, naming the first pair.
    ///
    /// Separate from [`read`](Self::read) so that a caller who never walks the
    /// part of the file that is broken can say so.
    pub fn every_reference_resolves(&self) -> Result<(), Error> {
        match self.broken.first() {
            None => Ok(()),
            Some(broken) => Err(Error {
                problem: Problem::ReferencePointsNowhere {
                    from: broken.from,
                    to: broken.to,
                },
                line: broken.line,
                column: broken.column,
            }),
        }
    }
}

/// Every instance the file wrote under one type name, built by
/// [`StepFile::instances_named`].
///
/// A named struct rather than `impl Iterator`, so a caller can write this
/// type down. [`std::iter::Filter`] is itself a named type, but only when
/// what it wraps a closure in is too — a closure's own type has no name at
/// all, so `Filter<std::slice::Iter<'a, Instance>, /* what goes here? */>`
/// is not writable either. Three ways to make the whole thing nameable, and
/// what each costs against what this struct does not:
///
/// - `Box<dyn Iterator<Item = &'a Instance> + 'a>` — one heap allocation to
///   build it, one virtual call per [`Iterator::next`].
/// - A collected [`Vec<&'a Instance>`](Vec) — every matching instance found
///   up front, whether the caller wants one of them or all. `Import::read`
///   asks for `"BREP_WITH_VOIDS"` only to call [`Iterator::next`] once, and a
///   `Vec` would have walked the rest of the file to build the entries that
///   call throws away.
/// - This: one reference and one borrowed `&str`, nothing allocated, and the
///   same lazy walk stopping at the first match a plain `.filter()` already
///   did before this struct existed.
#[derive(Debug, Clone)]
pub struct InstancesNamed<'a> {
    keyword: &'a str,
    remaining: std::slice::Iter<'a, Instance>,
}

impl<'a> Iterator for InstancesNamed<'a> {
    type Item = &'a Instance;

    fn next(&mut self) -> Option<&'a Instance> {
        self.remaining
            .find(|instance| instance.part(self.keyword).is_some())
    }
}

/// The `HEADER` section.
#[derive(Debug, Clone)]
pub struct Header {
    pub(crate) records: Vec<Record>,
    pub(crate) schemas: Vec<String>,
}

impl Header {
    /// Every header entity, in the order the file wrote them.
    ///
    /// `FILE_DESCRIPTION`, `FILE_NAME` and `FILE_SCHEMA` are always among them
    /// — a file missing one of the three is refused. Files carry others too,
    /// and this reader keeps them without knowing what they mean.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// The first header entity written under a type name.
    pub fn record(&self, keyword: &str) -> Option<&Record> {
        self.records.iter().find(|record| record.keyword == keyword)
    }

    /// The schema names from `FILE_SCHEMA`, such as
    /// `AUTOMOTIVE_DESIGN { 1 0 10303 214 ... }`.
    ///
    /// This is what says whether a file is AP203, AP214 or AP242, which decides
    /// which entities the mapping stage should expect to find.
    pub fn schemas(&self) -> &[String] {
        &self.schemas
    }
}

/// One `#123 = ...;` in a data section.
#[derive(Debug, Clone)]
pub struct Instance {
    pub(crate) id: EntityId,
    pub(crate) parts: Vec<Record>,
    /// Where the `#` starts, so a refusal about this instance can say where it
    /// is without the reader keeping a place on every value.
    pub(crate) at: usize,
}

impl Instance {
    /// The `#123` this instance was written under.
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// The type names and attributes this instance is made of.
    ///
    /// A simple instance, `#1=ADVANCED_FACE(...)`, has exactly one part. A
    /// complex instance, `#1=(A(...) B(...))`, has one per type it inherits
    /// from, in the order the file wrote them.
    pub fn parts(&self) -> &[Record] {
        &self.parts
    }

    /// The part written under a given type name, if this instance has one.
    ///
    /// For a simple instance this asks "is it an `ADVANCED_FACE`?". For a
    /// complex one it asks "is an `ADVANCED_FACE` one of the things it is?",
    /// which is the question the mapping stage wants either way. There is never
    /// more than one part to choose between: a complex instance that names a
    /// type twice is refused when it is read.
    pub fn part(&self, keyword: &str) -> Option<&Record> {
        self.parts.iter().find(|part| part.keyword == keyword)
    }
}

/// A type name and the attributes written after it.
#[derive(Debug, Clone)]
pub struct Record {
    pub(crate) keyword: String,
    pub(crate) attributes: Vec<Value>,
    /// Where each attribute that is a real, or wraps one once, was written to
    /// its last digit, by position. [`Value::Real`] is the number and nothing
    /// else, and a real inside a list keeps no place.
    pub(crate) last_digits: Box<[(usize, LastDigit)]>,
}

impl Record {
    /// The type name, exactly as the file wrote it.
    pub fn keyword(&self) -> &str {
        &self.keyword
    }

    /// The attributes, in the order the file wrote them.
    ///
    /// Their number and order are the file's, not the schema's: this reader
    /// does not know how many attributes an `ADVANCED_FACE` should have and
    /// does not pad or truncate to make one fit.
    pub fn attributes(&self) -> &[Value] {
        &self.attributes
    }

    /// One attribute by position, or `None` if the file wrote fewer than that.
    pub fn attribute(&self, position: usize) -> Option<&Value> {
        self.attributes.get(position)
    }

    /// The place of the last digit the attribute at `position` was written
    /// to, where it is a real.
    pub(crate) fn last_digit_of(&self, position: usize) -> Option<LastDigit> {
        self.last_digits
            .iter()
            .find(|(at, _)| *at == position)
            .map(|(_, last)| *last)
    }
}

/// One attribute value.
///
/// The variants are Part 21's own, kept apart the way the file kept them. In
/// particular [`Integer`](Value::Integer) and [`Real`](Value::Real) are two
/// different things: `1` and `1.` are different literals, files written for a
/// schema that wants a real do sometimes write the integer, and turning one
/// into the other here would be the reader deciding something the file did not
/// say.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `1`, `-4`, `+7`.
    Integer(i64),

    /// `1.`, `-4.5`, `1.E+03`. Always finite.
    Real(f64),

    /// `'a string'`, with `''` and every `\` escape already decoded.
    Text(String),

    /// `.T.`, `.F.`, `.UNKNOWN.`, `.MILLI.` — the name without its dots.
    ///
    /// Booleans and logicals are enumerations too, and stay that way: this
    /// reader does not know which attributes are booleans.
    Enumeration(String),

    /// `#123`.
    Reference(EntityId),

    /// `(...)`, of anything, including nothing.
    List(Vec<Value>),

    /// A value wearing a type name, `LENGTH_MEASURE(25.4)`.
    ///
    /// STEP writes these where an attribute could be one of several types.
    Typed {
        /// The type name, exactly as the file wrote it.
        keyword: String,
        /// What it wraps.
        value: Box<Value>,
    },

    /// `$` — the file says this attribute has no value.
    Nothing,

    /// `*` — the file says this attribute is worked out from the others.
    Derived,
}

impl Value {
    /// The number, if this is a [`Real`](Value::Real).
    ///
    /// An [`Integer`](Value::Integer) is not a real here. A file that writes
    /// `0` where its schema wants `0.` has written something else, and it is
    /// the mapping stage's business whether to accept it, not this reader's.
    pub fn as_real(&self) -> Option<f64> {
        match self {
            Value::Real(number) => Some(*number),
            Value::Integer(_)
            | Value::Text(_)
            | Value::Enumeration(_)
            | Value::Reference(_)
            | Value::List(_)
            | Value::Typed { .. }
            | Value::Nothing
            | Value::Derived => None,
        }
    }

    /// The decoded characters, if this is a [`Text`](Value::Text).
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(text) => Some(text),
            Value::Integer(_)
            | Value::Real(_)
            | Value::Enumeration(_)
            | Value::Reference(_)
            | Value::List(_)
            | Value::Typed { .. }
            | Value::Nothing
            | Value::Derived => None,
        }
    }

    /// The name between the dots, if this is an
    /// [`Enumeration`](Value::Enumeration).
    pub fn as_enumeration(&self) -> Option<&str> {
        match self {
            Value::Enumeration(name) => Some(name),
            Value::Integer(_)
            | Value::Real(_)
            | Value::Text(_)
            | Value::Reference(_)
            | Value::List(_)
            | Value::Typed { .. }
            | Value::Nothing
            | Value::Derived => None,
        }
    }

    /// The id, if this is a [`Reference`](Value::Reference).
    pub fn as_reference(&self) -> Option<EntityId> {
        match self {
            Value::Reference(id) => Some(*id),
            Value::Integer(_)
            | Value::Real(_)
            | Value::Text(_)
            | Value::Enumeration(_)
            | Value::List(_)
            | Value::Typed { .. }
            | Value::Nothing
            | Value::Derived => None,
        }
    }

    /// The items, if this is a [`List`](Value::List).
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(items) => Some(items),
            Value::Integer(_)
            | Value::Real(_)
            | Value::Text(_)
            | Value::Enumeration(_)
            | Value::Reference(_)
            | Value::Typed { .. }
            | Value::Nothing
            | Value::Derived => None,
        }
    }
}

/// The `#123` an instance is written under.
///
/// Part 21 puts no ceiling on the digits; this reader holds them in a `u64` and
/// refuses anything longer by name rather than wrapping it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u64);

impl fmt::Display for EntityId {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "#{}", self.0)
    }
}

/// An entity that mentions an id the file never defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrokenReference {
    /// The entity holding the reference.
    pub from: EntityId,
    /// The id it points at, which nothing defines.
    pub to: EntityId,
    /// The line the reference is written on, counting from one.
    pub line: usize,
    /// How far along that line, counting bytes from one.
    pub column: usize,
}
