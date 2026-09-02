//! A solid with a face on a torus, built through the public door.
//!
//! A **quarter bend of pipe**: a quarter of a torus, capped at each end by a
//! plane through the torus's own axis. It is the shape a mechanical file holds
//! and it is the shape the two crossing families a torus has are exactly enough
//! for — see `rings/mod.rs` for why a bend has a bar's structure in its own
//! chart and what about it is not a bar's.

use rcad_geom::{Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::Model;

mod rings;

use rings::a_quarter_bend;

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// A quarter bend built face by face is a sound solid.
#[test]
fn a_quarter_bend_built_face_by_face_is_a_solid() {
    let mut model = Model::new(a_working_size());
    let solid = a_quarter_bend(&mut model, 4.0, 1.0);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bend");

    let view = model.solid(solid);
    assert_eq!(view.faces().count(), 3, "two caps and one torus face");
    let on_a_ring = view
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Torus(_)))
        .count();
    assert_eq!(on_a_ring, 1);
    assert_eq!(view.shells().count(), 1);
}

/// A quarter bend measures its closed form.
///
/// A whole torus has volume `2π²Rr²` and area `4π²Rr` — Pappus's theorem, the
/// tube's cross-section swept round the centre circle — so a quarter of one has
/// a quarter of each, plus its two flat caps of `πr²` apiece.
///
/// **Compared against the closed form and the kernel's own published bound**,
/// not against a number recorded from a previous run: a measurement checked
/// against itself checks nothing.
#[test]
fn a_quarter_bend_measures_the_closed_form_within_the_published_bound() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let (ring, tube) = (4.0f64, 1.0f64);
    let solid = a_quarter_bend(&mut model, ring, tube);
    model.validate(&policy, &mut log).expect("a sound bend");

    let measured = model
        .solid(solid)
        .measure(
            Deflection::of(0.002).expect("a positive length"),
            &policy,
            &mut log,
        )
        .expect("a bend measures");

    let pi = core::f64::consts::PI;
    let volume = 2.0 * pi * pi * ring * tube * tube / 4.0;
    let area = 4.0 * pi * pi * ring * tube / 4.0 + 2.0 * pi * tube * tube;

    println!(
        "bend: volume {} against {volume} with a bound of {}, area {} against {area} with a \
         bound of {}",
        measured.volume(),
        measured.volume_error(),
        measured.surface_area(),
        measured.area_error(),
    );
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the mesh's volume {} misses 2π²Rr²/4 = {volume} by more than the bound {}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.surface_area() - area).abs() <= measured.area_error(),
        "the mesh's area {} misses 4π²Rr/4 + 2πr² = {area} by more than the bound {}",
        measured.surface_area(),
        measured.area_error()
    );

    // **The centroid is not the tube's centre circle's**, and the difference is
    // the whole of what makes this worth asserting. Each ring of the tube at
    // distance `s` from the axis sweeps a volume proportional to `s`, so the
    // outer half of the tube counts for more than the inner: the effective
    // radius is `∫s² dA / ∫s dA = R + r²/(4R)` over the tube's cross-section,
    // not `R`. Swept a quarter turn that puts the centroid at
    // `2(R² + r²/4)/(πR)` along each of the two axes and nowhere at all up the
    // third.
    let centroid = Point::new(
        2.0 * (ring * ring + tube * tube / 4.0) / (pi * ring),
        2.0 * (ring * ring + tube * tube / 4.0) / (pi * ring),
        0.0,
    );
    println!(
        "bend: centroid {:?} against {centroid:?} with a wobble of {}",
        measured.centroid(),
        measured.centroid_wobble()
    );
    assert!(
        measured.centroid().distance_to(centroid) <= measured.centroid_wobble(),
        "the centroid {:?} is not {centroid:?} to within {}",
        measured.centroid(),
        measured.centroid_wobble()
    );
}

/// A bend a long way from the origin measures what one at the origin does, to
/// within the bound the kernel publishes — **and a round bar moved the same way
/// does eight orders of magnitude better.**
///
/// §19's rule is that where a solid sits may not change what it measures, and
/// bit for bit is what that means. A bend three thousand units out does not
/// manage it, and the control says that is a **torus's** problem and not the
/// standing one:
///
/// | moved to `(1000, −2000, 3000)` | volume slipped by |
/// |---|---|
/// | a quarter bend, ring 4 tube 1 | `1.7e−5` |
/// | a round bar, radius 1 height 2, centred the same way | `1.8e−13` |
///
/// Both stay inside their own published bounds, so neither is a wrong answer,
/// and the bend's is a **gap**: what it costs is a measurement that moves in its
/// sixth significant figure when the solid does. Where to look, in order: a
/// bend's boundary is walked as **two circles and an arc of a third**, and the
/// arc carries pickers that `EdgeRun::shifted` has to carry with it, where a
/// bar's seam is `Straight` and carries nothing; and `how_far_the_points_of_reach`
/// charges a torus its whole `R + r` where a bar is charged its radius, which
/// feeds `how_far_moving_this_face_rounds` and so the room the walk is given.
/// Neither is measured here, and this test is the thing that would notice if the
/// figure moved.
#[test]
fn a_bend_moved_far_out_stays_inside_its_bound_and_slips_further_than_a_bar() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let deflection = Deflection::of(0.005).expect("a positive length");

    let mut here = Model::new(a_working_size());
    let at_the_origin = a_quarter_bend(&mut here, 4.0, 1.0);
    let measured_here = here
        .solid(at_the_origin)
        .measure(deflection, &policy, &mut log)
        .expect("a bend measures");

    let mut there = Model::new(a_working_size());
    let moved = a_quarter_bend(&mut there, 4.0, 1.0);
    there.place_solid(
        moved,
        &Isometry::translation(Vector::new(1000.0, -2000.0, 3000.0)).expect("a finite shift"),
    );
    let measured_there = there
        .solid(moved)
        .measure(deflection, &policy, &mut log)
        .expect("a moved bend measures");

    let bend_slipped = (measured_here.volume() - measured_there.volume()).abs();

    let mut bar_here = Model::new(a_working_size());
    let straight = bar_here
        .add_round_bar(
            1.0,
            2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a finite shift"),
        )
        .expect("a bar");
    let bar_measured_here = bar_here
        .solid(straight)
        .measure(deflection, &policy, &mut log)
        .expect("a bar measures");
    let mut bar_there = Model::new(a_working_size());
    let moved_bar = bar_there
        .add_round_bar(
            1.0,
            2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a finite shift"),
        )
        .expect("a bar");
    bar_there.place_solid(
        moved_bar,
        &Isometry::translation(Vector::new(1000.0, -2000.0, 3000.0)).expect("a finite shift"),
    );
    let bar_measured_there = bar_there
        .solid(moved_bar)
        .measure(deflection, &policy, &mut log)
        .expect("a moved bar measures");
    let bar_slipped = (bar_measured_here.volume() - bar_measured_there.volume()).abs();

    println!(
        "moved three thousand units out, a bend's volume slipped by {bend_slipped} and a round \
         bar's by {bar_slipped}"
    );
    assert!(
        bend_slipped <= measured_here.volume_error(),
        "a moved bend slipped {bend_slipped}, past its own published bound {}",
        measured_here.volume_error()
    );
    assert!(
        bar_slipped < bend_slipped,
        "the control is here to say a bend slips further than a bar, and it slipped \
         {bar_slipped} against {bend_slipped}"
    );
}
