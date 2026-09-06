//! Is the mesh point at a placed cone's apex bit-for-bit the apex its own face
//! reports?
//!
//! **A consumer is already betting on it.** rscene shades a cone by asking the
//! surface for a normal, and a cone has none at its apex — so its apex triangle
//! is special-cased on **exact** equality against `Cone::apex()`, deliberately.
//! A tolerance there would hand the wedge middle back for a mesh point that is
//! genuinely a whisker off the apex and sits on a real ruling, which is a
//! confidently wrong normal rather than a refusal. If the mesh point misses the
//! apex by an ulp the special case does not fire, the old gradient path answers,
//! and the answer is a garbage radial direction. That is the failure mode this
//! kernel exists to avoid, so the equality it rests on is rcad's to demonstrate
//! or to deny.
//!
//! # Why it was in doubt, and it was only in doubt for a *placed* face
//!
//! The two numbers arrive by different routes.
//!
//! - **The apex a consumer reads** is `FaceView::surface`, which composes the
//!   placement in: `Cone::transformed` carries the axis origin through
//!   `Isometry::apply_to_point`, once.
//! - **The apex the mesh emits** is worked out where the face is *stored*.
//!   `the_surface_this_face_is_meshed_on` reads a placed face's surface out of
//!   the arena rather than moving it into the meshing frame and back, and
//!   `Cone::point_at(u, 0)` on it is the stored apex exactly — the two terms it
//!   adds are both zero vectors. That point is then carried out to where the
//!   solid sits, and the movement it rides used to be
//!   `the_frame_this_face_is_meshed_in(face).inverse()` — which for a placed face
//!   is the placement **inverted twice**.
//!
//! `Isometry::inverse` is not a bit-for-bit involution. It conjugates the
//! quaternion, which only flips signs, but it replaces the shift with `−R⁻¹t`,
//! which turns a vector and rounds; inverting again turns it back and rounds
//! again. A pure shift survives that, and so does a pure turn about an axis
//! through the origin, whose shift is zero and stays zero. **A turn with a shift
//! on it is the case with nothing protecting it**, and it is the ordinary case:
//! it is what `Model::place_solid` is for and what an assembly out of a STEP
//! file is made of. `back_out_of_the_frame_this_face_is_meshed_in` rides a placed
//! point out on the placement itself, which is the frame's exact inverse.
//!
//! An unplaced face is a different question and already settled: its meshing
//! frame is `a_shift_that_loses_nothing`, whose subtractions meet Sterbenz's
//! condition, and
//! `taking_the_smallest_of_a_run_of_numbers_off_all_of_them_loses_no_bits` is
//! that demonstration. It is here as the control row.
//!
//! # The oracle is exact equality and nothing weaker
//!
//! Both sides are `f64` triples that either are the same bits or are not, so
//! this is the top row of the testing skill's table. The gap is printed, but it
//! is not what is asserted.
//!
//! # What stops it passing for the wrong reason
//!
//! Three things, and the third is the one that matters.
//!
//! 1. **A mesh with no point near the apex** would satisfy "no point is wrong"
//!    trivially. Every placement first has to put a point within a whisker of
//!    the apex — a whisker being `Isometry::POINT_ROUNDOFF` of how far the placed
//!    cone's own numbers go, which is the most two routes to one point can differ
//!    by if the arithmetic is behaving at all.
//! 2. **Every family is counted** and each is asserted to have run.
//! 3. **The families are told apart by measurement, not by assertion.** Each
//!    placement is also asked what inverting it twice *would* have cost, and the
//!    counts are checked against the three sentences above: no pure shift moves,
//!    no pure turn about an axis through the origin moves, and a real share of
//!    the turns with shifts on them do. Without that last count this file could
//!    go green over a family of placements that never exercised the bug, and it
//!    would look exactly the same.

mod cones;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, SolidKey};

use cones::a_pointed_cone;

/// The cone every row is built from: slope a half, base circle eight up the
/// axis, so it reaches four out and eight along from an apex at the origin.
const SLOPE: f64 = 0.5;
const HOW_FAR_UP_THE_BASE_SITS: f64 = 8.0;

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// A pointed cone, put where `placement` says.
fn a_pointed_cone_placed(placement: &Isometry) -> (Model, SolidKey) {
    let mut model = Model::new(a_working_size());
    let solid = a_pointed_cone(&mut model, SLOPE, HOW_FAR_UP_THE_BASE_SITS);
    model.place_solid(solid, placement);
    (model, solid)
}

/// The apex of the solid's one cone face, read the way a consumer reads it:
/// through a located [`FaceView`](rcad_topo::FaceView), placement composed in.
fn the_apex_a_consumer_reads(model: &Model, solid: SolidKey) -> Point {
    let mut apexes = model.solid(solid).faces().filter_map(|face| {
        match face.surface() {
            Surface::Cone(cone) => Some(cone.apex()),
            // A pointed cone's other face is its base cap. The kinds this kernel
            // holds are named out so that a sixth surface arriving here is a
            // panic rather than a silent "not a cone".
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Torus(_) => {
                None
            }
            other => panic!("a pointed cone has no {other:?} face"),
        }
    });
    let apex = apexes.next().expect("a pointed cone has a cone face");
    assert!(apexes.next().is_none(), "and it has only the one");
    apex
}

/// How far apart the two ways out of the meshing frame put the apex: the
/// placement itself, against the placement inverted twice.
///
/// Zero says this placement could not have caught the bug, whatever else the row
/// proves.
fn what_inverting_the_placement_twice_moves_the_apex_by(placement: &Isometry) -> f64 {
    let stored_apex = Point::origin();
    placement
        .inverse()
        .inverse()
        .apply_to_point(stored_apex)
        .distance_to(placement.apply_to_point(stored_apex))
}

/// One family of placements, meshed and checked.
///
/// Hands back how many of them the doubled inversion would have moved the apex
/// on, which is what says whether the family exercised anything.
fn every_placement_puts_a_mesh_point_on_the_apex(family: &str, placements: &[Isometry]) -> usize {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut the_doubled_inversion_would_have_moved = 0;
    let mut missed = Vec::new();

    for placement in placements {
        let (model, solid) = a_pointed_cone_placed(placement);
        let apex = the_apex_a_consumer_reads(&model, solid);
        let mesh = model
            .solid(solid)
            .mesh(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut log,
            )
            .expect("a pointed cone meshes at a hundredth wherever it is put");

        let nearest = mesh
            .points()
            .iter()
            .copied()
            .min_by(|a, b| {
                a.distance_squared_to(apex)
                    .total_cmp(&b.distance_squared_to(apex))
            })
            .expect("a mesh of a solid has points");
        let gap = nearest.distance_to(apex);
        // How far the placed cone's own numbers go, which is what a movement of
        // one of its points rounds against — the apex plus the cone's own reach,
        // because a turn can bring any of it up against any coordinate.
        let reach = apex.largest_coordinate_magnitude() + HOW_FAR_UP_THE_BASE_SITS;
        let whisker = Isometry::POINT_ROUNDOFF * reach;
        assert!(
            gap <= whisker,
            "{family}: the nearest mesh point to the apex {:?} is {:?}, {gap} away and past the \
             {whisker} two movements of one point can cost — so this placement is not measuring \
             the apex at all",
            apex.coordinates(),
            nearest.coordinates(),
        );

        if what_inverting_the_placement_twice_moves_the_apex_by(placement) > 0.0 {
            the_doubled_inversion_would_have_moved += 1;
        }
        if !mesh.points().contains(&apex) {
            missed.push((apex.coordinates(), nearest.coordinates(), gap));
        }
    }

    println!(
        "{family}: {} placements, {the_doubled_inversion_would_have_moved} of them ones the \
         doubled inversion moves the apex on",
        placements.len(),
    );
    assert!(
        missed.is_empty(),
        "{family}: {} of {} placements put no mesh point on the apex, so a consumer keying on \
         exact equality against Cone::apex() shades the tip with a gradient normal that is \
         radial where the surface has no normal at all. The first few, as \
         (apex, nearest point, how far apart): {:?}",
        missed.len(),
        placements.len(),
        &missed[..missed.len().min(4)],
    );
    the_doubled_inversion_would_have_moved
}

fn some_axes_through_the_origin() -> Vec<Axis> {
    [
        Vector::new(0.0, 0.0, 1.0),
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(1.0, 2.0, 3.0),
        Vector::new(-3.0, 1.0, 0.5),
    ]
    .into_iter()
    .map(|direction| {
        Axis::new(
            Point::origin(),
            direction.normalized().expect("a direction with length"),
        )
        .expect("a unit direction is an axis")
    })
    .collect()
}

fn some_shifts() -> Vec<Vector> {
    vec![
        Vector::new(3.0, 5.0, 7.0),
        Vector::new(-11.0, 0.25, 2.5),
        Vector::new(1.0e6, -2.0e6, 4.0e6),
        Vector::new(0.125, 0.0, -0.0625),
    ]
}

#[test]
fn a_cone_that_was_never_placed_meshes_a_point_on_its_own_apex() {
    let moved = every_placement_puts_a_mesh_point_on_the_apex(
        "no placement at all",
        &[Isometry::identity()],
    );
    assert_eq!(
        moved, 0,
        "the identity is its own inverse to the bit, so this row cannot catch a doubled \
         inversion and is here as the control"
    );
}

#[test]
fn a_shifted_cone_meshes_a_point_exactly_on_the_apex_its_face_reports() {
    let placements: Vec<Isometry> = some_shifts()
        .into_iter()
        .map(|shift| Isometry::translation(shift).expect("a finite shift"))
        .collect();
    let moved = every_placement_puts_a_mesh_point_on_the_apex("a shift alone", &placements);
    assert_eq!(
        moved, 0,
        "a shift has the identity turn, so inverting it twice negates its vector twice and \
         gets the same bits back — if this ever counts one, the sentence in \
         back_out_of_the_frame_this_face_is_meshed_in about a pure shift is wrong"
    );
}

#[test]
fn a_turned_cone_meshes_a_point_exactly_on_the_apex_its_face_reports() {
    let mut placements = Vec::new();
    for axis in some_axes_through_the_origin() {
        for turn in [0.7, 1.0, 2.5, -0.3, core::f64::consts::FRAC_PI_2] {
            placements.push(Isometry::rotation(&axis, turn).expect("a finite angle"));
        }
    }
    let moved = every_placement_puts_a_mesh_point_on_the_apex("a turn alone", &placements);
    assert_eq!(
        moved, 0,
        "a turn about an axis through the origin has a zero shift, and `−R⁻¹·0` is zero \
         whatever the turn — if this ever counts one, the sentence about a pure turn is wrong"
    );
}

#[test]
fn a_turned_and_shifted_cone_meshes_a_point_exactly_on_the_apex_its_face_reports() {
    let mut placements = Vec::new();
    for axis in some_axes_through_the_origin() {
        for turn in [0.7, 1.0, 2.5, -0.3, core::f64::consts::FRAC_PI_2] {
            let turned = Isometry::rotation(&axis, turn).expect("a finite angle");
            for shift in some_shifts() {
                let shifted = Isometry::translation(shift).expect("a finite shift");
                placements.push(shifted.compose(&turned));
            }
        }
    }
    let total = placements.len();
    let moved =
        every_placement_puts_a_mesh_point_on_the_apex("a turn and then a shift", &placements);
    assert!(
        moved * 4 >= total,
        "only {moved} of {total} turns-with-a-shift move the apex when the placement is \
         inverted twice, so this family barely exercises the thing it is here to catch"
    );
}
