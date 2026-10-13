//! How often a certified fuse with a cone in it gives up, family by family.
//!
//! §17.7's discipline, unchanged: the rates are **never blended**, every family
//! prints its own number with its own denominator, and every refusal is named
//! rather than counted. What is different here is that a cone is the first
//! surface this kernel holds whose crossings it mostly cannot name, so the
//! headline is expected to be bad — and a bad rate reported is worth more than a
//! good rate got by only drawing the shapes that work.
//!
//! # What decides every number below
//!
//! `intersect` names a plane against a cone in **all seven** of the shapes they
//! can meet in — its own table says so, and this file's
//! `a_block_wall_beside_a_cone_still_cuts_a_hyperbola_that_never_closes` puts a
//! block's wall through it and gets a curve back. Naming the crossing is not
//! where a cone's fusibility is decided, and neither is **whether the named
//! curve closes** any more:
//!
//! - a plane **square to the cone's axis** cuts a circle, which closes;
//! - a plane leaning less than the cone's own side cuts an ellipse, which
//!   closes;
//! - a plane **parallel to the axis**, which is four of a block's six faces when
//!   the block stands upright, cuts a **hyperbola**. `intersect` names it and
//!   hands back one endless run per nappe. That branch still never closes, and
//!   `arrange`'s `HowItRuns::AlongABend` orders points along it anyway, and
//!   since e2cd175d the fuse names each arc of it by the nappe it is on. Since
//!   §340 every one of the block family's fuses answers, which
//!   `every_fuse_of_a_block_and_a_cone_now_answers` pins;
//! - anything curved against a cone `intersect` does still refuse outright, and
//!   **one cone twice is not that pair**: `intersect` has always answered it as
//!   one surface, and since §214 `walk` does too. For a cone against a *different*
//!   cone off its axis the refusal is the degree-four space curve, and on its
//!   axis it is circles or the apex alone, which `intersect` names since Q1122;
//!   for a cone coaxial with a
//!   pipe it is two circles, which `intersect` names now, and where a cap plane
//!   square to their axis meets both, which `three_surfaces.rs` answers now too
//!   (Q981). The arrangement draws that pair since Q987, and since Q1118 the
//!   assembly joins its pieces, so a frustum against a bar of its narrow
//!   radius answers.
//!
//! The families are chosen to measure each of those rather than to average them
//! away.
//!
//! # **Every plane of the tool is asked, and that is a decision and not an
//! oversight**
//!
//! There is no face-level box test in the fuse, on purpose: `fuse/cuts.rs`'s own
//! note argues that skipping a face pair whose boxes miss *changes an answer*,
//! because a plane reaches past the face carrying it and two neighbours split at
//! the same point of the edge between them or the answer is not a manifold.
//! §17.4 threw the BVH out for the same reason arriving from the other side.
//!
//! So **a tool solid cannot hide its awkward planes by keeping them out of the
//! way**. A block standing upright beside a cone still has four planes parallel
//! to the cone's axis, every one of them is put to `intersect`, and every one of
//! them comes back a hyperbola.
//!
//! That used to be the whole of what a cone cost a fuse: a cone could be cut
//! only by a solid whose every plane leaned more steeply than the cone's side, or
//! not at all. **It is not any more.** Since e2cd175d named each hyperbola arc
//! the fuse writes by the nappe it is on, seven rows of the block family whose
//! block really cuts the frustum answer all six of their fuses, and
//! `what_the_answered_block_fuses_actually_say.rs` checks each of those answers
//! against the inputs' closed forms.
//!
//! A cube stood on its corner is the tidiest solid that does: its three edge
//! directions are then equally inclined to the vertical, so all six of its
//! planes lean at the same angle, and `a_cube_on_its_corner_leans_every_one_of
//! _its_planes_across_a_shallow_cone` measures that angle rather than assuming
//! it.

use std::collections::BTreeMap;

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// How many pairs each family draws.
const DRAWS: usize = 16;

/// A deterministic sequence, so that a rate is one number and not a range.
#[derive(Clone)]
struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    /// A whole number between `low` and `high`, both ends included.
    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }
}

/// The families, and what each is a measurement of.
#[derive(Clone, Copy)]
enum Family {
    /// A frustum against a cube stood on its corner, so that every one of the
    /// cube's six planes leans more steeply than the cone's own side and cuts
    /// it in an ellipse. **The control that says a cone can be fused at all.**
    AFrustumAgainstACornerUpCube,
    /// The same over a cone that ends in a point, so the answer carries the
    /// apex.
    APointedConeAgainstACornerUpCube,
    /// A frustum against a block that overlaps it sideways. Four of the block's
    /// faces run parallel to the cone's axis and cut it in hyperbolas.
    AFrustumAgainstABlock,
    /// A frustum against a round bar. The bar is only ever slid along `z` and
    /// the cone stands on `Axis::z()`, so this is a **cone coaxial with a
    /// pipe**, which crosses in exactly two circles and not in a degree-four
    /// curve (§50.11).
    AFrustumAgainstABar,
    /// Two coaxial frustums of different slopes and one apex: cone against
    /// cone, meeting at the apex alone.
    TwoFrustumsOnOneAxis,
}

impl Family {
    fn named(self) -> &'static str {
        match self {
            Family::AFrustumAgainstACornerUpCube => "a frustum against a corner-up cube",
            Family::APointedConeAgainstACornerUpCube => "a pointed cone against a corner-up cube",
            Family::AFrustumAgainstABlock => "a frustum against a block",
            Family::AFrustumAgainstABar => "a frustum against a bar",
            Family::TwoFrustumsOnOneAxis => "two frustums on one axis",
        }
    }
}

/// The turn that stands a cube on its corner: the diagonal `(1, 1, 1)` brought
/// upright.
///
/// Every one of the cube's six planes then leans at the same angle to the
/// vertical, which is the most a cube's planes can be made to lean *all at once*
/// — the three edge directions are orthonormal, so their vertical components
/// square to one and the smallest of them is never above `1/√3`.
fn stood_on_its_corner() -> Isometry {
    let axis = rcad_geom::Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0))
        .expect("a direction with length");
    Isometry::rotation(&axis, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn")
}

/// A box of the given lengths with its low corner at `at`.
fn a_block(
    model: &mut Model,
    [along_x, along_y, along_z]: [f64; 3],
    at: Vector,
) -> Option<SolidKey> {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .ok()?;
    model
        .add_extrusion(&rectangle, along_z, &Isometry::translation(at).ok()?)
        .ok()
}

fn a_pair(family: Family, draw: &mut Draw) -> Option<(Model, SolidKey, SolidKey)> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [slope, near, far] = a_cones_sizes(draw)?;

    Some(match family {
        Family::AFrustumAgainstACornerUpCube | Family::APointedConeAgainstACornerUpCube => {
            let cone = if matches!(family, Family::AFrustumAgainstACornerUpCube) {
                a_frustum(&mut model, slope, near, far)
            } else {
                a_pointed_cone(&mut model, slope, far)
            };
            let side = draw.whole(4, 9);
            let cube = a_block(&mut model, [side; 3], Vector::new(0.0, 0.0, 0.0))?;
            let up = draw.whole(1, far as i64 - 1);
            model
                .place_solid(
                    cube,
                    &Isometry::translation(Vector::new(0.0, 0.0, up))
                        .ok()?
                        .compose(&stood_on_its_corner()),
                )
                .ok()?;
            (model, cone, cube)
        }
        Family::AFrustumAgainstABlock => {
            let cone = a_frustum(&mut model, slope, near, far);
            let side = slope * far;
            // Clear of the apex on purpose: a block whose walls passed through
            // it would cut the cone in *two lines through the apex* and measure
            // that row instead of the hyperbola row, which is what this family
            // is for.
            let block = a_block(
                &mut model,
                [side, side, far],
                Vector::new(1.0, 1.0, near + 1.0),
            )?;
            (model, cone, block)
        }
        Family::AFrustumAgainstABar => {
            let cone = a_frustum(&mut model, slope, near, far);
            let bar = model
                .add_round_bar(
                    slope * near,
                    2.0 * far,
                    &Isometry::translation(Vector::new(0.0, 0.0, -far)).ok()?,
                )
                .ok()?;
            (model, cone, bar)
        }
        Family::TwoFrustumsOnOneAxis => {
            let cone = a_frustum(&mut model, slope, near, far);
            let other = a_frustum(&mut model, slope * 2.0, near, far);
            (model, cone, other)
        }
    })
}

/// A cone's slope and its near and far heights, `[slope, near, far]`, the first
/// thing every family draws.
///
/// A power-of-two slope, so every corner of the cone sits exactly on it, and
/// never above `1/√2` so that a cube stood on its corner leans every one of its
/// planes across it — see the note at the top of the file.
fn a_cones_sizes(draw: &mut Draw) -> Option<[f64; 3]> {
    let slope = 2.0_f64.powi(i32::try_from(draw.next() % 3).ok()? - 3);
    let far = draw.whole(6, 12);
    let near = draw.whole(1, 4);
    Some([slope, near, far])
}

/// Every rule, both ways round, over one family.
fn how_each_fuse_went(family: Family) -> (usize, usize, BTreeMap<String, usize>) {
    let policy = Policy::certified();
    let mut draw = Draw(0x5EED);
    let mut fuses = 0;
    let mut answered = 0;
    let mut gave_up: BTreeMap<String, usize> = BTreeMap::new();
    for _ in 0..DRAWS {
        let Some((mut model, cone, tool)) = a_pair(family, &mut draw) else {
            continue;
        };
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for (arguments, tools) in [([cone], [tool]), ([tool], [cone])] {
                fuses += 1;
                let mut log = ProvenanceLog::new();
                match model.fuse(&arguments, &tools, rule, &policy, &mut log) {
                    Ok(_) => {
                        answered += 1;
                        // A fuse hands back a sound model or it refuses. This is
                        // the half that makes the rate mean anything: a family
                        // that answered by writing a broken solid would look
                        // like a good rate.
                        model
                            .validate(&policy, &mut ProvenanceLog::new())
                            .expect("a fuse hands back a sound model or refuses");
                    }
                    Err(Error::Undecidable { records }) => {
                        let named = records
                            .records()
                            .last()
                            .map_or("nothing named", |record| record.predicate());
                        *gave_up.entry(named.to_owned()).or_default() += 1;
                    }
                    Err(why) => *gave_up.entry(format!("{why}")).or_default() += 1,
                }
            }
        }
    }
    (fuses, answered, gave_up)
}

/// The measurement, printed family by family and never blended.
///
/// # What is asserted and what is only reported
///
/// The two curved-tool families have **one assertion apiece**: the bar, since
/// Q1118, and the second frustum, since Q1122, are both asserted at everything
/// answered.
///
/// `AFrustumAgainstABar` is a cone coaxial with a pipe. That pair crosses in
/// exactly two circles, so the old shared reason — a degree-four space curve —
/// was never true of it (§50.11). **The pin was written to break the day
/// `intersect` named the pair. `intersect` names it now, and the pin did not
/// break**, because the fuse got one step further and stopped at a different
/// wall: where three surfaces meet was only worked out when two of them were
/// flat, and a cone, a pipe and a cap plane give one flat surface out of three.
/// All 96 refusals said so.
///
/// **`three_surfaces.rs` has learnt that case, and the rate did not move: 0 of
/// 96 again.** A plane square to the axis a cone and a pipe share cuts each in a
/// circle about one centre, so the three meet in a whole circle or nowhere, and
/// `what_a_plane_square_to_their_one_axis_leaves` tells which from the squared
/// radii (Q981). Every refusal moved on past that meet, and they no longer
/// shared one wall: 48 said *nothing can put points in order along this curve*
/// and 48 said *two curves on one face touch without crossing*.
///
/// **Both were the frustum's narrow rim, which is one circle three surfaces
/// cut, and Q987 traced and drew it: 0 of 96 again.** The bar has the narrow
/// cap's radius, so the cap's plane, the cone and the pipe all pass through
/// the rim. Traced on slope a half, near 2 and far 8: with the frustum handed
/// over first, the cap was arranged first, and the pipe's circle was drawn
/// beside the rim it is, so two crossings left each node the same way and bent
/// alike: the touch. With the bar first, its skin was, and the cone's two
/// circles there were refused as a pair nothing could put points in order
/// along: the order. `fuse/arrange.rs` now folds a curve
/// two partners cut a face in into one crossing, starts a circle a cone and a
/// pipe cross in, and reads the side of the cone off the rim it also cuts. Every
/// face is cut into the pieces it has, and all 96 refused one step on, in
/// `fuse/assemble.rs`, as an edge with one face along it. The rim came out of
/// the frustum's faces named by the cap's plane and the cone, at the frustum's
/// seam corner, and out of the bar's skin named by the cap's plane and the pipe,
/// started where that pair starts a circle: two names for one edge, each with
/// one face.
///
/// **Q1118 gave the rim one name and one corner: 96 of 96.** Two names of
/// closed curves are one curve where both are one plane against a ball, a pipe
/// or a cone and the three share more than a point, and the curve starts at the
/// corner a curve two coincident faces bring would. Every answer holds what
/// the closed forms say,
/// [`every_fuse_of_a_frustum_and_a_bar_of_its_narrow_radius_holds_what_the_closed_forms_say`].
///
/// `TwoFrustumsOnOneAxis` is a cone against another cone, and was pinned at
/// nothing answered while `intersect` refused every such pair. §50.2 item 4
/// held it out because two cones sharing an apex meet along rulings — but only
/// where their axes lean apart, and those are still refused. **This family's
/// two frustums share an apex and an axis, and meet at the apex alone**, which
/// is off both of their faces, so since Q1122 every one of its 96 fuses
/// answers, and `assemble`'s *"this crossing holds no straight line"* is right
/// about every cone-and-cone pair that reaches it.
///
/// The two corner-up-cube families are asserted at **something answered and
/// something refused**, and no more than that. Every plane of such a cube leans
/// more steeply than a shallow cone's own side, so `intersect` names every
/// crossing — but naming a crossing is where a cone's support currently ends,
/// and the arrangement, the selection and the mesher each have their own reasons
/// to give up further down. Those refusals are **named, counted and printed**,
/// and what they say is where the next lane's work is. A floor rather than a
/// ceiling is the honest assertion for a family whose real rate is the thing
/// being reported; a floor on the refusals too, because a family measured at
/// none refused would be a rate to re-derive rather than one to relax (§19.1).
///
/// The middle family, a frustum against an ordinary upright block, is the one
/// the cone work is measured on, and its history is the argument for pinning a
/// number rather than printing it.
///
/// It was pinned at **nothing answered, 96 of 96 refused, every one of them "a
/// plane cuts this cone in a hyperbola"** — written down before any of that work
/// was briefed, precisely so that an improvement would have to break a test
/// instead of moving a `println!` nobody reads. **It broke**, and what it says
/// having broken is worth more than the pin was:
///
/// - **6 of 96 answered.**
/// - The other **90 refusals all moved**, from `where_a_plane_and_a_cone_cross`
///   refusing to name the curve, to *"nothing can put points in order along this
///   curve"*. Naming and drawing a hyperbola were the whole of the geometry's
///   part; **ordering points along one is the arrangement's**.
///
/// **It has since happened a second time, and that time the rate did not move at
/// all.** `arrange`'s `HowItRuns::AlongABend` orders points along a hyperbola now,
/// and this family still answered 6 of 96 — the same six, the draw whose block
/// misses the frustum. What changed is all 90 names, and they no longer share
/// one: `select.rs`'s *a point inside a piece of a face* 60, a corner too far out
/// for the triangulator 14, a face whose boundary crosses itself 7, and four faces
/// along one edge 9.
/// [`every_fuse_of_a_block_and_a_cone_now_answers`] carries that breakdown's
/// history to where it ended, which is what §15.10 asks of a rate that stayed
/// still while everything under it moved.
///
/// **A third time, and this time it moved: 15 of 96.** §206's bulge count in
/// `select.rs` decides cap pieces that used to stop it, and nine more fuses come
/// back `Ok`. None of the nine is a cut: all nine are of pairs at
/// `slope · far = 1`, which are disjoint, and they hand an input straight back.
///
/// **A fourth time, and the rate stood still again: 15 of 96.** A cone's rim
/// circle has its two arcs named apart now, so the 24 that stopped at *four
/// faces along one edge* get past it — and all 24 stop at the next thing,
/// `rcad_geom::walk` refusing a pair of surfaces. The breakdown is five walls
/// instead of four and the floor is where it was.
///
/// **`intersect` is not what refuses them**, which this doc said until §214 went
/// looking: no call to `intersect` is on the stack at all. The mesher reaches
/// `walk` from the fuse's own volume measurement for shell nesting, and `walk`
/// asks its own question about a pair. For the 9 that pair is one cone twice —
/// `intersect` has an arm for exactly that and answers `SameSurface`, while
/// `walk` had none and fell through to *a cone crossing another curved surface*,
/// which the model does not contain. §214.
///
/// **A fifth time, and it moved again: 24 of 96.** `walk` has that arm now —
/// §214.4's fix (a), the circle a closed run on one cone names through its own
/// end — and the 9 are answers. The 15 stay, and they are a gap rather than a
/// defect: that pair really is the block's wall plane and the cone. Four walls
/// again. The smallest of the nine is
/// `tests/a_frustum_and_a_block_that_do_not_touch.rs`, which asks the kernel
/// whether its two solids touch rather than asserting it from this paragraph —
/// they do not.
///
/// **A sixth time, and the first where a block cuts a frustum: 66 of 96.**
/// e2cd175d names each hyperbola arc the fuse writes by the nappe it is on. It
/// had been named by the half-space through its chord, which holds the whole
/// other branch too, so `walk` picked the wrong branch across the apex. The 15
/// above and the 27 at the triangulator were all that, and all 42 answer — every
/// fuse of the seven rows whose block reaches into the frustum. One wall is left.
///
/// **A seventh time, and the last wall: 96 of 96.** §340 stopped the arrangement
/// gluing the patch a leaning wall cuts off the frustum's face to the rest of
/// that face through the bottom rim, and the 30 that could not find a point
/// inside a piece all answer.
///
/// So the assertion here stays a **floor at what has been measured**, in the
/// shape §19.1 asks for: this family may not answer fewer than it does today.
/// Re-derive it when it moves; do not relax it.
///
/// # A count does not say what is in the answers
///
/// `answered` counts fuses that came back `Ok`, and here that once hid the
/// difference between a cut and no cut: until e2cd175d every answer came from a
/// draw whose block misses the frustum altogether. The block's low corner is at
/// `(1, 1, ·)` and it only runs further out, so no point of it is nearer the
/// axis than `√2`; the frustum is never wider than `slope · far`. So
/// `slope · far < √2` makes them disjoint, and five rows of this grid land there.
///
/// `what_the_answered_block_fuses_actually_say.rs` checks both kinds: the rows
/// that miss hand an input straight back, to the bit and against a grid of
/// points, and the rows that cut account for their inputs' closed-form volumes
/// by inclusion and exclusion, the same whichever way round they are asked.
///
/// **And what the frustum refuses of the corner-up cubes is the cone's.** A
/// round bar of the same size in the same place, against the same cubes,
/// answers every fuse since Q1737, and the frustum refuses 12 —
/// [`a_bar_answers_every_corner_up_cube_so_what_a_frustum_refuses_there_is_the_cones`]
/// is that control, and Q1956 traces the 12.
#[test]
fn how_often_a_certified_fuse_with_a_cone_in_it_refuses() {
    let mut every = Vec::new();
    for family in [
        Family::AFrustumAgainstACornerUpCube,
        Family::APointedConeAgainstACornerUpCube,
        Family::AFrustumAgainstABlock,
        Family::AFrustumAgainstABar,
        Family::TwoFrustumsOnOneAxis,
    ] {
        let (fuses, answered, gave_up) = how_each_fuse_went(family);
        let refused: usize = gave_up.values().sum();
        println!(
            "{}: {fuses} fuses, {answered} answered, {refused} refused ({:.1}%): {gave_up:?}",
            family.named(),
            100.0 * refused as f64 / fuses as f64
        );
        assert!(fuses >= 90, "{}: only {fuses} fuses", family.named());
        every.push((family, fuses, answered, refused, gave_up));
    }

    for (family, fuses, answered, refused, gave_up) in every {
        match family {
            Family::AFrustumAgainstACornerUpCube | Family::APointedConeAgainstACornerUpCube => {
                assert!(
                    answered > 0,
                    "{}: every plane of a corner-up cube cuts a shallow cone in an ellipse, so \
                     some of these have to come through — and none of {fuses} did",
                    family.named()
                );
                assert!(
                    refused > 0,
                    "{}: measured at none refused of {fuses}, which is not what this family has \
                     ever given — re-derive the number rather than relax it",
                    family.named()
                );
            }
            Family::AFrustumAgainstABar => assert_eq!(
                (answered, refused),
                (96, 0),
                "{}: this is a cone coaxial with a pipe, which crosses in exactly two circles \
                 (§50.11). `intersect` names that pair, `three_surfaces.rs` answers a cap square \
                 to their axis (Q981), the arrangement draws the narrow rim the cap, the cone \
                 and the pipe share (Q987) and the assembly gives that rim one name (Q1118), so \
                 every fuse was measured answering. It answered {answered} of {fuses} here, \
                 refusing {gave_up:?}. When this breaks, re-derive the number over this whole \
                 family rather than relax it (§19.1)",
                family.named()
            ),
            Family::TwoFrustumsOnOneAxis => assert_eq!(
                (answered, refused),
                (96, 0),
                "{}: two frustums of one apex and two slopes on one axis meet at that apex and \
                 nowhere else, which `intersect` answers as touching since Q1122, so no face is \
                 cut by the other cone; every fuse was measured answering, and \
                 `a_cone_hollowed_along_its_axis.rs` checks what they hold. It answered \
                 {answered} of {fuses} here. When this breaks, re-derive the number over the \
                 family rather than relax it (§19.1)",
                family.named()
            ),
            Family::AFrustumAgainstABlock => assert!(
                answered >= 96,
                "{}: this family was measured at 6 answered of 96 the day the hyperbola landed, \
                 at 15 the day §206's bulge count landed, at 24 the day `walk` learnt that \
                 one cone twice is one surface (§214), at 66 the day e2cd175d named a \
                 hyperbola arc by its nappe and at 96 the day §340 stopped gluing two walks \
                 of one lump into one cell, and it answered {answered} of {fuses} here. A cone that could be cut yesterday and cannot today is a regression, not \
                 a rate: re-derive this number rather than relax it (§19.1)",
                family.named()
            ),
        }
    }
}

/// Every fuse of the family `AFrustumAgainstABar` draws, against the closed
/// forms, so that its rate counts answers that hold what they should.
///
/// The bar has the narrow cap's radius `r = slope · near` and runs from
/// `−far` to `far` up the frustum's axis: it passes up through the narrow
/// cap, whose rim it shares, and ends flush with the wide cap. Below the
/// narrow cap it is outside the frustum and above it inside, since the
/// frustum is `slope · z` wide at height `z`. So the frustum holds
/// `π slope² (far³ − near³) / 3`, the bar below the narrow cap
/// `πr² (far + near)` and the bar above it `πr² (far − near)`, and every rule
/// is one solid: the union is the frustum and the bar below, the intersection
/// the bar above, the frustum less the bar is the frustum bored through, and
/// the bar less the frustum is the bar below.
#[test]
fn every_fuse_of_a_frustum_and_a_bar_of_its_narrow_radius_holds_what_the_closed_forms_say() {
    let policy = Policy::certified();
    let how_fine = Deflection::of(0.01).expect("a positive length");
    let pi = core::f64::consts::PI;
    let mut draw = Draw(0x5EED);
    for _ in 0..DRAWS {
        let [slope, near, far] =
            a_cones_sizes(&mut draw.clone()).expect("every family draws these first");
        let (model, cone, bar) = a_pair(Family::AFrustumAgainstABar, &mut draw)
            .expect("every pair of this family is built");
        let r = slope * near;
        let frustum = pi * slope * slope * (far.powi(3) - near.powi(3)) / 3.0;
        let below = pi * r * r * (far + near);
        let above = pi * r * r * (far - near);
        for (one, other, rule, holds) in [
            (cone, bar, Rule::Union, frustum + below),
            (bar, cone, Rule::Union, frustum + below),
            (cone, bar, Rule::Intersection, above),
            (bar, cone, Rule::Intersection, above),
            (cone, bar, Rule::Difference, frustum - above),
            (bar, cone, Rule::Difference, below),
        ] {
            let mut fused = model.clone();
            let cone_first = one == cone;
            let what = format!(
                "the {rule:?} of slope {slope}, near {near}, far {far} (frustum first: \
                 {cone_first})"
            );
            let answer = fused
                .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} answers since Q1118, and refused {why}"));
            fused
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} does not validate: {why}"));
            let [solid] = answer.solids() else {
                panic!("{what} is one solid, not {:?}", answer.solids());
            };
            let measured = fused
                .solid(*solid)
                .expect("the fuse made the key")
                .measure(how_fine, &policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} measures: {why}"));
            assert!(
                (measured.volume() - holds).abs() <= measured.volume_error(),
                "{what} holds {} against {holds}, out by more than the {} its mesh declares",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// A cube stood on its corner leans every one of its planes across a shallow
/// cone, and this is that angle measured rather than assumed.
///
/// Every family that fuses anything rests on it. What is checked is the sign the
/// intersector actually reads, `E = (n·w)² − slope²‖n × w‖²`, through
/// `intersect` itself rather than through a re-derivation of it: each of the
/// cube's six planes crossed with the cone has to come back one closed curve.
#[test]
fn a_cube_on_its_corner_leans_every_one_of_its_planes_across_a_shallow_cone() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = a_block(&mut model, [6.0; 3], Vector::new(0.0, 0.0, 0.0)).expect("a cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(0.0, 0.0, 5.0))
                .expect("a shift")
                .compose(&stood_on_its_corner()),
        )
        .expect("this model made the key");

    let cone = rcad_geom::Surface::from(
        rcad_geom::Cone::new(rcad_geom::Axis::z(), 0.5).expect("a positive slope"),
    );
    let mut leans = Vec::new();
    for face in model.solid(cube).expect("this model made the key").faces() {
        let flat = face
            .surface()
            .as_plane()
            .copied()
            .expect("every face of a cube is flat");
        leans.push(flat.normal().z().abs());
        let met = rcad_geom::intersect(
            &rcad_geom::Surface::from(flat),
            &cone,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane leaning across a cone cuts an ellipse");
        assert!(
            matches!(met, rcad_geom::SurfaceMeet::Curves(ref runs) if runs.len() == 1),
            "this plane cuts the cone in {met:?} rather than in one closed curve"
        );
    }
    let shallowest = leans.iter().copied().fold(f64::INFINITY, f64::min);
    println!("a corner-up cube's planes lean at least {shallowest} of the way to its axis");
    assert!(
        shallowest > 0.5 / 1.25f64.sqrt(),
        "every plane has to lean more steeply than the cone's own side, and the shallowest \
         leans {shallowest}"
    );
}

/// A frustum intersected with a cube that swallows it whole is the frustum, and
/// it still measures its closed form.
///
/// **The closed form through a real fuse.** Every one of the cube's six planes is
/// put to `intersect` against the cone and comes back an ellipse; none of those
/// ellipses lands on the face, so what comes out is the frustum again — and its
/// volume is `πh(R² + Rr + r²)/3` and its skin `π(R + r)·slant` plus two discs,
/// checked against the kernel's own published bound rather than against a number
/// recorded from a previous run.
#[test]
fn a_frustum_swallowed_by_a_cube_comes_back_the_same_frustum() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_frustum(&mut model, 0.5, 4.0, 8.0);
    let cube = a_block(&mut model, [40.0; 3], Vector::new(-20.0, -20.0, -20.0))
        .expect("a cube big enough to swallow the frustum");
    model
        .place_solid(cube, &stood_on_its_corner())
        .expect("this model made the key");

    let answer = model
        .fuse(&[cone], &[cube], Rule::Intersection, &policy, &mut log)
        .expect("every plane of a corner-up cube cuts a shallow cone in an ellipse");
    let pieces: Vec<SolidKey> = answer.solids().to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("and the answer is a sound model");
    assert_eq!(pieces.len(), 1, "a frustum inside a cube is one solid");

    let how_fine = rcad_geom::Deflection::of(0.002).expect("a positive length");
    let measured = model
        .solid(pieces[0])
        .expect("this model made the key")
        .measure(how_fine, &policy, &mut ProvenanceLog::new())
        .expect("the answer measures");

    let (small, big, height) = (2.0f64, 4.0f64, 4.0f64);
    let volume = core::f64::consts::PI * height * (big * big + big * small + small * small) / 3.0;
    let slant = (height * height + (big - small) * (big - small)).sqrt();
    let area = core::f64::consts::PI * ((big + small) * slant + big * big + small * small);
    println!(
        "a frustum through an intersection: volume {} against {volume}, area {} against {area}",
        measured.volume(),
        measured.surface_area()
    );
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the answer's volume {} misses πh(R² + Rr + r²)/3 = {volume} by more than the bound {}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.surface_area() - area).abs() <= measured.area_error(),
        "the answer's area {} misses {area} by more than the bound {}",
        measured.surface_area(),
        measured.area_error()
    );
}

/// A cone-faced solid taken apart through views alone and described again is the
/// same solid.
///
/// The round trip §42.12 criterion 1 asks for, over a cone. Nothing is read out
/// of the arena: the faces, wires, coedges, edges and corners all come through
/// `FaceView` and friends, and what goes back in is a fresh `SolidBuilder`
/// description. The two are then required to measure the same **bit for bit**
/// and to answer the same fuse against a third solid — a corner-up cube taking a
/// bite out of each, with every piece's volume, area and centroid compared and
/// the piece count compared.
#[test]
fn a_frustum_taken_apart_through_views_and_described_again_is_the_same_frustum() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let built = a_frustum(&mut model, 0.5, 4.0, 8.0);

    let rebuilt = cones::describe_again(&model, built)
        .expect("a frustum can be described again")
        .build(&mut model, &policy, &mut log)
        .expect("and the sweep is happy with the description");
    model
        .validate(&policy, &mut log)
        .expect("the model holding both is sound");

    let how_fine = rcad_geom::Deflection::of(0.01).expect("a positive length");
    let [one, other] = [built, rebuilt].map(|solid| {
        model
            .solid(solid)
            .expect("this model made the key")
            .measure(how_fine, &policy, &mut ProvenanceLog::new())
            .expect("both measure")
    });
    assert_eq!(one.volume(), other.volume());
    assert_eq!(one.surface_area(), other.surface_area());
    assert_eq!(one.centroid(), other.centroid());

    // And the same cut, on a clone of the model each time, has to give the same
    // pieces — which is what says the two descriptions are the same *solid* and
    // not merely two solids of the same size.
    let cut = |which: SolidKey| {
        let mut copy = model.clone();
        let cube = a_block(&mut copy, [5.0; 3], Vector::new(0.0, 0.0, 0.0)).expect("a cube");
        copy.place_solid(cube, &stood_on_its_corner())
            .expect("this model made the key");
        let answer = copy
            .fuse(
                &[which],
                &[cube],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a corner-up cube cuts a shallow cone in ellipses");
        let keys: Vec<SolidKey> = answer.solids().to_vec();
        assert!(!keys.is_empty(), "the cut has to leave something behind");
        let mut pieces: Vec<(f64, f64, [f64; 3])> = keys
            .iter()
            .map(|&piece| {
                let measured = copy
                    .solid(piece)
                    .expect("this model made the key")
                    .measure(how_fine, &policy, &mut ProvenanceLog::new())
                    .expect("each piece measures");
                (
                    measured.volume(),
                    measured.surface_area(),
                    measured.centroid().coordinates(),
                )
            })
            .collect();
        pieces.sort_by(|one, other| one.0.total_cmp(&other.0));
        pieces
    };
    let from_the_original = cut(built);
    let took_material_away =
        from_the_original.iter().map(|piece| piece.0).sum::<f64>() < one.volume() - 0.01;
    assert!(
        took_material_away,
        "the cut has to take material away, or this compared nothing: {from_the_original:?}"
    );
    assert_eq!(from_the_original, cut(rebuilt));
}

/// A point inside a cone-faced solid is inside it, and one outside is outside.
///
/// The ray caster's cone arm, which is the one place a cone's quadratic can have
/// a **negative** squared term — every ray pointing further into the cone than
/// its own rulings do. A ball's and a pipe's never can, and the code said so in
/// an `unreachable!` until this lane.
#[test]
fn a_grid_of_points_is_classified_the_way_the_cone_says() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_frustum(&mut model, 0.5, 4.0, 8.0);
    let view = model.solid(solid).expect("this model made the key");

    let inside = |at: Point| {
        let height = at.z();
        let across = at.x().hypot(at.y());
        height > 4.0 && height < 8.0 && across < 0.5 * height
    };
    let outside = |at: Point| {
        let height = at.z();
        let across = at.x().hypot(at.y());
        !(4.0..=8.0).contains(&height) || across > 0.5 * height
    };

    let mut counted = 0;
    for x in -6..=6 {
        for y in -6..=6 {
            for z in 0..=12 {
                let at = Point::new(f64::from(x) * 0.7, f64::from(y) * 0.7, f64::from(z) * 0.9);
                let held = view.classify_point(at, &policy, &mut ProvenanceLog::new());
                if inside(at) {
                    assert_eq!(held, Containment::Inside, "{at:?} is inside the frustum");
                    counted += 1;
                } else if outside(at) {
                    assert_eq!(held, Containment::Outside, "{at:?} is outside the frustum");
                    counted += 1;
                }
            }
        }
    }
    assert!(counted > 1000, "only {counted} points were decided");
}

/// **The control that says which of a frustum's corner-up refusals are the
/// cone's own.**
///
/// The same cubes, stood on their corners in the same places, are put to a
/// frustum and to a round bar as wide as the frustum's wide end and as tall.
/// A cube's planes cut the bar in ellipses as they cut the cone, so what the
/// bar refuses is what this family costs any curved solid, and what the
/// frustum refuses past that is the cone's. Over the sixteen draws:
///
/// | | answered | refused |
/// |---|---:|---:|
/// | a frustum | 84 | 12 |
/// | a round bar of the same size | 96 | 0 |
///
/// **This control used to say the opposite**: the bar refused about as often
/// as the frustum, so the frustum's refusals were read as the family's. Q1737
/// taught the arrangement to count a loop a plane cuts round a bar from the
/// one corner whose box holds the loop's start, where that corner and the start
/// lie too close to put in order, and the bar now answers every fuse. So the
/// frustum's 12 are the cone's: all six fuses of draws 1 and 14 (slope a half,
/// far 8, near 4 and 3, a cube of 7 and of 8 at height 4), each refused as
/// *"could not settle 1 decisions the answer depends on"*. Q1956 traces them.
///
/// **Every answer is checked, not only counted**, because a fuse that wrote the
/// wrong solid would raise the count as much as one that wrote the right one.
/// [`the_six_fuses_held_to_their_inputs`] says how. Every draw's cube reaches
/// into the bar, so every one of the bar's 32 intersections holds something,
/// and so do the frustum's 28.
///
/// **The frustum is held to a floor at the 84 measured**, in the shape §19.1
/// asks for: fewer is a regression, and more is Q1956's work, when the floor is
/// re-derived rather than left behind.
#[test]
fn a_bar_answers_every_corner_up_cube_so_what_a_frustum_refuses_there_is_the_cones() {
    let policy = Policy::certified();
    let mut gave_up: [BTreeMap<String, usize>; 2] = [BTreeMap::new(), BTreeMap::new()];
    let mut answered = [0usize; 2];
    let mut tries = [0usize; 2];
    let mut cut = [0usize; 2];
    let mut draw = Draw(0x5EED);

    for at in 0..DRAWS {
        let slope = 2.0_f64.powi(i32::try_from(draw.next() % 3).expect("a small number") - 3);
        let far = draw.whole(6, 12);
        let near = draw.whole(1, 4);
        let side = draw.whole(4, 9);
        let up = draw.whole(1, far as i64 - 1);
        let placed = Isometry::translation(Vector::new(0.0, 0.0, up))
            .expect("a finite shift")
            .compose(&stood_on_its_corner());

        for which in 0..2 {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            // A bar as wide as the frustum's wide end and as tall as it is, so
            // the cube meets at least as much curved surface either way.
            let solid = if which == 0 {
                a_frustum(&mut model, slope, near, far)
            } else {
                model
                    .add_round_bar(
                        slope * far,
                        far - near,
                        &Isometry::translation(Vector::new(0.0, 0.0, near))
                            .expect("a finite shift"),
                    )
                    .expect("a bar of that radius")
            };
            let wide_at = |height: f64| {
                if which == 0 {
                    slope * height
                } else {
                    slope * far
                }
            };
            let cube = a_block(&mut model, [side; 3], Vector::new(0.0, 0.0, 0.0))
                .expect("a cube of a whole side");
            model
                .place_solid(cube, &placed)
                .expect("this model made the key");
            let back = placed.inverse();
            let in_both = |at: Point| {
                let in_the_solid =
                    at.z() > near && at.z() < far && at.x().hypot(at.y()) < wide_at(at.z());
                in_the_solid
                    && back
                        .apply_to_point(at)
                        .coordinates()
                        .iter()
                        .all(|&along| along > 0.0 && along < side)
            };
            let what = format!(
                "draw {at} ({}, slope {slope}, near {near}, far {far}, a cube of {side} at {up})",
                ["a frustum", "a bar"][which]
            );
            let sampled = sampled_volume(
                [slope * far, near, far],
                in_both,
                &mut Draw(0x5A3D + at as u64),
            );

            for (rule, answer) in
                the_six_fuses_held_to_their_inputs(&model, [solid, cube], sampled, &what, &policy)
            {
                tries[which] += 1;
                match answer {
                    Ok((volume, error)) => {
                        answered[which] += 1;
                        if matches!(rule, Rule::Intersection) && volume > error {
                            cut[which] += 1;
                        }
                    }
                    Err(why) => *gave_up[which].entry(why).or_default() += 1,
                }
            }
        }
    }

    println!(
        "a frustum: {} of {} answered, {:?}",
        answered[0], tries[0], gave_up[0]
    );
    println!(
        "a bar of the same size: {} of {} answered, {:?}",
        answered[1], tries[1], gave_up[1]
    );
    println!("intersections with something in them, frustum and bar: {cut:?}");
    assert_eq!(
        tries,
        [6 * DRAWS; 2],
        "every draw asks all six fuses of both pairs"
    );
    assert_eq!(
        (answered[1], &gave_up[1]),
        (tries[1], &BTreeMap::new()),
        "a bar against a corner-up cube was measured answering every fuse since Q1737, each \
         adding up with its inputs and its intersection where points drawn at random put it. \
         Re-derive this rather than relax it (§19.1)"
    );
    assert_eq!(
        cut[1],
        2 * DRAWS,
        "every draw's cube reaches into the bar, so no check above passed for want of a cut"
    );
    assert!(
        answered[0] >= 84,
        "a frustum against a corner-up cube was measured answering 84 of 96, every answer \
         checked as the bar's are, and it answered {} here. The 12 it refused are all six fuses \
         of draws 1 and 14, which Q1956 traces. Fewer is a regression; more is Q1956's work, \
         and then this floor is re-derived rather than left (§19.1)",
        answered[0]
    );
}

/// Union, intersection and difference of `solid` and `cube`, each both ways
/// round, every answer validated and measured, and every sum the answers
/// make up held to the inputs' own measured volumes:
///
/// - the union and the intersection make up the two inputs;
/// - the solid less the cube, and the intersection, make up the solid;
/// - the cube less the solid, and the intersection, make up the cube.
///
/// Those three hold whatever the intersection holds, so the intersection is
/// held as well to `sampled`, the volume points drawn at random say it has, and
/// its standard deviation: within five of those and its own declared error.
/// A fuse that refused is handed back by what it said, and checks nothing.
fn the_six_fuses_held_to_their_inputs(
    model: &Model,
    [solid, cube]: [SolidKey; 2],
    (sampled, sigma): (f64, f64),
    what: &str,
    policy: &Policy,
) -> SixAnswers {
    let fused = |rule, arguments, tools| {
        let answer = fused_and_measured(model, rule, arguments, tools, policy)
            .map_err(|why| format!("{why}"));
        if let Err(why) = &answer {
            let first = if arguments == cube {
                "the cube"
            } else {
                "the solid"
            };
            println!("{what}: the {rule:?} with {first} first refused {why}");
        }
        answer
    };
    let one_solid = measured(model, &[solid], policy);
    let one_cube = measured(model, &[cube], policy);
    let solid_less = fused(Rule::Difference, solid, cube);
    let cube_less = fused(Rule::Difference, cube, solid);
    let mut all = vec![
        (Rule::Difference, solid_less.clone()),
        (Rule::Difference, cube_less.clone()),
    ];
    for (one, other) in [(solid, cube), (cube, solid)] {
        let union = fused(Rule::Union, one, other);
        let both = fused(Rule::Intersection, one, other);
        if let Ok(both) = both {
            assert!(
                (both.0 - sampled).abs() <= 5.0 * sigma + both.1,
                "{what}: the intersection measures {} ± {}, and points drawn at random say \
                 {sampled} ± {sigma}",
                both.0,
                both.1
            );
            if let Ok(union) = union {
                within(
                    what,
                    "union and intersection make up the two inputs",
                    [union, both],
                    [one_solid, one_cube],
                );
            }
            if let Ok(solid_less) = solid_less {
                within(
                    what,
                    "the solid less the cube, and the intersection, make up the solid",
                    [solid_less, both],
                    [one_solid, (0.0, 0.0)],
                );
            }
            if let Ok(cube_less) = cube_less {
                within(
                    what,
                    "the cube less the solid, and the intersection, make up the cube",
                    [cube_less, both],
                    [one_cube, (0.0, 0.0)],
                );
            }
        }
        all.push((Rule::Union, union));
        all.push((Rule::Intersection, both));
    }
    all
}

/// Each fuse's rule, with the volume of what it handed back and that volume's
/// declared error, or what it said when it refused.
type SixAnswers = Vec<(Rule, Result<(f64, f64), String>)>;

/// One fuse on a copy of `model`, validated, with the volume of every solid
/// it hands back added up and the sum of their errors.
fn fused_and_measured(
    model: &Model,
    rule: Rule,
    arguments: SolidKey,
    tools: SolidKey,
    policy: &Policy,
) -> Result<(f64, f64), Error> {
    let mut model = model.clone();
    let answer = model.fuse(
        &[arguments],
        &[tools],
        rule,
        policy,
        &mut ProvenanceLog::new(),
    )?;
    model
        .validate(policy, &mut ProvenanceLog::new())
        .expect("a fuse hands back a sound model or refuses");
    Ok(measured(&model, answer.solids(), policy))
}

fn measured(model: &Model, solids: &[SolidKey], policy: &Policy) -> (f64, f64) {
    solids.iter().fold((0.0, 0.0), |(volume, error), &solid| {
        let it = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                policy,
                &mut ProvenanceLog::new(),
            )
            .expect("every solid here measures");
        (volume + it.volume(), error + it.volume_error())
    })
}

/// Two measured volumes against two others, within the errors all four
/// declare and the rounding of the three steps that take the difference.
fn within(what: &str, sum: &str, parts: [(f64, f64); 2], whole: [(f64, f64); 2]) {
    let off_by = parts[0].0 + parts[1].0 - whole[0].0 - whole[1].0;
    let sizes = parts[0].0.abs() + parts[1].0.abs() + whole[0].0.abs() + whole[1].0.abs();
    let bound =
        parts[0].1 + parts[1].1 + whole[0].1 + whole[1].1 + 3.0 * (f64::EPSILON / 2.0) * sizes;
    assert!(
        off_by.abs() <= bound,
        "{what}: {sum}, and are out by {off_by:e} against {bound:e}: {parts:?} against {whole:?}"
    );
}

/// The volume inside `holds`, from points drawn at random over the box
/// `[-radius, radius]² × [near, far]`, and one standard deviation of it.
fn sampled_volume(
    [radius, near, far]: [f64; 3],
    holds: impl Fn(Point) -> bool,
    draw: &mut Draw,
) -> (f64, f64) {
    const POINTS: u32 = 400_000;
    let mut uniform = |low: f64, high: f64| low + (high - low) * draw.next() as f64 / 2f64.powi(31);
    let mut inside = 0u32;
    for _ in 0..POINTS {
        let at = Point::new(
            uniform(-radius, radius),
            uniform(-radius, radius),
            uniform(near, far),
        );
        if holds(at) {
            inside += 1;
        }
    }
    let the_box = 4.0 * radius * radius * (far - near);
    let share = f64::from(inside) / f64::from(POINTS);
    (
        the_box * share,
        the_box * (share * (1.0 - share) / f64::from(POINTS)).sqrt(),
    )
}

/// `A ∪ B` is `B ∪ A` and `A ∩ B` is `B ∩ A`, over every cone pair that answers.
///
/// The metamorphic half of `fuses_that_must_agree.rs`, over a surface kind that
/// did not exist when that file was written. Union and intersection are the two
/// rules that commute; difference does not and is left out rather than fudged.
///
/// **Compared on the answer's own geometry and not on a measurement of it**, and
/// the first attempt at this compared measurements and failed by `5e-14` on a
/// volume of 220. That was not the fuse disagreeing: the two answers are the
/// same solid, and what differed was where each mesh happened to start round the
/// cone. A measurement is a mesh away from the thing being compared, so the
/// comparison is on what the fuse actually wrote — every corner of every face,
/// bit for bit.
///
/// **The count that answered is printed and floored**, because a family that
/// refused every fuse would satisfy this vacuously — §14.7's rule, and the whole
/// reason the two rates above are reported rather than assumed.
#[test]
fn uniting_a_cone_with_a_cube_gives_the_same_answer_whichever_way_round_it_is_asked() {
    let policy = Policy::certified();
    let mut compared = 0;
    let mut both_refused = 0;

    for family in [
        Family::AFrustumAgainstACornerUpCube,
        Family::APointedConeAgainstACornerUpCube,
    ] {
        let mut draw = Draw(0x5EED);
        for _ in 0..DRAWS {
            let Some((model, cone, cube)) = a_pair(family, &mut draw) else {
                continue;
            };
            for rule in [Rule::Union, Rule::Intersection] {
                let written = |arguments: [SolidKey; 1], tools: [SolidKey; 1]| {
                    let mut copy = model.clone();
                    let answer =
                        copy.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())?;
                    let keys: Vec<SolidKey> = answer.solids().to_vec();
                    copy.validate(&policy, &mut ProvenanceLog::new())?;
                    let mut corners: Vec<[u64; 3]> = Vec::new();
                    for piece in keys {
                        for face in copy.solid(piece)?.faces() {
                            for wire in face.wires() {
                                for at in wire.points() {
                                    corners.push(at.coordinates().map(f64::to_bits));
                                }
                            }
                        }
                    }
                    corners.sort_unstable();
                    Ok::<_, rcad_topo::Error>(corners)
                };
                match (written([cone], [cube]), written([cube], [cone])) {
                    (Ok(one), Ok(other)) => {
                        assert!(
                            !one.is_empty(),
                            "{rule:?} of a cone and a cube came back with no corners at all"
                        );
                        assert_eq!(
                            one, other,
                            "{rule:?} of a cone and a cube wrote different corners depending on \
                             which was named first"
                        );
                        compared += 1;
                    }
                    (Err(_), Err(_)) => both_refused += 1,
                    // One way round answered and the other refused, which is
                    // allowed: a refusal is about the arithmetic and the two
                    // orders ask different questions of it. Nothing to compare.
                    _ => (),
                }
            }
        }
    }

    println!(
        "a cone and a corner-up cube, both ways round: {compared} pairs compared, \
         {both_refused} refused both ways"
    );
    assert!(
        compared >= 4,
        "only {compared} pairs were ever compared, so this proved almost nothing"
    );
}

/// The curve is still a hyperbola and it still never closes — and the fuse
/// across it now answers, and adds up.
///
/// This test used to end by asserting that the fuse of the pair below came back
/// *"nothing can put points in order along this curve"*, which is where the
/// block family's 90 refusals stopped. `arrange`'s `HowItRuns::AlongABend` puts
/// points in order along a bend, so that step was turned round to *refuses, but
/// not for want of an order*. Then e2cd175d named each hyperbola arc the fuse
/// writes by the nappe it is on, where it had been named by the half-space
/// through its chord — which holds the whole other branch too, so `walk` picked
/// the wrong branch across the apex — and the refusal went away. The step is now
/// an answer checked for being right.
///
/// Three things, in the order the fuse asks them, over a block wall:
///
/// 1. **`intersect` names the curve.** The geometry is not where this stops, and
///    never was after the hyperbola landed.
/// 2. **`where_a_closed_crossing_starts` still refuses it**, with the sharp
///    name: *these two surfaces cross in a curve that runs on for ever rather
///    than a closed one*. A hyperbola branch is a conic and is not closed, so
///    going round it is not an order anything can use. That is a fact about the
///    curve and no amount of arrangement work changes it.
/// 3. **The union and the intersection of that same pair answer**, each into a
///    sound model, and they account for the two inputs: `U + I = A + B`, with
///    `A` and `B` measured from each input alone, within the sum of the four
///    bounds `measure` published. The intersection is required to be more than
///    its own bound first, so this is a draw where the block really cuts the
///    frustum and not one where it hands an input back.
///
/// So step 2 is what keeps this test honest. Without it, "the fuse got past the
/// ordering" could mean the curve had quietly stopped being a hyperbola.
#[test]
fn a_block_wall_beside_a_cone_still_cuts_a_hyperbola_that_never_closes() {
    let policy = Policy::certified();
    let mut draw = Draw(0x5EED);
    let (model, cone, block) =
        a_pair(Family::AFrustumAgainstABlock, &mut draw).expect("the family's first draw");

    let skin = model
        .solid(cone)
        .expect("this model made the key")
        .faces()
        .find(|face| matches!(face.surface(), rcad_geom::Surface::Cone(_)))
        .expect("a frustum has a cone face")
        .surface();
    let wall = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| {
            surface
                .as_plane()
                .is_some_and(|flat| flat.normal().z() == 0.0)
        })
        .expect("four of a block's walls run parallel to the cone's axis");

    let met = rcad_geom::intersect(&wall, &skin, &policy, &mut ProvenanceLog::new())
        .expect("naming a plane against a cone is no longer where this stops");
    let rcad_geom::SurfaceMeet::Curves(runs) = met else {
        panic!("a wall beside a cone cuts it in a curve, and this came back {met:?}");
    };
    println!("a block wall against the cone: {} run(s)", runs.len());

    for run in &runs {
        let stopped = rcad_geom::where_a_closed_crossing_starts(
            run,
            [&wall, &skin],
            &mut ProvenanceLog::new(),
        )
        .expect_err("a hyperbola branch never closes, so it has no place to start");
        assert!(
            matches!(
                stopped,
                rcad_geom::Error::TheCrossingOfTheseTwoSurfacesIsNotClosed
            ),
            "the ordering has to stop because the curve does not close, and it stopped on \
             {stopped:?}"
        );
    }

    let how_fine = Deflection::of(0.002).expect("a positive length");
    let volume_of = |model: &Model, pieces: &[SolidKey]| -> (f64, f64) {
        pieces.iter().fold((0.0, 0.0), |(volume, bound), &piece| {
            let it = model
                .solid(piece)
                .expect("this model made the key")
                .measure(how_fine, &policy, &mut ProvenanceLog::new())
                .expect("every solid here measures");
            assert!(
                it.volume_error().is_finite(),
                "a bound of {} checks nothing",
                it.volume_error()
            );
            (volume + it.volume(), bound + it.volume_error())
        })
    };
    let fused = |rule: Rule| -> (f64, f64) {
        let mut copy = model.clone();
        let answer = copy
            .fuse(&[cone], &[block], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!("the {rule:?} of a block wall across a cone answers since e2cd175d: {why}")
            });
        let pieces: Vec<SolidKey> = answer.solids().to_vec();
        copy.validate(&policy, &mut ProvenanceLog::new())
            .expect("a fuse hands back a sound model or refuses");
        volume_of(&copy, &pieces)
    };
    let (frustum, frustum_bound) = volume_of(&model, &[cone]);
    let (the_block, block_bound) = volume_of(&model, &[block]);
    let (union, union_bound) = fused(Rule::Union);
    let (intersection, intersection_bound) = fused(Rule::Intersection);
    assert!(
        intersection > intersection_bound,
        "this draw's block has to cut a piece out of the frustum, or nothing here is a cut; \
         the intersection measured {intersection} within {intersection_bound}"
    );
    let off_by = union + intersection - frustum - the_block;
    let bound = union_bound + intersection_bound + frustum_bound + block_bound;
    println!(
        "and the fuse of that same pair answers: U {union} + I {intersection} − A {frustum} − \
         B {the_block} = {off_by:e}, within {bound:e}"
    );
    assert!(
        off_by.abs() <= bound,
        "the union and the intersection of a block wall across a cone hold {off_by} more than \
         the two inputs, beyond the bound {bound}"
    );
}

/// Every fuse of a frustum against a block answers, and nothing is left to name.
///
/// §15.10: a rate that moves is published with the list of what moved. It has
/// moved six times. First the bend landed and the rate stood still — 6 of 96
/// either side — while all 90 refusals changed their name from *"nothing can put
/// points in order along this curve"* to four different walls. Then §206's bulge
/// count landed and the rate moved to 15 of 96, and the four walls moved under
/// it. Then a cone's rim circle had its two arcs named apart, and the rate stood
/// still at 15 of 96 while one of the four walls emptied into two new ones. Then
/// `walk` learnt the pair that is one cone twice, and the 9 that wall held became
/// answers: **24 of 96**, and four walls again. Then e2cd175d named each
/// hyperbola arc the fuse writes by the nappe it is on, and three of the four
/// walls emptied at once: **66 of 96**, and one wall. Then §340 emptied that
/// one: **96 of 96**, and no wall at all.
///
/// **The three that vanished were all misnamed hyperbola arcs.** The fuse used
/// to name an arc of a plane-cone hyperbola by the half-space through its
/// chord, and that half-space also holds the whole of the other branch, so
/// `rcad_geom::walk` picked the wrong branch across the cone's apex. Every one
/// of the 42 fuses behind these three walls answers now, and
/// `what_the_answered_block_fuses_actually_say.rs` checks each of them against
/// the inputs' closed forms rather than taking the count on trust.
///
/// Measured over this family's own draws rather than taken from anywhere:
///
/// - **a point inside a piece of a face**, 60 → 30 → 30 → **gone** (§340).
///   `select.rs` could not find a point inside a piece it had cut. Halved by
///   §206 teaching `in_the_bulge_of` to count the bulge of a cone whose crossing
///   with the piece's plane is a circle. §340.1 reads the last 30 as not
///   `select.rs`'s at all: `arrange.rs` had glued the patch a leaning wall cuts
///   off the frustum's face to the rest of that face, through the bottom rim,
///   which agreed with both, and handed on one cell that was two regions.
/// - **four faces along one edge**, 9 → 24 → **gone**. The answer was not
///   something a `Model` can hold, which is §17.7's catalogued case and not an
///   arithmetic refusal at all. It rose under §206 because fuses that used to
///   stop at the wall above got far enough to reach it, and what they reached
///   was the edge-naming gap `in_the_bulge_of`'s own doc named —
///   `assemble`'s `which_arc_of_a_circle_this_is` read a ball-and-plane circle
///   only, so two arcs of a cone's rim circle between the same corners were
///   named as one edge. Naming them apart empties this wall.
/// - **a corner too far out for the triangulator**, 14 → 18 → 18 → **gone**
///   (e2cd175d). A face went to be cut into triangles and a corner of it was
///   outside the range the triangulator takes — a corner of an arc drawn on the
///   wrong branch.
/// - **a face whose boundary crosses itself**, 7 → 9 → 9 → **gone** (e2cd175d).
///   The same step, refusing for the other of its two reasons.
/// - **a cone crossing another curved surface**, 9 → **gone**. A **defect**, and
///   §214.4 ruled it one: the pair was one cone twice, `==` true bit for bit, and
///   the message named a second curved surface a frustum and a block have not got
///   between them. `walk` answers that pair now — the circle the run's own end
///   names — and all 9 became answers.
/// - **a crossing in two pieces with no half-space to tell them apart**, 15 →
///   **gone** (e2cd175d). The pair there really is the block's wall plane and
///   the cone, a genuine hyperbola whose two branches were both held by the
///   run's picker — because the half-space it was handed was the one through
///   the chord, which holds both. Named by its nappe, the arc's half-space holds
///   one branch and `walk` takes it.
///
///   The 9 and the 15 together were exactly the 24 the four-faces wall emptied
///   into, and **that they are the same 24 was inferred, not measured**, which
///   this doc said the opposite of: `tests/which_arc_of_a_cones_rim_a_side_is.rs`
///   walks §49.6's four-configuration grid and contains no `Family` or `Draw(` at
///   all, so it cannot have pinned rows of this family. §214 got the 24 by a
///   census of this family instead, and they are the same 24.
///
/// Any refusal at all fails the test, so none of the five walls that have
/// emptied can come back unnoticed, and neither can a new one. That the answers
/// are the right solids is `what_the_answered_block_fuses_actually_say.rs`'s to
/// check, not this count's.
#[test]
fn every_fuse_of_a_block_and_a_cone_now_answers() {
    let (fuses, answered, gave_up) = how_each_fuse_went(Family::AFrustumAgainstABlock);
    println!("a frustum against a block: {answered} of {fuses} answered, and the rest {gave_up:?}");

    assert_eq!(
        (answered, gave_up),
        (96, BTreeMap::new()),
        "this family was measured at every fuse answering; re-derive the count when it \
         moves rather than widening the test (§19.1)"
    );
}
