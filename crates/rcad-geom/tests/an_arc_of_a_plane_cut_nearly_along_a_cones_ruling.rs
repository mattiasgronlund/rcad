//! An arc by the frustum of the ellipse a plane cuts a cone in, where the plane
//! is a rounding off parallel to one of the cone's rulings.
//!
//! The plane and the two ends are the ones a fuse built in
//! `a_plane_cut_along_a_cones_other_nappe.rs`: the top of a block `10` square,
//! leaned `2` about `y`, turned `1` about `z`, reaching up to `1`, against a
//! frustum of slope one half with its apex at the origin. The plane's lean
//! rounds to a hair off the cone's own, so the ellipse is `4.5e16` long and its
//! centre `2.2e16` out, and the arc between the two ends is `1.7` across.
//!
//! Walked round the ellipse, the arc's one middle point came out at `(0, 0, 4)`,
//! two off the cone (`docs/decisions.md` §406.5). Every point here is checked
//! against **both** surfaces, which say exactly where the crossing is.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Isometry, NamedRun, Picker, Plane, Point, Surface,
    Vector, walk,
};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::smallvec;

/// The cone of the frustum: slope one half about `+z`, apex at the origin.
fn the_cone() -> Cone {
    Cone::new(Axis::z(), 0.5).expect("a slope above zero")
}

/// The block's top face, placed as that file places it.
///
/// **Its bits are the point**: the normal is a turned unit vector a rounding
/// off the cone's ruling, which is what makes the ellipse this long, and no
/// constructor that scales a normal keeps them.
/// `the_top_is_the_plane_the_fuse_walked` checks them against the plane the
/// mesher handed the walk.
fn the_top() -> Plane {
    let about_y = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("a direction"),
        2.0f64.atan(),
    )
    .expect("a finite turn");
    let about_z = Isometry::rotation(&Axis::z(), 1.0).expect("a finite turn");
    let moved = |[x, y, z]: [f64; 3]| Isometry::translation(Vector::new(x, y, z)).expect("a move");
    let placed = about_z
        .compose(&moved([0.0, 0.0, -3.0]))
        .compose(&about_y)
        .compose(&moved([-5.0, -5.0, 0.0]));
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
        .expect("a plane")
        .transformed(&placed)
}

/// The plane above is the one `a_plane_cut_along_a_cones_other_nappe.rs`'s
/// fuse handed the walk for the block's top, printed bit for bit there.
#[test]
fn the_top_is_the_plane_the_fuse_walked() {
    let top = the_top();
    assert_eq!(
        (top.normal().components(), top.offset()),
        (
            [
                0.483_261_073_728_417_64,
                0.752_634_529_249_659_7,
                0.447_213_595_499_958
            ],
            2.658_359_213_500_127_3
        )
    );
}

/// Where the top, the cone and the block's two side walls meet, as the fuse
/// placed them: the two ends of the arc by the frustum.
fn the_ends() -> [Point; 2] {
    [
        Point::new(
            -0.054_782_226_828_283_195,
            1.629_574_607_719_640_2,
            3.260_990_336_999_413,
        ),
        Point::new(
            1.504_565_448_013_536_4,
            0.628_328_980_029_340_1,
            3.260_990_336_999_412,
        ),
    ]
}

/// The arc between the two ends that stays below the frustum's top, `z = 8`,
/// which the other arc, round past the ellipse's far end, does not.
fn the_arc() -> NamedRun {
    let below_the_top = HalfSpace::new(Vector::new(0.0, 0.0, -1.0), -8.0).expect("a half-space");
    NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(the_top()),
            picked_by: smallvec![Picker::InFrontOf(below_the_top)],
        },
        the_ends().map(|end| rcad_predicates::Point::from(end.coordinates())),
    )
    .expect("a run with two ends")
}

fn walked(deflection: f64) -> rcad_geom::PointsAlongACrossing {
    walk(
        &the_arc(),
        [&Surface::Cone(the_cone()), &Surface::Plane(the_top())],
        Deflection::of(deflection).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the arc is walked")
}

/// How far `point` sits off the cone, as a plain distance, read off a cross
/// product so that a miss does not cancel away.
fn how_far_off_the_cone(point: Point) -> f64 {
    let cone = the_cone();
    let along = cone.axis().direction();
    let out = point - cone.apex();
    (out.cross(along).length() - cone.slope() * out.dot(along).abs()).abs()
        / (along.length() * cone.slope().hypot(1.0))
}

/// The point of the crossing on the ruling at `round` radians round the cone,
/// worked out here and not by the walk: where that ruling meets the plane.
fn on_the_ruling_at(round: f64) -> Point {
    let top = the_top();
    let out = Vector::new(round.cos() * 0.5, round.sin() * 0.5, 1.0);
    let along = -top.signed_distance_to(Point::origin()) / top.normal().dot(out);
    Point::origin() + out * along
}

/// The arc's one middle point at a coarse deflection is on the plane and on the
/// cone, inside the deviation the walk publishes, and that deviation is a
/// rounding of the arc's own size and not of the ellipse's.
#[test]
fn the_middle_of_an_arc_by_the_frustum_is_on_the_plane_and_on_the_cone() {
    let along = walked(1.0);
    let [middle] = along.points() else {
        panic!(
            "one middle point at a deflection of one: {:?}",
            along.points()
        );
    };
    let deviation = along.deviation().magnitude();
    assert!(
        deviation < 1e-12,
        "charged {deviation:e}, which is a rounding of the ellipse and not the arc"
    );
    let off_the_plane = the_top().signed_distance_to(*middle).abs();
    let off_the_cone = how_far_off_the_cone(*middle);
    assert!(
        off_the_plane <= deviation && off_the_cone <= deviation,
        "{middle:?} is {off_the_plane:e} off the plane and {off_the_cone:e} off the cone, \
         against a deviation of {deviation:e}"
    );
}

/// Walked finely, every point is on both surfaces, and no stretch of the arc
/// between two neighbouring points strays further from their chord than the
/// deflection — the step count is the arc's bend, not the ellipse's.
#[test]
fn every_point_of_the_arc_walked_finely_is_on_both_surfaces_and_the_chords_keep_to_the_deflection()
{
    let deflection = 1e-3;
    let along = walked(deflection);
    let deviation = along.deviation().magnitude();
    let [first, last] = the_ends();
    let corners: Vec<Point> = core::iter::once(first)
        .chain(along.points().iter().copied())
        .chain(core::iter::once(last))
        .collect();
    assert!(
        corners.len() > 3,
        "a fine walk has points: {}",
        corners.len()
    );
    for point in along.points() {
        let off_the_plane = the_top().signed_distance_to(*point).abs();
        let off_the_cone = how_far_off_the_cone(*point);
        assert!(
            off_the_plane <= deviation && off_the_cone <= deviation,
            "{point:?} is {off_the_plane:e} off the plane and {off_the_cone:e} off the cone"
        );
    }
    let round = |point: Point| point.coordinates()[1].atan2(point.coordinates()[0]);
    let mut worst: f64 = 0.0;
    for pair in corners.windows(2) {
        let [from, to] = [pair[0], pair[1]];
        let chord = to - from;
        for share in [0.25, 0.5, 0.75] {
            let on_the_arc = on_the_ruling_at(round(from) + (round(to) - round(from)) * share);
            let out = on_the_arc - from;
            let strays = out.cross(chord).length() / chord.length();
            worst = worst.max(strays);
        }
    }
    assert!(
        worst <= deflection,
        "the arc strays {worst:e} from a chord, against a deflection of {deflection:e}"
    );
}
