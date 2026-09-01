//! The families of surface pairs to cross, and the exactly representable
//! movements the control needs.
//!
//! A **sibling** of `rcad-topo/tests/pairs/mod.rs` and never an extension of it.
//! §13 rule 5: M2.5's suite and M3's must not be able to go green together for
//! the same wrong reason, and sharing a generator is how that happens. That one
//! extrudes rectilinear outlines; this one draws bars, balls and pipes and
//! nothing extruded at all (§24.11).
//!
//! # Why several families and not one
//!
//! §17.7 fixed the shape: report the families separately and never blend them
//! into one percentage.
//!
//! - [`a_lattice_crossing`] is the **control**. Every plane is square to an
//!   axis, every offset, centre and radius is a whole number, so every
//!   discriminant is whole-number arithmetic and every one of them is exactly
//!   representable. `Policy::certified()` must decide all of it: one refusal is
//!   one bug (§24.18 criterion 5).
//! - [`a_leaning_crossing`] tilts the planes. The normal is still a whole-number
//!   triple, but scaling it to length one divides by a square root, so what is
//!   stored is a rounding of what was asked for — and a plane that leans over a
//!   pipe is the case M2 could not declare at all, an ellipse (§24.2).
//! - [`a_turned_crossing`] is the **headline family**: the control under one
//!   general rotation, which leaves not a single exactly representable
//!   coordinate between the two surfaces.
//! - [`a_crossing_somewhere_else`] is the control **written somewhere else in
//!   `f64`** — every number multiplied by `2^k` and moved by `2^m`, both exact.
//!   A crossing is the same crossing wherever it is written, so this is where
//!   the naming half's one refusal shows up as what it is: a gap that a rescale
//!   walks straight back out of (§16.10).
//! - [`a_quartic_lattice_crossing`] and [`a_quartic_turned_crossing`] are the
//!   two rows M3's second stage added — a ball crossing a pipe, and a pipe
//!   crossing a pipe. Reported apart from the four above for §17.7's reason:
//!   they refuse for reasons the planar pairs do not have, and blending the two
//!   would hide both.
//! - [`a_crossing_a_hair_from_the_knife_edge`] is **the band**, and it is the
//!   one family here that was written because §28.5 named its absence.
//!
//! # The band between the on-case and the far-off case
//!
//! §28.5: *"a generator built out of exact configurations and one rotation
//! reaches the on-case and the far-off case and skips the near-case"*. Turning
//! two exactly parallel directions by the same rotation leaves them exactly
//! parallel, and turning a plane that rested exactly on a ball leaves it resting
//! on it or a whole rounding clear — never a hair off. So every family above
//! draws pairs that are exactly tangent or plainly not, and nothing in between.
//!
//! That band is where the marcher's tangency and near-tangency branches live,
//! and §24.1 says those branches are the whole reason the marcher exists. A
//! corpus that cannot reach them is a corpus that cannot fail.
//!
//! [`AHair`] is how this one reaches it, and neither half of it is a fixed
//! distance — §17.7 already says a nudge of `1e-9` is not the answer, because it
//! is the band at one scale and nowhere else. **A count of last bits** moves a
//! number off an exact tangency by the same relative amount wherever it is
//! written, and **a lean of `2^-k`** tilts a direction off a coordinate one by
//! an angle that sweeps from a quarter turn down to below what `f64` can tell
//! from square.
//!
//! # What shrinks, and toward what
//!
//! Every parameter is a small whole number and nothing here generates a
//! coordinate directly, so a failing case shrinks to something a person can
//! draw. The floors are a unit ball at the origin, a pipe of radius one down
//! `z`, and a plane square to `x` through the origin; no turn; and no scaling.

// Shared between test binaries, and each one uses a different subset of it.
#![allow(dead_code)]

use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;
use rcad_geom::{Axis, Cylinder, Isometry, Plane, Point, Sphere, Surface, Vector};

/// How far a surface may be placed from the origin, along each axis.
///
/// Shapes reach about four across, so this puts a pair anywhere from wholly
/// apart to concentric.
const FURTHEST_PLACE: i32 = 6;

/// The largest radius a ball or a pipe is drawn with.
const LARGEST_RADIUS: u32 = 5;

/// How far a whole-number normal may lean.
///
/// Three, so a leaning plane can be as near square to an axis as `(0, 1, 3)` and
/// as far from it as `(3, 3, 3)`, and every triple in between is a different
/// rounding of a different irrational direction.
const FURTHEST_LEAN: i32 = 3;

/// How many steps a whole turn is generated in.
///
/// Sixty-four, matching `rcad-topo/tests/pairs/mod.rs`, so that a quarter turn
/// is reachable — the angle whose cosine is nearest zero without being zero,
/// where a turned pair comes closest to being axis-aligned without getting
/// there.
const TURN_STEPS: u32 = 64;

/// The smallest power of two [`a_crossing_somewhere_else`] writes a pair at.
///
/// **A gap and not a limit** (§16.10), and it is deliberately past the edge.
/// Every discriminant here is a sum of products of two numbers, so at `2^k`
/// those products go as `2^{2k}`; once one of them would round its low half onto
/// the subnormal grid, exact arithmetic declines to certify a product it cannot
/// represent and the crossing comes back `Undecidable`. The answer was there all
/// along, which is what
/// `every_crossing_a_scaling_refuses_is_one_the_same_shapes_on_the_lattice_can_name`
/// demonstrates, and it is the same shape as
/// `the_same_pipe_and_point_scaled_up_to_fit_put_the_point_outside` one degree
/// down.
///
/// Measured on this tree, one pair of each kind: the lowest scale that answers
/// is `-538` for a plane against a ball or a pipe and `-539` for two balls. Two
/// planes answer all the way down, because their discriminants are products of
/// two direction components and of two offsets rather than of a radius with
/// itself.
///
/// Six hundred and twenty, so about a sixth of the family is drawn past one end
/// or the other. A range that stopped short of the edge would leave this whole
/// family a second copy of the control.
const SMALLEST_SCALE: i32 = -620;

/// The largest power of two [`a_crossing_somewhere_else`] writes a pair at.
///
/// **A gap as well**, which is worth saying because the same constant next door
/// in `rcad-topo/tests/pairs/mod.rs` is a limit. There a solid at `2^k` encloses
/// more space than an `f64` holds and the answer stops existing in the number
/// type; here the crossing is perfectly ordinary and it is the *discriminant*
/// that overflows, so writing the same two surfaces back on the lattice reaches
/// it.
///
/// Measured on this tree: the highest scale that answers is `511` for a plane
/// against a ball or a pipe and `509` for two balls.
const LARGEST_SCALE: i32 = 620;

/// The power of two a pair is moved by, at most.
///
/// Thirty, matching the pairs module next door: far enough that the shift
/// dominates every coordinate, near enough that a shift plus a small whole
/// number still fits in fifty-three bits and so is exact.
const FURTHEST_SHIFT_EXPONENT: u32 = 30;

/// One surface, in whole numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    /// A flat cut. `normal` is never all zero.
    Flat {
        /// Which way it faces, before being scaled to length one.
        normal: [i32; 3],
        /// How far along that direction it sits.
        at: i32,
    },
    /// A ball's skin.
    Ball {
        /// Where the middle is.
        centre: [i32; 3],
        /// How far the skin is from it. Never zero.
        radius: u32,
    },
    /// A round pipe's skin. `along` is never all zero.
    Pipe {
        /// Which way the axis runs.
        along: [i32; 3],
        /// A point the axis goes through.
        through: [i32; 3],
        /// How far the skin is from the axis. Never zero.
        radius: u32,
    },
}

/// Two surfaces to cross, and how what they are written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crossing {
    /// The first surface.
    pub first: Piece,
    /// The second.
    pub second: Piece,
    /// The turn applied to both, if any.
    pub turn: Option<Turn>,
    /// Where in `f64` both are written.
    pub elsewhere: Elsewhere,
    /// How far the second surface is moved off the configuration it was drawn
    /// at, or `None` for not at all.
    pub hair: Option<AHair>,
}

/// How far the second surface of a pair is moved off an exact configuration.
///
/// Both halves are relative rather than absolute, so a pair drawn at any size
/// lands the same distance into the band (§17.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AHair {
    /// How many of the last bits the radius is moved by. Zero leaves the exact
    /// configuration, which is worth drawing too.
    pub ulps: i32,
    /// How far the direction is tilted off the coordinate one it was drawn
    /// along, as `2^-lean`, or `None` for not at all.
    pub lean: Option<u32>,
}

impl AHair {
    /// `value` moved by [`AHair::ulps`] of its own last bits.
    #[must_use]
    fn applied_to(self, value: f64) -> f64 {
        let mut moved = value;
        for _ in 0..self.ulps.unsigned_abs() {
            moved = if self.ulps > 0 {
                moved.next_up()
            } else {
                moved.next_down()
            };
        }
        moved
    }

    /// `direction` tilted by `2^-lean` towards the coordinate direction it leans
    /// on least.
    #[must_use]
    fn tilting(self, direction: Vector) -> Vector {
        let Some(lean) = self.lean else {
            return direction;
        };
        let [x, y, z] = direction.components().map(f64::abs);
        let sideways = if x <= y && x <= z {
            Vector::new(1.0, 0.0, 0.0)
        } else if y <= z {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(0.0, 0.0, 1.0)
        };
        direction + sideways * two_to_the(-(lean as i32))
    }
}

/// How far a pair is turned before it is crossed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turn {
    /// Which way the axis points. Never all zero.
    pub direction: [i32; 3],
    /// The angle, in sixty-fourths of a whole turn.
    pub steps: u32,
}

/// Where in `f64` a pair is written, as against where in space.
///
/// Two knobs, both exactly representable: multiply every number by `2^scale`,
/// and move every place by `2^shift` along each axis. A power of two times a
/// whole number is exact until it underflows, and a power of two plus a small
/// whole number is exact while both fit in fifty-three bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Elsewhere {
    /// The power of two every number is multiplied by.
    pub scale: i32,
    /// The power of two every place is moved by, or `None` for no move.
    pub shift: Option<u32>,
}

impl Elsewhere {
    /// The one every family but [`a_crossing_somewhere_else`] uses: written
    /// exactly as it was drawn.
    #[must_use]
    pub fn here() -> Self {
        Self {
            scale: 0,
            shift: None,
        }
    }
}

impl Crossing {
    /// The two surfaces, built.
    ///
    /// # Panics
    ///
    /// If a generated piece is not a surface. Nothing here generates one: every
    /// direction is non-zero, every radius is at least one, and both survive
    /// being multiplied by a power of two.
    #[must_use]
    pub fn surfaces(&self) -> [Surface; 2] {
        let turn = self
            .turn
            .map_or_else(Isometry::identity, |turn| turn.isometry());
        [(self.first, None), (self.second, self.hair)].map(|(piece, hair)| {
            piece
                .surface(self.elsewhere, hair)
                .expect("every generated piece is a surface")
                .transformed(&turn)
        })
    }

    /// How big the numbers in this pair are, which is what a published bound is
    /// a multiple of.
    #[must_use]
    pub fn how_big(&self) -> f64 {
        let widest = [self.first, self.second]
            .iter()
            .map(|piece| piece.how_big(self.elsewhere))
            .fold(0.0, f64::max);
        widest.max(f64::MIN_POSITIVE)
    }
}

impl Piece {
    /// This piece as a surface, written where `elsewhere` says and moved off it
    /// by as much as `hair` asks.
    fn surface(self, elsewhere: Elsewhere, hair: Option<AHair>) -> rcad_geom::Result<Surface> {
        let by = two_to_the(elsewhere.scale);
        let shift = elsewhere
            .shift
            .map_or(0.0, |shift| two_to_the(shift as i32));
        let placed = |whole: [i32; 3]| Point::from(whole.map(|step| f64::from(step) * by + shift));
        let sized = |radius: u32| {
            let radius = f64::from(radius) * by;
            hair.map_or(radius, |hair| hair.applied_to(radius))
        };
        let direction = |whole: [i32; 3]| {
            let along = Vector::from(whole.map(f64::from));
            hair.map_or(along, |hair| hair.tilting(along))
        };
        match self {
            Piece::Flat { normal, at } => {
                let facing = direction(normal);
                let reach = hair.map_or(f64::from(at) * by, |hair| {
                    hair.applied_to(f64::from(at) * by)
                });
                let through = Point::origin() + facing * (reach / facing.length());
                let through = Point::from(through.coordinates().map(|step| step + shift));
                Ok(Surface::Plane(Plane::from_point_and_normal(
                    through, facing,
                )?))
            }
            Piece::Ball { centre, radius } => {
                Ok(Surface::Sphere(Sphere::new(placed(centre), sized(radius))?))
            }
            Piece::Pipe {
                along,
                through,
                radius,
            } => Ok(Surface::Cylinder(Cylinder::new(
                Axis::new(placed(through), direction(along))?,
                sized(radius),
            )?)),
        }
    }

    /// The largest number this piece is placed or sized with.
    fn how_big(self, elsewhere: Elsewhere) -> f64 {
        let by = two_to_the(elsewhere.scale);
        let shift = elsewhere
            .shift
            .map_or(0.0, |shift| two_to_the(shift as i32));
        let furthest = |whole: [i32; 3]| {
            whole
                .iter()
                .map(|step| f64::from(*step).abs() * by + shift)
                .fold(0.0, f64::max)
        };
        match self {
            Piece::Flat { at, .. } => f64::from(at).abs() * by + shift,
            Piece::Ball { centre, radius } => furthest(centre).max(f64::from(radius) * by),
            Piece::Pipe {
                through, radius, ..
            } => furthest(through).max(f64::from(radius) * by),
        }
    }
}

impl Turn {
    /// The rotation itself.
    ///
    /// # Panics
    ///
    /// If the direction has no length. The generator never makes one.
    #[must_use]
    pub fn isometry(&self) -> Isometry {
        let direction = Vector::from(self.direction.map(f64::from));
        let axis = Axis::new(Point::origin(), direction).expect("a direction with length");
        let angle = core::f64::consts::TAU * f64::from(self.steps) / f64::from(TURN_STEPS);
        Isometry::rotation(&axis, angle).expect("a finite rotation")
    }
}

/// Two flat cuts and a third surface, and what all three are written in.
///
/// A [`Crossing`] with one more piece on it. The first two are always flat,
/// because that is the shape `meet_of_three_surfaces` answers and `docs/
/// decisions.md` §31.2 says why: a line and a quadric is a quadratic, and one
/// plane with two quadrics is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Corner {
    /// The first flat cut.
    pub first: Piece,
    /// The second flat cut.
    pub second: Piece,
    /// What the line between them runs into.
    pub third: Piece,
    /// The turn applied to all three, if any.
    pub turn: Option<Turn>,
    /// Where in `f64` all three are written.
    pub elsewhere: Elsewhere,
}

impl Corner {
    /// The three surfaces, built.
    ///
    /// # Panics
    ///
    /// If a generated piece is not a surface. Nothing here generates one.
    #[must_use]
    pub fn surfaces(&self) -> [Surface; 3] {
        let turn = self
            .turn
            .map_or_else(Isometry::identity, |turn| turn.isometry());
        [self.first, self.second, self.third].map(|piece| {
            piece
                .surface(self.elsewhere, None)
                .expect("every generated piece is a surface")
                .transformed(&turn)
        })
    }

    /// How big the numbers in this triple are, which is what a published bound
    /// is a multiple of.
    #[must_use]
    pub fn how_big(&self) -> f64 {
        let widest = [self.first, self.second, self.third]
            .iter()
            .map(|piece| piece.how_big(self.elsewhere))
            .fold(0.0, f64::max);
        widest.max(f64::MIN_POSITIVE)
    }
}

/// The control: every plane square to an axis, every number a whole one.
pub fn a_lattice_corner() -> impl Strategy<Value = Corner> {
    a_triple(an_axis_direction().boxed())
}

/// The same shapes with the planes and pipes tilted, so a stored direction is a
/// rounding of the one that was asked for.
pub fn a_leaning_corner() -> impl Strategy<Value = Corner> {
    a_triple(a_leaning_direction().boxed())
}

/// The headline: the control under one general rotation.
pub fn a_turned_corner() -> impl Strategy<Value = Corner> {
    (a_lattice_corner(), a_turn()).prop_map(|(corner, turn)| Corner {
        turn: Some(turn),
        ..corner
    })
}

/// The control, written somewhere else in `f64`.
pub fn a_corner_somewhere_else() -> impl Strategy<Value = Corner> {
    (a_lattice_corner(), an_elsewhere()).prop_map(|(corner, elsewhere)| Corner {
        elsewhere,
        ..corner
    })
}

/// Two flat cuts and one of the three surface kinds.
///
/// The third is drawn flat as often as it is drawn round, because three planes
/// is a real corner and the arithmetic that answers it is a different one.
fn a_triple(direction: BoxedStrategy<[i32; 3]>) -> impl Strategy<Value = Corner> {
    let flat = a_flat(direction.clone());
    (
        flat.clone(),
        flat.clone(),
        prop_oneof![flat, a_ball(), a_pipe(direction)],
    )
        .prop_map(|(first, second, third)| Corner {
            first,
            second,
            third,
            turn: None,
            elsewhere: Elsewhere::here(),
        })
}

/// The control: every plane square to an axis, every number a whole one.
pub fn a_lattice_crossing() -> impl Strategy<Value = Crossing> {
    a_pair(an_axis_direction().boxed())
}

/// The same shapes with the planes and pipes tilted, so a stored direction is a
/// rounding of the one that was asked for.
pub fn a_leaning_crossing() -> impl Strategy<Value = Crossing> {
    a_pair(a_leaning_direction().boxed())
}

/// The headline: the control under one general rotation.
pub fn a_turned_crossing() -> impl Strategy<Value = Crossing> {
    (a_lattice_crossing(), a_turn()).prop_map(|(crossing, turn)| Crossing {
        turn: Some(turn),
        ..crossing
    })
}

/// The control, written somewhere else in `f64`.
pub fn a_crossing_somewhere_else() -> impl Strategy<Value = Crossing> {
    (a_lattice_crossing(), an_elsewhere()).prop_map(|(crossing, elsewhere)| Crossing {
        elsewhere,
        ..crossing
    })
}

/// One of the four pairs whose crossing lies in a plane, either way round.
///
/// **No pipe crosses a pipe and no ball crosses a pipe here.** Those two cross
/// in a degree-four space curve and are drawn by [`a_quartic_pair`] instead, so
/// that the two kinds of refusal are reported apart (§17.7).
///
/// Either way round, because a caller has no reason to put a pair in one order
/// and the answer is about the pair rather than about the arguments.
fn a_pair(direction: BoxedStrategy<[i32; 3]>) -> impl Strategy<Value = Crossing> {
    let flat = a_flat(direction.clone());
    let both_ways = prop_oneof![
        (flat.clone(), flat.clone()),
        (flat.clone(), a_ball()),
        (flat, a_pipe(direction)),
        (a_ball(), a_ball()),
    ];
    (both_ways, prop::bool::ANY).prop_map(|((first, second), swapped)| {
        let (first, second) = if swapped {
            (second, first)
        } else {
            (first, second)
        };
        Crossing {
            first,
            second,
            turn: None,
            elsewhere: Elsewhere::here(),
            hair: None,
        }
    })
}

/// The two rows M3's second stage added, on the lattice.
pub fn a_quartic_lattice_crossing() -> impl Strategy<Value = Crossing> {
    a_quartic_pair(an_axis_direction().boxed())
}

/// The same two rows under one general rotation.
pub fn a_quartic_turned_crossing() -> impl Strategy<Value = Crossing> {
    (a_quartic_lattice_crossing(), a_turn()).prop_map(|(crossing, turn)| Crossing {
        turn: Some(turn),
        ..crossing
    })
}

/// A ball crossing a pipe, or a pipe crossing a pipe, either way round.
fn a_quartic_pair(direction: BoxedStrategy<[i32; 3]>) -> impl Strategy<Value = Crossing> {
    let both_ways = prop_oneof![
        (a_ball(), a_pipe(direction.clone())),
        (a_pipe(direction.clone()), a_pipe(direction)),
    ];
    (both_ways, prop::bool::ANY).prop_map(|((first, second), swapped)| {
        let (first, second) = if swapped {
            (second, first)
        } else {
            (first, second)
        };
        Crossing {
            first,
            second,
            turn: None,
            elsewhere: Elsewhere::here(),
            hair: None,
        }
    })
}

/// **The band.** Pairs drawn exactly tangent, then moved a handful of last bits
/// off it, with an optional hair of lean between two directions that were
/// exactly parallel.
///
/// Seven configurations, one for every way two of these surfaces can rest
/// against each other, and each of them arranged in whole numbers so that the
/// pair at `ulps = 0` is exactly on the knife edge:
///
/// | | tangent when |
/// |---|---|
/// | a plane on a ball | the plane sits a radius from the centre |
/// | a plane along a pipe | the plane sits a radius from the axis |
/// | two balls, outside and inside | the centres are `R₁ ± R₂` apart |
/// | a ball on the outside of a pipe | `R = d − r`, for a centre `d` off the axis |
/// | a ball across the whole of a pipe | `R = r + d`, where the two loops pinch |
/// | two pipes side by side | the axes are `r₁ + r₂` apart |
/// | two pipes leaning | the axes pass `r₁ + r₂` apart |
///
/// A pair drawn here and answered without complaint is a pair whose sign
/// questions were settled a few last bits from zero, which is what none of the
/// families above can produce.
pub fn a_crossing_a_hair_from_the_knife_edge() -> impl Strategy<Value = Crossing> {
    let resting =
        (1..=LARGEST_RADIUS, 1..=LARGEST_RADIUS, 0usize..7).prop_map(|(mine, theirs, which)| {
            let (bigger, smaller) = (mine.max(theirs), mine.min(theirs));
            let up = [0, 0, 1];
            let across = [1, 0, 0];
            let out = |reach: u32| [reach as i32, 0, 0];
            match which {
                0 => (
                    Piece::Flat {
                        normal: across,
                        at: mine as i32,
                    },
                    Piece::Ball {
                        centre: [0, 0, 0],
                        radius: mine,
                    },
                ),
                1 => (
                    Piece::Flat {
                        normal: across,
                        at: mine as i32,
                    },
                    Piece::Pipe {
                        along: up,
                        through: [0, 0, 0],
                        radius: mine,
                    },
                ),
                2 => (
                    Piece::Ball {
                        centre: [0, 0, 0],
                        radius: mine,
                    },
                    Piece::Ball {
                        centre: out(mine + theirs),
                        radius: theirs,
                    },
                ),
                3 => (
                    Piece::Ball {
                        centre: [0, 0, 0],
                        radius: bigger,
                    },
                    Piece::Ball {
                        centre: out(bigger - smaller),
                        radius: smaller,
                    },
                ),
                // A ball whose skin just reaches the near side of a pipe it sits
                // clear of: `R = d − r` with the centre `d` off the axis.
                4 => (
                    Piece::Pipe {
                        along: up,
                        through: [0, 0, 0],
                        radius: mine,
                    },
                    Piece::Ball {
                        centre: out(mine + theirs + theirs),
                        radius: mine + theirs,
                    },
                ),
                // A ball reaching exactly across the whole width of a pipe, so
                // the two loops it would leave pinch together.
                5 => (
                    Piece::Pipe {
                        along: up,
                        through: [0, 0, 0],
                        radius: mine + theirs,
                    },
                    Piece::Ball {
                        centre: out(theirs),
                        radius: mine + theirs + theirs,
                    },
                ),
                6 => (
                    Piece::Pipe {
                        along: up,
                        through: [0, 0, 0],
                        radius: mine,
                    },
                    Piece::Pipe {
                        along: up,
                        through: out(mine + theirs),
                        radius: theirs,
                    },
                ),
                _ => (
                    Piece::Pipe {
                        along: up,
                        through: [0, 0, 0],
                        radius: mine,
                    },
                    Piece::Pipe {
                        along: across,
                        through: [0, (mine + theirs) as i32, 0],
                        radius: theirs,
                    },
                ),
            }
        });
    (resting, a_hair(), prop::option::of(a_turn())).prop_map(|((first, second), hair, turn)| {
        Crossing {
            first,
            second,
            turn,
            elsewhere: Elsewhere::here(),
            hair: Some(hair),
        }
    })
}

/// How far off the knife edge a pair is drawn.
///
/// Four last bits either way, so the band is swept from one side to the other
/// with the exact configuration in the middle of it; and a lean from a quarter
/// turn down to `2^-60`, which is past where any `f64` dot product can tell a
/// direction from square to another.
fn a_hair() -> impl Strategy<Value = AHair> {
    (-4i32..=4, prop::option::of(0u32..=60)).prop_map(|(ulps, lean)| AHair { ulps, lean })
}

fn a_flat(direction: BoxedStrategy<[i32; 3]>) -> BoxedStrategy<Piece> {
    (direction, -FURTHEST_PLACE..=FURTHEST_PLACE)
        .prop_map(|(normal, at)| Piece::Flat { normal, at })
        .boxed()
}

fn a_ball() -> BoxedStrategy<Piece> {
    (a_place(), 1..=LARGEST_RADIUS)
        .prop_map(|(centre, radius)| Piece::Ball { centre, radius })
        .boxed()
}

fn a_pipe(direction: BoxedStrategy<[i32; 3]>) -> BoxedStrategy<Piece> {
    (direction, a_place(), 1..=LARGEST_RADIUS)
        .prop_map(|(along, through, radius)| Piece::Pipe {
            along,
            through,
            radius,
        })
        .boxed()
}

fn a_place() -> impl Strategy<Value = [i32; 3]> {
    [
        -FURTHEST_PLACE..=FURTHEST_PLACE,
        -FURTHEST_PLACE..=FURTHEST_PLACE,
        -FURTHEST_PLACE..=FURTHEST_PLACE,
    ]
}

/// One of the six coordinate directions, so scaling it to length one divides by
/// exactly one and changes nothing.
fn an_axis_direction() -> impl Strategy<Value = [i32; 3]> {
    (0usize..3, prop::bool::ANY).prop_map(|(axis, backwards)| {
        let mut direction = [0, 0, 0];
        direction[axis] = if backwards { -1 } else { 1 };
        direction
    })
}

/// A whole-number direction that is usually not a coordinate one.
fn a_leaning_direction() -> impl Strategy<Value = [i32; 3]> {
    [
        -FURTHEST_LEAN..=FURTHEST_LEAN,
        -FURTHEST_LEAN..=FURTHEST_LEAN,
        -FURTHEST_LEAN..=FURTHEST_LEAN,
    ]
    .prop_map(|direction| {
        if direction == [0, 0, 0] {
            [0, 0, 1]
        } else {
            direction
        }
    })
}

fn a_turn() -> impl Strategy<Value = Turn> {
    ([-3i32..=3, -3i32..=3, -3i32..=3], 1u32..TURN_STEPS).prop_map(|(direction, steps)| Turn {
        direction: if direction == [0, 0, 0] {
            [0, 0, 1]
        } else {
            direction
        },
        steps,
    })
}

fn an_elsewhere() -> impl Strategy<Value = Elsewhere> {
    (
        SMALLEST_SCALE..=LARGEST_SCALE,
        prop::option::of(0u32..=FURTHEST_SHIFT_EXPONENT),
    )
        .prop_map(|(scale, shift)| Elsewhere { scale, shift })
}

/// `2^exponent`, exactly, for any exponent `f64` can hold as a normal number.
fn two_to_the(exponent: i32) -> f64 {
    2f64.powi(exponent)
}
