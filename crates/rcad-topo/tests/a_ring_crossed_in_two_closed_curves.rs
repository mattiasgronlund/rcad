//! Three pairs that cross a whole ring in two closed curves, fused under every
//! rule, both ways round. `docs/decisions.md` §378.8.
//!
//! §378 named a closed crossing by its two surfaces alone and found a plane
//! through a ring's axis cuts it in **two** circles, which then shared one
//! name. Q756 told those two apart. These are the other pairs that cross a ring
//! in two closed curves, each built so that **no edge or corner of either
//! solid lies on either curve**, so both would start at a corner the two
//! surfaces alone name:
//!
//! 1. **A plane square to the ring's axis**: the ring of four round a tube of
//!    one about `+z`, and the block `[−6, 6] × [−6, 6] × [−2, ½]`. Its top cuts
//!    the ring in an inner and an outer circle, of radius `4 ± √3/2` at
//!    `z = ½`. The ring's only edges are its two equators at `z = 0`, half a
//!    unit below.
//! 2. **Two rings about one axis**: rings of four and of five round tubes of
//!    one, both about `+z` through the origin. The two tubes' circles, `(4, 0)`
//!    and `(5, 0)` across the axis, cross at radius `4½` and height `±√3/2`, so
//!    the rings cross in an upper and a lower circle. Each ring's edges are its
//!    equators at `z = 0`.
//! 3. **A plane through the ring's axis**: the same ring written as two faces
//!    joined along the circles `y = 0` cuts it in, with corners `(±3, 0, 0)`,
//!    and the block `[−6, 0] × [−6, 6] × [−2, 2]`. Its side `x = 0` cuts the
//!    ring in the circles round `(0, ±4, 0)`, and the ring's edges keep to
//!    `|x| ≥ 3`.
//!
//! What each rule should give, by Pappus — a region of the tube's circle swept
//! round the axis holds its area times `2π` times its middle's distance from
//! the axis. A throwaway numerical sum over the same regions, on a grid of
//! 1500 by 1500, agreed with each to four figures:
//!
//! | pair | union | intersection | ring less the other | the other less the ring |
//! |---|---|---|---|---|
//! | 1 | `360 + 8π(π/3 − √3/4)` = 375.436 | `8π(2π/3 + √3/4)` = 63.521 | `8π(π/3 − √3/4)` = 15.436 | 296.479 |
//! | 2 | `18π² − 9π(2π/3 − √3/2)` = 142.922 | `9π(2π/3 − √3/2)` = 34.731 | 44.226 | 63.965 |
//! | 3 | `288 + 4π²` = 327.478 | `4π²` = 39.478 | `4π²` | 248.522 |
//!
//! The whole ring is `2π²·4·1 = 8π²`, the ring of five `10π²`. The strip of the
//! tube's unit circle above `z = ½` is `π/3 − √3/4`. Two unit circles one apart
//! share `2π/3 − √3/2`, and that lens is even about radius `4½`.
//!
//! **Every one of the eighteen fuses is refused, before any curve is named.**
//! Pairs 1 and 3 are refused `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`,
//! not for the plane that cuts the ring but for the block's sides parallel to
//! the axis, which miss the ring altogether (Q618). Pair 2 is refused
//! `ACrossingBetweenATorusAndAnotherCurvedSurface`: `intersect` names no pair
//! of two different rings. §378.8 has where each is raised, and why each is a
//! gap.
//!
//! **Not pinned, because a panic is not a result**: asked through
//! `SolidView::classify_point` whether a point inside the tube is in either
//! whole ring, both panic in `holds.rs`'s
//! `what_a_ring_face_between_two_planes_holds`, "a face this was sent is
//! trimmed by planes only, and this one by Torus" (§378.8).

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, SurfaceMeet,
    Torus, Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const TUBE: f64 = 1.0;

/// Each whole ring is a sound solid of the volume Pappus gives it, so what the
/// fuses below refuse is the pair and not either ring.
#[test]
fn a_whole_ring_written_as_two_faces_validates_and_measures_what_pappus_gives() {
    let pi = core::f64::consts::PI;
    let rings: [(&str, fn(&mut Model) -> SolidKey, f64); 3] = [
        (
            "the ring of four cut along its equators",
            |model| a_whole_ring_cut_along_its_equators(model, 4.0),
            8.0 * pi * pi,
        ),
        (
            "the ring of five cut along its equators",
            |model| a_whole_ring_cut_along_its_equators(model, 5.0),
            10.0 * pi * pi,
        ),
        (
            "the ring of four cut along y = 0",
            a_whole_ring_cut_along_two_meridians,
            8.0 * pi * pi,
        ),
    ];
    for (what, build, pappus) in rings {
        let mut model = a_model();
        let ring = build(&mut model);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let measured = model
            .solid(ring)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        assert!(
            (measured.volume() - pappus).abs() <= measured.volume_error(),
            "{what}: {} against {pappus}, and the mesh may miss by {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// Pair 1. Should give the table's first row; refused for the block's four
/// sides, each 6 off the ring's axis against the ring's reach of 5.
#[test]
fn a_ring_and_a_block_whose_top_is_square_to_its_axis_are_refused_for_the_blocks_sides() {
    for (rule, ring_first, answer) in every_fuse_of(the_ring_and_the_block_square_to_its_axis) {
        assert!(
            matches!(
                answer,
                Err(Error::Geometry(
                    rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                ))
            ),
            "{rule:?}, ring first {ring_first}: {answer:?}"
        );
    }
}

/// Pair 2. Should give the table's second row; refused because `intersect`
/// names no crossing of two different rings, even about one axis.
#[test]
fn two_rings_about_one_axis_are_refused_before_their_two_circles_are_found() {
    for (rule, ring_first, answer) in every_fuse_of(the_two_rings) {
        assert!(
            matches!(
                answer,
                Err(Error::Geometry(
                    rcad_geom::Error::ACrossingBetweenATorusAndAnotherCurvedSurface
                ))
            ),
            "{rule:?}, ring of four first {ring_first}: {answer:?}"
        );
    }
}

/// Pair 3. Should give the table's third row; refused for the block's three
/// sides parallel to the axis and off it, as pair 1 is.
#[test]
fn a_ring_and_a_block_whose_side_holds_its_axis_are_refused_for_the_blocks_other_sides() {
    for (rule, ring_first, answer) in every_fuse_of(the_ring_and_the_block_through_its_axis) {
        assert!(
            matches!(
                answer,
                Err(Error::Geometry(
                    rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                ))
            ),
            "{rule:?}, ring first {ring_first}: {answer:?}"
        );
    }
}

/// Which of each block's faces `intersect` refuses against the ring, the
/// question the fuse's first step asks of every pair of faces (`cuts::gather`).
///
/// The face that cuts the ring is answered two closed curves, and the faces
/// square to the axis beyond the tube are answered nothing. Only the sides
/// parallel to the axis are refused, though each is further from it than the
/// ring reaches.
#[test]
fn only_the_sides_of_each_block_that_miss_the_ring_are_refused_against_it() {
    let ring = Surface::Torus(Torus::new(Axis::z(), 4.0, TUBE).expect("a tube narrower"));
    for (pair, the_pair, beyond_the_tube, missing_sides) in [
        (
            "square to the axis",
            the_ring_and_the_block_square_to_its_axis as fn(&mut Model) -> [SolidKey; 2],
            1,
            4,
        ),
        (
            "through the axis",
            the_ring_and_the_block_through_its_axis,
            2,
            3,
        ),
    ] {
        let mut model = a_model();
        let [_, block] = the_pair(&mut model);
        let mut two_curves = 0;
        let mut nothing = 0;
        let mut refused = 0;
        for face in model.solid(block).expect("this model made the key").faces() {
            match intersect(
                &face.surface(),
                &ring,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) {
                Ok(SurfaceMeet::Curves(runs)) if runs.len() == 2 => two_curves += 1,
                Ok(SurfaceMeet::Nothing) => nothing += 1,
                Err(rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic) => {
                    refused += 1;
                }
                other => panic!("{pair}: {:?} gave {other:?}", face.surface()),
            }
        }
        assert_eq!(
            (two_curves, nothing, refused),
            (1, beyond_the_tube, missing_sides),
            "{pair}: the cutting face, the faces beyond the tube, the sides that miss"
        );
    }
}

/// Each of the six fuses of the two solids `pair` writes into a fresh model:
/// the rule, whether the ring (the ring of four, for two rings) came first, and
/// what the fuse gave. A refused fuse leaves its model sound, which is checked.
fn every_fuse_of(
    pair: fn(&mut Model) -> [SolidKey; 2],
) -> Vec<(Rule, bool, Result<Vec<SolidKey>, Error>)> {
    let mut answers = Vec::with_capacity(6);
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ring_first in [true, false] {
            let mut model = a_model();
            let [ring, other] = pair(&mut model);
            let [first, second] = if ring_first {
                [ring, other]
            } else {
                [other, ring]
            };
            let answer = model
                .fuse(
                    &[first],
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .map(|fused| fused.solids().to_vec());
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{rule:?}, ring first {ring_first}: {why:?}"));
            answers.push((rule, ring_first, answer));
        }
    }
    answers
}

fn the_ring_and_the_block_square_to_its_axis(model: &mut Model) -> [SolidKey; 2] {
    let ring = a_whole_ring_cut_along_its_equators(model, 4.0);
    let block = a_block(model, [-6.0, -6.0, -2.0], [6.0, 6.0, 0.5]);
    [ring, block]
}

fn the_two_rings(model: &mut Model) -> [SolidKey; 2] {
    let four = a_whole_ring_cut_along_its_equators(model, 4.0);
    let five = a_whole_ring_cut_along_its_equators(model, 5.0);
    [four, five]
}

fn the_ring_and_the_block_through_its_axis(model: &mut Model) -> [SolidKey; 2] {
    let ring = a_whole_ring_cut_along_two_meridians(model);
    let block = a_block(model, [-6.0, -6.0, -2.0], [0.0, 6.0, 2.0]);
    [ring, block]
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// The block from `low` to `high`, its square extruded up and moved into place.
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

/// The whole ring of radius `ring` round a tube of [`TUBE`] about `+z`, written
/// as its top half and its bottom half, joined along its outer and its inner
/// equator.
///
/// Both faces are on the one torus, so each equator is a rim that names the
/// plane `z = 0` as what cut it, and its one corner on the `+x` axis says which
/// of the two circles that plane cuts it is. Each face is the band between
/// the two equators, walked as `rings/mod.rs`'s top half of a ring walks them.
fn a_whole_ring_cut_along_its_equators(model: &mut Model, ring: f64) -> SolidKey {
    let middle = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    a_whole_ring_in_two_faces(
        model,
        ring,
        middle,
        [
            Point::new(ring + TUBE, 0.0, 0.0),
            Point::new(ring - TUBE, 0.0, 0.0),
        ],
    )
}

/// The whole ring of four round a tube of [`TUBE`] about `+z`, written as its
/// half on `y ≥ 0` and its half on `y ≤ 0`, joined along the two circles the
/// plane `y = 0` cuts it in, round `(4, 0, 0)` and `(−4, 0, 0)`.
///
/// Each face is the half bend of `rings/mod.rs` without its seam, the shape
/// two quarter bends united come back as (§378.6), and each circle is walked
/// the way that half bend's wall walks its cap circle.
fn a_whole_ring_cut_along_two_meridians(model: &mut Model) -> SolidKey {
    let across = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
        .expect("the y axis is a unit normal");
    a_whole_ring_in_two_faces(
        model,
        4.0,
        across,
        [Point::new(3.0, 0.0, 0.0), Point::new(-3.0, 0.0, 0.0)],
    )
}

/// Two faces on the one torus, each bounded by the two circles `cut` cuts it
/// in, one through each of `corners`: the first face walks the first circle
/// `Reversed` and the second `Forward`, and the other face the other way.
fn a_whole_ring_in_two_faces(
    model: &mut Model,
    ring: f64,
    cut: Plane,
    corners: [Point; 2],
) -> SolidKey {
    let skin = Surface::Torus(Torus::new(Axis::z(), ring, TUBE).expect("a tube narrower"));
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
