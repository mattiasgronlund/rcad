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
//! naming the pipe: the arc there is that line, and on both planes.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, DraftWire, Error, Invalid, Model, Result, SolidBuilder,
    SolidKey,
};

const F: Orientation = Orientation::Forward;
const R: Orientation = Orientation::Reversed;

#[test]
fn a_disc_tilted_off_the_lid_is_refused_whichever_face_is_listed_first() {
    for disc_first in [false, true] {
        let (_, built) =
            a_bar_with_its_lid_split_by_a_disc(tilted_through_the_corner(), disc_first);

        assert!(
            matches!(
                built,
                Err(Error::Invalid(
                    Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ),
            "disc first {disc_first}: the builder gave {built:?}"
        );
    }
}

/// The same refusal from the sweep, for a model that never went through a
/// builder: the sound bar written out, its disc's plane swapped for the tilted
/// one in the bytes, and read back. `Deserialize` asks nothing about geometry,
/// so this is the route a model that holds the shape takes.
#[test]
fn the_sweep_refuses_the_tilted_disc_read_back_from_bytes() {
    for disc_first in [false, true] {
        let (model, built) = a_bar_with_its_lid_split_by_a_disc(the_lid(), disc_first);
        built.expect("the disc on the lid's own plane builds");
        let tilted =
            with_the_inner_plane_swapped(&model, disc_first, tilted_through_the_corner());

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
            let (model, built) = a_bar_with_its_lid_split_by_a_disc(disc, disc_first);

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
        let (_, built) =
            a_bar_with_its_lid_split_by_a_half_disc(tilted_along_the_chord(), half_first);

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

/// Why the tilted half disc is refused rather than drawn: the volume of the
/// model that holds it depends on which of its two lid faces is listed first,
/// by more than the mesh says it may miss by either way. If this stops
/// holding, the refusal's reason in `Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes`
/// has gone stale.
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

    assert!(
        (lid_first - half_first).abs() > lid_first_error + half_first_error,
        "the lid first measured {lid_first} ± {lid_first_error} and the half disc first \
         {half_first} ± {half_first_error}"
    );
}

#[test]
fn a_half_disc_on_the_lids_own_plane_still_builds_and_validates() {
    for half in [the_lid(), the_lid_one_ulp_higher()] {
        for half_first in [false, true] {
            let (model, built) = a_bar_with_its_lid_split_by_a_half_disc(half, half_first);

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
        Cylinder::new(Axis::new(Point::origin(), up()).expect("an axis"), radius)
            .expect("a pipe"),
    )
}

/// The lid split by a circle of radius 1 into an annulus on the lid's plane and
/// a disc on `disc`. The circle names the pipe of radius 1 about the bar's
/// axis.
fn a_bar_with_its_lid_split_by_a_disc(disc: Plane, disc_first: bool) -> (Model, Result<SolidKey>) {
    a_bar_whose_lid_is_split(disc, disc_first, |builder| {
        let on_the_circle = builder
            .add_vertex(Point::new(1.0, 0.0, 4.0))
            .expect("a corner");
        let circle = builder
            .add_edge(
                on_the_circle,
                on_the_circle,
                EdgeRun::RightRoundWhereItCrosses { and: pipe(1.0) },
            )
            .expect("an edge");
        (
            one_step_round(builder, circle, R),
            one_step_round(builder, circle, F),
        )
    })
}

/// The lid split by the half of the circle of radius 1 at `x ≥ 0` and the
/// chord `x = 0` across it, the half disc on `half`. The arc names the pipe of
/// radius 1 about the bar's axis and is picked out by the half-space `x ≥ 0`.
fn a_bar_with_its_lid_split_by_a_half_disc(
    half: Plane,
    half_first: bool,
) -> (Model, Result<SolidKey>) {
    a_bar_whose_lid_is_split(half, half_first, |builder| {
        let south = builder
            .add_vertex(Point::new(0.0, -1.0, 4.0))
            .expect("a corner");
        let north = builder
            .add_vertex(Point::new(0.0, 1.0, 4.0))
            .expect("a corner");
        let east = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
            .expect("a half-space");
        let arc = builder
            .add_edge(
                south,
                north,
                EdgeRun::PartOfWhereItCrosses {
                    and: pipe(1.0),
                    picked_by: smallvec::smallvec![Picker::from(east)],
                },
            )
            .expect("an edge");
        let chord = builder
            .add_edge(north, south, EdgeRun::Straight)
            .expect("an edge");
        let round = |builder: &mut SolidBuilder, way| {
            let steps = [arc, chord]
                .map(|edge| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
            builder.add_wire(&steps).expect("a wire")
        };
        (round(builder, R), round(builder, F))
    })
}

/// A bar of radius 3 up `z` from 0 to 4, its lid split in two by what `split`
/// adds: the wire it hands back first goes round the hole it leaves in the
/// lid, and the second round the piece inside, which is put on `inner`. Every
/// edge is walked the way Q386's probe found the sound bar wants.
fn a_bar_whose_lid_is_split(
    inner: Plane,
    inner_first: bool,
    split: impl FnOnce(&mut SolidBuilder) -> (DraftWire, DraftWire),
) -> (Model, Result<SolidKey>) {
    let base = Plane::from_point_and_normal(Point::origin(), -up()).expect("a plane");
    let mut builder = SolidBuilder::new();
    let at_the_bottom = builder
        .add_vertex(Point::new(3.0, 0.0, 0.0))
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

/// The bar with its lid split by a half disc on the lid's own plane, which
/// builds, with the half disc's plane then swapped for the tilted one in the
/// bytes.
fn the_tilted_half_disc_read_back_from_bytes(half_first: bool) -> Model {
    let (model, built) = a_bar_with_its_lid_split_by_a_half_disc(the_lid(), half_first);
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

/// The tetrahedron with corners `(5, 5, 0)`, `(5, 5, 5)`, `(0, 5, 0)` and
/// `(5, 0, 0)`, whose edge between the first two runs down the line where its
/// faces on `y = 5` and `x = 5` cross.
///
/// That edge names the pipe of radius 5 about the line through `(2, 1)` up
/// `z`, which holds it: `3² + 4² = 5²`. The plane `x = 5` crosses the pipe in
/// the lines at `y = 5` and `y = −3`, and `y = 5` in those at `x = 5` and
/// `x = −1`, so the edge is picked out by the half-space facing `(3, 4)` from
/// half way out to it, which holds `(5, 5)` and neither of the others. Every
/// other edge is straight.
fn a_tetrahedron_with_an_edge_down_a_pipe(model: &mut Model) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let [a, b, c, d] = [
        Point::new(5.0, 5.0, 0.0),
        Point::new(5.0, 5.0, 5.0),
        Point::new(0.0, 5.0, 0.0),
        Point::new(5.0, 0.0, 0.0),
    ]
    .map(|at| builder.add_vertex(at).expect("a corner"));
    let the_pipe = Cylinder::new(
        Axis::new(Point::new(2.0, 1.0, 0.0), up()).expect("an axis"),
        5.0,
    )
    .expect("a pipe");
    let towards_the_edge =
        HalfSpace::through_a_point_facing(Point::new(3.5, 3.0, 0.0), Vector::new(3.0, 4.0, 0.0))
            .expect("a half-space");
    let down_the_pipe = builder
        .add_edge(
            a,
            b,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Cylinder(the_pipe),
                picked_by: smallvec::smallvec![Picker::from(towards_the_edge)],
            },
        )
        .expect("an edge");
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
        let steps = steps.map(|(edge, way)| {
            DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge"))
        });
        let round = builder.add_wire(&steps).expect("a wire");
        let plane = Plane::from_point_and_normal(through, facing).expect("a plane");
        builder
            .add_face(Surface::Plane(plane), &[round])
            .expect("a face")
    };
    let faces = [
        face(
            &mut builder,
            Vector::new(0.0, 1.0, 0.0),
            Point::new(0.0, 5.0, 0.0),
            [(ac, F), (bc, R), (down_the_pipe, R)],
        ),
        face(
            &mut builder,
            Vector::new(1.0, 0.0, 0.0),
            Point::new(5.0, 0.0, 0.0),
            [(down_the_pipe, F), (bd, F), (ad, R)],
        ),
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
