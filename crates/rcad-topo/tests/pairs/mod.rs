//! Three families of solid pairs to fuse, and the exactly representable
//! movements one of the properties needs.
//!
//! # Why three families and not one
//!
//! `docs/decisions.md` §12.3 is the section where one family of shapes found
//! one family of bugs, and §17.7 says what to do about it here: report three
//! configuration families separately and never as one number.
//!
//! - [`a_lattice_pair`] is the **control**. Rectilinear outlines on the integer
//!   grid, extruded by a whole number, put down at a whole-number place. Every
//!   plane is axis-aligned, every corner is a whole number, and every corner the
//!   fuse invents is the meet of three axis-aligned planes and so a whole number
//!   too. `Policy::certified()` must decide all of it: a refusal here is a bug
//!   in our filters, not a fact about geometry.
//! - [`a_turned_pair`] is the **headline family**. The same configurations under
//!   one general rotation, which leaves not a single exactly representable
//!   coordinate between them. Refusals are expected and are the thing M2.5
//!   exists to measure.
//! - [`a_flush_pair`] is **deliberately degenerate**. Both solids are the same
//!   shape, and the second is offset by that shape's own extent — so the two
//!   share coordinates exactly and the coplanarity is true rather than nearly
//!   true. §17.7 rejects nudging a valid pair by `1e-9`: that makes the *near*
//!   case and never the *on* case.
//!
//! # Why these shapes
//!
//! Rectilinear, because the control family's whole worth is that its answers are
//! exactly representable, and one diagonal edge ends that. A profile corner at
//! `(1, 1)` gives a side face whose normal is `(1, 1, 0)` scaled to unit length,
//! which is irrational; the corners the fuse then solves for are no longer whole
//! numbers, volumes stop being exact, and the control can no longer tell a
//! wrongly refused input from a hard one. §17.7 says "integer grid,
//! axis-aligned" and this is why.
//!
//! Within that, four outlines chosen for what each of the others cannot make:
//! a plain [`Shape::Block`]; an [`Shape::Ell`] with one reflex corner; a
//! [`Shape::Comb`] with several, so a cut line crosses the outline more than
//! twice; and a [`Shape::Frame`], the only one with an inner wire, so the answer
//! has to keep holes apart from outlines.
//!
//! # What shrinks, and toward what
//!
//! Every parameter is a small whole number and nothing here generates a
//! coordinate directly, so a failing case shrinks to something a person can draw
//! (roadmap §11.3, and `rcad/tests/shapes/mod.rs` for the same argument at
//! length). The floors are: a block of one by one, extruded by one, at the
//! origin, fused with another of the same; no turn; and for the degenerate
//! family two solids in exactly the same place.
//!
//! Every parameter combination builds, so nothing filters. Widths and depths are
//! generated at their lower bound or above, and the sizes that have to stay
//! inside them are clamped rather than rejected — a clamp keeps shrinking
//! monotone, and `prop_filter` would throw away shrink candidates.

// Shared between test binaries, and each one uses a different subset of it.
#![allow(dead_code)]

use proptest::prelude::*;
use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, SolidKey};

/// Roughly how big a generated configuration is, for [`Scale`].
pub const WORKING_SIZE: f64 = 100.0;

/// How far a solid may be shifted along each axis.
///
/// Shapes reach about six across, so this puts a pair anywhere from wholly
/// apart to wholly inside one another.
const FURTHEST_SHIFT: i32 = 5;

/// How many steps a whole turn is generated in.
///
/// Sixty-four so that a quarter turn is reachable — the angle whose cosine is
/// nearest zero without being zero, which is where a rotated configuration comes
/// closest to being axis-aligned without ever getting there.
const TURN_STEPS: u32 = 64;

/// A rectilinear outline on the integer grid, and its hole if it has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A plain rectangle.
    Block {
        /// How far it reaches along `x`.
        width: u32,
        /// How far it reaches along `y`.
        depth: u32,
    },
    /// A rectangle with a rectangular bite out of its far corner, so one corner
    /// of the outline turns back on itself.
    Ell {
        /// How far the whole thing reaches along `x`.
        width: u32,
        /// How far the whole thing reaches along `y`.
        depth: u32,
        /// How wide the bite is, clamped to leave at least one behind.
        bite_width: u32,
        /// How deep the bite is, clamped to leave at least one behind.
        bite_depth: u32,
    },
    /// Square teeth standing on a base one thick.
    Comb {
        /// How many teeth.
        teeth: u32,
        /// How far a tooth stands above the base.
        tooth_height: u32,
    },
    /// A rectangle with a rectangular hole in the middle of it, walls one thick.
    Frame {
        /// How far it reaches along `x`.
        width: u32,
        /// How far it reaches along `y`.
        depth: u32,
    },
}

/// One solid: an outline, how far it is extruded, and where it is put down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// What it is cut from.
    pub shape: Shape,
    /// How far it is extruded along `z`.
    pub height: u32,
    /// Where the outline's own origin ends up.
    pub at: [i32; 3],
}

/// A quarter turn about the `z` axis and a whole-number shift.
///
/// Every one of these is exact in `f64`: a quarter turn about `z` sends
/// `(x, y, z)` to `(-y, x, z)`, which is a swap and a negation, and adding a
/// whole number to a whole number is exact while both stay well inside `2⁵³`.
/// That is what makes the transform-invariance property a bit-for-bit claim
/// rather than one needing a tolerance.
///
/// Deliberately *not* general rotations, and deliberately not built through
/// [`Isometry::rotation`]. That constructor takes an angle, and `cos(π/2)` in
/// `f64` is `6.1e-17` rather than zero, so even a quarter turn arrives as a
/// matrix a rounding away from the one that was asked for. A test comparing
/// those two answers would need a tolerance, and tuning a tolerance until a test
/// passes makes the test measure the tolerance (§14.1 question 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rigid {
    /// How many quarter turns about `z`, anticlockwise.
    pub quarters: u32,
    /// How far to move afterwards, along each axis.
    pub shift: [i32; 3],
}

/// A general rotation: a whole-number axis direction and a share of a whole
/// turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turn {
    /// Which way the axis points. Never all zero.
    pub direction: [i32; 3],
    /// The angle, in sixty-fourths of a whole turn.
    pub steps: u32,
}

/// A whole configuration: the solids, and the one rotation applied to all of
/// them.
///
/// The turn is shared rather than one per solid on purpose. It leaves the solids
/// in the same places relative to each other as the lattice configuration they
/// came from, so a refusal is attributable to the rotation and nothing else —
/// which is the comparison §17.7's control row exists to make.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    /// The solids, in the order a fuse should be handed them.
    pub solids: Vec<Placed>,
    /// The rotation every solid is turned by, if any.
    pub turn: Option<Turn>,
}

/// Two rectilinear solids on the integer grid, near enough to interact.
pub fn a_lattice_pair() -> impl Strategy<Value = Configuration> {
    lattice_solids(2).prop_map(|solids| Configuration { solids, turn: None })
}

/// Three of them, for the property that fusing all three at once agrees with
/// fusing two and then the third.
pub fn a_lattice_trio() -> impl Strategy<Value = Configuration> {
    lattice_solids(3).prop_map(|solids| Configuration { solids, turn: None })
}

/// The lattice pairs under one general rotation, which leaves nothing about them
/// exactly representable.
pub fn a_turned_pair() -> impl Strategy<Value = Configuration> {
    (lattice_solids(2), a_turn()).prop_map(|(solids, turn)| Configuration {
        solids,
        turn: Some(turn),
    })
}

/// Two solids of the same shape, the second offset by a whole number of that
/// shape's own extents.
///
/// The offsets are `0`, one whole extent, and half of one, each way. So the pair
/// is coincident, or flush face to face, or meeting along an edge, or with one's
/// corner exactly on the other's face — every one of them exact, because the
/// same number appears in both solids rather than the two being a nanometre
/// apart (§17.7, roadmap §11.3).
pub fn a_flush_pair() -> impl Strategy<Value = Configuration> {
    (a_shape(), 1u32..=3, [0u32..5, 0u32..5, 0u32..5]).prop_map(|(shape, height, choices)| {
        let extent = [
            i32::try_from(shape.width()).expect("a small whole number"),
            i32::try_from(shape.depth()).expect("a small whole number"),
            i32::try_from(height).expect("a small whole number"),
        ];
        let at = [0, 1, 2].map(|axis| flush_offsets(extent[axis])[choices[axis] as usize]);
        Configuration {
            solids: vec![
                Placed {
                    shape,
                    height,
                    at: [0, 0, 0],
                },
                Placed { shape, height, at },
            ],
            turn: None,
        }
    })
}

/// One solid and a second one built from the same numbers in the same place.
///
/// Two solids rather than one solid twice, because a fuse names its two sides by
/// key and the same key on both sides would be a different question.
pub fn a_solid_and_its_copy() -> impl Strategy<Value = Configuration> {
    a_copied_solid().prop_map(|solids| Configuration { solids, turn: None })
}

/// The same under a general rotation, where the two solids still share every
/// coordinate because they are turned by the very same rotation.
pub fn a_turned_solid_and_its_copy() -> impl Strategy<Value = Configuration> {
    (a_copied_solid(), a_turn()).prop_map(|(solids, turn)| Configuration {
        solids,
        turn: Some(turn),
    })
}

/// A quarter turn or three, and a whole-number shift.
pub fn a_rigid_movement() -> impl Strategy<Value = Rigid> {
    (
        0u32..4,
        [
            -FURTHEST_SHIFT..=FURTHEST_SHIFT,
            -FURTHEST_SHIFT..=FURTHEST_SHIFT,
            -FURTHEST_SHIFT..=FURTHEST_SHIFT,
        ],
    )
        .prop_map(|(quarters, shift)| Rigid { quarters, shift })
}

impl Configuration {
    /// Build the solids into a fresh model.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        self.built_after(&Rigid::still())
    }

    /// Build them with `movement` applied first, exactly.
    ///
    /// The quarter turn is applied to the outline's whole-number corners and the
    /// shift to where the solid is put down, so no `f64` rounds anywhere on the
    /// way in. A [`Configuration`] with a [`Turn`] is turned afterwards, which
    /// does round — the two are composed in that order because the general
    /// rotation is the thing being measured and the quarter turn is the thing
    /// being trusted.
    #[must_use]
    pub fn built_after(&self, movement: &Rigid) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let keys: Vec<SolidKey> = self
            .solids
            .iter()
            .map(|placed| placed.built_after(movement, &mut model))
            .collect();
        if let Some(turn) = self.turn {
            let turn = turn.isometry();
            for &key in &keys {
                model.place_solid(key, &turn);
            }
        }
        (model, keys)
    }
}

impl Placed {
    fn built_after(&self, movement: &Rigid, model: &mut Model) -> SolidKey {
        let rings = self.shape.rings();
        let flat = |ring: &[[i32; 2]]| -> Vec<[f64; 2]> {
            ring.iter()
                .map(|&[x, y]| {
                    let [x, y, _] = movement.whole_point([x + self.at[0], y + self.at[1], 0]);
                    [f64::from(x), f64::from(y)]
                })
                .collect()
        };
        let mut builder = ProfileBuilder::new(flat(&rings.outline));
        if let Some(hole) = &rings.hole {
            builder = builder.with_hole(flat(hole));
        }
        let profile = builder
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("every generated outline is a sound profile");
        let lift = Vector::new(0.0, 0.0, f64::from(self.at[2] + movement.shift[2]));
        model
            .add_extrusion(
                &profile,
                f64::from(self.height),
                &Isometry::translation(lift).expect("a whole-number lift"),
            )
            .expect("every generated profile extrudes")
    }
}

impl Rigid {
    /// The movement that changes nothing.
    #[must_use]
    pub fn still() -> Self {
        Self {
            quarters: 0,
            shift: [0, 0, 0],
        }
    }

    /// Where a point ends up. Exact, whatever the coordinates are.
    #[must_use]
    pub fn point(&self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        let [x, y, z] = self.direction([x, y, z]);
        [
            x + f64::from(self.shift[0]),
            y + f64::from(self.shift[1]),
            z + f64::from(self.shift[2]),
        ]
    }

    /// Which way a direction ends up pointing. A swap and a negation, so exact.
    #[must_use]
    pub fn direction(&self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        match self.quarters % 4 {
            0 => [x, y, z],
            1 => [-y, x, z],
            2 => [-x, -y, z],
            _ => [y, -x, z],
        }
    }

    /// The plane `a x + b y + c z + d = 0` after the movement.
    ///
    /// The normal turns with the movement and the constant picks up how far the
    /// plane slid along its own normal, `d' = d - n' · t`. With an axis-aligned
    /// normal that dot product is one whole-number term and two zeroes, so it is
    /// exact — which is the second half of why this family is rectilinear.
    #[must_use]
    pub fn plane(&self, [a, b, c, d]: [f64; 4]) -> [f64; 4] {
        let [a, b, c] = self.direction([a, b, c]);
        let along = a * f64::from(self.shift[0])
            + b * f64::from(self.shift[1])
            + c * f64::from(self.shift[2]);
        [a, b, c, d - along]
    }

    fn whole_point(&self, [x, y, z]: [i32; 3]) -> [i32; 3] {
        let [x, y, z] = match self.quarters % 4 {
            0 => [x, y, z],
            1 => [-y, x, z],
            2 => [-x, -y, z],
            _ => [y, -x, z],
        };
        [x + self.shift[0], y + self.shift[1], z]
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
        let direction = Vector::new(
            f64::from(self.direction[0]),
            f64::from(self.direction[1]),
            f64::from(self.direction[2]),
        );
        let axis = Axis::new(Point::origin(), direction).expect("a direction with length");
        let angle = core::f64::consts::TAU * f64::from(self.steps) / f64::from(TURN_STEPS);
        Isometry::rotation(&axis, angle).expect("a finite rotation")
    }
}

/// The rings of a rectilinear outline: the outline itself and its hole.
#[derive(Clone, Debug)]
pub struct Rings {
    /// The outline, anticlockwise.
    pub outline: Vec<[i32; 2]>,
    /// Its one hole, if it has one.
    pub hole: Option<Vec<[i32; 2]>>,
}

impl Shape {
    /// How far the shape reaches along `x`.
    #[must_use]
    pub fn width(self) -> u32 {
        match self {
            Shape::Block { width, .. } | Shape::Ell { width, .. } | Shape::Frame { width, .. } => {
                width
            }
            Shape::Comb { teeth, .. } => 2 * teeth - 1,
        }
    }

    /// How far the shape reaches along `y`.
    #[must_use]
    pub fn depth(self) -> u32 {
        match self {
            Shape::Block { depth, .. } | Shape::Ell { depth, .. } | Shape::Frame { depth, .. } => {
                depth
            }
            Shape::Comb { tooth_height, .. } => 1 + tooth_height,
        }
    }

    /// The corners, anticlockwise, with the outline's own origin at `(0, 0)`.
    #[must_use]
    pub fn rings(self) -> Rings {
        match self {
            Shape::Block { width, depth } => Rings {
                outline: rectangle([0, 0], [whole(width), whole(depth)]),
                hole: None,
            },
            Shape::Ell {
                width,
                depth,
                bite_width,
                bite_depth,
            } => {
                let (width, depth) = (whole(width), whole(depth));
                let bite_width = whole(bite_width).min(width - 1);
                let bite_depth = whole(bite_depth).min(depth - 1);
                Rings {
                    outline: vec![
                        [0, 0],
                        [width, 0],
                        [width, depth - bite_depth],
                        [width - bite_width, depth - bite_depth],
                        [width - bite_width, depth],
                        [0, depth],
                    ],
                    hole: None,
                }
            }
            Shape::Comb {
                teeth,
                tooth_height,
            } => Rings {
                outline: comb(whole(teeth), whole(tooth_height)),
                hole: None,
            },
            Shape::Frame { width, depth } => {
                let (width, depth) = (whole(width), whole(depth));
                Rings {
                    outline: rectangle([0, 0], [width, depth]),
                    hole: Some(rectangle([1, 1], [width - 1, depth - 1])),
                }
            }
        }
    }
}

/// One of the four outlines, each shrinking to its own smallest.
fn a_shape() -> impl Strategy<Value = Shape> {
    prop_oneof![
        (1u32..=4, 1u32..=4).prop_map(|(width, depth)| Shape::Block { width, depth }),
        (2u32..=5, 2u32..=5, 1u32..=4, 1u32..=4).prop_map(
            |(width, depth, bite_width, bite_depth)| Shape::Ell {
                width,
                depth,
                bite_width,
                bite_depth,
            }
        ),
        (1u32..=3, 1u32..=3).prop_map(|(teeth, tooth_height)| Shape::Comb {
            teeth,
            tooth_height,
        }),
        (3u32..=5, 3u32..=5).prop_map(|(width, depth)| Shape::Frame { width, depth }),
    ]
}

/// `count` solids, the first at the origin and the rest shifted about it.
fn lattice_solids(count: usize) -> impl Strategy<Value = Vec<Placed>> {
    proptest::collection::vec(
        (
            a_shape(),
            1u32..=3,
            [
                -FURTHEST_SHIFT..=FURTHEST_SHIFT,
                -FURTHEST_SHIFT..=FURTHEST_SHIFT,
                -FURTHEST_SHIFT..=FURTHEST_SHIFT,
            ],
        ),
        count,
    )
    .prop_map(|solids| {
        solids
            .into_iter()
            .enumerate()
            .map(|(index, (shape, height, at))| Placed {
                shape,
                height,
                at: if index == 0 { [0, 0, 0] } else { at },
            })
            .collect()
    })
}

/// One solid, twice over, in the same place both times.
fn a_copied_solid() -> impl Strategy<Value = Vec<Placed>> {
    (a_shape(), 1u32..=3).prop_map(|(shape, height)| {
        let placed = Placed {
            shape,
            height,
            at: [0, 0, 0],
        };
        vec![placed, placed]
    })
}

/// A general rotation about a whole-number axis through the origin.
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

/// Where the second solid of a degenerate pair sits along one axis.
///
/// In the order shrinking walks: right on top of the first, then flush against
/// it either way, then halfway through it either way.
fn flush_offsets(extent: i32) -> [i32; 5] {
    [0, extent, -extent, extent / 2, -(extent / 2)]
}

fn rectangle([left, bottom]: [i32; 2], [right, top]: [i32; 2]) -> Vec<[i32; 2]> {
    vec![[left, bottom], [right, bottom], [right, top], [left, top]]
}

/// Square teeth on a base one thick, the rectilinear form of the comb in
/// `rcad/tests/shapes/mod.rs`.
fn comb(teeth: i32, tooth_height: i32) -> Vec<[i32; 2]> {
    let top = 1 + tooth_height;
    let span = 2 * teeth - 1;

    let mut corners = vec![[0, 0], [span, 0]];
    for tooth in (0..teeth).rev() {
        let left = 2 * tooth;
        corners.push([left + 1, top]);
        corners.push([left, top]);
        if tooth > 0 {
            corners.push([left, 1]);
            corners.push([left - 1, 1]);
        }
    }
    corners
}

fn whole(small: u32) -> i32 {
    i32::try_from(small).expect("a small whole number")
}
