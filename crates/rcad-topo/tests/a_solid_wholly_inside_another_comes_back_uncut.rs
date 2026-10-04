//! A solid wholly inside another, fused all six ways, comes back as one of the
//! two or the pair of them, with no face cut.
//!
//! Four pairs, each inner solid strictly inside the outer one and touching
//! nothing of it:
//!
//! - a frustum about `z`, slope a half, between `z = 2` and `z = 6` (radius 1
//!   to 3), inside a bar of radius 4 from `z = 1` to `z = 7`;
//! - a pointed cone about `z`, slope a half, its apex at the origin and its
//!   base of radius 2 at `z = 4`, inside a bar of radius 3 from `z = −1` to
//!   `z = 5`;
//! - a bar of radius 3 from `z = −2` to `z = 2` inside a ball of radius 5 at the
//!   origin: its rims are `√(3² + 2²) = √13` out, against 5;
//! - a box from `(−1, −1, −1)` to `(1, 1, 1)` inside a ball of radius 3 at the
//!   origin.
//!
//! Union, intersection and difference, each both ways round. The union is the
//! outer solid, the intersection the inner one, the outer less the inner is the
//! outer with a hollow the shape of the inner, two shells, and the inner less
//! the outer is nothing. Each answer is held against its closed-form volume
//! within the bound `measure` publishes for its mesh, and its faces are counted
//! against the faces of the solids it should be: a face cut where the two
//! solids do not meet comes back as more faces than that.
//!
//! **The first three refused all six on `8566306e`**, each
//! `Geometry(WhereThreeSurfacesMeetNeedsTwoOfThemFlat)`, as Q362 found them on
//! `d0d669f0`. The two solids were linked, so every face of each was cut by
//! every surface of both (§48), and the arrangement asked where a plane square
//! to the axis and two surfaces turning about it meet. The proof that two
//! solids are apart (§400) had no bound for a face on a pipe or a cone, and the
//! box round a flat cap reaches farther from the axis at its corners than the
//! cap does: the box round the frustum's top cap is `√18` out where the bar's
//! wall is 4. `no_face_of_one_meets_the_other` in `fuse/cuts.rs` now bounds
//! such faces by a band round their axis, how far up it and how far out, and
//! all three are proved apart and answer. The box in the ball answered on the
//! base already, its faces flat and straight-sided and so boxed by the proof
//! as it stood.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

#[test]
fn a_frustum_inside_a_bar_fuses_all_six_ways_into_one_of_the_two() {
    every_fuse_of(APair::AFrustumInABar);
}

#[test]
fn a_pointed_cone_inside_a_bar_fuses_all_six_ways_into_one_of_the_two() {
    every_fuse_of(APair::APointedConeInABar);
}

#[test]
fn a_bar_inside_a_ball_fuses_all_six_ways_into_one_of_the_two() {
    every_fuse_of(APair::ABarInABall);
}

#[test]
fn a_box_inside_a_ball_fuses_all_six_ways_into_one_of_the_two() {
    every_fuse_of(APair::ABoxInABall);
}

#[derive(Clone, Copy, Debug)]
enum APair {
    AFrustumInABar,
    APointedConeInABar,
    ABarInABall,
    ABoxInABall,
}

/// What one solid of a pair is: its volume, and how many of its faces are flat
/// and how many curved.
struct What {
    volume: f64,
    faces: [usize; 2],
}

impl APair {
    /// The outer solid and the inner one.
    fn built(self, model: &mut Model) -> [SolidKey; 2] {
        match self {
            Self::AFrustumInABar => {
                let bar = model
                    .add_round_bar(4.0, 6.0, &at(0.0, 0.0, 1.0))
                    .expect("a bar of radius four");
                [bar, a_frustum(model, 0.5, 2.0, 6.0)]
            }
            Self::APointedConeInABar => {
                let bar = model
                    .add_round_bar(3.0, 6.0, &at(0.0, 0.0, -1.0))
                    .expect("a bar of radius three");
                [bar, a_pointed_cone(model, 0.5, 4.0)]
            }
            Self::ABarInABall => {
                let ball = model
                    .add_ball(5.0, &at(0.0, 0.0, 0.0))
                    .expect("a ball of radius five");
                let bar = model
                    .add_round_bar(3.0, 4.0, &at(0.0, 0.0, -2.0))
                    .expect("a bar of radius three");
                [ball, bar]
            }
            Self::ABoxInABall => {
                let ball = model
                    .add_ball(3.0, &at(0.0, 0.0, 0.0))
                    .expect("a ball of radius three");
                let cube = model
                    .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a cube of side two");
                model
                    .place_solid(cube, &at(-1.0, -1.0, -1.0))
                    .expect("this model made the key");
                [ball, cube]
            }
        }
    }

    /// The outer solid and the inner one, in closed form.
    fn what(self) -> [What; 2] {
        let a_bar = |radius: f64, height: f64| What {
            volume: PI * radius * radius * height,
            faces: [2, 1],
        };
        let a_ball = |radius: f64| What {
            volume: 4.0 * PI * radius * radius * radius / 3.0,
            faces: [0, 1],
        };
        match self {
            // `πh(R² + Rr + r²)/3` with `h = 4`, `R = 3`, `r = 1`.
            Self::AFrustumInABar => [
                a_bar(4.0, 6.0),
                What {
                    volume: PI * 4.0 * (9.0 + 3.0 + 1.0) / 3.0,
                    faces: [2, 1],
                },
            ],
            // `πr²h/3` with `r = 2`, `h = 4`.
            Self::APointedConeInABar => [
                a_bar(3.0, 6.0),
                What {
                    volume: PI * 4.0 * 4.0 / 3.0,
                    faces: [1, 1],
                },
            ],
            Self::ABarInABall => [a_ball(5.0), a_bar(3.0, 4.0)],
            Self::ABoxInABall => [
                a_ball(3.0),
                What {
                    volume: 8.0,
                    faces: [6, 0],
                },
            ],
        }
    }
}

fn every_fuse_of(pair: APair) {
    let [outer, inner] = pair.what();
    for outer_first in [true, false] {
        let asked = format!("{pair:?}, the union, the outer first {outer_first}");
        let (model, answer) = fused(pair, Rule::Union, outer_first);
        let [solid] = answer[..] else {
            panic!("{asked} is one solid, not {answer:?}");
        };
        assert_is(&model, solid, outer.volume, [outer.faces], &asked);

        let asked = format!("{pair:?}, the intersection, the outer first {outer_first}");
        let (model, answer) = fused(pair, Rule::Intersection, outer_first);
        let [solid] = answer[..] else {
            panic!("{asked} is one solid, not {answer:?}");
        };
        assert_is(&model, solid, inner.volume, [inner.faces], &asked);
    }

    let asked = format!("{pair:?}, the outer less the inner");
    let (model, answer) = fused(pair, Rule::Difference, true);
    let [solid] = answer[..] else {
        panic!("{asked} is one solid, not {answer:?}");
    };
    assert_is(
        &model,
        solid,
        outer.volume - inner.volume,
        [outer.faces, inner.faces],
        &asked,
    );

    let (_, answer) = fused(pair, Rule::Difference, false);
    assert!(
        answer.is_empty(),
        "{pair:?}, the inner less the outer, is nothing, not {answer:?}"
    );
}

/// The pair fused by `rule`, the outer solid handed over first or second.
fn fused(pair: APair, rule: Rule, outer_first: bool) -> (Model, Vec<SolidKey>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [outer, inner] = pair.built(&mut model);
    let [first, second] = if outer_first {
        [outer, inner]
    } else {
        [inner, outer]
    };
    let answer = model
        .fuse(
            &[first],
            &[second],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| {
            panic!("{pair:?} under {rule:?}, the outer first {outer_first}, is refused: {why:?}")
        })
        .solids()
        .to_vec();
    (model, answer)
}

/// The solid validates, has one shell for each solid in `shells`, and as many
/// flat and curved faces as they have between them, and holds `volume` within
/// the bound its mesh publishes.
fn assert_is(
    model: &Model,
    solid: SolidKey,
    volume: f64,
    shells: impl AsRef<[[usize; 2]]>,
    asked: &str,
) {
    let shells = shells.as_ref();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked} left a model that does not validate: {why}"));
    let view = model.solid(solid).expect("this model made the key");
    assert_eq!(
        view.shells().count(),
        shells.len(),
        "{asked}: how many shells"
    );
    let (flat, curved): (Vec<_>, Vec<_>) = view
        .faces()
        .partition(|face| face.surface().as_plane().is_some());
    let faces = shells.iter().fold([0, 0], |[flat, curved], shell| {
        [flat + shell[0], curved + shell[1]]
    });
    assert_eq!(
        [flat.len(), curved.len()],
        faces,
        "{asked}: how many flat faces and how many curved, where one more is a face cut \
         where the two solids do not meet"
    );
    let measured = view
        .measure(
            Deflection::of(0.01).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{asked} does not measure: {why}"));
    println!(
        "{asked}: {} give or take {} against {volume}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{asked} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
