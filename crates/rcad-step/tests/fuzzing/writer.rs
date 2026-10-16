//! Arbitrary bytes into `rcad_step::write`, which must never panic.
//!
//! CLAUDE.md's rule that a caller's bytes may not panic, and the reader's own
//! fuzz target is why this one exists: that target found a real crash in its
//! first five minutes, and the writer had no target at all until this one.
//!
//! **`rcad_step::write` takes a `&Model`, not bytes, so this has to make one
//! first.** `Model` implements `serde`'s `Deserialize`
//! (`crates/rcad-topo/src/serialize.rs`) — that is precisely the route a
//! stranger's bytes take into a `Model`, whether from a saved file or from the
//! fuzzer — so arbitrary bytes go through `postcard::from_bytes` and whatever
//! comes out, valid or not, goes straight into the writer. A `Model` a fuzzer
//! built is under no obligation to be one `Model::validate` would let through,
//! and the writer owes an `Err` either way, never a crash.
//!
//! **A model that does come out is checked against what the door promised**,
//! not just handed on. `Deserialize` refuses a corner that is at no point and a
//! slop that is not a length (`docs/decisions.md` §44.5), and walking past a
//! model whose corner is `NaN` would leave that promise untested by the thing
//! best placed to break it.
//!
//! **What the corpus actually reaches is the reason [`Reached`] counts faces.**
//! Sixteen of 1180 checked-in seeds decoded at all and every one of the sixteen
//! was empty — at most one solid, no faces, no vertices — so the door check
//! walked three empty iterators 1180 times and the writer never got past
//! `TheModelHasNoSolid` or `ASolidWithNoShell`. `two_cubes_that_write` and
//! `a_ball_of_radius_two` are the two seeds that fix that. They were one
//! either side; since Q730 the ball is written too.
//!
//! **A file that was written is read back and compared with the model that went
//! in** (`docs/decisions.md` §129.5 item 3). Not panicking was all this target
//! asked before, and the writer is the highest-throughput target there is. The
//! written text parses, every reference in it points somewhere, `Import::read`
//! turns it back into a model that validates, and the same number of solids
//! comes back — the same door `round_trip_one` in
//! `a_real_file_written_out_and_read_back_is_the_same_solid.rs` checks. That
//! test's last leg, a `Rule::Difference` against a third solid, is left out: it
//! is a fuse, and a fuse that panics on a sound model here would be a fuse
//! defect reported by the writer's target.
//!
//! **What is compared, and why it stops short of the measurement for some
//! solids** (`docs/decisions.md` §249.6, amending §129.5 item 3). Every
//! solid's located numbers are checked bit for bit: each face's surface, and
//! for every coedge its two corners and its edge's run, in walk order. These
//! are the numbers the writer writes and the reader reads, and §249.2 found
//! every one of them come back the same bits for a turned round bar. A cone's
//! slope is the one exception: the file carries its half-angle, no angle reads
//! back as most slopes, and the slope is compared as a measurement (§366). A
//! whole ball turned off `z` is the other: it is written cut open on the `z`
//! line, the only line this kernel's ball pinches on, and compared by its
//! surface, its poles and its seam (§404).
//! Volume, surface area and centroid are then also checked bit for bit, but
//! only for a solid whose every number came back the same bits and that sits
//! wholly on planes or carries no placement at all. A
//! curved solid placed with a turn is meshed in the frame it is stored in on
//! the way out and in the frame the reader baked it into on the way back — two
//! different roundings of the same true surface — so its measurement can come
//! back a few ulps off with nothing wrong in either the writer or the reader
//! (§249.5, a limit, not a defect). `two_turns_that_write_the_same_placement.rs`
//! pins that limit down: two turns that turn the three axes to the same nine
//! numbers, so that STEP would write them as one placement, still leave a bar
//! measuring differently.
//!
//! What the source explains is counted and not failed. A model that does not
//! pass `Model::validate` was never a solid, so a file of it owes nothing on
//! the way back; a solid that does not measure has no answer to compare.
//! Everything else that differs panics, because it is a sound model this
//! writer and this reader disagree about.
//!
//! **A sound model is one whose every solid also passes
//! `Model::validate_solid`**, because that is the question the reader asks.
//! It builds each solid through `SolidBuilder::build`, which judges the solid
//! in a model of its own. `validate` can pass a model whose solid is refused
//! alone: a corner may sit off its plane by up to about `1e-9` times the
//! largest coordinate, and a far neighbour makes that room wider
//! (`docs/decisions.md` §242.6). Twelve of the 2026-09-26 crash files were
//! that, and nothing else (§326).
//!
//! **Both of the kernel's checks size that room where the file puts the
//! solid, or where it is stored if that gives less** (§363). The writer writes
//! each solid where its placement puts it and the reader sizes the room from
//! there, so a solid moved toward the origin gets less room from the reader
//! than from where it is stored. Two more of those crash files were that, one
//! on a flat face and one on a curved one (§326.3, §327), and the harness
//! counts them here too.

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

// `rcad-topo` owns the door, so the check on what it promised lives there and
// this reaches across to it. Both this file and `fuse.rs` carried a copy of it
// before, which is two places to update and one to forget.
#[path = "../../../rcad-topo/tests/fuzzing/the_door.rs"]
mod the_door;

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane,
    Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{CoedgeView, DraftStep, Model, SolidBuilder, SolidKey, SolidView, WireView};

use the_door::every_number_the_door_promised_is_real;

/// How far one draw of bytes got.
#[derive(Clone, Copy, Debug, Default)]
pub struct Reached {
    /// Byte strings handed to the decoder.
    pub draws: usize,
    /// Byte strings that decoded into a model.
    pub decoded: usize,
    /// Faces across every model that decoded.
    pub faces: usize,
    /// Models this writer wrote out as a Part 21 file.
    pub written: usize,
    /// Models this writer refused by name.
    pub refused: usize,
    /// Written models that do not pass `Model::validate`, so there is no solid
    /// to compare the file against.
    pub written_but_unsound: usize,
    /// Written models that pass `Model::validate` but hold a solid that
    /// `Model::validate_solid` refuses. The reader builds each solid through
    /// `SolidBuilder::build`, which judges it alone, and a neighbour can widen
    /// the room `validate` gives a corner (`docs/decisions.md` §242.6). A
    /// solid moved toward the origin is refused here too once its room, sized
    /// where it sits, is too small for it (§363).
    pub written_but_a_solid_unsound_alone: usize,
    /// Written files the reader turned back into a model.
    pub read_back: usize,
    /// Read-back models not compared because a solid of the model that went in
    /// does not measure.
    pub did_not_measure: usize,
    /// Solids whose located numbers were compared bit for bit with the solid
    /// that went in, and matched — plus, for a solid wholly flat or at no
    /// placement, whose volume, surface area and centroid also matched bit
    /// for bit (§249.6).
    pub solids_the_same: usize,
    /// Models written, read back, and the same in every solid.
    pub came_back_the_same: usize,
}

impl Reached {
    /// What two draws reached between them.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self {
            draws: self.draws + other.draws,
            decoded: self.decoded + other.decoded,
            faces: self.faces + other.faces,
            written: self.written + other.written,
            refused: self.refused + other.refused,
            written_but_unsound: self.written_but_unsound + other.written_but_unsound,
            written_but_a_solid_unsound_alone: self.written_but_a_solid_unsound_alone
                + other.written_but_a_solid_unsound_alone,
            read_back: self.read_back + other.read_back,
            did_not_measure: self.did_not_measure + other.did_not_measure,
            solids_the_same: self.solids_the_same + other.solids_the_same,
            came_back_the_same: self.came_back_the_same + other.came_back_the_same,
        }
    }
}

/// **One draw of bytes, decoded the way a stranger's saved file is, written out,
/// and read back.**
///
/// The fuzz target and the suite's replay both call this and nothing narrower.
///
/// # Panics
///
/// If a model came through `Deserialize` carrying a number the door promised it
/// would refuse, or if a sound model was written as a file that does not come
/// back as the same model.
#[must_use]
pub fn one_model_decoded_and_written(bytes: &[u8]) -> Reached {
    let mut reached = Reached {
        draws: 1,
        ..Reached::default()
    };
    let Ok(model) = postcard::from_bytes::<Model>(bytes) else {
        return reached;
    };
    reached.decoded = 1;
    reached.faces = model.faces().count();
    every_number_the_door_promised_is_real(&model);
    match rcad_step::write(&model, "fuzz") {
        Ok(text) => {
            reached.written = 1;
            reached = reached.and(read_back_and_compared(&model, &text));
        }
        Err(_) => reached.refused = 1,
    }
    reached
}

/// Read `text` back and compare it with `source`, the model it was written
/// from — `round_trip_one`'s checks, in
/// `a_real_file_written_out_and_read_back_is_the_same_solid.rs`.
fn read_back_and_compared(source: &Model, text: &str) -> Reached {
    let mut reached = Reached::default();
    let file = StepFile::read(text.as_bytes()).unwrap_or_else(|why| {
        panic!("this writer wrote a file its own reader cannot parse: {why}")
    });
    let broken = file.references_that_point_nowhere();
    assert!(
        broken.is_empty(),
        "this writer wrote {} reference(s) to an entity it never wrote",
        broken.len()
    );

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    if source.validate(&policy, &mut log).is_err() {
        reached.written_but_unsound = 1;
        return reached;
    }
    if source.solids().any(|solid| {
        source
            .validate_solid(solid.key(), &policy, &mut log)
            .is_err()
    }) {
        reached.written_but_a_solid_unsound_alone = 1;
        return reached;
    }
    let import = Import::read(&file, &policy, &mut log).unwrap_or_else(|why| {
        panic!("a sound model was written as a file the reader refuses: {why}")
    });
    let there = import.model();
    reached.read_back = 1;
    if let Err(why) = there.validate(&policy, &mut log) {
        panic!("a sound model came back from its own file as one that is not: {why}");
    }

    let source_solids: Vec<SolidKey> = source.solids().map(|solid| solid.key()).collect();
    let there_solids: Vec<SolidKey> = there.solids().map(|solid| solid.key()).collect();
    assert_eq!(
        source_solids.len(),
        there_solids.len(),
        "a sound model's solids changed in number on the way through its own file"
    );

    // The model that went in carries whatever scale its bytes named, and the one
    // read back carries the scale of its own vertices (`Import`). The mesh is
    // taken from the second so that a drawn scale of 1e-300 cannot ask for a
    // mesh finer than the solid, and it is the same mesh on both sides.
    let deflection = Deflection::for_scale(there.scale());
    for (&here_key, &back_key) in source_solids.iter().zip(&there_solids) {
        let here_solid = source.solid(here_key).expect("this model made the key");
        let back_solid = there.solid(back_key).expect("this model made the key");
        let every_number_the_same_bits = assert_located_numbers_the_same(here_solid, back_solid);

        let Ok(here) = here_solid.measure(deflection, &policy, &mut log) else {
            reached.did_not_measure = 1;
            return reached;
        };
        let back = back_solid
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| {
                panic!("a solid that measured does not measure after its own file: {why}")
            });
        if every_number_the_same_bits
            && (here_solid.is_wholly_flat() || solid_has_no_placement(here_solid))
        {
            assert!(
                here.volume() == back.volume() && here.surface_area() == back.surface_area(),
                "a sound solid came back from its own file with a different volume or area: \
                 {here:?} went in, {back:?} came back"
            );
            assert!(
                here.centroid() == back.centroid(),
                "a sound solid came back from its own file with a different centroid: \
                 {here:?} went in, {back:?} came back"
            );
        }
        reached.solids_the_same += 1;
    }
    reached.came_back_the_same = 1;
    reached
}

/// Every located number of `here` and `back`, walked the way §249.1 walked it
/// by hand: each face's surface, and for every coedge its two corners and its
/// edge's run, in walk order.
///
/// Compared as `{:?}` text rather than with `==`, because derived float
/// equality treats `0.0` and `-0.0` as one number and this has to mean the
/// same bits (§249.1). A mismatch here is never a limit but in one number:
/// the writer writes every other one unchanged and the reader parses it back
/// exactly (§249.2), so a difference is a defect in one of the two, not a
/// frame the file cannot name.
///
/// **That one number is a cone's slope.** STEP writes a cone's half-angle and
/// the reader takes its tangent, and for most slopes over one no angle's
/// tangent is the slope (`docs/decisions.md` §366). So a cone's apex and axis
/// are compared bit for bit and its slope as a measurement — see
/// [`a_slope_comes_back_as_near_as_a_half_angle_carries_it`] — and this
/// answers whether every number came back the same bits, which is what lets
/// [`read_back_and_compared`] compare the solid's measurement bit for bit too.
///
/// **A drawn circle's pipe can come back with another origin and not be a
/// mismatch.** [`the_same_run`] calls it the same pipe where its origin is on
/// the kernel's line or where the kernel's walk centres the circle, and that
/// does not clear the answer, so an unplaced solid with such a pipe is still
/// measured bit for bit. Whether a pipe that leans and starts off its flat
/// face can then measure an ulp off has not been run.
///
/// A plane is the one surface whose numbers are not written as they are
/// stored. The writer picks the point it writes so that the reader's offset
/// comes back the same bits (§313). The normal it writes as stored, and the
/// reader keeps it when it is as near length one as a turn leaves a normal
/// (§355). The plane a ball's seam is cut from comes back the same way: the
/// reader reads it off the circle's placement, and the writer places the
/// circle at a centre that plane reads back from (Q822).
///
/// **A whole ball the writer put back on the world's chart is the one face
/// not compared corner by corner.** Two kinds end up there
/// (`docs/decisions.md` §404): a turn that moved the ball's poles off the `z`
/// line it pinches on, and a turn about `z` itself whose own seam no centre
/// the reader holds can carry, in the bands
/// [`a_whole_ball_whose_seam_was_rewritten`] finds by comparing seams rather
/// than by the poles. Either way [`assert_cut_again_on_the_z_line`] compares
/// the face by its surface, its poles and its seam; its solid is not then
/// measured bit for bit, which a turned solid is not anyway.
///
/// # Panics
///
/// If a face, a coedge, a corner or a run differs between `here` and `back`.
fn assert_located_numbers_the_same(here: SolidView, back: SolidView) -> bool {
    let mut every_number_the_same_bits = true;
    let here_faces: Vec<_> = here.faces().collect();
    let back_faces: Vec<_> = back.faces().collect();
    assert_eq!(
        here_faces.len(),
        back_faces.len(),
        "a sound solid came back from its own file with a different number of faces"
    );
    for (here_face, back_face) in here_faces.iter().zip(&back_faces) {
        let (here_surface, back_surface) = (here_face.surface(), back_face.surface());
        if let (Surface::Cone(here_cone), Surface::Cone(back_cone)) = (here_surface, back_surface) {
            assert_eq!(
                format!("{:?}", here_cone.axis()),
                format!("{:?}", back_cone.axis()),
                "a sound solid's face came back from its own file with a different surface"
            );
            assert!(
                a_slope_comes_back_as_near_as_a_half_angle_carries_it(
                    here_cone.slope(),
                    back_cone.slope()
                ),
                "a sound solid's cone came back from its own file with a slope further off \
                 than a half-angle carries: {} went in, {} came back",
                here_cone.slope(),
                back_cone.slope()
            );
            every_number_the_same_bits &=
                here_cone.slope().to_bits() == back_cone.slope().to_bits();
        } else {
            assert_eq!(
                format!("{here_surface:?}"),
                format!("{back_surface:?}"),
                "a sound solid's face came back from its own file with a different surface"
            );
        }
        if let Some(ball) = a_whole_ball_turned_off_z(here_face)
            .or_else(|| a_whole_ball_whose_seam_was_rewritten(here_face, back_face))
        {
            assert_cut_again_on_the_z_line(ball, back_face);
            every_number_the_same_bits = false;
            continue;
        }
        let here_coedges: Vec<_> = here_face.wires().flat_map(WireView::coedges).collect();
        let back_coedges: Vec<_> = back_face.wires().flat_map(WireView::coedges).collect();
        assert_eq!(
            here_coedges.len(),
            back_coedges.len(),
            "a sound solid's face came back from its own file with a different number of coedges"
        );
        for (here_coedge, back_coedge) in here_coedges.iter().zip(&back_coedges) {
            assert_eq!(
                format!("{:?}", here_coedge.start().point()),
                format!("{:?}", back_coedge.start().point()),
                "a sound solid's coedge came back from its own file starting at a different corner"
            );
            assert_eq!(
                format!("{:?}", here_coedge.end().point()),
                format!("{:?}", back_coedge.end().point()),
                "a sound solid's coedge came back from its own file ending at a different corner"
            );
            let (here_run, back_run) = (here_coedge.edge().run(), back_coedge.edge().run());
            let corners = [here_coedge.start().point(), here_coedge.end().point()];
            assert!(
                the_same_run(&here_run, &back_run, corners, here_surface),
                "a sound solid's coedge came back from its own file with a different edge run: \
                 {here_run:?} went in, {back_run:?} came back"
            );
        }
    }
    every_number_the_same_bits
}

/// The ball `face` covers whole, where a turn moved a pole of it off the `z`
/// line its located ball pinches on, or `None`.
fn a_whole_ball_turned_off_z(face: &rcad_topo::FaceView<'_>) -> Option<rcad_geom::Sphere> {
    let Surface::Sphere(ball) = face.surface() else {
        return None;
    };
    let mut wires = face.wires();
    let (Some(wire), None) = (wires.next(), wires.next()) else {
        return None;
    };
    let poles = the_poles_of_a_whole_ball(wire)?;
    poles
        .iter()
        .any(|pole| {
            face.surface()
                .the_way_its_first_coordinate_runs_at(*pole)
                .is_some()
        })
        .then_some(ball)
}

/// The ball `here_face` covers whole, where its poles stand where the ball
/// pinches (so [`a_whole_ball_turned_off_z`] does not catch it) but its own
/// seam came back from the file as a different run than the one it went in
/// with, or `None`.
///
/// A turn about `z` leaves a ball's poles on the line it pinches on — only a
/// turn about some other axis moves them off it. For most such turns the
/// writer carries the seam as it is, and the seam that comes back is still
/// compared corner by corner. For a turn in the bands
/// `docs/decisions.md` §404 names, no centre the reader holds can carry that
/// seam's own plane, and the writer puts the whole ball back on the world's
/// chart instead, with its seam wherever `add_ball` cuts that
/// (`rcad_step::write`'s `a_whole_ball_written_on_its_chart`). Comparing the
/// two seams by [`the_same_run`] is how this tells the two apart without
/// asking the writer which one it did (Q1319).
fn a_whole_ball_whose_seam_was_rewritten(
    here_face: &rcad_topo::FaceView<'_>,
    back_face: &rcad_topo::FaceView<'_>,
) -> Option<rcad_geom::Sphere> {
    let Surface::Sphere(ball) = here_face.surface() else {
        return None;
    };
    let here_seam = the_seam_of_a_whole_ball(here_face)?;
    let back_seam = the_seam_of_a_whole_ball(back_face)?;
    let corners = [here_seam.start().point(), here_seam.end().point()];
    let same = the_same_run(
        &here_seam.edge().run(),
        &back_seam.edge().run(),
        corners,
        here_face.surface(),
    );
    (!same).then_some(ball)
}

/// The two poles of a wire that walks one edge there and back with a pole at
/// each turn, in walk order, or `None` for any other wire.
fn the_poles_of_a_whole_ball(wire: WireView) -> Option<[Point; 2]> {
    the_seam_and_poles_of_a_whole_ball(wire).map(|(_, poles)| poles)
}

/// The seam of the one wire of `face`, the way [`the_seam_and_poles_of_a_whole_ball`]
/// finds it, or `None` where `face` is not one wire walking one edge there and
/// back with a pole at each turn.
fn the_seam_of_a_whole_ball<'m>(face: &rcad_topo::FaceView<'m>) -> Option<CoedgeView<'m>> {
    let mut wires = face.wires();
    let (Some(wire), None) = (wires.next(), wires.next()) else {
        return None;
    };
    the_seam_and_poles_of_a_whole_ball(wire).map(|(seam, _)| seam)
}

/// The seam of a wire that walks one edge there and back with a pole at each
/// turn, as the coedge walking it the first time, and the two poles in walk
/// order, or `None` for any other wire.
fn the_seam_and_poles_of_a_whole_ball(wire: WireView<'_>) -> Option<(CoedgeView<'_>, [Point; 2])> {
    use rcad_topo::WireStepView::{Along, Pole};
    let walk: Vec<_> = wire.steps().collect();
    let [first, second, third, fourth] = walk[..] else {
        return None;
    };
    let ((Along(there), Pole(one), Along(back), Pole(other))
    | (Pole(one), Along(there), Pole(other), Along(back))) = (first, second, third, fourth)
    else {
        return None;
    };
    (there.edge().key() == back.edge().key()).then(|| (there, [one.point(), other.point()]))
}

/// That `back` is the whole of `ball` cut open again on the `z` line, the way
/// the writer writes a whole ball turned off it (`docs/decisions.md` §404):
/// its poles at `centre ± (0, 0, r)` to the bit, and its seam the run
/// `add_ball` cuts between them, in the plane through the centre square to `y`
/// on the side facing `+x`. Its surface was compared before this.
///
/// **This is the one face compared by its poles and not corner by corner.**
/// This kernel's ball pinches only on the `z` line, so the poles the turn
/// moved off it have no way into a file and back; the ball they bound does.
///
/// # Panics
///
/// If `back` is not that face.
fn assert_cut_again_on_the_z_line(ball: rcad_geom::Sphere, back: &rcad_topo::FaceView<'_>) {
    let mut wires = back.wires();
    let (Some(wire), None) = (wires.next(), wires.next()) else {
        panic!("a sound solid's whole ball came back from its own file with other than one wire");
    };
    let poles = the_poles_of_a_whole_ball(wire).unwrap_or_else(|| {
        panic!("a sound solid's whole ball came back from its own file cut some other way")
    });
    let centre = ball.centre();
    let [x, y, z] = centre.coordinates();
    let [north, south] = [z + ball.radius(), z - ball.radius()].map(|at| Point::new(x, y, at));
    let came_back = format!("{poles:?}");
    assert!(
        came_back == format!("{:?}", [north, south])
            || came_back == format!("{:?}", [south, north]),
        "a sound solid's whole ball came back from its own file with its poles at {came_back}, \
         not at {north:?} and {south:?}"
    );
    let seam = wire
        .coedges()
        .next()
        .expect("a wire with two steps along its edge");
    let cut = Plane::from_point_and_normal(centre, Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let facing_x = HalfSpace::through_a_point_facing(centre, Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis has length");
    let add_balls_seam = EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cut),
        picked_by: [Picker::from(facing_x)].into_iter().collect(),
    };
    let back_run = seam.edge().run();
    assert!(
        the_same_run(
            &add_balls_seam,
            &back_run,
            [seam.start().point(), seam.end().point()],
            Surface::Sphere(ball)
        ),
        "a sound solid's whole ball came back from its own file with a different edge run: \
         {add_balls_seam:?} was written, {back_run:?} came back"
    );
}

/// Whether a cone's slope came back as near as a written half-angle can carry
/// it: within what the tangents of the two `f64` angles either side of
/// `atan(here)` differ by.
///
/// STEP writes a cone's half-angle and the reader takes its tangent, so a
/// slope comes back exactly only where some angle's tangent is that slope. For
/// most slopes over one none is, and slopes one and two of the four
/// [`a_slope`] draws are among them (`docs/decisions.md` §366). So the slope
/// is compared as a measurement, the way §249.6 compares one. The writer
/// writes the angle whose tangent comes nearest, and
/// `most_slopes_over_one_come_back_from_no_half_angle` in
/// `crates/rcad-step/src/write.rs` finds it within a third of this reach at
/// worst over a hundred thousand slopes from `1e-3` to `1e3`.
fn a_slope_comes_back_as_near_as_a_half_angle_carries_it(here: f64, back: f64) -> bool {
    let angle = here.atan();
    let reach = (angle.next_up().tan() - angle.next_down().tan()).abs();
    (back - here).abs() <= reach
}

/// Whether two runs are the same, which is the same bits except for a flat
/// picker of an arc: that one is the same when it faces the same way.
///
/// **A picker is the one number of a run no file carries.** STEP says which arc
/// of a circle an edge is by the way the circle turns and the edge's two ends,
/// and the reader builds its own half-space from those — facing
/// `(B − A) × w`, not scaled to length one, and let down by a rounding so both
/// ends are in front of it (`Walk::the_arc_between`). The kernel's own ball
/// faces `(1, 0, 0)`, so a ball's seam comes back facing `(2r, 0, 0)`: the same
/// half-space, spelled another way. What decides which arc is picked is which
/// way the half-space faces, since both stand on the chord between the two
/// ends, and those ends and the surface `and` names are compared bit for bit.
/// So a flat picker is compared by facing the same way, and everything else in
/// a run as the same bits.
///
/// **Where `and` is a plane, the arc lies in it**, and facing the same way is
/// facing the same side of the chord within that plane: along
/// `normal × chord`, both picker normals dotted with it the same sign and
/// neither zero. A ball turned by any angle needs that. Its picker is
/// `(1, 0, 0)` turned and the reader's is `(B − A) × w`, a multiple of the seam
/// plane's normal turned a quarter, and the two are not along each other in
/// exact arithmetic unless the diameter is a power of two (Q822,
/// `docs/decisions.md` §366). The sign is read in plain `f64`, as the writer
/// reads its own (`Writing::a_seam_of_one_ball`): a picker standing on the
/// chord faces along that line or against it, never square to it.
///
/// **Where the face walking the edge is flat, the arc lies in the face's plane
/// instead**, and is told the same way. A circle drawn on a flat cap is named
/// by the pipe of its radius (`Writing::an_arc_of_a_circle_on_a_flat_face`),
/// and turned about `z` by `0.1` and moved, the kernel's picker was
/// `(0.09983341664682815, −0.9950041652780258, 0)` and the reader's
/// `(0.19966683329365686, −1.9900083305560514, 0)`, the same side of the
/// chord and not along each other to the bit
/// (`how_often_a_turned_disc_drawn_on_a_bar_comes_back_from_its_own_file`).
/// An edge is walked from both its faces, so one with a curved face on its
/// other side is still held to the test below from that side; only an edge
/// between two flat faces is told by its side of the chord alone.
/// Beside any other surface the arc's plane is not to hand, so the pickers
/// must be along each other — the cross product exactly zero — and not
/// opposite.
///
/// **A pipe picker is no file carries either, and is told by what it picks,
/// not by its numbers.** `InsideAPipe` and `OutsideAPipe` each hold a
/// [`Cylinder`] the reader rebuilds from the ring's own axis and radius
/// (`which_of_a_rings_two_circles`, `crates/rcad-step/src/brep.rs`), and that
/// rebuild can carry a subnormal the kernel's own axis never had — turning
/// `(0, 0, 1)` into `(1.5e-322, 8e-323, 1)`, which is no longer along the
/// kernel's axis to the bit. So two pickers of the same arm are compared by
/// what they pick: both must place `corners`, the coedge's own two ends, on
/// the kept side — `Decision::Pos` — of both the kernel's picker and the
/// read-back one. A picker that gets the actual ends of the actual edge wrong
/// is not the same picker whatever its numbers say, and one that gets them
/// right is the one this run needs, numbers or no.
///
/// **Where `and` is a pipe, its origin travels and its line does not.** The
/// writer moves a drawn circle's pipe to where its axis meets the flat face
/// it is drawn on, when the pipe's own origin is not already there
/// (`Writing::an_arc_of_a_circle_on_a_flat_face`), keeping the axis direction
/// and the radius exactly as the kernel wrote them — checked before the file
/// is written, by asking the same constructors the reader will. So two `and`
/// pipes are the same pipe when their directions and radii match to the bit
/// and the origin read back is one of two places. Either it sits on the
/// kernel's own line — the origin delta's cross product with the direction is
/// exactly zero, not a point-for-point equality, since a cylinder's own axis
/// origin can sit anywhere along its line. Or, where the arc's plane is to
/// hand, it is to the bit where `rcad_geom::walk` centres the circle that
/// plane crosses the kernel's pipe in ([`where_the_walk_centres_a_circle`]),
/// which is the point the writer works out and writes.
///
/// **Under a turn and a move together, that point rounds off the kernel's
/// line** — a division and a multiply, each rounded once more than the
/// kernel's own origin was — and no other centre would do: for the disc
/// turned about `y` by `5` and moved by `(3, 5, 10)`, no point an `f64` holds
/// lies exactly on the kernel's line where the top holds it
/// ([`no_centre_on_a_turned_and_moved_discs_own_pipe_axis_is_one_its_top_holds`]).
/// Comparing what the two pipes walk, bit for bit, does not call them the
/// same either: the reader's walk centres its circle again from the origin it
/// read back, so its points need not land on the kernel's.
///
/// Asking instead whether `corners` — the coedge's own two ends — are
/// certified `Decision::Zero` on the read-back pipe's skin looks more
/// principled, the same test `picks_the_real_ends` below runs for a picker,
/// but it is the wrong tool here: those corners come from an arc built by
/// turning the kernel's own circle, and the kernel's own pipe does not
/// certify them `Zero` either.
fn the_same_run(here: &EdgeRun, back: &EdgeRun, corners: [Point; 2], walked_by: Surface) -> bool {
    let same_bits = || format!("{here:?}") == format!("{back:?}");
    let (
        EdgeRun::PartOfWhereItCrosses {
            and: here_and,
            picked_by: here_picks,
        },
        EdgeRun::PartOfWhereItCrosses {
            and: back_and,
            picked_by: back_picks,
        },
    ) = (here, back)
    else {
        return same_bits();
    };
    let chord = corners[1] - corners[0];
    let the_arcs_plane = if let Surface::Plane(cut) = here_and {
        Some(*cut)
    } else if let Surface::Plane(face) = walked_by {
        Some(face)
    } else {
        None
    };
    let facing_the_same_way = |here: &HalfSpace, back: &HalfSpace| {
        if let Some(cut) = the_arcs_plane {
            let into_an_arc = cut.normal().cross(chord);
            let [here_side, back_side] =
                [here, back].map(|picker| picker.normal().dot(into_an_arc));
            (here_side > 0.0 && back_side > 0.0) || (here_side < 0.0 && back_side < 0.0)
        } else {
            here.normal().cross(back.normal()) == Vector::zero()
                && here.normal().dot(back.normal()) > 0.0
        }
    };
    let picks_the_real_ends = |picker: &Picker| {
        corners.into_iter().all(|corner| {
            picker.side_of_point(PredicatePoint::from(corner.coordinates())) == Decision::Pos
        })
    };
    let same_pipe = |here: &Cylinder, back: &Cylinder| {
        let read_back = back.axis().origin();
        let on_the_kernels_line =
            (read_back - here.axis().origin()).cross(here.axis().direction()) == Vector::zero();
        let where_the_kernels_walk_centres_it = the_arcs_plane
            .and_then(|flat| where_the_walk_centres_a_circle(&flat, here))
            .is_some_and(|centre| {
                centre.coordinates().map(f64::to_bits) == read_back.coordinates().map(f64::to_bits)
            });
        here.axis().direction() == back.axis().direction()
            && here.radius() == back.radius()
            && (on_the_kernels_line || where_the_kernels_walk_centres_it)
    };
    let same_and = match (here_and, back_and) {
        (Surface::Cylinder(here_pipe), Surface::Cylinder(back_pipe)) => {
            same_pipe(here_pipe, back_pipe)
        }
        (here, back) => format!("{here:?}") == format!("{back:?}"),
    };
    same_and
        && here_picks.len() == back_picks.len()
        && here_picks.iter().zip(back_picks).all(|pair| match pair {
            (Picker::InFrontOf(here), Picker::InFrontOf(back)) => facing_the_same_way(here, back),
            (Picker::OutsideAPipe(_), Picker::OutsideAPipe(_))
            | (Picker::InsideAPipe(_), Picker::InsideAPipe(_)) => {
                picks_the_real_ends(pair.0) && picks_the_real_ends(pair.1)
            }
            (here, back) => format!("{here:?}") == format!("{back:?}"),
        })
}

/// Where `rcad_geom::walk` centres the circle `flat` crosses `pipe` in: the
/// pipe's own origin moved up its axis to `flat`, in plain `f64`, as
/// `walk.rs`'s `where_a_plane_cuts_a_pipe` works it out and the writer's
/// `where_a_plane_crosses_a_pipe` too. `None` where the axis runs along
/// `flat` and meets it nowhere.
fn where_the_walk_centres_a_circle(flat: &Plane, pipe: &Cylinder) -> Option<Point> {
    let along = pipe.axis().direction();
    let leans = flat.normal().dot(along);
    if leans == 0.0 {
        return None;
    }
    let from = pipe.axis().origin();
    let up = (flat.offset() - flat.normal().dot(from - Point::origin())) / leans;
    Some(from + along * up)
}

/// Whether `solid` carries no placement at all: every shell it uses sits at
/// exactly [`Isometry::identity`].
///
/// A move with no turn is still a placement here — §249.4 item 2 measured one
/// ulp off a bar moved by `(0.1, 0.2, 0.3)` and no turn, so only the identity
/// is exempt from the measurement check above.
///
/// # Why shells are enough, and a face's own location is not also checked
///
/// A face use carries a location of its own (`rcad_topo`'s `FaceUse::loc`),
/// composed with its shell's on the way to a face view — so in the type this
/// function does not look far enough. It does not have to: every solid this
/// function ever sees came from `postcard::from_bytes::<Model>`
/// (`one_model_decoded_and_written`, this file), and that is `Model`'s
/// `Deserialize` impl (`crates/rcad-topo/src/serialize.rs:167`), which calls
/// `check_shape_is_possible` before handing back a `Model`. That check refuses
/// the bytes outright — `Invalid::AFaceUseCarriesAPlacement`
/// (`crates/rcad-topo/src/validate.rs:1060-1074`) — if any face use's own
/// location is not exactly the identity. `crates/rcad-topo/src/model.rs`'s
/// `a_file_that_shifts_one_face_off_its_shell_never_becomes_a_model` pins that
/// down through this same decode call. So a face use's own location is always
/// the identity here, on every solid this function is asked about, and a
/// shell's location is the only placement such a solid can carry.
fn solid_has_no_placement(solid: SolidView) -> bool {
    solid
        .shells()
        .all(|shell| shell.location() == Isometry::identity())
}

/// The bar whose measurement §249.1 saw come back a few ulps off: a curved
/// solid turned, so its measurement is left out, and nothing else is.
///
/// Built without new `use` lines: this file is also compiled into the fuzz
/// target, where `#[test]` functions are dropped and an import only they used
/// would be unused.
#[test]
fn a_round_bar_turned_an_eighth_about_z_comes_back_from_its_own_file() {
    let eighth_turn = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_4)
        .expect("a finite angle about a unit axis");
    let bytes = postcard::to_allocvec(&one_solid_placed_by(a_round_bar, &eighth_turn))
        .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// A cube of side two turned a tenth of a radian about `x` and moved by whole
/// units. Three of its faces sit on planes whose offset the foot of the
/// perpendicular, the point the writer used to write, reads back an ulp off,
/// and all six keep their normals through the file (§313).
#[test]
fn a_cube_turned_a_tenth_of_a_radian_comes_back_on_the_same_planes() {
    let turned = Isometry::translation(Vector::new(-13.0, 1.0, 8.0))
        .expect("a finite offset")
        .compose(&Isometry::rotation(&Axis::x(), 0.1).expect("a finite angle about a unit axis"));
    let bytes = postcard::to_allocvec(&one_solid_placed_by(a_cube, &turned))
        .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// Moved along its own axis, so that the first face compared, the bottom cap,
/// is on another plane.
#[test]
#[should_panic(expected = "with a different surface")]
fn a_round_bar_moved_one_unit_up_its_axis_is_told_apart_by_its_surfaces() {
    let moved = Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a finite offset");
    let here = one_solid_placed_by(a_round_bar, &Isometry::identity());
    let back = one_solid_placed_by(a_round_bar, &moved);
    assert_located_numbers_the_same(the_one_solid(&here), the_one_solid(&back));
}

/// A quarter turn about the bar's own axis leaves its three surfaces the same
/// bits and moves the corner on its seam.
#[test]
#[should_panic(expected = "at a different corner")]
fn a_round_bar_turned_a_quarter_about_its_axis_is_told_apart_by_its_corners() {
    let quarter_turn = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_2)
        .expect("a finite angle about a unit axis");
    let here = one_solid_placed_by(a_round_bar, &Isometry::identity());
    let back = one_solid_placed_by(a_round_bar, &quarter_turn);
    assert_located_numbers_the_same(the_one_solid(&here), the_one_solid(&back));
}

/// A quarter turn about the line through a ball's poles leaves its sphere and
/// its two poles the same bits and turns the half-plane its seam is cut from.
#[test]
#[should_panic(expected = "with a different edge run")]
fn a_ball_turned_a_quarter_about_its_poles_is_told_apart_by_its_seam() {
    let quarter_turn = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_2)
        .expect("a finite angle about a unit axis");
    let here = one_solid_placed_by(a_ball, &Isometry::identity());
    let back = one_solid_placed_by(a_ball, &quarter_turn);
    assert_located_numbers_the_same(the_one_solid(&here), the_one_solid(&back));
}

/// The first of the seven turned balls whose seam
/// `how_often_a_model_the_mutator_draws_is_refused` (in
/// `every_model_the_writer_fuzzer_kept_is_written_again.rs`) saw come back a
/// different run (`docs/decisions.md` §366.4): its seam is cut from a plane facing
/// `(0.1660299346975594, −0.9861207130895915, 0)`, a normal the turn left an ulp
/// off length one. The reader built that plane again through
/// `Plane::from_point_and_normal`, which scaled it to `0.16602993469755936`, and
/// dotted the ball's centre with it for an offset an ulp off the one the turn
/// left (Q822).
#[test]
fn a_ball_turned_about_its_poles_keeps_its_seam_plane_through_its_own_file() {
    let model = the_ball_the_mutator_drew_turned_about_its_poles();
    let text = rcad_step::write(&model, "a turned ball").expect("a turned ball is written");
    let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
    let back = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball this writer wrote is read back");
    let went_in = format!("{:?}", the_plane_its_seam_is_cut_from(&model));
    assert!(went_in.contains("0.1660299346975594,"), "{went_in}");
    assert_eq!(
        went_in,
        format!("{:?}", the_plane_its_seam_is_cut_from(back.model()))
    );
}

/// The same ball comes back the same in every number this target compares.
///
/// **Its seam's picker is not the one that went in, and cannot be.** The
/// kernel's picker is `(1, 0, 0)` turned, and the reader builds its own as
/// `(B − A) × w` from the two poles and the circle's axis, which is twelve
/// times the seam plane's normal turned a quarter, each component rounded. So
/// the two are not along each other even in exact arithmetic, and comparing
/// them by a cross product of exactly zero failed for every radius whose
/// diameter is not a power of two. Both face the same side of the seam within
/// its plane, so both pick the same half, which is what [`the_same_run`] asks.
#[test]
fn a_ball_turned_about_its_poles_comes_back_from_its_own_file() {
    let bytes = postcard::to_allocvec(&the_ball_the_mutator_drew_turned_about_its_poles())
        .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// Draw 367's ball: radius three at `(10, 7, −15)`, turned about `z`. Its seam
/// plane's offset, `0.80`, is small beside the centre's coordinates, so a step
/// of either one alone moves the reader's dot past it, and the writer finds
/// the circle's centre by stepping both (`a_centre_its_plane_reads_back_from`).
#[test]
fn a_turned_ball_whose_seam_plane_passes_near_the_origin_comes_back_from_its_own_file() {
    let turned = Isometry::translation(Vector::new(10.0, 7.0, -15.0))
        .expect("a finite offset")
        .compose(
            &Isometry::rotation(&Axis::z(), 3.818_263_087_845_863_4)
                .expect("a finite angle about a unit axis"),
        );
    let model = one_solid_placed_by(a_ball_of_radius_three, &turned);
    let cut = format!("{:?}", the_plane_its_seam_is_cut_from(&model));
    assert!(
        cut.contains("[[0.6262005639152322, -0.779662012510707, 0.0]]")
            && cut.contains("offset: 0.8043715515773728 "),
        "{cut}"
    );
    let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// A seam picked by a half-space facing the other half of its circle is told
/// apart from the seam that went in, though both stand on the same chord.
#[test]
fn a_seam_picked_from_the_other_side_is_a_different_run() {
    let model = the_ball_the_mutator_drew_turned_about_its_poles();
    let seam = the_seam_of_the_one_ball(&model);
    let here = seam.edge().run();
    let EdgeRun::PartOfWhereItCrosses { and, picked_by } = &here else {
        panic!("a ball's seam is part of where a plane crosses it, not {here:?}");
    };
    let [Picker::InFrontOf(picking)] = picked_by.as_slice() else {
        panic!("a ball's seam is picked by one half-space, not {picked_by:?}");
    };
    let the_other_half = EdgeRun::PartOfWhereItCrosses {
        and: *and,
        picked_by: [Picker::from(picking.reversed())].into_iter().collect(),
    };
    let corners = [seam.start().point(), seam.end().point()];
    let ball = the_one_solid(&model)
        .faces()
        .next()
        .expect("a ball has one face")
        .surface();
    assert!(the_same_run(&here, &here, corners, ball));
    assert!(!the_same_run(&here, &the_other_half, corners, ball));
}

/// The ball draw 89 of `how_often_a_model_the_mutator_draws_is_refused` (in
/// `every_model_the_writer_fuzzer_kept_is_written_again.rs`) builds: radius
/// six, turned about `z` by an angle `draw.fraction()` gave and
/// moved by whole units. The angle is the one whose turn has that draw's
/// quaternion, bit for bit.
fn the_ball_the_mutator_drew_turned_about_its_poles() -> Model {
    let turned = Isometry::translation(Vector::new(11.0, -5.0, -9.0))
        .expect("a finite offset")
        .compose(
            &Isometry::rotation(&Axis::z(), 3.308_395_003_925_273)
                .expect("a finite angle about a unit axis"),
        );
    one_solid_placed_by(a_ball_of_radius_six, &turned)
}

fn the_seam_of_the_one_ball(model: &Model) -> rcad_topo::CoedgeView<'_> {
    the_one_solid(model)
        .faces()
        .next()
        .expect("a ball has one face")
        .wires()
        .flat_map(WireView::coedges)
        .next()
        .expect("a ball's one wire runs down its seam")
}

fn the_plane_its_seam_is_cut_from(model: &Model) -> Plane {
    let run = the_seam_of_the_one_ball(model).edge().run();
    let EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cut),
        ..
    } = run
    else {
        panic!("a ball's seam is cut from a plane, not {run:?}");
    };
    cut
}

/// The frustum and the pointed cone this target draws, at each of its four
/// slopes, where they are built and turned a quarter about `x`: each is
/// written and comes back the same. Slopes one and two come back an ulp or two
/// off whatever half-angle is written (§366), so for those the slope is held
/// to what a half-angle carries and the measurement is not compared.
#[test]
fn every_cone_this_target_draws_comes_back_from_its_own_file() {
    let quarter_turn = Isometry::rotation(&Axis::x(), core::f64::consts::FRAC_PI_2)
        .expect("a finite angle about a unit axis");
    for slope in [0.25, 0.5, 1.0, 2.0] {
        for placement in [Isometry::identity(), quarter_turn] {
            for pointed in [false, true] {
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let solid = if pointed {
                    a_pointed_cone(&mut model, slope, 6.0)
                } else {
                    a_frustum(&mut model, slope, 2.0, 6.0)
                }
                .expect("each cone fits in a model this size");
                model
                    .place_solid(solid, &placement)
                    .expect("the key came from this model");
                let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
                let reached = one_model_decoded_and_written(&bytes);
                assert_eq!(
                    (reached.came_back_the_same, reached.solids_the_same),
                    (1, 1),
                    "slope {slope}, pointed {pointed}, {placement:?}: {reached:?}"
                );
            }
        }
    }
}

/// A frustum and a pointed cone of slope a quarter and a half, the two of this
/// target's four slopes some half-angle carries exactly, come back from their
/// own file with the slope they went in with, bit for bit, so their volume,
/// area and centroid are compared bit for bit too.
///
/// **This is what pins the writer's half-angle search end to end.** Written as
/// `atan(0.5)` alone, a half comes back `0.49999999999999994`, which
/// [`a_slope_comes_back_as_near_as_a_half_angle_carries_it`] lets through, and
/// the measurement is then not compared (§366.2).
#[test]
fn a_cone_of_a_slope_some_half_angle_carries_comes_back_with_that_slope_to_the_bit() {
    for slope in [0.25, 0.5] {
        for pointed in [false, true] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            if pointed {
                a_pointed_cone(&mut model, slope, 6.0)
            } else {
                a_frustum(&mut model, slope, 2.0, 6.0)
            }
            .expect("each cone fits in a model this size");
            let text = rcad_step::write(&model, "a cone").expect("a cone is written");
            let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
            let back = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a cone this writer wrote is read back");
            let slopes: Vec<f64> = back
                .model()
                .faces()
                .filter_map(|face| {
                    let Surface::Cone(cone) = face.surface() else {
                        return None;
                    };
                    Some(cone.slope())
                })
                .collect();
            assert_eq!(
                slopes.iter().map(|back| back.to_bits()).collect::<Vec<_>>(),
                [slope.to_bits()],
                "slope {slope}, pointed {pointed}: came back {slopes:?}"
            );
        }
    }
}

/// The quarter bend this target draws, at the ring and tube of the checked-in
/// seed and at the smallest and largest it draws, where it is built and turned
/// a quarter about `x`: each is written, with its two cap circles round the
/// tube (Q407), and comes back the same.
#[test]
fn every_bend_this_target_draws_comes_back_from_its_own_file() {
    let quarter_turn = Isometry::rotation(&Axis::x(), core::f64::consts::FRAC_PI_2)
        .expect("a finite angle about a unit axis");
    for (ring, tube) in [(6.0, 2.0), (2.0, 1.0), (12.0, 4.0)] {
        for placement in [Isometry::identity(), quarter_turn] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let solid = a_quarter_bend(&mut model, ring, tube).expect("each bend fits this model");
            model
                .place_solid(solid, &placement)
                .expect("the key came from this model");
            let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
            let reached = one_model_decoded_and_written(&bytes);
            assert_eq!(
                (reached.came_back_the_same, reached.solids_the_same),
                (1, 1),
                "ring {ring}, tube {tube}, {placement:?}: {reached:?}, {:?}",
                rcad_step::write(&model, "a bend").err()
            );
        }
    }
}

/// A bend of ring seven and tube one, turned about `x`, `y` and `z` by every
/// tenth of a radian up to a whole turn, in place and moved by whole units:
/// how many the writer writes, the reader reads back, and come back the same
/// solid.
///
/// **This counts what is left, and fails when a fix moves it.** The writer
/// writes all 372: turned about `y` by 5.0 and moved, it refused the seam
/// until Q843 looked further for its centre
/// (`a_bend_turned_five_about_y_and_moved_comes_back_from_its_own_file`).
///
/// **All 372 come back the same, every located number bit for bit, and the
/// ring's axis with the bits the file wrote.** The writer writes the axis as
/// the model places it, a turn of `z` an ulp or so off length one, and the
/// reader keeps it (`Axis::as_written`, in `geometry.rs`'s `TOROIDAL_SURFACE`
/// arm), as it keeps the normal of the seam's plane (§313.3).
///
/// Until Q1000 the reader built that axis with `Axis::new`, which scales it
/// again and moved the last bit of 76 of these axes, the count pinned below.
/// 48 of those bends were read back with the ring an ulp moved, and 28 were
/// refused with `CannotNameWhichPieceOfACrossingThisIs`: turned about `x` by
/// 2.5 the seam's plane and the ring's axis were
/// `(0, -0.5984721441039565, -0.8011436155469336)` and
/// `(…, -0.8011436155469337)`, whose cross product is `(5.55e-17, 0, 0)` and
/// not exactly nothing, so `rcad_geom`'s walk took the seam's plane for one
/// holding the axis (`where_a_plane_cuts_a_torus`) and neither meridian circle
/// held the seam's two ends (`which_of_these_circles_this_run_is`).
/// `docs/decisions.md` §355.6.
#[test]
fn how_often_a_turned_bend_comes_back_from_its_own_file() {
    let (mut bends, mut writer_refused, mut the_same, mut ring_moved) = (0, 0, 0, 0);
    let mut new_would_have_moved_the_ring = 0;
    let mut reader_refused = Vec::new();
    for axis in [Axis::x(), Axis::y(), Axis::z()] {
        for tenths in 1..63 {
            for moved in [Vector::zero(), Vector::new(3.0, 5.0, 10.0)] {
                let turned = Isometry::translation(moved)
                    .expect("a finite offset")
                    .compose(
                        &Isometry::rotation(&axis, f64::from(tenths) * 0.1)
                            .expect("a finite angle about a unit axis"),
                    );
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let solid = a_quarter_bend(&mut model, 7.0, 1.0).expect("a bend fits this model");
                model
                    .place_solid(solid, &turned)
                    .expect("the key came from this model");
                bends += 1;
                let Ok(text) = rcad_step::write(&model, "a turned bend") else {
                    writer_refused += 1;
                    continue;
                };
                let written = the_ring_axis(the_one_solid(&model));
                let scaled_again = Axis::new(written.origin(), written.direction())
                    .expect("a ring's placed axis is finite and has a length");
                if format!("{scaled_again:?}") != format!("{written:?}") {
                    new_would_have_moved_the_ring += 1;
                }
                let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
                match Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()) {
                    Ok(back) => {
                        let read = the_ring_axis(the_one_solid(back.model()));
                        if format!("{read:?}") != format!("{written:?}") {
                            ring_moved += 1;
                            continue;
                        }
                        assert!(
                            assert_located_numbers_the_same(
                                the_one_solid(&model),
                                the_one_solid(back.model())
                            ),
                            "{turned:?}"
                        );
                        the_same += 1;
                    }
                    Err(why) => reader_refused.push(format!("{turned:?}: {why:?}")),
                }
            }
        }
    }
    println!(
        "{bends} turned bends: {writer_refused} refused by the writer, {the_same} the same, \
         {ring_moved} with the ring's axis moved, {} refused by the reader; Axis::new would \
         have moved {new_would_have_moved_the_ring} of the written ring axes",
        reader_refused.len()
    );
    assert_eq!(
        (
            bends,
            writer_refused,
            the_same,
            ring_moved,
            reader_refused.len()
        ),
        (372, 0, 372, 0, 0),
        "{reader_refused:?}"
    );
    assert_eq!(new_would_have_moved_the_ring, 76);
}

/// A round bar, a frustum and a pointed cone, turned as
/// [`how_often_a_turned_bend_comes_back_from_its_own_file`] turns its bend:
/// all 1 116 are written, and every one comes back with its pipe's or cone's
/// axis as the file wrote it, and the same solid.
///
/// The reader's `CYLINDRICAL_SURFACE` arm and `cone_from` keep the axis
/// (`Axis::as_written`), and `cone_from` moves it to the apex rather than
/// build it again (`Axis::moved_to`). The count pinned last is how many of the
/// written axes `Axis::new` would have moved, which is how many came back
/// moved while the reader built them with it (`docs/decisions.md` §355.6).
#[test]
fn how_often_a_turned_bar_or_cone_comes_back_from_its_own_file() {
    let shapes: [fn(&mut Model) -> rcad_topo::Result<SolidKey>; 3] = [
        a_round_bar,
        |model| a_frustum(model, 0.5, 2.0, 6.0),
        |model| a_pointed_cone(model, 0.5, 6.0),
    ];
    let (mut drawn, mut writer_refused, mut the_same, mut axis_moved) = (0, 0, 0, 0);
    let mut new_would_have_moved_the_axis = 0;
    let mut reader_refused = Vec::new();
    for shape in shapes {
        for axis in [Axis::x(), Axis::y(), Axis::z()] {
            for tenths in 1..63 {
                for moved in [Vector::zero(), Vector::new(3.0, 5.0, 10.0)] {
                    let turned = Isometry::translation(moved)
                        .expect("a finite offset")
                        .compose(
                            &Isometry::rotation(&axis, f64::from(tenths) * 0.1)
                                .expect("a finite angle about a unit axis"),
                        );
                    let model = one_solid_placed_by(shape, &turned);
                    drawn += 1;
                    let Ok(text) = rcad_step::write(&model, "a turned bar or cone") else {
                        writer_refused += 1;
                        continue;
                    };
                    let written = the_curved_axis(the_one_solid(&model));
                    let scaled_again = Axis::new(written.origin(), written.direction())
                        .expect("a placed axis is finite and has a length");
                    if format!("{scaled_again:?}") != format!("{written:?}") {
                        new_would_have_moved_the_axis += 1;
                    }
                    let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
                    match Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()) {
                        Ok(back) => {
                            let read = the_curved_axis(the_one_solid(back.model()));
                            if format!("{read:?}") != format!("{written:?}") {
                                axis_moved += 1;
                                continue;
                            }
                            assert!(
                                assert_located_numbers_the_same(
                                    the_one_solid(&model),
                                    the_one_solid(back.model())
                                ),
                                "{turned:?}"
                            );
                            the_same += 1;
                        }
                        Err(why) => reader_refused.push(format!("{turned:?}: {why:?}")),
                    }
                }
            }
        }
    }
    println!(
        "{drawn} turned bars and cones: {writer_refused} refused by the writer, {the_same} the \
         same, {axis_moved} with the axis moved, {} refused by the reader; Axis::new would have \
         moved {new_would_have_moved_the_axis} of the written axes",
        reader_refused.len()
    );
    assert_eq!(
        (
            drawn,
            writer_refused,
            the_same,
            axis_moved,
            reader_refused.len()
        ),
        (1116, 0, 1116, 0, 0),
        "{reader_refused:?}"
    );
    assert_eq!(new_would_have_moved_the_axis, 228);
}

/// The axis of the one pipe or cone `solid` is bounded by, where it is placed.
fn the_curved_axis(solid: SolidView) -> Axis {
    let mut axes = solid.faces().filter_map(|face| {
        let surface = face.surface();
        if let Surface::Cylinder(pipe) = surface {
            return Some(pipe.axis());
        }
        let Surface::Cone(cone) = surface else {
            return None;
        };
        Some(cone.axis())
    });
    let axis = axes.next().expect("a bar or a cone has one curved face");
    assert!(axes.next().is_none(), "a bar or a cone has one curved face");
    axis
}

/// A round bar with a disc drawn on its top, turned as
/// [`how_often_a_turned_bend_comes_back_from_its_own_file`] turns its bend:
/// all 372 are written, every one comes back with the pipe that names the
/// drawn circle on the axis the file wrote, and the same solid.
///
/// The writer writes that circle where the reader's `Axis::as_written` gives
/// the pipe back (`Writing::an_arc_of_a_circle_on_a_flat_face`), and the
/// reader builds the pipe with it (`Walk::the_surface_that_names_the_curve`).
/// Until Q1001 both asked `Axis::new`, which moves the last bit of 76 of the
/// written axes, the count pinned last, and the writer refused exactly those
/// 76 with `ACurveTheFacesEitherSideOfItDoNotName` between two planes. With
/// the writer moved alone the reader read the 76 back with the axis moved, and
/// with the reader moved alone the writer still refused them
/// (`docs/decisions.md` §355.6).
#[test]
fn how_often_a_turned_disc_drawn_on_a_bar_comes_back_from_its_own_file() {
    let (mut drawn, mut the_same, mut axis_moved) = (0, 0, 0);
    let mut new_would_have_moved_the_axis = 0;
    let (mut writer_refused, mut reader_refused) = (Vec::new(), Vec::new());
    for axis in [Axis::x(), Axis::y(), Axis::z()] {
        for tenths in 1..63 {
            for moved in [Vector::zero(), Vector::new(3.0, 5.0, 10.0)] {
                let turned = Isometry::translation(moved)
                    .expect("a finite offset")
                    .compose(
                        &Isometry::rotation(&axis, f64::from(tenths) * 0.1)
                            .expect("a finite angle about a unit axis"),
                    );
                let model = one_solid_placed_by(a_bar_with_a_disc_drawn_on_its_top, &turned);
                drawn += 1;
                let written = the_drawn_circles_axis(the_one_solid(&model));
                let scaled_again = Axis::new(written.origin(), written.direction())
                    .expect("a placed axis is finite and has a length");
                if format!("{scaled_again:?}") != format!("{written:?}") {
                    new_would_have_moved_the_axis += 1;
                }
                let text = match rcad_step::write(&model, "a turned disc drawn on a bar") {
                    Ok(text) => text,
                    Err(why) => {
                        writer_refused.push(format!("{turned:?}: {why:?}"));
                        continue;
                    }
                };
                let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
                match Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()) {
                    Ok(back) => {
                        let read = the_drawn_circles_axis(the_one_solid(back.model()));
                        if format!("{read:?}") != format!("{written:?}") {
                            axis_moved += 1;
                            continue;
                        }
                        assert!(
                            assert_located_numbers_the_same(
                                the_one_solid(&model),
                                the_one_solid(back.model())
                            ),
                            "{turned:?}"
                        );
                        the_same += 1;
                    }
                    Err(why) => reader_refused.push(format!("{turned:?}: {why:?}")),
                }
            }
        }
    }
    println!(
        "{drawn} turned discs drawn on a bar: {} refused by the writer, {the_same} the same, \
         {axis_moved} with the drawn circle's axis moved, {} refused by the reader; Axis::new \
         would have moved {new_would_have_moved_the_axis} of the written axes",
        writer_refused.len(),
        reader_refused.len()
    );
    assert_eq!(
        (
            drawn,
            writer_refused.len(),
            the_same,
            axis_moved,
            reader_refused.len()
        ),
        (372, 0, 372, 0, 0),
        "writer: {writer_refused:?}\nreader: {reader_refused:?}"
    );
    assert_eq!(new_would_have_moved_the_axis, 76);
}

/// The axis of the pipe that names the circle drawn on the flat top of
/// [`a_bar_with_a_disc_drawn_on_its_top`], where it is placed: both halves of
/// the circle name the one pipe.
fn the_drawn_circles_axis(solid: SolidView) -> Axis {
    let axes: Vec<Axis> = solid
        .faces()
        .flat_map(|face| face.wires().flat_map(WireView::coedges).collect::<Vec<_>>())
        .filter_map(|coedge| {
            let EdgeRun::PartOfWhereItCrosses {
                and: Surface::Cylinder(pipe),
                ..
            } = coedge.edge().run()
            else {
                return None;
            };
            Some(pipe.axis())
        })
        .collect();
    let [first, ..] = axes[..] else {
        panic!("the disc drawn on the bar has no arc");
    };
    assert!(
        axes.len() == 4
            && axes
                .iter()
                .all(|axis| format!("{axis:?}") == format!("{first:?}")),
        "two halves of one circle, each walked by the disc and by the cap round it: {axes:?}"
    );
    first
}

/// The bend of [`how_often_a_turned_bend_comes_back_from_its_own_file`] turned
/// about `x` by `2.5`, read back from its own file, keeps its ring's axis
/// through its model's saved bytes.
///
/// A model read back from a file stores the ring's axis as the file wrote it,
/// an ulp off length one, where a model built here stores `z` and a placement.
/// Saved and loaded, that axis comes back through `Axis`'s `Deserialize`,
/// which is `Axis::as_written`. Through `Axis::new` it came back
/// `(0, -0.5984721441039565, -0.8011436155469337)`, an ulp from what was saved
/// (`docs/decisions.md` §355.6).
#[test]
fn a_turned_bend_read_back_from_its_own_file_keeps_its_ring_axis_through_its_bytes() {
    let turned = Isometry::rotation(&Axis::x(), 2.5).expect("a finite angle about a unit axis");
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_quarter_bend(&mut model, 7.0, 1.0).expect("a bend fits this model");
    model
        .place_solid(solid, &turned)
        .expect("the key came from this model");
    let written = the_ring_axis(the_one_solid(&model));

    let text = rcad_step::write(&model, "a turned bend").expect("the bend is written");
    let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
    let back = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the bend is read back");
    let read = the_ring_axis(the_one_solid(back.model()));
    assert_eq!(format!("{read:?}"), format!("{written:?}"));

    let bytes = postcard::to_allocvec(back.model()).expect("a model always encodes");
    let loaded: Model = postcard::from_bytes(&bytes).expect("a model's own bytes decode");
    let reloaded = the_ring_axis(the_one_solid(&loaded));
    assert_eq!(
        format!("{reloaded:?}"),
        format!("{written:?}"),
        "the ring's axis moved between saving the read-back model and loading it"
    );
}

/// The axis of the one ring `solid` is bounded by, where it is placed.
fn the_ring_axis(solid: SolidView) -> Axis {
    let mut rings = solid.faces().filter_map(|face| {
        let Surface::Torus(ring) = face.surface() else {
            return None;
        };
        Some(ring.axis())
    });
    let ring = rings.next().expect("a bend has one ring");
    assert!(rings.next().is_none(), "a bend has one ring");
    ring
}

/// The one bend of [`how_often_a_turned_bend_comes_back_from_its_own_file`]
/// the writer refused, ring seven and tube one turned about `y` by `5.0` and
/// moved by `(3, 5, 10)`, is written and comes back the same, every located
/// number bit for bit.
///
/// **Its seam's plane reads back from one centre the reader holds, and it is
/// twenty steps out.** The plane's offset is the turned normal dotted with the
/// move in plain `f64`, `32` ulps from the ring's centre dotted and rounded
/// once, and of every `f64` centre the reader could hold on the ring, the one
/// it reads back from is `23` steps of `x` and `19` of `z` from the ring's
/// centre, at `15.88` of the sixteen units the reader allows
/// (`a_turned_bends_seam_has_one_centre_its_plane_reads_back_from` in
/// `write.rs`). The writer stepped four and refused the seam by name (Q843).
#[test]
fn a_bend_turned_five_about_y_and_moved_comes_back_from_its_own_file() {
    let turned = Isometry::translation(Vector::new(3.0, 5.0, 10.0))
        .expect("a finite offset")
        .compose(&Isometry::rotation(&Axis::y(), 5.0).expect("a finite angle about a unit axis"));
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_quarter_bend(&mut model, 7.0, 1.0).expect("a bend fits this model");
    model
        .place_solid(solid, &turned)
        .expect("the key came from this model");
    let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}, {:?}",
        rcad_step::write(&model, "a turned bend").err()
    );
}

fn a_round_bar(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_round_bar(3.0, 5.0, &Isometry::identity())
}

fn a_ball(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_ball(2.0, &Isometry::identity())
}

fn a_ball_of_radius_six(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_ball(6.0, &Isometry::identity())
}

fn a_ball_of_radius_three(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_ball(3.0, &Isometry::identity())
}

fn a_cube(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A model holding one shape, built at the origin and then placed by
/// `placement`, the order the mutator builds one in.
fn one_solid_placed_by(
    shape: fn(&mut Model) -> rcad_topo::Result<SolidKey>,
    placement: &Isometry,
) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = shape(&mut model).expect("each shape fits in a model this size");
    model
        .place_solid(solid, placement)
        .expect("the key came from this model");
    model
}

fn the_one_solid(model: &Model) -> SolidView<'_> {
    model.solids().next().expect("the model holds one solid")
}

/// Change `data[..size]` into the `postcard` bytes of a model, and return how
/// many bytes that took.
///
/// # Every mutation is still a model
///
/// The input is `postcard` bytes of a [`Model`], the route a stranger's saved
/// file takes, and libFuzzer's own byte mutations almost never land on bytes
/// that still decode: 19 of the 1183 seeds checked in on 2026-09-25 did, and 16
/// of those were empty. So this replaces libFuzzer's mutator. It decodes the
/// input and makes one change a model can take — a byte changed where the
/// result still decodes, one more solid, one solid moved or turned, or a fresh
/// model — and encodes the result. In two 60-second runs on 2026-09-25, 11 145
/// of 11 179 and 12 272 of 12 309 mutated inputs decoded (`docs/decisions.md`
/// §241).
///
/// The shapes are the ones the writer has to spell differently: a cube, a round
/// bar, a ball, a frustum, a pointed cone and a quarter bend of pipe, which is
/// the torus a mechanical file holds. No builder in the kernel makes the last
/// three, so they are described face by face here, each with its corners exact
/// on its curved surface — a power-of-two slope on a cone, the inner equator on
/// a torus — because `Model::validate` asks that every corner of a curved face
/// be certified on it.
///
/// An input that does not decode becomes a fresh model, so the 1164 seeds that
/// never reached the writer each start a model rather than staying bytes.
///
/// The one input this leaves as it came is one where even a fresh one-solid
/// model does not fit in `max_size`. With no `-max_len`, libFuzzer's limit is
/// 4096 bytes — its own floor, since the corpus's longest seed is 4064 — and
/// the longest one-solid model drawn here, the quarter bend, is 681.
///
/// `byte_mutate` is libFuzzer's own mutation (`libfuzzer_sys::fuzzer_mutate`),
/// passed in rather than called directly, because this file is also compiled
/// into the ordinary test suite (`mod fuzzing` in
/// `every_model_the_writer_fuzzer_kept_is_written_again.rs`), which does not
/// depend on `libfuzzer-sys` and cannot safely call its mutation outside a real
/// fuzzing run (`docs/decisions.md` §241.1: it draws from libFuzzer's own
/// generator, not from `seed`). The fuzz target passes the real one; the
/// replay test passes a stand-in, over seeds chosen so it is never reached.
pub fn still_a_model(
    data: &mut [u8],
    size: usize,
    max_size: usize,
    seed: u32,
    byte_mutate: fn(&mut [u8], usize, usize) -> usize,
) -> usize {
    let mut draw = Draw::new(seed);
    let mut model = match postcard::from_bytes::<Model>(&data[..size]) {
        Ok(model) => model,
        Err(_) => a_fresh_model(&mut draw),
    };
    match draw.below(8) {
        0..=2 => {
            if let Some(nudged) =
                one_byte_changed_and_still_a_model(data, size, max_size, byte_mutate)
            {
                return nudged;
            }
            one_more_solid(&mut model, &mut draw);
        }
        3 | 4 => one_more_solid(&mut model, &mut draw),
        5 | 6 => one_solid_moved(&mut model, &mut draw),
        _ => model = a_fresh_model(&mut draw),
    }
    written_over(data, max_size, &model)
        .or_else(|| written_over(data, max_size, &a_fresh_model(&mut draw)))
        .unwrap_or(size)
}

/// Encode `model` over the front of `data`, if it fits in `max_size`.
fn written_over(data: &mut [u8], max_size: usize, model: &Model) -> Option<usize> {
    let bytes = postcard::to_allocvec(model).expect("a model always encodes");
    if bytes.len() > max_size {
        return None;
    }
    data[..bytes.len()].copy_from_slice(&bytes);
    Some(bytes.len())
}

/// libFuzzer's own mutation, kept only if the bytes still decode.
///
/// This is the half that reaches the writer with a model `Model::validate`
/// would refuse — a corner nudged off its face, a plane turned a little — which
/// no builder makes and which the writer owes an `Err` for rather than a crash.
fn one_byte_changed_and_still_a_model(
    data: &mut [u8],
    size: usize,
    max_size: usize,
    byte_mutate: fn(&mut [u8], usize, usize) -> usize,
) -> Option<usize> {
    let original = data[..size].to_vec();
    for _ in 0..4 {
        let nudged = byte_mutate(data, size, max_size);
        if postcard::from_bytes::<Model>(&data[..nudged]).is_ok() {
            return Some(nudged);
        }
        data[..size].copy_from_slice(&original);
    }
    None
}

/// A model of the size every shape here is drawn at, holding one of them.
fn a_fresh_model(draw: &mut Draw) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_drawn_solid(&mut model, draw)
        .expect("every shape drawn here is sound in a model of this size");
    maybe_moved(&mut model, solid, draw);
    model
}

/// Add one drawn shape to `model`, or start again if the model will not take
/// it.
///
/// A model that came through the door can carry any scale at all, and a shape
/// a few units across is refused outright in a model whose working size is
/// `1e-300`. That refusal is the builder's answer to a caller, not a defect,
/// so the mutation moves on rather than keeping a model it could not change.
fn one_more_solid(model: &mut Model, draw: &mut Draw) {
    match a_drawn_solid(model, draw) {
        Ok(solid) => maybe_moved(model, solid, draw),
        Err(_) => *model = a_fresh_model(draw),
    }
}

/// Move or turn one of `model`'s solids.
///
/// Only a sound model is moved. `Model::place_solid` checks the solid it moved
/// and panics if the solid is broken, which is right for a solid the kernel
/// built and wrong to ask of one a changed byte broke on the way in.
fn one_solid_moved(model: &mut Model, draw: &mut Draw) {
    let sound = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .is_ok();
    let count = model.solid_count() as u64;
    if !sound || count == 0 {
        one_more_solid(model, draw);
        return;
    }
    let which = usize::try_from(draw.below(count)).expect("below a usize count");
    let solid = model
        .solids()
        .nth(which)
        .expect("fewer than the model's own count")
        .key();
    model
        .place_solid(solid, &a_movement(draw))
        .expect("the key came from this model");
}

/// Leave the new solid where it was built a third of the time, move it a
/// third, and move and turn it the last third.
fn maybe_moved(model: &mut Model, solid: SolidKey, draw: &mut Draw) {
    if draw.below(3) == 0 {
        return;
    }
    model
        .place_solid(solid, &a_movement(draw))
        .expect("the key came from this model");
}

/// A move by whole units, and half the time a turn about one of the three axes
/// or a slanted one — by a multiple of an eighth of a turn, which is not exact
/// in `f64`, or by any angle at all.
fn a_movement(draw: &mut Draw) -> Isometry {
    let offset = Vector::new(
        draw.whole(-16, 16),
        draw.whole(-16, 16),
        draw.whole(-16, 16),
    );
    let moved = Isometry::translation(offset).expect("a finite offset");
    if draw.below(2) == 0 {
        return moved;
    }
    let axis = match draw.below(4) {
        0 => Axis::x(),
        1 => Axis::y(),
        2 => Axis::z(),
        _ => Axis::new(
            Point::origin(),
            Vector::new(draw.whole(1, 4), draw.whole(-4, 4), draw.whole(-4, 4)),
        )
        .expect("a direction whose first component is at least one"),
    };
    let angle = if draw.below(2) == 0 {
        draw.whole(1, 7) * core::f64::consts::FRAC_PI_4
    } else {
        draw.fraction() * core::f64::consts::TAU
    };
    let turned = Isometry::rotation(&axis, angle).expect("a finite angle about a unit axis");
    moved.compose(&turned)
}

/// One of the six shapes, with whole-number sizes and a power-of-two slope.
fn a_drawn_solid(model: &mut Model, draw: &mut Draw) -> rcad_topo::Result<SolidKey> {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    match draw.below(6) {
        0 => model.add_cube(draw.whole(1, 8), &policy, &mut log),
        1 => model.add_round_bar(draw.whole(1, 8), draw.whole(1, 16), &Isometry::identity()),
        2 => model.add_ball(draw.whole(1, 8), &Isometry::identity()),
        3 => {
            let near = draw.whole(1, 8);
            let far = near + draw.whole(1, 8);
            a_frustum(model, a_slope(draw), near, far)
        }
        4 => a_pointed_cone(model, a_slope(draw), draw.whole(1, 8)),
        _ => {
            let tube = draw.whole(1, 4);
            a_quarter_bend(model, tube + draw.whole(1, 8), tube)
        }
    }
}

/// A power of two, so a cone's corner at `(slope · height, 0, height)` is on
/// the cone exactly.
fn a_slope(draw: &mut Draw) -> f64 {
    [0.25, 0.5, 1.0, 2.0][usize::try_from(draw.below(4)).expect("below four")]
}

/// A frustum about `+z` with its apex at the origin and its caps at `near` and
/// `far` up the axis: two cap circles, a seam up the ruling over `+x`, three
/// faces — a round bar's structure on a cone.
fn a_frustum(model: &mut Model, slope: f64, near: f64, far: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let bottom_corner = builder
        .add_vertex(Point::new(slope * near, 0.0, near))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cone is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cone in one closed curve");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cone in one closed curve");

    let bottom = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, near),
        Vector::new(0.0, 0.0, -1.0),
        bottom_circle,
        Orientation::Reversed,
    );
    let top = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, far),
        Vector::new(0.0, 0.0, 1.0),
        top_circle,
        Orientation::Forward,
    );
    let side = a_wall(
        &mut builder,
        Surface::Cone(skin),
        (bottom_circle, Orientation::Forward),
        seam,
        top_circle,
    );
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A cone that ends in a point: a base cap at `far` up the axis and one face on
/// the cone reaching down to the apex. Two faces, one edge, one vertex, and the
/// apex is not a vertex at all — it is where the cone's own chart runs out.
fn a_pointed_cone(model: &mut Model, slope: f64, far: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let rim = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");
    let cap = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, far),
        Vector::new(0.0, 0.0, 1.0),
        circle,
        Orientation::Forward,
    );
    let side_coedge = builder
        .add_coedge(circle, Orientation::Reversed)
        .expect("an edge of this build");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("one closed edge is a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone, bounded by its base circle");
    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A quarter of a torus about `+z`, from the cap in the plane `y = 0` to the
/// cap in the plane `x = 0`, with its seam along the inner equator.
///
/// The corners sit on the inner equator, `(ring − tube, 0, 0)` and
/// `(0, ring − tube, 0)`, where the torus's quartic is a difference of two
/// products that are equal number for number, so it cancels exactly for any
/// sizes. The seam names which of the two circles the plane `z = 0` cuts by
/// saying it is inside the pipe of the ring's own radius, and which quarter of
/// that circle by a half-space.
///
/// The wall is used forwards, because a torus faces out of its tube, and each
/// circle is walked right-handed about the outward normal of the face it
/// bounds — the way a file written as the standard says holds a bend
/// (`docs/decisions.md` §361).
fn a_quarter_bend(model: &mut Model, ring: f64, tube: f64) -> rcad_topo::Result<SolidKey> {
    a_quarter_bend_picked_inside(
        model,
        ring,
        tube,
        Cylinder::new(Axis::z(), ring).expect("the ring's own radius"),
    )
}

/// Like [`a_quarter_bend`], but the seam's `InsideAPipe` picker holds `pipe`
/// rather than the ring's own, untouched one — Q1293's pins build it with the
/// pipe a file's own round trip can carry: a subnormal axis, or a radius an
/// ulp off the ring's.
fn a_quarter_bend_picked_inside(
    model: &mut Model,
    ring: f64,
    tube: f64,
    pipe: Cylinder,
) -> rcad_topo::Result<SolidKey> {
    let skin = Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring");
    let mut builder = SolidBuilder::new();
    let inner = ring - tube;
    let start_corner = builder
        .add_vertex(Point::new(inner, 0.0, 0.0))
        .expect("a finite point");
    let end_corner = builder
        .add_vertex(Point::new(0.0, inner, 0.0))
        .expect("a finite point");
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder
        .add_edge(
            start_corner,
            end_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(flat_through_the_middle),
                picked_by: [
                    Picker::InsideAPipe(pipe),
                    Picker::from(
                        HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                            .expect("a normal with a length"),
                    ),
                ]
                .into_iter()
                .collect(),
            },
        )
        .expect("a quarter of the inner equator");
    let start_circle = builder
        .add_edge(
            start_corner,
            start_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("one whole circle of the two this plane cuts");
    let end_circle = builder
        .add_edge(
            end_corner,
            end_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("one whole circle of the two this plane cuts");

    let start_cap = a_cap(
        &mut builder,
        Point::origin(),
        Vector::new(0.0, -1.0, 0.0),
        start_circle,
        Orientation::Forward,
    );
    let end_cap = a_cap(
        &mut builder,
        Point::origin(),
        Vector::new(-1.0, 0.0, 0.0),
        end_circle,
        Orientation::Reversed,
    );
    let side = a_wall(
        &mut builder,
        Surface::Torus(skin),
        (start_circle, Orientation::Reversed),
        seam,
        end_circle,
    );
    builder
        .add_shell(&[
            (start_cap, Orientation::Forward),
            (end_cap, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// Q1293's crash-0109edfc: a bend whose ring is four and tube two comes back
/// from its own file though the reader rebuilds the seam's pipe picker from
/// the ring's own axis and radius (`which_of_a_rings_two_circles`,
/// `crates/rcad-step/src/brep.rs`), and that rebuild can carry a subnormal
/// the kernel's own axis never had — `(0, 0, 1)` read back as
/// `(1.7e-322, 0, 1)`. [`the_same_run`] must still call this the same run,
/// because both axes pick the same two corners.
#[test]
fn a_bends_seam_picked_inside_a_pipe_is_the_same_run_with_a_subnormal_axis() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend = a_quarter_bend(&mut model, 4.0, 2.0).expect("a bend fits this model");
    model
        .place_solid(bend, &Isometry::identity())
        .expect("the key came from this model");
    let seam = the_seam_of_a_bend(&model);
    let here = seam.edge().run();
    let EdgeRun::PartOfWhereItCrosses { and, picked_by } = &here else {
        panic!("a bend's seam is part of where a plane crosses it, not {here:?}");
    };
    let subnormal_axis = Axis::new(Point::origin(), Vector::new(1.7e-322, 0.0, 1.0))
        .expect("a finite, non-zero direction");
    let rebuilt_from_a_file = EdgeRun::PartOfWhereItCrosses {
        and: *and,
        picked_by: picked_by
            .iter()
            .map(|picker| match picker {
                Picker::InsideAPipe(pipe) => Picker::InsideAPipe(
                    Cylinder::new(subnormal_axis, pipe.radius()).expect("a positive radius"),
                ),
                other => *other,
            })
            .collect(),
    };
    let corners = [seam.start().point(), seam.end().point()];
    assert!(the_same_run(&here, &rebuilt_from_a_file, corners, *and));
}

/// Q1293's crash-16aad257: a bend whose ring is nine and tube four comes back
/// from its own file though the reader rebuilds the ring's own radius an ulp
/// off, `9.000000000000004` for `9.0`.
#[test]
fn a_bends_seam_picked_inside_a_pipe_is_the_same_run_with_a_radius_an_ulp_off() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend = a_quarter_bend(&mut model, 9.0, 4.0).expect("a bend fits this model");
    model
        .place_solid(bend, &Isometry::identity())
        .expect("the key came from this model");
    let seam = the_seam_of_a_bend(&model);
    let here = seam.edge().run();
    let EdgeRun::PartOfWhereItCrosses { and, picked_by } = &here else {
        panic!("a bend's seam is part of where a plane crosses it, not {here:?}");
    };
    let rebuilt_from_a_file = EdgeRun::PartOfWhereItCrosses {
        and: *and,
        picked_by: picked_by
            .iter()
            .map(|picker| match picker {
                Picker::InsideAPipe(pipe) => Picker::InsideAPipe(
                    Cylinder::new(pipe.axis(), 9.000_000_000_000_004).expect("a positive radius"),
                ),
                other => *other,
            })
            .collect(),
    };
    let corners = [seam.start().point(), seam.end().point()];
    assert!(the_same_run(&here, &rebuilt_from_a_file, corners, *and));
}

/// The same seam picked from outside its ring's own pipe, rather than inside
/// it, is still a different run: [`the_same_run`]'s pipe-picker arm only
/// pairs `InsideAPipe` with `InsideAPipe` and `OutsideAPipe` with
/// `OutsideAPipe`, so it must not call these two the same.
#[test]
fn a_bends_seam_picked_outside_its_pipe_is_a_different_run_from_the_one_picked_inside_it() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend = a_quarter_bend(&mut model, 9.0, 4.0).expect("a bend fits this model");
    model
        .place_solid(bend, &Isometry::identity())
        .expect("the key came from this model");
    let seam = the_seam_of_a_bend(&model);
    let here = seam.edge().run();
    let EdgeRun::PartOfWhereItCrosses { and, picked_by } = &here else {
        panic!("a bend's seam is part of where a plane crosses it, not {here:?}");
    };
    let picked_from_outside = EdgeRun::PartOfWhereItCrosses {
        and: *and,
        picked_by: picked_by
            .iter()
            .map(|picker| match picker {
                Picker::InsideAPipe(pipe) => Picker::OutsideAPipe(*pipe),
                other => *other,
            })
            .collect(),
    };
    let corners = [seam.start().point(), seam.end().point()];
    assert!(the_same_run(&here, &here, corners, *and));
    assert!(!the_same_run(&here, &picked_from_outside, corners, *and));
}

/// Q1293's pin (1), crash-0109edfc: a bend of ring four and tube two whose
/// seam is picked inside the pipe of radius four through the origin along
/// `(1.7e-322, 0, 1)` — the subnormal axis the reader's own round trip can
/// carry, not the ring's own `z` — and by `InFrontOf` the half-space of
/// normal `(1, 1, 0)` and offset zero, placed by the turn about `z` of
/// quaternion `w` `0.9698928535888416`, `z` `0.24353203599792378` and moved
/// by `(-5, 12, 3)`.
#[test]
fn a_bend_with_a_subnormal_pipe_axis_turned_and_moved_comes_back_from_its_own_file() {
    let pipe = Cylinder::new(
        Axis::new(Point::origin(), Vector::new(1.7e-322, 0.0, 1.0))
            .expect("a finite, non-zero direction"),
        4.0,
    )
    .expect("a positive radius");
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend =
        a_quarter_bend_picked_inside(&mut model, 4.0, 2.0, pipe).expect("a bend fits this model");
    let placement = Isometry::try_from((
        [-5.0, 12.0, 3.0],
        [0.9698928535888416, 0.0, 0.0, 0.24353203599792378],
    ))
    .expect("a finite offset and a unit-length turn");
    model
        .place_solid(bend, &placement)
        .expect("the key came from this model");
    let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// Q1293's pin (2), crash-16aad257: a bend of ring nine and tube four whose
/// seam is picked inside the pipe of radius `9.000000000000004` about `z` —
/// an ulp off the ring's own nine — moved by `(9, 20, -5)`.
#[test]
fn a_bend_with_its_pipe_radius_an_ulp_off_moved_comes_back_from_its_own_file() {
    let pipe = Cylinder::new(Axis::z(), 9.000_000_000_000_004).expect("a positive radius");
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend =
        a_quarter_bend_picked_inside(&mut model, 9.0, 4.0, pipe).expect("a bend fits this model");
    let moved = Isometry::translation(Vector::new(9.0, 20.0, -5.0)).expect("a finite offset");
    model
        .place_solid(bend, &moved)
        .expect("the key came from this model");
    let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// The seam of [`a_quarter_bend`]: the one coedge whose edge crosses the
/// plane through the ring's own middle, not one of its two closed circles.
fn the_seam_of_a_bend(model: &Model) -> CoedgeView<'_> {
    the_one_solid(model)
        .faces()
        .flat_map(|face| face.wires().flat_map(WireView::coedges).collect::<Vec<_>>())
        .find(|coedge| matches!(coedge.edge().run(), EdgeRun::PartOfWhereItCrosses { .. }))
        .expect("a bend has one seam cut from its ring's own plane")
}

/// A round bar of radius two about `+z`, four tall, with a disc of radius one
/// drawn on its top cap: the cap is a ring round the disc and the disc a face
/// of its own on the same plane, so the circle between them is two half
/// circles that name the pipe of radius one standing on the cap.
fn a_bar_with_a_disc_drawn_on_its_top(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    a_bar_with_a_disc_drawn_by_a_pipe_from(model, Point::new(0.0, 0.0, 4.0), 1.0)
}

/// Like [`a_bar_with_a_disc_drawn_on_its_top`], but the drawn pipe's own axis
/// starts at `pipe_origin` rather than at the disc's own centre, and the disc
/// has `radius` rather than one. `pipe_origin` still sits on the one line
/// through the disc's centre pointing `+z` — the writer moves a drawn
/// circle's pipe to where its axis meets the flat face it is drawn on
/// when the pipe's own origin is not already there
/// (`Writing::an_arc_of_a_circle_on_a_flat_face`), so a `pipe_origin` off the
/// disc's own plane is what exercises that move.
fn a_bar_with_a_disc_drawn_by_a_pipe_from(
    model: &mut Model,
    pipe_origin: Point,
    radius: f64,
) -> rcad_topo::Result<SolidKey> {
    let up = Vector::new(0.0, 0.0, 1.0);
    let centre = Point::new(0.0, 0.0, 4.0);
    let drawn = Cylinder::new(
        Axis::new(pipe_origin, up).expect("a unit direction"),
        radius,
    )
    .expect("a positive radius");
    let half_of_the_rim = |facing: Vector| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Cylinder(drawn),
        picked_by: [Picker::from(
            HalfSpace::through_a_point_facing(centre, facing).expect("a facing with a length"),
        )]
        .into_iter()
        .collect(),
    };
    let mut builder = SolidBuilder::new();
    let [foot, rim, near, far] = [
        Point::new(2.0, 0.0, 0.0),
        Point::new(2.0, 0.0, 4.0),
        Point::new(radius, 0.0, 4.0),
        Point::new(-radius, 0.0, 4.0),
    ]
    .map(|at| builder.add_vertex(at).expect("a finite point"));
    let bottom_circle = builder
        .add_edge(foot, foot, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the bottom cap cuts the pipe in one closed curve");
    let seam = builder
        .add_edge(foot, rim, EdgeRun::Straight)
        .expect("a ruling of a pipe is straight");
    let top_circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the top cap cuts the pipe in one closed curve");
    let one_half = builder
        .add_edge(near, far, half_of_the_rim(Vector::new(0.0, 1.0, 0.0)))
        .expect("half of the drawn circle");
    let other_half = builder
        .add_edge(far, near, half_of_the_rim(Vector::new(0.0, -1.0, 0.0)))
        .expect("the other half of the drawn circle");

    let bottom = a_cap(
        &mut builder,
        Point::origin(),
        Vector::new(0.0, 0.0, -1.0),
        bottom_circle,
        Orientation::Reversed,
    );
    let side = a_wall(
        &mut builder,
        Surface::Cylinder(Cylinder::new(Axis::z(), 2.0).expect("a positive radius")),
        (bottom_circle, Orientation::Forward),
        seam,
        top_circle,
    );
    let top = Surface::Plane(Plane::from_point_and_normal(centre, up).expect("a unit normal"));
    let mut a_wire = |walked: &[(rcad_topo::DraftEdge, Orientation)]| {
        let steps: Vec<DraftStep> = walked
            .iter()
            .map(|&(edge, turn)| {
                DraftStep::Along(
                    builder
                        .add_coedge(edge, turn)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        builder.add_wire(&steps).expect("a wire that closes")
    };
    let round_the_cap = a_wire(&[(top_circle, Orientation::Forward)]);
    let round_the_hole = a_wire(&[
        (other_half, Orientation::Reversed),
        (one_half, Orientation::Reversed),
    ]);
    let round_the_disc = a_wire(&[
        (one_half, Orientation::Forward),
        (other_half, Orientation::Forward),
    ]);
    let cap = builder
        .add_face(top, &[round_the_cap, round_the_hole])
        .expect("the cap round the disc");
    let disc = builder
        .add_face(top, &[round_the_disc])
        .expect("the disc drawn on the cap");
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (side, Orientation::Forward),
            (cap, Orientation::Forward),
            (disc, Orientation::Forward),
        ])
        .expect("four faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// Q1293's and-Cylinder arm: a disc drawn by a pipe whose own axis starts
/// off the disc's own plane — at the bar's floor, not its centre — comes
/// back the same at no placement, moved alone, turned alone, and turned and
/// moved. The writer moves the written `Cylinder`'s axis to where it meets
/// the flat face (`Writing::an_arc_of_a_circle_on_a_flat_face`, `write.rs`),
/// so the reader's pipe keeps that moved origin; [`the_same_run`]'s
/// and-Cylinder arm must still call the two pipes the same because their
/// directions and radii match and the origin read back is where the kernel's
/// own walk centres the circle.
///
/// Turned and moved, that centre is off the kernel's line through
/// `(3, 5, 10)`, and no point an `f64` holds lies on that line where the top
/// holds it
/// ([`no_centre_on_a_turned_and_moved_discs_own_pipe_axis_is_one_its_top_holds`]).
#[test]
fn a_disc_drawn_by_a_pipe_off_its_own_plane_comes_back_from_its_own_file() {
    let moved_alone = Isometry::translation(Vector::new(3.0, 5.0, 10.0)).expect("a finite offset");
    let turned_alone =
        Isometry::rotation(&Axis::y(), 5.0).expect("a finite angle about a unit axis");
    let turned_and_moved = moved_alone.compose(&turned_alone);
    for placement in [
        Isometry::identity(),
        moved_alone,
        turned_alone,
        turned_and_moved,
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let solid =
            a_bar_with_a_disc_drawn_by_a_pipe_from(&mut model, Point::new(0.0, 0.0, 0.0), 1.0)
                .expect("a bar with a disc drawn by a pipe off its own plane fits this model");
        model
            .place_solid(solid, &placement)
            .expect("the key came from this model");
        let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
        let reached = one_model_decoded_and_written(&bytes);
        assert_eq!(
            (reached.came_back_the_same, reached.solids_the_same),
            (1, 1),
            "{placement:?}: {reached:?}, {:?}",
            rcad_step::write(&model, "a disc drawn by a pipe off its own plane").err()
        );
    }
}

/// The disc of [`a_disc_drawn_by_a_pipe_off_its_own_plane_comes_back_from_its_own_file`]
/// under each of the first 300 movements [`a_movement`] draws, which is how
/// the fuzzer places a solid: every one comes back the same. Until Q2135, a
/// turn off `z` together with a move could panic with a different edge run.
#[test]
fn every_turned_disc_drawn_by_a_pipe_off_its_top_comes_back_from_its_own_file() {
    for seed in 0..300 {
        let placement = a_movement(&mut Draw::new(seed));
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let solid =
            a_bar_with_a_disc_drawn_by_a_pipe_from(&mut model, Point::new(0.0, 0.0, 0.0), 1.0)
                .expect("a bar with a disc drawn by a pipe off its own plane fits this model");
        model
            .place_solid(solid, &placement)
            .expect("the key came from this model");
        let bytes = postcard::to_allocvec(&model).expect("a model always encodes");
        let reached = one_model_decoded_and_written(&bytes);
        assert_eq!(
            (reached.came_back_the_same, reached.solids_the_same),
            (1, 1),
            "{seed}, {placement:?}: {reached:?}"
        );
    }
}

/// **The writer cannot write the kernel's own pipe for the turned and moved
/// disc**, which is why [`the_same_run`] takes the origin it reads back at
/// where the kernel's walk centres the circle, and not only on the kernel's
/// line (`docs/decisions.md` §16.10).
///
/// The file names the pipe by a `CIRCLE`, and the reader builds the pipe
/// through the circle's centre, with the axis and radius it was written with,
/// and names the run by it only where the top holds that centre: within
/// `16` roundings of the largest number in the bound (`brep.rs`'s
/// `a_surface_holds_the_circle`). So the one way to bring the kernel's pipe
/// back is a centre an `f64` holds, exactly on the kernel's line, that close
/// to the top. Searched here with four times that room, along the whole
/// stretch of the line it allows, in integers: for the disc turned about `y`
/// by `5` and moved by `(3, 5, 10)` there is no such point, and of the 86
/// among the first 300 movements of
/// [`every_turned_disc_drawn_by_a_pipe_off_its_top_comes_back_from_its_own_file`]
/// whose centre lands off the line, 80 have none.
#[test]
fn no_centre_on_a_turned_and_moved_discs_own_pipe_axis_is_one_its_top_holds() {
    let the_rows_disc = Isometry::translation(Vector::new(3.0, 5.0, 10.0))
        .expect("a finite offset")
        .compose(&Isometry::rotation(&Axis::y(), 5.0).expect("a finite angle about a unit axis"));
    assert_eq!(
        centres_on_the_drawn_pipes_own_axis_its_top_could_hold(&the_rows_disc),
        Some(Vec::new())
    );
    let (mut off_the_line, mut with_none) = (0, 0);
    for seed in 0..300 {
        let placement = a_movement(&mut Draw::new(seed));
        let Some(centres) = centres_on_the_drawn_pipes_own_axis_its_top_could_hold(&placement)
        else {
            continue;
        };
        off_the_line += 1;
        with_none += usize::from(centres.is_empty());
    }
    println!(
        "{off_the_line} of 300 movements write a centre off the kernel's line, and {with_none} \
         of those have no centre on it that the top could hold"
    );
    assert_eq!((off_the_line, with_none), (86, 80));
}

/// Every point an `f64` holds that lies exactly on the axis of the drawn
/// pipe of [`a_bar_with_a_disc_drawn_by_a_pipe_from`] placed by `placement`,
/// close enough to its top for the top to hold it, or `None` where the
/// centre the writer works out lies on that axis in `f64` already.
fn centres_on_the_drawn_pipes_own_axis_its_top_could_hold(
    placement: &Isometry,
) -> Option<Vec<Point>> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_bar_with_a_disc_drawn_by_a_pipe_from(&mut model, Point::new(0.0, 0.0, 0.0), 1.0)
        .expect("a bar with a disc drawn by a pipe off its own plane fits this model");
    model
        .place_solid(solid, placement)
        .expect("the key came from this model");
    let (pipe, top) = the_drawn_pipe_and_the_top_it_is_drawn_on(&model);
    let (from, along) = (pipe.axis().origin(), pipe.axis().direction());
    let centre =
        where_the_walk_centres_a_circle(&top, &pipe).expect("the drawn pipe crosses its top");
    if (centre - from).cross(along) == Vector::zero() {
        return None;
    }
    let leans = top.normal().dot(along);
    let up = (centre - from).dot(along) / along.dot(along);
    let largest = centre
        .largest_coordinate_magnitude()
        .max(pipe.radius())
        .max(top.offset().abs());
    let room = 4.0 * 16.0 * rcad_geom::NEAREST_F64_POINT_ROUNDOFF * largest / leans.abs();
    Some(
        points_an_f64_holds_on_the_line_near(from, along, up, room)
            .expect("the room spans few enough f64s to try each"),
    )
}

/// The pipe that draws the disc of [`a_bar_with_a_disc_drawn_by_a_pipe_from`],
/// where `model` placed it, and the plane of the flat face it is drawn on.
fn the_drawn_pipe_and_the_top_it_is_drawn_on(model: &Model) -> (Cylinder, Plane) {
    the_one_solid(model)
        .faces()
        .find_map(|face| {
            let Surface::Plane(flat) = face.surface() else {
                return None;
            };
            face.wires().flat_map(WireView::coedges).find_map(|coedge| {
                let EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Cylinder(pipe),
                    ..
                } = coedge.edge().run()
                else {
                    return None;
                };
                Some((pipe, flat))
            })
        })
        .expect("a disc drawn by a pipe has an edge where the pipe crosses its flat face")
}

/// Every point an `f64` holds that lies exactly on the line `from + t·along`
/// for some `t` within `band` of `near`, worked out in integers; `None` where
/// there are more than a million to try along the coordinate that has
/// fewest, or a product outgrows an `i128`.
///
/// Each `f64` the line's coordinate `i` takes in the band names one `t`,
/// and the point at that `t` is one an `f64` holds when every other
/// coordinate `from_j + (x − from_i)·along_j / along_i` is an `f64` too. So
/// one coordinate is walked `f64` by `f64` and the others are tested, which
/// finds every such point there is.
fn points_an_f64_holds_on_the_line_near(
    from: Point,
    along: Vector,
    near: f64,
    band: f64,
) -> Option<Vec<Point>> {
    let [from, along] = [from.coordinates(), along.components()];
    let how_many_along = |i: usize| {
        let ends = [near - band, near + band].map(|t| from[i] + t * along[i]);
        let nearest_nought = if ends[0].signum() == ends[1].signum() {
            ends[0].abs().min(ends[1].abs())
        } else {
            0.0
        };
        (ends[1] - ends[0]).abs() / (nearest_nought.next_up() - nearest_nought)
    };
    let walked = (0..3)
        .filter(|&i| along[i] != 0.0)
        .min_by(|&a, &b| how_many_along(a).total_cmp(&how_many_along(b)))
        .expect("a direction is not nothing");
    let ends = [near - band, near + band].map(|t| from[walked] + t * along[walked]);
    let [low, high] = [
        ends[0].min(ends[1]).next_down(),
        ends[0].max(ends[1]).next_up(),
    ];
    let mut points = Vec::new();
    let mut x = low;
    let mut tried = 0;
    while x <= high {
        tried += 1;
        if tried > 1_000_000 {
            return None;
        }
        if ((x - from[walked]) / along[walked] - near).abs() > band {
            x = x.next_up();
            continue;
        }
        let mut point = [0.0; 3];
        point[walked] = x;
        let mut held = true;
        for other in (0..3).filter(|&j| j != walked) {
            match the_coordinate_there(from, along, walked, x, other)? {
                Some(value) => point[other] = value,
                None => {
                    held = false;
                    break;
                }
            }
        }
        if held {
            points.push(Point::new(point[0], point[1], point[2]));
        }
        x = x.next_up();
    }
    Some(points)
}

/// Coordinate `other` of the point on the line where coordinate `walked` is
/// `x`, as `Some(Some(value))` where an `f64` holds it exactly, `Some(None)`
/// where none does, and `None` where a product outgrows an `i128`.
fn the_coordinate_there(
    from: [f64; 3],
    along: [f64; 3],
    walked: usize,
    x: f64,
    other: usize,
) -> Option<Option<f64>> {
    let gone = Dyadic::of(x).plus(Dyadic::of(-from[walked]))?;
    let [over, by] = [Dyadic::of(along[walked]), Dyadic::of(along[other])];
    let top = gone.whole.checked_mul(by.whole)?;
    if top % over.whole != 0 {
        return Some(None);
    }
    let moved = Dyadic {
        whole: top / over.whole,
        power: gone.power + by.power - over.power,
    };
    Some(moved.plus(Dyadic::of(from[other]))?.as_f64())
}

/// `whole · 2^power`, exactly.
#[derive(Clone, Copy)]
struct Dyadic {
    whole: i128,
    power: i32,
}

impl Dyadic {
    fn of(value: f64) -> Self {
        assert!(value.is_finite(), "{value} is a number");
        if value == 0.0 {
            return Self { whole: 0, power: 0 };
        }
        let bits = value.to_bits();
        let stored = i32::try_from((bits >> 52) & 0x7ff).expect("eleven bits");
        let fraction = i128::from(bits & ((1 << 52) - 1));
        let (whole, power) = if stored == 0 {
            (fraction, -1074)
        } else {
            (fraction | (1 << 52), stored - 1075)
        };
        let signed = if bits >> 63 == 1 { -whole } else { whole };
        Self {
            whole: signed,
            power,
        }
        .reduced()
    }

    fn reduced(self) -> Self {
        if self.whole == 0 {
            return Self { whole: 0, power: 0 };
        }
        let zeros = self.whole.trailing_zeros();
        Self {
            whole: self.whole >> zeros,
            power: self.power + i32::try_from(zeros).expect("under 128"),
        }
    }

    fn plus(self, other: Self) -> Option<Self> {
        let power = self.power.min(other.power);
        let lifted = |number: Self| {
            let shift = u32::try_from(number.power - power).ok()?;
            if shift > 120 {
                return None;
            }
            number.whole.checked_mul(1_i128 << shift)
        };
        Some(
            Self {
                whole: lifted(self)?.checked_add(lifted(other)?)?,
                power,
            }
            .reduced(),
        )
    }

    fn as_f64(self) -> Option<f64> {
        let bits = 128 - self.whole.unsigned_abs().leading_zeros();
        let fits = self.whole == 0
            || (bits <= 53
                && self.power >= -1074
                && self.power + i32::try_from(bits).expect("under 129") <= 1024);
        fits.then(|| self.whole as f64 * 2_f64.powi(self.power))
    }
}

/// The same disc's drawn pipe, compared against one of another radius about
/// the same axis, is still a different run: [`the_same_run`]'s and-Cylinder
/// arm only calls two pipes the same when their radii match to the bit.
#[test]
fn a_disc_drawn_by_a_pipe_of_another_radius_is_a_different_run() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar =
        a_bar_with_a_disc_drawn_on_its_top(&mut model).expect("a bar with a disc fits this model");
    model
        .place_solid(bar, &Isometry::identity())
        .expect("the key came from this model");
    let drawn = a_drawn_discs_half_circle(&model);
    let here = drawn.edge().run();
    let EdgeRun::PartOfWhereItCrosses {
        and: Surface::Cylinder(pipe),
        picked_by,
    } = &here
    else {
        panic!("the disc's half circle is part of where its pipe crosses the cap, not {here:?}");
    };
    let another_radius = EdgeRun::PartOfWhereItCrosses {
        and: Surface::Cylinder(
            Cylinder::new(pipe.axis(), pipe.radius() + 0.5).expect("a positive radius"),
        ),
        picked_by: picked_by.clone(),
    };
    let corners = [drawn.start().point(), drawn.end().point()];
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a unit normal"),
    );
    assert!(the_same_run(&here, &here, corners, cap));
    assert!(!the_same_run(&here, &another_radius, corners, cap));
}

/// One of the four coedges that walk the circle drawn on
/// [`a_bar_with_a_disc_drawn_on_its_top`]'s cap: part of where the drawn
/// pipe crosses the flat face.
fn a_drawn_discs_half_circle(model: &Model) -> CoedgeView<'_> {
    the_one_solid(model)
        .faces()
        .flat_map(|face| face.wires().flat_map(WireView::coedges).collect::<Vec<_>>())
        .find(|coedge| {
            matches!(
                coedge.edge().run(),
                EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Cylinder(_),
                    ..
                }
            )
        })
        .expect("a disc drawn on a bar has one circle, where its pipe crosses the cap")
}

/// A flat face through `at`, facing `outward`, trimmed by one closed edge.
fn a_cap(
    builder: &mut SolidBuilder,
    at: Point,
    outward: Vector,
    circle: rcad_topo::DraftEdge,
    turn: Orientation,
) -> rcad_topo::DraftFace {
    let plane = Plane::from_point_and_normal(at, outward).expect("a coordinate direction");
    let coedge = builder
        .add_coedge(circle, turn)
        .expect("an edge of this build");
    let wire = builder
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("one closed edge is a wire");
    builder
        .add_face(Surface::Plane(plane), &[wire])
        .expect("a cap is a plane trimmed by its circle")
}

/// The curved face between two closed edges: round the first the way it is
/// used, along the seam, back round the second the other way, and back down
/// the seam — the seam's two coedges in one wire, which is what makes it a
/// seam.
fn a_wall(
    builder: &mut SolidBuilder,
    surface: Surface,
    (first, round_the_first): (rcad_topo::DraftEdge, Orientation),
    seam: rcad_topo::DraftEdge,
    second: rcad_topo::DraftEdge,
) -> rcad_topo::DraftFace {
    let round_the_wall: Vec<DraftStep> = [
        (first, round_the_first),
        (seam, Orientation::Forward),
        (second, round_the_first.reversed()),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round one end, along the seam, round the other, and back");
    builder
        .add_face(surface, &[wire])
        .expect("a face on the curved surface")
}

/// Numbers drawn from the seed libFuzzer hands the mutator, so a mutation is
/// the same every time its seed is — except libFuzzer's own byte mutation,
/// which draws from libFuzzer's generator and not from this seed.
struct Draw(u64);

impl Draw {
    fn new(seed: u32) -> Self {
        Self(u64::from(seed))
    }

    /// SplitMix64: every seed, even zero, gives a well-mixed stream.
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, count: u64) -> u64 {
        self.next() % count
    }

    /// A whole number from `low` to `high`, both included.
    fn whole(&mut self, low: i32, high: i32) -> f64 {
        let span = u64::try_from(high - low + 1).expect("low is not above high");
        f64::from(low) + self.below(span) as f64
    }

    /// A number from zero up to but not including one.
    fn fraction(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }
}
