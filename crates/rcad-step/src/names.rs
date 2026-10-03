//! Every word the reader reads by name, written down once.
//!
//! `fuzz/step.dict` teaches the reader fuzzer these words, and the test at the
//! foot of this file holds the dictionary to exactly this list. The list is
//! what the reader reads rather than a copy of it: every [`Reading`] accessor
//! takes a [`Keyword`], every `match` on a record's type goes through
//! [`Keyword::of`], and only this module can make either kind of name — so a
//! word the reader starts to match has to be added here, and the test then asks
//! for it in the dictionary too.
//!
//! **The lists are built from the spellings and not from the constants**, so a
//! name listed here that the reader no longer reads leaves its constant unused
//! and `dead_code` fails the lint.
//!
//! **A file that matches on a constant must import it.** Without the import
//! `Some(CIRCLE)` is a new binding that matches everything, and the compiler
//! only warns — which `just lint` then refuses.
//!
//! What this cannot see: a string literal handed to the public
//! [`Instance::part`], [`StepFile::instances_named`] or [`Header::record`], or
//! compared with `==` against [`Record::keyword`], still compiles. The lexer's
//! punctuation and the string escapes `text.rs` decodes are bytes rather than
//! names and are not listed here either.
//!
//! [`Reading`]: crate::geometry::Reading
//! [`Instance::part`]: crate::Instance::part
//! [`StepFile::instances_named`]: crate::StepFile::instances_named
//! [`Header::record`]: crate::Header::record
//! [`Record::keyword`]: crate::Record::keyword

/// An entity type or a framing word, spelled the way a Part 21 file spells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Keyword(&'static str);

impl Keyword {
    pub(crate) const fn name(self) -> &'static str {
        self.0
    }

    /// The keyword `word` is, if the reader reads it by name at all.
    pub(crate) fn of(word: &str) -> Option<Self> {
        EVERY_KEYWORD
            .iter()
            .copied()
            .find(|keyword| keyword.0 == word)
    }
}

/// An enumeration value, spelled without the dots a Part 21 file writes round
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnumerationName(&'static str);

impl EnumerationName {
    /// The enumeration value `word` is, if the reader reads it by name at all.
    pub(crate) fn of(word: &str) -> Option<Self> {
        EVERY_ENUMERATION_NAME
            .iter()
            .copied()
            .find(|enumeration| enumeration.0 == word)
    }
}

/// One constant per name, and the list of every name from the same spellings.
macro_rules! names {
    ($kind:ident, $every:ident, [$($name:ident $(= $spelling:literal)?),* $(,)?]) => {
        $(pub(crate) const $name: $kind = $kind(names!(@spelled $name $($spelling)?));)*

        pub(crate) const $every: &[$kind] = &[$($kind(names!(@spelled $name $($spelling)?))),*];
    };
    (@spelled $name:ident $spelling:literal) => {
        $spelling
    };
    (@spelled $name:ident) => {
        stringify!($name)
    };
}

names!(
    Keyword,
    EVERY_KEYWORD,
    [
        ISO_10303_21 = "ISO-10303-21",
        END_ISO_10303_21 = "END-ISO-10303-21",
        HEADER,
        DATA,
        ENDSEC,
        FILE_DESCRIPTION,
        FILE_NAME,
        FILE_SCHEMA,
        AXIS2_PLACEMENT_3D,
        CARTESIAN_POINT,
        DIRECTION,
        PLANE,
        CYLINDRICAL_SURFACE,
        SPHERICAL_SURFACE,
        CONICAL_SURFACE,
        TOROIDAL_SURFACE,
        CIRCLE,
        ELLIPSE,
        LINE,
        MANIFOLD_SOLID_BREP,
        CLOSED_SHELL,
        ADVANCED_FACE,
        FACE_SURFACE,
        FACE_BOUND,
        FACE_OUTER_BOUND,
        EDGE_LOOP,
        ORIENTED_EDGE,
        EDGE_CURVE,
        VERTEX_POINT,
        VERTEX_LOOP,
        BREP_WITH_VOIDS,
        SHELL_BASED_SURFACE_MODEL,
        GEOMETRIC_CURVE_SET,
        GEOMETRIC_SET,
        TESSELLATED_GEOMETRIC_SET,
        TESSELLATED_CURVE_SET,
        COMPLEX_TRIANGULATED_SURFACE_SET,
        TRIANGULATED_SURFACE_SET,
        NEXT_ASSEMBLY_USAGE_OCCURRENCE,
        ITEM_DEFINED_TRANSFORMATION,
        MAPPED_ITEM,
        REPRESENTATION_MAP,
        LENGTH_UNIT,
        SI_UNIT,
        CONVERSION_BASED_UNIT,
        LENGTH_MEASURE_WITH_UNIT,
        MEASURE_WITH_UNIT,
        PLANE_ANGLE_UNIT,
        PLANE_ANGLE_MEASURE_WITH_UNIT,
        GLOBAL_UNIT_ASSIGNED_CONTEXT,
        GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT,
        UNCERTAINTY_MEASURE_WITH_UNIT,
    ]
);

names!(
    EnumerationName,
    EVERY_ENUMERATION_NAME,
    [TRUE = "T", FALSE = "F", METRE, MICRO, MILLI, CENTI, RADIAN,]
);

#[cfg(test)]
mod tests {
    use super::{EVERY_ENUMERATION_NAME, EVERY_KEYWORD};

    const STEP_DICT: &str = include_str!("../../../fuzz/step.dict");

    /// Every name the reader reads is in `fuzz/step.dict`, and every name the
    /// dictionary carries is one the reader reads.
    ///
    /// The dictionary's names are its entries longer than one character; the
    /// rest are the lexer's one-byte punctuation, which
    /// `tests/the_reader_fuzz_dictionary_names_what_the_reader_reads.rs` checks.
    /// Compared as sorted lists rather than sets, so a name written twice on
    /// either side shows too.
    #[test]
    fn the_fuzz_dictionary_carries_every_name_the_reader_reads_and_no_other() {
        let in_the_dictionary = sorted(names_in_the_dictionary());
        let read_by_name = sorted(
            EVERY_KEYWORD
                .iter()
                .map(|keyword| keyword.name().to_owned())
                .chain(
                    EVERY_ENUMERATION_NAME
                        .iter()
                        .map(|enumeration| format!(".{}.", enumeration.0)),
                )
                .collect(),
        );
        assert_eq!(
            in_the_dictionary,
            read_by_name,
            "fuzz/step.dict and crates/rcad-step/src/names.rs have drifted apart\n  \
             in the dictionary and not read by name: {:?}\n  \
             read by name and not in the dictionary: {:?}",
            left_over(&in_the_dictionary, &read_by_name),
            left_over(&read_by_name, &in_the_dictionary),
        );
    }

    fn names_in_the_dictionary() -> Vec<String> {
        STEP_DICT
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                line.strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
                    .unwrap_or_else(|| panic!("not a quoted libFuzzer dictionary token: {line}"))
            })
            .filter(|entry| entry.len() > 1)
            .map(str::to_owned)
            .collect()
    }

    fn sorted(mut names: Vec<String>) -> Vec<String> {
        names.sort_unstable();
        names
    }

    /// What is in `these` once `those` has taken one copy of each of its own
    /// names out.
    fn left_over(these: &[String], those: &[String]) -> Vec<String> {
        let mut unmatched = those.to_vec();
        let mut left = Vec::new();
        for name in these {
            match unmatched.iter().position(|other| other == name) {
                Some(at) => {
                    unmatched.swap_remove(at);
                }
                None => left.push(name.clone()),
            }
        }
        left
    }
}
