//! A fuse answer that carries a curved surface as several faces, taken back in
//! by the next fuse.
//!
//! §223.4: `block ∪ frustum` leaves the frustum's cone as several faces, and the
//! fuse could not take that answer back in. Two gaps stand in the way, one
//! behind the other (§223.2):
//!
//! 1. **The arc's picker.** The first fuse lowered the picker of the block
//!    floor's arc just far enough for the corners *as it stored them* to read on
//!    or in front. The second fuse re-solves each corner into a box, the
//!    picker's plane ran through both boxes, and the second union refused with
//!    [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`]. The picker now
//!    clears each corner grown by the room a re-solved corner is allowed.
//! 2. **The cone pieces were never joined.** Behind the picker, the second union
//!    refused with [`rcad_topo::Error::AFaceMeetsItsNeighbourOnItsOwnSurface`]:
//!    nothing folded pieces of one curved surface into one face the way pieces
//!    of one plane are folded. The fuse now bakes pieces of one cone as one face
//!    (row Q898), and behind them is a third gap, the frustum's floor. Its
//!    longer arc's picker has the circle's centre in front, which `holds.rs`
//!    now reads (row Q971) and the fuse's own arrangement does not yet.
//!
//! # The risk the picker's room carries
//!
//! Lowering the picker further can take its plane past the centre of the circle
//! the arc is on. An arc is then no longer named by "both ends in front, centre
//! on or behind", and the next fuse refuses it. That happens for an arc within a
//! few `1e-14` of a semicircle, and it is built here: a block whose wall passes
//! `t` from the frustum's axis, for `t` down to `2⁻⁴⁸`.

mod cones;

use rcad_geom::{Deflection, EdgeRun, Isometry, Picker, Scale, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

use cones::a_frustum;

/// `(block ∪ frustum) ∪ cube`, with a cube that touches nothing, is the first
/// union and the cube.
///
/// It refused with [`Error::AFaceMeetsItsNeighbourOnItsOwnSurface`] while the
/// first union kept the frustum's cone as seven faces, cut apart by the block's
/// walls `x = 1` and `y = 1` where they run on below the block (§223.4). The
/// fuse now joins pieces of one cone the way it joins pieces of one plane
/// (§224.2), and behind that a wall's corner on the cone was taken to be on
/// both branches of the hyperbola the wall's plane cuts the cone in.
#[test]
fn a_block_and_frustum_united_can_be_united_again() {
    let (model, united) = a_block_and_frustum_united();
    the_union_again_with_a_far_cube(&model, &united)
        .expect("the answer of a fuse can be fused again");
}

/// `block ∪ frustum` carries the frustum's cone as one face, and it is sound.
///
/// The fuse cuts the cone apart where the block's walls `x = 1` and `y = 1` run
/// on below the block, and kept it as seven faces before pieces of one cone
/// were joined (§223.4, §224.2). Joined, the wall with its notch is a band
/// right round the axis, which needs a seam: one of the cut edges from the
/// floor circle up to the notch stays, as the seam.
///
/// The volume is the frustum's and the block's less what they share, worked out
/// by hand: the region `x > 1, y > 1` inside the circle of radius `z / 2`,
/// summed from `z = 5` to `z = 9`.
#[test]
fn a_block_and_frustum_united_keep_the_cone_as_one_face() {
    let (model, united) = a_block_and_frustum_united();
    let shared = simpson(5.0, 9.0, |z| a_quarter_disk_past_one(z / 2.0));
    let frustum = core::f64::consts::PI / 3.0 * 0.25 * (9.0_f64.powi(3) - 4.0_f64.powi(3));
    let block = 4.5 * 4.5 * 9.0;
    one_curved_face_sound_and_measured(&model, &united, frustum + block - shared);
}

/// `block ∪ bar`, the frustum's pair with a round bar of radius 2.5 in its
/// place, carries the bar's wall as one face, and it is sound.
///
/// The block's walls cross a pipe along its own lines, so the pieces of the
/// wall meet along straight edges rather than arcs of a hyperbola, and the seam
/// the joined band keeps is one of those.
#[test]
fn a_block_and_bar_united_keep_the_wall_as_one_face() {
    let (model, united) = a_block_and_bar_united();
    let bar = core::f64::consts::PI * 2.5 * 2.5 * 9.0;
    let block = 4.5 * 4.5 * 9.0;
    let shared = 4.0 * a_quarter_disk_past_one(2.5);
    one_curved_face_sound_and_measured(&model, &united, bar + block - shared);
}

/// `(block ∪ bar) ∪ cube`, with a cube that touches nothing, answers.
///
/// This used to refuse with [`Error::AFaceTrimmedByACurveThisCannotFollow`]
/// once the wall is one face. Q876's change reaches only a pipe wall cut
/// along the axis by two *parallel* planes, and this wall's two cuts,
/// `x = 1` and `y = 1`, are square to each other rather than parallel, so
/// Q876 is not what answers this: run on Q876's own base (`81d9526f`), the
/// test already passed. Un-ignored rather than traced further.
#[test]
fn a_block_and_bar_united_can_be_united_again() {
    let (model, united) = a_block_and_bar_united();
    the_union_again_with_a_far_cube(&model, &united)
        .expect("the answer of a fuse can be fused again");
}

/// One solid, one face on something other than a plane, `Model::validate`
/// passes, and the measured volume is within its own published error of
/// `exactly`.
fn one_curved_face_sound_and_measured(model: &Model, united: &[SolidKey], exactly: f64) {
    let [solid] = united else {
        panic!("the union is one solid, not {}", united.len());
    };
    let view = model.solid(*solid).expect("this model made the key");
    let curved = view
        .faces()
        .filter(|face| face.surface().as_plane().is_none())
        .count();
    assert_eq!(curved, 1, "the curved surface is one face");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    model
        .validate(&policy, &mut log)
        .expect("the union is sound");
    let measured = view
        .measure(Deflection::for_scale(model.scale()), &policy, &mut log)
        .expect("the union measures");
    assert!(
        (measured.volume() - exactly).abs() <= measured.volume_error(),
        "measured {} against {exactly}, allowed {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The area of `x > 1, y > 1` inside a circle of radius `r` about the origin,
/// for `r > √2`.
fn a_quarter_disk_past_one(r: f64) -> f64 {
    let far = (r * r - 1.0).sqrt();
    let under = |x: f64| x / 2.0 * (r * r - x * x).sqrt() + r * r / 2.0 * (x / r).asin();
    under(far) - under(1.0) - (far - 1.0)
}

/// Simpson's rule over 2000 steps, for a smooth `f` on `[from, to]`.
fn simpson(from: f64, to: f64, f: impl Fn(f64) -> f64) -> f64 {
    let steps = 2000;
    let width = (to - from) / f64::from(steps);
    let inner: f64 = (1..steps)
        .map(|step| {
            let weight = if step % 2 == 1 { 4.0 } else { 2.0 };
            weight * f(from + width * f64::from(step))
        })
        .sum();
    width / 3.0 * (f(from) + inner + f(to))
}

/// The frustum's pair with a round bar of radius 2.5 from `z = 0` to `z = 9`.
fn a_block_and_bar_united() -> (Model, Vec<SolidKey>) {
    let mut model = a_model();
    let bar = model
        .add_round_bar(2.5, 9.0, &Isometry::identity())
        .expect("a round bar");
    let block = a_box(
        &mut model,
        [[0.0, 0.0], [4.5, 0.0], [4.5, 4.5], [0.0, 4.5]],
        9.0,
        [1.0, 1.0, 5.0],
    );
    fused(&model, &[block], &[bar], Rule::Union).expect("a block and a bar through its floor unite")
}

/// §222.3 items 4 and 5's pair: a frustum about `z` and a block over its
/// narrow end, its low corner at `(1, 1, 5)`.
fn a_block_and_frustum_united() -> (Model, Vec<SolidKey>) {
    let mut model = a_model();
    let frustum = a_frustum(&mut model, 0.5, 4.0, 9.0);
    let block = a_box(
        &mut model,
        [[0.0, 0.0], [4.5, 0.0], [4.5, 4.5], [0.0, 4.5]],
        9.0,
        [1.0, 1.0, 5.0],
    );
    fused(&model, &[block], &[frustum], Rule::Union)
        .expect("a block and a frustum through its floor unite")
}

/// An arc of the block's floor that is nearly a semicircle keeps the circle's
/// centre behind its picker, however near a semicircle it is — down to `2⁻⁴⁸`
/// of the circle's radius 2.5, past which the first union no longer answers
/// for its own reasons. `t = 2⁻²` is left out because its wall meets the circle
/// at `(0, −2.5, 5)`, on the frustum's seam, and that refuses on its own.
///
/// **This is the risk the picker's room carried, and it was real.** Lowering
/// the picker against every corner grown by that room, with nothing else
/// changed, turned the centre to in front of it from `t = 2⁻⁴⁴` down: the room
/// is `64 ε` times the corner's size, about `7e-14` here, and the centre sits
/// `t` behind the chord. The same change also refused ten tests of half balls
/// whose arcs are exact semicircles. So the room is granted only while the
/// centre stays on or behind, and otherwise the picker is what the stored ends
/// alone earn, as before.
#[test]
fn a_near_semicircle_keeps_its_centre_behind_its_picker() {
    for exponent in [1, 10, 20, 30, 40, 42, 44, 45, 46, 47, 48] {
        let t = 2.0_f64.powi(-exponent);
        let (model, united) = a_frustum_united_with_a_wall_near_its_axis(t)
            .unwrap_or_else(|why| panic!("t = 2^-{exponent}: the first union refused: {why:?}"));
        let the_floors_arc = the_picker_facing_into_the_block(&model, &united, t);
        assert_eq!(
            the_floors_arc.side_of_point(PredicatePoint::from([0.0, 0.0, 5.0])),
            Decision::Neg,
            "t = 2^-{exponent}: the circle's centre is not behind the floor arc's picker"
        );
    }
}

/// With the centre kept behind, the arcs the room can be granted get past the
/// picker on the way back in, past the cone pieces and past `holds.rs`'s
/// reading of the frustum's floor, and stop where the fuse's arrangement reads
/// that floor's longer arc: the disc at `z = 4`, its circle cut in two where
/// the wall's plane crosses it.
///
/// `t = 2⁻³⁰` and `2⁻⁴⁰` refused at the block floor's picker with
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] before the room. `2⁻⁴⁴` and
/// `2⁻⁴⁵` still do: there the room would take the plane past the centre, so it
/// is not granted, and the re-solved corners straddle the picker as before.
///
/// **The cone pieces are passed in two steps** (row Q898). The first union keeps
/// the cone as five faces, cut apart by the wall's plane and by the block's
/// floor and top, and they were refused
/// [`Error::AFaceMeetsItsNeighbourOnItsOwnSurface`] until pieces of one cone
/// were baked as one face. Baked as one, they were refused `Undecidable` on
/// "order along a crossing" at every `t`: the corner where the wall's cut meets
/// the floor circle was read out of the model, exact, while the wall's plane,
/// still a crossing of the joined face, met that circle at a point solved a few
/// ulps away. The corner now keeps the wall's plane from the run of the edge
/// left out, is solved from the three, and is that meet.
///
/// **Then it stopped at `holds.rs`'s reading of the floor** (row Q971), and the
/// first union's floor refused on its own the same way, before any second fuse:
/// one of its two arcs is the longer, so its picker has the circle's centre in
/// front, and `holds.rs` read an arc only off a picker with the centre behind.
/// It now also reads one whose other arc's middle is behind
/// ([`the_floor_disc_a_wall_near_its_axis_cuts_in_two_holds_its_disc`]).
///
/// **What stops it now is the same refusal on the same face, from the fuse's
/// arrangement**: `which_way_an_edge_runs_round` in `fuse/arrange.rs` reads the
/// floor's arcs through its own `the_way_into_an_arc`, which asks only the two
/// ways with the centre behind. The answer this should become is the first
/// union and the cube, measured against the frustum's and the block's volumes
/// less what they share, worked out by hand, and the cube's 8: the wall's plane
/// is `x − y/10 = t`, so at each height from `z = 5` to `z = 8` the two share
/// the slice of the frustum's circle of radius `z / 2` past a line `t/√1.01`
/// from its centre.
#[test]
fn a_frustum_and_a_wall_near_its_axis_taken_back_in_stop_at_the_floor_discs_long_arc() {
    for exponent in [1, 10, 30, 40, 42] {
        let t = 2.0_f64.powi(-exponent);
        let (model, united) =
            a_frustum_united_with_a_wall_near_its_axis(t).expect("the first union answers");
        let floor = the_floor_disc(&model, &united);
        let refusal = the_union_again_with_a_far_cube(&model, &united)
            .expect_err("the fuse cannot read the floor disc's longer arc");
        assert!(
            matches!(refusal, Error::AFaceTrimmedByACurveThisCannotFollow { face } if face == floor),
            "t = 2^-{exponent}: the second union refused before reaching the floor disc: \
             {refusal:?}"
        );
    }
}

/// The first union's floor, the disc at `z = 4` whose circle the wall's plane
/// cuts in two, holds the points of its disc either side of that cut and none
/// past its circle of radius 2.
///
/// It used to hold nothing: [`Error::AFaceTrimmedByACurveThisCannotFollow`] at
/// every point. Of its two arcs the longer one is picked by a half-space
/// through the chord lowered just past both ends, so the circle's centre is in
/// front of it, and `holds.rs` read an arc only off a picker with the centre
/// behind. It now also reads one whose other arc's middle is behind.
#[test]
fn the_floor_disc_a_wall_near_its_axis_cuts_in_two_holds_its_disc() {
    for exponent in [1, 10, 30, 40, 42] {
        let t = 2.0_f64.powi(-exponent);
        let (model, united) =
            a_frustum_united_with_a_wall_near_its_axis(t).expect("the first union answers");
        let floor = the_floor_disc(&model, &united);
        let view = model.solid(united[0]).expect("this model made the key");
        for (at, wanted) in [
            ([0.0, 0.0, 4.0], OnAFace::Held),
            ([-1.5, 0.0, 4.0], OnAFace::Held),
            ([1.5, 0.0, 4.0], OnAFace::Held),
            ([-2.5, 0.0, 4.0], OnAFace::NotHeld),
            ([2.5, 0.0, 4.0], OnAFace::NotHeld),
            ([0.0, 2.5, 4.0], OnAFace::NotHeld),
        ] {
            let asked = view
                .face_holds_the_point(
                    floor,
                    PredicatePoint::from(at),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| {
                    panic!("t = 2^-{exponent}: the floor disc refused {at:?}: {why:?}")
                });
            assert_eq!(asked, wanted, "t = 2^-{exponent}: the floor disc at {at:?}");
        }
    }
}

/// The one face of the union on the plane `z = 4`, the frustum's floor.
fn the_floor_disc(model: &Model, solids: &[SolidKey]) -> FaceKey {
    let floors: Vec<FaceKey> = solids
        .iter()
        .flat_map(|&solid| model.solid(solid).expect("this model made the key").faces())
        .filter(|face| {
            face.surface()
                .as_plane()
                .is_some_and(|flat| flat.normal().z().abs() == 1.0 && flat.offset().abs() == 4.0)
        })
        .map(|face| face.key())
        .collect();
    let [floor] = floors.as_slice() else {
        panic!("the union has one floor at z = 4, not {}", floors.len());
    };
    *floor
}

/// A frustum about `z` and a block over `z ∈ [5, 8]` whose wall leans across
/// the axis and passes `t` from it at the floor, so the block's floor keeps an
/// arc of the cone's radius-2.5 circle that is `t` short of a semicircle.
///
/// The wall leans so its corners are not on the grid: a wall square to an axis
/// puts both ends of the arc exactly on the chord's own plane, and a picker
/// there is never lowered at all.
fn a_frustum_united_with_a_wall_near_its_axis(t: f64) -> Result<(Model, Vec<SolidKey>), Error> {
    let mut model = a_model();
    let frustum = a_frustum(&mut model, 0.5, 4.0, 9.0);
    let block = a_box(
        &mut model,
        [[t - 0.5, -5.0], [5.0, -5.0], [5.0, 5.0], [t + 0.5, 5.0]],
        3.0,
        [0.0, 0.0, 5.0],
    );
    fused(&model, &[block], &[frustum], Rule::Union)
}

/// The picker of the arc the block's floor keeps: the one at `z = 5` facing
/// into the block, towards `+x`.
fn the_picker_facing_into_the_block(
    model: &Model,
    solids: &[SolidKey],
    t: f64,
) -> rcad_geom::HalfSpace {
    let mut found = Vec::new();
    for &solid in solids {
        for face in model.solid(solid).expect("this model made the key").faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    let edge = coedge.edge();
                    let on_the_floor = [edge.start().point(), edge.end().point()]
                        .iter()
                        .all(|end| end.coordinates()[2] == 5.0);
                    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = edge.run() else {
                        continue;
                    };
                    for picker in &picked_by {
                        if let Picker::InFrontOf(half) = picker
                            && on_the_floor
                            && half.normal().x() > 0.0
                            && !found.iter().any(|(seen, _)| *seen == edge.key())
                        {
                            found.push((edge.key(), *half));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        found.len(),
        1,
        "t = {t:e}: one arc on the floor faces into the block"
    );
    found[0].1
}

fn the_union_again_with_a_far_cube(model: &Model, united: &[SolidKey]) -> Result<(), Error> {
    let mut model = model.clone();
    let far = a_box(
        &mut model,
        [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
        2.0,
        [20.0, 20.0, 20.0],
    );
    fused(&model, united, &[far], Rule::Union).map(|_| ())
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// A box: `outline` in the `xy` plane, `tall` high, moved by `at`.
fn a_box(model: &mut Model, outline: [[f64; 2]; 4], tall: f64, at: [f64; 3]) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("four corners that turn one way");
    model
        .add_extrusion(
            &outline,
            tall,
            &Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite shift"),
        )
        .expect("a box")
}

/// Fuse a copy of `model`. A certified answer that recorded a decision it could
/// not prove is a failure.
fn fused(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Result<(Model, Vec<SolidKey>), Error> {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let answer = model.fuse(arguments, tools, rule, &Policy::certified(), &mut log)?;
    assert!(
        log.is_empty(),
        "a certified {rule:?} recorded {} decisions it could not prove",
        log.len()
    );
    let solids = answer.solids().to_vec();
    Ok((model, solids))
}
