//! Whether each refusal of this crate is a limit, a gap or not known.
//!
//! `docs/decisions.md` §437.6. A **limit** says no answer this kernel can hold
//! exists; a **gap** says one exists and the arithmetic did not reach it; and
//! **not known** says nobody has shown either. Every variant that claims a
//! limit or a gap is reached here, through the public API, and asked which it
//! is. Not known claims nothing, so it needs no test, except where a variant
//! was once said to be a limit and is shown here to be both kinds.
//!
//! A gap is asserted only on a shape refused **every way round**: a pair that
//! refuses one way and answers the other proves nothing about the refusal
//! (§140.5).

#![forbid(unsafe_code)]

use rcad_geom::{
    Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftFace, DraftStep, DraftVertex, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder,
    SolidKey,
};

/// Profiles and sizes that describe no solid are limits: there is nothing for
/// any arithmetic to reach.
#[test]
fn profiles_and_sizes_that_describe_no_solid_are_limits() {
    let mut model = a_model();
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit square");
    let profile = |points: Vec<[f64; 2]>| {
        ProfileBuilder::new(points)
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect_err("a profile that encloses nothing")
    };
    let refusals = [
        profile(vec![[0.0, 0.0], [1.0, 0.0]]),
        profile(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 0.0], [1.0, 1.0]]),
        profile(vec![[0.0, 0.0], [f64::NAN, 0.0], [1.0, 1.0]]),
        model
            .add_extrusion(&square, 0.0, &Isometry::identity())
            .expect_err("an extrusion of no height"),
        model
            .add_cube(-1.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect_err("a cube of negative side"),
    ];
    assert_eq!(
        refusals.each_ref().map(name_of),
        [
            "TooFewPoints",
            "RepeatedPoint",
            "PointNotFinite",
            "HeightNotPositive",
            "HeightNotPositive",
        ]
    );
    for refusal in &refusals {
        assert_eq!(refusal.limit_or_gap().to_string(), "limit", "{refusal:?}");
    }
}

/// A cap whose normal points into the bar asks for everything outside two
/// planes, which is no solid.
#[test]
fn a_bar_whose_cap_faces_into_it_is_a_limit() {
    for (caps, which) in [
        (
            [
                a_cap_square_to_the_axis(0.0, 1.0),
                a_cap_square_to_the_axis(2.0, 1.0),
            ],
            0,
        ),
        (
            [
                a_cap_square_to_the_axis(0.0, -1.0),
                a_cap_square_to_the_axis(2.0, -1.0),
            ],
            1,
        ),
    ] {
        let refusal = a_model()
            .add_bar_cut_off_by(
                1.0,
                caps,
                &Isometry::identity(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a cap facing into the bar");
        assert!(
            matches!(refusal, Error::ACapOfABarFacesIntoIt { which: refused } if refused == which),
            "{refusal:?}"
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "limit");
    }
}

/// A key the model never made, and a face the solid does not use, name nothing
/// there is an answer about.
#[test]
fn a_key_that_names_nothing_here_is_a_limit() {
    let mut mine = a_model();
    let small = a_cube(&mut mine, 1.0);
    let mut theirs = a_model();
    a_cube(&mut theirs, 10.0);
    let Err(stranger) = theirs.solid(small) else {
        panic!("a key from another model reached a solid");
    };
    assert!(
        matches!(stranger, Error::KeyFromAnotherModel { entity: "solid" }),
        "{stranger:?}"
    );

    let other = a_cube(&mut mine, 2.0);
    let somebody_elses = mine
        .solid(other)
        .expect("this model made the key")
        .faces()
        .next()
        .expect("a cube has faces")
        .key();
    let not_its_own = mine
        .solid(small)
        .expect("this model made the key")
        .measure_one_face(
            somebody_elses,
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a face of the other cube");
    assert!(
        matches!(not_its_own, Error::NoSuchFaceInThisSolid { face } if face == somebody_elses),
        "{not_its_own:?}"
    );

    for refusal in [stranger, not_its_own] {
        assert_eq!(refusal.limit_or_gap().to_string(), "limit", "{refusal:?}");
    }
}

/// Too much skin to measure is both kinds, so it says not known.
///
/// An area past the largest `f64` has no `f64` to be handed back in: a face of
/// a cube of side `1e200` is `1e400`, and a slab `1e154` wide and one thick
/// holds `1e308` of space inside `2e308` of skin. Those are limits. A
/// parallelogram whose area is under `f64::MAX`, worked out exactly here from
/// its corners, is refused too, top and bottom alike, because the rounded sum
/// of its fan comes out past it: a gap.
#[test]
fn an_area_too_large_to_measure_is_not_known_because_a_face_under_f64_max_is_refused_too() {
    let mut model = a_model();
    let huge = a_cube(&mut model, 1e200);
    let solid = model.solid(huge).expect("this model made the key");
    let face = solid.faces().next().expect("a cube has faces").key();
    let one_face = solid
        .measure_one_face(
            face,
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a face of 1e400");

    let mut wide = Model::new(Scale::millimetres(1e154).expect("a working size"));
    let slab = block(&mut wide, [1e154, 1e154, 1.0]);
    let whole = wide
        .solid(slab)
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("2e308 of skin");

    let mut refusals = vec![one_face, whole];
    refusals.extend(the_faces_of_a_parallelogram_just_under_f64_max_refused());
    for refusal in &refusals {
        assert!(
            matches!(refusal, Error::SurfaceAreaTooLargeToMeasure),
            "{refusal:?}"
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "not known");
    }
}

/// What measuring the top and the bottom of a thin prism gives, when the
/// prism's cross-section is a parallelogram with an area just under
/// `f64::MAX`.
///
/// The corners are whole numbers times `2^460`, so the area is
/// `(p·s − q·r)·2^920` exactly, and `f64::MAX` is `(2^104 − 2^51)·2^920`. The
/// numbers were found by a search over such parallelograms, which met 30 of
/// 3200 faces refused like this.
fn the_faces_of_a_parallelogram_just_under_f64_max_refused() -> Vec<Error> {
    let [p, q, r, s]: [i64; 4] = [
        4_755_221_860_500_385,
        4_426_005_373_008_903,
        -2_708_312_355_644_458,
        1_744_483_182_727_879,
    ];
    let area_over_two_to_the_920 = i128::from(p) * i128::from(s) - i128::from(q) * i128::from(r);
    assert!(
        area_over_two_to_the_920 < (1i128 << 104) - (1i128 << 51),
        "the area is under f64::MAX"
    );
    let corners = [[0, 0], [p, q], [p + r, q + s], [r, s]];
    let unit = two_to_the(460);
    let profile =
        ProfileBuilder::new(corners.map(|corner| corner.map(|whole| exactly(whole) * unit)))
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a parallelogram");
    let mut model = Model::new(Scale::millimetres(two_to_the(513)).expect("a working size"));
    let prism = model
        .add_extrusion(&profile, 0.5, &Isometry::identity())
        .expect("building a prism needs no decision");
    let solid = model.solid(prism).expect("this model made the key");
    let ends: Vec<_> = solid
        .faces()
        .filter(|face| {
            face.surface()
                .as_plane()
                .is_some_and(|plane| plane.normal().components()[2] != 0.0)
        })
        .map(|face| face.key())
        .collect();
    assert_eq!(ends.len(), 2, "a prism has a top and a bottom");
    ends.into_iter()
        .map(|end| {
            solid
                .measure_one_face(
                    end,
                    a_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect_err("an area just under f64::MAX")
        })
        .collect()
}

/// `whole` as an `f64`, which must hold it with nothing rounded.
fn exactly(whole: i64) -> f64 {
    let held = whole as f64;
    assert_eq!(held as i64, whole, "{whole} is not an f64");
    held
}

/// Too long to measure is both kinds, so it says not known.
///
/// A cube of side `1e308` has faces `4e308` round, past `f64::MAX`: a limit. A
/// cube of side `f64::MAX / 4` has faces exactly `f64::MAX` round, an `f64` with
/// nothing rounded in it, and the sum of its four sides, rounded outwards, still
/// runs past `f64::MAX`: a gap.
#[test]
fn a_wire_too_long_to_measure_is_not_known_because_one_exactly_f64_max_long_is_refused_too() {
    let quarter = f64::MAX / 4.0;
    assert_eq!(4.0 * quarter, f64::MAX, "the gap's answer is an f64");
    for side in [1e308, quarter] {
        let mut model = Model::new(Scale::millimetres(1.0).expect("a scale"));
        let cube = a_cube(&mut model, side);
        let solid = model.solid(cube).expect("this model made the key");
        let face = solid.faces().next().expect("a cube has faces");
        let refusal = face
            .outer_wire()
            .length()
            .expect_err("four sides past what the sum keeps room for");
        assert!(
            matches!(refusal, Error::LengthTooLargeToMeasure),
            "a side of {side:e}: {refusal:?}"
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "not known");
    }
}

/// Too big to measure is both kinds, so it says not known.
///
/// Measuring a cube of side `1e103` asks for a volume of `1e309`, which no
/// `f64` holds: a limit. Uniting a cube of side `2^342` with a copy of itself
/// asks for that same cube, which the model already holds, and is refused
/// either way round because the fuse measures a volume it reads only the sign
/// of: a gap.
#[test]
fn a_volume_too_large_to_measure_is_not_known_because_a_fuse_refuses_a_cube_it_already_holds() {
    let mut model = a_model();
    let cube = a_cube(&mut model, 1e103);
    let measured = model
        .solid(cube)
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a volume of 1e309");

    let far = two_to_the(342);
    let mut model = Model::new(Scale::millimetres(100.0 * far).expect("a working size"));
    let one = block(&mut model, [far, far, far]);
    let copy = block(&mut model, [far, far, far]);
    let mut refusals = vec![measured];
    for [first, second] in [[one, copy], [copy, one]] {
        refusals.push(
            model
                .fuse(
                    &[first],
                    &[second],
                    Rule::Union,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect_err("the union of a cube of side 2^342 with itself"),
        );
    }

    for refusal in &refusals {
        assert!(
            matches!(refusal, Error::VolumeTooLargeToMeasure),
            "{refusal:?}"
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "not known");
    }
}

/// How long an ellipse is has an answer, between `2π` and `2π·√1.09` for this
/// one, and nothing here works it out yet.
#[test]
fn how_long_an_ellipse_is_is_a_gap() {
    let mut model = Model::new(Scale::millimetres(1.0).expect("a scale"));
    let square = a_cap_square_to_the_axis(0.0, -1.0);
    let leaning =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 5.0), Vector::new(0.3, 0.0, 1.0))
            .expect("the top cap");
    let bar = model
        .add_bar_cut_off_by(
            1.0,
            [square, leaning],
            &Isometry::identity(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar with a leaning top");
    let solid = model.solid(bar).expect("this model made the key");
    let cap = solid
        .faces()
        .find(|face| face.surface().as_plane() == Some(&leaning))
        .expect("the leaning cap");
    let ellipse = cap
        .outer_wire()
        .coedges()
        .next()
        .expect("one edge round the cap")
        .edge();
    let refusal = ellipse.length().expect_err("an ellipse");
    assert!(
        matches!(
            refusal,
            Error::CannotYetMeasureHowLongThisCurveIs { edge } if edge == ellipse.key()
        ),
        "{refusal:?}"
    );
    assert_eq!(refusal.limit_or_gap().to_string(), "gap");
}

/// What one call of the solid builder refuses is a limit of the draft as it was
/// handed over: a draft with a corner that is not a number, or a wire of two
/// steps, describes no solid.
#[test]
fn what_one_builder_call_refuses_is_a_limit_of_the_draft() {
    let mut refusals = Vec::new();

    let mut builder = SolidBuilder::new();
    refusals.push(
        builder
            .add_vertex(Point::new(0.0, f64::NAN, 0.0))
            .expect_err("a corner that is not a number"),
    );

    let mut builder = SolidBuilder::new();
    let corner = builder.add_vertex(Point::origin()).expect("finite");
    refusals.push(
        builder
            .add_edge(corner, corner, EdgeRun::Straight)
            .expect_err("a straight edge from a corner back to itself"),
    );

    let mut one = SolidBuilder::new();
    let stranger = one.add_vertex(Point::new(9.0, 9.0, 9.0)).expect("finite");
    let mut other = SolidBuilder::new();
    let mine = other.add_vertex(Point::origin()).expect("finite");
    refusals.push(
        other
            .add_edge(mine, stranger, EdgeRun::Straight)
            .expect_err("a corner from another builder"),
    );

    let mut builder = SolidBuilder::new();
    let ends = [Point::origin(), Point::new(1.0, 0.0, 0.0)]
        .map(|end| builder.add_vertex(end).expect("finite"));
    let edge = builder
        .add_edge(ends[0], ends[1], EdgeRun::Straight)
        .expect("two different corners");
    let there = builder
        .add_coedge(edge, Orientation::Forward)
        .expect("the edge is here");
    let back = builder
        .add_coedge(edge, Orientation::Reversed)
        .expect("the edge is here");
    refusals.push(
        builder
            .add_wire(&[DraftStep::Along(there), DraftStep::Along(back)])
            .expect_err("a wire of two steps"),
    );

    let mut builder = SolidBuilder::new();
    let corners: Vec<DraftVertex> = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        .into_iter()
        .map(|[x, y]| builder.add_vertex(Point::new(x, y, 0.0)).expect("finite"))
        .collect();
    let broken = [(0, 1), (2, 3), (3, 0)].map(|(from, to)| {
        let edge = builder
            .add_edge(corners[from], corners[to], EdgeRun::Straight)
            .expect("a side of a square");
        DraftStep::Along(
            builder
                .add_coedge(edge, Orientation::Forward)
                .expect("the edge is here"),
        )
    });
    refusals.push(
        builder
            .add_wire(&broken)
            .expect_err("three sides of a square that do not join up"),
    );

    let mut builder = SolidBuilder::new();
    let steps = a_triangle(&mut builder);
    builder.add_wire(&steps).expect("a triangle closes");
    refusals.push(
        builder
            .add_wire(&steps)
            .expect_err("the same coedges in a second wire"),
    );

    let mut builder = SolidBuilder::new();
    refusals.push(
        builder
            .add_face(a_flat_floor(), &[])
            .expect_err("a face with no wires"),
    );
    refusals.push(builder.add_shell(&[]).expect_err("a shell with no faces"));

    let (builder, _) = ACube::default().described();
    refusals.push(built(builder, &mut a_model()).expect_err("a description with no shell at all"));

    assert_eq!(
        refusals.iter().map(name_of).collect::<Vec<_>>(),
        [
            "ADraftPointIsNotFinite",
            "ADraftEdgeHasNoLength",
            "ADraftHandleCameFromAnotherBuilder",
            "ADraftWireIsTooShort",
            "ADraftWireDoesNotClose",
            "ADraftEntityIsUsedTwice",
            "ADraftFaceHasNoWires",
            "ADraftShellHasNoFaces",
            "ADraftSolidHasNoShell",
        ]
    );
    for refusal in &refusals {
        assert_eq!(refusal.limit_or_gap().to_string(), "limit", "{refusal:?}");
    }
}

/// What takes the whole draft to see is a limit of the draft too: a skin open
/// along an edge, or closed round a loop given backwards, encloses no solid.
#[test]
fn what_the_whole_draft_refuses_is_a_limit_of_the_draft() {
    let refusals = [
        ACube {
            spare_corner: true,
            ..ACube::default()
        },
        ACube {
            leave_out: Some(1),
            ..ACube::default()
        },
        ACube {
            walk_backwards: Some(1),
            ..ACube::default()
        },
    ]
    .map(|broken| {
        let (mut builder, faces) = broken.described();
        builder.add_shell(&faces).expect("a shell of some faces");
        built(builder, &mut a_model()).expect_err("a draft broken in one place")
    });
    assert_eq!(
        refusals.each_ref().map(name_of),
        [
            "ADraftEntityIsUsedByNothing",
            "ADraftEdgeIsNotWalkedByTwoCoedges",
            "ADraftEdgeIsWalkedTheSameWayTwice",
        ]
    );
    for refusal in &refusals {
        assert_eq!(refusal.limit_or_gap().to_string(), "limit", "{refusal:?}");
    }
}

/// A model wider than its smallest step by more than `f64` resolves is a limit
/// of today's kernel, which holds every number as one `f64` (§437.6), where
/// the step is really there: two blocks a thousand apart, one of them fused
/// with a step of `2^-30`. It was a gap where the fuse left the step itself:
/// the plate less a block cornered on its round tab's rim was refused for an
/// arc `1.78e-15` long that its answer, the plate, does not have, until Q2296
/// folded the corner that made it (`a_block_cornered_on_a_round_tabs_rim.rs`).
/// The refusal carries nothing that tells the two apart, so it says not known.
#[test]
fn a_model_too_wide_for_its_smallest_step_is_not_known_to_be_a_limit() {
    let tiny = two_to_the(-30);
    let mut model = Model::new(Scale::millimetres(4.0).expect("a working size"));
    let _far = two_millimetre_block_at(&mut model, Vector::new(1024.0, 0.0, 0.0));
    let here = two_millimetre_block_at(&mut model, Vector::zero());
    let corner = 2.0 - tiny;
    let overlapping = two_millimetre_block_at(&mut model, Vector::new(corner, corner, corner));
    model
        .fuse(
            &[here],
            &[overlapping],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two blocks overlapping in one corner unite");

    let refusal = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("the far block and the union together are too wide to compute in");
    assert!(
        matches!(refusal, Error::Invalid(Invalid::DynamicRangeTooWide { .. })),
        "{refusal:?}"
    );
    assert_eq!(refusal.limit_or_gap().to_string(), "not known");
}

/// A refusal that carries a refusal of geometry, or of the sweep, says what the
/// one it carries says, whichever of the three that is.
#[test]
fn a_refusal_carrying_another_says_what_the_one_it_carries_says() {
    let mut model = a_model();
    let refusal = model
        .add_ball(0.0, &Isometry::identity())
        .expect_err("a ball of no size");
    assert!(
        matches!(
            refusal,
            Error::Geometry(rcad_geom::Error::RadiusNotPositive { .. })
        ),
        "{refusal:?}"
    );
    assert_eq!(refusal.limit_or_gap().to_string(), "limit");

    let cube = a_cube(&mut model, 1.0);
    let edge = model
        .solid(cube)
        .expect("this model made the key")
        .faces()
        .next()
        .expect("a cube has faces")
        .outer_wire()
        .coedges()
        .next()
        .expect("a face of a cube has sides")
        .edge()
        .key();
    let geometry = || {
        [
            Sphere::new(Point::origin(), 0.0).expect_err("a ball of no size"),
            rcad_geom::Error::CannotWorkOutHowBigACrossingIs,
            rcad_geom::Error::PointsAreInALine,
        ]
    };
    for (carried, wanted) in geometry().into_iter().zip(["limit", "gap", "not known"]) {
        assert_eq!(carried.limit_or_gap().to_string(), wanted, "{carried:?}");
    }
    for (why, wanted) in geometry().into_iter().zip(["limit", "gap", "not known"]) {
        assert_eq!(Error::Geometry(why).limit_or_gap().to_string(), wanted);
    }
    for (why, wanted) in geometry().into_iter().zip(["limit", "gap", "not known"]) {
        let carrying = Error::ADraftEdgeHasACrossingThatCannotBeFollowed {
            edge: 0,
            why: Box::new(why),
        };
        assert_eq!(carrying.limit_or_gap().to_string(), wanted);
    }
    for (why, wanted) in geometry().into_iter().zip(["limit", "gap", "not known"]) {
        let carrying = Error::ACrossingAlongThisEdgeCouldNotBeFollowed {
            edge,
            why: Box::new(why),
        };
        assert_eq!(carrying.limit_or_gap().to_string(), wanted);
    }

    let swept = Error::Invalid(Invalid::FaceHasNoWires);
    assert_eq!(swept.limit_or_gap().to_string(), "not known");
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a length")
}

fn a_cube(model: &mut Model, side: f64) -> SolidKey {
    model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("a cube of side {side:e}: {why}"))
}

/// A box of the given lengths, with its low corner at the origin.
fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, along_z, &Isometry::identity())
        .expect("building a box needs no decision")
}

fn two_millimetre_block_at(model: &mut Model, at: Vector) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a two-millimetre square");
    model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(at).expect("a plain translation"),
        )
        .expect("a two-millimetre block")
}

/// `2^exponent`, exactly.
fn two_to_the(exponent: i32) -> f64 {
    f64::from_bits(u64::try_from(1023 + exponent).expect("a normal exponent") << 52)
}

fn a_cap_square_to_the_axis(height: f64, outward: f64) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, outward))
        .expect("the z axis is a unit normal")
}

fn a_flat_floor() -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    )
}

/// The variant's name, which is what its `Debug` starts with.
fn name_of(refusal: &Error) -> String {
    let debug = format!("{refusal:?}");
    let end = debug
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(debug.len());
    debug[..end].to_owned()
}

fn built(builder: SolidBuilder, model: &mut Model) -> rcad_topo::Result<SolidKey> {
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// Three corners in the `z = 0` plane and the three coedges round them.
fn a_triangle(builder: &mut SolidBuilder) -> Vec<DraftStep> {
    let corners: Vec<DraftVertex> = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
        .into_iter()
        .map(|[x, y]| {
            builder
                .add_vertex(Point::new(x, y, 0.0))
                .expect("a corner of a triangle")
        })
        .collect();
    [(0, 1), (1, 2), (2, 0)]
        .into_iter()
        .map(|(from, to)| {
            let edge = builder
                .add_edge(corners[from], corners[to], EdgeRun::Straight)
                .expect("a side of a triangle");
            DraftStep::Along(
                builder
                    .add_coedge(edge, Orientation::Forward)
                    .expect("the edge is here"),
            )
        })
        .collect()
}

/// The unit cube as a description, with a knob for the one thing each test
/// breaks. `what_the_solid_builder_refuses.rs` has the same cube.
#[derive(Default)]
struct ACube {
    /// Describe every face but this one, which leaves the skin open.
    leave_out: Option<usize>,
    /// Walk this face's loop the other way round. It still closes.
    walk_backwards: Option<usize>,
    /// Describe a ninth corner no edge and no pole names.
    spare_corner: bool,
}

impl ACube {
    fn described(self) -> (SolidBuilder, Vec<(DraftFace, Orientation)>) {
        let mut builder = SolidBuilder::new();
        let corners: Vec<DraftVertex> = CORNERS
            .iter()
            .map(|&[x, y, z]| {
                builder
                    .add_vertex(Point::new(x, y, z))
                    .expect("a corner of the unit cube")
            })
            .collect();
        let edges: Vec<_> = SIDES
            .iter()
            .map(|&[from, to]| {
                builder
                    .add_edge(corners[from], corners[to], EdgeRun::Straight)
                    .expect("a side of the unit cube")
            })
            .collect();

        let mut faces = Vec::new();
        for (which, (through, normal, walk)) in FACES.into_iter().enumerate() {
            if self.leave_out == Some(which) {
                continue;
            }
            let mut walk = walk.to_vec();
            if self.walk_backwards == Some(which) {
                walk.reverse();
                for step in &mut walk {
                    step.1 = !step.1;
                }
            }
            let steps: Vec<DraftStep> = walk
                .into_iter()
                .map(|(side, forwards)| {
                    let orientation = if forwards {
                        Orientation::Forward
                    } else {
                        Orientation::Reversed
                    };
                    DraftStep::Along(
                        builder
                            .add_coedge(edges[side], orientation)
                            .expect("the side is here"),
                    )
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("a face of a cube closes");
            let surface = Surface::Plane(
                Plane::from_point_and_normal(
                    Point::new(through[0], through[1], through[2]),
                    Vector::new(normal[0], normal[1], normal[2]),
                )
                .expect("an axis is a unit normal"),
            );
            faces.push((
                builder
                    .add_face(surface, &[wire])
                    .expect("a face has its wire"),
                Orientation::Forward,
            ));
        }

        if self.spare_corner {
            builder
                .add_vertex(Point::new(5.0, 5.0, 5.0))
                .expect("a corner nothing names is still a finite point");
        }
        (builder, faces)
    }
}

/// The eight corners of the unit cube, `(x, y, z)` bit by bit.
const CORNERS: [[f64; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0],
];

/// The twelve sides: the bottom ring, the top ring, then the four uprights.
const SIDES: [[usize; 2]; 12] = [
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

/// The six faces: a point on the plane, its outward normal, and the four sides
/// of its loop with `true` for walking a side the way it is stored.
///
/// Every loop runs anticlockwise seen from outside, which is anticlockwise about
/// the outward normal — what `Model::validate` asks of an outer wire.
type AFace = ([f64; 3], [f64; 3], [(usize, bool); 4]);
const FACES: [AFace; 6] = [
    (
        [0.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [(3, false), (2, false), (1, false), (0, false)],
    ),
    (
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [(4, true), (5, true), (6, true), (7, true)],
    ),
    (
        [0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [(0, true), (9, true), (4, false), (8, false)],
    ),
    (
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [(1, true), (10, true), (5, false), (9, false)],
    ),
    (
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [(2, true), (11, true), (6, false), (10, false)],
    ),
    (
        [0.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [(3, true), (8, true), (7, false), (11, false)],
    ),
];
