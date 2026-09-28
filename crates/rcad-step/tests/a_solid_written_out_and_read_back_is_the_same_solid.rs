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

use rcad_geom::{Deflection, Isometry, Plane, Point, Scale, Unit, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{CannotWrite, Import, StepFile};
use rcad_topo::{Measurement, Model, ProfileBuilder, Rule, SolidKey};

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

/// A ball is refused, by the name of the thing an `EDGE_LOOP` cannot say.
///
/// **A refusal recorded as a test rather than as a sentence.** A ball's one
/// wire is four steps and two of them stand still on a pole, which STEP writes
/// as a `VERTEX_LOOP` and
/// [`Refusal::ALoopThisReaderDoesNotWalk`](rcad_step::Refusal) will not read
/// back. Writing a file this kernel cannot read is not a round trip, so the
/// writer refuses first. The ball's seam is refused for a second, independent
/// reason — see the next test.
#[test]
fn a_ball_is_refused_because_an_edge_loop_cannot_stand_still_at_a_pole() {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 4.0).expect("a working size"));
    model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");

    let refused = rcad_step::write(&model, "a ball");
    assert!(
        matches!(refused, Err(CannotWrite::AWireThatStandsStillAtAPole)),
        "expected the pole to stop it, got {refused:?}"
    );
}

/// A bar cut off by a slanted plane meets it in an **ellipse**, and two
/// surfaces that cross in an ellipse are two surfaces this writer will not
/// guess a circle from.
///
/// This is the demonstration [`CannotWrite::ACurveTheFacesEitherSideOfItDoNotName`]
/// needs (§16.10): the refusal is reachable from a shape this kernel builds,
/// and it fires on a pair the writer *can* name when the plane is square across
/// the pipe — the round bar above is that same pair, accepted. So the check is
/// asking something.
///
/// The gap it stands for is `EdgeView::run` being `pub(crate)`: the model knows
/// exactly what this edge runs along, and a located run wants an
/// `EdgeRun::transformed`, which wants a `HalfSpace::transformed` in
/// `rcad-geom`. §42.5's lane reported that missing function for a placement
/// argument and §42.10's for a turned arc; this is its third consumer.
#[test]
fn an_ellipse_where_a_slanted_cap_meets_a_bar_is_a_curve_the_writer_cannot_name() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::new(Unit::Millimetre, 8.0).expect("a working size"));
    let flat = Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let slanted =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 8.0), Vector::new(0.0, 1.0, 2.0))
            .expect("a normal with a length");
    model
        .add_bar_cut_off_by(
            2.0,
            [flat, slanted],
            &Isometry::identity(),
            &policy,
            &mut log,
        )
        .expect("a bar with one slanted cap");

    let refused = rcad_step::write(&model, "a bar with a slanted cap");
    assert!(
        matches!(
            refused,
            Err(CannotWrite::ACurveTheFacesEitherSideOfItDoNotName { closed: true, .. })
        ),
        "expected the ellipse to stop it, got {refused:?}"
    );
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
