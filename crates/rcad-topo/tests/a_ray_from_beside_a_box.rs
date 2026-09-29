//! Every public way to a solid with both a torus face and a flat face with
//! straight sides, and what refuses each (`docs/decisions.md` §323.3).
//!
//! A ray from a point near a torus face is fired from an exact point beside
//! it, once no face of the solid is shown to meet the box round both. A flat
//! face with straight sides is its fan of triangles, and a box on the plane of
//! one and off the face was refused until `classify.rs` held the two up against
//! the coordinate axes too. Only a solid with both kinds of face can ask that,
//! and none of these three builds one today — so the answers are pinned in
//! `classify.rs`'s own tests, on one solid holding two shells, the way a fuse
//! writes two pieces that do not touch. Each test here fails the day its way
//! opens, and that is the day the pin can move out here.

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, Invalid, Model, ProfileBuilder, Result, Rule, SolidBuilder, SolidKey,
};
use smallvec::smallvec;

mod rings;

use rings::a_quarter_bend;

/// A quarter bend whose first cap is glued onto a block's side is refused,
/// whichever way round the cap's circle is walked.
///
/// The solid is real: the block `[2, 6] × [−2, 0] × [−2, 2]` and a bend of
/// ring four and tube one standing on its side `y = 0`, the side holding the
/// bend's cap circle as a hole. One of the two walks has to be the right one,
/// because the side and the wall walk the circle once each way.
#[test]
fn a_bend_growing_out_of_a_blocks_side_is_refused_either_way_round() {
    for the_side_walks_it in [Orientation::Reversed, Orientation::Forward] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let built = a_bend_out_of_a_block(&mut model, the_side_walks_it);
        assert!(
            matches!(
                built,
                Err(Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface))
            ),
            "the block's side walking the circle {the_side_walks_it:?}: {built:?}"
        );
    }
}

/// A solid built face by face has one shell, so a block with a bend-shaped
/// hollow sealed inside it cannot be described.
#[test]
fn a_block_with_a_hollow_inside_it_cannot_be_built_face_by_face() {
    let mut builder = SolidBuilder::new();
    let a_disk = |builder: &mut SolidBuilder, z: f64| -> Result<_> {
        let corner = builder.add_vertex(Point::new(1.0, 0.0, z))?;
        let rim = builder.add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)?;
        let coedge = builder.add_coedge(rim, Orientation::Forward)?;
        let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
        let plane = Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, 1.0))
            .expect("a unit normal");
        builder.add_face(Surface::Plane(plane), &[wire])
    };
    let outer = a_disk(&mut builder, 1.0).expect("a disk is described");
    let hollow = a_disk(&mut builder, 0.0).expect("a disk is described");
    builder
        .add_shell(&[(outer, Orientation::Forward)])
        .expect("the first shell is described");
    let second = builder.add_shell(&[(hollow, Orientation::Forward)]);
    assert!(
        matches!(second, Err(Error::ASolidBuiltFaceByFaceHasOneShell)),
        "{second:?}"
    );
}

/// A bend wholly inside an L-shaped block is not cut out of it: the fuse asks
/// where the L's sides cross the bend's ring and is refused, though no side
/// comes near the bend.
#[test]
fn an_l_less_a_bend_wholly_inside_it_is_refused_as_a_quartic() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outline = ProfileBuilder::new(vec![
        [-6.0, -6.0],
        [20.0, -6.0],
        [20.0, 6.0],
        [6.0, 6.0],
        [6.0, 20.0],
        [-6.0, 20.0],
    ])
    .build(&policy, &mut ProvenanceLog::new())
    .expect("an L");
    let l = model
        .add_extrusion(
            &outline,
            4.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite shift"),
        )
        .expect("an L-shaped block");
    let bend = a_quarter_bend(&mut model, 4.0, 1.0);
    for rule in [Rule::Difference, Rule::Union] {
        let answer = model.clone().fuse(
            &[l],
            &[bend],
            rule,
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(
                answer,
                Err(Error::Geometry(
                    rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                ))
            ),
            "{rule:?}: {answer:?}"
        );
    }
}

/// The block `[2, 6] × [−2, 0] × [−2, 2]` with a quarter bend of ring four and
/// tube one about `+z` growing out of its side `y = 0`, from its cap there
/// round to its cap on `x = 0`.
///
/// The side `y = 0` holds the bend's first cap circle as a hole, walked
/// `the_side_walks_it`; the bend's wall walks it the other way. The other five
/// sides are rectangles.
fn a_bend_out_of_a_block(model: &mut Model, the_side_walks_it: Orientation) -> Result<SolidKey> {
    let the_wall_walks_it = match the_side_walks_it {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
    };
    let mut builder = SolidBuilder::new();
    let mut corners = BTreeMap::new();
    for x in [2, 6] {
        for y in [-2, 0] {
            for z in [-2, 2] {
                let at = Point::new(f64::from(x), f64::from(y), f64::from(z));
                corners.insert([x, y, z], builder.add_vertex(at)?);
            }
        }
    }
    let start_corner = builder.add_vertex(Point::new(3.0, 0.0, 0.0))?;
    let end_corner = builder.add_vertex(Point::new(0.0, 3.0, 0.0))?;

    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder.add_edge(
        start_corner,
        end_corner,
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(flat_through_the_middle),
            picked_by: smallvec![
                Picker::InsideAPipe(Cylinder::new(Axis::z(), 4.0).expect("the ring's radius")),
                Picker::from(
                    HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                        .expect("a normal with a length")
                ),
            ],
        },
    )?;
    let start_circle = builder.add_edge(
        start_corner,
        start_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let end_circle =
        builder.add_edge(end_corner, end_corner, EdgeRun::RightRoundWhereItsFacesCross)?;

    let mut straight = BTreeMap::new();
    let mut sides = Vec::new();
    for (normal, round) in [
        ([0, 1, 0], [[2, 0, -2], [2, 0, 2], [6, 0, 2], [6, 0, -2]]),
        ([0, -1, 0], [[2, -2, -2], [6, -2, -2], [6, -2, 2], [2, -2, 2]]),
        ([1, 0, 0], [[6, -2, -2], [6, 0, -2], [6, 0, 2], [6, -2, 2]]),
        ([-1, 0, 0], [[2, -2, -2], [2, -2, 2], [2, 0, 2], [2, 0, -2]]),
        ([0, 0, 1], [[2, -2, 2], [6, -2, 2], [6, 0, 2], [2, 0, 2]]),
        ([0, 0, -1], [[2, -2, -2], [2, 0, -2], [6, 0, -2], [6, -2, -2]]),
    ] {
        let mut steps = Vec::with_capacity(4);
        for position in 0..4 {
            let from = round[position];
            let to = round[(position + 1) % 4];
            let (edge, turn) = if let Some(&edge) = straight.get(&(to, from)) {
                (edge, Orientation::Reversed)
            } else {
                let edge = builder.add_edge(corners[&from], corners[&to], EdgeRun::Straight)?;
                straight.insert((from, to), edge);
                (edge, Orientation::Forward)
            };
            steps.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
        }
        let mut wires = vec![builder.add_wire(&steps)?];
        if normal == [0, 1, 0] {
            let round_the_cap = builder.add_coedge(start_circle, the_side_walks_it)?;
            wires.push(builder.add_wire(&[DraftStep::Along(round_the_cap)])?);
        }
        let [x, y, z] = normal.map(f64::from);
        let on_it = round[0].map(f64::from);
        let plane = Plane::from_point_and_normal(
            Point::new(on_it[0], on_it[1], on_it[2]),
            Vector::new(x, y, z),
        )
        .expect("a coordinate direction is a unit normal");
        sides.push(builder.add_face(Surface::Plane(plane), &wires)?);
    }

    let end_cap = {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
            .expect("a coordinate direction is a unit normal");
        let coedge = builder.add_coedge(end_circle, Orientation::Forward)?;
        let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
        builder.add_face(Surface::Plane(plane), &[wire])?
    };

    let mut round_the_wall = Vec::with_capacity(4);
    for (edge, turn) in [
        (start_circle, the_wall_walks_it),
        (seam, Orientation::Forward),
        (end_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ] {
        round_the_wall.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    let wire = builder.add_wire(&round_the_wall)?;
    let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
    let wall = builder.add_face(Surface::Torus(ring), &[wire])?;

    let mut shell: Vec<_> = sides
        .into_iter()
        .map(|side| (side, Orientation::Forward))
        .collect();
    shell.push((end_cap, Orientation::Forward));
    shell.push((wall, Orientation::Reversed));
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
