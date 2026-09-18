//! `kicad/LED_D5.0mm-3.step`'s shape, built through the public door, so the
//! thing §124 worked out on paper can be watched happening.
//!
//! §124 read the two halves of the closed-edge rule out of the source and
//! concluded that this file asks both faces of its one closed edge to be walked
//! **Forward**, which no solid can be given: the builder's own
//! `check_every_edge_is_walked_once_each_way` needs one of each. §124.4 booked
//! the missing half as debt — *"no throwaway `SolidBuilder` model was actually
//! built and fed both possible `Orientation` pairs to watch both fail"* — and
//! this file is that model.
//!
//! # What is here
//!
//! The dome and the shaft carry the file's own numbers: a sphere of radius
//! `2.5` on `#41` and a cylinder of radius `2.5` on `#74` whose axis leans
//! three femtometres out of the sphere's. They are meant to be tangent along
//! the seam and the file's own rounding is the whole of what makes them cross —
//! `a_hemisphere_sitting_on_a_pipe_of_the_same_radius_barely_crosses_it_at_all`
//! measures that angle at `8.1e-16` radians, under four `f64` steps of one, and
//! this file builds a solid out of the same two surfaces rather than measuring
//! them again.
//!
//! The rest of the solid is not the file's. `LED_D5.0mm-3` closes its shaft off
//! with a flange and a lead frame; here it is one flat floor, because what the
//! rule reads off each face is that face's own corner **away** from the shared
//! edge, and a floor gives the shaft one exactly as the file's `#49` does.
//!
//! # A dome on a shaft is tangent or it is two seams, and there is nothing else
//!
//! It would be natural to read §124 as being about an unlucky file. It is not,
//! and the arithmetic that says so is one line: a cylinder of radius `r` about
//! an axis and a sphere of radius `R` centred **on** that axis cross where
//! `(z − h)² = R² − r²`, which is **two** circles whenever `R > r`, none
//! whenever `R < r`, and exactly one only when `R = r` — where the two surfaces
//! are tangent along it.
//!
//! So a dome sitting square on a shaft has a single closed seam **if and only
//! if** the two are the same radius, which is to say if and only if they are
//! tangent. There is no transversal version of this shape to compare against;
//! `a_dome_on_a_narrower_shaft_has_two_seams_and_not_one` is that statement as a
//! test.
//!
//! # What this pins, and what it must not be read as
//!
//! Every assertion below says what the kernel does **today**. None of them says
//! the answers are right — §124.3 says the opposite, that this is a sound solid
//! the certified rule cannot reach. The day either family of fixes in §124.4
//! lands, these tests fail, and that failure is the fix arriving rather than a
//! regression.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Sphere, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Result, SolidBuilder, SolidKey};

/// Neither way round the one closed edge can be given to this solid.
///
/// This is §124.2 end to end. A reader has two pairs to choose between — the
/// dome Forward and the shaft Reversed, or the other way about — and the
/// certified rule refuses both, because it has worked out that **both** faces
/// walk the edge forward and only one of them ever can.
///
/// The refusal is the one `kicad/LED_D5.0mm-3.step` gives on import, by name.
#[test]
fn a_dome_on_a_shaft_of_its_own_radius_is_refused_whichever_way_its_seam_is_walked() {
    for on_the_dome in [Orientation::Forward, Orientation::Reversed] {
        let refused = the_led(on_the_dome, on_the_dome.reversed())
            .expect_err("§124 says neither way round is accepted");
        assert!(
            matches!(
                refused,
                Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface)
            ),
            "the dome walking its seam {on_the_dome:?} was refused as {refused:?}, and not by \
             the closed-edge rule the file is refused by"
        );
    }
}

/// And the pair the rule does ask for never reaches the rule at all.
///
/// The two per-face answers §124.1 traced are both `walks_it_forward = true`,
/// so the description that would satisfy the rule has both coedges Forward.
/// That description is refused one door earlier, by the invariant that says a
/// skin closes along an edge only if the two faces walk it opposite ways — so
/// what the rule asks for is **unsatisfiable** rather than merely unmet, which
/// is the whole of §124's verdict.
#[test]
fn the_pair_the_rule_asks_for_is_refused_before_the_rule_ever_runs() {
    let refused = the_led(Orientation::Forward, Orientation::Forward)
        .expect_err("a skin does not close along an edge both its faces walk the same way");
    assert!(
        matches!(refused, Error::ADraftEdgeIsWalkedTheSameWayTwice { .. }),
        "both faces walking the seam forward was refused as {refused:?}"
    );
}

/// The corners the rule reads each face's material side off, and how far each
/// is from being a close call.
///
/// §124.3 says the gap is that `turning` is local at the shared corner while
/// `material_side` is read at whichever **other** corner the face happens to
/// carry. These are those other corners, and the point of the numbers is that
/// neither is anywhere near the surface it is being measured against: the
/// distant half of the rule is not where the arithmetic is thin, which is what
/// leaves the tangency as the only thing that can have gone wrong.
#[test]
fn the_two_corners_the_rule_reads_the_material_side_from_are_nowhere_near_a_close_call() {
    let pieces = the_files_own_shape();

    let out_of_the_shaft = how_far_off_the_axis(&pieces.axis, pieces.pole);
    assert!(
        out_of_the_shaft < 1e-14,
        "the dome's pole sits on the shaft's own axis, not {out_of_the_shaft:e} off it"
    );

    let out_of_the_dome = (pieces.bottom - pieces.centre).length();
    assert!(
        (out_of_the_dome - (2.5f64 * 2.5 + 5.1 * 5.1).sqrt()).abs() < 1e-12,
        "the shaft's floor corner is √(2.5² + 5.1²) from the dome's centre, not {out_of_the_dome}"
    );
    assert!(
        out_of_the_dome > 5.6,
        "and that is more than twice the dome's own radius out, not {out_of_the_dome}"
    );

    let on_both = how_far_off_the_axis(&pieces.axis, pieces.seam);
    assert!(
        (on_both - 2.5).abs() < 1e-14,
        "the shared corner is on the shaft as well as the dome, not {on_both} off its axis"
    );
}

/// There is no transversal dome-on-a-shaft to compare against.
///
/// Pull the shaft in to `1.5` under a dome of `2.5` and the two surfaces do
/// cross properly — but they cross in **two** circles, one at `z = 2` and one
/// at `z = −2`, so `EdgeRun::RightRoundWhereItsFacesCross` is not what names
/// either of them. Widen the shaft back out and the lower circle climbs until
/// the two meet, which is the same-radius case and is tangent.
///
/// So §124 is not about one unlucky file. Every dome that sits square on a
/// shaft and shares **one** closed edge with it is tangent along that edge by
/// construction.
#[test]
fn a_dome_on_a_narrower_shaft_has_two_seams_and_not_one() {
    let dome = Sphere::new(Point::origin(), 2.5).expect("the dome");
    let shaft = Cylinder::new(Axis::z(), 1.5).expect("the narrower shaft");
    for corner in [Point::new(1.5, 0.0, 2.0), Point::new(1.5, 0.0, -2.0)] {
        assert_eq!(
            (corner - dome.centre()).length_squared(),
            dome.radius() * dome.radius(),
            "{corner:?} is on the dome"
        );
        assert_eq!(
            how_far_off_the_axis(&shaft.axis(), corner),
            shaft.radius(),
            "{corner:?} is on the shaft too"
        );
    }
}

/// The rule itself is satisfiable, and that is what says the pair above is
/// about the tangency.
///
/// The same description with the shaft pulled in to `1.5` puts a corner where
/// the two surfaces genuinely cross, and there the rule takes **one** of the
/// two pairs and refuses the other — which is what a closed edge is supposed to
/// get. So the double refusal above is not this file describing a solid wrongly.
///
/// That near shape is not a sound solid and this does not pretend it is: its
/// seam edge claims one closed curve where there are two, per the test above,
/// and the pair the rule lets through goes on to be refused later, in the mesh.
/// What is asserted is only where the refusals fall either side of the
/// closed-edge rule, because that rule is the whole of what §124 is about.
#[test]
fn the_closed_edge_rule_takes_one_pair_of_the_two_where_the_surfaces_really_cross() {
    let walked_against_its_surface = |outcome: &std::result::Result<SolidKey, Error>| {
        matches!(
            outcome,
            Err(Error::Invalid(
                Invalid::AClosedEdgeIsWalkedAgainstItsSurface
            ))
        )
    };
    let tried = [Orientation::Forward, Orientation::Reversed].map(|on_the_dome| {
        build(
            &a_dome_on_a_narrower_shaft(),
            on_the_dome,
            on_the_dome.reversed(),
        )
    });
    assert_eq!(
        tried
            .iter()
            .filter(|got| walked_against_its_surface(got))
            .count(),
        1,
        "a corner where the two surfaces really cross has one way round the rule accepts \
         and one it refuses, and got {tried:?}"
    );
}

/// How far a point is from a shaft's axis.
fn how_far_off_the_axis(axis: &Axis, point: Point) -> f64 {
    let along = point - axis.origin();
    let up = axis.direction();
    (along - up * (along.dot(up) / up.dot(up))).length()
}

/// The file's own dome and shaft, of the same radius and near enough the same
/// axis.
fn the_led(on_the_dome: Orientation, on_the_shaft: Orientation) -> Result<SolidKey> {
    build(&the_files_own_shape(), on_the_dome, on_the_shaft)
}

fn the_files_own_shape() -> Pieces {
    let centre = Point::new(2.54, -5.051514762044E-015, 9.1);
    let axis = Axis::new(
        Point::new(2.54, -3.030908857227E-015, 9.1),
        Vector::new(-0.0, 3.330669073875E-016, -1.0),
    )
    .expect("the shaft's axis");
    Pieces {
        dome: Surface::Sphere(Sphere::new(centre, 2.5).expect("the dome")),
        shaft: Surface::Cylinder(Cylinder::new(axis, 2.5).expect("the shaft")),
        axis,
        centre,
        // The file's `#23` and `#50`, and the dome's own pole — which is the
        // file's `#32` put back exactly on the sphere, so that the corner check
        // has nothing to say about it and only the seam is ever in question.
        seam: Point::new(5.04, -5.051514762044E-015, 9.1),
        pole: Point::new(2.54, -5.051514762044E-015, 11.6),
        bottom: Point::new(5.04, -2.22044604925E-015, 4.0),
    }
}

/// The same shape with the shaft pulled in, where the corner is a real
/// crossing. Every corner is exact on both its surfaces: `1.5² + 2² = 2.5²`.
fn a_dome_on_a_narrower_shaft() -> Pieces {
    Pieces {
        dome: Surface::Sphere(Sphere::new(Point::origin(), 2.5).expect("the dome")),
        shaft: Surface::Cylinder(Cylinder::new(Axis::z(), 1.5).expect("the narrower shaft")),
        axis: Axis::z(),
        centre: Point::origin(),
        seam: Point::new(1.5, 0.0, 2.0),
        pole: Point::new(0.0, 0.0, 2.5),
        bottom: Point::new(1.5, 0.0, -1.0),
    }
}

/// A dome, a shaft under it, and a floor under that.
struct Pieces {
    dome: Surface,
    shaft: Surface,
    /// The shaft's own axis, so a test can measure a corner against it without
    /// taking the cylinder apart again.
    axis: Axis,
    centre: Point,
    /// The one corner of the closed edge the dome and the shaft share.
    seam: Point,
    /// The dome's own corner away from that edge, which is what the rule reads
    /// the dome's material side off.
    pole: Point,
    /// The shaft's, likewise.
    bottom: Point,
}

fn build(pieces: &Pieces, on_the_dome: Orientation, on_the_shaft: Orientation) -> Result<SolidKey> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    describe(pieces, on_the_dome, on_the_shaft)?.build(
        &mut model,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// Three faces, four edges, three corners — the file's own `#17` and `#44` plus
/// a floor to close the shaft off.
///
/// The dome is cut open up one meridian to its pole, which is the arrangement
/// the file uses (`#19` walks the seam circle, then `#30` out to the pole and
/// straight back) and is what gives the dome a corner off the shaft at all.
fn describe(
    pieces: &Pieces,
    on_the_dome: Orientation,
    on_the_shaft: Orientation,
) -> Result<SolidBuilder> {
    let mut builder = SolidBuilder::new();

    let seam = builder.add_vertex(pieces.seam)?;
    let pole = builder.add_vertex(pieces.pole)?;
    let bottom = builder.add_vertex(pieces.bottom)?;

    let seam_circle = builder.add_edge(seam, seam, EdgeRun::RightRoundWhereItsFacesCross)?;
    let floor_circle = builder.add_edge(bottom, bottom, EdgeRun::RightRoundWhereItsFacesCross)?;
    let up_the_shaft = builder.add_edge(bottom, seam, EdgeRun::Straight)?;
    let up_the_dome = builder.add_edge(seam, pole, the_meridian(pieces))?;

    let dome_walks = builder.add_coedge(seam_circle, on_the_dome)?;
    let out = builder.add_coedge(up_the_dome, Orientation::Forward)?;
    let back = builder.add_coedge(up_the_dome, Orientation::Reversed)?;
    let round_the_dome = builder.add_wire(&[
        DraftStep::Along(dome_walks),
        DraftStep::Along(out),
        DraftStep::Along(back),
    ])?;
    let dome = builder.add_face(pieces.dome, &[round_the_dome])?;

    let round_the_floor = builder.add_coedge(floor_circle, Orientation::Forward)?;
    let up = builder.add_coedge(up_the_shaft, Orientation::Forward)?;
    let shaft_walks = builder.add_coedge(seam_circle, on_the_shaft)?;
    let down = builder.add_coedge(up_the_shaft, Orientation::Reversed)?;
    let round_the_shaft = builder.add_wire(&[
        DraftStep::Along(round_the_floor),
        DraftStep::Along(up),
        DraftStep::Along(shaft_walks),
        DraftStep::Along(down),
    ])?;
    let shaft = builder.add_face(pieces.shaft, &[round_the_shaft])?;

    let under_the_floor = builder.add_coedge(floor_circle, Orientation::Reversed)?;
    let across_the_bottom = builder.add_wire(&[DraftStep::Along(under_the_floor)])?;
    let floor = builder.add_face(
        Surface::Plane(
            Plane::from_point_and_normal(pieces.bottom, Vector::new(0.0, 0.0, -1.0))
                .expect("the z axis is a unit normal"),
        ),
        &[across_the_bottom],
    )?;

    builder.add_shell(&[
        (dome, Orientation::Forward),
        (shaft, Orientation::Forward),
        (floor, Orientation::Forward),
    ])?;
    Ok(builder)
}

/// The piece of the dome's own meridian circle between the seam corner and the
/// pole.
///
/// The plane through the centre facing `y` cuts the dome in one whole circle,
/// and the two half-spaces cut that down to the quarter with `x` past the
/// centre and `z` past the seam.
fn the_meridian(pieces: &Pieces) -> EdgeRun {
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(
            Plane::from_point_and_normal(pieces.centre, Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal"),
        ),
        picked_by: smallvec::smallvec![
            Picker::from(
                HalfSpace::through_a_point_facing(pieces.centre, Vector::new(1.0, 0.0, 0.0))
                    .expect("the x axis has length")
            ),
            Picker::from(
                HalfSpace::through_a_point_facing(pieces.seam, Vector::new(0.0, 0.0, 1.0))
                    .expect("the z axis has length")
            ),
        ],
    }
}
