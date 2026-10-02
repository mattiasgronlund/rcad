//! A solid the kernel built, written out as ISO 10303-21 and read back, is the
//! same solid.
//!
//! `docs/decisions.md` §42.12. The reader landed with no oracle of its own: it
//! was checked against real files and against `step-io` at the *parse* layer,
//! which leaves the geometry mapping checked by eye and by published
//! dimensions. This is the missing oracle — generate a solid this kernel
//! already knows how to build, write it, read it back, and require the same
//! answer.
//!
//! **What a round trip cannot check, said first.** It checks this writer
//! against this reader, so a misunderstanding of AP203 the two share cancels
//! out and passes green. Two things stand against that and neither is this
//! file: `tests/written_by_hand/` holds files nothing in this crate produced,
//! and `a_second_reader_reads_what_this_writer_wrote.rs` puts these same bytes
//! through `step-io`.
//!
//! **The oracles, strongest first** (the testing skill's table), and they are
//! `building_a_solid_out_of_faces.rs`'s: volume, area and centroid compared
//! **bit for bit**, and then the same `Rule::Difference` against a third solid
//! with every piece's volume, area and centroid compared bit for bit and the
//! piece count compared. Two solids that measure the same can still fuse
//! differently, because a fuse reads topology the sums never touch.
//!
//! **Why the model is built at the scale the file will come back at.** A
//! [`Scale`] is the largest coordinate magnitude and the unit, and
//! [`Import`] sets it from the file it read. Building the original at that same
//! scale is what makes the comparison one of *shapes*: two models at different
//! scales would differ in `allowed_wobble`, which is an input to the fuse, and
//! a difference there would be a difference in the question rather than in the
//! answer. The test asserts the two scales came out equal rather than assuming
//! it.

use rcad_geom::{Deflection, EdgeRun, Isometry, Plane, Point, Scale, Surface, Unit, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{CannotWrite, Import, StepFile};
use rcad_topo::{EdgeKey, Measurement, Model, ProfileBuilder, Rule, SolidKey};

/// How fine the mesh the measurement sums over is.
///
/// The **same** number on both sides, and given rather than taken from either
/// model's scale: a measurement is a sum over a mesh, and comparing two sums
/// bit for bit means asking for the same mesh.
fn the_same_mesh_on_both_sides() -> Deflection {
    Deflection::of(0.01).expect("a hundredth of the smallest shape here")
}

/// A cube of side two: six planes and twelve straight edges, every one of them
/// named by the two planes either side of it.
#[test]
fn a_cube_written_out_and_read_back_is_the_same_cube() {
    let mut covered = Covered::new();
    covered.the_same_solid_comes_back(
        "a cube of side two",
        Scale::new(Unit::Millimetre, 2.0).expect("a working size"),
        &|model| {
            model
                .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a cube of side two")
        },
        // The corner octant, on a whole-number lattice.
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, 1.0, 1.0)),
    );
    covered.report("a cube");
}

/// A block pushed out of a profile, which is the other way this kernel makes a
/// plane-bounded solid.
#[test]
fn a_block_pushed_out_of_a_profile_written_out_and_read_back_is_the_same_block() {
    let mut covered = Covered::new();
    covered.the_same_solid_comes_back(
        "a block three by five by seven",
        Scale::new(Unit::Millimetre, 7.0).expect("a working size"),
        &|model| a_block(model, [3.0, 5.0, 7.0], Vector::zero()),
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(2.0, 4.0, 6.0)),
    );
    covered.report("a block pushed out of a profile");
}

/// The one with a curved face: a pipe, two cap circles the caps' own planes
/// name, and a seam the pipe is on both sides of.
#[test]
fn a_round_bar_written_out_and_read_back_is_the_same_round_bar() {
    let mut covered = Covered::new();
    covered.the_same_solid_comes_back(
        "a bar of radius two, eight tall",
        Scale::new(Unit::Millimetre, 8.0).expect("a working size"),
        &|model| {
            model
                .add_round_bar(2.0, 8.0, &Isometry::identity())
                .expect("a bar of radius two")
        },
        // `a_notch_across_the_seam` in `degenerate_curved_fuses.rs` is this
        // cut: it runs across the bar's seam at `(2, 0, z)`, so the comparison
        // reaches the one piece of topology a round trip is most likely to get
        // wrong.
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 3.0)),
    );
    covered.report("a round bar");
}

/// The same bar moved along `y` by amounts down to the smallest `f64`, written
/// out, read back and found the same, corner for corner and bit for bit.
///
/// `docs/decisions.md` §246. The file carries each move in every corner, so the
/// bar read back has its bottom cap's corner at `v = d` on the plane `z = 0`,
/// where the bar that went in is meshed in its placement's inverse and hands
/// the triangulator zero. Spade takes nothing under `2⁻¹⁴²` that is not zero,
/// and until the triangulator flushed such a coordinate to zero every move
/// from `2⁻¹⁴³` down was refused on the way back in; §246.3's table is these
/// moves. `2⁻¹⁰⁰⁷` is the one the fuzzer found, and `2⁻¹⁰⁷⁴` is subnormal.
///
/// Not [`Covered::the_same_solid_comes_back`], because the cut it compares is a
/// fuse, and a fuse with this moved bar refuses for reasons of its own
/// (§246.6).
#[test]
fn a_round_bar_moved_by_less_than_two_to_the_minus_142_comes_back_the_same() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    for exponent in [100_u64, 141, 142, 143, 200, 1007, 1022] {
        let moved_by = f64::from_bits((1023 - exponent) << 52);
        for moved_by in [moved_by, -moved_by] {
            a_moved_bar_comes_back_the_same(moved_by, &policy, &mut log);
        }
    }
    for subnormal in [1_u64 << 51, 1] {
        a_moved_bar_comes_back_the_same(f64::from_bits(subnormal), &policy, &mut log);
    }
}

fn a_moved_bar_comes_back_the_same(moved_by: f64, policy: &Policy, log: &mut ProvenanceLog) {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 8.0).expect("a working size"));
    let bar = model
        .add_round_bar(2.0, 8.0, &Isometry::identity())
        .expect("a bar of radius two");
    let move_it = Isometry::translation(Vector::new(0.0, moved_by, 0.0)).expect("a move");
    model
        .place_solid(bar, &move_it)
        .expect("a bar can be moved");
    model
        .validate(policy, log)
        .unwrap_or_else(|why| panic!("the bar moved by {moved_by:e} is not sound: {why}"));

    let text = rcad_step::write(&model, "a moved bar")
        .unwrap_or_else(|why| panic!("the bar moved by {moved_by:e} is not written: {why}"));
    let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
    let import = Import::read(&file, policy, log).unwrap_or_else(|why| {
        panic!("the bar moved by {moved_by:e} is refused on the way back in: {why}")
    });
    let [(_, read_back)] = import.solids() else {
        panic!("one bar came back as {}", import.solids().len());
    };
    let there = import.model();
    there
        .validate(policy, log)
        .unwrap_or_else(|why| panic!("the bar moved by {moved_by:e} comes back unsound: {why}"));

    let went = model.solid(bar).expect("this model made the key");
    let came = there.solid(*read_back).expect("this model made the key");
    assert_eq!(
        where_its_corners_are(went),
        where_its_corners_are(came),
        "the bar moved by {moved_by:e} comes back with its corners somewhere else"
    );
    assert!(
        where_its_corners_are(went).contains(&[2.0, moved_by, 0.0].map(f64::to_bits)),
        "the bar was not moved by {moved_by:e} at all, so this compared two bars at the origin"
    );

    let deflection = the_same_mesh_on_both_sides();
    let here = went
        .measure(deflection, policy, log)
        .unwrap_or_else(|why| panic!("the bar moved by {moved_by:e} does not measure: {why}"));
    let back = came.measure(deflection, policy, log).unwrap_or_else(|why| {
        panic!("the bar moved by {moved_by:e} does not measure after the round trip: {why}")
    });
    assert_eq!(
        here.volume(),
        back.volume(),
        "volume, moved by {moved_by:e}"
    );
    assert_eq!(
        here.surface_area(),
        back.surface_area(),
        "area, moved by {moved_by:e}"
    );
    assert_eq!(
        here.centroid(),
        back.centroid(),
        "centroid, moved by {moved_by:e}"
    );
}

/// Every corner of every wire of a solid, where the solid puts it, as bits and
/// in order.
fn where_its_corners_are(solid: rcad_topo::SolidView<'_>) -> Vec<[u64; 3]> {
    let mut corners: Vec<[u64; 3]> = solid
        .faces()
        .flat_map(|face| face.wires().collect::<Vec<_>>())
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .map(|coedge| coedge.start().point().coordinates().map(f64::to_bits))
        .collect();
    corners.sort_unstable();
    corners.dedup();
    corners
}

/// A ball: one face, a seam from pole to pole, and two poles the wire stands
/// still at.
///
/// A ball's one wire is four steps, and two of them stand still on a pole.
/// The file leaves both poles unwritten and walks the seam there and back, the
/// way `kicad/LED_D5.0mm.step` writes its dome, and the reader puts a pole
/// back at each turn. The seam is half of a great circle, which is written as
/// the `CIRCLE` of the ball's own centre and radius turned the way that picks
/// the half the model's seam is.
#[test]
fn a_ball_written_out_and_read_back_is_the_same_ball() {
    let mut covered = Covered::new();
    covered.the_same_solid_comes_back(
        "a ball of radius four",
        Scale::new(Unit::Millimetre, 4.0).expect("a working size"),
        &|model| {
            model
                .add_ball(4.0, &Isometry::identity())
                .expect("a ball of radius four")
        },
        // A block under the ball whose top is the equator's plane: it cuts the
        // seam in two at `(4, 0, 0)`, so the comparison reaches the seam a
        // round trip rewrote.
        &|model| a_block(model, [10.0, 10.0, 5.0], Vector::new(-5.0, -5.0, -5.0)),
    );
    covered.report("a ball");
}

/// A ball turned so its poles leave the `z` line through its centre is
/// refused, because this kernel's sphere pinches only on that line.
///
/// The reader would put each pole back where it was written, which is where
/// the ball it reads does not pinch, and `Model::validate` refuses such a pole.
/// A ball moved without a turn is still written: its poles stay on the line.
#[test]
fn a_ball_turned_off_z_is_refused_at_its_pole() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let turned = Isometry::rotation(&rcad_geom::Axis::x(), 0.5).expect("a finite angle");
    model.add_ball(4.0, &turned).expect("a ball of radius four");

    let refused = rcad_step::write(&model, "a turned ball");
    assert!(
        matches!(refused, Err(CannotWrite::AWireThatStandsStillAtAPole)),
        "expected the pole to stop it, got {refused:?}"
    );
}

/// A bar cut off by a slanted plane meets it in an **ellipse**, and it is
/// written as one and comes back the same bar.
///
/// The plane leans `θ` off square with `tan θ = 1/2`, so the ellipse is the
/// pipe's radius `2` one way and `2/cos θ = √5` the other. Until Q409 the
/// writer refused every cap that was not exactly square to its pipe.
#[test]
fn a_bar_with_a_slanted_cap_is_written_with_an_ellipse_and_comes_back_the_same() {
    let build = |model: &mut Model| {
        let flat =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, -1.0))
                .expect("the z axis is a unit normal");
        let slanted =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 8.0), Vector::new(0.0, 1.0, 2.0))
                .expect("a normal with a length");
        model
            .add_bar_cut_off_by(
                2.0,
                [flat, slanted],
                &Isometry::identity(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar with one slanted cap")
    };
    let mut model = Model::new(Scale::new(Unit::Millimetre, 8.0).expect("a working size"));
    build(&mut model);
    let text = rcad_step::write(&model, "a bar with a slanted cap")
        .unwrap_or_else(|why| panic!("the slanted cap is not written: {why}"));
    assert_eq!(
        text.matches("ELLIPSE(").count(),
        1,
        "one cap leans and one is square, so one ellipse: {text}"
    );

    let mut covered = Covered::new();
    covered.the_same_solid_comes_back(
        "a bar of radius two with a slanted cap",
        Scale::new(Unit::Millimetre, 8.0).expect("a working size"),
        &build,
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 3.0)),
    );
    covered.report("a bar with a slanted cap");
}

/// A bar whose axis leans off its caps' normals by a subnormal angle is
/// written with a circle about each cap's normal and comes back the same bar.
///
/// The writer fuzzer's byte-nudged bars had such axes (Q409): an `x` part of
/// `4.78e-299`, `6.2e-320` or `8e-323` and caps square to `z`. The cap's normal
/// then crosses the axis to a vector that is not zero, which the writer used to
/// refuse. At that lean the ellipse the cap cuts has two sizes that round to
/// the same `f64`, so it is a circle in the cap's plane.
///
/// **Two round trips, and the second is the one compared bit for bit.** The bar
/// is built turned, and a turned solid is measured in its own frame, where its
/// caps lean and its axis does not; the bar read back has no turn, so its mesh
/// is another mesh and its centroid differs in the sixteenth digit for that
/// reason alone. The bar read back from the first file is the same shape
/// without the turn — it is checked to lean the same way — and it is written
/// and read back again and compared corner for corner and bit for bit.
///
/// Not [`Covered::the_same_solid_comes_back`], because a fuse with this bar
/// refuses for a reason of its own: the cut's crossing with the leaning pipe
/// comes in two pieces no `f64` half-space tells apart. It is required to
/// refuse the same way on both sides.
#[test]
fn a_bar_whose_axis_leans_off_its_caps_by_a_subnormal_comes_back_the_same() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let round_trip = |called: &str, model: &Model, log: &mut ProvenanceLog| {
        let text = rcad_step::write(model, called)
            .unwrap_or_else(|why| panic!("{called} is not written: {why}"));
        let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
        let import = Import::read(&file, &policy, log)
            .unwrap_or_else(|why| panic!("{called} is refused on the way back in: {why}"));
        let [(_, read_back)] = import.solids() else {
            panic!("{called} came back as {} solids", import.solids().len());
        };
        let read_back = *read_back;
        import
            .model()
            .validate(&policy, log)
            .unwrap_or_else(|why| panic!("{called} comes back unsound: {why}"));
        (import.model().clone(), read_back)
    };
    for lean in [4.78e-299, 6.2e-320, 8e-323] {
        let called = format!("a bar leaning {lean:e} off its caps");
        let mut turned = Model::new(Scale::new(Unit::Millimetre, 8.0).expect("a working size"));
        let bar = a_bar_leaning_off_its_caps(&mut turned, lean);
        the_axis_leans_off_a_cap_by(&turned, bar, lean);

        let (once, went) = round_trip(&called, &turned, &mut log);
        the_axis_leans_off_a_cap_by(&once, went, lean);
        let (twice, came) = round_trip(&called, &once, &mut log);

        let [went, came] = [(&once, went), (&twice, came)]
            .map(|(model, key)| model.solid(key).expect("this model made the key"));
        assert_eq!(
            where_its_corners_are(went),
            where_its_corners_are(came),
            "{called}: corners"
        );
        let deflection = the_same_mesh_on_both_sides();
        let [here, back] = [went, came].map(|solid| {
            solid
                .measure(deflection, &policy, &mut log)
                .unwrap_or_else(|why| panic!("{called} does not measure: {why}"))
        });
        assert_eq!(here.volume(), back.volume(), "{called}: volume");
        assert_eq!(here.surface_area(), back.surface_area(), "{called}: area");
        assert_eq!(here.centroid(), back.centroid(), "{called}: centroid");

        let cutter =
            |model: &mut Model| a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 3.0));
        let [cut_here, cut_back] =
            [(&once, went.key()), (&twice, came.key())].map(|(model, key)| {
                cut_by_a_third_solid(model, key, &cutter, deflection)
                    .map(|pieces| pieces.len())
                    .map_err(|why| why.to_string())
            });
        assert_eq!(cut_here, cut_back, "{called}: the cut");
    }
}

/// A bar of radius two, eight tall, turned by `lean` about `y`, whose caps
/// are leaned back by `lean` before the turn so that they end up square to `z`
/// while the pipe's axis does not.
fn a_bar_leaning_off_its_caps(model: &mut Model, lean: f64) -> SolidKey {
    let bottom = Plane::from_point_and_normal(Point::origin(), Vector::new(lean, 0.0, -1.0))
        .expect("a normal with a length");
    let top = Plane::from_point_and_normal(Point::new(0.0, 0.0, 8.0), Vector::new(-lean, 0.0, 1.0))
        .expect("a normal with a length");
    let turned = Isometry::rotation(&rcad_geom::Axis::y(), lean).expect("a finite angle");
    model
        .add_bar_cut_off_by(
            2.0,
            [bottom, top],
            &turned,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar leaning off its caps")
}

/// That the bar built is the shape the test says: the pipe's axis has a part
/// off `z` no bigger than `lean`, and some cap's normal crosses it to a vector
/// that is not zero, which is the comparison the writer used to refuse on.
fn the_axis_leans_off_a_cap_by(model: &Model, bar: SolidKey, lean: f64) {
    let surfaces: Vec<Surface> = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .collect();
    let pipes: Vec<_> = surfaces
        .iter()
        .filter(|surface| surface.as_plane().is_none())
        .collect();
    let [Surface::Cylinder(pipe)] = pipes.as_slice() else {
        panic!("a bar has one curved face and it is a pipe, not {pipes:?}");
    };
    let axis = pipe.axis().direction();
    let [x, y, _] = axis.components();
    assert!(
        (x != 0.0 || y != 0.0) && x.abs().max(y.abs()) <= lean,
        "the axis {axis:?} does not lean by a part of {lean:e}"
    );
    let leaning_caps = surfaces
        .iter()
        .filter_map(Surface::as_plane)
        .filter(|flat| flat.normal().cross(axis) != Vector::zero())
        .count();
    assert!(
        leaning_caps > 0,
        "no cap's normal leans off the axis {axis:?}, so this would not reach the comparison"
    );
}

/// The short side of a bar sliced along its length is written with an arc
/// where each cap stops short of right round the pipe, and comes back the same.
///
/// The pipe has radius five and the slice is the plane `y = −3`, so the two
/// corners of each cap are at `(±4, −3)` and on the pipe exactly. This side
/// keeps the short arc of each cap's circle, under the slice, and the long side
/// below keeps the long one: the two arcs with the same two ends, so the arc
/// written has to be the one the model holds and not its sibling. Until Q766
/// the writer refused both: it named a plane and a pipe only for an edge that
/// goes right round.
///
/// **Both orders of the two faces**, because the writer is handed them in the
/// order the solid lists its faces and matches on that order. The bar is built
/// face by face with its bottom cap listed before its pipe and its top cap
/// after, so it hands the writer one arc flat face first and one pipe first,
/// and the test counts that.
#[test]
fn the_short_side_of_a_bar_sliced_along_its_length_is_written_with_arcs_and_comes_back_the_same() {
    let scale = Scale::new(Unit::Millimetre, 4.0).expect("a working size");
    let build = |model: &mut Model| a_bar_sliced_along_its_length(model, Side::Short);
    let mut model = Model::new(scale);
    let built = build(&mut model);
    assert_eq!(
        arcs_between_a_flat_face_and_a_pipe(&model, built),
        [1, 1],
        "one arc flat face first and one pipe first"
    );

    let mut covered = Covered::new();
    covered.the_same_solid_comes_back(
        "the short side of a bar sliced along its length",
        scale,
        &build,
        // Across the top cap's arc, where the pipe is furthest from the slice.
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(-1.0, -6.0, 3.0)),
    );
    covered.report("the short side of a bar sliced along its length");
}

/// The long side of the same bar is written with its two long arcs and comes
/// back the same solid, corner for corner and bit for bit — **and the fuse
/// then refuses the copy read back where it cuts the original**, which is the
/// reader's gap and not this writer's.
///
/// The model names each long arc by a half-space whose plane is the slice, so
/// both corners are on it. The reader names an arc by a half-space through the
/// corners that it lowers by a few roundings so that both are sure to be in
/// front (`Walk::the_arc_between`). For a short arc that is a picker the fuse
/// reads, because the circle's centre is behind it. For a long arc the centre
/// is in front too, and `a_picker_that_tells_the_two_arcs_apart` in
/// `rcad-topo` reads a picker only when its plane holds both ends or the centre
/// is not in front, so the cut stops at
/// [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`]. The test
/// checks both pickers on both sides, so it is not the writer's arc that
/// fails here: the arc the copy holds is the one that went out, measured the
/// same.
#[test]
fn the_long_side_of_a_bar_sliced_along_its_length_comes_back_the_same_and_its_long_arcs_stop_a_fuse()
 {
    use rcad_geom::Picker;
    use rcad_predicates::{Decision, Point as PredicatePoint};

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let built = a_bar_sliced_along_its_length(&mut model, Side::Long);
    assert_eq!(
        arcs_between_a_flat_face_and_a_pipe(&model, built),
        [1, 1],
        "one arc flat face first and one pipe first"
    );

    let text = rcad_step::write(&model, "the long side of a bar sliced along its length")
        .unwrap_or_else(|why| panic!("the long side is not written: {why}"));
    let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
    let import = Import::read(&file, &policy, &mut log)
        .unwrap_or_else(|why| panic!("the long side is refused on the way back in: {why}"));
    let [(_, came)] = import.solids() else {
        panic!("one solid came back as {}", import.solids().len());
    };
    let came = *came;
    let there = import.model();
    there
        .validate(&policy, &mut log)
        .unwrap_or_else(|why| panic!("the long side comes back unsound: {why}"));
    assert_eq!(there.scale(), model.scale(), "the scale moved");

    let [went_solid, came_solid] = [(&model, built), (there, came)]
        .map(|(from, key)| from.solid(key).expect("this model made the key"));
    assert_eq!(
        where_its_corners_are(went_solid),
        where_its_corners_are(came_solid),
        "corners"
    );
    let deflection = the_same_mesh_on_both_sides();
    let [here, back] = [went_solid, came_solid].map(|solid| {
        solid
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| panic!("the long side does not measure: {why}"))
    });
    assert_eq!(here.volume(), back.volume(), "volume");
    assert_eq!(here.surface_area(), back.surface_area(), "area");
    assert_eq!(here.centroid(), back.centroid(), "centroid");

    let where_the_arcs_ends_and_centre_are = |model: &Model, solid: SolidKey| {
        let mut sides = Vec::new();
        for face in model.solid(solid).expect("this model made the key").faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    let edge = coedge.edge();
                    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = edge.run() else {
                        continue;
                    };
                    let [Picker::InFrontOf(half)] = picked_by.as_slice() else {
                        panic!("a long arc picked by {picked_by:?}");
                    };
                    let [start, end] = [edge.start().point(), edge.end().point()];
                    let centre = Point::new(0.0, 0.0, start.z());
                    sides.push(
                        [start, end, centre]
                            .map(|at| half.side_of_point(PredicatePoint::from(at.coordinates()))),
                    );
                }
            }
        }
        sides
    };
    let built_sides = where_the_arcs_ends_and_centre_are(&model, built);
    let read_sides = where_the_arcs_ends_and_centre_are(there, came);
    assert_eq!(built_sides.len(), 4, "two arcs, each walked by two faces");
    assert!(
        built_sides
            .iter()
            .all(|[start, end, _]| *start == Decision::Zero && *end == Decision::Zero),
        "the model's own pickers hold both ends of each arc: {built_sides:?}"
    );
    assert_eq!(read_sides.len(), 4, "two arcs, each walked by two faces");
    assert!(
        read_sides
            .iter()
            .all(|sides| sides.iter().all(|side| *side == Decision::Pos)),
        "the reader's pickers have both ends and the centre in front: {read_sides:?}"
    );

    let cutter = |model: &mut Model| a_block(model, [2.0, 3.0, 2.0], Vector::new(-1.0, 3.0, 3.0));
    let cut_here = cut_by_a_third_solid(&model, built, &cutter, deflection)
        .unwrap_or_else(|why| panic!("the long side is not cut: {why}"));
    assert!(!cut_here.is_empty(), "the cut left nothing");
    let cut_back = cut_by_a_third_solid(there, came, &cutter, deflection);
    assert!(
        matches!(
            cut_back,
            Err(rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow { .. })
        ),
        "the copy read back was expected to stop the cut where the reader's picker is one \
         the fuse does not read, and got {:?}; if it now cuts, compare the pieces here",
        cut_back.map(|pieces| pieces.len())
    );
}

/// Which side of the slice [`a_bar_sliced_along_its_length`] keeps.
#[derive(Clone, Copy)]
enum Side {
    /// Over `y = −3`, with the long arc of each cap's circle.
    Long,
    /// Under it, with the short arc.
    Short,
}

/// A bar of radius five about `z`, four tall, sliced along its length by the
/// plane `y = −3` and kept on `side`, built face by face: bottom cap, pipe, top
/// cap, slice.
fn a_bar_sliced_along_its_length(model: &mut Model, side: Side) -> SolidKey {
    use rcad_geom::{Axis, Cylinder, HalfSpace, Orientation, Picker};
    use rcad_topo::{DraftStep, SolidBuilder};

    let height = 4.0;
    let over = match side {
        Side::Long => 1.0,
        Side::Short => -1.0,
    };
    let a_plane = |at: Point, facing: Vector| {
        Plane::from_point_and_normal(at, facing).expect("an axis direction has length")
    };
    let bottom = a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0));
    let top = a_plane(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0));
    let slice = a_plane(Point::new(0.0, -3.0, 0.0), Vector::new(0.0, -over, 0.0));
    let pipe = Cylinder::new(Axis::z(), 5.0).expect("a positive radius");
    let the_arc_on = |cap: Plane| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cap),
        picked_by: [Picker::from(
            HalfSpace::through_a_point_facing(
                Point::new(0.0, -3.0, 0.0),
                Vector::new(0.0, over, 0.0),
            )
            .expect("an axis direction has length"),
        )]
        .into(),
    };

    let mut builder = SolidBuilder::new();
    let corners = [(4.0, 0.0), (-4.0, 0.0), (4.0, height), (-4.0, height)].map(|(x, z)| {
        builder
            .add_vertex(Point::new(x, -3.0, z))
            .expect("a finite corner")
    });
    let edges = [
        (0, 1, the_arc_on(bottom)),
        (1, 0, EdgeRun::Straight),
        (2, 3, the_arc_on(top)),
        (3, 2, EdgeRun::Straight),
        (0, 2, EdgeRun::Straight),
        (1, 3, EdgeRun::Straight),
    ]
    .map(|(from, to, run)| {
        builder
            .add_edge(corners[from], corners[to], run)
            .expect("an edge between two corners")
    });

    let (forward, back) = (Orientation::Forward, Orientation::Reversed);
    let wires: [&[(usize, Orientation)]; 4] = match side {
        Side::Long => [
            &[(1, back), (0, back)],
            &[(0, forward), (5, forward), (2, back), (4, back)],
            &[(2, forward), (3, forward)],
            &[(1, forward), (4, forward), (3, back), (5, back)],
        ],
        Side::Short => [
            &[(0, forward), (1, forward)],
            &[(0, back), (4, forward), (2, forward), (5, back)],
            &[(2, back), (3, back)],
            &[(1, back), (5, forward), (3, forward), (4, back)],
        ],
    };
    let surfaces = [
        Surface::Plane(bottom),
        Surface::Cylinder(pipe),
        Surface::Plane(top),
        Surface::Plane(slice),
    ];
    let mut skin = Vec::with_capacity(surfaces.len());
    for (surface, steps) in surfaces.into_iter().zip(wires) {
        let walked: Vec<DraftStep> = steps
            .iter()
            .map(|&(which, way)| {
                DraftStep::Along(builder.add_coedge(edges[which], way).expect("a coedge"))
            })
            .collect();
        let wire = builder.add_wire(&walked).expect("a wire that closes");
        skin.push((
            builder.add_face(surface, &[wire]).expect("a face"),
            Orientation::Forward,
        ));
    }
    builder.add_shell(&skin).expect("a shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the bar sliced along its length is not sound: {why}"))
}

/// A slab with a round scoop out of its top, whose curved face is a pipe used
/// the other way round, is written with each loop closing and comes back the
/// same.
///
/// The slab is `|x| ≤ 4`, `−6 ≤ y ≤ −3`, four tall, less the pipe of radius
/// five about `z`, which takes a scoop down to `y = −5`. Its curved face has the
/// material outside the pipe, so the shell uses it reversed, and its one wire
/// is four edges: the two short arcs and two straight sides. A wire of a face
/// used reversed comes out of `WireView::steps` walked the other way, and the
/// writer used to write that walk with each coedge's own way along its edge,
/// so the loop it wrote did not close and the reader refused it by name
/// (`ADraftWireDoesNotClose`) — what stopped `SW_NKK_BB15AH` and the two
/// `Relay_StandexMeder_DIP` relays once Q766 let them past the writer.
///
/// Not [`Covered::the_same_solid_comes_back`], because the fuse cannot cut the
/// slab as built — a block through its flat back leaves a decision the fuse
/// cannot settle, before any round trip — so the cut is required to come out
/// the same on both sides, refusal and all.
#[test]
fn a_slab_with_a_round_scoop_out_of_it_is_written_with_loops_that_close_and_comes_back_the_same() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::new(Unit::Millimetre, 6.0).expect("a working size"));
    let built = a_slab_with_a_round_scoop_out_of_it(&mut model);
    assert!(
        model
            .solid(built)
            .expect("this model made the key")
            .faces()
            .any(|face| face.orientation().is_reversed() && face.surface().as_plane().is_none()),
        "the scoop is a curved face used reversed, or this test does not reach the writer's walk"
    );
    assert_eq!(
        arcs_between_a_flat_face_and_a_pipe(&model, built),
        [1, 1],
        "one arc flat face first and one pipe first"
    );

    let text = rcad_step::write(&model, "a slab with a round scoop out of it")
        .unwrap_or_else(|why| panic!("the slab is not written: {why}"));
    let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
    let import = Import::read(&file, &policy, &mut log)
        .unwrap_or_else(|why| panic!("the slab is refused on the way back in: {why}"));
    let [(_, came)] = import.solids() else {
        panic!("one solid came back as {}", import.solids().len());
    };
    let came = *came;
    let there = import.model();
    there
        .validate(&policy, &mut log)
        .unwrap_or_else(|why| panic!("the slab comes back unsound: {why}"));
    assert_eq!(there.scale(), model.scale(), "the scale moved");

    let [went_solid, came_solid] = [(&model, built), (there, came)]
        .map(|(from, key)| from.solid(key).expect("this model made the key"));
    assert_eq!(
        where_its_corners_are(went_solid),
        where_its_corners_are(came_solid),
        "corners"
    );
    let deflection = the_same_mesh_on_both_sides();
    let [here, back] = [went_solid, came_solid].map(|solid| {
        solid
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| panic!("the slab does not measure: {why}"))
    });
    assert_eq!(here.volume(), back.volume(), "volume");
    assert_eq!(here.surface_area(), back.surface_area(), "area");
    assert_eq!(here.centroid(), back.centroid(), "centroid");

    let cutter = |model: &mut Model| a_block(model, [2.0, 1.5, 2.0], Vector::new(-1.0, -7.0, 1.0));
    let [cut_here, cut_back] = [(&model, built), (there, came)].map(|(from, key)| {
        cut_by_a_third_solid(from, key, &cutter, deflection)
            .map(|pieces| {
                pieces
                    .iter()
                    .map(|piece| (piece.volume(), piece.surface_area(), piece.centroid()))
                    .collect::<Vec<_>>()
            })
            .map_err(|why| why.to_string())
    });
    assert_eq!(cut_here, cut_back, "the cut");
}

/// The slab of the test above, built face by face: bottom cap, scoop, top cap,
/// then the three flat sides.
fn a_slab_with_a_round_scoop_out_of_it(model: &mut Model) -> SolidKey {
    use rcad_geom::{Axis, Cylinder, HalfSpace, Orientation, Picker};
    use rcad_topo::{DraftStep, SolidBuilder};

    let height = 4.0;
    let a_plane = |at: Point, facing: Vector| {
        Plane::from_point_and_normal(at, facing).expect("an axis direction has length")
    };
    let bottom = a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0));
    let top = a_plane(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0));
    let the_short_arc_on = |cap: Plane| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cap),
        picked_by: [Picker::from(
            HalfSpace::through_a_point_facing(
                Point::new(0.0, -3.0, 0.0),
                Vector::new(0.0, -1.0, 0.0),
            )
            .expect("an axis direction has length"),
        )]
        .into(),
    };

    let mut builder = SolidBuilder::new();
    let corners = [
        (4.0, -3.0, 0.0),
        (-4.0, -3.0, 0.0),
        (4.0, -3.0, height),
        (-4.0, -3.0, height),
        (4.0, -6.0, 0.0),
        (-4.0, -6.0, 0.0),
        (4.0, -6.0, height),
        (-4.0, -6.0, height),
    ]
    .map(|(x, y, z)| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a finite corner")
    });
    let edges = [
        (0, 1, the_short_arc_on(bottom)),
        (2, 3, the_short_arc_on(top)),
        (0, 2, EdgeRun::Straight),
        (1, 3, EdgeRun::Straight),
        (4, 5, EdgeRun::Straight),
        (6, 7, EdgeRun::Straight),
        (0, 4, EdgeRun::Straight),
        (1, 5, EdgeRun::Straight),
        (2, 6, EdgeRun::Straight),
        (3, 7, EdgeRun::Straight),
        (4, 6, EdgeRun::Straight),
        (5, 7, EdgeRun::Straight),
    ]
    .map(|(from, to, run)| {
        builder
            .add_edge(corners[from], corners[to], run)
            .expect("an edge between two corners")
    });

    let (forward, back) = (Orientation::Forward, Orientation::Reversed);
    let faces = [
        (Surface::Plane(bottom), forward),
        (
            Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a positive radius")),
            back,
        ),
        (Surface::Plane(top), forward),
        (
            Surface::Plane(a_plane(
                Point::new(4.0, 0.0, 0.0),
                Vector::new(1.0, 0.0, 0.0),
            )),
            forward,
        ),
        (
            Surface::Plane(a_plane(
                Point::new(-4.0, 0.0, 0.0),
                Vector::new(-1.0, 0.0, 0.0),
            )),
            forward,
        ),
        (
            Surface::Plane(a_plane(
                Point::new(0.0, -6.0, 0.0),
                Vector::new(0.0, -1.0, 0.0),
            )),
            forward,
        ),
    ];
    let wires: [&[(usize, Orientation)]; 6] = [
        &[(4, forward), (7, back), (0, back), (6, forward)],
        &[(0, forward), (3, forward), (1, back), (2, back)],
        &[(8, back), (1, forward), (9, forward), (5, back)],
        &[(6, back), (2, forward), (8, forward), (10, back)],
        &[(7, forward), (11, forward), (9, back), (3, back)],
        &[(4, back), (10, forward), (5, forward), (11, back)],
    ];
    let mut skin = Vec::with_capacity(faces.len());
    for ((surface, used), steps) in faces.into_iter().zip(wires) {
        let walked: Vec<DraftStep> = steps
            .iter()
            .map(|&(which, way)| {
                DraftStep::Along(builder.add_coedge(edges[which], way).expect("a coedge"))
            })
            .collect();
        let wire = builder.add_wire(&walked).expect("a wire that closes");
        skin.push((builder.add_face(surface, &[wire]).expect("a face"), used));
    }
    builder.add_shell(&skin).expect("a shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the slab with a round scoop out of it is not sound: {why}"))
}

/// How many open edges of `solid` that bend run between a flat face and a
/// pipe: with the flat face walked first, and with the pipe walked first. That
/// is the order the writer is handed the two faces in, since it walks the
/// faces in the order the solid lists them.
fn arcs_between_a_flat_face_and_a_pipe(model: &Model, solid: SolidKey) -> [usize; 2] {
    let mut either_side: Vec<(EdgeKey, Vec<Surface>)> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if !matches!(edge.run(), EdgeRun::PartOfWhereItCrosses { .. }) {
                    continue;
                }
                match either_side.iter_mut().find(|(key, _)| *key == edge.key()) {
                    Some((_, faces)) => faces.push(face.surface()),
                    None => either_side.push((edge.key(), vec![face.surface()])),
                }
            }
        }
    }
    let flat_first = either_side
        .iter()
        .filter(|(_, faces)| matches!(faces.as_slice(), [Surface::Plane(_), Surface::Cylinder(_)]))
        .count();
    let pipe_first = either_side
        .iter()
        .filter(|(_, faces)| matches!(faces.as_slice(), [Surface::Cylinder(_), Surface::Plane(_)]))
        .count();
    [flat_first, pipe_first]
}

/// A bar with a dome on top, the way `kicad/LED_D5.0mm-3.step` draws its lens:
/// the rim where the dome meets the bar is two half circles, and the dome's
/// seam runs from the rim up to its pole.
///
/// Until Q767 the writer refused both: it named an edge between a pipe and a
/// ball nowhere, and an edge with the one ball on both sides only where it ran
/// from pole to pole. The reader names such an arc by the plane its circle lies
/// in and walks where that plane crosses the bar or the dome, so the writer
/// writes the circle in that plane and about its own normal, and picks the
/// half by `same_sense`; written turned over instead, the plane would come back
/// reversed.
#[test]
fn a_bar_with_a_dome_on_top_is_written_with_its_rim_and_seam_and_comes_back_the_same() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 6.0).expect("a working size"));
    let built = a_bar_with_a_dome_on_top(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [["ball", "ball"], ["ball", "pipe"], ["ball", "pipe"]],
        "the dome's seam and the two halves of its rim"
    );
    let text = comes_back_the_same("a bar with a dome on top", &model, built, &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 3.0))
    });
    assert!(
        text.contains(",.F.);"),
        "one half of the rim runs against its circle, or this does not reach same_sense"
    );
}

/// A slice off the side of a ball, `|p| ≤ 5`, `z ≥ 0`, `y ≤ −3`: the ball and
/// two flat faces, which cross it in an arc of the equator and an arc of a
/// circle of radius four.
///
/// `kicad/LED_SideEmitter_Rectangular_W4.5mm_H1.6mm.step` holds a dome sitting
/// on a flat face like this, and until Q767 the writer refused any edge
/// between a flat face and a ball. It now writes the circle the face's plane
/// crosses the ball in, turned so the arc the reader picks is the one the
/// model holds.
#[test]
fn a_slice_off_the_side_of_a_ball_is_written_with_its_arcs_and_comes_back_the_same() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let built = a_slice_off_the_side_of_a_ball(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [["ball", "plane"], ["ball", "plane"]],
        "an arc on each flat face"
    );
    comes_back_the_same("a slice off the side of a ball", &model, built, &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(-1.0, -5.0, 1.0))
    });
}

/// Two bars of radius five meeting at a mitre, sliced along their length by
/// `z = −3`: the two pipes cross in an arc of an ellipse in the mitre's plane.
///
/// Twelve KiCad relays bend their pins like this, and until Q767 the writer
/// refused an edge between two pipes unless it was the straight seam of one.
/// The arc is written as the `ELLIPSE` the mitre's plane crosses a pipe in, so
/// the plane the reader walks comes back bit for bit.
#[test]
fn two_bars_meeting_at_a_mitre_are_written_with_an_arc_of_an_ellipse_and_come_back_the_same() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 8.0).expect("a working size"));
    let built = two_bars_meeting_at_a_mitre(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [["pipe", "pipe"], ["pipe", "plane"], ["pipe", "plane"]],
        "the mitre and the arc on each end"
    );
    let text = comes_back_the_same("two bars meeting at a mitre", &model, built, &|model| {
        a_block(model, [4.0, 4.0, 2.0], Vector::new(-2.0, -2.0, -6.0))
    });
    assert!(
        text.contains("ELLIPSE("),
        "the mitre is an arc of an ellipse"
    );
}

/// A bar with a smaller disc drawn on its top: the disc and the ring round it
/// are two flat faces on one plane, and the circle between them is two arcs
/// that the model names by the pipe of the circle's radius.
///
/// The writer wrote every open edge between two flat faces as a straight
/// `LINE`, so it wrote these arcs as chords, and twenty KiCad LEDs, whose
/// flat flanges carry circles drawn this way, came back refused as
/// `BoundaryCrossesItself` once Q767 let them past their domes. It now writes
/// the circle.
#[test]
fn a_disc_drawn_on_the_top_of_a_bar_is_written_with_arcs_and_comes_back_the_same() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let built = a_bar_with_a_disc_drawn_on_its_top(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [["plane", "plane"], ["plane", "plane"]],
        "the two halves of the disc's rim"
    );
    let text = comes_back_the_same("a disc drawn on a bar", &model, built, &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(0.0, -1.0, 3.0))
    });
    assert_eq!(
        text.matches("CIRCLE(").count(),
        4,
        "the bar's two caps and the two halves of the disc's rim"
    );
}

/// The outer side of a rod bent round a ring, whose ring meets a pipe in four
/// arcs: the tube's circle where a straight pipe carries on from the bend, and
/// the circles where the pipe about the ring's own axis crosses it.
///
/// `kicad/Relay_SPDT_Finder_34.51_Vertical.step` bends its pins this way, and
/// until Q773 the writer refused an edge between a pipe and a ring. It is
/// written the way an arc between a pipe and a ball is, in the plane its run
/// names, where that plane crosses the pipe. The relay itself still stops on
/// an edge between a flat face and a ring.
#[test]
fn the_outer_side_of_a_rod_bent_round_a_ring_is_written_with_its_arcs_and_comes_back_the_same() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 5.75).expect("a working size"));
    let built = the_outer_side_of_a_rod_bent_round_a_ring(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [
            ["pipe", "plane"],
            ["pipe", "plane"],
            ["pipe", "ring"],
            ["pipe", "ring"],
            ["pipe", "ring"],
            ["pipe", "ring"],
        ],
        "four arcs where the ring meets a pipe, and an arc on each end"
    );
    comes_back_the_same(
        "the outer side of a rod bent round a ring",
        &model,
        built,
        &|model| a_block(model, [2.0, 2.0, 4.0], Vector::new(4.0, 4.0, -2.0)),
    );
}

/// Write `built` out, read it back, and require the same solid: sound, the
/// same corners bit for bit, the same volume, area and centroid, and the same
/// answer to a cut by `cutter`, refusal and all. Hands back what was written.
///
/// Not [`Covered::the_same_solid_comes_back`], because the fuse cannot cut every
/// one of these shapes, and a cut refused alike on both sides is still the same
/// answer.
fn comes_back_the_same(
    called: &str,
    model: &Model,
    built: SolidKey,
    cutter: &dyn Fn(&mut Model) -> SolidKey,
) -> String {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let text = rcad_step::write(model, called)
        .unwrap_or_else(|why| panic!("{called} is not written: {why}"));
    let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
    let import = Import::read(&file, &policy, &mut log)
        .unwrap_or_else(|why| panic!("{called} is refused on the way back in: {why}"));
    let [(_, came)] = import.solids() else {
        panic!("one solid came back as {}", import.solids().len());
    };
    let came = *came;
    let there = import.model();
    there
        .validate(&policy, &mut log)
        .unwrap_or_else(|why| panic!("{called} comes back unsound: {why}"));
    assert_eq!(there.scale(), model.scale(), "{called}: the scale moved");

    let [went_solid, came_solid] = [(model, built), (there, came)]
        .map(|(from, key)| from.solid(key).expect("this model made the key"));
    assert_eq!(
        where_its_corners_are(went_solid),
        where_its_corners_are(came_solid),
        "{called}: corners"
    );
    let deflection = the_same_mesh_on_both_sides();
    let [here, back] = [went_solid, came_solid].map(|solid| {
        solid
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| panic!("{called} does not measure: {why}"))
    });
    assert_eq!(here.volume(), back.volume(), "{called}: volume");
    assert_eq!(here.surface_area(), back.surface_area(), "{called}: area");
    assert_eq!(here.centroid(), back.centroid(), "{called}: centroid");

    let [cut_here, cut_back] = [(model, built), (there, came)].map(|(from, key)| {
        cut_by_a_third_solid(from, key, cutter, deflection)
            .map(|pieces| {
                pieces
                    .iter()
                    .map(|piece| (piece.volume(), piece.surface_area(), piece.centroid()))
                    .collect::<Vec<_>>()
            })
            .map_err(|why| why.to_string())
    });
    assert_eq!(cut_here, cut_back, "{called}: the cut");
    text
}

/// What the two faces either side of each open edge of `solid` that bends sit
/// on, each pair in name order and the pairs sorted.
fn open_bends(model: &Model, solid: SolidKey) -> Vec<[&'static str; 2]> {
    let kind = |surface: Surface| match surface {
        Surface::Plane(_) => "plane",
        Surface::Cylinder(_) => "pipe",
        Surface::Sphere(_) => "ball",
        Surface::Torus(_) => "ring",
        Surface::Cone(_) => "cone",
        _ => "a surface this test does not name",
    };
    let mut either_side: Vec<(EdgeKey, Vec<&'static str>)> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if !matches!(edge.run(), EdgeRun::PartOfWhereItCrosses { .. }) {
                    continue;
                }
                match either_side.iter_mut().find(|(key, _)| *key == edge.key()) {
                    Some((_, faces)) => faces.push(kind(face.surface())),
                    None => either_side.push((edge.key(), vec![kind(face.surface())])),
                }
            }
        }
    }
    let mut pairs: Vec<[&'static str; 2]> = either_side
        .into_iter()
        .map(|(_, mut faces)| {
            faces.sort_unstable();
            let [first, second] = faces[..] else {
                panic!("an edge walked by {faces:?}");
            };
            [first, second]
        })
        .collect();
    pairs.sort_unstable();
    pairs
}

/// One step of a wire of [`a_solid_drawn_face_by_face`]: along an edge of its
/// table, forwards or reversed, or standing still on one of its corners.
#[derive(Clone, Copy)]
enum Step {
    Forward(usize),
    Reversed(usize),
    Pole(usize),
}

/// A solid built face by face from three tables: its corners, its edges as
/// `(from, to, run)` between them, and each face's surface and wires. Every
/// face is used the way its surface faces but those numbered in `inside_out`,
/// which the shell uses the other way round.
fn a_solid_drawn_face_by_face(
    model: &mut Model,
    corners: &[Point],
    edges: Vec<(usize, usize, EdgeRun)>,
    faces: Vec<(Surface, Vec<Vec<Step>>)>,
    inside_out: &[usize],
) -> SolidKey {
    use rcad_geom::Orientation;
    use rcad_topo::{DraftStep, SolidBuilder};

    let mut builder = SolidBuilder::new();
    let corners: Vec<_> = corners
        .iter()
        .map(|&at| builder.add_vertex(at).expect("a finite corner"))
        .collect();
    let edges: Vec<_> = edges
        .into_iter()
        .map(|(from, to, run)| {
            builder
                .add_edge(corners[from], corners[to], run)
                .expect("an edge between two corners")
        })
        .collect();
    let mut skin = Vec::with_capacity(faces.len());
    for (which, (surface, wires)) in faces.into_iter().enumerate() {
        let wires: Vec<_> = wires
            .iter()
            .map(|steps| {
                let walked: Vec<DraftStep> = steps
                    .iter()
                    .map(|&step| match step {
                        Step::Forward(which) => DraftStep::Along(
                            builder
                                .add_coedge(edges[which], Orientation::Forward)
                                .expect("a coedge"),
                        ),
                        Step::Reversed(which) => DraftStep::Along(
                            builder
                                .add_coedge(edges[which], Orientation::Reversed)
                                .expect("a coedge"),
                        ),
                        Step::Pole(at) => DraftStep::Pole(corners[at]),
                    })
                    .collect();
                builder.add_wire(&walked).expect("a wire that closes")
            })
            .collect();
        let used = if inside_out.contains(&which) {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        skin.push((builder.add_face(surface, &wires).expect("a face"), used));
    }
    builder.add_shell(&skin).expect("a shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the solid drawn face by face is not sound: {why}"))
}

/// Part of where a face crosses `and`: the side of the plane through `through`
/// that `facing` points to.
fn part_of(and: Surface, through: Point, facing: Vector) -> EdgeRun {
    use rcad_geom::{HalfSpace, Picker};

    EdgeRun::PartOfWhereItCrosses {
        and,
        picked_by: [Picker::from(
            HalfSpace::through_a_point_facing(through, facing).expect("a facing with a length"),
        )]
        .into(),
    }
}

fn a_plane(at: Point, facing: Vector) -> Plane {
    Plane::from_point_and_normal(at, facing).expect("a facing with a length")
}

/// A bar of radius two about `z`, four tall, with a dome of the same radius on
/// top: its bottom cap, its pipe with a seam at `x = 2`, and the dome, whose
/// rim is two half circles and whose seam runs from the rim at `(2, 0, 4)` to
/// the pole at `(0, 0, 6)`.
fn a_bar_with_a_dome_on_top(model: &mut Model) -> SolidKey {
    use Step::{Forward, Pole, Reversed};
    use rcad_geom::{Axis, Cylinder, Sphere};

    let rim = Surface::Plane(a_plane(
        Point::new(0.0, 0.0, 4.0),
        Vector::new(0.0, 0.0, 1.0),
    ));
    let up_the_dome = Surface::Plane(a_plane(Point::origin(), Vector::new(0.0, 1.0, 0.0)));
    let centre = Point::new(0.0, 0.0, 4.0);
    a_solid_drawn_face_by_face(
        model,
        &[
            Point::new(2.0, 0.0, 0.0),
            Point::new(2.0, 0.0, 4.0),
            Point::new(-2.0, 0.0, 4.0),
            Point::new(0.0, 0.0, 6.0),
        ],
        vec![
            (0, 0, EdgeRun::RightRoundWhereItsFacesCross),
            (0, 1, EdgeRun::Straight),
            (1, 2, part_of(rim, centre, Vector::new(0.0, 1.0, 0.0))),
            (2, 1, part_of(rim, centre, Vector::new(0.0, -1.0, 0.0))),
            (
                1,
                3,
                part_of(up_the_dome, centre, Vector::new(1.0, 0.0, 0.0)),
            ),
        ],
        vec![
            (
                Surface::Plane(a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0))),
                vec![vec![Reversed(0)]],
            ),
            (
                Surface::Cylinder(Cylinder::new(Axis::z(), 2.0).expect("a positive radius")),
                vec![vec![
                    Forward(0),
                    Forward(1),
                    Reversed(3),
                    Reversed(2),
                    Reversed(1),
                ]],
            ),
            (
                Surface::Sphere(Sphere::new(centre, 2.0).expect("a positive radius")),
                vec![vec![
                    Forward(2),
                    Forward(3),
                    Forward(4),
                    Pole(3),
                    Reversed(4),
                ]],
            ),
        ],
        &[],
    )
}

/// The ball of radius five about the origin, cut down to `z ≥ 0` and
/// `y ≤ −3`: the ball, the floor `z = 0` and the wall `y = −3`, with corners at
/// `(±4, −3, 0)`.
fn a_slice_off_the_side_of_a_ball(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::Sphere;

    let floor = a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0));
    let wall = a_plane(Point::new(0.0, -3.0, 0.0), Vector::new(0.0, 1.0, 0.0));
    a_solid_drawn_face_by_face(
        model,
        &[Point::new(4.0, -3.0, 0.0), Point::new(-4.0, -3.0, 0.0)],
        vec![
            (
                1,
                0,
                part_of(
                    Surface::Plane(floor),
                    Point::new(0.0, -3.0, 0.0),
                    Vector::new(0.0, -1.0, 0.0),
                ),
            ),
            (
                0,
                1,
                part_of(
                    Surface::Plane(wall),
                    Point::origin(),
                    Vector::new(0.0, 0.0, 1.0),
                ),
            ),
            (1, 0, EdgeRun::Straight),
        ],
        vec![
            (
                Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a positive radius")),
                vec![vec![Forward(0), Forward(1)]],
            ),
            (Surface::Plane(floor), vec![vec![Reversed(0), Forward(2)]]),
            (Surface::Plane(wall), vec![vec![Reversed(1), Reversed(2)]]),
        ],
        &[],
    )
}

/// A bar of radius five along `x` from its cap at `x = −8` and one along `y` to
/// its cap at `y = 8`, meeting at the mitre `x + y = 0`, and sliced along
/// their length by `z = −3`, keeping what is under it.
fn two_bars_meeting_at_a_mitre(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder};

    let under = Point::new(0.0, 0.0, -3.0);
    let down = Vector::new(0.0, 0.0, -1.0);
    let end_of_the_first = a_plane(Point::new(-8.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0));
    let end_of_the_second = a_plane(Point::new(0.0, 8.0, 0.0), Vector::new(0.0, 1.0, 0.0));
    let mitre = Surface::Plane(a_plane(Point::origin(), Vector::new(1.0, 1.0, 0.0)));
    let along = |direction: Vector| {
        Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::origin(), direction).expect("a direction"),
                5.0,
            )
            .expect("a positive radius"),
        )
    };
    a_solid_drawn_face_by_face(
        model,
        &[
            Point::new(-8.0, -4.0, -3.0),
            Point::new(-8.0, 4.0, -3.0),
            Point::new(4.0, -4.0, -3.0),
            Point::new(-4.0, 4.0, -3.0),
            Point::new(4.0, 8.0, -3.0),
            Point::new(-4.0, 8.0, -3.0),
        ],
        vec![
            (0, 1, part_of(Surface::Plane(end_of_the_first), under, down)),
            (1, 0, EdgeRun::Straight),
            (0, 2, EdgeRun::Straight),
            (3, 1, EdgeRun::Straight),
            (2, 3, part_of(mitre, under, down)),
            (2, 4, EdgeRun::Straight),
            (5, 3, EdgeRun::Straight),
            (
                4,
                5,
                part_of(Surface::Plane(end_of_the_second), under, down),
            ),
            (5, 4, EdgeRun::Straight),
        ],
        vec![
            (
                Surface::Plane(a_plane(under, Vector::new(0.0, 0.0, 1.0))),
                vec![vec![
                    Forward(2),
                    Forward(5),
                    Reversed(8),
                    Forward(6),
                    Forward(3),
                    Forward(1),
                ]],
            ),
            (
                along(Vector::new(1.0, 0.0, 0.0)),
                vec![vec![Reversed(3), Reversed(4), Reversed(2), Forward(0)]],
            ),
            (
                along(Vector::new(0.0, 1.0, 0.0)),
                vec![vec![Forward(4), Reversed(6), Reversed(7), Reversed(5)]],
            ),
            (
                Surface::Plane(end_of_the_first),
                vec![vec![Reversed(0), Reversed(1)]],
            ),
            (
                Surface::Plane(end_of_the_second),
                vec![vec![Forward(7), Forward(8)]],
            ),
        ],
        &[],
    )
}

/// A bar of radius two about `z`, four tall, with a disc of radius one drawn
/// on its top cap: the cap is a ring round the disc and the disc a face of its
/// own on the same plane, and the circle between them is two half circles the
/// model names by the pipe of radius one standing on the cap.
fn a_bar_with_a_disc_drawn_on_its_top(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder};

    let centre = Point::new(0.0, 0.0, 4.0);
    let top = Surface::Plane(a_plane(centre, Vector::new(0.0, 0.0, 1.0)));
    let drawn = Surface::Cylinder(
        Cylinder::new(
            Axis::new(centre, Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
            1.0,
        )
        .expect("a positive radius"),
    );
    a_solid_drawn_face_by_face(
        model,
        &[
            Point::new(2.0, 0.0, 0.0),
            Point::new(2.0, 0.0, 4.0),
            Point::new(1.0, 0.0, 4.0),
            Point::new(-1.0, 0.0, 4.0),
        ],
        vec![
            (0, 0, EdgeRun::RightRoundWhereItsFacesCross),
            (0, 1, EdgeRun::Straight),
            (1, 1, EdgeRun::RightRoundWhereItsFacesCross),
            (2, 3, part_of(drawn, centre, Vector::new(0.0, 1.0, 0.0))),
            (3, 2, part_of(drawn, centre, Vector::new(0.0, -1.0, 0.0))),
        ],
        vec![
            (
                Surface::Plane(a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0))),
                vec![vec![Reversed(0)]],
            ),
            (
                Surface::Cylinder(Cylinder::new(Axis::z(), 2.0).expect("a positive radius")),
                vec![vec![Forward(0), Forward(1), Reversed(2), Reversed(1)]],
            ),
            (top, vec![vec![Forward(2)], vec![Reversed(4), Reversed(3)]]),
            (top, vec![vec![Forward(3), Forward(4)]]),
        ],
        &[],
    )
}

/// The outer side of a rod bent round a ring, `r ≥ 5.75` about `z`: a pipe of
/// radius `1.25` along `y` at `x = 5` from its end at `y = −4` up to `y = 0`,
/// a quarter of the ring of radius five with that tube from there round to
/// `x = 0`, and a pipe along `x` at `y = 5` on to its end at `x = −4`. The cut
/// is the plane `x = 5.75` beside the first pipe, the pipe of radius `5.75`
/// about `z` beside the ring and the plane `y = 5.75` beside the second pipe.
///
/// So the ring meets a pipe in four arcs: two halves of its tube's circle
/// where a pipe carries on from it, and two quarters of the circles at
/// `z = ±1` where the pipe about its own axis crosses it.
fn the_outer_side_of_a_rod_bent_round_a_ring(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder, Torus};

    let x = Vector::new(1.0, 0.0, 0.0);
    let y = Vector::new(0.0, 1.0, 0.0);
    let z = Vector::new(0.0, 0.0, 1.0);
    let flat = |at: Point, facing: Vector| Surface::Plane(a_plane(at, facing));
    let pipe = |at: Point, along: Vector, radius: f64| {
        Surface::Cylinder(
            Cylinder::new(Axis::new(at, along).expect("a direction"), radius)
                .expect("a positive radius"),
        )
    };
    let round_the_ring = |height: f64| {
        part_of(
            flat(Point::new(0.0, 0.0, height), z),
            Point::new(5.75, 0.0, height),
            Vector::new(1.0, 1.0, 0.0),
        )
    };
    a_solid_drawn_face_by_face(
        model,
        &[
            Point::new(5.75, 0.0, 1.0),
            Point::new(5.75, 0.0, -1.0),
            Point::new(0.0, 5.75, 1.0),
            Point::new(0.0, 5.75, -1.0),
            Point::new(5.75, -4.0, 1.0),
            Point::new(5.75, -4.0, -1.0),
            Point::new(-4.0, 5.75, 1.0),
            Point::new(-4.0, 5.75, -1.0),
        ],
        vec![
            (
                0,
                1,
                part_of(flat(Point::origin(), y), Point::new(5.75, 0.0, 0.0), x),
            ),
            (
                2,
                3,
                part_of(flat(Point::origin(), x), Point::new(0.0, 5.75, 0.0), y),
            ),
            (0, 2, round_the_ring(1.0)),
            (1, 3, round_the_ring(-1.0)),
            (4, 0, EdgeRun::Straight),
            (5, 1, EdgeRun::Straight),
            (1, 0, EdgeRun::Straight),
            (3, 2, EdgeRun::Straight),
            (2, 6, EdgeRun::Straight),
            (3, 7, EdgeRun::Straight),
            (
                4,
                5,
                part_of(
                    flat(Point::new(0.0, -4.0, 0.0), -y),
                    Point::new(5.75, 0.0, 0.0),
                    x,
                ),
            ),
            (5, 4, EdgeRun::Straight),
            (
                6,
                7,
                part_of(
                    flat(Point::new(-4.0, 0.0, 0.0), -x),
                    Point::new(0.0, 5.75, 0.0),
                    y,
                ),
            ),
            (7, 6, EdgeRun::Straight),
        ],
        vec![
            (
                Surface::Torus(Torus::new(Axis::z(), 5.0, 1.25).expect("a working ring")),
                vec![vec![Forward(0), Forward(3), Reversed(1), Reversed(2)]],
            ),
            (
                pipe(Point::new(5.0, 0.0, 0.0), y, 1.25),
                vec![vec![Reversed(4), Forward(10), Forward(5), Reversed(0)]],
            ),
            (
                pipe(Point::new(0.0, 5.0, 0.0), x, 1.25),
                vec![vec![Forward(1), Forward(9), Reversed(12), Reversed(8)]],
            ),
            (
                flat(Point::new(5.75, 0.0, 0.0), -x),
                vec![vec![Forward(4), Reversed(6), Reversed(5), Forward(11)]],
            ),
            (
                pipe(Point::origin(), z, 5.75),
                vec![vec![Forward(2), Reversed(7), Reversed(3), Forward(6)]],
            ),
            (
                flat(Point::new(0.0, 5.75, 0.0), -y),
                vec![vec![Forward(7), Forward(8), Reversed(13), Reversed(9)]],
            ),
            (
                flat(Point::new(0.0, -4.0, 0.0), -y),
                vec![vec![Reversed(10), Reversed(11)]],
            ),
            (
                flat(Point::new(-4.0, 0.0, 0.0), -x),
                vec![vec![Forward(12), Forward(13)]],
            ),
        ],
        &[4],
    )
}

/// A model with no solid in it is refused rather than written as a file with no
/// solid in it, which is the file this reader refuses.
#[test]
fn a_model_with_nothing_in_it_is_refused_rather_than_written() {
    let model = Model::new(Scale::new(Unit::Millimetre, 1.0).expect("a working size"));
    assert!(matches!(
        rcad_step::write(&model, "nothing"),
        Err(CannotWrite::TheModelHasNoSolid)
    ));
}

/// Every number this writer writes reads back as the same `f64`, bit for bit.
///
/// This is the claim [`rcad_step::write`]'s `real` rests on and §15.6 asks for:
/// `{:?}` is Rust's shortest text that reads back as the same `f64`, so a
/// coordinate goes out as decimal text and comes back as the number it left as,
/// and the round trip above is comparing shapes rather than conversions.
///
/// The draw is the numbers a model is made of and the ones that are hardest to
/// print: the ends of the range, the smallest number there is, a negative zero
/// — which is a different `f64` from zero and has to stay one — and a spread of
/// bit patterns nobody chose by hand. Checked **through this crate's own
/// reader**, not through `f64::from_str`, because it is the reader's tokenizer
/// the claim is about.
#[test]
fn every_number_this_writer_writes_reads_back_bit_for_bit() {
    let mut drawn = Vec::new();
    for number in [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.1,
        1.0 / 3.0,
        core::f64::consts::PI,
        25.4,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        f64::MIN_POSITIVE / 2.0,
        f64::from_bits(1),
        1e-300,
        1e300,
        2.0_f64.powi(-1074),
    ] {
        drawn.push(number);
    }
    // A spread nobody chose by hand: a multiplicative walk over the exponent
    // range with a mantissa that never repeats, and its negative.
    let mut bits = 0x3ff0_0000_0000_0001_u64;
    for _ in 0..2000 {
        bits = bits.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let number = f64::from_bits(bits);
        if number.is_finite() {
            drawn.push(number);
        }
    }

    let mut checked = 0;
    for number in drawn {
        let text = format!(
            "ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION((''),'2;1');\n\
             FILE_NAME('','',(''),(''),'','','');\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\n\
             ENDSEC;\nDATA;\n#1=CARTESIAN_POINT('',({0},{0},{0}));\nENDSEC;\n\
             END-ISO-10303-21;\n",
            rcad_step_real(number)
        );
        let file = StepFile::read(text.as_bytes())
            .unwrap_or_else(|why| panic!("{number:?} wrote a file this reader refuses: {why}"));
        let point = file.instances().next().expect("the one point");
        let list = point.parts()[0]
            .attribute(1)
            .and_then(rcad_step::Value::as_list)
            .expect("three coordinates");
        for value in list {
            let read = value
                .as_real()
                .unwrap_or_else(|| panic!("{number:?} came back as {value:?} and not a real"));
            assert_eq!(
                read.to_bits(),
                number.to_bits(),
                "{number:?} was written as {} and read back as {read:?}",
                rcad_step_real(number)
            );
            checked += 1;
        }
    }
    println!("{checked} numbers written as decimal text and read back bit for bit");
    assert!(
        checked > 6000,
        "the draw must not shrink without somebody saying so: {checked}"
    );
}

/// The writer's own `real`, written again here so the claim above is about the
/// text a file carries rather than about a function this test imported.
///
/// It is three lines and the reason for not sharing it is the testing skill's:
/// a check that calls the code it is checking can go green with it for the same
/// wrong reason. What this test compares is *text* against *the reader*, so the
/// text has to be spelled out.
fn rcad_step_real(number: f64) -> String {
    let written = format!("{number:?}");
    match written.split_once('e') {
        None => written,
        Some((mantissa, exponent)) => {
            let mantissa = if mantissa.contains('.') {
                mantissa.to_owned()
            } else {
                format!("{mantissa}.")
            };
            format!("{mantissa}E{exponent}")
        }
    }
}

/// How many shapes went out and came back, so that a suite in which everything
/// quietly refused cannot pass.
struct Covered {
    shapes: usize,
    pieces: usize,
}

impl Covered {
    fn new() -> Self {
        Self {
            shapes: 0,
            pieces: 0,
        }
    }

    /// Build a solid, write it, read it back, and require the same answer to
    /// every question this kernel can ask of it.
    fn the_same_solid_comes_back(
        &mut self,
        called: &str,
        scale: Scale,
        build: &dyn Fn(&mut Model) -> SolidKey,
        cutter: &dyn Fn(&mut Model) -> SolidKey,
    ) {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(scale);
        let built = build(&mut model);

        let text = rcad_step::write(&model, called)
            .unwrap_or_else(|why| panic!("{called} could not be written: {why}"));
        let file = StepFile::read(text.as_bytes())
            .unwrap_or_else(|why| panic!("{called} wrote a file this reader refuses: {why}"));
        assert_eq!(
            file.references_that_point_nowhere(),
            [],
            "{called} wrote a reference to an entity it never wrote"
        );
        let import = Import::read(&file, &policy, &mut log).unwrap_or_else(|why| {
            panic!("{called} wrote a file this reader will not import: {why}")
        });
        assert_eq!(
            import.integers_read_as_reals(),
            0,
            "{called}: this writer writes every real with a decimal point"
        );
        let [(_, read_back)] = import.solids() else {
            panic!(
                "{called} was one solid and came back as {}",
                import.solids().len()
            );
        };
        let read_back = *read_back;
        let there = import.model();
        assert_eq!(
            there.scale(),
            model.scale(),
            "{called} came back at another scale, so the two fuses below would be asked \
             different questions"
        );
        there
            .validate(&policy, &mut log)
            .unwrap_or_else(|why| panic!("{called} came back as a model that is not sound: {why}"));

        let deflection = the_same_mesh_on_both_sides();
        let here = model
            .solid(built)
            .expect("this model made the key")
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| panic!("{called} does not measure: {why}"));
        let back = there
            .solid(read_back)
            .expect("this model made the key")
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| panic!("{called} does not measure after the round trip: {why}"));
        assert_eq!(here.volume(), back.volume(), "{called}: volume");
        assert_eq!(here.surface_area(), back.surface_area(), "{called}: area");
        assert_eq!(here.centroid(), back.centroid(), "{called}: centroid");

        let cut_here = cut_by_a_third_solid(&model, built, cutter, deflection)
            .unwrap_or_else(|why| panic!("{called} is not cut by the third solid: {why}"));
        let cut_back = cut_by_a_third_solid(there, read_back, cutter, deflection)
            .unwrap_or_else(|why| panic!("{called} after the round trip is not cut: {why}"));
        assert_eq!(
            cut_here.len(),
            cut_back.len(),
            "{called}: the two fuses came back with different numbers of pieces"
        );
        assert!(
            !cut_here.is_empty(),
            "{called}: the cut left nothing to compare"
        );
        let left: f64 = cut_here.iter().map(|piece| piece.volume()).sum();
        assert!(
            left < here.volume(),
            "{called}: the cut took nothing away, so this compared two solids nothing \
             happened to: {left} of {}",
            here.volume()
        );
        for (one, other) in cut_here.iter().zip(&cut_back) {
            assert_eq!(one.volume(), other.volume(), "{called}: a piece's volume");
            assert_eq!(
                one.surface_area(),
                other.surface_area(),
                "{called}: a piece's area"
            );
            assert_eq!(
                one.centroid(),
                other.centroid(),
                "{called}: a piece's centroid"
            );
            self.pieces += 1;
        }
        self.shapes += 1;
    }

    fn report(&self, family: &str) {
        println!(
            "round trip: {} shape(s) of {family} written, read back and compared, \
             {} fused piece(s) compared",
            self.shapes, self.pieces
        );
        assert!(
            self.shapes > 0 && self.pieces > 0,
            "{family} compared nothing at all"
        );
    }
}

/// One solid of the model, cut by a third one it overlaps, measured piece by
/// piece.
///
/// Done on a clone so the two runs this is called from cannot see each other's
/// answer.
fn cut_by_a_third_solid(
    model: &Model,
    which: SolidKey,
    cutter: &dyn Fn(&mut Model) -> SolidKey,
    deflection: Deflection,
) -> rcad_topo::Result<Vec<Measurement>> {
    let policy = Policy::certified();
    let position = model
        .solids()
        .position(|solid| solid.key() == which)
        .expect("the solid is in the model it came from");
    let mut scratch = model.clone();
    let keep = scratch
        .solids()
        .nth(position)
        .expect("a clone holds the same solids in the same order")
        .key();
    let knife = cutter(&mut scratch);

    let fused = scratch.fuse(
        &[keep],
        &[knife],
        Rule::Difference,
        &policy,
        &mut ProvenanceLog::new(),
    )?;
    let pieces: Vec<SolidKey> = fused.solids().to_vec();
    pieces
        .into_iter()
        .map(|piece| {
            scratch
                .solid(piece)?
                .measure(deflection, &policy, &mut ProvenanceLog::new())
        })
        .collect()
}

/// A box of the given lengths, with its low corner at `at`.
fn a_block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}
