//! A ball in two faces on its one sphere, fused again with a cube (row Q903,
//! `docs/decisions.md` §220.1, §400.5).
//!
//! Two solids here carry such a ball. One is §220.1's: a ball of radius 5 split
//! down its polar axis into two lunes, built through `SolidBuilder`, the rim
//! between them naming the plane `x = 0`. The other is what a union writes: a
//! block from `(-2, -2, 0)` to `(0, 0, 2)` and a ball of radius 1 at
//! `(1, -0.5, 1)` touching its face `x = 0`, whose plane `y = 0` cuts the ball
//! in two (`a_rim_that_names_its_cutting_surface.rs`).
//!
//! # Beside a far cube they answer
//!
//! Both were refused `AFaceMeetsItsNeighbourOnItsOwnSurface` at
//! `Arrangement::crossing_for` (rows Q376, Q584, Q890): the arrangement asked
//! where one face meets the face across the rim, and that is the same sphere.
//! Since row Q575 the bake hands the faces of an input on one ball over as one
//! face and leaves the rim out the way it leaves a seam out, so nothing asks.
//! The lunes beside a far cube answer under every rule both ways round. So does
//! the draw row Q171 found, the same ball at `(1, 0.5, 1)`, half a unit clear
//! of the block: since §400 that ball is not cut at all, and comes back whole.
//!
//! # Where a point has to be placed against the ball, they do not
//!
//! Neither ball in two faces can say whether a point is inside it.
//! `classify_point` comes back `CannotFollowAFacesTrimming` at the ball's own
//! centre, because `face_holds_the_point` takes the face across the rim, on the
//! same sphere, for what trims each face, and refuses
//! `AFaceMissesPartOfWhereItsSurfacesCross` at every point asked: row Q977's bar
//! in two bands, on a ball. So a fuse that has to place a piece against the ball
//! is refused `Undecidable` at "a point in a piece that every solid could
//! place". The union of the touching draw is refused fused again even with a
//! far cube, because its block's face `x = 0` still touches the ball and has to
//! be placed against it; and the lunes are refused with a cube across their top.
//! Every one of those answers exists, so under §16.10 it is a gap, and it is
//! pinned here by name until `holds.rs` reads the rim by the plane its run
//! names, as the mesher and `validate` already do.

use core::f64::consts::PI;

use rcad_geom::{
    Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, Error, Model, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;
const LUNES: f64 = 4.0 * PI * RADIUS * RADIUS * RADIUS / 3.0;
const SMALL_BALL: f64 = 4.0 * PI / 3.0;
const BLOCK: f64 = 8.0;
const CUBE: f64 = 8.0;
const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];
const PLACED_AGAINST_EVERY_SOLID: &str = "a point in a piece that every solid could place";

/// §220.1's lunes beside a cube of side 2 at `(20, 20, 20)`: under every rule,
/// with the lunes as the argument and as the tool, and with their rim walked
/// either way, the answer is the solids as they stand.
#[test]
fn two_lunes_beside_a_far_cube_come_back_as_they_stand() {
    for way in [Orientation::Forward, Orientation::Reversed] {
        for (rule, lunes_first, truths) in every_fuse_of_two_apart(&[LUNES], &[CUBE]) {
            let what = format!("{rule:?}, rim walked {way:?}, lunes first {lunes_first}");
            let (mut model, lunes) = two_lunes(way);
            let cube = a_cube(&mut model, 2.0, [20.0, 20.0, 20.0]);
            let (arguments, tools) = if lunes_first {
                ([lunes], [cube])
            } else {
                ([cube], [lunes])
            };
            let answer = fused(&mut model, &arguments, &tools, rule, &what);
            assert_the_solids_measure(&model, &answer, &truths, &what);
        }
    }
}

/// Row Q171's draw, a block and a ball half a unit clear of it united, then
/// fused with a cube of side 2 at `(20, 20, 20)`. The ball comes back from the
/// union in its one face, and every fuse after it is the three solids as they
/// stand, or those the rule keeps.
#[test]
fn a_ball_clear_of_a_block_united_and_fused_again_with_a_far_cube_comes_back_as_it_stands() {
    for ball_first in [true, false] {
        let (mut model, first) = a_block_and_a_ball_united([1.0, 0.5, 1.0], ball_first);
        assert_eq!(
            faces_on_a_ball(&model, &first),
            1,
            "ball first {ball_first}: a ball clear of the block is not cut (§400)"
        );
        for (rule, union_first, truths) in every_fuse_of_two_apart(&[BLOCK, SMALL_BALL], &[CUBE]) {
            let what = format!("ball first {ball_first}, {rule:?}, union first {union_first}");
            let mut model = model.clone();
            let cube = a_cube(&mut model, 2.0, [20.0, 20.0, 20.0]);
            let (arguments, tools) = if union_first {
                (first.clone(), vec![cube])
            } else {
                (vec![cube], first.clone())
            };
            let answer = fused(&mut model, &arguments, &tools, rule, &what);
            assert_the_solids_measure(&model, &answer, &truths, &what);
        }
    }
}

/// The same draw with the ball moved to touch the block's face `x = 0`. The
/// union keeps the ball in two faces either side of `y = 0`, and fused again
/// with a far cube every rule both ways round is refused where the block's face
/// `x = 0` is placed against the ball.
///
/// The answers are the block, the ball and the cube as they stand under a
/// union, nothing under an intersection, the block and the ball less the cube,
/// and the cube less them.
#[test]
fn a_ball_touching_a_block_united_and_fused_again_with_a_far_cube_is_refused_where_it_places_a_point()
 {
    let (model, first) = a_block_and_a_ball_united([1.0, -0.5, 1.0], false);
    assert_the_solids_measure(&model, &first, &[BLOCK, SMALL_BALL], "the union");
    assert_eq!(
        faces_on_a_ball(&model, &first),
        2,
        "the union keeps the ball in two faces"
    );
    for rule in RULES {
        for union_first in [true, false] {
            let what = format!("{rule:?}, union first {union_first}");
            let mut model = model.clone();
            let cube = a_cube(&mut model, 2.0, [20.0, 20.0, 20.0]);
            let (arguments, tools) = if union_first {
                (first.clone(), vec![cube])
            } else {
                (vec![cube], first.clone())
            };
            let outcome = model.fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert_eq!(
                refused_at(outcome, &what),
                PLACED_AGAINST_EVERY_SOLID,
                "{what}"
            );
        }
    }
}

/// The lunes with a cube of side 12 across their top, from `(-6, -6, 3)`, which
/// takes the cap past `z = 3` off both of them and reaches the ball nowhere
/// else. Every rule both ways round, with the rim walked either way, is refused
/// where a piece is placed against the lunes.
///
/// The cap is `πh²(3r − h)/3` with `h = 2`, `52π/3`. The answers are the
/// lunes and the cube less the cap, `448π/3 + 1728`; the cap; the lunes less it,
/// `448π/3`; and the cube less it, `1728 − 52π/3`.
#[test]
fn two_lunes_with_a_cube_across_their_top_are_refused_where_they_place_a_point() {
    for way in [Orientation::Forward, Orientation::Reversed] {
        for rule in RULES {
            for lunes_first in [true, false] {
                let what = format!("{rule:?}, rim walked {way:?}, lunes first {lunes_first}");
                let (mut model, lunes) = two_lunes(way);
                let cube = a_cube(&mut model, 12.0, [-6.0, -6.0, 3.0]);
                let (arguments, tools) = if lunes_first {
                    ([lunes], [cube])
                } else {
                    ([cube], [lunes])
                };
                let outcome = model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                assert_eq!(
                    refused_at(outcome, &what),
                    PLACED_AGAINST_EVERY_SOLID,
                    "{what}"
                );
            }
        }
    }
}

/// What stops both refusals above: a ball in two faces cannot say its own
/// centre is inside it, though nothing about the point is close to anything.
#[test]
fn a_ball_in_two_faces_cannot_say_its_own_centre_is_inside() {
    for way in [Orientation::Forward, Orientation::Reversed] {
        let (model, lunes) = two_lunes(way);
        let found = model
            .solid(lunes)
            .expect("this model made the key")
            .classify_point(
                Point::origin(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
        assert!(
            matches!(found, Containment::CannotFollowAFacesTrimming { .. }),
            "the lunes, rim walked {way:?}: {found:?}"
        );
    }

    let (model, first) = a_block_and_a_ball_united([1.0, -0.5, 1.0], false);
    let [_, ball] = first[..] else {
        panic!("the block and the ball apart, got {first:?}");
    };
    assert_eq!(
        faces_on_a_ball(&model, &[ball]),
        2,
        "the second solid is the ball"
    );
    let found = model
        .solid(ball)
        .expect("this model made the key")
        .classify_point(
            Point::new(1.0, -0.5, 1.0),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
    assert!(
        matches!(found, Containment::CannotFollowAFacesTrimming { .. }),
        "the ball the union wrote: {found:?}"
    );
}

/// Every rule, with `one` as the argument and as the tool, and the volumes of
/// the solids each answer is when the two never meet: both sides as they stand
/// under a union, nothing under an intersection, and the argument under a
/// difference.
fn every_fuse_of_two_apart(one: &[f64], other: &[f64]) -> [(Rule, bool, Vec<f64>); 6] {
    let both = [one, other].concat();
    [
        (Rule::Union, true, both.clone()),
        (Rule::Union, false, both),
        (Rule::Intersection, true, Vec::new()),
        (Rule::Intersection, false, Vec::new()),
        (Rule::Difference, true, one.to_vec()),
        (Rule::Difference, false, other.to_vec()),
    ]
}

/// The two lunes of the ball of `RADIUS` about the origin, with the great
/// circle between them written as a rim that names the plane `x = 0` and walked
/// `way` by the first of them. Its one corner is `(0, 5, 0)`.
fn two_lunes(way: Orientation) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(25.0 * RADIUS).expect("a positive size"));
    let skin = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let cut = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal");

    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(0.0, RADIUS, 0.0))
        .expect("a point on the ball and on the plane");
    let rim = builder
        .add_edge(
            corner,
            corner,
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(cut),
            },
        )
        .expect("a rim is a whole closed curve, so one corner twice is an edge");
    let mut lune = |walked| {
        let step = DraftStep::Along(
            builder
                .add_coedge(rim, walked)
                .expect("an edge of this builder"),
        );
        let wire = builder.add_wire(&[step]).expect("a closed wire");
        builder
            .add_face(skin, &[wire])
            .expect("a face with one wire")
    };
    let east = lune(way);
    let west = lune(way.reversed());
    builder
        .add_shell(&[(east, Orientation::Forward), (west, Orientation::Forward)])
        .expect("two lunes close up");
    let solid = builder
        .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball split in two is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball split in two is a sound model");
    (model, solid)
}

/// A cube of side 2 from `(-2, -2, 0)` united with a ball of radius 1 at
/// `centre`, with the ball as the argument or as the tool.
fn a_block_and_a_ball_united(centre: [f64; 3], ball_first: bool) -> (Model, Vec<SolidKey>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let block = a_cube(&mut model, 2.0, [-2.0, -2.0, 0.0]);
    let ball = model
        .add_ball(1.0, &shifted(centre))
        .expect("a ball of radius 1");
    let (arguments, tools) = if ball_first {
        ([ball], [block])
    } else {
        ([block], [ball])
    };
    let answer = fused(
        &mut model,
        &arguments,
        &tools,
        Rule::Union,
        &format!("the block and the ball at {centre:?}"),
    );
    (model, answer)
}

fn a_cube(model: &mut Model, side: f64, corner: [f64; 3]) -> SolidKey {
    let cube = model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(cube, &shifted(corner))
        .expect("this model made the key");
    cube
}

fn shifted([x, y, z]: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

/// The fuse's solids, with the model it wrote them into checked sound.
fn fused(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
    what: &str,
) -> Vec<SolidKey> {
    let answer = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: refused {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    answer
}

/// How many faces of `solids` are on a ball.
fn faces_on_a_ball(model: &Model, solids: &[SolidKey]) -> usize {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
                .count()
        })
        .sum()
}

/// One solid for each of `truths`, each measuring as its own one inside the
/// bound its measurement publishes for itself.
fn assert_the_solids_measure(model: &Model, answer: &[SolidKey], truths: &[f64], what: &str) {
    assert_eq!(
        answer.len(),
        truths.len(),
        "{what}: {answer:?} against {truths:?}"
    );
    let mut left = truths.to_vec();
    for &solid in answer {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
        let at = left
            .iter()
            .position(|&truth| (measured.volume() - truth).abs() <= measured.volume_error())
            .unwrap_or_else(|| {
                panic!(
                    "{what}: {} measured, {} off, against {left:?}",
                    measured.volume(),
                    measured.volume_error()
                )
            });
        left.remove(at);
    }
}

/// The predicate the first record of an `Undecidable` refusal names.
fn refused_at<T: core::fmt::Debug>(outcome: Result<T, Error>, what: &str) -> &'static str {
    let Err(Error::Undecidable { records }) = outcome else {
        panic!("{what}: refused Undecidable, and came back {outcome:?}");
    };
    records
        .records()
        .first()
        .unwrap_or_else(|| panic!("{what}: names what it could not decide"))
        .predicate()
}
