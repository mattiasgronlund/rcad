//! Why a unit ball asked for a *coarser* mesh sometimes comes back with a
//! *finer* one, pinned deflection by deflection.
//!
//! **Nothing here is a broken promise.** At every deflection measured below the
//! sag stays under what [`Mesh::deviation`] said it would be, and that stays
//! under the deflection asked for. What steps about is *how much better than
//! promised* the answer is, and it does not step downwards as the ask gets
//! coarser: a unit ball at a deflection of 0.700 sags 31.22% of its radius and
//! at 0.750 it sags 22.54%.
//!
//! **Where the steps come from.** Three whole numbers decide a ball's mesh, and
//! they change at three different places:
//!
//! 1. How many steps the **seam** — the meridian the model carries as an edge —
//!    is refined into. That is the curve evaluator's `ceil`, and on a half turn
//!    it drops from 4 steps to 3 at a deflection of `(1 − cos(π/6)) / 0.1875`.
//! 2. How many steps each **cut** — the three meridians `mesh.rs` puts in to
//!    keep a patch inside a quarter turn — is walked in. That is
//!    `how_many_steps`' `ceil`, and it drops from 4 to 3 at `π² / (1.5 · 3²)`.
//! 3. Whether the triangulator has to put a point back **in the middle of a
//!    cell**. A cell of the latitude-by-longitude grid is **charged**
//!    `chord² / (3 · radius)` across its diagonal, and when that charge is over
//!    the deflection a point goes in the middle of it.
//!
//! **That third one is a charge and not a measurement, and the difference is a
//! factor of two.** The equatorial cell below is charged `2.5 / 3 = 0.8333` of
//! the radius. What it really strays is `0.3876`: its four corners
//! `(±30°, 0°)` and `(±30°, 90°)` are coplanar — they all satisfy
//! `x + y = cos 30°` — that plane is `cos 30° / √2 = 0.6124` from the centre,
//! and `1 − 0.6124` is what the surface bulges past it. So the repair fires
//! `2.15` times earlier than the geometry alone would ask, and the thresholds
//! below are where the *charge* crosses the deflection, which is what the code
//! tests and therefore what the mesh does.
//!
//! (1) and (2) are within 0.017 of each other but not equal, so there is a
//! window where the seam is cut into 3 and the cuts are still cut into 4. Worse,
//! (2) firing makes each cell **taller**, its diagonal longer, and its own
//! charge larger — large enough to trip (3), which then puts four points back.
//! That is the whole of the wrinkle: coarsening the grid by one row made the
//! cells miss their own charge, and the repair overshoots what the finer grid
//! gave.
//!
//! **This test is written to fail the day the row count changes.** It pins a
//! wrinkle, not a promise. If the steps are ironed out — the grid sized so that
//! it never has to be repaired, say — every table below is wrong and should be
//! replaced rather than loosened. The two assertions worth keeping in that case
//! are the ones about the promise, which are in
//! `how_far_a_balls_mesh_points_really_sit_from_the_ball.rs` as well.

use rcad_geom::{Deflection, Isometry, Point, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Mesh, Model, SolidKey};

/// A unit ball asked for a coarser mesh at 0.750 than at 0.700 comes back
/// closer to the sphere, not further from it.
///
/// The two numbers a downstream report quoted. They are measured here from the
/// ball's own centre and radius, and both of them keep the promise: the sag is
/// under the stated deviation and the stated deviation is under the deflection.
#[test]
fn a_unit_ball_sags_less_at_a_deflection_of_three_quarters_than_at_seven_tenths() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model.add_ball(1.0, &Isometry::identity()).expect("a ball");

    let sags = [(0.700, 31.2150), (0.750, 22.5403)].map(|(asked, was)| {
        let mesh = mesh_of(&model, ball, asked);
        let sag = worst_sag_across_the_triangles(&mesh, 1.0);
        let stated = mesh.deviation().magnitude();
        println!(
            "a unit ball asked for {asked}: {} points, sag {:.4}% of the radius, stated {:.4}%",
            mesh.points().len(),
            100.0 * sag,
            100.0 * stated
        );
        assert!(
            sag <= stated && stated <= asked,
            "a unit ball asked for {asked} said {stated} and sagged {sag}"
        );
        assert_close(
            100.0 * sag,
            was,
            &format!("the sag at a deflection of {asked}"),
        );
        sag
    });

    assert!(
        sags[1] < sags[0],
        "asking for a coarser mesh at 0.750 no longer gives a finer one, so this file is stale"
    );
}

/// Every place a unit ball's mesh changes shape between a deflection of two
/// fifths and a whole radius, and what it changes to.
///
/// One row per change, swept at a thousandth of a radius: the deflection the
/// change first shows at, how many points the mesh has, how many steps the seam
/// meridian is cut into, how many steps a cut meridian is cut into, the sag as a
/// share of the radius, and the deviation the mesh states.
///
/// The two rows to read together are 0.715 and 0.732. The first is the seam
/// getting coarser on its own; the second is the cuts following it, and the sag
/// **dropping** by eleven points of the radius when they do.
const EVERY_CHANGE_OF_SHAPE: [(f64, usize, usize, usize, f64, f64); 7] = [
    (0.400, 30, 5, 5, 10.4379, 27.4810),
    (0.406, 29, 4, 5, 12.3837, 31.8334),
    (0.412, 22, 4, 4, 16.1139, 41.1544),
    (0.667, 14, 4, 4, 31.2150, 66.6667),
    (0.715, 13, 3, 4, 33.6675, 66.6667),
    (0.732, 14, 3, 3, 22.5403, 50.0000),
    (0.834, 10, 3, 3, 36.7544, 83.3333),
];

/// The sweep that [`EVERY_CHANGE_OF_SHAPE`] was read off, run again.
///
/// Two things are checked at every one of the 601 deflections and not only at
/// the seven rows: the sag stays under the stated deviation, and the stated
/// deviation stays under the deflection asked for. Those are the promise. The
/// table is the wrinkle.
#[test]
fn every_change_of_shape_a_unit_balls_mesh_makes_between_two_fifths_and_a_whole_radius() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model.add_ball(1.0, &Isometry::identity()).expect("a ball");

    let mut seen: Vec<(f64, usize, usize, usize, f64, f64)> = Vec::new();
    for thousandths in 400..=1000 {
        let asked = f64::from(thousandths) / 1000.0;
        let mesh = mesh_of(&model, ball, asked);
        let sag = worst_sag_across_the_triangles(&mesh, 1.0);
        let stated = mesh.deviation().magnitude();
        assert!(
            sag <= stated && stated <= asked,
            "a unit ball asked for {asked} said {stated} and sagged {sag}"
        );

        let shape = (
            asked,
            mesh.points().len(),
            how_many_rows_along(&mesh, 0.0),
            how_many_rows_along(&mesh, core::f64::consts::FRAC_PI_2),
            100.0 * sag,
            100.0 * stated,
        );
        if seen.last().map(|last| (last.1, last.2, last.3)) != Some((shape.1, shape.2, shape.3)) {
            println!(
                "at {:.3}: {} points, seam in {}, cuts in {}, sag {:.4}%, stated {:.4}%",
                shape.0, shape.1, shape.2, shape.3, shape.4, shape.5
            );
            seen.push(shape);
        }
    }

    assert_eq!(
        seen.len(),
        EVERY_CHANGE_OF_SHAPE.len(),
        "a unit ball's mesh changes shape {} times over this range and the table has {} rows",
        seen.len(),
        EVERY_CHANGE_OF_SHAPE.len()
    );
    for (got, want) in seen.iter().zip(EVERY_CHANGE_OF_SHAPE) {
        assert_eq!(
            (got.0, got.1, got.2, got.3),
            (want.0, want.1, want.2, want.3),
            "the change of shape at {:.3} is not the one the table has at {:.3}",
            got.0,
            want.0
        );
        assert_close(
            got.4,
            want.4,
            &format!("the sag at a deflection of {:.3}", got.0),
        );
        assert_close(
            got.5,
            want.5,
            &format!("the deviation at a deflection of {:.3}", got.0),
        );
    }
}

/// The four grid changes in the table sit exactly where the two `ceil`s put
/// them, which is the whole reason the sag steps.
///
/// The seam is refined by the curve evaluator, which `mesh.rs` asks for a
/// sagitta of `WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO` of the deflection — three
/// sixteenths — and which cuts a half turn into `m` steps once `1 − cos(π/2m)`
/// fits inside that. A cut is walked by `how_many_steps`, which allows a chord
/// of `√(3 · radius · deflection · ½)` and cuts a half turn into `n` steps once
/// `π/n` radians of arc fits inside that chord.
///
/// The two are *nearly* the same schedule and not the same one, which is what
/// leaves a window where the seam has 3 steps and the cuts still have 4.
#[test]
fn the_two_ceilings_that_put_the_grid_changes_where_they_are() {
    let a_seam_of_this_many_steps =
        |m: f64| (1.0 - (core::f64::consts::PI / (2.0 * m)).cos()) / 0.1875;
    let a_cut_of_this_many_steps = |n: f64| core::f64::consts::PI.powi(2) / (3.0 * 0.5 * n.powi(2));

    for (which, first_seen_at, worked_out) in [
        (
            "the seam drops to 4 steps",
            0.406,
            a_seam_of_this_many_steps(4.0),
        ),
        (
            "a cut drops to 4 steps",
            0.412,
            a_cut_of_this_many_steps(4.0),
        ),
        (
            "the seam drops to 3 steps",
            0.715,
            a_seam_of_this_many_steps(3.0),
        ),
        (
            "a cut drops to 3 steps",
            0.732,
            a_cut_of_this_many_steps(3.0),
        ),
    ] {
        println!("{which}: first seen at {first_seen_at:.3}, worked out as {worked_out:.6}");
        assert!(
            first_seen_at - 0.001 < worked_out && worked_out <= first_seen_at,
            "{which} is first seen at {first_seen_at} but the ceiling puts it at {worked_out}"
        );
    }
}

/// The extra points at 0.750 are one in the middle of every cell round the
/// equator — the only cells of a three-row grid that are not triangles standing
/// on a pole — and they are
/// there because the cell's own diagonal strays further than 0.750.
///
/// This is the step that runs the wrong way. Below 0.834 the grid of 3 latitude
/// rows by 4 longitude columns has a cell diagonal that strays five sixths of
/// the radius, which is more than was asked for, so the triangulator puts a
/// point in the middle of each of the four cells round the equator. Above 0.834
/// five sixths is inside what was asked and the four points are not there — the
/// mesh drops from 14 points to 10 and the sag rises from 22.54% to 36.75%.
#[test]
fn the_coarser_grid_needs_a_point_back_in_the_middle_of_every_cell_round_the_equator() {
    let below = latitude_and_longitude(-30.0_f64.to_radians(), 0.0);
    let across = latitude_and_longitude(30.0_f64.to_radians(), core::f64::consts::FRAC_PI_2);
    let how_far_the_cell_strays = below.distance_to(across).powi(2) / 3.0;
    println!("the diagonal of one cell of a 3-by-4 grid strays {how_far_the_cell_strays:.6}");
    assert_close(
        how_far_the_cell_strays,
        5.0 / 6.0,
        "the stray across a cell diagonal",
    );
    assert!(
        0.750 < how_far_the_cell_strays && how_far_the_cell_strays < 0.900,
        "the cell diagonal no longer straddles the two deflections this test compares"
    );

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model.add_ball(1.0, &Isometry::identity()).expect("a ball");
    let repaired = mesh_of(&model, ball, 0.750);
    let left_alone = mesh_of(&model, ball, 0.900);

    for (asked, mesh) in [(0.750, &repaired), (0.900, &left_alone)] {
        assert_eq!(
            how_many_rows_along(mesh, 0.0),
            3,
            "a unit ball asked for {asked} is no longer cut into 3 latitude rows"
        );
    }
    assert_eq!(repaired.points().len(), left_alone.points().len() + 4);

    let on_the_equator = |mesh: &Mesh| mesh.points().iter().filter(|p| p.z().abs() < 1e-9).count();
    assert_eq!(
        on_the_equator(&repaired),
        4,
        "the four points repairing the cells of a mesh asked for 0.750 are gone"
    );
    assert_eq!(
        on_the_equator(&left_alone),
        0,
        "a mesh asked for 0.900 has points on the equator, so its cells were repaired too"
    );
}

fn mesh_of(model: &Model, ball: SolidKey, asked: f64) -> Mesh {
    model
        .solid(ball)
        .expect("this model made the key")
        .mesh(
            Deflection::of(asked).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a mesh")
}

/// The furthest any point *on* a triangle of `mesh` sits from the sphere of
/// `radius` about the origin.
///
/// Twenty-one barycentric samples per triangle, the same grid
/// `how_far_a_balls_mesh_points_really_sit_from_the_ball.rs` uses, so the
/// numbers here and there are the same numbers. It undersamples: a grid of 40
/// steps reads 32.13% where this reads 31.22% at a deflection of 0.700. That
/// makes every sag below a lower bound on the real one, which is the safe way
/// round for the assertion that the sag stays under the stated deviation, and
/// the wrong way round for nothing this file claims.
fn worst_sag_across_the_triangles(mesh: &Mesh, radius: f64) -> f64 {
    const STEPS: usize = 5;
    let mut worst: f64 = 0.0;
    for triangle in mesh.triangles() {
        let [a, b, c] = triangle.corners().map(|i| mesh.points()[i as usize]);
        for i in 0..=STEPS {
            for j in 0..=(STEPS - i) {
                let (u, v) = (i as f64 / STEPS as f64, j as f64 / STEPS as f64);
                let w = 1.0 - u - v;
                let inside = Point::new(
                    u * a.x() + v * b.x() + w * c.x(),
                    u * a.y() + v * b.y() + w * c.y(),
                    u * a.z() + v * b.z() + w * c.z(),
                );
                worst = worst.max((Point::origin().distance_to(inside) - radius).abs());
            }
        }
    }
    worst
}

/// How many steps the meridian at `longitude` is cut into, counted off the mesh
/// points that sit on it.
///
/// The two poles sit on every meridian, so they are taken in whatever longitude
/// they are asked for; everything else has to be on the meridian itself. What
/// comes back is one less than the number of latitudes, which is the number of
/// steps between them.
fn how_many_rows_along(mesh: &Mesh, longitude: f64) -> usize {
    let mut latitudes: Vec<f64> = mesh
        .points()
        .iter()
        .filter(|p| {
            let out = p.x().hypot(p.y());
            out < 1e-9 || (p.y().atan2(p.x()) - longitude).abs() < 1e-9
        })
        .map(|p| p.z())
        .collect();
    latitudes.sort_by(|a, b| a.partial_cmp(b).expect("a mesh point is a number"));
    latitudes.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    latitudes.len() - 1
}

fn latitude_and_longitude(latitude: f64, longitude: f64) -> Point {
    Point::new(
        latitude.cos() * longitude.cos(),
        latitude.cos() * longitude.sin(),
        latitude.sin(),
    )
}

/// Every number pinned in this file is the same arithmetic every time, so the
/// room here is for the fourth decimal place the table was written to and not
/// for a different mesh — the gaps this file is about are whole percentage
/// points of the radius.
fn assert_close(got: f64, want: f64, what: &str) {
    assert!(
        (got - want).abs() < 1e-3,
        "{what} is {got} and this file was written when it was {want}"
    );
}
