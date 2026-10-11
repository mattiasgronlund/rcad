//! Where a lid written a few ulps off stops being one plane spelt twice.
//!
//! The bar of radius 3 from `z = 0` to `4`, its lid split by the circle of
//! radius 1 into an annulus on the lid and a disc on a plane parallel to it, or
//! by half that circle and its chord into the rest of the lid and a half disc.
//! The circle is a closed edge between two flat faces and the half circle an
//! open one; each names the pipe of radius 1 to say where it goes. When the two
//! planes are proved different, Q574 and Q604 let the edge through only if the
//! curve on each plane reaches off the other by no more than
//! `what_writing_a_plane_down_costs` in `validate.rs`: sixteen times
//! `Plane::SIGNED_DISTANCE_ROUNDOFF`, which is `4ε`, times the largest
//! coordinate.
//!
//! **The largest coordinate is read off the corners**, not off the planes or the
//! surfaces (`largest_coordinate_in`), and every corner of this bar is where it
//! was put: `(3, 0, 0)`, `(3, 0, 4)`, and `(1, 0, 4)` or `(0, ±1, 4)`, whichever
//! plane the inner piece is on. So it is exactly 4, and the room is
//! `16 × 4ε × 4 = 256ε = 2⁻⁴⁴`, about `5.684e-14`.
//!
//! **The two planes are level, so the reach is the gap between them.** The
//! curve on one plane is off the other by the distance between the planes, and
//! nothing for the tilt. The corner is on the lid and off the inner plane by
//! that gap, which `Plane::signed_distance_to` works out with no rounding, since
//! it takes `4` from an offset within a factor of two of it.
//!
//! **Above the lid that room is 64 ulps, and below it 128.** An ulp of an
//! offset in `[4, 8)` is `2⁻⁵⁰`, and of one in `[2, 4)` it is `2⁻⁵¹`. So the
//! inner piece builds at 64 ulps above the lid and is refused at 65, builds at
//! 128 ulps below and is refused at 129. The same room counts as a different
//! number of ulps either side of 4, which is why there are pins for both.
//!
//! A refusal is asked of `SolidBuilder::build` and of `Model::validate`, the
//! second through a model that built on the lid's own plane and had its inner
//! plane swapped in the bytes, since a builder that refuses leaves nothing to
//! sweep. `docs/decisions.md` §336.3 and §15.6.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, DraftWire, Error, Invalid, Model, Result, SolidBuilder,
    SolidKey,
};

const F: Orientation = Orientation::Forward;
const R: Orientation = Orientation::Reversed;

/// The room `what_writing_a_plane_down_costs` gives this bar: sixteen times
/// `4ε` times the largest coordinate, 4. It is 64 ulps of 4, and the gap
/// between the lid and the plane 64 ulps above it is that room exactly.
#[test]
fn sixty_four_ulps_above_the_lid_is_exactly_what_writing_a_plane_down_costs() {
    let room = 16.0 * Plane::SIGNED_DISTANCE_ROUNDOFF * 4.0;

    assert_eq!(room, 64.0 * ulp_above(4.0));
    assert_eq!(the_lid_moved_by(64).offset() - the_lid().offset(), room);
    assert_eq!(the_lid().offset() - the_lid_moved_by(-128).offset(), room);
}

#[test]
fn a_disc_63_or_64_ulps_above_the_lid_builds_and_passes_the_sweep() {
    for ulps in [63, 64] {
        for inner_first in [false, true] {
            builds_and_passes_the_sweep(Split::Disc, ulps, inner_first);
        }
    }
}

#[test]
fn a_disc_65_ulps_above_the_lid_is_refused_by_the_builder_and_by_the_sweep() {
    for inner_first in [false, true] {
        is_refused_by_the_builder_and_by_the_sweep(Split::Disc, 65, inner_first);
    }
}

/// 65 ulps below the lid is only half as far as 65 above it, so it builds, and
/// the last that builds is 128 below.
#[test]
fn a_disc_65_or_128_ulps_below_the_lid_builds_and_passes_the_sweep() {
    for ulps in [-65, -128] {
        for inner_first in [false, true] {
            builds_and_passes_the_sweep(Split::Disc, ulps, inner_first);
        }
    }
}

#[test]
fn a_disc_129_ulps_below_the_lid_is_refused_by_the_builder_and_by_the_sweep() {
    for inner_first in [false, true] {
        is_refused_by_the_builder_and_by_the_sweep(Split::Disc, -129, inner_first);
    }
}

/// The open edge is asked against the same room as the closed one, so the half
/// disc stops building where the disc does, on both sides of the lid.
#[test]
fn a_half_disc_stops_building_where_the_disc_does() {
    for inner_first in [false, true] {
        for ulps in [64, -128] {
            builds_and_passes_the_sweep(Split::HalfDisc, ulps, inner_first);
        }
        for ulps in [65, -129] {
            is_refused_by_the_builder_and_by_the_sweep(Split::HalfDisc, ulps, inner_first);
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Split {
    Disc,
    HalfDisc,
}

fn builds_and_passes_the_sweep(split: Split, ulps: i64, inner_first: bool) {
    let (model, built) = the_bar(split, the_lid_moved_by(ulps), inner_first);
    built.unwrap_or_else(|why| {
        panic!("{split:?} {ulps} ulps off the lid, inner first {inner_first}: refused with {why:?}")
    });
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| {
            panic!(
                "{split:?} {ulps} ulps off the lid, inner first {inner_first}: built, and then \
                 the sweep refused it with {why:?}"
            )
        });
}

fn is_refused_by_the_builder_and_by_the_sweep(split: Split, ulps: i64, inner_first: bool) {
    let inner = the_lid_moved_by(ulps);
    let (_, built) = the_bar(split, inner, inner_first);
    assert!(
        is_the_refusal_for(split, &built),
        "{split:?} {ulps} ulps off the lid, inner first {inner_first}: the builder gave {built:?}"
    );

    let (on_the_lid, built) = the_bar(split, the_lid(), inner_first);
    built.expect("the inner piece on the lid's own plane builds");
    let swept = with_the_inner_plane_swapped(&on_the_lid, inner_first, inner)
        .validate(&Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        is_the_refusal_for(split, &swept),
        "{split:?} {ulps} ulps off the lid, inner first {inner_first}: the sweep gave {swept:?}"
    );
}

fn is_the_refusal_for<T>(split: Split, answer: &Result<T>) -> bool {
    match split {
        Split::Disc => matches!(
            answer,
            Err(Error::Invalid(
                Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
            ))
        ),
        Split::HalfDisc => matches!(
            answer,
            Err(Error::Invalid(
                Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes
            ))
        ),
    }
}

fn the_bar(split: Split, inner: Plane, inner_first: bool) -> (Model, Result<SolidKey>) {
    match split {
        Split::Disc => a_bar_with_its_lid_split_by_a_disc(inner, inner_first),
        Split::HalfDisc => a_bar_with_its_lid_split_by_a_half_disc(inner, inner_first),
    }
}

fn the_lid() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), up()).expect("a plane")
}

/// The lid with its offset `ulps` representable numbers up from 4, or down
/// for a negative count. The offset is positive, so up in its bits is up in `z`.
fn the_lid_moved_by(ulps: i64) -> Plane {
    let lid = the_lid();
    assert_eq!(lid.offset(), 4.0, "the lid is 4 up its own normal");
    let bits = lid
        .offset()
        .to_bits()
        .checked_add_signed(ulps)
        .expect("a few ulps from 4");
    lid.moved_to(f64::from_bits(bits)).expect("an offset")
}

fn ulp_above(of: f64) -> f64 {
    f64::from_bits(of.to_bits() + 1) - of
}

fn up() -> Vector {
    Vector::new(0.0, 0.0, 1.0)
}

/// The pipe of radius 1 up the bar's axis, which crosses the lid in the circle
/// that splits it.
fn the_pipe_the_circle_names() -> Surface {
    Surface::Cylinder(
        Cylinder::new(Axis::new(Point::origin(), up()).expect("an axis"), 1.0).expect("a pipe"),
    )
}

/// The lid split by the circle of radius 1 into an annulus on the lid's plane
/// and a disc on `disc`, the circle naming the pipe of radius 1.
fn a_bar_with_its_lid_split_by_a_disc(disc: Plane, disc_first: bool) -> (Model, Result<SolidKey>) {
    a_bar_whose_lid_is_split(disc, disc_first, |builder| {
        let on_the_circle = builder
            .add_vertex(Point::new(1.0, 0.0, 4.0))
            .expect("a corner");
        let circle = builder
            .add_edge(
                on_the_circle,
                on_the_circle,
                EdgeRun::RightRoundWhereItCrosses {
                    and: the_pipe_the_circle_names(),
                },
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
/// radius 1 and is picked out by the half-space `x ≥ 0`.
fn a_bar_with_its_lid_split_by_a_half_disc(
    half: Plane,
    half_first: bool,
) -> (Model, Result<SolidKey>) {
    let east = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("a half-space");
    let run = EdgeRun::PartOfWhereItCrosses {
        and: the_pipe_the_circle_names(),
        picked_by: smallvec::smallvec![Picker::from(east)],
    };
    a_bar_whose_lid_is_split(half, half_first, |builder| {
        let south = builder
            .add_vertex(Point::new(0.0, -1.0, 4.0))
            .expect("a corner");
        let north = builder
            .add_vertex(Point::new(0.0, 1.0, 4.0))
            .expect("a corner");
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
    })
}

/// A bar of radius 3 up `z` from 0 to 4, its lid split in two by what `split`
/// adds: the wire it hands back first goes round the hole it leaves in the
/// lid, and the second round the piece inside, which is put on `inner`.
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
    let wall_surface = Surface::Cylinder(
        Cylinder::new(Axis::new(Point::origin(), up()).expect("an axis"), 3.0).expect("a pipe"),
    );
    let wall = builder.add_face(wall_surface, &[round]).expect("a face");
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

/// `model` written out with postcard, the inner piece's plane overwritten with
/// `swapped_in`, and read back. `Deserialize` asks nothing about geometry.
///
/// The lid's plane is written twice, once for the rest of the lid and once for
/// the piece inside it, in the order the faces were described.
fn with_the_inner_plane_swapped(model: &Model, inner_first: bool, swapped_in: Plane) -> Model {
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
    bytes[at..at + stored.len()].copy_from_slice(&the_bytes_of(swapped_in));
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
