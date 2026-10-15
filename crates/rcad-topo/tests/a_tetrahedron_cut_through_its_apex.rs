//! A tetrahedron cut by a block whose face runs through its apex (queue row
//! Q1839, `docs/decisions.md` §294.22).
//!
//! The tetrahedron has its apex at `(0, 0, 10)` over the base `(−3, −3, 0)`,
//! `(3, −3, 0)`, `(0, 3, 0)`, and is built face by face with `SolidBuilder`,
//! each face's plane from `Plane::through_three_points`. The block runs `x`
//! from −5 to 5, `y` from 0 to 5 and `z` from −1 to 11, so its face `y = 0`
//! holds the apex, and cuts the tetrahedron into 15 on the block's side and 45
//! off it. Both are placed by one of the four placements of §294.8.
//!
//! **The front face misses the apex.** `through_three_points` builds the face
//! facing `−y` through the apex along `(0, −10, 3)` scaled to length one, and
//! `10 × 0.28734788556634544` rounds, so the stored plane is a rounding off
//! the apex. The fuse reads the apex as where the three side planes meet, and
//! that is `5.8e−17` above the cut. So the numbers the kernel is handed have a
//! triangle about `1e−16` across at the top of the front face, above the cut,
//! and the half that holds it holds a feature `f64` cannot hold beside
//! coordinates near 10. A front face built through the apex exactly answers.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{
    DraftEdge, DraftStep, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

/// The front face's plane, and the two side planes, at the stored apex: the
/// front face is proved off it, and a front face built along `(0, −10, 3)` at
/// a length that keeps every product exact holds the apex and both corners of
/// the base in front of it.
#[test]
fn the_front_face_through_three_points_is_stored_a_rounding_off_the_apex() {
    let mut model = a_model();
    let tetrahedron = a_tetrahedron(&mut model, Front::ThroughThreePoints);
    let [front, right, left] = the_side_planes(&model, tetrahedron, &Isometry::identity());
    let apex = rcad_predicates::Point::from(APEX);
    assert_eq!(front.side_of_point(apex), Decision::Pos);
    assert_eq!(right.side_of_point(apex), Decision::Zero);
    assert_eq!(left.side_of_point(apex), Decision::Zero);

    let mut model = a_model();
    let tetrahedron = a_tetrahedron(&mut model, Front::ThroughTheApexExactly);
    let [front, ..] = the_side_planes(&model, tetrahedron, &Isometry::identity());
    for corner in [APEX, [-3.0, -3.0, 0.0], [3.0, -3.0, 0.0]] {
        assert_eq!(
            front.side_of_point(rcad_predicates::Point::from(corner)),
            Decision::Zero,
            "{corner:?}"
        );
    }
}

/// **Which side of the cut the apex is on, as the fuse reads it**: where the
/// three side planes meet, against the block's face `y = 0`, whose outside is
/// `−y`. At every placement it is off the cut, so the cut does not run through
/// the apex in the numbers the kernel is handed. `Neg` is inside the block, so
/// the intersection holds the speck at the apex; `Pos` is outside, so the
/// difference holds it. The half that holds it is refused by the range check
/// at every placement (the pins below). The other half has the cut's top
/// edge, a rounding long, and answers where both ends of that edge are
/// written as one `f64` triple. Shifted, they are an ulp apart, and that half
/// is refused too.
///
/// Built through the apex exactly and left where it was built, the four
/// planes meet at one point.
#[test]
fn where_the_three_side_planes_meet_is_off_the_cut_at_every_placement() {
    for (placement, side) in [
        (Placement::WhereItWasBuilt, Decision::Neg),
        (Placement::Shifted, Decision::Neg),
        (Placement::Turned, Decision::Pos),
        (Placement::TurnedThenShifted, Decision::Pos),
    ] {
        assert_eq!(
            the_side_of_the_cut_the_apex_is_on(Front::ThroughThreePoints, placement),
            side,
            "{placement:?}"
        );
    }
    assert_eq!(
        the_side_of_the_cut_the_apex_is_on(
            Front::ThroughTheApexExactly,
            Placement::WhereItWasBuilt
        ),
        Decision::Zero
    );
}

#[test]
fn unplaced_less_the_block_answers_45_under_certified() {
    answers(
        Placement::WhereItWasBuilt,
        Rule::Difference,
        &Policy::certified(),
    );
}

#[test]
fn unplaced_less_the_block_answers_45_under_tolerant() {
    answers(
        Placement::WhereItWasBuilt,
        Rule::Difference,
        &Policy::tolerant(100.0),
    );
}

#[test]
fn unplaced_and_the_block_is_refused_under_certified() {
    is_refused(
        Placement::WhereItWasBuilt,
        Rule::Intersection,
        4.474687036223253e16,
    );
}

#[test]
fn unplaced_and_the_block_answers_15_under_tolerant() {
    answers(
        Placement::WhereItWasBuilt,
        Rule::Intersection,
        &Policy::tolerant(100.0),
    );
}

#[test]
fn shifted_less_the_block_is_refused_under_certified() {
    is_refused(Placement::Shifted, Rule::Difference, 1.5199648742375424e16);
}

#[test]
fn shifted_less_the_block_answers_45_under_tolerant() {
    answers(
        Placement::Shifted,
        Rule::Difference,
        &Policy::tolerant(100.0),
    );
}

#[test]
fn shifted_and_the_block_is_refused_under_certified() {
    is_refused(
        Placement::Shifted,
        Rule::Intersection,
        1.5199648742375424e16,
    );
}

#[test]
fn shifted_and_the_block_answers_15_under_tolerant() {
    answers(
        Placement::Shifted,
        Rule::Intersection,
        &Policy::tolerant(100.0),
    );
}

#[test]
fn turned_less_the_block_is_refused_under_certified() {
    is_refused(Placement::Turned, Rule::Difference, 3_183_396_951_313_399.0);
}

#[test]
fn turned_less_the_block_answers_45_under_tolerant() {
    answers(
        Placement::Turned,
        Rule::Difference,
        &Policy::tolerant(100.0),
    );
}

#[test]
fn turned_and_the_block_answers_15_under_certified() {
    answers(Placement::Turned, Rule::Intersection, &Policy::certified());
}

#[test]
fn turned_and_the_block_answers_15_under_tolerant() {
    answers(
        Placement::Turned,
        Rule::Intersection,
        &Policy::tolerant(100.0),
    );
}

#[test]
fn turned_then_shifted_less_the_block_is_refused_under_certified() {
    is_refused(
        Placement::TurnedThenShifted,
        Rule::Difference,
        1_281_735_828_939_219.8,
    );
}

#[test]
fn turned_then_shifted_less_the_block_answers_45_under_tolerant() {
    answers(
        Placement::TurnedThenShifted,
        Rule::Difference,
        &Policy::tolerant(100.0),
    );
}

#[test]
fn turned_then_shifted_and_the_block_answers_15_under_certified() {
    answers(
        Placement::TurnedThenShifted,
        Rule::Intersection,
        &Policy::certified(),
    );
}

#[test]
fn turned_then_shifted_and_the_block_answers_15_under_tolerant() {
    answers(
        Placement::TurnedThenShifted,
        Rule::Intersection,
        &Policy::tolerant(100.0),
    );
}

/// **The caller's road.** With the front face built through the apex
/// exactly, the two fused where they were built and the answer placed after
/// answer at every placement, under `certified`, with nothing in the log.
#[test]
fn a_front_face_through_the_apex_fused_where_built_and_then_placed_answers_at_every_placement() {
    let certified = Policy::certified();
    for placement in Placement::EVERY {
        for rule in [Rule::Difference, Rule::Intersection] {
            let case = format!("{placement:?}, {rule:?}");
            let mut model = a_model();
            let tetrahedron = a_tetrahedron(&mut model, Front::ThroughTheApexExactly);
            let block = a_block(&mut model, &Isometry::identity());
            let mut log = ProvenanceLog::new();
            let answer = model
                .fuse(&[tetrahedron], &[block], rule, &certified, &mut log)
                .unwrap_or_else(|error| panic!("{case}: {error:?}"));
            for &solid in answer.solids() {
                model
                    .place_solid(solid, &placement.movement())
                    .expect("the fuse made the key");
            }
            assert!(log.records().is_empty(), "{case}");
            the_answer_holds_its_half(&model, answer.solids(), rule, &certified, &case);
        }
    }
}

/// Placed first, the front face built through the apex exactly answers where
/// it stands, and is refused where the placement rounds each plane again, as
/// §448.7 reads the pyramid of §294.21. So the shifted and turned refusals do
/// not come from the front face's rounding but from the placement's.
#[test]
fn a_front_face_through_the_apex_placed_first_is_refused_where_the_placement_rounds() {
    let certified = Policy::certified();
    for (placement, refused) in [
        (Placement::WhereItWasBuilt, [false, false]),
        (Placement::Shifted, [true, true]),
        (Placement::Turned, [true, false]),
        (Placement::TurnedThenShifted, [true, false]),
    ] {
        for (rule, refused) in [Rule::Difference, Rule::Intersection]
            .into_iter()
            .zip(refused)
        {
            let case = format!("{placement:?}, {rule:?}");
            let mut model = a_model();
            let tetrahedron = a_tetrahedron(&mut model, Front::ThroughTheApexExactly);
            model
                .place_solid(tetrahedron, &placement.movement())
                .expect("this model made the key");
            let block = a_block(&mut model, &placement.movement());
            let mut log = ProvenanceLog::new();
            match model.fuse(&[tetrahedron], &[block], rule, &certified, &mut log) {
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if refused => {
                    assert!(ratio > 1e15, "{case}: {ratio}");
                }
                Ok(answer) if !refused => {
                    assert!(log.records().is_empty(), "{case}");
                    the_answer_holds_its_half(&model, answer.solids(), rule, &certified, &case);
                }
                other => panic!("{case}: {other:?}"),
            }
        }
    }
}

/// The fuse of the tetrahedron and the block, placed, answers one solid that
/// holds its half and validates. Under `tolerant` its log holds one record,
/// where the corners of the speck at the apex are taken as the apex.
fn answers(placement: Placement, rule: Rule, policy: &Policy) {
    let case = format!("{placement:?}, {rule:?}, {policy:?}");
    let mut log = ProvenanceLog::new();
    let (model, answer) = the_fuse(placement, rule, policy, &mut log);
    let answer = answer.unwrap_or_else(|error| panic!("{case}: {error:?}"));
    let records: Vec<&str> = log
        .records()
        .iter()
        .map(|record| record.predicate())
        .collect();
    let expected: &[&str] = if *policy == Policy::certified() {
        &[]
    } else {
        &["three meets of one plane taken as an input corner"]
    };
    assert_eq!(records, expected, "{case}");
    the_answer_holds_its_half(&model, &answer, rule, policy, &case);
}

/// The fuse is refused on the range of its answer, at `ratio` to `1e−9`.
fn is_refused(placement: Placement, rule: Rule, ratio: f64) {
    let case = format!("{placement:?}, {rule:?}");
    let (_, answer) = the_fuse(
        placement,
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    match answer {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio: found })) => {
            assert!(
                (found - ratio).abs() <= 1e-9 * ratio,
                "{case}: {found:e} for {ratio:e}"
            );
        }
        other => panic!("{case}: {other:?}"),
    }
}

fn the_fuse(
    placement: Placement,
    rule: Rule,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let mut model = a_model();
    let tetrahedron = a_tetrahedron(&mut model, Front::ThroughThreePoints);
    model
        .place_solid(tetrahedron, &placement.movement())
        .expect("this model made the key");
    let block = a_block(&mut model, &placement.movement());
    let answer = model
        .fuse(&[tetrahedron], &[block], rule, policy, log)
        .map(|answer| answer.solids().to_vec());
    (model, answer)
}

/// One solid, holding 45 for the difference and 15 for the intersection to
/// `1e−12`, in a model that validates.
fn the_answer_holds_its_half(
    model: &Model,
    solids: &[SolidKey],
    rule: Rule,
    policy: &Policy,
    case: &str,
) {
    let [solid] = solids else {
        panic!("{case}: {} solids", solids.len());
    };
    let half = match rule {
        Rule::Difference => 45.0,
        Rule::Intersection => 15.0,
        other => panic!("{case}: no half for {other:?}"),
    };
    let volume = model
        .solid(*solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::of(0.01).expect("a deflection"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a measure")
        .volume();
    assert!(
        (volume - half).abs() <= 1e-12 * half,
        "{case}: {volume} for {half}"
    );
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|error| panic!("{case}: {error:?}"));
}

/// The sign of the block's face `y = 0`, outside `−y`, where the tetrahedron's
/// three side planes meet, both placed.
fn the_side_of_the_cut_the_apex_is_on(front: Front, placement: Placement) -> Decision {
    let mut model = a_model();
    let tetrahedron = a_tetrahedron(&mut model, front);
    model
        .place_solid(tetrahedron, &placement.movement())
        .expect("this model made the key");
    let block = a_block(&mut model, &placement.movement());
    let three = the_side_planes(&model, tetrahedron, &placement.movement())
        .map(|plane| plane.coefficients());
    let cut = the_plane_facing(
        &model,
        block,
        &placement.movement(),
        Vector::new(0.0, -1.0, 0.0),
    );
    side_of_a_plane_where_three_planes_meet(three, cut.coefficients())
        .expect("three side planes of a tetrahedron meet in a point")
}

/// The front, right and left side planes of the tetrahedron, placed.
fn the_side_planes(model: &Model, tetrahedron: SolidKey, placement: &Isometry) -> [Plane; 3] {
    [
        Vector::new(0.0, -10.0, 3.0),
        Vector::new(20.0, 10.0, 3.0),
        Vector::new(-20.0, 10.0, 3.0),
    ]
    .map(|way| the_plane_facing(model, tetrahedron, placement, way))
}

/// The plane of the face of `solid` facing most nearly `way` once `way` is
/// placed.
fn the_plane_facing(model: &Model, solid: SolidKey, placement: &Isometry, way: Vector) -> Plane {
    let way = placement.apply_to_vector(way);
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| face.plane())
        .max_by(|one, other| one.normal().dot(way).total_cmp(&other.normal().dot(way)))
        .expect("a solid has faces")
}

/// The four placements of §294.8.
#[derive(Clone, Copy, Debug)]
enum Placement {
    WhereItWasBuilt,
    Shifted,
    Turned,
    TurnedThenShifted,
}

impl Placement {
    const EVERY: [Self; 4] = [
        Self::WhereItWasBuilt,
        Self::Shifted,
        Self::Turned,
        Self::TurnedThenShifted,
    ];

    fn movement(self) -> Isometry {
        let shift =
            Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation");
        let turn = Isometry::rotation(&Axis::x(), 0.4).expect("a turn about x");
        match self {
            Self::WhereItWasBuilt => Isometry::identity(),
            Self::Shifted => shift,
            Self::Turned => turn,
            Self::TurnedThenShifted => shift.compose(&turn),
        }
    }
}

/// How the face facing `−y`, through the apex and the two corners of the base
/// in front, gets its plane.
#[derive(Clone, Copy, Debug)]
enum Front {
    /// `Plane::through_three_points`, as every other face.
    ThroughThreePoints,
    /// Along `(0, −10, 3)` at the length [`K`], through the apex.
    ThroughTheApexExactly,
}

/// `1/√109` rounded to a multiple of `2⁻⁵⁰`. `10K`, `3K` and `30K` are each an
/// `f64`, so the front face along `(0, −10K, 3K)` through the apex has the
/// offset `30K` exactly and holds the apex and `(±3, −3, 0)` exactly. `109K²`
/// is `8ε` short of one, inside `Plane::NORMAL_LENGTH_ROUNDOFF`, so
/// `Plane::from_point_and_normal_as_written` keeps those bits.
const K: f64 = 0.095_782_628_522_115_05;

const APEX: [f64; 3] = [0.0, 0.0, 10.0];

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_block(model: &mut Model, placement: &Isometry) -> SolidKey {
    let footprint = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let own = Isometry::translation(Vector::new(-5.0, 0.0, -1.0)).expect("a plain translation");
    model
        .add_extrusion(&footprint, 12.0, &placement.compose(&own))
        .expect("a block")
}

fn a_tetrahedron(model: &mut Model, front: Front) -> SolidKey {
    let points = [APEX, [-3.0, -3.0, 0.0], [3.0, -3.0, 0.0], [0.0, 3.0, 0.0]]
        .map(|[x, y, z]| Point::new(x, y, z));
    let faces: [&[usize]; 4] = [&[0, 1, 2], &[0, 2, 3], &[0, 3, 1], &[1, 3, 2]];

    let mut builder = SolidBuilder::new();
    let vertices = points.map(|point| builder.add_vertex(point).expect("a finite corner"));
    let mut edges: Vec<([usize; 2], DraftEdge)> = Vec::new();
    let drafted = faces.map(|face| {
        let steps: Vec<DraftStep> = (0..face.len())
            .map(|position| {
                let (from, to) = (face[position], face[(position + 1) % face.len()]);
                let (edge, orientation) = match edges.iter().find(|(ends, _)| *ends == [to, from]) {
                    Some(&(_, edge)) => (edge, Orientation::Reversed),
                    None => {
                        let edge = builder
                            .add_edge(vertices[from], vertices[to], EdgeRun::Straight)
                            .expect("a side between two corners");
                        edges.push(([from, to], edge));
                        (edge, Orientation::Forward)
                    }
                };
                DraftStep::Along(
                    builder
                        .add_coedge(edge, orientation)
                        .expect("one use of a side"),
                )
            })
            .collect();
        let wire = builder.add_wire(&steps).expect("a closed boundary");
        let plane = match front {
            Front::ThroughTheApexExactly if face == [0, 1, 2] => {
                Plane::from_point_and_normal_as_written(
                    points[0],
                    Vector::new(0.0, -10.0 * K, 3.0 * K),
                )
                .expect("a plane through the apex")
            }
            Front::ThroughTheApexExactly | Front::ThroughThreePoints => {
                Plane::through_three_points(points[face[0]], points[face[1]], points[face[2]])
                    .expect("three corners of a face not in a line")
            }
        };
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a face of a tetrahedron")
    });
    builder
        .add_shell(&drafted.map(|face| (face, Orientation::Forward)))
        .expect("four faces close a tetrahedron");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a tetrahedron is a sound solid")
}
