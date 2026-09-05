//! If a solid meshes at one deflection, does it mesh at every coarser one?
//!
//! A consumer that retries a failed mesh at 1e-3, 1e-2 and 1e-1 of the model's
//! size is asking this without saying so. It tries every rung, so a success it
//! only reaches after coarsening is not a mesh it misses — but what the answer
//! is worth depends on which way it goes:
//!
//! - **If a refusal never comes back once a finer rung has worked**, then a
//!   solid that only meshes after coarsening is a finger pointing at this
//!   kernel. The mesher's answer to "too rough" is more triangles, so a rung
//!   that had to be climbed is a bug indicator and not a fact of life.
//! - **If a refusal can come back**, "it meshes at 1e-1" says nothing at all
//!   about 1e-2, and no consumer may treat the coarsest rung as a superset of
//!   the others.
//!
//! Either answer is worth having, so a non-monotone case here is this file
//! working and not this file failing.
//!
//! # What came out, and it took a fix to get there
//!
//! **It was the second one, and the case that said so was a bug.** This file
//! first found a pointed cone of slope one up to two that meshed at a
//! ten-thousandth of its size, refused at a thousandth, and meshed again at a
//! hundredth — six refusals over twenty-one deflections in that stretch, with
//! nine meshes sitting *finer* than the coarsest of them. That is
//! `docs/decisions.md` §51.7, and the refusal was
//! [`CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection`] every time.
//!
//! It was a **gap and not a limit** (§16.10). Cutting a cone's face up to its
//! apex leaves the apex in the patch at two longitudes a quarter turn apart, so
//! every triangulation of it has a triangle with two corners on the apex — in 3D
//! a ruling, which lies exactly on the cone and strays by nothing.
//! `rcad-mesh`'s bound had collapsed the piece into one chord measured on the
//! circle of its widest corner, which charged those apex corners as though they
//! sat out there too, and the refinement chased a deflection it could never meet
//! until the packing floor stopped it.
//! `how_far_a_flat_piece_of_a_cone_strays` keeps each corner's own radius now,
//! and `rcad-mesh`'s `a_cone_patch_that_reaches_its_apex.rs` measures the mesh
//! that comes back against the cone's own closed-form distance.
//!
//! **So on this family nothing refuses at any rung**, and every ladder is
//! upward-closed. That is not the same as "a coarser deflection always meshes":
//! it is twenty solids over six rungs with no counter-example in them, which is
//! what §16.10 lets this file say and no more.
//!
//! [`the_floor_refusal_needs_an_absolute_deflection_and_then_it_behaves`] is
//! what keeps the silence readable: it reaches a refusal on purpose, so a sweep
//! that saw none is a sweep whose machinery can still report one. **It is not a
//! stand-in for the refusal that went away, and it should not be read as one.**
//! What it reaches is [`Error::DeflectionBelowWhatAMeshCanReach`], the up-front
//! guard in `room_a_mesh_has_to_work_in` that runs before a single triangle is
//! cut. What §51.7 reached was the packing floor inside `rcad-mesh`'s
//! `refine_the_inside`, at the far end of the run. Different refusal, different
//! code path; all the sentinel shows is that a refusal reaching this file is
//! named and counted rather than swallowed.
//!
//! Nothing else in the workspace reaches the packing floor either, now that the
//! cone bound no longer sends anything at it — counted, and written down on
//! [`CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection`] itself.
//!
//! [`CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection`]:
//!     rcad_mesh::CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection
//!
//! # What is recorded, and why it is not a boolean
//!
//! [`SolidView::mesh`](rcad_topo::SolidView::mesh) refuses from three places —
//! `room_a_mesh_has_to_work_in` before anything runs, the boundary walk in the
//! first pass, and `cut_up` in the second — and the *kind* of refusal decides
//! whether a coarser retry could ever have helped. A deflection under the floor
//! the arithmetic can place a point to is plainly about the deflection; a face
//! that goes right round its surface with no seam to cut it open is plainly not.
//! §15.10 is the rule that a pattern which moves is reported with the cases that
//! moved it, so every rung of every solid carries its refusal by name and the
//! table below is printed in full rather than summed into a rate.
//!
//! # The ladder is relative and the reason is §25.3
//!
//! [`Deflection`] is **a length in the model's own unit** — `Deflection::of`
//! takes one and `Deflection::RELATIVE_DEFAULT` is what a caller multiplies by
//! the model's nominal size to get one. A ball of radius `0.25` and a bar eleven
//! units long are not asked the same question by a deflection of `0.1`: for one
//! it is most of the shape and for the other it is a rounding. So each solid is
//! swept at its own size times the ladder, which is the same request the
//! consumer's "1e-3, 1e-2, 1e-1 relative" makes.
//!
//! # What this file may and may not conclude
//!
//! §16.10: a verdict is scoped to the family that was swept. What comes out of
//! here is a statement about **these** solids over **these** rungs, and
//! [`the_verdict_is_scoped_to_the_family_that_was_swept`] is what stops it being
//! written wider.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

mod cones;
mod rings;

/// The ladder, as a share of each solid's own size, finest first.
///
/// Four decades in six rungs. The consumer's three — 1e-3, 1e-2, 1e-1 — are all
/// on it, with one finer rung below them to reach the floor the arithmetic can
/// place a point to and two coarser ones above them to reach the other end,
/// where a deflection is a whole radius and there is nothing left to refine
/// towards.
const HOW_COARSE: [f64; 6] = [1e-4, 1e-3, 1e-2, 1e-1, 0.3, 1.0];

/// One solid to sweep, with the model it lives in and the size the ladder is
/// read against.
///
/// The size is what the shape was **built** with — a ball's radius, a bar's
/// length or width whichever is larger, the far end of a cone — and not
/// anything measured off a mesh. A ladder read against a measurement would move
/// when the measurement did.
struct Shape {
    what: String,
    model: Model,
    solid: SolidKey,
    size: f64,
}

/// Every solid this file sweeps.
///
/// Curved shapes and fused answers, because a wholly flat solid meshes at every
/// deflection and proves nothing — the two cubes are in only as the control that
/// says so. Each lives in its own [`Model`] so that the ball a long way from the
/// origin can have a working size to match without dragging the others out with
/// it.
fn the_family() -> Vec<Shape> {
    let mut family = Vec::new();
    let mut add = |what: String, model: Model, solid: SolidKey, size: f64| {
        family.push(Shape {
            what,
            model,
            solid,
            size,
        });
    };

    for side in [1.0, 4.0] {
        let mut model = a_model(side);
        let cube = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a whole-number cube");
        add(format!("cube of side {side}"), model, cube, side);
    }

    for radius in [0.25, 1.0, 3.0, 10.0] {
        let mut model = a_model(radius);
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball about the origin");
        add(format!("ball of radius {radius}"), model, ball, radius);
    }

    let mut model = a_model(1000.0);
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(1000.0, 0.0, 0.0)).expect("a finite placement"),
        )
        .expect("a ball a thousand units out");
    add(
        "ball of radius 1 a thousand units out".to_owned(),
        model,
        ball,
        1.0,
    );

    for (radius, height) in [
        (0.25f64, 0.5f64),
        (1.0, 0.5),
        (1.0, 5.0),
        (3.0, 0.5),
        (10.0, 5.0),
    ] {
        let mut model = a_model(radius.max(height));
        let bar = model
            .add_round_bar(radius, height, &Isometry::identity())
            .expect("a bar about the origin");
        add(
            format!("bar {radius} across and {height} tall"),
            model,
            bar,
            (2.0 * radius).max(height),
        );
    }

    for (slope, near, far) in [(0.5, 1.0, 3.0), (0.25, 2.0, 8.0), (2.0, 1.0, 2.0)] {
        let mut model = a_model(far);
        let frustum = cones::a_frustum(&mut model, slope, near, far);
        add(
            format!("frustum of slope {slope} from {near} to {far}"),
            model,
            frustum,
            far,
        );
    }

    for (slope, far) in [(0.5, 4.0), (1.0, 2.0)] {
        let mut model = a_model(far);
        let cone = cones::a_pointed_cone(&mut model, slope, far);
        add(
            format!("pointed cone of slope {slope} up to {far}"),
            model,
            cone,
            far,
        );
    }

    for (ring, tube) in [(4.0, 1.0), (10.0, 2.0)] {
        let mut model = a_model(ring + tube);
        let bend = rings::a_quarter_bend(&mut model, ring, tube);
        add(
            format!("quarter bend of ring {ring} and tube {tube}"),
            model,
            bend,
            ring + tube,
        );
    }

    let (model, solid, size) = a_block_with_a_ball_and_a_bar_united_into_it();
    add(
        "a block with a ball half sunk in it and a bar through it".to_owned(),
        model,
        solid,
        size,
    );

    family
}

fn a_model(size: f64) -> Model {
    Model::new(Scale::millimetres(100.0 * size).expect("a working size"))
}

/// §48's curved trio, united in one fuse, exactly as
/// `a_ball_and_a_bar_in_one_block.rs` builds it.
///
/// Eleven faces: six of the block, the ball's lower half, and the bar's wall and
/// cap at either end. It is the one fused answer here, and it is in because a
/// fuse's faces are trimmed by curves the builders never draw.
fn a_block_with_a_ball_and_a_bar_united_into_it() -> (Model, SolidKey, f64) {
    const SIDE: f64 = 7.0;
    const RADIUS: f64 = 1.0;
    let mut model = a_model(SIDE);
    let halfway = SIDE / 2.0;
    let block = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    let ball = model
        .add_ball(
            RADIUS,
            &Isometry::translation(Vector::new(2.0, halfway, 0.0)).expect("a shift"),
        )
        .expect("a whole-number ball");
    let bar = model
        .add_round_bar(
            RADIUS,
            SIDE + 4.0,
            &Isometry::translation(Vector::new(5.0, halfway, -2.0)).expect("a shift"),
        )
        .expect("a bar long enough to go right through");
    let united = model
        .fuse(
            &[block],
            &[ball, bar],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball and a bar that miss each other unite with a block");
    let solids = united.solids().to_vec();
    assert_eq!(solids.len(), 1, "one block with two lumps on it");
    (model, solids[0], SIDE + 4.0)
}

/// What one rung of the ladder came back with.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Rung {
    /// A mesh came back, with this many triangles.
    Meshed(usize),
    /// The mesher refused, and this is the refusal by name.
    Refused(String),
}

impl Rung {
    fn meshed(&self) -> bool {
        matches!(self, Self::Meshed(_))
    }

    fn how_it_reads(&self) -> String {
        match self {
            Self::Meshed(triangles) => format!("meshed ({triangles} triangles)"),
            Self::Refused(kind) => kind.clone(),
        }
    }
}

/// Mesh one solid at one deflection and say which of the two it was.
///
/// Under [`Policy::certified()`], because a tolerant policy answers a different
/// question: it takes a deflection under the floor and quietly meshes at the
/// floor instead, so the one refusal that is unambiguously about the deflection
/// would never appear.
fn how_a_mesh_answered(shape: &Shape, deflection: f64) -> Rung {
    let asked = Deflection::of(deflection).expect("a positive length");
    match shape.model.solid(shape.solid).mesh(
        asked,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(mesh) => Rung::Meshed(mesh.triangles().len()),
        Err(why) => Rung::Refused(what_it_refused(&why)),
    }
}

/// A refusal as a name that can be counted and compared.
///
/// [`Error`] is `#[non_exhaustive]`, so the wildcard is unavoidable — and every
/// variant [`SolidView::mesh`](rcad_topo::SolidView::mesh)'s own `# Errors`
/// section names is spelled out above it, so the wildcard only ever catches one
/// this file did not know could happen. Reaching it is a finding rather than a
/// pass, which is why it keeps the word `unexpected`.
fn what_it_refused(why: &Error) -> String {
    match why {
        Error::DeflectionBelowWhatAMeshCanReach { .. } => {
            "the deflection is under the floor".to_owned()
        }
        Error::MoreMeshPointsThanASolidWillHold { .. } => {
            "more points than a mesh holds".to_owned()
        }
        Error::CannotMeshAFace { why, .. } => {
            format!(
                "the triangulator refused a face: {}",
                what_the_triangulator_said(why)
            )
        }
        Error::Undecidable { .. } => "a predicate inside could not settle a sign".to_owned(),
        Error::AWireWillNotLieFlatOnItsFace { .. } => {
            "a wire will not lie flat on its face".to_owned()
        }
        Error::AWireDoesNotFanIntoALoopToMesh { .. } => {
            "a wire does not fan into a loop".to_owned()
        }
        Error::ACurvedFaceGoesRightRoundWithNoSeam { .. } => {
            "a curved face goes right round with no seam".to_owned()
        }
        Error::ACurvedFaceCannotBeCutSmallEnoughToMesh { .. } => {
            "a curved face cannot be cut small enough".to_owned()
        }
        Error::CurvedFaceWithAHoleGoesTooFarRound { .. } => {
            "a curved face with a hole goes too far round".to_owned()
        }
        Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked { .. } => {
            "an edge without two faces along it".to_owned()
        }
        Error::Geometry(_) => "the geometry layer refused".to_owned(),
        _ => "an unexpected refusal".to_owned(),
    }
}

/// Which of the triangulator's own refusals it was.
///
/// [`Error::CannotMeshAFace`] carries a
/// [`CannotTriangulate`](rcad_mesh::CannotTriangulate), and collapsing the seven
/// of those into one name would hide the whole point: one of them —
/// `BoundarySideStraysFurtherThanTheDeflection` — says the boundary the *caller*
/// walked is already too coarse for the deflection the caller asked for, which
/// is a different thing from a domain that cannot be filled at all.
fn what_the_triangulator_said(why: &rcad_mesh::CannotTriangulate) -> &'static str {
    use rcad_mesh::CannotTriangulate as No;
    match why {
        No::BoundaryCrossesItself { .. } => "the boundary crosses itself",
        No::CornersCoincide { .. } => "two corners are the same place",
        No::LoopsTouch { .. } => "two loops touch",
        No::CannotTellIfLoopsTouch { .. } => "nothing could prove whether two loops touch",
        No::CoordinateOutOfRange { .. } => "a scaled corner is out of range",
        No::BoundarySideStraysFurtherThanTheDeflection { .. } => {
            "a side of the boundary already strays further than the deflection"
        }
        No::NoMeshCanGetThatCloseToTheSurface { .. } => "no mesh can get that close to the surface",
        No::MoreTrianglesThanAPatchWillHold { .. } => "more triangles than a patch holds",
        No::RanOutOfRoomBeforeMeetingTheDeflection { .. } => {
            "the refinement ran out of room before meeting the deflection"
        }
        _ => "an unexpected refusal from the triangulator",
    }
}

/// The first rung that refused although a finer one had already meshed, if
/// there is one.
///
/// The ladder runs fine to coarse, so monotone means the meshes are an
/// **upward-closed** run: once a rung works, every rung above it works. What
/// breaks that is a refusal with a success anywhere below it, and this is the
/// index of the first such refusal.
fn a_rung_that_refused_above_one_that_worked(pattern: &[Rung]) -> Option<usize> {
    let mut worked_below = false;
    for (rung, answer) in pattern.iter().enumerate() {
        match answer {
            Rung::Meshed(_) => worked_below = true,
            Rung::Refused(_) if worked_below => return Some(rung),
            Rung::Refused(_) => {}
        }
    }
    None
}

/// Every solid whose ladder is **not** upward-closed, by name.
///
/// **None, since the cone bound was fixed.** The one entry that used to be here
/// was a pointed cone of slope one up to two, meshing at a ten-thousandth of its
/// size, refusing at a thousandth, and meshing again at a hundredth — the §51.7
/// case, and a gap in `rcad-mesh`'s cone bound rather than a fact about the
/// shape. The module doc says what it was and where the fix is.
///
/// Pinned as a list rather than as a count, because §15.10 says a number that
/// moves is reported with the cases that moved it: a solid joining this list
/// fails here, and so does the old entry coming back.
///
/// **This pin on its own does not catch every way the cone bound could break,
/// and here is one it misses.** Charge every corner the widest corner's radius
/// while leaving the rest of the arm alone, and both pointed cones refuse at
/// `1e-4` and mesh at every rung above it — which is upward-closed, so this list
/// stays empty and this test passes with the mesher broken. Measured, not
/// argued: the same mutation makes
/// [`the_stretch_the_pointed_cone_used_to_refuse_in_meshes_all_the_way_through`]
/// and [`which_refusals_never_turned_into_a_mesh_anywhere_on_the_ladder`] both
/// fail. The second of those runs on this very family over these very six
/// rungs, so it is not the twenty-one-rung ladder alone that carries the file —
/// what this list cannot see is a refusal with nothing finer than it, and
/// [`WHAT_EVERY_REFUSAL_HERE_LOOKED_LIKE`] sees that.
const THE_ONES_THAT_ARE_NOT_UPWARD_CLOSED: [&str; 0] = [];

/// The whole sweep, printed rung by rung and then judged.
///
/// Nothing here is asserted about *which* rungs work. What is asserted is that
/// enough of them worked for there to be a question at all, that no mesh came
/// back empty, and then that the ones whose ladder is not upward-closed are
/// exactly [`THE_ONES_THAT_ARE_NOT_UPWARD_CLOSED`] — which is now none of them.
///
/// **There is no longer an assertion that something refused here**, because
/// after the §51.7 fix nothing on this family does, at any of these rungs.
/// [`the_floor_refusal_needs_an_absolute_deflection_and_then_it_behaves`] is
/// what stands in its place, and it stands in for **part** of the job: it
/// reaches a refusal on purpose and shows it healing, so the machinery that
/// would report one is known to work. It does not reach the refusal this
/// assertion used to see — that was `rcad-mesh`'s packing floor at the end of
/// the refinement and this is the deflection floor before it starts. The module
/// doc says which is which.
#[test]
fn the_verdict_is_scoped_to_the_family_that_was_swept() {
    let family = the_family();
    let mut report = String::new();
    let mut not_monotone = Vec::new();
    let mut solids_that_ever_meshed = 0;
    let mut empty_meshes = Vec::new();

    for shape in &family {
        let pattern: Vec<Rung> = HOW_COARSE
            .iter()
            .map(|share| how_a_mesh_answered(shape, share * shape.size))
            .collect();

        let rungs: Vec<String> = HOW_COARSE
            .iter()
            .zip(&pattern)
            .map(|(share, answer)| format!("{share:>7} -> {}", answer.how_it_reads()))
            .collect();
        report.push_str(&format!(
            "{} (size {})\n    {}\n",
            shape.what,
            shape.size,
            rungs.join("\n    ")
        ));

        if pattern.iter().any(|answer| answer.meshed()) {
            solids_that_ever_meshed += 1;
        }
        for (share, answer) in HOW_COARSE.iter().zip(&pattern) {
            if matches!(answer, Rung::Meshed(0)) {
                empty_meshes.push(format!("{} at {share} of its size", shape.what));
            }
        }
        if let Some(rung) = a_rung_that_refused_above_one_that_worked(&pattern) {
            not_monotone.push(format!(
                "{}: {} at {} of its size, although {} of its size meshed",
                shape.what,
                pattern[rung].how_it_reads(),
                HOW_COARSE[rung],
                HOW_COARSE[..rung]
                    .iter()
                    .zip(&pattern)
                    .filter(|(_, answer)| answer.meshed())
                    .map(|(share, _)| share.to_string())
                    .collect::<Vec<_>>()
                    .join(" and ")
            ));
        }
    }

    println!(
        "{} solids over {} rungs, each rung a share of that solid's own size\n\n{report}",
        family.len(),
        HOW_COARSE.len()
    );

    assert!(
        empty_meshes.is_empty(),
        "a mesh with no triangles in it is not a mesh, and these came back as \
         successes: {empty_meshes:?}"
    );
    assert!(
        solids_that_ever_meshed >= family.len() / 2,
        "if hardly anything meshed there is no monotonicity to test: only \
         {solids_that_ever_meshed} of {} ever did",
        family.len()
    );
    let expected: Vec<String> = THE_ONES_THAT_ARE_NOT_UPWARD_CLOSED
        .iter()
        .map(|case| (*case).to_owned())
        .collect();
    assert_eq!(
        not_monotone, expected,
        "a coarser deflection does not always mesh, and this is the case list \
         of which solids it does not hold for"
    );
}

/// The check above would notice, which is the only reason its silence means
/// anything.
///
/// A sweep that finds no counter-example is worth exactly what the search that
/// failed to find one is worth, and a search that could not have found one is
/// worth nothing. So the pattern the real sweep is hunting for is written out by
/// hand here and handed to the same function.
#[test]
fn a_success_with_a_refusal_above_it_is_what_this_is_looking_for() {
    let refused = |kind: &str| Rung::Refused(kind.to_owned());
    let works_then_stops = [
        refused("the deflection is under the floor"),
        Rung::Meshed(120),
        Rung::Meshed(40),
        refused("a curved face goes right round with no seam"),
        Rung::Meshed(8),
        Rung::Meshed(8),
    ];
    assert_eq!(
        a_rung_that_refused_above_one_that_worked(&works_then_stops),
        Some(3),
        "a refusal at the fourth rung with two successes below it is the whole \
         shape of a non-monotone answer"
    );

    let refuses_then_works = [
        refused("the deflection is under the floor"),
        refused("the deflection is under the floor"),
        Rung::Meshed(600),
        Rung::Meshed(120),
        Rung::Meshed(40),
        Rung::Meshed(8),
    ];
    assert_eq!(
        a_rung_that_refused_above_one_that_worked(&refuses_then_works),
        None,
        "climbing to a rung that works and staying there is the monotone shape"
    );

    let nothing_ever_worked: Vec<Rung> = (0..6)
        .map(|_| refused("a wire will not lie flat on its face"))
        .collect();
    assert_eq!(
        a_rung_that_refused_above_one_that_worked(&nothing_ever_worked),
        None,
        "a solid that never meshes has no success for a refusal to sit above"
    );
}

/// The stretch that used to be ragged, at twenty-one deflections instead of the
/// three [`HOW_COARSE`] puts in it — and every one of them meshes now.
///
/// A pointed cone of slope one up to two, swept a tenth of a decade at a time
/// from a ten-thousandth of its size to a hundredth. What used to come out was
/// not a band with meshes on both sides of it but refusals and meshes
/// **interleaved** — six refusals, nine meshes finer than the coarsest of them,
/// and the answer changing six times going up — so no consumer could compute
/// "coarser than the last refusal" either.
///
/// That was `rcad-mesh`'s cone bound charging a triangle with two corners on the
/// apex for a quarter turn it does not take; the module doc has the mechanism
/// and `a_cone_patch_that_reaches_its_apex.rs` in `rcad-mesh` has the
/// measurement. What is left here is the same twenty-one deflections asserted to
/// all mesh, on the shape that was the counter-example, so the ragged edge
/// coming back fails loudly and at the deflections it came back at.
///
/// **What was established about that pattern, and what was not.** Running this
/// very test with `crates/rcad-mesh/src/triangulate.rs` put back to the commit
/// before the fix gives, rung by rung: meshes at `1e-4` through `3.162e-4`,
/// a refusal at `3.981e-4`, meshes at `5.012e-4` and `6.310e-4`, refusals at
/// `7.943e-4`, `1.000e-3` and `1.259e-3`, a mesh at `1.585e-3`, refusals at
/// `1.995e-3` and `2.512e-3`, then meshes from `3.162e-3` to `1e-2`. Six
/// refusals, nine meshes finer than the coarsest of them, the answer changing
/// six times going up. Why each individual refusal happened is settled: the
/// collapsed bound charged a ruling for a turn it does not take, and the
/// refinement fanned the apex until the packing floor declined the next point.
///
/// **Why that came out ragged rather than as one floor or one band was never
/// worked out, and nothing here should be read as saying it was.** A floor and
/// a band are both monotone and this was neither, so whatever decided which
/// rungs fell on which side was finer-grained than "the bound was too big" —
/// but no law was traced and none is claimed. It cannot be traced from this
/// tree any more either, since the pattern only exists under a bound the code
/// no longer computes. That is a fair place to leave it: the cause is removed,
/// and the assertion below is what would notice the pattern returning.
///
/// The triangle counts are printed and not asserted. A finer deflection giving
/// more triangles is what the mesher is for, but it is not a claim this file
/// establishes and §16.10 says an unmeasured claim does not go in a test.
#[test]
fn the_stretch_the_pointed_cone_used_to_refuse_in_meshes_all_the_way_through() {
    let mut model = a_model(2.0);
    let cone = cones::a_pointed_cone(&mut model, 1.0, 2.0);
    let shape = Shape {
        what: "pointed cone of slope 1 up to 2".to_owned(),
        model,
        solid: cone,
        size: 2.0,
    };

    let ladder: Vec<f64> = (0..21)
        .map(|step| 1e-4 * 10f64.powf(f64::from(step) / 10.0))
        .collect();
    let pattern: Vec<Rung> = ladder
        .iter()
        .map(|share| how_a_mesh_answered(&shape, share * shape.size))
        .collect();

    let refused: Vec<String> = ladder
        .iter()
        .zip(&pattern)
        .filter_map(|(share, answer)| match answer {
            Rung::Refused(kind) => Some(format!("{share:.6} -> {kind}")),
            Rung::Meshed(_) => None,
        })
        .collect();

    println!(
        "{:#?}",
        ladder
            .iter()
            .zip(&pattern)
            .map(|(share, answer)| format!("{share:.6} -> {}", answer.how_it_reads()))
            .collect::<Vec<_>>()
    );

    assert_eq!(
        refused,
        Vec::<String>::new(),
        "these six deflections used to refuse and the whole stretch meshes now, so \
         a refusal here is the §51.7 ragged edge coming back"
    );
}

/// The one refusal a relative ladder cannot reach, reached with an absolute
/// deflection instead — because otherwise this file would report it as absent
/// and mean two different things by that.
///
/// [`Error::DeflectionBelowWhatAMeshCanReach`] is the refusal that is
/// unambiguously about the deflection, and it never appeared in the sweep above.
/// That is not because the sweep could not provoke it: it is because the floor
/// is set by how far a point rounds where the solid sits, which for a ball one
/// unit across a thousand units out is around `1e-13` — eight decades below the
/// finest rung a consumer would ever call "relative". A file that reported it as
/// never happening, alongside a cone whose refusal is a real hole, would be
/// putting two very different absences under one word.
///
/// So: the same solid the family holds, asked for `1e-16` of a unit and then for
/// a ten-thousandth of its size. Refused, then meshed — the monotone shape, and
/// the contrast the cone is measured against.
///
/// **What this does not cover.** The refusal reached here is
/// [`Error::DeflectionBelowWhatAMeshCanReach`], which `room_a_mesh_has_to_work_in`
/// raises before any triangle is cut. The §51.7 refusal this file used to see
/// was `RanOutOfRoomBeforeMeetingTheDeflection`, raised by `rcad-mesh`'s
/// `refine_the_inside` when the packing floor leaves it nowhere to put the next
/// point — a different variant on a different code path at the other end of the
/// run. This test is not evidence about that one, and the sweep above no longer
/// reaches it.
#[test]
fn the_floor_refusal_needs_an_absolute_deflection_and_then_it_behaves() {
    let mut model = a_model(1000.0);
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(1000.0, 0.0, 0.0)).expect("a finite placement"),
        )
        .expect("a ball a thousand units out");
    let shape = Shape {
        what: "ball of radius 1 a thousand units out".to_owned(),
        model,
        solid: ball,
        size: 1.0,
    };

    assert_eq!(
        how_a_mesh_answered(&shape, 1e-16),
        Rung::Refused("the deflection is under the floor".to_owned()),
        "a ten-thousandth of a femtometre on a solid a thousand units out is \
         under what the arithmetic can place a point to"
    );
    assert!(
        how_a_mesh_answered(&shape, HOW_COARSE[0] * shape.size).meshed(),
        "the finest rung of the relative ladder is far above that floor, which \
         is why the sweep never saw this refusal"
    );
}

/// Every refusal kind that appeared, with whether the same solid ever meshed at
/// some other rung.
///
/// **None, since the §51.7 fix.** The one kind that used to be here was the
/// triangulator running out of room, and it healed at another rung of the same
/// solid. So this family reaches no refusal a consumer could be told not to
/// retry, and it reaches no refusal at all. That is a narrow answer and it is
/// the honest one — `ACurvedFaceGoesRightRoundWithNoSeam` and
/// `AnEdgeWithoutTwoFacesAlongItCannotBeWalked` plainly cannot care what
/// deflection they were asked at, but nothing here reaches either of them, and
/// §16.10 says a family that was not swept is not a family that was measured.
const WHAT_EVERY_REFUSAL_HERE_LOOKED_LIKE: [(&str, bool); 0] = [];

/// A refusal that comes back at every rung is one no consumer should retry, and
/// this measures which of them those are rather than reasoning about it.
///
/// The consumer in the lane brief climbs three rungs before it gives up. Every
/// climb costs a whole mesh attempt, so a refusal kind that never once turned
/// into a mesh anywhere on the ladder is a retry that provably did nothing on
/// this family. Pinned as [`WHAT_EVERY_REFUSAL_HERE_LOOKED_LIKE`] with the solid
/// counts printed beside it, per §15.10.
#[test]
fn which_refusals_never_turned_into_a_mesh_anywhere_on_the_ladder() {
    let family = the_family();
    let mut kinds: Vec<(String, Vec<String>, bool)> = Vec::new();

    for shape in &family {
        let pattern: Vec<Rung> = HOW_COARSE
            .iter()
            .map(|share| how_a_mesh_answered(shape, share * shape.size))
            .collect();
        let ever_meshed = pattern.iter().any(|answer| answer.meshed());
        for answer in &pattern {
            let Rung::Refused(kind) = answer else {
                continue;
            };
            match kinds.iter_mut().find(|(seen, _, _)| seen == kind.as_str()) {
                Some((_, from, healed)) => {
                    from.push(shape.what.clone());
                    *healed |= ever_meshed;
                }
                None => kinds.push((kind.clone(), vec![shape.what.clone()], ever_meshed)),
            }
        }
    }

    let mut report = String::new();
    for (kind, from, healed) in &kinds {
        report.push_str(&format!(
            "{kind}: {} rungs over {} solids, {}\n",
            from.len(),
            {
                let mut solids = from.clone();
                solids.sort_unstable();
                solids.dedup();
                solids.len()
            },
            if *healed {
                "some other rung of the same solid meshed"
            } else {
                "no rung of any of those solids ever meshed"
            }
        ));
    }
    println!("{report}");

    let what_happened: Vec<(String, bool)> = kinds
        .iter()
        .map(|(kind, _, healed)| (kind.clone(), *healed))
        .collect();
    let expected: Vec<(String, bool)> = WHAT_EVERY_REFUSAL_HERE_LOOKED_LIKE
        .iter()
        .map(|(kind, healed)| ((*kind).to_owned(), *healed))
        .collect();
    assert_eq!(
        what_happened, expected,
        "the refusal kinds this family reaches, and whether each one healed at \
         some other rung of the same solid"
    );
}

/// How wide the family is, pinned, because the verdict is scoped to it.
///
/// §16.10: "these solids over these rungs" is only a scope if the numbers in it
/// are held. A lane that quietly drops half the shapes would otherwise leave the
/// same sentence standing over a smaller sweep.
#[test]
fn the_family_is_the_size_the_verdict_says_it_is() {
    let family = the_family();
    assert_eq!(family.len(), 20, "the family the verdict is scoped to");
    let curved = family
        .iter()
        .filter(|shape| !shape.what.starts_with("cube"))
        .count();
    assert_eq!(curved, 18, "two cubes are the control and the rest bend");
    for shape in &family {
        assert!(
            shape.size.is_finite() && shape.size > 0.0 && HOW_COARSE[0] * shape.size > 0.0,
            "{} has no size to read a relative ladder against",
            shape.what
        );
    }
}
