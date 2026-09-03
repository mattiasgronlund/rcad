//! Every KiCad file that imports today is written back out, read back, and
//! required to be the same solid — bit for bit, and, where the shapes fuse at
//! all, the same fuse answer against a third solid. Which files manage it is
//! pinned by name in [`KICAD_FILES_THAT_ROUND_TRIP`], with the reason the rest
//! do not.
//!
//! `a_solid_written_out_and_read_back_is_the_same_solid.rs` checks the writer
//! against solids **this kernel built**. This is a stronger check: a solid
//! that came out of a stranger's file, with a stranger's idea of how to name a
//! cylinder or where to put a seam. Point `RCAD_STEP_CORPUS` at a directory
//! holding `kicad/` and this runs; without it, this test **fails** rather than
//! skips — green has to mean the corpus ran (decisions.md §44.10).
//!
//! **A failure here is the point of the exercise, not a setback.** Nothing
//! panics on a corpus file's account: every disagreement is caught, grouped by
//! its reason and counted, and the run reports the round-trip count with its
//! denominator and every group. The only things that do panic are ones no
//! corpus content could cause — a file this crate's own reader could not parse,
//! or a solid whose bounding box does not have two corners.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_geom::{Deflection, Isometry, Point, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{CannotWrite, Import, StepFile};
use rcad_topo::{Fused, Measurement, Model, ProfileBuilder, Rule, SolidKey};

/// One round trip's outcome: how many fused pieces it added to the tally, or
/// the one reason it could not be compared.
enum Outcome {
    Compared {
        pieces: usize,
        /// Whether `Scale::nominal_size` came back different even though the
        /// solid itself measured the same — see `round_trip_one`.
        scale_metadata_differed: bool,
        /// Why each solid with nothing to compare had nothing to compare.
        no_fuse: Vec<&'static str>,
    },
    Failed(String),
}

#[test]
fn every_kicad_file_that_imports_round_trips_to_the_same_solid() {
    let root = corpus();
    let files = files_under(&root.join("kicad"));
    assert!(
        !files.is_empty(),
        "kicad holds no STEP files, so this test would pass by doing nothing"
    );

    let mut denominator = 0;
    let mut round_tripped: BTreeSet<String> = BTreeSet::new();
    let mut pieces_compared = 0;
    let mut no_fuse_to_compare = 0;
    let mut scale_metadata_differed = 0;
    let mut failures: BTreeMap<String, usize> = BTreeMap::new();
    let mut no_fuse_reasons: BTreeMap<&'static str, usize> = BTreeMap::new();

    for path in &files {
        let bytes = fs::read(path).expect("a file the walk just listed");
        let file = StepFile::read(&bytes).expect("every file in the corpus parses");
        let mut log = ProvenanceLog::new();
        let Ok(import) = Import::read(&file, &Policy::certified(), &mut log) else {
            continue;
        };
        denominator += 1;

        match round_trip_one(&name_of(path), import.model()) {
            Outcome::Compared {
                pieces,
                scale_metadata_differed: differed,
                no_fuse,
            } => {
                round_tripped.insert(name_of(path));
                if pieces > 0 {
                    pieces_compared += pieces;
                } else {
                    no_fuse_to_compare += 1;
                }
                if differed {
                    scale_metadata_differed += 1;
                }
                for because in no_fuse {
                    *no_fuse_reasons.entry(because).or_default() += 1;
                }
            }
            Outcome::Failed(reason) => {
                *failures.entry(canonical(&reason)).or_default() += 1;
            }
        }
    }

    println!(
        "corpus round trip: {} / {denominator} imported KiCad files round-tripped \
         to the same solid, {pieces_compared} fused piece(s) compared, {no_fuse_to_compare} had \
         no fuse to compare, {scale_metadata_differed} matched as a solid despite \
         Scale::nominal_size changing",
        round_tripped.len()
    );
    for (because, count) in &no_fuse_reasons {
        println!("    {count:>4}  no fuse to compare: {because}");
    }
    let mut reasons: Vec<_> = failures.iter().collect();
    reasons.sort_by_key(|&(name, count)| (std::cmp::Reverse(*count), name.clone()));
    for (reason, count) in reasons {
        println!("    {count:>4}  {reason}");
    }

    assert_eq!(
        denominator, 97,
        "the count of KiCad files that import must not shift without somebody saying so"
    );
    assert_eq!(
        scale_metadata_differed, 0,
        "Scale::nominal_size is set from the file's vertices alone, so a round trip must not \
         move it even when a LINE's own point or a PLANE's placement origin rounds"
    );
    assert_pinned_round_trips(&round_tripped);
}

/// The 45 imported KiCad files this kernel can write back out and read back as
/// the same solid, out of the 97 that import.
///
/// **Pinned by name, not by rate**, for the reason `KICAD_FILES_THAT_IMPORT` is
/// (§44.10): a rate cannot say which file moved. Regenerate it from the failing
/// assertion, which prints what appeared and what disappeared.
///
/// # The 52 that are missing are the writer's gap and not the reader's
///
/// Every one of them fails as *writer refuses: an edge between a plane and a
/// pipe names no curve* — 49 of those, two more for a pipe against a pipe, and
/// one for a plane against a ball. A file only reaches here **after** importing
/// and passing the full sweep, so the solid is sound and the kernel simply has
/// no `SURFACE_CURVE` to write for the circle where a flat face meets a round
/// one.
///
/// **That gap is older than this list, and neither the curved rounding nor the
/// two-step wire created it.** At `d23c700` this run said 45 of 46 with one such
/// refusal; it now says 45 of 97 with 52. So **this list is exactly the old 46
/// less `Relay_DPDT_Finder_30.22.step`, which already failed**, nothing that
/// used to round trip stopped, and every file the two later lanes brought in —
/// the curved rounding's fifty and the two-step wire's one — runs into the same
/// writer. Closing that is a writer lane and not this list's business. What this
/// list is for is that the 45 which do round trip cannot quietly become 44.
const KICAD_FILES_THAT_ROUND_TRIP: &[&str] = &[
    "LED_Rectangular_W3.0mm_H2.0mm.step",
    "LED_Rectangular_W3.9mm_H1.8mm.step",
    "LED_Rectangular_W3.9mm_H1.9mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O1.27mm_Z1.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O1.27mm_Z3.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O1.27mm_Z5.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O3.81mm_Z1.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O3.81mm_Z3.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O3.81mm_Z5.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O6.35mm_Z1.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O6.35mm_Z3.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O6.35mm_Z5.0mm.step",
    "LED_Rectangular_W5.0mm_H5.0mm.step",
    "PinHeader_1x04_P2.54mm_Vertical.step",
    "PinHeader_1x11_P2.54mm_Vertical.step",
    "PinHeader_1x18_P2.54mm_Vertical.step",
    "PinHeader_1x25_P2.54mm_Vertical.step",
    "PinHeader_1x32_P2.54mm_Vertical.step",
    "PinHeader_1x39_P2.54mm_Vertical.step",
    "PinHeader_2x01_P2.54mm_Vertical.step",
    "PinHeader_2x08_P2.54mm_Vertical.step",
    "PinHeader_2x15_P2.54mm_Vertical.step",
    "PinHeader_2x22_P2.54mm_Vertical.step",
    "PinHeader_2x29_P2.54mm_Vertical.step",
    "PinHeader_2x36_P2.54mm_Vertical.step",
    "Relay_DPDT_Omron_G5V-2.step",
    "Relay_SPDT_HJR-4102.step",
    "Relay_SPDT_Omron_G6E.step",
    "Relay_SPDT_Omron_G6EK.step",
    "Relay_SPDT_StandexMeder_SIL_Form1C.step",
    "Relay_SPST_StandexMeder_SIL_Form1A.step",
    "Relay_SPST_StandexMeder_SIL_Form1B.step",
    "Relay_StandexMeder_UMS.step",
    "SW_DIP_SPSTx02_Piano_CTS_Series194-2MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx03_Piano_CTS_Series194-3MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx04_Piano_CTS_Series194-4MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx05_Piano_CTS_Series194-5MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx06_Piano_CTS_Series194-6MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx07_Piano_CTS_Series194-7MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx08_Piano_CTS_Series194-8MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx09_Piano_CTS_Series194-9MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx10_Piano_CTS_Series194-10MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx11_Piano_CTS_Series194-11MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx12_Piano_CTS_Series194-12MSTN_W7.62mm_P2.54mm.step",
];

/// Every KiCad file that round-trips must be one of
/// [`KICAD_FILES_THAT_ROUND_TRIP`], and every one of those must round trip —
/// checked as a set so a failure names what appeared and what disappeared.
fn assert_pinned_round_trips(round_tripped: &BTreeSet<String>) {
    let pinned: BTreeSet<&str> = KICAD_FILES_THAT_ROUND_TRIP.iter().copied().collect();
    let got: BTreeSet<&str> = round_tripped.iter().map(String::as_str).collect();
    let newly_round_tripping: Vec<&str> = got.difference(&pinned).copied().collect();
    let stopped_round_tripping: Vec<&str> = pinned.difference(&got).copied().collect();
    assert!(
        newly_round_tripping.is_empty() && stopped_round_tripping.is_empty(),
        "the kicad files that round trip moved (docs/decisions.md §44.10): \
         {} file(s) now round trip and are not pinned: {newly_round_tripping:?}; \
         {} pinned file(s) stopped: {stopped_round_tripping:?}",
        newly_round_tripping.len(),
        stopped_round_tripping.len()
    );
}

/// Write `source` out, read it back, and compare every question this kernel
/// can ask of it against the original — bit for bit.
fn round_trip_one(name: &str, source: &Model) -> Outcome {
    let called = name.trim_end_matches(".step").trim_end_matches(".stp");
    let policy = Policy::certified();

    let text = match rcad_step::write(source, called) {
        Ok(text) => text,
        Err(why) => return Outcome::Failed(format!("writer refuses: {}", grouped_write(&why))),
    };
    let file = match StepFile::read(text.as_bytes()) {
        Ok(file) => file,
        Err(why) => panic!("{name}: this writer wrote a file its own reader cannot parse: {why}"),
    };
    if !file.references_that_point_nowhere().is_empty() {
        panic!("{name}: this writer wrote a reference to an entity it never wrote");
    }

    let mut log = ProvenanceLog::new();
    let there = match Import::read(&file, &policy, &mut log) {
        Ok(import) => import,
        Err(why) => {
            return Outcome::Failed(format!(
                "round trip: the reader refuses the file this writer wrote: {}",
                first_line(&format!("{why}"))
            ));
        }
    };
    let there = there.model();

    // `Scale::nominal_size` is the largest coordinate any of the file's
    // vertices carries (`Import::biggest_coordinate_in`). It used to also
    // walk points that name no vertex at all — a `LINE`'s own defining point,
    // a general-normal `PLANE`'s placement origin, rounded by a few ULPs on
    // the way out — and ten corpus files came back with a moved
    // `Scale::nominal_size` for a reason that was never the solid. Fixed;
    // this is still measured and reported rather than assumed, and the two
    // sides are still measured with **one** `Deflection`, taken from
    // `source`, so the comparison below is not asking the two a different
    // question.
    let scale_metadata_differed = there.scale() != source.scale();
    if let Err(why) = there.validate(&policy, &mut log) {
        panic!("{name}: the round trip came back as a model that is not sound: {why}");
    }

    let source_solids: Vec<SolidKey> = source.solids().map(|solid| solid.key()).collect();
    let there_solids: Vec<SolidKey> = there.solids().map(|solid| solid.key()).collect();
    if source_solids.len() != there_solids.len() {
        return Outcome::Failed(format!(
            "round trip: {} solid(s) became {} after the round trip",
            source_solids.len(),
            there_solids.len()
        ));
    }

    let deflection = Deflection::for_scale(source.scale());
    let mut pieces = 0;
    let mut no_fuse: Vec<&'static str> = Vec::new();
    for (&here_key, &back_key) in source_solids.iter().zip(&there_solids) {
        let here = match source
            .solid(here_key)
            .measure(deflection, &policy, &mut log)
        {
            Ok(measured) => measured,
            Err(why) => return Outcome::Failed(format!("does not measure: {why}")),
        };
        let back = match there.solid(back_key).measure(deflection, &policy, &mut log) {
            Ok(measured) => measured,
            Err(why) => {
                return Outcome::Failed(format!("does not measure after the round trip: {why}"));
            }
        };
        if here.volume() != back.volume() || here.surface_area() != back.surface_area() {
            return Outcome::Failed("round trip: volume or area differs".to_owned());
        }
        if here.centroid() != back.centroid() {
            return Outcome::Failed("round trip: centroid differs".to_owned());
        }

        match fuse_against_a_third_solid(source, here_key, there, back_key, deflection) {
            FuseComparison::BothAgree { compared } => pieces += compared,
            FuseComparison::NothingToCompare { because } => no_fuse.push(because),
            FuseComparison::Disagree(reason) => return Outcome::Failed(reason),
        }
    }
    Outcome::Compared {
        pieces,
        scale_metadata_differed,
        no_fuse,
    }
}

/// What comparing two fuse answers against the same third solid came to.
enum FuseComparison {
    /// Both sides fused, and every piece agreed. `compared` is how many.
    BothAgree { compared: usize },
    /// Neither side had a fuse to compare, and this is why — a bounding box
    /// too thin to cut, or the fuse refusing the same way on both sides
    /// (still an agreement, just not one with pieces to measure).
    NothingToCompare { because: &'static str },
    /// The two sides disagreed, with the reason.
    Disagree(String),
}

/// Cut `here_key` (of `source`) and `back_key` (of `there`) by the same third
/// solid — sized from `here_key`'s own bounding box, so it means the same
/// thing on both sides — and compare the two `Rule::Difference` answers piece
/// by piece, bit for bit.
///
/// Done on clones, same as `a_solid_written_out_and_read_back_is_the_same_solid.rs`'s
/// own `cut_by_a_third_solid`, so the two runs cannot see each other's answer.
fn fuse_against_a_third_solid(
    source: &Model,
    here_key: SolidKey,
    there: &Model,
    back_key: SolidKey,
    deflection: Deflection,
) -> FuseComparison {
    let Some((low, high)) = source.solid(here_key).box_around_it() else {
        return FuseComparison::NothingToCompare {
            because: "the solid has no bounding box",
        };
    };
    let policy = Policy::certified();

    let cut_here = cut_by_a_centered_box(source, here_key, low, high, deflection);
    let cut_back = cut_by_a_centered_box(there, back_key, low, high, deflection);
    match (cut_here, cut_back) {
        (Ok(None), Ok(None)) => FuseComparison::NothingToCompare {
            because: "the bounding box is too thin along an axis to cut",
        },
        (Ok(Some(here)), Ok(Some(back))) => {
            if here.len() != back.len() {
                return FuseComparison::Disagree(
                    "round trip: the two fuses came back with different numbers of pieces"
                        .to_owned(),
                );
            }
            for (one, other) in here.iter().zip(&back) {
                if one.volume() != other.volume()
                    || one.surface_area() != other.surface_area()
                    || one.centroid() != other.centroid()
                {
                    return FuseComparison::Disagree(
                        "round trip: a fused piece differs after the round trip".to_owned(),
                    );
                }
            }
            FuseComparison::BothAgree {
                compared: here.len(),
            }
        }
        (Ok(Some(_)), Ok(None)) | (Ok(None), Ok(Some(_))) => FuseComparison::Disagree(
            "round trip: one side left a solid too thin along an axis to cut and the other did \
             not"
            .to_owned(),
        ),
        (Err(before), Err(after))
            if canonical(&first_line(&format!("{before}")))
                == canonical(&first_line(&format!("{after}"))) =>
        {
            let _ = policy;
            FuseComparison::NothingToCompare {
                because: "the fuse refuses the same way on both sides",
            }
        }
        (Err(before), Err(after)) => FuseComparison::Disagree(format!(
            "round trip: both sides refuse to fuse, but for different reasons: {} vs {}",
            first_line(&format!("{before}")),
            first_line(&format!("{after}"))
        )),
        (Ok(_), Err(why)) => FuseComparison::Disagree(format!(
            "round trip: fuses before the round trip, refuses after: {}",
            first_line(&format!("{why}"))
        )),
        (Err(why), Ok(_)) => FuseComparison::Disagree(format!(
            "round trip: refuses to fuse before the round trip, fuses after: {}",
            first_line(&format!("{why}"))
        )),
    }
}

/// Cut `which` by a box occupying the middle half of its own bounding box
/// along every axis, and measure every piece — or `Ok(None)` when the
/// bounding box is too thin along some axis to build a cutter from at all,
/// which is the same non-claim on both sides of the round trip.
fn cut_by_a_centered_box(
    model: &Model,
    which: SolidKey,
    low: Point,
    high: Point,
    deflection: Deflection,
) -> rcad_topo::Result<Option<Vec<Measurement>>> {
    let policy = Policy::certified();
    let size = [0, 1, 2].map(|axis| (high.coordinates()[axis] - low.coordinates()[axis]) * 0.5);
    if size.iter().any(|&s| s <= 0.0) {
        return Ok(None);
    }
    let at = Vector::new(
        low.coordinates()[0] + size[0] * 0.5,
        low.coordinates()[1] + size[1] * 0.5,
        low.coordinates()[2] + size[2] * 0.5,
    );

    let position = model
        .solids()
        .position(|solid| solid.key() == which)
        .expect("the solid is in the model it came from");
    let mut scratch = model.clone();
    let keep = scratch
        .solids()
        .nth(position)
        .expect("a clone holds the same solids in the same order")
        .key();
    let knife = a_block(&mut scratch, size, at);

    let fused: Fused = scratch.fuse(
        &[keep],
        &[knife],
        Rule::Difference,
        &policy,
        &mut ProvenanceLog::new(),
    )?;
    let pieces: Vec<SolidKey> = fused.solids().to_vec();
    let measured = pieces
        .into_iter()
        .map(|piece| {
            scratch
                .solid(piece)
                .measure(deflection, &policy, &mut ProvenanceLog::new())
        })
        .collect::<rcad_topo::Result<Vec<_>>>()?;
    Ok(Some(measured))
}

/// A box of the given lengths, with its low corner at `at`.
fn a_block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// The name a `CannotWrite` is grouped under.
fn grouped_write(why: &CannotWrite) -> String {
    match why {
        CannotWrite::TheModelHasNoSolid => "the model has no solid".to_owned(),
        CannotWrite::ASolidWithNoShell => "a solid has no shell".to_owned(),
        CannotWrite::ASolidWithMoreThanOneShell { shells } => {
            format!("a solid has {shells} shells")
        }
        CannotWrite::AWireThatStandsStillAtAPole => "a wire stands still at a pole".to_owned(),
        CannotWrite::ACurveTheFacesEitherSideOfItDoNotName { first, second, .. } => {
            format!("an edge between a {first} and a {second} names no curve")
        }
        CannotWrite::AnEdgeThatIsNotWalkedTwice { walks } => {
            format!("an edge is walked {walks} times")
        }
        CannotWrite::ARunKindThisWriterHasNoCurveFor => {
            "a run kind this writer has no curve for".to_owned()
        }
        CannotWrite::ASurfaceThisWriterHasNoEntityFor => {
            "a surface kind this writer has no entity for".to_owned()
        }
        CannotWrite::ALengthUnitThisWriterHasNoSpellingFor => {
            "a unit this writer has no spelling for".to_owned()
        }
        // `CannotWrite` is `#[non_exhaustive]` and grows; a new variant that is
        // not named here would otherwise be counted under somebody else's name.
        _ => format!("unnamed: {why}"),
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or(text).to_owned()
}

/// `text` with every run of digits (and the `.`/`-` that hang off one) folded
/// to a single `#`, so two reasons that differ only in which vertex or by how
/// many coordinate spacings group under one name instead of one each.
fn canonical(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            out.push('#');
            while chars
                .peek()
                .is_some_and(|next| next.is_ascii_digit() || *next == '.' || *next == 'e')
            {
                chars.next();
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!(
            "RCAD_STEP_CORPUS is not set. Point it at a directory holding kicad/ (see this \
             file's header) — this test does not skip on a missing corpus."
        )
    });
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "RCAD_STEP_CORPUS points at {}, which is not a directory",
        root.display()
    );
    root
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
