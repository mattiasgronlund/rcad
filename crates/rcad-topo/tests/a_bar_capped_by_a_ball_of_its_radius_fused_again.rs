//! A round bar of radius 2.5 up `z` from 0 to 5 united with a ball of radius
//! 2.5 at `(0, 0, 5)`: a bar with a rounded end, `125π/3`, the shape a
//! designer gets by rounding a pin's end. The ball touches the bar's skin
//! round the rim `z = 5` and never crosses it.
//!
//! The union answers, and every fuse of what it gives was refused
//! `ACrossingThisCannotBeOrderedAlong` under intersection and difference,
//! with a box far off too (Q2999). The rim is one closed side between the
//! bar's skin and the ball's, and it was baked against the ball: `rcad-geom`'s
//! `where_a_closed_crossing_starts` marched the touch circle and refused it at
//! the tangency, `CannotFollowACrossingWhereItsSurfacesAreTangent` at
//! `SettlingAPointOntoBothSurfaces`. The union names the rim's plane in its
//! run, and the rim is baked against that plane now, where the two faces
//! touch round it (`fuse/bake.rs`'s `the_surface_a_side_is_baked_against`).
//! A wall across the rim then cuts an arc out of it that the pipe's piece
//! names by the pipe and the plane and the ball's by the ball and the plane;
//! `fuse/assemble.rs`'s `the_name_each_arc_of_a_conic_goes_by` names the two
//! as one edge, where they were two with one face each and the walls were
//! refused `ResultIsNotManifold { count: 1 }` (Q3000).

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// The radius of the bar and of the ball on its end.
const RADIUS: f64 = 2.5;

/// The bar, `π r² · 5`, and the half of the ball above the rim, `2π r³/3`.
const THE_WHOLE: f64 = 125.0 * PI / 3.0;

#[test]
fn a_bar_capped_by_a_ball_of_its_radius_measures_its_closed_form() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let capped = a_bar_capped_by_a_ball(&mut model);
    let measured = measured(&model, capped);
    assert!(
        (measured.volume - THE_WHOLE).abs() <= measured.error,
        "{measured:?}"
    );
}

#[test]
fn a_bar_capped_by_a_ball_of_its_radius_is_left_whole_by_a_box_far_off() {
    let wanted = [
        (Rule::Intersection, Gives::Nothing),
        (Rule::Difference, Gives::Holds(THE_WHOLE)),
    ];
    let gives = what_each_rule_gives([10.0, 10.0, 10.0], [11.0, 11.0, 11.0], &wanted);
    assert!(all_as_wanted(&gives, &wanted), "{gives:?}");
}

/// The wall `x = 0.5` crosses the rim, and the box behind it reaches below
/// the rim to `z = 3`. The bar's part of what it holds is the slice of the
/// disc past `x = 0.5`, `r² acos(0.5/r) − 0.5 √(r² − 0.25)`, two high; the
/// ball's is half the cap two high past that wall, `π · 2² (3r − 2)/6`.
#[test]
fn a_bar_capped_by_a_ball_of_its_radius_is_cut_by_a_wall_across_its_rim() {
    let held =
        2.0 * (RADIUS * RADIUS * (0.5 / RADIUS).acos() - 0.5 * 6.0_f64.sqrt()) + 11.0 * PI / 3.0;
    let wanted = [
        (Rule::Intersection, Gives::Holds(held)),
        (Rule::Difference, Gives::Holds(THE_WHOLE - held)),
    ];
    let gives = what_each_rule_gives([0.5, -4.0, 3.0], [4.0, 4.0, 8.0], &wanted);
    assert!(all_as_wanted(&gives, &wanted), "{gives:?}");
}

/// As the wall `x = 0.5`, at `y = 0.2`: the cap past it is `2.3` high.
#[test]
fn a_bar_capped_by_a_ball_of_its_radius_is_cut_by_a_wall_off_its_axis_across_its_rim() {
    let held = 2.0 * (RADIUS * RADIUS * (0.2 / RADIUS).acos() - 0.2 * 6.21_f64.sqrt())
        + PI * 2.3 * 2.3 * (3.0 * RADIUS - 2.3) / 6.0;
    let wanted = [
        (Rule::Intersection, Gives::Holds(held)),
        (Rule::Difference, Gives::Holds(THE_WHOLE - held)),
    ];
    let gives = what_each_rule_gives([-4.0, 0.2, 3.0], [4.0, 4.0, 8.0], &wanted);
    assert!(all_as_wanted(&gives, &wanted), "{gives:?}");
}

/// A slab from `z = 1` to `z = 2` across the bar, under the rim: it holds a
/// disc one high, and leaves the bar below it and the rounded end above.
#[test]
fn a_bar_capped_by_a_ball_of_its_radius_is_cut_by_a_slab_across_its_bar() {
    let disc = PI * RADIUS * RADIUS;
    let wanted = [
        (Rule::Intersection, Gives::Holds(disc)),
        (
            Rule::Difference,
            Gives::HoldsTwo(disc, THE_WHOLE - 2.0 * disc),
        ),
    ];
    let gives = what_each_rule_gives([-4.0, -4.0, 1.0], [4.0, 4.0, 2.0], &wanted);
    assert!(all_as_wanted(&gives, &wanted), "{gives:?}");
}

fn all_as_wanted(gives: &[Result<Vec<Measured>, Error>], wanted: &[(Rule, Gives)]) -> bool {
    gives
        .iter()
        .zip(wanted)
        .all(|(answer, (_, wanted))| wanted.is(answer))
}

/// What the bar with the ball on its end fused with the block from `low` to
/// `high` comes back with under each rule of `wanted`, in its order.
fn what_each_rule_gives(
    low: [f64; 3],
    high: [f64; 3],
    wanted: &[(Rule, Gives)],
) -> Vec<Result<Vec<Measured>, Error>> {
    wanted
        .iter()
        .map(|&(rule, _)| {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let capped = a_bar_capped_by_a_ball(&mut model);
            let block = block(&mut model, low, high);
            what_one_fuse_gives(&mut model, &[capped], &[block], rule)
        })
        .collect()
}

/// The bar of radius 2.5 up `z` from 0 to 5 united with the ball of radius
/// 2.5 at `(0, 0, 5)`.
fn a_bar_capped_by_a_ball(model: &mut Model) -> SolidKey {
    let bar = model
        .add_round_bar(RADIUS, 5.0, &Isometry::identity())
        .expect("a bar");
    let ball = model.add_ball(RADIUS, &at(0.0, 0.0, 5.0)).expect("a ball");
    let united = model
        .fuse(
            &[bar],
            &[ball],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar and a ball on its end unite")
        .solids()
        .to_vec();
    let [capped] = united[..] else {
        panic!("one solid, got {united:?}");
    };
    capped
}

/// What one fuse is pinned to give.
#[derive(Clone, Copy, Debug)]
enum Gives {
    /// One solid of this volume.
    Holds(f64),
    /// Two solids of these volumes, the smaller first.
    HoldsTwo(f64, f64),
    /// No solid at all.
    Nothing,
}

impl Gives {
    fn is(&self, answer: &Result<Vec<Measured>, Error>) -> bool {
        let near = |solid: Measured, wanted: f64| (solid.volume - wanted).abs() <= solid.error;
        match (self, answer) {
            (Gives::Holds(wanted), Ok(volumes)) => {
                matches!(volumes[..], [one] if near(one, *wanted))
            }
            (Gives::HoldsTwo(smaller, larger), Ok(volumes)) => {
                matches!(volumes[..], [one, other] if near(one, *smaller) && near(other, *larger))
            }
            (Gives::Nothing, Ok(volumes)) => volumes.is_empty(),
            _ => false,
        }
    }
}

/// One solid of an answer.
#[derive(Clone, Copy, Debug)]
struct Measured {
    volume: f64,
    /// How far off [`Measured::volume`] may be.
    error: f64,
}

/// What `one` fused with `other` under `rule` comes back with: `Ok` with each
/// solid, the smallest first, once the model validates, or the refusal.
fn what_one_fuse_gives(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    rule: Rule,
) -> Result<Vec<Measured>, Error> {
    let answer = model.fuse(
        one,
        other,
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )?;
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("an answer validates");
    let mut solids: Vec<Measured> = answer
        .solids()
        .iter()
        .map(|&solid| measured(model, solid))
        .collect();
    solids.sort_by(|one, other| one.volume.total_cmp(&other.volume));
    Ok(solids)
}

fn measured(model: &Model, solid: SolidKey) -> Measured {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid measures");
    Measured {
        volume: measured.volume(),
        error: measured.volume_error(),
    }
}

fn block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, high[2] - low[2], &at(0.0, 0.0, low[2]))
        .expect("a block")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
