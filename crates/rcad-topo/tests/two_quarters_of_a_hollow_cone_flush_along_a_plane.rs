//! Two quarters of one hollow cone, flush along a plane (Q2537).
//!
//! Each quarter is the cone `r = z / 2` about `z`, cut to a quarter turn by a
//! prism on its axis and bored through by a bar on that axis, then turned about
//! `z`. One is turned by `start` and the other by `start - π/2`, so the two lie
//! side by side on one cone round one bore and touch along the plane at angle
//! `start`, apart only by how each turn rounds. Their intersection is that
//! plane: nothing, or a sliver no thicker than the rounding.

use core::f64::consts::FRAC_PI_2;

use rcad_geom::{Axis, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

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

/// At every start but 1.1 the other quarter's side plane, an ulp off the first
/// quarter's own, cuts a sliver off the first quarter's bottom cap at `z = 4`,
/// between the two planes, from the bore to the cone. The other quarter starts
/// at `z = 4.5`, so the sliver is plainly outside it. But no point inside a
/// sliver an ulp wide is found, and each of its four corners lies on the cone
/// or the bore the two quarters share. With the other quarter first, the
/// sliver is one of the other's top cap at `z = 8.5` instead, above the first
/// quarter's top at `z = 8`. Each was refused `a point inside a piece of a
/// face` until Q2551, at every start but 1.1, both ways round.
///
/// Before Q2537, a quarter's cone was refused
/// [`Error::CannotTellTheCellsOfAFaceApart`] at every start instead, in the
/// arrangement, before any piece was placed.
#[test]
fn two_hollow_quarters_of_a_cone_flush_along_a_plane_meet_in_nothing_or_a_sliver() {
    for start in STARTS {
        for the_first_first in [true, false] {
            let what = format!("start {start}, the first first {the_first_first}");
            let measured = the_two_quarters_intersected(start, the_first_first)
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

fn the_two_quarters_intersected(
    start: f64,
    the_first_first: bool,
) -> Result<Vec<(f64, f64)>, Error> {
    let mut model = a_model();
    let first = a_hollow_quarter_turned(&mut model, start, 4.0);
    let other = a_hollow_quarter_turned(&mut model, start - FRAC_PI_2, 4.5);
    fused_and_measured(
        &mut model,
        first,
        other,
        Rule::Intersection,
        the_first_first,
    )
}

fn a_hollow_quarter_turned(model: &mut Model, turn: f64, near: f64) -> SolidKey {
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
