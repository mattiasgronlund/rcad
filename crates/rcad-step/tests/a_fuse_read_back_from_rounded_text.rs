//! A configuration written out, its numbers rounded the way a real exporter
//! rounds them, and read back, fuses to the same answer — or is refused, and
//! the refusal is counted.
//!
//! `docs/decisions.md` §191.
//!
//! # Why the rounding is put in by hand
//!
//! Three KiCad findings — §124's LED dome, §130–§136's pin-header corner at
//! `derive_a_corner`, and §176 — came from numbers an exporter wrote a few ulps
//! off the exact value: an axis tilted by `1.7e-18`, a fourth plane ten ulps
//! off a corner. None of the generators in `rcad-topo/tests/pairs` ever makes
//! that, because every family is built out of whole numbers and at most one
//! clean rotation.
//!
//! **This crate's own writer cannot make it either.** It prints every real with
//! `{:?}`, Rust's shortest text that reads back as the same `f64`, so writing
//! and reading back is exact — `a_solid_written_out_and_read_back_is_the_same_solid.rs`
//! relies on that bit for bit. So the rounding is injected: the Part 21 text is
//! rewritten with every `REAL` cut to [`DIGITS_EXPORTERS_WRITE`] significant
//! digits, and that text goes through [`Import`].
//!
//! The digit counts are measured and not chosen. Over the 323 KiCad files in
//! the corpus, 1 629 759 reals, the most digits any real in a file carries is
//! 13 in 251 files and 14 in 72; the long reals are 11 digits (1.96%), 12
//! (5.88%) and 13 (9.71%), and everything else is five digits or fewer — the
//! whole numbers and short decimals a footprint is drawn in. Those files are
//! almost all Open CASCADE's (`FreeCAD` and `kicad StepUp`), which writes about
//! thirteen significant digits in exponent form and twelve decimals in fixed
//! form, and that is where the 11 and 12 come from.
//!
//! [`DIGITS_THAT_READ_BACK_EXACTLY`] is the control row: seventeen significant
//! digits always read back as the same `f64`, and [`with_every_real_rounded`]
//! asserts that at seventeen it changed no number at all. So the control row is
//! the configuration exactly as built, at the scale the file gives it, and it is
//! the baseline every rounded row is compared against — the one difference
//! between the two is the rounding.
//!
//! # What it checks
//!
//! For each drawn configuration and each digit count, under
//! `Policy::certified()` and each [`Rule`]:
//!
//! - **A wrong answer fails the test.** A panic; a reader that hands back a
//!   model [`Model::validate`] refuses; a fuse that answers with a model
//!   `validate` refuses; or two answers whose volumes differ by more than
//!   [`how_far_rounding_can_move_a_volume`] allows.
//! - **A refusal the rounding introduced is counted, not failed** — the control
//!   decided it and the rounded file did not. That count, per family and digit
//!   count, is the number this file exists to print.
//!
//! # What it found, on the fixed seed
//!
//! - A turned pair: 109 of 144 fuses decided from exact text, and 107, 105 and
//!   111 decided at 11, 12 and 13 digits. Rounding introduces 8, 10 and 4
//!   refusals and rescues 6 at each. Before §368 **every one of the 109 was
//!   refused** at all three digit counts, `Undecidable` at "where three
//!   surfaces meet": a gap, shown on its smallest case by
//!   [`a_turned_cube_read_back_from_thirteen_digits_is_united_with_nothing`].
//! - A skew pair: 122 of 144 decided from exact text, and no refusal
//!   introduced at any of the three digit counts. Before §368, 77 of the 122
//!   were refused at 13 digits.
//! - A lattice pair, a bar through a block and a ball and a block: no real
//!   moves at any of the three digit counts, because every number is a whole
//!   one, so those rows are controls and not measurements. The ball and block
//!   decide all 144 fuses at each.
//! - A frustum and a block: written since §366, and all 144 fuses decided from
//!   exact text. The reader refuses the rounded file,
//!   `ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn`, in 48 of 48 at 11 and 12
//!   digits and 40 of 48 at 13. Of the 24 fuses of the eight read at 13
//!   digits, 21 are decided and rounding introduces 3 `Undecidable`.
//! - The writer dropped a solid's placement:
//!   [`a_cube_moved_along_x_is_written_where_it_was_moved_to`], fixed in §191.
//!
//! # What it cannot catch
//!
//! It checks this writer's shapes, rounded, against this reader. A file an
//! exporter writes carries more than rounded numbers: vertices that are not on
//! their surfaces by the exporter's own tolerance, curves given as B-splines,
//! placements. None of that is here. And rounding is applied to every real the
//! same way, so a point shared by two entities stays shared; an exporter that
//! writes the same corner twice with two different roundings is a different
//! file, and the corpus tests are where that is met.

#[path = "../../rcad-topo/tests/cones/mod.rs"]
mod cones;
#[path = "../../rcad-topo/tests/pairs/mod.rs"]
mod pairs;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::panic::{AssertUnwindSafe, catch_unwind};

use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use rcad_geom::{Deflection, Isometry, Scale, Unit, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Model, Rule, SolidKey};

use pairs::{
    BallAndBlock, Configuration, CurvedPair, FrustumAndBlock, a_ball_and_a_block, a_curved_pair,
    a_frustum_and_a_block, a_lattice_pair, a_skew_pair, a_turned_pair,
};

/// The significant digits an Open CASCADE exporter writes a long real with, as
/// counted over the KiCad corpus. The module comment has the counts.
const DIGITS_EXPORTERS_WRITE: [usize; 3] = [11, 12, 13];

/// Seventeen significant digits name every `f64` exactly.
const DIGITS_THAT_READ_BACK_EXACTLY: usize = 17;

/// How many configurations each family draws.
///
/// Every configuration is written once, read back four times, and fused three
/// times per reading plus three times as built — fifteen fuses a case — so the
/// count is what keeps a family near a second.
const CASES: u32 = 48;

/// The three rules every reading is fused under.
const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

#[test]
fn a_lattice_pair_read_back_from_rounded_text() {
    a_family_read_back_from_rounded_text("a lattice pair", a_lattice_pair(), Configuration::build)
        .all_answers_right();
}

#[test]
fn a_turned_pair_read_back_from_rounded_text() {
    a_family_read_back_from_rounded_text("a turned pair", a_turned_pair(), Configuration::build)
        .all_answers_right();
}

#[test]
fn a_skew_pair_read_back_from_rounded_text() {
    a_family_read_back_from_rounded_text("a skew pair", a_skew_pair(), Configuration::build)
        .all_answers_right();
}

#[test]
fn a_block_with_a_bar_through_it_read_back_from_rounded_text() {
    a_family_read_back_from_rounded_text("a curved pair", a_curved_pair(), CurvedPair::build)
        .all_answers_right();
}

/// Every frustum of the family is written and read back, since the writer
/// names the circle where a flat cap meets a cone (`docs/decisions.md` §366).
/// Until then the writer refused all 48 as
/// `ACurveTheFacesEitherSideOfItDoNotName` and nothing reached the reader.
#[test]
fn a_frustum_up_through_a_block_read_back_from_rounded_text() {
    let tally = a_family_read_back_from_rounded_text(
        "a frustum and a block",
        a_frustum_and_a_block(),
        FrustumAndBlock::build,
    );
    tally.all_answers_right();
    assert!(
        tally.not_written.is_empty(),
        "every frustum and block was expected to be written: {:?}",
        tally.not_written
    );
}

/// Every ball of the family is written and read back, since the writer leaves
/// a ball's poles unwritten and the reader puts them back where the seam turns
/// (`a_ball_written_out_and_read_back_is_the_same_ball` in
/// `a_solid_written_out_and_read_back_is_the_same_solid.rs`). Until then the
/// writer refused all 48 at the pole and nothing reached the reader.
///
/// Every number in the family is a whole one, so no real moves at any digit
/// count and this row is a control, like the lattice pair.
#[test]
fn a_ball_and_a_block_read_back_from_rounded_text() {
    let tally = a_family_read_back_from_rounded_text(
        "a ball and a block",
        a_ball_and_a_block(),
        BallAndBlock::build,
    );
    tally.all_answers_right();
    assert!(
        tally.not_written.is_empty(),
        "every ball and block was expected to be written: {:?}",
        tally.not_written
    );
}

/// A cube moved five along `x` is written where it was moved to (§191).
///
/// `rcad_step::write` used to walk each solid's shell by key —
/// `model.shell(key)` in `solid_from_shell` — and a shell looked up by key does
/// not carry the solid's placement, so every face and corner went out unplaced.
/// Every family in this file is built with at least one placement
/// (`place_solid` for a turn, the lift `add_extrusion` takes, the shift
/// `add_round_bar` takes), so every family's file was a different
/// configuration from the one built — a turned pair came back as its lattice
/// pair with no real to round. It corrupted silently rather than refusing.
#[test]
fn a_cube_moved_along_x_is_written_where_it_was_moved_to() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::new(Unit::Millimetre, 7.0).expect("a working size"));
    let cube = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(5.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("this model made the key");
    let built = model
        .solid(cube)
        .and_then(|solid| solid.measure(the_mesh(), &policy, &mut ProvenanceLog::new()))
        .expect("a cube measures");

    let text = rcad_step::write(&model, "a cube moved along x").expect("a cube is written");
    let back = read_back(&text).expect("a cube is read back");
    let [(_, read)] = back.solids() else {
        panic!("one cube came back as {} solids", back.solids().len())
    };
    let read = back
        .model()
        .solid(*read)
        .and_then(|solid| solid.measure(the_mesh(), &policy, &mut ProvenanceLog::new()))
        .expect("a cube measures");
    assert_eq!(
        read.centroid(),
        built.centroid(),
        "the cube was built centred at {:?} and read back centred at {:?}",
        built.centroid(),
        read.centroid()
    );
}

/// **A gap and not a limit** (§16.10), shown on the smallest thing that had
/// it: one turned cube, read back from thirteen-digit text, united with
/// nothing. §368 closed it.
///
/// The answer exists and is known without computing it — a solid united with
/// nothing is itself, and [`Model::validate`] accepts that solid, so it is a
/// sound model the fuse is asked to hand back unchanged. Rounding puts each
/// corner a few ulps off the meet of its three rounded planes. Until §368 the
/// fuse asked the stored corner to agree with that meet within the corner's own
/// rounding alone and refused, `Undecidable` at "where three surfaces meet":
/// §75.6's forbidden direction, validate accepting what the fuse cannot bake,
/// and the shape of §130–§136's pin-header corner. Now the fuse grants the
/// corner the room validate allows it, and hands back the cube as it was
/// read, to within what rounding the file can move a volume by.
#[test]
fn a_turned_cube_read_back_from_thirteen_digits_is_united_with_nothing() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let cube = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let axis = rcad_geom::Axis::new(rcad_geom::Point::origin(), Vector::new(1.0, 1.0, 1.0))
        .expect("a direction with length");
    model
        .place_solid(
            cube,
            &Isometry::rotation(&axis, 0.5).expect("a finite turn"),
        )
        .expect("this model made the key");
    model
        .fuse(
            &[cube],
            &[],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the cube as built unites with nothing");
    let built = model
        .solid(cube)
        .and_then(|solid| solid.measure(the_mesh(), &policy, &mut ProvenanceLog::new()))
        .expect("a cube measures");

    let text = rcad_step::write(&model, "a turned cube").expect("a cube is written");
    let (rounded, moved) = with_every_real_rounded(&text, 13);
    assert!(
        moved > 0,
        "a turned cube has reals thirteen digits cannot hold"
    );
    let back = read_back(&rounded).expect("the rounded cube is read back");
    back.model()
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the rounded cube is a sound model");
    let largest = back.model().scale().nominal_size();
    let (_, turned) = *back.solids().last().expect("the baked cube");
    let mut scratch = back.into_model();
    let answer = scratch
        .fuse(
            &[turned],
            &[],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a solid validate accepts is one the fuse has to be able to bake");

    let [itself] = answer.solids() else {
        panic!(
            "a cube united with nothing is one cube, not {} solids",
            answer.solids().len()
        )
    };
    scratch
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the answer is as sound as the cube read back");
    let united = scratch
        .solid(*itself)
        .and_then(|solid| solid.measure(the_mesh(), &policy, &mut ProvenanceLog::new()))
        .expect("the answer measures");
    let moved = (united.volume() - built.volume()).abs();
    let allowed =
        how_far_rounding_can_move_a_volume(13, largest, built.surface_area() + built.area_error())
            + built.volume_error()
            + united.volume_error();
    assert!(
        moved <= allowed,
        "the cube was built with volume {} and came back {}, {moved:e} apart where rounding can \
         move it {allowed:e}",
        built.volume(),
        united.volume()
    );
}

/// Draw [`CASES`] configurations on a fixed seed, run each through
/// [`one_configuration`], and print what came of them.
///
/// Nothing fails inside the runner: a case that finds a wrong answer records it
/// and the next case runs, so the refusal counts cover the whole draw and are
/// not cut short at the first defect. [`Tally::all_answers_right`] fails the
/// test afterwards, naming every configuration that went wrong.
fn a_family_read_back_from_rounded_text<T: Debug + Clone>(
    called: &str,
    strategy: impl Strategy<Value = T>,
    build: impl Fn(&T) -> (Model, Vec<SolidKey>),
) -> Tally {
    let tally = RefCell::new(Tally::default());
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: CASES,
            failure_persistence: None,
            ..Config::default()
        },
        TestRng::from_seed(RngAlgorithm::ChaCha, &[191; 32]),
    );
    runner
        .run(&strategy, |drawn| {
            let (model, keys) = build(&drawn);
            let went = catch_unwind(AssertUnwindSafe(|| {
                one_configuration(model, &keys, &mut tally.borrow_mut())
            }));
            let mut tally = tally.borrow_mut();
            match went {
                Ok(Ok(())) => {}
                Ok(Err(wrong)) => tally.wrong.push(format!("{drawn:?}: {wrong}")),
                Err(panic) => tally.wrong.push(format!(
                    "{drawn:?}: panicked: {}",
                    panic
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| panic.downcast_ref::<&str>().copied())
                        .unwrap_or("with no message")
                )),
            }
            Ok(())
        })
        .expect("the runner is never failed from inside");
    let tally = tally.into_inner();
    tally.print(called);
    tally
}

/// Write the configuration, read it back at every digit count, and compare every
/// fuse of a rounded reading against the same fuse of the exact one.
///
/// The pair is baked first — each solid united with nothing, which gives a new
/// solid whose faces carry the placement in their own numbers — because the
/// writer used to drop a solid's placement
/// ([`a_cube_moved_along_x_is_written_where_it_was_moved_to`]); the bake is now
/// redundant and costs nothing but two more solids in the file. The file then
/// holds the two built solids unplaced and the two baked ones, and only the
/// baked ones are read.
///
/// # Errors
///
/// A sentence naming the wrong answer. A refusal is never one.
fn one_configuration(mut model: Model, keys: &[SolidKey], tally: &mut Tally) -> Result<(), String> {
    let policy = Policy::certified();
    let [argument, tool] = keys else {
        panic!(
            "every family here is a pair, and this one is {} solids",
            keys.len()
        )
    };
    for rule in RULES {
        let as_built = fused(&model, *argument, *tool, rule)?;
        tally.as_built.count(&as_built);
    }
    let mut baked = Vec::with_capacity(2);
    for key in [*argument, *tool] {
        match model.fuse(&[key], &[], Rule::Union, &policy, &mut ProvenanceLog::new()) {
            Ok(answer) => match answer.solids() {
                [one] => baked.push(*one),
                many => {
                    return Err(format!(
                        "a solid united with nothing came to {} solids",
                        many.len()
                    ));
                }
            },
            Err(refused) => {
                *tally.not_baked.entry(name_of(&refused)).or_default() += 1;
                return Ok(());
            }
        }
    }
    let baked = [baked[0], baked[1]];
    let written = model.solid_count();

    let text = match rcad_step::write(&model, "a configuration") {
        Ok(text) => text,
        Err(refused) => {
            *tally.not_written.entry(name_of(&refused)).or_default() += 1;
            return Ok(());
        }
    };

    let (exact, _) = with_every_real_rounded(&text, DIGITS_THAT_READ_BACK_EXACTLY);
    let exact =
        read_back(&exact).map_err(|refused| format!("the exact text was refused: {refused}"))?;
    let exact_keys = the_baked_pair_in(&exact, written)?;
    exact
        .model()
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("the exact text read back as an unsound model: {why}"))?;
    the_file_holds_the_pair_that_was_built(&model, baked, exact.model(), exact_keys)?;
    let skin = skin_of(exact.model(), &exact_keys)?;
    let largest = exact.model().scale().nominal_size();

    let mut control = Vec::with_capacity(RULES.len());
    for rule in RULES {
        let answer = fused(exact.model(), exact_keys[0], exact_keys[1], rule)?;
        tally.exact.count(&answer);
        control.push(answer);
    }

    for digits in DIGITS_EXPORTERS_WRITE {
        let row = tally.rounded.entry(digits).or_default();
        let (rounded, moved) = with_every_real_rounded(&text, digits);
        row.files += 1;
        row.files_with_a_real_moved += usize::from(moved > 0);
        let rounded = match read_back(&rounded) {
            Ok(rounded) => rounded,
            Err(refused) => {
                *row.reader_refused.entry(refused).or_default() += 1;
                continue;
            }
        };
        let rounded_keys = the_baked_pair_in(&rounded, written)?;
        rounded
            .model()
            .validate(&policy, &mut ProvenanceLog::new())
            .map_err(|why| {
                format!("at {digits} digits the reader handed back an unsound model: {why}")
            })?;
        let bound = how_far_rounding_can_move_a_volume(digits, largest, skin);

        for (rule, control) in RULES.into_iter().zip(&control) {
            let answer = fused(rounded.model(), rounded_keys[0], rounded_keys[1], rule)?;
            row.fuses.count(&answer);
            match (control, &answer) {
                (Ok(exact), Ok(rounded)) => {
                    let moved = (exact.volume - rounded.volume).abs();
                    let allowed = bound + exact.volume_error + rounded.volume_error;
                    row.worst_share_of_the_bound =
                        row.worst_share_of_the_bound.max(moved / allowed);
                    if moved > allowed {
                        return Err(format!(
                            "{rule:?} at {digits} digits: volume {} against {} exactly, \
                             {moved:e} apart where rounding can move it {allowed:e}",
                            rounded.volume, exact.volume
                        ));
                    }
                }
                (Ok(_), Err(refused)) => {
                    *row.introduced.entry(refused.clone()).or_default() += 1;
                }
                (Err(_), Ok(_)) => row.rescued += 1,
                (Err(_), Err(_)) => {}
            }
        }
    }
    Ok(())
}

/// What a fuse came to: its volume and how far the mesh can have it out, or the
/// name of the refusal.
///
/// # Errors
///
/// A wrong answer: an `Ok` whose model [`Model::validate`] refuses, or an
/// answer that does not measure.
fn fused(
    model: &Model,
    argument: SolidKey,
    tool: SolidKey,
    rule: Rule,
) -> Result<Result<Measured, String>, String> {
    let policy = Policy::certified();
    let mut scratch = model.clone();
    let answer = match scratch.fuse(
        &[argument],
        &[tool],
        rule,
        &policy,
        &mut ProvenanceLog::new(),
    ) {
        Ok(answer) => answer,
        Err(refused) => return Ok(Err(name_of(&refused))),
    };
    scratch
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("{rule:?} answered with a model that is not sound: {why}"))?;
    let mut measured = Measured::default();
    for &piece in answer.solids() {
        let it = scratch
            .solid(piece)
            .and_then(|solid| solid.measure(the_mesh(), &policy, &mut ProvenanceLog::new()))
            .map_err(|why| {
                format!("{rule:?} answered with a piece that does not measure: {why}")
            })?;
        measured.volume += it.volume();
        measured.volume_error += it.volume_error();
    }
    Ok(Ok(measured))
}

/// Read a Part 21 text into a model, or the name of what refused it.
fn read_back(text: &str) -> Result<Import, String> {
    let file = StepFile::read(text.as_bytes()).map_err(|refused| name_of(&refused))?;
    Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .map_err(|refused| name_of(&refused))
}

/// The last two solids the file came back with, which are the baked pair: the
/// writer writes a model's solids in the order the model holds them, and the
/// bake added them last.
fn the_baked_pair_in(import: &Import, written: usize) -> Result<[SolidKey; 2], String> {
    match import.solids() {
        [.., (_, argument), (_, tool)] if import.solids().len() == written => {
            Ok([*argument, *tool])
        }
        other => Err(format!("{written} solids came back as {}", other.len())),
    }
}

/// The two solids read back exactly are the two that were built: same volume,
/// same centroid, to within the length the kernel itself calls nothing.
///
/// The file holds every number of the baked solids to the last bit, so what is
/// left between the two measurements is two measurements of one shape at two
/// different [`Scale`](rcad_geom::Scale)s. The bound is the kernel's own
/// `default_linear_tolerance` as `δ`, carried through a volume the way
/// [`how_far_rounding_can_move_a_volume`] carries a rounding.
///
/// # Errors
///
/// When the file is some other pair — which is what the placement defect made
/// of every family before the bake.
fn the_file_holds_the_pair_that_was_built(
    built: &Model,
    built_keys: [SolidKey; 2],
    read: &Model,
    read_keys: [SolidKey; 2],
) -> Result<(), String> {
    let policy = Policy::certified();
    let nothing = read.scale().default_linear_tolerance();
    for (built_key, read_key) in built_keys.into_iter().zip(read_keys) {
        let measure = |model: &Model, key: SolidKey| {
            model
                .solid(key)
                .and_then(|solid| solid.measure(the_mesh(), &policy, &mut ProvenanceLog::new()))
                .map_err(|why| format!("a solid does not measure: {why}"))
        };
        let here = measure(built, built_key)?;
        let there = measure(read, read_key)?;
        let skin = here.surface_area() + here.area_error();
        let volume_allowed = 3.0 * nothing * skin + here.volume_error() + there.volume_error();
        let centroid_allowed = nothing + here.centroid_wobble() + there.centroid_wobble();
        let volume_moved = (here.volume() - there.volume()).abs();
        let centroid_moved = here.centroid().distance_to(there.centroid());
        if volume_moved > volume_allowed || centroid_moved > centroid_allowed {
            return Err(format!(
                "the file is not the pair that was built: volume {} became {}, centroid {:?} \
                 became {:?}",
                here.volume(),
                there.volume(),
                here.centroid(),
                there.centroid()
            ));
        }
    }
    Ok(())
}

/// How much skin the two inputs have between them, counting what the mesh may
/// have missed.
fn skin_of(model: &Model, keys: &[SolidKey; 2]) -> Result<f64, String> {
    let mut skin = 0.0;
    for &key in keys {
        let measured = model
            .solid(key)
            .and_then(|solid| {
                solid.measure(the_mesh(), &Policy::certified(), &mut ProvenanceLog::new())
            })
            .map_err(|why| format!("an input does not measure: {why}"))?;
        skin += measured.surface_area() + measured.area_error();
    }
    Ok(skin)
}

/// How far the volume of any fuse of the two inputs can move when every real in
/// the file is rounded to `digits` significant digits.
///
/// # The derivation
///
/// Rounding to `N` significant digits moves a number `x` by at most `ε|x|`,
/// `ε = ½·10^(1−N)`. Write `L` for the largest vertex coordinate in the file —
/// the model's nominal size, which is what [`Import`] sets it to. In every
/// family here every point of every face lies within `[−2L, 2L]³`: flat solids
/// are the convex hull of their vertices, and a bar's or frustum's circle has
/// its seam vertex at `centre + r` with the centre inside the block, so `r ≤ L`
/// and the far side is within `2L`. So a point on a face is within `2√3·L <
/// 3.5L` of the origin.
///
/// Each surface is written as a placement (a location, a unit axis) and at most
/// one length or angle. How far rounding moves the surface, at any point `q` of
/// the model:
///
/// - the location moves by at most `ε·2√3·L < 3.5εL`;
/// - a unit axis's components move by `ε` each, so it turns by at most `2ε`
///   once renormalised, and that swings `q`, at most `7L` from the location, by
///   `14εL`;
/// - a radius, at most `2L`, moves by `2εL`;
/// - a cone's half-angle, below `π/2`, moves by `επ/2`, which swings a point at
///   most `7L` up the slant by `11εL`.
///
/// So every surface moves by at most `δ = 32εL` (30.5 rounded up), and every
/// vertex by less.
///
/// A fuse is a set operation, and set operations do not grow a difference:
/// `vol((A∘B) Δ (A′∘B′)) ≤ vol(A Δ A′) + vol(B Δ B′)` for union, intersection
/// and difference alike. `A Δ A′` lies within `δ` of the skin of `A`, and that
/// shell's volume is `2δ·area(A)` to first order. The second- and third-order
/// terms (`δ²` times edge length, `δ³` times the corner count) are each below
/// `δ·area` while `δ` is far below the smallest feature, which is one whole unit
/// here and `δ` is under `1e-7` for these digit counts. So the bound is
/// `3δ·(area(A) + area(B))`, with `area` taken as measured plus its own
/// published error.
///
/// Each of the two volumes compared is a mesh sum and carries its own
/// [`Measurement::volume_error`](rcad_topo::Measurement::volume_error); the
/// caller adds both. Nothing here was tuned: no run moved this number.
fn how_far_rounding_can_move_a_volume(digits: usize, largest: f64, skin: f64) -> f64 {
    let exponent = i32::try_from(digits).expect("a handful of digits") - 1;
    let epsilon = 0.5 * 10f64.powi(-exponent);
    let delta = 32.0 * epsilon * largest;
    3.0 * delta * skin
}

/// The Part 21 text with every `REAL` in its data section cut to `digits`
/// significant digits, correctly rounded.
///
/// A `REAL` is the only token with a decimal point that is not inside a string,
/// and this writer never writes one without it. Entity names (`#12`),
/// keywords with digits in them (`AXIS2_PLACEMENT_3D`), enumerations (`.T.`)
/// and strings are copied as they stand.
///
/// Also says how many reals came out as a different `f64`, so a row where
/// rounding happened to change nothing — a lattice pair's whole numbers — is
/// not mistaken for one that survived it.
///
/// # Panics
///
/// At [`DIGITS_THAT_READ_BACK_EXACTLY`], if any real came out as a different
/// `f64` — which would make the control row something other than the
/// configuration as built.
fn with_every_real_rounded(text: &str, digits: usize) -> (String, usize) {
    let data = text
        .find("DATA;")
        .expect("a Part 21 text has a data section");
    let bytes = text.as_bytes();
    let mut moved = 0;
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..data]);
    let mut at = data;
    while at < bytes.len() {
        let here = bytes[at];
        let next = bytes.get(at + 1).copied().unwrap_or(b' ');
        let end = if here == b'\'' {
            at + 2 + text[at + 1..].find('\'').expect("a string that ends")
        } else if here.is_ascii_alphabetic() || here == b'_' || here == b'#' {
            at + 1
                + bytes[at + 1..]
                    .iter()
                    .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_')
                    .count()
        } else if here.is_ascii_digit() || ((here == b'-' || here == b'+') && next.is_ascii_digit())
        {
            let end = the_end_of_a_number(bytes, at);
            let number = &text[at..end];
            if number.contains('.') {
                let (written, changed) = rounded(number, digits);
                out.push_str(&written);
                moved += usize::from(changed);
            } else {
                out.push_str(number);
            }
            at = end;
            continue;
        } else {
            assert!(here.is_ascii(), "outside a string this writer writes ASCII");
            at + 1
        };
        out.push_str(&text[at..end]);
        at = end;
    }
    assert!(
        digits < DIGITS_THAT_READ_BACK_EXACTLY || moved == 0,
        "{moved} reals came out different at seventeen digits"
    );
    (out, moved)
}

/// Where the number starting at `at` ends: a sign, digits, and then a decimal
/// point, digits and an exponent if they are there.
fn the_end_of_a_number(bytes: &[u8], at: usize) -> usize {
    let digits_from = |from: usize| {
        from + bytes[from..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count()
    };
    let sign = |from: usize| usize::from(matches!(bytes.get(from), Some(b'-' | b'+')));
    let mut end = digits_from(at + sign(at));
    if bytes.get(end) == Some(&b'.') {
        end = digits_from(end + 1);
        if bytes.get(end) == Some(&b'E') {
            end = digits_from(end + 1 + sign(end + 1));
        }
    }
    end
}

/// One real, as Part 21 text, correctly rounded to `digits` significant digits,
/// and whether that made it a different `f64`.
fn rounded(number: &str, digits: usize) -> (String, bool) {
    let value: f64 = number
        .parse()
        .unwrap_or_else(|_| panic!("{number} is a real this writer wrote"));
    let written = format!("{value:.*E}", digits - 1);
    let written = if written.contains('.') {
        written
    } else {
        written.replacen('E', ".E", 1)
    };
    let back: f64 = written.parse().expect("a real this function wrote");
    (written, back.to_bits() != value.to_bits())
}

/// The same mesh for every measurement on every side, so two measurements of
/// one shape differ only by what the shape differs by.
fn the_mesh() -> Deflection {
    Deflection::of(0.05).expect("a twentieth of the smallest radius here")
}

/// The name of an error's variant, without the fields — enough to group
/// refusals by kind.
fn name_of(error: &impl Debug) -> String {
    let written = format!("{error:?}");
    written
        .split(|character: char| !character.is_ascii_alphanumeric())
        .next()
        .unwrap_or_default()
        .to_owned()
}

#[derive(Clone, Copy, Debug, Default)]
struct Measured {
    volume: f64,
    volume_error: f64,
}

/// What came of one family's draw.
#[derive(Debug, Default)]
struct Tally {
    as_built: Fuses,
    exact: Fuses,
    not_baked: BTreeMap<String, usize>,
    not_written: BTreeMap<String, usize>,
    rounded: BTreeMap<usize, Row>,
    wrong: Vec<String>,
}

/// One digit count's row.
#[derive(Debug, Default)]
struct Row {
    files: usize,
    files_with_a_real_moved: usize,
    reader_refused: BTreeMap<String, usize>,
    fuses: Fuses,
    /// Decided exactly, refused rounded — the headline.
    introduced: BTreeMap<String, usize>,
    /// Refused exactly, decided rounded.
    rescued: usize,
    worst_share_of_the_bound: f64,
}

#[derive(Debug, Default)]
struct Fuses {
    decided: usize,
    refused: BTreeMap<String, usize>,
}

impl Fuses {
    fn count(&mut self, answer: &Result<Measured, String>) {
        match answer {
            Ok(_) => self.decided += 1,
            Err(refused) => *self.refused.entry(refused.clone()).or_default() += 1,
        }
    }
}

impl Tally {
    fn print(&self, called: &str) {
        println!("{called}: {CASES} configurations, three rules each, Policy::certified()");
        println!(
            "  as built:            {} decided, refused {:?}",
            self.as_built.decided, self.as_built.refused
        );
        if !self.not_baked.is_empty() {
            println!("  the bake refused:    {:?}", self.not_baked);
        }
        if !self.not_written.is_empty() {
            println!("  the writer refused:  {:?}", self.not_written);
        }
        println!(
            "  read back exactly:   {} decided, refused {:?}",
            self.exact.decided, self.exact.refused
        );
        for (digits, row) in &self.rounded {
            let introduced: usize = row.introduced.values().sum();
            println!(
                "  {digits} digits ({} of {} files had a real moved): {} decided, refused {:?}; \
                 {introduced} refusals the rounding introduced {:?}, {} rescued; reader refused \
                 {:?}; worst volume move {:.2e} of the bound",
                row.files_with_a_real_moved,
                row.files,
                row.fuses.decided,
                row.fuses.refused,
                row.introduced,
                row.rescued,
                row.reader_refused,
                row.worst_share_of_the_bound
            );
        }
        for wrong in &self.wrong {
            println!("  WRONG: {wrong}");
        }
    }

    fn all_answers_right(&self) {
        assert!(
            self.wrong.is_empty(),
            "{} wrong answer(s); the first: {}",
            self.wrong.len(),
            self.wrong[0]
        );
    }
}
