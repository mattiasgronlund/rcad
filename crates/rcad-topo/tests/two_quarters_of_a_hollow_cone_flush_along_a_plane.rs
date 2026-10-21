//! Two quarters of one hollow cone, flush along a plane (Q2537).
//!
//! Each quarter is the cone `r = z / 2` about `z`, cut to a quarter turn by a
//! prism on its axis and bored through by a bar on that axis, then turned about
//! `z`. One is turned by `start` and the other by `start - π/2`, so the two lie
//! side by side on one cone round one bore and touch along the plane at angle
//! `start`, apart only by how each turn rounds. Their intersection is that
//! plane: nothing, or a sliver no thicker than the rounding. Nothing is what
//! one start answers, and a sliver too thin for `f64` is what the others are
//! refused for (Q2551, Q2562). The same two quarters with no bore are refused
//! for the same slivers at the same starts (Q2543).

use core::f64::consts::FRAC_PI_2;
use core::ops::Range;

use rcad_geom::{Axis, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

/// The angles Q2355's lane found the refusal at, every one it tried.
const STARTS: [f64; 9] = [
    -2.0,
    -1.984,
    -2.4645,
    2.7745,
    -3.1 + 0.0155 * 117.0,
    -3.1 + 0.0155 * 58.0,
    -1.75,
    0.3,
    1.1,
];

/// The starts of [`STARTS`] at which the intersection answers, both ways round.
/// At each of the others the two quarters overlap in a sliver too thin to
/// answer, as [`at_every_other_start_they_overlap_in_a_sliver_too_thin_for_f64`]
/// says.
const STARTS_THAT_ANSWER: [f64; 1] = [1.1];

#[test]
fn two_quarters_of_a_cone_flush_along_a_plane_meet_in_nothing_or_a_sliver() {
    for quarters in [HOLLOW, UNBORED] {
        for start in STARTS_THAT_ANSWER {
            for the_first_first in [true, false] {
                let what = format!(
                    "{} quarters, start {start}, the first first {the_first_first}",
                    quarters.name
                );
                let measured = the_two_quarters_intersected(&quarters, start, the_first_first, 0.0)
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                for (volume, error) in measured {
                    assert!(
                        volume.abs() <= error,
                        "{what}: a sliver of {volume} past its error {error}"
                    );
                }
            }
        }
    }
}

/// At every other start the two side planes, which should be one, round about
/// `1e-16` radians into each other, and the two quarters overlap in the sliver
/// between them, from the bore, or the axis with no bore, to the cone and from
/// `z = 4.5` to `z = 8`. That sliver is too thin for `f64`, and its refusal is
/// a limit, not a gap (`docs/decisions.md` §16.10).
///
/// At six starts it is refused [`Invalid::DynamicRangeTooWide`], far past
/// [`rcad_geom::DynamicRange::LARGEST_SUPPORTED_RATIO`], at the ratios
/// [`Quarters::ratios_of_the_sliver`] names: the top at `8.5` over the
/// sliver's shortest arc, as for the wedge of
/// [`turned_a_trillionth_of_a_radian_further_the_wedge_is_too_thin_for_f64`].
/// §87.3 calls the range refusal of a sliver where two nearly flush faces meet
/// a limit. At [`STARTS_WHERE_TWO_ENDS_ARE_ONE_POINT`] a refusal comes first:
/// the sliver's arc on `z = 4.5` runs between the places where the two side
/// planes cross the cone there, and those are one `f64` triple. Worked out to
/// sixty digits from the two planes' normals as stored, they are `9.6e-18`
/// apart at `-2` and `4.6e-17` at `0.3`, and each coordinate of the one rounds
/// to the same `f64` as the other's. So no tighter solve would part them, and
/// [`Error::TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint`] is that variant's
/// limit. The bore does not reach that arc, so with no bore it is refused at
/// the same point.
///
/// Until Q2551 each of these was refused `a point inside a piece of a face`
/// first: a sliver of the first quarter's bottom cap at `z = 4` between the two
/// planes, or with the other quarter first one of the other's top cap at
/// `z = 8.5`, which no point inside was found for, and then the first
/// quarter's cone between the two planes from `z = 4` to `z = 4.5`, which the
/// other quarter's bottom cap cuts off. Before Q2537, a quarter's cone was
/// refused [`Error::CannotTellTheCellsOfAFaceApart`] at every start instead, in
/// the arrangement, before any piece was placed.
#[test]
fn at_every_other_start_they_overlap_in_a_sliver_too_thin_for_f64() {
    for start in STARTS {
        if STARTS_THAT_ANSWER.contains(&start) {
            continue;
        }
        for the_first_first in [true, false] {
            let what = format!("start {start}, the first first {the_first_first}");
            if STARTS_WHERE_TWO_ENDS_ARE_ONE_POINT.contains(&start) {
                let hollow = where_two_ends_round_to_one_point(&HOLLOW, start, the_first_first);
                let unbored = where_two_ends_round_to_one_point(&UNBORED, start, the_first_first);
                assert_eq!(hollow.z(), 4.5, "{what}: {hollow:?}");
                assert_eq!(unbored, hollow, "{what}: with no bore");
            } else {
                for quarters in [HOLLOW, UNBORED] {
                    let what = format!("{} quarters, {what}", quarters.name);
                    let intersected =
                        the_two_quarters_intersected(&quarters, start, the_first_first, 0.0);
                    let Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) = intersected
                    else {
                        panic!("{what}: {intersected:?}");
                    };
                    assert!(
                        quarters.ratios_of_the_sliver.contains(&ratio),
                        "{what}: a ratio of {ratio}"
                    );
                }
            }
        }
    }
}

/// The starts of [`STARTS`] where the sliver's arc on `z = 4.5` runs between
/// two places that are one `f64` triple.
const STARTS_WHERE_TWO_ENDS_ARE_ONE_POINT: [f64; 2] = [-2.0, 0.3];

fn where_two_ends_round_to_one_point(
    quarters: &Quarters,
    start: f64,
    the_first_first: bool,
) -> Point {
    let intersected = the_two_quarters_intersected(quarters, start, the_first_first, 0.0);
    let Err(Error::TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint { at, .. }) = intersected else {
        panic!(
            "{} quarters, start {start}, the first first {the_first_first}: {intersected:?}",
            quarters.name
        );
    };
    at
}

/// What refuses [`at_every_other_start_they_overlap_in_a_sliver_too_thin_for_f64`]
/// is the sliver's thinness and nothing else. Turned a billionth of a radian
/// further into the first quarter, the other overlaps it in a wedge of that
/// angle, and every start answers it, both ways round, with one solid of the
/// wedge's volume, bored or not.
#[test]
fn turned_a_billionth_of_a_radian_further_they_overlap_in_a_wedge_of_known_volume() {
    let further = 1e-9;
    for quarters in [HOLLOW, UNBORED] {
        for start in STARTS {
            for the_first_first in [true, false] {
                let what = format!(
                    "{} quarters, start {start}, the first first {the_first_first}",
                    quarters.name
                );
                let measured =
                    the_two_quarters_intersected(&quarters, start, the_first_first, further)
                        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                let [(volume, error)] = measured[..] else {
                    panic!("{what}: one wedge, got {measured:?}");
                };
                let expected = quarters.wedge_volume_per_radian * further;
                assert!(
                    (volume - expected).abs()
                        <= error + quarters.wedge_volume_per_radian * HOW_FAR_THE_TURNS_ROUND,
                    "{what}: a wedge of {volume}, not {expected}, error {error}"
                );
            }
        }
    }
}

/// How far the wedge's angle can be from the turn asked for. Each quarter's
/// turn is a number below `4.1` that rounds once or twice, by at most half of
/// its spacing, `4.4e-16`, and the sine and cosine each rotation is built from
/// round as well.
const HOW_FAR_THE_TURNS_ROUND: f64 = 2e-15;

/// Turned a trillionth of a radian further instead, the wedge's shortest arc
/// is a trillionth of [`Quarters::radius_of_the_shortest_arc`] long and its
/// top is at `8.5`, so it spans a ratio of about `8.5e12` with the bore and
/// `3.8e12` without, past
/// [`rcad_geom::DynamicRange::LARGEST_SUPPORTED_RATIO`], and every start
/// refuses it for the range, both ways round.
#[test]
fn turned_a_trillionth_of_a_radian_further_the_wedge_is_too_thin_for_f64() {
    let further = 1e-12;
    for quarters in [HOLLOW, UNBORED] {
        for start in STARTS {
            for the_first_first in [true, false] {
                let what = format!(
                    "{} quarters, start {start}, the first first {the_first_first}",
                    quarters.name
                );
                let intersected =
                    the_two_quarters_intersected(&quarters, start, the_first_first, further);
                let Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) = intersected
                else {
                    panic!("{what}: {intersected:?}");
                };
                let the_top_over_the_shortest_arc =
                    8.5 / (quarters.radius_of_the_shortest_arc * further);
                assert!(
                    (ratio / the_top_over_the_shortest_arc - 1.0).abs() < 0.01,
                    "{what}: a ratio of {ratio}"
                );
            }
        }
    }
}

/// One kind of quarter, and what the sliver and the wedge two of them overlap
/// in measure.
struct Quarters {
    name: &'static str,
    turned: fn(&mut Model, f64, f64) -> SolidKey,
    /// The ratios the sliver of
    /// [`at_every_other_start_they_overlap_in_a_sliver_too_thin_for_f64`] is
    /// refused at, a little wider than the ones measured.
    ratios_of_the_sliver: Range<f64>,
    /// The volume of the wedge between the two quarters' side planes, per
    /// radian of its angle: half the turn times the area between the bore, or
    /// the axis, and the cone, `r = z / 2`, from where the other quarter
    /// starts, `z = 4.5`, to where the first ends, `z = 8`.
    wedge_volume_per_radian: f64,
    /// The radius of the wedge's shortest arc: the bore's, or with no bore the
    /// cone's at `z = 4.5`.
    radius_of_the_shortest_arc: f64,
}

/// Measured `3.42e16` to `1.53e17`.
const HOLLOW: Quarters = Quarters {
    name: "hollow",
    turned: a_hollow_quarter_turned,
    ratios_of_the_sliver: 3.4e16..1.6e17,
    wedge_volume_per_radian: (8.0 * 8.0 * 8.0 - 4.5 * 4.5 * 4.5) / 24.0 - (8.0 - 4.5) / 2.0,
    radius_of_the_shortest_arc: 1.0,
};

/// Measured `1.27e16` to `7.66e16`.
const UNBORED: Quarters = Quarters {
    name: "unbored",
    turned: an_unbored_quarter_turned,
    ratios_of_the_sliver: 1.2e16..8e16,
    wedge_volume_per_radian: (8.0 * 8.0 * 8.0 - 4.5 * 4.5 * 4.5) / 24.0,
    radius_of_the_shortest_arc: 2.25,
};

/// The two quarters, the other turned `further` into the first, intersected.
fn the_two_quarters_intersected(
    quarters: &Quarters,
    start: f64,
    the_first_first: bool,
    further: f64,
) -> Result<Vec<(f64, f64)>, Error> {
    let mut model = a_model();
    let first = (quarters.turned)(&mut model, start, 4.0);
    let other = (quarters.turned)(&mut model, start - FRAC_PI_2 + further, 4.5);
    fused_and_measured(
        &mut model,
        first,
        other,
        Rule::Intersection,
        the_first_first,
    )
}

fn a_hollow_quarter_turned(model: &mut Model, turn: f64, near: f64) -> SolidKey {
    let quarter = a_quarter(model, near);
    let quarter = bored(model, quarter);
    turned(model, quarter, turn)
}

fn an_unbored_quarter_turned(model: &mut Model, turn: f64, near: f64) -> SolidKey {
    let quarter = a_quarter(model, near);
    turned(model, quarter, turn)
}

fn a_quarter(model: &mut Model, near: f64) -> SolidKey {
    let policy = Policy::certified();
    let frustum = a_frustum(model, near, near + 4.0);
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [12.0, 0.0], [0.0, 12.0]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&triangle, 6.0, &at_height(3.0))
        .expect("a prism");
    let answer = model
        .fuse(
            &[prism],
            &[frustum],
            Rule::Intersection,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a prism and a frustum on its corner answer");
    let &[quarter] = answer.solids() else {
        panic!("one quarter, got {:?}", answer.solids());
    };
    quarter
}

fn bored(model: &mut Model, quarter: SolidKey) -> SolidKey {
    let policy = Policy::certified();
    let bar = model
        .add_round_bar(1.0, 6.0, &at_height(3.0))
        .expect("a round bar");
    let answer = model
        .fuse(
            &[quarter],
            &[bar],
            Rule::Difference,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a quarter frustum less a bar along its axis answers");
    let &[quarter] = answer.solids() else {
        panic!("one hollow quarter, got {:?}", answer.solids());
    };
    quarter
}

fn turned(model: &mut Model, quarter: SolidKey, turn: f64) -> SolidKey {
    model
        .place_solid(
            quarter,
            &Isometry::rotation(&Axis::z(), turn).expect("a real angle"),
        )
        .expect("this model made the key");
    quarter
}

/// The cone `r = z / 2` about `z` between heights `near` and `far`, capped.
fn a_frustum(model: &mut Model, near: f64, far: f64) -> SolidKey {
    let slope = 0.5;
    let skin = rcad_geom::Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let at = |height: f64| Point::new(slope * height, 0.0, height);
    let bottom_corner = builder.add_vertex(at(near)).expect("a finite point");
    let top_corner = builder.add_vertex(at(far)).expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a circle");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a circle");
    let cap = |builder: &mut SolidBuilder, at: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
                .expect("a unit normal");
        let coedge = builder.add_coedge(edge, turn).expect("an edge");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap")
    };
    let bottom = cap(
        &mut builder,
        near,
        -1.0,
        bottom_circle,
        Orientation::Reversed,
    );
    let top = cap(&mut builder, far, 1.0, top_circle, Orientation::Forward);
    let round_the_wall = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| DraftStep::Along(builder.add_coedge(edge, turn).expect("an edge")));
    let wire = builder.add_wire(&round_the_wall).expect("a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[wire])
        .expect("a face on the cone");
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a frustum")
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn at_height(z: f64) -> Isometry {
    Isometry::translation(Vector::new(0.0, 0.0, z)).expect("a finite shift")
}

fn fused_and_measured(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    one_first: bool,
) -> Result<Vec<(f64, f64)>, Error> {
    let (arguments, tools) = if one_first {
        ([one], [other])
    } else {
        ([other], [one])
    };
    let policy = Policy::certified();
    let answer = model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())?;
    model.validate(&policy, &mut ProvenanceLog::new())?;
    let deflection = rcad_geom::Deflection::of(0.01).expect("a deflection");
    answer
        .solids()
        .iter()
        .map(|&solid| {
            let measured =
                model
                    .solid(solid)?
                    .measure(deflection, &policy, &mut ProvenanceLog::new())?;
            Ok((measured.volume(), measured.volume_error()))
        })
        .collect()
}
