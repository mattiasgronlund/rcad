//! A frustum whose wall is two bands, one naming the cone about `+z` and the
//! other about `−z`, described through the public builder.
//!
//! A cone here is both nappes, so the two spellings are one surface, and
//! `intersect` says so (§357). The rim between the two bands is one closed edge
//! with the cone either side of it, written once each way, and the run on it is
//! `RightRoundWhereItsFacesCross` — so it is walked by `rcad_geom::walk`'s arm
//! for one cone twice. **That arm asked whether the two held the same numbers**,
//! and refused this rim `ACrossingBetweenAConeAndAnotherCurvedSurface` in both
//! arrangements: the builder could not build the frustum at all.
//!
//! **The builder is the door.** A fuse that cuts a rim between two faces on one
//! cone written two ways names the plane that cut it
//! (`Assembly::the_surface_that_cut_each_closed_rim` folds the spelling), so it
//! does not write this run itself. Handed this frustum, united with a block
//! clear of it, the fuse joins the two bands into one wall about `−z` and the
//! rim is gone from its answer.
//!
//! # Which way the rim runs
//!
//! A closed edge runs right-handed about the axis of the curved surface along it
//! that sorts first, and for the rim that is the cone about `−z` whichever band
//! it bounds. So the rim runs clockwise seen from above in both arrangements,
//! and each cap circle runs right-handed about its own band's axis.
//!
//! # The volume
//!
//! The frustum from `z = 2` to `z = 6` of the cone with apex at the origin and
//! slope one half: caps of radius 1 and 3, so `π·4·(1 + 3 + 9)/3 = 52π/3`.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

const SLOPE: f64 = 0.5;
const BOTTOM: f64 = 2.0;
const RIM: f64 = 4.0;
const TOP: f64 = 6.0;

#[test]
fn a_frustum_banded_in_both_spellings_of_its_cone_builds_and_measures() {
    for (lower, upper, which) in arrangements() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let frustum = a_banded_frustum(&mut model, lower, upper)
            .unwrap_or_else(|why| panic!("{which}: refused {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{which}: invalid {why:?}"));
        assert_it_measures(&model, frustum, the_frustums_volume(), which);
    }
}

/// The frustum and a block standing clear of it, united: the answer is both
/// solids whole, and the frustum's wall comes back as one face in the spelling
/// that sorts first, with no rim across it.
#[test]
fn that_frustum_united_with_a_block_clear_of_it_keeps_its_volume() {
    for (lower, upper, which) in arrangements() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let frustum = a_banded_frustum(&mut model, lower, upper)
            .unwrap_or_else(|why| panic!("{which}: refused {why:?}"));
        let block = a_block_clear_of_the_frustum(&mut model);
        for (first, second, order) in [
            (frustum, block, "frustum first"),
            (block, frustum, "block first"),
        ] {
            let mut model = model.clone();
            let what = format!("{which}, {order}");
            let solids = model
                .fuse(
                    &[first],
                    &[second],
                    Rule::Union,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: refused {why:?}"))
                .solids()
                .to_vec();
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: invalid {why:?}"));
            assert_eq!(solids.len(), 2, "{what}: the two never touch");
            let mut cone_faces: Vec<Surface> = solids
                .iter()
                .flat_map(|solid| {
                    model
                        .solid(*solid)
                        .expect("this model made the key")
                        .faces()
                        .map(|face| face.surface())
                        .filter(|surface| matches!(surface, Surface::Cone(_)))
                        .collect::<Vec<_>>()
                })
                .collect();
            cone_faces.dedup();
            assert_eq!(
                cone_faces,
                [Surface::Cone(
                    Cone::new(Axis::z().reversed(), SLOPE).expect("a positive slope")
                )],
                "{what}: the fuse joins the two bands into one wall, in the spelling that sorts \
                 first, so the rim between them is gone from the answer"
            );
            let wanted = [
                the_frustums_volume(),
                BLOCK_SIDE * BLOCK_SIDE * BLOCK_HEIGHT,
            ];
            let mut measured: Vec<f64> = solids
                .iter()
                .map(|solid| assert_it_measures_one_of(&model, *solid, wanted, &what))
                .collect();
            measured.sort_by(f64::total_cmp);
            assert!(
                (measured[0] - wanted[1]).abs() < 1e-9,
                "{what}: one of the two answers is the block, {measured:?}"
            );
        }
    }
}

fn arrangements() -> [(Axis, Axis, &'static str); 2] {
    [
        (Axis::z(), Axis::z().reversed(), "lower band about +z"),
        (Axis::z().reversed(), Axis::z(), "lower band about -z"),
    ]
}

/// `|volume − want|` no more than the error `measure` publishes with it.
fn assert_it_measures(model: &Model, solid: SolidKey, want: f64, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: does not measure: {why:?}"));
    assert!(
        (measured.volume() - want).abs() <= measured.volume_error(),
        "{what} is {} against {want}, and the mesh may miss by {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The volume of whichever of `wanted` this solid measures inside its bound.
fn assert_it_measures_one_of(model: &Model, solid: SolidKey, wanted: [f64; 2], what: &str) -> f64 {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: does not measure: {why:?}"));
    let Some(want) = wanted
        .into_iter()
        .find(|want| (measured.volume() - want).abs() <= measured.volume_error())
    else {
        panic!(
            "{what}: a solid of {} ± {} is neither the frustum nor the block of {wanted:?}",
            measured.volume(),
            measured.volume_error()
        );
    };
    want
}

/// `π·h·(r₁² + r₁r₂ + r₂²)/3`, with each cap's radius half its height.
fn the_frustums_volume() -> f64 {
    let (low, high) = (SLOPE * BOTTOM, SLOPE * TOP);
    core::f64::consts::PI * (TOP - BOTTOM) * (low * low + low * high + high * high) / 3.0
}

const BLOCK_SIDE: f64 = 2.0;
const BLOCK_HEIGHT: f64 = 4.0;

/// A block from `(4, 4, 2)`, whose nearest upright edge is `4√2` from the axis
/// where the frustum is never wider than 3.
fn a_block_clear_of_the_frustum(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [BLOCK_SIDE, 0.0],
        [BLOCK_SIDE, BLOCK_SIDE],
        [0.0, BLOCK_SIDE],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square is a profile");
    model
        .add_extrusion(
            &square,
            BLOCK_HEIGHT,
            &Isometry::translation(Vector::new(4.0, 4.0, BOTTOM)).expect("a finite shift"),
        )
        .expect("a block is a solid")
}

/// The frustum from `z = 2` to `z = 6`, its wall split level at `z = 4` into a
/// lower band naming the cone about `lower` and an upper band naming it about
/// `upper`.
fn a_banded_frustum(model: &mut Model, lower: Axis, upper: Axis) -> rcad_topo::Result<SolidKey> {
    let [lower_skin, upper_skin] =
        [lower, upper].map(|axis| Cone::new(axis, SLOPE).expect("a positive slope"));
    let the_rims_spelling = if Surface::Cone(lower_skin)
        .how_it_sorts_against(&Surface::Cone(upper_skin))
        .is_le()
    {
        lower
    } else {
        upper
    };
    let mut builder = SolidBuilder::new();
    let at = |height: f64| Point::new(SLOPE * height, 0.0, height);
    let bottom_corner = builder.add_vertex(at(BOTTOM))?;
    let rim_corner = builder.add_vertex(at(RIM))?;
    let top_corner = builder.add_vertex(at(TOP))?;
    let lower_seam = builder.add_edge(bottom_corner, rim_corner, EdgeRun::Straight)?;
    let upper_seam = builder.add_edge(rim_corner, top_corner, EdgeRun::Straight)?;
    let [bottom_circle, rim, top_circle] = [bottom_corner, rim_corner, top_corner]
        .map(|corner| builder.add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross));
    let (bottom_circle, rim, top_circle) = (bottom_circle?, rim?, top_circle?);

    let bottom = a_cap(
        &mut builder,
        BOTTOM,
        -1.0,
        bottom_circle,
        turned_for(lower, Orientation::Reversed),
    )?;
    let top = a_cap(
        &mut builder,
        TOP,
        1.0,
        top_circle,
        turned_for(upper, Orientation::Forward),
    )?;
    let lower_band = a_band(
        &mut builder,
        lower_skin,
        (bottom_circle, turned_for(lower, Orientation::Forward)),
        lower_seam,
        (rim, turned_for(the_rims_spelling, Orientation::Reversed)),
    )?;
    let upper_band = a_band(
        &mut builder,
        upper_skin,
        (rim, turned_for(the_rims_spelling, Orientation::Forward)),
        upper_seam,
        (top_circle, turned_for(upper, Orientation::Reversed)),
    )?;
    builder.add_shell(&[
        (bottom, Orientation::Forward),
        (top, Orientation::Forward),
        (lower_band, Orientation::Forward),
        (upper_band, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// How a coedge on a circle is used, given how it would be used if the circle
/// ran right-handed about `+z`: turned round when the surface that says which
/// way the circle runs names the cone about `−z`.
fn turned_for(axis: Axis, about_up: Orientation) -> Orientation {
    if axis.direction().components()[2] < 0.0 {
        about_up.reversed()
    } else {
        about_up
    }
}

fn a_cap(
    builder: &mut SolidBuilder,
    at: f64,
    outward: f64,
    edge: DraftEdge,
    turn: Orientation,
) -> rcad_topo::Result<DraftFace> {
    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
            .expect("the z axis is a unit normal");
    let coedge = builder.add_coedge(edge, turn)?;
    let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
    builder.add_face(Surface::Plane(plane), &[wire])
}

/// One band of the wall: round its lower circle, up the seam, back round its
/// upper circle, down the seam.
fn a_band(
    builder: &mut SolidBuilder,
    skin: Cone,
    lower: (DraftEdge, Orientation),
    seam: DraftEdge,
    upper: (DraftEdge, Orientation),
) -> rcad_topo::Result<DraftFace> {
    let mut steps = Vec::new();
    for (edge, turn) in [
        lower,
        (seam, Orientation::Forward),
        upper,
        (seam, Orientation::Reversed),
    ] {
        steps.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    let wire = builder.add_wire(&steps)?;
    builder.add_face(Surface::Cone(skin), &[wire])
}
