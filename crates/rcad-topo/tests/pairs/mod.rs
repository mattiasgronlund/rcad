//! The families of solid configurations to fuse, and the exactly representable
//! movements some of the properties need.
//!
//! # Why several families and not one
//!
//! `docs/decisions.md` §12.3 is the section where one family of shapes found
//! one family of bugs, and §17.7 says what to do about it here: report the
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
//! - [`a_pair_somewhere_else`] is the **control written somewhere else in
//!   `f64`**. The same configurations with every coordinate multiplied by `2^k`
//!   and moved by `2^m`, both exact, so the answer has to be the same answer
//!   moved the same way, bit for bit. It found the two defects written up in
//!   `a_solid_a_long_way_from_the_origin.rs`.
//! - [`a_chain_of_fuses`] is **four solids instead of two**, so a fuse can be
//!   handed an answer three fuses old — which is the only place a corner with
//!   four or more planes round it, or a face joined out of coplanar pieces, ever
//!   turns up as an *input*.
//! - [`a_skew_pair`] is the **only pair that is not square to itself**. Each
//!   solid is leaned over by its own whole-number shear, so the corners stay
//!   exact while the walls stop being parallel between the two solids — which is
//!   the one way to put three cut lines through one point on a face.
//!
//! Each of the last three answers §14.1's four questions at its own strategy.
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

use std::ops::RangeInclusive;

use proptest::prelude::*;
use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, SolidKey};

use crate::cones::a_frustum;

/// Roughly how big a generated configuration is, for [`Scale`].
pub const WORKING_SIZE: f64 = 100.0;

/// How far a solid may be shifted along each axis.
///
/// Shapes reach about six across, so this puts a pair anywhere from wholly
/// apart to wholly inside one another.
const FURTHEST_SHIFT: i32 = 5;

/// The smallest power of two [`a_pair_somewhere_else`] writes a configuration
/// at.
///
/// **A limit and not a gap** (§16.10), and it is §12.2's precondition arriving
/// end to end. `orient3d` is a determinant of products of coordinates, and at
/// `2^k` those products go as `2^{2k}`; once they fall subnormal the relative
/// error model Shewchuk's filter is derived under stops holding, level 1
/// declines by design, and expansion arithmetic refuses to certify a product it
/// cannot represent exactly. So the fuse comes back `Undecidable` naming
/// `orient3d`, which is the honest answer and not a timid one.
///
/// Measured on this tree, two overlapping three-by-three blocks: the last scale
/// that answers is `-359` unshifted and `-332` shifted by `2^30`. Two hundred
/// and eighty leaves fifty powers of two of room for the larger shapes, and
/// `2^-280` is about `8e-85`, which is well past anything a relative bound has
/// ever been asked about here.
const SMALLEST_SCALE: i32 = -280;

/// The largest power of two [`a_pair_somewhere_else`] writes a configuration at.
///
/// **A limit and not a gap** (§16.10), and it is a long way out. A pair written
/// at `2^k` encloses about `2^{3k}` of space, so past `k = 339` six times the
/// volume is more than the largest `f64`, `signed_volume` overflows, and
/// `Model::validate` refuses a solid that encloses an infinite amount of space.
/// The answer stops existing in the number type; nothing about the arithmetic
/// gives up first.
///
/// Measured on this tree, two overlapping three-by-three blocks: the last scale
/// that answers is `339`, both unshifted and shifted by `2^30`. Two hundred and
/// eighty leaves fifty-nine powers of two of room, and matches
/// [`SMALLEST_SCALE`] at the other end.
///
/// This was `30` while `fuse/select.rs`'s `InPlane::of` stepped off a plane by
/// three *absolute* distances, which vanish into a plane whose offset is past
/// about `2^72` — see
/// `a_cube_a_long_way_from_the_origin_fuses_with_a_copy_of_itself` in
/// `crates/rcad-topo/tests/a_solid_a_long_way_from_the_origin.rs`. That was a
/// gap this family found, and the family could not reach past it until it was
/// fixed.
const LARGEST_SCALE: i32 = 280;

/// The furthest [`a_pair_somewhere_else`] moves a configuration from the origin,
/// as a power of two.
///
/// **The edge here is `DynamicRange`**, which refuses a model whose largest
/// coordinate is more than `1e11` times its smallest feature: with a smallest
/// feature of one that is `m = 37`. Thirty gives a spread of about `1e9`, which
/// is the number this family was always meant to reach.
///
/// It was `24` while `SolidView::signed_volume` summed `a · (b × c)` over
/// triangles measured from the **world origin**, which made a solid `2^m` away
/// add terms of size `2^{3m}` to reach a volume of order one and lose every
/// digit past `m = 28` — see
/// `a_frame_a_long_way_from_the_origin_encloses_what_a_frame_encloses`. The
/// tetrahedra now hang from a corner of the solid, so a frame's volume comes
/// back exactly eight out to `2^50`, and the gap between `1.7e7` and `1e9` that
/// this constant used to record is closed.
const FURTHEST_SHIFT_EXPONENT: u32 = 30;

/// How far [`a_skew_pair`] may lean an outline over.
///
/// Two, and the ceiling is what the lean does to a wall one thick. A lean of two
/// is the matrix `[[1, 2], [0, 1]]`, whose singular values are `2.414` and
/// `0.414`, so a [`Shape::Frame`]'s one-thick wall comes out about two fifths as
/// thick in the squeezed direction. That is still a wall — the determinant is
/// one, so nothing ever collapses — and it is deliberately near the sliver of
/// §7.4's sixth catalogue entry without being one.
const SHARPEST_SKEW: i32 = 2;

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

/// One solid: an outline, how the outline is leaned over, how far it is
/// extruded, and where it is put down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// What it is cut from.
    pub shape: Shape,
    /// How far the outline leans over before it is extruded.
    pub skew: Skew,
    /// How far it is extruded along `z`.
    pub height: u32,
    /// Where the outline's own origin ends up.
    pub at: [i32; 3],
}

/// How far an outline leans over, in whole numbers, in its own plane.
///
/// One slide: a corner moves along `x` by [`Skew::x_per_y`] for every step it
/// stands along `y`. As a matrix that is `[[1, x_per_y], [0, 1]]`, whose
/// determinant is `1` for every whole number, and a determinant of one is three
/// things at once:
///
/// - **The outline stays an outline.** An affine map takes a simple polygon to a
///   simple polygon, and a positive determinant keeps it anticlockwise, so a
///   leaning [`Shape`] never needs checking and never gets filtered. A hole
///   stays strictly inside its outline for the same reason.
/// - **Every corner stays a whole number.** Whole numbers in, whole numbers
///   out — which is what separates this family from [`a_turned_pair`], where the
///   very first coordinate is already irrational.
/// - **The volume does not change.** Leaning an extrusion over shears the prism,
///   so the answer encloses exactly what the lattice configuration it came from
///   encloses.
///
/// # Why one slide and not two
///
/// A second slide the other way — `y` by so much per step of `x` — was written,
/// measured and taken out again. It is the only way to leave a solid with *no*
/// wall along `(1, 0)`, so on paper it is the only way for two solids to share
/// no wall direction at all, which sounds like exactly what this family wants.
///
/// Measured, it does not do that. It squeezes. Composing the two slides gives
/// `[[1, a], [b, 1 + ab]]`, and at `a = b = 2` the singular values are `5.83`
/// and `0.17` — a squeeze of 34 to 1, against 5.83 to 1 for one slide of two.
/// A grid of refusal rates over every pair of leans settles which of the two
/// effects the number is reporting: a solid leaned `[1, 1]` against a flat one
/// refuses 39.6% of the time, and against *another leaned exactly the same way*
/// — which is a pair square to itself, an affine image of a lattice pair —
/// 41.7%. The same number. **The second slide measures how badly one solid is
/// squeezed and not how the two sit against each other**, so it would have
/// bought a headline rate that mostly reports slivers.
///
/// [`Skew::none`] is the identity, which is what every other family uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skew {
    /// How far a corner slides along `x` for each step along `y`.
    pub x_per_y: i32,
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

/// A [`Rigid`] as the model's own coordinates see it.
///
/// The same quarter turn, and the shift multiplied by whatever power of two the
/// configuration is written at. Built by [`Rigid::in_the_world_of`] and used to
/// move an *answer*, where the coordinates are the model's rather than the
/// generator's whole numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Movement {
    /// How many quarter turns about `z`, anticlockwise.
    pub quarters: u32,
    /// How far to move afterwards, along each axis, in the model's own numbers.
    pub shift: [f64; 3],
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

/// Where in `f64` a configuration is built, as against where in space.
///
/// Two knobs, both exactly representable, both applied to every coordinate of
/// every solid: multiply by `2^scale`, and move by `2^shift` along each axis.
/// A power of two times a whole number is exact until it underflows, and a
/// power of two plus a small whole number is exact while both fit in 53 bits,
/// so a configuration built [`Elsewhere`] is the same configuration written in
/// different `f64` and its answer must be the same answer written the same way.
///
/// [`Elsewhere::here`] is the one every other family uses: no scaling and no
/// shift, so the numbers are the small whole numbers `Shape` makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Elsewhere {
    /// The power of two every coordinate is multiplied by.
    pub scale: i32,
    /// The power of two every coordinate is moved by along each axis, if any.
    /// `None` leaves the configuration where the lattice family puts it.
    pub shift: Option<u32>,
}

/// A whole configuration: the solids, the one rotation applied to all of
/// them, and where in `f64` it is written.
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
    /// Where in `f64` the whole thing is written.
    pub elsewhere: Elsewhere,
}

/// Two rectilinear solids on the integer grid, near enough to interact.
pub fn a_lattice_pair() -> impl Strategy<Value = Configuration> {
    lattice_solids(2).prop_map(|solids| Configuration {
        solids,
        turn: None,
        elsewhere: Elsewhere::here(),
    })
}

/// Three of them, for the property that fusing all three at once agrees with
/// fusing two and then the third.
pub fn a_lattice_trio() -> impl Strategy<Value = Configuration> {
    lattice_solids(3).prop_map(|solids| Configuration {
        solids,
        turn: None,
        elsewhere: Elsewhere::here(),
    })
}

/// The lattice pairs under one general rotation, which leaves nothing about them
/// exactly representable.
pub fn a_turned_pair() -> impl Strategy<Value = Configuration> {
    (lattice_solids(2), a_turn()).prop_map(|(solids, turn)| Configuration {
        solids,
        turn: Some(turn),
        elsewhere: Elsewhere::here(),
    })
}

/// The lattice pairs, each solid leaned over by its own whole-number shear.
///
/// The **only family in which the two solids are not axis-aligned to each
/// other**. Every other family turns both solids by the same rotation, so
/// whatever the world coordinates look like the pair stays as square to itself
/// as the lattice configuration it came from. Here the two shears differ, and
/// that is the whole point.
///
/// # What no other family can express
///
/// A face of an answer is cut by lines: where the other solid's planes meet the
/// face, and the face's own boundary. In every family that exists today both
/// solids' wall planes have normals drawn from **one** set of three orthogonal
/// directions — the axes, or the axes under the one shared rotation. A wall
/// plane meets a face plane in a line, and with three orthogonal normals in play
/// the lines on any one face come in at most **two** directions.
///
/// Three distinct lines through one point need three distinct directions, so
/// **no configuration in the corpus has ever put three concurrent cut lines on a
/// face.** A leaning solid keeps its walls along `(1, 0)` and tilts the rest to
/// `(x_per_y, 1)`, so a pair leaning by two different amounts has walls in three
/// directions instead of two. That reaches the arrangement the others
/// structurally cannot: the vertex where three cut lines meet, which is where a
/// face's arrangement has to settle an ordering that is a tie rather than a
/// comparison.
///
/// # §14.1's four questions
///
/// 1. **Exactly true, or needing a bound?** Exactly true for what is asserted,
///    and the reason is worth being precise about, because this family sits
///    between the two kinds the file already has.
///
///    Every **corner** that goes in is a whole number: [`Skew::corner`] is whole
///    numbers to whole numbers, and so are `at`, the quarter turn and the lift.
///    Every **horizontal** plane is exact too — a cap's normal is `(0, 0, ±1)`
///    and its offset a small whole number.
///
///    Every **tilted wall** plane is not, and cannot be. `build_side_face` takes
///    the wall normal as `(along_y, -along_x, 0)` and
///    `Plane::from_point_and_normal` normalizes it, and the only exactly
///    representable unit normals are the signed axis directions. That is a
///    proof and not a hope: an `f64` is a dyadic rational, so a unit normal with
///    `f64` components gives whole numbers with `p₁² + p₂² + p₃² = 4^k`; squares
///    are `0` or `1` mod `4`, so all three have to be even, so every solution
///    descends to one of `4^{k-1}`, and the descent bottoms out at
///    `1 = 1 + 0 + 0`. Only `(±2^k, 0, 0)` and its permutations survive.
///
///    So what runs here is what is exactly true whatever the coefficients are:
///    the two runs of one question agreeing bit for bit, and every plane of an
///    answer being a plane of an input. **Inclusion and exclusion is not
///    asserted**, for the same reason §17.8 gives for turned pairs — the corners
///    the fuse invents are meets of tilted planes and a bound on the sum of the
///    four volumes needs a bound on how badly conditioned those triples are,
///    which nothing in this tree has. **Neither is transform invariance**, and
///    that one is specific to this family: [`Movement::plane`] slides a plane
///    along its own normal by `n · t`, which is exact when `n` is a signed unit
///    axis and a rounding otherwise, so the property would be comparing the
///    generator's arithmetic against the kernel's rather than the kernel against
///    itself.
/// 2. **Do the two runs take different paths?** Yes, and further apart than
///    swapping the sides usually manages. The lines cutting a face arrive in
///    three or four directions instead of two, so the order the splitter meets
///    them in differs between the two runs at every concurrent vertex — which is
///    exactly the tie a partition can resolve two ways and still be right about
///    the solid. §17.4 settles that a splitter which cut by one line and then
///    cut the pieces by the next would compute a genuinely different partition
///    each way round; until now no generated input could make three lines meet
///    to test it.
/// 3. **What family of inputs?** The lattice configurations, one shear per
///    solid, drawn after them so the shapes are the shapes. Nothing is added to
///    [`a_shape`] — §17.7's recorded figures run a fixed seed over it, and the
///    handoff rule is that new shapes go in new strategies with rows of their
///    own.
/// 4. **Which gap would this have caught?** §17.8's headline lesson with the
///    numbers on it: three million fuses over the shipped generators found
///    nothing in 934 seconds, and one new shape family found a defect in 0.02
///    seconds and under forty fuses. That section names the mechanism —
///    "the corpus that has already run is worth almost nothing; the shape nobody
///    has drawn yet is worth almost everything" — and names the family of
///    arrangement the corpus could not draw. This is the cheapest shape that
///    changes the *arrangement* rather than the numbers: one integer shear per
///    solid, no new outline, no new arithmetic on the way in.
pub fn a_skew_pair() -> impl Strategy<Value = Configuration> {
    (lattice_solids(2), a_skew(), a_skew()).prop_map(|(solids, first, second)| Configuration {
        solids: solids
            .into_iter()
            .zip([first, second])
            .map(|(placed, skew)| Placed { skew, ..placed })
            .collect(),
        turn: None,
        elsewhere: Elsewhere::here(),
    })
}

/// The same pairs with both solids leaning the *same* way, which is
/// [`a_skew_pair`]'s control.
///
/// A shear is affine, so leaning both solids of a lattice pair over by the same
/// amount is an affine image of that pair: same shapes, same overlaps, same
/// touchings, same arrangement of cut lines on every face, corner for corner.
/// The lattice family decides all of that and refuses **zero**. What is left
/// between the two rows is one thing and one thing only — the walls are on
/// tilted planes, whose unit normals are rounded rather than exact.
///
/// So this row prices the tilt on its own, and [`a_skew_pair`]'s row minus this
/// one is what the *arrangement* costs. §17.7's control row exists to make
/// exactly that subtraction, and without it the skew family's number would be
/// two effects added together with no way to tell which is which.
pub fn a_pair_leaning_the_same_way() -> impl Strategy<Value = Configuration> {
    (lattice_solids(2), a_skew()).prop_map(|(solids, skew)| Configuration {
        solids: solids
            .into_iter()
            .map(|placed| Placed { skew, ..placed })
            .collect(),
        turn: None,
        elsewhere: Elsewhere::here(),
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
                    skew: Skew::none(),
                    height,
                    at: [0, 0, 0],
                },
                Placed {
                    shape,
                    skew: Skew::none(),
                    height,
                    at,
                },
            ],
            turn: None,
            elsewhere: Elsewhere::here(),
        }
    })
}

/// One solid and a second one built from the same numbers in the same place.
///
/// Two solids rather than one solid twice, because a fuse names its two sides by
/// key and the same key on both sides would be a different question.
pub fn a_solid_and_its_copy() -> impl Strategy<Value = Configuration> {
    a_copied_solid().prop_map(|solids| Configuration {
        solids,
        turn: None,
        elsewhere: Elsewhere::here(),
    })
}

/// The same under a general rotation, where the two solids still share every
/// coordinate because they are turned by the very same rotation.
pub fn a_turned_solid_and_its_copy() -> impl Strategy<Value = Configuration> {
    (a_copied_solid(), a_turn()).prop_map(|(solids, turn)| Configuration {
        solids,
        turn: Some(turn),
        elsewhere: Elsewhere::here(),
    })
}

/// Four lattice solids, for a chain of fuses deeper than any other family
/// reaches.
///
/// # §14.1's four questions
///
/// 1. **Exactly true, or needing a bound?** Exactly true. Everything here is the
///    lattice family's arithmetic — axis-aligned planes, whole-number corners,
///    Cramer solves with a determinant of ±1 — so `fuse([A,B,C,D])` and uniting
///    the four one at a time are two exact computations of one answer, and the
///    properties compare volume, area, centroid and shell count with no
///    tolerance anywhere.
/// 2. **Do the two runs take different paths?** Yes, and further than anything
///    at depth two does, because the fourth fuse's *inputs* have properties no
///    generated input ever has. A corner of the third answer can have four or
///    more planes round it, which is the only way to reach
///    `bake::derive_a_corner`'s over-determined branch and the
///    `Invalid::VertexOffItsFacePlane` refusal that hangs off it. Faces of that
///    answer can be joined out of coplanar pieces of two different inputs. And
///    its shells came out of `nest_the_shells` rather than out of
///    `add_extrusion`. §17.8 records that associativity found the coplanar-corner
///    hole at depth two and that §17.1's argument is about depth three and four;
///    the oracle that found it had never run that deep.
/// 3. **What family of inputs?** The lattice shapes, four at a time instead of
///    two or three. Nothing is added to [`a_shape`] — §17.7's recorded figures
///    run a fixed seed over it.
/// 4. **Which gap would this have caught?** §17.8's, the one it says the
///    catalogue could not see: "a corner between two coplanar faces has two
///    planes round it and not three, so the *next* fuse over such an answer
///    divides by a determinant of exactly zero". That was found by a depth-two
///    property and fixed; nothing in the tree then went deeper to see whether
///    depth three and four have their own version of it.
pub fn a_chain_of_fuses() -> impl Strategy<Value = Configuration> {
    lattice_solids(4).prop_map(|solids| Configuration {
        solids,
        turn: None,
        elsewhere: Elsewhere::here(),
    })
}

/// One turned solid and two to four more built from the same numbers in the
/// same place.
///
/// Every copy carries bit-identical planes, so uniting them one at a time has to
/// come back bit-identical from the second union on — exactly true even under a
/// general rotation, because no arithmetic anywhere has two different numbers to
/// disagree about. The first union is where the corners a solid cached get
/// re-derived from the planes they are samples of, which `docs/decisions.md`
/// §17.1 asks for and which is allowed to move them an ulp.
pub fn a_turned_solid_and_several_copies() -> impl Strategy<Value = Configuration> {
    (a_shape(), 1u32..=3, 2usize..=4, a_turn()).prop_map(|(shape, height, copies, turn)| {
        let placed = Placed {
            shape,
            skew: Skew::none(),
            height,
            at: [0, 0, 0],
        };
        Configuration {
            solids: vec![placed; copies + 1],
            turn: Some(turn),
            elsewhere: Elsewhere::here(),
        }
    })
}

/// The lattice pairs, written somewhere else in `f64`.
///
/// The same configurations [`a_lattice_pair`] makes, with every coordinate
/// multiplied by `2^k` and moved by `2^m` along each axis. Nothing about the
/// geometry changes; everything about the numbers does.
///
/// # §14.1's four questions
///
/// 1. **Exactly true, or needing a bound?** Exactly true, with no bound
///    anywhere. The shift is added to the outline's corners as whole numbers,
///    and `2^m` plus a coordinate under thirty-two is under `2^25`, which an
///    `f64` holds exactly with twenty-eight bits to spare. Multiplying by `2^k`
///    only moves an exponent, so it is exact until it underflows, and
///    [`SMALLEST_SCALE`] stops well above where that starts. So the answer to a
///    fuse over one of these has to be the `k = 0, m = 0` answer with the same
///    movement applied, **bit for bit**, and there is no tolerance to tune.
/// 2. **Do the two runs take different paths?** Yes, and it is measured rather
///    than argued. Two overlapping three-by-three blocks answer down to
///    `k = -359` unshifted and only down to `k = -332` when they are shifted by
///    `2^30` — so the shift changes what the arithmetic can settle, which it
///    could not do if every size went down the same branch. (The upper end no
///    longer tells them apart: both answer up to `k = 339`, where the volume
///    overflows. While `InPlane::of` stepped off a plane by an absolute
///    distance they parted at `70` and `42`, and that difference was the
///    defect.) What the low end
///    walks toward is §12.2's precondition exactly: below its edge the fuse
///    refuses naming `orient3d`, which is the level-1 filter declining because
///    its products have fallen subnormal and expansion arithmetic then refusing
///    to certify a product it cannot hold. And [`Policy`] derives its budget
///    from the working size and from what a predicate `Measures`, so a predicate
///    carrying the wrong power of that size (§12.4) can only show itself at a
///    size other than one.
/// 3. **What family of inputs?** Old shapes, new numbers — the cheapest form
///    §13 rule 5 takes, and deliberately not a fourth shape family, because what
///    is being varied here is the `f64` and not the geometry. Nothing is added
///    to [`a_shape`]: §17.7's recorded figures run a fixed seed over it, and
///    changing it would silently change numbers written into `docs/decisions.md`.
/// 4. **Which gap would this have caught?** §12.4's, and by name. That section
///    records an `Approximation` magnitude carrying the units of the wrong
///    predicate degree, and says outright that **no end-to-end outcome flip was
///    ever built** — the bug was "real, latent, and would have become reachable
///    the moment a second predicate took constructed input". This is that
///    end-to-end flip: the same configuration decided at two sizes, where a
///    budget derived from the working size in the wrong power lands on opposite
///    sides of the same question.
///
/// # Where the edges are, and what kind of edge each one is
///
/// Both ends of `k` were measured rather than guessed, and both are now
/// **limits** (§16.10) — the answer stops existing in `f64` and the kernel says
/// so: products fall subnormal at the bottom, the volume overflows at the top.
/// The top used to be a **gap**, and finding it is what this family was for.
/// [`SMALLEST_SCALE`], [`LARGEST_SCALE`] and [`FURTHEST_SHIFT_EXPONENT`] carry
/// the measurements and the reasoning one each.
pub fn a_pair_somewhere_else() -> impl Strategy<Value = Configuration> {
    (lattice_solids(2), an_elsewhere()).prop_map(|(solids, elsewhere)| Configuration {
        solids,
        turn: None,
        elsewhere,
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
        let mut model =
            Model::new(Scale::millimetres(self.elsewhere.working_size()).expect("a working size"));
        let keys: Vec<SolidKey> = self
            .solids
            .iter()
            .map(|placed| placed.built_after(movement, self.elsewhere, &mut model))
            .collect();
        match self.turn {
            Some(turn) => turned_by((model, keys), turn),
            None => (model, keys),
        }
    }
}

/// Every solid of a built configuration turned by the same rotation.
///
/// The one place a [`Turn`] is applied, for the flat families and the curved
/// ones alike, so that a curved configuration turned by a [`Turn`] is turned
/// by exactly the arithmetic a turned flat one is.
fn turned_by((mut model, keys): (Model, Vec<SolidKey>), turn: Turn) -> (Model, Vec<SolidKey>) {
    let turn = turn.isometry();
    for &key in &keys {
        model
            .place_solid(key, &turn)
            .expect("this model made the key");
    }
    (model, keys)
}

impl Placed {
    /// Build this solid, `movement` applied to its whole numbers and
    /// `elsewhere` applied to the `f64` that come out of them.
    ///
    /// The shift goes on before the quarter turn, which is what keeps the
    /// world-coordinate movement a quarter turn and a shift of `2^scale` times
    /// a whole number rather than something with the shift's own image in it.
    /// [`Rigid::in_the_world_of`] is the other half of that arrangement.
    ///
    /// The [`Skew`] goes on before all of it, and it goes on **after `at`**
    /// rather than to the outline's own corners. That ordering is what makes
    /// [`a_pair_leaning_the_same_way`] a control: leaning a solid's outline and
    /// then putting it down somewhere is `S(ring) + at`, which is not the image
    /// of anything, while leaning the solid where it stands is
    /// `S(ring + at) = S(ring) + S(at)` — so two solids leaning by the same
    /// amount are the one affine image `S` of the whole lattice pair, corner for
    /// corner. Get it the wrong way round and the control still runs, still
    /// reports a number, and no longer prices what it says it prices.
    ///
    /// Every step here is whole numbers to whole numbers, so the corners that
    /// reach [`ProfileBuilder`] are exact however far the outline leans, and a
    /// shrink toward [`Skew::none`] lands on the very lattice configuration the
    /// case came from.
    fn built_after(&self, movement: &Rigid, elsewhere: Elsewhere, model: &mut Model) -> SolidKey {
        let rings = self.shape.rings();
        let away = elsewhere.shift_along_each_axis();
        let flat = |ring: &[[i32; 2]]| -> Vec<[f64; 2]> {
            ring.iter()
                .map(|&[x, y]| {
                    let [x, y] = self.skew.corner([x + self.at[0], y + self.at[1]]);
                    let [x, y, _] = movement.whole_point([x + away, y + away, 0]);
                    [elsewhere.scaled(x), elsewhere.scaled(y)]
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
        let lift = Vector::new(
            0.0,
            0.0,
            elsewhere.scaled(self.at[2] + away + movement.shift[2]),
        );
        model
            .add_extrusion(
                &profile,
                elsewhere.scaled(whole(self.height)),
                &Isometry::translation(lift).expect("a whole-number lift"),
            )
            .expect("every generated profile extrudes")
    }
}

impl Skew {
    /// The shear that leaves an outline where it is.
    #[must_use]
    pub fn none() -> Self {
        Self { x_per_y: 0 }
    }

    /// Whether this shear leaves the outline alone.
    #[must_use]
    pub fn is_none(self) -> bool {
        self == Self::none()
    }

    /// Where a whole-number corner ends up, still a whole number.
    #[must_use]
    pub fn corner(self, [x, y]: [i32; 2]) -> [i32; 2] {
        [x + self.x_per_y * y, y]
    }
}

impl Elsewhere {
    /// Where the lattice family builds: no scaling, no shift.
    #[must_use]
    pub fn here() -> Self {
        Self {
            scale: 0,
            shift: None,
        }
    }

    /// How far every coordinate is moved along every axis. Zero, or a power of
    /// two.
    #[must_use]
    pub fn shift_along_each_axis(self) -> i32 {
        self.shift.map_or(0, |exponent| 1 << exponent)
    }

    /// A whole number as this configuration writes it: `whole * 2^scale`.
    ///
    /// Exact. Multiplying by a power of two only moves the exponent, and
    /// [`SMALLEST_SCALE`] keeps the result out of the subnormals where that
    /// stops being true.
    #[must_use]
    pub fn scaled(self, whole: i32) -> f64 {
        f64::from(whole) * two_to_the(self.scale)
    }

    /// The nominal size to build a [`Scale`] from: [`WORKING_SIZE`] moved by
    /// the same power of two everything else is.
    #[must_use]
    pub fn working_size(self) -> f64 {
        WORKING_SIZE * two_to_the(self.scale)
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

    /// The same movement as the world coordinates of `elsewhere` see it.
    ///
    /// [`Rigid::shift`] is a whole number of lattice steps and the lattice is
    /// multiplied by `2^scale` on the way into the model, so in the model's own
    /// coordinates the shift is that whole number times `2^scale` — a power of
    /// two times a small whole number, which is exact, so the claim stays a
    /// bit-for-bit one. The quarter turn does not change at all: a swap and a
    /// negation is the same map at every size.
    ///
    /// The shift `elsewhere` itself applies does not appear here. It goes on
    /// *before* the quarter turn in [`Placed::built_after`], so the turn carries
    /// it along with the shape and it cancels out of the difference between the
    /// two runs.
    #[must_use]
    pub fn in_the_world_of(&self, elsewhere: Elsewhere) -> Movement {
        Movement {
            quarters: self.quarters,
            shift: self.shift.map(|step| elsewhere.scaled(step)),
        }
    }

    /// Where a whole-number point ends up, still a whole number.
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

impl Movement {
    /// Where a point ends up. Exact, whatever the coordinates are.
    #[must_use]
    pub fn point(&self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        let [x, y, z] = self.direction([x, y, z]);
        [x + self.shift[0], y + self.shift[1], z + self.shift[2]]
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
    /// normal that dot product is one term and two zeroes, and that one term is
    /// a power of two times a small whole number, so it is exact — which is the
    /// second half of why these families are rectilinear.
    #[must_use]
    pub fn plane(&self, [a, b, c, d]: [f64; 4]) -> [f64; 4] {
        let [a, b, c] = self.direction([a, b, c]);
        let along = a * self.shift[0] + b * self.shift[1] + c * self.shift[2];
        [a, b, c, d - along]
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
                skew: Skew::none(),
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
            skew: Skew::none(),
            height,
            at: [0, 0, 0],
        };
        vec![placed, placed]
    })
}

/// Two whole-number exponents, each shrinking toward where the lattice family
/// already builds.
///
/// `scale` shrinks toward `0`, which is a multiplier of one, and `shift` shrinks
/// toward `None`, which is no move at all. So a failure shrinks to "an existing
/// lattice configuration, and it fails at `2^-300`" rather than to a heap of
/// coordinates nobody can read.
fn an_elsewhere() -> impl Strategy<Value = Elsewhere> {
    (
        SMALLEST_SCALE..=LARGEST_SCALE,
        proptest::option::of(0u32..=FURTHEST_SHIFT_EXPONENT),
    )
        .prop_map(|(scale, shift)| Elsewhere { scale, shift })
}

/// `2^exponent`, exactly.
///
/// Assembled from the bits rather than through `powi`, which is repeated
/// multiplication and rounds on the way to a far exponent.
///
/// # Panics
///
/// Outside the normal exponent range, which [`SMALLEST_SCALE`] and
/// [`LARGEST_SCALE`] keep the generator well inside.
fn two_to_the(exponent: i32) -> f64 {
    let biased = u64::try_from(1023 + exponent).expect("a normal binary exponent");
    assert!(biased < 2047, "a normal binary exponent, not {exponent}");
    f64::from_bits(biased << 52)
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

/// How far one outline leans over, shrinking toward not leaning at all.
///
/// So a failing case shrinks to "this lattice configuration, with one solid
/// leaned over by one", which is a thing a person can draw.
fn a_skew() -> impl Strategy<Value = Skew> {
    (-SHARPEST_SKEW..=SHARPEST_SKEW).prop_map(|x_per_y| Skew { x_per_y })
}

/// A block with a round bar driven through it, a whole unit clear of every
/// wall.
///
/// **A family of its own rather than an arm of [`Shape`]**, and deliberately so:
/// adding a curved outline to `a_shape()` would change the corpus every measured
/// family above draws from, and every number §17.7 and §22 record with it. This
/// generator shares nothing with them.
///
/// The bar is kept clear of the block's walls because §36.14 measured that
/// arrangement at nothing refused, and a metamorphic property is only worth what
/// it compares — a family the fuse refuses outright leaves every property here
/// green having asserted nothing (§14.7). The degenerate arrangements a bar
/// tangent to a wall or flush with a cap reaches are roadmap §7.4's catalogue
/// entries, and `degenerate_curved_fuses.rs` is where they are hand-written one
/// at a time.
///
/// Every number is a small whole one, so the block's corners and the bar's
/// radius and seam are exact. What is **not** exact is any corner where the
/// bar's wall meets a wall of the block, and that is the point of running the
/// properties over this family at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurvedPair {
    /// How wide the block is, and how tall.
    pub side: u32,
    /// The bar's radius.
    pub radius: u32,
    /// Where the bar's axis stands, in the block's own coordinates.
    pub at: [u32; 2],
}

impl CurvedPair {
    /// Build the block and then the bar, in that order.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let side = f64::from(self.side);
        let block = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a whole-number cube");
        let bar = model
            .add_round_bar(
                f64::from(self.radius),
                side + 4.0,
                &Isometry::translation(Vector::new(
                    f64::from(self.at[0]),
                    f64::from(self.at[1]),
                    -2.0,
                ))
                .expect("a whole-number shift"),
            )
            .expect("a bar long enough to go right through");
        (model, vec![block, bar])
    }

    /// The same bar built twice in the same place, and no block.
    ///
    /// What `A \ A` is asked of. Both copies come out of the same numbers by the
    /// same code, so they are bit-identical rather than nearly so.
    #[must_use]
    pub fn built_as_a_bar_and_a_copy_of_it(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let mut bar = || {
            model
                .add_round_bar(
                    f64::from(self.radius),
                    f64::from(self.side),
                    &Isometry::identity(),
                )
                .expect("a whole-number bar")
        };
        let one = bar();
        let copy = bar();
        (model, vec![one, copy])
    }

    /// The same block built twice in the same place, and no bar.
    ///
    /// The control for [`CurvedPair::built_as_a_bar_and_a_copy_of_it`]: the same
    /// question with every face flat, which the flat families already answer, so
    /// a difference between the two rows is the curve and nothing else.
    #[must_use]
    pub fn built_as_a_block_and_a_copy_of_it(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let mut cube = || {
            model
                .add_cube(
                    f64::from(self.side),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a whole-number cube")
        };
        let one = cube();
        let copy = cube();
        (model, vec![one, copy])
    }
}

/// A block with a bar through it, the bar clear of every wall by at least one.
///
/// Shrinks toward a block of side four with a bar of radius one up its middle.
pub fn a_curved_pair() -> impl Strategy<Value = CurvedPair> {
    (4u32..=8)
        .prop_flat_map(|side| (Just(side), 1u32..=(side / 2 - 1).min(2)))
        .prop_flat_map(|(side, radius)| {
            // A whole unit of room between the bar and every wall, at both ends
            // of the range, so no drawn pair is tangent to anything.
            let room = radius + 1..=side - radius - 1;
            (Just(side), Just(radius), [room.clone(), room])
                .prop_map(|(side, radius, at)| CurvedPair { side, radius, at })
        })
}

/// A block with a ball sitting half in it, centred on the block's floor.
///
/// The second curved family, and the first one that is not a pipe. Where the
/// ball's skin meets the floor is a circle in the plane `z = 0`, so a union
/// leaves a dome on top of the block and a shared answer is half a ball.
///
/// The centre is a whole-number point on the floor and the radius is a whole
/// number, so both are exact. What is **not** exact is any corner where the
/// ball's skin meets a wall of the block — the same thing [`CurvedPair`] is for,
/// on a surface that curves both ways instead of one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BallAndBlock {
    /// How wide the block is, and how tall.
    pub side: u32,
    /// The ball's radius.
    pub radius: u32,
    /// Where the ball's centre stands on the floor, in the block's coordinates.
    pub at: [u32; 2],
}

impl BallAndBlock {
    /// Build the block and then the ball, in that order.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let block = model
            .add_cube(
                f64::from(self.side),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a whole-number cube");
        let ball = model
            .add_ball(
                f64::from(self.radius),
                &Isometry::translation(Vector::new(
                    f64::from(self.at[0]),
                    f64::from(self.at[1]),
                    0.0,
                ))
                .expect("a whole-number shift"),
            )
            .expect("a whole-number ball");
        (model, vec![block, ball])
    }

    /// The same ball built twice in the same place, and no block.
    ///
    /// What `A \ A` is asked of, and the counterpart of
    /// [`CurvedPair::built_as_a_bar_and_a_copy_of_it`]. Both copies come out of
    /// the same numbers by the same code, so they are bit-identical rather than
    /// nearly so.
    #[must_use]
    pub fn built_as_a_ball_and_a_copy_of_it(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let mut ball = || {
            model
                .add_ball(f64::from(self.radius), &Isometry::identity())
                .expect("a whole-number ball")
        };
        let one = ball();
        let copy = ball();
        (model, vec![one, copy])
    }
}

/// A block with a ball half sunk into its floor, the ball clear of every side
/// wall by at least one and never taller than the block.
///
/// Shrinks toward a block of side four with a ball of radius one on the middle
/// of its floor.
pub fn a_ball_and_a_block() -> impl Strategy<Value = BallAndBlock> {
    (4u32..=8)
        .prop_flat_map(|side| (Just(side), 1u32..=(side / 2 - 1).min(2)))
        .prop_flat_map(|(side, radius)| {
            // A whole unit of room between the ball and every side wall, so no
            // drawn pair is tangent to anything.
            let room = radius + 1..=side - radius - 1;
            (Just(side), Just(radius), [room.clone(), room])
                .prop_map(|(side, radius, at)| BallAndBlock { side, radius, at })
        })
}

/// Two balls whose skins cross in one circle: whole-number radii, apart along
/// `z` by a whole number strictly between the difference of the two radii and
/// their sum, so neither nests inside the other and neither stands clear.
///
/// **The only family here with two round faces and no flat one.** Every other
/// curved pair puts a ball or a bar against a block's flat wall, where the
/// fuse's selection table takes a Plane/Sphere or Plane/Cylinder row; here both
/// faces carry a [`rcad_geom::Surface::Sphere`] instead.
///
/// **This plain a pair does not reach `fuse/assemble.rs`'s `[Sphere, Sphere]`
/// row of `which_arc_of_a_circle_walked_this_way`, though.** That row is read
/// off the crossing circle's own two corner names
/// (`fuse/select.rs`'s `SidesItShares::name_the_sides`, the one caller that can
/// reach it), and `name_the_sides` only runs once `fuse/select.rs`'s point
/// search cannot place a piece on its own. For two whole balls every piece —
/// the lens and the two crescents — is a plain volume a point inside it decides
/// at once, so the point search places all of them and `name_the_sides` is
/// never called: an `eprintln!` at its one call site printed nothing across
/// 213 fuses (71 drawn pairs, union/intersection/difference) that
/// [`two_balls_crossing_in_a_circle`] drew. Reaching that row needs a third
/// solid or tool that leaves at least one piece the point search cannot place
/// on its own — not a plain pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TwoBalls {
    /// The first ball's radius, centred at the origin.
    pub first: u32,
    /// The second ball's radius.
    pub second: u32,
    /// How far the second ball's centre stands from the first, along `z`.
    pub apart: u32,
}

impl TwoBalls {
    /// Build the first ball at the origin and the second `apart` up `z`.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let first = model
            .add_ball(f64::from(self.first), &Isometry::identity())
            .expect("a whole-number ball");
        let second = model
            .add_ball(
                f64::from(self.second),
                &Isometry::translation(Vector::new(0.0, 0.0, f64::from(self.apart)))
                    .expect("a whole-number shift"),
            )
            .expect("a whole-number ball");
        (model, vec![first, second])
    }
}

/// Two balls of whole-number radii, apart just far enough to cross in one
/// circle and not far enough to stand clear or to nest one inside the other.
///
/// A circle survives between `|first − second|` and `first + second` apart, so
/// `apart` is drawn strictly inside that range: both ends are excluded because
/// the near one is where the smaller ball sits entirely inside the larger —
/// its own fixed degenerate case, not this family's — and the far one is where
/// the two stand tangent rather than crossing.
///
/// Shrinks toward two balls of radius one, their centres one apart.
pub fn two_balls_crossing_in_a_circle() -> impl Strategy<Value = TwoBalls> {
    (1u32..=6, 1u32..=6)
        .prop_flat_map(|(first, second)| {
            let closest = first.abs_diff(second) + 1;
            let farthest = first + second - 1;
            (Just(first), Just(second), closest..=farthest)
        })
        .prop_map(|(first, second, apart)| TwoBalls {
            first,
            second,
            apart,
        })
}

/// A block with a ball half sunk into its floor and a round bar driven through
/// it, the ball and the bar well apart from each other.
///
/// **The first curved fixture that is not a pair**, and it exists because
/// associativity needs three solids: `(A ∪ B) ∪ C` against `A ∪ (B ∪ C)` cannot
/// be asked of two. Every other curved family here is a pair, so until this one
/// no algebraic law needing a trio had ever run over a round face at all.
///
/// # Why the ball and the bar never touch
///
/// Because a sphere crossing a pipe is a quartic, and nothing in this kernel can
/// put points in order along one — `docs/decisions.md` §48.2 item 4, and
/// `Error::ACrossingThisCannotBeOrderedAlong` is what comes back. One grouping
/// of the trio fuses the ball against the bar directly, so a fixture where those
/// two overlap would refuse on that grouping every time and leave the law green
/// having compared nothing (§14.7).
///
/// Kept apart, both curved solids still meet the **block**, which is where every
/// crossing the fuse has to arrange comes from: a circle where the ball's skin
/// cuts the floor, and two circles where the bar's wall cuts the floor and the
/// roof. That is a real curved arrangement in every grouping, not an easy one
/// bought by moving the round faces out of each other's way.
///
/// # What it does today
///
/// Every pair of the three fuses. **No trio of them does**, and
/// `fuses_that_must_agree.rs`'s
/// `uniting_a_ball_and_a_bar_with_a_block_does_not_depend_on_which_two_go_first`
/// pins the refusal and says which face stops it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BallBarAndBlock {
    /// How wide the block is, and how tall.
    pub side: u32,
    /// The ball's radius.
    pub ball_radius: u32,
    /// The bar's radius.
    pub bar_radius: u32,
}

impl BallBarAndBlock {
    /// Build the block, then the ball, then the bar, in that order.
    ///
    /// The ball stands a unit clear of one side wall and the bar a unit clear of
    /// the far one, both on the line halfway across the block, which is what
    /// keeps them apart without any number having to be checked at build time.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let side = f64::from(self.side);
        let halfway = f64::from(self.side / 2);
        let block = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a whole-number cube");
        let ball = model
            .add_ball(
                f64::from(self.ball_radius),
                &Isometry::translation(Vector::new(self.ball_stands_at(), halfway, 0.0))
                    .expect("a whole-number shift"),
            )
            .expect("a whole-number ball");
        let bar = model
            .add_round_bar(
                f64::from(self.bar_radius),
                side + 4.0,
                &Isometry::translation(Vector::new(self.bar_stands_at(), halfway, -2.0))
                    .expect("a whole-number shift"),
            )
            .expect("a bar long enough to go right through");
        (model, vec![block, ball, bar])
    }

    /// How far along `x` the ball's centre stands, a unit clear of the near wall.
    #[must_use]
    pub fn ball_stands_at(&self) -> f64 {
        f64::from(self.ball_radius + 1)
    }

    /// How far along `x` the bar's axis stands, a unit clear of the far wall.
    #[must_use]
    pub fn bar_stands_at(&self) -> f64 {
        f64::from(self.side - self.bar_radius - 1)
    }
}

/// A block with a ball in its floor and a bar through it, everything a whole
/// unit clear of everything else.
///
/// The block is drawn wide enough to hold both with a unit of room at every
/// wall and a unit of room between the two round solids, which is
/// `2·ball + 2·bar + 3` at its narrowest.
///
/// Shrinks toward a block of side seven with a ball of radius one against one
/// wall and a bar of radius one against the other.
pub fn a_ball_a_bar_and_a_block() -> impl Strategy<Value = BallBarAndBlock> {
    (1u32..=2, 1u32..=2)
        .prop_flat_map(|(ball_radius, bar_radius)| {
            let narrowest = 2 * ball_radius + 2 * bar_radius + 3;
            (
                Just(ball_radius),
                Just(bar_radius),
                narrowest..=narrowest + 4,
            )
        })
        .prop_map(|(ball_radius, bar_radius, side)| BallBarAndBlock {
            side,
            ball_radius,
            bar_radius,
        })
}

/// A block with a frustum standing up through its floor.
///
/// The third curved family, and the first whose surface is neither a pipe nor a
/// ball. A cone is the one round surface here that gets *wider* as it goes, so
/// the frustum is put where that matters: its narrow cap sticks out below the
/// block's floor, its wide cap sits inside the block, and the block's floor
/// crosses the cone's wall in a circle.
///
/// **Clear of every side wall on purpose.** A wall passing through the cone cuts
/// it in a hyperbola, and `how_often_a_fuse_with_a_cone_refuses.rs` pins that
/// arrangement — its "a frustum against a block" family — at 6 answers out of
/// 96, with the other 90 refused because nothing can put points in order along
/// the curve. A family the fuse mostly refuses leaves every property here green
/// having compared nothing (§14.7), and §47.2 is what those six answers turned
/// out to be worth. The degenerate arrangements are hand-written one at a time
/// elsewhere.
///
/// # Why the slope is one over a power of two
///
/// `Model::validate` asks that every corner of a curved face be certified on its
/// surface. A cone's corner has to satisfy `radius = slope · height`, a product
/// of two stored numbers, and a slope of `0.6` would round that off its own
/// face. So the two cap radii are the whole numbers here and the heights are
/// derived: a cap of radius `r` sits `r · up_per_unit_wider` up the axis, exact
/// for every `r` an `f64` can hold. `tests/cones/mod.rs` carries the same note at
/// length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrustumAndBlock {
    /// How wide the block is, and how tall.
    pub side: u32,
    /// The radius of the frustum's lower cap, the one below the floor.
    pub narrow: u32,
    /// The radius of the frustum's upper cap, the one inside the block.
    pub wide: u32,
    /// How far up the axis the cone goes for each unit it gains in radius.
    ///
    /// The slope is one over this, so it is always a power of two.
    pub up_per_unit_wider: u32,
    /// Where the frustum's axis stands, in the block's own coordinates.
    pub at: [u32; 2],
}

impl FrustumAndBlock {
    /// Build the block and then the frustum, in that order.
    ///
    /// The frustum is dropped by one more than its lower cap's height, so that
    /// cap ends up one unit below the block's floor and the upper cap ends up
    /// inside the block.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let block = model
            .add_cube(
                f64::from(self.side),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a whole-number cube");
        let frustum = a_frustum(&mut model, self.slope(), self.near(), self.far());
        model
            .place_solid(
                frustum,
                &Isometry::translation(Vector::new(
                    f64::from(self.at[0]),
                    f64::from(self.at[1]),
                    -1.0 - self.near(),
                ))
                .expect("a whole-number shift"),
            )
            .expect("this model made the key");
        (model, vec![block, frustum])
    }

    /// The same frustum built twice in the same place, and no block.
    ///
    /// What `A \ A` is asked of, and the counterpart of
    /// [`BallAndBlock::built_as_a_ball_and_a_copy_of_it`]. Both copies come out
    /// of the same numbers by the same code, so they are bit-identical rather
    /// than nearly so.
    #[must_use]
    pub fn built_as_a_frustum_and_a_copy_of_it(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let one = a_frustum(&mut model, self.slope(), self.near(), self.far());
        let copy = a_frustum(&mut model, self.slope(), self.near(), self.far());
        (model, vec![one, copy])
    }

    /// How far out the cone goes per unit up, which is one over a power of two.
    #[must_use]
    pub fn slope(&self) -> f64 {
        1.0 / f64::from(self.up_per_unit_wider)
    }

    /// How far up the axis the lower cap sits, before the frustum is dropped.
    #[must_use]
    pub fn near(&self) -> f64 {
        f64::from(self.narrow * self.up_per_unit_wider)
    }

    /// How far up the axis the upper cap sits, before the frustum is dropped.
    #[must_use]
    pub fn far(&self) -> f64 {
        f64::from(self.wide * self.up_per_unit_wider)
    }
}

/// A block with a frustum up through its floor, the frustum clear of every side
/// wall by at least one and its upper cap at least one below the block's roof.
///
/// Three slopes — a half, a quarter and an eighth — so the family is not one
/// cone shape drawn at three sizes.
///
/// Shrinks toward a block of side six with a frustum of slope one half standing
/// in the middle of its floor, running from a cap of radius one a unit below the
/// floor to a cap of radius two a unit above it.
pub fn a_frustum_and_a_block() -> impl Strategy<Value = FrustumAndBlock> {
    (1u32..=3)
        .prop_flat_map(|halvings| {
            let up_per_unit_wider = 1u32 << halvings;
            // Tall enough to hold a frustum that gains a whole unit of radius,
            // and at least six across so the wide cap clears both walls.
            (Just(up_per_unit_wider), up_per_unit_wider.max(6)..=12)
        })
        .prop_flat_map(|(up_per_unit_wider, side)| {
            // A whole unit of room between the wide cap and every side wall.
            let widest = (side / 2 - 1).min(3);
            (Just(side), Just(up_per_unit_wider), 1u32..widest)
        })
        .prop_flat_map(|(side, up_per_unit_wider, narrow)| {
            // No taller than the block, so the wide cap stays a unit below the
            // roof.
            let widest = (side / 2 - 1).min(3).min(narrow + side / up_per_unit_wider);
            (
                Just(side),
                Just(up_per_unit_wider),
                Just(narrow),
                narrow + 1..=widest,
            )
        })
        .prop_flat_map(|(side, up_per_unit_wider, narrow, wide)| {
            let room = wide + 1..=side - wide - 1;
            (
                Just(side),
                Just(narrow),
                Just(wide),
                Just(up_per_unit_wider),
                [room.clone(), room],
            )
                .prop_map(|(side, narrow, wide, up_per_unit_wider, at)| {
                    FrustumAndBlock {
                        side,
                        narrow,
                        wide,
                        up_per_unit_wider,
                        at,
                    }
                })
        })
}

/// A curved fixture that builds itself into a fresh model.
///
/// What lets [`Turned`] turn every curved family by one piece of code rather
/// than one per family.
pub trait Fixture {
    /// Build the solids, in the order a fuse should be handed them.
    fn built(&self) -> (Model, Vec<SolidKey>);
}

impl Fixture for CurvedPair {
    fn built(&self) -> (Model, Vec<SolidKey>) {
        self.build()
    }
}

impl Fixture for BallAndBlock {
    fn built(&self) -> (Model, Vec<SolidKey>) {
        self.build()
    }
}

impl Fixture for BallBarAndBlock {
    fn built(&self) -> (Model, Vec<SolidKey>) {
        self.build()
    }
}

impl Fixture for FrustumAndBlock {
    fn built(&self) -> (Model, Vec<SolidKey>) {
        self.build()
    }
}

impl Fixture for LeaningBar {
    fn built(&self) -> (Model, Vec<SolidKey>) {
        self.build()
    }
}

/// A curved fixture with every solid of it turned by one general rotation.
///
/// §191's first row. Every curved family above stands its round solid straight
/// up `z` at whole-number places, so no curved face any of them builds has an
/// axis that is not exactly `z` — and the KiCad LED dome of §124 was a curved
/// face whose axis came in a few ulps off one. This is the same move
/// [`a_turned_pair`] makes for the flat families, by the same code: the
/// fixture is built exactly as its unturned family builds it, and then
/// [`turned_by`] turns every solid in it, so the solids sit where they sat
/// relative to each other and a refusal is attributable to the rotation and
/// nothing else. The unturned family is the control row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turned<P> {
    /// The fixture as its own family draws it.
    pub fixture: P,
    /// The rotation every solid of it is turned by.
    pub turn: Turn,
}

impl<P: Fixture> Turned<P> {
    /// Build the fixture, then turn every solid of it.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        turned_by(self.fixture.built(), self.turn)
    }
}

/// Whatever `fixtures` draws, under one general rotation drawn beside it.
///
/// Shrinks toward the fixture's own floor turned one sixty-fourth of a turn
/// about `z`.
pub fn turned<P: Strategy>(fixtures: P) -> impl Strategy<Value = Turned<P::Value>>
where
    P::Value: Fixture,
{
    (fixtures, a_turn()).prop_map(|(fixture, turn)| Turned { fixture, turn })
}

/// A block with a round bar leaning through it: in at the floor, out at the
/// roof, a whole unit clear of every side wall at both.
///
/// §191's second row, and a situation no other family reaches. Every bar
/// elsewhere in this file stands square to the block's floor, so the floor and
/// the roof cut it in circles. A bar whose axis leans cuts them in **ellipses**,
/// and its wall meets the floor along a curve whose every point is a square
/// root of something other than a radius.
///
/// The lean is a whole-number direction, `(lean[0], lean[1], up)`, and the bar
/// is turned onto it by [`Isometry::rotation`] — the only constructor with a
/// turn in it — so the axis the model stores is that direction rounded, not
/// the direction itself. The block is not turned at all, which is the
/// difference from [`Turned`]: here the bar and the block are no longer square
/// to each other.
///
/// Kept clear of tangency the way [`a_curved_pair`] is, a whole unit of room
/// between each ellipse and the nearest side wall, because a bar tangent to a
/// wall is another row's subject. [`CurvedPair`] is the control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaningBar {
    /// How wide the block is, and how tall.
    pub side: u32,
    /// The bar's radius.
    pub radius: u32,
    /// Where the bar's axis passes through the height halfway up the block, in
    /// the block's own coordinates.
    pub at: [u32; 2],
    /// How far the axis leans along `x` and `y` for every `up` it climbs.
    pub lean: [i32; 2],
    /// How far the axis climbs for each `lean`.
    pub up: u32,
}

impl LeaningBar {
    /// Build the block and then the bar, in that order.
    ///
    /// The bar is built standing on the origin, stood on its middle, turned
    /// onto its lean about the origin, and then carried to where its axis
    /// crosses the block's middle height.
    #[must_use]
    pub fn build(&self) -> (Model, Vec<SolidKey>) {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let side = f64::from(self.side);
        let block = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a whole-number cube");
        let length = self.length();
        let stood_on_its_middle = Isometry::translation(Vector::new(0.0, 0.0, -length / 2.0))
            .expect("a whole-number shift");
        let [across, along] = self.lean.map(f64::from);
        let leaning = Isometry::rotation(
            &Axis::new(Point::origin(), Vector::new(-along, across, 0.0))
                .expect("a lean that is never straight up"),
            across.hypot(along).atan2(f64::from(self.up)),
        )
        .expect("a finite turn");
        let carried = Isometry::translation(Vector::new(
            f64::from(self.at[0]),
            f64::from(self.at[1]),
            side / 2.0,
        ))
        .expect("a whole-number shift");
        let bar = model
            .add_round_bar(
                f64::from(self.radius),
                length,
                &carried.compose(&leaning.compose(&stood_on_its_middle)),
            )
            .expect("a bar long enough to go right through");
        (model, vec![block, bar])
    }

    /// How long the bar is: long enough that both caps clear the block by more
    /// than a unit, whatever it leans by.
    ///
    /// Half of it is `side + 2`, and the steepest lean drawn is `√2` across for
    /// `2` up, whose cosine is `√(2/3)`. A cap's nearest point to the block is
    /// then at least `(side + 2)·√(2/3) − radius` above the middle, which is more
    /// than `side/2 + 1` for every side [`a_leaning_bar`] draws.
    #[must_use]
    pub fn length(&self) -> f64 {
        2.0 * f64::from(self.side + 2)
    }
}

/// A block with a bar leaning through it, the bar a whole unit clear of every
/// side wall where it goes in and where it comes out.
///
/// The lean is one of the eight whole-number directions one step across for
/// two to four steps up — so the bar leans between fourteen and thirty-five
/// degrees off upright. The block is one of the five narrowest that leave
/// room for the bar at that lean.
///
/// Shrinks toward a bar of radius one leaning one along `x` for four up,
/// through the smallest block that holds it.
pub fn a_leaning_bar() -> impl Strategy<Value = LeaningBar> {
    let leans = proptest::sample::select(vec![
        [1, 0],
        [0, 1],
        [-1, 0],
        [0, -1],
        [1, 1],
        [-1, 1],
        [-1, -1],
        [1, -1],
    ]);
    (1u32..=2, leans, (2u32..=4).prop_map(|up| 6 - up))
        .prop_flat_map(|(radius, lean, up)| {
            // The first five sides with room, and not a range of them: rounding
            // the two ends inward can leave a side with no room between two
            // that have some.
            let sides: Vec<u32> = (4u32..)
                .filter(|&side| {
                    room_for_a_leaning_bar(side, radius, lean, up)
                        .iter()
                        .all(|room| !room.is_empty())
                })
                .take(5)
                .collect();
            (
                Just(radius),
                Just(lean),
                Just(up),
                proptest::sample::select(sides),
            )
        })
        .prop_flat_map(|(radius, lean, up, side)| {
            let [x, y] = room_for_a_leaning_bar(side, radius, lean, up);
            (Just(side), Just(radius), Just(lean), Just(up), [x, y]).prop_map(
                |(side, radius, lean, up, at)| LeaningBar {
                    side,
                    radius,
                    at,
                    lean,
                    up,
                },
            )
        })
}

/// Where along `x` and along `y` a leaning bar's axis may cross the block's
/// middle height and still clear every side wall by a whole unit.
///
/// At the floor and the roof the axis has drifted `side/2 · |lean| / up` from
/// the middle, and the ellipse the bar cuts reaches at most `radius · sec θ`
/// from its axis, `θ` being the lean off upright. Both ends are rounded inward,
/// so the unit of room is at least a unit.
pub fn room_for_a_leaning_bar(
    side: u32,
    radius: u32,
    lean: [i32; 2],
    up: u32,
) -> [RangeInclusive<u32>; 2] {
    let up = f64::from(up);
    let across = f64::from(lean[0]).hypot(f64::from(lean[1]));
    let widest = f64::from(radius) * across.hypot(up) / up;
    lean.map(|toward| {
        let drift = f64::from(side) / 2.0 * f64::from(toward.unsigned_abs()) / up;
        let reach = drift + widest;
        let low = (1.0 + reach).ceil();
        let high = (f64::from(side) - 1.0 - reach).floor();
        // An empty range when the block is too narrow, which the caller reads.
        let [low, high] = [low, high].map(|end| end.max(0.0) as u32);
        low..=high
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
