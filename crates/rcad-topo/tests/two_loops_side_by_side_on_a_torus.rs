//! Two circles round the tube of one ring bound two bands of it, and a torus
//! face between them is one band or the other. Which one is said only by which
//! way the face walks its two circles.
//!
//! Each band here is built through [`SolidBuilder`] as a closed solid: the
//! band and a flat cap on each circle's plane through the axis. The circles are
//! at `0` and at a quarter or half turn round the ring, and each band is walked
//! from the first circle round to the second anticlockwise seen from `+z`, or
//! clockwise. By Pappus a band over the ring angle `φ` holds `φ·R·π·r²`.
//!
//! **The mesher used to put the second circle the nearer way round the ring
//! from the first**, whichever way the face walked them (§280.8). Between two
//! circles half a turn apart that is a tie, broken towards the lower half. The
//! lower half built, and the upper half was refused
//! `FacesDoNotAgreeWhichWayTheyFace`: its mesh was the lower half's, with the
//! upper half's caps. `docs/decisions.md` §343.
//!
//! **The three-quarter band reaches the mesher through the builder** since
//! §392. `Model::validate` refused it `AClosedEdgeIsWalkedAgainstItsSurface`,
//! reading which side of each cap's plane the wall keeps off the far circle's
//! corner, which past half the ring is the wrong side. It now holds the wall's
//! two circles to being walked opposite ways and each cap to lying inside the
//! tube, and neither reads a corner.
//!
//! Every bend here is built the way `tests/rings/mod.rs` builds one, with the
//! wall used forwards, as a torus faces out of its tube, and the seam left out
//! (§329, §361).

use rcad_geom::{
    Axis, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Result, SolidBuilder, SolidKey};

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;

/// Three deflections, the coarsest the working size's own.
const DEFLECTIONS: [f64; 3] = [0.1, 0.03, 0.01];

/// Both halves of a ring between the circles at `0` and at half a turn build,
/// validate, and measure `4π²`, each on its own side of the plane `y = 0`.
///
/// **The volume cannot tell the two halves apart**, since they are the same
/// shape; the centroid can. The upper half lies wholly in `y ≥ 0` and so its
/// centroid is at `y > 0`, and the lower half's at `y < 0`.
#[test]
fn both_halves_of_a_ring_between_two_circles_half_a_turn_apart_are_the_half_they_are() {
    let policy = Policy::certified();
    for (way, above_or_below) in [(Way::Anticlockwise, 1.0), (Way::Clockwise, -1.0)] {
        let mut model = Model::new(a_working_size());
        let band = match a_band(&mut model, [-1.0, 0.0], way) {
            Ok(band) => band,
            Err(refused) => panic!("the {way:?} half was refused: {refused:?}"),
        };
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .expect("the model holding the half is sound");
        for length in DEFLECTIONS {
            let measured = model
                .solid(band)
                .expect("this model made the key")
                .measure(
                    Deflection::of(length).expect("a length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a half ring measures");
            as_pappus_says(measured.volume(), measured.volume_error(), 2.0, length);
            let y = measured.centroid().y();
            assert!(
                y * above_or_below > 0.0,
                "the {way:?} half at {length} has its centroid at y = {y}"
            );
        }
    }
}

/// The quarter of a ring between the circles at `0` and at a quarter turn
/// builds, validates and measures `2π²`.
#[test]
fn the_quarter_band_between_two_circles_a_quarter_turn_apart_measures_what_pappus_says() {
    let policy = Policy::certified();
    let mut model = Model::new(a_working_size());
    let band = a_band(&mut model, [0.0, 1.0], Way::Anticlockwise)
        .expect("the quarter band between two circles is a solid");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the model holding the quarter is sound");
    for length in DEFLECTIONS {
        let measured = model
            .solid(band)
            .expect("this model made the key")
            .measure(
                Deflection::of(length).expect("a length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a quarter band measures");
        as_pappus_says(measured.volume(), measured.volume_error(), 1.0, length);
    }
}

/// The three quarters of the same ring between the same two circles builds,
/// validates and measures `6π²`.
///
/// It is §243.3's three-quarter bend with its seam left out. The sweep used to
/// refuse it `AClosedEdgeIsWalkedAgainstItsSurface`: it read which side of each
/// cap's plane the wall keeps off the other circle's corner, which for a band
/// past half the ring is the side it does not keep. A wall's two circles round
/// the tube are now held to being walked opposite ways, and each cap to lying
/// inside the tube (§392).
#[test]
fn the_three_quarter_band_between_the_same_two_circles_measures_what_pappus_says() {
    let policy = Policy::certified();
    let mut model = Model::new(a_working_size());
    let band = a_band(&mut model, [0.0, 1.0], Way::Clockwise)
        .expect("the three-quarter band between two circles is a solid");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the model holding the three quarters is sound");
    for length in DEFLECTIONS {
        let measured = model
            .solid(band)
            .expect("this model made the key")
            .measure(
                Deflection::of(length).expect("a length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a three-quarter band measures");
        as_pappus_says(measured.volume(), measured.volume_error(), 3.0, length);
    }
}

/// A wall walking both its circles the same way round the tube is refused
/// `AClosedEdgeIsWalkedAgainstItsSurface`, whichever way round the ring the
/// caps face.
///
/// Each cap faces the way its own circle's walk says and so is sound on its
/// own: what is wrong is the wall, whose first circle says the band runs on
/// round the ring from it and whose second says the same of itself. No band
/// is both. §343.3 left this unrun.
#[test]
fn a_wall_walking_both_its_circles_the_same_way_round_the_tube_is_refused() {
    for way in [Way::Anticlockwise, Way::Clockwise] {
        let mut model = Model::new(a_working_size());
        let refused = a_band_walked(&mut model, [0.0, 1.0], way, Walks::TheSameWay);
        assert!(
            matches!(
                refused,
                Err(Error::Invalid(
                    Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                ))
            ),
            "{way:?}: the wall walking both circles one way came back {refused:?}"
        );
        assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
    }
}

/// Which way round the ring a band runs from the circle at `0` to the other
/// one, seen from `+z`.
#[derive(Clone, Copy, Debug)]
enum Way {
    Anticlockwise,
    Clockwise,
}

/// How a band's wall walks its second circle against its first.
#[derive(Clone, Copy)]
enum Walks {
    /// The other way, as every band does.
    Opposite,
    /// The same way, with the second cap turned to face the way its circle is
    /// then walked round it.
    TheSameWay,
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// Whether a volume is Pappus's for `quarters` quarter turns of the ring,
/// within what its measurement says it can be out by.
fn as_pappus_says(volume: f64, error: f64, quarters: f64, length: f64) {
    let pappus =
        quarters * core::f64::consts::FRAC_PI_2 * RING * core::f64::consts::PI * TUBE * TUBE;
    assert!(
        (volume - pappus).abs() <= error,
        "{quarters} quarters at {length}: {volume} where Pappus gives {pappus}, allowed {error}"
    );
}

/// The band of the ring about `+z` between the circle round `(RING, 0, 0)` and
/// the one round `RING·(cos θ, sin θ, 0)`, `second` being `(cos θ, sin θ)`
/// written out exactly, with a flat cap on each circle's plane.
///
/// Anticlockwise is `tests/rings/mod.rs`'s bend from `+x` with its seam left
/// out: the first cap faces `(0, −1, 0)`, the second faces on round the ring,
/// the wall walks the first circle backward and the second forward and is used
/// forwards. Clockwise is the rest of the ring between the same two circles:
/// both caps face the other way and every coedge on a circle is turned over.
fn a_band(model: &mut Model, second: [f64; 2], way: Way) -> Result<SolidKey> {
    a_band_walked(model, second, way, Walks::Opposite)
}

/// [`a_band`] with its wall walking the second circle as `walks` says.
fn a_band_walked(
    model: &mut Model,
    [cos, sin]: [f64; 2],
    way: Way,
    walks: Walks,
) -> Result<SolidKey> {
    let inner = RING - TUBE;
    let mut builder = SolidBuilder::new();
    let first_corner = builder.add_vertex(Point::new(inner, 0.0, 0.0))?;
    let second_corner = builder.add_vertex(Point::new(cos * inner, sin * inner, 0.0))?;
    let first_circle = builder.add_edge(
        first_corner,
        first_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let second_circle = builder.add_edge(
        second_corner,
        second_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let (facing, first_on_the_wall) = match way {
        Way::Anticlockwise => (1.0, Orientation::Reversed),
        Way::Clockwise => (-1.0, Orientation::Forward),
    };
    let (second_on_the_wall, second_facing) = match walks {
        Walks::Opposite => (first_on_the_wall.reversed(), facing),
        Walks::TheSameWay => (first_on_the_wall, -facing),
    };
    let cap = |builder: &mut SolidBuilder, facing: Vector, circle, turn| -> Result<_> {
        let plane = Plane::from_point_and_normal(Point::origin(), facing)
            .expect("a coordinate direction is a unit normal");
        let round_it = builder.add_coedge(circle, turn)?;
        let wire = builder.add_wire(&[DraftStep::Along(round_it)])?;
        builder.add_face(Surface::Plane(plane), &[wire])
    };
    let first_cap = cap(
        &mut builder,
        Vector::new(0.0, -facing, 0.0),
        first_circle,
        first_on_the_wall.reversed(),
    )?;
    let second_cap = cap(
        &mut builder,
        Vector::new(
            written_out(-sin * second_facing),
            written_out(cos * second_facing),
            0.0,
        ),
        second_circle,
        second_on_the_wall.reversed(),
    )?;

    let mut wires = Vec::with_capacity(2);
    for (circle, turn) in [
        (first_circle, first_on_the_wall),
        (second_circle, second_on_the_wall),
    ] {
        let round_it = builder.add_coedge(circle, turn)?;
        wires.push(builder.add_wire(&[DraftStep::Along(round_it)])?);
    }
    let ring = Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower than its ring");
    let wall = builder.add_face(Surface::Torus(ring), &wires)?;

    builder.add_shell(&[
        (first_cap, Orientation::Forward),
        (second_cap, Orientation::Forward),
        (wall, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// `−0.0` written as `+0.0`, which is how a person writing a zero out writes it.
fn written_out(number: f64) -> f64 {
    if number == 0.0 { 0.0 } else { number }
}
