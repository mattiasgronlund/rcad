//! Everything the reader will not read, refused by the name it is refused
//! under.
//!
//! This is the honest half of "M5 heals nothing" (decisions.md §42.9, option
//! A): a caller can tell exactly what was in the file and what was not, so
//! every refusal here is checked against a named [`Problem`] and not merely
//! against `is_err()`. Checking `is_err()` would pass if every one of these
//! came back as the same shrug.
//!
//! A file is somebody else's bytes, so none of these may panic. That is
//! checked here for the cases anybody thought of, and by
//! `fuzz/fuzz_targets/no_step_file_makes_the_reader_panic.rs` for the ones
//! nobody did.

use rcad_step::{EntityId, Problem, StepFile, Value};

fn file_holding(data: &str) -> Vec<u8> {
    format!(
        "ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
{data}
ENDSEC;
END-ISO-10303-21;
"
    )
    .into_bytes()
}

fn refusal_for(data: &str) -> Problem {
    let bytes = file_holding(data);
    StepFile::read(&bytes)
        .expect_err("this data is supposed to be refused")
        .problem
}

#[test]
fn something_that_is_not_a_step_file_at_all_is_refused_before_anything_else() {
    for bytes in [
        b"".as_slice(),
        b"hello".as_slice(),
        b"<?xml version='1.0'?>".as_slice(),
        &[0xff, 0xfe, 0x00, 0x00],
        b"iso-10303-21;".as_slice(),
    ] {
        assert_eq!(
            StepFile::read(bytes)
                .expect_err("none of these is a Part 21 file")
                .problem,
            Problem::NotAPart21File,
            "for {:?}",
            String::from_utf8_lossy(bytes)
        );
    }
}

#[test]
fn a_file_cut_off_part_way_through_says_where_it_stopped_instead_of_reading_what_is_left() {
    let whole = file_holding("#1=CARTESIAN_POINT('',(0.,0.,0.));");
    let mut refusals = 0;
    for cut in 1..whole.len() {
        let Err(refusal) = StepFile::read(&whole[..cut]) else {
            continue;
        };
        refusals += 1;
        assert!(
            refusal.line >= 1 && refusal.column >= 1,
            "a refusal has to say where: {refusal}"
        );
    }
    assert_eq!(
        refusals,
        whole.len() - 2,
        "every cut but the two that leave the file whole must be refused"
    );
}

#[test]
fn a_string_that_is_never_closed_is_refused_rather_than_running_to_the_end_of_the_file() {
    assert_eq!(
        refusal_for("#1=A('never closed);"),
        Problem::StringNeverEnds
    );
    assert_eq!(
        refusal_for("#1=A('doubled at the end'');"),
        Problem::StringNeverEnds,
        "two apostrophes are one apostrophe, so this string is still open"
    );
}

#[test]
fn a_comment_that_is_never_closed_is_refused_rather_than_swallowing_the_file() {
    assert_eq!(
        refusal_for("#1=A(1); /* and then"),
        Problem::CommentNeverEnds
    );
}

#[test]
fn a_reference_to_an_entity_the_file_never_defines_is_named_with_both_ids() {
    let bytes = file_holding("#1=A(#7);\n#2=A((#7,#8));");
    let file = StepFile::read(&bytes).expect("nothing about the syntax is wrong");

    let broken = file.references_that_point_nowhere();
    assert_eq!(
        broken
            .iter()
            .map(|reference| (reference.from, reference.to))
            .collect::<Vec<_>>(),
        [
            (EntityId(1), EntityId(7)),
            (EntityId(2), EntityId(7)),
            (EntityId(2), EntityId(8)),
        ]
    );
    assert_eq!(broken[0].line, 8, "#1 is on the eighth line of the file");
    assert!(file.instance(EntityId(7)).is_none());

    assert_eq!(
        file.every_reference_resolves()
            .expect_err("three references point nowhere")
            .problem,
        Problem::ReferencePointsNowhere {
            from: EntityId(1),
            to: EntityId(7),
        }
    );
}

#[test]
fn two_instances_claiming_the_same_id_are_refused_rather_than_one_of_them_chosen() {
    assert_eq!(
        refusal_for("#1=A(1);\n#1=A(2);"),
        Problem::EntityIdUsedTwice { id: EntityId(1) }
    );
}

#[test]
fn an_entity_id_too_big_to_hold_is_refused_rather_than_wrapped() {
    assert_eq!(
        refusal_for("#18446744073709551616=A(1);"),
        Problem::EntityIdTooLarge {
            text: "18446744073709551616".into(),
        }
    );
    let bytes = file_holding("#18446744073709551615=A(1);");
    let file = StepFile::read(&bytes).expect("the largest u64 still fits");
    assert!(file.instance(EntityId(u64::MAX)).is_some());
}

#[test]
fn a_number_too_big_to_hold_is_refused_rather_than_rounded_to_something_else() {
    assert_eq!(
        refusal_for("#1=A(9223372036854775808);"),
        Problem::IntegerTooLarge {
            text: "9223372036854775808".into(),
        }
    );
    assert_eq!(
        refusal_for("#1=A(1.E+999);"),
        Problem::RealTooLarge {
            text: "1.E+999".into(),
        }
    );
    let bytes = file_holding("#1=A(1.E-999);");
    let file = StepFile::read(&bytes).expect("a number too small is still a number");
    assert_eq!(
        file.instance(EntityId(1))
            .expect("the file defines #1")
            .parts()[0]
            .attribute(0),
        Some(&Value::Real(0.0)),
        "an f64 that underflows has an answer, and it is the answer the file gets"
    );
}

#[test]
fn a_number_with_an_exponent_but_no_decimal_point_is_refused_by_that_name() {
    assert_eq!(
        refusal_for("#1=A(1E3);"),
        Problem::ExponentWithoutADecimalPoint { text: "1E3".into() },
        "Part 21 has no such literal: an integer has no exponent and a real needs its dot"
    );
}

#[test]
fn values_nested_deeper_than_the_reader_can_walk_are_refused_and_do_not_overflow_the_stack() {
    let deep = format!(
        "#1=A({}1{});",
        "(".repeat(rcad_step::DEEPEST_NESTING + 4),
        ")".repeat(rcad_step::DEEPEST_NESTING + 4)
    );
    assert_eq!(
        refusal_for(&deep),
        Problem::NestedTooDeeply {
            limit: rcad_step::DEEPEST_NESTING,
        }
    );

    let hostile = format!("#1=A({});", "(".repeat(200_000));
    assert_eq!(
        refusal_for(&hostile),
        Problem::NestedTooDeeply {
            limit: rcad_step::DEEPEST_NESTING,
        },
        "the point of the limit is that this returns rather than crashing"
    );
}

#[test]
fn a_keyword_that_is_not_upper_case_is_refused_rather_than_folded_into_one_that_is() {
    assert_eq!(
        refusal_for("#1=advanced_face('',(#2),#3,.T.);"),
        Problem::KeywordIsNotUpperCase {
            keyword: "advanced_face".into(),
        }
    );
}

#[test]
fn a_binary_value_is_refused_by_name_because_nothing_downstream_could_be_handed_one() {
    assert_eq!(
        refusal_for("#1=A(\"0F3A\");"),
        Problem::BinaryValueThisReaderDoesNotRead
    );
}

#[test]
fn an_escape_in_a_string_the_reader_does_not_know_is_named_and_not_dropped() {
    assert_eq!(
        refusal_for(r"#1=A('\Q\x');"),
        Problem::EscapeThisReaderDoesNotKnow {
            escape: r"\Q\x".into(),
        }
    );
    assert_eq!(
        refusal_for(r"#1=A('\X2\00');"),
        Problem::EscapeThisReaderDoesNotKnow {
            escape: r"\X2\00".into(),
        },
        "a \\X2\\ run that is neither four hex digits nor its terminator"
    );
    assert_eq!(
        refusal_for(r"#1=A('\X2\D83D\X0\');"),
        Problem::EscapeThisReaderDoesNotKnow {
            escape: r"\X2\D83D".into(),
        },
        "half of a surrogate pair is not a character"
    );
}

#[test]
fn a_string_switching_to_an_alphabet_this_reader_does_not_have_says_which_one() {
    assert_eq!(
        refusal_for(r"#1=A('\PG\\S\A');"),
        Problem::AlphabetThisReaderDoesNotHave { page: 'G' }
    );
    let bytes = file_holding(r"#1=A('\PA\\S\A');");
    assert!(
        StepFile::read(&bytes).is_ok(),
        "page A is ISO 8859-1, which is the one it has"
    );
}

#[test]
fn bytes_in_a_string_that_are_neither_ascii_nor_utf_8_are_refused_rather_than_guessed_at() {
    let mut bytes = file_holding("#1=A('  ');");
    let spot = bytes
        .windows(2)
        .position(|pair| pair == b"  ")
        .expect("the two spaces are in there");
    bytes[spot] = 0xe9;
    bytes[spot + 1] = 0x20;
    assert_eq!(
        StepFile::read(&bytes)
            .expect_err("0xe9 is Latin-1's e-acute and not valid UTF-8 on its own")
            .problem,
        Problem::StringIsNotText
    );
}

#[test]
fn a_header_missing_one_of_the_three_entities_iso_10303_21_requires_says_which() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','','');
ENDSEC;
DATA;
ENDSEC;
END-ISO-10303-21;
";
    assert_eq!(
        StepFile::read(bytes)
            .expect_err("FILE_SCHEMA is not optional")
            .problem,
        Problem::HeaderEntityMissing {
            keyword: "FILE_SCHEMA",
        }
    );
}

#[test]
fn a_reference_in_the_header_is_refused_because_there_is_nothing_there_to_point_at() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','',#4);
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
#4=A(1);
ENDSEC;
END-ISO-10303-21;
";
    assert_eq!(
        StepFile::read(bytes)
            .expect_err("the header has no entities")
            .problem,
        Problem::ReferenceInTheHeader { id: EntityId(4) }
    );
}

#[test]
fn a_file_schema_that_is_not_a_list_of_names_is_refused_rather_than_read_as_an_empty_one() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','','');
FILE_SCHEMA('AUTOMOTIVE_DESIGN');
ENDSEC;
DATA;
ENDSEC;
END-ISO-10303-21;
";
    assert_eq!(
        StepFile::read(bytes)
            .expect_err("FILE_SCHEMA holds a list, not a bare string")
            .problem,
        Problem::SchemaNamesAreNotAListOfStrings
    );
}

#[test]
fn a_section_this_reader_does_not_read_is_refused_by_the_name_the_file_gives_it() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
ANCHOR;
ENDSEC;
DATA;
ENDSEC;
END-ISO-10303-21;
";
    assert_eq!(
        StepFile::read(bytes)
            .expect_err("edition 3's anchor section is not read here")
            .problem,
        Problem::SectionThisReaderDoesNotRead {
            keyword: "ANCHOR".into(),
        }
    );
}

#[test]
fn a_complex_instance_with_nothing_in_it_is_refused_with_its_id() {
    assert_eq!(
        refusal_for("#1=();"),
        Problem::ComplexInstanceHasNoParts { id: EntityId(1) }
    );
}

#[test]
fn anything_written_after_the_closing_line_is_refused_rather_than_ignored() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
ENDSEC;
END-ISO-10303-21;
#1=A(1);
";
    assert!(
        matches!(
            StepFile::read(bytes)
                .expect_err("the file said it had ended")
                .problem,
            Problem::NotWhatWasExpected { .. }
        ),
        "data after the end is data nobody would ever read"
    );
}

#[test]
fn every_refusal_says_which_line_and_column_it_happened_at() {
    let refusal = StepFile::read(&file_holding("#1=A(1);\n#2=A('open"))
        .expect_err("the string on the second data line is never closed");
    assert_eq!(refusal.line, 9);
    assert_eq!(refusal.column, 6);
    assert!(
        refusal.to_string().contains("line 9, column 6"),
        "what a caller reads has to carry the place too: {refusal}"
    );
}

#[test]
fn a_complex_instance_that_says_it_is_the_same_thing_twice_is_refused_with_the_name() {
    assert_eq!(
        refusal_for("#1=(LENGTH_UNIT()NAMED_UNIT(*)LENGTH_UNIT());"),
        Problem::ComplexInstanceRepeatsAType {
            id: EntityId(1),
            keyword: "LENGTH_UNIT".into(),
        },
        "nothing could say which of the two Instance::part should hand back"
    );
}
