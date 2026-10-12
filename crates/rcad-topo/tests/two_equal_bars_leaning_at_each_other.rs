//! Two bars of radius 1/4 whose axes meet at a slant (Q1646).
//!
//! The first bar runs along `x` from `through − (2, 0, 0)`, 4 long. The second
//! is turned onto `x` and then by `degrees` about `z`, and is either 32 long and
//! centred on `through = (11, 0, 0)`, so it runs right through the first, or 2
//! long and starts at `through = (11, −3.25, 0.5)`, so its near cap passes
//! through where the axes meet. Each test runs all six fuses — three rules,
//! each bar first in turn. Through `(11, 0, 0)` at 50° to 100° every one
//! answers; the rest are still refused, each by the variant pinned here.
//!
//! **What they should come to**, with `r = 1/4` and `a` the angle: the first
//! bar is `A = π/4`. What two pipes of one width share where their axes cross at
//! `a` is `16r³/(3 sin a)`, Steinmetz's `16r³/3` at a right angle, which is
//! `1/(12 sin a)` here. Through `(11, 0, 0)` the second bar is `B = 2π` and
//! runs well past that solid both ways, so the shared part is `I = 1/(12 sin
//! a)`. Through `(11, −3.25, 0.5)` the second bar is `B = π/8` and starts at
//! the meeting point. Turning every point half a turn about that point keeps
//! both axes where they are, so it keeps the shared solid too, and it swaps the
//! two sides of the cap: the cap cuts the solid in two equal halves, and `I =
//! 1/(24 sin a)`. In both, the union is `A + B − I`, the first less the second
//! `A − I` and the second less the first `B − I`.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

/// The second bar runs right through the first at 50°, 60°, 70° or 100°, and
/// every fuse answers what the module doc says, with `I = 1/(12 sin a)`: the
/// union in 8 faces, the shared part in 4, and each bar less the other as two
/// solids of 3 faces, which touch at the two points the pipes are tangent at.
///
/// Before Q1646 these were refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`,
/// where each bar's far cap cuts its own pipe in a circle and the arrangement
/// could not prove that circle misses the other pipe; and `a_half_of_an_ellipse`
/// did not know a half picked by flat surfaces that do not face along the
/// pipes' stored directions. Both are fixed in `fuse/arrange.rs`.
///
/// Then until Q1660 every fuse was refused telling the cells of a pipe apart,
/// the way the square Steinmetz cross was
/// (`two_bars_of_one_radius_whose_axes_meet_cross_in_four_half_ellipses_and_every_fuse_answers`).
/// The cap circle at `x = 9` is to be matched with one of the four walks the
/// half ellipses cut the pipe into, and which one is read off how often each
/// walk goes round the axis, counted where it crosses a half-plane through the
/// axis. Of the six half-planes tried, two ran along the pipe's own axis, and
/// each of the other four ran through a node: the two points the halves meet
/// at sit on `±z`, and the middles of the four halves on `±y`. The half-plane
/// is now chosen half way across the widest gap between the nodes. Past it
/// every fuse was refused `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` in
/// `fuse/assemble.rs`, where the way along an edge out of a point the two
/// pipes are tangent at was read as nought.
///
/// The long pipe is also cut where it crosses the first bar's cap planes,
/// `x = 9` and `x = 13`, far off either cap. Those cuts are joined back, since
/// each runs right round the pipe as one closed curve
/// (`joined_only_across_whole_rings` in `fuse/assemble.rs`), so the faces
/// counted below are those of the bars and nothing more. The joining keeps the
/// middle of each half ellipse as a corner, since a half folded into one edge
/// from touch point to touch point cannot be named.
#[test]
fn a_long_bar_through_another_at_a_slant_answers_every_fuse() {
    let (first, second) = (PI / 4.0, 2.0 * PI);
    for degrees in [50.0_f64, 60.0, 70.0, 100.0] {
        let shared = 1.0 / (12.0 * degrees.to_radians().sin());
        for (rule, first_first) in six_fuses() {
            let (volume, faces): (f64, &[usize]) = match (rule, first_first) {
                (Rule::Union, _) => (first + second - shared, &[8]),
                (Rule::Intersection, _) => (shared, &[4]),
                (Rule::Difference, true) => (first - shared, &[3, 3]),
                (Rule::Difference, false) => (second - shared, &[3, 3]),
                (_, _) => unreachable!("six_fuses names three rules and no other"),
            };
            let what = format!(
                "{degrees}° {rule:?} (first first: {first_first}), solids of {faces:?} faces and volume {volume}"
            );
            let mut model =
                Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
            let solids = fuse_in(&mut model, degrees, [11.0, 0.0, 0.0], rule, first_first)
                .unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
            answers(&model, &solids, volume, faces, &what);
        }
    }
}

/// The second bar runs right through the first at 120°. Its pickers are its
/// stored directions, so what `a_half_of_an_ellipse` learned in Q1646 does
/// not touch it. Its stored direction is `(−0.5, 0.866…, −2.2e−16)`, a hair off
/// the flat `z = 0`.
///
/// **Every fuse gets past the arrangement and is refused making the faces into
/// triangles, building their wires or naming a joined edge**: `CannotMeshAFace
/// { why: CornersCoincide }`, `AWireWillNotLieFlatOnItsFace`,
/// `ACurvedFaceGoesRightRoundWithNoSeam` or `CannotNameWhichPieceOfACrossingThisIs`,
/// as pinned below. Before Q1646 all six were refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, at the far caps. Not traced
/// further. A lead, not run: the pipe's direction is off `z = 0`, so the two
/// points the halves meet at are off the `±z` half-planes and the count of how
/// often each walk goes round the axis settles, where at 50° it cannot.
///
/// **The unions moved in Q1660.** They were refused `CornersCoincide` with the
/// first bar first and `AWireWillNotLieFlatOnItsFace` with the second. Q1660
/// joins back the cuts the other bar's cap planes make in the long pipe, and
/// that joining walks straight through the middle of each half ellipse unless
/// both of the half's ends are points where the pipes are tangent, read as
/// `∇S × ∇T` exactly nought. Here the two points sit at `y = ±6.4e−17`, and the
/// cross there is `(1.6e−17, 0, 0)` at both, measured with a probe in
/// `the_corners_of_each_run_between_two_tangent_points`, so each half is
/// folded into one edge from touch point to touch point, which cannot be
/// named. Either way it is refused.
///
/// **A gap, by §16.10**: the answer is `1/(12 sin 120°)` shared, inside the
/// range a model holds.
#[test]
fn a_long_bar_through_another_at_120_degrees_is_refused_meshing_or_building_a_wire() {
    for (rule, first_first) in six_fuses() {
        let said = fuse(120.0, [11.0, 0.0, 0.0], rule, first_first);
        let pinned = match (rule, first_first) {
            (Rule::Union, _) => matches!(
                said,
                Err(Error::Geometry(
                    rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs
                ))
            ),
            (Rule::Intersection, false) | (Rule::Difference, true) => {
                matches!(
                    said,
                    Err(Error::CannotMeshAFace {
                        why: rcad_mesh::CannotTriangulate::CornersCoincide { .. },
                        ..
                    })
                )
            }
            (Rule::Difference, false) => {
                matches!(said, Err(Error::AWireWillNotLieFlatOnItsFace { .. }))
            }
            (Rule::Intersection, true) => {
                matches!(said, Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { .. }))
            }
            (_, _) => unreachable!("six_fuses names three rules and no other"),
        };
        assert!(pinned, "{rule:?} (first first: {first_first}): {said:?}");
    }
}

/// The second bar starts where the axes meet, at 50°, 60°, 70°, 100° or 120°.
///
/// **Every fuse is refused**: with the first bar first
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, and with the second first
/// `Undecidable`, asking which side of a pipe a box round `(11, −3.25, 0.25)`
/// is on. Neither moved in Q1646 or Q1660. At 120° both stand since Q1645
/// named the crossing there; before it, `intersect` refused all six
/// `CannotNameWhichPieceOfACrossingThisIs`.
///
/// **Where each is refused (Q1663).** With the first bar first,
/// `place_the_meets` in `fuse/arrange.rs`, splitting the first bar's pipe,
/// asks `meet_of_three_surfaces` where that pipe, the second bar's near cap
/// and the second pipe meet: the pair is the cap's cut across the first pipe
/// and one half ellipse of the two pipes. They do share points, so it is not
/// waved through as two cuts that share nothing. With the second first,
/// `place_the_corners`, splitting a face of the second bar, asks through
/// `passes_through` whether the first pipe runs through the corner of the
/// near cap. That corner is the bottom of `add_round_bar`'s seam, at `(r, 0,
/// 0)` before the move, and the turn onto `x` puts it at `c − r·ẑ`, one of the two points where
/// the pipes touch; its box is `3.1e−13` either way and straddles the first
/// pipe.
///
/// The second bar's near cap is square to its axis through the meeting point
/// `c`, so it holds both points `c ± r·ẑ` where the two pipes touch, `(11,
/// −3.25, 0.25)` and `(11, −3.25, 0.75)`: there the cap and both pipes meet,
/// two of the three tangent to each other. At 50° to 100° the second pipe's
/// stored direction lies in `z = 0`, so both points are exact in `f64`; at 120°
/// it is a hair off `z = 0`, as the test above says. **That holds before the
/// cap is turned, and not in the numbers the arrangement is handed**: the turn
/// rounds the cap, and the cap as stored misses both points by a rounding
/// (`the_near_cap_as_the_model_hands_it_out_misses_both_points_the_pipes_touch_at`).
///
/// **A gap, by §16.10**: the answer is `1/(24 sin a)` shared. The stored cap
/// proves itself off both points, so nothing exact asked of the stored
/// surfaces names them as the meet; the cap holding `c` is the second bar's
/// own incidence, cap through the start of its axis, kept by its move as a
/// seam corner's is on the seam plane (§294.9). Since Q1814 `fuse/bake.rs`
/// reads it on the numbers before the move and carries it, moved, on the
/// second bar's near cap and wall (`caps_through_the_start_of_their_axis`).
/// A first bar's wall told it names the cap's meet with the two pipes as the
/// node where the four halves meet, and is split in six at 50° to 100°
/// (`a_wall_told_a_cap_holds_where_the_axes_meet_is_split_where_the_cap_meets_the_halves`
/// in `fuse/arrange.rs`). **The first bar's wall is not told yet**: gathering
/// the cuts in `fuse/cuts.rs` does not copy it from the face that cuts onto
/// the face cut, so the refusal with the first bar first stands. At 120° a
/// wall told it is refused on the order round the cap's crossing instead,
/// which starts inside the box round `c − r·n̂`. The corner at `c − r·ẑ`
/// needs the same: it is on the cap's circle by construction, and that circle
/// meets the first pipe at `c ± r·ẑ` because the cap holds `c`. Then the
/// cells of the pipes meet the half-planes of the test above.
#[test]
fn a_bar_starting_where_it_crosses_another_at_a_slant_is_refused() {
    for degrees in [50.0, 60.0, 70.0, 100.0, 120.0] {
        for (rule, first_first) in six_fuses() {
            let said = fuse(degrees, [11.0, -3.25, 0.5], rule, first_first);
            let pinned = if first_first {
                matches!(
                    said,
                    Err(Error::Geometry(
                        rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                    ))
                )
            } else {
                matches!(said, Err(Error::Undecidable { .. }))
            };
            assert!(
                pinned,
                "{degrees}° {rule:?} (first first: {first_first}): {said:?}"
            );
        }
    }
}

/// The second bar's near cap, as the model hands it out, is proved off both
/// points where the pipes touch, `(11, −3.25, 0.25)` and `(11, −3.25, 0.75)`,
/// at 50°, 60°, 70° and 100°, where both are exact in `f64`. Before the move
/// the cap is `z = 0` and holds the start of the bar's axis exactly; the turn
/// rounds the cap's normal and offset, and the rounded cap misses `c`. So the
/// meet of the cap with the two pipes cannot be shown to be those two points
/// from the stored cap (Q1663).
#[test]
fn the_near_cap_as_the_model_hands_it_out_misses_both_points_the_pipes_touch_at() {
    for degrees in [50.0, 60.0, 70.0, 100.0] {
        let mut model =
            Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
        let [_, second] = two_bars(&mut model, degrees, [11.0, -3.25, 0.5]);
        let near_cap = model
            .solid(second)
            .expect("the second bar")
            .faces()
            .filter_map(|face| face.surface().as_plane().copied())
            .find(|cap| {
                cap.signed_distance_to(rcad_geom::Point::new(11.0, -3.25, 0.5))
                    .abs()
                    < 1e-12
            })
            .expect("a bar starting at the meeting point has a cap within a rounding of it");
        for z in [0.25, 0.75] {
            let side = near_cap.side_of_point(PredicatePoint::from([11.0, -3.25, z]));
            assert!(
                matches!(side, Decision::Pos | Decision::Neg),
                "{degrees}°: the near cap {near_cap:?} at (11, -3.25, {z}): {side:?}"
            );
        }
    }
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

fn fuse(degrees: f64, through: [f64; 3], rule: Rule, first_first: bool) -> Result<usize, Error> {
    let mut model =
        Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
    fuse_in(&mut model, degrees, through, rule, first_first).map(|solids| solids.len())
}

fn fuse_in(
    model: &mut Model,
    degrees: f64,
    through: [f64; 3],
    rule: Rule,
    first_first: bool,
) -> Result<Vec<SolidKey>, Error> {
    let [first, second] = two_bars(model, degrees, through);
    let (arguments, tools) = if first_first {
        ([first], [second])
    } else {
        ([second], [first])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

/// Assert that `solids` are one solid for each of `faces`, each with that
/// many faces, that the model validates, and that together they measure
/// `volume` to within the mesh's own error. `what` is the right answer, which
/// is what every message carries.
fn answers(model: &Model, solids: &[SolidKey], volume: f64, faces: &[usize], what: &str) {
    let policy = Policy::certified();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let mut counted = Vec::with_capacity(solids.len());
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let view = model.solid(solid).expect("a solid the fuse made");
        counted.push(view.faces().count());
        let measure = view
            .measure(
                Deflection::of(0.001).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("the answer measures");
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert_eq!(counted, faces, "{what}: faces of each solid");
    assert!(
        (measured - volume).abs() <= error,
        "{what}: measured {measured} and the mesh may miss by {error}"
    );
}

fn two_bars(model: &mut Model, degrees: f64, through: [f64; 3]) -> [SolidKey; 2] {
    let onto_x = Isometry::quarter_turns(&Axis::y(), 1).expect("a turn about y");
    let first_at = Isometry::translation(rcad_geom::Vector::new(
        through[0] - 2.0,
        through[1],
        through[2],
    ))
    .expect("a finite move")
    .compose(&onto_x);
    let first = model.add_round_bar(0.25, 4.0, &first_at).expect("a bar");
    let turned = Isometry::rotation(&Axis::z(), f64::to_radians(degrees))
        .expect("a turn about z")
        .compose(&onto_x);
    let along = turned.apply_to_vector(rcad_geom::Vector::new(0.0, 0.0, 1.0));
    let (start, length) = if through[1] == 0.0 {
        (
            [
                through[0] - 16.0 * along.x(),
                through[1] - 16.0 * along.y(),
                through[2] - 16.0 * along.z(),
            ],
            32.0,
        )
    } else {
        (through, 2.0)
    };
    let second_at = Isometry::translation(rcad_geom::Vector::new(start[0], start[1], start[2]))
        .expect("a finite move")
        .compose(&turned);
    let second = model
        .add_round_bar(0.25, length, &second_at)
        .expect("a bar");
    [first, second]
}
