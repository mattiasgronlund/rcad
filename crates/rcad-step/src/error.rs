use crate::graph::EntityId;

/// What stopped the reader, and where in the file it stopped.
///
/// Every refusal carries a place, because the whole point of this reader is
/// that a caller can tell exactly what was in the file and what was not
/// (decisions.md §42.11).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{problem} (line {line}, column {column})")]
pub struct Error {
    /// What went wrong.
    pub problem: Problem,
    /// The line it happened on, counting from one.
    pub line: usize,
    /// How far along that line it happened, counting bytes from one.
    pub column: usize,
}

/// What the reader could not read.
///
/// Nothing here is repaired, guessed at, or replaced by a default. A file that
/// trips one of these is refused by a name that says which entity and why.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Problem {
    /// The file does not begin with `ISO-10303-21;`.
    #[error("this does not begin with ISO-10303-21;, so it is not a Part 21 file")]
    NotAPart21File,

    /// The file ran out in the middle of something.
    #[error("the file stops before its {wanted}")]
    FileStopsEarly {
        /// What the reader was still waiting for.
        wanted: &'static str,
    },

    /// Something else stood where the grammar wanted one particular thing.
    #[error("expected {wanted}, found {found}")]
    NotWhatWasExpected {
        /// What the grammar allows here.
        wanted: &'static str,
        /// What the file has instead.
        found: String,
    },

    /// A section keyword this reader does not know, such as Part 21 edition 3's
    /// `ANCHOR`, `REFERENCE` or `SIGNATURE`.
    #[error("this file has a {keyword} section, which this reader does not read")]
    SectionThisReaderDoesNotRead {
        /// The section keyword, as the file wrote it.
        keyword: String,
    },

    /// A quoted string was opened and the file ended before it closed.
    #[error("a string is opened and never closed")]
    StringNeverEnds,

    /// A `/*` comment was opened and the file ended before `*/`.
    #[error("a comment is opened and never closed")]
    CommentNeverEnds,

    /// A `\` directive inside a string that Part 21 does not define, or one
    /// that is cut short.
    #[error("a string holds the escape {escape}, which is not one this reader knows")]
    EscapeThisReaderDoesNotKnow {
        /// The escape as far as the reader could read it.
        escape: String,
    },

    /// A string holds bytes that are not text: raw bytes above 127 that are not
    /// valid UTF-8.
    ///
    /// Part 21 editions 1 and 2 say a string outside plain ASCII is written
    /// with `\X\`, `\X2\` or `\S\` escapes; edition 3 allows UTF-8 directly.
    /// Anything else would have to be guessed at, and this reader does not
    /// guess.
    #[error("a string holds bytes that are neither ASCII nor UTF-8")]
    StringIsNotText,

    /// A `\P` alphabet switch to a page other than `A` (ISO 8859-1).
    #[error("a string switches to alphabet page {page}, and this reader only has page A")]
    AlphabetThisReaderDoesNotHave {
        /// The page letter the file asked for.
        page: char,
    },

    /// An entity id that does not fit in a `u64`.
    #[error("the entity id #{text} is too big to hold")]
    EntityIdTooLarge {
        /// The digits, as the file wrote them.
        text: String,
    },

    /// An integer that does not fit in an `i64`.
    #[error("the integer {text} is too big to hold")]
    IntegerTooLarge {
        /// The number, as the file wrote them.
        text: String,
    },

    /// A real whose value in `f64` is infinite, so the file asked for a number
    /// this reader cannot hold rather than a number it rounded.
    #[error("the real {text} is too big to hold in an f64")]
    RealTooLarge {
        /// The number, as the file wrote it.
        text: String,
    },

    /// A number written with an exponent but no decimal point, such as `1E3`.
    ///
    /// Part 21 has no such literal: an integer has no exponent and a real must
    /// carry a `.` before its `E`.
    #[error("{text} has an exponent but no decimal point, which Part 21 has no literal for")]
    ExponentWithoutADecimalPoint {
        /// The number, as the file wrote it.
        text: String,
    },

    /// A keyword with a lower-case letter in it. Part 21 keywords are upper
    /// case, and folding the case would be a guess about which entity is meant.
    #[error("the keyword {keyword} is not upper case, and Part 21 keywords are")]
    KeywordIsNotUpperCase {
        /// The keyword, as the file wrote it.
        keyword: String,
    },

    /// A binary literal, `"0F3A..."`.
    ///
    /// No entity this reader's consumer walks carries one, so there is nothing
    /// to hand a decoded value to (decisions.md §13 rule 3). It is refused by
    /// name rather than skipped.
    #[error("this file holds a binary value, which this reader does not read")]
    BinaryValueThisReaderDoesNotRead,

    /// Lists, or values wearing type names, nested deeper than
    /// [`DEEPEST_NESTING`](crate::DEEPEST_NESTING).
    ///
    /// The reader walks nested values with the call stack, so the limit is what
    /// stands between a hostile file and a stack overflow, which would be a
    /// crash and not an `Err`.
    #[error("values are nested more than {limit} deep")]
    NestedTooDeeply {
        /// The limit that was passed.
        limit: usize,
    },

    /// Two instances in the file claim the same id. Picking one would be a
    /// guess about which the references mean.
    #[error("{id} is given to two different instances")]
    EntityIdUsedTwice {
        /// The id both instances claim.
        id: EntityId,
    },

    /// A complex instance written `#1=();`, with no parts in it.
    #[error("{id} is a complex instance with nothing in it")]
    ComplexInstanceHasNoParts {
        /// The instance's id.
        id: EntityId,
    },

    /// A `#123` reference in the header section, where Part 21 has no entities
    /// for it to point at.
    #[error("the header mentions {id}, and the header has no entities to point at")]
    ReferenceInTheHeader {
        /// The id the header mentioned.
        id: EntityId,
    },

    /// One of the three header entities Part 21 requires is not there.
    #[error("the header has no {keyword}, which Part 21 requires")]
    HeaderEntityMissing {
        /// Which one: `FILE_DESCRIPTION`, `FILE_NAME` or `FILE_SCHEMA`.
        keyword: &'static str,
    },

    /// `FILE_SCHEMA` does not hold one list of strings.
    #[error("FILE_SCHEMA does not hold one list of schema names")]
    SchemaNamesAreNotAListOfStrings,

    /// An entity mentions an id the file never defines.
    ///
    /// Raised by [`StepFile::every_reference_resolves`](crate::StepFile::every_reference_resolves),
    /// never during the parse: whether a dangling reference in a part of the
    /// file you never walk should stop you is the caller's decision, and every
    /// one of them is listed by
    /// [`StepFile::references_that_point_nowhere`](crate::StepFile::references_that_point_nowhere).
    #[error("{from} mentions {to}, which this file never defines")]
    ReferencePointsNowhere {
        /// The entity that holds the reference.
        from: EntityId,
        /// The id it points at.
        to: EntityId,
    },
}
