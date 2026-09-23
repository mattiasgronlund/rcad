//! What a cone's tip costs the published bounds, and that it costs something
//! finite.
//!
//! A pointed cone's curved face reaches its apex, where the radius of normal
//! curvature is nothing at all. The shell argument behind
//! `Measurement::volume_error` and `Measurement::area_error` needs an inner
//! offset surface to project onto, and within `deviation` of the apex there is
//! none — so both bounds came back `inf` for every solid holding a cone's apex.
//! `inf` is honest and it is useless: a measured volume with an infinite bound
//! cannot be held against anything.
//!
//! What closes it is splitting the cone face at
//! `v = 2·deviation·√(1 + slope²)/slope` and charging the tip below that split
//! its **whole** size rather than a share of it. These tests hold the answer
//! against a shape whose volume and area are known in closed form, so a bound
//! that is merely finite is not enough to pass them.

use rcad_geom::{Deflection, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Measurement, Model};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// `π·r²·h/3` for the cone `a_pointed_cone(SLOPE, HEIGHT)` describes.
fn the_cones_own_volume() -> f64 {
    let radius = SLOPE * HEIGHT;
    core::f64::consts::PI * radius * radius * HEIGHT / 3.0
}

/// `π·r·l` round the side and `π·r²` across the cap.
fn the_cones_own_surface() -> f64 {
    let radius = SLOPE * HEIGHT;
    let slant = radius.hypot(HEIGHT);
    core::f64::consts::PI * radius * (slant + radius)
}

fn measure_a_pointed_cone(deflection: f64) -> Measurement {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    model
        .solid(cone)
        .expect("this model made the key")
        .measure(
            Deflection::of(deflection).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a pointed cone measures")
}

/// The bare pointed cone of `docs/decisions.md` §200.3, at the deflection that
/// file measures at. Its volume came back `134.02461503321067` with a bound of
/// `inf`; the bound is finite now, and the true `128π/3` is inside it.
#[test]
fn a_pointed_cones_measured_volume_is_inside_a_finite_bound() {
    let measured = measure_a_pointed_cone(0.002);
    let truth = the_cones_own_volume();
    assert!(
        measured.volume_error().is_finite(),
        "the bound is {}, on a volume of {}",
        measured.volume_error(),
        measured.volume()
    );
    assert!(
        (measured.volume() - truth).abs() <= measured.volume_error(),
        "measured {} against πr²h/3 = {truth}, off by {}, bound {}",
        measured.volume(),
        (measured.volume() - truth).abs(),
        measured.volume_error()
    );
}

/// The same for the skin: `π·r·l` round the side and `π·r²` across the cap.
#[test]
fn a_pointed_cones_measured_surface_is_inside_a_finite_bound() {
    let measured = measure_a_pointed_cone(0.002);
    let truth = the_cones_own_surface();
    assert!(
        measured.area_error().is_finite(),
        "the bound is {}, on an area of {}",
        measured.area_error(),
        measured.surface_area()
    );
    assert!(
        (measured.surface_area() - truth).abs() <= measured.area_error(),
        "measured {} against πr(l + r) = {truth}, off by {}, bound {}",
        measured.surface_area(),
        (measured.surface_area() - truth).abs(),
        measured.area_error()
    );
}

/// A bound that is finite but not earned would sit still as the mesh got
/// finer. This one shrinks, and the measured answer walks in towards the
/// closed form with it.
#[test]
fn the_bound_on_a_pointed_cone_shrinks_as_the_deflection_does() {
    let truth = the_cones_own_volume();
    let mut coarser: Option<Measurement> = None;
    let mut bounds = Vec::new();
    for deflection in [0.008, 0.002, 0.000_5] {
        let finer = measure_a_pointed_cone(deflection);
        assert!(
            (finer.volume() - truth).abs() <= finer.volume_error(),
            "at {deflection}: measured {} against {truth}, bound {}",
            finer.volume(),
            finer.volume_error()
        );
        if let Some(coarser) = coarser {
            assert!(
                finer.volume_error() < coarser.volume_error(),
                "at {deflection} the bound is {}, no better than {} one step coarser",
                finer.volume_error(),
                coarser.volume_error()
            );
            assert!(
                (finer.volume() - truth).abs() < (coarser.volume() - truth).abs(),
                "at {deflection} the answer is {}, no nearer {truth} than {}",
                finer.volume(),
                coarser.volume()
            );
        }
        bounds.push(finer.volume_error());
        coarser = Some(finer);
    }
    assert_eq!(bounds.len(), 3, "three deflections were measured");
}

/// A frustum of the same cone keeps the bound it always had: its smallest
/// radius is at its near cap, far outside the tip, so nothing here touches it.
#[test]
fn a_frustum_of_the_same_cone_is_bounded_by_its_near_cap_as_before() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let frustum = cones::a_frustum(&mut model, SLOPE, 4.0, HEIGHT);
    let measured = model
        .solid(frustum)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.002).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a frustum measures");
    let truth = {
        let (near, far) = (SLOPE * 4.0, SLOPE * HEIGHT);
        core::f64::consts::PI * (far * far * HEIGHT - near * near * 4.0) / 3.0
    };
    assert!(
        (measured.volume() - truth).abs() <= measured.volume_error(),
        "measured {} against {truth}, bound {}",
        measured.volume(),
        measured.volume_error()
    );
    // The near cap sits at a radius of normal curvature of `slope·4·√1.25 =
    // 2.236`, hundreds of deviations clear of the split at `2·d·(1 + slope²) =
    // 0.005` — so no tip is charged here, and the bound is the shell over the
    // side alone: its area `π(2 + 4)·√20` times very nearly the deviation. A
    // tip charged to this frustum would show up as a bound far over that.
    let side = core::f64::consts::PI * 6.0 * 20.0f64.sqrt();
    assert!(
        (0.99 * side * 0.002..1.01 * side * 0.002).contains(&measured.volume_error()),
        "a frustum's bound is {}, where the shell over its side alone is {}",
        measured.volume_error(),
        side * 0.002
    );
}
