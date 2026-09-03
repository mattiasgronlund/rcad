//! The parts of ISO 10303-21 that real files use, each written out by hand and
//! checked against what it is supposed to mean.
//!
//! Every case here came from looking at the 367 files of M5's corpus and asking
//! what shape they take: complex instances in every AP242 export, `\X2\` in the
//! six files carrying geometric-tolerance symbols, `\X\0A` in the four carrying
//! multi-line drawing notes, reals written `1.` in all 367, comments in 156 of
//! them.
//!
//! Not a tautology: each case asserts the value that came out, not merely that
//! reading returned `Ok`.

use rcad_step::{EntityId, StepFile, Value};

fn file_holding(data: &str) -> Vec<u8> {
    format!(
        "ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('a','2026-09-02T00:00:00',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN {{ 1 0 10303 214 1 1 1 1 }}'));
ENDSEC;
DATA;
{data}
ENDSEC;
END-ISO-10303-21;
"
    )
    .into_bytes()
}

fn only_attribute(data: &str) -> Value {
    let bytes = file_holding(data);
    let file = StepFile::read(&bytes).expect("this file is well formed");
    let instance = file
        .instance(EntityId(1))
        .expect("the file defines exactly #1");
    let [attribute] = instance.parts()[0].attributes() else {
        panic!("#1 was written with one attribute");
    };
    attribute.clone()
}

#[test]
fn the_three_header_entities_come_back_with_the_schema_name_the_file_declared() {
    let bytes = file_holding("#1=A(0);");
    let file = StepFile::read(&bytes).expect("this file is well formed");
    assert_eq!(
        file.header().schemas(),
        ["AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }"]
    );
    assert_eq!(file.header().records().len(), 3);
    assert_eq!(
        file.header()
            .record("FILE_NAME")
            .expect("every file has one")
            .attribute(0)
            .and_then(Value::as_text),
        Some("a")
    );
}

#[test]
fn a_complex_instance_comes_back_as_one_instance_wearing_every_type_it_inherits() {
    let bytes = file_holding(
        "#1=(
NAMED_UNIT(*)
LENGTH_UNIT()
SI_UNIT(.MILLI.,.METRE.)
);",
    );
    let file = StepFile::read(&bytes).expect("this file is well formed");
    let instance = file.instance(EntityId(1)).expect("the file defines #1");
    assert_eq!(instance.parts().len(), 3);
    assert_eq!(
        instance
            .parts()
            .iter()
            .map(|part| part.keyword())
            .collect::<Vec<_>>(),
        ["NAMED_UNIT", "LENGTH_UNIT", "SI_UNIT"]
    );
    assert_eq!(instance.parts()[0].attributes(), [Value::Derived]);
    assert!(instance.part("LENGTH_UNIT").is_some());
    assert!(instance.part("PLANE_ANGLE_UNIT").is_none());
    assert_eq!(
        instance
            .part("SI_UNIT")
            .expect("it is one of the three")
            .attribute(0)
            .and_then(Value::as_enumeration),
        Some("MILLI")
    );
    assert_eq!(file.instances_named("LENGTH_UNIT").count(), 1);
}

#[test]
fn a_real_written_with_a_bare_point_and_one_written_with_an_exponent_are_both_read() {
    assert_eq!(only_attribute("#1=A(1.);"), Value::Real(1.0));
    assert_eq!(only_attribute("#1=A(1.E+03);"), Value::Real(1000.0));
    assert_eq!(only_attribute("#1=A(-2.5E-10);"), Value::Real(-2.5e-10));
    assert_eq!(only_attribute("#1=A(+0.);"), Value::Real(0.0));
}

#[test]
fn one_and_one_point_are_two_different_values_and_neither_becomes_the_other() {
    assert_eq!(only_attribute("#1=A(1);"), Value::Integer(1));
    assert_eq!(only_attribute("#1=A(1.);"), Value::Real(1.0));
    assert_eq!(only_attribute("#1=A(1);").as_real(), None);
    assert_eq!(only_attribute("#1=A(-4);"), Value::Integer(-4));
}

#[test]
fn a_string_gives_back_its_apostrophes_its_backslashes_and_its_escaped_characters() {
    assert_eq!(
        only_attribute("#1=A('it''s');").as_text(),
        Some("it's"),
        "two apostrophes in a row stand for one"
    );
    assert_eq!(only_attribute(r"#1=A('a\\b');").as_text(), Some(r"a\b"));
    assert_eq!(
        only_attribute(r"#1=A('one\X\0Atwo');").as_text(),
        Some("one\ntwo"),
        r"\X\ carries one octet, and 0A is a newline"
    );
    assert_eq!(
        only_attribute(r"#1=A('\X2\2316\X0\');").as_text(),
        Some("\u{2316}"),
        r"\X2\ carries UTF-16, and 2316 is the position symbol AP242 marks datums with"
    );
    assert_eq!(
        only_attribute(r"#1=A('\X2\00480049\X0\!');").as_text(),
        Some("HI!"),
        r"a \X2\ run holds as many code units as it likes"
    );
    assert_eq!(
        only_attribute(r"#1=A('\X2\D83DDE00\X0\');").as_text(),
        Some("\u{1f600}"),
        "a surrogate pair is one character, not two halves"
    );
    assert_eq!(
        only_attribute(r"#1=A('\X4\0001F600\X0\');").as_text(),
        Some("\u{1f600}"),
        r"\X4\ carries whole code points"
    );
    assert_eq!(
        only_attribute(r"#1=A('\S\A');").as_text(),
        Some("\u{c1}"),
        r"\S\ moves the next character into the upper half of ISO 8859-1"
    );
    assert_eq!(
        only_attribute("#1=A('caf\u{e9}');").as_text(),
        Some("café"),
        "Part 21 edition 3 lets a file write UTF-8 straight out"
    );
}

#[test]
fn the_dollar_and_the_star_stay_apart_because_the_file_meant_different_things_by_them() {
    assert_eq!(only_attribute("#1=A($);"), Value::Nothing);
    assert_eq!(only_attribute("#1=A(*);"), Value::Derived);
}

#[test]
fn a_list_holds_whatever_the_file_put_in_it_including_nothing() {
    assert_eq!(only_attribute("#1=A(());"), Value::List(vec![]));
    assert_eq!(
        only_attribute("#1=A((1,2.,'three',.T.,$));"),
        Value::List(vec![
            Value::Integer(1),
            Value::Real(2.0),
            Value::Text("three".into()),
            Value::Enumeration("T".into()),
            Value::Nothing,
        ])
    );
    assert_eq!(
        only_attribute("#1=A(((1),(2)));"),
        Value::List(vec![
            Value::List(vec![Value::Integer(1)]),
            Value::List(vec![Value::Integer(2)]),
        ])
    );
}

#[test]
fn a_value_wearing_a_type_name_keeps_the_name_the_file_gave_it() {
    assert_eq!(
        only_attribute("#1=A(POSITIVE_LENGTH_MEASURE(25.4));"),
        Value::Typed {
            keyword: "POSITIVE_LENGTH_MEASURE".into(),
            value: Box::new(Value::Real(25.4)),
        }
    );
}

#[test]
fn comments_are_passed_over_wherever_a_file_puts_them() {
    let bytes =
        file_holding("/* before */ #1 /* after the id */ = A( /* inside */ 1 , 2 ) ; /* after */");
    let file = StepFile::read(&bytes).expect("comments are not part of the data");
    assert_eq!(
        file.instance(EntityId(1))
            .expect("the file defines #1")
            .parts()[0]
            .attributes(),
        [Value::Integer(1), Value::Integer(2)]
    );
}

#[test]
fn two_entities_that_point_at_each_other_are_read_and_both_ends_resolve() {
    let bytes = file_holding("#1=A(#2);\n#2=A(#1);");
    let file = StepFile::read(&bytes).expect("a cycle is a shape a graph may take");
    file.every_reference_resolves()
        .expect("both ends are defined");
    let first = file.instance(EntityId(1)).expect("the file defines #1");
    assert_eq!(
        first.parts()[0].attribute(0).and_then(Value::as_reference),
        Some(EntityId(2))
    );
    let second = file.instance(EntityId(2)).expect("the file defines #2");
    assert_eq!(
        second.parts()[0].attribute(0).and_then(Value::as_reference),
        Some(EntityId(1))
    );
}

#[test]
fn the_census_counts_a_complex_instance_once_under_each_type_it_wears() {
    let bytes = file_holding(
        "#1=CARTESIAN_POINT('',(0.,0.,0.));
#2=CARTESIAN_POINT('',(1.,0.,0.));
#3=(NAMED_UNIT(*)LENGTH_UNIT()SI_UNIT(.MILLI.,.METRE.));",
    );
    let file = StepFile::read(&bytes).expect("this file is well formed");
    let census = file.how_many_of_each_type();
    assert_eq!(census.get("CARTESIAN_POINT"), Some(&2));
    assert_eq!(census.get("SI_UNIT"), Some(&1));
    assert_eq!(census.get("LENGTH_UNIT"), Some(&1));
    assert_eq!(census.get("ADVANCED_FACE"), None);
    assert_eq!(census.len(), 4);
}

#[test]
fn a_file_may_carry_several_data_sections_and_they_read_as_one_run_of_instances() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
#1=A(#2);
ENDSEC;
DATA;
#2=A(1);
ENDSEC;
END-ISO-10303-21;
";
    let file = StepFile::read(bytes).expect("Part 21 allows more than one data section");
    assert_eq!(file.instances().len(), 2);
    file.every_reference_resolves()
        .expect("#2 is defined, in the other section");
}

#[test]
fn an_escape_whose_character_is_an_apostrophe_takes_both_bytes_of_the_pair() {
    // Found by `just fuzz-reader`: `\S\` read only the first byte of the `''`
    // that stands for one apostrophe, and the decoder then met the second on
    // its own, where a string's own grammar says one cannot stand, and
    // asserted. A caller's bytes may not panic (CLAUDE.md, and decisions.md
    // §42.12 criterion 5), so the lone apostrophe was this reader's bug and not
    // the file's.
    assert_eq!(
        only_attribute(r"#1=A('\S\''');").as_text(),
        Some("\u{a7}"),
        r"\S\ applied to an apostrophe is the section sign"
    );
    assert_eq!(
        only_attribute(r"#1=A('a\S\''b');").as_text(),
        Some("a\u{a7}b"),
        "and the decoder carries on from after the pair, not from inside it"
    );
    assert_eq!(
        only_attribute(r"#1=A('\S\A''');").as_text(),
        Some("\u{c1}'"),
        "an ordinary character after the directive still takes one byte"
    );
}
