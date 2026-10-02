//! A half bend and a quarter bend on one torus, fused under every rule in both
//! orders (`docs/decisions.md` §377).
//!
//! Every fuse here refused until the half bend's wall could be followed. Its
//! two caps are stored on the one plane `y = 0`, so the wall is trimmed by one
//! plane along both of its circles, every corner of the wall is on that plane,
//! and `holds.rs` found no corner to read the wall's side off.
//! `classify_uncertain_point` answered `CannotFollowAFacesTrimming` on the wall
//! at every point, the centres of its own caps too (§232.6), and every fuse
//! stopped on "a point in a piece that every solid could place".
//!
//! The ring is four and the tube one, so the whole ring holds `2π²·4·1 = 8π²`
//! and each quarter of it `2π²`. Every answer is held to that, within what
//! `measure` says it can be out by, and passes `Model::validate`.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Scale, Surface};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, OnAFace, Rule, SolidKey};

mod rings;

use rings::a_bend_turned_and_written_out;

/// What one fuse comes to.
#[derive(Clone, Copy, Debug)]
enum Answer {
    /// No solid.
    Nothing,
    /// One solid holding this many quarters of the ring.
    Quarters(u32),
    /// Refused with `ResultIsNotManifold`, which `fuse/assemble.rs` raises
    /// once the pieces are chosen; not traced (§377).
    NotManifold,
}

/// A quarter bend inside the half bend, at either end of it.
///
/// The quarter turned `0` runs from `+x` to `+y`, and turned `1` from `+y` to
/// `−x`; the half bend runs from `+x` round through `+y` to `−x`. So the
/// intersection is the quarter, the half less the quarter is the other quarter,
/// and the quarter less the half is nothing. The union is the half bend, and
/// is refused.
#[test]
fn a_quarter_bend_inside_a_half_bend_fuses_to_the_quarters_it_should() {
    for quarter_turned in [0, 1] {
        for (rule, half_first, answer) in [
            (Rule::Intersection, true, Answer::Quarters(1)),
            (Rule::Intersection, false, Answer::Quarters(1)),
            (Rule::Difference, true, Answer::Quarters(1)),
            (Rule::Difference, false, Answer::Nothing),
            (Rule::Union, true, Answer::NotManifold),
            (Rule::Union, false, Answer::NotManifold),
        ] {
            a_fuse_comes_to(quarter_turned, half_first, rule, answer);
        }
    }
}

/// A quarter bend against one of the half bend's caps, on its other side.
///
/// The quarter turned `2` runs from `−x` to `−y` and turned `3` from `−y` to
/// `+x`, so each shares one cap's disk with the half bend and nothing else.
/// The intersection is nothing and the quarter less the half is the quarter.
/// The half less the quarter is the half bend and the union is three quarters
/// of the ring, and both are refused.
#[test]
fn a_quarter_bend_against_a_half_bends_cap_fuses_to_the_quarters_it_should() {
    for quarter_turned in [2, 3] {
        for (rule, half_first, answer) in [
            (Rule::Intersection, true, Answer::Nothing),
            (Rule::Intersection, false, Answer::Nothing),
            (Rule::Difference, false, Answer::Quarters(1)),
            (Rule::Difference, true, Answer::NotManifold),
            (Rule::Union, true, Answer::NotManifold),
            (Rule::Union, false, Answer::NotManifold),
        ] {
            a_fuse_comes_to(quarter_turned, half_first, rule, answer);
        }
    }
}

/// The half bend's wall holds the half of the ring at `y > 0`, and the points
/// §232.6 tried are placed.
///
/// `(±4, 0, 0)` are the centres of the half bend's two caps, which is where
/// the fuse looked for a point inside each cap. They are not on the ring, so
/// the wall answers them off the one sign it has, the plane `y = 0`'s, and
/// says boundary; the solid puts them on its skin. The other points asked of
/// the wall are on the ring.
#[test]
fn a_half_bends_wall_holds_the_half_and_its_caps_centres_are_on_its_skin() {
    let policy = Policy::certified();
    let mut model = a_model();
    let half =
        a_bend_turned_and_written_out(&mut model, 4.0, 1.0, 2, 0).expect("a half bend is a solid");
    let solid = model.solid(half).expect("this model made the key");
    let wall = solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Torus(_)))
        .expect("a bend has a wall")
        .key();
    for (at, should) in [
        ([4.0, 0.0, 0.0], OnAFace::OnItsBoundary),
        ([-4.0, 0.0, 0.0], OnAFace::OnItsBoundary),
        ([5.0, 0.0, 0.0], OnAFace::OnItsBoundary),
        ([-3.0, 0.0, 0.0], OnAFace::OnItsBoundary),
        ([0.0, 5.0, 0.0], OnAFace::Held),
        ([-3.0, 4.0, 0.0], OnAFace::Held),
        ([0.0, -5.0, 0.0], OnAFace::NotHeld),
        ([3.0, -4.0, 0.0], OnAFace::NotHeld),
    ] {
        let held = solid
            .face_holds_the_point(
                wall,
                PredicatePoint::from(at),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{at:?}: {why:?}"));
        assert_eq!(held, should, "{at:?}");
    }
    for (at, should) in [
        ([4.0, 0.0, 0.0], Containment::OnBoundary),
        ([-4.0, 0.0, 0.0], Containment::OnBoundary),
        ([0.0, 4.0, 0.0], Containment::Inside),
        ([0.0, -4.0, 0.0], Containment::Outside),
    ] {
        let placed = solid.classify_uncertain_point(
            PredicatePoint::from(at),
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert_eq!(placed, should, "{at:?}");
    }
}

/// Fuse the half bend with a quarter bend turned `quarter_turned` quarter
/// turns, in the order `half_first` says, and check it comes to `answer`.
fn a_fuse_comes_to(quarter_turned: u32, half_first: bool, rule: Rule, answer: Answer) {
    let policy = Policy::certified();
    let mut model = a_model();
    let half =
        a_bend_turned_and_written_out(&mut model, 4.0, 1.0, 2, 0).expect("a half bend is a solid");
    let quarter = a_bend_turned_and_written_out(&mut model, 4.0, 1.0, 1, quarter_turned)
        .expect("a quarter bend is a solid");
    let (one, other) = if half_first {
        (half, quarter)
    } else {
        (quarter, half)
    };
    let named = format!(
        "{rule:?} with the half bend {} and the quarter turned {quarter_turned}",
        if half_first { "first" } else { "second" }
    );
    let fused = model.fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new());
    let quarters = match (answer, fused) {
        (Answer::NotManifold, Err(Error::ResultIsNotManifold { .. })) => return,
        (Answer::Nothing, Ok(fused)) => {
            assert!(fused.solids().is_empty(), "{named}: {fused:?}");
            return;
        }
        (Answer::Quarters(quarters), Ok(fused)) => {
            let &[solid] = fused.solids() else {
                panic!("{named}: {fused:?}");
            };
            (solid, quarters)
        }
        (answer, fused) => panic!("{named} should come to {answer:?}, and came to {fused:?}"),
    };
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{named}: the answer does not validate: {why:?}"));
    holds_quarters_of_the_ring(&model, quarters, &named);
}

/// Whether `solid` holds `quarters` quarters of the ring, `2π²` each by Pappus,
/// within what `measure` says it can be out by.
fn holds_quarters_of_the_ring(model: &Model, (solid, quarters): (SolidKey, u32), named: &str) {
    let measured = model
        .solid(solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::of(0.01).expect("a hundredth of the tube"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{named}: the answer does not measure: {why:?}"));
    let pappus = f64::from(quarters) * 2.0 * PI * PI;
    assert!(
        (measured.volume() - pappus).abs() <= measured.volume_error(),
        "{named} holds {pappus}, and measures {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}
