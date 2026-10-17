//! Where three surfaces meet.
//!
//! [`intersect`](crate::intersect) names the curve two surfaces cross in; this
//! names the **points** a third surface cuts out of that curve, which is what a
//! corner of an arrangement is (`docs/decisions.md` §31.2 rows 4 and 5).
//!
//! # Two planes and a quadric is the road, and twelve ways round it
//!
//! Two planes cross in a line, and a line cut by a quadric is a quadratic in one
//! unknown whose discriminant is a polynomial in the stored numbers. That is the
//! one route that is worked out here, and everything else either arrives at it,
//! is settled by a sign without it, or is refused.
//!
//! One plane and two quadrics is a quartic in general — the plane crosses each
//! quadric in a conic and two conics in one plane meet where a quartic is zero —
//! and reaching *that* takes an algebraic solver rather than a discriminant. But
//! **seven families of it are not a quartic at all**, and all seven are
//! ordinary mechanical CAD rather than curiosities. An eighth takes a ring in
//! place of one quadric:
//!
//! - **Two quadrics that bend the same way.** Write a quadric out as
//!   `p·(M p) + 2b·p + c`. Two of them with the same `M` differ by something
//!   *linear*, so every point both of them hold sits on one flat surface — the
//!   plane `2(b₁ − b₂)·p = c₂ − c₁`, whose every number this file works out
//!   exactly. Two balls have it, two pipes running the same way have it, and two
//!   cones of one slope about one direction have it. So the triple is two planes
//!   and a quadric after all, and the road above answers it.
//!   [`where_a_plane_and_two_quadrics_of_one_bend_meet`].
//! - **Two surfaces turned about one axis, and a plane square to it.** A ball,
//!   a pipe and a cone are each cut by such a plane in a circle about the point
//!   where the axis crosses it, and two circles about one centre are one circle
//!   or share nothing — so the answer is whether two squared radii are equal, a
//!   bar through a ball and a cap across both.
//!   [`what_a_plane_square_to_their_one_axis_leaves`].
//! - **Two surfaces turned about one axis, and a plane that holds it.** A cone
//!   with a pipe, a second cone or a ball on its axis, or a pipe with a ball on
//!   its axis, share circles square to the axis at heights that are the roots
//!   of one quadratic, and the plane cuts each circle in two points, at its
//!   radius either side of the axis — a wall through the circle a frustum
//!   crosses a bar in.
//!   [`what_a_plane_through_their_one_axis_leaves`].
//! - **A plane that only grazes one of them.** A plane that misses a quadric
//!   leaves the three sharing nothing, and a plane that *touches* one at a single
//!   point leaves one candidate — so the whole question is which side of the
//!   third surface that point is on. [`what_a_plane_that_grazes_one_of_them_leaves`].
//! - **Two surfaces turned about one axis, and a plane beside it.** The same
//!   circles, each cut in a chord, touched or missed by a plane parallel to the
//!   axis and off it, as a sign at a root of that quadratic says — a D-flat on
//!   a shaft across the circle a chamfer cone crosses it in. A plane that leans
//!   to the axis and is no nearer square to it than to parallel is asked the
//!   same way, after every other road: the same D-flat placed by a turn, whose
//!   side comes out a rounding off parallel, or a cut at half a right angle
//!   across those circles.
//!   [`what_a_plane_beside_their_one_axis_leaves`].
//! - **A plane that cuts both in circles, about two centres.** A plane cuts a
//!   ball in a circle wherever it is, and a pipe or a cone square to it, and
//!   two circles in one plane differ by a line, as two quadrics of one bend
//!   differ by a plane — a bar's cap across a ball off its axis, or a floor a
//!   bar and a cone stand on side by side.
//!   [`what_a_plane_that_cuts_both_in_circles_leaves`].
//! - **A plane along a pipe.** A plane parallel to a pipe's axis cuts it in
//!   two lines along it, touches it along one or misses it, and each line meets
//!   the other surface where a quadratic is zero — a wall through a bar's axis
//!   and a ball off it, or through the axes of two bars that cross.
//!   [`what_a_plane_along_a_pipe_leaves`].
//! - **A ring and a ball, a pipe or a cone on its axis, and a plane that holds
//!   it or is square to it.** The two share circles square to the axis, at the heights
//!   [`intersect`](crate::intersect) names them by, and the plane cuts each
//!   in two points — a bend's end cap across the circles a bar of the bend's
//!   own radius crosses its tube in. A pipe of the tube's radius along the
//!   ring's core circle instead shares a circle and a loop with it: the plane
//!   square to the pipe holds the circle, and any other cuts the loop in up to
//!   four points.
//!   [`what_a_plane_through_a_rings_axis_leaves`]. A plane square to the axis
//!   cuts the ring in two circles and the other surface in one, all about one
//!   centre, so the three share a whole circle or nothing — a bead's flat face
//!   on a shaft of its outer radius.
//!   [`what_a_plane_square_to_a_rings_axis_leaves`].
//!
//! **A ninth is a quartic, but one with no root to find.** A plane cuts a
//! ball, a pipe or a cone in a curve that keeps between two distances from
//! where the axis crosses it, and where the bands of two such curves keep
//! apart the three share nothing. A cap a rounding off square to a bar and its
//! chamfer cone is it: [`Isometry::compose`](crate::Isometry::compose) rounds,
//! so a part placed twice can carry such a cap.
//! [`what_a_plane_that_cuts_them_in_bands_apart_leaves`].
//!
//! **A tenth is a quartic whose two roots are found one at a time.** Where
//! the bands overlap and the plane is near enough square to both axes, one
//! curve is a line bent by little across the other, and it crosses it once
//! on each side of where that one's axis crosses the plane. A sign at each
//! end of each arc says a crossing is there, a bound on how the line turns
//! says it is the only one, and a few rounds of two equations shrink a box
//! round each — the same cap with a bar and a cone it cuts in one circle on
//! paper, or a cap leaning a hundredth to a bar and its chamfer cone. A ball
//! whose centre is on a pipe's or a cone's axis is taken first, since the two
//! touch along a circle on a plane known exactly: a slanted cut through where
//! a rod's round end meets it.
//! [`what_a_plane_that_cuts_them_in_bands_that_cross_leaves`].
//!
//! **An eleventh is a plane that reaches a pipe only far along it.** A plane
//! a rounding off parallel to a pipe's axis, and farther from it than the
//! pipe's radius, reaches the pipe only `10¹⁶` or more along it. Out there a
//! ball, a second pipe that leans to the first or a cone with no ruling along
//! the pipe is far from the pipe, and a bound on its own expression along the
//! pipe proves it — the side of a block three from a bar and its chamfer
//! cone, all placed by one turn. The band road gives the cone's cut by a
//! plane that steep no outer edge, so it does not reach that.
//! [`what_a_plane_beside_a_pipe_leaves`].
//!
//! **A twelfth is a quartic that one wall mirrors down to a quadratic.** The
//! wall through a pipe's or a cone's axis and the plane's normal is a mirror
//! of both. Where it is a mirror of the other curved surface too — a ball
//! whose centre is on it, or a pipe or a cone whose axis lies in it — the two
//! curved surfaces differ on the plane by a quadratic in the height up the
//! first one's axis alone, and the points are the ends of that one's chords
//! at its roots. A cut steeper than half a right angle to a bar and its
//! chamfer cone is it, and so is a bar leaning to the floor with a ball on its
//! axis, whose stored direction rounds and leaves the ball's centre a rounding
//! off it, and a ball resting on a rod cut leaning through where they touch.
//! [`what_a_plane_one_wall_mirrors_with_both_leaves`].
//!
//! **Three curved surfaces, two of which bend the same way, are one plane and
//! two curved surfaces** by the first family's own argument: every point the
//! two share is on the plane they differ by, and every point on that plane and
//! one of them is on the other. So the plane takes the place of one of the
//! two, and the twelve roads answer the triple — two cones of one slope about one
//! axis and a ball, or two bars running the same way and a third.
//! [`where_three_curved_surfaces_meet`].
//!
//! What is left over is refused by name (§16.10, §31.2): one plane and two
//! quadrics that bend differently, cross the plane in curves, share no axis
//! the plane is square to, parallel to or no nearer square to than parallel,
//! are not both cut by it in circles,
//! neither of which is a pipe it runs along, whose bands overlap, and whose
//! curves the plane leans too far for one to be a line bent by little across
//! the other, or which touch rather than cross, which the plane does not
//! reach only far along a pipe, and which no wall through a pipe's or a
//! cone's axis and the plane's normal mirrors both — a plane leaning well off
//! square to a pipe's axis, which cuts it in a long ellipse, against a ball
//! whose centre is off the wall through that axis and the plane's normal, or a
//! cap of one bar across a second bar leaning to it whose axis passes the
//! first one's by rather than crossing it —
//! a plane along a pipe and a cone one of whose rulings runs along the
//! pipe, three curved surfaces no two of which differ by a plane a stored
//! [`Plane`] is exactly, and any triple with a torus and fewer than two planes
//! but the eighth family.
//! [`Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`] says which of the two
//! kinds of refusal it is in its own doc.
//!
//! # The answer depends on which three surfaces they are, and not on the order
//!
//! Three surfaces can meet at more than one point, so the three names do not
//! pick a corner out on their own and a fuse has to count along the line
//! (§31.3). That count is only well defined if the line has a direction the
//! three surfaces fix between them — so the three are put in a **fixed order**
//! before anything is worked out, and the points come back running along one
//! direction of the line the two planes cross in, the same direction every
//! time for that pair in that order. **It is not always `n₁ × n₂`**:
//! [`the_line_two_planes_cross_in`]'s own doc says which direction it
//! actually is, and
//! `crates/rcad-geom/tests/a_line_steeper_than_a_cones_rulings.rs`'s second
//! test has a pair of planes where it comes out `−(n₁ × n₂)` instead. What
//! `fuse(A, B)` and `fuse(B, A)` need is only that the direction is fixed by
//! the pair and the order, not which of the two signs it carries, so both
//! still number the same corner the same way.
//!
//! # A sign and a size are two different numbers here
//!
//! [`Exact::square_minus`] hands back a sign and never a value, because it
//! multiplies both sides by a positive power of two (§32.1). It is what settles
//! which way the discriminant points where nothing else can. **How big the
//! discriminant is comes from somewhere else** — the same difference summed
//! without the slide — and mixing the two up would put a corner wherever the
//! slide happened to land.

use rcad_predicates::{
    Decision, Interval, Level, Measures, NOTHING_IS_KNOWN, Point as PredicatePoint, Policy,
    ProvenanceLog, Record, SideOfSurface,
};
use smallvec::{SmallVec, smallvec};

use crate::enclosure::{
    cross_enclosing, dot_enclosing, exactly, quotient_or_everything, root_enclosing,
    squared_length_enclosing,
};
use crate::exact::{Exact, Nearly, binary_exponent, multiplied_by_a_power_of_two, slid_exactly};
use crate::intersect::{
    Question, a_pipe_lies_along_a_rings_core, ask, intersect, squared_length_of_a_cross_product,
    the_heights_a_ring_crosses_a_round_surface_at,
};
use crate::plane::which_side_of_a_flat_surface;
use crate::real_roots::where_a_polynomial_is_zero;
use crate::{
    Cone, Cylinder, Error, Plane, PlaneAgreement, Point, Result, Sphere, Surface, SurfaceMeet,
    Torus,
};

/// The name the three-surface meet goes by in provenance records.
pub const WHERE_THREE_SURFACES_MEET: &str = "where three surfaces meet";

/// The name a line-meets-one-surface question goes by in provenance records.
pub const WHERE_A_LINE_MEETS_A_SURFACE: &str = "where a line meets a surface";

/// What three surfaces have in common.
///
/// [`MeetOfThree::At`] is the corner case a fuse is after. The other two arms
/// are answers about the geometry and not failures: three surfaces that share a
/// whole curve have no corner to name, and three that share nothing have none
/// either.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum MeetOfThree {
    /// They have no point in common at all.
    Nowhere,
    /// They meet at these points — one, two, three or four — running along the
    /// line the two planes cross in.
    ///
    /// Three and four take a torus where two of the three are planes: a
    /// straight line meets a quadric twice and a ring four times, which is the
    /// whole of what makes a torus not a quadric. Where one plane holds the axis
    /// two curved surfaces are turned about, there is no such line, and the up
    /// to four points come in the order
    /// [`what_a_plane_through_their_one_axis_leaves`] gives: two circles, two
    /// points on each, or a cone's apex and one circle. A plane beside that
    /// axis, parallel to it or leaning, gives them in the same order, two on a
    /// circle it cuts and one on a circle it touches
    /// ([`what_a_plane_beside_their_one_axis_leaves`]). A plane one wall
    /// mirrors with both gives them the same way, up the axis of the pipe or
    /// the cone the wall is read off: the lower height first, and at each
    /// height the `−(n × w)` end of the chord first
    /// ([`what_a_plane_one_wall_mirrors_with_both_leaves`]). A
    /// plane that cuts both curved surfaces in circles gives them along the
    /// line it crosses the line the two circles differ by in, as two planes do
    /// ([`what_a_plane_that_cuts_both_in_circles_leaves`]). A plane along a
    /// pipe gives up to two on each line it cuts the pipe in, the line on one
    /// side of the axis first and the lower point first along each, in the
    /// order [`what_a_plane_along_a_pipe_leaves`] names. A plane that cuts two
    /// round surfaces in curves that cross gives the two crossings in the
    /// order [`two_points_or_nowhere`] finds them: the one on the side of `−τ`
    /// first, for `τ = n × ĝ / |n|` and `ĝ` the way the difference of the two
    /// rises along the plane.
    ///
    /// Most are a box the true meeting point is somewhere inside, because
    /// solving a quadratic divides and a quotient of two ranges almost never
    /// lands on an `f64`. That width is the point of it (§2.5).
    ///
    /// **A point whose every coordinate is an `f64` is that point**, handed back
    /// as [`PredicatePoint::Exact`], wherever the numbers are known exactly and
    /// each quotient is checked to land on one. A line that touches a ball
    /// touches it at `−B/A` along itself, and at a corner such as `(0, 0, 16)` a
    /// box of any width stops every question whose true answer is zero there —
    /// which way a wall turns against the ball it touches is one of them. Such
    /// a point is not a rounding of the box: it is the one number the box holds.
    At(SmallVec<[PredicatePoint; 2]>),
    /// They share a whole curve or a whole surface, so there is no corner here.
    ///
    /// Two faces on one plane with a third surface across them, or a plane pair
    /// whose line runs down a pipe's own wall.
    MoreThanAPoint,
}

impl MeetOfThree {
    /// This meet, which [`meet_of_three_surfaces`] handed back for `surfaces`,
    /// with the box round a meet of **three planes, or of two planes and a
    /// ball, a pipe or a cone,** narrowed by one checked step. Any other meet
    /// comes back as it is.
    ///
    /// Cramer's rule sums its numerators in intervals, and each numerator is a
    /// small number built out of products near one that all round. Where two of
    /// the three planes are nearly parallel the determinant is small too, so the
    /// box is those roundings divided by a small number: `±1.43e-10` round a meet
    /// that is `1e-33` from `(5, 0, 0)`, on a draw where two walls lean
    /// `1.5e-6` off parallel (§281.1). The width says how badly the rounding
    /// could have gone and nothing about where in the box the meet is.
    ///
    /// **The step.** Take the middle of that box as a guess. For each plane work
    /// out how far the guess misses it, `offset − normal · guess`, from the
    /// stored numbers with nothing rounded. Then the meet is the guess plus
    /// `(r₁ (n₂ × n₃) + r₂ (n₃ × n₁) + r₃ (n₁ × n₂)) / det`, with the cross
    /// products and `det` worked out the same way and read as enclosures. Every
    /// term of that numerator carries a factor of how far the guess misses, so
    /// what rounds there is a rounding of something small. The answer is where
    /// the stepped box and Cramer's box overlap. Both hold the meet, so the
    /// overlap is sound: it is a proof and not a threshold (§17), and on that
    /// draw it is `8.9e-16` wide (§281.2).
    ///
    /// **Two planes and a quadric take the same step**, with the quadric in the
    /// third plane's row ([`narrowed_onto_a_quadric_by_one_checked_step`]).
    /// Their meet is a root of a quadratic along the line the planes cross in,
    /// and the box is that root's box carried into space through a division
    /// and a product, each rounding outward. A block turned by `−π/4` about `x`
    /// meets a round bar's cap circle `2.5e-15` round from its rim start
    /// `(5, 0, 0)` in a box `±3` ulps wide along `x`, and round the circle
    /// from `(0, −5, 0)` the two cannot be put in order from it. Narrowed, the
    /// box is `[5 − 1 ulp, 5]` along `x`, which orders them (§284.3, Q209).
    ///
    /// **Then each side is narrowed to the root along the line**, where the
    /// quadric's sign there can be asked exactly
    /// ([`narrowed_onto_the_root_along_the_line`]). A meet at a graze, far out
    /// where a line a rounding off parallel to a cone's ruling meets the cone
    /// again, is one the step cannot narrow, and this boxes it between two
    /// neighbouring `f64` on each axis (§406.3).
    ///
    /// **Why this is not what [`meet_of_three_surfaces`] hands back to everyone.**
    /// A narrower box round a meet is not a narrower box round a stored corner:
    /// `rcad-topo`'s `bake.rs` derives a face's corners through
    /// [`meet_of_three_surfaces`] and holds its stored corners to that box, and a
    /// stored corner sits ulps off the planes it is on. Cramer's slack covers
    /// that; the narrowed box does not. Narrowed for every caller, nine tests
    /// failed and the draw above refused on another face (§281.3). The one
    /// caller is `rcad-topo`'s arrangement of a face. It asks the narrowed box
    /// whether every point of the meet lies within a corner's rounding, before
    /// it folds the meet into the corner (§345), and, since Q866, for a meet of
    /// three planes as well as one with a curved surface in it, which of two
    /// points comes first along a crossing where the meet's own box cannot say
    /// (Q209). That question is worked out only when the order asks it, from
    /// the node's own box and the three surfaces it was solved from, rather
    /// than kept on the node (§401.4).
    ///
    /// The three planes are narrowed in [`meet_of_three_surfaces`]'s own fixed
    /// order, so the box that comes back depends on which three they are and
    /// not on the order they were asked in (§31.3).
    ///
    /// # Panics
    ///
    /// Where `self` was not worked out from `surfaces` and the stepped box
    /// misses it altogether: the two are then boxes round two different
    /// points, and a corner placed from where they overlap would be nowhere.
    #[must_use]
    pub fn boxed_tight(&self, surfaces: [&Surface; 3]) -> MeetOfThree {
        let [Surface::Plane(one), Surface::Plane(other), third] = in_a_fixed_order(surfaces) else {
            return self.clone();
        };
        let MeetOfThree::At(places) = self else {
            return self.clone();
        };
        if let Surface::Plane(last) = third {
            return MeetOfThree::At(
                places
                    .iter()
                    .map(|&place| narrowed_by_one_checked_step([&one, &other, &last], place))
                    .collect(),
            );
        }
        let Some(quadric) = how_a_surface_is_written(&third) else {
            return self.clone();
        };
        MeetOfThree::At(
            places
                .iter()
                .map(|&place| {
                    let stepped = narrowed_onto_a_quadric_by_one_checked_step(
                        [&one, &other],
                        &quadric,
                        place,
                    );
                    narrowed_onto_the_root_along_the_line([&one, &other], &quadric, stepped)
                })
                .collect(),
        )
    }

    /// Which way the line `planes` cross in runs through `quadric`, at the one
    /// place inside `near` where it meets it: the sign of `∇Q · (n₁ × n₂)`
    /// there, for the planes' stored normals in the order given and `Q` the
    /// quadric's own expression, which is below nothing inside a ball, a pipe
    /// or a cone and above it outside. `None` where that is not proved.
    ///
    /// **Proved from two exact signs, and no gradient is worked out.** Along an
    /// axis the line is not square to, its point with coordinate `s` is
    /// `N(s) / E` for `E` the part of `n₁ × n₂` along that axis
    /// ([`narrowed_onto_the_root_along_the_line`]), and `E² Q(N(s)/E)` is a
    /// polynomial in `s` of degree two at most. Where it is above nothing at
    /// one end of `near`'s side and below at the other, it has one root
    /// between them, that root is where the line meets the quadric inside
    /// `near`, and it is simple, since a double root does not change sign. So
    /// the polynomial climbs through it exactly when it is above nothing at the
    /// high end, and its slope there is `E² ∇Q · (n₁ × n₂) / E`.
    ///
    /// **What it is for.** A line a rounding off parallel to a cone's ruling
    /// meets the cone `2.26e16` out at a graze: `∇Q · (n₁ × n₂)` is a few
    /// units, a cancellation of terms near `1e16`, and worked out over even a
    /// box one ulp wide its enclosure is `17` wide and straddles nothing. That
    /// sign is which way two curves leave the place they meet, and `rcad-topo`'s
    /// arrangement asks it there (§406.3).
    #[must_use]
    pub fn which_way_a_line_runs_through_a_quadric(
        planes: [&Plane; 2],
        quadric: &Surface,
        near: PredicatePoint,
    ) -> Option<Decision> {
        let PredicatePoint::Constructed(boxed) = near else {
            return None;
        };
        let written = how_a_surface_is_written(quadric)?;
        let [one, other] = planes.map(|plane| plane.normal().components());
        (0..3).find_map(|axis| {
            let (low, high) = (boxed[axis].lo(), boxed[axis].hi());
            if !(low.is_finite() && high.is_finite()) {
                return None;
            }
            let sign_at = |along: f64| {
                how_the_quadric_leans_where_the_line_reaches(planes, &written, axis, along)
            };
            let at_the_high_end = sign_at(high)?;
            if !one_root_between(sign_at(low)?, at_the_high_end) {
                return None;
            }
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            let across = Exact::product_of(one[next], other[after])?
                .minus(&Exact::product_of(one[after], other[next])?)
                .sign();
            if across == Decision::Pos {
                Some(at_the_high_end)
            } else if across == Decision::Neg {
                Some(if at_the_high_end == Decision::Pos {
                    Decision::Neg
                } else {
                    Decision::Pos
                })
            } else {
                None
            }
        })
    }

    /// Whether the point of the line `planes` cross in whose coordinate along
    /// `axis` is `along` is proved off `quadric`: the quadric's own expression
    /// there, worked out exactly ([`narrowed_onto_the_root_along_the_line`]),
    /// is above or below nothing. `false` where it is nothing, where the line
    /// is square to that axis, where `along` is not finite, or where a product
    /// falls out of range.
    ///
    /// **What it is for.** A meet of that line with `quadric` is then not at
    /// `along`, so two meets whose boxes along `axis` share only `along` lie
    /// one either side of it. A block edge grazing a ball centred at `x = 16`
    /// meets it at `16 ∓ 2⁻⁵¹`, in the boxes `[15.999999999999998, 16]` and
    /// `[16, 16.000000000000004]`, and the edge's point at `16` is `2⁻¹⁰³`
    /// inside the ball. `rcad-topo`'s arrangement puts the two in order along
    /// the edge by it (§317).
    ///
    /// # Panics
    ///
    /// If `axis` is not `0`, `1` or `2`.
    #[must_use]
    pub fn the_line_is_off_the_quadric_at(
        planes: [&Plane; 2],
        quadric: &Surface,
        axis: usize,
        along: f64,
    ) -> bool {
        assert!(axis < 3, "an axis is 0, 1 or 2, not {axis}");
        if !along.is_finite() {
            return false;
        }
        let Some(written) = how_a_surface_is_written(quadric) else {
            return false;
        };
        let sign = how_the_quadric_leans_where_the_line_reaches(planes, &written, axis, along);
        sign == Some(Decision::Pos) || sign == Some(Decision::Neg)
    }
}

/// Where three surfaces meet.
///
/// # Errors
///
/// [`Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`] when fewer than two of
/// the three are planes and none of the twelve ways round that the module doc
/// names runs — for three curved surfaces, after the plane two of them differ
/// by has taken one's place, or where no two of them differ by one.
///
/// [`Error::Undecidable`] when a sign the answer depends on could not be
/// settled, or when a product the exact arithmetic needs falls out of `f64`'s
/// range, both where the three stand and at both slides below. Both carry the
/// records of where they stand.
///
/// [`Error::Predicate`] wrapping
/// [`PlanesDoNotMeetInAPoint`](rcad_predicates::Error::PlanesDoNotMeetInAPoint)
/// when three planes do meet at one point and neither route below can put a box
/// round it.
///
/// # Three surfaces written far down the range are slid up it first
///
/// Three surfaces about `1e-159` across multiply out to terms near `1e-318`,
/// where every product loses bits, and what is kept of them is too loose to
/// place a corner by (`Nearly::is_within_a_rounding`, `docs/decisions.md`
/// §322.4). The same three multiplied by a power of two are the same geometry at
/// another size, and multiplying by a power of two is exact while every number
/// stays normal. So a refusal where they stand is asked again of the triple slid
/// until its lengths sit either side of one, then, if that refuses too, slid
/// until the largest sits at `2⁵⁰⁰`, and a corner found there is slid back. §349
/// says what a slide does to each kind of surface and where it cannot go, and
/// §349.11 why there are two.
pub fn meet_of_three_surfaces(
    surfaces: [&Surface; 3],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let mut records = ProvenanceLog::new();
    let refused = match the_meet_where_they_stand(surfaces, policy, &mut records) {
        Err(refused @ Error::Undecidable { .. }) => refused,
        answered => {
            records.drain_into(log);
            return answered;
        }
    };
    if let Some(answer) = the_meet_slid_into_range(surfaces) {
        return Ok(answer);
    }
    records.drain_into(log);
    Err(refused)
}

/// Where three surfaces meet, worked out at the size they are written.
fn the_meet_where_they_stand(
    surfaces: [&Surface; 3],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let [first, second, third] = in_a_fixed_order(surfaces);
    let at = the_places_these_three_sit(&first, &second, &third);
    let Surface::Plane(one) = &first else {
        return where_three_curved_surfaces_meet([&first, &second, &third], &at, policy, log);
    };
    let Surface::Plane(other) = &second else {
        return where_one_plane_and_two_curved_surfaces_meet(
            one, &second, &third, &at, policy, log,
        );
    };
    if let Surface::Plane(last) = &third {
        return where_three_planes_meet(one, other, last, policy, log);
    }
    let Some(line) = the_line_two_planes_cross_in(one, other, &at, policy, log)? else {
        return what_a_third_surface_leaves_of_one_plane(one, other, &third, &at, policy, log);
    };
    // **A torus is the only surface here that a line does not meet twice**, and
    // it takes its own route rather than a quadratic's. A torus sorts after
    // every other kind, and two of the three must be planes to get this far, so
    // a torus anywhere in the triple is `third` — which is what makes this one
    // check cover every route in.
    let found = if let Surface::Torus(ring) = &third {
        where_a_line_meets_a_ring(WHERE_THREE_SURFACES_MEET, &line, ring, &at)?
    } else {
        let quadratic = how_a_quadric_cuts_the_line(&line, &third)
            .ok_or_else(|| an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, &at))?;
        where_a_line_meets_a_quadric(WHERE_THREE_SURFACES_MEET, &quadratic, &at, policy, log)?
    };
    the_points_those_steps_name(
        WHERE_THREE_SURFACES_MEET,
        &line,
        found,
        &at,
        policy,
        log,
        || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, &at),
    )
}

/// The meet of the same three surfaces slid along the range by a power of two,
/// slid back — or `None` where no slide moves anything, no slide is exact, or
/// every slid triple refuses too.
///
/// **The first slide centres the binades the lengths span on one**, rather than
/// bringing the largest to one. What runs out is a product of two small
/// lengths falling off the bottom or of two large ones running off the top,
/// and centring leaves the two ends the same room. For a triple of one size
/// the two choices are the same slide. For one whose places sit `2¹²` out and
/// whose ball is `2⁻⁶⁰⁰` across they are not: the largest brought to one leaves
/// the radius squared at `2⁻¹²²⁴`, and centred it is `2⁻⁶¹²`. Measured on the
/// `somewhere else` row of
/// `every_point_where_three_surfaces_meet_is_on_all_three_of_them.rs`: 180 of
/// 2051 triples refused with no slide, 52 with the largest brought to one, and
/// none centred — every one of the 52 a place moved out by `2^shift`.
///
/// **Where that refuses too, the second brings the largest length to
/// [`HIGHEST_BINADE_A_SLIDE_LIFTS_TO`].** Lengths that span more than about a
/// thousand binades leave a centred product of the two largest past the top of
/// `f64`. Brought down so the largest sits at `2⁵⁰⁰`, no product of two lengths
/// runs off the top, and one that falls off the bottom is far under a rounding
/// of the products near the top. Measured on the `each at its own size` row of
/// the same file: 277 of 2051 triples refused with the centred slide alone, the
/// first at 1000 binades of spread, and 108 with both, the first at 1200
/// (`docs/decisions.md` §349.11).
///
/// **Asked under a certified policy whatever the caller's was**, because a
/// tolerant budget is a length at the size the caller wrote, and at another
/// size it would be a different budget. Refusing is the conservative half of
/// that: a tolerant policy only accepts more, so what this refuses is an answer
/// not given, never one given wrongly. And a certified answer logs nothing,
/// which is the truth about it.
fn the_meet_slid_into_range(surfaces: [&Surface; 3]) -> Option<MeetOfThree> {
    let binades: SmallVec<[i32; 15]> = surfaces
        .iter()
        .flat_map(|surface| the_lengths_of(surface))
        .filter(|length| *length != 0.0)
        .map(binary_exponent)
        .collect();
    let (Some(lowest), Some(highest)) = (binades.iter().min(), binades.iter().max()) else {
        return None;
    };
    [
        -(lowest + highest) / 2,
        HIGHEST_BINADE_A_SLIDE_LIFTS_TO - highest,
    ]
    .into_iter()
    .filter(|up| *up != 0)
    .find_map(|up| the_meet_slid_by(surfaces, up))
}

/// Where the second slide puts the binade of the largest length.
///
/// A length below `2⁵⁰¹` times another is below `2¹⁰⁰²`, which leaves room to
/// add up a few such products under `2¹⁰²⁴`. Every length below it stays a
/// normal number, so the slide stays exact, while the lengths span at most
/// `500 + 1022 = 1522` binades. Chosen by that argument, not tuned by a sweep
/// (`docs/decisions.md` §349.11).
const HIGHEST_BINADE_A_SLIDE_LIFTS_TO: i32 = 500;

/// The meet of the same three surfaces slid by `2^up`, slid back.
fn the_meet_slid_by(surfaces: [&Surface; 3], up: i32) -> Option<MeetOfThree> {
    let [first, second, third] = surfaces.map(|surface| the_same_surface_slid_by(surface, up));
    let (first, second, third) = (first?, second?, third?);
    match the_meet_where_they_stand(
        [&first, &second, &third],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .ok()?
    {
        MeetOfThree::At(points) => {
            let back: SmallVec<[PredicatePoint; 2]> = points
                .iter()
                .map(|point| the_point_slid_back(*point, -up))
                .collect();
            back.iter()
                .all(|point| point.uncertainty().is_finite())
                .then_some(MeetOfThree::At(back))
        }
        MeetOfThree::Nowhere => Some(MeetOfThree::Nowhere),
        MeetOfThree::MoreThanAPoint => Some(MeetOfThree::MoreThanAPoint),
    }
}

/// The numbers a surface stores that are lengths, and so move when space is
/// scaled. A direction, a normal and a cone's slope are not among them: the
/// first two say which way and the third is one length over another.
fn the_lengths_of(surface: &Surface) -> SmallVec<[f64; 5]> {
    match surface {
        Surface::Plane(flat) => smallvec![flat.offset()],
        Surface::Sphere(ball) => {
            let [x, y, z] = ball.centre().coordinates();
            smallvec![x, y, z, ball.radius()]
        }
        Surface::Cylinder(pipe) => {
            let [x, y, z] = pipe.axis().origin().coordinates();
            smallvec![x, y, z, pipe.radius()]
        }
        Surface::Cone(cone) => SmallVec::from_slice(&cone.apex().coordinates()),
        Surface::Torus(ring) => {
            let [x, y, z] = ring.centre().coordinates();
            smallvec![x, y, z, ring.ring_radius(), ring.tube_radius()]
        }
    }
}

/// The same surface with space scaled by `2^exponent`, or `None` where one of
/// its lengths would not stay a normal number and so would not be exactly that
/// length moved.
///
/// Every length is multiplied and nothing else is touched. A plane keeps its
/// normal and an axis its direction through `moved_to`, which keeps the stored
/// bits: going back through [`Axis::new`](crate::Axis::new) would scale a unit
/// direction to length one a second time and can move its last bit, which
/// would be another surface.
///
/// # Panics
///
/// If a constructor refuses what an exact slide of a surface it built hands
/// back. Every length that comes through is finite, a radius above zero is
/// still above zero, and a tube thinner than its ring is still thinner.
fn the_same_surface_slid_by(surface: &Surface, exponent: i32) -> Option<Surface> {
    const BUILT: &str = "an exact slide of a surface is a surface";
    let slid = |value: f64| slid_exactly(value, exponent);
    let slid_point = |place: Point| -> Option<Point> {
        let [x, y, z] = place.coordinates();
        Some(Point::new(slid(x)?, slid(y)?, slid(z)?))
    };
    let moved = match surface {
        Surface::Plane(flat) => Surface::Plane(flat.moved_to(slid(flat.offset())?).expect(BUILT)),
        Surface::Sphere(ball) => Surface::Sphere(
            Sphere::new(slid_point(ball.centre())?, slid(ball.radius())?).expect(BUILT),
        ),
        Surface::Cylinder(pipe) => Surface::Cylinder(
            Cylinder::new(
                pipe.axis()
                    .moved_to(slid_point(pipe.axis().origin())?)
                    .expect(BUILT),
                slid(pipe.radius())?,
            )
            .expect(BUILT),
        ),
        Surface::Cone(cone) => Surface::Cone(
            Cone::new(
                cone.axis().moved_to(slid_point(cone.apex())?).expect(BUILT),
                cone.slope(),
            )
            .expect(BUILT),
        ),
        Surface::Torus(ring) => Surface::Torus(
            Torus::new(
                ring.axis()
                    .moved_to(slid_point(ring.centre())?)
                    .expect(BUILT),
                slid(ring.ring_radius())?,
                slid(ring.tube_radius())?,
            )
            .expect(BUILT),
        ),
    };
    Some(moved)
}

/// A corner found in the slid triple, slid back by `2^exponent`.
///
/// An end that comes back unchanged when it is slid up again was moved
/// exactly. One that does not landed among the subnormals and was rounded to
/// the nearest of them, less than one subnormal off, so it is stepped one
/// subnormal further out and the box still holds the corner. One that runs off
/// the top is infinite, and the caller reads that as the refusal it is.
fn the_point_slid_back(point: PredicatePoint, exponent: i32) -> PredicatePoint {
    let exactly = |value: f64| {
        let moved = multiplied_by_a_power_of_two(value, exponent);
        (multiplied_by_a_power_of_two(moved, -exponent) == value).then_some(moved)
    };
    if let PredicatePoint::Exact(place) = point
        && let [Some(x), Some(y), Some(z)] = place.map(exactly)
    {
        return PredicatePoint::Exact([x, y, z]);
    }
    let outward = |end: f64, step: fn(f64) -> f64| {
        exactly(end).unwrap_or_else(|| step(multiplied_by_a_power_of_two(end, exponent)))
    };
    PredicatePoint::Constructed(point.enclosure().map(|side| {
        Interval::new(
            outward(side.lo(), f64::next_down),
            outward(side.hi(), f64::next_up),
        )
        .expect("both ends moved the same way, and outward where they rounded")
    }))
}

/// The steps a line meets a surface at, turned into the points they name.
///
/// **The one place a step becomes a point**, so the two questions have one
/// answer between them and not two: [`where_a_line_meets_a_surface`] hands back
/// the steps and [`meet_of_three_surfaces`] hands back what this makes of them.
///
/// A [`MeetsAt::MaybeATouch`] is decided here, because a point either is a
/// corner or is not and there is no third thing for one to be. Which way it
/// goes is the caller's policy: a certified one refuses and a tolerant one
/// reads the touch as real, which is [`ask`]'s own rule and not a second one.
///
/// `unplaceable` says what to return when a step is real but the point it names
/// sits past the top of `f64`'s range — [`where_three_planes_meet`] has a
/// refusal from Cramer's rule to hand back there and nothing else does.
fn the_points_those_steps_name(
    named: &'static str,
    line: &ALine,
    found: WhereALineMeetsIt,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
    unplaceable: impl FnOnce() -> Error,
) -> Result<MeetOfThree> {
    let steps = match found {
        WhereALineMeetsIt::Nowhere => return Ok(MeetOfThree::Nowhere),
        WhereALineMeetsIt::MoreThanAPoint => return Ok(MeetOfThree::MoreThanAPoint),
        WhereALineMeetsIt::At(steps) => steps,
    };
    let mut places: SmallVec<[PredicatePoint; 2]> = SmallVec::new();
    for step in steps {
        let step = match step {
            MeetsAt::Here(step) => step,
            MeetsAt::MaybeATouch {
                inside,
                how_far_it_reaches,
            } => {
                // The quartic is a length to the fourth and `Measures` stops at
                // a volume; `where_three_planes_meet`'s note says why the
                // nearest arm is taken rather than a new one invented here, and
                // §28 says what should replace the lot.
                let touching = ask(
                    &how_far_it_reaches_over(named, how_far_it_reaches),
                    Measures::Volume,
                    at,
                    policy,
                    log,
                )?;
                assert!(
                    matches!(touching, Decision::Pos | Decision::Neg | Decision::Zero),
                    "a sign question answers with a sign: {touching:?}"
                );
                if !matches!(touching, Decision::Zero) {
                    continue;
                }
                inside
            }
        };
        places.push(the_point_at(line, step));
    }
    if places.iter().any(|place| !place.uncertainty().is_finite()) {
        return Err(unplaceable());
    }
    if places.is_empty() {
        return Ok(MeetOfThree::Nowhere);
    }
    Ok(MeetOfThree::At(places))
}

/// Where a line meets a surface, measured **along the line** rather than in
/// space.
///
/// A ray caster's question is about `t`: a meeting is ahead of where the ray
/// started or behind it, and that is the sign of `t` and nothing else. Handing
/// back points and letting the caller recover `t` from one would round the one
/// quantity [`where_a_line_meets_a_surface`] is careful about.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum WhereALineMeetsIt {
    /// The line misses the surface entirely.
    Nowhere,
    /// It meets the surface at these steps along itself, ascending — one, two,
    /// three or four of them. Three and four take a torus.
    At(SmallVec<[MeetsAt; 2]>),
    /// The line lies in the surface, so there is no step that names a meeting:
    /// a line in a plane, or one down a pipe's own wall.
    MoreThanAPoint,
}

/// One step along a line at which it meets a surface.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeetsAt {
    /// A meeting, this far along. A box, because solving for a step divides and
    /// a quotient of two ranges almost never lands on an `f64` (§2.5) — and a
    /// box of no width where the line touches a quadric at a step that does.
    Here(Interval),
    /// A touch somewhere in this range that no `f64` names, so no sign
    /// evaluation anywhere tells it from a near miss.
    ///
    /// **Not a refusal and not a meeting** — it is the predicate-level
    /// `Undecidable` this question has, and the caller's policy is what says
    /// which of the two it is worth. A **gap**:
    /// [`real_roots`](crate::real_roots) names the family of fixes.
    MaybeATouch {
        /// The box the touch would be in. Two neighbouring `f64`, or one.
        inside: Interval,
        /// How far the surface's own polynomial reaches over that box. It
        /// straddles zero, or this would not be here.
        how_far_it_reaches: Interval,
    },
}

/// Where a line meets one surface: nowhere, at up to four steps along itself, or
/// lying along the whole of it.
///
/// The line is `from + t·along`, and what comes back is `t`. **Both are boxes**,
/// because the point a fuse classifies is one: a corner the fuse built is known
/// to lie in a box and nothing recovers what was never there (§2.5). Where the
/// six boxes each name a single `f64` — which is what a corner on the lattice
/// comes to, and what `as_exact_as_it_is` exists to notice — every coefficient
/// below is an exact expansion and the exact level decides every sign. Where
/// they do not, the coefficients are boxes, the exact level cannot run at all,
/// and a sign whose box straddles zero is undecided.
///
/// This is [`meet_of_three_surfaces`] with the line handed over directly
/// instead of built from two crossing planes — the same coefficients and the
/// same cascade, not a second answer to the same question. It is what a ray
/// caster calls: a ray is not two planes, and [`real_roots`](crate::real_roots)
/// — the machinery a torus needs — is `pub(crate)`, so a caller outside this
/// crate could reach a plane, a ball, a pipe or a cone but never a ring
/// without this.
///
/// # Errors
///
/// [`Error::Undecidable`] when a sign the answer depends on could not be
/// settled, when a product the exact arithmetic needs falls out of `f64`'s
/// range, or when a step comes out past the top of that range. All three carry
/// the records.
///
/// The first is a **gap** when the line's own numbers are exact — the same line
/// and surface written elsewhere on the number line are answered — and it is
/// what a box of a line honestly comes to when it is not.
///
/// [`Error::WhereALineMeetsARingNeedsTheLineExactly`] for a torus asked about a
/// line that is a box. A **gap**, and [`real_roots`](crate::real_roots)'s own
/// doc says what closes it.
///
/// # A line and a surface written far down the range are slid up it first
///
/// A ball `2⁻⁵²⁸` across asked about a line its own size multiplies out to
/// terms near `2⁻¹⁰⁵⁶`, and a ring's quartic to terms near `2⁻²¹¹²`, which
/// `f64` does not hold. So a refusal where they stand is asked again with every
/// length multiplied by one power of two, the way [`meet_of_three_surfaces`]
/// asks again. **The line's direction is one of the lengths**, so the step along
/// it does not move: `2ᵏ·from + t·2ᵏ·along` is `2ᵏ` times the point at `t`. What
/// does move is how far a ring's quartic reaches over a touch, a length to the
/// fourth, and it is slid back down. `docs/decisions.md` §349.8.
///
/// **The slide is not kept for tiny numbers.** Any refusal is asked again once
/// the binades are off centre. A quarter bend of ring one and tube `1 − 2⁻³⁰`,
/// cast at from its cap's corner `(2⁻³⁰, 0, 0)`, is answered slid by `2¹⁴`,
/// where [`real_roots`](crate::real_roots) gives up on the quartic as written.
pub fn where_a_line_meets_a_surface(
    from: [Interval; 3],
    along: [Interval; 3],
    surface: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    let mut records = ProvenanceLog::new();
    let refused =
        match where_a_line_meets_it_where_they_stand(from, along, surface, policy, &mut records) {
            Err(refused @ Error::Undecidable { .. }) => refused,
            answered => {
                records.drain_into(log);
                return answered;
            }
        };
    match where_a_line_meets_it_slid_into_range(from, along, surface) {
        None | Some(Err(Error::Undecidable { .. })) => {
            records.drain_into(log);
            Err(refused)
        }
        Some(slid) => slid,
    }
}

/// Where a line meets one surface, worked out at the size they are written.
fn where_a_line_meets_it_where_they_stand(
    from: [Interval; 3],
    along: [Interval; 3],
    surface: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    let at = [
        PredicatePoint::Constructed(from),
        the_point_a_surface_sits_at(surface),
    ];
    let line = the_line_itself(from, along);
    let found = match surface {
        Surface::Plane(flat) => where_a_line_crosses_one_plane(
            WHERE_A_LINE_MEETS_A_SURFACE,
            &line,
            flat,
            &at,
            policy,
            log,
        )?,
        Surface::Torus(ring) => {
            where_a_line_meets_a_ring(WHERE_A_LINE_MEETS_A_SURFACE, &line, ring, &at)?
        }
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) => {
            let quadratic = how_a_quadric_cuts_the_line(&line, surface).ok_or_else(|| {
                an_exact_step_could_not_be_taken(WHERE_A_LINE_MEETS_A_SURFACE, &at)
            })?;
            where_a_line_meets_a_quadric(
                WHERE_A_LINE_MEETS_A_SURFACE,
                &quadratic,
                &at,
                policy,
                log,
            )?
        }
    };
    // A step that runs off the end of the number line is a crossing past the
    // top of `f64`'s range, which is nothing a caller can use — the same
    // reading `where_three_planes_meet` gives an unbounded box.
    if let WhereALineMeetsIt::At(steps) = &found
        && steps
            .iter()
            .any(|step| !(step.somewhere().lo().is_finite() && step.somewhere().hi().is_finite()))
    {
        return Err(an_exact_step_could_not_be_taken(
            WHERE_A_LINE_MEETS_A_SURFACE,
            &at,
        ));
    }
    Ok(found)
}

/// Where the same line meets the same surface slid along the range by a power
/// of two — or `None` where no slide moves anything or the slide is not exact.
///
/// A refusal of the slid question comes back as it is. One by name, such as a
/// ring asked about a line that is a box, is the refusal the question has at
/// any size, where the question as it stands may have fallen out of `f64`
/// before reaching it.
///
/// The slide centres the binades the lengths span on one, the line's two boxes
/// among them, for the reason [`the_meet_slid_into_range`] gives, and it is
/// asked under a certified policy for the reason given there too. A box slides
/// end by end, and an end that would not stay a normal number sinks the slide
/// as a surface's length does: a slid box that was rounded would hold other
/// lines, and the exact level could no longer run on a line that was exact.
///
/// The steps come back as they are, since the line's direction slid with it.
/// How far a ring's quartic reaches over a touch is a length to the fourth,
/// so it comes back slid by four times the slide, outward.
fn where_a_line_meets_it_slid_into_range(
    from: [Interval; 3],
    along: [Interval; 3],
    surface: &Surface,
) -> Option<Result<WhereALineMeetsIt>> {
    let binades: SmallVec<[i32; 17]> = from
        .iter()
        .chain(&along)
        .flat_map(|side| [side.lo(), side.hi()])
        .chain(the_lengths_of(surface))
        .filter(|length| *length != 0.0)
        .map(binary_exponent)
        .collect();
    let (Some(lowest), Some(highest)) = (binades.iter().min(), binades.iter().max()) else {
        return None;
    };
    let up = -(lowest + highest) / 2;
    if up == 0 {
        return None;
    }
    let slid_box = |boxed: [Interval; 3]| -> Option<[Interval; 3]> {
        let [x, y, z] = boxed.map(|side| {
            Some(
                Interval::new(slid_exactly(side.lo(), up)?, slid_exactly(side.hi(), up)?)
                    .expect("both ends moved exactly, and the same way"),
            )
        });
        Some([x?, y?, z?])
    };
    let found = where_a_line_meets_it_where_they_stand(
        slid_box(from)?,
        slid_box(along)?,
        &the_same_surface_slid_by(surface, up)?,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Ok(WhereALineMeetsIt::At(steps)) = found else {
        return Some(found);
    };
    Some(Ok(WhereALineMeetsIt::At(
        steps
            .into_iter()
            .map(|step| match step {
                MeetsAt::Here(step) => MeetsAt::Here(step),
                MeetsAt::MaybeATouch {
                    inside,
                    how_far_it_reaches,
                } => MeetsAt::MaybeATouch {
                    inside,
                    how_far_it_reaches: a_reach_slid_back(how_far_it_reaches, -4 * up),
                },
            })
            .collect(),
    )))
}

/// How far a ring's quartic reaches over a box, slid by `2^exponent` and
/// outward wherever that rounds.
///
/// Four times a slide can be past the 2046 binades
/// [`multiplied_by_a_power_of_two`] spans in its two steps — `2⁻⁵²⁸` is four
/// times 528 — so it goes in steps of at most a thousand, each a factor `f64`
/// holds. An end that comes out a normal number was moved exactly. One that
/// comes out a subnormal or zero was rounded at each step that left it there,
/// and those roundings come to less than one subnormal between them, since
/// each later step shrinks what the earlier ones lost; it is stepped one
/// further out. One that runs off the top is infinite, which holds anything.
fn a_reach_slid_back(reach: Interval, exponent: i32) -> Interval {
    let outward = |end: f64, step: fn(f64) -> f64| {
        let mut moved = end;
        let mut left = exponent;
        while left != 0 {
            let part = left.clamp(-1000, 1000);
            moved *= 2f64.powi(part);
            left -= part;
        }
        if end == 0.0 || moved.is_normal() {
            moved
        } else {
            step(moved)
        }
    };
    Interval::new(
        outward(reach.lo(), f64::next_down),
        outward(reach.hi(), f64::next_up),
    )
    .expect("both ends moved the same way, and outward where they rounded")
}

impl MeetsAt {
    /// The box this step is somewhere inside, whichever of the two it is.
    #[must_use]
    pub fn somewhere(self) -> Interval {
        match self {
            MeetsAt::Here(step) | MeetsAt::MaybeATouch { inside: step, .. } => step,
        }
    }
}

/// A line built straight from a ray's own two boxes, written the way [`ALine`]
/// writes one — `(through + s·direction) / divided_by` — with the divisor left
/// at one because there are no two planes here to divide by.
fn the_line_itself(from: [Interval; 3], along: [Interval; 3]) -> ALine {
    ALine {
        through: from.map(Term::from_a_box),
        direction: along.map(Term::from_a_box),
        divided_by: Term::of(1.0),
    }
}

/// The three surfaces in a fixed order, so what comes back depends on which
/// three they are and not on which face asked first.
///
/// Sorted on the numbers each one stores, planes first. That is a sound key for
/// exactly the reason §32.5's trap gives: two faces that are the same plane hold
/// the same four numbers. **A [`HalfSpace`](crate::HalfSpace) would not be one**
/// and §33.8 says why, which is the other half of the same rule.
fn in_a_fixed_order(surfaces: [&Surface; 3]) -> [Surface; 3] {
    let mut sorted = surfaces.map(|surface| *surface);
    sorted.sort_by(Surface::how_it_sorts_against);
    sorted
}

/// The three points a provenance record names these surfaces by.
fn the_places_these_three_sit(
    first: &Surface,
    second: &Surface,
    third: &Surface,
) -> [PredicatePoint; 3] {
    [first, second, third].map(the_point_a_surface_sits_at)
}

/// The point a provenance record names one surface by.
fn the_point_a_surface_sits_at(surface: &Surface) -> PredicatePoint {
    PredicatePoint::from(match surface {
        Surface::Plane(flat) => flat.origin().coordinates(),
        Surface::Sphere(ball) => ball.centre().coordinates(),
        Surface::Cylinder(pipe) => pipe.axis().origin().coordinates(),
        Surface::Cone(cone) => cone.apex().coordinates(),
        Surface::Torus(ring) => ring.centre().coordinates(),
    })
}

/// Three planes: Cramer's rule, and an exact second look when it gives up.
///
/// [`Plane::meet_of_three`] is the one authority on where three planes meet and
/// it stays that (§23.8). What it cannot do is tell "there is no single point"
/// from "there is one and interval arithmetic could not bound it", because its
/// determinant is worked out in intervals and a determinant that cancels
/// encloses zero either way.
///
/// So its refusal is read rather than passed on. The same determinant summed
/// through [`Exact`] is known to full relative accuracy — §29's lesson, one
/// question up — and it says which of the two happened. Where it says the three
/// really do meet at a point, the point is placed off the line the first two
/// planes cross in, which divides by that same well-known determinant instead of
/// by an interval one.
///
/// **A box with a side that runs on for ever is read as a refusal too.** Cramer's
/// rule divides one interval determinant by another and neither is checked for
/// overflow, so three planes whose offsets are near the top of the range and
/// whose normals lean at a small angle come back as `Ok` holding a box that says
/// nothing at all — and a caller would read that as a place. It is treated
/// exactly as a refusal is, which is what
/// `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`
/// demonstrates.
fn where_three_planes_meet(
    one: &Plane,
    other: &Plane,
    last: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let at = the_places_these_three_sit(
        &Surface::Plane(*one),
        &Surface::Plane(*other),
        &Surface::Plane(*last),
    );
    let by_cramers_rule = Plane::meet_of_three(one, other, last);
    if let Ok(point) = by_cramers_rule
        && point.uncertainty().is_finite()
    {
        return Ok(MeetOfThree::At(smallvec![point]));
    }
    let refused = by_cramers_rule
        .err()
        .unwrap_or_else(|| an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, &at));

    let Some(line) = the_line_two_planes_cross_in(one, other, &at, policy, log)? else {
        return what_a_third_surface_leaves_of_one_plane(
            one,
            other,
            &Surface::Plane(*last),
            &at,
            policy,
            log,
        );
    };

    let found =
        where_a_line_crosses_one_plane(WHERE_THREE_SURFACES_MEET, &line, last, &at, policy, log)?;
    // Three planes leaning at a shrinking angle meet further and further away,
    // and past the top of `f64`'s range there is nothing left for a box to be
    // around — so this route runs out where Cramer's did, and what is passed on
    // is the first refusal rather than a second one saying the same thing.
    the_points_those_steps_name(
        WHERE_THREE_SURFACES_MEET,
        &line,
        found,
        &at,
        policy,
        log,
        || refused,
    )
}

/// `place`, a box round where three planes meet, narrowed by one checked step
/// from its middle ([`MeetOfThree::boxed_tight`] says what the step is).
///
/// **The box is handed back as it was where the step cannot be taken**: a
/// product that overflows, or a determinant whose enclosure reaches zero. It
/// holds the meet either way, so nothing is lost but the narrowing. A product
/// that falls off the bottom of the range is kept, as [`Nearly`] keeps it,
/// because the floor of a draw can store its offset as a subnormal (§281.2).
///
/// # Panics
///
/// If the stepped box and `place` do not overlap. Both hold the one point the
/// three planes meet at, so an empty overlap is arithmetic that lost its
/// soundness somewhere, and a corner placed from it would be nowhere.
fn narrowed_by_one_checked_step(planes: [&Plane; 3], place: PredicatePoint) -> PredicatePoint {
    let PredicatePoint::Constructed(boxed) = place else {
        return place;
    };
    let guess = boxed.map(|side| 0.5 * side.lo() + 0.5 * side.hi());
    let Some(step) = the_step_to_the_meet(planes, guess) else {
        return place;
    };
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        where_two_boxes_of_one_number_overlap(
            Interval::exactly(guess[axis]) + step[axis],
            boxed[axis],
        )
    }))
}

/// How far the meet of three planes is from `guess`, as an enclosure, or
/// `None` where a product it needs overflows.
///
/// A determinant whose enclosure reaches zero comes back as a step that is
/// everything, which leaves the caller's box as it was.
fn the_step_to_the_meet(planes: [&Plane; 3], guess: [f64; 3]) -> Option<[Interval; 3]> {
    let normals = planes.map(|plane| plane.normal().components());
    let misses: [Interval; 3] = [
        how_far_a_guess_misses(planes[0], guess)?,
        how_far_a_guess_misses(planes[1], guess)?,
        how_far_a_guess_misses(planes[2], guess)?,
    ];
    let across = [
        nearly_cross_of(normals[1], normals[2])?,
        nearly_cross_of(normals[2], normals[0])?,
        nearly_cross_of(normals[0], normals[1])?,
    ];
    let mut determinant = Nearly::of(0.0);
    for axis in 0..3 {
        determinant = determinant.plus(&Nearly::of(normals[0][axis]).times(&across[0][axis])?);
    }
    let determinant = determinant.enclosure();
    Some(core::array::from_fn(|axis| {
        let over = (0..3).fold(Interval::exactly(0.0), |so_far, row| {
            so_far + misses[row] * across[row][axis].enclosure()
        });
        quotient_or_everything(over, determinant)
    }))
}

/// `place`, a box round where two planes and a quadric meet, narrowed by one
/// checked step from its middle: [`narrowed_by_one_checked_step`]'s step, with
/// the quadric in the third plane's row.
///
/// **The row.** Write the quadric `Q(p) = p·(M p) + 2b·p + c`, with `M`
/// symmetric. Then for any two points `Q(x) − Q(g) = (x − g)·(M(x + g) + 2b)`,
/// exactly. At the meet `x`, `Q(x)` is zero, so `x − g` solves the system whose
/// rows are the two normals and `M(x + g) + 2b`, with right-hand sides how far
/// the guess misses each plane and `−Q(g)`. `x` lies in `place`, so
/// `M(place + g) + 2b` worked out in intervals holds that row, and Cramer's rule
/// worked out in intervals over it holds the answer for the row's true value.
/// `Q(g)` is worked out from the stored numbers like the planes' misses, so
/// every term of the numerator again carries a factor of how far the guess
/// misses, and what rounds there is a rounding of something small.
///
/// **The box is handed back as it was where the step cannot be taken**: a
/// product that overflows, or a determinant whose enclosure reaches zero,
/// which a line that all but grazes the quadric gives.
///
/// # Panics
///
/// If the stepped box and `place` do not overlap, as
/// [`narrowed_by_one_checked_step`] does.
fn narrowed_onto_a_quadric_by_one_checked_step(
    planes: [&Plane; 2],
    quadric: &HowASurfaceIsWritten,
    place: PredicatePoint,
) -> PredicatePoint {
    let PredicatePoint::Constructed(boxed) = place else {
        return place;
    };
    let guess = boxed.map(|side| 0.5 * side.lo() + 0.5 * side.hi());
    let Some(step) = the_step_onto_the_quadric(planes, quadric, boxed, guess) else {
        return place;
    };
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        where_two_boxes_of_one_number_overlap(
            Interval::exactly(guess[axis]) + step[axis],
            boxed[axis],
        )
    }))
}

/// How far the meet of two planes and a quadric inside `boxed` is from
/// `guess`, as an enclosure, or `None` where a product it needs overflows.
fn the_step_onto_the_quadric(
    planes: [&Plane; 2],
    quadric: &HowASurfaceIsWritten,
    boxed: [Interval; 3],
    guess: [f64; 3],
) -> Option<[Interval; 3]> {
    let misses = [
        how_far_a_guess_misses(planes[0], guess)?,
        how_far_a_guess_misses(planes[1], guess)?,
        -how_far_a_quadric_reaches_at(quadric, guess)?.enclosure(),
    ];
    let [one, other] = planes.map(|plane| plane.normal().components());
    let row: [Interval; 3] = core::array::from_fn(|axis| {
        (0..3).fold(
            Interval::exactly(2.0) * quadric.half_linear[axis].enclosure(),
            |so_far, column| {
                so_far
                    + quadric.squared[axis][column].enclosure()
                        * (boxed[column] + Interval::exactly(guess[column]))
            },
        )
    });
    let flat = nearly_cross_of(one, other)?.map(|component| component.enclosure());
    let across = [
        cross_enclosing(exactly(other), row),
        cross_enclosing(row, exactly(one)),
        flat,
    ];
    let determinant = dot_enclosing(row, flat);
    Some(core::array::from_fn(|axis| {
        let over = (0..3).fold(Interval::exactly(0.0), |so_far, which| {
            so_far + misses[which] * across[which][axis]
        });
        quotient_or_everything(over, determinant)
    }))
}

/// `Q(guess)` for a quadric written out, from the stored numbers with nothing
/// rounded but what falls off the bottom of the range, or `None` where a
/// product overflows.
fn how_far_a_quadric_reaches_at(quadric: &HowASurfaceIsWritten, guess: [f64; 3]) -> Option<Nearly> {
    let mut total = Nearly::from(quadric.constant.clone());
    for axis in 0..3 {
        let mut pulled = Nearly::from(quadric.half_linear[axis].clone()).times(&Nearly::of(2.0))?;
        for (column, &along) in guess.iter().enumerate() {
            pulled = pulled.plus(
                &Nearly::from(quadric.squared[axis][column].clone()).times(&Nearly::of(along))?,
            );
        }
        total = total.plus(&pulled.times(&Nearly::of(guess[axis]))?);
    }
    Some(total)
}

/// `offset − normal · guess` for one plane, from the stored numbers with
/// nothing rounded but what falls off the bottom of the range.
///
/// The offset enters as an entry of the sum and never as a factor, which is
/// what lets a subnormal offset through where an exact product of it would be
/// refused (§281.2).
fn how_far_a_guess_misses(plane: &Plane, guess: [f64; 3]) -> Option<Interval> {
    let along = Nearly::dot_of(plane.normal().components(), guess)?;
    Some(Nearly::of(plane.offset()).minus(&along).enclosure())
}

/// `u × v` for two triples of stored numbers, keeping what falls off the
/// bottom of the range as [`Nearly`] does.
fn nearly_cross_of(u: [f64; 3], v: [f64; 3]) -> Option<[Nearly; 3]> {
    let component = |axis: usize| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        Some(Nearly::product_of(u[next], v[after])?.minus(&Nearly::product_of(u[after], v[next])?))
    };
    Some([component(0)?, component(1)?, component(2)?])
}

/// Where two enclosures of one number overlap.
///
/// # Panics
///
/// If they do not. Two boxes that each hold the same number share at least that
/// number, so an empty overlap is a broken enclosure.
fn where_two_boxes_of_one_number_overlap(one: Interval, other: Interval) -> Interval {
    let (lo, hi) = (one.lo().max(other.lo()), one.hi().min(other.hi()));
    Interval::new(lo, hi).unwrap_or_else(|_| {
        panic!("two boxes that each hold the same number do not overlap: {one:?} and {other:?}")
    })
}

/// `place`, a box round where two planes and a quadric meet, with each side
/// narrowed to the two `f64` either side of the meet, where the quadric's sign
/// along the line can be asked exactly and changes once across that side.
///
/// **Why the checked step is not enough.** A line a rounding off parallel to a
/// cone's ruling meets the cone a second time far out, `2.26e16` along the
/// edges of a block leaned along a ruling, and it meets it there at a graze:
/// the cone's gradient is `4e16` long and its dot with the line about `10`.
/// The step's row is that gradient worked out over the whole box, which is
/// about 30 wide, so the determinant straddles zero and the step gives up.
/// Along the line the root is simple and the quadratic's slope there is well
/// clear of zero, so asking the sign is enough.
///
/// **The sign along one axis.** Where the line's direction `n₁ × n₂` has a
/// part `E` along an axis, the point of the line whose coordinate there is `s`
/// is `N(s) / E`, with every part of `N` worked out exactly from the stored
/// planes and `s`. So `E² Q(N/E) = N·(M N) + 2E b·N + E² c` has the sign of
/// the quadric there, and nothing in it rounds. The meet's coordinate is a root
/// of it in the box's side. Where the signs at the two ends are opposite and
/// neither is zero, the polynomial, of degree two at most and not nothing, has
/// exactly one root between them, so that root is the meet's coordinate, and
/// halving keeps it between two numbers where the signs still differ. A zero
/// met on the way is the coordinate itself. Anything else hands the side back
/// as it was: an end where the quadric is nothing, two ends of one sign, the
/// line square to the axis, or a product out of range.
fn narrowed_onto_the_root_along_the_line(
    planes: [&Plane; 2],
    quadric: &HowASurfaceIsWritten,
    place: PredicatePoint,
) -> PredicatePoint {
    let PredicatePoint::Constructed(boxed) = place else {
        return place;
    };
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        the_root_along_the_line_inside(planes, quadric, axis, boxed[axis]).unwrap_or(boxed[axis])
    }))
}

/// The meet's coordinate along `axis`, boxed between two neighbouring `f64`
/// inside `side`, or `None` where the signs at the ends of `side` do not show
/// one root between them ([`narrowed_onto_the_root_along_the_line`]).
fn the_root_along_the_line_inside(
    planes: [&Plane; 2],
    quadric: &HowASurfaceIsWritten,
    axis: usize,
    side: Interval,
) -> Option<Interval> {
    let sign_at =
        |along: f64| how_the_quadric_leans_where_the_line_reaches(planes, quadric, axis, along);
    let (mut low, mut high) = (side.lo(), side.hi());
    if !(low.is_finite() && high.is_finite()) {
        return None;
    }
    let low_sign = sign_at(low)?;
    if !one_root_between(low_sign, sign_at(high)?) {
        return None;
    }
    // Each halving halves the width, so this many reach the two neighbouring
    // `f64` from any side a step of the box hands over that is not stretched
    // down into the subnormals; one that is ends here, still holding the root.
    for _ in 0..HALVINGS {
        if low.next_up() >= high {
            break;
        }
        let middle = (0.5 * low + 0.5 * high).clamp(low.next_up(), high.next_down());
        let sign = sign_at(middle)?;
        if sign == Decision::Zero {
            return Some(Interval::exactly(middle));
        }
        if sign == low_sign {
            low = middle;
        } else {
            high = middle;
        }
    }
    Some(Interval::new(low, high).expect("the ends were only ever moved towards each other"))
}

/// Whether a polynomial of degree two at most, not nothing, with these signs at
/// the two ends of a stretch has one root inside it: the two are strict and
/// differ, so the count of roots between them is odd.
fn one_root_between(at_one_end: Decision, at_the_other: Decision) -> bool {
    matches!(at_one_end, Decision::Pos | Decision::Neg)
        && matches!(at_the_other, Decision::Pos | Decision::Neg)
        && at_one_end != at_the_other
}

/// How many times [`the_root_along_the_line_inside`] halves a side at most.
const HALVINGS: usize = 64;

/// The sign of the quadric at the point of the two planes' line whose
/// coordinate along `axis` is `along`, worked out exactly, or `None` where the
/// line is square to that axis or a product falls out of range.
fn how_the_quadric_leans_where_the_line_reaches(
    [one, other]: [&Plane; 2],
    quadric: &HowASurfaceIsWritten,
    axis: usize,
    along: f64,
) -> Option<Decision> {
    let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
    let (mine, theirs) = (one.normal().components(), other.normal().components());
    let across = Exact::product_of(mine[next], theirs[after])?
        .minus(&Exact::product_of(mine[after], theirs[next])?);
    if !matches!(across.sign(), Decision::Pos | Decision::Neg) {
        return None;
    }
    let short_of = |plane: &Plane, normal: [f64; 3]| -> Option<Exact> {
        Some(Exact::of(plane.offset()).minus(&Exact::product_of(normal[axis], along)?))
    };
    let (mine_short, theirs_short) = (short_of(one, mine)?, short_of(other, theirs)?);
    let mut reaches: [Exact; 3] = core::array::from_fn(|_| Exact::of(0.0));
    reaches[axis] = across.times(&Exact::of(along))?;
    reaches[next] = mine_short
        .times(&Exact::of(theirs[after]))?
        .minus(&theirs_short.times(&Exact::of(mine[after]))?);
    reaches[after] = theirs_short
        .times(&Exact::of(mine[next]))?
        .minus(&mine_short.times(&Exact::of(theirs[next]))?);

    let mut squared = Exact::of(0.0);
    let mut linear = Exact::of(0.0);
    for ((entries, half), reach) in quadric
        .squared
        .iter()
        .zip(&quadric.half_linear)
        .zip(&reaches)
    {
        let mut pulled = Exact::of(0.0);
        for (entry, along) in entries.iter().zip(&reaches) {
            pulled = pulled.plus(&entry.times(along)?);
        }
        squared = squared.plus(&reach.times(&pulled)?);
        linear = linear.plus(&half.times(reach)?);
    }
    let twice_across = across.times(&Exact::of(2.0))?;
    let total = squared
        .plus(&twice_across.times(&linear)?)
        .plus(&across.times(&across)?.times(&quadric.constant)?);
    let sign = total.sign();
    match sign {
        Decision::Pos | Decision::Neg | Decision::Zero => Some(sign),
        Decision::Undecidable { .. } => None,
        other => unreachable!("an exact sign is a sign or a refusal: {other:?}"),
    }
}

/// What a third surface leaves of two planes that never cross in a line.
///
/// Either they are two different parallel planes, which already share nothing,
/// or they are one plane — and then the question is where that plane crosses the
/// third surface, which [`intersect`] answers and this does not answer a second
/// time (§23.8). §31.3's coincident-face case, and it costs one `match`.
fn what_a_third_surface_leaves_of_one_plane(
    one: &Plane,
    other: &Plane,
    third: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    match one.agreement_with(other) {
        PlaneAgreement::Different => Ok(MeetOfThree::Nowhere),
        PlaneAgreement::SameWay | PlaneAgreement::OppositeWay => {
            what_one_plane_and_one_curved_surface_share(one, third, policy, log)
        }
        PlaneAgreement::CannotTell => Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        )),
    }
}

/// The whole of what one plane and one other surface have in common, read as an
/// answer about three surfaces.
///
/// Two callers reach it, and both have already worked out that the third surface
/// takes nothing away: two planes that turned out to be one plane, and — through
/// [`where_a_plane_and_two_quadrics_of_one_bend_meet`] — two quadrics whose
/// difference holds everywhere, which is two ways of writing one surface.
/// [`intersect`] is the one authority on where a plane and a curved surface
/// cross and this does not answer it a second time (§23.8).
fn what_one_plane_and_one_curved_surface_share(
    flat: &Plane,
    other: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    match intersect(&Surface::Plane(*flat), other, policy, log)? {
        SurfaceMeet::Nothing => Ok(MeetOfThree::Nowhere),
        SurfaceMeet::TouchAtOnePoint { at } => Ok(MeetOfThree::At(smallvec![at])),
        SurfaceMeet::Curves(_) | SurfaceMeet::SameSurface => Ok(MeetOfThree::MoreThanAPoint),
    }
}

/// Three curved surfaces, by way of the plane two of them differ by.
///
/// Two quadrics that square `p` the same way differ by something flat, as
/// [`where_a_plane_and_two_quadrics_of_one_bend_meet`] says: `Q₁ − Q₂` has no
/// `p·p` left in it. **The three share exactly what that plane, one of the two
/// and the third share.** A point on both quadrics is on the plane, because
/// `Q₁ − Q₂` is nothing where `Q₁` and `Q₂` both are; and a point on the plane
/// and on `Q₁` is on `Q₂`, because `Q₂ = Q₁ − (Q₁ − Q₂)`. So the plane takes
/// the place of the second of the two, and the six roads answer the triple.
/// A cone of slope one with its apex at `z = 0`, a second with its apex at
/// `z = 3`, and a ball of radius `√148` about `(0, 0, 15)` are the plane
/// `z = 1.5` and two curved surfaces, and the ball does not reach that plane.
///
/// **The roads take a stored [`Plane`], and a stored plane's normal has length
/// one**, so the difference is scaled to one and then checked against the
/// exact numbers it was scaled from ([`what_two_quadrics_differ_by`]). Where
/// the check says they are one plane, nothing was lost. Where it does not —
/// two balls whose centres lie along a diagonal, whose plane `x + y = 3` is
/// stored with an offset of `3/√2` rounded — the refusal stands. That is a
/// **gap** and not a limit (§16.10): the points are there, and roads that took
/// the plane as the exact expansions it is, the way the first road already
/// takes the flat surface it builds, would reach them.
///
/// **Bending the same way passes from pair to pair**, because it is nine
/// numbers being equal, so either one pair of the three bends the same way or
/// all three do. Where all three do, the plane any pair gives leaves two
/// quadrics that bend the same way, and the first road answers it, so the
/// first pair whose plane is stored exactly is as good as any.
///
/// Two quadrics whose difference has no `p` in it at all name no plane. Where
/// it is a number other than nothing — two pipes on one axis with two radii —
/// the equation `0 = d` holds nowhere, and neither does the meet. Where it is
/// nothing, the two are one surface written twice and the base's refusal is
/// kept, whatever the other pairs say, so that it does not hang on which
/// pair is looked at first.
fn where_three_curved_surfaces_meet(
    [first, second, third]: [&Surface; 3],
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let written = [first, second, third].map(how_a_surface_is_written);
    let differences = [(0, 1), (0, 2), (1, 2)].map(|(one, other)| {
        what_two_quadrics_differ_by(written[one].as_ref(), written[other].as_ref())
    });
    if differences
        .iter()
        .any(|difference| matches!(difference, WhatTwoQuadricsDifferBy::Nothing))
    {
        return Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat);
    }
    if differences
        .iter()
        .any(|difference| matches!(difference, WhatTwoQuadricsDifferBy::ANumber))
    {
        return Ok(MeetOfThree::Nowhere);
    }
    // The first of each pair stands for the pair, beside the one surface the
    // pair leaves out, in the fixed order the three arrived in.
    let kept = [[first, third], [first, second], [first, second]];
    for (difference, [one, other]) in differences.iter().zip(kept) {
        if let WhatTwoQuadricsDifferBy::APlane(flat) = difference {
            return where_one_plane_and_two_curved_surfaces_meet(flat, one, other, at, policy, log);
        }
    }
    if differences
        .iter()
        .any(|difference| matches!(difference, WhatTwoQuadricsDifferBy::SomethingWouldNotFit))
    {
        return Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        ));
    }
    Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
}

/// One plane and two curved surfaces, by the roads the module doc names.
///
/// **Nothing here guesses.** Each road either reaches an answer it has earned or
/// hands the question on, and what nothing reaches is the refusal that was here
/// before any road was: the quartic is real, its roots are real, and reaching
/// them takes a solver this file does not have.
///
/// The road for a plane square to two surfaces' one axis comes before the one
/// for a plane that grazes, because a plane through a cone's apex square to its
/// axis is both, and the first answers it from stored numbers alone. The same
/// holds for a ring in place of one of the two, and where the plane misses or
/// touches the other surface both roads say the three share nothing. A plane
/// that holds the axis is never square to it, and it cuts a pipe or a cone
/// about that axis in two lines, so it neither misses nor grazes one: that road
/// shares no triple with the ones either side of it.
///
/// The road for a plane beside the axis comes after the one that grazes. Such
/// a plane can miss a pipe or a ball, or touch one, and the road for a plane
/// that grazes answered those before the road beside the axis existed; they
/// keep that answer, and the road beside the axis takes only a plane that cuts
/// both surfaces in curves.
///
/// The road for bands apart comes after all of those, and it only ever
/// answers that the three share nothing. Every triple an earlier road answers
/// keeps that answer, and the band road takes what they all hand on. The road
/// for bands that cross comes next and takes what the band road hands on, so
/// it never changes an answer either. The road for a plane beside a pipe comes
/// next, takes what every road before it hands on, and also only ever answers
/// that the three share nothing. Then the road beside the axis is asked again
/// for a plane that leans to the axis, no nearer square to it than to
/// parallel, and takes only what every road before it handed on. Last of all
/// the road for a plane one wall mirrors with both is asked, and takes only
/// what every other road handed on, so no answer one of them gives moves.
fn where_one_plane_and_two_curved_surfaces_meet(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    if let Some(answer) =
        where_a_plane_and_two_quadrics_of_one_bend_meet(flat, one, other, at, policy, log)?
    {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_square_to_their_one_axis_leaves(flat, one, other, at)? {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_square_to_a_rings_axis_leaves(flat, one, other, at)? {
        return Ok(answer);
    }
    if let Some(answer) =
        what_a_plane_through_their_one_axis_leaves(flat, one, other, at, policy, log)?
    {
        return Ok(answer);
    }
    if let Some(answer) =
        what_a_plane_through_a_rings_axis_leaves(flat, one, other, at, policy, log)?
    {
        return Ok(answer);
    }
    if let Some(answer) =
        what_a_plane_that_grazes_one_of_them_leaves(flat, one, other, at, policy, log)?
    {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_beside_their_one_axis_leaves(
        flat,
        one,
        other,
        BesideTheAxis::Parallel,
        at,
        policy,
        log,
    )? {
        return Ok(answer);
    }
    if let Some(answer) =
        what_a_plane_that_cuts_both_in_circles_leaves(flat, one, other, at, policy, log)?
    {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_along_a_pipe_leaves(flat, one, other, at)? {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_that_cuts_them_in_bands_apart_leaves(flat, one, other) {
        return Ok(answer);
    }
    if let Some(answer) =
        what_a_plane_that_cuts_them_in_bands_that_cross_leaves(flat, one, other, at, policy, log)?
    {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_beside_a_pipe_leaves(flat, one, other) {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_beside_their_one_axis_leaves(
        flat,
        one,
        other,
        BesideTheAxis::Leaning,
        at,
        policy,
        log,
    )? {
        return Ok(answer);
    }
    if let Some(answer) =
        what_a_plane_one_wall_mirrors_with_both_leaves(flat, one, other, at, policy, log)?
    {
        return Ok(answer);
    }
    Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
}

/// A plane and two quadrics that bend the same way, or `None` when they do not
/// bend the same way and this road is not the one.
///
/// Write a quadric as `Q(p) = p·(M p) + 2b·p + c`. Where `M₁` and `M₂` are the
/// same nine numbers, `Q₁ − Q₂` has no `p·p` left in it: it is the flat surface
/// `2(b₁ − b₂)·p = c₂ − c₁`, and **every point the two quadrics share lies on
/// it**, because `Q₂ = Q₁ − (Q₁ − Q₂)`. So the three surfaces share exactly what
/// the plane, that flat surface and the first quadric share, which is two planes
/// and a quadric and is worked out above.
///
/// Two balls bend the same way, two pipes running the same way do, and two cones
/// of one slope about one direction do. A ball and a pipe never do:
/// [`HowASurfaceIsWritten::squared`] is `I` for the one and `(w·w)I − wwᵀ` for
/// the other, and a direction with no length is not a pipe. Two pipes match when
/// their stored directions are the same or opposite and at no other time, which
/// `two_bars_whose_axes_are_written_pointing_opposite_ways_still_meet_a_cap`
/// walks. Two that cross are left to the roads below, and
/// `a_wall_through_the_axes_of_two_bars_that_cross_meets_them_where_their_lines_cross`
/// holds that side: a wall through both axes is [`what_a_plane_along_a_pipe_leaves`]'s.
///
/// **The flat surface it builds is not a [`Plane`]**, and could not be: its
/// numbers are differences of products of the stored ones and an `f64` almost
/// never holds one. It is carried as the [`Exact`] expansions it really is, and
/// [`what_a_stored_plane_and_an_exact_one_share`] crosses it with the stored
/// plane without rounding anything on the way.
fn where_a_plane_and_two_quadrics_of_one_bend_meet(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    let (Some(mine), Some(theirs)) = (
        how_a_surface_is_written(one),
        how_a_surface_is_written(other),
    ) else {
        return Ok(None);
    };
    if !they_bend_the_same_way(&mine, &theirs) {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple and not a reason to try the next road.
    let (leaning, reach) = the_flat_surface_two_quadrics_differ_by(&mine, &theirs);
    where_a_stored_plane_an_exact_one_and_a_quadric_meet(
        flat, &leaning, &reach, one, at, policy, log,
    )
    .map(Some)
}

/// What the stored plane, the flat surface `leaning·p = reach` and `quadric`
/// share, for a flat surface that holds every point the stored plane, `quadric`
/// and a second curved surface share, and on the stored plane holds nothing of
/// `quadric` the second surface does not.
///
/// Where the two planes are one, the three share what the stored plane and
/// `quadric` share; where they never meet, nothing; and where they cross in a
/// line, the points that line meets `quadric` at. Arithmetic that runs out is a
/// refusal about this triple.
fn where_a_stored_plane_an_exact_one_and_a_quadric_meet(
    flat: &Plane,
    leaning: &[Exact; 3],
    reach: &Exact,
    quadric: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let Some(shared) = what_a_stored_plane_and_an_exact_one_share(flat, leaning, reach) else {
        return Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        ));
    };

    let line = match shared {
        WhatTwoPlanesShare::Nothing => return Ok(MeetOfThree::Nowhere),
        WhatTwoPlanesShare::TheWholeOfTheStoredOne => {
            return what_one_plane_and_one_curved_surface_share(flat, quadric, policy, log);
        }
        WhatTwoPlanesShare::ALine(line) => line,
    };
    let quadratic = how_a_quadric_cuts_the_line(&line, quadric)
        .ok_or_else(|| an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at))?;
    let found =
        where_a_line_meets_a_quadric(WHERE_THREE_SURFACES_MEET, &quadratic, at, policy, log)?;
    the_points_those_steps_name(
        WHERE_THREE_SURFACES_MEET,
        &line,
        found,
        at,
        policy,
        log,
        || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at),
    )
}

/// What two curved surfaces differ by, as [`where_three_curved_surfaces_meet`]
/// reads it.
enum WhatTwoQuadricsDifferBy {
    /// Something that is not flat: they square `p` two different ways, or one
    /// of them is not written out as a quadric.
    SomethingCurved,
    /// A plane, and this stored plane is exactly it.
    APlane(Plane),
    /// A plane that no stored plane is exactly, because scaling its normal to
    /// length one rounded it or its offset.
    APlaneNoStoredOneIs,
    /// A number other than nothing, so the two share no point at all.
    ANumber,
    /// Nothing: the two are one surface written twice.
    Nothing,
    /// A product falls out of `f64`'s range, so which of the others it is was
    /// not worked out.
    SomethingWouldNotFit,
}

/// What `mine` minus `theirs` is, for two surfaces written out by
/// [`how_a_surface_is_written`]. `None` is a surface that is not a quadric, or
/// whose numbers fell out of range, and the family hands it on.
///
/// **The stored plane is checked, not trusted.** The difference is divided
/// through by its widest normal component, each quotient taken exactly where
/// it is an `f64` ([`the_f64_quotient_of`]), and handed to
/// [`Plane::from_coefficients`], which scales the normal to length one. It is
/// kept only where [`what_a_stored_plane_and_an_exact_one_share`] says that
/// plane and the exact one are the whole of each other, which is decided
/// exactly, so a plane kept here is the difference and not one near it.
///
/// Dividing first is what keeps two cones of a slope whose square no `f64`
/// holds. Two cones of slope `s` up `z` with their apexes at heights `h₁` and
/// `h₂` differ by `2s²(h₁ − h₂)·z = s²(h₁² − h₂²)`, each side an expansion of
/// several `f64`, and rounding the two sides before dividing can land an ulp
/// off the plane `z = (h₁ + h₂)/2` that their exact quotient is.
fn what_two_quadrics_differ_by(
    mine: Option<&HowASurfaceIsWritten>,
    theirs: Option<&HowASurfaceIsWritten>,
) -> WhatTwoQuadricsDifferBy {
    let (Some(mine), Some(theirs)) = (mine, theirs) else {
        return WhatTwoQuadricsDifferBy::SomethingCurved;
    };
    if !they_bend_the_same_way(mine, theirs) {
        return WhatTwoQuadricsDifferBy::SomethingCurved;
    }
    let (leaning, reach) = the_flat_surface_two_quadrics_differ_by(mine, theirs);
    let signs = [&leaning[0], &leaning[1], &leaning[2], &reach].map(Exact::sign);
    if !signs
        .iter()
        .all(|sign| matches!(sign, Decision::Pos | Decision::Neg | Decision::Zero))
    {
        return WhatTwoQuadricsDifferBy::SomethingWouldNotFit;
    }
    if signs[..3].iter().all(|sign| matches!(sign, Decision::Zero)) {
        return if matches!(signs[3], Decision::Zero) {
            WhatTwoQuadricsDifferBy::Nothing
        } else {
            WhatTwoQuadricsDifferBy::ANumber
        };
    }
    let widest = &leaning[(0..3)
        .max_by(|&one, &other| {
            let size = |axis: usize| leaning[axis].nearest_f64().abs();
            size(one).total_cmp(&size(other))
        })
        .expect("three axes")];
    let over_the_widest = |value: &Exact| {
        the_f64_quotient_of(value, widest)
            .unwrap_or_else(|| value.nearest_f64() / widest.nearest_f64())
    };
    let [a, b, c] = leaning.each_ref().map(over_the_widest);
    let Ok(flat) = Plane::from_coefficients([a, b, c, -over_the_widest(&reach)]) else {
        return WhatTwoQuadricsDifferBy::APlaneNoStoredOneIs;
    };
    match what_a_stored_plane_and_an_exact_one_share(&flat, &leaning, &reach) {
        Some(WhatTwoPlanesShare::TheWholeOfTheStoredOne) => WhatTwoQuadricsDifferBy::APlane(flat),
        Some(WhatTwoPlanesShare::ALine(_) | WhatTwoPlanesShare::Nothing) => {
            WhatTwoQuadricsDifferBy::APlaneNoStoredOneIs
        }
        None => WhatTwoQuadricsDifferBy::SomethingWouldNotFit,
    }
}

/// `Q₁ − Q₂` for two quadrics that square `p` the same way, as the normal and
/// offset of the flat surface `2(b₁ − b₂)·p = c₂ − c₁` it is nothing on.
fn the_flat_surface_two_quadrics_differ_by(
    mine: &HowASurfaceIsWritten,
    theirs: &HowASurfaceIsWritten,
) -> ([Exact; 3], Exact) {
    let leaning = core::array::from_fn(|axis| {
        let apart = mine.half_linear[axis].minus(&theirs.half_linear[axis]);
        apart.plus(&apart)
    });
    (leaning, theirs.constant.minus(&mine.constant))
}

/// A quadric written out as `p·(M p) + 2b·p + c`, exactly.
///
/// The three names are the ones [`Quadratic`] uses for the same three parts of a
/// polynomial in one unknown, because they are the same three parts.
struct HowASurfaceIsWritten {
    /// `M`, the part that squares `p`. Symmetric, and all nine stored rather
    /// than the six that are different, because nine is what a comparison reads
    /// and there is no arithmetic here to save.
    squared: [[Exact; 3]; 3],
    /// `b`, which is **half** the part that is linear in `p`.
    half_linear: [Exact; 3],
    /// `c`, the part with no `p` in it.
    constant: Exact,
}

/// A quadric written out, or `None` for a surface that is not one — a torus,
/// whose own expression is a quartic, or a plane.
///
/// A plane is answered above and never arrives here. `None` rather than a panic,
/// because the reason it never arrives is that planes sort first — a fact about
/// [`Surface::how_it_sorts_against`] in another file, which is the trap
/// [`how_a_quadric_cuts_the_line`]'s own doc names.
///
/// `None` also where a product it needs falls out of `f64`'s range. That is a
/// refusal to say **whether the family applies**, so it hands the question on
/// rather than ending it.
fn how_a_surface_is_written(surface: &Surface) -> Option<HowASurfaceIsWritten> {
    match surface {
        Surface::Sphere(ball) => how_a_ball_is_written(ball),
        Surface::Cylinder(pipe) => how_a_pipe_is_written(pipe),
        Surface::Cone(cone) => how_a_cone_is_written(cone),
        Surface::Plane(_) | Surface::Torus(_) => None,
    }
}

/// `‖p − c‖² − R²`, which is [`Sphere`]'s own sign expression and not a second
/// spelling of it.
fn how_a_ball_is_written(ball: &Sphere) -> Option<HowASurfaceIsWritten> {
    let centre = ball.centre().coordinates();
    Some(HowASurfaceIsWritten {
        squared: core::array::from_fn(|row| {
            core::array::from_fn(|column| Exact::of(if row == column { 1.0 } else { 0.0 }))
        }),
        half_linear: core::array::from_fn(|axis| Exact::of(-centre[axis])),
        constant: Exact::dot_of(centre, centre)?
            .minus(&Exact::product_of(ball.radius(), ball.radius())?),
    })
}

/// `‖(p − o) × w‖² − r²(w·w)`, which is [`Cylinder`]'s own sign expression,
/// opened out as `(w·w)(p·p) − (p·w)²` plus what the axis point leaves.
///
/// **The radius is squared against `w·w`**, because everything else here is
/// degree two in the stored direction and `r²` on its own is degree zero. Scale
/// `w` by any number and every one of these nine, three and one coefficients
/// scales by the square of it, so the surface they name is the pipe of radius
/// exactly `r` whatever length the direction got stored at — which is the
/// surface [`SideOfSurface`](rcad_predicates::SideOfSurface) certifies for a
/// pipe since §118.10. Take the `w·w` away and the constant term alone stops
/// scaling, and what is written out is the pipe of radius `r/‖w‖` (§27.1).
fn how_a_pipe_is_written(pipe: &Cylinder) -> Option<HowASurfaceIsWritten> {
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let mut written = how_a_ruled_surface_is_written(along, origin, &Exact::of(1.0))?;
    written.constant = written.constant.minus(
        &Exact::product_of(pipe.radius(), pipe.radius())?.times(&Exact::dot_of(along, along)?)?,
    );
    Some(written)
}

/// `‖(p − apex) × w‖² − slope²((p − apex)·w)²`, which is [`Cone`]'s own sign
/// expression, opened out as `(w·w)(p·p) − (1 + slope²)(p·w)²` plus what the
/// apex leaves.
fn how_a_cone_is_written(cone: &Cone) -> Option<HowASurfaceIsWritten> {
    let along = cone.axis().direction().components();
    let apex = cone.apex().coordinates();
    let squeeze = Exact::of(1.0).plus(&Exact::product_of(cone.slope(), cone.slope())?);
    how_a_ruled_surface_is_written(along, apex, &squeeze)
}

/// `(w·w)(x·x) − squeeze·(x·w)²` for `x = p − place`, written out in `p`.
///
/// A pipe and a cone are the same expression with a different `squeeze` — one
/// for a pipe, `1 + slope²` for a cone — because `‖x × w‖²` is
/// `(w·w)(x·x) − (x·w)²` and a cone takes `slope²(x·w)²` off that as well.
fn how_a_ruled_surface_is_written(
    along: [f64; 3],
    place: [f64; 3],
    squeeze: &Exact,
) -> Option<HowASurfaceIsWritten> {
    let axis_squared = Exact::dot_of(along, along)?;
    let reach = Exact::dot_of(place, along)?;
    let squeezed_reach = squeeze.times(&reach)?;

    let mut squared = nothing_squared();
    for (row, entries) in squared.iter_mut().enumerate() {
        for (column, entry) in entries.iter_mut().enumerate() {
            let leaning = squeeze.times(&Exact::product_of(along[row], along[column])?)?;
            *entry = if row == column {
                axis_squared.minus(&leaning)
            } else {
                Exact::of(0.0).minus(&leaning)
            };
        }
    }

    let mut half_linear = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in half_linear.iter_mut().enumerate() {
        *entry = squeezed_reach
            .times(&Exact::of(along[axis]))?
            .minus(&axis_squared.times(&Exact::of(place[axis]))?);
    }

    Some(HowASurfaceIsWritten {
        squared,
        half_linear,
        constant: axis_squared
            .times(&Exact::dot_of(place, place)?)?
            .minus(&squeezed_reach.times(&reach)?),
    })
}

/// Nine zeroes, to be written over.
fn nothing_squared() -> [[Exact; 3]; 3] {
    core::array::from_fn(|_| core::array::from_fn(|_| Exact::of(0.0)))
}

/// Whether two quadrics square `p` the same way, so their difference is flat.
///
/// Nine exact comparisons and no arithmetic that can round: an expansion always
/// has a sign, so this always answers.
fn they_bend_the_same_way(mine: &HowASurfaceIsWritten, theirs: &HowASurfaceIsWritten) -> bool {
    mine.squared
        .iter()
        .zip(theirs.squared.iter())
        .all(|(row, other)| {
            row.iter()
                .zip(other.iter())
                .all(|(entry, against)| matches!(entry.minus(against).sign(), Decision::Zero))
        })
}

/// What a stored plane and a plane written out exactly have in common.
///
/// The line is boxed, and the trade is the opposite of the one
/// [`SurfaceMeet::Curves`](crate::SurfaceMeet) refuses: this is never the answer
/// a fuse hands about, only a step on the way to one, and the arm carrying the
/// line is followed by a degree-fourteen exact discriminant that costs far more
/// than an allocation. Widening the other two arms to two hundred bytes to save
/// it would be paying in the wrong place.
enum WhatTwoPlanesShare {
    /// One endless line.
    ALine(Box<ALine>),
    /// The whole of the stored plane, because the two are one plane — or because
    /// the exact one is the equation `0 = 0`, which holds everywhere and is what
    /// two quadrics written with the same numbers come to.
    TheWholeOfTheStoredOne,
    /// Nothing at all: two parallel planes that are not the same one, or the
    /// equation `0 = d` for a `d` that is not zero, which two quadrics that
    /// differ only in size come to.
    Nothing,
}

/// What a stored plane and `normal·p = offset` share, or `None` when a product
/// it needs falls out of `f64`'s range.
///
/// The same shape as [`the_line_two_planes_cross_in`] and for the same reasons —
/// the zeroed coordinate is the one whose minor is largest, the ranking is
/// worked out in `f64` and only ranks, and that a minor is **not zero** is
/// decided exactly. What is different is the second plane's numbers being
/// [`Exact`] expansions, and what that buys is the end of it: two normals whose
/// every minor is exactly zero are parallel, decided, with no cascade to ask and
/// no policy to consult.
fn what_a_stored_plane_and_an_exact_one_share(
    flat: &Plane,
    normal: &[Exact; 3],
    offset: &Exact,
) -> Option<WhatTwoPlanesShare> {
    let mine = Exact::of_each(flat.normal().components());
    let my_offset = Exact::of(flat.offset());
    let theirs = normal;
    let their_offset = offset;

    let mut by_size = [0usize, 1, 2];
    let ranked: [f64; 3] = core::array::from_fn(|axis| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        mine[next].nearest_f64() * theirs[after].nearest_f64()
            - mine[after].nearest_f64() * theirs[next].nearest_f64()
    });
    by_size.sort_by(|&this, &that| ranked[that].abs().total_cmp(&ranked[this].abs()));

    let mut something_would_not_fit = false;
    for axis in by_size {
        let (across, and) = ((axis + 1) % 3, (axis + 2) % 3);
        let Some(minor) =
            exactly_this_minus_that(&mine[across], &theirs[and], &mine[and], &theirs[across])
        else {
            something_would_not_fit = true;
            continue;
        };
        let which_way = minor.sign();
        if matches!(which_way, Decision::Zero) {
            continue;
        }
        // An expansion always has a sign, so nothing reaches the second half
        // today. It is read as arithmetic that ran out rather than as a zero
        // minor, because falling through to the parallel test below on a sign
        // nobody proved would answer a question this never asked.
        if !matches!(which_way, Decision::Pos | Decision::Neg) {
            something_would_not_fit = true;
            continue;
        }
        let (Some(here), Some(there), Some(direction)) = (
            exactly_this_minus_that(&my_offset, &theirs[and], their_offset, &mine[and]),
            exactly_this_minus_that(&mine[across], their_offset, &theirs[across], &my_offset),
            cross_of_two_exact(&mine, theirs),
        ) else {
            something_would_not_fit = true;
            continue;
        };
        let mut through = [Term::of(0.0), Term::of(0.0), Term::of(0.0)];
        through[across] = Term::exact(here);
        through[and] = Term::exact(there);
        return Some(WhatTwoPlanesShare::ALine(Box::new(ALine {
            through,
            direction: direction.map(Term::exact),
            divided_by: Term::exact(minor),
        })));
    }
    if something_would_not_fit {
        return None;
    }

    // Parallel, so what tells the same plane from two that never meet is whether
    // the offsets are in the same ratio as the normals — `d₂·n₁ − d₁·n₂`,
    // componentwise (§17.4). A stored plane's normal has length, so where the
    // exact one's normal is all zeroes this reads `d₂·n₁`, which is zero exactly
    // when `d₂` is: the equation `0 = 0` holds everywhere and `0 = d₂` nowhere.
    for axis in 0..3 {
        let same = exactly_this_minus_that(their_offset, &mine[axis], &my_offset, &theirs[axis])?;
        let which_way = same.sign();
        if matches!(which_way, Decision::Zero) {
            continue;
        }
        if matches!(which_way, Decision::Pos | Decision::Neg) {
            return Some(WhatTwoPlanesShare::Nothing);
        }
        return None;
    }
    Some(WhatTwoPlanesShare::TheWholeOfTheStoredOne)
}

/// `first · second − third · fourth` over four expansions, or `None` when either
/// product would round.
fn exactly_this_minus_that(
    first: &Exact,
    second: &Exact,
    third: &Exact,
    fourth: &Exact,
) -> Option<Exact> {
    Some(first.times(second)?.minus(&third.times(fourth)?))
}

/// `u × v` for two triples of expansions, exactly.
fn cross_of_two_exact(u: &[Exact; 3], v: &[Exact; 3]) -> Option<[Exact; 3]> {
    let mut across = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, component) in across.iter_mut().enumerate() {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        *component = exactly_this_minus_that(&u[next], &v[after], &u[after], &v[next])?;
    }
    Some(across)
}

/// What a plane square to the one axis two curved surfaces share leaves of the
/// three, or `None` where the three are not that.
///
/// A ball, a pipe and a cone each look the same from every side of their axis —
/// and any line through a ball's centre is an axis of it — so a plane square to
/// the axis cuts each in a circle about the point where the axis crosses the
/// plane, or in that one point, or misses it. **Two circles about one centre
/// are one circle or share nothing**, so the whole of the answer is whether
/// their radii are equal: no quartic, and no point to place but the one a
/// radius of zero names. A bar of radius three through a ball of radius five,
/// and a cap across both at height two, is `3²` against `5² − 2²`, and they
/// share nothing.
///
/// Each radius is compared squared and times `n·n`, as
/// [`ACircleAboutAnAxis::squared_radius`] says, which keeps every one a
/// polynomial in the stored numbers whatever length the normal is stored at.
///
/// **Square, and one axis, are both decided exactly**: a pipe's or a cone's
/// stored direction crossed with the plane's normal is zero in every
/// component, and so is the gap between the two points on the axes crossed with
/// it. A direction a rounding off square cuts an ellipse, which is the quartic
/// again, and is handed on: where the ellipse and the other curve keep apart,
/// [`what_a_plane_that_cuts_them_in_bands_apart_leaves`] proves it, and where
/// they cross, as the cuts of a bar and a cone that share one circle on paper
/// do, [`what_a_plane_that_cuts_them_in_bands_that_cross_leaves`] finds the
/// crossings. A triple no road after this one takes is refused. So is a
/// triple where a product those two checks
/// need falls out of `f64`'s range, because that is a refusal to say whether
/// this road is the one — the same rule [`how_a_surface_is_written`] keeps.
///
/// Two radii of zero are the one point where the axis crosses the plane. Where
/// one of the two is a cone, that point is its apex, which is stored and so is
/// handed back as itself. Two balls that both touch the plane there have no
/// stored point to hand back, and go on to
/// [`what_a_plane_that_grazes_one_of_them_leaves`], which places a touch.
fn what_a_plane_square_to_their_one_axis_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
) -> Result<Option<MeetOfThree>> {
    let (Some(mine), Some(theirs)) = (
        the_circle_a_square_plane_cuts(flat, one),
        the_circle_a_square_plane_cuts(flat, other),
    ) else {
        return Ok(None);
    };
    let Some(off_one_axis) = Exact::cross_of(
        &Exact::gap_between(mine.on_the_axis, theirs.on_the_axis),
        flat.normal().components(),
    ) else {
        return Ok(None);
    };
    if !off_one_axis
        .iter()
        .all(|component| matches!(component.sign(), Decision::Zero))
    {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple, which a slide along the range can retry.
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let (Some(my_reach), Some(their_reach)) = (mine.squared_radius, theirs.squared_radius) else {
        return Err(refused());
    };
    let signs = [&my_reach, &their_reach, &my_reach.minus(&their_reach)].map(Exact::sign);
    if !signs
        .iter()
        .all(|sign| matches!(sign, Decision::Pos | Decision::Neg | Decision::Zero))
    {
        return Err(refused());
    }
    let [my_way, their_way, how_they_differ] = signs;
    if matches!(my_way, Decision::Neg)
        || matches!(their_way, Decision::Neg)
        || !matches!(how_they_differ, Decision::Zero)
    {
        return Ok(Some(MeetOfThree::Nowhere));
    }
    if matches!(my_way, Decision::Pos) {
        return Ok(Some(MeetOfThree::MoreThanAPoint));
    }
    Ok(mine
        .apex
        .or(theirs.apex)
        .map(|apex| MeetOfThree::At(smallvec![PredicatePoint::Exact(apex)])))
}

/// The circle a plane square to a ball's, a pipe's or a cone's axis cuts it in.
struct ACircleAboutAnAxis {
    /// A stored point on the axis: a ball's centre, a pipe's axis origin or a
    /// cone's apex.
    on_the_axis: [f64; 3],
    /// The circle's radius squared and times `n·n`, exactly, or `None` where a
    /// product it needs falls out of `f64`'s range.
    ///
    /// - A ball, `(n·n)R² − (n·c − d)²`, which is [`intersect`]'s own `room`
    ///   for a plane and a ball, and below zero where the plane misses it.
    /// - A pipe, `(n·n)r²`, because a pipe's radius is `r` whatever length its
    ///   direction is stored at (§118.10).
    /// - A cone, `slope²(n·a − d)²`: its radius at the plane is `slope` times
    ///   how far the plane is from the apex along the axis, and that distance
    ///   is `(n·a − d)/‖n‖`.
    squared_radius: Option<Exact>,
    /// The apex, where this is a cone: the one point a radius of zero is.
    apex: Option<[f64; 3]>,
}

/// The circle a plane square to `surface`'s axis cuts it in, or `None` where
/// the surface has no axis square to the plane — a pipe or a cone that leans,
/// a torus, a plane — or where whether its axis leans could not be worked out
/// exactly.
///
/// A torus about an axis square to the plane is cut in circles too, but in two
/// of them, and [`what_a_plane_square_to_a_rings_axis_leaves`] takes it.
fn the_circle_a_square_plane_cuts(flat: &Plane, surface: &Surface) -> Option<ACircleAboutAnAxis> {
    let normal = flat.normal().components();
    let square_to_the_plane = |along: [f64; 3]| it_runs_square_to(flat, along);
    match surface {
        Surface::Sphere(ball) => Some(ACircleAboutAnAxis {
            on_the_axis: ball.centre().coordinates(),
            squared_radius: how_wide_a_plane_cuts_a_ball(flat, ball),
            apex: None,
        }),
        Surface::Cylinder(pipe) => {
            square_to_the_plane(pipe.axis().direction().components()).then(|| ACircleAboutAnAxis {
                on_the_axis: pipe.axis().origin().coordinates(),
                squared_radius: Exact::dot_of(normal, normal).and_then(|squared_normal| {
                    squared_normal.times(&Exact::product_of(pipe.radius(), pipe.radius())?)
                }),
                apex: None,
            })
        }
        Surface::Cone(cone) => {
            square_to_the_plane(cone.axis().direction().components()).then(|| ACircleAboutAnAxis {
                on_the_axis: cone.apex().coordinates(),
                squared_radius: how_wide_a_square_plane_cuts_a_cone(flat, cone),
                apex: Some(cone.apex().coordinates()),
            })
        }
        Surface::Plane(_) | Surface::Torus(_) => None,
    }
}

/// `(n·n)R² − (n·c − d)²`, exactly, or `None` where a product falls out of
/// range.
fn how_wide_a_plane_cuts_a_ball(flat: &Plane, ball: &Sphere) -> Option<Exact> {
    let normal = flat.normal().components();
    let reach = how_far_a_plane_is_from(flat, ball.centre().coordinates())?;
    Some(
        Exact::dot_of(normal, normal)?
            .times(&Exact::product_of(ball.radius(), ball.radius())?)?
            .minus(&reach.times(&reach)?),
    )
}

/// `slope²(n·a − d)²`, exactly, or `None` where a product falls out of range.
fn how_wide_a_square_plane_cuts_a_cone(flat: &Plane, cone: &Cone) -> Option<Exact> {
    let reach = how_far_a_plane_is_from(flat, cone.apex().coordinates())?;
    Exact::product_of(cone.slope(), cone.slope())?.times(&reach.times(&reach)?)
}

/// `n·p − d`, exactly: how far `place` is in front of the plane, times `‖n‖`.
fn how_far_a_plane_is_from(flat: &Plane, place: [f64; 3]) -> Option<Exact> {
    Some(Exact::dot_of(flat.normal().components(), place)?.minus(&Exact::of(flat.offset())))
}

/// What a plane square to a ring's axis leaves of the ring and a ball, a pipe
/// or a cone on that axis, or `None` where the three are not that.
///
/// The plane cuts the ring in two circles about the point where the axis
/// crosses it, in one where it rests on the tube, or not at all, and it cuts
/// the other surface in one circle about that same point
/// ([`the_circle_a_square_plane_cuts`]). **Circles about one centre are one
/// circle or share nothing**, as in
/// [`what_a_plane_square_to_their_one_axis_leaves`], and no circle of the ring
/// is a point, since the tube never reaches the axis. So the three share a
/// whole circle or nothing, and there is no point to place: the top rim of a
/// bar of radius 5 under the top half of ring 4/1 is the ring's outer equator,
/// and `z = 0`, the bar and the ring share all of it.
///
/// **Which, is one exact sign.** With `h` how far the plane is from the ring's
/// centre, it cuts the tube where `s² = r² − h²` is not below nothing, in
/// circles of radii `R ± s`. The other circle's radius `ρ` is one of them
/// exactly where `(ρ − R)² = s²`, which is `ρ² + R² − s² = 2ρR`. Times `n·n`,
/// with `P = (n·n)ρ²` as [`ACircleAboutAnAxis::squared_radius`] gives it and
/// `H = n·c − d` for `c` the ring's centre, the left side is
///
/// `A = P + (n·n)(R² − r²) + H²`
///
/// and the right side is `2ρR(n·n)`. Where `P` is not below nothing, `A` is
/// above nothing, because [`Torus::new`] keeps `R > r`; so the two sides are
/// equal exactly where their squares are, `A² = 4P·(n·n)R²`. Where `P` is
/// below nothing the plane misses a ball, and the square on the right is below
/// nothing where `A²` never is. Where `s²` is below nothing the plane is past
/// the tube, and `(ρ − R)² = s²` holds for no `ρ`. So `A² − 4P·(n·n)R²` is
/// nothing exactly where the three share a circle, and otherwise they share
/// nothing.
///
/// **Square, and one axis, are decided exactly**, as
/// [`what_a_plane_square_to_their_one_axis_leaves`] decides them, and a ring
/// leaning a rounding off square is handed on. Two rings are handed on too.
fn what_a_plane_square_to_a_rings_axis_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
) -> Result<Option<MeetOfThree>> {
    let ((Surface::Torus(ring), round) | (round, Surface::Torus(ring))) = (one, other) else {
        return Ok(None);
    };
    if !it_runs_square_to(flat, ring.axis().direction().components()) {
        return Ok(None);
    }
    let Some(circle) = the_circle_a_square_plane_cuts(flat, round) else {
        return Ok(None);
    };
    let Some(off_one_axis) = Exact::cross_of(
        &Exact::gap_between(circle.on_the_axis, ring.centre().coordinates()),
        flat.normal().components(),
    ) else {
        return Ok(None);
    };
    if !off_one_axis
        .iter()
        .all(|component| matches!(component.sign(), Decision::Zero))
    {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple, which a slide along the range can retry.
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let Some(apart) = how_far_a_circle_is_from_a_rings(flat, ring, &circle) else {
        return Err(refused());
    };
    match apart.sign() {
        Decision::Zero => Ok(Some(MeetOfThree::MoreThanAPoint)),
        Decision::Pos | Decision::Neg => Ok(Some(MeetOfThree::Nowhere)),
        Decision::Undecidable { .. } => Err(refused()),
        other => unreachable!("an exact sign is a sign or a refusal: {other:?}"),
    }
}

/// `A² − 4P·(n·n)R²` for the circle a plane square to a ring's axis cuts
/// another surface in, as [`what_a_plane_square_to_a_rings_axis_leaves`]
/// writes it, exactly, or `None` where a product falls out of `f64`'s range.
fn how_far_a_circle_is_from_a_rings(
    flat: &Plane,
    ring: &Torus,
    circle: &ACircleAboutAnAxis,
) -> Option<Exact> {
    let normal = flat.normal().components();
    let squared_normal = Exact::dot_of(normal, normal)?;
    let round = circle.squared_radius.as_ref()?;
    let (core, tube) = (ring.ring_radius(), ring.tube_radius());
    let wide_core = squared_normal.times(&Exact::product_of(core, core)?)?;
    let wide_tube = squared_normal.times(&Exact::product_of(tube, tube)?)?;
    let height = how_far_a_plane_is_from(flat, ring.centre().coordinates())?;
    let left = round
        .plus(&wide_core)
        .minus(&wide_tube)
        .plus(&height.times(&height)?);
    let twice = round.times(&wide_core)?;
    let twice = twice.plus(&twice);
    Some(left.times(&left)?.minus(&twice.plus(&twice)))
}

/// Whether `along` is square to the plane, decided exactly: crossed with the
/// plane's normal it is nothing in every component. `false` where a product
/// that needs falls out of `f64`'s range.
fn it_runs_square_to(flat: &Plane, along: [f64; 3]) -> bool {
    exact_cross_of(flat.normal().components(), along).is_some_and(|across| {
        across
            .iter()
            .all(|component| matches!(component.sign(), Decision::Zero))
    })
}

/// What a plane that holds the one axis two curved surfaces are turned about
/// leaves of the three, or `None` where the three are not that.
///
/// A ball, a pipe and a cone about one axis are each one distance from it at
/// each height along it, so **two of them share whole circles square to the
/// axis and nothing else**, and a plane holding the axis cuts each such circle
/// in two points, at its radius either side of the axis. No quartic: the
/// heights are the roots of one quadratic, and the points at each height are a
/// square root out from the axis.
///
/// **The heights.** Measure a point `p` by `t = (p − o)·w` along the axis, for
/// `o` and `w` the stored apex and direction of the cone, or the stored origin
/// and direction of the pipe where neither is a cone, and by `ρ`, how far it is
/// from the axis. With `W = w·w`, each surface is `W·ρ² = g(t)`:
///
/// | surface | `g(t)` |
/// |---|---|
/// | a cone of slope `s` whose apex is at `t_a` | `s²(t − t_a)²` |
/// | a pipe of radius `r` | `r²W` |
/// | a ball of radius `R` whose centre is at `t_c` | `R²W − (t − t_c)²` |
///
/// A pipe is `r` wide whatever length its direction is stored at (§118.10),
/// and a slope is a ratio, so no other direction's length enters. The two
/// share a circle at each root of `g₁ − g₂`, a quadratic whose three numbers
/// are worked out exactly, and [`where_a_line_meets_a_quadric`] answers it as
/// it answers any quadratic in one unknown: nowhere, at one root or two, or at
/// every height alike, which is one surface written twice and so
/// [`MeetOfThree::MoreThanAPoint`].
///
/// **The points at a height.** The plane's normal `n` is square to `w`, so
/// `n × w` runs across the axis inside the plane and is `√((n·n)W)` long, and
/// the points at `t` are `o + (t·w ± √(G/(n·n))·(n × w))/W` for `G = g(t)`.
/// `G` is read off the cone, or the pipe where there is none, because neither
/// is ever below zero: a pipe's `r²W` is above it, and a cone's `s²t²`, with
/// `o` its apex, is zero at its apex alone. So whether the apex is a root is
/// decided exactly — the quadratic's constant term is zero — and there the
/// meet is the apex, stored, and one point and not two. Two cones of one apex
/// meet there and nowhere else, and a ball through a cone's apex meets it
/// there and at a second height.
///
/// **Every sign is exact and every point is a box**, a few roundings wide,
/// worked out in intervals from the root's own box. A cone of slope one and a
/// ball of radius 3 four up its axis meet at heights `2 ± √2/2`, which no
/// `f64` holds. A box of no width is the point it names and comes back as
/// [`PredicatePoint::Exact`]: a bar of radius 4 crosses that cone at `(0, ±4,
/// ±4)` in the wall `x = 0`.
///
/// **The order is one the three fix.** Lower heights first along `w` turned so
/// that its first component that is not zero is above zero, which is the same
/// for a cone stored with `w` and with `−w`, and for whichever of two cones `o`
/// is read from; and at each height the point on the `−(n × w)` side first.
///
/// **One axis, and the plane holding it, are decided exactly** — the stored
/// directions crossed are zero in every component, so is each stored point's
/// gap to `o` crossed with `w`, and so are `n·w` and `n·o − d` — and a product
/// those checks need falling out of range hands the question on, as
/// [`what_a_plane_square_to_their_one_axis_leaves`] does.
///
/// **A plane parallel to the axis but off it is not this road.** It cuts each
/// circle in a chord, touches it or misses it, and which of the three takes the
/// sign of `G·(n·n) − (n·o − d)²·W` at a root, decided exactly. It is handed
/// on, past the road for a plane that grazes, to
/// [`what_a_plane_beside_their_one_axis_leaves`]. So, after every other road,
/// is a plane that leans to the axis, which cuts each circle the same way.
fn what_a_plane_through_their_one_axis_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    let Some(axis) = the_axis_both_are_turned_about(one, other) else {
        return Ok(None);
    };
    if !it_holds_the_axis(flat, &axis) {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple, which a slide along the range can retry.
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let Some((heights, reach)) = the_heights_they_share_circles_at(one, other, &axis) else {
        return Err(refused());
    };
    let steps =
        match where_a_line_meets_a_quadric(WHERE_THREE_SURFACES_MEET, &heights, at, policy, log)? {
            WhereALineMeetsIt::Nowhere => return Ok(Some(MeetOfThree::Nowhere)),
            WhereALineMeetsIt::MoreThanAPoint => return Ok(Some(MeetOfThree::MoreThanAPoint)),
            WhereALineMeetsIt::At(steps) => steps,
        };

    let at_the_apex = which_height_is_the_apex(&axis, &heights, steps.len());
    let mut places: SmallVec<[PredicatePoint; 2]> = SmallVec::new();
    for (which, step) in steps.iter().enumerate() {
        let height = match step {
            MeetsAt::Here(height) => *height,
            MeetsAt::MaybeATouch { .. } => {
                unreachable!(
                    "a quadratic's roots are read off its discriminant, never searched for"
                )
            }
        };
        if at_the_apex == Some(which) {
            places.push(PredicatePoint::Exact(axis.through));
            continue;
        }
        places.extend(the_two_points_at(flat, &axis, &reach, height));
    }
    if places.iter().any(|place| !place.uncertainty().is_finite()) {
        return Err(refused());
    }
    Ok(Some(MeetOfThree::At(places)))
}

/// The one axis two curved surfaces are turned about, read off one of them.
struct AnAxis {
    /// The surface it is read off: the cone, or the pipe where neither is a
    /// cone, or the ring where one is
    /// ([`what_a_plane_through_a_rings_axis_leaves`]).
    read_off: Surface,
    /// That cone's apex, that pipe's axis origin, or that ring's centre.
    through: [f64; 3],
    /// That surface's stored direction, turned round where its first component
    /// that is not zero is below zero, which is exact.
    along: [f64; 3],
}

/// The axis `one` and `other` are both turned about, or `None` where there is
/// none to read off — two balls, a torus — where they are not proved to share
/// it, or where a product that proof needs falls out of range.
fn the_axis_both_are_turned_about(one: &Surface, other: &Surface) -> Option<AnAxis> {
    let a_cone = |surface: &&Surface| matches!(surface, Surface::Cone(_));
    let a_pipe = |surface: &&Surface| matches!(surface, Surface::Cylinder(_));
    let read_off = [one, other]
        .into_iter()
        .find(a_cone)
        .or_else(|| [one, other].into_iter().find(a_pipe))?;
    let axis =
        the_axis_read_off(read_off).expect("the axis is read off a cone or a pipe, just found");
    (it_is_turned_about(one, &axis) && it_is_turned_about(other, &axis)).then_some(axis)
}

/// The axis of a cone or a pipe, through its apex or its axis origin and
/// along its stored direction turned round where the first component that is
/// not zero is below zero, which is exact — or `None` for any other surface.
fn the_axis_read_off(surface: &Surface) -> Option<AnAxis> {
    let (through, along) = match surface {
        Surface::Cone(cone) => (cone.apex().coordinates(), cone.axis().direction()),
        Surface::Cylinder(pipe) => (pipe.axis().origin().coordinates(), pipe.axis().direction()),
        Surface::Plane(_) | Surface::Sphere(_) | Surface::Torus(_) => return None,
    };
    let along = along.components();
    let backwards = along
        .iter()
        .find(|component| **component != 0.0)
        .is_some_and(|component| *component < 0.0);
    Some(AnAxis {
        read_off: *surface,
        through,
        along: if backwards {
            along.map(|component| -component)
        } else {
            along
        },
    })
}

/// Whether `surface` is turned about `axis`, proved exactly: a cone's or a
/// pipe's direction runs along it and its stored point is on it, and a ball's
/// centre is on it.
fn it_is_turned_about(surface: &Surface, axis: &AnAxis) -> bool {
    let all_zero = |components: Option<[Exact; 3]>| {
        components.is_some_and(|components| {
            components
                .iter()
                .all(|component| matches!(component.sign(), Decision::Zero))
        })
    };
    let on_it = |place: [f64; 3]| {
        all_zero(Exact::cross_of(
            &Exact::gap_between(axis.through, place),
            axis.along,
        ))
    };
    let along_it = |direction: [f64; 3]| all_zero(exact_cross_of(direction, axis.along));
    match surface {
        Surface::Sphere(ball) => on_it(ball.centre().coordinates()),
        Surface::Cylinder(pipe) => {
            along_it(pipe.axis().direction().components())
                && on_it(pipe.axis().origin().coordinates())
        }
        Surface::Cone(cone) => {
            along_it(cone.axis().direction().components()) && on_it(cone.apex().coordinates())
        }
        Surface::Plane(_) | Surface::Torus(_) => false,
    }
}

/// Whether the plane holds the whole axis: `n·w` and `n·o − d` are both
/// exactly zero.
fn it_holds_the_axis(flat: &Plane, axis: &AnAxis) -> bool {
    let zero = |value: Option<Exact>| value.is_some_and(|value| value.sign() == Decision::Zero);
    zero(Exact::dot_of(flat.normal().components(), axis.along))
        && zero(how_far_a_plane_is_from(flat, axis.through))
}

/// `g₁ − g₂`, whose roots are the heights at which `one` and `other` share a
/// circle, and `g` of the surface `axis` is read off, both as
/// [`how_far_out_it_reaches`] writes them — or `None` where a product falls
/// out of range.
fn the_heights_they_share_circles_at(
    one: &Surface,
    other: &Surface,
    axis: &AnAxis,
) -> Option<(Quadratic, Quadratic)> {
    let mine = how_far_out_it_reaches(one, axis)?;
    let theirs = how_far_out_it_reaches(other, axis)?;
    let reach = how_far_out_it_reaches(&axis.read_off, axis)?;
    let heights = Quadratic {
        squared: mine.squared.minus(&theirs.squared),
        half_linear: mine.half_linear.minus(&theirs.half_linear),
        constant: mine.constant.minus(&theirs.constant),
    };
    Some((heights, reach))
}

/// `g(t)`, how far out from `axis` a ball, a pipe or a cone turned about it
/// reaches at each height, as `W·ρ²` — the table in
/// [`what_a_plane_through_their_one_axis_leaves`] — or `None` where a product
/// falls out of range.
///
/// # Panics
///
/// On a plane or a torus, which [`it_is_turned_about`] has already turned
/// away, and which [`what_a_plane_through_a_rings_axis_leaves`] hands on
/// before it asks.
fn how_far_out_it_reaches(surface: &Surface, axis: &AnAxis) -> Option<Quadratic> {
    let squared_axis = Exact::dot_of(axis.along, axis.along)?;
    let height_of = |place: [f64; 3]| {
        Exact::dot_of_two_exact(
            &Exact::gap_between(axis.through, place),
            &Exact::of_each(axis.along),
        )
    };
    let [squared, half_linear, constant] = match surface {
        Surface::Cone(cone) => {
            let squared_slope = Exact::product_of(cone.slope(), cone.slope())?;
            let apex = height_of(cone.apex().coordinates())?;
            let leaning = squared_slope.times(&apex)?;
            [
                squared_slope,
                Exact::of(0.0).minus(&leaning),
                leaning.times(&apex)?,
            ]
        }
        Surface::Cylinder(pipe) => [
            Exact::of(0.0),
            Exact::of(0.0),
            Exact::product_of(pipe.radius(), pipe.radius())?.times(&squared_axis)?,
        ],
        Surface::Sphere(ball) => {
            let centre = height_of(ball.centre().coordinates())?;
            [
                Exact::of(-1.0),
                centre.clone(),
                Exact::product_of(ball.radius(), ball.radius())?
                    .times(&squared_axis)?
                    .minus(&centre.times(&centre)?),
            ]
        }
        Surface::Plane(_) | Surface::Torus(_) => {
            panic!("{surface:?} is not turned about an axis this road reads")
        }
    };
    Some(Quadratic {
        squared: Term::exact(squared),
        half_linear: Term::exact(half_linear),
        constant: Term::exact(constant),
    })
}

/// Which of the `found` heights is the apex of the cone the axis is read off,
/// or `None` where the axis is a pipe's or the apex is not a root.
///
/// The apex is at `t = 0`, so it is a root exactly where `heights`' constant
/// term is zero. One root is then the apex. Of two, the other is `−2B/A`, so
/// the apex is the lower exactly where `A` and `B` differ in sign.
///
/// # Panics
///
/// If two roots come with `A` or `B` zero: two roots take a quadratic term,
/// and with no constant term a discriminant of `B²` above zero takes `B`.
fn which_height_is_the_apex(axis: &AnAxis, heights: &Quadratic, found: usize) -> Option<usize> {
    if !matches!(axis.read_off, Surface::Cone(_)) || heights.constant.sign() != Decision::Zero {
        return None;
    }
    if found == 1 {
        return Some(0);
    }
    let [squared, half_linear] = [&heights.squared, &heights.half_linear].map(Term::sign);
    assert!(
        found == 2
            && matches!(squared, Decision::Pos | Decision::Neg)
            && matches!(half_linear, Decision::Pos | Decision::Neg),
        "two roots of a quadratic with no constant term take A and B both off zero: \
         {squared:?}, {half_linear:?}, {found} roots"
    );
    Some(usize::from(squared == half_linear))
}

/// The two points at `height` where the plane through the axis crosses the
/// circle there, the `−(n × w)` side first, with `reach` the surface the axis
/// is read off as [`how_far_out_it_reaches`] writes it.
fn the_two_points_at(
    flat: &Plane,
    axis: &AnAxis,
    reach: &Quadratic,
    height: Interval,
) -> [PredicatePoint; 2] {
    let normal = flat.normal().components();
    let out_there = reach.squared.enclosure() * (height * height)
        + Interval::exactly(2.0) * reach.half_linear.enclosure() * height
        + reach.constant.enclosure();
    let across = root_enclosing(quotient_or_everything(
        out_there,
        dot_enclosing(exactly(normal), exactly(normal)),
    ));
    let sideways = cross_enclosing(exactly(normal), exactly(axis.along));
    let squared_axis = dot_enclosing(exactly(axis.along), exactly(axis.along));
    [-across, across].map(|side| {
        the_point_a_box_names(core::array::from_fn(|coordinate| {
            Interval::exactly(axis.through[coordinate])
                + quotient_or_everything(
                    height * Interval::exactly(axis.along[coordinate])
                        + side * sideways[coordinate],
                    squared_axis,
                )
        }))
    })
}

/// The point a box of no width names, and the box itself where it has width.
fn the_point_a_box_names(boxed: [Interval; 3]) -> PredicatePoint {
    if boxed.iter().all(|range| range.lo() == range.hi()) {
        // An interval's zero lower end reads back as `−0.0`, and adding `0.0`
        // writes it as the `0.0` a stored point would hold.
        PredicatePoint::Exact(boxed.map(|range| range.lo() + 0.0))
    } else {
        PredicatePoint::Constructed(boxed)
    }
}

/// What a plane that holds a ring's axis leaves of the ring and a ball, a pipe
/// or a cone on that axis, or `None` where the three are not that.
///
/// The ring and the other surface share whole circles square to the axis and
/// nothing else, at the heights [`the_heights_a_ring_crosses_a_round_surface_at`]
/// finds, which are the heights [`intersect`] names each circle by. A plane
/// holding the axis cuts each circle in two points, at its radius either side
/// of the axis, as it does in [`what_a_plane_through_their_one_axis_leaves`]:
/// a bend's end cap meets a bar of the bend's own radius on its axis where the
/// circles at the top and the bottom of the tube cross it, `(±4, 0, ±1)` for
/// ring four and tube one. No quartic: the heights are each a square root, and
/// the tube never reaches the axis, so every circle is wider than nothing and
/// cut twice.
///
/// **The radius at a height is read off the other surface**, as
/// [`how_far_out_it_reaches`] writes it about the axis through the ring's
/// centre along its stored direction. That direction is turned the way
/// [`the_axis_both_are_turned_about`] turns one, and the order is the one
/// [`what_a_plane_through_their_one_axis_leaves`] gives: lower heights first
/// along it, and at each height the `−(n × w)` side first. Every point is a
/// box worked out in intervals from its height's box.
///
/// **One axis, and the plane holding it, are decided exactly**: the plane by
/// [`it_holds_the_axis`], and the other surface by the questions
/// [`intersect`] asks before it names a circle. A ball or a cone that is not
/// on the ring's axis is not this road and is handed on, as is a second ring.
/// A pipe off the axis is answered here too:
/// [`the_heights_a_ring_crosses_a_round_surface_at`] names no heights for one
/// along the ring's axis, off it and clear of the tube, and this function
/// then answers [`MeetOfThree::Nowhere`] — but for a pipe of the tube's
/// radius along the ring's core circle, which
/// [`what_a_plane_through_a_rings_axis_leaves_of_a_pipe_along_its_core`]
/// answers.
fn what_a_plane_through_a_rings_axis_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    let ((Surface::Torus(ring), round) | (round, Surface::Torus(ring))) = (one, other) else {
        return Ok(None);
    };
    let along = ring.axis().direction().components();
    let backwards = along
        .iter()
        .find(|component| **component != 0.0)
        .is_some_and(|component| *component < 0.0);
    let axis = AnAxis {
        read_off: Surface::Torus(*ring),
        through: ring.centre().coordinates(),
        along: if backwards {
            along.map(|component| -component)
        } else {
            along
        },
    };
    if !it_holds_the_axis(flat, &axis) {
        return Ok(None);
    }
    if let Surface::Cylinder(pipe) = round
        && a_pipe_lies_along_a_rings_core(ring, pipe, policy, log)?
    {
        return what_a_plane_through_a_rings_axis_leaves_of_a_pipe_along_its_core(
            flat, ring, pipe, at, policy, log,
        )
        .map(Some);
    }
    let mut heights = match the_heights_a_ring_crosses_a_round_surface_at(ring, round, policy, log)
    {
        Ok(heights) => heights,
        Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface) => return Ok(None),
        Err(refused) => return Err(refused),
    };
    if heights.is_empty() {
        return Ok(Some(MeetOfThree::Nowhere));
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple, which a slide along the range can retry.
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let Some(reach) = how_far_out_it_reaches(round, &axis) else {
        return Err(refused());
    };
    if backwards {
        heights = heights.iter().map(|height| -*height).collect();
    }
    heights.sort_by(|one, other| one.lo().total_cmp(&other.lo()));
    let apart = heights.windows(2).all(|pair| pair[0].hi() < pair[1].lo());
    if !apart {
        return Err(refused());
    }
    let places: SmallVec<[PredicatePoint; 2]> = heights
        .iter()
        .flat_map(|&height| the_two_points_at(flat, &axis, &reach, height))
        .collect();
    if places.iter().any(|place| !place.uncertainty().is_finite()) {
        return Err(refused());
    }
    Ok(Some(MeetOfThree::At(places)))
}

/// What a plane holding a ring's axis leaves of the ring and a pipe of its
/// tube's radius along its core circle ([`a_pipe_lies_along_a_rings_core`]):
/// **more than a point where the plane is square to the pipe, and up to four
/// points where it is not.**
///
/// The ring and the pipe share a meridian circle and a loop ([`intersect`]).
/// The circle lies in the plane through the ring's centre square to the pipe,
/// and that plane cuts the pipe in the circle alone, so a bend's cap, the
/// bend's ring and the pipe of a bar butted onto the cap share the cap's whole
/// rim. The plane holds the ring's centre already ([`it_holds_the_axis`]).
///
/// **Any other plane holding the axis cuts the loop and misses the circle**,
/// since it meets the square plane only along the axis and the circle stays
/// `R − r` clear of that. In [`intersect`]'s frame the loop is where the pipe
/// meets `y² = 4R² − 4Rx`, a parabola swept along the axis, so the plane cuts
/// that in two lines along the axis, and each line meets the pipe in up to two
/// points. With `c` the ring's centre, `o` and `d` the pipe's point and
/// direction, `M = (d·d)(o − c) − ((o − c)·d)d` the way out to where the pipe
/// touches the core circle times `d·d`, and `n` the plane's normal, the line
/// through `c + M/(d·d) − s²M/4R² + s·d` is one of them where
///
/// `(n·M)(d·d)·s² − 4R²(n·d)(d·d)·s − 4R²(n·M) = 0`.
///
/// `n·M` is zero exactly where the plane is square to the pipe, so that one
/// sign tells the two cases apart, and otherwise the discriminant is a sum of
/// two squares with the second above zero: always two lines. A bar of ring 3/1
/// with its far end on `x + y = 0` is that plane, `s = 6(√2 − 1)`, and the
/// loop crosses it at `x = −y = s`, `z = ±√(1 − s⁴/144)`
/// (`where_a_plane_through_a_rings_axis_cuts_the_loop_a_pipe_along_its_core_makes`).
///
/// Each line is a box, so whether it meets the pipe is settled on intervals
/// ([`where_a_line_meets_a_quadric`]): the loop's tip exactly on the plane is
/// a touch no box tells from a near miss, refused under a certified policy.
///
/// # Errors
///
/// [`Error::Undecidable`] where a sign could not be settled, where a product
/// falls out of `f64`'s range, or where two of the points could not be told
/// apart.
fn what_a_plane_through_a_rings_axis_leaves_of_a_pipe_along_its_core(
    flat: &Plane,
    ring: &Torus,
    pipe: &Cylinder,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let loop_in_the_plane = how_a_plane_cuts_a_rings_loop(flat, ring, pipe).ok_or_else(refused)?;
    let square = &loop_in_the_plane.lines.squared;
    let sign = ask(
        &the_sign_of(WHERE_THREE_SURFACES_MEET, square),
        Measures::Length,
        at,
        policy,
        log,
    )?;
    match sign {
        Decision::Zero => return Ok(MeetOfThree::MoreThanAPoint),
        Decision::Pos | Decision::Neg => {}
        Decision::Undecidable { .. } => {
            unreachable!("`ask` hands an undecided sign back as an error")
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
    let mut places: SmallVec<[PredicatePoint; 2]> = SmallVec::new();
    for step in the_two_places_along_it(&loop_in_the_plane.lines, sign) {
        let line = loop_in_the_plane.the_line_along_the_axis_at(ring, pipe, step);
        let across_the_pipe =
            how_a_quadric_cuts_the_line(&line, &Surface::Cylinder(*pipe)).ok_or_else(refused)?;
        let found = where_a_line_meets_a_quadric(
            WHERE_THREE_SURFACES_MEET,
            &across_the_pipe,
            at,
            policy,
            log,
        )?;
        match the_points_those_steps_name(
            WHERE_THREE_SURFACES_MEET,
            &line,
            found,
            at,
            policy,
            log,
            refused,
        )? {
            MeetOfThree::At(more) => places.extend(more),
            MeetOfThree::Nowhere => (),
            MeetOfThree::MoreThanAPoint => unreachable!(
                "a line along the ring's axis runs square to the pipe, so never down its wall"
            ),
        }
    }
    let apart = places.iter().enumerate().all(|(index, one)| {
        places[index + 1..]
            .iter()
            .all(|other| two_boxes_are_apart(one.enclosure(), other.enclosure()))
    });
    if !apart {
        return Err(refused());
    }
    if places.is_empty() {
        return Ok(MeetOfThree::Nowhere);
    }
    Ok(MeetOfThree::At(places))
}

/// The numbers [`what_a_plane_through_a_rings_axis_leaves_of_a_pipe_along_its_core`]
/// reads the two lines off.
struct ALoopInAPlane {
    /// `(n·M)(d·d)·s² − 4R²(n·d)(d·d)·s − 4R²(n·M)`, zero at each line.
    lines: Quadratic,
    /// `M`, the way out to where the pipe touches the core circle, times `d·d`.
    out_to_the_touch: [Term; 3],
    /// `d·d`.
    squared_along: Term,
    /// `R²`.
    squared_ring: Term,
}

impl ALoopInAPlane {
    /// The line along the ring's axis through `c + M/(d·d) − s²M/4R² + s·d`,
    /// with `s` the box `step`.
    fn the_line_along_the_axis_at(&self, ring: &Torus, pipe: &Cylinder, step: Interval) -> ALine {
        let centre = ring.centre().coordinates();
        let along = pipe.axis().direction().components();
        let squared_along = self.squared_along.enclosure();
        let four_squared_ring = Interval::exactly(4.0) * self.squared_ring.enclosure();
        let through = core::array::from_fn(|axis| {
            let touch = self.out_to_the_touch[axis].enclosure();
            Term::from_a_box(
                Interval::exactly(centre[axis]) + quotient_or_everything(touch, squared_along)
                    - quotient_or_everything(step * step * touch, four_squared_ring)
                    + step * Interval::exactly(along[axis]),
            )
        });
        ALine {
            through,
            direction: ring.axis().direction().components().map(Term::of),
            divided_by: Term::of(1.0),
        }
    }
}

/// The quadratic whose roots are the two lines along a ring's axis where a
/// plane holding the axis cuts the parabola the loop of a pipe along the
/// core circle lies on, or `None` where a product it needs falls out of
/// `f64`'s range.
fn how_a_plane_cuts_a_rings_loop(
    flat: &Plane,
    ring: &Torus,
    pipe: &Cylinder,
) -> Option<ALoopInAPlane> {
    let centre = ring.centre().coordinates();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    let normal = flat.normal().components();
    let off_the_centre: [Term; 3] =
        core::array::from_fn(|axis| Term::of(origin[axis]).minus(&Term::of(centre[axis])));
    let direction = along.map(Term::of);
    let squared_along = Term::Stored(Nearly::dot_of(along, along)?);
    let reach_along = dot_of(&off_the_centre, &direction)?;
    let mut out_to_the_touch = [Term::of(0.0), Term::of(0.0), Term::of(0.0)];
    for (axis, touch) in out_to_the_touch.iter_mut().enumerate() {
        *touch = squared_along
            .times(&off_the_centre[axis])?
            .minus(&reach_along.times(&direction[axis])?);
    }
    let normal_out = dot_of(&normal.map(Term::of), &out_to_the_touch)?;
    let normal_along = Term::Stored(Nearly::dot_of(normal, along)?);
    let squared_ring = Term::Stored(Nearly::product_of(ring.ring_radius(), ring.ring_radius())?);
    let four_squared_ring = Term::of(4.0).times(&squared_ring)?;
    let lines = Quadratic {
        squared: normal_out.times(&squared_along)?,
        half_linear: Term::of(-2.0)
            .times(&squared_ring)?
            .times(&normal_along)?
            .times(&squared_along)?,
        constant: Term::of(0.0).minus(&four_squared_ring.times(&normal_out)?),
    };
    let coefficients = [&lines.squared, &lines.half_linear, &lines.constant];
    if coefficients
        .iter()
        .any(|term| term.lost_more_than_a_rounding())
    {
        return None;
    }
    Some(ALoopInAPlane {
        lines,
        out_to_the_touch,
        squared_along,
        squared_ring,
    })
}

/// Whether two boxes share no point: apart along at least one axis.
fn two_boxes_are_apart(one: [Interval; 3], other: [Interval; 3]) -> bool {
    (0..3).any(|axis| one[axis].hi() < other[axis].lo() || other[axis].hi() < one[axis].lo())
}

/// What a plane that misses one of the two curved surfaces, or touches it at a
/// single point, leaves of the three — or `None` where it does neither.
///
/// **A plane that misses a quadric settles the triple on its own**: nothing on
/// the plane is on that quadric, so nothing is on all three. And a plane that
/// touches one at a single point leaves exactly one candidate, so the whole of
/// what is left is which side of the *other* surface that point is on. Neither
/// needs a quartic and neither is a guess.
///
/// A ball resting on a block's floor is the second: the floor touches the ball
/// at one point, and whether a bar also resting on that floor passes through it
/// is one sign question. It is answered the same way at every distance, because
/// nothing in it depends on how far apart the two round solids are.
///
/// [`intersect`] refusing is not passed on. It says this road could not be
/// walked, which is a different thing from the triple having no answer, and the
/// other surface is still worth asking about. What is passed on is the refusal
/// at the end of every road, which was the answer before this road existed.
fn what_a_plane_that_grazes_one_of_them_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    for (this, that) in [(one, other), (other, one)] {
        match intersect(&Surface::Plane(*flat), this, policy, log) {
            Ok(SurfaceMeet::Nothing) => return Ok(Some(MeetOfThree::Nowhere)),
            Ok(SurfaceMeet::TouchAtOnePoint { at: touch }) => {
                return whether_a_touch_is_on_it(touch, that, at).map(Some);
            }
            Ok(SurfaceMeet::Curves(_) | SurfaceMeet::SameSurface) | Err(_) => (),
        }
    }
    Ok(None)
}

/// Whether the one point a plane touches a surface at is on a second surface
/// too.
///
/// **Certified and never tolerant**, and that is the one place in this file
/// where the caller's policy does not reach.
/// [`SideOfSurface`](rcad_predicates::SideOfSurface) hands back a
/// [`Decision`] and not the [`Level`] it reached, so there is no [`Record`] to
/// put in front of a policy and nothing to convert against its budget. Refusing
/// is the conservative half of that: a tolerant policy only ever accepts more,
/// so a refusal here is never an answer given wrongly — only one not given.
/// Closing it takes a `decide`-shaped route into a surface's own sign
/// expression, which is a change to another crate.
fn whether_a_touch_is_on_it(
    touch: PredicatePoint,
    surface: &Surface,
    at: &[PredicatePoint],
) -> Result<MeetOfThree> {
    match which_side_of_it(surface, touch) {
        Decision::Zero => Ok(MeetOfThree::At(smallvec![touch])),
        Decision::Pos | Decision::Neg => Ok(MeetOfThree::Nowhere),
        Decision::Undecidable { enclosure } => Err(a_touching_point_could_not_be_placed(
            WHERE_THREE_SURFACES_MEET,
            at,
            enclosure,
        )),
        other => unreachable!("a side-of question answers with one of the four: {other:?}"),
    }
}

/// Which side of a surface a point is on, for every kind there is.
fn which_side_of_it(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => which_side_of_a_flat_surface(
            WHERE_THREE_SURFACES_MEET,
            flat.normal().components(),
            flat.offset(),
            point,
        ),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        Surface::Cone(cone) => cone.side_of(point),
        Surface::Torus(ring) => ring.side_of(point),
    }
}

/// The refusal for a touching point that could be put neither on the third
/// surface nor off it.
///
/// **Not the exact arithmetic running out.** Where a plane touches a quadric is
/// a quotient and rarely lands on an `f64` (§2.5), so the box has width and an
/// interval evaluation of the third surface's own expression over it straddles
/// zero. A **gap** where the touch is on the lattice — that case is answered,
/// and a ball on a block's floor is it — and what a box of a point honestly
/// comes to where it is not.
fn a_touching_point_could_not_be_placed(
    named: &'static str,
    at: &[PredicatePoint],
    enclosure: f64,
) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        named,
        at,
        Level::Interval,
        enclosure,
        Measures::Area,
    ));
    Error::Undecidable { records }
}

/// What a plane that runs beside the one axis two curved surfaces are turned
/// about leaves of the three, or `None` where the three are not that or
/// `which` does not take this plane.
///
/// A D-flat ground along a shaft is one. The flat runs along the shaft's axis
/// and clear of it, and crosses the chamfer cone at the shaft's end, or a ball
/// on its axis, where the two round surfaces share a circle. Placed by a turn,
/// the flat comes out a rounding off parallel to the axis; a cut at half a
/// right angle across the circle a chamfer cone crosses a shaft in leans to
/// the axis on purpose. Both cut the same circles the same way.
///
/// **The circles are the ones a plane through the axis cuts**, at the roots of
/// `g₁ − g₂` that [`what_a_plane_through_their_one_axis_leaves`] works out,
/// with the same `o`, `w`, `W` and `g`. For the plane's normal `n` write
/// `N = n·n`, `k = n·w`, `e = n·o − d` and `A = N·W − k²`, which is
/// `|n × w|²`. At a height `t` whose circle is `W·ρ² = G` the plane misses the
/// circle, touches it or cuts it in a chord as `K = A·G − (e·W + k·t)²` is
/// below, at or above zero, and the chord's ends are
/// `o + ((t·N + e·k)·w − (e·W + t·k)·n)/A ± (√(K/W)/A)·(n × w)`: the foot,
/// where the plane comes nearest the circle's centre in the circle's own
/// plane, and `√(K/W)/A` along `n × w` either side of it. A touch is the foot
/// alone. A plane parallel to the axis has `k = 0`, and there `K` is
/// `W·(N·G − e²·W)` and the foot `o + t·w/W − (e/N)·n`.
///
/// **Which of the three is decided exactly, at the root itself and not at a
/// box round it.** `K` is `κ₂t² + 2κ₁t + κ₀` for `κ = A·g`, with `k²` taken
/// off `κ₂`, `e·W·k` off `κ₁` and `(e·W)²` off `κ₀`, and
/// [`the_sign_of_the_chord_at`] puts the root into it with nothing rounded: a
/// root that is a fraction as the fraction, and one that is a square root —
/// which is where a cone meets a ball — through the one comparison of two
/// squares that tells a chord from a touch. The plane `x + y = 2` touches both
/// circles a cone of slope one and a ball of radius two about its apex share,
/// at `(1, 1, ±√2)`, and the comparison there is of two numbers that are both
/// exactly zero.
///
/// **It is asked twice.** With [`BesideTheAxis::Parallel`] it takes a plane
/// with `k = 0`, after [`what_a_plane_that_grazes_one_of_them_leaves`], which
/// already answered a plane beside the axis that misses a pipe or a ball, or
/// touches one: those answers stand as they were, and this road takes what
/// that one hands on, a plane that cuts both surfaces in curves. The road
/// through the axis is asked before either, so `e` there is never zero.
///
/// With [`BesideTheAxis::Leaning`] it takes a plane with `k ≠ 0` and
/// `k² ≤ A`, no nearer square to the axis than to parallel, after every other
/// road, so no answer one of them gave moves. The foot moves `√(N/A)` per unit
/// of `t`, which grows without bound as the plane comes square, and the root
/// `t` is a box a rounding wide; so a plane nearer square is left to the band
/// roads, which give its points in narrower boxes. Asked ahead of them with no
/// limit on the lean, this road took two caps of
/// `tests/where_a_plane_and_two_round_surfaces_meet.rs` that lean off square
/// and gave their points in far wider boxes than those roads do.
///
/// Arithmetic that runs out after the family is settled is
/// [`an_exact_step_could_not_be_taken`], a **gap**: the points are there, and
/// the same three slid along the range are asked again.
fn what_a_plane_beside_their_one_axis_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    which: BesideTheAxis,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    let Some(axis) = the_axis_both_are_turned_about(one, other) else {
        return Ok(None);
    };
    let Some(slant) = how_the_plane_sits_to(flat, &axis) else {
        return Ok(None);
    };
    let signs = [&slant.leaning, &slant.off_it, &slant.across].map(Exact::sign);
    if !signs
        .iter()
        .all(|sign| matches!(sign, Decision::Pos | Decision::Neg | Decision::Zero))
    {
        return Ok(None);
    }
    let [leaning, off_it, across] = signs;
    let parallel = leaning == Decision::Zero;
    if parallel != (which == BesideTheAxis::Parallel) {
        return Ok(None);
    }
    assert!(
        !parallel || off_it != Decision::Zero,
        "a plane holding the axis is answered by the road through it, asked before this one"
    );
    // `across` is `|n × w|²`, zero only for a plane square to the axis. That
    // road is asked first and answers every such plane whose products are in
    // range, so one that reaches here is handed on as it was handed on there.
    if across == Decision::Zero {
        return Ok(None);
    }
    if !parallel && !slant.no_nearer_square_than_parallel() {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple, which a slide along the range can retry.
    let Some((heights, reach)) = the_heights_they_share_circles_at(one, other, &axis) else {
        return Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        ));
    };
    let chords = ChordsUpAnAxis {
        axis: &axis,
        slant,
        reach,
        heights,
    };
    the_ends_of_the_chords_at_the_roots(flat, &chords, one, at, policy, log).map(Some)
}

/// What a plane leaves of two curved surfaces where one wall is a mirror of
/// all three, or `None` where no wall is.
///
/// A cut steeper than half a right angle to a bar and its chamfer cone is
/// one; so is a bar leaning to the floor with a ball on its axis, whose stored
/// direction rounds and leaves the ball's centre a rounding off it; and so is
/// a ball resting on a rod, cut leaning through where the two touch.
///
/// **The wall.** Read the axis `o + s·w` off a pipe or a cone `X` of the two,
/// and write `m = n × w` for the plane's normal `n`. The wall `m·p = m·o`
/// holds the axis and the plane's normal, so it is a mirror of `X` and of the
/// plane. It is a mirror of the other surface `Y` as well where that is a ball
/// whose centre is on the wall, or a pipe or a cone whose axis lies in it,
/// each proved with exact products: `m·(c − o) = 0`, or `m·w' = 0` and
/// `m·(o' − o) = 0`. A plane square to the axis has `m = 0` and no wall.
///
/// **On the plane the two differ by a function of the height alone.** Write
/// `Q_X` and `Q_Y` as [`how_a_surface_is_written`] writes them, `W = w·w`, `λ`
/// one for a ball and `w'·w'` for a pipe or a cone, and `D = λ·Q_X − W·Q_Y`.
/// Along `m`, `Q_X` squares by `W·|m|²` and `Q_Y` by `λ·|m|²`, since `m` is
/// square to both axes, so `D` has no square along `m`. The mirror takes `D`
/// to itself and the plane to itself, so on the plane `D` has no odd power of
/// the distance along `m` either. So on the plane `D` is `F(t)`, a quadratic
/// in the height `t = (p − o)·w` alone, and `t` runs right across the plane:
/// its other direction, `(n·n)w − (n·w)n`, rises by `|m|²` up the axis.
///
/// **`F` is worked out on one line of the plane.** Take a component `m_j` of
/// `m` that is not zero, and write `q = e_j × n`, `r = e_j × w` and
/// `e = n·o − d`. Then `p(t) = o + (t·q + e·r)/m_j` is on the plane at height
/// `t`, since `n·q = 0`, `n·r = −m_j`, `w·q = m_j` and `w·r = 0`. For
/// `z = m_j·o + e·r` and `D` written `p·(S p) + 2b·p + c`, `m_j²·F(t)` is
/// `(q·S q)t² + 2(q·S z + m_j·b·q)t + z·S z + 2m_j·b·z + m_j²·c`, every number
/// exactly.
///
/// **The points are the ends of `X`'s chords at the roots of `F`**, worked out
/// as on the road beside two surfaces' one axis
/// ([`the_ends_of_the_chords_at_the_roots`]). A point of `X` on the plane at a
/// root has `D = 0` and `Q_X = 0`, so `W·Q_Y = 0` and it is on `Y`; and a
/// point of all three has `D = 0`, so its height is a root. Where `F` is
/// nothing at every height the plane cuts `X` and `Y` in one curve.
///
/// `X` is the first of the two that is a pipe or a cone and has such a wall,
/// and the points come lower heights up its axis first. Asked after every
/// other road, so no answer one of them gave moves. Arithmetic that runs out
/// once the wall is found is [`an_exact_step_could_not_be_taken`], a **gap**:
/// the points are there, and the same three slid along the range are asked
/// again.
fn what_a_plane_one_wall_mirrors_with_both_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    let Some((theirs, mirror)) =
        [(one, other), (other, one)]
            .into_iter()
            .find_map(|(mine, theirs)| {
                Some((theirs, the_wall_that_mirrors_all_three(flat, mine, theirs)?))
            })
    else {
        return Ok(None);
    };
    let (Some(slant), Some(reach), Some(heights)) = (
        how_the_plane_sits_to(flat, &mirror.axis),
        how_far_out_it_reaches(&mirror.axis.read_off, &mirror.axis),
        how_far_apart_they_are_up_the_cut(flat, &mirror, theirs),
    ) else {
        return Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        ));
    };
    assert_eq!(
        slant.across.sign(),
        Decision::Pos,
        "`(n·n)W − k²` is `|m|²`, worked out exactly, and `m` is not nothing"
    );
    let chords = ChordsUpAnAxis {
        axis: &mirror.axis,
        slant,
        reach,
        heights,
    };
    the_ends_of_the_chords_at_the_roots(flat, &chords, &mirror.axis.read_off, at, policy, log)
        .map(Some)
}

/// A wall that is a mirror of a plane and two curved surfaces, read off the
/// one of the two that is a pipe or a cone.
struct AMirror {
    /// That pipe's or cone's axis.
    axis: AnAxis,
    /// `m = n × w`, square to the wall, for `n` the plane's normal.
    square_to_it: [Exact; 3],
    /// `λ`: one where the other surface is a ball, and `w'·w'` for its stored
    /// direction where it is a pipe or a cone.
    weight: Exact,
}

/// The wall through the axis of `mine` and the normal of `flat`, where `mine`
/// is a pipe or a cone and the wall is a mirror of `theirs` as well — or
/// `None` where it is not, where the plane is square to the axis and there is
/// no such wall, or where a product the proof needs falls out of range.
fn the_wall_that_mirrors_all_three(
    flat: &Plane,
    mine: &Surface,
    theirs: &Surface,
) -> Option<AMirror> {
    let axis = the_axis_read_off(mine)?;
    let square_to_it = exact_cross_of(flat.normal().components(), axis.along)?;
    if square_to_it
        .iter()
        .all(|component| component.sign() == Decision::Zero)
    {
        return None;
    }
    let nothing = |value: Option<Exact>| value.is_some_and(|value| value.sign() == Decision::Zero);
    let on_it = |place: [f64; 3]| {
        nothing(Exact::dot_of_two_exact(
            &square_to_it,
            &Exact::gap_between(axis.through, place),
        ))
    };
    let weight = match theirs {
        Surface::Sphere(ball) => {
            if !on_it(ball.centre().coordinates()) {
                return None;
            }
            Exact::of(1.0)
        }
        Surface::Cylinder(_) | Surface::Cone(_) => {
            let their_axis = the_axis_read_off(theirs)?;
            let along_it = nothing(Exact::dot_of_two_exact(
                &square_to_it,
                &Exact::of_each(their_axis.along),
            ));
            if !(along_it && on_it(their_axis.through)) {
                return None;
            }
            Exact::dot_of(their_axis.along, their_axis.along)?
        }
        Surface::Plane(_) | Surface::Torus(_) => return None,
    };
    Some(AMirror {
        axis,
        square_to_it,
        weight,
    })
}

/// `m_j²·F(t)`, for `F(t)` what `λ·Q_X − W·Q_Y` comes to on `flat` at the
/// height `t` up the mirror's axis, in
/// [`what_a_plane_one_wall_mirrors_with_both_leaves`]'s names — or `None`
/// where a product falls out of range.
fn how_far_apart_they_are_up_the_cut(
    flat: &Plane,
    mirror: &AMirror,
    theirs: &Surface,
) -> Option<Quadratic> {
    let AMirror {
        axis,
        square_to_it,
        weight,
    } = mirror;
    let picked = (0..3)
        .find(|&component| square_to_it[component].sign() != Decision::Zero)
        .expect("a wall's normal is not nothing");
    let mut unit = [0.0; 3];
    unit[picked] = 1.0;
    let up_the_cut = exact_cross_of(unit, flat.normal().components())?;
    let off_the_axis = exact_cross_of(unit, axis.along)?;
    let scale = &square_to_it[picked];
    let off = how_far_a_plane_is_from(flat, axis.through)?;
    let apart = weighed_difference(
        &how_a_surface_is_written(&axis.read_off)?,
        weight,
        &how_a_surface_is_written(theirs)?,
        &Exact::dot_of(axis.along, axis.along)?,
    )?;
    let mut start = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (coordinate, entry) in start.iter_mut().enumerate() {
        *entry = scale
            .times(&Exact::of(axis.through[coordinate]))?
            .plus(&off.times(&off_the_axis[coordinate])?);
    }
    let bent_up = rows_times(&apart.squared, &up_the_cut)?;
    let bent_start = rows_times(&apart.squared, &start)?;
    let linear_up = scale.times(&Exact::dot_of_two_exact(&apart.half_linear, &up_the_cut)?)?;
    let linear_start = scale.times(&Exact::dot_of_two_exact(&apart.half_linear, &start)?)?;
    Some(Quadratic {
        squared: Term::exact(Exact::dot_of_two_exact(&up_the_cut, &bent_up)?),
        half_linear: Term::exact(
            Exact::dot_of_two_exact(&up_the_cut, &bent_start)?.plus(&linear_up),
        ),
        constant: Term::exact(
            Exact::dot_of_two_exact(&start, &bent_start)?
                .plus(&linear_start)
                .plus(&linear_start)
                .plus(&scale.times(scale)?.times(&apart.constant)?),
        ),
    })
}

/// `by·mine − against·theirs`, every coefficient exactly, or `None` where a
/// product falls out of range.
fn weighed_difference(
    mine: &HowASurfaceIsWritten,
    by: &Exact,
    theirs: &HowASurfaceIsWritten,
    against: &Exact,
) -> Option<HowASurfaceIsWritten> {
    let apart = |mine: &Exact, theirs: &Exact| Some(by.times(mine)?.minus(&against.times(theirs)?));
    let mut squared = nothing_squared();
    for (row, entries) in squared.iter_mut().enumerate() {
        for (column, entry) in entries.iter_mut().enumerate() {
            *entry = apart(&mine.squared[row][column], &theirs.squared[row][column])?;
        }
    }
    let mut half_linear = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in half_linear.iter_mut().enumerate() {
        *entry = apart(&mine.half_linear[axis], &theirs.half_linear[axis])?;
    }
    Some(HowASurfaceIsWritten {
        squared,
        half_linear,
        constant: apart(&mine.constant, &theirs.constant)?,
    })
}

/// The circles of one curved surface up its axis, a plane beside that axis,
/// and the heights up it at which the plane's points are to be looked for:
/// what [`the_ends_of_the_chords_at_the_roots`] is handed.
struct ChordsUpAnAxis<'a> {
    /// The axis, read off the surface whose circles the plane cuts.
    axis: &'a AnAxis,
    /// How the plane sits to it.
    slant: HowAPlaneSits,
    /// `g(t)` of the surface `axis` is read off, as
    /// [`how_far_out_it_reaches`] writes it.
    reach: Quadratic,
    /// A quadratic in the height `t`, whose roots are the heights.
    heights: Quadratic,
}

/// Where `flat` meets the circles of `chords`, at the roots of its heights:
/// at each root the plane misses the circle there, touches it at the foot, or
/// cuts it in a chord with two ends, as the sign of the chord at the root
/// says, worked out exactly ([`what_a_plane_beside_their_one_axis_leaves`]
/// has the arithmetic). The lower root's points come first, and the
/// `−(n × w)` end of a chord before the other.
///
/// Where the heights' quadratic is nothing at every height, the plane shares
/// with the three what it shares with `whole`.
///
/// Arithmetic that runs out is [`an_exact_step_could_not_be_taken`]: the
/// caller has settled that the three are its family, so it is a refusal about
/// this triple, which a slide along the range can retry.
fn the_ends_of_the_chords_at_the_roots(
    flat: &Plane,
    chords: &ChordsUpAnAxis,
    whole: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let ChordsUpAnAxis {
        axis,
        slant,
        reach,
        heights,
    } = chords;
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let steps =
        match where_a_line_meets_a_quadric(WHERE_THREE_SURFACES_MEET, heights, at, policy, log)? {
            WhereALineMeetsIt::Nowhere => return Ok(MeetOfThree::Nowhere),
            WhereALineMeetsIt::MoreThanAPoint => {
                return what_one_plane_and_one_curved_surface_share(flat, whole, policy, log);
            }
            WhereALineMeetsIt::At(steps) => steps,
        };
    // `steps` comes back ascending ([`WhereALineMeetsIt::At`]'s own promise),
    // and a negative `A` is what turns `(−B − √Δ)/A` above `(−B + √Δ)/A`
    // (`where_a_line_meets_a_quadric`'s note on `A`), so the two identities
    // below are swapped to match whichever root `the_two_places_along_it`
    // put first.
    let roots: SmallVec<[WhichRoot; 2]> = match (heights.squared.sign(), steps.len()) {
        (Decision::Zero, 1) => smallvec![WhichRoot::OfALine],
        (Decision::Pos | Decision::Neg, 1) => smallvec![WhichRoot::Twice],
        (Decision::Pos, 2) => smallvec![WhichRoot::TakingTheRootAway, WhichRoot::AddingTheRoot],
        (Decision::Neg, 2) => smallvec![WhichRoot::AddingTheRoot, WhichRoot::TakingTheRootAway],
        (squared, found) => unreachable!(
            "a quadratic has one root or two and a line has one, not {found} with A {squared:?}"
        ),
    };
    let (Some(chord), Some(squared), Some(half_linear), Some(constant)) = (
        how_wide_the_chord_is(axis, reach, slant),
        heights.squared.exactly(),
        heights.half_linear.exactly(),
        heights.constant.exactly(),
    ) else {
        return Err(refused());
    };
    let shared = [squared, half_linear, constant];

    // A height whose circle the plane misses is dropped here, and each that is
    // kept says whether the plane only touches its circle.
    let mut cuts: SmallVec<[(Interval, bool); 2]> = SmallVec::new();
    for (step, root) in steps.iter().zip(roots) {
        let height = match step {
            MeetsAt::Here(height) => *height,
            MeetsAt::MaybeATouch { .. } => {
                unreachable!(
                    "a quadratic's roots are read off its discriminant, never searched for"
                )
            }
        };
        let Some(how) = the_sign_of_the_chord_at(root, &chord, &shared) else {
            return Err(refused());
        };
        if how != Decision::Neg {
            cuts.push((height, how == Decision::Zero));
        }
    }
    // Lower heights first, as on the road through the axis — already true
    // here, since `steps` came back ascending and the loop above kept that
    // order.
    let mut places: SmallVec<[PredicatePoint; 2]> = SmallVec::new();
    for (height, it_only_touches) in cuts {
        places.extend(the_ends_of_the_chord_at(
            flat,
            axis,
            reach,
            slant,
            height,
            it_only_touches,
        ));
    }
    if places.iter().any(|place| !place.uncertainty().is_finite()) {
        return Err(refused());
    }
    if places.is_empty() {
        return Ok(MeetOfThree::Nowhere);
    }
    Ok(MeetOfThree::At(places))
}

/// Which root of the heights' quadratic `A t² + 2B t + C` a height is, in the
/// form [`where_a_line_meets_a_quadric`] worked it out in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhichRoot {
    /// `−C/(2B)`, the one root where `A` is zero.
    OfALine,
    /// `−B/A`, the one root where the discriminant `Δ = B² − AC` is zero.
    Twice,
    /// `(−B − √Δ)/A`.
    TakingTheRootAway,
    /// `(−B + √Δ)/A`.
    AddingTheRoot,
}

/// How a plane `n·p = d` sits to an axis `o + s·w`, each number exactly.
struct HowAPlaneSits {
    /// `k = n·w`, zero where the plane is parallel to the axis.
    leaning: Exact,
    /// `e = n·o − d`, how far the axis's stored point is in front of the
    /// plane, times `‖n‖`.
    off_it: Exact,
    /// `(n·n)W − k²`, which is `|n × w|²`: zero where the plane is square to
    /// the axis and above zero everywhere else.
    across: Exact,
}

impl HowAPlaneSits {
    /// Whether `k² ≤ A`, which is the plane making at most half a right angle
    /// with the axis: `k` is `‖n‖‖w‖` times the cosine of the angle between
    /// the normal and the axis, and `√A` the same times its sine. A height
    /// boxed `δ` wide then moves the foot by at most `√2·δ/√W`, since the foot
    /// moves `√(N/A)` per unit of `t` and `N·W = A + k²` is at most `2A`.
    fn no_nearer_square_than_parallel(&self) -> bool {
        let Some(squared) = self.leaning.times(&self.leaning) else {
            return false;
        };
        matches!(
            squared.minus(&self.across).sign(),
            Decision::Neg | Decision::Zero
        )
    }
}

/// Which planes beside their one axis a call of
/// [`what_a_plane_beside_their_one_axis_leaves`] takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BesideTheAxis {
    /// A plane parallel to the axis, asked where that road always was.
    Parallel,
    /// A plane leaning to the axis and no nearer square to it than parallel,
    /// asked after every other road, so no answer one of them gave moves.
    Leaning,
}

/// How `flat` sits to `axis`, or `None` where a product falls out of range.
fn how_the_plane_sits_to(flat: &Plane, axis: &AnAxis) -> Option<HowAPlaneSits> {
    let normal = flat.normal().components();
    let leaning = Exact::dot_of(normal, axis.along)?;
    let across = Exact::dot_of(normal, normal)?
        .times(&Exact::dot_of(axis.along, axis.along)?)?
        .minus(&leaning.times(&leaning)?);
    Some(HowAPlaneSits {
        leaning,
        off_it: how_far_a_plane_is_from(flat, axis.through)?,
        across,
    })
}

/// `κ₂`, `κ₁` and `κ₀` of `K(t) = A·g(t) − (e·W + k·t)²`, for
/// `A = (n·n)W − k²`, whose sign at a height says whether the plane misses,
/// touches or cuts the circle there — or `None` where a product falls out of
/// range.
fn how_wide_the_chord_is(
    axis: &AnAxis,
    reach: &Quadratic,
    slant: &HowAPlaneSits,
) -> Option<[Exact; 3]> {
    let HowAPlaneSits {
        leaning,
        off_it,
        across,
    } = slant;
    let level = off_it.times(&Exact::dot_of(axis.along, axis.along)?)?;
    let scaled = |term: &Term| across.times(&term.exactly()?);
    Some([
        scaled(&reach.squared)?.minus(&leaning.times(leaning)?),
        scaled(&reach.half_linear)?.minus(&level.times(leaning)?),
        scaled(&reach.constant)?.minus(&level.times(&level)?),
    ])
}

/// The sign of `k` at `root` of `A t² + 2B t + C`, for `chord` `k`'s three
/// numbers and `shared` the quadratic's, exactly — or `None` where a product
/// falls out of range.
///
/// - **A root that is a fraction `X/Y`** — `−C/(2B)`, or `−B/A` — is put in
///   over its denominator squared: `Y²·k(X/Y) = κ₂X² + 2κ₁XY + κ₀Y²`, which has
///   the sign of `k` there because `Y²` is above zero.
/// - **A root `(−B ± √Δ)/A`** is a square root. There `A·k − κ₂·(A t² + 2B t +
///   C)` has no `t²` left in it, and the quadratic is zero at the root, so
///   `A·k(t) = L·t + M` for `L = 2(A·κ₁ − κ₂·B)` and `M = A·κ₀ − κ₂·C`. Put the
///   root in and `A²·k(t) = P ± L·√Δ`, for `P = M·A − L·B`. Its sign is
///   [`the_sign_with_a_root_in_it`].
fn the_sign_of_the_chord_at(
    root: WhichRoot,
    chord: &[Exact; 3],
    shared: &[Exact; 3],
) -> Option<Decision> {
    let [squared, half_linear, constant] = chord;
    let [a, b, c] = shared;
    let nothing = Exact::of(0.0);
    let over_its_denominator = |over: &Exact, under: &Exact| -> Option<Exact> {
        let across = half_linear.times(over)?.times(under)?;
        Some(
            squared
                .times(over)?
                .times(over)?
                .plus(&across)
                .plus(&across)
                .plus(&constant.times(under)?.times(under)?),
        )
    };
    let sign = match root {
        WhichRoot::OfALine => over_its_denominator(&nothing.minus(c), &b.plus(b))?.sign(),
        WhichRoot::Twice => over_its_denominator(&nothing.minus(b), a)?.sign(),
        WhichRoot::TakingTheRootAway | WhichRoot::AddingTheRoot => {
            let half_the_lean = a.times(half_linear)?.minus(&squared.times(b)?);
            let lean = half_the_lean.plus(&half_the_lean);
            let level = a.times(constant)?.minus(&squared.times(c)?);
            let plain = level.times(a)?.minus(&lean.times(b)?);
            let rooted = if root == WhichRoot::AddingTheRoot {
                lean
            } else {
                nothing.minus(&lean)
            };
            let discriminant = b.times(b)?.minus(&a.times(c)?);
            the_sign_with_a_root_in_it(&plain, &rooted, &discriminant)?
        }
    };
    matches!(sign, Decision::Pos | Decision::Neg | Decision::Zero).then_some(sign)
}

/// The sign of `plain + rooted·√under`, exactly, for `under` not below zero —
/// or `None` where a sign is not one or a product falls out of range.
///
/// Where `plain` and `rooted` point the same way, or one of the two parts is
/// nothing, the sum points that way. Where they point opposite ways, the
/// larger part wins, and which is larger is the sign of `plain² − rooted²·under`;
/// the two equal is the sum at zero.
///
/// # Panics
///
/// If `under` is below zero: it is a discriminant the cascade has already
/// found above zero.
fn the_sign_with_a_root_in_it(plain: &Exact, rooted: &Exact, under: &Exact) -> Option<Decision> {
    let [mine, theirs, inside] = [plain, rooted, under].map(Exact::sign);
    if ![mine, theirs, inside]
        .iter()
        .all(|sign| matches!(sign, Decision::Pos | Decision::Neg | Decision::Zero))
    {
        return None;
    }
    assert!(
        inside != Decision::Neg,
        "a square root is only taken of a discriminant found above zero: {under:?}"
    );
    if theirs == Decision::Zero || inside == Decision::Zero || mine == theirs {
        return Some(mine);
    }
    if mine == Decision::Zero {
        return Some(theirs);
    }
    let larger = plain
        .square_minus(&rooted.times(rooted)?.times(under)?)?
        .sign();
    Some(match larger {
        Decision::Pos => mine,
        Decision::Neg => theirs,
        Decision::Zero => Decision::Zero,
        Decision::Undecidable { .. } => return None,
        other => unreachable!("an exact sign is a sign or a refusal: {other:?}"),
    })
}

/// Where the plane beside the axis meets the circle at `height`, which it
/// does not miss: the foot, where the plane comes nearest the circle's centre
/// in the circle's own plane, where it only touches the circle, and the
/// chord's two ends, the `−(n × w)` side first, where it cuts it.
fn the_ends_of_the_chord_at(
    flat: &Plane,
    axis: &AnAxis,
    reach: &Quadratic,
    slant: &HowAPlaneSits,
    height: Interval,
    it_only_touches: bool,
) -> SmallVec<[PredicatePoint; 2]> {
    let normal = flat.normal().components();
    let squared_normal = dot_enclosing(exactly(normal), exactly(normal));
    let squared_axis = dot_enclosing(exactly(axis.along), exactly(axis.along));
    let [leaning, off, across] =
        [&slant.leaning, &slant.off_it, &slant.across].map(Exact::enclosure);
    let level = off * squared_axis + height * leaning;
    let along = height * squared_normal + off * leaning;
    let foot: [Interval; 3] = core::array::from_fn(|coordinate| {
        Interval::exactly(axis.through[coordinate])
            + quotient_or_everything(
                along * Interval::exactly(axis.along[coordinate])
                    - level * Interval::exactly(normal[coordinate]),
                across,
            )
    });
    if it_only_touches {
        return smallvec![the_point_a_box_names(foot)];
    }
    let out_there = reach.squared.enclosure() * (height * height)
        + Interval::exactly(2.0) * reach.half_linear.enclosure() * height
        + reach.constant.enclosure();
    let half = quotient_or_everything(
        root_enclosing(quotient_or_everything(
            out_there * across - level * level,
            squared_axis,
        )),
        across,
    );
    let sideways = cross_enclosing(exactly(normal), exactly(axis.along));
    [-half, half]
        .into_iter()
        .map(|side| {
            the_point_a_box_names(core::array::from_fn(|coordinate| {
                foot[coordinate] + side * sideways[coordinate]
            }))
        })
        .collect()
}

/// What a plane leaves of the three where it cuts each of the two curved
/// surfaces in a circle — a ball anywhere, and a pipe or a cone whose axis the
/// plane is square to, each about an axis of its own — or `None` where the
/// three are not that.
///
/// A bar's flat end across a ball off its axis is one, and so is a floor that
/// a bar and a cone stand on side by side. A ball resting on that floor only
/// touches it, and [`what_a_plane_that_grazes_one_of_them_leaves`] answers
/// that first.
///
/// **Each surface is a circle on the plane.** Write the plane `n·p = d` and
/// `N = n·n`. A ball is `p·p + 2b·p + c` everywhere. A pipe or a cone about `w`
/// is `(w·w)(p·p) − σ(p·w)² + 2b·p + c`, with `σ` one for a pipe and
/// `1 + slope²` for a cone ([`how_a_ruled_surface_is_written`]). Where `w` is a
/// multiple of `n`, every point of the plane has `p·w = (w·n)d/N`, so there
/// `σ(p·w)²` is `σ(w·n)d(w·p)/N`, which is linear in `p`, and `N` times the
/// surface is `s(p·p) + l·p + k` with `s = N(w·w)`, `l = 2Nb − σ(w·n)d·w` and
/// `k = Nc`. A ball's is its own three numbers, with `s` one.
///
/// **Two circles in one plane differ by a line.** `s` is above zero for each,
/// so `s₂C₁ − s₁C₂` has no `p·p` in it: it is the flat surface
/// `(s₂l₁ − s₁l₂)·p = s₁k₂ − s₂k₁`, and on the plane a point of it and of the
/// first surface is a point of the second as well. That is the first family's
/// argument with circles in place of quadrics, and
/// [`where_a_stored_plane_an_exact_one_and_a_quadric_meet`] answers the two
/// planes and the first surface as it does there. The bar of radius two up `z`
/// and the ball of radius three at `(0, 3, 5)` differ on the cap `z = 5` by
/// `6y + 5z = 29`, so the three meet at `(±4√2/3, 2/3, 5)`.
///
/// **What it hands on.** A plane that leans to a pipe's or a cone's axis cuts
/// it in an ellipse, a parabola or a hyperbola, whose `p·p` part is not a
/// multiple of a circle's, so no `s₂C₁ − s₁C₂` takes the squares away and what
/// is left is the quartic the module doc names.
/// [`what_a_plane_that_cuts_them_in_bands_apart_leaves`] answers it where the
/// two curves keep apart, and the rest is refused. Two such conics of one shape
/// would differ by a line again — a mitre between two pipes of one radius cuts
/// both in one ellipse — and this road does not look for them.
///
/// It is asked after every road that takes two surfaces turned about one axis
/// but one, which answer a circle about one centre against another, and after
/// the road for a plane that grazes, so it takes only circles about two centres
/// or planes that cut both. The one is the leaning call of
/// [`what_a_plane_beside_their_one_axis_leaves`], asked last of all.
/// [`what_a_plane_along_a_pipe_leaves`] shares no triple with it: a pipe square
/// to the plane is not a pipe along it.
///
/// Arithmetic that runs out after the family is settled is
/// [`an_exact_step_could_not_be_taken`], a **gap**: the points are there.
fn what_a_plane_that_cuts_both_in_circles_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    if !(a_plane_cuts_it_in_a_circle(flat, one) && a_plane_cuts_it_in_a_circle(flat, other)) {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple and not a reason to try the next road.
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let (Some(mine), Some(theirs)) = (
        the_circle_a_plane_cuts(flat, one),
        the_circle_a_plane_cuts(flat, other),
    ) else {
        return Err(refused());
    };
    let Some((leaning, reach)) = the_line_two_circles_differ_by(&mine, &theirs) else {
        return Err(refused());
    };
    where_a_stored_plane_an_exact_one_and_a_quadric_meet(
        flat, &leaning, &reach, one, at, policy, log,
    )
    .map(|answer| Some(with_no_negative_zero(answer)))
}

/// `answer` with every `-0.0` in an exact corner made `+0.0`, which is the
/// same number.
///
/// Corners compare by value, where the two zeros are one, but a corner keyed
/// on its bits gets two keys for one point; `exactly_at` in `rcad-topo`'s
/// `fuse/assemble.rs` folds them for that reason. The line two circles differ
/// by hands `x = 0` back as `-0.0` for the floor across a bar and a ball off
/// its axis, so this road folds them before any key is taken. Adding `0.0`
/// turns `-0.0` into `+0.0` under round-to-nearest and leaves every other
/// number as it is. A box is left alone: `inari` hands the lower end of any
/// box that starts at zero back as `-0.0` whatever it was built from.
fn with_no_negative_zero(answer: MeetOfThree) -> MeetOfThree {
    let MeetOfThree::At(places) = answer else {
        return answer;
    };
    MeetOfThree::At(
        places
            .into_iter()
            .map(|place| match place.exact_coordinates() {
                Some(at) => PredicatePoint::Exact(at.map(|value| value + 0.0)),
                None => place,
            })
            .collect(),
    )
}

/// Whether `flat` cuts `surface` in a circle, or in nothing, about the point
/// an axis of it crosses the plane at: every plane cuts a ball so, and only a
/// plane square to its axis cuts a pipe or a cone so. A product that falls out
/// of range says no, and hands the triple on.
fn a_plane_cuts_it_in_a_circle(flat: &Plane, surface: &Surface) -> bool {
    let along = match surface {
        Surface::Sphere(_) => return true,
        Surface::Cylinder(pipe) => pipe.axis().direction().components(),
        Surface::Cone(cone) => cone.axis().direction().components(),
        Surface::Plane(_) | Surface::Torus(_) => return false,
    };
    exact_cross_of(flat.normal().components(), along).is_some_and(|across| {
        across
            .iter()
            .all(|component| component.sign() == Decision::Zero)
    })
}

/// `squared·(p·p) + linear·p + constant`, exactly: a surface on a plane that
/// cuts it in a circle, times a number above zero.
struct ACircle {
    squared: Exact,
    linear: [Exact; 3],
    constant: Exact,
}

/// The circle `flat` cuts `surface` in, as
/// [`what_a_plane_that_cuts_both_in_circles_leaves`] writes it, or `None`
/// where a product falls out of range.
///
/// # Panics
///
/// If `surface` is a plane or a ring: [`a_plane_cuts_it_in_a_circle`] said
/// no to both before this was asked.
fn the_circle_a_plane_cuts(flat: &Plane, surface: &Surface) -> Option<ACircle> {
    let (along, squeeze, written) = match surface {
        Surface::Sphere(ball) => {
            let written = how_a_ball_is_written(ball)?;
            return Some(ACircle {
                squared: Exact::of(1.0),
                linear: written.half_linear.map(|half| half.plus(&half)),
                constant: written.constant,
            });
        }
        Surface::Cylinder(pipe) => (
            pipe.axis().direction().components(),
            Exact::of(1.0),
            how_a_pipe_is_written(pipe)?,
        ),
        Surface::Cone(cone) => (
            cone.axis().direction().components(),
            Exact::of(1.0).plus(&Exact::product_of(cone.slope(), cone.slope())?),
            how_a_cone_is_written(cone)?,
        ),
        Surface::Plane(_) | Surface::Torus(_) => {
            unreachable!("only a ball, a pipe or a cone is cut in a circle: {surface:?}")
        }
    };
    let normal = flat.normal().components();
    let squared_normal = Exact::dot_of(normal, normal)?;
    let pulled = squeeze
        .times(&Exact::dot_of(along, normal)?)?
        .times(&Exact::of(flat.offset()))?;
    let mut linear = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in linear.iter_mut().enumerate() {
        let half = &written.half_linear[axis];
        *entry = squared_normal
            .times(&half.plus(half))?
            .minus(&pulled.times(&Exact::of(along[axis]))?);
    }
    Some(ACircle {
        squared: squared_normal.times(&Exact::dot_of(along, along)?)?,
        linear,
        constant: squared_normal.times(&written.constant)?,
    })
}

/// `s₂C₁ − s₁C₂ = 0` for two circles in one plane, as `leaning·p = reach`, or
/// `None` where a product falls out of range.
fn the_line_two_circles_differ_by(mine: &ACircle, theirs: &ACircle) -> Option<([Exact; 3], Exact)> {
    let mut leaning = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in leaning.iter_mut().enumerate() {
        *entry = theirs
            .squared
            .times(&mine.linear[axis])?
            .minus(&mine.squared.times(&theirs.linear[axis])?);
    }
    let reach = mine
        .squared
        .times(&theirs.constant)?
        .minus(&theirs.squared.times(&mine.constant)?);
    Some((leaning, reach))
}

/// What a plane that runs along a pipe — parallel to its axis, through it or
/// beside it — leaves of the three, where the other curved surface is a ball,
/// a second pipe or a cone, or `None` where the three are not that.
///
/// A wall that halves a bar, through a ball off the bar's axis, is one, and
/// so is the wall through the axes of two bars that cross.
///
/// **The plane cuts the pipe in lines along it.** For a pipe through `o`
/// along `w` of radius `r`, with `n·w = 0`, take `N = n·n`, `W = w·w`,
/// `E = N·W`, `e = n × w` and `δ = n·o − d`. Every point
/// `p = (V + u·e)/E + γ·w`, with `V = E·o − δW·n`, is on the plane, since
/// `n·V = E(n·o) − δE` and `n·e = 0`, and its squared distance from the axis
/// is `(δ²W + u²)/E`, since `e·e = E`. So it is on the pipe where `u² = K` for
/// `K = W(r²N − δ²)`: the plane misses the pipe, touches it along the one line
/// `u = 0`, or cuts it in the two lines `u = ∓√K`, as `K` is below, at or
/// above zero.
///
/// **Each line meets the other surface where a quadratic is zero.** Write the
/// other as `p·(Mp) + 2b·p + c`. `E²` times it along a line is
/// `E²A·γ² + 2Eβ·γ + Γ`, for `A = w·Mw`, `β = β₀ + β₁u` with
/// `β₀ = (Mw)·V + E(w·b)` and `β₁ = (Mw)·e`, and `Γ = γ₀ + 2γ₁u + γ₂u²` with
/// `γ₀ = V·(MV) + 2E(b·V) + E²c`, `γ₁ = e·(MV) + E(b·e)` and `γ₂ = e·(Me)`.
/// Its discriminant over `E²` is `β² − AΓ`, and with `u²` put back as `K` that
/// is `plain + lean·u`, for `plain = (β₁² − Aγ₂)K + β₀² − Aγ₀` and
/// `lean = 2(β₀β₁ − Aγ₁)`. [`the_sign_with_a_root_in_it`] decides its sign at
/// `u = ±√K` exactly, so whether a line misses the other surface, touches it
/// at `γ = −β/(EA)` or crosses it at `γ = (−β ∓ √(β² − AΓ))/(EA)` is never
/// read off a box. The bar of radius two up `z` meets the wall `x = 0` in
/// `y = ±2`, and the ball of radius three at `(0, 3, 5)` in `y = 2` at
/// `z = 5 ± 2√2` and in `y = −2` nowhere.
///
/// The line `u = −√K` comes first, and the lower `γ` first on each line.
///
/// **What it leaves to the refusal.** Where `A` is zero the pipe runs along
/// the other surface — a second pipe the same way, which the first family
/// answers, or a cone one of whose rulings runs along `w` — and the quadratic
/// on a line is a line's: this road hands that on. A plane that leans to the
/// pipe's axis is not along it and cuts it in an ellipse, which
/// [`what_a_plane_that_cuts_them_in_bands_apart_leaves`] is asked about next.
/// One that leans by a rounding and keeps off the pipe reaches it only far
/// along it, which [`what_a_plane_beside_a_pipe_leaves`] is asked about after
/// that. One that leans by a rounding and cuts a pipe where it shares circles
/// with the other surface about one axis — a D-flat on a chamfered shaft,
/// placed by a turn — is taken by the leaning call of
/// [`what_a_plane_beside_their_one_axis_leaves`], asked last of all.
///
/// It is asked after every road but those three and the one for bands that
/// cross, so every triple an earlier road answers keeps its answer. Arithmetic
/// that runs out after the family is settled is
/// [`an_exact_step_could_not_be_taken`], a **gap**: the points are there.
fn what_a_plane_along_a_pipe_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
) -> Result<Option<MeetOfThree>> {
    let normal = flat.normal().components();
    let runs_along = |pipe: &Cylinder| {
        Exact::dot_of(normal, pipe.axis().direction().components())
            .is_some_and(|leaning| leaning.sign() == Decision::Zero)
    };
    let Some((pipe, rest)) = [(one, other), (other, one)]
        .into_iter()
        .find_map(|(this, that)| match this {
            Surface::Cylinder(pipe) if runs_along(pipe) => Some((pipe, that)),
            Surface::Plane(_)
            | Surface::Sphere(_)
            | Surface::Cylinder(_)
            | Surface::Cone(_)
            | Surface::Torus(_) => None,
        })
    else {
        return Ok(None);
    };
    let Some(written) = how_a_surface_is_written(rest) else {
        return Ok(None);
    };
    let along = Exact::of_each(pipe.axis().direction().components());
    let Some(bend) = rows_times(&written.squared, &along)
        .and_then(|towards| Exact::dot_of_two_exact(&along, &towards))
    else {
        return Ok(None);
    };
    if !matches!(bend.sign(), Decision::Pos | Decision::Neg) {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple, which a slide along the range can retry.
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let Some(lines) = the_lines_a_plane_along_a_pipe_cuts(flat, pipe, &written, bend) else {
        return Err(refused());
    };
    let Some((plain, lean)) = lines.the_discriminant() else {
        return Err(refused());
    };
    let sides: SmallVec<[f64; 2]> = match lines.reach.sign() {
        Decision::Neg => return Ok(Some(MeetOfThree::Nowhere)),
        Decision::Zero => smallvec![0.0],
        Decision::Pos => smallvec![-1.0, 1.0],
        Decision::Undecidable { .. } => return Err(refused()),
        other => unreachable!("an exact sign is a sign or a refusal: {other:?}"),
    };
    let mut places: SmallVec<[PredicatePoint; 2]> = SmallVec::new();
    for side in sides {
        let rooted = if side < 0.0 {
            Exact::of(0.0).minus(&lean)
        } else {
            lean.clone()
        };
        let Some(crossing) = the_sign_with_a_root_in_it(&plain, &rooted, &lines.reach) else {
            return Err(refused());
        };
        match crossing {
            Decision::Neg => {}
            Decision::Zero => places.extend(lines.where_one_meets_it(side, &plain, &lean, true)),
            Decision::Pos => places.extend(lines.where_one_meets_it(side, &plain, &lean, false)),
            Decision::Undecidable { .. } => return Err(refused()),
            other => unreachable!("an exact sign is a sign or a refusal: {other:?}"),
        }
    }
    if places.iter().any(|place| !place.uncertainty().is_finite()) {
        return Err(refused());
    }
    if places.is_empty() {
        return Ok(Some(MeetOfThree::Nowhere));
    }
    Ok(Some(MeetOfThree::At(places)))
}

/// `M v` for a square of exact numbers and an exact triple, exactly.
fn rows_times(rows: &[[Exact; 3]; 3], value: &[Exact; 3]) -> Option<[Exact; 3]> {
    Some([
        Exact::dot_of_two_exact(&rows[0], value)?,
        Exact::dot_of_two_exact(&rows[1], value)?,
        Exact::dot_of_two_exact(&rows[2], value)?,
    ])
}

/// The lines a plane along a pipe cuts it in, `p = (V + u·e)/E + γ·w` for
/// `u² = K`, and `E²` times a second curved surface along them,
/// `E²A·γ² + 2E(β₀ + β₁u)·γ + (γ₀ + 2γ₁u + γ₂u²)`, every number exactly, in
/// [`what_a_plane_along_a_pipe_leaves`]'s names.
struct TheLinesAlongAPipe {
    /// `w`.
    along: [f64; 3],
    /// `E`.
    scale: Exact,
    /// `e`.
    sideways: [Exact; 3],
    /// `V`.
    through: [Exact; 3],
    /// `K`.
    reach: Exact,
    /// `A`.
    bend: Exact,
    /// `β₀` and `β₁`.
    half_linear: [Exact; 2],
    /// `γ₀`, `γ₁` and `γ₂`.
    constant: [Exact; 3],
}

/// The lines `flat` cuts `pipe` in, and `written` along them, or `None` where
/// a product falls out of range.
fn the_lines_a_plane_along_a_pipe_cuts(
    flat: &Plane,
    pipe: &Cylinder,
    written: &HowASurfaceIsWritten,
    bend: Exact,
) -> Option<TheLinesAlongAPipe> {
    let normal = flat.normal().components();
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let squared_normal = Exact::dot_of(normal, normal)?;
    let squared_axis = Exact::dot_of(along, along)?;
    let scale = squared_normal.times(&squared_axis)?;
    let sideways = exact_cross_of(normal, along)?;
    let off = how_far_a_plane_is_from(flat, origin)?;
    let reach = squared_axis.times(
        &Exact::product_of(pipe.radius(), pipe.radius())?
            .times(&squared_normal)?
            .minus(&off.times(&off)?),
    )?;
    let pulled = off.times(&squared_axis)?;
    let mut through = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in through.iter_mut().enumerate() {
        *entry = scale
            .times(&Exact::of(origin[axis]))?
            .minus(&pulled.times(&Exact::of(normal[axis]))?);
    }

    let w = Exact::of_each(along);
    let b = &written.half_linear;
    let dot = Exact::dot_of_two_exact;
    let towards = rows_times(&written.squared, &w)?;
    let from_through = rows_times(&written.squared, &through)?;
    let from_sideways = rows_times(&written.squared, &sideways)?;
    let b_through = dot(b, &through)?;
    let half_linear = [
        dot(&towards, &through)?.plus(&scale.times(&dot(&w, b)?)?),
        dot(&towards, &sideways)?,
    ];
    let constant = [
        dot(&through, &from_through)?
            .plus(&scale.times(&b_through.plus(&b_through))?)
            .plus(&scale.times(&scale)?.times(&written.constant)?),
        dot(&sideways, &from_through)?.plus(&scale.times(&dot(b, &sideways)?)?),
        dot(&sideways, &from_sideways)?,
    ];
    Some(TheLinesAlongAPipe {
        along,
        scale,
        sideways,
        through,
        reach,
        bend,
        half_linear,
        constant,
    })
}

impl TheLinesAlongAPipe {
    /// `β² − AΓ` with `u²` put back as `K`, as `(plain, lean)` for
    /// `plain + lean·u`, or `None` where a product falls out of range.
    fn the_discriminant(&self) -> Option<(Exact, Exact)> {
        let [plain_half, leaning_half] = &self.half_linear;
        let [plain_constant, leaning_constant, squared_constant] = &self.constant;
        let bend = &self.bend;
        let plain = leaning_half
            .times(leaning_half)?
            .minus(&bend.times(squared_constant)?)
            .times(&self.reach)?
            .plus(&plain_half.times(plain_half)?)
            .minus(&bend.times(plain_constant)?);
        let half_lean = plain_half
            .times(leaning_half)?
            .minus(&bend.times(leaning_constant)?);
        Some((plain, half_lean.plus(&half_lean)))
    }

    /// Where the line on `side` of the axis — `u = side·√K` — meets the other
    /// surface: once, at `−β/(EA)`, where it only touches it, and otherwise at
    /// `(−β ∓ √(β² − AΓ))/(EA)`, the lower `γ` first.
    fn where_one_meets_it(
        &self,
        side: f64,
        plain: &Exact,
        lean: &Exact,
        it_only_touches: bool,
    ) -> SmallVec<[PredicatePoint; 2]> {
        let across = Interval::exactly(side) * root_enclosing(self.reach.enclosure());
        let scale = self.scale.enclosure();
        let [plain_half, leaning_half] = self.half_linear.each_ref().map(Exact::enclosure);
        let half = plain_half + leaning_half * across;
        let under = scale * self.bend.enclosure();
        let steps: SmallVec<[Interval; 2]> = if it_only_touches {
            smallvec![quotient_or_everything(-half, under)]
        } else {
            let root = root_enclosing(plain.enclosure() + lean.enclosure() * across);
            let [taking, adding] =
                [-half - root, -half + root].map(|top| quotient_or_everything(top, under));
            // `E` is above zero, so a negative `A` is what turns
            // `(−β − √Δ)/(EA)` above `(−β + √Δ)/(EA)`.
            if self.bend.sign() == Decision::Neg {
                smallvec![adding, taking]
            } else {
                smallvec![taking, adding]
            }
        };
        steps
            .into_iter()
            .map(|step| {
                the_point_a_box_names(core::array::from_fn(|axis| {
                    quotient_or_everything(
                        self.through[axis].enclosure() + across * self.sideways[axis].enclosure(),
                        scale,
                    ) + step * Interval::exactly(self.along[axis])
                }))
            })
            .collect()
    }
}

/// What a plane leaves of a ball, a pipe or a cone and a second one where the
/// curves it cuts them in are proved apart, or `None` where that is not proved.
///
/// **Each curve lies in a band about where an axis crosses the plane.** Write
/// the plane `n·p = d`, and give each surface a stored point `o` on an axis and
/// that axis's direction `w`: a ball's centre and the plane's own normal, a
/// pipe's axis origin and direction, or a cone's apex and direction. The axis
/// crosses the plane at `c = o + w·m/k`, for `k = n·w` and `m = d − n·o`. Take
/// a point `p = c + v` of the plane and `t` the angle between `n` and `w`.
/// Since `v` is square to `n`, its part along the axis is at most `|v|·sin t`.
///
/// - A pipe of radius `r` holds `p` where `|v|² − (v·ŵ)² = r²`, so
///   `r ≤ |v| ≤ r / cos t`. A ball is the same with `t = 0`: a circle of
///   radius `ρ` about `c`, with `ρ² = (N R² − m²)/N` for `N = n·n`.
/// - A cone of slope `s` holds `p` where `|v|² − x² = s²(h + x)²`, with `x`
///   the part of `v` along the axis and `h = m √(w·w) / k` how far the plane
///   is from the apex along it. With `ρ = s|h|`, `|x| ≤ |v| sin t` gives
///   `ρ / (1 + s sin t) ≤ |v|`, and `|v| ≤ ρ / (cos t − s sin t)` where the
///   bottom is above nothing. Where it is not, the plane runs at least as
///   steep as a ruling and the cut has no outer edge.
///
/// So every point of each cut is between `ρ − ε⁻` and `ρ + ε⁺` from its `c`,
/// and the pipe's band is the cone's with `s = 0`, the ball's the pipe's with
/// `t = 0`. A point on both is `|c₁ − c₂|` at most nearer one centre than the
/// other, so the three share nothing where `ρ₂ − ε₂⁻ − ρ₁ − ε₁⁺` or
/// `ρ₁ − ε₁⁻ − ρ₂ − ε₂⁺` is above `|c₁ − c₂|`. The cap through `(0, 0, 5)`
/// with normal `(0, 1, 100)` cuts the bar of radius three up `z` within
/// `3.00015` of the axis and the cone of slope one from the origin no nearer
/// than `4.95`, and the three share nothing.
///
/// **Every number is proved and none is a tolerance.** On a cap one rounding
/// off square to a bar and a cone that it cuts in one circle on paper, `ρ₂ − ρ₁`,
/// both slacks and `|c₁ − c₂|` are each about a rounding, and so is the margin
/// between them. So nothing that cancels is left to the enclosures:
/// `ρ₂ − ρ₁` is `(P₂Q₁ − P₁Q₂) / (Q₁Q₂(ρ₁ + ρ₂))` for `ρ² = P/Q`, with the top
/// worked out exactly, and `c₁ − c₂` is `G / (k₁k₂)` for
/// `G = k₁k₂(o₁ − o₂) + k₂m₁w₁ − k₁m₂w₂`, exactly, so each comes out to a few
/// roundings of itself. `sin² t` is `|n × w|²/(N(w·w))` with the top exact,
/// and `1 − cos t` is `sin² t / (1 + cos t)`. No number here is past degree
/// twelve in the stored numbers, which [`Exact::times`] multiplies without
/// falling off the bottom of the range.
///
/// **What it leaves** is two bands that overlap, where the curves may cross,
/// to [`what_a_plane_that_cuts_them_in_bands_that_cross_leaves`]. It leaves
/// to the refusal a ring, a plane that misses a ball, and a plane parallel to
/// a pipe's or a cone's axis, since it gives none of those a band. A plane a
/// rounding off parallel to a pipe and a cone about one axis gets bands too
/// wide to keep
/// apart, the cone's with no outer edge, so the side of a block three from a
/// bar and its chamfer cone, the block and the bar placed by one turn, is
/// handed on, and [`what_a_plane_beside_a_pipe_leaves`] answers it.
///
/// It is asked after every road but the one for bands that cross, the one
/// for a plane beside a pipe and the second call of the one for a plane beside
/// two surfaces' one axis, and only ever answers that the three share
/// nothing, so a triple an earlier road answers keeps its answer, and where a
/// product falls out of range it hands the triple on.
fn what_a_plane_that_cuts_them_in_bands_apart_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
) -> Option<MeetOfThree> {
    let mine = the_band_a_plane_cuts(flat, one)?;
    let theirs = the_band_a_plane_cuts(flat, other)?;
    let between = how_far_apart_the_centres_of_two_bands_are(&mine, &theirs)?;
    let [my_top, my_bottom] = &mine.squared_radius;
    let [their_top, their_bottom] = &theirs.squared_radius;
    let squared_gap = their_top
        .times(my_bottom)?
        .minus(&my_top.times(their_bottom)?);
    let gap = quotient_or_everything(
        squared_gap.enclosure(),
        my_bottom.times(their_bottom)?.enclosure() * (mine.radius + theirs.radius),
    );
    let theirs_beyond_mine = gap - theirs.inside - mine.outside - between;
    let mine_beyond_theirs = -gap - mine.inside - theirs.outside - between;
    (theirs_beyond_mine.lo() > 0.0 || mine_beyond_theirs.lo() > 0.0).then_some(MeetOfThree::Nowhere)
}

/// The band a plane cuts a ball, a pipe or a cone in, in
/// [`what_a_plane_that_cuts_them_in_bands_apart_leaves`]'s names: every point
/// of the cut is between `radius − inside` and `radius + outside` from where
/// the axis crosses the plane.
struct ABand {
    /// `o`.
    on_the_axis: [f64; 3],
    /// `w`.
    along: [f64; 3],
    /// `k`, exactly, and above or below nothing.
    leaning: Exact,
    /// `m`, exactly.
    reach: Exact,
    /// `P` and `Q`, exactly, with `Q` above nothing.
    squared_radius: [Exact; 2],
    /// `ρ`.
    radius: Interval,
    /// `ε⁻`.
    inside: Interval,
    /// `ε⁺`.
    outside: Interval,
}

/// The band `flat` cuts `surface` in, or `None` where it is not a ball, a pipe
/// or a cone, where the plane runs along its axis or misses a ball, or where a
/// product falls out of range.
fn the_band_a_plane_cuts(flat: &Plane, surface: &Surface) -> Option<ABand> {
    let normal = flat.normal().components();
    let (on_the_axis, along, slope) = match surface {
        Surface::Sphere(ball) => (ball.centre().coordinates(), normal, 0.0),
        Surface::Cylinder(pipe) => (
            pipe.axis().origin().coordinates(),
            pipe.axis().direction().components(),
            0.0,
        ),
        Surface::Cone(cone) => (
            cone.apex().coordinates(),
            cone.axis().direction().components(),
            cone.slope(),
        ),
        Surface::Plane(_) | Surface::Torus(_) => return None,
    };
    let leaning = Exact::dot_of(normal, along)?;
    if !matches!(leaning.sign(), Decision::Pos | Decision::Neg) {
        return None;
    }
    let reach = Exact::of(flat.offset()).minus(&Exact::dot_of(normal, on_the_axis)?);
    let squared_normal = Exact::dot_of(normal, normal)?;
    let squared_axis = Exact::dot_of(along, along)?;
    let squared_radius = match surface {
        Surface::Sphere(ball) => [
            squared_normal
                .times(&Exact::product_of(ball.radius(), ball.radius())?)?
                .minus(&reach.times(&reach)?),
            squared_normal.clone(),
        ],
        Surface::Cylinder(pipe) => [
            Exact::product_of(pipe.radius(), pipe.radius())?,
            Exact::of(1.0),
        ],
        Surface::Cone(cone) => [
            Exact::product_of(cone.slope(), cone.slope())?
                .times(&reach.times(&reach)?)?
                .times(&squared_axis)?,
            leaning.times(&leaning)?,
        ],
        Surface::Plane(_) | Surface::Torus(_) => {
            unreachable!("only a ball, a pipe or a cone has a band: {surface:?}")
        }
    };
    if !matches!(squared_radius[0].sign(), Decision::Pos | Decision::Zero) {
        return None;
    }
    let radius = root_enclosing(quotient_or_everything(
        squared_radius[0].enclosure(),
        squared_radius[1].enclosure(),
    ));

    let both_squared = squared_normal.enclosure() * squared_axis.enclosure();
    let squared_sine = quotient_or_everything(
        Exact::squared_length_of(&exact_cross_of(normal, along)?)?.enclosure(),
        both_squared,
    );
    let sine = root_enclosing(squared_sine);
    let level = if leaning.sign() == Decision::Neg {
        -leaning.enclosure()
    } else {
        leaning.enclosure()
    };
    let cosine = quotient_or_everything(level, root_enclosing(both_squared));
    let one = Interval::exactly(1.0);
    let slope = Interval::exactly(slope);
    let inside = radius * quotient_or_everything(slope * sine, one + slope * sine);
    let room = cosine - slope * sine;
    let outside = if room.lo() > 0.0 {
        radius
            * quotient_or_everything(
                quotient_or_everything(squared_sine, one + cosine) + slope * sine,
                room,
            )
    } else {
        Interval::ENTIRE
    };
    Some(ABand {
        on_the_axis,
        along,
        leaning,
        reach,
        squared_radius,
        radius,
        inside,
        outside,
    })
}

/// `|c₁ − c₂|`, the distance between where the two bands' axes cross the
/// plane, as `|G| / |k₁k₂|` with `G` worked out exactly, or `None` where a
/// product falls out of range.
fn how_far_apart_the_centres_of_two_bands_are(mine: &ABand, theirs: &ABand) -> Option<Interval> {
    let both = mine.leaning.times(&theirs.leaning)?;
    let from_theirs = Exact::gap_between(theirs.on_the_axis, mine.on_the_axis);
    let my_reach = theirs.leaning.times(&mine.reach)?;
    let their_reach = mine.leaning.times(&theirs.reach)?;
    let mut apart = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in apart.iter_mut().enumerate() {
        *entry = both
            .times(&from_theirs[axis])?
            .plus(&my_reach.times(&Exact::of(mine.along[axis]))?)
            .minus(&their_reach.times(&Exact::of(theirs.along[axis]))?);
    }
    let size = if both.sign() == Decision::Neg {
        -both.enclosure()
    } else {
        both.enclosure()
    };
    Some(quotient_or_everything(
        root_enclosing(Exact::squared_length_of(&apart)?.enclosure()),
        size,
    ))
}

/// **A plane that cuts a ball, a pipe or a cone and a second of them in two
/// curves whose bands overlap**, which is what
/// [`what_a_plane_that_cuts_them_in_bands_apart_leaves`] hands on: the two
/// points where the curves cross, nothing where they keep apart after all, or
/// `None` where neither is proved and the triple goes on to the refusal.
///
/// A ball whose centre is on a pipe's or a cone's axis is asked first, since
/// where the two touch along a whole circle the curves the plane cuts them in
/// touch rather than cross, and the road for curves that cross cannot tell
/// that from two crossings close together. The circle is on a plane known
/// exactly, and the plane, that one and the ball are an earlier road's.
///
/// Then each of the two in turn is the one whose band is used, and the road
/// is [`two_points_or_nowhere`].
fn what_a_plane_that_cuts_them_in_bands_that_cross_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    let ball_and_round = if matches!(one, Surface::Sphere(_)) {
        Some((one, other))
    } else if matches!(other, Surface::Sphere(_)) {
        Some((other, one))
    } else {
        None
    };
    if let Some((ball, round)) = ball_and_round {
        match where_a_ball_on_the_axis_meets_it(ball, round) {
            Some(ABallOnTheAxis::MeetsItNowhere) => return Ok(Some(MeetOfThree::Nowhere)),
            Some(ABallOnTheAxis::MeetsItOnAPlane { leaning, reach }) => {
                return where_a_stored_plane_an_exact_one_and_a_quadric_meet(
                    flat, &leaning, &reach, ball, at, policy, log,
                )
                .map(Some);
            }
            None => {}
        }
    }
    Ok(two_points_or_nowhere(flat, one, other).or_else(|| two_points_or_nowhere(flat, other, one)))
}

/// What a ball whose centre is on a pipe's or a cone's axis shares with it.
enum ABallOnTheAxis {
    /// Not a point.
    MeetsItNowhere,
    /// Every point the two share is on `leaning·p = reach`, and every point of
    /// the ball on that plane is on the other one.
    MeetsItOnAPlane { leaning: [Exact; 3], reach: Exact },
}

/// Where `ball` and `round`, a pipe or a cone, share nothing or share one
/// circle, or `None` where the ball's centre is off the axis, where they
/// share two circles, or where a product falls out of range.
///
/// With `W = w·w` for the axis direction `w` and `β` one for a pipe and
/// `1 + slope²` for a cone, `h = W·ball − round` squares `p` only as
/// `β(w·p)²`, and is `β(w·p)² + 2b·p + c` written out. Where `b` runs along
/// `w`, as it does exactly when the ball's centre is on the axis, `b·w = t`
/// gives `βW²h = (βW(w·p) + t)² + βW²c − t²`. So the two share nothing where
/// `βW²c − t²` is above nothing, and share the circle on `βW(w·p) = −t` where
/// it is nothing — a ball resting in a cone, or a ball of a pipe's radius
/// about a point on its axis. Every number here is exact, so both are proved
/// and neither is a rounding's guess.
fn where_a_ball_on_the_axis_meets_it(ball: &Surface, round: &Surface) -> Option<ABallOnTheAxis> {
    let (along, squeeze) = match round {
        Surface::Cylinder(pipe) => (pipe.axis().direction().components(), Exact::of(1.0)),
        Surface::Cone(cone) => (
            cone.axis().direction().components(),
            Exact::of(1.0).plus(&Exact::product_of(cone.slope(), cone.slope())?),
        ),
        Surface::Plane(_) | Surface::Sphere(_) | Surface::Torus(_) => return None,
    };
    let ball_written = how_a_surface_is_written(ball)?;
    let round_written = how_a_surface_is_written(round)?;
    let squared_along = Exact::dot_of(along, along)?;
    let mut linear = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in linear.iter_mut().enumerate() {
        *entry = squared_along
            .times(&ball_written.half_linear[axis])?
            .minus(&round_written.half_linear[axis]);
    }
    let constant = squared_along
        .times(&ball_written.constant)?
        .minus(&round_written.constant);
    if Exact::cross_of(&linear, along)?
        .iter()
        .any(|entry| entry.sign() != Decision::Zero)
    {
        return None;
    }
    let along_the_axis = Exact::dot_of_two_exact(&linear, &Exact::of_each(along))?;
    let squeezed = squeeze.times(&squared_along)?;
    let room = squeezed
        .times(&squared_along)?
        .times(&constant)?
        .minus(&along_the_axis.times(&along_the_axis)?);
    let sign = room.sign();
    if sign == Decision::Pos {
        return Some(ABallOnTheAxis::MeetsItNowhere);
    }
    if sign != Decision::Zero {
        return None;
    }
    let mut leaning = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in leaning.iter_mut().enumerate() {
        *entry = squeezed.times(&Exact::of(along[axis]))?;
    }
    Some(ABallOnTheAxis::MeetsItOnAPlane {
        leaning,
        reach: Exact::of(0.0).minus(&along_the_axis),
    })
}

/// Where the curve `flat` cuts `mine` in crosses the one it cuts `theirs` in:
/// two points, or nowhere, or `None` where neither is proved.
///
/// **The curves.** Both surfaces are written as
/// `Qᵢ = Wᵢ|p − oᵢ|² − βᵢ((p − oᵢ)·wᵢ)² − γᵢ` ([`ARoundSurface`]). Take `c`
/// where `mine`'s axis crosses the plane, and write a point of the plane as
/// `c + v`. Every point of `mine`'s curve has `|v|` between `r_in` and `r_out`,
/// the edges of its band ([`the_band_a_plane_cuts`]), and the curve is a
/// circle or an ellipse with `c` inside, so a line through `c` crosses it
/// once on each side of `c`. A point on both curves is a point on `mine`'s
/// where `h = W₂Q₁ − W₁Q₂` is zero, and on the plane
/// `h = h₀ + g·v + q(v)` with `q(v) = W₁β₂(w₂·v)² − W₂β₁(w₁·v)²`. Only the
/// parts of `w₁` and `w₂` along the plane reach `q`, and a plane a rounding off
/// square to both leaves those a rounding long, so `|q| ≤ κ|v|²` for a `κ` of
/// about a rounding and `h` is a line `h₀ + g·v` bent by no more than
/// `E = κ r_out²`.
///
/// **Nowhere** where `|h₀|` is more than `|g| r_out + E`: `h` is not zero
/// anywhere on the curve.
///
/// **Two points** where the line `h = 0`, widened by `E` either side into a
/// strip, keeps inside `r_in` of `c`. Write `v = λτ + μĝ`, with `ĝ` along `g`
/// and `τ = n × ĝ / |n|` across it. The curve crosses the line through `c`
/// along `ĝ` at a point with `μ` above `r_in` and one with `μ` below `−r_in`,
/// and `h` is above nothing at the first and below at the second. So `h` is
/// zero somewhere on each of the two arcs between them, the arc with `λ`
/// below nothing and the one with `λ` above. It is zero only once on each
/// where it turns the same way at every point of the arc inside the strip.
/// That turn is `n·(∇h × ∇Q₁)` along the plane, which is `|n||g|·2W₁λ` give
/// or take `|g|δ₂ + δ₁(W₁r_out + δ₂)`, for `δ₁ = 2κ r_out` how far `∇h` is from
/// `g` and `δ₂ = β₁|w₁|² r_out + |f₁|/2` how far `∇Q₁/2` is from `W₁v`, with
/// `f₁` the part of `∇Q₁` at `c` along the plane. So the road asks that `λ`
/// keeps far enough from nothing inside the strip for the first to win.
///
/// **Where they are.** From `h = 0`, `μ = (−q − h₀)/|g|`, and from `Q₁ = 0`,
/// `λ² = (−Q₁(c) − f₁·v + β₁(w₁·v)²)/W₁ − μ²`. Each holds at the point, so
/// each turns a box that holds the point into one that holds it too, and a
/// few rounds of both shrink the box the strip starts to a rounding or two.
///
/// **The numbers.** `k(c − oᵢ)` is `k(o₁ − oᵢ) + m w₁` for `k` and `m` of the
/// band, so `k²Qᵢ(c)` and `k∇Qᵢ(c)` are worked out exactly, and so are `k²h₀`
/// and `k g`, with nothing past degree twelve in the stored numbers. Only the
/// quotients by `k` and the frame are enclosures, each a few roundings wide.
fn two_points_or_nowhere(flat: &Plane, mine: &Surface, theirs: &Surface) -> Option<MeetOfThree> {
    let band = the_band_a_plane_cuts(flat, mine)?;
    let normal = flat.normal().components();
    let my_surface = ARoundSurface::of(mine, normal)?;
    let their_surface = ARoundSurface::of(theirs, normal)?;
    let (my_value, my_slant) = my_surface.where_the_axis_crosses(&band)?;
    let (their_value, their_slant) = their_surface.where_the_axis_crosses(&band)?;
    let level = their_surface
        .squared_along
        .times(&my_value)?
        .minus(&my_surface.squared_along.times(&their_value)?);
    let mut slant = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in slant.iter_mut().enumerate() {
        *entry = their_surface
            .squared_along
            .times(&my_slant[axis])?
            .minus(&my_surface.squared_along.times(&their_slant[axis])?);
    }

    let squared_normal = Exact::dot_of(normal, normal)?;
    let leaning = band.leaning.enclosure();
    let squared_leaning = leaning * leaning;
    let off_the_line = quotient_or_everything(level.enclosure(), squared_leaning);
    let my_value = quotient_or_everything(my_value.enclosure(), squared_leaning);
    let rising = the_part_along_the_plane(&slant, normal, &squared_normal, leaning)?;
    let my_slant = the_part_along_the_plane(&my_slant, normal, &squared_normal, leaning)?;
    let one = Interval::exactly(1.0);
    let my_axis = the_part_along_the_plane(
        &Exact::of_each(my_surface.along),
        normal,
        &squared_normal,
        one,
    )?;
    let their_axis = the_part_along_the_plane(
        &Exact::of_each(their_surface.along),
        normal,
        &squared_normal,
        one,
    )?;
    let [my_square, their_square] =
        [&my_surface, &their_surface].map(|it| it.squared_along.enclosure());
    let [my_squeeze, their_squeeze] =
        [&my_surface, &their_surface].map(|it| it.squeeze.enclosure());

    let rise = root_enclosing(squared_length_enclosing(rising));
    let outer = band.radius + band.outside;
    let inner = band.radius - band.inside;
    (rise.lo() > 0.0 && rise.hi().is_finite() && outer.hi().is_finite() && inner.lo() > 0.0)
        .then_some(())?;
    let bend = my_square * their_squeeze * squared_length_enclosing(their_axis)
        + their_square * my_squeeze * squared_length_enclosing(my_axis);
    let most_bent = (bend * outer * outer).hi();
    if !most_bent.is_finite() {
        return None;
    }
    if how_far_from_nothing(off_the_line) > (rise * outer).hi() + most_bent {
        return Some(MeetOfThree::Nowhere);
    }

    let strip = quotient_or_everything(
        Interval::new(-most_bent, most_bent).expect("a bound above nothing") - off_the_line,
        rise,
    );
    let far = Interval::exactly(strip.lo().abs().max(strip.hi().abs()));
    (far.hi() < inner.lo()).then_some(())?;
    let near = if strip.contains_zero() {
        Interval::exactly(0.0)
    } else {
        Interval::exactly(strip.lo().abs().min(strip.hi().abs()))
    };
    let least = (inner * inner - far * far).lo();
    let most = (outer * outer - near * near).hi();
    (least > 0.0).then_some(())?;
    let across = root_enclosing(Interval::new(least, most).ok()?);

    let two = Interval::exactly(2.0);
    let swing = two * bend * outer;
    let my_slant_size = root_enclosing(squared_length_enclosing(my_slant));
    let pull = my_squeeze * squared_length_enclosing(my_axis) * outer
        + my_slant_size * Interval::exactly(0.5);
    let turn = rise * my_square * Interval::exactly(across.lo())
        - (rise * pull + swing * (my_square * outer + pull));
    (turn.lo() > 0.0).then_some(())?;

    let up = rising.map(|entry| quotient_or_everything(entry, rise));
    let side = cross_enclosing(exactly(normal), up)
        .map(|entry| quotient_or_everything(entry, root_enclosing(squared_normal.enclosure())));
    let [my_axis_across, my_axis_up] = [side, up].map(|it| dot_enclosing(my_axis, it));
    let [their_axis_across, their_axis_up] = [side, up].map(|it| dot_enclosing(their_axis, it));
    let [my_slant_across, my_slant_up] = [side, up].map(|it| dot_enclosing(my_slant, it));
    let crossing = |sign: f64| {
        let sign = Interval::exactly(sign);
        let mut sideways = sign * across;
        let mut upwards = strip;
        for _ in 0..8 {
            let widths = (sideways.width(), upwards.width());
            let mine_along = squared(my_axis_across * sideways + my_axis_up * upwards);
            let theirs_along = squared(their_axis_across * sideways + their_axis_up * upwards);
            let bent =
                my_square * their_squeeze * theirs_along - their_square * my_squeeze * mine_along;
            upwards = both(upwards, quotient_or_everything(-bent - off_the_line, rise));
            let squared_reach = quotient_or_everything(
                -my_value - (my_slant_across * sideways + my_slant_up * upwards)
                    + my_squeeze * mine_along,
                my_square,
            );
            sideways = both(
                sideways,
                sign * root_enclosing(squared_reach - squared(upwards)),
            );
            if sideways.width() >= widths.0 && upwards.width() >= widths.1 {
                break;
            }
        }
        let towards = quotient_or_everything(band.reach.enclosure(), leaning);
        the_point_a_box_names(core::array::from_fn(|axis| {
            Interval::exactly(band.on_the_axis[axis])
                + Interval::exactly(band.along[axis]) * towards
                + side[axis] * sideways
                + up[axis] * upwards
        }))
    };
    Some(MeetOfThree::At(smallvec![crossing(-1.0), crossing(1.0)]))
}

/// A ball, a pipe or a cone written as
/// `W|p − o|² − β((p − o)·w)² − γ`, the same surface its own sign expression
/// names: a ball as `|p − o|² − R²`, a pipe as `W|p − o|² − ((p − o)·w)² − Wr²`
/// and a cone as `W|p − o|² − (1 + slope²)((p − o)·w)²`, with `W = w·w`.
struct ARoundSurface {
    /// `o`: a ball's centre, a pipe's axis point or a cone's apex.
    on_the_axis: [f64; 3],
    /// `w`, which for a ball is the plane's normal and squeezed by nothing.
    along: [f64; 3],
    /// `W`.
    squared_along: Exact,
    /// `β`.
    squeeze: Exact,
    /// `γ`.
    constant: Exact,
}

impl ARoundSurface {
    /// `surface` written out, or `None` where it is not a ball, a pipe or a
    /// cone, or where a product falls out of range.
    fn of(surface: &Surface, normal: [f64; 3]) -> Option<Self> {
        match surface {
            Surface::Sphere(ball) => Some(ARoundSurface {
                on_the_axis: ball.centre().coordinates(),
                along: normal,
                squared_along: Exact::of(1.0),
                squeeze: Exact::of(0.0),
                constant: Exact::product_of(ball.radius(), ball.radius())?,
            }),
            Surface::Cylinder(pipe) => {
                let along = pipe.axis().direction().components();
                let squared_along = Exact::dot_of(along, along)?;
                let constant =
                    squared_along.times(&Exact::product_of(pipe.radius(), pipe.radius())?)?;
                Some(ARoundSurface {
                    on_the_axis: pipe.axis().origin().coordinates(),
                    along,
                    squared_along,
                    squeeze: Exact::of(1.0),
                    constant,
                })
            }
            Surface::Cone(cone) => {
                let along = cone.axis().direction().components();
                Some(ARoundSurface {
                    on_the_axis: cone.apex().coordinates(),
                    along,
                    squared_along: Exact::dot_of(along, along)?,
                    squeeze: Exact::of(1.0).plus(&Exact::product_of(cone.slope(), cone.slope())?),
                    constant: Exact::of(0.0),
                })
            }
            Surface::Plane(_) | Surface::Torus(_) => None,
        }
    }

    /// `k²Q(c)` and `k∇Q(c)`, exactly, at `c` where `band`'s axis crosses its
    /// plane, from `k(c − o) = k(o_band − o) + m w_band`.
    fn where_the_axis_crosses(&self, band: &ABand) -> Option<(Exact, [Exact; 3])> {
        let from_here = Exact::gap_between(self.on_the_axis, band.on_the_axis);
        let mut reach = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
        for (axis, entry) in reach.iter_mut().enumerate() {
            *entry = band
                .leaning
                .times(&from_here[axis])?
                .plus(&band.reach.times(&Exact::of(band.along[axis]))?);
        }
        let along_it = Exact::dot_of_two_exact(&reach, &Exact::of_each(self.along))?;
        let value = self
            .squared_along
            .times(&Exact::squared_length_of(&reach)?)?
            .minus(&self.squeeze.times(&along_it.times(&along_it)?)?)
            .minus(&self.constant.times(&band.leaning.times(&band.leaning)?)?);
        let squeezed = self.squeeze.times(&along_it)?;
        let mut slant = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
        for (axis, entry) in slant.iter_mut().enumerate() {
            let half = self
                .squared_along
                .times(&reach[axis])?
                .minus(&squeezed.times(&Exact::of(self.along[axis]))?);
            *entry = half.plus(&half);
        }
        Some((value, slant))
    }
}

/// The part of `x / under` along the plane with normal `n`, as
/// `(N x − (x·n) n) / (N under)` with the top worked out exactly, or `None`
/// where a product falls out of range.
fn the_part_along_the_plane(
    x: &[Exact; 3],
    normal: [f64; 3],
    squared_normal: &Exact,
    under: Interval,
) -> Option<[Interval; 3]> {
    let across = Exact::dot_of_two_exact(x, &Exact::of_each(normal))?;
    let below = squared_normal.enclosure() * under;
    let mut part = [Interval::ENTIRE; 3];
    for (axis, entry) in part.iter_mut().enumerate() {
        let over = squared_normal
            .times(&x[axis])?
            .minus(&across.times(&Exact::of(normal[axis]))?);
        *entry = quotient_or_everything(over.enclosure(), below);
    }
    Some(part)
}

/// The least `|x|` can be for an `x` in `value`.
fn how_far_from_nothing(value: Interval) -> f64 {
    if value.hi() < 0.0 {
        -value.hi()
    } else if value.lo() > 0.0 {
        value.lo()
    } else {
        0.0
    }
}

/// `x²` for every `x` in `value`, which is never below nothing where `value`
/// holds nothing, as `value * value` would be.
fn squared(value: Interval) -> Interval {
    if !value.contains_zero() {
        return value * value;
    }
    let size = Interval::exactly(value.lo().abs().max(value.hi().abs()));
    Interval::new(0.0, (size * size).hi()).expect("a square is not below nothing")
}

/// Where two boxes that each hold one root overlap.
///
/// # Panics
///
/// Where they do not overlap, which means one of them does not hold the root
/// after all.
fn both(one: Interval, other: Interval) -> Interval {
    Interval::new(one.lo().max(other.lo()), one.hi().min(other.hi()))
        .unwrap_or_else(|_| panic!("two boxes that each hold one root miss: {one:?} and {other:?}"))
}

/// **A plane that reaches a pipe only far along it, past where a ball, a
/// second pipe or a cone could share a point with the pipe**: nothing, or
/// `None` where that is not proved.
///
/// A plane a rounding off parallel to a bar's axis and farther from it than
/// its radius reaches the bar only very far along it, and a chamfer cone on
/// the bar is as wide as that there. [`what_a_plane_along_a_pipe_leaves`]
/// takes only a plane exactly parallel, and the band roads give a cone cut by
/// a plane that steep no outer edge, so neither reaches it.
///
/// **Where the plane reaches the pipe.** Write the pipe as the points
/// `p = o + τw + v`, for `v` square to `w` and `r` long, and the plane as
/// `n·p = d`, with `N = n·n`, `k = n·w` and `δ = n·o − d`. On the plane
/// `τk = −(δ + n·v)`, and `|n·v|` is at most `r√N`. So where `|δ|` is more
/// than `r√N` — the plane keeps off the pipe where it passes `o` — every point
/// the two share has `|τ||k| ≥ m` for `m = |δ| − r√N`.
///
/// **The other surface there.** Write it `Q(p) = p·(Mp) + 2b·p + c`. Along
/// the pipe, `Q(o + τw + v) = Aτ² + 2βτ + γ` for `A = w·(Mw)`,
/// `β = (Mw)·o + b·w + (Mw)·v` and `γ = Q(o) + 2(Mo + b)·v + v·(Mv)`. So `|β|`
/// is at most `B = |(Mw)·o + b·w| + r|Mw|` and `|γ|` at most
/// `C = |Q(o)| + 2r|Mo + b| + r²|M|`, with `|M|` the root of the sum of its
/// nine entries squared. Where `|A|m² > 2Bm|k| + Ck²`, `|A|τ² > 2B|τ| + C`
/// at the nearest `|τ|` the plane reaches, `m/|k|`. That puts it past
/// `2B/|A|`, where `|A|τ² − 2B|τ| − C` only grows further out. So `Q` has the
/// sign of `A` everywhere the plane reaches the pipe, and the three share
/// nothing.
///
/// **Every number is proved and none is a tolerance**: `k`, `δ`, `A`, `N` and
/// the rest are worked out exactly and only then boxed, and each side of the
/// comparison is taken at its worst end. The side of a block three from a bar
/// of radius two and its chamfer cone of slope one, all placed by one turn, is
/// one: `k` is a rounding or less, so the side reaches the bar no nearer than
/// about `10¹⁶` along it, where the cone is as wide.
///
/// It is asked after every road but the leaning call of
/// [`what_a_plane_beside_their_one_axis_leaves`], and only ever answers that
/// the three share nothing, so every triple an earlier road answers keeps its
/// answer.
fn what_a_plane_beside_a_pipe_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
) -> Option<MeetOfThree> {
    [(one, other), (other, one)]
        .into_iter()
        .any(|(this, that)| match this {
            Surface::Cylinder(pipe) => {
                the_plane_reaches_the_pipe_only_off_the_other(flat, pipe, that) == Some(true)
            }
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Cone(_) | Surface::Torus(_) => false,
        })
        .then_some(MeetOfThree::Nowhere)
}

/// Whether every point `flat` shares with `pipe` is proved off `other`, as
/// [`what_a_plane_beside_a_pipe_leaves`] proves it, or `None` where `other` is
/// not a ball, a pipe or a cone, or where a product falls out of range.
fn the_plane_reaches_the_pipe_only_off_the_other(
    flat: &Plane,
    pipe: &Cylinder,
    other: &Surface,
) -> Option<bool> {
    let written = how_a_surface_is_written(other)?;
    let normal = flat.normal().components();
    let origin = pipe.axis().origin().coordinates();
    let leaning = Exact::dot_of(normal, pipe.axis().direction().components())?;
    let off = how_far_a_plane_is_from(flat, origin)?;
    let squared_normal = Exact::dot_of(normal, normal)?;

    let along = Exact::of_each(pipe.axis().direction().components());
    let from = Exact::of_each(origin);
    let half_linear = &written.half_linear;
    let towards = rows_times(&written.squared, &along)?;
    let bend = Exact::dot_of_two_exact(&along, &towards)?;
    let level = Exact::dot_of_two_exact(&towards, &from)?
        .plus(&Exact::dot_of_two_exact(half_linear, &along)?);
    let from_the_origin = rows_times(&written.squared, &from)?;
    let pull: [Exact; 3] =
        core::array::from_fn(|axis| from_the_origin[axis].plus(&half_linear[axis]));
    let linear_at_the_origin = Exact::dot_of_two_exact(half_linear, &from)?;
    let at_the_origin = Exact::dot_of_two_exact(&from, &from_the_origin)?
        .plus(&linear_at_the_origin.plus(&linear_at_the_origin))
        .plus(&written.constant);
    let mut squared_size = Exact::of(0.0);
    for row in &written.squared {
        squared_size = squared_size.plus(&Exact::squared_length_of(row)?);
    }

    let most = |value: &Exact| {
        let value = value.enclosure();
        Interval::exactly(value.lo().abs().max(value.hi().abs()))
    };
    let least = |value: &Exact| Interval::exactly(how_far_from_nothing(value.enclosure()));
    let length =
        |value: &[Exact; 3]| Some(root_enclosing(Exact::squared_length_of(value)?.enclosure()));
    let radius = Interval::exactly(pipe.radius());
    let two = Interval::exactly(2.0);
    let room = least(&off) - radius * root_enclosing(squared_normal.enclosure());
    if room.lo() <= 0.0 {
        return Some(false);
    }
    let room = Interval::exactly(room.lo());
    let tilt = most(&leaning);
    let sway = most(&level) + radius * length(&towards)?;
    let spread = most(&at_the_origin)
        + two * radius * length(&pull)?
        + radius * radius * root_enclosing(squared_size.enclosure());
    let held = least(&bend) * room * room;
    let pushed = two * sway * room * tilt + spread * tilt * tilt;
    Some(held.lo() > pushed.hi())
}

/// A line, written as `(through + s · direction) / divided_by`.
///
/// Two planes crossing build one, and so does a ray. **The point on the line two
/// planes give is the one with a zero coordinate, not the foot of the
/// perpendicular from the origin**, and that choice is what keeps the degrees
/// down: the perpendicular foot needs `‖n₁ × n₂‖²` underneath it and carries
/// four more degrees into everything built on it.
struct ALine {
    through: [Term; 3],
    direction: [Term; 3],
    divided_by: Term,
}

/// One number a line or a surface is written out of, at the best level the
/// caller's own numbers allow.
///
/// **Which of the two it is is settled once, at the top, and never mixes.** Two
/// planes crossing hand over stored `f64`, so everything built from them is an
/// exact expansion and level 3 decides every sign. A ray from a corner a fuse
/// *built* hands over boxes, so everything built from them is a box, level 3
/// cannot run at all, and a sign whose box straddles zero is undecided — which
/// is the honest answer and not a shortcoming of the arithmetic (§2.5). A box
/// that names one `f64` is that `f64`, which is what
/// [`Term::from_a_box`] reads and `as_exact_as_it_is` exists to notice.
///
/// **A product of stored numbers that falls off the bottom of the range stays
/// on the stored side.** It is held as a [`Nearly`]: the expansion, and a box
/// round what fell off, a smallest subnormal per product that rounded. Its
/// exact level then cannot run, and its enclosure is that expansion and that
/// box — tight enough to decide a discriminant of `3` where a term of `2⁻²⁰¹⁴`
/// went missing, which a round bar moved by `2⁻¹⁰⁰⁷` against a cube's wall
/// asks for (`docs/decisions.md` §322). Before, such a product refused the
/// whole meet with an unbounded enclosure.
#[derive(Clone)]
enum Term {
    /// Worked out from stored numbers, so known exactly — or, where a product
    /// fell off the bottom of the range, to within a box of subnormals.
    Stored(Nearly),
    /// Known only as a box, because the line it was built from is one.
    Boxed(Interval),
}

impl Term {
    /// A stored number, exactly.
    fn of(value: f64) -> Self {
        Term::Stored(Nearly::of(value))
    }

    /// A number worked out exactly.
    fn exact(value: Exact) -> Self {
        Term::Stored(value.into())
    }

    /// A box, or the number it names where it names one.
    fn from_a_box(enclosure: Interval) -> Self {
        if enclosure.lo() == enclosure.hi() {
            return Term::of(enclosure.lo());
        }
        Term::Boxed(enclosure)
    }

    /// This number as an enclosure.
    fn enclosure(&self) -> Interval {
        match self {
            Term::Stored(value) => value.enclosure(),
            Term::Boxed(enclosure) => *enclosure,
        }
    }

    /// This number exactly, or `None` where it was never known that well.
    fn exactly(&self) -> Option<Exact> {
        match self {
            Term::Stored(value) => value.exactly(),
            Term::Boxed(_) => None,
        }
    }

    /// Whether this was worked out from stored numbers and lost more than a
    /// rounding of itself off the bottom of the range on the way — the one case
    /// where it is refused as it always was ([`Nearly::is_within_a_rounding`]).
    fn lost_more_than_a_rounding(&self) -> bool {
        match self {
            Term::Stored(value) => !value.is_within_a_rounding(),
            Term::Boxed(_) => false,
        }
    }

    /// Whether this was worked out from stored numbers and lost anything at
    /// all off the bottom of the range.
    fn lost_anything(&self) -> bool {
        matches!(self, Term::Stored(value) if value.exactly().is_none())
    }

    /// Which way this number points, or [`NOTHING_IS_KNOWN`] where a box
    /// straddles zero.
    fn sign(&self) -> Decision {
        if let Some(value) = self.exactly() {
            return value.sign();
        }
        let enclosure = self.enclosure();
        if enclosure.lo() > 0.0 {
            return Decision::Pos;
        }
        if enclosure.hi() < 0.0 {
            return Decision::Neg;
        }
        NOTHING_IS_KNOWN
    }

    /// This plus `other`.
    fn plus(&self, other: &Term) -> Term {
        match (self, other) {
            (Term::Stored(mine), Term::Stored(theirs)) => Term::Stored(mine.plus(theirs)),
            _ => Term::Boxed(self.enclosure() + other.enclosure()),
        }
    }

    /// This minus `other`.
    fn minus(&self, other: &Term) -> Term {
        match (self, other) {
            (Term::Stored(mine), Term::Stored(theirs)) => Term::Stored(mine.minus(theirs)),
            _ => Term::Boxed(self.enclosure() - other.enclosure()),
        }
    }

    /// This times `other`, or `None` when both are stored and the product
    /// overflows.
    ///
    /// **Two stored numbers whose product does not fit do not quietly become a
    /// box.** That refusal is the one every discriminant in
    /// [`intersect`](crate::intersect) has, and answering it with an interval
    /// instead would give a caller a second, wider answer to a question it had
    /// already been told nobody could take. A product that falls off the
    /// *bottom* is another matter, and [`Term`]'s own doc says why it is kept.
    fn times(&self, other: &Term) -> Option<Term> {
        match (self, other) {
            (Term::Stored(mine), Term::Stored(theirs)) => Some(Term::Stored(mine.times(theirs)?)),
            _ => Some(Term::Boxed(self.enclosure() * other.enclosure())),
        }
    }
}

/// `a · b` over two triples.
fn dot_of(u: &[Term; 3], v: &[Term; 3]) -> Option<Term> {
    let mut total = Term::of(0.0);
    for axis in 0..3 {
        total = total.plus(&u[axis].times(&v[axis])?);
    }
    Some(total)
}

/// `u · u` for one triple.
fn squared_length_of(u: &[Term; 3]) -> Option<Term> {
    dot_of(u, u)
}

/// `u × v` for a triple and a triple of stored numbers.
fn cross_of(u: &[Term; 3], v: [f64; 3]) -> Option<[Term; 3]> {
    let component = |axis: usize| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        let left = u[next].times(&Term::of(v[after]))?;
        let right = u[after].times(&Term::of(v[next]))?;
        Some(left.minus(&right))
    };
    Some([component(0)?, component(1)?, component(2)?])
}

/// The line two planes cross in, or `None` when they never do.
///
/// The coordinate that is zeroed is the one whose two-by-two minor of the two
/// normals is largest, so the division below is by the best-conditioned of the
/// three. Which one is picked is worked out in `f64` and only ranks the
/// candidates; that a minor is **not zero** is decided exactly, and an axis
/// whose minor cannot be built at all is passed over rather than trusted.
///
/// **The points this hands back run along `n₁ × n₂ / minor`, not along
/// `n₁ × n₂` itself.** `minor` is that same largest two-by-two component of
/// `n₁ × n₂`, so dividing by it always turns that one component positive —
/// and turns the other two round with it when `minor` was negative to start
/// with. A caller after `n₁ × n₂`'s own sign, rather than after one fixed
/// direction for the pair, reads it off the planes' own normals and never off
/// the points here.
///
/// # Errors
///
/// [`Error::Undecidable`] when the two planes are not parallel and no minor
/// could be worked out exactly, or when whether they are parallel could not be
/// settled at all.
fn the_line_two_planes_cross_in(
    one: &Plane,
    other: &Plane,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<ALine>> {
    let [mine, theirs] = [one, other].map(|plane| plane.normal().components());
    let [my_offset, their_offset] = [one, other].map(Plane::offset);

    let mut by_size = [0usize, 1, 2];
    let ranked = one.normal().cross(other.normal()).components();
    by_size.sort_by(|&this, &that| ranked[that].abs().total_cmp(&ranked[this].abs()));

    for axis in by_size {
        let (across, and) = ((axis + 1) % 3, (axis + 2) % 3);
        let Some(minor) = cross_of_two_pairs(mine[across], theirs[and], mine[and], theirs[across])
        else {
            continue;
        };
        if !matches!(minor.sign(), Decision::Pos | Decision::Neg) {
            continue;
        }
        let (Some(here), Some(there), Some(direction)) = (
            cross_of_two_pairs(my_offset, theirs[and], their_offset, mine[and]),
            cross_of_two_pairs(mine[across], their_offset, theirs[across], my_offset),
            exact_cross_of(mine, theirs),
        ) else {
            continue;
        };
        let mut through = [Term::of(0.0), Term::of(0.0), Term::of(0.0)];
        through[across] = Term::exact(here);
        through[and] = Term::exact(there);
        return Ok(Some(ALine {
            through,
            direction: direction.map(Term::exact),
            divided_by: Term::exact(minor),
        }));
    }

    // Every minor came back zero or could not be built, and those are different
    // answers: the first says the two planes are parallel and the second says
    // this arithmetic ran out. `‖n₁ × n₂‖²` is the same question asked where
    // level 2 can also have a go at it.
    let across = squared_length_of_a_cross_product(WHERE_THREE_SURFACES_MEET, mine, theirs);
    match ask(&across, Measures::Length, at, policy, log)? {
        Decision::Zero => Ok(None),
        Decision::Pos => Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        )),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }
}

/// Where a line crosses one plane: nowhere, at one step along it, or lying in
/// it.
///
/// Shared between two callers that reach a plane by a different road each:
/// [`where_three_planes_meet`] already has the line two other planes cross in,
/// and [`where_a_line_meets_a_surface`] has a line handed straight over.
fn where_a_line_crosses_one_plane(
    named: &'static str,
    line: &ALine,
    plane: &Plane,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    let normal = plane.normal().components().map(Term::of);
    let (Some(along), Some(across)) = (
        dot_of(&line.direction, &normal),
        dot_of(&line.through, &normal).and_then(|reach| {
            Some(reach.minus(&line.divided_by.times(&Term::of(plane.offset()))?))
        }),
    ) else {
        return Err(an_exact_step_could_not_be_taken(named, at));
    };
    // Three planes have Cramer's rule to fall back on in
    // `where_three_planes_meet`, so a product that fell off the bottom here is
    // refused as it was and that route is left to answer.
    if along.lost_anything() || across.lost_anything() {
        return Err(an_exact_step_could_not_be_taken(named, at));
    }

    // `along` is the determinant of the three normals, which is dimensionless.
    // `Measures` has no arm for that; see the note in
    // `intersect::where_a_plane_and_a_pipe_cross` for why `Length` is the one
    // whose conversion is the identity, and §28 for what should replace it.
    match ask(
        &the_sign_of(named, &along),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Pos | Decision::Neg => Ok(WhereALineMeetsIt::At(smallvec![MeetsAt::Here(
            quotient_or_everything(-across.enclosure(), along.enclosure())
        )])),
        Decision::Zero => match ask(
            &the_sign_of(named, &across),
            Measures::Length,
            at,
            policy,
            log,
        )? {
            Decision::Zero => Ok(WhereALineMeetsIt::MoreThanAPoint),
            Decision::Pos | Decision::Neg => Ok(WhereALineMeetsIt::Nowhere),
            other => unreachable!("a sign question answers with a sign: {other:?}"),
        },
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// Where the line meets a ring: nowhere, or at up to four points.
///
/// **A line never lies along a torus**, which is what makes this shorter than
/// the quadric route below it: the leading coefficient is `(w·w)(D·D)²` for a
/// direction `D` two crossing planes handed over and an axis direction a stored
/// [`Axis`](crate::Axis) normalised, and neither of those is zero. So the
/// quartic is a quartic, never the zero polynomial, and there is no
/// [`MeetOfThree::MoreThanAPoint`] to answer with.
///
/// # Errors
///
/// [`Error::Undecidable`] when a product the exact arithmetic needs falls out of
/// `f64`'s range, and when the line all but touches the ring — a **gap** and not
/// a limit, and [`real_roots`](crate::real_roots) names the family of fixes. A
/// touch that lands on an `f64` is answered rather than refused, and a bend of
/// pipe's own seam is that case.
fn where_a_line_meets_a_ring(
    named: &'static str,
    line: &ALine,
    ring: &Torus,
    at: &[PredicatePoint],
) -> Result<WhereALineMeetsIt> {
    let quartic = how_a_ring_cuts_the_line(line, ring)
        .filter(|quartic| !quartic.iter().any(Term::lost_anything))
        .ok_or_else(|| an_exact_step_could_not_be_taken(named, at))?;
    // **The quartic route is the one place level 3 is not optional.** Its roots
    // are found by evaluating signs along the line, and a coefficient that is a
    // box has no sign to evaluate anywhere near a root — so a line that is a box
    // is turned away here rather than answered with four boxes nobody can order.
    let exactly: Option<Vec<Exact>> = quartic.iter().map(Term::exactly).collect();
    let quartic = exactly.ok_or(Error::WhereALineMeetsARingNeedsTheLineExactly)?;
    assert!(
        matches!(quartic[0].sign(), Decision::Pos),
        "a line's direction and a ring's axis both have length, so this is above zero"
    );
    let found = where_a_polynomial_is_zero(&quartic)
        .ok_or_else(|| an_exact_step_could_not_be_taken(named, at))?;

    let mut steps: SmallVec<[MeetsAt; 2]> = found.at.iter().copied().map(MeetsAt::Here).collect();
    steps.extend(found.or_maybe_at.iter().map(|unsure| MeetsAt::MaybeATouch {
        inside: unsure.inside,
        how_far_it_reaches: unsure.how_far_it_reaches,
    }));
    if steps.is_empty() {
        return Ok(WhereALineMeetsIt::Nowhere);
    }
    Ok(WhereALineMeetsIt::At(steps))
}

/// The quartic a ring cuts out of the line, highest power first, or `None` when
/// a product it needs falls out of `f64`'s range.
///
/// A ring's own sign expression is `(w·w)(x·x + R² − r²)² − 4R²‖x × w‖²` for
/// `x = p − centre`. Writing the line as `(V + s·D)/m` and multiplying the whole
/// of it by `m⁴` — positive, so no sign moves — turns that into
///
/// ```text
/// (w·w)·A(s)²  −  4R²m²·B(s)
/// A(s) = (V + sD)·(V + sD) + m²(R² − r²)
/// B(s) = ‖(V + sD) × w‖²
/// ```
///
/// with `A` and `B` quadratics in `s`, so the whole is a quartic whose every
/// coefficient is a polynomial in the numbers as stored. **Both radii are
/// squared against `m²` and neither carries a `w·w` of its own**, because both
/// sit inside a term the `(w·w)` out front already multiplies. That makes the
/// quartic degree two in `w` all through, so the ring these numbers name is the
/// one the caller's two radii describe whatever length the direction got stored
/// at — the same rule [`Torus`]'s own sign predicate is certified under, and the
/// same rule a pipe follows since §118.10, though a pipe has to be handed its
/// `w·w` by hand because nothing else multiplies its radius (§27.1).
fn how_a_ring_cuts_the_line(line: &ALine, ring: &Torus) -> Option<[Term; 5]> {
    let along = ring.axis().direction().components();
    let reach = the_line_offset_to(line, ring.centre().coordinates())?;
    let axis_squared = Term::Stored(Nearly::dot_of(along, along)?);
    let divisor_squared = line.divided_by.times(&line.divided_by)?;
    let ring_squared = Term::Stored(Nearly::product_of(ring.ring_radius(), ring.ring_radius())?);
    let hollow = ring_squared.minus(&Term::Stored(Nearly::product_of(
        ring.tube_radius(),
        ring.tube_radius(),
    )?));

    let bulge = [
        squared_length_of(&line.direction)?,
        twice(&dot_of(&reach, &line.direction)?)?,
        squared_length_of(&reach)?.plus(&divisor_squared.times(&hollow)?),
    ];
    let sideways = cross_of(&line.direction, along)?;
    let across = cross_of(&reach, along)?;
    let round = [
        squared_length_of(&sideways)?,
        twice(&dot_of(&across, &sideways)?)?,
        squared_length_of(&across)?,
    ];
    let four_ring_squared = twice(&twice(&ring_squared)?)?.times(&divisor_squared)?;

    Some([
        axis_squared.times(&bulge[0].times(&bulge[0])?)?,
        axis_squared.times(&twice(&bulge[0].times(&bulge[1])?)?)?,
        axis_squared
            .times(
                &bulge[1]
                    .times(&bulge[1])?
                    .plus(&twice(&bulge[0].times(&bulge[2])?)?),
            )?
            .minus(&four_ring_squared.times(&round[0])?),
        axis_squared
            .times(&twice(&bulge[1].times(&bulge[2])?)?)?
            .minus(&four_ring_squared.times(&round[1])?),
        axis_squared
            .times(&bulge[2].times(&bulge[2])?)?
            .minus(&four_ring_squared.times(&round[2])?),
    ])
}

/// Twice a number, or `None` when that runs off the top of the range.
fn twice(value: &Term) -> Option<Term> {
    value.times(&Term::of(2.0))
}

/// How far the quartic reaches over a box it turns in, as a question the cascade
/// can ask.
///
/// There is no exact level here and that is the refusal, not an omission: the
/// turning point is not an `f64`, so no exact evaluation reaches it. What level
/// 2 hands over is a box round the height of the turn, and a policy that will
/// not read a zero out of a box that straddles one is refused.
fn how_far_it_reaches_over(
    named: &'static str,
    reach: Interval,
) -> Question<impl Fn() -> Interval, impl Fn() -> Option<Exact>> {
    Question {
        named,
        enclosing: move || reach,
        exactly: move || None,
    }
}

/// `A·s² + 2B·s + C`, where `s` runs along the line.
///
/// Kept as three numbers and never as one polynomial, because the sign of the
/// discriminant and the size of the roots are read off them differently: the
/// sign goes through [`Exact::square_minus`], which slides both sides up the
/// range and hands back a sign with no size in it, and the roots go through the
/// enclosures of these three.
struct Quadratic {
    squared: Term,
    half_linear: Term,
    constant: Term,
}

impl Quadratic {
    /// `B² − A·C` with nothing slid anywhere, so it is a **value** as well as a
    /// sign — or `None` when a product it needs falls out of `f64`'s range.
    ///
    /// [`Exact::square_minus`] reaches further, because it slides both halves
    /// into the window where squaring is exact; what it hands back is a sign
    /// multiplied by a positive power of two, and reading a size off it is the
    /// mistake §32.1 names. This is the same number without the slide, and it is
    /// what the roots are worked out from.
    fn difference_of_the_two_halves(&self) -> Option<Term> {
        Some(
            self.half_linear
                .times(&self.half_linear)?
                .minus(&self.squared.times(&self.constant)?),
        )
    }
}

/// The quadratic a ball or a pipe cuts out of the line, or `None` when a product
/// it needs falls out of `f64`'s range.
///
/// # Panics
///
/// On a plane, which is not a quadric and is answered by
/// [`where_three_planes_meet`] sixteen lines above the only call, and on a
/// torus, which is not a quadric either and is taken by
/// [`where_a_line_meets_a_ring`] two lines after that.
///
/// The plane arm is unreachable by adjacency — same function, same value, no
/// branch between. **The torus arm is not**: one check covers three positions
/// only because a torus sorts after every other kind, which is a fact about
/// [`Surface::how_it_sorts_against`] in another file and nothing here makes
/// anyone re-read it.
/// `where_two_planes_and_a_ring_meet_is_answered_at_up_to_four_places`
/// (`crates/rcad-geom/tests/the_shapes_a_plane_cuts_a_torus_in.rs`) walks a
/// torus through all three positions, and two of them and three, so the claim is
/// demonstrated rather than argued.
fn how_a_quadric_cuts_the_line(line: &ALine, quadric: &Surface) -> Option<Quadratic> {
    let quadratic = match quadric {
        Surface::Sphere(ball) => how_a_ball_cuts_the_line(line, ball),
        Surface::Cylinder(pipe) => how_a_pipe_cuts_the_line(line, pipe),
        Surface::Cone(cone) => how_a_cone_cuts_the_line(line, cone),
        Surface::Plane(flat) => panic!("{flat:?} is flat, so three planes is the question"),
        Surface::Torus(ring) => panic!("{ring:?} is a quartic and is refused before this"),
    }?;
    let coefficients = [
        &quadratic.squared,
        &quadratic.half_linear,
        &quadratic.constant,
    ];
    (!coefficients
        .iter()
        .any(|term| term.lost_more_than_a_rounding()))
    .then_some(quadratic)
}

/// `‖V + s·D‖² − m²R²` where `V` is the line's point offset to the ball's
/// centre, a **degree ten** discriminant in the stored numbers.
///
/// `docs/decisions.md` §31.2 guessed about fourteen for this row and §34 records
/// what it is. Both halves of `B² − A·C` come to ten: `A` is `D·D` at four, `B`
/// is `V·D` at five and `C` is `V·V − m²R²` at six.
fn how_a_ball_cuts_the_line(line: &ALine, ball: &Sphere) -> Option<Quadratic> {
    let reach = the_line_offset_to(line, ball.centre().coordinates())?;
    Some(Quadratic {
        squared: squared_length_of(&line.direction)?,
        half_linear: dot_of(&reach, &line.direction)?,
        constant: squared_length_of(&reach)?.minus(&squared_radius_of(line, ball.radius())?),
    })
}

/// `‖(V + s·D) × w‖² − m²r²(w·w)` for the pipe's stored axis direction `w`, a
/// **degree fourteen** discriminant.
///
/// The radius is squared against `m²(w·w)` and not against `m²` alone, so that
/// these three numbers name the pipe of radius exactly `r` — which is what
/// [`SideOfSurface`](rcad_predicates::SideOfSurface) has certified for a pipe
/// since §118.10 made it homogeneous in `w`. A discriminant that left the `w·w`
/// out would be certified about the pipe of radius `r/‖w‖` instead, and the two
/// halves of a meet would be answering about surfaces a rounding apart (§27.1).
///
/// **The `w·w` does not move the degree.** `A` is `‖D × w‖²` at six, `B` is
/// `(D × w)·(V × w)` at seven and `C` is now eight on both of its terms where
/// the radius term used to be six, so `B² − A·C` is fourteen either way — and it
/// is only now that `A·C`'s two halves are the same degree as each other.
/// `how_many_f64_the_discriminant_of_a_line_and_a_quadric_takes_to_hold` below
/// measures what that costs in `f64` rather than arguing it.
fn how_a_pipe_cuts_the_line(line: &ALine, pipe: &Cylinder) -> Option<Quadratic> {
    let along = pipe.axis().direction().components();
    let reach = the_line_offset_to(line, pipe.axis().origin().coordinates())?;
    let sideways = cross_of(&line.direction, along)?;
    let across = cross_of(&reach, along)?;
    let axis_squared = Term::Stored(Nearly::dot_of(along, along)?);
    Some(Quadratic {
        squared: squared_length_of(&sideways)?,
        half_linear: dot_of(&sideways, &across)?,
        constant: squared_length_of(&across)?
            .minus(&squared_radius_of(line, pipe.radius())?.times(&axis_squared)?),
    })
}

/// `‖(V + s·D) × w‖² − slope²·((V + s·D)·w)²` for the cone's stored axis
/// direction `w`.
///
/// **Neither an `m²` nor a `w·w` correction, and that is the cone being easier
/// than the pipe rather than two corrections being forgotten.** A pipe's
/// constant term compares an area against `r²`, which is degree zero in both the
/// line's divisor and the stored direction, so it has to be given `m²(w·w)` to
/// match — see [`squared_radius_of`]. A cone has no radius: both halves here are
/// degree two in `V + s·D` and degree two in `w`, and the slope is a pure ratio,
/// so the divisor and the direction's length both factor out of the whole
/// expression and every sign read off it is untouched.
fn how_a_cone_cuts_the_line(line: &ALine, cone: &Cone) -> Option<Quadratic> {
    let along = cone.axis().direction().components();
    let up = along.map(Term::of);
    let reach = the_line_offset_to(line, cone.apex().coordinates())?;
    let sideways = cross_of(&line.direction, along)?;
    let across = cross_of(&reach, along)?;
    let slope_squared = Term::Stored(Nearly::product_of(cone.slope(), cone.slope())?);
    let direction_up = dot_of(&line.direction, &up)?;
    let reach_up = dot_of(&reach, &up)?;
    Some(Quadratic {
        squared: squared_length_of(&sideways)?
            .minus(&slope_squared.times(&direction_up.times(&direction_up)?)?),
        half_linear: dot_of(&sideways, &across)?
            .minus(&slope_squared.times(&direction_up.times(&reach_up)?)?),
        constant: squared_length_of(&across)?
            .minus(&slope_squared.times(&reach_up.times(&reach_up)?)?),
    })
}

/// The line's own point, moved so `place` is the origin, and still scaled by the
/// line's divisor.
fn the_line_offset_to(line: &ALine, place: [f64; 3]) -> Option<[Term; 3]> {
    let mut reach = [Term::of(0.0), Term::of(0.0), Term::of(0.0)];
    for axis in 0..3 {
        reach[axis] = line.through[axis].minus(&line.divided_by.times(&Term::of(place[axis]))?);
    }
    Some(reach)
}

/// `m²r²`, which is what a radius has to be compared against once the line has
/// been written with its divisor left in.
///
/// That is the whole of it for a ball, whose sign expression is
/// `‖p − c‖² − R²` and carries no direction. A pipe multiplies what comes back
/// by `w·w` on top, because its other side is `‖x × w‖²` — see
/// [`how_a_pipe_cuts_the_line`].
fn squared_radius_of(line: &ALine, radius: f64) -> Option<Term> {
    line.divided_by
        .times(&line.divided_by)?
        .times(&Term::Stored(Nearly::product_of(radius, radius)?))
}

/// Where the line meets the quadric: nowhere, at one step along it, at two, or
/// along the whole of it.
fn where_a_line_meets_a_quadric(
    named: &'static str,
    quadratic: &Quadratic,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    // `A` is dimensionless — see the note in `where_three_planes_meet`.
    //
    // **`A` below zero is a real answer and a cone is what brings it.** A ball's
    // is `D·D` and a pipe's is `‖D × w‖²`, and a sum of squares is never
    // negative — which is what the arm here used to say. A cone's is
    // `‖D × w‖² − slope²(D·w)²`, negative for every line pointing further into
    // the cone than its own rulings do, which is most of the lines through it.
    // The roots are read off the same three numbers either way, and
    // [`the_two_places_along_it`] never divides by `A` except through Forsythe's
    // pairing, so nothing below cares about the sign.
    let squared_sign = match ask(
        &the_sign_of(named, &quadratic.squared),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        sign @ (Decision::Pos | Decision::Neg) => sign,
        Decision::Zero => {
            return where_a_line_meets_a_quadric_it_runs_alongside(
                named, quadratic, at, policy, log,
            );
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    };

    match ask(
        &the_discriminant_of(named, quadratic),
        Measures::Area,
        at,
        policy,
        log,
    )? {
        Decision::Neg => Ok(WhereALineMeetsIt::Nowhere),
        Decision::Zero => Ok(WhereALineMeetsIt::At(smallvec![MeetsAt::Here(
            where_it_touches(quadratic)
        )])),
        Decision::Pos => {
            let [lower, upper] = the_two_places_along_it(quadratic, squared_sign);
            Ok(WhereALineMeetsIt::At(smallvec![
                MeetsAt::Here(lower),
                MeetsAt::Here(upper),
            ]))
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// How far along the line it touches the quadric: `−B/A`, the one root a zero
/// discriminant leaves.
///
/// **That step exactly where it is an `f64`**, as a box of no width. A step with
/// width places the touching point as a box, and at a touch the questions a
/// fuse asks next are the ones whose true answer is zero — which way a wall
/// turns against the ball it touches is `orient3d` of three vectors in one
/// plane. No box with width round a zero decides that, however narrow; the
/// exact point does. Where `B` or `A` is not known exactly, or their quotient
/// is not an `f64`, the step is the box it always was.
fn where_it_touches(quadratic: &Quadratic) -> Interval {
    let exactly = quadratic
        .half_linear
        .exactly()
        .zip(quadratic.squared.exactly())
        .and_then(|(half_linear, squared)| {
            the_f64_quotient_of(&Exact::of(0.0).minus(&half_linear), &squared)
        });
    exactly.map_or_else(
        || {
            quotient_or_everything(
                -quadratic.half_linear.enclosure(),
                quadratic.squared.enclosure(),
            )
        },
        Interval::exactly,
    )
}

/// Where the line meets the quadric when the squared term is exactly zero, so
/// the equation is `2B·s + C = 0` and not a quadratic at all.
///
/// **A pipe and a cone reach this differently, and the difference is one root.**
/// A pipe's `A` is `‖D × w‖²`, so `A = 0` means the line runs along the axis and
/// then `B = (D × w)·(V × w)` is exactly zero too: the line lies on the wall or
/// misses it, and there is no single point either way. That was the whole of
/// this before there were cones. A cone's `A = 0` means the line runs parallel
/// to one of its rulings, and such a line crosses the cone at exactly **one**
/// point, with `B` nowhere near zero.
///
/// So the linear term is asked about first. For a ball and a pipe the answer is
/// always [`Decision::Zero`] and always certified — `B` is a dot product with an
/// exactly-zero vector — so this asks one more question than it used to and
/// gives every old answer back unchanged.
fn where_a_line_meets_a_quadric_it_runs_alongside(
    named: &'static str,
    quadratic: &Quadratic,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    match ask(
        &the_sign_of(named, &quadratic.half_linear),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Pos | Decision::Neg => {
            let twice_the_linear = Interval::exactly(2.0) * quadratic.half_linear.enclosure();
            return Ok(WhereALineMeetsIt::At(smallvec![MeetsAt::Here(
                quotient_or_everything(-quadratic.constant.enclosure(), twice_the_linear)
            )]));
        }
        Decision::Zero => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
    match ask(
        &the_sign_of(named, &quadratic.constant),
        Measures::Area,
        at,
        policy,
        log,
    )? {
        Decision::Zero => Ok(WhereALineMeetsIt::MoreThanAPoint),
        Decision::Pos | Decision::Neg => Ok(WhereALineMeetsIt::Nowhere),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// How far along the line each of the two meeting points sits, smaller first.
///
/// **`(−B ± √Δ) / A` on its own is not good enough, and the case that shows it
/// is not exotic.** A line a hair off parallel to a pipe's axis has `A = ‖D × w‖²`
/// down at `1e-32`, and then the two terms of `−B ± √Δ` cancel for one of the
/// two roots and what is left is a rounding of `B` divided by that `A`. Measured
/// on a bar's own cap plane turned by a sixty-fourth: a box **fifteen hundred
/// times the shape**, for a corner that is perfectly well determined.
///
/// So the pair is worked out the way a quadratic solver has since Forsythe.
/// `q = −(B + sign(B)·√Δ)` adds two numbers of the same sign and cancels
/// nothing; one root is `q/A` and the other is `C/q`, because the two roots
/// multiply to `C/A`. Both forms are valid enclosures of the same number — they
/// are the same identity — so the narrower of the two is taken for each root and
/// there is no branch to get wrong when `q` runs through zero.
///
/// **`squared_sign` is `A`'s sign, already certified by the caller.** `(−B −
/// √Δ)/A` and `(−B + √Δ)/A` are ascending in that order only while `A` is
/// positive; a cone's `A` is `‖D × w‖² − slope²(D·w)²`, negative for a line
/// that points further into the cone than its own rulings do (the note on `A`
/// in [`where_a_line_meets_a_quadric`]), and dividing by a negative `A` turns
/// the pair around. The caller already knows which; asking again here would
/// risk a plain enclosure check disagreeing with the certified answer it
/// handed in.
fn the_two_places_along_it(quadratic: &Quadratic, squared_sign: Decision) -> [Interval; 2] {
    // **Not read off `square_minus`**, which slid both sides up the range and
    // hands back a sign (§32.1).
    let root = root_enclosing(the_size_of_the_discriminant(quadratic));
    let squared = quadratic.squared.enclosure();
    let half_linear = quadratic.half_linear.enclosure();
    let plainly = |which: Interval| quotient_or_everything(-half_linear + which, squared);

    let leans_back = matches!(quadratic.half_linear.sign(), Decision::Neg);
    let steady = if leans_back {
        root - half_linear
    } else {
        -half_linear - root
    };
    let (well_placed, its_partner) = (
        quotient_or_everything(steady, squared),
        quotient_or_everything(quadratic.constant.enclosure(), steady),
    );
    let (lower, upper) = if leans_back {
        (its_partner, well_placed)
    } else {
        (well_placed, its_partner)
    };
    let smaller_when_a_positive = the_tighter(plainly(-root), lower);
    let larger_when_a_positive = the_tighter(plainly(root), upper);
    match squared_sign {
        Decision::Pos => [smaller_when_a_positive, larger_when_a_positive],
        Decision::Neg => [larger_when_a_positive, smaller_when_a_positive],
        other => {
            unreachable!("the caller certified squared_sign as Pos or Neg, not {other:?}")
        }
    }
}

/// The narrower of two enclosures of the same number.
fn the_tighter(one: Interval, other: Interval) -> Interval {
    if other.width() < one.width() {
        other
    } else {
        one
    }
}

/// The point `step` of the way along the line: exactly, where
/// [`the_point_exactly_at`] can name it, and a box round it everywhere else.
fn the_point_at(line: &ALine, step: Interval) -> PredicatePoint {
    if let Some(place) = the_point_exactly_at(line, step) {
        return PredicatePoint::Exact(place);
    }
    let divisor = line.divided_by.enclosure();
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        quotient_or_everything(
            line.through[axis].enclosure() + step * line.direction[axis].enclosure(),
            divisor,
        )
    }))
}

/// The point a step of no width names, where the line is known exactly and
/// each coordinate `(through + step·direction) / divided_by` is an `f64` — or
/// `None`.
///
/// A step of no width is one number and not a box round one, so the point it
/// names is one point; working it out in intervals would put back the width
/// [`where_it_touches`] kept out of the step.
fn the_point_exactly_at(line: &ALine, step: Interval) -> Option<[f64; 3]> {
    if step.lo() != step.hi() {
        return None;
    }
    let along = Term::of(step.lo());
    let divisor = line.divided_by.exactly()?;
    let mut place = [0.0; 3];
    for (axis, coordinate) in place.iter_mut().enumerate() {
        let over = line.through[axis].plus(&line.direction[axis].times(&along)?);
        *coordinate = the_f64_quotient_of(&over.exactly()?, &divisor)?;
    }
    Some(place)
}

/// `over / under`, where that is exactly an `f64` — or `None` where it is not,
/// or where a product the check takes would round.
///
/// **An answer comes back only where `under · q − over` is exactly zero**, so a
/// guess that misses costs an exact answer and never gives a wrong one.
///
/// The first guess divides the two numbers rounded, and each rounding can put
/// it a unit in the last place further off. One step of Newton's method on
/// `under · q − over`, whose value at the guess is worked out exactly, takes it
/// the rest of the way: where the quotient is an `f64` the step is a few units
/// long and wrong by a rounding of that, far less than the half unit that would
/// land it on a neighbour.
/// `an_f64_quotient_is_found_where_the_first_guess_lands_beside_it` counts the
/// quotients the first guess alone misses and the step does not.
///
/// # Panics
///
/// If `under` is zero. Both callers divide by a number already decided not to
/// be: a quadratic's `A` and a line's divisor.
fn the_f64_quotient_of(over: &Exact, under: &Exact) -> Option<f64> {
    assert!(
        matches!(under.sign(), Decision::Pos | Decision::Neg),
        "a quotient is only asked of a divisor decided not to be zero: {under:?}"
    );
    let divisor = under.nearest_f64();
    let missed_by = |guess: f64| Some(under.times(&Exact::of(guess))?.minus(over));
    let guess = over.nearest_f64() / divisor;
    let settled = guess - missed_by(guess)?.nearest_f64() / divisor;
    match missed_by(settled)?.sign() {
        Decision::Zero => Some(settled),
        Decision::Pos | Decision::Neg | Decision::Undecidable { .. } => None,
        other => unreachable!("an exact sign is a sign or a refusal: {other:?}"),
    }
}

/// The sign of a number, as a question the cascade can ask.
///
/// A number that is only a box has **no exact level**, and that is the box being
/// all there ever was rather than a step missed: nothing recovers a number a
/// caller never had (§2.5).
fn the_sign_of<'a>(
    named: &'static str,
    value: &'a Term,
) -> Question<impl Fn() -> Interval + 'a, impl Fn() -> Option<Exact> + 'a> {
    Question {
        named,
        enclosing: move || value.enclosure(),
        exactly: move || value.exactly(),
    }
}

/// `B² − A·C`, as a question the cascade can ask.
fn the_discriminant_of<'a>(
    named: &'static str,
    quadratic: &'a Quadratic,
) -> Question<impl Fn() -> Interval + 'a, impl Fn() -> Option<Exact> + 'a> {
    Question {
        named,
        enclosing: move || the_size_of_the_discriminant(quadratic),
        exactly: move || {
            quadratic
                .half_linear
                .exactly()?
                .square_minus(&quadratic.squared.times(&quadratic.constant)?.exactly()?)
        },
    }
}

/// How big the discriminant is, which is a different number from what says which
/// way it points.
///
/// **Summed through the expansion and not in intervals**, and the difference is
/// a box a rounding wide against a box two hundred times the shape. A line
/// nearly tangent to a quadric has `B²` and `A·C` two large numbers that cancel
/// to nothing, and an interval evaluation of the difference reaches out to the
/// rounding of the larger — so `√Δ` carries it, and so does every point built on
/// it. Summed exactly the difference is known to full relative accuracy, which
/// is §29's lesson and §33.4's, one question up.
///
/// The fallback is the interval form, for the rare case where a product the
/// exact route needs falls out of range. It is honest and it is wide.
fn the_size_of_the_discriminant(quadratic: &Quadratic) -> Interval {
    if let Some(exactly) = quadratic.difference_of_the_two_halves()
        && !exactly.lost_more_than_a_rounding()
    {
        return exactly.enclosure();
    }
    let half_linear = quadratic.half_linear.enclosure();
    half_linear * half_linear - quadratic.squared.enclosure() * quadratic.constant.enclosure()
}

/// `first · second − third · fourth`, exactly, or `None` when either product
/// would round.
fn cross_of_two_pairs(first: f64, second: f64, third: f64, fourth: f64) -> Option<Exact> {
    Some(Exact::product_of(first, second)?.minus(&Exact::product_of(third, fourth)?))
}

/// `u × v` for two triples of stored numbers, exactly.
fn exact_cross_of(u: [f64; 3], v: [f64; 3]) -> Option<[Exact; 3]> {
    let mut across = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, component) in across.iter_mut().enumerate() {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        *component = cross_of_two_pairs(u[next], v[after], u[after], v[next])?;
    }
    Some(across)
}

/// The refusal for a step of exact arithmetic that could not be taken at all.
///
/// A **gap** and never a limit, and the same one every discriminant in
/// [`intersect`](crate::intersect) has: a product whose low half would land
/// among the subnormals, or one that overflows. The same three surfaces written
/// somewhere else on the number line are answered.
fn an_exact_step_could_not_be_taken(named: &'static str, at: &[PredicatePoint]) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        named,
        at,
        Level::Exact,
        f64::INFINITY,
        Measures::Area,
    ));
    Error::Undecidable { records }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, Point, Vector};

    fn flat(normal: [f64; 3], through: [f64; 3]) -> Surface {
        Surface::Plane(
            Plane::from_point_and_normal(
                Point::new(through[0], through[1], through[2]),
                Vector::new(normal[0], normal[1], normal[2]),
            )
            .expect("a normal with length"),
        )
    }

    fn meet(surfaces: [&Surface; 3]) -> Result<MeetOfThree> {
        meet_of_three_surfaces(surfaces, &Policy::certified(), &mut ProvenanceLog::new())
    }

    fn middles(answer: &MeetOfThree) -> Vec<[f64; 3]> {
        let MeetOfThree::At(points) = answer else {
            panic!("expected points, got {answer:?}");
        };
        points
            .iter()
            .map(|point| point.enclosure().map(|side| (side.lo() + side.hi()) / 2.0))
            .collect()
    }

    /// A reach of `±2¹⁰⁰⁰` slid down by `2⁻²⁰⁶⁰` lands on `±2⁻¹⁰⁶⁰`, a subnormal
    /// `f64` holds, and the box that comes back holds it.
    ///
    /// `multiplied_by_a_power_of_two` takes `2⁻²⁰⁶⁰` as two factors of
    /// `2⁻¹⁰³⁰`, and `2f64.powi(-1030)` is zero, so it would put both ends at
    /// zero and one subnormal out from there is `2⁻¹⁰⁷⁴`, which does not hold
    /// `2⁻¹⁰⁶⁰`. A ring's reach is slid by four times a slide, so this many
    /// binades is a slide of 515, which a ring `2⁻⁵¹⁵` across asks for.
    #[test]
    fn a_reach_slid_down_past_2046_binades_still_holds_where_it_lands() {
        let lands = 2f64.powi(-1000) * 2f64.powi(-60);
        assert!(lands > 0.0 && !lands.is_normal(), "2^-1060 is a subnormal");
        let reach = Interval::new(-(2f64.powi(1000)), 2f64.powi(1000)).expect("a box");
        let slid = a_reach_slid_back(reach, -2060);
        assert!(
            slid.lo() <= -lands && lands <= slid.hi(),
            "{slid:?} does not hold ±2^-1060"
        );
        assert_eq!(
            (slid.lo(), slid.hi()),
            (-lands.next_up(), lands.next_up()),
            "one subnormal out from where it landed, and no further"
        );
    }

    /// Ring 12/5 about `z` and the radius-5 pipe through `(12, 0, 0)` along
    /// `y`, which share the circle in `y = 0` and a loop through `(9, 12, ±4)`.
    /// The plane `y = 0` holds the circle; `4x = 3y` holds the axis and cuts
    /// the loop's parabola `x = 12 − y²/48` at `y = 12` and `y = −48`, and only
    /// the first is close enough to the pipe. Ring 3/1 against the bar a bend's
    /// cap is butted on, with its far end on `x + y = 0`, meets that end at
    /// `x = −y = 6(√2 − 1)`, `z = ±√(1 − y⁴/144)`.
    #[test]
    fn where_a_plane_through_a_rings_axis_cuts_the_loop_a_pipe_along_its_core_makes() {
        let ring = |core: f64, tube: f64| {
            Surface::Torus(Torus::new(Axis::z(), core, tube).expect("a ring thicker than nothing"))
        };
        let pipe = |through: f64, along: f64, radius: f64| {
            Surface::Cylinder(
                Cylinder::new(
                    Axis::new(Point::new(through, 0.0, 0.0), Vector::new(0.0, along, 0.0))
                        .expect("an axis"),
                    radius,
                )
                .expect("a radius"),
            )
        };
        let (big, along_y) = (ring(12.0, 5.0), pipe(12.0, 1.0, 5.0));
        let cap = flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(
            meet([&cap, &big, &along_y]),
            Ok(MeetOfThree::MoreThanAPoint)
        );

        let leaning = flat([12.0, -9.0, 0.0], [0.0, 0.0, 0.0]);
        for order in [[&leaning, &big, &along_y], [&along_y, &leaning, &big]] {
            let MeetOfThree::At(places) = meet(order).expect("a plane holding the axis") else {
                panic!("two points on the loop");
            };
            let held: Vec<bool> = places
                .iter()
                .zip([[9.0, 12.0, -4.0], [9.0, 12.0, 4.0]])
                .map(|(place, point)| {
                    place
                        .enclosure()
                        .iter()
                        .zip(point)
                        .all(|(side, coordinate)| {
                            side.lo() <= coordinate && coordinate <= side.hi()
                        })
                })
                .collect();
            assert_eq!(held, vec![true, true], "{places:?}");
        }

        let (small, bar) = (ring(3.0, 1.0), pipe(3.0, -1.0, 1.0));
        let far_end = flat([1.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        let answer = meet([&far_end, &small, &bar]).expect("the bar's far end crosses the loop");
        let out = 6.0 * (2.0_f64.sqrt() - 1.0);
        let up = (1.0 - out.powi(4) / 144.0).sqrt();
        let missed: Vec<f64> = middles(&answer)
            .iter()
            .zip([[out, -out, -up], [out, -out, up]])
            .map(|(found, point)| {
                (0..3)
                    .map(|axis| (found[axis] - point[axis]).abs())
                    .fold(0.0, f64::max)
            })
            .collect();
        assert!(
            missed.len() == 2 && missed.iter().all(|by| *by < 1e-12),
            "{answer:?} missed by {missed:?}"
        );
    }

    /// A plane square to a ring's axis cuts it in two circles about the axis,
    /// and a bar, a ball or a cone on that axis in one, so the three share one
    /// whole circle or nothing.
    ///
    /// `z = 0` cuts ring 4/1 in its equators, of radii 5 and 3: the top rim of
    /// a bar of radius 5 standing under the top half of the ring is the outer
    /// one, and a bar of radius 3 meets the inner. `z = 3` cuts ring 8/5 in
    /// circles of radii `8 ± 4`, a ball of radius 13 about `(0, 0, 8)` in one
    /// of radius 12, and a cone of slope one with its apex at `(0, 0, −1)` in
    /// one of radius 4. A plane past the tube, or a round surface whose circle
    /// is neither, leaves nothing. A bar of radius 5 off the ring's axis cuts
    /// `z = 0` in a circle about another centre, which this does not take.
    #[test]
    fn a_plane_square_to_a_rings_axis_and_a_round_surface_on_it_share_a_circle_or_nothing() {
        let ring = |core: f64, tube: f64| {
            Surface::Torus(Torus::new(Axis::z(), core, tube).expect("a ring thicker than nothing"))
        };
        let on_z = |through: [f64; 3]| {
            Axis::new(
                Point::new(through[0], through[1], through[2]),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("an axis")
        };
        let bar = |through: [f64; 3], radius: f64| {
            Surface::Cylinder(Cylinder::new(on_z(through), radius).expect("a radius"))
        };
        let ball = |height: f64, radius: f64| {
            Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, height), radius).expect("a radius"))
        };
        let cone =
            |apex: f64| Surface::Cone(Cone::new(on_z([0.0, 0.0, apex]), 1.0).expect("a slope"));
        let across = |height: f64, up: f64| flat([0.0, 0.0, up], [0.0, 0.0, height]);
        let (thin, fat) = (ring(4.0, 1.0), ring(8.0, 5.0));

        let shared = [
            (across(0.0, 1.0), &thin, bar([0.0, 0.0, -2.0], 5.0)),
            (across(0.0, -1.0), &thin, bar([0.0, 0.0, -2.0], 5.0)),
            (across(0.0, 1.0), &thin, bar([0.0, 0.0, 0.0], 3.0)),
            (across(3.0, 1.0), &fat, bar([0.0, 0.0, 0.0], 12.0)),
            (across(3.0, -1.0), &fat, ball(8.0, 13.0)),
            (across(3.0, 1.0), &fat, cone(-1.0)),
        ];
        for (plane, ring, round) in &shared {
            for order in [[plane, *ring, round], [round, plane, *ring]] {
                assert_eq!(
                    meet(order),
                    Ok(MeetOfThree::MoreThanAPoint),
                    "{plane:?} {round:?}"
                );
            }
        }

        let apart = [
            (across(0.0, 1.0), &thin, bar([0.0, 0.0, 0.0], 4.5)),
            (across(0.0, 1.0), &thin, bar([0.0, 0.0, 0.0], 1.0)),
            (across(3.0, 1.0), &fat, ball(8.0, 12.0)),
            (across(3.0, 1.0), &fat, cone(-2.0)),
            (across(6.0, 1.0), &fat, bar([0.0, 0.0, 0.0], 8.0)),
            (across(1.0, 1.0), &thin, ball(9.0, 1.0)),
        ];
        for (plane, ring, round) in &apart {
            assert_eq!(
                meet([plane, *ring, round]),
                Ok(MeetOfThree::Nowhere),
                "{plane:?} {round:?}"
            );
        }

        assert_eq!(
            meet([&across(0.0, 1.0), &thin, &bar([1.0, 0.0, 0.0], 5.0)]),
            Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
        );
    }

    #[test]
    fn three_planes_square_to_the_axes_meet_where_they_say() {
        let answer = meet([
            &flat([1.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
            &flat([0.0, 1.0, 0.0], [0.0, 3.0, 0.0]),
            &flat([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]),
        ])
        .expect("three planes square to each other meet");
        assert_eq!(middles(&answer), vec![[2.0, 3.0, 4.0]]);
    }

    #[test]
    fn two_planes_through_a_balls_centre_meet_it_at_both_poles() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let answer = meet([
            &flat([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            &ball,
        ])
        .expect("a line through the centre cuts a ball twice");
        assert_eq!(middles(&answer), vec![[0.0, 0.0, -5.0], [0.0, 0.0, 5.0]]);
    }

    #[test]
    fn two_planes_meeting_a_pipe_land_on_its_wall() {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        let answer = meet([
            &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            &flat([0.0, 0.0, 1.0], [0.0, 0.0, 7.0]),
            &pipe,
        ])
        .expect("a line across a pipe cuts it twice");
        assert_eq!(middles(&answer), vec![[-3.0, 0.0, 7.0], [3.0, 0.0, 7.0]]);
    }

    #[test]
    fn a_line_that_grazes_a_ball_meets_it_once_and_one_that_misses_it_not_at_all() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let grazing = meet([
            &flat([1.0, 0.0, 0.0], [5.0, 0.0, 0.0]),
            &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            &ball,
        ])
        .expect("a tangent line still meets the ball");
        assert_eq!(middles(&grazing), vec![[5.0, 0.0, 0.0]]);

        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [6.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &ball,
            ]),
            Ok(MeetOfThree::Nowhere)
        );
    }

    /// A line that touches a ball where every coordinate is an `f64` comes
    /// back as that point, not as a box round it: straight down the lattice,
    /// and down the `z` axis between two walls whose normals are not on it.
    ///
    /// The second is the corner a ball touching a prism's upright edge makes,
    /// and as a box with width in `z` it stopped every fuse of the two under a
    /// certified policy: `crates/rcad-topo/tests/a_ball_touching_a_prisms_edge_exactly.rs`.
    #[test]
    fn a_line_that_touches_a_ball_at_a_point_f64_holds_touches_it_exactly_there() {
        let ball = |centre: [f64; 3], radius: f64| {
            Surface::Sphere(
                Sphere::new(Point::new(centre[0], centre[1], centre[2]), radius).expect("a radius"),
            )
        };
        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [5.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &ball([0.0; 3], 5.0),
            ]),
            Ok(MeetOfThree::At(smallvec![PredicatePoint::Exact([
                5.0, 0.0, 0.0
            ])]))
        );
        assert_eq!(
            meet([
                &flat([3.0, -4.0, 0.0], [0.0; 3]),
                &flat([-1.0, -1.0, 0.0], [0.0; 3]),
                &ball([1.0, 0.0, 16.0], 1.0),
            ]),
            Ok(MeetOfThree::At(smallvec![PredicatePoint::Exact([
                0.0, 0.0, 16.0
            ])]))
        );
    }

    /// Where `over / under` is an `f64`, [`the_f64_quotient_of`] finds it —
    /// including where the first guess, the two rounded and divided, lands on
    /// a neighbour of it. And where it is not an `f64`, nothing comes back.
    ///
    /// `under` is a number and a part of a unit in its last place, so rounding
    /// it moves it, and that is one of the three roundings that can move the
    /// guess. The counts say what each way of guessing does with the same
    /// quotients. Measured: of 100 000, the first guess alone missed 37 726,
    /// the first guess and its two neighbours missed 137, and the Newton step
    /// missed none.
    #[test]
    fn an_f64_quotient_is_found_where_the_first_guess_lands_beside_it() {
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut one_in_one_to_two = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            f64::from_bits((0x3ff << 52) | (state >> 12))
        };
        let (mut the_guess_missed, mut a_neighbour_missed) = (0, 0);
        const TRIES: usize = 100_000;
        for attempt in 0..TRIES {
            let quotient = if attempt % 2 == 0 {
                one_in_one_to_two()
            } else {
                -one_in_one_to_two()
            };
            let under = Exact::of(one_in_one_to_two())
                .plus(&Exact::of(one_in_one_to_two() * 2f64.powi(-54)));
            let over = under
                .times(&Exact::of(quotient))
                .expect("nothing here is near the ends of the range");
            let guess = over.nearest_f64() / under.nearest_f64();
            if guess != quotient {
                the_guess_missed += 1;
            }
            if ![guess, guess.next_up(), guess.next_down()].contains(&quotient) {
                a_neighbour_missed += 1;
            }
            assert_eq!(
                the_f64_quotient_of(&over, &under),
                Some(quotient),
                "{over:?} / {under:?}"
            );
        }
        println!(
            "of {TRIES} quotients that are an f64, the first guess missed {the_guess_missed}, \
             the guess and its two neighbours missed {a_neighbour_missed}, the Newton step none"
        );
        assert!(
            the_guess_missed > 0,
            "no first guess missed, so this search says nothing about the step"
        );

        assert_eq!(the_f64_quotient_of(&Exact::of(1.0), &Exact::of(3.0)), None);
        for under in [3.0, -3.0] {
            assert_eq!(
                the_f64_quotient_of(&Exact::of(0.0), &Exact::of(under)).map(f64::to_bits),
                Some(0.0f64.to_bits()),
                "a zero over {under} is the zero every other zero is, and not -0.0"
            );
        }
    }

    #[test]
    fn a_line_down_a_pipes_own_wall_is_more_than_a_point() {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [3.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &pipe,
            ]),
            Ok(MeetOfThree::MoreThanAPoint)
        );
    }

    #[test]
    fn a_line_that_runs_beside_a_pipe_meets_it_nowhere() {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [4.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &pipe,
            ]),
            Ok(MeetOfThree::Nowhere)
        );
    }

    /// Two faces on one plane with a curved surface across them share the whole
    /// of what the plane and the curve share, and that is a curve rather than a
    /// corner (§31.3's coincident-face case).
    #[test]
    fn two_faces_on_one_plane_and_a_ball_share_a_whole_circle() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let wall = flat([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        assert_eq!(meet([&wall, &wall, &ball]), Ok(MeetOfThree::MoreThanAPoint));

        let elsewhere = flat([0.0, 0.0, 1.0], [0.0, 0.0, 9.0]);
        assert_eq!(meet([&wall, &elsewhere, &ball]), Ok(MeetOfThree::Nowhere));
    }

    /// Three planes that do meet at a point, and meet it past the top of the
    /// range, come back as a refusal rather than as a box that says nothing.
    ///
    /// Cramer's rule divides one interval determinant by another and checks
    /// neither for overflow, so `Plane::meet_of_three` hands back `Ok` holding a
    /// box whose every side runs the whole number line — which a caller would
    /// read as a place. The two planes below cross in a line at the top of the
    /// range and the third leans across it by about `1e-16`, so the meet is at
    /// about `1e315` and no `f64` holds it.
    ///
    /// A **limit**, and §34.10 filed it as a gap. The second half below writes
    /// the same three planes lower down and they are placed — but that is a
    /// different triple with a different answer, and *this* triple's answer is a
    /// point at about `1e315` that no `f64` names. §16.10's two words go by
    /// whether the answer exists, and here it does not. The refusal is right and
    /// there is nothing to fix. §39.
    #[test]
    fn three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere() {
        let leaning = [1.0, 1.0, 2f64.powi(-50)];
        let triple = |reach: f64| {
            [
                flat([1.0, 0.0, 0.0], [reach, 0.0, 0.0]),
                flat([0.0, 1.0, 0.0], [0.0, reach, 0.0]),
                flat(leaning, [0.0, 0.0, 0.0]),
            ]
        };

        let far = triple(1e300);
        let answer = meet([&far[0], &far[1], &far[2]]);
        assert!(
            matches!(
                answer,
                Err(Error::Predicate(_)) | Err(Error::Undecidable { .. })
            ),
            "three planes meeting past the top of the range came back as {answer:?}"
        );

        // The same three planes written lower down are placed, so what ran out
        // is the range and not the geometry.
        let near = triple(1.0);
        let placed = meet([&near[0], &near[1], &near[2]]).expect("these three meet at a point");
        let MeetOfThree::At(points) = &placed else {
            panic!("expected one point, got {placed:?}");
        };
        assert_eq!(points.len(), 1);
        assert!(points[0].uncertainty().is_finite());
    }

    /// A floor and two walls a millionth off parallel: Cramer's box round
    /// where they meet is about `1e-10` wide, and one checked step boxes the
    /// same meet to a few ulps (§281.2, §345). The narrowed box is checked to
    /// hold the meet with nothing rounded, from the stored numbers: along each
    /// axis the meet is `D / det`, so it is inside `[lo, hi]` when
    /// `D − lo·det` and `hi·det − D` both have the sign of `det`.
    #[test]
    fn a_meet_of_two_walls_nearly_parallel_is_boxed_to_a_few_ulps_and_still_holds_it() {
        for lean in [1e-6, 1.5e-6, 3e-6] {
            let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
            let wall = flat([0.6, 0.0, 0.8], [5.0, 0.0, 0.0]);
            let leaning = flat([0.6, lean, 0.8], [5.0, 0.0, 0.0]);
            let three = [&floor, &wall, &leaning];
            let by_cramer = meet(three).expect("three planes meet");
            let MeetOfThree::At(narrowed) = by_cramer.boxed_tight(three) else {
                panic!("three planes a millionth off parallel meet at a point");
            };
            let MeetOfThree::At(by_cramer) = by_cramer else {
                panic!("three planes a millionth off parallel meet at a point");
            };
            let (wide, tight) = (by_cramer[0].enclosure(), narrowed[0].enclosure());
            for axis in 0..3 {
                assert!(
                    wide[axis].lo() <= tight[axis].lo() && tight[axis].hi() <= wide[axis].hi(),
                    "leaning {lean}: the narrowed box {tight:?} is not inside Cramer's {wide:?}"
                );
            }
            assert!(
                by_cramer[0].uncertainty() > 1e-12,
                "leaning {lean}: Cramer's box is already narrow, {wide:?}"
            );
            assert!(
                narrowed[0].uncertainty() < 1e-14,
                "leaning {lean}: one step left the box {tight:?}"
            );
            let planes = in_a_fixed_order(three).map(|surface| {
                let Surface::Plane(flat) = surface else {
                    unreachable!("all three are planes")
                };
                flat
            });
            assert!(
                (0..3).all(|axis| the_meet_is_inside(&planes, axis, tight[axis])),
                "leaning {lean}: the narrowed box {tight:?} does not hold the meet"
            );
            let just_past = Interval::new(tight[0].hi() + 1e-12, tight[0].hi() + 2e-12)
                .expect("a box past the narrowed one");
            assert!(
                !the_meet_is_inside(&planes, 0, just_past),
                "leaning {lean}: the exact check holds the meet in a box past it, {just_past:?}"
            );
        }
    }

    /// Whether the one point three planes meet at lies inside `side` along
    /// `axis`, decided exactly.
    fn the_meet_is_inside(planes: &[Plane; 3], axis: usize, side: Interval) -> bool {
        let normals = planes.map(|plane| plane.normal().components());
        let across = [
            exact_cross_of(normals[1], normals[2]).expect("an exact cross product"),
            exact_cross_of(normals[2], normals[0]).expect("an exact cross product"),
            exact_cross_of(normals[0], normals[1]).expect("an exact cross product"),
        ];
        let determinant = Exact::dot_of_two_exact(&Exact::of_each(normals[0]), &across[0])
            .expect("an exact determinant");
        let numerator = (0..3).fold(Exact::of(0.0), |so_far, row| {
            so_far.plus(
                &Exact::of(planes[row].offset())
                    .times(&across[row][axis])
                    .expect("an exact product"),
            )
        });
        let above_the_bottom = numerator.minus(
            &Exact::of(side.lo())
                .times(&determinant)
                .expect("an exact product"),
        );
        let below_the_top = Exact::of(side.hi())
            .times(&determinant)
            .expect("an exact product")
            .minus(&numerator);
        let agrees = |value: &Exact| match (value.sign(), determinant.sign()) {
            (Decision::Zero, _) => true,
            (Decision::Pos, Decision::Pos) | (Decision::Neg, Decision::Neg) => true,
            (Decision::Pos | Decision::Neg, Decision::Pos | Decision::Neg) => false,
            other => panic!("an exact sign that is not a sign: {other:?}"),
        };
        agrees(&above_the_bottom) && agrees(&below_the_top)
    }

    /// A bar's bottom cap, `z = 0`, a block's face turned by `−π/4` about `x` to
    /// lean along `z = y`, and the bar's wall, `x² + y² = 25`: the meet near
    /// the rim start `(5, 0, 0)` that §284 could not order round the cap
    /// circle. The quadratic along the two planes' line boxes it `±3` ulps of
    /// `5` along `x`, and one checked step onto the wall boxes it one ulp
    /// wide. Each side of the narrowed box is checked to hold the meet with
    /// nothing rounded, from the stored numbers. On `z = 0` the turned face
    /// `n·p = d` puts the meet at `y = d / n_y`, and the wall then puts it at
    /// `x² = 25 − y²` with `x` above zero, so `x` lies in `[lo, hi]` when
    /// `25 n_y² − d²` lies between `lo² n_y²` and `hi² n_y²`.
    #[test]
    fn a_meet_of_two_planes_and_a_pipe_is_boxed_to_one_ulp_by_one_step_and_still_holds_it() {
        let lean = crate::Isometry::rotation(&Axis::x(), -core::f64::consts::FRAC_PI_4)
            .expect("a finite turn")
            .compose(
                &crate::Isometry::translation(Vector::new(0.0, 0.0, -20.0))
                    .expect("a finite shift"),
            );
        let cap = flat([0.0, 0.0, -1.0], [0.0, 0.0, 0.0]);
        let turned = flat([0.0, -1.0, 0.0], [0.0, 0.0, 0.0]).transformed(&lean);
        let wall = Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a radius"));
        let three = [&cap, &turned, &wall];
        let as_placed = meet(three).expect("two planes and a pipe meet");
        let MeetOfThree::At(narrowed) = as_placed.boxed_tight(three) else {
            panic!("the turned face crosses the cap circle at two points");
        };
        let MeetOfThree::At(as_placed) = as_placed else {
            panic!("the turned face crosses the cap circle at two points");
        };
        let near = (0..as_placed.len())
            .find(|&which| as_placed[which].enclosure()[0].lo() > 0.0)
            .expect("one of the two is near (5, 0, 0)");
        let (wide, tight) = (as_placed[near].enclosure(), narrowed[near].enclosure());
        for axis in 0..3 {
            assert!(
                wide[axis].lo() <= tight[axis].lo() && tight[axis].hi() <= wide[axis].hi(),
                "the narrowed box {tight:?} is not inside the quadratic's {wide:?}"
            );
        }
        let ulp = 5.0_f64.next_up() - 5.0;
        assert!(
            wide[0].width() >= 4.0 * ulp,
            "the quadratic's box is already narrow along x, {wide:?}"
        );
        assert!(
            tight[0].width() <= ulp,
            "one step left the box {tight:?} wider than an ulp along x"
        );

        let Surface::Plane(turned) = turned else {
            unreachable!("a turned plane is a plane")
        };
        let [_, n_y, n_z] = turned.normal().components();
        assert!(n_z != 0.0 && n_y < 0.0, "the turned face leans along z = y");
        let offset = turned.offset();
        let exactly =
            |one: f64, other: f64| Exact::product_of(one, other).expect("an exact product");
        let at_or_above_zero =
            |value: Exact| matches!(value.sign(), Decision::Pos | Decision::Zero);
        let inside_along_y = at_or_above_zero(
            Exact::of(tight[1].lo())
                .times(&Exact::of(n_y))
                .expect("an exact product")
                .minus(&Exact::of(offset)),
        ) && at_or_above_zero(
            Exact::of(offset).minus(
                &Exact::of(tight[1].hi())
                    .times(&Exact::of(n_y))
                    .expect("an exact product"),
            ),
        );
        assert!(
            inside_along_y,
            "the narrowed box {tight:?} misses the meet's y"
        );
        let squared_x_times = exactly(5.0, 5.0)
            .times(&exactly(n_y, n_y))
            .expect("an exact product")
            .minus(&exactly(offset, offset));
        let past = |side: f64| {
            exactly(side, side)
                .times(&exactly(n_y, n_y))
                .expect("an exact product")
        };
        let holds_x = |lo: f64, hi: f64| {
            lo > 0.0
                && at_or_above_zero(squared_x_times.minus(&past(lo)))
                && at_or_above_zero(past(hi).minus(&squared_x_times))
        };
        assert!(
            holds_x(tight[0].lo(), tight[0].hi()),
            "the narrowed box {tight:?} misses the meet's x"
        );
        assert!(
            !holds_x(5.0_f64.next_up(), 5.0_f64.next_up().next_up()),
            "the exact check holds the meet's x in a box one ulp past it"
        );
        println!("as placed {wide:?}, narrowed {tight:?}");
    }

    /// The bottom face of the block leaned `2` and turned `1` in
    /// `a_plane_cut_along_a_cones_other_nappe.rs`, one of its side walls, and
    /// the frustum's cone, by the numbers the block stores. The side edge runs
    /// a rounding off parallel to a ruling, so it meets the cone a second time
    /// `2.26e16` out, at a graze. The quadratic boxes that meet several ulps
    /// wide and the checked step cannot narrow it, since its row straddles
    /// nothing over a box that wide; the sign along the line boxes each side
    /// between two neighbouring `f64` (§406.3).
    #[test]
    fn the_far_meet_of_an_edge_leaned_along_a_ruling_is_boxed_between_two_neighbouring_f64() {
        let plane = |normal: [f64; 3], offset: f64| {
            let made = Plane::from_coefficients([normal[0], normal[1], normal[2], -offset])
                .expect("a plane");
            assert_eq!(
                (made.normal().components(), made.offset()),
                (normal, offset),
                "the plane is stored as the block stores it"
            );
            Surface::Plane(made)
        };
        let bottom = plane(
            [
                -0.48326107372841764,
                -0.7526345292496597,
                -0.447213595499958,
            ],
            1.3416407864998727,
        );
        let side = plane(
            [
                -0.8414709848078965,
                0.5403023058681398,
                5.551115123125783e-17,
            ],
            4.999999999999998,
        );
        let cone = Surface::Cone(Cone::new(Axis::z(), 0.5).expect("a slope"));
        let three = [&side, &bottom, &cone];
        let as_placed = meet(three).expect("two planes and a cone meet");
        let MeetOfThree::At(places) = &as_placed else {
            panic!("the side edge meets the cone at two points: {as_placed:?}");
        };
        let far = places
            .iter()
            .position(|place| place.enclosure()[2].hi() < -1e16)
            .expect("one meet is 2.26e16 down the lower nappe");
        let MeetOfThree::At(narrowed) = as_placed.boxed_tight(three) else {
            panic!("a meet at points boxed tighter is a meet at those points");
        };
        let (wide, tight) = (places[far].enclosure(), narrowed[far].enclosure());
        for axis in 0..3 {
            assert!(
                wide[axis].lo() <= tight[axis].lo() && tight[axis].hi() <= wide[axis].hi(),
                "the narrowed box {tight:?} is not inside the quadratic's {wide:?}"
            );
            assert!(
                tight[axis].hi() <= tight[axis].lo().next_up(),
                "the narrowed box {tight:?} is wider than two neighbouring f64 along {axis}"
            );
        }
        let ulps_wide = |side: Interval| {
            let mut count = 0;
            let mut at = side.lo();
            while at < side.hi() {
                at = at.next_up();
                count += 1;
            }
            count
        };
        assert!(
            (0..3).all(|axis| ulps_wide(wide[axis]) >= 4),
            "the quadratic's box {wide:?} is already narrow"
        );

        let [Surface::Plane(one), Surface::Plane(other), third] = in_a_fixed_order(three) else {
            unreachable!("two planes sort before a cone")
        };
        let written = how_a_surface_is_written(&third).expect("a cone is a quadric");
        assert_eq!(
            narrowed_onto_a_quadric_by_one_checked_step([&one, &other], &written, places[far]),
            places[far],
            "the checked step narrows it on its own"
        );
    }

    /// The sign of a ball's expression along the `x` axis, which the planes
    /// `y = 0` and `z = 0` cross in, asked where nothing rounds: inside at
    /// `4`, on it at `5` and outside at `6`.
    #[test]
    fn the_sign_along_two_planes_line_is_inside_on_or_outside_the_ball_where_it_is() {
        let floor = Plane::from_coefficients([0.0, 1.0, 0.0, 0.0]).expect("a plane");
        let wall = Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a plane");
        let ball = how_a_surface_is_written(&Surface::Sphere(
            Sphere::new(Point::origin(), 5.0).expect("a radius"),
        ))
        .expect("a ball is a quadric");
        let sign_at =
            |x: f64| how_the_quadric_leans_where_the_line_reaches([&floor, &wall], &ball, 0, x);
        assert_eq!(sign_at(4.0), Some(Decision::Neg));
        assert_eq!(sign_at(5.0), Some(Decision::Zero));
        assert_eq!(sign_at(6.0), Some(Decision::Pos));
        assert_eq!(sign_at(-6.0), Some(Decision::Pos));
        assert_eq!(
            how_the_quadric_leans_where_the_line_reaches([&floor, &wall], &ball, 1, 0.0),
            None,
            "the line is square to y, so no point of it is picked out by y"
        );
    }

    /// The `x` axis runs into the ball at `x = −5` and out of it at `x = 5`, so
    /// the ball's gradient points against `n₁ × n₂ = +x` at the first and with
    /// it at the second, and turns round with the two planes.
    #[test]
    fn a_line_runs_into_a_ball_where_it_meets_it_first_and_out_where_it_meets_it_last() {
        let floor = Plane::from_coefficients([0.0, 1.0, 0.0, 0.0]).expect("a plane");
        let wall = Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a plane");
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let near = |x: f64| {
            PredicatePoint::Constructed(
                [x, 0.0, 0.0].map(|at| Interval::new(at - 1e-9, at + 1e-9).expect("a range")),
            )
        };
        let way = |planes: [&Plane; 2], x: f64| {
            MeetOfThree::which_way_a_line_runs_through_a_quadric(planes, &ball, near(x))
        };
        assert_eq!(way([&floor, &wall], -5.0), Some(Decision::Neg));
        assert_eq!(way([&floor, &wall], 5.0), Some(Decision::Pos));
        assert_eq!(way([&wall, &floor], -5.0), Some(Decision::Pos));
        assert_eq!(way([&wall, &floor], 5.0), Some(Decision::Neg));
        assert_eq!(
            way([&floor, &wall], 0.0),
            None,
            "a box round the middle holds no meet"
        );
    }

    /// §244.3's block edge, `y = 1` and `z = −2⁻²⁵`, grazing a ball of radius
    /// `1 + 2⁻⁵¹` centred at `x = 16`: its point at `16` is `2⁻¹⁰³` inside the
    /// ball, and the boxes of its two meets with the ball end and start there.
    /// A ball of radius five holds the `x` axis's point at `5` and not at `4`.
    #[test]
    fn a_line_is_proved_off_a_ball_where_it_is_inside_it_and_not_where_it_is_on_it() {
        let top = -f64::powi(2.0, -25);
        let lid = Plane::from_coefficients([0.0, 0.0, 1.0, -top]).expect("a plane");
        let side = Plane::from_coefficients([0.0, 1.0, 0.0, -1.0]).expect("a plane");
        let grazed = Surface::Sphere(
            Sphere::new(Point::new(16.0, 0.0, 0.0), 1.0 + f64::powi(2.0, -51)).expect("a radius"),
        );
        let off = |planes: [&Plane; 2], quadric: &Surface, axis: usize, along: f64| {
            MeetOfThree::the_line_is_off_the_quadric_at(planes, quadric, axis, along)
        };
        let MeetOfThree::At(places) = meet([&grazed, &Surface::Plane(lid), &Surface::Plane(side)])
            .expect("a ball and two planes")
        else {
            panic!("the edge cuts the ball");
        };
        let ends: Vec<[f64; 2]> = places
            .iter()
            .map(|place| [place.enclosure()[0].lo(), place.enclosure()[0].hi()])
            .collect();
        assert_eq!(
            ends,
            [
                [15.999_999_999_999_998, 16.0],
                [16.0, 16.000_000_000_000_004]
            ]
        );
        assert!(off([&lid, &side], &grazed, 0, 16.0), "inside at 16");
        assert!(off([&side, &lid], &grazed, 0, 16.0), "either way round");
        assert!(
            !off([&lid, &side], &grazed, 1, 1.0),
            "the line is square to y"
        );
        assert!(!off([&lid, &side], &grazed, 0, f64::INFINITY));

        let floor = Plane::from_coefficients([0.0, 1.0, 0.0, 0.0]).expect("a plane");
        let wall = Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a plane");
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        assert!(off([&floor, &wall], &ball, 0, 4.0), "inside at 4");
        assert!(!off([&floor, &wall], &ball, 0, 5.0), "on it at 5");
        assert!(off([&floor, &wall], &ball, 0, 6.0), "outside at 6");
        assert!(
            !off([&floor, &wall], &Surface::Plane(lid), 0, 4.0),
            "a plane is not a quadric"
        );
    }

    /// The ball sits off the bar's axis, so the floor cuts the two in circles
    /// about two different centres, `x² + y² = 9` and `(x − 4)² + y² = 25`.
    /// Those differ by the line `x = 0`, which meets the first at `(0, ±3, 0)`,
    /// each an `f64`. About one centre they would be a circle of radius five
    /// and one of three, and [`what_a_plane_square_to_their_one_axis_leaves`]
    /// answers that.
    #[test]
    fn a_floor_across_a_bar_and_a_ball_off_its_axis_meets_them_where_their_circles_cross() {
        let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
        let ball = Surface::Sphere(Sphere::new(Point::new(4.0, 0.0, 0.0), 5.0).expect("a radius"));
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        let answer = meet([&floor, &ball, &pipe]).expect("two circles in one plane are answered");
        on_all_three([&floor, &ball, &pipe], &answer);
        assert_eq!(
            answer,
            MeetOfThree::At(smallvec![
                PredicatePoint::Exact([0.0, -3.0, 0.0]),
                PredicatePoint::Exact([0.0, 3.0, 0.0])
            ])
        );
        assert_eq!(meet([&pipe, &floor, &ball]), Ok(answer), "in any order");
    }

    /// The bar of radius 2 up `z` and the ball of radius 3 at `(0, 3, 5)`, and
    /// the bar's cap `z = 5`. The cap cuts them in `x² + y² = 4` and
    /// `x² + (y − 3)² = 9`, which differ by the line `y = 2/3`, so the three
    /// meet at `(±4√2/3, 2/3, 5)`, neither of them an `f64`.
    #[test]
    fn a_bars_cap_across_a_ball_off_its_axis_meets_them_where_their_circles_cross() {
        let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, 5.0]);
        let bar = a_bar_up_z_of_radius(2.0);
        let ball = the_ball_of_radius_at_0_3_5(3.0);
        let answer = meet([&cap, &bar, &ball]).expect("two circles in one plane are answered");
        on_all_three([&cap, &bar, &ball], &answer);
        let across = 4.0 * 2f64.sqrt() / 3.0;
        narrow_boxes_round(
            &answer,
            &[[-across, 2.0 / 3.0, 5.0], [across, 2.0 / 3.0, 5.0]],
        );
        assert_eq!(meet([&ball, &bar, &cap]), Ok(answer), "in any order");
    }

    /// The wall `x = 0` holds the bar's axis, so it cuts the bar in the lines
    /// `y = ±2`. `y = 2` meets the ball where `1 + (z − 5)² = 9`, at
    /// `(0, 2, 5 ± 2√2)`, the lower first, and `y = −2` misses it.
    #[test]
    fn a_wall_through_a_bars_axis_meets_a_ball_off_it_on_the_one_line_that_reaches_it() {
        let wall = the_wall_through_the_axis();
        let bar = a_bar_up_z_of_radius(2.0);
        let ball = the_ball_of_radius_at_0_3_5(3.0);
        let answer = meet([&wall, &bar, &ball]).expect("a wall along a bar is answered");
        on_all_three([&wall, &bar, &ball], &answer);
        let up = 2.0 * 2f64.sqrt();
        narrow_boxes_round(&answer, &[[0.0, 2.0, 5.0 - up], [0.0, 2.0, 5.0 + up]]);
        assert_eq!(meet([&ball, &wall, &bar]), Ok(answer), "in any order");
    }

    /// The ball of radius 1 at `(0, 3, 5)` only touches the line `y = 2` the
    /// wall cuts the bar in, so the three meet at `(0, 2, 5)` alone.
    #[test]
    fn a_wall_through_a_bars_axis_meets_a_ball_that_touches_one_of_its_lines_once() {
        let wall = the_wall_through_the_axis();
        let bar = a_bar_up_z_of_radius(2.0);
        let ball = the_ball_of_radius_at_0_3_5(1.0);
        let answer = meet([&wall, &bar, &ball]).expect("a wall along a bar is answered");
        on_all_three([&wall, &bar, &ball], &answer);
        assert_eq!(
            answer,
            MeetOfThree::At(smallvec![PredicatePoint::Exact([0.0, 2.0, 5.0])])
        );
        assert_eq!(meet([&bar, &ball, &wall]), Ok(answer), "in any order");
    }

    /// The wall `y = 0` holds the axes of a bar of radius 4 up `z` and of one
    /// of radius 1 along `x` at `z = 5`, so it cuts them in `x = ±4` and in
    /// `z = 4` and `z = 6`, which cross at four points of the lattice.
    #[test]
    fn a_wall_through_the_axes_of_two_bars_that_cross_meets_them_where_their_lines_cross() {
        let wall = flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        let upright = a_bar_up_z_of_radius(4.0);
        let across = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(0.0, 0.0, 5.0), Vector::new(1.0, 0.0, 0.0))
                    .expect("a direction"),
                1.0,
            )
            .expect("a radius"),
        );
        let answer = meet([&wall, &upright, &across]).expect("a wall along a bar is answered");
        on_all_three([&wall, &upright, &across], &answer);
        assert_eq!(
            answer,
            MeetOfThree::At(smallvec![
                PredicatePoint::Exact([-4.0, 0.0, 4.0]),
                PredicatePoint::Exact([-4.0, 0.0, 6.0]),
                PredicatePoint::Exact([4.0, 0.0, 4.0]),
                PredicatePoint::Exact([4.0, 0.0, 6.0])
            ])
        );
        assert_eq!(meet([&across, &wall, &upright]), Ok(answer), "in any order");
    }

    /// A cone bends against the pipe's own direction, so `w·Mw` is below zero
    /// there and the quadratic along each line opens downwards. The wall
    /// `x = 0` cuts the bar of radius 1 up `z` through `(0, 3, 0)` in `y = 2`
    /// and `y = 4`, and the cone of slope 1 at the origin in `y = ±z`, so the
    /// three meet at `(0, 2, ±2)` and `(0, 4, ±4)`, the lower `z` first on
    /// each line all the same.
    #[test]
    fn a_wall_through_a_bars_axis_meets_a_cone_beside_it_lower_point_first_on_each_line() {
        let wall = the_wall_through_the_axis();
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let bar = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(0.0, 3.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                    .expect("a direction"),
                1.0,
            )
            .expect("a radius"),
        );
        let answer = meet([&wall, &cone, &bar]).expect("a wall along a bar is answered");
        on_all_three([&wall, &cone, &bar], &answer);
        assert_eq!(
            answer,
            MeetOfThree::At(smallvec![
                PredicatePoint::Exact([0.0, 2.0, -2.0]),
                PredicatePoint::Exact([0.0, 2.0, 2.0]),
                PredicatePoint::Exact([0.0, 4.0, -4.0]),
                PredicatePoint::Exact([0.0, 4.0, 4.0])
            ])
        );
        assert_eq!(meet([&bar, &cone, &wall]), Ok(answer), "in any order");
    }

    /// A plane that leans to the bar's axis cuts it in an ellipse, and the
    /// ball in a circle: through `(0, 0, 5)` with normal `(0, 0.6, 0.8)`, one
    /// of radius `√(9 − 1.8²) = 2.4` about `(0, 1.92, 3.56)`, which runs
    /// through `(0, 0, 5)` on the bar's axis and out to `(0, 3.84, 2.12)` past
    /// the bar, so it crosses the ellipse. An ellipse and a circle meet where a
    /// quartic is zero, and this was refused by name until Q2196.
    ///
    /// The wall `x = 0` holds the bar's axis, the plane's normal and the
    /// ball's centre, so it is a mirror of all three, and the mirror road
    /// answers it. On the plane `z = 5 − 0.75y`, and the bar and the ball
    /// differ by `0.5625y² − 6y + 4`, nothing at `y = 16/3 − 8√3/3`, about
    /// `0.7145`, and at about `9.95`, past the bar. So the three meet at
    /// `(±√(4 − y²), y, 5 − 0.75y)` for the first, the `−(n × w) = (−0.6, 0,
    /// 0)` end first, each a rounding off it.
    #[test]
    fn a_plane_leaning_to_a_bars_axis_through_a_ball_off_it_meets_both_at_two_corners() {
        let leaning = flat([0.0, 0.6, 0.8], [0.0, 0.0, 5.0]);
        let bar = a_bar_up_z_of_radius(2.0);
        let ball = the_ball_of_radius_at_0_3_5(3.0);
        let answer = meet([&leaning, &bar, &ball]).expect("one wall mirrors all three");
        on_all_three([&leaning, &bar, &ball], &answer);
        let y = (16.0 - 8.0 * 3.0_f64.sqrt()) / 3.0;
        let x = (4.0 - y * y).sqrt();
        let z = 5.0 - 0.75 * y;
        narrow_boxes_near(&answer, &[[-x, y, z], [x, y, z]]);
        assert_eq!(meet([&ball, &leaning, &bar]), Ok(answer), "in any order");
    }

    /// `-0.0 == 0.0`, so no pin that compares corners sees which zero a corner
    /// carries, but a corner keyed on its bits does: `(-0.0, 3, 0)` and
    /// `(0, 3, 0)` are one point and two keys. The floor across a bar of
    /// radius 3 up `z` and a ball of radius 5 at `(4, 0, 0)` came back at
    /// `(-0.0, ±3, 0)` before the road added zero to what it hands back. With
    /// the ball at `(0, 4, 0)` the circles differ by `y = 0` instead, and the
    /// corners are `(±3, 0, 0)`.
    ///
    /// Only exact corners are asked about, since a box's lower end at zero is
    /// `-0.0` whatever the box was built from, as the last line shows.
    #[test]
    fn no_corner_where_a_plane_cuts_two_curved_surfaces_in_circles_is_at_a_negative_zero() {
        let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
        let pipe = a_bar_up_z_of_radius(3.0);
        let ball_at = |centre: Point| Surface::Sphere(Sphere::new(centre, 5.0).expect("a radius"));
        let beside_on_x = ball_at(Point::new(4.0, 0.0, 0.0));
        let beside_on_y = ball_at(Point::new(0.0, 4.0, 0.0));
        let negative_zero = (-0.0f64).to_bits();
        for ball in [&beside_on_x, &beside_on_y] {
            for order in [
                [&floor, ball, &pipe],
                [&floor, &pipe, ball],
                [ball, &floor, &pipe],
                [ball, &pipe, &floor],
                [&pipe, &floor, ball],
                [&pipe, ball, &floor],
            ] {
                let answer = meet(order).expect("two circles in one plane are answered");
                let MeetOfThree::At(points) = &answer else {
                    panic!("expected points, got {answer:?}");
                };
                assert_eq!(points.len(), 2, "{answer:?}");
                for point in points {
                    let at = point
                        .exact_coordinates()
                        .unwrap_or_else(|| panic!("{point:?} is on the lattice"));
                    assert!(
                        at.iter().all(|value| value.to_bits() != negative_zero),
                        "{point:?} from {order:?} has a negative zero in it"
                    );
                }
            }
        }
        assert_eq!(Interval::exactly(0.0).lo().to_bits(), negative_zero);
    }

    fn a_bar_up_z_of_radius(radius: f64) -> Surface {
        Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a radius"))
    }

    fn the_ball_of_radius_at_0_3_5(radius: f64) -> Surface {
        Surface::Sphere(Sphere::new(Point::new(0.0, 3.0, 5.0), radius).expect("a radius"))
    }

    /// Each point of `answer` is a box no wider than `1e-14` round the point in
    /// its own place in `wanted`.
    fn narrow_boxes_round(answer: &MeetOfThree, wanted: &[[f64; 3]]) {
        let MeetOfThree::At(points) = answer else {
            panic!("expected points, got {answer:?}");
        };
        assert_eq!(points.len(), wanted.len(), "{answer:?}");
        for (point, wanted) in points.iter().zip(wanted) {
            let boxed = point.enclosure();
            assert!(
                (0..3).all(|axis| boxed[axis].lo() <= wanted[axis]
                    && wanted[axis] <= boxed[axis].hi()
                    && boxed[axis].width() < 1e-14),
                "{point:?} is not a narrow box round {wanted:?}"
            );
        }
    }

    fn a_cone(apex: [f64; 3], up: [f64; 3], slope: f64) -> Surface {
        Surface::Cone(
            Cone::new(
                Axis::new(
                    Point::new(apex[0], apex[1], apex[2]),
                    Vector::new(up[0], up[1], up[2]),
                )
                .expect("a direction"),
                slope,
            )
            .expect("a slope"),
        )
    }

    /// The wall `x = 0`, facing `−x` the way the block that halves a frustum
    /// in `rcad-topo`'s `a_cone_hollowed_along_its_axis.rs` stores it.
    fn the_wall_through_the_axis() -> Surface {
        flat([-1.0, 0.0, 0.0], [0.0, 0.0, 0.0])
    }

    /// Every point of `answer` is on each of `surfaces`, or in a box that
    /// straddles it.
    fn on_all_three(surfaces: [&Surface; 3], answer: &MeetOfThree) {
        let MeetOfThree::At(points) = answer else {
            panic!("expected points, got {answer:?}");
        };
        for point in points {
            for surface in surfaces {
                assert!(
                    !matches!(
                        which_side_of_it(surface, *point),
                        Decision::Pos | Decision::Neg
                    ),
                    "{point:?} is off {surface:?}"
                );
            }
        }
    }

    /// The wall `x = 0` holds the axis of a cone of slope 1 at the origin and
    /// of one of slope 2 whose apex is two up. The two cross where
    /// `t = 2|t − 2|`, at `t = 4/3` and `t = 4`, in circles of those radii, so
    /// the wall meets the three at `(0, ±4/3, 4/3)` and `(0, ±4, 4)`, the lower
    /// circle first and `−y` first on each, since `n × w` is `+y`. The upper two
    /// are on the lattice and come back as themselves.
    #[test]
    fn a_wall_through_the_axis_meets_a_cone_and_a_steeper_one_on_both_their_circles() {
        let wall = the_wall_through_the_axis();
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let steeper = a_cone([0.0, 0.0, 2.0], [0.0, 0.0, 1.0], 2.0);
        let answer = meet([&wall, &cone, &steeper]).expect("a wall through the axis is answered");
        on_all_three([&wall, &cone, &steeper], &answer);
        let MeetOfThree::At(points) = &answer else {
            panic!("expected points, got {answer:?}");
        };
        assert_eq!(points.len(), 4, "{answer:?}");
        let third = 4.0 / 3.0;
        for (point, wanted) in points[..2]
            .iter()
            .zip([[0.0, -third, third], [0.0, third, third]])
        {
            let boxed = point.enclosure();
            assert!(
                (0..3).all(|axis| boxed[axis].lo() <= wanted[axis]
                    && wanted[axis] <= boxed[axis].hi()
                    && boxed[axis].width() < 1e-14),
                "{point:?} is not a narrow box round {wanted:?}"
            );
        }
        assert_eq!(
            points[2..],
            [
                PredicatePoint::Exact([0.0, -4.0, 4.0]),
                PredicatePoint::Exact([0.0, 4.0, 4.0])
            ]
        );
        assert_eq!(
            meet([&steeper, &wall, &cone]),
            Ok(answer),
            "the answer depends on which three they are, not the order they are asked in"
        );
    }

    /// A bar of radius 4 about the cone's own axis crosses it at `t = ±4`, and
    /// the wall meets the three at the four points `(0, ±4, ±4)`, each an
    /// `f64`. Turning the cone's and the bar's stored directions round to `−z`
    /// is the same three surfaces, and gives the same four in the same order.
    #[test]
    fn a_wall_through_the_axis_meets_a_cone_and_a_bar_round_it_at_four_points_on_the_lattice() {
        let wall = the_wall_through_the_axis();
        let wanted = MeetOfThree::At(smallvec![
            PredicatePoint::Exact([0.0, -4.0, -4.0]),
            PredicatePoint::Exact([0.0, 4.0, -4.0]),
            PredicatePoint::Exact([0.0, -4.0, 4.0]),
            PredicatePoint::Exact([0.0, 4.0, 4.0]),
        ]);
        for up in [[0.0, 0.0, 1.0], [0.0, 0.0, -1.0]] {
            let cone = a_cone([0.0, 0.0, 0.0], up, 1.0);
            let bar = Surface::Cylinder(
                Cylinder::new(
                    Axis::new(Point::origin(), Vector::new(up[0], up[1], up[2]))
                        .expect("a direction"),
                    4.0,
                )
                .expect("a radius"),
            );
            assert_eq!(meet([&wall, &cone, &bar]), Ok(wanted.clone()), "up {up:?}");
        }
    }

    /// A ball of radius 3 four up the cone's axis crosses it where
    /// `2t² − 8t + 7 = 0`, at `t = 2 ± √2/2`, which no `f64` holds. The wall
    /// meets the three at `(0, ±t, t)` for each, in boxes a few roundings wide.
    #[test]
    fn a_wall_through_the_axis_meets_a_cone_and_a_ball_on_it_in_four_narrow_boxes() {
        let wall = the_wall_through_the_axis();
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 3.0).expect("a radius"));
        let answer = meet([&wall, &cone, &ball]).expect("a wall through the axis is answered");
        on_all_three([&wall, &cone, &ball], &answer);
        let MeetOfThree::At(points) = &answer else {
            panic!("expected points, got {answer:?}");
        };
        let [lower, upper] = [-1.0, 1.0].map(|side| 2.0 + side * 2.0_f64.sqrt() / 2.0);
        let wanted = [
            [0.0, -lower, lower],
            [0.0, lower, lower],
            [0.0, -upper, upper],
            [0.0, upper, upper],
        ];
        assert_eq!(points.len(), 4, "{answer:?}");
        for (point, wanted) in points.iter().zip(wanted) {
            let boxed = point.enclosure();
            assert!(
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-14 && boxed[axis].width() < 1e-14
                }),
                "{point:?} is not a narrow box round {wanted:?}"
            );
        }
    }

    /// Two cones of one apex and two slopes share the apex and nothing else,
    /// and so does the wall through their axis with them: the stored apex, one
    /// point and not two.
    #[test]
    fn a_wall_through_the_axis_of_two_cones_of_one_apex_meets_them_at_the_apex_alone() {
        let wall = the_wall_through_the_axis();
        let cone = a_cone([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], 1.0);
        let steeper = a_cone([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], 2.0);
        assert_eq!(
            meet([&wall, &cone, &steeper]),
            Ok(MeetOfThree::At(smallvec![PredicatePoint::Exact([
                0.0, 0.0, 1.0
            ])]))
        );
    }

    /// A ball of radius 5 centred five up the cone's axis runs through its
    /// apex, so `t = 0` is one of the two heights, where `2t² − 10t = 0`. The
    /// wall meets the three at the apex, below, and at `(0, ±5, 5)`.
    #[test]
    fn a_wall_through_the_axis_of_a_cone_and_a_ball_through_its_apex_meets_them_at_the_apex_too() {
        let wall = the_wall_through_the_axis();
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 5.0), 5.0).expect("a radius"));
        assert_eq!(
            meet([&wall, &cone, &ball]),
            Ok(MeetOfThree::At(smallvec![
                PredicatePoint::Exact([0.0, 0.0, 0.0]),
                PredicatePoint::Exact([0.0, -5.0, 5.0]),
                PredicatePoint::Exact([0.0, 5.0, 5.0]),
            ]))
        );
    }

    /// A ball of radius 1 five up inside the cone's nappe never reaches it:
    /// `(1 + s²)R² − s²t_c² = 2 − 25` is below zero, and the three share
    /// nothing.
    #[test]
    fn a_wall_through_the_axis_of_a_cone_and_a_ball_inside_it_meets_nothing() {
        let wall = the_wall_through_the_axis();
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 5.0), 1.0).expect("a radius"));
        assert_eq!(meet([&wall, &cone, &ball]), Ok(MeetOfThree::Nowhere));
    }

    /// The wall `x = 1` runs beside the axis and cuts each circle the cone and
    /// the bar cross in, at `t = ±4`, in a chord: `(1, ±√15, ±4)`, since
    /// `1 + 15 = 16`. The lower circle first and `−y` first on each, since
    /// `n × w` is `+y`.
    #[test]
    fn a_wall_beside_the_axis_of_a_cone_and_a_bar_cuts_both_their_circles_in_a_chord() {
        let wall = flat([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let bar = Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius"));
        let answer = meet([&wall, &cone, &bar]).expect("a wall beside the axis is answered");
        on_all_three([&wall, &cone, &bar], &answer);
        let MeetOfThree::At(points) = &answer else {
            panic!("expected points, got {answer:?}");
        };
        let across = 15.0_f64.sqrt();
        let wanted = [
            [1.0, -across, -4.0],
            [1.0, across, -4.0],
            [1.0, -across, 4.0],
            [1.0, across, 4.0],
        ];
        assert_eq!(points.len(), 4, "{answer:?}");
        for (point, wanted) in points.iter().zip(wanted) {
            let boxed = point.enclosure();
            assert!(
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-14 && boxed[axis].width() < 1e-14
                }),
                "{point:?} is not a narrow box round {wanted:?}"
            );
        }
        assert_eq!(
            meet([&bar, &wall, &cone]),
            Ok(answer),
            "the answer depends on which three they are, not the order they are asked in"
        );
    }

    /// A ball of radius 3 four up the cone's axis crosses it at
    /// `t = 2 ± √2/2`, about `1.29` and `2.71`, in circles of those radii. The
    /// wall `x = 2` cuts the ball and the cone, misses the lower circle and
    /// cuts the upper in `(2, ±√(t² − 4), t)`; the wall `x = 2.75` cuts both
    /// surfaces still and misses both circles. Each is decided at the square
    /// root itself.
    #[test]
    fn a_wall_beside_the_axis_of_a_cone_and_a_ball_misses_one_circle_and_cuts_the_other() {
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 3.0).expect("a radius"));

        let wall = flat([1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        let answer = meet([&wall, &cone, &ball]).expect("a wall beside the axis is answered");
        on_all_three([&wall, &cone, &ball], &answer);
        let MeetOfThree::At(points) = &answer else {
            panic!("expected points, got {answer:?}");
        };
        let height = 2.0 + 2.0_f64.sqrt() / 2.0;
        let across = (height * height - 4.0).sqrt();
        assert_eq!(points.len(), 2, "{answer:?}");
        for (point, wanted) in points
            .iter()
            .zip([[2.0, across, height], [2.0, -across, height]])
        {
            let boxed = point.enclosure();
            assert!(
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-14 && boxed[axis].width() < 1e-13
                }),
                "{point:?} is not a narrow box round {wanted:?}"
            );
        }

        let further = flat([1.0, 0.0, 0.0], [2.75, 0.0, 0.0]);
        assert_eq!(meet([&further, &cone, &ball]), Ok(MeetOfThree::Nowhere));
    }

    /// A ball of radius 2 about a cone's apex crosses the cone of slope one at
    /// `t = ±√2`, in circles of radius `√2`, and the wall `x + y = 2` is `√2`
    /// from the axis: it touches both circles, at `(1, 1, ±√2)`, one point
    /// each. The bar of radius 4 and the cone share circles of radius 4, and
    /// the wall `x = 4` touches them at `(4, 0, ±4)`, which are `f64`.
    #[test]
    fn a_wall_beside_the_axis_that_touches_the_circles_two_surfaces_share_meets_them_once_each() {
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 2.0).expect("a radius"));
        let wall = flat([1.0, 1.0, 0.0], [1.0, 1.0, 0.0]);
        let answer = meet([&wall, &cone, &ball]).expect("a wall beside the axis is answered");
        on_all_three([&wall, &cone, &ball], &answer);
        let MeetOfThree::At(points) = &answer else {
            panic!("expected points, got {answer:?}");
        };
        let height = 2.0_f64.sqrt();
        assert_eq!(points.len(), 2, "{answer:?}");
        for (point, wanted) in points.iter().zip([[1.0, 1.0, -height], [1.0, 1.0, height]]) {
            let boxed = point.enclosure();
            assert!(
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-14 && boxed[axis].width() < 1e-14
                }),
                "{point:?} is not a narrow box round {wanted:?}"
            );
        }

        let bar = Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius"));
        let touching = flat([1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
        assert_eq!(
            meet([&touching, &cone, &bar]),
            Ok(MeetOfThree::At(smallvec![
                PredicatePoint::Exact([4.0, 0.0, -4.0]),
                PredicatePoint::Exact([4.0, 0.0, 4.0]),
            ]))
        );
    }

    /// The wall `x + 2⁻⁵³z = 3`, a rounding off parallel to a bar of radius 2
    /// up `z` and its chamfer cone of slope one with the apex at `(0, 0, −1)`,
    /// and one past the bar. It reaches the bar only from `z = 2⁵³` up, where
    /// the cone is that wide, so the three share nothing. Before the road for a
    /// plane beside a pipe this was refused: the road along a pipe takes only a
    /// wall exactly parallel, and the band road gives the cone's cut no outer
    /// edge.
    #[test]
    fn a_wall_a_rounding_off_parallel_to_a_bar_and_its_chamfer_cone_meets_them_nowhere() {
        let wall = flat([1.0, 0.0, 2f64.powi(-53)], [3.0, 0.0, 0.0]);
        let bar = a_bar_up_z_of_radius(2.0);
        let chamfer = a_cone([0.0, 0.0, -1.0], [0.0, 0.0, 1.0], 1.0);
        for order in [
            [&wall, &bar, &chamfer],
            [&chamfer, &wall, &bar],
            [&bar, &chamfer, &wall],
        ] {
            assert_eq!(meet(order), Ok(MeetOfThree::Nowhere), "{order:?}");
        }
    }

    /// The wall `x + 2⁻⁵³z = 1`, a rounding off parallel to a bar of radius 2
    /// up `z` and its chamfer cone of slope one with the apex at `(0, 0, −1)`,
    /// cuts both circles of radius 2 the two share, at `z = −3` and `z = 1`, in
    /// a chord `√3` either side of `y = 0`: a D-flat on a chamfered shaft. The
    /// lower circle first and `+y` first on each, since `n × w` is `−y`.
    /// Before row Q2177 this was refused: the road along a pipe takes only a
    /// wall exactly parallel, and the road for a plane beside a pipe only one
    /// that misses it.
    #[test]
    fn a_wall_a_rounding_off_parallel_to_a_bar_cuts_both_circles_it_shares_with_its_chamfer() {
        let wall = flat([1.0, 0.0, 2f64.powi(-53)], [1.0, 0.0, 0.0]);
        let bar = a_bar_up_z_of_radius(2.0);
        let chamfer = a_cone([0.0, 0.0, -1.0], [0.0, 0.0, 1.0], 1.0);
        let answer = meet([&wall, &bar, &chamfer]).expect("a wall along the bar is answered");
        on_all_three([&wall, &bar, &chamfer], &answer);
        let across = 3.0_f64.sqrt();
        narrow_boxes_near(
            &answer,
            &[
                [1.0, across, -3.0],
                [1.0, -across, -3.0],
                [1.0, across, 1.0],
                [1.0, -across, 1.0],
            ],
        );
        for order in [[&chamfer, &wall, &bar], [&bar, &chamfer, &wall]] {
            assert_eq!(
                meet(order).as_ref(),
                Ok(&answer),
                "the answer depends on which three they are, not the order they are asked in"
            );
        }
    }

    /// The plane `x + z = 0` leans to the axis of a cone of slope 2 at the
    /// origin and of a bar of radius 4 about it, and holds the cone's apex but
    /// not the axis. The two share circles of radius 4 at `z = ±2`, and the
    /// plane cuts each in a chord `√12` either side of `y = 0`, at `x = ∓2`. The
    /// plane `x + z = 7` misses both. The lower circle first and `+y` first on
    /// each, since `n × w` is `−y`.
    #[test]
    fn a_plane_leaning_through_a_cones_apex_cuts_both_circles_it_shares_with_a_bar() {
        let leaning = flat([1.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
        let cone = a_cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
        let bar = a_bar_up_z_of_radius(4.0);
        let answer = meet([&leaning, &cone, &bar]).expect("a leaning plane is answered");
        on_all_three([&leaning, &cone, &bar], &answer);
        let across = 12.0_f64.sqrt();
        narrow_boxes_near(
            &answer,
            &[
                [2.0, across, -2.0],
                [2.0, -across, -2.0],
                [-2.0, across, 2.0],
                [-2.0, -across, 2.0],
            ],
        );

        let further = flat([1.0, 0.0, 1.0], [7.0, 0.0, 0.0]);
        assert_eq!(meet([&further, &cone, &bar]), Ok(MeetOfThree::Nowhere));
    }

    /// Each point of `answer` is a box no wider than `1e-13` whose middle is
    /// within `1e-14` of the point in its own place in `wanted`, for a corner
    /// that is a rounding off the `f64` named.
    fn narrow_boxes_near(answer: &MeetOfThree, wanted: &[[f64; 3]]) {
        let MeetOfThree::At(points) = answer else {
            panic!("expected points, got {answer:?}");
        };
        assert_eq!(points.len(), wanted.len(), "{answer:?}");
        for (point, wanted) in points.iter().zip(wanted) {
            let boxed = point.enclosure();
            assert!(
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-14 && boxed[axis].width() < 1e-13
                }),
                "{point:?} is not a narrow box near {wanted:?}"
            );
        }
    }

    /// The sign of `P + L·√Δ` for every way the two parts can point, a tie
    /// among them, against the same sum worked out in `f64`, which holds every
    /// one of these exactly.
    #[test]
    fn a_sum_with_a_square_root_in_it_takes_the_sign_of_the_larger_part() {
        for (plain, rooted, under) in [
            (3.0, 1.0, 4.0),
            (-3.0, -1.0, 4.0),
            (3.0, -1.0, 4.0),
            (1.0, -1.0, 4.0),
            (-3.0, 1.0, 4.0),
            (-1.0, 1.0, 4.0),
            (2.0, -1.0, 4.0),
            (0.0, -1.0, 4.0),
            (5.0, 0.0, 4.0),
            (-5.0, 3.0, 0.0),
        ] {
            let sum = f64::mul_add(rooted, f64::sqrt(under), plain);
            let wanted = if sum > 0.0 {
                Decision::Pos
            } else if sum < 0.0 {
                Decision::Neg
            } else {
                Decision::Zero
            };
            assert_eq!(
                the_sign_with_a_root_in_it(
                    &Exact::of(plain),
                    &Exact::of(rooted),
                    &Exact::of(under)
                ),
                Some(wanted),
                "{plain} + {rooted}·√{under}"
            );
        }
    }

    /// What the degree of the discriminant costs in `f64`, measured.
    ///
    /// §31.2 row 4 guesses "about 14" and §34 records what it is: **ten** for
    /// two planes and a ball, **fourteen** for two planes and a pipe. A degree
    /// is a fact about the expression and is derived term by term where each
    /// half is built; what a degree does not say is how many `f64` the exact
    /// value takes to hold, and that is the number a caller pays.
    ///
    /// Measured here so §29's question — how wide an expansion this arithmetic
    /// asks `Exact` to carry — has an answer for the row above the ones §29
    /// measured. The ceiling is a **watch and not a bound**: nothing derives it,
    /// and what it is there to catch is the discriminant's expansion growing by
    /// an order of magnitude without anybody noticing.
    #[test]
    fn how_many_f64_the_discriminant_of_a_line_and_a_quadric_takes_to_hold() {
        let turn = crate::Isometry::rotation(
            &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction"),
            0.4,
        )
        .expect("a finite angle");
        let ball = Surface::Sphere(Sphere::new(Point::new(1.0, 2.0, 0.0), 5.0).expect("a radius"));
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(0.0, 1.0, 0.0), Vector::new(1.0, 3.0, 5.0))
                    .expect("a direction"),
                3.0,
            )
            .expect("a radius"),
        );
        let one = flat([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let other = flat([0.0, 1.0, 3.0], [0.0, 1.0, 0.0]);

        let mut widest = 0usize;
        for turned in [false, true] {
            let placed = |surface: &Surface| {
                if turned {
                    surface.transformed(&turn)
                } else {
                    *surface
                }
            };
            let (Surface::Plane(first), Surface::Plane(second)) = (placed(&one), placed(&other))
            else {
                unreachable!("both of these are flat");
            };
            let line = the_line_two_planes_cross_in(
                &first,
                &second,
                &[PredicatePoint::from([0.0; 3]); 2],
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("these two planes are not parallel")
            .expect("and they cross");

            for (what, quadric) in [("a ball", placed(&ball)), ("a pipe", placed(&pipe))] {
                let quadratic =
                    how_a_quadric_cuts_the_line(&line, &quadric).expect("all of this fits");
                // Two planes hand over stored `f64`, so every one of these is
                // exact and the `expect`s below are the measurement being of
                // the thing it says it is measuring.
                let exactly = |term: &Term| term.exactly().expect("two planes give exact numbers");
                let discriminant = exactly(
                    &quadratic
                        .difference_of_the_two_halves()
                        .expect("all of this fits"),
                );
                widest = widest.max(discriminant.entries());
                println!(
                    "{what}{}: A {} entries, B {}, C {}, B²−AC {} entries spanning {} binary orders",
                    if turned {
                        ", turned"
                    } else {
                        ", on the lattice"
                    },
                    exactly(&quadratic.squared).entries(),
                    exactly(&quadratic.half_linear).entries(),
                    exactly(&quadratic.constant).entries(),
                    discriminant.entries(),
                    discriminant.spread(),
                );
            }
        }
        assert!(
            widest < MOST_F64_A_DISCRIMINANT_HAS_BEEN_SEEN_TO_NEED,
            "the discriminant took {widest} entries to hold"
        );
    }

    /// A ceiling on how many `f64` the discriminant of a line and a quadric has
    /// been seen to need.
    ///
    /// **A watch and not a published bound.** `Exact::times` grows a product to
    /// at most twice the two lengths multiplied, which for a degree-fourteen
    /// discriminant is a number with five zeroes on it; what keeps the real
    /// figure two orders below that is `expansion_sum` dropping every roundoff
    /// that comes out exactly zero. Nothing derives the real figure, so this is
    /// set an order of magnitude above what is measured and its job is to notice
    /// a change of shape rather than to certify one.
    ///
    /// Measured on the four rows above: five entries for a ball on the lattice,
    /// nineteen for a pipe on the lattice, thirty-six and fifty-seven turned.
    /// The turned pipe's discriminant spans **761 binary orders**, which is
    /// inside the 995 [`Exact::square_minus`]'s window leaves and not by much —
    /// so the row above this one is the row that runs out first if the degree
    /// ever goes up again.
    const MOST_F64_A_DISCRIMINANT_HAS_BEEN_SEEN_TO_NEED: usize = 512;
}
