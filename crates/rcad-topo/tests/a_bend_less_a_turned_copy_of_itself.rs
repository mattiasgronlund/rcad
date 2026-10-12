//! A quarter bend and a copy of it turned an eighth of a turn round the ring,
//! and the points the fuse asks about on the way (`docs/decisions.md` §323).
//!
//! The two bends share their torus. The fuse cuts each bend's wall where the
//! other's caps cross it and has to place every piece against the other bend,
//! and **every point it can offer in a curved piece is a box** — a blend of the
//! piece's corners put back onto the surface. A ray from a box has no exact
//! line, and a ring answers only an exact line, so all three rules refused on
//! "a point in a piece that every solid could place" (§243.5).
//!
//! What answers now is one exact point beside the box, fired from only once no
//! face of the solid is shown to meet the box. And firing from exact points
//! showed the other half of this: a ring's face counted its crossings the wrong
//! way round, which no test had seen because a ray through a bend's wall alone
//! came to `−1` and was read as inside.
//!
//! Every rule now answers, since the bends' walls are used forwards (§361).

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, Rule, SolidKey};

mod rings;

use rings::a_quarter_bend;

fn a_bend_and_a_copy_turned_an_eighth_of_a_turn() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend = a_quarter_bend(&mut model, 4.0, 1.0);
    let turned = a_quarter_bend(&mut model, 4.0, 1.0);
    model
        .place_solid(
            turned,
            &Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_4)
                .expect("an eighth of a turn is a real angle"),
        )
        .expect("this model made the key");
    (model, bend, turned)
}

/// A box the size `select.rs` puts round a point it puts back onto a curved
/// surface: `256 ε` for every unit out from the origin.
fn a_box_round(at: [f64; 3]) -> PredicatePoint {
    let reach = 256.0 * f64::EPSILON * at.iter().fold(1.0f64, |most, each| most.max(each.abs()));
    PredicatePoint::Constructed(
        at.map(|coordinate| Interval::around(coordinate, reach).expect("a finite place")),
    )
}

/// Where a bend of ring four and tube one has its wall, `angle` round the ring
/// from `+x` and `tube_angle` round the tube from the outer equator.
fn on_the_wall(angle: f64, tube_angle: f64) -> [f64; 3] {
    let out = 4.0 + tube_angle.cos();
    [out * angle.cos(), out * angle.sin(), tube_angle.sin()]
}

/// Points against a quarter bend from `+x` to `+y`, each answered as the true
/// answer says.
///
/// `(−1, 3.5, 0)` is past the cap on `x = 0`, in the tube the bend would have
/// if it went on. A ray from it goes in through that cap and out through the
/// wall, and with the wall's crossings counted the wrong way round that came to
/// `−2`, which is not zero and was read as inside. `(3.5, −1, 0)` is the same
/// past the other cap. Every point here has exact coordinates, and the ones
/// whose ray to the first target has an exact step go straight to the ring.
#[test]
fn a_point_past_a_bends_cap_is_outside_it_and_one_in_its_tube_is_inside() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend = a_quarter_bend(&mut model, 4.0, 1.0);
    let solid = model.solid(bend).expect("this model made the key");
    for (at, should) in [
        ([-1.0, 3.5, 0.0], Containment::Outside),
        ([3.5, -1.0, 0.0], Containment::Outside),
        ([-3.0, -3.0, 0.0], Containment::Outside),
        ([1.0, 1.0, 0.0], Containment::Outside),
        ([4.0, 4.0, 0.0], Containment::Outside),
        ([4.0, 1.0, 0.0], Containment::Inside),
        ([1.0, 4.0, 0.5], Containment::Inside),
        ([3.5, 3.5, 0.0], Containment::Inside),
    ] {
        let mut log = ProvenanceLog::new();
        let placed = solid.classify_point(Point::from(at), &policy, &mut log);
        assert_eq!(placed, should, "{at:?}");
        assert!(log.is_empty(), "{at:?} was decided exactly: {log:?}");
    }
}

/// A box that no face of the solid meets is answered, and so is an exact point
/// whose ray has no exact step; a box on the solid's own wall is not, and is
/// `Undecidable`.
///
/// The first two are the very points the fuse offered: the middle of the tube
/// at the turned bend's first cap, which is where a ring crossing that cap's
/// plane was worked out, and a point on the shared torus a fifth of the way
/// round the first bend's wall. The third is a box on the first bend's own
/// wall — it meets the wall, so no one point beside it can speak for it, and
/// the ring's own sign cannot place it, so the ring grazes every ray from it
/// before any is fired, as a ball's wall grazes a box on it. Until Q1763 it
/// was refused `CannotFireARayAtAFacesSurface`, naming the wall
/// (`docs/decisions.md` §323.3).
#[test]
fn a_box_no_face_meets_is_answered_and_one_on_the_wall_is_not() {
    let policy = Policy::certified();
    let (model, bend, turned) = a_bend_and_a_copy_turned_an_eighth_of_a_turn();
    let bend = model.solid(bend).expect("this model made the key");
    let turned = model.solid(turned).expect("this model made the key");
    let middle_of_the_tube = 4.0 * core::f64::consts::FRAC_1_SQRT_2;

    let the_cap_on_the_turned_bend = [middle_of_the_tube, middle_of_the_tube, 0.0];
    let exactly_there = PredicatePoint::from(the_cap_on_the_turned_bend);
    for point in [exactly_there, a_box_round(the_cap_on_the_turned_bend)] {
        let placed = bend.classify_uncertain_point(point, &policy, &mut ProvenanceLog::new());
        assert_eq!(placed, Containment::Inside, "{point:?}");
    }

    let a_fifth_of_the_way_round = a_box_round(on_the_wall(core::f64::consts::PI / 10.0, 0.0));
    let placed = turned.classify_uncertain_point(
        a_fifth_of_the_way_round,
        &policy,
        &mut ProvenanceLog::new(),
    );
    assert_eq!(
        placed,
        Containment::Outside,
        "the turned bend starts an eighth of a turn round"
    );

    let placed =
        bend.classify_uncertain_point(a_fifth_of_the_way_round, &policy, &mut ProvenanceLog::new());
    assert!(
        matches!(placed, Containment::Undecidable { .. }),
        "a box on the bend's own wall: {placed:?}"
    );
}

/// Every rule answers one solid that validates and holds what Pappus says.
///
/// A disk of radius one carried once round a circle of radius four holds
/// `π · 2π · 4 = 8π²`, so an eighth of the ring holds `π²`. The difference is
/// the eighth from the first bend's cap on `y = 0` to the turned bend's first
/// cap; the intersection is the eighth from there to the first bend's second
/// cap; the union is the three eighths from `y = 0` round to the turned bend's
/// second cap. The union keeps its wall in three faces, one per piece of the
/// two walls, because nothing joins pieces of a torus into one face yet.
///
/// All three used to stop writing the kept pieces back out,
/// `TheFuseWroteABrokenAnswer { CoedgesRunTheSameWay }`, as a bend and a
/// bit-for-bit copy of itself did (§243.5), until the bends' walls were used
/// forwards (§361).
#[test]
fn a_bend_and_a_copy_turned_an_eighth_of_a_turn_fuse_under_every_rule() {
    let an_eighth_of_the_ring = PI * PI;
    for (rule, eighths, faces) in [
        (Rule::Difference, 1.0, 3),
        (Rule::Intersection, 1.0, 3),
        (Rule::Union, 3.0, 5),
    ] {
        let policy = Policy::certified();
        let (mut model, bend, turned) = a_bend_and_a_copy_turned_an_eighth_of_a_turn();
        let answer = model
            .fuse(&[bend], &[turned], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?}: {why:?}"));
        let &[solid] = answer.solids() else {
            panic!("{rule:?} answers {:?}", answer.solids());
        };
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?}: {why:?}"));
        let solid = model.solid(solid).expect("the fuse made the key");
        assert_eq!(solid.faces().count(), faces, "{rule:?}");
        let measured = solid
            .measure(
                Deflection::of(0.01).expect("a hundredth of the tube"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?} does not measure: {why:?}"));
        let holds = eighths * an_eighth_of_the_ring;
        assert!(
            (measured.volume() - holds).abs() <= measured.volume_error(),
            "{rule:?} holds {holds}, and measures {} give or take {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}
