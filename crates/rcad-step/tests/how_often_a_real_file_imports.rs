//! §42.12 criterion 10: the import rate, family by family and never blended,
//! with the reason every refusal gave.
//!
//! Two corpora and a control, and the three are three different questions.
//!
//! - **The control** is files this test writes itself, out of the shapes this
//!   kernel already builds. It has to come in at **100 %**. It is what makes the
//!   other two numbers mean anything: a rate with no control cannot tell a
//!   reader that refuses real files from a reader that refuses everything.
//! - **`nist/`** — 44 ASME Y14.5 GD&T certification parts, built to exercise
//!   every surface a CAD system might mis-tolerance.
//! - **`kicad/`** — 323 connector and switch housings out of KiCad's own
//!   footprint library.
//!
//! The two real corpora disagree by a factor of eight about what a real file
//! holds (§42's spike) and are never added together. Neither is in the tree;
//! point `RCAD_STEP_CORPUS` at a directory holding `nist/` and `kicad/` and this
//! runs, and without it the corpus half skips and says so while the control half
//! still runs.

mod written_by_hand;

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use written_by_hand::a_block;

/// Every file this kernel wrote itself imports, and the model it makes is
/// sound.
///
/// A block at eleven sizes over eleven orders of magnitude and in every unit
/// this kernel has. One refusal here is one bug.
#[test]
fn every_file_this_kernel_wrote_itself_imports() {
    let mut written = 0;
    for power in -5..6 {
        let side = f64::from(2.0_f32).powi(power);
        for unit in ["$", ".MILLI.", ".CENTI.", ".MICRO."] {
            let text = a_block([side, side * 2.0, side * 4.0], unit);
            let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
            let mut log = ProvenanceLog::new();
            let import = Import::read(&file, &Policy::certified(), &mut log)
                .unwrap_or_else(|why| panic!("a block of side {side} in {unit} did not import: {why}"));
            import
                .model()
                .validate(&Policy::certified(), &mut log)
                .expect("and every imported model validates under a certified policy");
            written += 1;
        }
    }
    println!("control: {written} / {written} imported");
    assert_eq!(written, 44, "the control must not shrink without somebody saying so");
}

/// What one corpus came to.
#[derive(Default)]
struct Tally {
    imported: usize,
    solids: usize,
    refused: BTreeMap<String, usize>,
    surfaces_wanted: BTreeMap<String, usize>,
    curves_wanted: BTreeMap<String, usize>,
    integers_read_as_reals: usize,
    files_needing_that: usize,
    walked_past: BTreeMap<&'static str, usize>,
    units: BTreeMap<String, usize>,
    over_precise: usize,
}

#[test]
fn how_often_a_real_file_imports() {
    let Some(root) = corpus() else {
        return;
    };

    for source in ["nist", "kicad"] {
        let files = files_under(&root.join(source));
        assert!(
            !files.is_empty(),
            "{source} holds no STEP files, so this test would pass by doing nothing"
        );
        let mut tally = Tally::default();
        for path in &files {
            let bytes = fs::read(path).expect("a file the walk just listed");
            let file = StepFile::read(&bytes).expect("every file in both corpora parses");
            let mut log = ProvenanceLog::new();
            match Import::read(&file, &Policy::certified(), &mut log) {
                Ok(import) => {
                    tally.imported += 1;
                    tally.solids += import.solids().len();
                    *tally
                        .units
                        .entry(import.declared_unit().symbol().to_owned())
                        .or_default() += 1;
                    if import.integers_read_as_reals() > 0 {
                        tally.files_needing_that += 1;
                        tally.integers_read_as_reals += import.integers_read_as_reals();
                    }
                    if import.asks_for_more_precision_than_its_numbers_carry() {
                        tally.over_precise += 1;
                    }
                    for (keyword, count) in import.shapes_this_reader_did_not_take() {
                        *tally.walked_past.entry(keyword).or_default() += count;
                    }
                    import
                        .model()
                        .validate(&Policy::certified(), &mut log)
                        .unwrap_or_else(|why| {
                            panic!(
                                "{} imported and then failed the sweep, which is a defect and \
                                 not a refusal: {why}",
                                name_of(path)
                            )
                        });
                }
                Err(why) => {
                    *tally.refused.entry(grouped(&why)).or_default() += 1;
                    match &why {
                        Refusal::ASurfaceThisKernelDoesNotHold { keyword, .. } => {
                            *tally.surfaces_wanted.entry(keyword.clone()).or_default() += 1;
                        }
                        Refusal::ACurveThisKernelDoesNotHold { keyword, .. } => {
                            *tally.curves_wanted.entry(keyword.clone()).or_default() += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        report(source, &files, &tally);
    }
}

fn report(source: &str, files: &[PathBuf], tally: &Tally) {
    let denominator = files.len();
    #[expect(clippy::cast_precision_loss, reason = "a percentage of a few hundred files")]
    let rate = 100.0 * tally.imported as f64 / denominator as f64;
    println!("=== {source}: {} / {denominator} imported = {rate:.1}%", tally.imported);
    println!("    {} solids", tally.solids);
    println!("    units declared: {:?}", tally.units);
    println!(
        "    {} files wrote an integer where the schema wants a real, {} times in all",
        tally.files_needing_that, tally.integers_read_as_reals
    );
    println!(
        "    {} files ask for more precision than their own numbers carry",
        tally.over_precise
    );
    println!("    shapes this reader did not take: {:?}", tally.walked_past);
    let mut reasons: Vec<_> = tally.refused.iter().collect();
    reasons.sort_by_key(|&(name, count)| (std::cmp::Reverse(*count), name.clone()));
    for (reason, count) in reasons {
        println!("    {count:>4}  {reason}");
    }
    if !tally.surfaces_wanted.is_empty() {
        println!("    surfaces asked for: {:?}", tally.surfaces_wanted);
    }
    if !tally.curves_wanted.is_empty() {
        println!("    curves asked for: {:?}", tally.curves_wanted);
    }
}

/// The name a refusal is counted under, which is the variant and never the
/// entity it names.
fn grouped(why: &Refusal) -> String {
    match why {
        Refusal::TheFileHoldsNoSolid => "the file holds no solid".to_owned(),
        Refusal::ASolidWithAVoidInIt { .. } => "a solid with a void in it".to_owned(),
        Refusal::ASurfaceThisKernelDoesNotHold { keyword, .. } => {
            format!("a surface this kernel does not hold: {keyword}")
        }
        Refusal::ACurveThisKernelDoesNotHold { keyword, .. } => {
            format!("a curve this kernel does not hold: {keyword}")
        }
        Refusal::ALoopThisReaderDoesNotWalk { keyword, .. } => {
            format!("a loop this reader does not walk: {keyword}")
        }
        Refusal::AFaceThatDoesNotSayWhichBoundIsTheOuterOne { .. } => {
            "a face that does not say which bound is the outer one".to_owned()
        }
        Refusal::AnArcThatStartsAndEndsInTheSamePlace { .. } => {
            "an arc that starts and ends in the same place".to_owned()
        }
        Refusal::AnArcThisReaderCouldNotName { .. } => {
            "an arc this reader could not name".to_owned()
        }
        Refusal::ALengthUnitThisKernelDoesNotHave { described, .. } => {
            format!("a length unit this kernel does not have: {described}")
        }
        Refusal::TheFileDeclaresNoLengthUnit => "the file declares no length unit".to_owned(),
        Refusal::TheFileDeclaresTwoDifferentLengthUnits { .. } => {
            "the file declares two different length units".to_owned()
        }
        Refusal::APlacementThisReaderCannotApply { keyword, .. } => {
            format!("a placement this reader cannot apply: {keyword}")
        }
        Refusal::AnEntityIsNotTheKindTheSchemaWants { wanted, found, .. } => {
            format!("a {found} where the schema wants a {wanted}")
        }
        Refusal::AnAttributeIsNotWhatTheSchemaWants {
            keyword, wanted, ..
        } => format!("{keyword}'s attribute is not the {wanted} the schema wants"),
        Refusal::AnIntegerTooBigToBeTheRealItStandsFor { .. } => {
            "an integer too big to be the real it stands for".to_owned()
        }
        Refusal::AReferencePointsNowhere { .. } => "a reference points nowhere".to_owned(),
        Refusal::TheGeometryIsNotAWorkingShape { why, .. } => {
            format!("the geometry is not a working shape: {why}")
        }
        Refusal::ThisEntityOfTheSolidWasRefused { why, .. }
        | Refusal::TheSolidTheFileDescribesIsNotOne { why, .. } => {
            format!("the builder or the sweep: {}", first_line(&format!("{why}")))
        }
        Refusal::TheFileHasNoSize => "the file has no size".to_owned(),
        // `Refusal` is `#[non_exhaustive]` and grows; a new variant that is not
        // named here would otherwise be counted under somebody else's name.
        _ => format!("unnamed: {why}"),
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or(text).to_owned()
}

fn corpus() -> Option<PathBuf> {
    let Some(root) = env::var_os("RCAD_STEP_CORPUS") else {
        println!("RCAD_STEP_CORPUS is not set, so there is no corpus to import; skipping");
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
