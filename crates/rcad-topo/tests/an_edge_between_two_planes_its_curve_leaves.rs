//! A round bar whose flat lid is split in two, with the piece inside tilted off
//! the lid, so that an edge between two flat faces is drawn on one plane and
//! leaves the other.
//!
//! **Split by a circle** into an annulus and a disc, the disc tilted through
//! the circle's one corner. The circle is a closed edge between two flat faces,
//! and its run names the pipe of radius 1 to say where it goes. That pipe
//! crosses the lid in a circle and the tilted disc's plane in an ellipse, so
//! the two faces put the edge in two different places. The kernel drew
//! whichever the solid listed first: Q386's probe measured `113.003` with the
//! annulus first and `111.644` with the disc first, each `± 0.376`, for a bar
//! that is `36π`. Q574 refuses the edge instead, in `SolidBuilder::build` and
//! in `Model::validate`.
//!
//! **Split by half that circle and its chord**, the half disc tilted along the
//! chord. Both ends of the arc are on both planes, and the arc between them is
//! half a circle on the lid and half an ellipse on the tilted plane: the same
//! two places for an open edge, and the same dependence on which face is listed
//! first, which `the_tilted_half_disc_measures_by_which_face_is_listed_first`
//! holds. Q604 refuses that one too.
//!
//! The same bar with the inner piece on the lid's own plane, or on it written a
//! hair apart, is one plane spelt twice and still builds. So does a solid whose
//! two flat faces cross in one of a pipe's own straight lines, with the run
//! naming the pipe: the arc there is that line, and on both planes. And so does
//! one whose two faces cross in one of the straight lines through a cone's
//! apex, with the run naming the cone (Q613) — but not one whose two faces
//! cross in a line that is not one of the cone's own.
//!
//! **The circle may name a ball, a cone or a torus instead of the pipe**, each
//! of which crosses the lid in that same circle of radius 1. With the inner
//! piece a hair off the lid every one of them still builds (Q612) — a torus's
//! only with the lid listed first when the piece leans, since the walk draws no
//! circle on a plane leaning off square to a torus's axis. The far side of the
//! circle is then off the other plane by less than writing a plane down costs.
//! With the inner piece leaning a little further, so that the far side really
//! is off by more than that, every one of them is still refused.
//!
//! **The half disc's arc may name a cone the lid crosses in a parabola or a
//! hyperbola**, which runs on for ever and has no width. A hair off the lid
//! that arc still builds, bounded by its two ends (Q1086), and leaning further
//! it is still refused. An arc from one branch of a hyperbola to the other is
//! no arc, and the walk refuses it.
//!
//! **The checks after each mutation do not ask this, and need not** (Q614).
//! Moving a solid and copying one write no plane and no run, so a tilted piece
//! reaches them only from a file, and the sweep refuses it there. The kernel's
//! own solids have no such edge. A builder and a fuse sweep what they will
//! write in a model of their own first, and write nothing when it refuses.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane,
    Point, Scale, Sphere, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, DraftWire, Error, Invalid, Model, ProfileBuilder, Result,
    Rule, SolidBuilder, SolidKey,
};

const F: Orientation = Orientation::Forward;
const R: Orientation = Orientation::Reversed;

/// The builder sweeps what it was told in a model of its own before it writes
/// anything into the caller's, so the model it was handed holds nothing after.
#[test]
fn a_disc_tilted_off_the_lid_is_refused_whichever_face_is_listed_first() {
    for disc_first in [false, true] {
        let (model, built) =
            a_bar_with_its_lid_split_by_a_disc(pipe(1.0), tilted_through_the_corner(), disc_first);

        assert!(
            matches!(
                built,
                Err(Error::Invalid(
                    Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ),
            "disc first {disc_first}: the builder gave {built:?}"
        );
        assert_eq!(model.solids().count(), 0, "disc first {disc_first}");
    }
}

/// The same refusal from the sweep, for a model that never went through a
/// builder: the sound bar written out, its disc's plane swapped for the tilted
/// one in the bytes, and read back. `Deserialize` asks nothing about geometry,
/// so this is the route a model that holds the shape takes.
#[test]
fn the_sweep_refuses_the_tilted_disc_read_back_from_bytes() {
    for disc_first in [false, true] {
        let (model, built) = a_bar_with_its_lid_split_by_a_disc(pipe(1.0), the_lid(), disc_first);
        built.expect("the disc on the lid's own plane builds");
        let tilted = with_the_inner_plane_swapped(&model, disc_first, tilted_through_the_corner());

        let swept = tilted.validate(&Policy::certified(), &mut ProvenanceLog::new());

        assert!(
            matches!(
                swept,
                Err(Error::Invalid(
                    Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ),
            "disc first {disc_first}: the sweep gave {swept:?}"
        );
    }
}

#[test]
fn a_disc_on_the_lids_own_plane_still_builds_and_validates() {
    for disc in [the_lid(), the_lid_one_ulp_higher()] {
        for disc_first in [false, true] {
            let (model, built) = a_bar_with_its_lid_split_by_a_disc(pipe(1.0), disc, disc_first);

            built.unwrap_or_else(|why| {
                panic!("disc {disc:?}, disc first {disc_first}: refused with {why:?}")
            });
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("and the model it went into passes the sweep");
        }
    }
}

#[test]
fn a_half_disc_tilted_along_its_chord_is_refused_whichever_face_is_listed_first() {
    for half_first in [false, true] {
        let (_, built) = a_bar_with_its_lid_split_by_a_half_disc(
            pipe(1.0),
            tilted_along_the_chord(),
            half_first,
        );

        assert!(
            matches!(
                built,
                Err(Error::Invalid(
                    Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ),
            "half disc first {half_first}: the builder gave {built:?}"
        );
    }
}

#[test]
fn the_sweep_refuses_the_tilted_half_disc_read_back_from_bytes() {
    for half_first in [false, true] {
        let tilted = the_tilted_half_disc_read_back_from_bytes(half_first);

        let swept = tilted.validate(&Policy::certified(), &mut ProvenanceLog::new());

        assert!(
            matches!(
                swept,
                Err(Error::Invalid(
                    Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ),
            "half disc first {half_first}: the sweep gave {swept:?}"
        );
    }
}

/// A fuse sweeps its answer before it writes it into the caller's model: it
/// writes it into a throwaway model first and refuses it there if the sweep
/// does, with this check among the sweep's and the answer's own room. So a
/// check of its own after the write would have nothing left to refuse.
///
/// Each bar here runs from `z = −1000` up to its lid at 4, with the piece of
/// its lid `2e-12` off the lid at the far side of its curve: a disc and a half
/// disc that high above the lid, a disc leaning through its corner and a half
/// disc leaning along its chord. That
/// is inside the `1.4e-11` room a corner `1000` out gets, so each builds and
/// passes the sweep, and outside the `5.7e-14` room of the answer cut down to
/// the top 4. Whatever the fuse answers, the model is left holding the two
/// solids it held, or every solid it made passes the sweep.
///
/// Today none of them reaches that sweep: two planes apart meet nowhere, and
/// the fuse draws the edge of a piece leaning off the lid where the two planes
/// cross, so it refuses both leaning pieces by name, as
/// `a_piece_leaning_off_the_lid_is_refused_for_the_edge_round_it` pins.
#[test]
fn a_fuse_writes_no_piece_of_a_lid_off_the_lid_into_the_callers_model() {
    let above =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0 + 2e-12), up()).expect("a plane");
    let mut wrote = Vec::new();
    for inner_first in [false, true] {
        let [disc_leaning, half_disc_leaning] = the_tall_bars_leaning_off_the_lid(inner_first);
        for (which, (mut model, built)) in [
            (
                "disc above",
                a_bar_whose_lid_is_split(-1000.0, above, inner_first, a_circle_naming(pipe(1.0))),
            ),
            (
                "half disc above",
                a_bar_whose_lid_is_split(
                    -1000.0,
                    above,
                    inner_first,
                    a_half_circle_and_its_chord(pipe(1.0)),
                ),
            ),
            ("disc leaning", disc_leaning),
            ("half disc leaning", half_disc_leaning),
        ] {
            let bar = built.unwrap_or_else(|why| panic!("{which}: the tall bar gave {why:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{which}: the sweep gave {why:?}"));
            let block = a_block_whose_floor_is_at(0.0, &mut model);

            let fused = model.fuse(
                &[bar],
                &[block],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );

            let what_it_left = match &fused {
                Err(_) if model.solids().count() == 2 => continue,
                Err(_) => format!("{} solids", model.solids().count()),
                Ok(fused) => {
                    let swept: Result<Vec<()>> = fused
                        .solids()
                        .iter()
                        .map(|&made| {
                            model.validate_solid(
                                made,
                                &Policy::certified(),
                                &mut ProvenanceLog::new(),
                            )
                        })
                        .collect();
                    match swept {
                        Ok(_) => continue,
                        Err(why) => format!("a solid the sweep refuses with {why:?}"),
                    }
                }
            };
            wrote.push(format!(
                "{which}, inner piece first {inner_first}: {fused:?}, leaving {what_it_left}"
            ));
        }
    }
    assert!(wrote.is_empty(), "{wrote:#?}");
}

/// The tall bars of `a_fuse_writes_no_piece_of_a_lid_off_the_lid_into_the_callers_model`
/// whose piece leans off the lid, each intersected with the block from
/// `z = 0`. The fuse draws the edge round the piece where the lid's plane and
/// the piece's cross, a straight line, and not along the pipe its run names.
/// On that line the disc's circle is a closed edge on a crossing that does not
/// close (Q1675), and the half disc's arc covers the one stretch its chord
/// covers (Q1914). The fuse panicked on each, and refuses each by name now,
/// until the edge is drawn along the pipe (Q2125).
#[test]
fn a_piece_leaning_off_the_lid_is_refused_for_the_edge_round_it() {
    let mut not_refused = Vec::new();
    for inner_first in [false, true] {
        for (which, (mut model, built)) in ["disc", "half disc"]
            .into_iter()
            .zip(the_tall_bars_leaning_off_the_lid(inner_first))
        {
            let bar = built.unwrap_or_else(|why| panic!("{which}: the tall bar gave {why:?}"));
            let block = a_block_whose_floor_is_at(0.0, &mut model);

            let fused = model.fuse(
                &[bar],
                &[block],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );

            if !matches!(
                fused,
                Err(Error::AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve { .. })
            ) {
                not_refused.push(format!(
                    "{which}, inner piece first {inner_first}: {fused:?}"
                ));
            }
        }
    }
    assert!(not_refused.is_empty(), "{not_refused:#?}");
}

/// The bar from `z = 0` whose disc leans a tenth through its corner, read back
/// from bytes: the sweep refuses it, and a fuse does not sweep what it is
/// handed. Fused every way with a cube of side 2 through the bar's floor, or
/// moved `(10, 10, 0)` clear of the bar, the disc's circle is drawn on the
/// straight line the lid's plane and the disc's cross in, a closed edge on a
/// crossing that does not close. The fuse panicked on all 12 (Q1675).
#[test]
fn a_fuse_with_the_tilted_disc_read_back_from_bytes_is_refused_for_the_edge_round_it() {
    let mut not_refused = Vec::new();
    for disc_first in [false, true] {
        let (model, built) = a_bar_with_its_lid_split_by_a_disc(pipe(1.0), the_lid(), disc_first);
        built.expect("the disc on the lid's own plane builds");
        for at in [Vector::new(-1.0, -1.0, -1.0), Vector::new(10.0, 10.0, 0.0)] {
            for rule in [Rule::Union, Rule::Difference, Rule::Intersection] {
                let mut tilted =
                    with_the_inner_plane_swapped(&model, disc_first, tilted_through_the_corner());
                let bar = tilted.solids().next().expect("the bar").key();
                let cube = tilted
                    .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a cube");
                tilted
                    .place_solid(cube, &Isometry::translation(at).expect("a movement"))
                    .expect("the cube moves");

                let fused = tilted.fuse(
                    &[bar],
                    &[cube],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );

                if !matches!(
                    fused,
                    Err(Error::AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve { .. })
                ) {
                    not_refused.push(format!(
                        "disc first {disc_first}, the cube at {at:?}, {rule:?}: {fused:?}"
                    ));
                }
            }
        }
    }
    assert!(not_refused.is_empty(), "{not_refused:#?}");
}

/// Moving a solid and copying one are the two mutations
/// `Model::check_touched_edge` runs after that write no plane, no corner and
/// no run of their own: a movement goes on the shell, and a copy is the
/// original bit for bit. So a tilted piece reaches them only in a solid that
/// already held it, and the one door such a solid comes in by is
/// `Deserialize`, which asks nothing that needs arithmetic
/// (`docs/decisions.md` §44.5). Both answer, and the sweep still refuses the
/// bar, and its copy, by the same name. A panic here would be a panic on what
/// a caller's bytes said.
#[test]
fn moving_or_copying_a_tilted_piece_read_back_from_bytes_leaves_it_for_the_sweep() {
    let away = Isometry::translation(Vector::new(10.0, 0.0, 0.0)).expect("a movement");
    let mut not_left = Vec::new();
    for inner_first in [false, true] {
        for (mut model, expected) in the_two_tilted_pieces_read_back_from_bytes(inner_first) {
            let bar = model.solids().next().expect("the bar").key();

            model.place_solid(bar, &away).expect("the bar moves");
            let copy = model.copy_solid(bar).expect("the bar copies");

            for (which, solid) in [("bar", bar), ("copy", copy)] {
                let swept =
                    model.validate_solid(solid, &Policy::certified(), &mut ProvenanceLog::new());
                if !matches!(&swept, Err(Error::Invalid(broken)) if *broken == expected) {
                    not_left.push(format!(
                        "{expected:?}, inner piece first {inner_first}, the {which}: {swept:?}"
                    ));
                }
            }
        }
    }
    assert!(not_left.is_empty(), "{not_left:#?}");
}

/// The rest of what `Model::check_touched_edge` runs after is the kernel's own
/// solids, and none of them has an edge this check could refuse: it asks only
/// of an edge that runs `RightRoundWhereItCrosses` or `PartOfWhereItCrosses`
/// with every face along it flat, and every such edge these write has a ball
/// or a pipe along it. The ball's seam is the one such edge, so the count of
/// them is not nought.
#[test]
fn no_solid_the_kernel_makes_has_a_named_crossing_between_flat_faces_alone() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(10.0).expect("a size"));
    let ring = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .with_hole(vec![[1.0, 1.0], [1.0, 3.0], [3.0, 3.0], [3.0, 1.0]])
        .build(&policy, &mut log)
        .expect("a square with a square hole");
    let leaning_caps = [
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.25, 0.0, -1.0)),
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 3.0), Vector::new(0.0, 0.5, 1.0)),
    ]
    .map(|cap| cap.expect("a plane"));
    let made = [
        model.add_cube(2.0, &policy, &mut log),
        model.add_extrusion(&ring, 3.0, &Isometry::identity()),
        model.add_round_bar(1.0, 2.0, &Isometry::identity()),
        model.add_ball(1.5, &Isometry::identity()),
        model.add_bar_cut_off_by(1.0, leaning_caps, &Isometry::identity(), &policy, &mut log),
    ]
    .map(|made| made.expect("the kernel makes it"));

    let mut asked_about = 0;
    let mut between_flat_faces_alone = Vec::new();
    for key in made {
        let solid = model.solid(key).expect("a solid of this model");
        for edge in solid
            .faces()
            .flat_map(|face| face.wires())
            .flat_map(|wire| wire.coedges())
            .map(|coedge| coedge.edge())
        {
            let run = edge.run();
            if !matches!(
                run,
                EdgeRun::RightRoundWhereItCrosses { .. } | EdgeRun::PartOfWhereItCrosses { .. }
            ) {
                continue;
            }
            asked_about += 1;
            let something_curved_along_it = edge.neighbouring_faces().any(|face| {
                let face = model.face(face).expect("a face of this model");
                !matches!(face.surface(), Surface::Plane(_))
            });
            if !something_curved_along_it {
                between_flat_faces_alone.push(format!("{run:?}"));
            }
        }
    }
    assert!(
        asked_about > 0,
        "no edge of these is one the check asks about"
    );
    assert!(
        between_flat_faces_alone.is_empty(),
        "{between_flat_faces_alone:#?}"
    );
}

/// Why the tilted half disc is refused rather than drawn: the volume of the
/// model that holds it depends on which of its two lid faces is listed
/// first, by far more than rounding could ever account for. Charging each
/// flat face for the mesh points that tilt off its own plane now admits that
/// gap into the measure's own error bars, so the two orderings' ranges
/// overlap instead of missing each other — the measure owns up to the gap
/// rather than hiding it. If this stops holding, the refusal's reason in
/// `Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes` has gone stale.
#[test]
fn the_tilted_half_disc_measures_by_which_face_is_listed_first() {
    let [(lid_first, lid_first_error), (half_first, half_first_error)] =
        [false, true].map(|half_first| {
            let model = the_tilted_half_disc_read_back_from_bytes(half_first);
            let solid = model.solids().next().expect("the bar");
            let measured = solid
                .measure(
                    Deflection::of(0.001).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("the bar measures");
            (measured.volume(), measured.volume_error())
        });

    let apart = (lid_first - half_first).abs();
    assert!(
        apart > 1e-6,
        "the lid first measured {lid_first} and the half disc first {half_first}, only \
         {apart} apart — that is rounding, not the order-dependence this test is about"
    );
    assert!(
        apart <= lid_first_error + half_first_error,
        "the lid first measured {lid_first} ± {lid_first_error} and the half disc first \
         {half_first} ± {half_first_error}, {apart} apart but the ranges still miss each \
         other instead of overlapping"
    );
}

#[test]
fn a_half_disc_on_the_lids_own_plane_still_builds_and_validates() {
    for half in [the_lid(), the_lid_one_ulp_higher()] {
        for half_first in [false, true] {
            let (model, built) =
                a_bar_with_its_lid_split_by_a_half_disc(pipe(1.0), half, half_first);

            built.unwrap_or_else(|why| {
                panic!("half disc {half:?}, half disc first {half_first}: refused with {why:?}")
            });
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("and the model it went into passes the sweep");
        }
    }
}

/// Two flat faces on the planes `x = 5` and `y = 5` crossing in the line
/// `x = y = 5`, which is one of the straight lines the pipe of radius 5 about
/// `(2, 1)` is made of. The edge along it names that pipe, so its run names a
/// curved surface between two planes that are not one — and it is still one
/// line, on both of them. The walk draws it straight from either face, and so
/// must the sweep let it through.
#[test]
fn two_faces_crossing_in_a_pipes_own_line_still_build() {
    let mut model = Model::new(Scale::millimetres(10.0).expect("a size"));

    let built = a_tetrahedron_with_an_edge_down_a_pipe(&mut model);

    built.unwrap_or_else(|why| panic!("refused with {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and the model it went into passes the sweep");
}

/// The same two faces and the same line, with the edge naming a cone that line
/// is one of the straight lines through the apex of: [`down_a_cones_ruling`].
/// Both planes hold the apex and cut the cone in straight lines, so the walk
/// draws the edge straight from either face, and the sweep lets it through
/// whichever is listed first (Q613).
#[test]
fn two_faces_crossing_in_a_line_through_a_cones_apex_still_build() {
    for x_first in [false, true] {
        let mut model = Model::new(Scale::millimetres(10.0).expect("a size"));

        let built =
            a_tetrahedron_whose_upright_edge_runs(&mut model, down_a_cones_ruling(), x_first);

        built.unwrap_or_else(|why| panic!("x = 5 first {x_first}: refused with {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("and the model it went into passes the sweep");
    }
}

/// The same two faces with the edge naming a cone through both its ends that
/// the line `x = y = 5` is not one of the straight lines of. Each lets through
/// a bound that is wider than the one that lets the cone above through:
///
/// - **One face holds the apex, and the other does not.** The cone with its
///   apex at `(0, 5, 2.5)`, about `x`, with slope `1/2`: the face on `y = 5`
///   holds the apex and the axis, and cuts the cone in two straight lines
///   through the apex, one through each end, so from that face the edge is
///   drawn straight and on both planes. But `x = 5` cuts the cone in the circle
///   of radius `2.5` about `(5, 5, 2.5)`, and the half of it the edge names from
///   there bows `2.5` off `y = 5`. A bound read off the ends once either face
///   draws the edge straight lets this through.
/// - **Both faces lean at the cone's own angle, and neither holds the apex.**
///   The cone with its apex at `(2.5, 3.75, 2.5)`, about `(1, 1, 0)`, with
///   slope 1: each face is parallel to one of its straight lines, and cuts it in
///   a parabola through both ends, which bows `1.25` off `y = 5` from the face
///   on `x = 5` and `2.5` off `x = 5` from the other. A bound read off the ends
///   wherever the lean alone says straight lines, without the apex, lets this
///   through.
#[test]
fn two_faces_whose_line_is_not_one_of_the_cones_own_are_still_refused() {
    let half_a_circle = {
        let axis =
            Axis::new(Point::new(0.0, 5.0, 2.5), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
        let away_from_y = HalfSpace::through_a_point_facing(
            Point::new(5.0, 5.0, 2.5),
            Vector::new(0.0, -1.0, 0.0),
        )
        .expect("a half-space");
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cone(Cone::new(axis, 0.5).expect("a cone")),
            picked_by: smallvec::smallvec![Picker::from(away_from_y)],
        }
    };
    let two_parabolas = {
        let axis =
            Axis::new(Point::new(2.5, 3.75, 2.5), Vector::new(1.0, 1.0, 0.0)).expect("an axis");
        let the_nappe_both_ends_are_on =
            HalfSpace::through_a_point_facing(axis.origin(), axis.direction())
                .expect("a half-space");
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cone(Cone::new(axis, 1.0).expect("a cone")),
            picked_by: smallvec::smallvec![Picker::from(the_nappe_both_ends_are_on)],
        }
    };
    let mut not_refused = Vec::new();
    for run in [half_a_circle, two_parabolas] {
        for x_first in [false, true] {
            let mut model = Model::new(Scale::millimetres(10.0).expect("a size"));
            let built = a_tetrahedron_whose_upright_edge_runs(&mut model, run.clone(), x_first);
            if !matches!(
                built,
                Err(Error::Invalid(
                    Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ) {
                not_refused.push(format!("{run:?}, x = 5 first {x_first}: {built:?}"));
            }
        }
    }
    assert!(not_refused.is_empty(), "{not_refused:#?}");
}

#[test]
fn a_disc_a_hair_off_the_lid_builds_when_its_circle_names_a_ball() {
    for inner_first in [false, true] {
        a_hair_off_the_lid_builds(a_ball(), inner_first);
    }
}

/// A pointed cone below the lid, a frustum's cone whose apex is twice the bar's
/// height below it, and a cone whose apex is above the lid so that the circle is
/// on its other nappe.
#[test]
fn a_disc_a_hair_off_the_lid_builds_when_its_circle_names_a_cone_on_either_nappe() {
    for cone in three_cones() {
        for inner_first in [false, true] {
            a_hair_off_the_lid_builds(cone, inner_first);
        }
    }
}

/// The leaning pieces only with the lid listed first: a plane leaning off square
/// to a torus's axis is one the walk has no circle for, so with the leaning piece
/// first the bar is refused for that, by the walk and not by this check.
#[test]
fn a_disc_a_hair_off_the_lid_builds_when_its_circle_names_a_torus() {
    a_hair_off_the_lid_builds(a_torus(), false);
    let above = the_lid_one_ulp_higher();
    the_bar_builds(a_torus(), [above, above], true);
}

/// The inner piece leaning far enough that its circle's far side really is
/// further off the other plane than writing a plane down costs: `8e-14` against
/// the `5.7e-14` the bar has room for. Leaning `4e-14` through the disc's one
/// corner puts the far side of the circle `8e-14` off, and leaning `8e-14` along
/// the half disc's chord puts the middle of the half circle `8e-14` off.
///
/// The circle is 2 across on the lid, and a bound of half that would let the
/// disc through: `1 × 4e-14` is inside the room.
#[test]
fn a_disc_leaning_off_the_lid_past_what_writing_a_plane_down_costs_is_still_refused() {
    let surfaces = [pipe(1.0), a_ball(), a_torus()]
        .into_iter()
        .chain(three_cones());
    let mut not_refused = Vec::new();
    for named in surfaces {
        for inner_first in [false, true] {
            let disc = leaning_off_the_lid_through(Point::new(1.0, 0.0, 4.0), 4e-14);
            let (_, built) = a_bar_with_its_lid_split_by_a_disc(named, disc, inner_first);
            if !matches!(
                built,
                Err(Error::Invalid(
                    Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ) {
                not_refused.push(format!("{named:?}, disc first {inner_first}: {built:?}"));
            }

            let half = leaning_off_the_lid_through(Point::new(0.0, 0.0, 4.0), 8e-14);
            let (_, built) = a_bar_with_its_lid_split_by_a_half_disc(named, half, inner_first);
            if !matches!(
                built,
                Err(Error::Invalid(
                    Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ) {
                not_refused.push(format!(
                    "{named:?}, half disc first {inner_first}: {built:?}"
                ));
            }
        }
    }
    assert!(not_refused.is_empty(), "{not_refused:#?}");
}

/// The half disc's arc naming a cone the lid crosses in a parabola, or in one
/// branch of a hyperbola, with the half disc one ulp above the lid and leaning
/// `1e-14` off it. Each builds and passes the sweep (Q1086). The far side of
/// the arc is then at most `1e-14` off the other plane, against the `5.7e-14`
/// the bar has room for.
///
/// The lean is the way that keeps the half disc's own plane crossing the
/// parabola's cone in a hyperbola, and that leaning half disc only with the lid
/// listed first: with the half disc first the walk refuses to name which branch
/// of that hyperbola, a lean off the lid's parabola, the arc is on —
/// `CannotNameWhichPieceOfACrossingThisIs`, the walk's refusal and not this
/// check's. Leaning the other way, the half disc's plane crosses that cone in an
/// ellipse, whose width is the whole ellipse's, and the bar is refused
/// `AnArcBetweenTwoFlatFacesOnDifferentPlanes` either way round.
#[test]
fn a_half_disc_a_hair_off_the_lid_builds_when_its_arc_is_a_parabola_or_a_hyperbola() {
    let leaning = leaning_off_the_lid_through(Point::new(0.0, 0.0, 4.0), -1e-14);
    let parabola = a_cone_the_lid_crosses_in_a_parabola();
    let hyperbola = a_cone_the_lid_crosses_in_a_hyperbola();
    let mut not_built = Vec::new();
    for (named, half, half_first) in [
        (parabola, the_lid_one_ulp_higher(), false),
        (parabola, the_lid_one_ulp_higher(), true),
        (parabola, leaning, false),
        (hyperbola, the_lid_one_ulp_higher(), false),
        (hyperbola, the_lid_one_ulp_higher(), true),
        (hyperbola, leaning, false),
        (hyperbola, leaning, true),
    ] {
        let (model, built) = a_bar_with_its_lid_split_by_a_half_disc(named, half, half_first);
        let swept =
            built.and_then(|_| model.validate(&Policy::certified(), &mut ProvenanceLog::new()));
        if let Err(why) = swept {
            not_built.push(format!(
                "{named:?} on {half:?}, half disc first {half_first}: {why:?}"
            ));
        }
    }
    assert!(not_built.is_empty(), "{not_built:#?}");
}

/// An arc naming the cone the lid crosses in a hyperbola, from `(0, −1, 4)` on
/// the branch nearer the bar's axis to `(2.5, 1, 4)` on the other branch, which
/// is on the cone's other nappe. No curve on the lid runs between the two, so
/// there is no arc for the bound read off its ends to cover: with the piece it
/// closes off one ulp above the lid, which is where that bound is asked, the bar
/// is refused whichever face is listed first, by the walk, which will not name
/// a branch for two ends on two sides of the apex.
#[test]
fn an_arc_from_one_branch_of_a_hyperbola_to_the_other_is_refused() {
    let y_at_least_minus_one =
        HalfSpace::through_a_point_facing(Point::new(0.0, -1.0, 0.0), Vector::new(0.0, 1.0, 0.0))
            .expect("a half-space");
    let run = EdgeRun::PartOfWhereItCrosses {
        and: a_cone_the_lid_crosses_in_a_hyperbola(),
        picked_by: smallvec::smallvec![Picker::from(y_at_least_minus_one)],
    };
    for inner_first in [false, true] {
        let (_, built) = a_bar_with_its_lid_split_by_an_arc_and_its_chord(
            run.clone(),
            [Point::new(0.0, -1.0, 4.0), Point::new(2.5, 1.0, 4.0)],
            the_lid_one_ulp_higher(),
            inner_first,
        );

        assert!(
            matches!(
                built,
                Err(Error::Geometry(
                    rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs
                ))
            ),
            "inner piece first {inner_first}: the builder gave {built:?}"
        );
    }
}

/// The same two arcs with the half disc leaning `2e-13` off the lid either way,
/// which puts the parabola's lowest point `(1, 0, 4)` `2e-13` off the other
/// plane and the hyperbola's `(0.5, 0, 4)` `1e-13`, against the `5.7e-14` the
/// bar has room for. Every one is still refused.
#[test]
fn a_half_disc_leaning_past_the_room_is_still_refused_when_its_arc_is_a_parabola_or_a_hyperbola() {
    let mut not_refused = Vec::new();
    for named in [
        a_cone_the_lid_crosses_in_a_parabola(),
        a_cone_the_lid_crosses_in_a_hyperbola(),
    ] {
        for lean in [2e-13, -2e-13] {
            let half = leaning_off_the_lid_through(Point::new(0.0, 0.0, 4.0), lean);
            for half_first in [false, true] {
                let (_, built) = a_bar_with_its_lid_split_by_a_half_disc(named, half, half_first);
                if !matches!(
                    built,
                    Err(Error::Invalid(
                        Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes
                    ))
                ) {
                    not_refused.push(format!(
                        "{named:?} leaning {lean}, half disc first {half_first}: {built:?}"
                    ));
                }
            }
        }
    }
    assert!(not_refused.is_empty(), "{not_refused:#?}");
}

/// The bar whose circle names `named`, with the inner piece one ulp above the
/// lid and then leaning `1e-14` off it, builds and passes the sweep.
///
/// The far side of the circle is then at most `2e-14` off the other plane, and
/// writing a plane down costs `5.7e-14` here: sixteen times
/// `Plane::SIGNED_DISTANCE_ROUNDOFF`, which is `4ε`, times the largest
/// coordinate, which is `4`.
fn a_hair_off_the_lid_builds(named: Surface, inner_first: bool) {
    let above = the_lid_one_ulp_higher();
    the_bar_builds(named, [above, above], inner_first);
    let leaning = [
        leaning_off_the_lid_through(Point::new(1.0, 0.0, 4.0), 1e-14),
        leaning_off_the_lid_through(Point::new(0.0, 0.0, 4.0), 1e-14),
    ];
    the_bar_builds(named, leaning, inner_first);
}

/// The bar split by a disc on `disc` and by a half disc on `half`, the circle
/// naming `named`, each builds and passes the sweep.
fn the_bar_builds(named: Surface, [disc, half]: [Plane; 2], inner_first: bool) {
    let (model, built) = a_bar_with_its_lid_split_by_a_disc(named, disc, inner_first);
    built.unwrap_or_else(|why| {
        panic!("{named:?} on disc {disc:?}, disc first {inner_first}: refused with {why:?}")
    });
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and the model it went into passes the sweep");

    let (model, built) = a_bar_with_its_lid_split_by_a_half_disc(named, half, inner_first);
    built.unwrap_or_else(|why| {
        panic!("{named:?} on half disc {half:?}, half first {inner_first}: refused with {why:?}")
    });
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and the model it went into passes the sweep");
}

/// The ball of radius 1 about the middle of the lid.
fn a_ball() -> Surface {
    Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 1.0).expect("a ball"))
}

/// Three cones up the bar's axis, each crossing the lid in the circle of radius
/// 1: slope 1 from an apex 1 below the lid, slope `1/8` from an apex 8 below it,
/// and slope 1 from an apex 1 above it, where the circle is on the nappe below
/// the apex.
fn three_cones() -> [Surface; 3] {
    [(3.0, 1.0), (-4.0, 0.125), (5.0, 1.0)].map(|(apex, slope)| {
        let axis = Axis::new(Point::new(0.0, 0.0, apex), up()).expect("an axis");
        Surface::Cone(Cone::new(axis, slope).expect("a cone"))
    })
}

/// The cone of slope 1 with its apex at `(1, 0, 3.5)`, half a unit under the
/// lid, about `(−1, 0, 1)`: one of its straight lines runs level along `−x`, so
/// the lid crosses it in the parabola `x = 1 − y²`. That runs through the half
/// disc's two corners `(0, ±1, 4)`, and the half of it at `x ≥ 0` goes round
/// through `(1, 0, 4)`, its point nearest the apex.
fn a_cone_the_lid_crosses_in_a_parabola() -> Surface {
    let axis = Axis::new(Point::new(1.0, 0.0, 3.5), Vector::new(-1.0, 0.0, 1.0)).expect("an axis");
    Surface::Cone(Cone::new(axis, 1.0).expect("a cone"))
}

/// The cone of slope 1 lying on its side, with its apex at `(1.25, 0, 3.25)`
/// and its axis along `−x`, so the lid crosses it in the hyperbola
/// `(x − 1.25)² = y² + 0.75²`. The branch nearer the bar's axis runs through
/// the half disc's two corners `(0, ±1, 4)`, and the half of it at `x ≥ 0`
/// goes round through `(0.5, 0, 4)`, its point nearest the apex. The other
/// branch is on the other nappe, at `x ≥ 2`.
fn a_cone_the_lid_crosses_in_a_hyperbola() -> Surface {
    let axis =
        Axis::new(Point::new(1.25, 0.0, 3.25), Vector::new(-1.0, 0.0, 0.0)).expect("an axis");
    Surface::Cone(Cone::new(axis, 1.0).expect("a cone"))
}

/// The torus about the bar's axis, centred in the lid, whose outer circle on the
/// lid is the one of radius 1: a ring of `3/4` and a tube of `1/4`.
fn a_torus() -> Surface {
    let axis = Axis::new(Point::new(0.0, 0.0, 4.0), up()).expect("an axis");
    Surface::Torus(Torus::new(axis, 0.75, 0.25).expect("a torus"))
}

/// A plane through `through`, a point of the lid, leaning `by` off it along `x`.
fn leaning_off_the_lid_through(through: Point, by: f64) -> Plane {
    Plane::from_point_and_normal(through, Vector::new(-by, 0.0, 1.0)).expect("a plane")
}

/// The plane the disc is tilted onto: through the circle's corner `(1, 0, 4)`,
/// leaning a tenth along `x`.
fn tilted_through_the_corner() -> Plane {
    Plane::from_point_and_normal(Point::new(1.0, 0.0, 4.0), Vector::new(-0.1, 0.0, 1.0))
        .expect("a plane")
}

/// The plane the half disc is tilted onto: through its chord `x = 0` on the
/// lid, leaning a tenth along `x`.
fn tilted_along_the_chord() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(-0.1, 0.0, 1.0))
        .expect("a plane")
}

fn the_lid() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), up()).expect("a plane")
}

fn the_lid_one_ulp_higher() -> Plane {
    the_lid()
        .moved_to(f64::from_bits(the_lid().offset().to_bits() + 1))
        .expect("an offset")
}

fn up() -> Vector {
    Vector::new(0.0, 0.0, 1.0)
}

/// A pipe of `radius` up the bar's axis.
fn pipe(radius: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(Axis::new(Point::origin(), up()).expect("an axis"), radius).expect("a pipe"),
    )
}

/// The lid split by a circle of radius 1 into an annulus on the lid's plane and
/// a disc on `disc`. The circle names `named`, which crosses the lid in it: the
/// pipe of radius 1 about the bar's axis, say.
///
/// A torus's circle round its axis is walked the other way from a pipe's, a
/// ball's or a cone's, so the hole walks it forwards there: walked as for the
/// others, the bar is refused `AClosedEdgeIsWalkedAgainstItsSurface`.
fn a_bar_with_its_lid_split_by_a_disc(
    named: Surface,
    disc: Plane,
    disc_first: bool,
) -> (Model, Result<SolidKey>) {
    a_bar_whose_lid_is_split(0.0, disc, disc_first, a_circle_naming(named))
}

/// The circle of radius 1 on the lid through `(1, 0, 4)`, naming `named`, and
/// the wires round the hole it leaves and round the disc inside it.
fn a_circle_naming(named: Surface) -> impl FnOnce(&mut SolidBuilder) -> (DraftWire, DraftWire) {
    move |builder| {
        let on_the_circle = builder
            .add_vertex(Point::new(1.0, 0.0, 4.0))
            .expect("a corner");
        let circle = builder
            .add_edge(
                on_the_circle,
                on_the_circle,
                EdgeRun::RightRoundWhereItCrosses { and: named },
            )
            .expect("an edge");
        let (round_the_hole, round_the_disc) = if matches!(named, Surface::Torus(_)) {
            (F, R)
        } else {
            (R, F)
        };
        (
            one_step_round(builder, circle, round_the_hole),
            one_step_round(builder, circle, round_the_disc),
        )
    }
}

/// The lid split by the half of the circle of radius 1 at `x ≥ 0` and the
/// chord `x = 0` across it, the half disc on `half`. The arc names `named`,
/// which crosses the lid in that circle, and is picked out by the half-space
/// `x ≥ 0`.
fn a_bar_with_its_lid_split_by_a_half_disc(
    named: Surface,
    half: Plane,
    half_first: bool,
) -> (Model, Result<SolidKey>) {
    a_bar_whose_lid_is_split(0.0, half, half_first, a_half_circle_and_its_chord(named))
}

/// The half of the circle of radius 1 on the lid at `x ≥ 0`, naming `named`
/// and picked out by the half-space `x ≥ 0`, and the chord `x = 0` across it.
fn a_half_circle_and_its_chord(
    named: Surface,
) -> impl FnOnce(&mut SolidBuilder) -> (DraftWire, DraftWire) {
    let east = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("a half-space");
    let run = EdgeRun::PartOfWhereItCrosses {
        and: named,
        picked_by: smallvec::smallvec![Picker::from(east)],
    };
    an_arc_and_its_chord(run, [Point::new(0.0, -1.0, 4.0), Point::new(0.0, 1.0, 4.0)])
}

/// The lid split by an arc that runs `run` between two corners on it and by
/// the chord back between them, the piece they close off on `inner`.
fn a_bar_with_its_lid_split_by_an_arc_and_its_chord(
    run: EdgeRun,
    [from, to]: [Point; 2],
    inner: Plane,
    inner_first: bool,
) -> (Model, Result<SolidKey>) {
    a_bar_whose_lid_is_split(
        0.0,
        inner,
        inner_first,
        an_arc_and_its_chord(run, [from, to]),
    )
}

/// An arc on the lid that runs `run` between two corners on it, the chord back
/// between them, and the wires round the hole they leave and round the piece
/// they close off.
fn an_arc_and_its_chord(
    run: EdgeRun,
    [from, to]: [Point; 2],
) -> impl FnOnce(&mut SolidBuilder) -> (DraftWire, DraftWire) {
    move |builder| {
        let south = builder.add_vertex(from).expect("a corner");
        let north = builder.add_vertex(to).expect("a corner");
        let arc = builder.add_edge(south, north, run).expect("an edge");
        let chord = builder
            .add_edge(north, south, EdgeRun::Straight)
            .expect("an edge");
        let round = |builder: &mut SolidBuilder, way| {
            let steps = [arc, chord]
                .map(|edge| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
            builder.add_wire(&steps).expect("a wire")
        };
        (round(builder, R), round(builder, F))
    }
}

/// A bar of radius 3 up `z` from `floor` to 4, its lid split in two by what
/// `split` adds: the wire it hands back first goes round the hole it leaves in
/// the lid, and the second round the piece inside, which is put on `inner`.
/// Every edge is walked the way Q386's probe found the sound bar wants.
fn a_bar_whose_lid_is_split(
    floor: f64,
    inner: Plane,
    inner_first: bool,
    split: impl FnOnce(&mut SolidBuilder) -> (DraftWire, DraftWire),
) -> (Model, Result<SolidKey>) {
    let base = Plane::from_point_and_normal(Point::new(0.0, 0.0, floor), -up()).expect("a plane");
    let mut builder = SolidBuilder::new();
    let at_the_bottom = builder
        .add_vertex(Point::new(3.0, 0.0, floor))
        .expect("a corner");
    let at_the_top = builder
        .add_vertex(Point::new(3.0, 0.0, 4.0))
        .expect("a corner");
    let bottom = builder
        .add_edge(
            at_the_bottom,
            at_the_bottom,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("an edge");
    let top = builder
        .add_edge(
            at_the_top,
            at_the_top,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("an edge");
    let seam = builder
        .add_edge(at_the_bottom, at_the_top, EdgeRun::Straight)
        .expect("an edge");

    let steps = [(bottom, F), (seam, F), (top, R), (seam, R)]
        .map(|(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
    let round = builder.add_wire(&steps).expect("a wire");
    let wall = builder.add_face(pipe(3.0), &[round]).expect("a face");
    let (hole, round_the_inner) = split(&mut builder);
    let rest_of_the_lid = |builder: &mut SolidBuilder| {
        let outer = one_step_round(builder, top, F);
        builder
            .add_face(Surface::Plane(the_lid()), &[outer, hole])
            .expect("a face")
    };
    let inside = |builder: &mut SolidBuilder| {
        builder
            .add_face(Surface::Plane(inner), &[round_the_inner])
            .expect("a face")
    };
    let (first, second) = if inner_first {
        let listed_first = inside(&mut builder);
        (listed_first, rest_of_the_lid(&mut builder))
    } else {
        let listed_first = rest_of_the_lid(&mut builder);
        (listed_first, inside(&mut builder))
    };
    let floor_round = one_step_round(&mut builder, bottom, R);
    let floor = builder
        .add_face(Surface::Plane(base), &[floor_round])
        .expect("a face");
    let faces: [DraftFace; 4] = [wall, first, second, floor];
    builder
        .add_shell(&faces.map(|face| (face, F)))
        .expect("a shell");
    let mut model = Model::new(Scale::millimetres(10.0).expect("a size"));
    let built = builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new());
    (model, built)
}

fn one_step_round(builder: &mut SolidBuilder, edge: DraftEdge, way: Orientation) -> DraftWire {
    let coedge = builder.add_coedge(edge, way).expect("a coedge");
    builder
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("a wire")
}

/// The tilted disc and the tilted half disc read back from bytes, each with the
/// name the sweep refuses it by.
fn the_two_tilted_pieces_read_back_from_bytes(inner_first: bool) -> [(Model, Invalid); 2] {
    let (model, built) = a_bar_with_its_lid_split_by_a_disc(pipe(1.0), the_lid(), inner_first);
    built.expect("the disc on the lid's own plane builds");
    [
        (
            with_the_inner_plane_swapped(&model, inner_first, tilted_through_the_corner()),
            Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes,
        ),
        (
            the_tilted_half_disc_read_back_from_bytes(inner_first),
            Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes,
        ),
    ]
}

/// The bar from `z = −1000` with the piece of its lid leaning `2e-12` off the
/// lid at the far side of its curve: the disc leaning through the circle's
/// corner `(1, 0, 4)`, and the half disc leaning along its chord.
fn the_tall_bars_leaning_off_the_lid(inner_first: bool) -> [(Model, Result<SolidKey>); 2] {
    [
        a_bar_whose_lid_is_split(
            -1000.0,
            leaning_off_the_lid_through(Point::new(1.0, 0.0, 4.0), 1e-12),
            inner_first,
            a_circle_naming(pipe(1.0)),
        ),
        a_bar_whose_lid_is_split(
            -1000.0,
            leaning_off_the_lid_through(Point::new(0.0, 0.0, 4.0), 2e-12),
            inner_first,
            a_half_circle_and_its_chord(pipe(1.0)),
        ),
    ]
}

/// A cube of side 10 whose floor is at `z = floor`, wide enough to hold the
/// bar's wall and tall enough to hold its lid.
fn a_block_whose_floor_is_at(floor: f64, model: &mut Model) -> SolidKey {
    let block = model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    let under_the_bar = Isometry::translation(Vector::new(-5.0, -5.0, floor)).expect("a movement");
    model
        .place_solid(block, &under_the_bar)
        .expect("the cube moves");
    block
}

/// The bar with its lid split by a half disc on the lid's own plane, which
/// builds, with the half disc's plane then swapped for the tilted one in the
/// bytes.
fn the_tilted_half_disc_read_back_from_bytes(half_first: bool) -> Model {
    let (model, built) = a_bar_with_its_lid_split_by_a_half_disc(pipe(1.0), the_lid(), half_first);
    built.expect("the half disc on the lid's own plane builds");
    with_the_inner_plane_swapped(&model, half_first, tilted_along_the_chord())
}

/// `model` written out with postcard, the inner piece's plane overwritten with
/// `tilted`, and read back.
///
/// The lid's plane is written twice, once for the rest of the lid and once
/// for the piece inside it, in the order the faces were described; which of
/// the two is the piece inside is `inner_first`.
fn with_the_inner_plane_swapped(model: &Model, inner_first: bool, tilted: Plane) -> Model {
    let mut bytes = postcard::to_allocvec(model).expect("a model writes out");
    let stored = the_bytes_of(the_lid());
    let places: Vec<usize> = bytes
        .windows(stored.len())
        .enumerate()
        .filter(|(_, window)| *window == stored.as_slice())
        .map(|(at, _)| at)
        .collect();
    let [one, the_other] = places[..] else {
        panic!("the lid's plane is written once per face on it, and it was found {places:?}");
    };
    let at = if inner_first { one } else { the_other };
    bytes[at..at + stored.len()].copy_from_slice(&the_bytes_of(tilted));
    postcard::from_bytes(&bytes).expect("reading back does not ask where a face is")
}

/// A plane the way postcard writes one: its four coefficients, little-endian.
fn the_bytes_of(plane: Plane) -> Vec<u8> {
    plane
        .coefficients()
        .into_iter()
        .flat_map(f64::to_le_bytes)
        .collect()
}

/// The tetrahedron [`a_tetrahedron_whose_upright_edge_runs`] builds, its
/// upright edge naming the pipe of radius 5 about the line through `(2, 1)` up
/// `z`, which holds it: `3² + 4² = 5²`. The plane `x = 5` crosses the pipe in
/// the lines at `y = 5` and `y = −3`, and `y = 5` in those at `x = 5` and
/// `x = −1`, so the edge is picked out by the half-space facing `(3, 4)` from
/// half way out to it, which holds `(5, 5)` and neither of the others.
fn a_tetrahedron_with_an_edge_down_a_pipe(model: &mut Model) -> Result<SolidKey> {
    let the_pipe = Cylinder::new(
        Axis::new(Point::new(2.0, 1.0, 0.0), up()).expect("an axis"),
        5.0,
    )
    .expect("a pipe");
    let towards_the_edge =
        HalfSpace::through_a_point_facing(Point::new(3.5, 3.0, 0.0), Vector::new(3.0, 4.0, 0.0))
            .expect("a half-space");
    let run = EdgeRun::PartOfWhereItCrosses {
        and: Surface::Cylinder(the_pipe),
        picked_by: smallvec::smallvec![Picker::from(towards_the_edge)],
    };
    a_tetrahedron_whose_upright_edge_runs(model, run, false)
}

/// The run along the line `x = y = 5` that names the cone with its apex at
/// `(5, 5, −5)`, about `(1, 0, 1)`, with slope 1.
///
/// That line is one of the cone's own: it leaves the apex straight up, at 45°
/// to an axis 45° off upright. `x = 5` holds the apex and rests along the cone
/// on that line alone, and `y = 5` holds the apex and cuts the cone in that line
/// and in `z = −5`. The run is picked out by the two half-spaces either side of
/// `x = 5`, which between them hold that plane and nothing else: the whole of the
/// one line, and of the other only the apex — the same pair `intersect` names a
/// line through a cone's apex with.
fn down_a_cones_ruling() -> EdgeRun {
    let axis = Axis::new(Point::new(5.0, 5.0, -5.0), Vector::new(1.0, 0.0, 1.0)).expect("an axis");
    let cone = Cone::new(axis, 1.0).expect("a cone");
    let east = HalfSpace::through_a_point_facing(cone.apex(), Vector::new(1.0, 0.0, 0.0))
        .expect("a half-space");
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Cone(cone),
        picked_by: smallvec::smallvec![Picker::from(east), Picker::from(east.reversed())],
    }
}

/// The tetrahedron with corners `(5, 5, 0)`, `(5, 5, 5)`, `(0, 5, 0)` and
/// `(5, 0, 0)`, whose upright edge between the first two runs down the line
/// where its faces on `y = 5` and `x = 5` cross, and runs `run` there. Every
/// other edge is straight. The face on `y = 5` is listed first unless `x_first`.
fn a_tetrahedron_whose_upright_edge_runs(
    model: &mut Model,
    run: EdgeRun,
    x_first: bool,
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let [a, b, c, d] = [
        Point::new(5.0, 5.0, 0.0),
        Point::new(5.0, 5.0, 5.0),
        Point::new(0.0, 5.0, 0.0),
        Point::new(5.0, 0.0, 0.0),
    ]
    .map(|at| builder.add_vertex(at).expect("a corner"));
    let upright = builder.add_edge(a, b, run).expect("an edge");
    let straight = |builder: &mut SolidBuilder, from, to| {
        builder
            .add_edge(from, to, EdgeRun::Straight)
            .expect("an edge")
    };
    let ac = straight(&mut builder, a, c);
    let ad = straight(&mut builder, a, d);
    let bc = straight(&mut builder, b, c);
    let bd = straight(&mut builder, b, d);
    let cd = straight(&mut builder, c, d);

    let face = |builder: &mut SolidBuilder, facing: Vector, through: Point, steps: [_; 3]| {
        let steps = steps
            .map(|(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
        let round = builder.add_wire(&steps).expect("a wire");
        let plane = Plane::from_point_and_normal(through, facing).expect("a plane");
        builder
            .add_face(Surface::Plane(plane), &[round])
            .expect("a face")
    };
    let on_y = |builder: &mut SolidBuilder| {
        face(
            builder,
            Vector::new(0.0, 1.0, 0.0),
            Point::new(0.0, 5.0, 0.0),
            [(ac, F), (bc, R), (upright, R)],
        )
    };
    let on_x = |builder: &mut SolidBuilder| {
        face(
            builder,
            Vector::new(1.0, 0.0, 0.0),
            Point::new(5.0, 0.0, 0.0),
            [(upright, F), (bd, F), (ad, R)],
        )
    };
    let [first, second] = if x_first {
        let listed_first = on_x(&mut builder);
        [listed_first, on_y(&mut builder)]
    } else {
        let listed_first = on_y(&mut builder);
        [listed_first, on_x(&mut builder)]
    };
    let faces = [
        first,
        second,
        face(
            &mut builder,
            Vector::new(0.0, 0.0, -1.0),
            Point::origin(),
            [(ad, F), (cd, R), (ac, R)],
        ),
        face(
            &mut builder,
            Vector::new(-1.0, -1.0, 1.0),
            Point::new(0.0, 5.0, 0.0),
            [(bc, F), (cd, F), (bd, R)],
        ),
    ];
    builder
        .add_shell(&faces.map(|face| (face, F)))
        .expect("a shell");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
