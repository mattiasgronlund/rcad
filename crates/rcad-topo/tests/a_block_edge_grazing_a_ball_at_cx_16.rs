//! §244.3's block edge along `x`, grazing a ball centred at `x = 16`: the four
//! questions a fuse of the two used to refuse on, each now decided, and what it
//! refuses on after them.
//!
//! # The shape
//!
//! A ball of radius `1 + 2⁻⁵¹` centred at `(16, 0, 0)`, and a block whose top
//! is `z = −2⁻²⁵` and whose side is `y = 1`. That edge meets the ball at
//! `x = 16 ± 2⁻⁵¹`, which are not `f64`s: the two places come back as the boxes
//! `[15.999999999999998, 16]` and `[16, 16.000000000000004]`.
//!
//! # The order of the two places round each circle is decided
//!
//! The walked start of both the top's circle and the side's is `(16, 1, −2⁻²⁵)`,
//! on the edge between the two places and `2⁻¹⁰³` inside the ball. All three
//! points lie on one line along `x`, so going round either circle from that
//! start the two places sweep exactly zero, and their boxes touch at `16`.
//! From §244.5 until Q699 a certified fuse refused on which of the two comes
//! first. It is decided now: the one point of either box at `x = 16` is the
//! start, which is not on the ball, so one place is proved past the start and
//! the other short of it, and the one past it along the way the circle runs
//! comes first (`fuse/arrange.rs`,
//! `two_places_whose_boxes_end_at_the_start_put_the_one_past_it_first`).
//!
//! # The turn at each place is decided
//!
//! Splitting the ball's face, `Arrangement::turns_anticlockwise` asks which way
//! two crossings turn at a place they meet: the sign of the determinant of the
//! ball's gradient there and the two planes' normals. Over the place's box the
//! gradient's `x` is `[−2⁻⁴⁹, 0]` and the sign cannot be told. It is `∓2⁻⁵¹`,
//! never zero (the second test below), and since Q914 it is read off which of
//! the edge's two meets with the ball the place is, by two exact signs of the
//! ball along the edge (`fuse/arrange.rs`,
//! `the_turn_on_a_balls_face_where_a_block_edge_grazes_it_is_read_off_the_root`).
//! Until then every certified fuse with the ball first refused on it, the gap
//! `docs/decisions.md` §317 measured.
//!
//! # The order of the two places along the edge is decided
//!
//! On the block's top and side the two places lie on the edge, in boxes that
//! share only `16` along it, so read as boxes they are apart by a range from
//! nothing up. Until Q914 every certified fuse with the block first refused on
//! that. The edge's point at `16` is `2⁻¹⁰³` inside the ball, so neither place
//! is at `16`, and the one whose box ends there comes first along the edge's
//! `x` (`Arrangement::as_the_quadric_says`).
//!
//! # The sliver between the two places is placed
//!
//! Every fuse, with either solid first and under either policy, then refused on
//! a point inside a piece of a face between the two places. The two places come
//! out one corner, `(16, 1, −2⁻²⁵)`, and both arcs
//! of the ball's circle through them run from that corner back to it.
//! `Edges::of` named both arcs as one closed edge, so §365.3's side rule matched
//! the ball's sliver to the block's top along the long arc, inside the block
//! there, and to the block's wall across its true side, outside it, and
//! `select` refused the sliver. Since Q1073 the side rule names such an arc by
//! the names of its two places and which way it runs between them
//! (`fuse/assemble.rs`, `the_edge_each_side_of_every_piece_is`), so the two arcs
//! are two names and the sliver is placed by its side on the wall
//! (`fuse/select.rs`,
//! `the_sliver_of_a_ball_between_two_places_one_corner_apart_is_placed_by_its_side_on_the_wall`).
//!
//! # What it refuses on next
//!
//! Every fuse then refuses while it puts the answer together, the same under
//! either policy (the pin below):
//!
//! | rule         | ball first                    | block first                   |
//! |--------------|-------------------------------|-------------------------------|
//! | union        | an edge with 4 faces along it | an edge with 4 faces along it |
//! | intersection | a corner twice in one loop    | a corner twice in one loop    |
//! | difference   | an edge with 4 faces along it | a corner twice in one loop    |
//!
//! "An edge with 4 faces" is `ResultIsNotManifold { count: 4 }`: the answer's
//! own naming makes the two arcs through the one corner one closed edge again,
//! so the long arc's two faces and the short arc's two faces all lie along it.
//! "A corner twice in one loop" is `CannotMeshAFace` with
//! `CornersCoincide { position: 5 }`, from the mesh that measures each shell
//! before the answer is committed: a face's loop runs into the one corner, round
//! a closed edge, and out of it again, so its sixth corner is an earlier one.
//!
//! This is a limit, refused under the wrong name. The answer needs a corner at
//! each place, `16 − 2⁻⁵¹` and `16 + 2⁻⁵¹`, and `f64` holds neither: its spacing
//! at `16` is `2⁻⁴⁸`. The two would be an edge `2⁻⁵⁰` long in a shape whose
//! coordinates run to `18`, a spread of about `2⁵⁴`, where the kernel supports
//! `1e11` (`rcad-geom`'s `LARGEST_SUPPORTED_RATIO`), and scaling the whole
//! shape by a power of two moves both lengths alike. The honest refusal is
//! `Invalid::DynamicRangeTooWide`, and nothing gives it: the fuse's
//! `check_dynamic_range` skips an edge whose two ends are one corner, and runs
//! after both checks that refuse above (§317's Q1073 note). No fuse answers, so
//! none is held to a volume.

use rcad_geom::{
    Isometry, MeetOfThree, Plane, Point, Scale, Sphere, Surface, Vector, meet_of_three_surfaces,
};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, orient3d_of_points,
};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule};

/// Where each fuse stops, with the ball first or not.
const WHERE_EACH_FUSE_STOPS: [(bool, Rule, Stop); 6] = [
    (true, Rule::Union, Stop::AnEdgeWithFourFaces),
    (true, Rule::Intersection, Stop::ACornerTwiceInOneLoop),
    (true, Rule::Difference, Stop::AnEdgeWithFourFaces),
    (false, Rule::Union, Stop::AnEdgeWithFourFaces),
    (false, Rule::Intersection, Stop::ACornerTwiceInOneLoop),
    (false, Rule::Difference, Stop::ACornerTwiceInOneLoop),
];

/// The two refusals the fuses stop on.
///
/// Exhaustive on purpose: it is this file's own table, and the pin matches it
/// whole.
#[derive(Clone, Copy, Debug)]
enum Stop {
    /// `ResultIsNotManifold { count: 4 }`.
    AnEdgeWithFourFaces,
    /// `CannotMeshAFace` with `CornersCoincide { position: 5 }`.
    ACornerTwiceInOneLoop,
}

/// The pin: every fuse places the sliver between the two places, and then
/// refuses while it puts the answer together, as the table in the module doc
/// has it, under either policy.
///
/// No fuse answers, so none is validated or held to its volume.
///
/// Until Q1073 every one refused on a point inside a piece of a face between
/// the two places, with those two places as the record's inputs. Until Q914 a
/// certified fuse refused on the turn at the place below `16` with the ball
/// first, and on the order of the two places along the edge with the block
/// first; under `Policy::tolerant(25.0)` every one was
/// `ResultIsNotManifold { count: 3 }`.
#[test]
fn every_fuse_refuses_on_the_two_places_f64_holds_as_one_corner() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for (ball_first, rule, stop) in WHERE_EACH_FUSE_STOPS {
            let case = (policy.strictness(), ball_first, rule, stop);
            let given = fuse_a_block_edge_grazing_a_ball(
                rule,
                &policy,
                ball_first,
                &mut ProvenanceLog::new(),
            );
            let as_pinned = match stop {
                Stop::AnEdgeWithFourFaces => {
                    matches!(given, Err(Error::ResultIsNotManifold { count: 4 }))
                }
                Stop::ACornerTwiceInOneLoop => matches!(
                    given,
                    Err(Error::CannotMeshAFace {
                        why: CannotTriangulate::CornersCoincide { position: 5 },
                        ..
                    })
                ),
            };
            assert!(as_pinned, "{case:?}: {given:?}");
        }
    }
}

/// The measurement that made the turn a gap: the sign it needs exists, and
/// the same three surfaces solved about the ball's own centre reach it.
///
/// Moving the ball to the origin moves nothing else. Both planes' normals have
/// no `x` part, so a shift along `x` leaves their four numbers as they were,
/// and the two problems are one problem `16` apart. The turn reads only the
/// place *minus the centre*, so it is the same number in both.
#[test]
fn the_sign_that_turn_needs_is_there_when_the_ball_is_solved_about_its_own_centre() {
    let (lid, side) = the_block_edge();
    for flat in [lid, side] {
        assert_eq!(
            flat.normal().components()[0],
            0.0,
            "a shift along x must leave the plane as it was"
        );
    }

    let placed = where_the_edge_meets_a_ball_centred_at(16.0);
    let centred = where_the_edge_meets_a_ball_centred_at(0.0);

    for place in &placed {
        let turn = the_turn_at(gradient_of_the_ball(*place, 16.0));
        assert!(
            matches!(turn, Decision::Undecidable { .. }),
            "about x = 16 the box of the place touches the centre: {place:?} gave {turn:?}"
        );
    }

    let quarter = f64::powi(2.0, -51);
    let top = -f64::powi(2.0, -25);
    let mut signs = Vec::new();
    for place in &centred {
        let [x, y, z] = collapsed(*place);
        assert!(x == -quarter || x == quarter, "{place:?}");
        assert_eq!([y, z], [1.0, top]);
        let turn = the_turn_at(gradient_of_the_ball(*place, 0.0));
        let expected = if x < 0.0 {
            Decision::Pos
        } else {
            Decision::Neg
        };
        assert_eq!(turn, expected, "the determinant is minus x, at x = {x:e}");
        signs.push(turn);

        let holding_it = placed
            .iter()
            .filter(|world| {
                let [box_of_x, ..] = gradient_of_the_ball(**world, 16.0).enclosure();
                box_of_x.lo() <= x && x <= box_of_x.hi()
            })
            .count();
        assert_eq!(
            holding_it, 1,
            "each place about the centre is inside exactly one place about x = 16"
        );
    }
    signs.sort_by_key(|sign| *sign == Decision::Pos);
    assert_eq!(signs, [Decision::Neg, Decision::Pos]);
}

fn fuse_a_block_edge_grazing_a_ball(
    rule: Rule,
    policy: &Policy,
    ball_first: bool,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    let centre_x = 16.0;
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let ball = model
        .add_ball(
            1.0 + f64::powi(2.0, -51),
            &Isometry::translation(Vector::new(centre_x, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let square = ProfileBuilder::new(vec![
        [centre_x - 2.0, -2.0],
        [centre_x + 2.0, -2.0],
        [centre_x + 2.0, 1.0],
        [centre_x - 2.0, 1.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let top = -f64::powi(2.0, -25);
    let block = model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, top - 2.0)).expect("a finite shift"),
        )
        .expect("a positive height");
    if ball_first {
        model.fuse(&[ball], &[block], rule, policy, log)
    } else {
        model.fuse(&[block], &[ball], rule, policy, log)
    }
}

/// The block's top, `z = −2⁻²⁵`, and its side, `y = 1`, facing the way the
/// fuse's record of the turn had them.
fn the_block_edge() -> (Plane, Plane) {
    let top = -f64::powi(2.0, -25);
    let lid = Plane::from_point_and_normal(Point::new(0.0, 0.0, top), Vector::new(0.0, 0.0, 1.0))
        .expect("a unit normal");
    let side = Plane::from_point_and_normal(Point::new(0.0, 1.0, 0.0), Vector::new(0.0, 1.0, 0.0))
        .expect("a unit normal");
    (lid, side)
}

fn where_the_edge_meets_a_ball_centred_at(centre_x: f64) -> Vec<PredicatePoint> {
    let (lid, side) = the_block_edge();
    let ball = Sphere::new(Point::new(centre_x, 0.0, 0.0), 1.0 + f64::powi(2.0, -51))
        .expect("a positive radius");
    let meet = meet_of_three_surfaces(
        [
            &Surface::Sphere(ball),
            &Surface::Plane(lid),
            &Surface::Plane(side),
        ],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a ball and two planes");
    let MeetOfThree::At(places) = meet else {
        panic!("the edge grazes the ball at two places: {meet:?}");
    };
    assert_eq!(places.len(), 2, "{places:?}");
    places.to_vec()
}

/// What `fuse/arrange.rs`'s `gradient_at` makes of a ball: the place minus the
/// centre, taken from the place's box.
fn gradient_of_the_ball(place: PredicatePoint, centre_x: f64) -> PredicatePoint {
    let [x, y, z] = place.enclosure();
    PredicatePoint::Constructed([x - Interval::exactly(centre_x), y, z])
}

/// `turns_anticlockwise`'s determinant, with the rows in the order the fuse's
/// record of it had them.
fn the_turn_at(gradient: PredicatePoint) -> Decision {
    orient3d_of_points(
        gradient,
        PredicatePoint::from([0.0, 0.0, 1.0]),
        PredicatePoint::from([0.0, 1.0, 0.0]),
        PredicatePoint::from([0.0; 3]),
    )
}

/// The one `f64` triple a box holds.
///
/// # Panics
///
/// If any side of the box is wider than a point.
fn collapsed(place: PredicatePoint) -> [f64; 3] {
    place.enclosure().map(|side| {
        assert_eq!(side.lo(), side.hi(), "{place:?} is not one point");
        side.lo()
    })
}
