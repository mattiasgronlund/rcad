//! Measuring one face of a solid: its area and where it balances, each with how
//! far out it could be.
//!
//! The consumer is a harness that runs OpenCASCADE's test corpus against this
//! kernel, where a shape is taken apart into faces and each face's properties
//! are checked. A refusal is an answer it can use; a wrong number is not.
//!
//! **Why these are not tautologies.** Every oracle here is a textbook formula
//! worked out in this file — `side²`, `πr²`, `2πrh`, `4πr²`, `π(r₁ + r₂)·slant`,
//! `π²Rr` and the centroids that go with them — and never a number the kernel
//! produced. The one law that *is* checked against the kernel, that the faces
//! add up to the solid, is checked bit for bit, so it cannot pass by being
//! close.

mod cones;
mod rings;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Model, ProfileBuilder, Rule, SolidKey};

const PI: f64 = core::f64::consts::PI;

/// What one face measured as, read off through the accessors.
#[derive(Clone, Copy, Debug)]
struct OneFace {
    area: f64,
    area_error: f64,
    centroid: Point,
    wobble: f64,
}

fn one_face(model: &Model, solid: SolidKey, face: FaceKey, how_fine: f64) -> OneFace {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure_one_face(
            face,
            Deflection::of(how_fine).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("measuring a face at a deflection of {how_fine}: {why}"));
    OneFace {
        area: measured.area(),
        area_error: measured.area_error(),
        centroid: measured.centroid(),
        wobble: measured.centroid_wobble(),
    }
}

fn faces_of(model: &Model, solid: SolidKey) -> Vec<FaceKey> {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| face.key())
        .collect()
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A turn of one radian about a slanted axis, and then a million units out
/// along all three axes.
///
/// **Not a third of a turn about `(1, 1, 1)`**, which the other measuring tests
/// use: that turn swaps the axes round, so a cube turned by it is axis-aligned
/// again and, once the million units have rounded away what the turn got wrong,
/// sits on whole numbers — and a test of it proves nothing about a slanted face.
fn turned_and_a_million_units_out() -> Isometry {
    let axis = Axis::new(Point::new(0.5, -1.5, 2.0), Vector::new(1.0, 2.0, 3.0))
        .expect("a direction with length");
    Isometry::translation(Vector::new(1e6, -1e6, 1e6))
        .expect("a shift")
        .compose(&Isometry::rotation(&axis, 1.0).expect("a rotation"))
}

/// Every face of a cube, a plate with a square hole and an L is exactly the
/// textbook's: the area on the nose and the middle the nearest `f64` to the
/// true one, with all three errors exactly zero.
///
/// Whole-number corners, so every product and sum over the fan is exact and
/// the one division rounds once. The plate's top face has a hole, which its
/// fan takes back with triangles wound the other way; the L's top face is not
/// convex. Both are what a fan weighed without its sign would get wrong.
#[test]
fn a_flat_face_measures_exactly_the_textbooks_area_and_middle() {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
    let plate = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");
    let ell = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let ell = model
        .add_extrusion(&ell, 1.5, &Isometry::identity())
        .expect("an L one and a half thick");

    let mut cube_faces = Vec::new();
    for face in faces_of(&model, cube) {
        let measured = one_face(&model, cube, face, 0.1);
        assert_eq!(measured.area, 4.0, "a face of a cube of side two");
        assert_eq!(
            [measured.area_error, measured.wobble],
            [0.0, 0.0],
            "a polyhedron's face is measured exactly"
        );
        cube_faces.push(measured.centroid.coordinates());
    }
    cube_faces.sort_by(|one, other| one.partial_cmp(other).expect("finite"));
    assert_eq!(
        cube_faces,
        [
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 2.0],
            [1.0, 2.0, 1.0],
            [2.0, 1.0, 1.0],
        ],
        "the middle of each side of a cube of side two"
    );

    let top_of = |solid: SolidKey, height: f64| {
        faces_of(&model, solid)
            .into_iter()
            .map(|face| one_face(&model, solid, face, 0.1))
            .find(|measured| measured.centroid.coordinates()[2] == height)
            .expect("the top face")
    };
    let top = top_of(plate, 3.0);
    assert_eq!(top.area, 96.0, "ten by ten less two by two");
    assert_eq!(top.centroid.coordinates(), [5.0, 5.0, 3.0]);
    let top = top_of(ell, 1.5);
    assert_eq!(top.area, 12.0, "four by four less two by two");
    // Sixteen at (2, 2) less four at (3, 3), over twelve.
    let middle = 20.0 / 12.0;
    assert_eq!(top.centroid.coordinates(), [middle, middle, 1.5]);
}

/// The same cube turned and put a million units out is measured within a few
/// roundings of `side²` — and not on it, by far more than the area error it
/// publishes.
///
/// That is `Measurement`'s own convention, followed rather than reinvented: what
/// is exact is the sum over the corners the solid holds, and placing the cube
/// rounded those corners before anything here saw them — to the spacing of
/// `f64` at a million, `2⁻³³`, which is `1.16e-10`. Each corner then sits within
/// half that of where the turn put it on each axis, so within `0.87` of it
/// all told; a side of two moved that far at both ends changes the face's area
/// by at most its perimeter times that, and its middle by that much before the
/// middle is itself rounded at a million. Measured:
/// the area came out `1.25e-10` off `4` and the middle `1.16e-10` off, which
/// is a rounding's worth of placement and nothing of the fan's own.
///
/// **The area error covers the adding up and not the placing.** Since §338 it
/// is how far the face's `f64` sum landed from the exact area of the corners
/// the solid holds, which is under two spacings of `f64` at `4`; the corners
/// the placement rounded are those corners, so `side²` itself stays outside
/// it. What would cover that too is an error that charges the corners' own
/// rounding — for the solid and every face at once, since the faces add up to
/// the solid.
#[test]
fn a_cube_turned_and_far_away_measures_within_a_few_roundings_of_its_side_squared() {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let placement = turned_and_a_million_units_out();
    model
        .place_solid(cube, &placement)
        .expect("this model made the key");

    let mut worst: (f64, f64) = (0.0, 0.0);
    for face in faces_of(&model, cube) {
        let measured = one_face(&model, cube, face, 0.1);
        assert_eq!(measured.wobble, 0.0);
        assert!(
            measured.area_error > 0.0 && measured.area_error <= 2.0 * 4.0 * f64::EPSILON,
            "a face's sum rounded by {}",
            measured.area_error
        );
        let middles = [
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 2.0],
            [1.0, 2.0, 1.0],
            [2.0, 1.0, 1.0],
        ]
        .map(|middle| placement.apply_to_point(Point::from(middle)));
        let off_the_middle = middles
            .iter()
            .map(|middle| middle.distance_to(measured.centroid))
            .fold(f64::INFINITY, f64::min);
        worst = (
            worst.0.max((measured.area - 4.0).abs()),
            worst.1.max(off_the_middle),
        );
    }
    let spacing = 2.0_f64.powi(-33);
    let a_corner_moved = 0.5 * spacing * 3.0_f64.sqrt();
    assert!(
        worst.0 > 0.0 && worst.0 <= 8.0 * a_corner_moved,
        "a face of a turned cube is {} off side²",
        worst.0
    );
    assert!(
        // The corners moved, and then the measured middle and the one worked
        // out here each rounded once more at a million.
        worst.1 > 0.0 && worst.1 <= 3.0 * a_corner_moved,
        "and its middle {} off where the turn puts it",
        worst.1
    );
}

/// A face, what the textbook says its area is, and where it says it balances.
type Truth = (FaceKey, f64, Point);

/// A round bar of radius three and height five: two caps of `πr²` centred on
/// the axis at either end, and a wall of `2πrh` centred half way up.
fn a_round_bar(model: &mut Model, placement: &Isometry) -> (SolidKey, Vec<Truth>) {
    let (radius, height) = (3.0, 5.0);
    let bar = model
        .add_round_bar(radius, height, placement)
        .expect("a bar three across and five tall");
    let at = |z: f64| placement.apply_to_point(Point::new(0.0, 0.0, z));
    let truths = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .map(|face| {
            let surface = face.surface();
            if let Some(plane) = surface.as_plane() {
                let bottom = plane.signed_distance_to(at(0.0)).abs() < 1e-6;
                return (
                    face.key(),
                    PI * radius * radius,
                    at(if bottom { 0.0 } else { height }),
                );
            }
            assert!(
                matches!(surface, Surface::Cylinder(_)),
                "a round bar's one curved face is its wall"
            );
            (face.key(), 2.0 * PI * radius * height, at(height / 2.0))
        })
        .collect();
    (bar, truths)
}

/// A ball of radius three: `4πr²` of skin, balanced on its centre.
fn a_ball(model: &mut Model, placement: &Isometry) -> (SolidKey, Vec<Truth>) {
    let radius = 3.0;
    let ball = model.add_ball(radius, placement).expect("a ball");
    let faces = faces_of(model, ball);
    assert_eq!(faces.len(), 1, "a ball is one face");
    (
        ball,
        vec![(
            faces[0],
            4.0 * PI * radius * radius,
            placement.apply_to_point(Point::origin()),
        )],
    )
}

/// A frustum of slope a half from two up the axis to six: caps of radius one
/// and three, and a side of `π(r₁ + r₂)·slant` balanced on the axis at
/// `⅔(z₂³ − z₁³)/(z₂² − z₁²)`, because a band of it at height `z` is `z` wide.
fn a_frustum(model: &mut Model, placement: &Isometry) -> (SolidKey, Vec<Truth>) {
    let (slope, near, far) = (0.5, 2.0, 6.0);
    let frustum = cones::a_frustum(model, slope, near, far);
    model
        .place_solid(frustum, placement)
        .expect("this model made the key");
    let at = |z: f64| placement.apply_to_point(Point::new(0.0, 0.0, z));
    let slant = (far - near) * slope.hypot(1.0);
    let middle = 2.0 / 3.0 * (far.powi(3) - near.powi(3)) / (far * far - near * near);
    let truths = cone_faces(model, frustum, at(near), |kind| match kind {
        Kind::NearCap => (PI * (slope * near).powi(2), at(near)),
        Kind::FarCap => (PI * (slope * far).powi(2), at(far)),
        Kind::Side => (PI * slope * (near + far) * slant, at(middle)),
    });
    (frustum, truths)
}

/// A cone ending in a point, of slope a half and four tall: a cap of radius two
/// and a side of `πrl` balanced two thirds of the way up from the apex.
fn a_pointed_cone(model: &mut Model, placement: &Isometry) -> (SolidKey, Vec<Truth>) {
    let (slope, far) = (0.5, 4.0);
    let cone = cones::a_pointed_cone(model, slope, far);
    model
        .place_solid(cone, placement)
        .expect("this model made the key");
    let at = |z: f64| placement.apply_to_point(Point::new(0.0, 0.0, z));
    let radius = slope * far;
    let truths = cone_faces(model, cone, at(0.0), |kind| match kind {
        Kind::FarCap => (PI * radius * radius, at(far)),
        Kind::Side => (PI * radius * radius.hypot(far), at(2.0 / 3.0 * far)),
        Kind::NearCap => panic!("a pointed cone has no cap at its apex"),
    });
    (cone, truths)
}

enum Kind {
    NearCap,
    FarCap,
    Side,
}

/// The faces of a cone-faced solid, a cap told apart by whether its plane runs
/// through `near`.
fn cone_faces(
    model: &Model,
    solid: SolidKey,
    near: Point,
    truth: impl Fn(Kind) -> (f64, Point),
) -> Vec<Truth> {
    let view = model.solid(solid).expect("this model made the key");
    view.faces()
        .map(|face| {
            let surface = face.surface();
            let kind = match surface.as_plane() {
                Some(plane) if plane.signed_distance_to(near).abs() < 1e-6 => Kind::NearCap,
                Some(_) => Kind::FarCap,
                None => {
                    assert!(
                        matches!(surface, Surface::Cone(_)),
                        "a cone-faced solid's one curved face is on the cone"
                    );
                    Kind::Side
                }
            };
            let (area, middle) = truth(kind);
            (face.key(), area, middle)
        })
        .collect()
}

/// A quarter bend of ring four and tube one: a tube face of `π²Rr` balanced on
/// the diagonal at `(2R² + r²)/(πR)` along each axis, and two caps of `πr²`
/// centred on the ring.
fn a_bend(model: &mut Model, placement: &Isometry) -> (SolidKey, Vec<Truth>) {
    let (ring, tube) = (4.0, 1.0);
    let bend = rings::a_quarter_bend(model, ring, tube);
    model
        .place_solid(bend, placement)
        .expect("this model made the key");
    let at = |x: f64, y: f64| placement.apply_to_point(Point::new(x, y, 0.0));
    let across = (2.0 * ring * ring + tube * tube) / (PI * ring);
    let view = model.solid(bend).expect("this model made the key");
    let truths = view
        .faces()
        .map(|face| {
            let surface = face.surface();
            match surface.as_plane() {
                Some(plane) if plane.signed_distance_to(at(ring, 0.0)).abs() < 1e-6 => {
                    (face.key(), PI * tube * tube, at(ring, 0.0))
                }
                Some(_) => (face.key(), PI * tube * tube, at(0.0, ring)),
                None => {
                    assert!(
                        matches!(surface, Surface::Torus(_)),
                        "a bend's one curved face is its tube"
                    );
                    (face.key(), PI * PI * ring * tube, at(across, across))
                }
            }
        })
        .collect();
    (bend, truths)
}

type CurvedShape = (
    &'static str,
    fn(&mut Model, &Isometry) -> (SolidKey, Vec<Truth>),
);

const CURVED_SHAPES: [CurvedShape; 5] = [
    ("a round bar", a_round_bar),
    ("a ball", a_ball),
    ("a frustum", a_frustum),
    ("a pointed cone", a_pointed_cone),
    ("a bend", a_bend),
];

/// §15.6 for both published errors of a face: the textbook area is inside
/// `[area − area_error, area + area_error]` and the textbook centroid is within
/// `centroid_wobble` of the measured one, for every face of every shape, at
/// four deflections, where each was built and turned a million units out.
///
/// The closest any came to its bound is floored, so a bound grown so loose that
/// nothing comes near it fails here too. Measured over the 96 faces with a
/// finite wobble: an area came to **0.253** of its error — the area bound's own
/// quarter, which `Mesh::deviation` records for the solid — and a centroid to
/// **0.0022** of its wobble. So the wobble is about five hundred times what
/// these meshes really get wrong where they get the most wrong: symmetric
/// meshes of symmetric faces put the middle almost exactly where it belongs,
/// and the bound cannot know that.
#[test]
fn the_middle_of_each_face_is_within_its_wobble() {
    let mut closest_area: f64 = 0.0;
    let mut closest_middle: f64 = 0.0;
    let mut checked = 0;
    for placement in [Isometry::identity(), turned_and_a_million_units_out()] {
        for (what, build) in CURVED_SHAPES {
            let mut model = a_model();
            let (solid, truths) = build(&mut model, &placement);
            for how_fine in [0.3, 0.1, 0.03, 0.01] {
                for &(face, area, middle) in &truths {
                    let measured = one_face(&model, solid, face, how_fine);
                    let off = (measured.area - area).abs();
                    assert!(
                        off <= measured.area_error,
                        "{what} at {how_fine}: a face of area {area} measured {} and owned \
                         up to {}",
                        measured.area,
                        measured.area_error
                    );
                    let moved = measured.centroid.distance_to(middle);
                    assert!(
                        moved <= measured.wobble,
                        "{what} at {how_fine}: a face balancing at {middle:?} measured \
                         {:?}, {moved} away, and owned up to {}",
                        measured.centroid,
                        measured.wobble
                    );
                    if measured.wobble.is_finite() {
                        checked += 1;
                        closest_area = closest_area.max(off / measured.area_error);
                        closest_middle = closest_middle.max(moved / measured.wobble);
                    }
                }
            }
        }
    }
    assert!(checked >= 90, "only {checked} faces had a finite wobble");
    assert!(
        closest_area > 0.2 && closest_middle > 0.002,
        "the closest an area came to its error was {closest_area}, and a centroid to its \
         wobble {closest_middle}"
    );
}

/// The faces of a solid add up to its surface area **bit for bit**, when they
/// are added in the order the solid hands them out, starting from zero; and
/// their errors add up to its error to within the rounding of adding them in a
/// different order.
///
/// A cube, every curved shape above at the origin and far out, and a fuse of a
/// block with a bar through it. The errors are held to within eight roundings
/// of the solid's.
///
/// **Bit for bit for a curved solid as well as a flat one**, because a face is
/// measured over exactly the triangles the solid's area is summed over, in a
/// mesh that comes out the same every time it is cut (§25.7). The errors are
/// not bit for bit because the solid adds all its flat faces' rims before it
/// multiplies by the deviation, and a face multiplies its own.
#[test]
fn the_faces_of_a_solid_add_up_to_its_skin_bit_for_bit() {
    let mut model = a_model();
    let mut solids = Vec::new();
    solids.push((
        "a cube",
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube"),
    ));
    for placement in [Isometry::identity(), turned_and_a_million_units_out()] {
        for (what, build) in CURVED_SHAPES {
            solids.push((what, build(&mut model, &placement).0));
        }
    }
    let block = model
        .add_cube(4.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block");
    let bar = model
        .add_round_bar(
            1.0,
            6.0,
            &Isometry::translation(Vector::new(2.0, 2.0, -1.0)).expect("a shift"),
        )
        .expect("a bar right through it");
    let fused = model
        .fuse(
            &[block],
            &[bar],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar through a block unites")
        .solids()
        .to_vec();
    assert_eq!(fused.len(), 1, "one solid");
    solids.push(("a block with a bar through it", fused[0]));

    for (what, solid) in solids {
        let how_fine = 0.05;
        let whole = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(how_fine).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} measures: {why}"));
        let mut area = 0.0;
        let mut error = 0.0;
        for face in faces_of(&model, solid) {
            let measured = one_face(&model, solid, face, how_fine);
            area += measured.area;
            error += measured.area_error;
        }
        assert_eq!(
            area.to_bits(),
            whole.surface_area().to_bits(),
            "{what}: its faces add up to {area} and it measures {}",
            whole.surface_area()
        );
        assert!(
            (error - whole.area_error()).abs() <= 8.0 * f64::EPSILON * whole.area_error(),
            "{what}: its faces' errors add up to {error} and its own is {}",
            whole.area_error()
        );
    }
}

/// How big `area_error` is against the area, at a deflection of `0.01`, for
/// every face of every curved shape above — the numbers
/// `FaceMeasurement::area_error` publishes, each to half a percent.
///
/// Listed smallest face first. The bar's caps and wall and the ball are radius
/// three, the frustum's caps radius one and three, the pointed cone's cap
/// radius two, and the bend's tube radius one: a curved face costs about twice
/// the deviation over its radius, and a flat cap about the deviation times its
/// rim over its area, which is the same over its radius.
#[test]
fn what_a_faces_area_error_is_against_its_area() {
    let published: [&[f64]; 5] = [
        &[3.32e-3, 3.32e-3, 3.33e-3],
        &[6.70e-3],
        &[1.97e-2, 6.56e-3, 1.78e-2],
        &[6.63e-3, 1.81e-2],
        &[2.00e-2, 2.00e-2, 2.03e-2],
    ];
    for ((what, build), published) in CURVED_SHAPES.into_iter().zip(published) {
        let mut model = a_model();
        let (solid, mut truths) = build(&mut model, &Isometry::identity());
        truths.sort_by(|one, other| one.1.total_cmp(&other.1));
        assert_eq!(truths.len(), published.len(), "{what}: one number a face");
        for (&(face, area, _), &share) in truths.iter().zip(published) {
            let measured = one_face(&model, solid, face, 0.01);
            let ratio = measured.area_error / measured.area;
            assert!(
                (ratio - share).abs() <= 0.005 * share,
                "{what}: a face of area {area} is charged {ratio:e} of itself, and \
                 {share:e} is what is published"
            );
        }
    }
}

/// What cannot be measured refuses by name.
#[test]
fn what_cannot_be_measured_is_refused_by_name() {
    let mut model = a_model();
    let bar = model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a bar");
    let other = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    let huge = model
        .add_cube(1e200, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side 1e200");
    let asked = |solid: SolidKey, face: FaceKey, how_fine: f64| {
        model
            .solid(solid)
            .expect("this model made the key")
            .measure_one_face(
                face,
                Deflection::of(how_fine).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
    };

    let somebody_elses = faces_of(&model, other)[0];
    assert!(
        matches!(
            asked(bar, somebody_elses, 0.1),
            Err(Error::NoSuchFaceInThisSolid { face }) if face == somebody_elses
        ),
        "a cube's face is not the bar's"
    );
    for face in faces_of(&model, huge) {
        assert!(
            matches!(
                asked(huge, face, 0.1),
                Err(Error::SurfaceAreaTooLargeToMeasure)
            ),
            "a face of 1e400 does not fit in an f64"
        );
    }
    let wall = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cylinder(_)))
        .expect("a bar has a wall")
        .key();
    assert!(
        matches!(
            asked(bar, wall, 1e-300),
            Err(Error::DeflectionBelowWhatAMeshCanReach { .. })
        ),
        "a deflection no point can be placed to"
    );
}
