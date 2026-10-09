//! A half disk plate cut by a whole ring about its axis, and a bar standing in
//! the ring's hole (queue row Q1305).
//!
//! The ring is a whole torus about `z`, ring 4 and tube 1, built as two faces
//! cut along its equators. The plate is the half, `x ≥ 0`, of a disk of radius
//! 11/2 with `z` in `[−2, 1/2]`. The bar is a disk of radius 23/8 with `z` in
//! `[1/4, 3/4]`, on the same axis. The plate's top, `z = 1/2`, meets the ring in
//! two circles, of radius `4 ± √3/2`, so it is cut into a half disk inside the
//! inner circle, a half annulus between the two and a piece beyond the outer
//! circle.
//!
//! # What the row was after
//!
//! `in_the_bulge_of` in `fuse/select.rs` counted nothing for a side along a
//! torus. On a plane square to the ring's axis, a side along the outer circle
//! whose material is outside the ring bites into its piece, and the segment of
//! that bite holds points of the inner disk. So the piece beyond the outer
//! circle could hold a point of the inner disk, and the half disk inside the
//! inner circle none of its own. `across_the_inner_disk` now counts that bite,
//! and two unit pins in `select.rs`'s tests, which failed before it, hold the
//! count: `a_plate_beyond_a_rings_outer_circle_holds_no_point_of_the_inner_disk`
//! and `a_half_disk_inside_a_rings_inner_circle_holds_its_own_points`.
//!
//! **The ring alone cannot show a wrong answer.** The point of the inner disk
//! and the true points of the piece beyond the outer circle are both outside
//! the ring, so either one places the piece the same. A wrong answer needs a
//! third solid that holds the inner disk, and the bar is that solid. The
//! plate's top beyond the outer circle has an arc of radius 11/2 as a side, and
//! `half_way_into_either_arc` offers `(2.75, 0, 1/2)` for it: in the inner
//! disk, and inside the bar. Wrongly held, it would place that piece inside the
//! bar.
//!
//! # What the fuse does
//!
//! None of the three shapes that hold the ring reaches that question, under
//! any rule either way round:
//!
//! - the plate and the ring are refused *"a point inside a piece of a face"*,
//!   on the plate top's half annulus, corners `(0, ±3.134, 1/2)` and
//!   `(0, ±4.866, 1/2)`; a model of that piece on `z = 0` in `select.rs`'s
//!   tests holds three points offered for it, so the cause lies in something
//!   the real piece has and that model does not, and is not traced;
//! - the plate against the ring and the bar together is refused by the same
//!   predicate, on a piece not traced;
//! - the bar and the ring, which never touch, are refused
//!   `CannotTellTheCellsOfAFaceApart`, cause not traced.
//!
//! The plate and the bar, the control, answer, and their volumes are right.
//! Each refusal is pinned here, so the change that answers one fails its pin
//! and has to check the answers against the volumes below. For the plate
//! against the ring and the bar, with `S` the plate and ring plus the plate
//! and bar: the union is `605π/16 + 8π² + 529π/128 − S`, about 175.72567; the
//! intersection `S`, about 35.00625; the plate less the others
//! `605π/16 − S`, about 83.78523; and the others less the plate
//! `8π² + 529π/128 − S`, about 56.93420.
//!
//! # The volumes, by hand
//!
//! - Plate: `π·(11/2)²/2 · 5/2 = 605π/16`.
//! - Ring: `2π·4 · π·1² = 8π²`, by Pappus.
//! - Plate and ring: the half, `x ≥ 0`, of the ring below `z = 1/2`. The tube's
//!   cross-section above `z = 1/2` is a segment of the unit circle, `π/3 − √3/4`,
//!   so below it is `2π/3 + √3/4`. That cross-section is symmetric about
//!   `ρ = 4`, so by Pappus half its turn is `4π(2π/3 + √3/4) = 8π²/3 + π√3`.
//! - Bar: `π(23/8)² · 1/2 = 529π/128`.
//! - Plate and bar: the half bar with `z` in `[1/4, 1/2]`, `529π/512`.
//! - Ring and bar: nothing. The ring's inner wall is at `ρ = 4 − √(1 − z²)`,
//!   at least `3.03` for `z` in `[1/4, 3/4]`, and the bar's radius is `2.875`.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;
const PLATE: f64 = 5.5;
const PLATE_TOP: f64 = 0.5;
const PLATE_BOTTOM: f64 = -2.0;
const BAR: f64 = 23.0 / 8.0;
const BAR_BOTTOM: f64 = 0.25;
const BAR_TOP: f64 = 0.75;

const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The predicate the plate top's half annulus is refused by.
const INSIDE_A_PIECE: &str = "a point inside a piece of a face";

#[test]
fn a_half_disk_plate_and_a_whole_ring_are_refused_on_the_plates_half_annulus() {
    every_way(the_plate, the_ring, |_, rule, plate_first, answer| {
        assert_eq!(
            refused_by(answer),
            INSIDE_A_PIECE,
            "{rule:?}, plate first: {plate_first}"
        );
    });
}

#[test]
fn a_half_disk_plate_against_a_ring_and_a_bar_in_its_hole_is_refused_on_the_half_annulus() {
    every_way(
        the_plate,
        the_ring_and_the_bar,
        |_, rule, plate_first, answer| {
            assert_eq!(
                refused_by(answer),
                INSIDE_A_PIECE,
                "{rule:?}, plate first: {plate_first}"
            );
        },
    );
}

#[test]
fn a_bar_in_a_rings_hole_touching_nothing_cannot_tell_the_cells_of_a_face_apart() {
    every_way(the_bar, the_ring, |_, rule, bar_first, answer| {
        assert!(
            matches!(answer, Err(Error::CannotTellTheCellsOfAFaceApart { .. })),
            "{rule:?}, bar first: {bar_first}: {answer:?}"
        );
    });
}

#[test]
fn a_half_disk_plate_and_a_bar_on_its_axis_fuse_to_their_volumes() {
    let plate = 605.0 * PI / 16.0;
    let bar = 529.0 * PI / 128.0;
    let shared = 529.0 * PI / 512.0;
    every_way(the_plate, the_bar, |model, rule, plate_first, answer| {
        let what = format!("{rule:?}, plate first: {plate_first}");
        let solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why:?}"));
        let volume = match (rule, plate_first) {
            (Rule::Union, _) => plate + bar - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, true) => plate - shared,
            (Rule::Difference, false) => bar - shared,
            (other, _) => panic!("a rule this file does not run: {other:?}"),
        };
        assert_eq!(solids.len(), 1, "{what}: {solids:?}");
        assert_measures(model, &solids, volume, &what);
    });
}

/// Builds `one` and `other` into a fresh model for each rule, both ways round,
/// fuses them, and hands the answer to `check` with the model it is in and
/// whether `one` went first.
fn every_way(
    one: fn(&mut Model) -> Vec<SolidKey>,
    other: fn(&mut Model) -> Vec<SolidKey>,
    check: impl Fn(&Model, Rule, bool, rcad_topo::Result<Vec<SolidKey>>),
) {
    for rule in RULES {
        for one_first in [true, false] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let one = one(&mut model);
            let other = other(&mut model);
            let (first, second) = if one_first {
                (one, other)
            } else {
                (other, one)
            };
            let answer = model
                .fuse(
                    &first,
                    &second,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .map(|fused| fused.solids().to_vec());
            if answer.is_ok() {
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| {
                        panic!("{rule:?}, one first: {one_first}: does not validate: {why}")
                    });
            }
            check(&model, rule, one_first, answer);
        }
    }
}

/// The predicate the last record of an `Undecidable` names.
fn refused_by(answer: rcad_topo::Result<Vec<SolidKey>>) -> String {
    match answer {
        Err(Error::Undecidable { records }) => records
            .records()
            .last()
            .expect("a refusal names why")
            .predicate()
            .to_owned(),
        other => panic!("refused as undecidable, not {other:?}"),
    }
}

fn assert_measures(model: &Model, solids: &[SolidKey], volume: f64, what: &str) {
    let (measured, error) = solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::for_scale(model.scale()),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} does not measure: {why}"))
        })
        .fold((0.0, 0.0), |(volume, error), each| {
            (volume + each.volume(), error + each.volume_error())
        });
    assert!(
        (measured - volume).abs() <= error,
        "{what} holds {volume} and this one says {measured} give or take {error}"
    );
}

fn the_plate(model: &mut Model) -> Vec<SolidKey> {
    vec![the_half_disk_plate(model)]
}

fn the_ring(model: &mut Model) -> Vec<SolidKey> {
    vec![a_whole_ring_cut_along_its_equators(model)]
}

fn the_bar(model: &mut Model) -> Vec<SolidKey> {
    vec![a_bar_on_the_axis(model, BAR, BAR_BOTTOM, BAR_TOP)]
}

fn the_ring_and_the_bar(model: &mut Model) -> Vec<SolidKey> {
    vec![
        a_whole_ring_cut_along_its_equators(model),
        a_bar_on_the_axis(model, BAR, BAR_BOTTOM, BAR_TOP),
    ]
}

/// The half, `x ≥ 0`, of a disk of radius 11/2 with `z` in `[−2, 1/2]`: a bar
/// on the axis intersected with a block.
fn the_half_disk_plate(model: &mut Model) -> SolidKey {
    let disk = a_bar_on_the_axis(model, PLATE, PLATE_BOTTOM, PLATE_TOP);
    let block = a_block(model, [0.0, -6.0, -3.0], [6.0, 6.0, 1.0]);
    let fused = model
        .fuse(
            &[disk],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar and a block on its axis");
    let &[plate] = fused.solids() else {
        panic!("{fused:?}");
    };
    plate
}

fn a_bar_on_the_axis(model: &mut Model, radius: f64, bottom: f64, top: f64) -> SolidKey {
    let lift = Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite shift");
    model
        .add_round_bar(radius, top - bottom, &lift)
        .expect("a bar")
}

fn a_block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let [x0, y0, z0] = low;
    let [x1, y1, z1] = high;
    let square = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &square,
            z1 - z0,
            &Isometry::translation(Vector::new(0.0, 0.0, z0)).expect("a finite shift"),
        )
        .expect("a block")
}

/// A whole torus about `z`, ring 4 and tube 1, as two faces: the half below
/// `z = 0` and the half above, meeting along the outer and the inner equator.
fn a_whole_ring_cut_along_its_equators(model: &mut Model) -> SolidKey {
    let cut = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let corners = [
        Point::new(-(RING + TUBE), 0.0, 0.0),
        Point::new(-(RING - TUBE), 0.0, 0.0),
    ];
    let skin = Surface::Torus(Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower"));
    let mut builder = SolidBuilder::new();
    let [one, other] = corners.map(|corner| {
        let corner = builder
            .add_vertex(corner)
            .expect("a corner on the ring and on the plane");
        builder
            .add_edge(
                corner,
                corner,
                EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(cut),
                },
            )
            .expect("a rim is a whole closed curve, so one corner twice is an edge")
    });
    let face = |builder: &mut SolidBuilder, walks: [(_, Orientation); 2]| {
        let wires = walks.map(|(edge, turn)| {
            let coedge = builder
                .add_coedge(edge, turn)
                .expect("an edge of this build");
            builder
                .add_wire(&[DraftStep::Along(coedge)])
                .expect("one closed edge is a wire")
        });
        builder
            .add_face(skin, &wires)
            .expect("a band between two circles")
    };
    let first = face(
        &mut builder,
        [(one, Orientation::Reversed), (other, Orientation::Forward)],
    );
    let second = face(
        &mut builder,
        [(one, Orientation::Forward), (other, Orientation::Reversed)],
    );
    builder
        .add_shell(&[
            (first, Orientation::Forward),
            (second, Orientation::Forward),
        ])
        .expect("two halves close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole ring in two faces is a solid")
}
