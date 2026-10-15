//! Why the mesher refused the fuses of `a_plane_cut_along_a_cones_other_nappe.rs`
//! that got past selection and no further (`docs/decisions.md` §285).
//!
//! There were eighteen, and every one was refused in the check the fuse makes
//! before it writes a curved answer: the answer meshed as coarsely as a mesh
//! gets, for the sign of the space it encloses. Two causes:
//!
//! - **Sixteen caps of the pointed cone.** The piece of the cone inside the
//!   block runs right round the apex, so the mesher cuts it into four quarters
//!   up to the apex, each cut leaving from the rim corner nearest its quarter.
//!   The coarsest walk gives that rim four corners, and when the rim the block
//!   keeps is under a third of a turn two quarters pick the same corner. The
//!   patch between them was that corner and the apex twice, and the
//!   triangulator refused it: `CornersCoincide { position: 2 }`. The cuts leave
//!   from four different corners now (§354), and all sixteen answer right. The
//!   last two, whose steep face misses the apex by a rounding, measured at
//!   `0.02` and no finer until Q2040: their cap's rim runs down a ruling with
//!   its corners' longitudes a few ulps apart, and a cut left from one of
//!   those corners down the ruling, a sliver off the rim.
//! - **Two folds on the frustum.** The block's steep face and the frustum's
//!   face share the two arcs they cross in, each walked as two chords, and both
//!   faces draw the side between the two arcs' middles:
//!   `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`. That mesh
//!   retries a fold at half the room now, as §208.3's zero ladder already did,
//!   and both answer right (§285.6's experiment B).

use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::collections::BTreeMap;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// Every fuse of the family's two apex-near families that refuses in the
/// mesher, recounted on this tree: the 96 fuses of
/// the two `a_pointed_cone_with_its_apex_near_a_face_at_a_lean_of_…` tests
/// and the 32 of `a_frustum_whose_apex_is_inside_the_block_answers_right_or_refuses`.
///
/// There were eighteen (§285.1). Sixteen were caps of the pointed cone refused
/// `CornersCoincide { position: 2 }`, and none is refused there since the
/// quarter cuts leave from four different corners (§354). The other two were
/// the frustum's intersection at lean three eighths, crossing `−1/4`, turned 3
/// and 3.5, refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`
/// at the coarsest mesh (§285.4), and none is refused there since that mesh
/// retries a fold at half the room.
#[test]
fn the_mesher_refuses_no_cap_of_the_pointed_cone_and_no_fold_on_the_frustum() {
    assert_eq!(the_mesher_refusals(), Vec::<MesherRefusal>::new());
}

/// The sixteen caps, predicted from the shape alone for every block whose steep
/// face crosses the axis above the apex.
///
/// The piece of the cone inside the block keeps the rim for `half` either side
/// of the block's middle, where `cos(half) = lean · (8 − crossing) / 4`, and the
/// rest of its boundary is the steep face's crossing, round the far side of the
/// apex. The coarsest walk gives that loop four corners: both ends of the rim
/// arc and its middle, and the middle of the crossing, half a turn from the
/// rim's middle (§285.2 has them as the mesher printed them). The mesher cuts a
/// cap a quarter turn at a time from the corner it starts at, each cut leaving
/// from the corner nearest its quarter, and a patch between two cuts that leave
/// from one corner has no width.
///
/// Two quarters are nearest one corner **exactly when the kept rim, `2 · half`,
/// is under a third of a turn**, so `half < π/3`: then the crossing is more than
/// two thirds of the turn and holds two quarters with one corner between them.
/// Lean three eighths keeps `half ≈ 0.74` and `0.79`; lean one quarter keeps
/// `1.06` and `1.08`.
///
/// Every turn of lean three eighths refused `CornersCoincide { position: 2 }`
/// for it until the cuts were made to leave from four different corners
/// (§354), and now every one of the twelve answers right.
#[test]
fn two_quarters_are_nearest_one_corner_when_the_kept_rim_is_under_a_third_of_a_turn_and_every_cap_answers()
 {
    let cone = THE_POINTED_CONE;
    let mut seen = Vec::new();
    for lean in [0.25, 0.375] {
        for crossing in [0.125, 0.5] {
            let half = (lean * (cone.far - crossing) / (cone.slope * cone.far)).acos();
            let share = two_quarters_share_a_corner(half);
            assert_eq!(share, half < PI / 3.0, "half the kept rim is {half}");
            for turn in [0.0, 1.0, 3.0] {
                let said = what_it_gives(cone, a_leaning_block(lean, crossing, turn), INTERSECTION);
                assert_eq!(
                    said, "right",
                    "lean {lean}, crossing {crossing}, turn {turn}"
                );
            }
            seen.push((lean, crossing, share));
        }
    }
    assert_eq!(
        seen,
        [
            (0.25, 0.125, false),
            (0.25, 0.5, false),
            (0.375, 0.125, true),
            (0.375, 0.5, true)
        ]
    );
}

/// The intersections whose steep face is meant to run through the apex,
/// crossing zero, all measure at `0.002`, and which side of that face the apex
/// really is on never said whether one of them measures at all.
///
/// The block's placement is rounded, so its steep face misses the apex by a
/// rounding. The block itself puts the apex inside it at two of the six, and
/// on its boundary at the other four; at `(0.25, 3.0)`, `(0.375, 0.0)` and
/// `(0.375, 3.0)` the steep face's stored plane has an offset of exactly
/// zero, so the apex sits exactly on it, 5 to 12 units inside every other
/// stored plane, and `OnBoundary` is the stored planes' answer (§346.7) where
/// the corners alone used to say Inside, Inside and Outside. Since Q846's
/// four-corner cuts (§354), only `(0.25, 1.0)` still failed to measure at
/// every deflection, while `(0.375, 1.0)`, inside too, measured. Since Q2040
/// it measures as well, so all six measure, whichever side of the face the
/// apex is on: a cut no longer leaves from a corner on the ruling the rim runs
/// down, a few ulps of longitude off the corners below it.
#[test]
fn which_side_of_the_face_meant_through_the_apex_it_is_on_does_not_say_whether_it_measures() {
    let policy = Policy::certified();
    let mut seen = Vec::new();
    for lean in [0.25, 0.375] {
        for turn in [0.0, 1.0, 3.0] {
            let block = a_leaning_block(lean, 0.0, turn);
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let (_, block_key) = build(&mut model, THE_POINTED_CONE, block);
            let apex = model
                .solid(block_key)
                .expect("this model made the key")
                .classify_point(Point::origin(), &policy, &mut ProvenanceLog::new());
            let does_not_measure = what_it_gives(THE_POINTED_CONE, block, INTERSECTION)
                .starts_with("right on the grid");
            seen.push((lean, turn, apex, does_not_measure));
        }
    }
    assert_eq!(
        seen,
        [
            (0.25, 0.0, Containment::OnBoundary, false),
            (0.25, 1.0, Containment::Inside, false),
            (0.25, 3.0, Containment::OnBoundary, false),
            (0.375, 0.0, Containment::OnBoundary, false),
            (0.375, 1.0, Containment::Inside, false),
            (0.375, 3.0, Containment::OnBoundary, false),
        ]
    );
}

/// The eighteen, each held to the family's four checks, with the measure asked
/// down a ladder of five deflections instead of at the finest alone.
///
/// All eighteen refused before there was an answer to check until the quarter
/// cuts left from four different corners (§354) and the coarsest mesh retried
/// a fold. Then sixteen answered right: the twelve caps that already did, the
/// frustum's two folds, and the two at lean three eighths whose steep face
/// misses the apex by a rounding (Q846). The last two, at lean one quarter,
/// held every grid point and measured at `0.02` and no finer, refused
/// `ASideCutsOffOneCornerOfACurvedPatch`, and all eighteen answer right since
/// Q2040 (§354.7). It is the harness for §285's and §354's experiments: with
/// one applied, the same test prints how many answer right and how the rest
/// refuse.
#[test]
fn the_eighteen_held_to_the_familys_four_checks() {
    let mut cases = Vec::new();
    for (lean, crossing, turn) in [
        (0.25, 0.0, 1.0),
        (0.375, 0.0, 1.0),
        (0.375, 0.125, 0.0),
        (0.375, 0.125, 1.0),
        (0.375, 0.125, 3.0),
        (0.375, 0.5, 0.0),
        (0.375, 0.5, 1.0),
        (0.375, 0.5, 3.0),
    ] {
        for rule in [INTERSECTION, BLOCK_LESS_CONE] {
            cases.push((THE_POINTED_CONE, lean, crossing, turn, rule));
        }
    }
    for turn in [3.0, 3.5] {
        cases.push((THE_FRUSTUM, 0.375, -0.25, turn, INTERSECTION));
    }
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for (cone, lean, crossing, turn, rule) in cases {
        let said = what_it_gives(cone, a_leaning_block(lean, crossing, turn), rule);
        println!(
            "near {} lean {lean} crossing {crossing} turn {turn} {}: {said}",
            cone.near,
            named_rule(rule)
        );
        *tally.entry(said).or_default() += 1;
    }
    println!("{tally:#?}");
    assert_eq!(tally, [("right".to_owned(), 18)].into());
}

/// The block whose steep face is meant to run through the apex, at a lean of
/// a quarter, turned an eighth of a turn and turned `2`, measures at the
/// finest deflection the family asks for.
///
/// The steep face's stored plane misses the apex by `8.9e-16` turned an eighth,
/// so the piece of the cone inside the block is a cap whose rim runs down near
/// one ruling to a rounding above the apex, round it, and up near another. The
/// rim's corners down the first ruling have longitudes within an ulp of a
/// whole turn of each other: `-0.26179938779914963` at `z = 8` and
/// `-0.26179938779915013` at `2.92`.
/// The first quarter cut left from the corner at `7.3`, down the ruling, and
/// the patch between it and the rim was a sliver the triangulator refused
/// `ASideCutsOffOneCornerOfACurvedPatch` at every deflection; turned `2`, the
/// same cap was refused `LoopsTouch` finer than `0.01` (Q2040).
#[test]
fn a_steep_face_meant_through_the_apex_turned_other_ways_measures_at_the_finest_deflection() {
    let measured = [FRAC_PI_4, 2.0].map(|turn| {
        how_the_finest_measure_goes(THE_POINTED_CONE, a_leaning_block(0.25, 0.0, turn))
    });
    assert_eq!(measured, ["right", "right"].map(str::to_owned));
}

const INTERSECTION: (Rule, bool) = (Rule::Intersection, true);
const BLOCK_LESS_CONE: (Rule, bool) = (Rule::Difference, false);

/// Union, intersection, cone less block, block less cone.
const RULES: [(Rule, bool); 4] = [
    (Rule::Union, true),
    INTERSECTION,
    (Rule::Difference, true),
    BLOCK_LESS_CONE,
];

/// Slope one half about `+z`, from `z = 2` to `z = 8`.
const THE_FRUSTUM: ConeShape = ConeShape {
    slope: 0.5,
    near: 2.0,
    far: 8.0,
};

/// Slope one half about `+z`, from the apex at the origin to `z = 8`.
const THE_POINTED_CONE: ConeShape = ConeShape {
    slope: 0.5,
    near: 0.0,
    far: 8.0,
};

#[derive(Clone, Copy, Debug)]
struct ConeShape {
    slope: f64,
    near: f64,
    far: f64,
}

#[derive(Clone, Copy, Debug)]
struct BoxShape {
    size: [f64; 3],
    placed: Isometry,
}

type MesherRefusal = (&'static str, f64, f64, f64, &'static str, String);

/// Every refusal in the mesher over the two families, in the order the family
/// tests run them.
fn the_mesher_refusals() -> Vec<MesherRefusal> {
    let mut refused = Vec::new();
    for lean in [0.25, 0.375] {
        for crossing in [0.0, 0.125, -0.125, 0.5] {
            for turn in [0.0, 1.0, 3.0] {
                for rule in RULES {
                    let block = a_leaning_block(lean, crossing, turn);
                    if let Some(why) = what_the_mesher_refused(THE_POINTED_CONE, block, rule) {
                        refused.push(("pointed", lean, crossing, turn, named_rule(rule), why));
                    }
                }
            }
        }
    }
    for (lean, crossing) in [(0.25, 1.0), (0.375, 0.5), (0.25, 0.0), (0.375, -0.25)] {
        for turn in [3.0, 3.5] {
            for rule in RULES {
                let block = a_leaning_block(lean, crossing, turn);
                if let Some(why) = what_the_mesher_refused(THE_FRUSTUM, block, rule) {
                    refused.push(("frustum", lean, crossing, turn, named_rule(rule), why));
                }
            }
        }
    }
    refused
}

/// What the mesher said when the fuse refused there, and `None` when the fuse
/// answered or refused anywhere else.
fn what_the_mesher_refused(
    cone: ConeShape,
    block: BoxShape,
    (rule, cone_first): (Rule, bool),
) -> Option<String> {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let (cone_key, block_key) = build(&mut model, cone, block);
    let (arguments, tools) = if cone_first {
        ([cone_key], [block_key])
    } else {
        ([block_key], [cone_key])
    };
    match model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new()) {
        Err(Error::CannotMeshAFace { why, .. }) => Some(format!("{why:?}")),
        Err(fold @ Error::TwoPatchesOfTheMeshDrewTheSameSide { .. }) => Some(format!("{fold:?}")),
        Ok(_) | Err(_) => None,
    }
}

fn named_rule(rule: (Rule, bool)) -> &'static str {
    match rule {
        (Rule::Union, _) => "union",
        (Rule::Intersection, _) => "intersection",
        (Rule::Difference, true) => "cone less block",
        (Rule::Difference, false) => "block less cone",
        _ => panic!("a rule this file does not run"),
    }
}

/// The four corners the coarsest walk gives a cap that keeps the rim for `half`
/// either side of the block's middle, as longitudes from that middle in the
/// order the mesher walks them, and whether two of the four quarter-turn cuts
/// leave from one of them — the same corner twice in a row, the last and the
/// first included.
fn two_quarters_share_a_corner(half: f64) -> bool {
    let corners = [half, 0.0, -half, -PI];
    let nearest: Vec<usize> = (0..4)
        .map(|step| {
            let quarter = half - f64::from(step) * FRAC_PI_2;
            (0..corners.len())
                .min_by(|&one, &other| {
                    (corners[one] - quarter)
                        .abs()
                        .total_cmp(&(corners[other] - quarter).abs())
                })
                .expect("four corners")
        })
        .collect();
    (0..4).any(|step| nearest[step] == nearest[(step + 1) % 4])
}

/// `"right"` when the fuse answers and the answer passes every check the family
/// makes; what refused when anything refused.
///
/// # Panics
///
/// When the fuse answers wrongly.
fn what_it_gives(cone: ConeShape, block: BoxShape, (rule, cone_first): (Rule, bool)) -> String {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let (cone_key, block_key) = build(&mut model, cone, block);
    let (arguments, tools) = if cone_first {
        ([cone_key], [block_key])
    } else {
        ([block_key], [cone_key])
    };
    let answer = match model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(answer) => answer,
        Err(why) => return format!("the fuse refused {}", named(&why)),
    };
    let what = format!("{rule:?} (cone first: {cone_first}) of {cone:?} and {block:?}");
    let pieces: Vec<SolidKey> = answer.solids().to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|error| panic!("{what}: the answer does not validate: {error:?}"));
    assert_eq!(pieces.len(), 1, "{what}: how many solids");

    let cone_volume = PI * cone.slope * cone.slope * (cone.far.powi(3) - cone.near.powi(3)) / 3.0;
    let block_volume: f64 = block.size.iter().product();
    let shared = shared_volume(cone, block);
    let want = if cone_first {
        shared
    } else {
        block_volume - shared
    };
    let mut decided = 0;
    for p in clear_grid_points(cone, block) {
        let truth = in_cone(cone, p) == cone_first && in_block(block, p);
        match model
            .solid(pieces[0])
            .expect("this model made the key")
            .classify_point(p, &policy, &mut ProvenanceLog::new())
        {
            Containment::Inside => {
                assert!(truth, "{what}: at {p:?}");
                decided += 1;
            }
            Containment::Outside => {
                assert!(!truth, "{what}: at {p:?}");
                decided += 1;
            }
            Containment::OnBoundary
            | Containment::Undecidable { .. }
            | Containment::CannotFollowAFacesTrimming { .. }
            | Containment::CannotFireARayAtAFacesSurface { .. }
            | _ => {}
        }
    }
    assert!(
        decided > 9_300,
        "{what}: only {decided} grid points could be classified"
    );
    let mut refusals = Vec::new();
    for how_fine in [0.002, 0.005, 0.01, 0.02, 0.05] {
        let deflection = Deflection::of(how_fine).expect("a positive length");
        match model
            .solid(pieces[0])
            .expect("this model made the key")
            .measure(deflection, &policy, &mut ProvenanceLog::new())
        {
            Ok(measured) => {
                let slack = measured.volume_error() + 0.002 * cone_volume;
                assert!(
                    (measured.volume() - want).abs() <= slack,
                    "{what}: volume {} at {how_fine}, where the shapes say {want} (allowed {slack})",
                    measured.volume()
                );
                if refusals.is_empty() {
                    return "right".to_owned();
                }
                return format!(
                    "right on the grid, measured right at {how_fine}, refused finer: {refusals:?}"
                );
            }
            Err(why) => refusals.push(named(&why)),
        }
    }
    format!("right on the grid ({decided} points), and every measure refused: {refusals:?}")
}

/// `"right"` when the intersection of the cone and the block is one solid that
/// measures at `0.002` within its bound of the volume the shapes share, as
/// [`what_it_gives`] holds a volume to; what refused when anything refused.
///
/// # Panics
///
/// When the intersection is not one solid, or measures a volume the shapes do
/// not share.
fn how_the_finest_measure_goes(cone: ConeShape, block: BoxShape) -> String {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let (cone_key, block_key) = build(&mut model, cone, block);
    let answer = match model.fuse(
        &[cone_key],
        &[block_key],
        Rule::Intersection,
        &policy,
        &mut ProvenanceLog::new(),
    ) {
        Ok(answer) => answer,
        Err(why) => return format!("the fuse refused {}", named(&why)),
    };
    let [piece] = answer.solids() else {
        panic!(
            "{block:?}: the intersection is {} solids",
            answer.solids().len()
        );
    };
    let deflection = Deflection::of(0.002).expect("a positive length");
    match model
        .solid(*piece)
        .expect("this model made the key")
        .measure(deflection, &policy, &mut ProvenanceLog::new())
    {
        Ok(measured) => {
            let want = shared_volume(cone, block);
            let cone_volume =
                PI * cone.slope * cone.slope * (cone.far.powi(3) - cone.near.powi(3)) / 3.0;
            let slack = measured.volume_error() + 0.002 * cone_volume;
            assert!(
                (measured.volume() - want).abs() <= slack,
                "{block:?}: volume {}, where the shapes say {want} (allowed {slack})",
                measured.volume()
            );
            "right".to_owned()
        }
        Err(why) => named(&why),
    }
}

/// A refusal as it prints, but for a face that would not mesh only what the
/// triangulator said, without the key of a face in a model that is gone.
fn named(refusal: &Error) -> String {
    if let Error::CannotMeshAFace { why, .. } = refusal {
        return format!("CannotMeshAFace {why:?}");
    }
    format!("{refusal:?}")
}

fn build(model: &mut Model, cone: ConeShape, block: BoxShape) -> (SolidKey, SolidKey) {
    let cone = if cone.near == 0.0 {
        a_pointed_cone(model, cone.slope, cone.far)
    } else {
        a_frustum(model, cone.slope, cone.near, cone.far)
    };
    let [x, y, z] = block.size;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    let block = model
        .add_extrusion(&rectangle, z, &block.placed)
        .expect("a rectangle extrudes");
    (cone, block)
}

/// `a_plane_cut_along_a_cones_other_nappe.rs`'s block: 12 by 10 by 15, its
/// low-`x` face the plane `x = lean · (z − crossing)`, turned about the axis.
fn a_leaning_block(lean: f64, crossing: f64, turn: f64) -> BoxShape {
    let placed = about_z(turn)
        .compose(&moved([-lean * crossing, 0.0, 0.0]))
        .compose(&about_y(lean.atan()))
        .compose(&moved([0.0, -5.0, -6.0]));
    BoxShape {
        size: [12.0, 10.0, 15.0],
        placed,
    }
}

fn about_y(angle: f64) -> Isometry {
    let axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("a direction");
    Isometry::rotation(&axis, angle).expect("a finite turn")
}

fn about_z(angle: f64) -> Isometry {
    Isometry::rotation(&Axis::z(), angle).expect("a finite turn")
}

fn moved([x, y, z]: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite move")
}

/// Points half a unit apart over `|x|, |y| ≤ 6`, `|z| ≤ 12`, nudged off the
/// lattice, keeping those further than `0.02` from every face of either input.
fn clear_grid_points(cone: ConeShape, block: BoxShape) -> impl Iterator<Item = Point> {
    (-12..=12).flat_map(move |i| {
        (-12..=12).flat_map(move |j| {
            (-24..=24).filter_map(move |k| {
                let p = Point::new(
                    f64::from(i) * 0.5 + 0.1,
                    f64::from(j) * 0.5 + 0.07,
                    f64::from(k) * 0.5 + 0.03,
                );
                (distance_to_a_boundary(cone, block, p) >= 0.02).then_some(p)
            })
        })
    })
}

fn in_cone(cone: ConeShape, p: Point) -> bool {
    let r = p.x().hypot(p.y());
    p.z() >= cone.near && p.z() <= cone.far && r <= cone.slope * p.z()
}

fn in_block(block: BoxShape, p: Point) -> bool {
    let local = block.placed.inverse().apply_to_point(p).coordinates();
    (0..3).all(|i| local[i] >= 0.0 && local[i] <= block.size[i])
}

/// Never more than the distance to the nearest of the cone's surface, its two
/// cap planes and the box's six planes.
fn distance_to_a_boundary(cone: ConeShape, block: BoxShape, p: Point) -> f64 {
    let r = p.x().hypot(p.y());
    let to_side = (cone.slope * p.z() - r).abs() / cone.slope.hypot(1.0);
    let to_caps = (p.z() - cone.near).abs().min((p.z() - cone.far).abs());
    let local = block.placed.inverse().apply_to_point(p).coordinates();
    let to_block = (0..3)
        .map(|i| local[i].abs().min((local[i] - block.size[i]).abs()))
        .fold(f64::INFINITY, f64::min);
    to_side.min(to_caps).min(to_block)
}

/// The volume the cone and the box share, over 800 × 800 vertical columns:
/// along each the cone is one interval in `z`, and the box another.
fn shared_volume(cone: ConeShape, block: BoxShape) -> f64 {
    let reach = cone.slope * cone.far;
    let steps = 800;
    let h = 2.0 * reach / f64::from(steps);
    let inverse = block.placed.inverse();
    let up = inverse
        .apply_to_vector(Vector::new(0.0, 0.0, 1.0))
        .components();
    let mut total = 0.0;
    for i in 0..steps {
        for j in 0..steps {
            let x = -reach + (f64::from(i) + 0.5) * h;
            let y = -reach + (f64::from(j) + 0.5) * h;
            let (mut from, mut to) = (cone.near.max(x.hypot(y) / cone.slope), cone.far);
            let at = inverse.apply_to_point(Point::new(x, y, 0.0)).coordinates();
            for k in 0..3 {
                if up[k].abs() < 1e-15 {
                    if at[k] < 0.0 || at[k] > block.size[k] {
                        to = from;
                    }
                    continue;
                }
                let one = -at[k] / up[k];
                let other = (block.size[k] - at[k]) / up[k];
                from = from.max(one.min(other));
                to = to.min(one.max(other));
            }
            if to > from {
                total += (to - from) * h * h;
            }
        }
    }
    total
}
