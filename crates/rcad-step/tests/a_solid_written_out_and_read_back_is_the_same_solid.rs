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

/// The same bar turned by `place_solid` after it is built, rather than built
/// turned: a round bar of radius three, five tall, turned half a turn about
/// `y`.
///
/// `docs/decisions.md` §241.6. Q405's lane saw this turn and the one radian
/// turn below written, at 2392 and 2697 bytes, only through a probe since
/// removed, which its verifier could not rerun; this is the pin.
#[test]
fn a_round_bar_turned_a_half_turn_about_y_written_out_and_read_back_is_the_same_bar() {
    let turn = Isometry::rotation(&rcad_geom::Axis::y(), std::f64::consts::PI).expect("a turn");
    a_turned_bar_comes_back_the_same(
        &turn,
        "a bar of radius three, five tall, turned half a turn about y",
    );
}

/// The same bar, turned one radian about the axis through the origin and
/// `(1, 1, 1)`.
///
/// `docs/decisions.md` §241.6, the other of the two turns the comment above
/// names.
#[test]
fn a_round_bar_turned_one_radian_about_one_one_one_written_out_and_read_back_is_the_same_bar() {
    let diagonal = rcad_geom::Axis::new(Point::origin(), Vector::new(1.0, 1.0, 1.0))
        .expect("a finite axis through the origin");
    let turn = Isometry::rotation(&diagonal, 1.0).expect("a turn");
    a_turned_bar_comes_back_the_same(
        &turn,
        "a bar of radius three, five tall, turned one radian about (1,1,1)",
    );
}

/// A round bar `place_solid` turns, written and read back, held to
/// `where_its_corners_are` and to a rounding, rather than to
/// [`Covered::the_same_solid_comes_back`]'s bit-for-bit measurement and fuse.
///
/// **Neither half of that oracle holds for a placed solid, tried first.**
/// `add_round_bar(3.0, 5.0, &Isometry::identity())` then `place_solid` leaves
/// the turn composed into the solid's location, which `FaceView` applies
/// lazily at measurement time; the file instead writes the turn baked into
/// plain numbers (§42.8's "stored at the identity"), so the round-tripped
/// model's mesh takes the same points through a different sum. The two areas
/// came back `150.72991482151727` and `150.7299148215173`, a difference inside
/// the rounding `docs/decisions.md` §338 already publishes on
/// [`Measurement::volume_error`] and [`Measurement::area_error`] for "a turned
/// one" — so volume, area and centroid are compared to that rounding instead.
/// Corners are still compared bit for bit, since they are the file's own
/// written digits parsed back rather than summed.
///
/// **The fuse does not hold either, tried second.** Cutting the turned,
/// placed bar with a block — turned the same way, so the two overlap exactly
/// as they would unplaced — refused with `Error::Undecidable` for both turns
/// below and for every block offset tried, including offsets nowhere near a
/// tangency. §246.6 found the same refusal, `"where a plane and a pipe
/// cross"`, for a bar only *moved*, not turned, by `2⁻¹⁰⁰⁷`: `place_solid`
/// leaves the pipe's own numbers one rounding off whatever a fresh
/// `add_round_bar` at that placement would write, and the certified policy's
/// exact arithmetic refuses rather than guess at the sign that rounding
/// unsettles. Not diagnosed further here, matching §246.6's own "not
/// diagnosed here" for the same refusal. This is why the writer fuzz harness
/// falls back to located corners for a placed solid instead of a fuse.
fn a_turned_bar_comes_back_the_same(turn: &Isometry, called: &str) {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let scale = Scale::new(Unit::Millimetre, 5.0).expect("a working size");

    let mut model = Model::new(scale);
    let bar = model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a bar of radius three");
    model.place_solid(bar, turn).expect("the bar can be turned");
    model
        .validate(&policy, &mut log)
        .unwrap_or_else(|why| panic!("{called} is not sound before it is written: {why}"));

    let mut untouched = Model::new(scale);
    let reference = untouched
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a bar of radius three");

    let text = rcad_step::write(&model, called)
        .unwrap_or_else(|why| panic!("{called} is not written: {why}"));
    let file = StepFile::read(text.as_bytes())
        .unwrap_or_else(|why| panic!("{called} wrote a file this reader refuses: {why}"));
    let import = Import::read(&file, &policy, &mut log)
        .unwrap_or_else(|why| panic!("{called} is refused on the way back in: {why}"));
    let [(_, read_back)] = import.solids() else {
        panic!(
            "{called} was one solid and came back as {}",
            import.solids().len()
        );
    };
    let read_back = *read_back;
    let there = import.model();
    there
        .validate(&policy, &mut log)
        .unwrap_or_else(|why| panic!("{called} comes back as a model that is not sound: {why}"));

    let went = model.solid(bar).expect("this model made the key");
    let came = there.solid(read_back).expect("this model made the key");
    assert_eq!(
        where_its_corners_are(went),
        where_its_corners_are(came),
        "{called} comes back with its corners somewhere else"
    );
    assert_ne!(
        where_its_corners_are(went),
        where_its_corners_are(untouched.solid(reference).expect("this model made the key")),
        "{called} was not turned at all, so this compared two bars at the same place"
    );

    let deflection = the_same_mesh_on_both_sides();
    let here = went
        .measure(deflection, &policy, &mut log)
        .unwrap_or_else(|why| panic!("{called} does not measure: {why}"));
    let back = came
        .measure(deflection, &policy, &mut log)
        .unwrap_or_else(|why| panic!("{called} does not measure after the round trip: {why}"));
    assert!(
        (here.volume() - back.volume()).abs() <= here.volume_error().max(back.volume_error()),
        "{called}: volume {} vs {}, more than {} or {} of rounding apart",
        here.volume(),
        back.volume(),
        here.volume_error(),
        back.volume_error()
    );
    assert!(
        (here.surface_area() - back.surface_area()).abs()
            <= here.area_error().max(back.area_error()),
        "{called}: area {} vs {}, more than {} or {} of rounding apart",
        here.surface_area(),
        back.surface_area(),
        here.area_error(),
        back.area_error()
    );
    assert!(
        here.centroid().distance_to(back.centroid())
            <= here.centroid_wobble().max(back.centroid_wobble()),
        "{called}: centroid {:?} vs {:?}, further apart than the rounding allows",
        here.centroid(),
        back.centroid()
    );
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
/// the `CIRCLE` of the ball's own centre and radius about its plane's normal,
/// with the seam running along it or against it to pick the half the model's
/// seam is.
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

/// A ball turned so its poles leave the `z` line through its centre is written
/// as the same ball with its poles on that line, and comes back as that ball.
///
/// This kernel's sphere pinches only on the `z` line through its centre, so
/// the poles cannot go out where the turn put them: the reader would put them
/// back where the ball it reads does not pinch. The file is the one written
/// for the same ball moved to that centre and not turned, byte for byte
/// (`docs/decisions.md` §404). Until then the writer refused it at its pole.
#[test]
fn a_ball_turned_off_z_comes_back_as_the_same_ball_with_its_poles_on_z() {
    let half_a_radian_about_x = Isometry::rotation(&rcad_geom::Axis::x(), 0.5).expect("a turn");
    let about_y_and_moved = Isometry::translation(Vector::new(3.0, -7.0, 11.0))
        .expect("a finite offset")
        .compose(&Isometry::rotation(&rcad_geom::Axis::y(), 1.1).expect("a turn"));
    for (radius, turned, centre) in [
        (4.0, half_a_radian_about_x, Point::origin()),
        (2.5, about_y_and_moved, Point::new(3.0, -7.0, 11.0)),
    ] {
        let scale = Scale::new(Unit::Millimetre, 20.0).expect("a working size");
        let mut model = Model::new(scale);
        model.add_ball(radius, &turned).expect("a ball");
        let mut moved = Model::new(scale);
        let to_its_centre = Isometry::translation(centre - Point::origin()).expect("a move");
        moved.add_ball(radius, &to_its_centre).expect("a ball");
        let ball = the_one_face(&model).surface();
        assert_eq!(
            format!("{ball:?}"),
            format!("{:?}", the_one_face(&moved).surface()),
            "the turned ball is not centred where the test says"
        );
        let poles_where_turned = the_poles_of(&model);
        assert!(
            poles_where_turned
                .iter()
                .all(|pole| ball.the_way_its_first_coordinate_runs_at(*pole).is_some()),
            "the turn left a pole on the z line: {poles_where_turned:?}"
        );

        let text = rcad_step::write(&model, "a ball").expect("a turned ball is written");
        assert_eq!(
            text,
            rcad_step::write(&moved, "a ball").expect("a moved ball is written")
        );
        let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let import = Import::read(&file, &policy, &mut log).expect("a turned ball is read back");
        let back = import.model();
        back.validate(&policy, &mut log)
            .expect("a turned ball comes back sound");
        assert_eq!(
            (
                back.faces().count(),
                back.edges().count(),
                back.vertices().count()
            ),
            (1, 1, 2)
        );
        assert_eq!(
            format!("{:?}", the_one_face(back).surface()),
            format!("{ball:?}")
        );
        let [x, y, z] = centre.coordinates();
        assert_eq!(
            the_poles_of(back),
            [Point::new(x, y, z - radius), Point::new(x, y, z + radius)]
        );
    }
}

/// A turned ball too small beside its height for `z ± r` to stand apart from
/// `z` is still refused at its pole: there are no two poles on the `z` line to
/// write it with.
#[test]
fn a_turned_ball_too_small_for_its_height_is_refused_at_its_pole() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 1e20).expect("a working size"));
    let high_and_turned = Isometry::translation(Vector::new(0.0, 0.0, 1e20))
        .expect("a finite offset")
        .compose(&Isometry::rotation(&rcad_geom::Axis::x(), 0.5).expect("a turn"));
    model
        .add_ball(1.0, &high_and_turned)
        .expect("a ball of radius one");
    assert_eq!(1e20 + 1.0, 1e20);

    let refused = rcad_step::write(&model, "a turned ball");
    assert!(
        matches!(refused, Err(CannotWrite::AWireThatStandsStillAtAPole)),
        "expected the pole to stop it, got {refused:?}"
    );
}

/// A ball a file cut open along a slit off the `z` line — here its poles are
/// along `x` — is read as the whole ball, with its poles on the `z` line and
/// the seam `add_ball` cuts. Written out again it is the file of that ball.
///
/// The slit is the file's: one `EDGE_CURVE` walked there and back, between
/// two corners where this kernel's ball does not pinch. Walked as written it
/// is a wire of two steps, which `SolidBuilder` refuses as too short to bound
/// anything (`docs/decisions.md` §404).
#[test]
fn a_ball_a_file_cut_open_off_the_z_line_is_read_as_the_same_ball() {
    let text = a_ball_cut_open_along_a_slit(["3.,2.,3.", "-1.,2.,3."], "2.", "1.,2.,3.", "2.");
    let back = the_slit_file_read(&text).expect("a ball cut open off z is read");
    assert_eq!(
        (
            back.faces().count(),
            back.edges().count(),
            back.vertices().count()
        ),
        (1, 1, 2)
    );
    assert_eq!(
        the_poles_of(&back),
        [Point::new(1.0, 2.0, 1.0), Point::new(1.0, 2.0, 5.0)]
    );
    let mut moved = Model::new(back.scale());
    moved
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(1.0, 2.0, 3.0)).expect("a move"),
        )
        .expect("a ball");
    assert_eq!(
        rcad_step::write(&back, "a ball").expect("the ball read is written"),
        rcad_step::write(&moved, "a ball").expect("a moved ball is written")
    );
}

/// The slit's corners at `centre ± 2·(cos 1, sin 1, 0)`, written in twelve
/// significant digits the way a real file writes them. Each misses the ball
/// by about `1e-12`, thousands of roundings, and is read as on it: the corners
/// are not read into the solid, so they are held to the digits the file was
/// written in (`docs/decisions.md` §404).
#[test]
fn a_slit_written_in_twelve_digits_is_read_as_the_whole_ball() {
    let text = a_ball_cut_open_along_a_slit(
        [
            "2.08060461174,3.68294196962,3.",
            "-0.0806046117363,0.317058030384,3.",
        ],
        "2.",
        "1.,2.,3.",
        "2.",
    );
    let back = the_slit_file_read(&text).expect("a twelve-digit slit is read");
    assert_eq!(
        the_poles_of(&back),
        [Point::new(1.0, 2.0, 1.0), Point::new(1.0, 2.0, 5.0)]
    );
}

/// A slit whose circle, or one of whose corners, is a thousandth off the ball
/// is not read as the whole ball: the file is walked as written and refused.
#[test]
fn a_slit_a_thousandth_off_its_ball_is_refused() {
    for text in [
        a_ball_cut_open_along_a_slit(["3.,2.,3.", "-1.,2.,3."], "2.002", "1.,2.,3.", "2."),
        a_ball_cut_open_along_a_slit(["3.002,2.,3.", "-1.,2.,3."], "2.", "1.,2.,3.", "2."),
    ] {
        let read = the_slit_file_read(&text);
        assert!(read.is_err(), "read as {read:?}");
    }
}

/// A ball too small beside its height for its poles to stand apart on the
/// `z` line is not read on it either, the writer's limit from the other side.
#[test]
fn a_slit_ball_too_small_for_its_height_is_refused() {
    let text =
        a_ball_cut_open_along_a_slit(["1.,0.,1.E20", "-1.,0.,1.E20"], "1.", "0.,0.,1.E20", "1.");
    let read = the_slit_file_read(&text);
    assert!(read.is_err(), "read as {read:?}");
}

/// The text of a file holding one ball of `radius` about `centre`, its axis
/// written along `x`, cut open along a `CIRCLE` of `circle_radius` about the
/// same centre in the plane square to `z`, between `corners`, walked there and
/// back.
fn a_ball_cut_open_along_a_slit(
    corners: [&str; 2],
    circle_radius: &str,
    centre: &str,
    radius: &str,
) -> String {
    format!(
        "ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('a ball cut open along a slit'),'2;1');
FILE_NAME('a turned slit','',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
#1=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.));
#2=(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.));
#3=(NAMED_UNIT(*)SOLID_ANGLE_UNIT()SI_UNIT($,.STERADIAN.));
#4=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-7),#1,'','');
#5=(GEOMETRIC_REPRESENTATION_CONTEXT(3)GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#4))GLOBAL_UNIT_ASSIGNED_CONTEXT((#1,#2,#3))REPRESENTATION_CONTEXT('',''));
#6=CARTESIAN_POINT('',({centre}));
#7=DIRECTION('',(1.,0.,0.));
#8=AXIS2_PLACEMENT_3D('',#6,#7,$);
#9=SPHERICAL_SURFACE('',#8,{radius});
#10=CARTESIAN_POINT('',({centre}));
#11=DIRECTION('',(0.,0.,1.));
#12=AXIS2_PLACEMENT_3D('',#10,#11,$);
#13=CIRCLE('',#12,{circle_radius});
#14=CARTESIAN_POINT('',({}));
#15=VERTEX_POINT('',#14);
#16=CARTESIAN_POINT('',({}));
#17=VERTEX_POINT('',#16);
#18=EDGE_CURVE('',#15,#17,#13,.T.);
#19=ORIENTED_EDGE('',*,*,#18,.T.);
#20=ORIENTED_EDGE('',*,*,#18,.F.);
#21=EDGE_LOOP('',(#19,#20));
#22=FACE_OUTER_BOUND('',#21,.T.);
#23=ADVANCED_FACE('',(#22),#9,.T.);
#24=CLOSED_SHELL('',(#23));
#25=MANIFOLD_SOLID_BREP('',#24);
ENDSEC;
END-ISO-10303-21;
",
        corners[0], corners[1]
    )
}

/// The model a slit file reads as, checked sound, or the reader's refusal.
fn the_slit_file_read(text: &str) -> Result<Model, String> {
    let file = StepFile::read(text.as_bytes()).expect("the slit file parses");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log).map_err(|why| format!("{why:?}"))?;
    let back = import.model().clone();
    back.validate(&policy, &mut log)
        .expect("a slit file read is sound");
    Ok(back)
}

/// The one face of the one solid of `model`.
fn the_one_face(model: &Model) -> rcad_topo::FaceView<'_> {
    let mut faces = model
        .solids()
        .flat_map(|solid| solid.faces().collect::<Vec<_>>());
    let (Some(face), None) = (faces.next(), faces.next()) else {
        panic!("a ball is one face");
    };
    face
}

/// The two poles the one face of `model` stands still at, lowest first.
fn the_poles_of(model: &Model) -> [Point; 2] {
    let mut poles: Vec<Point> = the_one_face(model)
        .wires()
        .flat_map(|wire| wire.steps().collect::<Vec<_>>())
        .filter_map(|step| {
            let rcad_topo::WireStepView::Pole(pole) = step else {
                return None;
            };
            Some(pole.point())
        })
        .collect();
    poles.sort_by(|one, other| one.coordinates()[2].total_cmp(&other.coordinates()[2]));
    let [low, high] = poles[..] else {
        panic!("a ball stands still at two poles, not at {poles:?}");
    };
    [low, high]
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
/// back the same solid, corner for corner and bit for bit, and a block cuts
/// the copy read back into the same pieces it cuts the bar as built.
///
/// The model names each long arc by a half-space whose plane is the slice, so
/// both corners are on it. `a_picker_that_tells_the_two_arcs_apart` in
/// `rcad-topo` reads a picker only when its plane holds both ends or the
/// circle's centre is not in front, and a long arc has its centre on its own
/// side of the chord. The reader used to name every arc by a plane through the
/// corners lowered by a few roundings, which put a long arc's centre in front,
/// and the copy's cut stopped at
/// [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`] (Q771). So the
/// test checks that the reader's pickers hold both ends exactly, as the
/// model's do, and that the cut comes out the same.
#[test]
fn the_long_side_of_a_bar_sliced_along_its_length_comes_back_the_same_and_cuts_the_same() {
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

    let cutter = |model: &mut Model| a_block(model, [2.0, 3.0, 2.0], Vector::new(-1.0, 3.0, 3.0));
    let cut_here = cut_by_a_third_solid(&model, built, &cutter, deflection)
        .unwrap_or_else(|why| panic!("the long side is not cut: {why}"));
    let left: f64 = cut_here.iter().map(|piece| piece.volume()).sum();
    assert!(
        !cut_here.is_empty() && left < here.volume(),
        "the cut has to leave something and take something away, or the comparison below \
         compares two solids nothing happened to: {left} of {}",
        here.volume()
    );
    let cut_back = cut_by_a_third_solid(there, came, &cutter, deflection)
        .unwrap_or_else(|why| panic!("the long side read back is not cut: {why}"));
    assert_eq!(cut_here.len(), cut_back.len(), "pieces");
    for (one, other) in cut_here.iter().zip(&cut_back) {
        assert_eq!(one.volume(), other.volume(), "a piece's volume");
        assert_eq!(one.surface_area(), other.surface_area(), "a piece's area");
        assert_eq!(one.centroid(), other.centroid(), "a piece's centroid");
    }

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
    for (whose, sides) in [
        (
            "the model's own",
            where_the_arcs_ends_and_centre_are(&model, built),
        ),
        (
            "the reader's",
            where_the_arcs_ends_and_centre_are(there, came),
        ),
    ] {
        assert_eq!(sides.len(), 4, "two arcs, each walked by two faces");
        assert!(
            sides
                .iter()
                .all(|[start, end, centre]| *start == Decision::Zero
                    && *end == Decision::Zero
                    && *centre == Decision::Pos),
            "{whose} pickers hold both ends of each long arc and have its centre in front: \
             {sides:?}"
        );
    }
}

/// The long side of a bar of radius `5s`, `s = 1 + 2⁻³⁰`, sliced along its
/// length through `(4s, 3s)` and `(−3s, −4s)`, comes back the same solid, and a
/// block cuts the copy read back into the same pieces it cuts the bar as built.
///
/// **This is the long arc Q771 left (Q939): no plane holds both its ends.** The
/// reader names the arc from `A` to `B` by a half-space facing `(B − A) × w`,
/// here `±(−7s, 7s, 0)`, or that scaled to length one. Dotted with either end the
/// first gives `−7s²`, which is 63 bits wide, and the second gives `∓c·s` with
/// `c` the rounded `1/√2`, which is `c + c·2⁻³⁰` and wider than 53 bits; neither
/// is an `f64`, so no offset puts an end on either plane. The test does not lean on
/// that arithmetic: for each normal it asks the predicate whether the plane
/// through one end at the nearest offset holds both, which any plane holding
/// both would have to be. So the reader keeps its lowered plane, with the
/// circle's centre in front (`a_half_space_naming_the_arc` in `brep.rs`), and
/// the test checks that too — the picker that neither way of
/// `a_picker_that_tells_the_two_arcs_apart` in `rcad-topo` reads. The model's own
/// picker faces `(−1, 1, 0)`, which dots with both ends to `−s` exactly.
///
/// **And the copy read back cuts the same, so for this solid there is no
/// defect.** A fuse reads an arc by `the_way_into_an_arc`, whose third way,
/// `into_the_arc_on_the_centres_side` (Q971, asked from the fuse's arrangement
/// since Q1033), reads the longer arc by the middle of the other arc being
/// behind its picker — as true of a lowered plane as of one through the chord.
/// The bar as built needs that third way here too, because the fuse solves each
/// corner into a box its plane runs through: with it taken out, the cut of the
/// bar as built is refused with "an edge of this face runs along part of a
/// curve rather than the whole of one".
#[test]
fn a_long_arc_whose_ends_no_plane_holds_comes_back_and_cuts_the_same() {
    use rcad_geom::{HalfSpace, Picker};
    use rcad_predicates::{Decision, Point as PredicatePoint};

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let s = 1.0 + f64::from_bits((1023 - 30) << 52);
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0 * s).expect("a working size"));
    let built = a_bar_sliced_along_a_chord(
        &mut model,
        5.0 * s,
        [[4.0 * s, 3.0 * s], [-3.0 * s, -4.0 * s]],
        Vector::new(-1.0, 1.0, 0.0),
        Side::Long,
    );

    let text = rcad_step::write(&model, "a bar sliced through two ends no plane holds")
        .unwrap_or_else(|why| panic!("the bar is not written: {why}"));
    let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
    let import = Import::read(&file, &policy, &mut log)
        .unwrap_or_else(|why| panic!("the bar is refused on the way back in: {why}"));
    let [(_, came)] = import.solids() else {
        panic!("one solid came back as {}", import.solids().len());
    };
    let came = *came;
    let there = import.model();
    there
        .validate(&policy, &mut log)
        .unwrap_or_else(|why| panic!("the bar comes back unsound: {why}"));
    assert_eq!(there.scale(), model.scale(), "the scale moved");

    let on =
        |half: &HalfSpace, at: Point| half.side_of_point(PredicatePoint::from(at.coordinates()));
    let the_long_arcs = |model: &Model, solid: SolidKey| {
        let mut arcs = Vec::new();
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
                    arcs.push((*half, [edge.start().point(), edge.end().point()]));
                }
            }
        }
        assert_eq!(arcs.len(), 4, "two arcs, each walked by two faces");
        arcs
    };
    for (half, [start, end]) in the_long_arcs(&model, built) {
        assert_eq!(
            [on(&half, start), on(&half, end)],
            [Decision::Zero; 2],
            "the model's own picker holds both ends"
        );
    }
    for (half, [start, end]) in the_long_arcs(there, came) {
        let centre = Point::new(0.0, 0.0, start.z());
        let sides = [start, end, centre].map(|at| on(&half, at));
        assert!(
            sides[..2]
                .iter()
                .all(|side| matches!(side, Decision::Zero | Decision::Pos))
                && sides[..2] != [Decision::Zero; 2]
                && sides[2] == Decision::Pos,
            "the reader's picker is the lowered plane, both ends on or in front of it and \
             the centre in front: {sides:?}"
        );
        let facing = half.normal();
        for normal in [
            facing,
            facing.normalized().expect("a picker's normal has length"),
        ] {
            let through = HalfSpace::through_a_point_facing(start, normal)
                .expect("a picker's normal has length");
            assert_ne!(
                [on(&through, start), on(&through, end)],
                [Decision::Zero; 2],
                "a plane facing {normal:?} holds both ends"
            );
        }
    }

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
            .unwrap_or_else(|why| panic!("the bar does not measure: {why}"))
    });
    assert_eq!(here.volume(), back.volume(), "volume");
    assert_eq!(here.surface_area(), back.surface_area(), "area");
    assert_eq!(here.centroid(), back.centroid(), "centroid");

    let cutter = |model: &mut Model| a_block(model, [2.0, 3.0, 2.0], Vector::new(-1.0, 3.0, 3.0));
    let cut_here = cut_by_a_third_solid(&model, built, &cutter, deflection)
        .unwrap_or_else(|why| panic!("the bar is not cut: {why}"));
    let left: f64 = cut_here.iter().map(|piece| piece.volume()).sum();
    assert!(
        !cut_here.is_empty() && left < here.volume(),
        "the cut has to leave something and take something away: {left} of {}",
        here.volume()
    );
    let cut_back = cut_by_a_third_solid(there, came, &cutter, deflection)
        .unwrap_or_else(|why| panic!("the bar read back is not cut: {why}"));
    assert_eq!(cut_here.len(), cut_back.len(), "pieces");
    for (one, other) in cut_here.iter().zip(&cut_back) {
        assert_eq!(one.volume(), other.volume(), "a piece's volume");
        assert_eq!(one.surface_area(), other.surface_area(), "a piece's area");
        assert_eq!(one.centroid(), other.centroid(), "a piece's centroid");
    }
}

/// The long side of a bar sliced a hair inside its rim, so that the short arc
/// left on each cap stands `7.7e-16` off its chord, is refused as built and
/// refused read back from a file, and by the same refusal (Q1460).
///
/// **The reader's plane would not tell this arc from the other one.** No plane
/// holds both ends of the long arc, so the reader names it by a plane lowered
/// below both ends by `8ε` times their spread (`a_half_space_naming_the_arc` in
/// `brep.rs`, and Q939's test above), and here the short arc is about a tenth
/// as thick as that lowering. So the lowered plane passes under the whole
/// circle, and the test checks that exactly: the plane misses the circle and
/// the centre is in front of it. The reader's rule is written out here, because
/// the reader refuses this file before it hands a picker back; written out, it
/// gives the very picker the reader gives a thicker bar it does read back.
///
/// **But the bar as built never gets that far.** `rcad_geom::walk` asks which
/// of the two arcs a picker holds at the middle of each, inside a box `256ε`
/// times the circle's size wide. The short arc's middle is inside that box from
/// the builder's plane through the chord, so the walk reads both arcs as picked
/// and refuses with `CannotNameWhichPieceOfACrossingThisIs`. The test calls the
/// walk on the arc itself: it walks the short side's picker and refuses the long
/// side's. The builder's plane holds both ends exactly, so which arc it picks
/// has an answer, which `a_picker_that_tells_the_two_arcs_apart` in `rcad-topo`
/// reads, and the walk not reaching it is a gap of its own.
///
/// **So the original and the copy refuse alike, and for this bar there is no
/// read-back defect to fix.** The walk's refusal reaches thicker arcs than this
/// one: a bar whose short arc is about three times as thick as the reader's
/// lowering is refused as well, and one about a hundred times as thick builds
/// and comes back. A file of the refused bar is the thick one's file with the
/// numbers swapped, since the writer is never handed a model to write.
///
/// The ends are points of the circle `x² + y² = R²` one step apart each way,
/// `(x + 1, −x)` and `(x, −(x + 1))`, whose radii solve `u² − 2R² = −1` with
/// `u = 2x + 1`. Of those, `R = 38613965` is the first whose short arc is
/// thinner than the lowering, `6625109` the one before and `1136689` the one
/// before that. Each is scaled by `(1 + 2⁻²⁰)` and a power of two, which keeps
/// every point exactly on its circle and leaves the reader no plane through
/// both ends.
#[test]
fn a_bar_sliced_a_hair_inside_its_rim_is_refused_built_and_read_back_alike() {
    use rcad_geom::{Axis, Cylinder, HalfSpace, NamedRun, Picker};
    use rcad_predicates::{Decision, Point as PredicatePoint};
    use rcad_step::Refusal;

    let scaled = |[near, far, round]: [f64; 3], by: f64| {
        assert_eq!(
            near * near + far * far,
            round * round,
            "both ends on the circle"
        );
        let s = 1.0 + f64::from_bits((1023 - 20) << 52);
        [near, far, round].map(|number| number * s * by)
    };
    let thin = scaled([27_304_196.0, 27_304_197.0, 38_613_965.0], 2f64.powi(-23));
    let thicker = scaled([4_684_659.0, 4_684_660.0, 6_625_109.0], 2f64.powi(-21));
    let thick = scaled([803_760.0, 803_761.0, 1_136_689.0], 2f64.powi(-18));
    let ends =
        |[near, far, _]: [f64; 3]| [Point::new(far, -near, 0.0), Point::new(near, -far, 0.0)];
    let a_model = || Model::new(Scale::new(Unit::Millimetre, 8.0).expect("a working size"));
    let a_bar = |numbers: [f64; 3], model: &mut Model| {
        let [near, far, round] = numbers;
        building_a_bar_sliced_along_a_chord(
            model,
            round,
            [[far, -near], [near, -far]],
            Vector::new(-1.0, 1.0, 0.0),
            Side::Long,
        )
    };
    let walk_refused = |why: &rcad_topo::Error| {
        matches!(
            why,
            rcad_topo::Error::Geometry(rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs)
        )
    };

    for numbers in [thin, thicker] {
        let built = a_bar(numbers, &mut a_model());
        assert!(
            built.as_ref().is_err_and(walk_refused),
            "the bar of radius {} is refused as built by the walk: {built:?}",
            numbers[2]
        );
    }

    let [_, _, round] = thin;
    let corners = ends(thin);
    let bottom = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("an axis direction has length");
    let pipe = Cylinder::new(Axis::z(), round).expect("a positive radius");
    let walked = |facing: Vector| {
        let picker = HalfSpace::through_a_point_facing(corners[0], facing)
            .expect("a direction square to the chord has length");
        let run = NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(bottom),
                picked_by: [Picker::from(picker)].into(),
            },
            corners.map(|corner| PredicatePoint::from(corner.coordinates())),
        )
        .expect("an arc with two ends");
        rcad_geom::walk(
            &run,
            [&Surface::Plane(bottom), &Surface::Cylinder(pipe)],
            the_same_mesh_on_both_sides(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|along| along.points().len())
    };
    assert!(
        walked(Vector::new(1.0, -1.0, 0.0)).is_ok(),
        "the short arc is walked"
    );
    let long = walked(Vector::new(-1.0, 1.0, 0.0));
    assert!(
        matches!(
            long,
            Err(rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs)
        ),
        "the walk cannot tell the long arc from the short: {long:?}"
    );

    let mut thick_model = a_model();
    a_bar(thick, &mut thick_model)
        .unwrap_or_else(|why| panic!("the thick bar is not sound: {why}"));
    let thick_text = rcad_step::write(&thick_model, "a bar sliced a hair inside its rim")
        .unwrap_or_else(|why| panic!("the thick bar is not written: {why}"));
    let read = |text: &str| {
        let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
        Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
    };
    let import =
        read(&thick_text).unwrap_or_else(|why| panic!("the thick bar is refused read back: {why}"));
    let [(_, came)] = import.solids() else {
        panic!("one solid came back as {}", import.solids().len());
    };
    let mut pickers = Vec::new();
    for face in import
        .model()
        .solid(*came)
        .expect("this model made the key")
        .faces()
    {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                if let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = coedge.edge().run() {
                    pickers.push(picked_by.clone());
                }
            }
        }
    }
    let readers_own = Picker::from(the_readers_lowered_plane(ends(thick)));
    assert_eq!(pickers.len(), 4, "two arcs, each walked by two faces");
    for picked_by in &pickers {
        assert_eq!(
            picked_by.as_slice(),
            [readers_own],
            "the reader's rule, written out"
        );
    }
    assert!(
        !the_whole_circle_is_in_front_of(&the_readers_lowered_plane(ends(thick)), thick[2]),
        "the thick bar's short arc dips behind the reader's plane"
    );

    let lowered = the_readers_lowered_plane(corners);
    assert!(
        the_whole_circle_is_in_front_of(&lowered, round),
        "the thin bar's whole circle is in front of the reader's plane: {lowered:?}"
    );
    let facing = lowered.normal();
    for normal in [
        facing,
        facing.normalized().expect("a picker's normal has length"),
    ] {
        let through = HalfSpace::through_a_point_facing(corners[0], normal)
            .expect("a picker's normal has length");
        assert_ne!(
            corners.map(|corner| through.side_of_point(PredicatePoint::from(corner.coordinates()))),
            [Decision::Zero; 2],
            "a plane facing {normal:?} holds both ends"
        );
    }

    let thin_text = the_same_file_with_other_numbers(&thick_text, thick, thin);
    let got = read(&thin_text).map(|import| import.solids().len());
    assert!(
        matches!(
            &got,
            Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }) if walk_refused(why)
        ),
        "the thin bar's file is refused by the walk, as the bar is: {got:?}"
    );
}

/// The half-space the reader names a cap's long arc between `ends` by, where no
/// plane holds both ends: facing `(B − A) × ẑ`, lowered below both ends by `8ε`
/// times their spread, as `a_half_space_holding_both` in `brep.rs` works it out.
fn the_readers_lowered_plane(ends: [Point; 2]) -> rcad_geom::HalfSpace {
    let facing = (ends[1] - ends[0]).cross(Vector::new(0.0, 0.0, 1.0));
    let below = |corner: Point| {
        let terms = facing.components().into_iter().zip(corner.coordinates());
        let spread: f64 = terms.clone().map(|(n, p)| (n * p).abs()).sum();
        let dot: f64 = terms.map(|(n, p)| n * p).sum();
        dot - 8.0 * f64::EPSILON * spread
    };
    rcad_geom::HalfSpace::new(facing, below(ends[0]).min(below(ends[1])))
        .expect("a direction square to the chord has length")
}

/// Whether every point of the circle of `radius` round the origin in the plane
/// `z = 0` is in front of `half`, worked out exactly: its centre is in front,
/// and its plane misses the circle, `R²(fx² + fy²) < e²`.
fn the_whole_circle_is_in_front_of(half: &rcad_geom::HalfSpace, radius: f64) -> bool {
    use rcad_predicates::Decision;
    use rcad_predicates::expansion::{expansion_sign, expansion_sum, scale_expansion};

    let normal = half.normal();
    let offset = half.offset();
    let [mut x_squared, mut y_squared] = [[0.0; 2]; 2];
    let x_length = scale_expansion(&[normal.x()], normal.x(), &mut x_squared);
    let y_length = scale_expansion(&[normal.y()], normal.y(), &mut y_squared);
    let mut flat_squared = [0.0; 4];
    let flat_length = expansion_sum(
        &x_squared[..x_length],
        &y_squared[..y_length],
        &mut flat_squared,
    );
    let mut times_radius = [0.0; 8];
    let once = scale_expansion(&flat_squared[..flat_length], radius, &mut times_radius);
    let mut times_radius_squared = [0.0; 16];
    let twice = scale_expansion(&times_radius[..once], radius, &mut times_radius_squared);
    let mut less_offset_squared = [0.0; 2];
    let offset_length = scale_expansion(&[-offset], offset, &mut less_offset_squared);
    let mut difference = [0.0; 18];
    let difference_length = expansion_sum(
        &times_radius_squared[..twice],
        &less_offset_squared[..offset_length],
        &mut difference,
    );
    assert_eq!(normal.z(), 0.0, "a plane square to the caps");
    let centre_in_front =
        half.side_of_point(rcad_predicates::Point::from([0.0; 3])) == Decision::Pos;
    centre_in_front && expansion_sign(&difference[..difference_length]) == Decision::Neg
}

/// `text`, a file of the bar [`a_bar_sliced_along_a_chord`] builds through the
/// ends and radius in `from`, with every number of those changed to the ones in
/// `to`, and the point the slice's plane is written through moved onto the
/// first end of `to`.
fn the_same_file_with_other_numbers(text: &str, from: [f64; 3], to: [f64; 3]) -> String {
    let mut swapped = text.to_owned();
    for (old, new) in from.into_iter().zip(to) {
        let [old, new] = [old, new].map(|number| format!("{number:?}"));
        assert!(swapped.contains(&old), "{old} is written");
        swapped = swapped.replace(&old, &new);
    }
    let mut lines: Vec<String> = swapped.lines().map(str::to_owned).collect();
    let slice_facing = lines
        .iter()
        .position(|line| {
            line.ends_with("DIRECTION('',(0.7071067811865476,-0.7071067811865476,0.0));")
        })
        .expect("the slice's facing is written");
    let (entity, _) = lines[slice_facing - 1]
        .split_once('=')
        .expect("an entity before it");
    let [near, far, _] = to;
    lines[slice_facing - 1] = format!("{entity}=CARTESIAN_POINT('',({far:?},{:?},0.0));", -near);
    lines.join("\n")
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
    a_bar_sliced_along_a_chord(
        model,
        5.0,
        [[4.0, -3.0], [-4.0, -3.0]],
        Vector::new(0.0, 1.0, 0.0),
        side,
    )
}

/// A bar of `radius` about `z`, four tall, sliced along its length by the plane
/// through `ends`, two points of its circle, and kept on `side`, built face by
/// face: bottom cap, pipe, top cap, slice.
///
/// `towards_the_axis` is square to the chord and points from it to the bar's
/// axis. Each cap's arc is picked by the half-space through the first end facing
/// it, for the long side, or facing away, for the short; that plane holds both
/// ends exactly where `towards_the_axis` dotted with each end gives one `f64`.
fn a_bar_sliced_along_a_chord(
    model: &mut Model,
    radius: f64,
    ends: [[f64; 2]; 2],
    towards_the_axis: Vector,
    side: Side,
) -> SolidKey {
    building_a_bar_sliced_along_a_chord(model, radius, ends, towards_the_axis, side)
        .unwrap_or_else(|why| panic!("the bar sliced along its length is not sound: {why}"))
}

/// [`a_bar_sliced_along_a_chord`], with what the builder said when it refuses.
fn building_a_bar_sliced_along_a_chord(
    model: &mut Model,
    radius: f64,
    ends: [[f64; 2]; 2],
    towards_the_axis: Vector,
    side: Side,
) -> Result<SolidKey, rcad_topo::Error> {
    use rcad_geom::{Axis, Cylinder, HalfSpace, Orientation, Picker};
    use rcad_topo::{DraftStep, SolidBuilder};

    let height = 4.0;
    let into_the_kept_side = match side {
        Side::Long => towards_the_axis,
        Side::Short => -towards_the_axis,
    };
    let [[from_x, from_y], [to_x, to_y]] = ends;
    let on_the_chord = Point::new(from_x, from_y, 0.0);
    let a_plane = |at: Point, facing: Vector| {
        Plane::from_point_and_normal(at, facing).expect("an axis direction has length")
    };
    let bottom = a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0));
    let top = a_plane(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0));
    let slice = a_plane(on_the_chord, -into_the_kept_side);
    let pipe = Cylinder::new(Axis::z(), radius).expect("a positive radius");
    let the_arc_on = |cap: Plane| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cap),
        picked_by: [Picker::from(
            HalfSpace::through_a_point_facing(on_the_chord, into_the_kept_side)
                .expect("a direction square to the chord has length"),
        )]
        .into(),
    };

    let mut builder = SolidBuilder::new();
    let corners = [
        (from_x, from_y, 0.0),
        (to_x, to_y, 0.0),
        (from_x, from_y, height),
        (to_x, to_y, height),
    ]
    .map(|(x, y, z)| {
        builder
            .add_vertex(Point::new(x, y, z))
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
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
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
/// Not [`Covered::the_same_solid_comes_back`], because this test also checks
/// that the scoop is a curved face used reversed, and that each cap meets it
/// in one arc. The cut by a third solid is required to answer on both sides,
/// each side's volume checked against the closed form.
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
            .unwrap_or_else(|why| panic!("the slab less the block does not answer: {why}"))
    });
    // The slab's own closed form is 4 times the area between the scoop
    // circle's chord at y = -3 and the one at y = -5:
    // 24 - (12 + 25 * asin(0.8) - 24). Of the block, only x in [-1, 1],
    // y in [-6, -5.5], z in [1, 3] lies in the slab's back, short of where
    // the scoop has eaten into it; the rest of the block is outside the
    // slab's y range or inside the scoop's void, so 2, not the block's own
    // 6, comes off.
    let slab = 4.0 * (24.0 - (12.0 + 25.0 * 0.8_f64.asin() - 24.0));
    let closed_form = slab - 2.0;
    for pieces in [&cut_here, &cut_back] {
        let [piece] = pieces.as_slice() else {
            panic!("the slab less the block is one solid, not {}", pieces.len());
        };
        assert!(
            (piece.volume() - closed_form).abs() <= piece.volume_error(),
            "the slab less the block measured {} ± {}, and the closed form is {closed_form}",
            piece.volume(),
            piece.volume_error()
        );
    }
    let as_tuples = |pieces: &[Measurement]| {
        pieces
            .iter()
            .map(|piece| (piece.volume(), piece.surface_area(), piece.centroid()))
            .collect::<Vec<_>>()
    };
    assert_eq!(as_tuples(&cut_here), as_tuples(&cut_back), "the cut");
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

/// A whole ball of radius four whose seam, from pole to pole on the side
/// facing `+x`, has its plane facing the other way from the edge's own turn:
/// once with the plane turned round to face `−y` and the seam drawn down from
/// the north pole, and once with the plane facing `+y` and the seam drawn up
/// from the south pole. `add_ball` draws it down with the plane facing `+y`.
///
/// **How the plane was chosen.** Writing `A`, `B` for the seam's ends and `n`
/// for its plane's normal, the circle about `n` runs from `A` to `B` along the
/// half in front of `(B − A) × n` (`Walk::the_arc_between`). Either choice
/// above puts `(B − A) × n` along `−x`, away from the half the run picks, so
/// the edge cannot run along the circle about `n`: the writer either turns
/// the circle over to about `−n` or runs the edge against the circle.
///
/// The reader names the seam by the plane through the circle's centre about
/// the circle's own axis (`Walk::the_surface_that_names_the_curve`), so the
/// circle turned over came back with the seam's plane reversed: `(0, 1, 0)`
/// for `(0, −1, 0)` and the other way round, and the ball facing `−y`
/// measured `267.77308911728187` read back where it measured
/// `267.7805495679834` built. No ball test before this one drew such a seam:
/// `add_ball`'s runs along the circle about its plane's normal. Since Q776 the
/// writer writes the circle about `n` and runs the edge against it
/// (`EDGE_CURVE` `same_sense` false), as every other seam on a ball has done
/// since Q767, and the plane comes back bit for bit.
#[test]
fn a_ball_whose_seam_runs_against_its_planes_turn_comes_back_with_the_same_plane() {
    let north = Point::new(0.0, 0.0, 4.0);
    let south = Point::new(0.0, 0.0, -4.0);
    for (called, from, to, facing) in [
        (
            "a ball whose seam's plane faces -y",
            north,
            south,
            Vector::new(0.0, -1.0, 0.0),
        ),
        (
            "a ball whose seam runs up from the south pole",
            south,
            north,
            Vector::new(0.0, 1.0, 0.0),
        ),
    ] {
        let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
        let built = a_whole_ball_drawn_face_by_face(&mut model, [from, to], facing);
        let text = rcad_step::write(&model, called)
            .unwrap_or_else(|why| panic!("{called} is not written: {why}"));
        let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
        let import = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{called} is refused on the way back in: {why}"));
        let [(_, came)] = import.solids() else {
            panic!("one solid came back as {}", import.solids().len());
        };
        let [here, back] = [(&model, built), (import.model(), *came)].map(|(from, key)| {
            the_plane_its_seam_is_cut_from(from.solid(key).expect("this model made the key"))
        });
        assert_eq!(
            back, here,
            "{called}: the seam's plane, as the bits of its normal and its offset"
        );
        assert!(
            text.contains(",.F.);"),
            "{called}: the seam runs against its circle, or this does not reach same_sense"
        );
        comes_back_the_same(called, &model, built, &|model| {
            a_block(model, [10.0, 10.0, 5.0], Vector::new(-5.0, -5.0, -5.0))
        });
    }
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

/// The same disc with its rim named by a ball rather than a pipe is refused by
/// name, rather than written as a chord.
///
/// The ball of radius one about the disc's centre crosses the top in exactly
/// the rim, so the model is sound. The writer reads a curve drawn on a flat
/// face off a pipe and nothing else, and until Q764 wrote any other as a
/// `LINE`: both halves of the rim went out as the chord from `(1, 0, 4)` to
/// `(−1, 0, 4)`, and the reader refused the file the writer had written.
#[test]
fn a_disc_drawn_on_a_bar_and_named_by_a_ball_is_refused_by_name_rather_than_written_as_a_chord() {
    use rcad_geom::Sphere;

    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let ball = Surface::Sphere(Sphere::new(the_discs_centre(), 1.0).expect("a positive radius"));
    let built = a_bar_with_a_disc_drawn_on_its_top_by(&mut model, ball);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball through the rim says where it runs");
    assert_eq!(
        open_bends(&model, built),
        [["plane", "plane"], ["plane", "plane"]],
        "the two halves of the disc's rim"
    );
    let refused = rcad_step::write(&model, "a disc drawn on a bar by a ball")
        .expect_err("a half circle named by a ball is not written as its chord");
    assert!(
        matches!(
            refused,
            CannotWrite::ACurveTheFacesEitherSideOfItDoNotName {
                first: "plane",
                second: "plane",
                closed: false,
            }
        ),
        "refused as {refused:?}"
    );
}

/// The same disc with its rim named by the pipe of radius one about the bar's
/// own axis, whose origin is on the floor four below the top, is written as
/// the file of the disc drawn by the pipe standing on the top, and comes back
/// the same.
///
/// The writer wrote the rim's circle at the pipe's axis origin, which no face
/// along the rim holds, so the reader named the rim by the plane `z = 0` the
/// circle lay in and refused the file (Q775). It now writes the circle where
/// the axis meets the top.
#[test]
fn a_disc_drawn_on_a_bar_by_the_pipe_about_its_own_axis_comes_back_the_same() {
    use rcad_geom::{Axis, Cylinder};

    let called = "a disc drawn by the pipe about the bar's axis";
    let scale = Scale::new(Unit::Millimetre, 4.0).expect("a working size");
    let mut model = Model::new(scale);
    let about_the_bars_axis =
        Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a positive radius"));
    let built = a_bar_with_a_disc_drawn_on_its_top_by(&mut model, about_the_bars_axis);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the pipe about the bar's axis says where the rim runs");
    let text = comes_back_the_same(called, &model, built, &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(0.0, -1.0, 3.0))
    });
    let mut standing_on_the_top = Model::new(scale);
    a_bar_with_a_disc_drawn_on_its_top(&mut standing_on_the_top);
    assert_eq!(
        text,
        rcad_step::write(&standing_on_the_top, called).expect("the disc on its own top"),
        "the rim's circle is where the axis meets the top"
    );
}

/// That disc turned and moved, drawn by the pipe about the bar's axis or by
/// that pipe moved up its axis to three above the top, is written, read back
/// and sound every time, and its rim comes back named by a pipe of the axis
/// direction and radius it went out with.
///
/// Where the axis meets the turned top is worked out in `f64` and rounds, so
/// the pipe read back runs through a point an ulp or so off the model's line;
/// the reader holds the circle there all the same. The turns are those of
/// `how_often_a_turned_disc_drawn_on_a_bar_comes_back_from_its_own_file`: a
/// tenth to `6.2` about each of `x`, `y` and `z`, unmoved and moved by
/// `(3, 5, 10)`, so 372 for each pipe.
#[test]
fn every_turned_disc_drawn_by_a_pipe_off_its_top_comes_back_from_its_own_file() {
    use rcad_geom::{Axis, Cylinder};

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let (mut came_back, mut refused) = (0, Vec::new());
    for standing_at in [0.0, 7.0] {
        let drawn = Cylinder::new(
            Axis::new(
                Point::new(0.0, 0.0, standing_at),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("a direction"),
            1.0,
        )
        .expect("a positive radius");
        for about in [Axis::x(), Axis::y(), Axis::z()] {
            for tenths in 1..63 {
                for moved in [Vector::zero(), Vector::new(3.0, 5.0, 10.0)] {
                    let turned = Isometry::translation(moved)
                        .expect("a finite offset")
                        .compose(
                            &Isometry::rotation(&about, f64::from(tenths) * 0.1)
                                .expect("a finite angle about a unit axis"),
                        );
                    let mut model =
                        Model::new(Scale::new(Unit::Millimetre, 16.0).expect("a working size"));
                    let built =
                        a_bar_with_a_disc_drawn_on_its_top_by(&mut model, Surface::Cylinder(drawn));
                    model
                        .place_solid(built, &turned)
                        .expect("a bar can be turned");
                    let went = the_pipes_its_runs_name(model.solid(built).expect("its key"));
                    match the_pipes_its_runs_name_back(&model, &policy, &mut log) {
                        Ok(back)
                            if back.len() == went.len()
                                && went.iter().zip(&back).all(|(went, back)| {
                                    went.axis().direction() == back.axis().direction()
                                        && went.radius() == back.radius()
                                }) =>
                        {
                            came_back += 1;
                        }
                        Ok(back) => {
                            refused.push(format!("{turned:?}: {went:?} came back {back:?}"))
                        }
                        Err(why) => refused.push(format!("{turned:?}: {why}")),
                    }
                }
            }
        }
    }
    assert_eq!((came_back, refused.first()), (744, None));
}

/// The same disc with its rim named by a pipe of radius `0.8` whose axis
/// leans off the top's normal along `(3, 0, 4)`, so the rim is an ellipse of
/// sizes `0.8` and `1` rather than a circle, is refused by name rather than
/// written as that ellipse.
///
/// The pipe's axis runs through the disc's centre along `(3, 0, 4) / 5`, and
/// crosses the top at `(±1, 0, 4)` along its long way and would cross it at
/// `(0, ±0.8, 4)` along its short one — `1 = 0.8 / (4 / 5)`, the same
/// sizes-from-a-lean arithmetic the slanted cap uses
/// (`a_bar_with_a_slanted_cap_is_written_with_an_ellipse_and_comes_back_the_same`).
/// The model is sound: `(1, 0, 4)` and `(-1, 0, 4)` are each exactly `0.8`
/// from that axis, so this is the disc's own rim and not some other curve.
///
/// `an_arc_of_a_circle_on_a_flat_face` only ever writes a `CIRCLE` about the
/// pipe's own axis — the curve the pipe's cross-section draws square to that
/// axis — which is this rim only where the axis is square to the face. Here
/// it leans, so the function's `held` check fails exactly as it does for a
/// run named by a ball, and the write refuses by name instead of writing the
/// ellipse the plane and pipe actually cross in.
///
/// **This is a limit, not a gap** (`docs/decisions.md` §16.10). The writer
/// could draw that ellipse — `where_a_plane_crosses_a_pipe` is a pure
/// function of the plane and the pipe, not of whether the pipe is a face —
/// but two pipes leaning the opposite way off the rim's long axis cross the
/// top in the same ellipse, and with no face here naming either pipe, no
/// reader has grounds to pick one over the other. The reader already says as
/// much for an `ELLIPSE` between two flat faces
/// (`Walk::the_surface_that_names_the_curve`'s doc comment, `brep.rs`): "no
/// single surface this kernel holds crosses a plane in a given ellipse and is
/// written down from the ellipse alone without choosing a slope for it — … a
/// tilted cylinder would do, and which tilt is a choice this reader has no
/// consumer to make it for". So no file this reader reads could carry this
/// rim: the curve alone does not say which pipe drew it, and the only face
/// that could is the one Q775 found this exact model does not have.
#[test]
fn a_disc_drawn_on_a_bar_by_a_leaning_pipe_is_refused_by_name_rather_than_written_as_an_ellipse() {
    use rcad_geom::{Axis, Cylinder};

    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let leaning = Surface::Cylinder(
        Cylinder::new(
            Axis::new(the_discs_centre(), Vector::new(3.0, 0.0, 4.0)).expect("a direction"),
            0.8,
        )
        .expect("a positive radius"),
    );
    let built = a_bar_with_a_disc_drawn_on_its_top_by(&mut model, leaning);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the leaning pipe crosses the top in exactly the rim");
    assert_eq!(
        open_bends(&model, built),
        [["plane", "plane"], ["plane", "plane"]],
        "the two halves of the disc's rim"
    );
    let refused = rcad_step::write(&model, "a disc drawn on a bar by a leaning pipe")
        .expect_err("an ellipse drawn by a leaning pipe is not written between two flat faces");
    assert!(
        matches!(
            refused,
            CannotWrite::ACurveTheFacesEitherSideOfItDoNotName {
                first: "plane",
                second: "plane",
                closed: false,
            }
        ),
        "refused as {refused:?}"
    );
}

/// Every pipe a run of `solid` names, where the solid puts it, in the order
/// its faces, wires and coedges are walked.
fn the_pipes_its_runs_name(solid: rcad_topo::SolidView<'_>) -> Vec<rcad_geom::Cylinder> {
    let mut pipes = Vec::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                if let EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Cylinder(pipe),
                    ..
                } = coedge.edge().run()
                {
                    pipes.push(pipe);
                }
            }
        }
    }
    pipes
}

/// [`the_pipes_its_runs_name`] for the one solid of `model` written out and
/// read back, which has to come back sound.
fn the_pipes_its_runs_name_back(
    model: &Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<rcad_geom::Cylinder>, String> {
    let text = rcad_step::write(model, "a turned disc").map_err(|why| format!("{why:?}"))?;
    let file = StepFile::read(text.as_bytes()).map_err(|why| why.to_string())?;
    let import = Import::read(&file, policy, log).map_err(|why| why.to_string())?;
    let [(_, came)] = import.solids() else {
        return Err(format!("{} solids came back", import.solids().len()));
    };
    let there = import.model();
    there.validate(policy, log).map_err(|why| why.to_string())?;
    Ok(the_pipes_its_runs_name(
        there.solid(*came).expect("this model made the key"),
    ))
}

/// Half a bar whose straight edge across the floor is named by a pipe it is
/// one of the straight lines of, rather than said to be straight, is written
/// with a `LINE` and comes back the same.
///
/// The floor and the flat side are two planes that cross in that edge, and
/// the pipe of radius five along `x` through `(0, 3, 4)` holds it as well.
/// The walk draws an edge straight where the face's plane holds the pipe's
/// axis direction. The writer sent every open edge between two flat faces
/// whose run names a pipe to an arc of that pipe's circle, which has no arc
/// running along the axis, and refused the edge by name (Q775).
///
/// The cut stays clear of the flat side: a block across the floor's edge is
/// refused on the solid that went out, as an edge that runs along part of a
/// curve rather than the whole of one, and cut on the one that came back, whose
/// edge is straight.
#[test]
fn a_bar_whose_straight_edge_is_named_by_a_pipe_it_lies_along_is_written_with_a_line_and_comes_back_the_same()
 {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let built = half_a_bar_whose_edge_across_the_floor_is_named_by_a_pipe(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [["pipe", "plane"], ["pipe", "plane"], ["plane", "plane"]],
        "the two half circles and the edge across the floor"
    );
    let text = comes_back_the_same(
        "half a bar whose edge across the floor is named by a pipe",
        &model,
        built,
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(-1.0, 1.0, 3.0)),
    );
    assert_eq!(
        text.matches("LINE(").count(),
        4,
        "the edge across the floor, the one across the top and the two up the sides"
    );
}

/// A bar whose seam is named by the plane through its axis, rather than said to
/// be straight, is written with a `LINE` and comes back the same.
///
/// That is the one run an open edge with the one pipe on both sides is written
/// as a `LINE` from, since Q764: a plane that holds two points of the wall at
/// one place round the axis holds the straight line between them. Read back,
/// the seam is straight, and the solid is the same one, cut and all.
#[test]
fn a_bar_whose_seam_is_named_by_the_plane_through_its_axis_is_written_with_a_line_and_comes_back_the_same()
 {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let built = a_bar_whose_seam_is_named_by_the_plane_through_its_axis(&mut model);
    assert_eq!(open_bends(&model, built), [["pipe", "pipe"]], "the seam");
    let text = comes_back_the_same(
        "a bar whose seam is named by its plane",
        &model,
        built,
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 1.0)),
    );
    assert_eq!(text.matches("LINE(").count(), 1, "the seam");
}

/// The outer side of a rod bent round a ring, whose ring meets a pipe in four
/// arcs: the tube's circle where a straight pipe carries on from the bend, and
/// the circles where the pipe about the ring's own axis crosses it.
///
/// `kicad/Relay_SPDT_Finder_34.51_Vertical.step` bends its pins this way, and
/// until Q773 the writer refused an edge between a pipe and a ring. It is
/// written the way an arc between a pipe and a ball is, in the plane its run
/// names, where that plane crosses the pipe. The relay then stopped on an
/// edge between a flat face and a ring, until Q774: see
/// `the_corner_of_a_slab_with_a_round_edge_is_written_where_its_faces_rest_on_the_ring`.
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

/// The corner of a slab with a round edge, whose top and bottom faces rest on
/// the ring the edge bends round and touch it along a quarter of the circle of
/// the ring's own radius.
///
/// `kicad/Relay_SPDT_Finder_34.51_Vertical.step` and the two
/// `Relay_1-Form-*_Schrack-RYII_RM5mm` relays round the corners of flat pins
/// this way, and until Q774 the writer refused any open edge between a flat
/// face and a ring. It now writes the circle where the face rests on the ring,
/// turned so that the arc the reader picks is the one the model holds. The
/// top arc's run names the top face's own plane and the bottom arc's names the
/// bottom face's plane turned round, which are the two the relays carry.
#[test]
fn the_corner_of_a_slab_with_a_round_edge_is_written_where_its_faces_rest_on_the_ring() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    let built = the_corner_of_a_slab_with_a_round_edge(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [
            ["pipe", "plane"],
            ["pipe", "plane"],
            ["pipe", "ring"],
            ["pipe", "ring"],
            ["plane", "ring"],
            ["plane", "ring"],
        ],
        "where the flat faces rest on the ring, where the round edge carries on \
         straight, and where the ends cut it"
    );
    comes_back_the_same(
        "the corner of a slab with a round edge",
        &model,
        built,
        &|model| a_block(model, [2.0, 2.0, 4.0], Vector::new(2.0, 2.0, -2.0)),
    );
}

/// The top of a bent rod sawn off flat at `z = 3`, three of its radius of
/// five above its middle, is written with the two circles the saw cut crosses
/// its ring in, and comes back the same.
///
/// A plane square to a ring's axis and nearer its centre than the tube's
/// radius crosses it in two circles about the axis, of radii `R ± √(r² − h²)`:
/// here `10 ± √(25 − 9)`, which is `14` and `6` with no rounding. The flat
/// face meets the ring along an open quarter of each, so each circle carries
/// an open edge between a plane and a ring. Until Q778 the writer wrote only
/// the circle a plane resting on the tube touches it along, and refused this
/// as `{ first: "plane", second: "ring", closed: false }`.
#[test]
fn the_top_of_a_bent_rod_sawn_off_flat_is_written_with_both_circles_the_cut_crosses_its_ring_in() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 14.0).expect("a working size"));
    let built = the_top_of_a_bent_rod_sawn_off_flat(&mut model);
    assert_eq!(
        open_bends(&model, built),
        [
            ["pipe", "plane"],
            ["pipe", "plane"],
            ["pipe", "ring"],
            ["pipe", "ring"],
            ["plane", "ring"],
            ["plane", "ring"],
        ],
        "the outer and the inner circle where the cut crosses the ring, where the \
         ring carries on as a pipe, and where the ends cut the pipes"
    );
    let text = comes_back_the_same(
        "the top of a bent rod sawn off flat",
        &model,
        built,
        &|model| a_block(model, [2.0, 2.0, 4.0], Vector::new(7.0, 7.0, 2.0)),
    );
    assert_eq!(
        the_radii_of_the_circles_written(&text),
        [5.0, 5.0, 5.0, 5.0, 6.0, 14.0].map(f64::to_bits),
        "the tube's circle at each end of each pipe and of the bend, and the \
         inner and the outer circle the cut crosses the ring in"
    );
}

/// The radius of every `CIRCLE` in the file, as its bits, smallest first.
fn the_radii_of_the_circles_written(text: &str) -> Vec<u64> {
    let mut radii: Vec<f64> = text
        .lines()
        .filter_map(|line| line.strip_suffix(");")?.split_once("=CIRCLE("))
        .map(|(_, body)| {
            let (_, radius) = body
                .rsplit_once(',')
                .unwrap_or_else(|| panic!("a CIRCLE with no radius: {body}"));
            radius.parse::<f64>().expect("a radius")
        })
        .collect();
    radii.sort_by(f64::total_cmp);
    radii.into_iter().map(f64::to_bits).collect()
}

/// The cap the plane `z = 3` cuts off a ball of radius five, whose rim is one
/// closed circle between the flat floor and the ball, is written and comes
/// back the same, its rim walked the same way round by both faces.
///
/// Until Q777 the writer wrote an edge between a flat face and a ball only
/// where it stopped short of right round, and refused this one by name as
/// `{ first: "plane", second: "ball", closed: true }`. A closed edge is stored
/// running the way the ball winds, about `+z`, and the reader reads which way
/// each face walks it off the circle's own turn. The floor faces `−z`, so the
/// circle the floor's plane crosses the ball in turns about `−z`, and the
/// writer turns it over.
#[test]
fn a_cap_cut_off_a_ball_along_one_closed_circle_is_written_and_comes_back_the_same() {
    let called = "a cap cut off a ball";
    let mut model = Model::new(Scale::new(Unit::Millimetre, 5.0).expect("a working size"));
    let built = a_ball_cut_by_a_flat_face_along_one_closed_circle(&mut model);
    let text = comes_back_the_same(called, &model, built, &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(-1.0, -1.0, 3.5))
    });
    assert_eq!(
        the_closed_circles_written(&text),
        [([-0.0, -0.0, 1.0].map(f64::to_bits), true)],
        "the rim, about the floor's normal turned over to +z, and run its own way"
    );
    let (back, came) = reads_back_as(called, &text, (&model, built), &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(-1.0, -1.0, 3.5))
    });
    let [here, there] = [model.solid(built), back.model().solid(came)]
        .map(|solid| the_runs_its_closed_edges_name(solid.expect("its key")));
    assert_eq!(
        there, here,
        "{called}: which way each face walks the rim, and what its run names"
    );
}

/// A rod of radius two with a ball of its own radius on its end, whose rim is
/// one closed circle named by the plane `z = 4` it lies in, is written, and
/// reads back as the same rod with its rim in two halves: the solid
/// [`a_bar_with_a_dome_on_top`] builds, bit for bit.
///
/// Until Q777 the writer wrote an edge between a pipe and a ball only where it
/// stopped short of right round, and refused this one by name as
/// `{ first: "pipe", second: "ball", closed: true }`. **The reader cuts such a
/// circle in two at its far point**, `(−2, 0, 4)`, and names each half by the
/// plane the file's circle lies in (`Walk::describe_the_halves_of_a_circle`,
/// §189.3), so the corner it adds is the one way the solid read back differs
/// from the one built, and it measures as the rod built with that corner. The
/// written rim turns about the plane's own normal, `+z`, which is the way the
/// ball winds and the model stores the rim; and each half reads back naming
/// that plane bit for bit.
#[test]
fn a_rod_with_a_ball_on_its_end_and_one_closed_rim_comes_back_with_its_rim_in_two_halves() {
    let scale = Scale::new(Unit::Millimetre, 6.0).expect("a working size");
    let mut model = Model::new(scale);
    let built = a_rod_with_a_ball_on_its_end_whose_rim_is_one_closed_circle(&mut model);
    let text = rcad_step::write(&model, "a rod with a ball on its end")
        .unwrap_or_else(|why| panic!("the rod with one closed rim is not written: {why}"));
    assert_eq!(
        the_closed_circles_written(&text),
        [
            ([0.0, 0.0, 1.0].map(f64::to_bits), true),
            ([0.0, 0.0, 1.0].map(f64::to_bits), true)
        ],
        "the floor's circle and the rim, each about +z and run its own way"
    );
    let mut halves = Model::new(scale);
    let halved = a_bar_with_a_dome_on_top(&mut halves);
    let (back, came) = reads_back_as(
        "a rod with a ball on its end",
        &text,
        (&halves, halved),
        &|model| a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 1.0)),
    );
    let rims: Vec<[u64; 4]> = the_runs_its_closed_edges_name(model.solid(built).expect("its key"))
        .into_iter()
        .filter_map(|(_, and)| and)
        .collect();
    let [rim, _] = rims[..] else {
        panic!("the rod's rim is walked twice, naming its plane each time: {rims:?}");
    };
    assert_eq!(
        the_planes_its_halves_name(back.model().solid(came).expect("its key")),
        [rim, rim],
        "the plane each half of the rim names read back, as the bits of its normal and offset"
    );
}

/// A closed rim between a rod and a ball of its own radius on its end, named
/// by its two faces alone, is refused by `SolidBuilder`, so no model built
/// face by face carries one for the writer to name.
///
/// The ball touches the pipe round the rim and crosses it nowhere, so there is
/// no crossing to follow (§124), and the builder refuses it as it settles a
/// point onto both. A fuse refuses the same dome on a shaft before any edge is
/// drawn (`TwoCurvesTouchWithoutCrossing` in `fuses_that_touch.rs`). So the one
/// closed rim between a pipe and a ball a model holds names the plane it lies
/// in, and that one is written.
#[test]
fn a_rods_closed_rim_named_by_its_faces_alone_is_refused_by_the_solid_builder() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 6.0).expect("a working size"));
    let refused = a_rod_with_a_ball_on_its_end_whose_rim_runs_or_why(
        &mut model,
        EdgeRun::RightRoundWhereItsFacesCross,
    )
    .expect_err("a rim the faces only touch round names no crossing to follow");
    let rcad_topo::Error::ADraftEdgeHasACrossingThatCannotBeFollowed { edge: 2, why } = &refused
    else {
        panic!("refused at another step, as {refused:?}");
    };
    assert!(
        matches!(
            **why,
            rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent { .. }
        ),
        "the rim is refused as {why:?}"
    );
}

/// The axis of every `CIRCLE` the file runs an edge right round, as the bits
/// it was written with, and that edge's `same_sense`, in the file's order.
fn the_closed_circles_written(text: &str) -> Vec<([u64; 3], bool)> {
    let bodies: std::collections::HashMap<&str, &str> = text
        .lines()
        .filter_map(|line| line.strip_suffix(';')?.split_once('='))
        .collect();
    let inside = |id: &str, keyword: &str| -> Vec<&str> {
        bodies[id]
            .strip_prefix(keyword)
            .and_then(|rest| rest.strip_prefix("(''"))
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or_else(|| panic!("{id} is not a {keyword}"))
            .split(',')
            .skip(1)
            .collect()
    };
    let mut circles = Vec::new();
    for line in text.lines() {
        let Some((id, body)) = line.strip_suffix(';').and_then(|line| line.split_once('=')) else {
            continue;
        };
        if !body.starts_with("EDGE_CURVE(") {
            continue;
        }
        let edge = inside(id, "EDGE_CURVE");
        if edge[0] != edge[1] || !bodies[edge[2]].starts_with("CIRCLE(") {
            continue;
        }
        let place = inside(edge[2], "CIRCLE")[0];
        let axis = inside(place, "AXIS2_PLACEMENT_3D")[1];
        let way: Vec<u64> = bodies[axis]
            .trim_start_matches("DIRECTION('',(")
            .trim_end_matches("))")
            .split(',')
            .map(|number| number.parse::<f64>().expect("a number").to_bits())
            .collect();
        circles.push(([way[0], way[1], way[2]], edge[3] == ".T."));
    }
    circles
}

/// For each use of a closed edge of `solid`, in the order its faces and wires
/// walk them: whether it is walked against the way the edge is stored, and
/// the bits of the plane its run names, if it names one.
fn the_runs_its_closed_edges_name(
    solid: rcad_topo::SolidView<'_>,
) -> Vec<(bool, Option<[u64; 4]>)> {
    let mut uses = Vec::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if !edge.is_closed().expect("a run kind this test knows") {
                    continue;
                }
                let and = if let EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(cut),
                } = edge.run()
                {
                    Some(the_bits_of(cut))
                } else {
                    None
                };
                uses.push((coedge.orientation_on_its_edge().is_reversed(), and));
            }
        }
    }
    uses
}

/// The bits of the plane each open run of `solid` names whose two faces are a
/// pipe and a ball, in the order its faces, wires and coedges walk them.
fn the_planes_its_halves_name(solid: rcad_topo::SolidView<'_>) -> Vec<[u64; 4]> {
    let mut planes = Vec::new();
    for face in solid.faces() {
        if !matches!(face.surface(), Surface::Cylinder(_)) {
            continue;
        }
        for wire in face.wires() {
            for coedge in wire.coedges() {
                if let EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Plane(cut),
                    ..
                } = coedge.edge().run()
                {
                    planes.push(the_bits_of(cut));
                }
            }
        }
    }
    planes
}

fn the_bits_of(cut: Plane) -> [u64; 4] {
    let [x, y, z] = cut.normal().components().map(f64::to_bits);
    [x, y, z, cut.offset().to_bits()]
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
    let text = rcad_step::write(model, called)
        .unwrap_or_else(|why| panic!("{called} is not written: {why}"));
    reads_back_as(called, &text, (model, built), cutter);
    text
}

/// Read `text` back and require it to be `expected`: sound, the same corners
/// bit for bit, the same volume, area and centroid, and the same answer to a
/// cut by `cutter`, refusal and all. Hands back what was read, and the solid
/// it holds.
fn reads_back_as(
    called: &str,
    text: &str,
    (model, built): (&Model, SolidKey),
    cutter: &dyn Fn(&mut Model) -> SolidKey,
) -> (Import, SolidKey) {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
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
    (import, came)
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
    a_solid_drawn_face_by_face_or_why(model, corners, edges, faces, inside_out)
        .unwrap_or_else(|why| panic!("the solid drawn face by face is not sound: {why}"))
}

/// [`a_solid_drawn_face_by_face`], or why `SolidBuilder` refuses it.
fn a_solid_drawn_face_by_face_or_why(
    model: &mut Model,
    corners: &[Point],
    edges: Vec<(usize, usize, EdgeRun)>,
    faces: Vec<(Surface, Vec<Vec<Step>>)>,
    inside_out: &[usize],
) -> rcad_topo::Result<SolidKey> {
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
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
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

/// The bar of [`a_bar_with_a_dome_on_top`] with its rim drawn as one closed
/// circle: one corner on it at `(2, 0, 4)`, and a run that names the plane
/// `z = 4` the circle lies in, since the pipe and the ball only touch there.
fn a_rod_with_a_ball_on_its_end_whose_rim_is_one_closed_circle(model: &mut Model) -> SolidKey {
    let centre = Point::new(0.0, 0.0, 4.0);
    let rim = Surface::Plane(a_plane(centre, Vector::new(0.0, 0.0, 1.0)));
    a_rod_with_a_ball_on_its_end_whose_rim_runs_or_why(
        model,
        EdgeRun::RightRoundWhereItCrosses { and: rim },
    )
    .unwrap_or_else(|why| panic!("the solid drawn face by face is not sound: {why}"))
}

/// That rod with its rim running along `rim`, or why `SolidBuilder` refuses it.
fn a_rod_with_a_ball_on_its_end_whose_rim_runs_or_why(
    model: &mut Model,
    rim: EdgeRun,
) -> rcad_topo::Result<SolidKey> {
    use Step::{Forward, Pole, Reversed};
    use rcad_geom::{Axis, Cylinder, Sphere};

    let centre = Point::new(0.0, 0.0, 4.0);
    let up_the_dome = Surface::Plane(a_plane(Point::origin(), Vector::new(0.0, 1.0, 0.0)));
    a_solid_drawn_face_by_face_or_why(
        model,
        &[
            Point::new(2.0, 0.0, 0.0),
            Point::new(2.0, 0.0, 4.0),
            Point::new(0.0, 0.0, 6.0),
        ],
        vec![
            (0, 0, EdgeRun::RightRoundWhereItsFacesCross),
            (0, 1, EdgeRun::Straight),
            (1, 1, rim),
            (
                1,
                2,
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
                vec![vec![Forward(0), Forward(1), Reversed(2), Reversed(1)]],
            ),
            (
                Surface::Sphere(Sphere::new(centre, 2.0).expect("a positive radius")),
                vec![vec![Forward(2), Forward(3), Pole(2), Reversed(3)]],
            ),
        ],
        &[],
    )
}

/// The cap the plane `z = 3` cuts off the top of the ball of radius five about
/// the origin: the ball, with a seam from the rim at `(4, 0, 3)` up to the pole,
/// and the floor facing `−z`, whose rim is one closed circle of radius four.
fn a_ball_cut_by_a_flat_face_along_one_closed_circle(model: &mut Model) -> SolidKey {
    use Step::{Forward, Pole, Reversed};
    use rcad_geom::Sphere;

    let floor = a_plane(Point::new(0.0, 0.0, 3.0), Vector::new(0.0, 0.0, -1.0));
    let up_the_ball = Surface::Plane(a_plane(Point::origin(), Vector::new(0.0, 1.0, 0.0)));
    a_solid_drawn_face_by_face(
        model,
        &[Point::new(4.0, 0.0, 3.0), Point::new(0.0, 0.0, 5.0)],
        vec![
            (0, 0, EdgeRun::RightRoundWhereItsFacesCross),
            (
                0,
                1,
                part_of(up_the_ball, Point::origin(), Vector::new(1.0, 0.0, 0.0)),
            ),
        ],
        vec![
            (Surface::Plane(floor), vec![vec![Reversed(0)]]),
            (
                Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a positive radius")),
                vec![vec![Forward(0), Forward(1), Pole(1), Reversed(1)]],
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

/// The whole ball of radius four about the origin, one face, with its seam
/// from `ends[0]` to `ends[1]` on the side facing `+x`, cut from the plane
/// `y = 0` facing `facing`.
fn a_whole_ball_drawn_face_by_face(
    model: &mut Model,
    ends: [Point; 2],
    facing: Vector,
) -> SolidKey {
    use Step::{Forward, Pole, Reversed};
    use rcad_geom::Sphere;

    let cut = Surface::Plane(a_plane(Point::origin(), facing));
    a_solid_drawn_face_by_face(
        model,
        &ends,
        vec![(
            0,
            1,
            part_of(cut, Point::origin(), Vector::new(1.0, 0.0, 0.0)),
        )],
        vec![(
            Surface::Sphere(Sphere::new(Point::origin(), 4.0).expect("a positive radius")),
            vec![vec![Forward(0), Pole(1), Reversed(0), Pole(0)]],
        )],
        &[],
    )
}

/// The plane the one seam of `solid` is cut from, as the bits of its normal
/// and then of its offset.
fn the_plane_its_seam_is_cut_from(solid: rcad_topo::SolidView<'_>) -> [u64; 4] {
    let mut planes: Vec<[u64; 4]> = solid
        .faces()
        .flat_map(|face| face.wires().collect::<Vec<_>>())
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .map(|coedge| {
            let run = coedge.edge().run();
            let EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(cut),
                ..
            } = run
            else {
                panic!("a ball's seam is cut from a plane, not run {run:?}");
            };
            let [x, y, z] = cut.normal().components().map(f64::to_bits);
            [x, y, z, cut.offset().to_bits()]
        })
        .collect();
    planes.dedup();
    let [plane] = planes[..] else {
        panic!("a whole ball has one seam, not {planes:?}");
    };
    plane
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
    use rcad_geom::{Axis, Cylinder};

    let drawn = Surface::Cylinder(
        Cylinder::new(
            Axis::new(the_discs_centre(), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
            1.0,
        )
        .expect("a positive radius"),
    );
    a_bar_with_a_disc_drawn_on_its_top_by(model, drawn)
}

/// [`a_bar_with_a_disc_drawn_on_its_top`] with the two half circles named by
/// `drawn`, which has to cross the top in the circle of radius one about
/// [`the_discs_centre`].
fn a_bar_with_a_disc_drawn_on_its_top_by(model: &mut Model, drawn: Surface) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder};

    let centre = the_discs_centre();
    let top = Surface::Plane(a_plane(centre, Vector::new(0.0, 0.0, 1.0)));
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

/// Where the disc [`a_bar_with_a_disc_drawn_on_its_top_by`] draws is centred,
/// on the bar's top.
fn the_discs_centre() -> Point {
    Point::new(0.0, 0.0, 4.0)
}

/// A bar of radius two about `z`, four tall, whose seam at `x = 2` is not
/// [`EdgeRun::Straight`] but the piece in front of `x = 0` of where the plane
/// `y = 0` crosses the wall.
fn a_bar_whose_seam_is_named_by_the_plane_through_its_axis(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder};

    let through_the_axis = Surface::Plane(a_plane(Point::origin(), Vector::new(0.0, 1.0, 0.0)));
    a_solid_drawn_face_by_face(
        model,
        &[Point::new(2.0, 0.0, 0.0), Point::new(2.0, 0.0, 4.0)],
        vec![
            (0, 0, EdgeRun::RightRoundWhereItsFacesCross),
            (
                0,
                1,
                part_of(
                    through_the_axis,
                    Point::origin(),
                    Vector::new(1.0, 0.0, 0.0),
                ),
            ),
            (1, 1, EdgeRun::RightRoundWhereItsFacesCross),
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
            (
                Surface::Plane(a_plane(
                    Point::new(0.0, 0.0, 4.0),
                    Vector::new(0.0, 0.0, 1.0),
                )),
                vec![vec![Forward(2)]],
            ),
        ],
        &[],
    )
}

/// The half `y ≥ 0` of a bar of radius two about `z`, four tall, whose edge
/// across the floor from `(2, 0, 0)` to `(−2, 0, 0)` is not
/// [`EdgeRun::Straight`] but the piece nearer the floor of where a face crosses
/// the pipe of radius five along `x` through `(0, 3, 4)`. That pipe crosses
/// the floor in the lines `y = 0` and `y = 6`, and the flat side in `z = 0` and
/// `z = 8`.
fn half_a_bar_whose_edge_across_the_floor_is_named_by_a_pipe(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder};

    let floor = a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0));
    let top = a_plane(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0));
    let along_the_floor = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(0.0, 3.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("a direction"),
            5.0,
        )
        .expect("a positive radius"),
    );
    let in_front = Vector::new(0.0, 1.0, 0.0);
    a_solid_drawn_face_by_face(
        model,
        &[
            Point::new(2.0, 0.0, 0.0),
            Point::new(-2.0, 0.0, 0.0),
            Point::new(2.0, 0.0, 4.0),
            Point::new(-2.0, 0.0, 4.0),
        ],
        vec![
            (
                0,
                1,
                part_of(
                    along_the_floor,
                    Point::new(0.0, 1.5, 2.0),
                    Vector::new(0.0, -4.0, -3.0),
                ),
            ),
            (
                1,
                0,
                part_of(Surface::Plane(floor), Point::origin(), in_front),
            ),
            (2, 3, EdgeRun::Straight),
            (
                3,
                2,
                part_of(Surface::Plane(top), Point::new(0.0, 0.0, 4.0), in_front),
            ),
            (0, 2, EdgeRun::Straight),
            (1, 3, EdgeRun::Straight),
        ],
        vec![
            (Surface::Plane(floor), vec![vec![Forward(0), Forward(1)]]),
            (Surface::Plane(top), vec![vec![Reversed(3), Reversed(2)]]),
            (
                Surface::Plane(a_plane(Point::origin(), Vector::new(0.0, -1.0, 0.0))),
                vec![vec![Forward(4), Forward(2), Reversed(5), Reversed(0)]],
            ),
            (
                Surface::Cylinder(Cylinder::new(Axis::z(), 2.0).expect("a positive radius")),
                vec![vec![Reversed(1), Forward(5), Forward(3), Reversed(4)]],
            ),
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

/// The corner of a slab two thick whose edge is rounded all the way from its
/// top face `z = 1` to its bottom face `z = −1`: a quarter of the ring of
/// radius four about `z` with a tube of radius one, and a pipe of that tube
/// carrying on straight from each end of it, along `y` at `x = 4` to the end
/// `y = −2` and along `x` at `y = 4` to the end `x = −2`.
///
/// So the top and bottom faces rest on the ring along quarters of the circles
/// of radius four at `z = ±1`, and the ring meets each pipe in half of its
/// tube's circle.
fn the_corner_of_a_slab_with_a_round_edge(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder, Torus};

    let x = Vector::new(1.0, 0.0, 0.0);
    let y = Vector::new(0.0, 1.0, 0.0);
    let z = Vector::new(0.0, 0.0, 1.0);
    let flat = |at: Point, facing: Vector| Surface::Plane(a_plane(at, facing));
    let pipe = |at: Point, along: Vector| {
        Surface::Cylinder(
            Cylinder::new(Axis::new(at, along).expect("a direction"), 1.0)
                .expect("a positive radius"),
        )
    };
    let top = flat(Point::new(0.0, 0.0, 1.0), z);
    let bottom = flat(Point::new(0.0, 0.0, -1.0), -z);
    let bottom_turned_round = flat(Point::new(0.0, 0.0, -1.0), z);
    let first_end = flat(Point::new(0.0, -2.0, 0.0), -y);
    let second_end = flat(Point::new(-2.0, 0.0, 0.0), -x);
    let corner = Point::new(4.0, 0.0, 0.0);
    a_solid_drawn_face_by_face(
        model,
        &[
            Point::new(4.0, -2.0, 1.0),
            Point::new(4.0, 0.0, 1.0),
            Point::new(0.0, 4.0, 1.0),
            Point::new(-2.0, 4.0, 1.0),
            Point::new(-2.0, -2.0, 1.0),
            Point::new(4.0, -2.0, -1.0),
            Point::new(4.0, 0.0, -1.0),
            Point::new(0.0, 4.0, -1.0),
            Point::new(-2.0, 4.0, -1.0),
            Point::new(-2.0, -2.0, -1.0),
        ],
        vec![
            (0, 1, EdgeRun::Straight),
            (1, 2, part_of(top, Point::new(4.0, 0.0, 1.0), x + y)),
            (2, 3, EdgeRun::Straight),
            (3, 4, EdgeRun::Straight),
            (4, 0, EdgeRun::Straight),
            (5, 6, EdgeRun::Straight),
            (
                6,
                7,
                part_of(bottom_turned_round, Point::new(4.0, 0.0, -1.0), x + y),
            ),
            (7, 8, EdgeRun::Straight),
            (8, 9, EdgeRun::Straight),
            (9, 5, EdgeRun::Straight),
            (1, 6, part_of(flat(Point::origin(), y), corner, x)),
            (
                2,
                7,
                part_of(flat(Point::origin(), x), Point::new(0.0, 4.0, 0.0), y),
            ),
            (5, 0, part_of(first_end, Point::new(4.0, -2.0, 0.0), x)),
            (8, 3, part_of(second_end, Point::new(-2.0, 4.0, 0.0), y)),
            (4, 9, EdgeRun::Straight),
        ],
        vec![
            (
                top,
                vec![vec![
                    Forward(0),
                    Forward(1),
                    Forward(2),
                    Forward(3),
                    Forward(4),
                ]],
            ),
            (
                bottom,
                vec![vec![
                    Reversed(9),
                    Reversed(8),
                    Reversed(7),
                    Reversed(6),
                    Reversed(5),
                ]],
            ),
            (
                Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a working ring")),
                vec![vec![Forward(6), Reversed(11), Reversed(1), Forward(10)]],
            ),
            (
                pipe(corner, y),
                vec![vec![Forward(5), Reversed(10), Reversed(0), Reversed(12)]],
            ),
            (
                pipe(Point::new(0.0, 4.0, 0.0), x),
                vec![vec![Forward(7), Forward(13), Reversed(2), Forward(11)]],
            ),
            (
                first_end,
                vec![vec![Forward(9), Forward(12), Reversed(4), Forward(14)]],
            ),
            (
                second_end,
                vec![vec![Forward(8), Reversed(14), Reversed(3), Reversed(13)]],
            ),
        ],
        &[],
    )
}

/// The part above `z = 3` of a rod of radius five bent round a quarter of the
/// ring of radius ten about `z`: a pipe along `y` at `x = 10` from its end at
/// `y = −4` up to `y = 0`, the quarter of the ring from there round to `x = 0`,
/// and a pipe along `x` at `y = 10` on to its end at `x = −4`.
///
/// So the flat face `z = 3` meets the ring along a quarter of the circle of
/// radius fourteen, from `(14, 0, 3)` to `(0, 14, 3)`, and a quarter of the
/// circle of radius six, from `(0, 6, 3)` to `(6, 0, 3)`, and meets each pipe
/// along two straight lines. The ring meets each pipe in the part of its
/// tube's circle above the cut, and each end cuts its pipe in the same.
fn the_top_of_a_bent_rod_sawn_off_flat(model: &mut Model) -> SolidKey {
    use Step::{Forward, Reversed};
    use rcad_geom::{Axis, Cylinder, Torus};

    let x = Vector::new(1.0, 0.0, 0.0);
    let y = Vector::new(0.0, 1.0, 0.0);
    let z = Vector::new(0.0, 0.0, 1.0);
    let flat = |at: Point, facing: Vector| Surface::Plane(a_plane(at, facing));
    let pipe = |at: Point, along: Vector| {
        Surface::Cylinder(
            Cylinder::new(Axis::new(at, along).expect("a direction"), 5.0)
                .expect("a positive radius"),
        )
    };
    let cut = flat(Point::new(0.0, 0.0, 3.0), -z);
    let first_end = flat(Point::new(0.0, -4.0, 0.0), -y);
    let second_end = flat(Point::new(-4.0, 0.0, 0.0), -x);
    a_solid_drawn_face_by_face(
        model,
        &[
            Point::new(14.0, -4.0, 3.0),
            Point::new(14.0, 0.0, 3.0),
            Point::new(0.0, 14.0, 3.0),
            Point::new(-4.0, 14.0, 3.0),
            Point::new(-4.0, 6.0, 3.0),
            Point::new(0.0, 6.0, 3.0),
            Point::new(6.0, 0.0, 3.0),
            Point::new(6.0, -4.0, 3.0),
        ],
        vec![
            (0, 1, EdgeRun::Straight),
            (1, 2, part_of(cut, Point::new(14.0, 0.0, 3.0), x + y)),
            (2, 3, EdgeRun::Straight),
            (3, 4, EdgeRun::Straight),
            (4, 5, EdgeRun::Straight),
            (5, 6, part_of(cut, Point::new(6.0, 0.0, 3.0), x + y)),
            (6, 7, EdgeRun::Straight),
            (7, 0, EdgeRun::Straight),
            (
                1,
                6,
                part_of(flat(Point::origin(), y), Point::new(10.0, 0.0, 3.0), z),
            ),
            (
                2,
                5,
                part_of(flat(Point::origin(), x), Point::new(0.0, 10.0, 3.0), z),
            ),
            (7, 0, part_of(first_end, Point::new(10.0, -4.0, 3.0), z)),
            (3, 4, part_of(second_end, Point::new(-4.0, 10.0, 3.0), z)),
        ],
        vec![
            (
                cut,
                vec![vec![
                    Reversed(0),
                    Reversed(7),
                    Reversed(6),
                    Reversed(5),
                    Reversed(4),
                    Reversed(3),
                    Reversed(2),
                    Reversed(1),
                ]],
            ),
            (
                Surface::Torus(Torus::new(Axis::z(), 10.0, 5.0).expect("a working ring")),
                vec![vec![Forward(1), Forward(9), Forward(5), Reversed(8)]],
            ),
            (
                pipe(Point::new(10.0, 0.0, 0.0), y),
                vec![vec![Forward(0), Forward(8), Forward(6), Forward(10)]],
            ),
            (
                pipe(Point::new(0.0, 10.0, 0.0), x),
                vec![vec![Forward(2), Forward(11), Forward(4), Reversed(9)]],
            ),
            (first_end, vec![vec![Forward(7), Reversed(10)]]),
            (second_end, vec![vec![Forward(3), Reversed(11)]]),
        ],
        &[],
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
