//! A piece of a pointed cone cut by a lid a hair off one of its rulings, so
//! that the lid's ellipse reaches millions of times further up the axis than
//! the two ends of the arc the piece keeps (`docs/decisions.md` §406.5).
//!
//! # The shape
//!
//! The cone has slope a half about `+z`, apex at the origin. A lid through two
//! ends `(w, ±0.5, 1.25)` leans `lean` past the ruling `(0.5, 0, 1)`: its
//! normal is `(−1, 0, 0.5 + lean)`, so it cuts the cone in an ellipse whose far
//! end is on that ruling `c / lean` up, for `c` the lid's offset along that
//! normal: `2²⁴ + 1.25`, or `1.7e7`, for both shapes here. A wall
//! `x = w` cuts the ellipse at the two ends, and the wall's hyperbola joins
//! them below. Both ends are on the cone to the bit, `0.375² + 0.5² = 0.625²`,
//! which `Model::validate` asks of a curved face's corners. Three faces and
//! three edges, through `SolidBuilder`.
//!
//! **The sliver** (`w = −0.375`) keeps the side of the wall the far end is on,
//! so the cone face's curved edge on the lid is the arc **past** the far end:
//! a face `1.7e7` long whose edges all end `1.25` up. **The tip** (`w = 0.375`)
//! keeps the other side, the arc that **misses** the far end, which runs from
//! `53°` either side of the far end round through the near end: a face under
//! `1.25` up, but with its ends `1.4` from the apex while the apex is `0.22`
//! from the lid, which is where the ruling runs nearly along the lid.
//!
//! # The sliver: charged by its ends, then by the ellipse
//!
//! The mesher charged an arc of such an ellipse by its two ends whichever arc
//! it was, and `rcad_geom::walk` walks only the arc that misses the far end
//! from its ends. So the sliver's floor came out `4.5e-11`, and every
//! deflection between that and the `5.3e-7` the walk can place a point to was
//! handed to the walk: refused `CannotPlaceAPointThatCloseToACrossing` under a
//! certified policy, refused the same way for a floor the mesher had picked
//! itself under `Policy::tolerant(1e4)`, and under `Policy::tolerant(1e9)`
//! waved through with nothing between the ends, which left the cone face
//! `ACurvedFaceGoesRightRoundWithNoSeam` — a refusal that names a malformed
//! shape, on a solid that validates. The mesher now asks the walk which arc it
//! is, charges the sliver its ellipse, and refuses every one of those by its
//! own name, `DeflectionBelowWhatAMeshCanReach`, at a floor of `6.1e-4`.
//!
//! **They stay refusals, and that is not this file's to change.** Under
//! `6.1e-4` is under what the walk's published charge for a point of a
//! crossing `1.7e7` out, `POINT_ALONG_A_CROSSING_ROUNDOFF` of it, leaves room
//! for, which every crossing that size pays. What was wrong was who refused
//! and by what number, and the floor a tolerant caller was handed. A tolerant
//! policy wide enough to take the floor itself, `Policy::tolerant(1e12)`, now
//! gets a mesh tried at `1.2e-3` where it got the malformed-shape refusal; that
//! run went past seven minutes and was not finished, so it is not pinned.
//!
//! # The sliver meshes over its floor
//!
//! Until Q2178 a caller-named deflection far above the floor was refused
//! `TwoPatchesOfTheMeshDrewTheSameSide`, at each of `1`, `10`, `100`, `1e3`
//! and `1e4` it was asked at, and so was the same sliver with a lid leaning
//! `2⁻⁶` past the ruling, at `0.1`, while `2⁻⁵` meshed. Q2178 keeps out of a
//! curved patch every side between two corners of one edge with a corner of
//! the edge between them, and each of those meshes now, down to a lean of
//! `2⁻¹⁶` at `0.1`. The pin is `a_sliver_leaning_a_sixty_fourth_or_less_meshes`.
//!
//! # The tip: a floor the mesher picks that the walk will not count to
//!
//! The arc that misses the far end is walked round the cone in steps of equal
//! angle, counted by a bound on its bend over the whole arc, and that bound is
//! largest at the ends, where the ruling runs nearly along the lid. At the
//! floor the mesher picks for a tolerant caller, `9.1e-11`, the count passes
//! `MOST_POINTS_ALONG_ONE_CROSSING` and the walk refuses
//! `TooManyPointsAlongOneCrossing`, for a number no caller chose. A certified
//! mesh at `1e-6` exists, `1404459` points, and so does one at `1e-3`. Whether
//! one exists at the floor is not shown: by estimate the arc bends at a radius
//! no smaller than the apex's distance from the lid, which wants a few hundred
//! thousand chords at that floor, under the cap, but that is an estimate and
//! not a mesh. The pin is the refusal.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, SolidBuilder, SolidKey};

const SLOPE: f64 = 0.5;

/// The sliver's lid: its ellipse reaches `2²⁴ + 1.25` up the axis.
const THE_SLIVERS_LEAN: f64 = 1.0 / 16_777_216.0;

/// The tip's lid: its ellipse reaches `2²⁴ + 1.25` up the axis too, with the
/// apex four times nearer the lid.
const THE_TIPS_LEAN: f64 = 1.0 / 67_108_864.0;

/// The sliver is a sound solid, and above its floor it meshes, and so does
/// the same sliver with its lid leaning a sixty-fourth past the ruling. Until
/// Q2178 both were refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4,
/// used: 4 }`, and a thirty-second was the steepest that meshed.
#[test]
fn a_sliver_leaning_a_sixty_fourth_or_less_meshes() {
    let mut model = a_model();
    let sliver = the_sliver(&mut model);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the sliver is a sound solid");
    let meshed = |model: &Model, solid: SolidKey, asked: f64, what: &str| {
        let mesh = model
            .solid(solid)
            .expect("this model made the key")
            .mesh(
                Deflection::of(asked).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} at {asked:e}: {why:?}"));
        let strays = mesh.deviation().magnitude();
        assert!(strays <= asked, "{what} at {asked:e} strays {strays}");
    };
    for asked in [100.0, 1e4] {
        meshed(&model, sliver, asked, "the sliver");
    }
    for lean in [1.0 / 32.0, 1.0 / 64.0] {
        let piece = a_piece_of_the_cone(&mut model, lean, -0.375, 1.0);
        meshed(&model, piece, 0.1, &format!("leaning {lean}"));
    }
}

/// **Fails without the change**: a certified caller asking for less than the
/// sliver's floor is refused by the mesher, at the floor of the ellipse it
/// really reaches, and not by the walk.
///
/// Charged by its ends, the floor was `4.5e-11`, `1e-6` was let through, and
/// the walk refused it `CannotPlaceAPointThatCloseToACrossing` with a closest
/// of `5.3e-7`.
#[test]
fn a_deflection_under_the_slivers_floor_is_refused_by_the_mesher() {
    let refused = the_sliver_meshed(1e-6, &Policy::certified());
    let Error::DeflectionBelowWhatAMeshCanReach { closest, .. } = refused else {
        panic!("refused by the walk rather than the mesher: {refused:?}")
    };
    assert_eq!(closest, 0.000_610_122_869_319_951_1);
}

/// **Fails without the change**: a tolerant caller whose budget is under the
/// sliver's floor is refused that floor by name, whatever it asked for.
///
/// Charged by its ends, `Policy::tolerant(1e4)` was handed a floor of
/// `9.1e-11` the mesher picked itself and the walk refused it, and
/// `Policy::tolerant(1e9)` had the arc waved through with nothing between its
/// ends and the cone face refused as `ACurvedFaceGoesRightRoundWithNoSeam`.
#[test]
fn a_tolerant_caller_is_refused_the_slivers_floor_by_name() {
    for budget in [1e4, 1e9] {
        for asked in [1e-6, 1e-300] {
            let refused = the_sliver_meshed(asked, &Policy::tolerant(budget));
            let Error::DeflectionBelowWhatAMeshCanReach { closest, .. } = refused else {
                panic!("tolerant({budget:e}) asked {asked:e}: {refused:?}")
            };
            assert_eq!(closest, 0.000_610_122_869_319_951_1);
        }
    }
}

/// The tip meshes at a deflection a caller names, and its mesh stays under
/// its ends, `1.25` up, though its ellipse reaches `1.7e7`.
#[test]
fn the_tip_meshes_under_its_ends() {
    let mut model = a_model();
    let tip = the_tip(&mut model);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the tip is a sound solid");
    let mesh = model
        .solid(tip)
        .expect("this model made the key")
        .mesh(
            Deflection::of(1e-3).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the tip meshes at a thousandth");
    let highest = mesh
        .points()
        .iter()
        .map(|point| point.coordinates()[2])
        .fold(0.0, f64::max);
    assert_eq!(highest, 1.25);
}

/// **The pin on the second half of the row**: at the floor the mesher picks
/// for a tolerant caller, the tip's arc round the cone is counted past
/// `MOST_POINTS_ALONG_ONE_CROSSING`, and the walk refuses the number the
/// mesher chose. The module doc says why it is pinned and not fixed.
#[test]
fn the_tip_at_the_floor_the_mesher_picks_is_too_many_points_round_the_cone() {
    let mut model = a_model();
    let tip = the_tip(&mut model);
    for budget in [1e4, 1e9] {
        let refused = model
            .solid(tip)
            .expect("this model made the key")
            .mesh(
                Deflection::of(1e-300).expect("a positive length"),
                &Policy::tolerant(budget),
                &mut ProvenanceLog::new(),
            )
            .expect_err("the walk will not count to the floor");
        assert!(
            matches!(
                refused,
                Error::Geometry(rcad_geom::Error::TooManyPointsAlongOneCrossing {
                    most: 1_048_576
                })
            ),
            "tolerant({budget:e}): {refused:?}"
        );
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn the_sliver_meshed(asked: f64, policy: &Policy) -> Error {
    let mut model = a_model();
    let sliver = the_sliver(&mut model);
    model
        .solid(sliver)
        .expect("this model made the key")
        .mesh(
            Deflection::of(asked).expect("a positive length"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .expect_err("no mesh is that fine")
}

fn the_sliver(model: &mut Model) -> SolidKey {
    a_piece_of_the_cone(model, THE_SLIVERS_LEAN, -0.375, 1.0)
}

fn the_tip(model: &mut Model) -> SolidKey {
    a_piece_of_the_cone(model, THE_TIPS_LEAN, 0.375, -1.0)
}

fn the_cone() -> Cone {
    Cone::new(Axis::z(), SLOPE).expect("a positive slope")
}

/// The part of the inside of the cone on the apex's side of the lid and on
/// the `keeps` side of the wall `x = wall`: `1` keeps `x ≥ wall`, `-1` keeps
/// `x ≤ wall`.
///
/// The cone face is wound clockwise seen from above, which keeps the apex on
/// its left seen from outside: along the lid's arc from one end round to the
/// other, and back along the wall's hyperbola through `(wall, 0, 0.75)`.
fn a_piece_of_the_cone(model: &mut Model, lean: f64, wall: f64, keeps: f64) -> SolidKey {
    let from = Point::new(wall, 0.5 * keeps, 1.25);
    let to = Point::new(wall, -0.5 * keeps, 1.25);
    let lid = Plane::from_point_and_normal(from, Vector::new(-1.0, 0.0, SLOPE + lean))
        .expect("a normal with a length");
    let the_wall =
        Plane::from_point_and_normal(Point::new(wall, 0.0, 0.0), Vector::new(-keeps, 0.0, 0.0))
            .expect("a normal with a length");
    let mut builder = SolidBuilder::new();
    let [start, end] = [from, to].map(|corner| builder.add_vertex(corner).expect("a finite point"));
    let arc = builder
        .add_edge(
            start,
            end,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(lid),
                picked_by: smallvec::smallvec![Picker::InFrontOf(
                    HalfSpace::through_a_point_facing(
                        Point::new(wall, 0.0, 0.0),
                        Vector::new(keeps, 0.0, 0.0)
                    )
                    .expect("a facing with a length")
                )],
            },
        )
        .expect("the arc of the lid's ellipse on the kept side of the wall");
    let below_the_ends =
        HalfSpace::new(Vector::new(0.0, 0.0, -1.0), -1.25).expect("a normal with a length");
    let above_the_apex =
        HalfSpace::new(Vector::new(0.0, 0.0, 1.0), 0.0).expect("a normal with a length");
    let hyperbola = builder
        .add_edge(
            end,
            start,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(the_wall),
                picked_by: smallvec::smallvec![
                    Picker::InFrontOf(below_the_ends),
                    Picker::InFrontOf(above_the_apex),
                ],
            },
        )
        .expect("the wall's hyperbola between the two ends");
    let chord = builder
        .add_edge(start, end, EdgeRun::Straight)
        .expect("the line the lid and the wall share");

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
    let skin_wire = wire_of(
        &mut builder,
        &[
            (arc, Orientation::Forward),
            (hyperbola, Orientation::Forward),
        ],
    );
    let lid_wire = wire_of(
        &mut builder,
        &[(arc, Orientation::Reversed), (chord, Orientation::Forward)],
    );
    let wall_wire = wire_of(
        &mut builder,
        &[
            (hyperbola, Orientation::Reversed),
            (chord, Orientation::Reversed),
        ],
    );
    let skin = builder
        .add_face(Surface::Cone(the_cone()), &[skin_wire])
        .expect("the piece of the cone, pinched at the apex");
    let lid_face = builder
        .add_face(Surface::Plane(lid), &[lid_wire])
        .expect("the lid, between its arc and the chord");
    let wall_face = builder
        .add_face(Surface::Plane(the_wall), &[wall_wire])
        .expect("the wall, between its hyperbola and the chord");
    builder
        .add_shell(&[
            (skin, Orientation::Forward),
            (lid_face, Orientation::Forward),
            (wall_face, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a piece of a cone described this way is a solid")
}
