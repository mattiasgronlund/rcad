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
//! reads which side of its chord an arc bulges the way the answer names the
//! arc, by `Edge::the_way_across_the_chord_into_the_arc`.
//!
//! **It used to read it off one rounded point**, the arc's corner plus the
//! direction the arc leaves that corner in, by the exact winding of that point
//! against the chord between the two stored corners. That weighs the tangent
//! against a chord whose ends are rounded, and on a chord a hair long the
//! rounding can lean the chord further than the arc turns off it. Q600 found
//! the point on the right side in everything the fuse built, because the fuse
//! then refused every sliver whose chord was shorter than a few millionths,
//! mostly for no point found inside the piece of the bar's cap between the two
//! circles. Once those pieces were placed (Q2551), the bar out past the wall
//! at 180 degrees and a hair of `1e-15` was built with its outline's two arcs
//! on a chord `1.41e-7` long, from `(-6.999999999999998, 7.05e-8)` to
//! `(-7, -7.05e-8)`. Its ends are two roundings apart in `x`, so the chord
//! leans `1.26e-8` off `y`, where the tube's circle turns only `1.01e-8` off
//! the chord over half its length. The tube's arc was read as bulging away
//! from the centre, neither ring of the cap held the other, and
//! `Assembly::rings_round` panicked; so it did for the bar at 180 and 300
//! degrees and a hair of `3.2e-16` (Q2589). The answer's own reading sees that
//! the rounding of the ends chose the side, and reads which way along the chord
//! the arc sets off instead.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

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

/// The bar out past the wall at the three places where the tube's arc was read
/// as bulging the wrong way and the fuse panicked: 180 degrees round and a
/// hair of `1e-15`, and 180 and 300 degrees and `10^-15.5`. Each union holds
/// both solids, on a chord under `1.5e-7`, and at 180 degrees that chord's two
/// ends are not on one `x`, though the bar's centre is on the `x` axis, so the
/// rounding leans it.
#[test]
fn a_bar_out_past_a_tubes_wall_on_a_chord_the_rounding_leans_holds_both() {
    for (degrees, hair) in [
        (180.0, 1e-15),
        (180.0, 10f64.powf(-15.5)),
        (300.0, 10f64.powf(-15.5)),
    ] {
        let angle = f64::to_radians(degrees);
        let centre = [5.0 * angle.cos(), 5.0 * angle.sin()];
        let radius = 7.0 - centre[0].hypot(centre[1]) + hair;
        let (mut model, tube, bar) = a_tube_and_a_bar(centre, radius);
        let answer = the_union(&mut model, tube, bar)
            .unwrap_or_else(|why| panic!("{degrees} degrees, {hair:e}: {why}"));
        assert_holds(&model, answer, centre, radius);
        let (chord, [start, end]) = the_shortest_chord(&model, answer)
            .unwrap_or_else(|| panic!("{degrees} degrees, {hair:e}: no arc has two ends"));
        assert!(
            chord < 1.5e-7,
            "{degrees} degrees, {hair:e}: the shortest chord is {chord:e} long"
        );
        if degrees == 180.0 {
            assert_ne!(
                start[0], end[0],
                "{hair:e}: the chord from {start:?} to {end:?} does not lean"
            );
        }
    }
}

/// Each bar, stood at three angles round the tube, walked in from a hundredth
/// across the wall to a ten-quadrillionth: every union the fuse builds holds
/// both solids, on chords down to `2.1e-8`, the bar into the hole at 300
/// degrees and a ten-quadrillionth. The one it refuses is the bar out past the
/// wall at 180 degrees and `1e-16`, `DynamicRangeTooWide`: `2 + 1e-16` is `2`,
/// and the bar's circle all but touches the tube's.
///
/// The last two half-decades into the hole are past what `f64` can place too:
/// `3.5 + 1e-16` is `3.5`, and the bar touches the hole rather than crossing
/// it.
#[test]
fn every_sliver_the_union_builds_holds_both_down_to_where_the_fuse_refuses() {
    let hairs: Vec<f64> = (4..=32).map(|k| 10f64.powf(-f64::from(k) / 2.0)).collect();
    let mut refused = Vec::new();
    let mut shortest = f64::INFINITY;
    for (what, from_the_axis, radius) in [
        (
            "into the hole",
            4.5,
            (|apart: f64, hair: f64| apart - 1.0 + hair) as fn(f64, f64) -> f64,
        ),
        ("out past the wall", 5.0, |apart, hair| 7.0 - apart + hair),
    ] {
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
                        if let Some((chord, _)) = the_shortest_chord(&model, answer) {
                            shortest = shortest.min(chord);
                        }
                    }
                    Err(why) => refused.push((
                        what,
                        degrees,
                        hair,
                        matches!(why, Error::Invalid(Invalid::DynamicRangeTooWide { .. })),
                    )),
                }
            }
        }
    }
    assert_eq!(
        refused,
        [("out past the wall", 180.0, 1e-16, true)],
        "which bars the fuse refuses, and whether for the range"
    );
    assert!(
        shortest < 3e-8,
        "the shortest chord any union here was built on is {shortest:e} long"
    );
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

/// How far apart the two ends of the shortest edge of `solid` with two ends
/// are, and where those ends are; `None` where every edge closes on itself.
fn the_shortest_chord(model: &Model, solid: SolidKey) -> Option<(f64, [[f64; 3]; 2])> {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .flat_map(|face| face.wires().collect::<Vec<_>>())
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .map(|coedge| coedge.edge())
        .map(|edge| {
            (
                edge.distance_between_its_ends(),
                [edge.start(), edge.end()].map(|end| end.point().coordinates()),
            )
        })
        .filter(|&(chord, _)| chord > 0.0)
        .min_by(|one, other| one.0.total_cmp(&other.0))
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
