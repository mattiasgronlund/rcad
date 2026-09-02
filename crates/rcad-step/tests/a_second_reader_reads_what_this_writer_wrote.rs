//! The bytes this crate's writer produces, read by somebody else's Part 21
//! parser.
//!
//! **This is the half a round trip cannot do.** A writer checked only against
//! the reader beside it passes on any misunderstanding the two share:
//! `a_solid_written_out_and_read_back_is_the_same_solid.rs` would stay green if
//! both sides agreed on a spelling Part 21 does not have. `step-io` is a second
//! implementation of the same standard, written by somebody who never saw this
//! one, and it is a **dev-dependency only** — `cargo tree -p rcad-step --edges
//! normal` does not show it (decisions.md §42.10).
//!
//! The comparison is at the layer the two share: `step_io::parser::parse_bytes`
//! hands back a graph of raw entities, which is exactly what
//! [`StepFile::read`] hands back, so it is id for id, type name for type name
//! and attribute count for attribute count.
//!
//! What this does **not** check is what the entities mean: `step-io`'s parser
//! knows nothing about `ADVANCED_FACE` and neither does this crate's. The
//! meaning is checked by `tests/written_by_hand/`, whose files nothing in this
//! crate produced.

use rcad_geom::{Isometry, Scale, Unit};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Record, StepFile};
use rcad_topo::Model;
use step_io::parser::RawEntity;

/// The three shapes the round trip covers, written out and handed to the other
/// reader.
#[test]
fn step_io_reads_every_file_this_writer_wrote_and_agrees_entity_for_entity() {
    let mut compared = 0;
    let mut entities = 0;
    for (called, text) in every_shape_this_writer_can_write() {
        let mine = StepFile::read(text.as_bytes())
            .unwrap_or_else(|why| panic!("{called}: this reader refused its own writer: {why}"));
        let theirs = step_io::parser::parse_bytes(text.as_bytes())
            .unwrap_or_else(|why| panic!("{called}: step-io refused this writer's bytes: {why:?}"));

        assert_eq!(
            mine.instances().len(),
            theirs.entities.len(),
            "{called}: {} entities here, {} there",
            mine.instances().len(),
            theirs.entities.len()
        );
        for instance in mine.instances() {
            let there = theirs
                .entities
                .get(&instance.id().0)
                .unwrap_or_else(|| panic!("{called}: step-io has no {}", instance.id()));
            let (their_names, their_attributes) = spread(there);
            let my_names: Vec<&str> = instance.parts().iter().map(Record::keyword).collect();
            assert_eq!(
                my_names,
                their_names,
                "{called}: {} is {my_names:?} here and {their_names:?} there",
                instance.id()
            );
            let mine: usize = instance
                .parts()
                .iter()
                .map(|part| part.attributes().len())
                .sum();
            assert_eq!(
                mine,
                their_attributes,
                "{called}: {} has {mine} attributes here and {their_attributes} there",
                instance.id()
            );
            entities += 1;
        }
        compared += 1;
    }
    println!(
        "step-io read {compared} file(s) this writer wrote, {entities} entities compared \
         id for id"
    );
    assert!(
        compared >= 3 && entities > 100,
        "this compared almost nothing: {compared} files, {entities} entities"
    );
}

/// A name full of the characters Part 21 has to escape goes out and comes back
/// as itself.
///
/// **The reader's own fuzz corpus is why this is here.** `\S\` applied to the
/// second half of a doubled apostrophe crashed the reader once, and the two
/// escapes a writer has to get right are exactly the two that found it: an
/// apostrophe is written twice, and anything outside printable ASCII goes out
/// as `\X2\`…`\X0\` UTF-16. A writer that got either wrong would produce a file
/// this reader refuses or, worse, one it reads as a different string.
#[test]
fn a_file_name_full_of_apostrophes_and_umlauts_comes_back_as_itself() {
    let ugly = [
        "it's a part",
        "''",
        "'",
        "Grönlund's fläns",
        "\u{2192} \u{1f600} \u{5f20}",
        "mixed 'quotes' and \u{e5}\u{e4}\u{f6} and 'more'",
        "",
    ];
    let mut model = Model::new(Scale::new(Unit::Millimetre, 2.0).expect("a working size"));
    model
        .add_round_bar(1.0, 2.0, &Isometry::identity())
        .expect("a bar to hang a name on");

    let mut checked = 0;
    for name in ugly {
        let text = rcad_step::write(&model, name).expect("a bar writes");
        let file = StepFile::read(text.as_bytes())
            .unwrap_or_else(|why| panic!("{name:?} wrote a file this reader refuses: {why}"));
        let written = file
            .header()
            .record("FILE_NAME")
            .expect("every file this writer writes has a FILE_NAME")
            .attribute(0)
            .and_then(rcad_step::Value::as_text)
            .expect("and its first attribute is the name");
        assert_eq!(
            written, name,
            "a name went out and came back as something else"
        );
        step_io::parser::parse_bytes(text.as_bytes())
            .unwrap_or_else(|why| panic!("step-io refused a file named {name:?}: {why:?}"));
        checked += 1;
    }
    assert_eq!(
        checked, 7,
        "the draw must not shrink without somebody saying so"
    );
}

/// One entity's type names, and how many attributes it holds end to end,
/// whether it is a simple instance or a complex one.
fn spread(entity: &RawEntity) -> (Vec<&str>, usize) {
    match entity {
        RawEntity::Simple {
            name, attributes, ..
        } => (vec![name.as_str()], attributes.len()),
        RawEntity::Complex { parts, .. } => (
            parts.iter().map(|part| part.name.as_str()).collect(),
            parts.iter().map(|part| part.attributes.len()).sum(),
        ),
    }
}

/// Every shape this writer can write today, each as a whole file.
fn every_shape_this_writer_can_write() -> Vec<(&'static str, String)> {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut written = Vec::new();

    let mut cube = Model::new(Scale::new(Unit::Millimetre, 2.0).expect("a working size"));
    cube.add_cube(2.0, &policy, &mut log).expect("a cube");
    written.push((
        "a cube",
        rcad_step::write(&cube, "a cube").expect("a cube writes"),
    ));

    let mut bar = Model::new(Scale::new(Unit::Millimetre, 8.0).expect("a working size"));
    bar.add_round_bar(2.0, 8.0, &Isometry::identity())
        .expect("a bar");
    written.push((
        "a round bar",
        rcad_step::write(&bar, "a round bar").expect("a bar writes"),
    ));

    // The one written in a unit no `SI_UNIT` prefix names, so the
    // `CONVERSION_BASED_UNIT` this writer produces is put through the other
    // reader too.
    let mut inches = Model::new(Scale::new(Unit::Inch, 1.0).expect("a working size"));
    inches
        .add_cube(1.0, &policy, &mut log)
        .expect("a cubic inch");
    written.push((
        "a cubic inch",
        rcad_step::write(&inches, "a cubic inch").expect("a cubic inch writes"),
    ));

    written
}
