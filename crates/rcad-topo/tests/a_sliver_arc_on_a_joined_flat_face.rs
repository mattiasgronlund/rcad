//! A bar standing a hair across one of a tube's walls, united with it: a joined
//! flat face with an arc that bulges a hair off its chord.
//!
//! The tube is a bar of radius 7 and height 4 less a bar of radius 1 standing
//! through it, as in `a_joined_flat_face_whose_rings_enclose_no_area.rs`. Two
//! bars are stood beside it, flush with both caps:
//!
//! - **Into the hole**: radius `3.5 + ε`, `4.5` from the axis, so it bites a
//!   hair out of the hole. The joined cap's hole is then most of the hole's
//!   circle and a sliver of the bar's.
//! - **Out past the wall**: radius `2 + ε`, `5` from the axis, so it pokes a
//!   hair out of the tube and leaves the hole alone. The joined cap's outline is
//!   a sliver of the bar's circle and all but a hair of the tube's, and the
//!   outline's two arcs share a chord a hair long.
//!
//! Every ring of both caps encloses no area between its corners, so
//! `Assembly::holds_the_other_rings` places each one by how its arcs bulge. It
//! reads which side of its chord an arc bulges by rounding a point, the arc's
//! corner plus the direction it leaves that corner in, and asking the exact
//! winding of that point. On an arc a hair off its chord that direction lies a
//! hair off the chord too, and Q600 asked whether the rounding could put the
//! point on the wrong side.
//!
//! **It cannot, in anything the fuse builds.** On a pipe the direction is as
//! long as the radius, so the rounded point sits off the chord by half the
//! chord's length, and the fuse refuses every sliver whose chord is shorter
//! than a few millionths — mostly because no point can be found inside the
//! piece of the bar's cap between the two circles. The sweep below walks each
//! bar down in half-decades until the fuse refuses, and every union it builds
//! on the way holds both solids. A reading forced the wrong way was tried on
//! the same shapes by hand: it made the joined face panic, with two rings each
//! holding the other or neither holding anything, and never wrote a wrong
//! answer.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

/// The bar that bites a hair out of the hole, a hundred-billionth deep.
#[test]
fn a_bar_a_hair_into_a_tubes_hole_united_with_it_holds_both() {
    let (mut model, tube, bar) = a_tube_and_a_bar([-4.5, 0.0], 3.5 + 1e-11);
    let answer = the_union(&mut model, tube, bar).expect("a hair this deep is built");
    assert_holds(&model, answer, [-4.5, 0.0], 3.5 + 1e-11);
}

/// The bar that pokes a hair out past the wall, a hundred-billionth out.
#[test]
fn a_bar_a_hair_out_past_a_tubes_wall_united_with_it_holds_both() {
    let (mut model, tube, bar) = a_tube_and_a_bar([-5.0, 0.0], 2.0 + 1e-11);
    let answer = the_union(&mut model, tube, bar).expect("a hair this far out is built");
    assert_holds(&model, answer, [-5.0, 0.0], 2.0 + 1e-11);
}

/// Each bar, stood at three angles round the tube, walked in from a hundredth
/// across the wall to a ten-quadrillionth: every union the fuse builds holds
/// both solids, and below the thinnest one it builds it returns an error
/// rather than an answer. Which error is not pinned: a decision it could not
/// settle, mostly, and a crossing in two pieces no half-space can tell apart.
///
/// The last two half-decades are past what `f64` can place: `3.5 + 1e-16` is
/// `3.5`, and the bar touches the hole rather than crossing it.
#[test]
fn every_sliver_the_union_builds_holds_both_down_to_where_the_fuse_refuses() {
    let hairs: Vec<f64> = (4..=32).map(|k| 10f64.powf(-f64::from(k) / 2.0)).collect();
    for (what, from_the_axis, radius) in [
        (
            "into the hole",
            4.5,
            (|apart: f64, hair: f64| apart - 1.0 + hair) as fn(f64, f64) -> f64,
        ),
        ("out past the wall", 5.0, |apart, hair| 7.0 - apart + hair),
    ] {
        let mut built = Vec::new();
        let mut refused = Vec::new();
        for degrees in [180.0, 225.0, 300.0] {
            let angle = f64::to_radians(degrees);
            let centre = [from_the_axis * angle.cos(), from_the_axis * angle.sin()];
            let apart = centre[0].hypot(centre[1]);
            for &hair in &hairs {
                let radius = radius(apart, hair);
                let (mut model, tube, bar) = a_tube_and_a_bar(centre, radius);
                match the_union(&mut model, tube, bar) {
                    Ok(answer) => {
                        assert_holds(&model, answer, centre, radius);
                        built.push(hair);
                    }
                    Err(_) => refused.push(hair),
                }
            }
        }
        let thinnest = built
            .iter()
            .copied()
            .filter(|&hair| hair >= 1e-14)
            .fold(f64::INFINITY, f64::min);
        assert!(
            thinnest <= 1e-11,
            "the bar {what} was built no thinner than {thinnest:e} across"
        );
        assert!(
            refused.iter().any(|&hair| hair < thinnest),
            "the bar {what} was built at every hair from {thinnest:e} down, so this sweep no \
             longer shows where the fuse stops, and Q600's margin has to be measured again"
        );
    }
}

fn a_tube_and_a_bar(centre: [f64; 2], radius: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outer = model
        .add_round_bar(7.0, 4.0, &Isometry::identity())
        .expect("a bar of radius seven");
    let through = model
        .add_round_bar(1.0, 6.0, &at(0.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let tube = model
        .fuse(
            &[outer],
            &[through],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the tube refused: {why}"));
    let [tube] = tube.solids() else {
        panic!("the tube is one solid, not {:?}", tube.solids());
    };
    let bar = model
        .add_round_bar(radius, 4.0, &at(centre[0], centre[1], 0.0))
        .expect("a bar with a radius");
    (model, *tube, bar)
}

fn the_union(model: &mut Model, tube: SolidKey, bar: SolidKey) -> Result<SolidKey, Error> {
    let answer = model.fuse(
        &[tube],
        &[bar],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )?;
    let [solid] = answer.solids() else {
        panic!("the union is one solid, not {:?}", answer.solids());
    };
    Ok(*solid)
}

/// The union validates and holds the tube and the bar less what they share.
fn assert_holds(model: &Model, solid: SolidKey, centre: [f64; 2], radius: f64) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the union left a model that does not validate: {why}"));
    let apart = centre[0].hypot(centre[1]);
    let shared =
        what_two_disks_share(7.0, radius, apart) - what_two_disks_share(1.0, radius, apart);
    let volume = 4.0 * (48.0 * PI + PI * radius * radius - shared);
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the union does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the union of a bar of radius {radius} at {centre:?} holds {volume} and this one says {} \
         give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The area a disk of radius `one` and a disk of radius `other`, their centres
/// `apart`, have in common: two circular segments either side of the chord
/// between the two crossings, or the whole of the smaller disk, or nothing.
fn what_two_disks_share(one: f64, other: f64, apart: f64) -> f64 {
    if apart >= one + other {
        return 0.0;
    }
    if apart <= (one - other).abs() {
        return PI * one.min(other).powi(2);
    }
    let segment = |mine: f64, theirs: f64| {
        let half_angle = ((apart * apart + mine * mine - theirs * theirs) / (2.0 * apart * mine))
            .clamp(-1.0, 1.0)
            .acos();
        mine * mine * (half_angle - half_angle.sin() * half_angle.cos())
    };
    segment(one, other) + segment(other, one)
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
