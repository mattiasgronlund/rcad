//! A model that says it is two hundred light-years across, and the room that
//! used to buy it.
//!
//! [`Model::allowed_wobble`] is how far off its own plane a face's corner may
//! sit, and until §70 the modelling half of it was
//! [`Scale::default_linear_tolerance`] and nothing else — the model's *own
//! declared* nominal size times [`Scale::RELATIVE_LINEAR_TOLERANCE`]. A `Scale`
//! arrives with the file, so that let the file decide how closely the file
//! would be checked: a ten-millimetre block declaring itself `1.8e21` mm across
//! bought `1.8e12` mm of room and could then be broken by any amount short of
//! that without the sweep noticing.
//!
//! That is not a hypothetical. `docs/decisions.md` §67's corpus entry is that
//! model — it declares `1.8446744073709552e21` and its largest coordinate is
//! `10.359375` — and its walls are twenty and ten millimetres off their own
//! planes. It passed `Model::validate` and crashed the fuse.
//!
//! The blocks here are built through [`SolidBuilder`] rather than carried as
//! bytes, because the hole is in the tolerance and not in any one corpus entry:
//! the same declaration breaks the check on any shape.

use rcad_geom::{EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, SolidBuilder, SolidKey};

/// Twenty millimetres, which is twice the block it is lifted off.
const A_WOBBLE_NOBODY_COULD_CALL_ROUNDING: f64 = 20.0;

#[test]
fn a_block_that_says_it_is_two_hundred_light_years_across_does_not_buy_room_to_be_broken() {
    let mut model = Model::new(
        Scale::millimetres(1.844_674_407_370_955_2e21).expect("a positive, finite size"),
    );

    let refused = a_ten_millimetre_block_with_one_corner_lifted(
        &mut model,
        A_WOBBLE_NOBODY_COULD_CALL_ROUNDING,
    )
    .expect_err("a corner twenty out of a ten-millimetre block is not rounding");

    assert!(
        matches!(
            refused,
            Error::Invalid(Invalid::VertexOffItsFacePlane { distance })
                if (distance - A_WOBBLE_NOBODY_COULD_CALL_ROUNDING).abs() < 1e-9
        ),
        "the corner is twenty off its own cap and that is what should be said: {refused:?}"
    );
}

/// The same block, whose declaration matches what it is, is untouched.
///
/// This is the half that says the cap costs a sound model nothing: ten
/// millimetres declared as ten millimetres allows `1e-8` mm, and a corner
/// half of that off its cap goes through exactly as it did before §70.
#[test]
fn a_block_that_says_what_it_is_keeps_the_room_its_own_size_earns() {
    let mut model = Model::new(Scale::millimetres(10.0).expect("ten millimetres is a size"));
    let allowed = Scale::RELATIVE_LINEAR_TOLERANCE * 10.0;

    a_ten_millimetre_block_with_one_corner_lifted(&mut model, allowed / 2.0)
        .expect("half the tolerance a ten-millimetre part earns is still sound");
}

/// A model whose corners really are that big keeps the room they earn.
///
/// The cap is the smaller of what the model says and what its corners show, so
/// it binds only on a model that overstates itself. Here the two agree, the
/// tolerance is `1e-3` mm, and a corner a tenth of that off its cap passes —
/// which is a wobble the ten-millimetre block above would have been refused
/// for a hundred thousand times over.
#[test]
fn a_block_a_million_millimetres_across_still_gets_a_millions_worth_of_room() {
    let mut model = Model::new(Scale::millimetres(1.0e6).expect("a million millimetres is a size"));
    let across = 1.0e6;
    let allowed = Scale::RELATIVE_LINEAR_TOLERANCE * across;

    a_block_with_one_corner_lifted(&mut model, across, allowed / 10.0)
        .expect("a tenth of the tolerance a million-millimetre part earns is sound");
}

fn a_ten_millimetre_block_with_one_corner_lifted(
    model: &mut Model,
    lift: f64,
) -> rcad_topo::Result<SolidKey> {
    a_block_with_one_corner_lifted(model, 10.0, lift)
}

/// A box `across` on a side at the origin, with the corner furthest from the
/// origin lifted `lift` straight up.
///
/// Straight up, because the four walls stand on planes of constant `x` or `y`:
/// moving that corner along `z` leaves it exactly on both walls that meet
/// there, so the cap is the only face with anything to say about it.
fn a_block_with_one_corner_lifted(
    model: &mut Model,
    across: f64,
    lift: f64,
) -> rcad_topo::Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let corners = [
        Point::new(0.0, 0.0, 0.0),
        Point::new(across, 0.0, 0.0),
        Point::new(across, across, 0.0),
        Point::new(0.0, across, 0.0),
        Point::new(0.0, 0.0, across),
        Point::new(across, 0.0, across),
        Point::new(across, across, across + lift),
        Point::new(0.0, across, across),
    ];
    let corners: Vec<_> = corners
        .into_iter()
        .map(|point| builder.add_vertex(point).expect("a finite point"))
        .collect();

    // Bottom loop, top loop, then the four uprights, in the order the faces
    // below name them.
    let sides = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    let sides: Vec<_> = sides
        .into_iter()
        .map(|[from, to]| {
            builder
                .add_edge(corners[from], corners[to], EdgeRun::Straight)
                .expect("two different corners make a straight side")
        })
        .collect();

    let faces = [
        (
            Point::new(0.0, 0.0, 0.0),
            Vector::new(0.0, 0.0, -1.0),
            [
                (3, Orientation::Reversed),
                (2, Orientation::Reversed),
                (1, Orientation::Reversed),
                (0, Orientation::Reversed),
            ],
        ),
        (
            Point::new(0.0, 0.0, across),
            Vector::new(0.0, 0.0, 1.0),
            [
                (4, Orientation::Forward),
                (5, Orientation::Forward),
                (6, Orientation::Forward),
                (7, Orientation::Forward),
            ],
        ),
        (
            Point::new(0.0, 0.0, 0.0),
            Vector::new(0.0, -1.0, 0.0),
            [
                (0, Orientation::Forward),
                (9, Orientation::Forward),
                (4, Orientation::Reversed),
                (8, Orientation::Reversed),
            ],
        ),
        (
            Point::new(across, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
            [
                (1, Orientation::Forward),
                (10, Orientation::Forward),
                (5, Orientation::Reversed),
                (9, Orientation::Reversed),
            ],
        ),
        (
            Point::new(0.0, across, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            [
                (2, Orientation::Forward),
                (11, Orientation::Forward),
                (6, Orientation::Reversed),
                (10, Orientation::Reversed),
            ],
        ),
        (
            Point::new(0.0, 0.0, 0.0),
            Vector::new(-1.0, 0.0, 0.0),
            [
                (3, Orientation::Forward),
                (8, Orientation::Forward),
                (7, Orientation::Reversed),
                (11, Orientation::Reversed),
            ],
        ),
    ];
    let skin: Vec<_> = faces
        .into_iter()
        .map(|(through, normal, walk)| {
            let steps: Vec<_> = walk
                .into_iter()
                .map(|(side, orientation)| {
                    DraftStep::Along(
                        builder
                            .add_coedge(sides[side], orientation)
                            .expect("a side of this build"),
                    )
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("four sides close");
            let plane = Plane::from_point_and_normal(through, normal).expect("a unit normal");
            let face = builder
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a wall or a cap");
            (face, Orientation::Forward)
        })
        .collect();
    builder.add_shell(&skin).expect("six faces close a box");

    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
