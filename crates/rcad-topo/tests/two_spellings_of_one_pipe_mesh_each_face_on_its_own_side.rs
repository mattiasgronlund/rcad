//! One pipe written two ways — its axis pointing up and pointing down — and
//! whether each face on it is meshed where it is.
//!
//! A closed edge carries no direction of its own: which way round it is stored
//! is the rule `rcad_geom::which_way_a_closed_crossing_turns` reads off a pair of
//! surfaces, and each face's coedge says only "with that way" or "against it".
//! For a circle between two faces on one pipe, the pair is the pipe that
//! `Surface::how_it_sorts_against` puts first and the circle's plane
//! (`docs/decisions.md` §336), and turning a pipe's axis round turns its
//! longitude round. So with the two spellings either side, the way the circle
//! is walked follows which spelling sorts first. Walked the other way from the
//! way its coedges were written, each face's triangles would land on the other
//! face's side — which is what Q14 found for a ball
//! (`a_ball_cut_through_its_equator_meshes_each_half_on_its_own_side.rs`), and
//! what a volume alone does not see.
//!
//! # Where such a circle is
//!
//! **Not in the answer of any fuse of two such bars that has been tried**:
//! stacked (`z` 0 to 2 and 2 to 4) or overlapping (0 to 4 and 2 to 6), either
//! spelling for each, united, intersected and differenced. Two bars stacked on
//! one pipe written opposite ways, united, come back with one wall on the lower
//! spelling and no circle between the bars
//! (`two_bars_on_one_pipe_answer_the_same_however_the_pipe_is_written` in
//! `fuse/assemble.rs`), so what is left to check there is the two cap circles,
//! and the first test does. Overlapping and intersected or differenced, the
//! answer is a bar with one wall and a rim where one bar's disc meets the other
//! bar's wall: a circle between a flat face and a curved one, which the second
//! test checks (`docs/decisions.md` §359). **Through the builder, yes**: a bar
//! in two bands, one on each spelling, with the circle between them naming its
//! plane — the run the STEP reader writes for a closed circle between two
//! curved faces that are not one surface bit for bit (§222.2). The third test
//! builds that.
//!
//! # The oracle
//!
//! Every face here is a disc or a band of the pipe, bounded by circles square to
//! its axis, so it lies between the lowest and the highest of its own corners
//! along the axis, and measures `πR²` if it is flat and `2πR` times that
//! height if it is not. Both are read off the model's corners and not off the
//! mesh. A face meshed on the wrong side of a circle leaves that span by the
//! height of the band across it.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, Error, FaceView, Invalid, Mesh, Model, Rule, SolidBuilder,
    SolidKey,
};

const RADIUS: f64 = 1.0;

const DEFLECTION: f64 = 0.01;

/// Two bars end to end on one pipe, `z` 0 to 2 and 2 to 4, each wall on either
/// spelling, united either way round and turned three ways first.
///
/// The turns about `x` are there because a turn moves which spelling sorts
/// first: `(0, 0, −1)` sorts before `(0, 0, 1)`, and turned by `1` about `x`
/// they are `(0, 0.84, −0.54)` and `(0, −0.84, 0.54)`, which sort the other way.
/// The fuse writes the joined wall onto the lower spelling in the turned numbers
/// it works in, and turns round whichever cap circle the other spelling wound.
#[test]
fn two_bars_stacked_on_one_pipe_mesh_each_face_where_it_is_however_the_pipe_is_written() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    let mut wrong = Vec::new();
    for (turn_name, turn) in the_turns(&[], &[(&Axis::x(), 1.0), (&Axis::x(), -1.0)]) {
        for (lower, upper) in [(up, up), (up, down), (down, up), (down, down)] {
            for lower_first in [true, false] {
                let what = format!(
                    "the bars on {} and {}, lower first {lower_first}, turned {turn_name}",
                    which_way(lower),
                    which_way(upper)
                );
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let policy = Policy::certified();
                let bars = [(0.0, 2.0, lower), (2.0, 4.0, upper)].map(|(from, to, wall)| {
                    let bar = a_bar_between(&mut model, from, to, wall);
                    model
                        .place_solid(bar, &turn)
                        .expect("this model made the key");
                    bar
                });
                let [arguments, tools] = if lower_first {
                    bars
                } else {
                    [bars[1], bars[0]]
                };
                let fused = model
                    .fuse(
                        &[arguments],
                        &[tools],
                        Rule::Union,
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{what}: two bars end to end unite: {why:?}"));
                model
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: the answer validates: {why:?}"));
                let [solid] = fused.solids() else {
                    panic!(
                        "{what}: two bars end to end are one bar: {:?}",
                        fused.solids()
                    );
                };
                let view = model.solid(*solid).expect("this model made the key");
                assert_eq!(
                    view.faces()
                        .filter(|face| face.surface().as_plane().is_none())
                        .count(),
                    1,
                    "{what}: the two walls are joined into one face"
                );
                wrong.extend(what_is_wrong_with_each_face(&model, *solid, &turn, &what));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Two bars overlapping on one pipe, `z` 0 to 4 and 2 to 6, each wall on
/// either spelling, united, intersected and differenced either way round and
/// turned as the first test turns them.
///
/// United they are a bar six high, and every other answer is a bar two high:
/// `z` 2 to 4 for the intersection, 0 to 2 for the lower less the upper and 4
/// to 6 for the upper less the lower. Each of those has a rim where one bar's
/// disc meets the other bar's wall, and with the two walls on opposite
/// spellings the disc's side of it was read against its own bar's pipe while
/// the edge is stored by the wall across it (`docs/decisions.md` §359).
///
/// Fails on the base for the 24 intersections and differences of opposite
/// spellings, refused `TheFuseWroteABrokenAnswer { CoedgesRunTheSameWay }`.
///
/// **The bars are `RADIUS`, `1`, and not the `3` of the file the STEP test
/// reads**: at radius `3`, turned by `1` about `x`, every one of these fuses is
/// refused `Undecidable` on "a point in a piece that every solid could place",
/// on one spelling as on two, which is a refusal of its own and not this one.
#[test]
fn two_overlapping_bars_on_one_pipe_fuse_into_one_bar_however_the_pipe_is_written() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    let mut wrong = Vec::new();
    for (turn_name, turn) in the_turns(&[], &[(&Axis::x(), 1.0), (&Axis::x(), -1.0)]) {
        for (lower, upper) in [(up, up), (up, down), (down, up), (down, down)] {
            for (rule, lower_first, rule_name, height) in [
                (Rule::Union, true, "united, the lower first", 6.0),
                (Rule::Union, false, "united, the upper first", 6.0),
                (
                    Rule::Intersection,
                    true,
                    "intersected, the lower first",
                    2.0,
                ),
                (
                    Rule::Intersection,
                    false,
                    "intersected, the upper first",
                    2.0,
                ),
                (Rule::Difference, true, "the lower less the upper", 2.0),
                (Rule::Difference, false, "the upper less the lower", 2.0),
            ] {
                let closed_form = core::f64::consts::PI * RADIUS * RADIUS * height;
                let what = format!(
                    "the bars on {} and {}, {rule_name}, turned {turn_name}",
                    which_way(lower),
                    which_way(upper)
                );
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let policy = Policy::certified();
                let bars = [(0.0, 4.0, lower), (2.0, 6.0, upper)].map(|(from, to, wall)| {
                    let bar = a_bar_between(&mut model, from, to, wall);
                    model
                        .place_solid(bar, &turn)
                        .expect("this model made the key");
                    bar
                });
                let [arguments, tools] = if lower_first {
                    bars
                } else {
                    [bars[1], bars[0]]
                };
                let fused = match model.fuse(
                    &[arguments],
                    &[tools],
                    rule,
                    &policy,
                    &mut ProvenanceLog::new(),
                ) {
                    Ok(fused) => fused,
                    Err(why) => {
                        wrong.push(format!("{what}: refused {why:?}"));
                        continue;
                    }
                };
                if let Err(why) = model.validate(&policy, &mut ProvenanceLog::new()) {
                    wrong.push(format!("{what}: the answer does not validate: {why:?}"));
                    continue;
                }
                let [solid] = fused.solids() else {
                    wrong.push(format!("{what}: {} solids", fused.solids().len()));
                    continue;
                };
                let measured = model
                    .solid(*solid)
                    .expect("this model made the key")
                    .measure(
                        Deflection::of(DEFLECTION).expect("a positive length"),
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{what}: the answer measures: {why:?}"));
                if (measured.volume() - closed_form).abs() > measured.volume_error() {
                    wrong.push(format!(
                        "{what}: measured {} ± {}, and by hand it is {closed_form}",
                        measured.volume(),
                        measured.volume_error()
                    ));
                }
                wrong.extend(what_is_wrong_with_each_face(&model, *solid, &turn, &what));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A bar in two bands, `z` 0 to 2 and 2 to 4, each band on either spelling,
/// with the circle between them naming its plane facing up or facing down.
///
/// The circle is offered both ways round, and exactly one of them builds: a
/// band bounded by two circles on a pipe is one region, so only one use of the
/// circle leaves each band on its own side of it. The one that builds is then
/// meshed with its faces listed either way round, placed three ways as well as
/// not at all.
///
/// **Only turns that keep the solid valid are here.** Turned by `π/2`, `1` or
/// `π` about `x` — the turns that put the up spelling first — `Model::validate`
/// refuses the placed bar `AClosedEdgeIsWalkedAgainstItsSurface`, though it was
/// built and validated unturned, and a bar in two bands on one spelling passes
/// every one of those turns. The mesh of every face still lands on its own side
/// at those turns, and the refusal is reported for a row of its own rather than
/// pinned here.
#[test]
fn a_bar_in_two_bands_on_the_two_spellings_meshes_each_band_on_its_own_side() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    let facing_up =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let turns = the_turns(
        &[1.0],
        &[
            (&Axis::x(), -1.0),
            (&Axis::y(), core::f64::consts::FRAC_PI_2),
        ],
    );
    let mut wrong = Vec::new();
    for (lower, upper) in [(up, up), (up, down), (down, up), (down, down)] {
        for rim_plane in [facing_up, facing_up.reversed()] {
            for lower_first in [true, false] {
                let what = format!(
                    "bands on {} and {}, the rim's plane facing {:+}, lower first {lower_first}",
                    which_way(lower),
                    which_way(upper),
                    rim_plane.normal().z()
                );
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let built: Vec<SolidKey> = [Orientation::Forward, Orientation::Reversed]
                    .into_iter()
                    .filter_map(|lower_walks_the_rim| {
                        match a_bar_in_two_bands(
                            &mut model,
                            [lower, upper],
                            rim_plane,
                            lower_walks_the_rim,
                            lower_first,
                            &Isometry::identity(),
                        ) {
                            Ok(bar) => Some(bar),
                            Err(Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface)) => {
                                None
                            }
                            Err(why) => panic!("{what}: refused {why:?}"),
                        }
                    })
                    .collect();
                let [bar] = built[..] else {
                    panic!(
                        "{what}: one way round the rim builds, and {} did",
                        built.len()
                    );
                };
                for (turn_name, turn) in &turns {
                    let mut placed = model.clone();
                    placed
                        .place_solid(bar, turn)
                        .expect("this model made the key");
                    let what = format!("{what}, turned {turn_name}");
                    placed
                        .validate(&Policy::certified(), &mut ProvenanceLog::new())
                        .unwrap_or_else(|why| panic!("{what}: the bar validates: {why:?}"));
                    wrong.extend(what_is_wrong_with_each_face(&placed, bar, turn, &what));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The bar in two bands of the test above, on one spelling and on two, turned
/// off `z` ten ways: built on the turned pipe, and built upright and placed.
///
/// The rim between the bands names its plane, and the bands either side of it
/// are on the one pipe, which crossed with itself bounds nothing — so the plane
/// is the only thing that says how far up the pipe the rim runs. The mesher
/// read the plane only off a run that is part of a crossing, and charged a rim
/// that goes right round by the pipe crossed with itself: the part of the
/// pipe's unit axis square to itself, which is zero, came out about `5e-16` on
/// a turned axis that is not exactly length one, and the rim was charged `2`
/// divided by that. Each of these turns was refused
/// `DeflectionBelowWhatAMeshCanReach` at `0.01` in all four ways of building
/// it, with a floor between `96460` and `356599`.
///
/// **The placed bars are not validated**, unlike in the test above: on two
/// spellings, `Model::validate` refuses some of them
/// `AClosedEdgeIsWalkedAgainstItsSurface`, which is a refusal of its own and
/// not this one. Each bar built on the turned pipe is validated by its build.
#[test]
fn a_bar_in_two_bands_turned_off_z_meshes_each_band_on_its_own_side() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    let facing_up =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let turns = [
        ((1.0, 0.0, 0.0), 0.25),
        ((0.0, 1.0, 0.0), 0.25),
        ((1.0, 1.0, 0.0), 0.25),
        ((1.0, 0.0, 1.0), 0.75),
        ((0.0, 1.0, 1.0), 0.75),
        ((1.0, 2.0, 3.0), 1.75),
        ((-1.0, 2.0, 0.5), 1.25),
        ((3.0, -1.0, 2.0), 0.5),
        ((1.0, 1.0, 1.0), 0.5),
        ((2.0, 0.0, -1.0), 1.0),
    ]
    .map(|((x, y, z), angle)| {
        let axis = Axis::new(Point::origin(), Vector::new(x, y, z)).expect("a direction");
        (
            format!("{angle} about {:?}", (x, y, z)),
            Isometry::rotation(&axis, angle).expect("a finite turn"),
        )
    });
    let mut wrong = Vec::new();
    for (turn_name, turn) in &turns {
        for (lower, upper) in [(up, up), (up, down)] {
            for built_turned in [true, false] {
                let what = format!(
                    "bands on {} and {}, turned {turn_name}, {}",
                    which_way(lower),
                    which_way(upper),
                    if built_turned {
                        "built turned"
                    } else {
                        "placed"
                    }
                );
                let built_on = if built_turned {
                    *turn
                } else {
                    Isometry::identity()
                };
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let built: Vec<SolidKey> = [Orientation::Forward, Orientation::Reversed]
                    .into_iter()
                    .filter_map(|lower_walks_the_rim| {
                        match a_bar_in_two_bands(
                            &mut model,
                            [lower, upper],
                            facing_up,
                            lower_walks_the_rim,
                            true,
                            &built_on,
                        ) {
                            Ok(bar) => Some(bar),
                            Err(Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface)) => {
                                None
                            }
                            Err(why) => panic!("{what}: refused {why:?}"),
                        }
                    })
                    .collect();
                let [bar] = built[..] else {
                    panic!(
                        "{what}: one way round the rim builds, and {} did",
                        built.len()
                    );
                };
                if !built_turned {
                    model
                        .place_solid(bar, turn)
                        .expect("this model made the key");
                }
                wrong.extend(what_is_wrong_with_each_face(&model, bar, turn, &what));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// No turn, turns about `z` by each of `about_z`, and each of `others`.
fn the_turns(about_z: &[f64], others: &[(&Axis, f64)]) -> Vec<(String, Isometry)> {
    let mut turns = vec![("not at all".to_owned(), Isometry::identity())];
    for &angle in about_z {
        turns.push((
            format!("{angle} about z"),
            Isometry::rotation(&Axis::z(), angle).expect("a finite turn"),
        ));
    }
    for &(axis, angle) in others {
        turns.push((
            format!("{angle} about {:?}", axis.direction().components()),
            Isometry::rotation(axis, angle).expect("a finite turn"),
        ));
    }
    turns
}

fn which_way(wall: Cylinder) -> &'static str {
    if wall.axis().direction().z() > 0.0 {
        "the pipe pointing up"
    } else {
        "the pipe pointing down"
    }
}

/// Each face of `solid` whose triangles leave the span of its own corners along
/// the turned axis by more than the mesh strays, or that does not measure its
/// closed form.
fn what_is_wrong_with_each_face(
    model: &Model,
    solid: SolidKey,
    turn: &Isometry,
    what: &str,
) -> Vec<String> {
    let view = model.solid(solid).expect("this model made the key");
    let mesh = view
        .mesh(
            Deflection::of(DEFLECTION).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the bar meshes: {why:?}"));
    let strays = mesh.deviation().magnitude();
    let axis = turn.apply_to_vector(Vector::new(0.0, 0.0, 1.0));
    let height = |at: Point| (at - turn.apply_to_point(Point::origin())).dot(axis);

    let mut wrong = Vec::new();
    for face in view.faces() {
        let (low, high) = the_span_of_its_corners(face, height);
        let closed_form = if face.surface().as_plane().is_some() {
            core::f64::consts::PI * RADIUS * RADIUS
        } else {
            2.0 * core::f64::consts::PI * RADIUS * (high - low)
        };
        let (area, beyond) = what_this_face_was_meshed_as(&mesh, face, height, (low, high));
        let under = (closed_form - area) / closed_form;
        if beyond > strays || !(0.0..3.0 * DEFLECTION / RADIUS).contains(&under) {
            wrong.push(format!(
                "{what}: the face on {:?} between heights {low} and {high} measures {area} \
                 against {closed_form}, and a point of it is {beyond} outside that where the \
                 mesh strays no more than {strays}",
                face.surface()
            ));
        }
    }
    wrong
}

/// The lowest and the highest of a face's corners along the turned axis.
fn the_span_of_its_corners(face: FaceView<'_>, height: impl Fn(Point) -> f64) -> (f64, f64) {
    face.wires()
        .flat_map(|wire| wire.coedges())
        .map(|coedge| height(coedge.start().point()))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), at| {
            (low.min(at), high.max(at))
        })
}

/// How much area the triangles of `face` cover, and how far its worst point is
/// outside `(low, high)`: negative where every point is inside.
fn what_this_face_was_meshed_as(
    mesh: &Mesh,
    face: FaceView<'_>,
    height: impl Fn(Point) -> f64,
    (low, high): (f64, f64),
) -> (f64, f64) {
    let mut area = 0.0;
    let mut beyond = f64::NEG_INFINITY;
    for triangle in mesh
        .triangles()
        .iter()
        .filter(|each| each.face() == face.key())
    {
        let [p, q, r] = triangle
            .corners()
            .map(|index| mesh.points()[index as usize]);
        area += (q - p).cross(r - p).length() / 2.0;
        for at in [p, q, r] {
            beyond = beyond.max(low - height(at)).max(height(at) - high);
        }
    }
    (area, beyond)
}

/// A bar of `RADIUS` on `wall` between two heights, built as
/// `two_bars_on_one_pipe_answer_the_same_however_the_pipe_is_written` builds
/// one, because `Model::add_round_bar` writes its own spelling.
fn a_bar_between(model: &mut Model, from: f64, to: f64, wall: Cylinder) -> SolidKey {
    let mut draft = SolidBuilder::new();
    let bottom_corner = draft
        .add_vertex(Point::new(RADIUS, 0.0, from))
        .expect("a corner");
    let top_corner = draft
        .add_vertex(Point::new(RADIUS, 0.0, to))
        .expect("a corner");
    let seam = draft
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("the seam up the wall");
    let [bottom_circle, top_circle] = [bottom_corner, top_corner].map(|corner| {
        draft
            .add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a cap circle")
    });
    let bottom = a_cap(
        &mut draft,
        from,
        -1.0,
        bottom_circle,
        wall,
        &Isometry::identity(),
    );
    let top = a_cap(&mut draft, to, 1.0, top_circle, wall, &Isometry::identity());
    let round_the_wall = [
        draft
            .add_coedge(
                bottom_circle,
                the_way_it_is_walked(wall, Orientation::Forward),
            )
            .expect("a use"),
        draft.add_coedge(seam, Orientation::Forward).expect("a use"),
        draft
            .add_coedge(
                top_circle,
                the_way_it_is_walked(wall, Orientation::Reversed),
            )
            .expect("a use"),
        draft
            .add_coedge(seam, Orientation::Reversed)
            .expect("a use"),
    ]
    .map(DraftStep::Along);
    let wire = draft.add_wire(&round_the_wall).expect("round the wall");
    let side = draft
        .add_face(Surface::Cylinder(wall), &[wire])
        .expect("the wall");
    draft
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("a closed skin");
    draft
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a round bar")
}

/// A bar of `RADIUS` from `z = 0` to `z = 4` in two bands, the lower on
/// `walls[0]` and the upper on `walls[1]`, with the circle at `z = 2` between
/// them written as crossing `rim_plane` and walked `lower_walks_the_rim` by the
/// lower band and the other way by the upper.
///
/// `lower_first` lists the lower band's face before the upper's, in the order
/// they are added and in the shell. Every point and surface is moved by
/// `built_on` before the builder sees it.
///
/// # Errors
///
/// Whatever [`SolidBuilder::build`] refuses the bar for.
fn a_bar_in_two_bands(
    model: &mut Model,
    walls: [Cylinder; 2],
    rim_plane: Plane,
    lower_walks_the_rim: Orientation,
    lower_first: bool,
    built_on: &Isometry,
) -> rcad_topo::Result<SolidKey> {
    let [lower, upper] = walls;
    let mut draft = SolidBuilder::new();
    let [at_0, at_2, at_4] = [0.0, 2.0, 4.0].map(|z| {
        draft
            .add_vertex(built_on.apply_to_point(Point::new(RADIUS, 0.0, z)))
            .expect("a corner")
    });
    let seam_below = draft.add_edge(at_0, at_2, EdgeRun::Straight)?;
    let seam_above = draft.add_edge(at_2, at_4, EdgeRun::Straight)?;
    let bottom_circle = draft.add_edge(at_0, at_0, EdgeRun::RightRoundWhereItsFacesCross)?;
    let rim = draft.add_edge(
        at_2,
        at_2,
        EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(rim_plane.transformed(built_on)),
        },
    )?;
    let top_circle = draft.add_edge(at_4, at_4, EdgeRun::RightRoundWhereItsFacesCross)?;
    let bottom = a_cap(&mut draft, 0.0, -1.0, bottom_circle, lower, built_on);
    let top = a_cap(&mut draft, 4.0, 1.0, top_circle, upper, built_on);

    let below = [
        (
            bottom_circle,
            the_way_it_is_walked(lower, Orientation::Forward),
        ),
        (seam_below, Orientation::Forward),
        (rim, lower_walks_the_rim),
        (seam_below, Orientation::Reversed),
    ];
    let above = [
        (rim, lower_walks_the_rim.reversed()),
        (seam_above, Orientation::Forward),
        (
            top_circle,
            the_way_it_is_walked(upper, Orientation::Reversed),
        ),
        (seam_above, Orientation::Reversed),
    ];
    let (lower_wall, upper_wall) = (lower.transformed(built_on), upper.transformed(built_on));
    let (low, high) = if lower_first {
        let low = a_band(&mut draft, lower_wall, below)?;
        (low, a_band(&mut draft, upper_wall, above)?)
    } else {
        let high = a_band(&mut draft, upper_wall, above)?;
        (a_band(&mut draft, lower_wall, below)?, high)
    };
    let faces = if lower_first {
        [bottom, low, high, top]
    } else {
        [top, high, low, bottom]
    };
    draft.add_shell(&faces.map(|face| (face, Orientation::Forward)))?;
    draft.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn a_band(
    draft: &mut SolidBuilder,
    wall: Cylinder,
    round: [(DraftEdge, Orientation); 4],
) -> rcad_topo::Result<DraftFace> {
    let mut steps = Vec::new();
    for (edge, way) in round {
        steps.push(DraftStep::Along(draft.add_coedge(edge, way)?));
    }
    let wire = draft.add_wire(&steps)?;
    draft.add_face(Surface::Cylinder(wall), &[wire])
}

/// A flat cap at height `z` facing `outward` along `z`, moved by `built_on`,
/// bounded by `circle` walked the way the wall on `wall` needs it: against the
/// wall's use, which is `Reversed` for the bottom cap and `Forward` for the top.
fn a_cap(
    draft: &mut SolidBuilder,
    z: f64,
    outward: f64,
    circle: DraftEdge,
    wall: Cylinder,
    built_on: &Isometry,
) -> DraftFace {
    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
            .expect("the z axis is a unit normal")
            .transformed(built_on);
    let used = if outward < 0.0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let coedge = draft
        .add_coedge(circle, the_way_it_is_walked(wall, used))
        .expect("a use of the circle");
    let wire = draft
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("a cap is one circle");
    draft
        .add_face(Surface::Plane(plane), &[wire])
        .expect("a cap face")
}

/// A cap circle between `wall` and a plane is stored running along `ŵ × ∇C`
/// for the wall's stored direction `ŵ`, so a wall written pointing down walks
/// both its cap circles the other way round: `turn` is how a wall pointing up
/// would use the circle.
fn the_way_it_is_walked(wall: Cylinder, turn: Orientation) -> Orientation {
    if wall.axis().direction().z() > 0.0 {
        turn
    } else {
        turn.reversed()
    }
}
