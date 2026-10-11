//! A box beside a solid with both a torus face and a flat face with straight
//! sides, asked through the public API, and every public way to such a solid
//! (`docs/decisions.md` §323.3).
//!
//! A ray from a point near a torus face is fired from an exact point beside
//! it, once no face of the solid is shown to meet the box round both. A flat
//! face with straight sides is its fan of triangles, and a box on the plane of
//! one and off the face was refused until `classify.rs` held the two up against
//! the coordinate axes and the directions across a triangle's sides, and added
//! up the fan of a face that is not convex. Only a solid with both kinds of
//! face can ask that. Of the three public ways to one, a bend growing out of a
//! block's side builds since the bend's wall was used forwards (§361), with its
//! cap circle drawn as one edge or as two half circles (Q524); a block with a
//! bend-shaped hollow inside it cannot be described, and an L less a bend
//! inside it is refused by the fuse. So the boxes are asked of the glued bend,
//! as it is, turned, and grown out of an L. They were asked in `classify.rs`'s
//! own tests, of one solid holding two shells, until that build opened (Q721).

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, NamedRun, Orientation, Picker, Plane,
    Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftEdge, DraftStep, Error, FaceView, Invalid, Model, ProfileBuilder, Result,
    Rule, SolidBuilder, SolidKey, SolidView, WireView,
};
use smallvec::smallvec;

mod rings;

use rings::a_quarter_bend;

/// A quarter bend whose first cap is glued onto a block's side builds when the
/// side walks the cap's circle forwards, measures the block and the bend
/// together, and is refused walked the other way.
///
/// The solid is real: the block `[2, 6] × [−2, 0] × [−2, 2]` and a bend of
/// ring four and tube one standing on its side `y = 0`, the side holding the
/// bend's cap circle as a hole. One of the two walks has to be the right one,
/// because the side and the wall walk the circle once each way. It holds
/// `4·2·4 = 32` and `2π²` by Pappus. Until the bend's wall was used forwards
/// both walks were refused `AClosedEdgeIsWalkedAgainstItsSurface` (§361).
#[test]
fn a_bend_growing_out_of_a_blocks_side_builds_walked_the_one_way_round() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let built = a_bend_out_of_a_block(&mut model, Orientation::Forward)
        .expect("a bend growing out of a block's side is a solid");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the block and the bend validate");
    let measured = model
        .solid(built)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a hundredth of the tube"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the block and the bend measure");
    let holds = 32.0 + 2.0 * core::f64::consts::PI * core::f64::consts::PI;
    assert!(
        (measured.volume() - holds).abs() <= measured.volume_error(),
        "the block and the bend hold {holds}, and measure {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );

    let mut elsewhere = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let turned_over = a_bend_out_of_a_block(&mut elsewhere, Orientation::Reversed);
    assert!(
        matches!(
            turned_over,
            Err(Error::Invalid(
                Invalid::AClosedEdgeIsWalkedAgainstItsSurface
            ))
        ),
        "the block's side walking the circle the same way as the wall: {turned_over:?}"
    );
}

/// The same glued bend with its cap circle drawn as two half circles builds
/// walked the one way round too, and is refused walked the other way.
///
/// Q342's lane drew it so and was refused `AWireWillNotLieFlatOnItsFace`, and
/// wrote down that it did not know whether that was a defect or a picker
/// written wrong. Neither: the next test shows the pickers pick the two
/// halves, and the input was wrong. That lane walked the cap's two halves the
/// way this test's `Forward` does, which is the way the side's hole forces, but
/// walked the end circle on the wall the other way and used the wall
/// `Reversed`, the way bends were built before §361. So its wall's wire went
/// round the tube the same way at both ends of the bend. A wire that goes
/// round the tube twice in all bounds no piece of the torus, whichever way the
/// face is used, and the mesher refused it by name. Walked `Reversed` here,
/// the wall's wire goes round the tube the same way at both ends too, and is
/// refused by that same name.
///
/// With one closed edge round the cap, the wrong walk is refused
/// `AClosedEdgeIsWalkedAgainstItsSurface` instead; the halves are not closed
/// edges, so that check does not see them, and the mesher is what refuses.
#[test]
fn a_bend_whose_cap_circle_is_two_halves_builds_walked_the_one_way_round() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let built = a_bend_out_of(
        &mut model,
        &THE_BLOCK,
        Orientation::Forward,
        TheCapCircle::TwoHalves,
    )
    .expect("a bend growing out of a block's side is a solid");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the block and the bend validate");
    let measured = model
        .solid(built)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a hundredth of the tube"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the block and the bend measure");
    let holds = 32.0 + 2.0 * core::f64::consts::PI * core::f64::consts::PI;
    assert!(
        (measured.volume() - holds).abs() <= measured.volume_error(),
        "the block and the bend hold {holds}, and measure {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );

    let mut elsewhere = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let turned_over = a_bend_out_of(
        &mut elsewhere,
        &THE_BLOCK,
        Orientation::Reversed,
        TheCapCircle::TwoHalves,
    );
    assert!(
        matches!(turned_over, Err(Error::AWireWillNotLieFlatOnItsFace { .. })),
        "the block's side walking the halves the same way as the wall: {turned_over:?}"
    );
}

/// The two half circles the glued bend is built with are the half of the cap
/// circle under its middle, from `(3, 0, 0)` to `(5, 0, 0)`, and the half over
/// it, back again.
///
/// Read off the built solid: the hole in the block's side, each of its two
/// edges walked between the surfaces of the two faces along it, and every
/// point of each walk on the circle round `(4, 0, 0)` of radius one, on its
/// own side of `z = 0`, and reaching the bottom or the top of the circle.
#[test]
fn the_cap_circles_two_halves_are_the_half_under_its_middle_and_the_half_over_it() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let built = a_bend_out_of(
        &mut model,
        &THE_BLOCK,
        Orientation::Forward,
        TheCapCircle::TwoHalves,
    )
    .expect("a bend growing out of a block's side is a solid");
    let solid = model.solid(built).expect("this model made the key");
    let side = solid
        .faces()
        .find(|face| face.wire_count() == 2)
        .expect("the block's side y = 0 holds the cap circle as a hole");
    let hole = side
        .wires()
        .find(|wire| wire.coedge_count() == 2)
        .expect("the hole is two half circles");

    let mut halves = Vec::with_capacity(2);
    for coedge in hole.coedges() {
        let edge = coedge.edge();
        let along: Vec<Surface> = edge
            .neighbouring_faces()
            .map(|key| {
                solid
                    .faces()
                    .find(|face| face.key() == key)
                    .expect("a face along an edge of this solid is one of its faces")
                    .surface()
            })
            .collect();
        let [one, other] = along[..] else {
            panic!("a half circle has two faces along it, and this one {along:?}");
        };
        let ends = [edge.start().point(), edge.end().point()];
        let run = NamedRun::new(edge.run(), ends.map(PredicatePoint::from))
            .expect("a half circle is not a closed run");
        let walked = rcad_geom::walk(
            &run,
            [&one, &other],
            Deflection::of(0.001).expect("a thousandth of the tube"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a half circle is walked");
        halves.push((ends.map(|end| end.coordinates()), walked));
    }

    let [(under_ends, under), (over_ends, over)] = &halves[..] else {
        panic!("two halves round the hole");
    };
    assert_eq!(*under_ends, [[3.0, 0.0, 0.0], [5.0, 0.0, 0.0]]);
    assert_eq!(*over_ends, [[5.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);
    for (walked, up) in [(under, -1.0), (over, 1.0)] {
        let off = walked.deviation().magnitude();
        let points: Vec<[f64; 3]> = walked.points().iter().map(|at| at.coordinates()).collect();
        assert!(
            !points.is_empty(),
            "a half circle has points between its ends"
        );
        for [x, y, z] in &points {
            let from_the_middle = ((x - 4.0) * (x - 4.0) + z * z).sqrt();
            assert!(
                y.abs() <= off && (from_the_middle - 1.0).abs() <= off,
                "({x}, {y}, {z}) is on the circle round (4, 0, 0) of radius one, to {off}"
            );
            assert!(
                z * up > 0.0,
                "({x}, {y}, {z}) is on the side of z = 0 that {up} points to"
            );
        }
        let furthest = points
            .iter()
            .map(|[_, _, z]| z * up)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            furthest > 0.999,
            "the half on the side {up} points to reaches {furthest} out of one"
        );
    }
}

/// Boxes on the planes of the glued block's sides, none on a side, and what
/// every point of each is against the block and the bend.
const BESIDE_THE_SIDES: [([f64; 3], Containment); 7] = [
    ([6.0, 1.0, 0.0], Containment::Outside),
    ([2.0, 1.0, 0.0], Containment::Outside),
    ([2.0, 3.5, 0.0], Containment::Inside),
    ([2.0, 3.5, 0.5], Containment::Inside),
    ([4.0, 1.0, 2.0], Containment::Outside),
    ([4.0, 1.0, -2.0], Containment::Outside),
    ([1.0, -2.0, 0.0], Containment::Outside),
];

/// A box on the plane of a side of the block a bend grows out of, and off the
/// side, is answered by a ray from an exact point beside it, since no face
/// meets the box round both; a box on a side is met by that side, and refused.
#[test]
fn a_box_beside_a_side_of_the_block_a_bend_grows_out_of_is_answered() {
    let policy = Policy::certified();
    let (model, built) = the_bend_out_of(&THE_BLOCK);
    let solid = model.solid(built).expect("this model made the key");
    for (at, should) in BESIDE_THE_SIDES {
        assert_eq!(what_the_shape_says(&THE_BLOCK, at), should, "{at:?}");
        let mut log = ProvenanceLog::new();
        let placed = solid.classify_uncertain_point(a_box_round(at), &policy, &mut log);
        assert_eq!(placed, should, "{at:?}");
        assert!(log.is_empty(), "{at:?} was settled exactly: {log:?}");
    }
    for on_a_side in [[4.0, -2.0, 0.0], [4.0, -1.0, 2.0]] {
        let placed = solid.classify_uncertain_point(
            a_box_round(on_a_side),
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(placed, Containment::CannotFireARayAtAFacesSurface { .. }),
            "{on_a_side:?}: {placed:?}"
        );
    }
}

/// The glued block turned leaves its top's corners a rounding off the top's
/// stored plane, and a box on that plane a quarter off the top, beside the
/// middle of one of its sides, is inside the bounding box of one of the top's
/// triangles. Only the directions across a triangle's sides keep the two
/// apart, and the box is answered outside.
#[test]
fn a_box_beside_the_top_of_a_turned_block_a_bend_grows_out_of_is_answered_outside() {
    let policy = Policy::certified();
    let (mut model, built) = the_bend_out_of(&THE_BLOCK);
    model
        .place_solid(built, &the_turn())
        .expect("this model made the key");
    let solid = model.solid(built).expect("this model made the key");
    let top = the_face_facing(
        solid,
        the_turn().apply_to_vector(Vector::new(0.0, 0.0, 1.0)),
    );
    assert!(
        !every_corner_is_on_its_plane(top),
        "the turn left every corner of the top on its plane"
    );
    let beside_the_top = [4.0, -2.25, 2.0];
    assert_eq!(
        what_the_shape_says(&THE_BLOCK, beside_the_top),
        Containment::Outside
    );
    let at = the_turn()
        .apply_to_point(Point::from(beside_the_top))
        .coordinates();
    let mut bounds_round_it = 0;
    top.for_each_triangle(|corners| {
        let corners = corners.map(|corner| corner.coordinates());
        let inside = (0..3).all(|axis| {
            let (low, high) = corners
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), corner| {
                    (low.min(corner[axis]), high.max(corner[axis]))
                });
            (low..=high).contains(&at[axis])
        });
        bounds_round_it += usize::from(inside);
    });
    assert!(
        bounds_round_it > 0,
        "no triangle's bounding box holds the box, so the coordinate axes alone settle it"
    );
    let mut log = ProvenanceLog::new();
    let placed = solid.classify_uncertain_point(a_box_round(at), &policy, &mut log);
    assert_eq!(placed, Containment::Outside, "{at:?}");
    assert!(log.is_empty(), "it was settled exactly: {log:?}");
}

/// The top of an L a bend grows out of is fanned from the end of one arm, so
/// two of its triangles, wound opposite ways, reach over the notch between the
/// arms. They add up to nothing there, and a box in the notch on the plane of
/// the top is answered outside; a box on an arm is met by the top, and
/// refused.
#[test]
fn a_box_in_the_notch_of_an_l_a_bend_grows_out_of_is_answered_outside() {
    let policy = Policy::certified();
    let (model, built) = the_bend_out_of(&AN_L);
    let solid = model.solid(built).expect("this model made the key");
    let in_the_notch = [4.25, -3.0, 2.0];
    let mut covered = [0; 2];
    the_face_facing(solid, Vector::new(0.0, 0.0, 1.0)).for_each_triangle(|corners| {
        if let Some(counter_clockwise) = which_way_it_covers(corners, in_the_notch) {
            covered[usize::from(counter_clockwise)] += 1;
        }
    });
    assert!(
        covered[0] > 0 && covered[1] > 0,
        "the top's fan should cover the notch both ways round: {covered:?}"
    );
    assert_eq!(
        what_the_shape_says(&AN_L, in_the_notch),
        Containment::Outside
    );
    let placed = solid.classify_uncertain_point(
        a_box_round(in_the_notch),
        &policy,
        &mut ProvenanceLog::new(),
    );
    assert_eq!(placed, Containment::Outside);

    let on_an_arm = [3.0, -3.0, 2.0];
    let placed =
        solid.classify_uncertain_point(a_box_round(on_an_arm), &policy, &mut ProvenanceLog::new());
    assert!(
        matches!(placed, Containment::CannotFireARayAtAFacesSurface { .. }),
        "{placed:?}"
    );
}

/// The L turned leaves its top's corners a rounding off its stored plane, and
/// then the two triangles over the notch are not in one plane. Every point of
/// the box in the notch is still outside the L, a quarter from the nearest
/// wall, so the answer exists and this is a gap: `its_fan_adds_up_to_nothing_over`
/// does not answer for a fan that is not flat.
#[test]
fn a_box_in_the_notch_of_a_turned_l_a_bend_grows_out_of_is_still_refused() {
    let policy = Policy::certified();
    let (mut model, built) = the_bend_out_of(&AN_L);
    model
        .place_solid(built, &the_turn())
        .expect("this model made the key");
    let solid = model.solid(built).expect("this model made the key");
    let top = the_face_facing(
        solid,
        the_turn().apply_to_vector(Vector::new(0.0, 0.0, 1.0)),
    );
    assert!(
        !every_corner_is_on_its_plane(top),
        "the turn left every corner of the top on its plane"
    );
    let in_the_notch = [4.25, -3.0, 2.0];
    assert_eq!(
        what_the_shape_says(&AN_L, in_the_notch),
        Containment::Outside
    );
    let at = the_turn()
        .apply_to_point(Point::from(in_the_notch))
        .coordinates();
    let placed =
        solid.classify_uncertain_point(a_box_round(at), &policy, &mut ProvenanceLog::new());
    assert!(
        matches!(placed, Containment::CannotFireARayAtAFacesSurface { .. }),
        "{placed:?}"
    );
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
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, 1.0))
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
        let answer = model
            .clone()
            .fuse(&[l], &[bend], rule, &policy, &mut ProvenanceLog::new());
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

/// A new model holding [`a_bend_out_of`] `outline`, walked the way that builds,
/// with one closed edge round the cap.
fn the_bend_out_of(outline: &[[i32; 2]]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let built = a_bend_out_of(
        &mut model,
        outline,
        Orientation::Forward,
        TheCapCircle::OneClosedEdge,
    )
    .expect("a bend growing out of a block's side is a solid");
    (model, built)
}

/// What `at` is against the block stood up on `outline` and the bend growing
/// out of it, read off the shape: inside the outline from `z = −2` to `z = 2`,
/// or inside the tube of radius one round the circle of radius four about
/// `+z`, in the quarter where `x` and `y` are both above nought.
///
/// Every point asked here is small whole numbers and quarters, so every sum
/// and product is exact. A point within an eighth of the skin along an axis is
/// not asked, since the skin itself is not read here.
fn what_the_shape_says(outline: &[[i32; 2]], at: [f64; 3]) -> Containment {
    let inside = |[x, y, z]: [f64; 3]| {
        let crossings = (0..outline.len())
            .filter(|&position| {
                let [from, to] = [outline[position], outline[(position + 1) % outline.len()]]
                    .map(|corner| corner.map(f64::from));
                from[0] == to[0] && from[0] > x && from[1].min(to[1]) <= y && y < from[1].max(to[1])
            })
            .count();
        let in_the_block = crossings % 2 == 1 && -2.0 < z && z < 2.0;
        let round = x * x + y * y;
        let in_the_bend =
            x > 0.0 && y > 0.0 && (round + z * z + 15.0) * (round + z * z + 15.0) < 64.0 * round;
        in_the_block || in_the_bend
    };
    let answer = inside(at);
    for axis in 0..3 {
        for step in [-0.125, 0.125] {
            let mut nudged = at;
            nudged[axis] += step;
            assert_eq!(
                inside(nudged),
                answer,
                "{at:?} is within an eighth of the skin along axis {axis}"
            );
        }
    }
    if answer {
        Containment::Inside
    } else {
        Containment::Outside
    }
}

/// A box the size `select.rs` puts round a point it puts back onto a curved
/// surface: `256 ε` for every unit out from the origin.
fn a_box_round(at: [f64; 3]) -> PredicatePoint {
    let reach = 256.0 * f64::EPSILON * at.iter().fold(1.0f64, |most, each| most.max(each.abs()));
    PredicatePoint::Constructed(
        at.map(|coordinate| Interval::around(coordinate, reach).expect("a finite place")),
    )
}

/// `0.7` about `(1, 2, 3)` through the origin, which lines no flat face of
/// the glued block up with the axes.
fn the_turn() -> Isometry {
    let slant = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    Isometry::rotation(&slant, 0.7).expect("a turn")
}

/// The flat face of `solid` whose plane faces most nearly `up`.
fn the_face_facing(solid: SolidView<'_>, up: Vector) -> FaceView<'_> {
    let up = up.components();
    let facing_up = |face: &FaceView<'_>| {
        face.surface().as_plane().map_or(f64::NEG_INFINITY, |flat| {
            let normal = flat.normal().components();
            (0..3).map(|axis| normal[axis] * up[axis]).sum::<f64>()
        })
    };
    solid
        .faces()
        .max_by(|one, other| facing_up(one).total_cmp(&facing_up(other)))
        .expect("a solid with faces")
}

fn every_corner_is_on_its_plane(face: FaceView<'_>) -> bool {
    let flat = *face.surface().as_plane().expect("a flat face");
    face.wires().flat_map(WireView::points).all(|corner| {
        flat.side_of_point(PredicatePoint::from(corner.coordinates())) == Decision::Zero
    })
}

/// Whether the triangle `corners`, lying in the plane `z = at[2]`, covers
/// `at` strictly inside it: `Some(true)` wound counter-clockwise seen from
/// above, `Some(false)` clockwise. Small whole numbers and quarters, so
/// every product here is exact.
fn which_way_it_covers(corners: [Point; 3], at: [f64; 3]) -> Option<bool> {
    let corners = corners.map(|corner| corner.coordinates());
    if corners.iter().any(|corner| corner[2] != at[2]) {
        return None;
    }
    let turn = |from: [f64; 3], to: [f64; 3], there: [f64; 3]| {
        (to[0] - from[0]) * (there[1] - from[1]) - (to[1] - from[1]) * (there[0] - from[0])
    };
    let [a, b, c] = corners;
    let whole = turn(a, b, c);
    let inside = [turn(a, b, at), turn(b, c, at), turn(c, a, at)]
        .iter()
        .all(|part| part * whole > 0.0);
    inside.then_some(whole > 0.0)
}

/// The block `[2, 6] × [−2, 0] × [−2, 2]` with a quarter bend of ring four and
/// tube one about `+z` growing out of its side `y = 0`, from its cap there
/// round to its cap on `x = 0`.
///
/// The side `y = 0` holds the bend's first cap circle as a hole, walked
/// `the_side_walks_it`; the bend's wall walks it the other way. The other five
/// sides are rectangles.
fn a_bend_out_of_a_block(model: &mut Model, the_side_walks_it: Orientation) -> Result<SolidKey> {
    a_bend_out_of(
        model,
        &THE_BLOCK,
        the_side_walks_it,
        TheCapCircle::OneClosedEdge,
    )
}

/// The block's outline seen from `+z`, `[2, 6] × [−2, 0]`, walked
/// counter-clockwise.
const THE_BLOCK: [[i32; 2]; 4] = [[2, -2], [6, -2], [6, 0], [2, 0]];

/// An L seen from `+z`, of arms two wide: the block's outline, and
/// `[2, 4] × [−4, −2]` under it. Between the arms is the notch
/// `[4, 6] × [−4, −2]`. It starts at the end of the first arm, so the fan of its
/// top reaches over the notch.
const AN_L: [[i32; 2]; 6] = [[6, 0], [2, 0], [2, -4], [4, -4], [4, -2], [6, -2]];

/// How the bend's first cap circle, the hole in the block's side, is drawn.
#[derive(Clone, Copy, Debug)]
enum TheCapCircle {
    /// One edge right round it, from its inner point `(3, 0, 0)` back to it.
    OneClosedEdge,
    /// Two half circles between its inner point `(3, 0, 0)` and its outer
    /// point `(5, 0, 0)`, each a piece of where the side's plane crosses the
    /// ring picked by `x ≥ 0`, which leaves the circle round `(4, 0, 0)` and
    /// not the one round `(−4, 0, 0)`, and by `z ≤ 0` for the one that sets
    /// off from the inner point, `z ≥ 0` for the one that comes back to it.
    TwoHalves,
}

/// [`a_bend_out_of_a_block`] with the block's outline `outline` instead, and
/// its cap circle drawn `cap`'s way.
///
/// The outline is seen from `+z`, walked counter-clockwise, its sides along
/// the axes, and it has the side from `(6, 0)` to `(2, 0)`, which holds the
/// cap circle; it is stood up from `z = −2` to `z = 2`.
///
/// Walked `Forward`, the side goes round the cap's two halves from `(3, 0, 0)`
/// under the middle to `(5, 0, 0)` and back over it, which is clockwise seen
/// from `+y`, the side's outward normal, as a hole in it has to be. `Forward`
/// is the walk that builds, whichever way the cap is drawn.
fn a_bend_out_of(
    model: &mut Model,
    outline: &[[i32; 2]],
    the_side_walks_it: Orientation,
    cap: TheCapCircle,
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let mut corners = BTreeMap::new();
    for &[x, y] in outline {
        for z in [-2, 2] {
            let at = Point::new(f64::from(x), f64::from(y), f64::from(z));
            corners.insert([x, y, z], builder.add_vertex(at)?);
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
    let round_the_cap_forwards: Vec<(DraftEdge, Orientation)> = match cap {
        TheCapCircle::OneClosedEdge => vec![(
            builder.add_edge(
                start_corner,
                start_corner,
                EdgeRun::RightRoundWhereItsFacesCross,
            )?,
            Orientation::Forward,
        )],
        TheCapCircle::TwoHalves => {
            let outer_corner = builder.add_vertex(Point::new(5.0, 0.0, 0.0))?;
            let under = builder.add_edge(start_corner, outer_corner, half_of_the_cap(-1.0))?;
            let over = builder.add_edge(outer_corner, start_corner, half_of_the_cap(1.0))?;
            vec![(under, Orientation::Forward), (over, Orientation::Forward)]
        }
    };
    let the_side_goes_round_the_cap = match the_side_walks_it {
        Orientation::Forward => round_the_cap_forwards,
        Orientation::Reversed => walked_the_other_way(&round_the_cap_forwards),
    };
    let the_wall_goes_round_the_cap = walked_the_other_way(&the_side_goes_round_the_cap);
    let end_circle = builder.add_edge(
        end_corner,
        end_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let mut rounds: Vec<([i32; 3], Vec<[i32; 3]>)> = Vec::with_capacity(outline.len() + 2);
    for (position, &[x, y]) in outline.iter().enumerate() {
        let [to_x, to_y] = outline[(position + 1) % outline.len()];
        assert!(
            x == to_x || y == to_y,
            "({x}, {y}) to ({to_x}, {to_y}) is not along an axis"
        );
        rounds.push((
            [(to_y - y).signum(), (x - to_x).signum(), 0],
            vec![[x, y, -2], [to_x, to_y, -2], [to_x, to_y, 2], [x, y, 2]],
        ));
    }
    rounds.push(([0, 0, 1], outline.iter().map(|&[x, y]| [x, y, 2]).collect()));
    rounds.push((
        [0, 0, -1],
        outline[..1]
            .iter()
            .chain(outline[1..].iter().rev())
            .map(|&[x, y]| [x, y, -2])
            .collect(),
    ));

    let mut straight = BTreeMap::new();
    let mut sides = Vec::new();
    for (normal, round) in rounds {
        let mut steps = Vec::with_capacity(round.len());
        for position in 0..round.len() {
            let from = round[position];
            let to = round[(position + 1) % round.len()];
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
            let mut round_the_cap = Vec::with_capacity(the_side_goes_round_the_cap.len());
            for &(edge, turn) in &the_side_goes_round_the_cap {
                round_the_cap.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
            }
            wires.push(builder.add_wire(&round_the_cap)?);
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
        let coedge = builder.add_coedge(end_circle, Orientation::Reversed)?;
        let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
        builder.add_face(Surface::Plane(plane), &[wire])?
    };

    let mut round_the_wall = Vec::with_capacity(5);
    for (edge, turn) in the_wall_goes_round_the_cap.into_iter().chain([
        (seam, Orientation::Forward),
        (end_circle, Orientation::Forward),
        (seam, Orientation::Reversed),
    ]) {
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
    shell.push((wall, Orientation::Forward));
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The same edges walked back the way they came: last first, each turned round.
fn walked_the_other_way(walk: &[(DraftEdge, Orientation)]) -> Vec<(DraftEdge, Orientation)> {
    walk.iter()
        .rev()
        .map(|&(edge, turn)| (edge, turn.reversed()))
        .collect()
}

/// The half of the cap circle round `(4, 0, 0)` on the side of `z = 0` that
/// `side`, `1` or `−1`, points to.
fn half_of_the_cap(side: f64) -> EdgeRun {
    let the_blocks_side = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(the_blocks_side),
        picked_by: smallvec![
            Picker::from(
                HalfSpace::new(Vector::new(1.0, 0.0, 0.0), 0.0).expect("a normal with a length")
            ),
            Picker::from(
                HalfSpace::new(Vector::new(0.0, 0.0, side), 0.0).expect("a normal with a length")
            ),
        ],
    }
}
