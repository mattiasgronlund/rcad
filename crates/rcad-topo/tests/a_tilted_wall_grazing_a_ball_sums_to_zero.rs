//! Two slanted walls of a prism meet in the `z` axis, and a ball grazes that
//! edge: going round either wall's circle from where it starts, the two places
//! the edge cuts the ball sweep exactly zero.
//!
//! # The shape
//!
//! A prism on the triangle `(0, 0)`, `(4, 3)`, `(−3, 3)`, 32 tall. Its walls
//! `3x = 4y` and `x = −y` both hold the `z` axis, and their stored normals,
//! `(0.6, −0.8, 0)` and `(−0.7071067811865476, −0.7071067811865476, 0)`, have
//! two parts that are not zero. Both offsets are exactly zero, so every point
//! `(0, 0, z)` is on both walls bit for bit.
//!
//! A ball of radius `1 + 2⁻⁵¹` at `(1, 2⁻²⁵, 16)`. The `z` axis misses its
//! centre by `√(1 + 2⁻⁵⁰)`, so it cuts the ball at `z = 16 ± 2⁻⁵¹`, which are
//! not `f64`s: `(2m)² + (2m²)² + 1 = (2m² + 1)²` with `m = 2²⁵`, scaled by
//! `2⁻⁵¹`, as in `docs/decisions.md` §244.2.
//!
//! # Why the sweep is zero
//!
//! Both walls' circles start at exactly `(0, 0, 16)`, on the axis between the
//! two places and `2⁻¹⁰³` inside the ball, so off the circle by a rounding.
//! The two places come back as boxes whose `x` and `y` are exactly zero and
//! whose `z` sides overlap. Every point of the three is on one line along `z`,
//! so the sweep `n · ((one − from) × (other − from))` multiplies each `z` by
//! an exact zero and comes out `[0, 0]`: proved zero for two different points.
//! Round either circle from a start that is not on it, "which comes first" has
//! no answer the boxes can give.
//!
//! # What this pins
//!
//! Until §244.5, `round_a_closed_crossing` answered `Zero` for that pair
//! because the boxes overlap, and the two places were folded into one node
//! without a policy being asked. Every certified fuse then refused on the turn
//! at the folded node. Now the pair goes to the policy: a certified fuse
//! refuses on the order of the two places, and a tolerant one merges them and
//! its intersection is the cap past the first wall. The union and the
//! difference under a tolerant policy do not mesh the ball's face, which is
//! another row's to mend (§244.5), and are not pinned here.

use rcad_geom::{
    Deflection, EdgeRun, Isometry, MeetOfThree, NamedRun, Point, Scale, Surface, Vector,
    meet_of_three_surfaces, where_a_closed_crossing_starts, which_comes_first_round_a_crossing,
};
use rcad_predicates::{
    Decision, Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog, Record,
};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The geometry the fuse is handed: both walls' circles start on the axis,
/// the two places are apart, and round either circle they sweep exactly zero.
#[test]
fn round_either_walls_circle_the_two_places_sweep_exactly_zero() {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
    let ball = a_ball_grazing_the_z_axis(&mut model);
    let walls = the_walls_through_the_z_axis(&model, prism);
    let skin = the_only_surface_of(&model, ball);

    let places = where_the_edge_cuts_the_ball(walls, skin);
    assert_ne!(places[0], places[1], "{places:?}");
    for place in places {
        let [x, y, z] = place.enclosure();
        assert_eq!([x.lo(), x.hi(), y.lo(), y.hi()], [0.0; 4], "{place:?}");
        assert!(z.lo() < 16.0 && 16.0 < z.hi(), "{place:?}");
    }

    for wall in walls {
        let Surface::Plane(flat) = wall else {
            panic!("a wall is a plane: {wall:?}");
        };
        let [nx, ny, nz] = flat.normal().components();
        assert!(nx != 0.0 && ny != 0.0 && nz == 0.0, "{flat:?}");
        assert_eq!(flat.offset(), 0.0, "{flat:?}");

        let run = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed crossing is a whole run");
        let from = where_a_closed_crossing_starts(&run, [&wall, &skin], &mut ProvenanceLog::new())
            .expect("a wall cuts the ball in a circle");
        assert_eq!(from.coordinates(), [0.0, 0.0, 16.0], "{wall:?}");

        let swept = which_comes_first_round_a_crossing(
            [&wall, &skin],
            PredicatePoint::from(from.coordinates()),
            places[0],
            places[1],
        )
        .expect("a plane and a ball cross in a circle");
        assert_eq!(swept, Decision::Zero, "{wall:?}");
    }
}

/// The regression: every certified fuse refuses on the order of the two
/// places, by name, with nothing else in its log.
///
/// With the ball first, the ball's face is split first, and the order asked is
/// round a wall's circle: the arm §244.5 changed, as wide as the wider box.
/// Before §244.5 the two were folded into one node there, and every fuse went
/// on to refuse on `orient3d`, the turn between the two walls' circles at that
/// node. With the prism first, a wall's face is split first, and the order
/// asked is along the edge the two walls share, a length, which the policy
/// was always asked about.
#[test]
fn a_certified_fuse_refuses_on_the_order_of_the_two_places() {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
    let ball = a_ball_grazing_the_z_axis(&mut model);
    let places = where_the_edge_cuts_the_ball(
        the_walls_through_the_z_axis(&model, prism),
        the_only_surface_of(&model, ball),
    );
    let widest = places[0].uncertainty().max(places[1].uncertainty());
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            let refused = match fuse(rule, ball_first, &Policy::certified()) {
                (_, Err(Error::Undecidable { records }), _) => records.records().to_vec(),
                (_, other, _) => panic!("{rule:?}, ball first {ball_first}: {other:?}"),
            };
            assert_eq!(
                refused.len(),
                1,
                "{rule:?}, ball first {ball_first}: {refused:#?}"
            );
            let refused = &refused[0];
            assert!(
                is_the_order_of(refused, places),
                "{rule:?}, ball first {ball_first}: {refused:#?}"
            );
            if ball_first {
                assert_eq!(refused.measures(), Measures::Volume, "{rule:?}");
                assert_eq!(refused.enclosure(), widest, "{rule:?}");
            } else {
                assert_eq!(refused.measures(), Measures::Length, "{rule:?}");
            }
        }
    }
}

/// A tolerant policy merges the two places, which are `2⁻⁵⁰` apart, and the
/// intersection is the cap of the ball past the first wall, within what the
/// measurement can promise.
///
/// The second wall takes nothing off that cap: the ball reaches past it only
/// in a sliver about `2⁻¹⁰³` wide round the axis.
#[test]
fn a_tolerant_intersection_is_the_cap_past_the_first_wall() {
    let radius = 1.0 + f64::powi(2.0, -51);
    let past = radius - (3.0 - 4.0 * f64::powi(2.0, -25)) / 5.0;
    let cap = core::f64::consts::PI * past * past * (3.0 * radius - past) / 3.0;
    for ball_first in [true, false] {
        let (model, fused, _) = fuse(Rule::Intersection, ball_first, &Policy::tolerant(25.0));
        let fused = fused.unwrap_or_else(|why| panic!("ball first {ball_first}: {why:?}"));
        let [piece] = fused.solids() else {
            panic!("ball first {ball_first}: {:?}", fused.solids());
        };
        let measured = model
            .solid(*piece)
            .expect("the fuse made it")
            .measure(
                Deflection::of(0.001).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a cap measures");
        assert!(
            (measured.volume() - cap).abs() <= measured.volume_error(),
            "ball first {ball_first}: {} ± {} against {cap}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

fn fuse(
    rule: Rule,
    ball_first: bool,
    policy: &Policy,
) -> (Model, rcad_topo::Result<Fused>, ProvenanceLog) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
    let ball = a_ball_grazing_the_z_axis(&mut model);
    let (one, other) = if ball_first {
        (ball, prism)
    } else {
        (prism, ball)
    };
    let mut log = ProvenanceLog::new();
    let fused = model.fuse(&[one], &[other], rule, policy, &mut log);
    (model, fused, log)
}

/// Whether a record is `order_along`'s about the two places, in either order.
fn is_the_order_of(record: &Record, places: [PredicatePoint; 2]) -> bool {
    let inputs = record.inputs();
    record.predicate() == ALONG_A_CROSSING
        && record.level() == Level::Interval
        && (inputs == places || inputs == [places[1], places[0]])
}

fn where_the_edge_cuts_the_ball(walls: [Surface; 2], skin: Surface) -> [PredicatePoint; 2] {
    let mut log = ProvenanceLog::new();
    let meet = meet_of_three_surfaces(
        [&walls[0], &walls[1], &skin],
        &Policy::certified(),
        &mut log,
    )
    .expect("a certified answer");
    assert!(log.is_empty(), "{log:?}");
    let MeetOfThree::At(places) = meet else {
        panic!("the z axis cuts the ball: {meet:?}");
    };
    let [one, other] = places[..] else {
        panic!("a line that nearly touches a ball cuts it twice: {places:?}");
    };
    [one, other]
}

fn a_prism_with_two_walls_through_the_z_axis(model: &mut Model) -> SolidKey {
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle");
    model
        .add_extrusion(&triangle, 32.0, &Isometry::identity())
        .expect("a positive height")
}

fn a_ball_grazing_the_z_axis(model: &mut Model) -> SolidKey {
    model
        .add_ball(
            1.0 + f64::powi(2.0, -51),
            &Isometry::translation(Vector::new(1.0, f64::powi(2.0, -25), 16.0))
                .expect("a finite shift"),
        )
        .expect("a positive radius")
}

/// The two walls whose planes hold the `z` axis, as the fuse is handed them.
fn the_walls_through_the_z_axis(model: &Model, prism: SolidKey) -> [Surface; 2] {
    let on_the_axis = |surface: &Surface| {
        let Surface::Plane(flat) = surface else {
            return false;
        };
        [0.0, 32.0].into_iter().all(|z| {
            flat.side_of_point(PredicatePoint::from(Point::new(0.0, 0.0, z).coordinates()))
                == Decision::Zero
        })
    };
    let walls: Vec<Surface> = model
        .solid(prism)
        .expect("the prism")
        .faces()
        .map(|face| face.surface())
        .filter(on_the_axis)
        .collect();
    let [one, other] = walls[..] else {
        panic!("a triangle with a corner at the origin has two walls there: {walls:?}");
    };
    [one, other]
}

fn the_only_surface_of(model: &Model, solid: SolidKey) -> Surface {
    let mut faces = model.solid(solid).expect("the ball").faces();
    let surface = faces.next().expect("a face").surface();
    assert!(faces.next().is_none(), "a ball is one face");
    surface
}
