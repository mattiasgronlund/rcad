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

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use written_by_hand::{a_block, a_quarter_disc_plate};

/// Every file this kernel wrote itself imports, and the model it makes is
/// sound.
///
/// A block and a quarter-disc plate, each at eleven sizes over eleven orders of
/// magnitude and in each of the four units this kernel names by an `SI_UNIT`
/// prefix. The plate is the half that matters: it has a cylindrical face, two
/// arcs and the half-spaces that say which arc, which is the whole of what a
/// block cannot exercise.
///
/// **One refusal here is one bug**, and it is what makes the two rates below
/// mean anything: a rate with no control cannot tell a reader that refuses real
/// files from a reader that refuses everything (§42.12 criterion 10).
///
/// The plate is **measured** as well as swept, because a sound solid and the
/// solid the file describes are two claims and the sweep only makes the first.
/// A quarter-disc of radius `r` and height `h` encloses `πr²h/4`, which is a
/// number nothing in this kernel supplied.
#[test]
fn every_file_this_kernel_wrote_itself_imports() {
    let mut written = 0;
    for power in -5..6 {
        let size = f64::from(2.0_f32).powi(power);
        for unit in ["$", ".MILLI.", ".CENTI.", ".MICRO."] {
            for (what, text) in [
                ("a block", a_block([size, size * 2.0, size * 4.0], unit)),
                ("a plate", a_quarter_disc_plate(size, size * 2.0, unit)),
            ] {
                let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
                let mut log = ProvenanceLog::new();
                let import = Import::read(&file, &Policy::certified(), &mut log)
                    .unwrap_or_else(|why| panic!("{what} of size {size} in {unit}: {why}"));
                import
                    .model()
                    .validate(&Policy::certified(), &mut log)
                    .expect("and every imported model validates under a certified policy");
                if what == "a plate" {
                    let [(_, key)] = import.solids() else {
                        panic!("one MANIFOLD_SOLID_BREP is one solid");
                    };
                    let measured = import
                        .model()
                        .solid(*key)
                        .measure(
                            Deflection::for_scale(import.model().scale()),
                            &Policy::certified(),
                            &mut log,
                        )
                        .expect("a plate measures");
                    let space = core::f64::consts::PI * size * size * (size * 2.0) / 4.0;
                    assert!(
                        (measured.volume() - space).abs() <= measured.volume_error(),
                        "a plate of radius {size} encloses {space} and this one says {} give \
                         or take {}",
                        measured.volume(),
                        measured.volume_error()
                    );
                }
                written += 1;
            }
        }
    }
    println!("control: {written} / {written} imported");
    assert_eq!(
        written, 88,
        "the control must not shrink without somebody saying so"
    );
}

/// A real file measures what its own name says it should.
///
/// **The independent figure this lane needs, and it is not this kernel's.**
/// Every other check here compares one measurement with another; this one
/// compares a measurement against a number KiCad's own library wrote into the
/// file name before anybody ran anything. A `PinHeader_2x15_P2.54mm` is fifteen
/// pitches of 2.54 mm long and two across, and a
/// `LED_Rectangular_W5.0mm_H2.0mm` is five by two — so a mapping that read a
/// coordinate as a radius, or a millimetre as an inch, or dropped a face, would
/// come out with a part the wrong size and say so here.
///
/// A nanometre of slack, which is a hundredth of what these files declare their
/// own uncertainty to be and a ten-millionth of the smallest thing they hold.
#[test]
fn a_real_file_measures_what_its_own_name_says() {
    let Some(root) = corpus() else {
        return;
    };
    let mut checked = 0;
    let mut dip_switches: Vec<(u32, f64)> = Vec::new();
    for path in files_under(&root.join("kicad")) {
        let claims = what_the_name_says(&name_of(&path));
        let switches = how_many_dip_switches(&name_of(&path));
        if claims.is_empty() && switches.is_none() {
            continue;
        }
        let bytes = fs::read(&path).expect("a file the walk just listed");
        let file = StepFile::read(&bytes).expect("every file in the corpus parses");
        let mut log = ProvenanceLog::new();
        let Ok(import) = Import::read(&file, &Policy::certified(), &mut log) else {
            continue;
        };
        let across = how_big_it_is(&import);
        if let Some(switches) = switches {
            dip_switches.push((switches, across[0]));
        }
        for (axis, expected) in claims {
            assert!(
                (across[axis] - expected).abs() < 1e-9,
                "{} says it is {expected} mm across axis {axis} and it measures {}",
                name_of(&path),
                across[axis]
            );
            checked += 1;
        }
    }
    // A DIP switch's name says how many switches it has and what pitch they sit
    // at, and says nothing about how much housing there is either side. So what
    // its family checks is the **difference**: one more switch is one more
    // 2.54 mm, whatever the ends are worth.
    dip_switches.sort_by_key(|&(switches, _)| switches);
    for pair in dip_switches.windows(2) {
        let [(fewer, shorter), (more, longer)] = pair else {
            unreachable!("windows(2) hands back two");
        };
        let expected = f64::from(more - fewer) * 2.54;
        assert!(
            (longer - shorter - expected).abs() < 1e-9,
            "{more} switches against {fewer} should be {expected} mm longer and is {}",
            longer - shorter
        );
        checked += 1;
    }

    println!("{checked} dimensions checked against the names the library wrote");
    assert!(
        checked >= 24,
        "only {checked} dimensions were checked, which is too few for this to be \
         saying anything"
    );
}

/// How many switches a DIP switch's own name says it has.
fn how_many_dip_switches(name: &str) -> Option<u32> {
    let rest = name.strip_prefix("SW_DIP_SPSTx")?;
    name.contains("_P2.54mm").then_some(())?;
    rest.split_once('_')?.0.parse().ok()
}

/// How far a whole import reaches along each axis.
fn how_big_it_is(import: &Import) -> [f64; 3] {
    let mut least = [f64::INFINITY; 3];
    let mut most = [f64::NEG_INFINITY; 3];
    for (_, key) in import.solids() {
        let Some((low, high)) = import.model().solid(*key).box_around_it() else {
            continue;
        };
        for axis in 0..3 {
            least[axis] = least[axis].min(low.coordinates()[axis]);
            most[axis] = most[axis].max(high.coordinates()[axis]);
        }
    }
    [0, 1, 2].map(|axis| most[axis] - least[axis])
}

/// What a KiCad file's own name claims about how big the part is, as `(axis,
/// millimetres)` pairs.
///
/// One family, and the ones that were tried and dropped are worth naming: a
/// `LED_Rectangular_W3.0mm_H2.0mm` measures **four** millimetres across,
/// because the name is about the body and the leads reach wider than it. A name
/// that describes a part of the part is not a claim about the whole one, and a
/// check built on it would have failed for a reason that is nobody's mistake.
fn what_the_name_says(name: &str) -> Vec<(usize, f64)> {
    if let Some(rest) = name.strip_prefix("PinHeader_")
        && name.contains("_P2.54mm_Vertical")
        && let Some((rows, rest)) = rest.split_once('x')
        && let Some((columns, _)) = rest.split_once('_')
        && let (Ok(rows), Ok(columns)) = (rows.parse::<u32>(), columns.parse::<u32>())
    {
        return vec![(0, f64::from(rows) * 2.54), (1, f64::from(columns) * 2.54)];
    }
    Vec::new()
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
    corner_misses_in_spacings: Vec<f64>,
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
                        Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn {
                            off_by,
                            coordinate_spacing,
                            ..
                        }
                        | Refusal::ACornerIsNotOnThePlaneOfTheArcItEndsOn {
                            off_by,
                            coordinate_spacing,
                            ..
                        } => tally
                            .corner_misses_in_spacings
                            .push(off_by / coordinate_spacing),
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
    #[expect(
        clippy::cast_precision_loss,
        reason = "a percentage of a few hundred files"
    )]
    let rate = 100.0 * tally.imported as f64 / denominator as f64;
    println!(
        "=== {source}: {} / {denominator} imported = {rate:.1}%",
        tally.imported
    );
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
    println!(
        "    shapes this reader did not take: {:?}",
        tally.walked_past
    );
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
    report_the_corner_misses(&tally.corner_misses_in_spacings);
}

/// How far the worst corner of each refused file is off the curved surface the
/// file says it is on, in that corner's own coordinate spacings.
///
/// **The split is the finding and it is a split.** Under one spacing the file's
/// numbers are as close to the surface as an `f64` can be written and the
/// refusal is about what a `Decision` can certify, not about the file. Over one
/// spacing — and the tail runs into the tens of thousands — the file carries a
/// modelling tolerance somebody chose, and no arithmetic anywhere would make
/// those corners land on their surfaces.
///
/// Two different problems with two different answers, and a single median would
/// hide that they are different. Both halves have to be there or this is not
/// the corpus it was measured on.
fn report_the_corner_misses(misses: &[f64]) {
    if misses.is_empty() {
        return;
    }
    let mut sorted = misses.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |part: f64| {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "an index into a few hundred"
        )]
        #[expect(clippy::cast_sign_loss, reason = "part is between zero and one")]
        #[expect(clippy::cast_precision_loss, reason = "a few hundred files")]
        let position = (part * (sorted.len() - 1) as f64) as usize;
        sorted[position]
    };
    let within_a_spacing = sorted.iter().filter(|miss| **miss < 1.0).count();
    println!(
        "    the worst corner of each of the {} refused files, in its own coordinate spacings: \
         least {:.3}, quartile {:.3}, median {:.3}, upper quartile {:.3}, most {:.1}",
        sorted.len(),
        at(0.0),
        at(0.25),
        at(0.5),
        at(0.75),
        at(1.0)
    );
    println!(
        "      {within_a_spacing} of them are inside one spacing — as close as an f64 can be \
         written — and {} are a modelling tolerance",
        sorted.len() - within_a_spacing
    );
    assert!(
        within_a_spacing > 0 && within_a_spacing < sorted.len(),
        "this refusal is two different problems and the corpus has to hold both, or the \
         report is describing a corpus this is not being run on"
    );
    assert!(
        at(1.0) > 1000.0,
        "a corpus whose worst corner is within a thousand spacings holds no file with a \
         modelling tolerance in it, and both corpora do"
    );
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
            format!(
                "the builder or the sweep: {}",
                first_line(&format!("{why}"))
            )
        }
        Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn { .. } => {
            "a corner is not on the curved face the file says it is on".to_owned()
        }
        Refusal::ACornerIsNotOnThePlaneOfTheArcItEndsOn { .. } => {
            "a corner is not on the plane of the arc it ends on".to_owned()
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
