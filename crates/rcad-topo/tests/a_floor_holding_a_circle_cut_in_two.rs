//! A flat face holding a whole circle that a straight cut splits in two
//! beside it, which used to panic in `assemble.rs` (queue rows Q20 and Q394).
//!
//! The circle is one lump of the face's drawing and the face's own rim with
//! the cut across it is another. Nothing the circle is drawn by tells the two
//! halves of the face apart, so §36.3's sign rule joined the circle's outer
//! walk to **both** halves, and through it the two halves to each other: one
//! piece with two outer wires. When every region of that piece is kept, the
//! fuse joins the coplanar pieces back into one face and finds two outlines,
//! and `rings_round` panicked "a face joined out of coplanar pieces has one
//! outline, and this one has two". `docs/decisions.md` §282 is the same glue
//! on a cone, where selection refused rather than panicked.
//!
//! Every answer here is checked three ways: the model validates, it has the
//! number of solids the shapes call for, and its volume is the one worked out
//! by hand within the error its own mesh declares.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

/// The lens of `the_way_into_an_ellipse_arc_in_a_fuse.rs`, worked out there by
/// hand: `128π / (27√3) + 64/9`.
fn the_lens() -> f64 {
    128.0 * std::f64::consts::PI / (27.0 * 3.0_f64.sqrt()) + 64.0 / 9.0
}

/// The lens inside a block `(x0, −6, 2)`–`(7, 6, 9)`, united both ways round and
/// taken out of the block.
///
/// The cone the lens is cut from crosses the block's floor `z = 2` in the whole
/// circle `r = 1`, and the lower lid's plane `x + z = 4` crosses it along
/// `x = 2`, cutting the floor in two with the circle in the left half. For
/// `x0 = −3` the upper lid's plane `z − x = 4` cuts it along `x = −2` as well,
/// and for `x0 = −2` that line is the floor's own edge.
///
/// **The block holds the whole lens**: the lens is between `z = 8/3` and `8`,
/// never further than `4` from the axis, and at `x ≥ 0`. So the union is the
/// block and the difference is the block less the lens.
#[test]
fn a_block_round_a_lens_whose_floor_the_cone_rings_and_a_lid_cuts_in_two() {
    for x0 in [-1.5, -2.0, -3.0] {
        let block = (7.0 - x0) * 12.0 * 7.0;
        for (rule, lens_first, want) in [
            (Rule::Union, true, block),
            (Rule::Union, false, block),
            (Rule::Difference, false, block - the_lens()),
        ] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let lens = a_lens_of_a_cone(&mut model);
            let around = a_block(&mut model, [x0, -6.0, 2.0], [7.0, 6.0, 9.0]);
            let (arguments, tools) = if lens_first {
                ([lens], [around])
            } else {
                ([around], [lens])
            };
            let named = format!("{rule:?} with the block from x = {x0}, lens first {lens_first}");
            what_the_fuse_measures(&mut model, &arguments, &tools, rule, 1, want, &named);
        }
    }
}

/// A block `[0, 10]³` holding a bar of radius 1 at `(3, 7)`, from `z = 2` to
/// `8`, and a box `[6, 8] × [2, 4] × [2, 5]`: flat faces and one pipe.
///
/// The bar's pipe crosses the block's floor and top in whole circles, and the
/// box's four walls cross them in straight lines that cut each into nine. No
/// cone and no curve that runs on for ever: the glue is not a curved face's.
/// The bar is `6π` and the box `12`, and both are wholly inside the block, so
/// the union is the block and the difference the block less both.
#[test]
fn a_block_holding_a_bar_and_a_box_whose_walls_cut_its_floor_in_nine() {
    let block = 1000.0;
    let bar = 6.0 * std::f64::consts::PI;
    for (rule, want) in [(Rule::Union, block), (Rule::Difference, block - bar - 12.0)] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let around = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        let round = model
            .add_round_bar(
                1.0,
                6.0,
                &Isometry::translation(Vector::new(3.0, 7.0, 2.0)).expect("a finite shift"),
            )
            .expect("a bar");
        let small = a_block(&mut model, [6.0, 2.0, 2.0], [8.0, 4.0, 5.0]);
        let named = format!("{rule:?} of the block and the bar with the box");
        what_the_fuse_measures(
            &mut model,
            &[around],
            &[round, small],
            rule,
            1,
            want,
            &named,
        );
    }
}

/// Fuse, validate, count the solids, and hold their volume to `want` within
/// what the mesh says it may miss by.
fn what_the_fuse_measures(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
    solids: usize,
    want: f64,
    named: &str,
) {
    let policy = Policy::certified();
    let fused = model
        .fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|refused| panic!("{named}: the fuse refused: {refused:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|broken| panic!("{named}: the answer is not a sound model: {broken:?}"));
    assert_eq!(fused.solids().len(), solids, "{named}: how many solids");
    let fine = Deflection::of(0.001).expect("a positive length");
    let (mut volume, mut error) = (0.0, 0.0);
    for &solid in fused.solids() {
        let measured = model
            .solid(solid)
            .expect("a solid the fuse made")
            .measure(fine, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{named}: the answer does not measure: {why:?}"));
        volume += measured.volume();
        error += measured.volume_error();
    }
    // The lens's closed form is itself rounded, a few units in the last place
    // of a number near 16; a flat answer measures exactly and declares nothing.
    let slack = 1e-12 * want.abs();
    assert!(
        (volume - want).abs() <= error + slack,
        "{named}: measured {volume} against {want}, and the mesh may miss by {error}"
    );
}

fn a_block(model: &mut Model, [x0, y0, z0]: [f64; 3], [x1, y1, z1]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &rectangle,
            z1 - z0,
            &Isometry::translation(Vector::new(0.0, 0.0, z0)).expect("a finite shift"),
        )
        .expect("a rectangle extrudes")
}

/// Slope a half, so `slope · height` rounds to nothing at the corners.
const SLOPE: f64 = 0.5;

/// Where both lids cross the axis, and how far up the cone the corners sit.
const LIDS_MEET_THE_AXIS_AT: f64 = 4.0;

/// How far behind the lower lid's corners its picker's plane stands.
const THE_LOWER_LIDS_PICKER_STANDS_BACK: f64 = 0.5;

/// The lens of a cone between the lids `x + z = 4` and `z − x = 4`, described
/// face by face exactly as `the_way_into_an_ellipse_arc_in_a_fuse.rs` does,
/// whose module doc says why every number in it is exact.
fn a_lens_of_a_cone(model: &mut Model) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let reach = SLOPE * LIDS_MEET_THE_AXIS_AT;
    let corner = |builder: &mut SolidBuilder, y: f64| {
        builder
            .add_vertex(Point::new(0.0, y, LIDS_MEET_THE_AXIS_AT))
            .expect("a finite point")
    };
    let positive = corner(&mut builder, reach);
    let negative = corner(&mut builder, -reach);

    let arc = |and: Plane, picker: HalfSpace| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(and),
        picked_by: smallvec::smallvec![Picker::from(picker)],
    };
    let facing_the_arcs = |from_x: f64| {
        HalfSpace::through_a_point_facing(Point::new(from_x, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis has length")
    };
    let lower_arc = builder
        .add_edge(
            positive,
            negative,
            arc(
                the_chord_plane(),
                facing_the_arcs(-THE_LOWER_LIDS_PICKER_STANDS_BACK),
            ),
        )
        .expect("the smaller half of the lower lid's ellipse");
    let upper_arc = builder
        .add_edge(
            negative,
            positive,
            arc(the_chord_plane(), facing_the_arcs(0.0)),
        )
        .expect("the bigger half of the upper lid's ellipse");
    let chord = builder
        .add_edge(negative, positive, EdgeRun::Straight)
        .expect("the line both lids hold");

    let wire_of = |builder: &mut SolidBuilder, steps: &[(_, Orientation)]| {
        let along: Vec<DraftStep> = steps
            .iter()
            .map(|(edge, turn)| {
                DraftStep::Along(
                    builder
                        .add_coedge(*edge, *turn)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        builder.add_wire(&along).expect("a closed wire")
    };
    let lower_wire = wire_of(
        &mut builder,
        &[
            (lower_arc, Orientation::Forward),
            (chord, Orientation::Forward),
        ],
    );
    let upper_wire = wire_of(
        &mut builder,
        &[
            (upper_arc, Orientation::Forward),
            (chord, Orientation::Reversed),
        ],
    );
    let skin_wire = wire_of(
        &mut builder,
        &[
            (lower_arc, Orientation::Reversed),
            (upper_arc, Orientation::Reversed),
        ],
    );

    let lower = builder
        .add_face(Surface::Plane(the_lid(-1.0, -1.0)), &[lower_wire])
        .expect("half of the lower lid's ellipse");
    let upper = builder
        .add_face(Surface::Plane(the_lid(-1.0, 1.0)), &[upper_wire])
        .expect("half of the upper lid's ellipse");
    let skin = builder
        .add_face(
            Surface::Cone(Cone::new(Axis::z(), SLOPE).expect("a positive slope")),
            &[skin_wire],
        )
        .expect("the piece of the cone between the two lids");

    builder
        .add_shell(&[
            (lower, Orientation::Forward),
            (upper, Orientation::Forward),
            (skin, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a lens described this way is a solid")
}

/// A lid through `(0, 0, 4)` on the axis, leaning `across` in `x` and `up` in
/// `z`.
fn the_lid(across: f64, up: f64) -> Plane {
    Plane::from_point_and_normal(
        Point::new(0.0, 0.0, LIDS_MEET_THE_AXIS_AT),
        Vector::new(across, 0.0, up),
    )
    .expect("a vector with a length")
}

/// The plane the chord lies in, which both arcs are named against.
fn the_chord_plane() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal")
}
