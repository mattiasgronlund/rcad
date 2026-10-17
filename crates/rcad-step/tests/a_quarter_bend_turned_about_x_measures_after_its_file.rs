//! A quarter bend of ring 8 and tube 1 about `z`, from `(7, 0, 0)` round to
//! `(0, 7, 0)`, turned about `x`, written to a file, read back and measured.
//! Queue row Q1344.
//!
//! Read back, the bend's seam reads its points an ulp either side of `π`, so
//! one quarter of its wall reached `1.5707963267948974` round against a
//! widest of `1.5707963267948966` and was cut again, from one corner of the
//! seam to the other. The half between that cut and the seam was `1.8e-15`
//! wide, the triangulator left most of it empty, and at a deflection of 1.8
//! the read-back bend was refused `TwoPatchesOfTheMeshDrewTheSameSide {
//! drawn: 1, used: 1 }` at 20 of the 3600 tenth-degree turns — 27.1, 38.6,
//! 44.9, 47, 47.8, 120.4, 136.2, 137, 138.1, 140.4, 143.6, 206.6 and on —
//! while the placed bend measured `26.320328572829254` at 27.1. At a
//! deflection of 2 the turn of 137 degrees was refused `CornersCoincide` by
//! the same sliver. It is solid 4 of the writer fuzz file `crash-df71416e` of
//! 2026-10-06. `BuildingAMesh::cut_in_two` in `rcad-topo`'s `mesh.rs` says why
//! a piece a rounding over a quarter turn is no longer cut.
//!
//! `a_quarter_bend` is the one `two_quarter_bends_on_one_torus.rs` in
//! `rcad-topo` builds, at this ring and tube.
//!
//! **Q1351 asked whether `HOW_FAR_A_READ_LONGITUDE_CAN_BE_OFF`
//! (`mesh.rs:622-623`, `8.9e-5` rad) covers a read-back bend's longitude
//! error once the bend is moved far from the origin.** A sweep moved this
//! bend out by `1e3`, `1e6` and `1e9` along `(1, 1, 1)`, every whole degree
//! about `x`, measured at deflections `0.5`, `1.8` and `2.0` — 3240 runs —
//! and found **no fold and no `CornersCoincide`** anywhere in it: neither the
//! `cut_in_two` route the margin guards nor the
//! `the_most_even_chord_that_stays_inside` route the row named as a second,
//! unrun one, ever fired.
//!
//! - At each of the three offsets, 1077 of the 1080 runs round-trip and
//!   measure the same volume. The other 3 (one per deflection, since the
//!   write step runs before deflection matters) are the turn of 45 degrees,
//!   refused at write time with `ACurveTheFacesEitherSideOfItDoNotName`, the
//!   same limit `write.rs:2850`'s own test pins for Q2184: no centre the
//!   reader could hold on the seam's circle reads back as the plane the
//!   writer cut it with. That is this file's one write-time refusal shape,
//!   not a new one.
//! - At `1e6` and `1e9` that count was once 12 of 1080: 1065 runs at each were
//!   refused on read-back `CannotNameWhichPieceOfACrossingThisIs`, out of
//!   `rcad-geom`'s `walk.rs`, before the longitude margin was ever reached.
//!   The mesher moves the read-back wall in near the origin before it walks
//!   the seam, and the seam's corners kept the rounding of where the file
//!   stored them, a million or a billion out, against a bound that knew only
//!   the circle's radius of 7. `NamedRun::moved_in_from` tells the walk how
//!   far out they were (Q2218), and run again after that the sweep came out
//!   as the first bullet says.
//!
//! So at this ring and tube every run that writes reads back and measures
//! the same out to a billion; the sweep does not say whether it was the
//! longitude margin that held each one.
//!
//! **The row's other half — ring `1000`, tube `0.01`, at the origin — could
//! not be swept at all.** Before any placement, measurement, write or read,
//! `SolidBuilder::build`'s own `Model::validate` call takes `2.1s` at a
//! ring/tube ratio of `1e3` but does not finish in `60s` at ratio `1e4`
//! (regardless of absolute scale — ring `100` and ring `1000` both hang the
//! same way) nor in `3000s` at the row's own ratio `1e5`. The cost tracks the
//! ratio, not the size, and is untraced past that one call; this half of the
//! row is "not measurable at this ratio" rather than a volume comparison,
//! and the blow-up itself is a separate performance question, not this
//! file's margin question.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point,
    Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{DraftStep, Measurement, Model, SolidBuilder, SolidKey};

const RING: f64 = 8.0;
const TUBE: f64 = 1.0;

/// Every tenth of a degree round `x` is swept at the deflection 20 turns were
/// refused at, one quarter turn to a test: all 3600 in one test took 21
/// seconds, past the longest a test outside the slow tier takes.
#[test]
fn every_tenth_of_a_degree_in_the_first_quarter_turn_about_x_measures_after_its_file() {
    every_tenth_of_a_degree_measures_after_its_file(0..900);
}

#[test]
fn every_tenth_of_a_degree_in_the_second_quarter_turn_about_x_measures_after_its_file() {
    every_tenth_of_a_degree_measures_after_its_file(900..1800);
}

#[test]
fn every_tenth_of_a_degree_in_the_third_quarter_turn_about_x_measures_after_its_file() {
    every_tenth_of_a_degree_measures_after_its_file(1800..2700);
}

#[test]
fn every_tenth_of_a_degree_in_the_fourth_quarter_turn_about_x_measures_after_its_file() {
    every_tenth_of_a_degree_measures_after_its_file(2700..3600);
}

fn every_tenth_of_a_degree_measures_after_its_file(tenths: std::ops::Range<u32>) {
    let mut refused = Vec::new();
    for tenth in tenths.clone() {
        let degrees = f64::from(tenth) / 10.0;
        if let Err(why) = the_same_volume_after_its_file(RING, TUBE, Vector::zero(), degrees, 1.8) {
            refused.push(format!("{degrees}: {why}"));
        }
    }
    assert!(
        refused.is_empty(),
        "{} of the {} turns about x from {} degrees did not measure the same after their \
         file at a deflection of 1.8:\n{}",
        refused.len(),
        tenths.len(),
        f64::from(tenths.start) / 10.0,
        refused.join("\n")
    );
}

/// The one turn the same sliver was refused `CornersCoincide` at, rather
/// than drawing a side once.
#[test]
fn the_bend_turned_137_degrees_about_x_measures_after_its_file_at_a_deflection_of_two() {
    the_same_volume_after_its_file(RING, TUBE, Vector::zero(), 137.0, 2.0)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// Moved out by `1e3` along `(1, 1, 1)` and turned `45°` about `x`: the same
/// limit as `write.rs:2850`'s own test (Q2184), reached here through this
/// file's own shape instead of that one's. No centre the reader could hold on
/// the seam's circle reads back as the plane the writer cut it with, at any
/// finite offset. Q1351's own sweep over every whole degree at three
/// deflections saw this refusal 3 times at offset `1e3` (once per deflection,
/// since the write step runs before deflection matters) and 3 more times each
/// at `1e6` and `1e9` — never a second shape of it.
#[test]
fn the_bend_moved_out_by_a_thousand_and_turned_45_degrees_refuses_to_write() {
    let offset = Vector::new(1e3, 1e3, 1e3);
    let why = the_same_volume_after_its_file(RING, TUBE, offset, 45.0, 0.5)
        .expect_err("the seam's own centre search is exhausted at this offset, same as Q2184");
    assert!(
        why.contains("ACurveTheFacesEitherSideOfItDoNotName"),
        "{why}"
    );
}

/// Moved out by `1e6` along `(1, 1, 1)` and turned one degree about `x`, the
/// bend reads back and measures the same. It used to be refused
/// `CannotNameWhichPieceOfACrossingThisIs` (Q2218): the mesher shifts the
/// read-back wall in near the origin before it walks the seam, and the seam's
/// second corner was `1.0133e-10` off the circle of radius 7 it lies on —
/// `0.87` of one rounding at `1e6`, where the file stored it — against a bound
/// of `3.979e-13` that scaled with the circle and not with how far out the
/// corner had been.
#[test]
fn the_bend_moved_out_by_a_million_measures_after_its_file() {
    the_same_volume_after_its_file(RING, TUBE, Vector::new(1e6, 1e6, 1e6), 1.0, 0.5)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The same at `1e9`, where one rounding is a thousand times wider and the
/// circle of radius 9 the cut plane also makes on the ring is still `2` from
/// the seam's corners.
#[test]
fn the_bend_moved_out_by_a_billion_measures_after_its_file() {
    the_same_volume_after_its_file(RING, TUBE, Vector::new(1e9, 1e9, 1e9), 1.0, 0.5)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The bend of `ring` and `tube`, moved by `offset` and turned `degrees`
/// about `x`, measured at `deflection`, then written, read back and measured
/// again: `Err` saying what went wrong unless both measure and the two
/// volumes are within the sum of their stated errors of each other.
fn the_same_volume_after_its_file(
    ring: f64,
    tube: f64,
    offset: Vector,
    degrees: f64,
    deflection: f64,
) -> Result<(), String> {
    let policy = Policy::certified();
    let deflection = Deflection::of(deflection).expect("a positive length");
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bend = a_quarter_bend(&mut model, ring, tube);
    let turn = Isometry::rotation(&Axis::x(), degrees.to_radians()).expect("a turn about an axis");
    let placement = Isometry::translation(offset)
        .expect("a finite offset")
        .compose(&turn);
    model
        .place_solid(bend, &placement)
        .expect("the bend is in the model");
    let here = model
        .solid(bend)
        .expect("this model made the key")
        .measure(deflection, &policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("the placed bend did not measure: {why:?}"))?;

    let text = rcad_step::write(&model, "a quarter bend")
        .map_err(|why| format!("the placed bend did not write: {why:?}"))?;
    let file = StepFile::read(text.as_bytes()).expect("the writer writes Part 21");
    let import = Import::read(&file, &policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("the file did not read back: {why:?}"))?;
    let [back] = import.model().solids().collect::<Vec<_>>()[..] else {
        return Err("the file did not read back as one solid".to_owned());
    };
    let back: Measurement = back
        .measure(deflection, &policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("placed {}, read back {why:?}", here.volume()))?;

    let apart = (back.volume() - here.volume()).abs();
    let allowed = here.volume_error() + back.volume_error();
    if apart > allowed {
        return Err(format!(
            "placed {} and read back {}, {apart} apart where each may miss by {} and {}",
            here.volume(),
            back.volume(),
            here.volume_error(),
            back.volume_error()
        ));
    }
    Ok(())
}

/// A quarter bend of ring `ring` and tube `tube` about `z` at the origin,
/// from the inner equator's point on `x` round to the one on `y`: the wall
/// used forwards, each cap circle walked right-handed about the outward
/// normal of the cap it bounds, and the seam along the inner equator.
fn a_quarter_bend(model: &mut Model, ring: f64, tube: f64) -> SolidKey {
    let skin =
        Surface::Torus(Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring"));
    let mut builder = SolidBuilder::new();
    let start_corner = builder
        .add_vertex(Point::new(ring - tube, 0.0, 0.0))
        .expect("a finite point");
    let end_corner = builder
        .add_vertex(Point::new(0.0, ring - tube, 0.0))
        .expect("a finite point");
    let middle = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let seam = builder
        .add_edge(
            start_corner,
            end_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(middle),
                picked_by: [
                    Picker::InsideAPipe(Cylinder::new(Axis::z(), ring).expect("a radius")),
                    Picker::from(
                        HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                            .expect("a normal with a length"),
                    ),
                ]
                .into_iter()
                .collect(),
            },
        )
        .expect("the seam runs along the inner equator");
    let start_circle = builder
        .add_edge(
            start_corner,
            start_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the ring in a circle through its corner");
    let end_circle = builder
        .add_edge(
            end_corner,
            end_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the ring in a circle through its corner");
    let cap = |builder: &mut SolidBuilder, [x, y]: [f64; 2], edge, turn| {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(x, y, 0.0))
            .expect("a coordinate direction is a unit normal");
        let coedge = builder
            .add_coedge(edge, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap is a plane trimmed by its circle")
    };
    let start_cap = cap(
        &mut builder,
        [0.0, -1.0],
        start_circle,
        Orientation::Forward,
    );
    let end_cap = cap(&mut builder, [-1.0, 0.0], end_circle, Orientation::Reversed);
    let round_the_wall = [
        (start_circle, Orientation::Reversed),
        (seam, Orientation::Forward),
        (end_circle, Orientation::Forward),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    });
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round one cap, along the seam, back round the other");
    let side = builder.add_face(skin, &[wire]).expect("a face on the ring");
    builder
        .add_shell(&[
            (start_cap, Orientation::Forward),
            (end_cap, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bend described this way is a solid")
}
