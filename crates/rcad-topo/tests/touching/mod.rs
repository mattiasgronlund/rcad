//! Two solids that touch without crossing, one catalogue cell at a time, each
//! beside the same two solids moved one unit apart.
//!
//! `pairs/mod.rs` keeps a whole unit of room between every curved solid and
//! every wall on purpose, so that nothing it draws is tangent to anything. Real
//! parts are tangent everywhere — every fillet is a round face tangent to two
//! flat ones — so this module draws exactly what that one steps round.
//! `docs/decisions.md` §191.
//!
//! # Every number is a small whole number
//!
//! Every radius, height and position here is an integer, and every cone slope a
//! power of two, so the input is exact and a refusal is about the geometry and
//! not about rounding. Shrinking goes to the smallest drawable shape of each
//! cell, never to a coordinate.
//!
//! # A cell and its control differ by the touch and nothing else
//!
//! Every [`Shape`] is built twice: [`Apart::Touching`], and
//! [`Apart::ClearByOne`], where the one thing that made the two solids touch is
//! moved one unit so that they do not. Where the touching row refuses and the
//! control answers, the difference is the tangency.
//!
//! **An inner solid is always at least two thinner than the one it sits in**,
//! so moving it one unit off the wall never puts it on the outer solid's axis.
//! A coaxial control would be a second special position and not a control.

#![allow(dead_code)]

use proptest::prelude::*;
use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, SolidKey};

#[path = "../cones/mod.rs"]
mod cones;
#[path = "../rings/mod.rs"]
mod rings;

/// Whether the two solids of a drawn shape touch, or stand one unit off it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Apart {
    Touching,
    ClearByOne,
}

/// Which way from the first solid's axis the second one stands.
///
/// A bar's seam runs up its wall at `+x` and a ball's seam is the meridian half
/// at `+x`, so `PlusX` puts the touch **on the first solid's seam**, `MinusX`
/// puts it on the second solid's seam, and `PlusY` puts it on neither.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Towards {
    PlusX,
    MinusX,
    PlusY,
}

impl Towards {
    fn by(self, distance: i32) -> [i32; 2] {
        match self {
            Towards::PlusX => [distance, 0],
            Towards::MinusX => [-distance, 0],
            Towards::PlusY => [0, distance],
        }
    }
}

/// What the insides of the two solids do when they touch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Insides {
    /// They share no volume: `A ∩ B` is nothing at all.
    Apart,
    /// The second is wholly inside the first: `B \ A` is nothing at all.
    SecondInsideFirst,
    /// They share volume, so neither of those holds.
    Overlap,
}

/// One cell of the catalogue, with its sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A cube and a second block standing on its top face: two flat faces
    /// flush, the flat reference for every coincident-face cell.
    ABlockOnABlock { side: u32, width: u32, at: [u32; 2] },
    /// A bar standing beside a cube's wall, touching it along one ruling.
    /// `on_its_seam` stands the bar at `−x` so the touch is on the bar's seam.
    ABarBesideAWall {
        side: u32,
        radius: u32,
        along: u32,
        on_its_seam: bool,
    },
    /// A bar standing in the inside corner of an L, tangent to both walls: the
    /// shape of a fillet.
    ABarInAnInsideCorner { arm: u32, thick: u32, radius: u32 },
    /// A bar standing on a cube's top face, its cap flush on it.
    ABarStandingOnABlock {
        side: u32,
        radius: u32,
        at: [u32; 2],
    },
    /// A ball resting on a cube's top face, at its south pole.
    ABallOnABlock {
        side: u32,
        radius: u32,
        at: [u32; 2],
    },
    /// A ball against a cube's side wall. `on_its_seam` puts it at `−x` so
    /// the touch is on the ball's seam.
    ABallAgainstAWall {
        side: u32,
        radius: u32,
        at: [u32; 2],
        on_its_seam: bool,
    },
    /// Two upright bars side by side, touching along a line.
    TwoBarsTouching {
        first: u32,
        second: u32,
        towards: Towards,
    },
    /// A thinner bar inside a thicker one, touching its wall along a line.
    ABarInsideABar {
        outer: u32,
        inner: u32,
        towards: Towards,
    },
    /// Two balls touching at one point.
    TwoBallsTouching {
        first: u32,
        second: u32,
        towards: Towards,
    },
    /// A ball inside a larger ball, touching it once.
    ABallInsideABall {
        outer: u32,
        inner: u32,
        towards: Towards,
    },
    /// A ball on a bar of the same radius, centred on the bar's top: the KiCad
    /// LED's dome (§124). The control widens the bar by one.
    ADomeOnAShaft { radius: u32, height: u32 },
    /// A ball against a bar's wall, touching it at one point.
    ABallAgainstABar {
        bar: u32,
        ball: u32,
        towards: Towards,
    },
    /// A ball inside a bar, touching its wall at one point.
    ABallInsideABar {
        bar: u32,
        ball: u32,
        towards: Towards,
    },
    /// A pointed cone standing on a cube's top face on its apex.
    AConesApexOnABlock { slope: Slope, at: [u32; 2] },
    /// A frustum whose narrow cap sits flush on a block's top face.
    AFrustumsCapOnABlock { slope: Slope, near: u32, tall: u32 },
    /// A quarter bend of pipe lying on a block, tangent to its top face along
    /// a quarter circle.
    ///
    /// The block's side walls stand clear of the **whole** torus and not only
    /// of the bend: a fuse crosses surfaces and not faces, and a wall that cuts
    /// the torus anywhere off its axis is a quartic, refused by name on the
    /// control as much as on the touch. That is what the first version of this
    /// cell measured, 100% refused in both rows.
    ABendLyingOnABlock { ring: u32, tube: u32 },
    /// A quarter bend with its `+x` cap flush against a block's wall, the wall
    /// in the plane `y = 0` through the torus's axis. The control slides the
    /// block along that plane until it no longer covers the cap.
    ABendsCapAgainstABlock { ring: u32, tube: u32 },
}

/// A cone's slope, always a power of two so a rim sits exactly on its cone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slope {
    Half,
    One,
    Two,
}

impl Slope {
    fn value(self) -> f64 {
        match self {
            Slope::Half => 0.5,
            Slope::One => 1.0,
            Slope::Two => 2.0,
        }
    }
}

/// A drawn shape, touching or one unit off it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawn {
    pub shape: Shape,
    pub apart: Apart,
}

/// Two solids in one model, and what their insides do.
pub struct Built {
    pub model: Model,
    pub first: SolidKey,
    pub second: SolidKey,
    pub insides: Insides,
}

impl Drawn {
    /// Build the first solid and then the second.
    ///
    /// # Panics
    ///
    /// If the kernel refuses to build either. Every shape here is a sound
    /// solid, so that is a finding and not a result.
    #[must_use]
    pub fn build(&self) -> Built {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let off = match self.apart {
            Apart::Touching => 0,
            Apart::ClearByOne => 1,
        };
        let (first, second, insides) = match self.shape {
            Shape::ABlockOnABlock { side, width, at } => {
                let side = to_i(side);
                let cube = a_block(&mut model, [0, 0, 0], [side, side, side]);
                let block = a_block(
                    &mut model,
                    [to_i(at[0]), to_i(at[1]), side + off],
                    [to_i(width), to_i(width), 2],
                );
                (cube, block, Insides::Apart)
            }
            Shape::ABarBesideAWall {
                side,
                radius,
                along,
                on_its_seam,
            } => {
                let (side, radius) = (to_i(side), to_i(radius));
                let cube = a_block(&mut model, [0, 0, 0], [side, side, side]);
                let x = if on_its_seam {
                    -radius - off
                } else {
                    side + radius + off
                };
                let bar = a_bar(&mut model, radius, side - 2, [x, to_i(along), 1]);
                (cube, bar, Insides::Apart)
            }
            Shape::ABarInAnInsideCorner { arm, thick, radius } => {
                let (arm, thick, radius) = (to_i(arm), to_i(thick), to_i(radius));
                let corner = an_l(&mut model, arm, thick, 4);
                let axis = thick + radius + off;
                let bar = a_bar(&mut model, radius, 2, [axis, axis, 1]);
                (corner, bar, Insides::Apart)
            }
            Shape::ABarStandingOnABlock { side, radius, at } => {
                let side = to_i(side);
                let cube = a_block(&mut model, [0, 0, 0], [side, side, side]);
                let bar = a_bar(
                    &mut model,
                    to_i(radius),
                    2,
                    [to_i(at[0]), to_i(at[1]), side + off],
                );
                (cube, bar, Insides::Apart)
            }
            Shape::ABallOnABlock { side, radius, at } => {
                let (side, radius) = (to_i(side), to_i(radius));
                let cube = a_block(&mut model, [0, 0, 0], [side, side, side]);
                let ball = a_ball(
                    &mut model,
                    radius,
                    [to_i(at[0]), to_i(at[1]), side + radius + off],
                );
                (cube, ball, Insides::Apart)
            }
            Shape::ABallAgainstAWall {
                side,
                radius,
                at,
                on_its_seam,
            } => {
                let (side, radius) = (to_i(side), to_i(radius));
                let cube = a_block(&mut model, [0, 0, 0], [side, side, side]);
                let x = if on_its_seam {
                    -radius - off
                } else {
                    side + radius + off
                };
                let ball = a_ball(&mut model, radius, [x, to_i(at[0]), to_i(at[1])]);
                (cube, ball, Insides::Apart)
            }
            Shape::TwoBarsTouching {
                first,
                second,
                towards,
            } => {
                let (first, second) = (to_i(first), to_i(second));
                let one = a_bar(&mut model, first, 4, [0, 0, 0]);
                let [x, y] = towards.by(first + second + off);
                let other = a_bar(&mut model, second, 2, [x, y, 1]);
                (one, other, Insides::Apart)
            }
            Shape::ABarInsideABar {
                outer,
                inner,
                towards,
            } => {
                let (outer, inner) = (to_i(outer), to_i(inner));
                let one = a_bar(&mut model, outer, 4, [0, 0, 0]);
                let [x, y] = towards.by(outer - inner - off);
                let other = a_bar(&mut model, inner, 2, [x, y, 1]);
                (one, other, Insides::SecondInsideFirst)
            }
            Shape::TwoBallsTouching {
                first,
                second,
                towards,
            } => {
                let (first, second) = (to_i(first), to_i(second));
                let one = a_ball(&mut model, first, [0, 0, 0]);
                let [x, y] = towards.by(first + second + off);
                let other = a_ball(&mut model, second, [x, y, 0]);
                (one, other, Insides::Apart)
            }
            Shape::ABallInsideABall {
                outer,
                inner,
                towards,
            } => {
                let (outer, inner) = (to_i(outer), to_i(inner));
                let one = a_ball(&mut model, outer, [0, 0, 0]);
                let [x, y] = towards.by(outer - inner - off);
                let other = a_ball(&mut model, inner, [x, y, 0]);
                (one, other, Insides::SecondInsideFirst)
            }
            Shape::ADomeOnAShaft { radius, height } => {
                let (radius, height) = (to_i(radius), to_i(height));
                let shaft = a_bar(&mut model, radius + off, height, [0, 0, 0]);
                let dome = a_ball(&mut model, radius, [0, 0, height]);
                (shaft, dome, Insides::Overlap)
            }
            Shape::ABallAgainstABar { bar, ball, towards } => {
                let (bar, ball) = (to_i(bar), to_i(ball));
                let shaft = a_bar(&mut model, bar, 2 * ball + 2, [0, 0, 0]);
                let [x, y] = towards.by(bar + ball + off);
                let round = a_ball(&mut model, ball, [x, y, ball + 1]);
                (shaft, round, Insides::Apart)
            }
            Shape::ABallInsideABar { bar, ball, towards } => {
                let (bar, ball) = (to_i(bar), to_i(ball));
                let shaft = a_bar(&mut model, bar, 2 * ball + 2, [0, 0, 0]);
                let [x, y] = towards.by(bar - ball - off);
                let round = a_ball(&mut model, ball, [x, y, ball + 1]);
                (shaft, round, Insides::SecondInsideFirst)
            }
            Shape::AConesApexOnABlock { slope, at } => {
                let cone = cones::a_pointed_cone(&mut model, slope.value(), 2.0);
                let block = a_block(
                    &mut model,
                    [-to_i(at[0]), -to_i(at[1]), -4 - off],
                    [4, 4, 4],
                );
                (block, cone, Insides::Apart)
            }
            Shape::AFrustumsCapOnABlock { slope, near, tall } => {
                let near_at = f64::from(near);
                let frustum = cones::a_frustum(
                    &mut model,
                    slope.value(),
                    near_at,
                    near_at + f64::from(tall),
                );
                let wide = 2 * to_i(near + tall) + 1;
                let block = a_block(
                    &mut model,
                    [-wide, -wide, to_i(near) - 2 - off],
                    [2 * wide, 2 * wide, 2],
                );
                (block, frustum, Insides::Apart)
            }
            Shape::ABendLyingOnABlock { ring, tube } => {
                let bend = rings::a_quarter_bend(&mut model, f64::from(ring), f64::from(tube));
                let (ring, tube) = (to_i(ring), to_i(tube));
                let reach = ring + tube + 1;
                let block = a_block(
                    &mut model,
                    [-reach, -reach, -tube - 2 - off],
                    [2 * reach, 2 * reach, 2],
                );
                (block, bend, Insides::Apart)
            }
            Shape::ABendsCapAgainstABlock { ring, tube } => {
                let bend = rings::a_quarter_bend(&mut model, f64::from(ring), f64::from(tube));
                let (ring, tube) = (to_i(ring), to_i(tube));
                let reach = ring + tube + 1;
                // Slid off along the cap's own plane rather than away from it:
                // every plane parallel to the cap and off the axis cuts the torus
                // in a quartic, which refuses for its own reason.
                let (from, across) = match self.apart {
                    Apart::Touching => (-reach, 2 * reach),
                    Apart::ClearByOne => (reach, 2),
                };
                let block = a_block(
                    &mut model,
                    [from, -reach - 1, -tube - 1],
                    [across, reach + 1, 2 * tube + 2],
                );
                (block, bend, Insides::Apart)
            }
        };
        Built {
            model,
            first,
            second,
            insides,
        }
    }
}

/// Both rows of one cell: every shape the cell draws, touching and one off it.
pub fn both_rows(shapes: impl Strategy<Value = Shape>) -> impl Strategy<Value = Drawn> {
    (
        shapes,
        prop_oneof![Just(Apart::Touching), Just(Apart::ClearByOne)],
    )
        .prop_map(|(shape, apart)| Drawn { shape, apart })
}

pub fn a_block_on_a_block() -> impl Strategy<Value = Shape> {
    (3u32..=5)
        .prop_flat_map(|side| (Just(side), 1..side - 1))
        .prop_flat_map(|(side, width)| {
            let room = 1..=side - width - 1;
            (Just(side), Just(width), [room.clone(), room])
        })
        .prop_map(|(side, width, at)| Shape::ABlockOnABlock { side, width, at })
}

pub fn a_bar_beside_a_wall() -> impl Strategy<Value = Shape> {
    (3u32..=5, 1u32..=2, any::<bool>())
        .prop_flat_map(|(side, radius, on_its_seam)| {
            (Just(side), Just(radius), 1..side, Just(on_its_seam))
        })
        .prop_map(
            |(side, radius, along, on_its_seam)| Shape::ABarBesideAWall {
                side,
                radius,
                along,
                on_its_seam,
            },
        )
}

pub fn a_bar_in_an_inside_corner() -> impl Strategy<Value = Shape> {
    (1u32..=2, 1u32..=2)
        .prop_flat_map(|(thick, radius)| {
            let narrowest = thick + 2 * radius + 2;
            (narrowest..=narrowest + 2, Just(thick), Just(radius))
        })
        .prop_map(|(arm, thick, radius)| Shape::ABarInAnInsideCorner { arm, thick, radius })
}

pub fn a_bar_standing_on_a_block() -> impl Strategy<Value = Shape> {
    (1u32..=2)
        .prop_flat_map(|radius| (2 * radius + 2..=2 * radius + 4, Just(radius)))
        .prop_flat_map(|(side, radius)| {
            let room = radius + 1..=side - radius - 1;
            (Just(side), Just(radius), [room.clone(), room])
        })
        .prop_map(|(side, radius, at)| Shape::ABarStandingOnABlock { side, radius, at })
}

pub fn a_ball_on_a_block() -> impl Strategy<Value = Shape> {
    (2u32..=5, 1u32..=2)
        .prop_flat_map(|(side, radius)| (Just(side), Just(radius), [1..side, 1..side]))
        .prop_map(|(side, radius, at)| Shape::ABallOnABlock { side, radius, at })
}

pub fn a_ball_against_a_wall() -> impl Strategy<Value = Shape> {
    (2u32..=5, 1u32..=2, any::<bool>())
        .prop_flat_map(|(side, radius, on_its_seam)| {
            (
                Just(side),
                Just(radius),
                [1..side, 1..side],
                Just(on_its_seam),
            )
        })
        .prop_map(|(side, radius, at, on_its_seam)| Shape::ABallAgainstAWall {
            side,
            radius,
            at,
            on_its_seam,
        })
}

fn a_direction() -> impl Strategy<Value = Towards> {
    prop_oneof![
        Just(Towards::PlusY),
        Just(Towards::PlusX),
        Just(Towards::MinusX)
    ]
}

pub fn two_bars_touching() -> impl Strategy<Value = Shape> {
    (1u32..=3, 1u32..=3, a_direction()).prop_map(|(first, second, towards)| {
        Shape::TwoBarsTouching {
            first,
            second,
            towards,
        }
    })
}

pub fn a_bar_inside_a_bar() -> impl Strategy<Value = Shape> {
    (1u32..=2, 2u32..=3, a_direction()).prop_map(|(inner, thicker, towards)| {
        Shape::ABarInsideABar {
            outer: inner + thicker,
            inner,
            towards,
        }
    })
}

pub fn two_balls_touching() -> impl Strategy<Value = Shape> {
    (1u32..=3, 1u32..=3, a_direction()).prop_map(|(first, second, towards)| {
        Shape::TwoBallsTouching {
            first,
            second,
            towards,
        }
    })
}

pub fn a_ball_inside_a_ball() -> impl Strategy<Value = Shape> {
    (1u32..=2, 2u32..=3, a_direction()).prop_map(|(inner, thicker, towards)| {
        Shape::ABallInsideABall {
            outer: inner + thicker,
            inner,
            towards,
        }
    })
}

pub fn a_dome_on_a_shaft() -> impl Strategy<Value = Shape> {
    (1u32..=3)
        .prop_flat_map(|radius| (Just(radius), radius + 1..=radius + 3))
        .prop_map(|(radius, height)| Shape::ADomeOnAShaft { radius, height })
}

pub fn a_ball_against_a_bar() -> impl Strategy<Value = Shape> {
    (1u32..=3, 1u32..=2, a_direction()).prop_map(|(bar, ball, towards)| Shape::ABallAgainstABar {
        bar,
        ball,
        towards,
    })
}

pub fn a_ball_inside_a_bar() -> impl Strategy<Value = Shape> {
    (1u32..=2, 2u32..=3, a_direction()).prop_map(|(ball, thicker, towards)| {
        Shape::ABallInsideABar {
            bar: ball + thicker,
            ball,
            towards,
        }
    })
}

fn a_slope() -> impl Strategy<Value = Slope> {
    prop_oneof![Just(Slope::One), Just(Slope::Half), Just(Slope::Two)]
}

pub fn a_cones_apex_on_a_block() -> impl Strategy<Value = Shape> {
    (a_slope(), [1u32..=3, 1u32..=3])
        .prop_map(|(slope, at)| Shape::AConesApexOnABlock { slope, at })
}

pub fn a_frustums_cap_on_a_block() -> impl Strategy<Value = Shape> {
    (a_slope(), 1u32..=3, 1u32..=3).prop_map(|(slope, near, tall)| Shape::AFrustumsCapOnABlock {
        slope,
        near,
        tall,
    })
}

pub fn a_bend_lying_on_a_block() -> impl Strategy<Value = Shape> {
    (1u32..=2)
        .prop_flat_map(|tube| (tube + 1..=tube + 3, Just(tube)))
        .prop_map(|(ring, tube)| Shape::ABendLyingOnABlock { ring, tube })
}

pub fn a_bends_cap_against_a_block() -> impl Strategy<Value = Shape> {
    (1u32..=2)
        .prop_flat_map(|tube| (tube + 1..=tube + 3, Just(tube)))
        .prop_map(|(ring, tube)| Shape::ABendsCapAgainstABlock { ring, tube })
}

fn to_i(number: u32) -> i32 {
    i32::try_from(number).expect("a small whole number")
}

/// A box with its low corner at `low` and the given lengths.
fn a_block(model: &mut Model, low: [i32; 3], size: [i32; 3]) -> SolidKey {
    let [x, y, z] = size.map(f64::from);
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(&rectangle, z, &at(low))
        .expect("building a box needs no decision")
}

/// An L standing on the floor: two arms `thick` across and `arm` long, meeting
/// at the origin, with the inside corner at `(thick, thick)`.
fn an_l(model: &mut Model, arm: i32, thick: i32, height: i32) -> SolidKey {
    let [arm, thick] = [arm, thick].map(f64::from);
    let outline = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [arm, 0.0],
        [arm, thick],
        [thick, thick],
        [thick, arm],
        [0.0, arm],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    model
        .add_extrusion(&outline, f64::from(height), &Isometry::identity())
        .expect("building an L needs no decision")
}

/// An upright bar whose bottom cap is centred on `foot`.
fn a_bar(model: &mut Model, radius: i32, height: i32, foot: [i32; 3]) -> SolidKey {
    model
        .add_round_bar(f64::from(radius), f64::from(height), &at(foot))
        .expect("a whole-number bar")
}

fn a_ball(model: &mut Model, radius: i32, centre: [i32; 3]) -> SolidKey {
    model
        .add_ball(f64::from(radius), &at(centre))
        .expect("a whole-number ball")
}

fn at([x, y, z]: [i32; 3]) -> Isometry {
    Isometry::translation(Vector::new(f64::from(x), f64::from(y), f64::from(z)))
        .expect("a whole-number shift")
}
