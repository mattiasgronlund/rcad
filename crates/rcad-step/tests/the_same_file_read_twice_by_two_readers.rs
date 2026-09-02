//! Reads every corpus file twice — once with this reader, once with `step-io`'s
//! own Part 21 parser — and compares what the two got.
//!
//! The oracle is differential (the testing skill's fourth row): nothing knows
//! what a STEP file "really" holds, but two parsers written from the same
//! standard have to agree about the ids, the type names and the shape of the
//! values, and where they do not, one of them is wrong and it is worth knowing
//! which.
//!
//! `step-io` is a **dev-dependency only** and never appears in a shipping
//! dependency graph — `cargo tree -p rcad-step -e normal` shows `thiserror` and
//! nothing else. Measuring with a tool is not depending on it (decisions.md
//! §42.10).
//!
//! Two disagreements are expected and asserted for, because they are the two
//! places the readers were built to different rules:
//!
//! - **Escapes in strings.** `step-io`'s parser leaves `\X\0A` and
//!   `\X2\2316\X0\` in the string as written and says so in its own doc
//!   comment; this reader turns them into the characters they stand for. A
//!   caller of `step-io`'s parser layer gets text it still has to decode.
//! - **Nothing else.** Every other difference is a failure of this test.
//!
//! Without `RCAD_STEP_CORPUS` the corpus half skips and the hand-written half
//! still runs, so the file is never green on nothing.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_step::{StepFile, Value};
use step_io::parser::{Attribute, RawEntity};

/// What the two readers said differently about one entity.
#[derive(Debug)]
enum Difference {
    /// Text that differs only because `step-io` left the escapes undecoded,
    /// which is the one disagreement this test expects.
    TextBeforeAndAfterDecoding,
    /// Anything else, said in words, because every one of them is a failure and
    /// what a failure needs is to be readable.
    Real(String),
}

#[test]
fn a_hand_written_file_reads_the_same_both_ways() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('','',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
#1=CARTESIAN_POINT('a point',(0.,1.5,-2.E+01));
#2=ADVANCED_FACE('',(#3),#4,.T.);
#3=(NAMED_UNIT(*)LENGTH_UNIT()SI_UNIT(.MILLI.,.METRE.));
#4=MEASURE(POSITIVE_LENGTH_MEASURE(25.4),$,*,7);
ENDSEC;
END-ISO-10303-21;
";
    let differences = compare(bytes);
    assert!(
        differences.is_empty(),
        "two readers disagreed about a file with nothing odd in it: {differences:?}"
    );
}

#[test]
fn the_two_readers_agree_about_every_corpus_file_except_where_escapes_are_decoded() {
    let Some(root) = corpus() else {
        return;
    };
    for source in ["nist", "kicad"] {
        let files = files_under(&root.join(source));
        assert!(
            !files.is_empty(),
            "{source} holds no STEP files, so this would compare nothing"
        );

        let mut compared = 0;
        let mut entities = 0;
        let mut escapes = 0;
        let mut real = Vec::new();
        for path in &files {
            let bytes = fs::read(path).expect("a file the walk just listed");
            let mine = StepFile::read(&bytes);
            let theirs = step_io::parser::parse_bytes(&bytes);
            match (&mine, &theirs) {
                (Ok(mine), Ok(theirs)) => {
                    compared += 1;
                    entities += mine.instances().len();
                    assert_eq!(
                        mine.instances().len(),
                        theirs.entities.len(),
                        "{}: {} entities here, {} there",
                        name_of(path),
                        mine.instances().len(),
                        theirs.entities.len()
                    );
                }
                (Err(refusal), Ok(_)) => {
                    real.push(format!(
                        "{}: only step-io read it ({refusal})",
                        name_of(path)
                    ));
                    continue;
                }
                (Ok(_), Err(refusal)) => {
                    real.push(format!(
                        "{}: only this reader read it ({refusal})",
                        name_of(path)
                    ));
                    continue;
                }
                (Err(_), Err(_)) => continue,
            }
            for difference in compare(&bytes) {
                match difference {
                    Difference::TextBeforeAndAfterDecoding => escapes += 1,
                    Difference::Real(said) => {
                        real.push(format!("{}: {said}", name_of(path)));
                    }
                }
            }
        }

        println!(
            "{source}: {compared} / {} files compared, {entities} entities, \
             {escapes} strings that step-io left undecoded",
            files.len()
        );
        for difference in &real {
            println!("  {difference}");
        }
        assert!(
            real.is_empty(),
            "{source}: {} real disagreements",
            real.len()
        );
        assert_eq!(
            compared,
            files.len(),
            "{source}: the two readers did not both read every file"
        );
        assert!(
            entities > files.len(),
            "{source}: {entities} entities is too few to have compared anything"
        );
    }
}

fn compare(bytes: &[u8]) -> Vec<Difference> {
    let mine = StepFile::read(bytes).expect("this reader reads it");
    let theirs = step_io::parser::parse_bytes(bytes).expect("step-io reads it");

    let my_ids: BTreeSet<u64> = mine.instances().map(|instance| instance.id().0).collect();
    let their_ids: BTreeSet<u64> = theirs.entities.keys().copied().collect();

    let mut differences = Vec::new();
    differences.extend(
        my_ids
            .difference(&their_ids)
            .map(|id| Difference::Real(format!("#{id} is here and not in step-io"))),
    );
    differences.extend(
        their_ids
            .difference(&my_ids)
            .map(|id| Difference::Real(format!("#{id} is in step-io and not here"))),
    );

    for instance in mine.instances() {
        let id = instance.id().0;
        let Some(other) = theirs.get(id) else {
            continue;
        };
        let (their_names, their_attributes) = spread(other);
        let my_names: Vec<String> = instance
            .parts()
            .iter()
            .map(|part| part.keyword().to_string())
            .collect();
        if my_names != their_names {
            differences.push(Difference::Real(format!(
                "#{id} is {my_names:?} here and {their_names:?} in step-io"
            )));
            continue;
        }
        let my_attributes: Vec<&Value> = instance
            .parts()
            .iter()
            .flat_map(|part| part.attributes())
            .collect();
        if my_attributes.len() != their_attributes.len() {
            differences.push(Difference::Real(format!(
                "#{id} has {} attributes here and {} in step-io",
                my_attributes.len(),
                their_attributes.len()
            )));
            continue;
        }
        for (mine, theirs) in my_attributes.iter().zip(&their_attributes) {
            if let Some(difference) = same(mine, theirs, id) {
                differences.push(difference);
            }
        }
    }
    differences
}

/// One entity's type names and all its attributes end to end, whether it is a
/// simple instance or a complex one.
fn spread(entity: &RawEntity) -> (Vec<String>, Vec<&Attribute>) {
    match entity {
        RawEntity::Simple {
            name, attributes, ..
        } => (vec![name.clone()], attributes.iter().collect()),
        RawEntity::Complex { parts, .. } => (
            parts.iter().map(|part| part.name.clone()).collect(),
            parts
                .iter()
                .flat_map(|part| part.attributes.iter())
                .collect(),
        ),
    }
}

fn same(mine: &Value, theirs: &Attribute, id: u64) -> Option<Difference> {
    let unlike = |mine: String, theirs: String| {
        Some(Difference::Real(format!(
            "#{id} holds {mine} here and {theirs} in step-io"
        )))
    };
    match (mine, theirs) {
        (Value::Integer(here), Attribute::Integer(there)) if here == there => None,
        (Value::Real(here), Attribute::Real(there)) if here == there => None,
        (Value::Enumeration(here), Attribute::Enum(there)) if here == there => None,
        (Value::Reference(here), Attribute::EntityRef(there)) if here.0 == *there => None,
        (Value::Nothing, Attribute::Unset) => None,
        (Value::Derived, Attribute::Derived) => None,
        (Value::Text(here), Attribute::String(there)) => {
            if here == there {
                None
            } else if there.contains('\\') {
                Some(Difference::TextBeforeAndAfterDecoding)
            } else {
                unlike(format!("{here:?}"), format!("{there:?}"))
            }
        }
        (Value::List(here), Attribute::List(there)) => {
            if here.len() != there.len() {
                return unlike(
                    format!("a list of {}", here.len()),
                    format!("a list of {}", there.len()),
                );
            }
            here.iter()
                .zip(there)
                .find_map(|(here, there)| same(here, there, id))
        }
        (
            Value::Typed {
                keyword: here,
                value: inside_here,
            },
            Attribute::Typed {
                type_name: there,
                value: inside_there,
            },
        ) => {
            if here != there {
                return unlike(here.clone(), there.clone());
            }
            same(inside_here, inside_there, id)
        }
        (mine, theirs) => unlike(format!("{mine:?}"), format!("{theirs:?}")),
    }
}

fn corpus() -> Option<PathBuf> {
    let Some(root) = env::var_os("RCAD_STEP_CORPUS") else {
        println!("RCAD_STEP_CORPUS is not set, so there is no corpus to compare; skipping");
        return None;
    };
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "RCAD_STEP_CORPUS points at {}, which is not a directory",
        root.display()
    );
    Some(root)
}

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_under(&path));
            continue;
        }
        let is_step = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("stp") || extension.eq_ignore_ascii_case("step")
            });
        if is_step {
            found.push(path);
        }
    }
    found.sort();
    found
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}
