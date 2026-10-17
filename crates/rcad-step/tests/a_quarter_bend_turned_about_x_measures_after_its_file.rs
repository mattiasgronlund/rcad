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

#[test]
fn probe_q1351_offset_sweep() {
    let mut by_name: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    let mut folds: Vec<String> = Vec::new();
    let mut total = 0u32;
    for &factor in &[1e3_f64, 1e6, 1e9] {
        let offset = Vector::new(factor, factor, factor);
        for whole in 0..360 {
            let degrees = f64::from(whole);
            for &deflection in &[0.5_f64, 1.8, 2.0] {
                total += 1;
                match the_same_volume_after_its_file(RING, TUBE, offset, degrees, deflection) {
                    Ok(()) => {}
                    Err(why) if why.contains(" apart where each may miss") => {
                        folds.push(format!(
                            "offset {factor} degrees {degrees} deflection {deflection}: {why}"
                        ));
                    }
                    Err(why) => {
                        let name = refusal_name(&why).to_owned();
                        *by_name.entry(name).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    println!(
        "total {total} folds {} by_name {by_name:?}\n{}",
        folds.len(),
        folds.join("\n")
    );
    panic!("probe");
}

fn refusal_name(why: &str) -> &str {
    let after = why.rsplit("why: ").next().unwrap_or(why);
    let after = after.trim_start_matches(|c: char| !c.is_alphabetic());
    after
        .split(|c: char| c == ' ' || c == '{' || c == '(')
        .next()
        .unwrap_or(after)
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
