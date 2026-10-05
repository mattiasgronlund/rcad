//! A round bar whose wall is two faces on one pipe, fused with a cube that
//! touches nothing, and with a round bar on its own pipe, which is refused.
//!
//! The bar runs from `z = 0` to `z = 4` and its wall is two bands, `z` 0 to 2
//! and 2 to 4, with the circle at `z = 2` between them. Each band is written on
//! either spelling of the pipe, its axis pointing up or pointing down. The bar is
//! built by hand, as `two_spellings_of_one_pipe_mesh_each_face_on_its_own_side.rs`
//! builds it, because nothing else writes a wall in two bands.
//!
//! Every fuse of it used to refuse `AFaceMeetsItsNeighbourOnItsOwnSurface`,
//! though the cube is far away: the two bands reached the arrangement as two
//! faces, the circle between them as an edge of each, and the arrangement asked
//! `intersect` where a band meets the band across that circle and heard
//! `SameSurface` (row Q575, `docs/decisions.md` §231.5, §325.5). The flat
//! joiner never ran on the bar, since it is curved. Now `bake` hands faces of an
//! input that are one surface used one way to the arrangement as one face, with
//! the edges between them left out the way a seam is.
//!
//! A round bar on the bar's own pipe is still refused (row Q899, §325.5), and
//! not by the fuse: neither band can say whether it holds a point of its wall,
//! because the face across the circle is on its own surface. The two tests at
//! the end pin that refusal, say what the answers are, and say what a temporary
//! fix showed.
//!
//! # The oracle
//!
//! The cube touches nothing, so every answer is the bar, the cube, both or
//! neither, and each solid is measured against `4π` and `1` inside the bound
//! its own measurement publishes.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftEdge, DraftFace, DraftStep, Error, FaceKey, Invalid, Model, Rule,
    SolidBuilder, SolidKey,
};

const RADIUS: f64 = 1.0;

const THE_BAR: f64 = PI * RADIUS * RADIUS * 4.0;

const THE_CUBE: f64 = 1.0;

/// The bar on every pair of spellings, its faces listed either way round,
/// fused with the cube under every rule with either one as the argument.
///
/// Only one way round the rim builds for each pair, as the file this bar is
/// copied from says, and the bar is not turned: turned, a bar in two bands on
/// two spellings fails `Model::validate` before any fuse, refused
/// `AClosedEdgeIsWalkedAgainstItsSurface`, and that is a row of its own.
#[test]
fn a_bar_whose_wall_is_two_bands_fuses_with_a_cube_that_touches_nothing() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    let mut wrong = Vec::new();
    for (lower, upper) in [(up, up), (up, down), (down, up), (down, down)] {
        for lower_first in [true, false] {
            let bar_is = format!(
                "bands on {} and {}, lower first {lower_first}",
                which_way(lower),
                which_way(upper)
            );
            let (model, bar) = the_bar(lower, upper, lower_first, &bar_is);
            for (rule, bar_first, truths) in [
                (Rule::Union, true, &[THE_BAR, THE_CUBE][..]),
                (Rule::Union, false, &[THE_BAR, THE_CUBE][..]),
                (Rule::Intersection, true, &[][..]),
                (Rule::Intersection, false, &[][..]),
                (Rule::Difference, true, &[THE_BAR][..]),
                (Rule::Difference, false, &[THE_CUBE][..]),
            ] {
                let what = format!("{bar_is}, {rule:?}, the bar first {bar_first}");
                if let Err(why) = fused_with_a_far_cube(&model, bar, rule, bar_first, truths) {
                    wrong.push(format!("{what}: {why}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A round bar on the bar's own pipe is refused by every fuse with the bar, and
/// what refuses it is the round bar's floor, not the two walls lying on one
/// pipe.
///
/// The round bar is `add_round_bar(1, 4)` standing on `z = 1`, or
/// `add_round_bar(1, 1)` standing on `z = 2.5`, which lies on the upper band
/// alone. Either one's floor is a disk inside the bar. The ray that places the
/// disk's middle crosses the bar's wall, where neither band can say whether it
/// holds the crossing ([`no_band_of_the_bar_can_say_what_it_holds`]), so every
/// fuse refuses `a point in a piece that every solid could place` at the
/// floor's one corner, `(1, 0, 1)` or `(1, 0, 2.5)`. That is every spelling
/// pair, either order of the faces, every rule and both orders of the
/// arguments: 48 fuses with each round bar.
///
/// The answers exist. With the long round bar the union is `5π`, the
/// intersection `3π`, and the difference `π` either way round. With the short
/// one the union is the bar, `4π`; the intersection is the short bar, `π`; the
/// bar less it is two solids, `2.5π` and `0.5π`; and it less the bar is
/// nothing.
///
/// **Why this is not `fuse/select.rs`.** The bar's wall goes into the fuse as
/// one face with two keys, one per band, and `a_face_of_the_input_holds` asks
/// a curved face about the first key only. Both are true, and a temporary
/// print showed the two keys in all 48 fuses with the long round bar. But no
/// fuse here gets that far,
/// and once it does, every key refuses the way the bands do below. With a
/// temporary change to `holds.rs` that read the circle at `z = 2` by the plane
/// its run names, all 48 fuses with the long round bar answered, validated and
/// measured what they should within the bound `measure` publishes. The short
/// one then refused the 24 fuses where the lower band was the wall's first key
/// and answered the other 24: its wall lies on the upper band only, the first
/// key alone said no, and every point of its wall then lay on the bar's skin.
/// Asking every key as well answered all 48. Both changes were taken out.
#[test]
fn a_round_bar_on_the_bars_own_pipe_is_refused_at_its_floor_inside_the_bar() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    let mut wrong = Vec::new();
    for (lower, upper) in [(up, up), (up, down), (down, up), (down, down)] {
        for lower_first in [true, false] {
            let bar_is = format!(
                "bands on {} and {}, lower first {lower_first}",
                which_way(lower),
                which_way(upper)
            );
            let (model, bar) = the_bar(lower, upper, lower_first, &bar_is);
            // The long round bar from `z = 1` to `z = 5`, and the short one
            // from `z = 2.5` to `z = 3.5` on the upper band alone.
            for (floor, height) in [(1.0, 4.0), (2.5, 1.0)] {
                for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                    for bar_first in [true, false] {
                        let mut model = model.clone();
                        let other = a_round_bar_on_the_pipe(&mut model, floor, height);
                        let (arguments, tools) = if bar_first {
                            ([bar], [other])
                        } else {
                            ([other], [bar])
                        };
                        let said = model
                            .fuse(
                                &arguments,
                                &tools,
                                rule,
                                &Policy::certified(),
                                &mut ProvenanceLog::new(),
                            )
                            .map(|fused| fused.solids().len());
                        let refused_at_the_floor = matches!(
                            &said,
                            Err(Error::Undecidable { records })
                                if records.records().iter().any(|record| {
                                    record.predicate()
                                        == "a point in a piece that every solid could place"
                                        && record.inputs()
                                            == [PredicatePoint::Exact([RADIUS, 0.0, floor])]
                                })
                        );
                        if !refused_at_the_floor {
                            wrong.push(format!(
                                "{bar_is}, the round bar's floor at z = {floor}, {rule:?}, \
                                 the bar first {bar_first}: {said:?}"
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The refusal above with no fuse: neither band of the bar can say whether it
/// holds a point of its wall, so a ray from inside the bar that crosses the
/// wall places nothing.
///
/// `SolidView::face_holds_the_point` reads what trims a curved face off the
/// faces across its edges, and across the circle at `z = 2` is the other band,
/// on the same pipe. `intersect` calls two spellings of one pipe one surface,
/// so the circle is not a curve the two cross in, and the band is refused
/// `AFaceMissesPartOfWhereItsSurfacesCross` wherever it is asked. The circle's
/// run names the plane `z = 2` as what it crosses. A temporary change to
/// `holds.rs` that read the band's trimming at that circle off that plane gave
/// the answers that exist: `(0, 0, 1)` is inside the bar, the lower band holds
/// `(0, −1, 1)` and not `(0, −1, 3)`, and the upper band the other way round.
#[test]
fn no_band_of_the_bar_can_say_what_it_holds() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    let policy = Policy::certified();
    let mut wrong = Vec::new();
    for (lower, upper) in [(up, up), (up, down), (down, up), (down, down)] {
        for lower_first in [true, false] {
            let bar_is = format!(
                "bands on {} and {}, lower first {lower_first}",
                which_way(lower),
                which_way(upper)
            );
            let (model, bar) = the_bar(lower, upper, lower_first, &bar_is);
            let solid = model.solid(bar).expect("this model made the key");
            let bands: Vec<FaceKey> = solid
                .faces()
                .filter(|face| face.surface().as_plane().is_none())
                .map(|face| face.key())
                .collect();
            assert_eq!(bands.len(), 2, "{bar_is}: the wall is two bands");
            let middle = solid.classify_point(
                Point::new(0.0, 0.0, 1.0),
                &policy,
                &mut ProvenanceLog::new(),
            );
            if !matches!(middle, Containment::CannotFollowAFacesTrimming { face } if bands.contains(&face))
            {
                wrong.push(format!("{bar_is}: (0, 0, 1) is {middle:?}"));
            }
            for &band in &bands {
                for at in [[0.0, -RADIUS, 1.0], [0.0, -RADIUS, 3.0]] {
                    let held = solid.face_holds_the_point(
                        band,
                        PredicatePoint::Exact(at),
                        &policy,
                        &mut ProvenanceLog::new(),
                    );
                    if !matches!(held, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == band)
                    {
                        wrong.push(format!("{bar_is}: {band:?} at {at:?} says {held:?}"));
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The bar united with the cube comes back with its wall as one face: the two
/// bands were one face from the moment they were baked, so nothing put the
/// circle between them back.
#[test]
fn the_bar_united_with_a_far_cube_comes_back_with_one_wall() {
    let up = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), RADIUS).expect("a radius");
    for (lower, upper) in [(up, up), (up, down)] {
        let bar_is = format!("bands on {} and {}", which_way(lower), which_way(upper));
        let (mut model, bar) = the_bar(lower, upper, true, &bar_is);
        let cube = a_far_cube(&mut model);
        let fused = model
            .fuse(
                &[bar],
                &[cube],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{bar_is}: refused {why:?}"));
        let walls: Vec<usize> = fused
            .solids()
            .iter()
            .map(|&solid| {
                model
                    .solid(solid)
                    .expect("this model made the key")
                    .faces()
                    .filter(|face| face.surface().as_plane().is_none())
                    .count()
            })
            .collect();
        assert!(
            walls.contains(&1) && walls.contains(&0),
            "{bar_is}: the bar has one curved face and the cube none, got {walls:?}"
        );
    }
}

/// The one way round the rim that builds, in a model of its own.
fn the_bar(lower: Cylinder, upper: Cylinder, lower_first: bool, what: &str) -> (Model, SolidKey) {
    let facing_up =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let built: Vec<SolidKey> = [Orientation::Forward, Orientation::Reversed]
        .into_iter()
        .filter_map(|lower_walks_the_rim| {
            match a_bar_in_two_bands(
                &mut model,
                [lower, upper],
                facing_up,
                lower_walks_the_rim,
                lower_first,
            ) {
                Ok(bar) => Some(bar),
                Err(Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface)) => None,
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
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the bar validates: {why:?}"));
    (model, bar)
}

/// Fuse the bar with a cube far from it in a copy of `model`, validate the
/// copy, and match each solid of the answer to one of `truths`.
fn fused_with_a_far_cube(
    model: &Model,
    bar: SolidKey,
    rule: Rule,
    bar_first: bool,
    truths: &[f64],
) -> Result<(), String> {
    let mut model = model.clone();
    let cube = a_far_cube(&mut model);
    let (arguments, tools) = if bar_first {
        ([bar], [cube])
    } else {
        ([cube], [bar])
    };
    let policy = Policy::certified();
    let fused = model
        .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("refused {why:?}"))?;
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("the answer does not validate: {why:?}"))?;
    if fused.solids().len() != truths.len() {
        return Err(format!(
            "{} solids against {truths:?}",
            fused.solids().len()
        ));
    }
    let mut left = truths.to_vec();
    for &solid in fused.solids() {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .map_err(|why| format!("a solid does not measure: {why:?}"))?;
        let Some(found) = left
            .iter()
            .position(|truth| (measured.volume() - truth).abs() <= measured.volume_error())
        else {
            return Err(format!(
                "a solid measures {} ± {} against {left:?}",
                measured.volume(),
                measured.volume_error()
            ));
        };
        left.remove(found);
    }
    Ok(())
}

/// A cube of side one with its low corner at `(20, 20, 20)`.
fn a_far_cube(model: &mut Model) -> SolidKey {
    let cube = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(20.0, 20.0, 20.0)).expect("a finite shift"),
        )
        .expect("this model made the key");
    cube
}

/// A round bar on the bar's own pipe, `height` tall, standing on `z = floor`.
fn a_round_bar_on_the_pipe(model: &mut Model, floor: f64, height: f64) -> SolidKey {
    model
        .add_round_bar(
            RADIUS,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, floor)).expect("a finite shift"),
        )
        .expect("a round bar")
}

fn which_way(wall: Cylinder) -> &'static str {
    if wall.axis().direction().z() > 0.0 {
        "the pipe pointing up"
    } else {
        "the pipe pointing down"
    }
}

/// A bar of `RADIUS` from `z = 0` to `z = 4` in two bands, the lower on
/// `walls[0]` and the upper on `walls[1]`, with the circle at `z = 2` between
/// them written as crossing `rim_plane` and walked `lower_walks_the_rim` by the
/// lower band and the other way by the upper.
///
/// `lower_first` lists the lower band's face before the upper's, in the order
/// they are added and in the shell.
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
) -> rcad_topo::Result<SolidKey> {
    let [lower, upper] = walls;
    let mut draft = SolidBuilder::new();
    let [at_0, at_2, at_4] = [0.0, 2.0, 4.0].map(|z| {
        draft
            .add_vertex(Point::new(RADIUS, 0.0, z))
            .expect("a corner")
    });
    let seam_below = draft.add_edge(at_0, at_2, EdgeRun::Straight)?;
    let seam_above = draft.add_edge(at_2, at_4, EdgeRun::Straight)?;
    let bottom_circle = draft.add_edge(at_0, at_0, EdgeRun::RightRoundWhereItsFacesCross)?;
    let rim = draft.add_edge(
        at_2,
        at_2,
        EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(rim_plane),
        },
    )?;
    let top_circle = draft.add_edge(at_4, at_4, EdgeRun::RightRoundWhereItsFacesCross)?;
    let bottom = a_cap(&mut draft, 0.0, -1.0, bottom_circle, lower);
    let top = a_cap(&mut draft, 4.0, 1.0, top_circle, upper);

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
    let (low, high) = if lower_first {
        let low = a_band(&mut draft, lower, below)?;
        (low, a_band(&mut draft, upper, above)?)
    } else {
        let high = a_band(&mut draft, upper, above)?;
        (a_band(&mut draft, lower, below)?, high)
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

/// A flat cap at height `z` facing `outward` along `z`, bounded by `circle`
/// walked the way the wall on `wall` needs it: against the wall's use, which is
/// `Reversed` for the bottom cap and `Forward` for the top.
fn a_cap(
    draft: &mut SolidBuilder,
    z: f64,
    outward: f64,
    circle: DraftEdge,
    wall: Cylinder,
) -> DraftFace {
    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
            .expect("the z axis is a unit normal");
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
