//! A round bar whose wall the reader finds its corners off, and which the
//! kernel's own checks now refuse too.
//!
//! Q169's crash file, `crash-4fa1eb2d42f7b22c3ff6455d6ae0de9bb736a93e`, holds
//! a round bar 15 tall whose side sits on a pipe of radius `2 + 91·2⁻⁵¹`, while
//! its two corners sit at radius 2: `91·2⁻⁵¹ ≈ 4.04e-14` inside the wall. A
//! corner of a curved face gets `16·ε·L` of room, where `L` is the largest
//! coordinate of any corner the check looks at. The fuzzer moved the bar by
//! `(0, −1, −10)`, the writer writes it there, and the reader reads `L` as 10
//! and gives `3.55e-14`, so it refuses the corner.
//!
//! `Model::validate` and `Model::validate_solid` used to read `L` where the
//! corners are stored, 15 here, which gives `5.33e-14`. They passed the bar and
//! the writer's fuzz harness panicked on it (`docs/decisions.md` §327). They
//! now read `L` where the corners are stored or where the placement puts them,
//! whichever is smaller (§363). So they refuse the moved bar as the reader
//! does, and the harness counts it rather than panicking. The unmoved bar is
//! the control, and it comes back the same.

#![forbid(unsafe_code)]

mod fuzzing;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{DraftStep, Invalid, Model, SolidBuilder};

use fuzzing::writer::one_model_decoded_and_written;

/// How far the wall is from the corners: 91 steps of `f64` above 2.
const THE_MISS: f64 = 91.0 * f64::EPSILON * 2.0;

/// Unmoved, the bar's largest coordinate is its top corner's 15 wherever it is
/// read, the reader gives it the room `validate` does, and it comes back the
/// same.
#[test]
fn the_bar_where_it_was_built_comes_back_the_same() {
    let bytes = postcard::to_allocvec(&a_bar_whose_wall_is_a_hair_outside_its_corners(
        &Isometry::identity(),
    ))
    .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// Moved as the fuzzer moved it, both of the kernel's checks refuse the bar as
/// a corner off its pipe, and the reader refuses its bottom corner, `#15` at
/// `(2, −1, −10)`, for the whole of the miss.
#[test]
fn the_bar_moved_as_the_fuzzer_moved_it_is_refused_by_validate_as_by_the_reader() {
    let model = a_bar_whose_wall_is_a_hair_outside_its_corners(&the_fuzzers_move());
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let whole = model.validate(&policy, &mut log);
    assert!(
        matches!(
            whole,
            Err(rcad_topo::Error::Invalid(Invalid::VertexOffItsFaceSurface))
        ),
        "where it sits the bar's largest coordinate is 10: {whole:?}"
    );
    let bar = model.solids().next().expect("one solid").key();
    let alone = model.validate_solid(bar, &policy, &mut log);
    assert!(
        matches!(
            alone,
            Err(rcad_topo::Error::Invalid(Invalid::VertexOffItsFaceSurface))
        ),
        "alone, the bar has the room of 10 too: {alone:?}"
    );

    let refusal = what_the_reader_says_of(&model);
    let Err(Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn { off_by, .. }) = refusal else {
        panic!("the reader refuses a corner off the pipe, not {refusal:?}");
    };
    assert_eq!(off_by, THE_MISS);
}

/// The fuzz harness asks `validate`, which refuses the model, so the file owes
/// the harness nothing and it stops there without asking the reader.
#[test]
fn the_writer_harness_counts_the_bar_moved_as_the_fuzzer_moved_it_rather_than_panicking() {
    let bytes = postcard::to_allocvec(&a_bar_whose_wall_is_a_hair_outside_its_corners(
        &the_fuzzers_move(),
    ))
    .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.written, reached.written_but_unsound, reached.read_back),
        (1, 1, 0),
        "{reached:?}"
    );
}

/// The reader's room is `16·ε` times the largest coordinate where the bar
/// sits, so the miss of `91·2⁻⁵¹` fits once that coordinate reaches
/// `91·2⁻⁵¹ / (16·2⁻⁵²) = 11.375`. Moved down by 3.5 its top corner is at
/// 11.5, and `validate` and the reader both take it. Moved down by 4 its top
/// corner is at 11, and both refuse it. Nothing about the bar differs between
/// the two but where it sits, and the two checks now draw the line in the same
/// place.
#[test]
fn validate_and_the_reader_both_take_the_bar_once_its_top_corner_is_past_11_375() {
    let at = |down: f64| {
        let moved = Isometry::translation(Vector::new(0.0, 0.0, -down)).expect("a finite offset");
        let model = a_bar_whose_wall_is_a_hair_outside_its_corners(&moved);
        let kernel = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
        (kernel, what_the_reader_says_of(&model))
    };
    let (kernel, reader) = at(3.5);
    assert!(kernel.is_ok() && reader.is_ok(), "{kernel:?} {reader:?}");
    let (kernel, reader) = at(4.0);
    assert!(
        matches!(
            kernel,
            Err(rcad_topo::Error::Invalid(Invalid::VertexOffItsFaceSurface))
        ),
        "{kernel:?}"
    );
    assert!(
        matches!(
            reader,
            Err(Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn { .. })
        ),
        "{reader:?}"
    );
}

fn the_fuzzers_move() -> Isometry {
    Isometry::translation(Vector::new(0.0, -1.0, -10.0)).expect("a finite offset")
}

fn what_the_reader_says_of(model: &Model) -> Result<(), Refusal> {
    let text = rcad_step::write(model, "a bar a hair narrow").expect("the writer writes a bar");
    let file = StepFile::read(text.as_bytes()).expect("the writer's own file parses");
    Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()).map(|_| ())
}

/// Solid 0 of the crash file: a round bar 15 tall with both corners at radius
/// 2 on the `x` axis, whose side is a pipe round `z` of radius `2 + THE_MISS`,
/// then moved by `placement`.
fn a_bar_whose_wall_is_a_hair_outside_its_corners(placement: &Isometry) -> Model {
    let height = 15.0;
    let radius = 2.0;
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut builder = SolidBuilder::new();
    let bottom_corner = builder
        .add_vertex(Point::new(radius, 0.0, 0.0))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(radius, 0.0, height))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cylinder is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cylinder in one closed curve");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cylinder in one closed curve");

    let mut cap = |z: f64, outward: f64, circle, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder
            .add_coedge(circle, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap is a plane trimmed by its circle")
    };
    let bottom = cap(0.0, -1.0, bottom_circle, Orientation::Reversed);
    let top = cap(height, 1.0, top_circle, Orientation::Forward);

    let round_the_wall: Vec<DraftStep> = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_circle, Orientation::Reversed),
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
    let side = builder
        .add_face(
            Surface::Cylinder(
                Cylinder::new(Axis::z(), radius + THE_MISS).expect("a positive radius"),
            ),
            &[wire],
        )
        .expect("a face on the cylinder");
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    let bar = builder
        .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the miss is inside the room the top corner's 15 gives");
    model
        .place_solid(bar, placement)
        .expect("the key came from this model");
    model
}
