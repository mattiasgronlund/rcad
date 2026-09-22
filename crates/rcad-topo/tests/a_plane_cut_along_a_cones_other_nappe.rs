//! Does cutting a flat face along the *whole* double cone ever give a wrong
//! answer, or only a refusal? `docs/decisions.md` §199.3 point 1 left that open.
//!
//! A cone face as rcad builds it lies on one nappe, but the fuse cuts a plane
//! face wherever the plane meets the cone's surface — including the other
//! nappe, and parts of this nappe the face does not cover
//! (`a_plane_across_a_cone.rs` shows one such cut, far above a frustum). Each
//! family below puts that stray cut somewhere it could matter, runs all three
//! rules (the difference both ways round), and holds every answer against the
//! shape itself:
//!
//! - the model holding the answer passes [`Model::validate`];
//! - it has the number of solids the shapes call for;
//! - its measured volume is within `measure`'s own bound (plus a small slack
//!   for the column sum below) of the volume worked out from the shapes: the
//!   frustum's and the box's are closed forms, and what they share is summed
//!   column by column, since along a vertical line each is one interval;
//! - on a grid of points kept clear of every boundary, the answer holds exactly
//!   the points the rule says it should.
//!
//! **Every family pins how many fuses answer and how each of the others
//! refuses.** A family where everything refuses measures nothing about wrong
//! answers, and that count is what says so.
//!
//! A pointed cone whose apex sits inside a small cube stood on its corner used
//! to answer wrong — both solids back untouched, as if they never met — and was
//! left out of the file for it. Since the cone's box reaches its apex it
//! refuses by name, and
//! [`a_corner_up_cube_centred_on_a_pointed_cones_apex_refuses_rather_than_answering_wrong`]
//! pins that.

use std::collections::BTreeMap;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// A block steeper than the cone that also crosses the frustum.
///
/// Its low-`x` face is the plane `x = lean · (z − crossing)`, leaning at a
/// quarter or three eighths against the cone's half, so it cuts both nappes:
/// through the frustum above, and through the lower nappe, where the frustum
/// has no face, below.
#[test]
fn a_steeper_plane_across_a_frustum_refuses_every_time() {
    let mut cases = Vec::new();
    for lean in [0.25, 0.375] {
        for crossing in [-1.0, -3.0, -6.0] {
            for turn in [0.0, 0.5, 2.0, 3.0] {
                cases.push(Case {
                    cone: THE_FRUSTUM,
                    block: a_leaning_block(lean, crossing, turn),
                    solids: OVERLAPPING,
                });
            }
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 0,
            refused: [
                ("a point inside a piece of a face", 84),
                ("order along a crossing", 12),
            ]
            .into()
        }
    );
}

/// The same blocks turned half round, so the frustum's apex is inside the
/// block and so is the stretch of the lower nappe round it.
#[test]
fn a_frustum_whose_apex_is_inside_the_block_refuses_every_time() {
    let mut cases = Vec::new();
    for (lean, crossing) in [(0.25, 1.0), (0.375, 0.5), (0.25, 0.0), (0.375, -0.25)] {
        for turn in [3.0, 3.5] {
            cases.push(Case {
                cone: THE_FRUSTUM,
                block: a_leaning_block(lean, crossing, turn),
                solids: OVERLAPPING,
            });
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 0,
            refused: [
                ("a point inside a piece of a face", 24),
                ("Geometry(CannotNameWhichPieceOfACrossingThisIs)", 8),
            ]
            .into()
        }
    );
}

/// A cone that ends in a point, with a steeper face through the apex or an
/// eighth or a half either side of it.
#[test]
fn a_pointed_cone_with_its_apex_near_a_steeper_face_refuses_every_time() {
    let mut cases = Vec::new();
    for lean in [0.25, 0.375] {
        for crossing in [0.0, 0.125, -0.125, 0.5] {
            for turn in [0.0, 1.0, 3.0] {
                cases.push(Case {
                    cone: THE_POINTED_CONE,
                    block: a_leaning_block(lean, crossing, turn),
                    solids: OVERLAPPING,
                });
            }
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 0,
            refused: [
                ("a point inside a piece of a face", 80),
                ("Geometry(CannotNameWhichPieceOfACrossingThisIs)", 16),
            ]
            .into()
        }
    );
}

/// A block that misses the frustum, below it, whose faces still cross the cone
/// where the frustum has none: its top cuts the upper nappe under the frustum's
/// bottom cap, and its bottom the lower nappe.
///
/// **This is the family where the stray cut is the only cut**, and every fuse
/// that answers hands both solids back as they were.
#[test]
fn a_block_below_a_frustum_is_cut_where_the_frustum_has_no_face_and_every_answer_is_right() {
    let mut cases = Vec::new();
    for (low, high) in [(-3.0, 1.0), (-6.0, -1.0), (-3.0, 1.5), (-8.0, 1.875)] {
        for lean in [0.0f64, 0.25, 2.0] {
            for turn in [0.0, 1.0] {
                let placed = about_z(turn)
                    .compose(&moved([0.0, 0.0, low]))
                    .compose(&about_y(lean.atan()))
                    .compose(&moved([-5.0, -5.0, 0.0]));
                cases.push(Case {
                    cone: THE_FRUSTUM,
                    block: BoxShape {
                        size: [10.0, 10.0, high - low],
                        placed,
                    },
                    solids: APART,
                });
            }
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 56,
            refused: [
                ("a point inside a piece of a face", 20),
                ("order along a crossing", 20),
            ]
            .into()
        }
    );
}

/// A cube stood on its corner round the frustum's apex, or just under the
/// frustum. Every plane of the cube leans less steeply than the cone, so each
/// cuts one nappe in an ellipse — and the three lower planes cut the lower
/// nappe, where the frustum has no face.
#[test]
fn a_corner_up_cube_round_a_frustums_apex_answers_right_or_refuses() {
    let mut cases = Vec::new();
    for side in [4.0, 6.0] {
        for middle in [0.0, 1.0, 2.5] {
            for turn in [0.0, 0.7] {
                cases.push(Case {
                    cone: THE_FRUSTUM,
                    block: a_corner_up_cube(side, middle, turn),
                    solids: OVERLAPPING,
                });
            }
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 16,
            refused: [
                ("a point inside a piece of a face", 16),
                ("AWireWillNotLieFlatOnItsFace", 16),
            ]
            .into()
        }
    );
}

/// The same cubes round a pointed cone, less the side-4 cube centred on the
/// apex, which has a test of its own below.
///
/// All 40 refused until the pointed cone's face could say which side of its cap
/// it lies on (`docs/decisions.md` §200.4's first item): its one corner is on
/// the cap, so no corner said, and every point asked of that face refused.
#[test]
fn a_corner_up_cube_round_a_pointed_cones_apex_answers_right_or_refuses() {
    let mut cases = Vec::new();
    for (side, middle) in [(4.0, 1.0), (4.0, 2.5), (6.0, 0.0), (6.0, 1.0), (6.0, 2.5)] {
        for turn in [0.0, 0.7] {
            cases.push(Case {
                cone: THE_POINTED_CONE,
                block: a_corner_up_cube(side, middle, turn),
                solids: OVERLAPPING,
            });
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 16,
            refused: [
                ("a point inside a piece of a face", 8),
                ("AWireWillNotLieFlatOnItsFace", 16),
            ]
            .into()
        }
    );
}

/// The side-4 cube stood on its corner and centred on the pointed cone's apex,
/// which `docs/decisions.md` §200.3 found answering wrong: every rule said `Ok`
/// and handed both solids back untouched, because the cone's box stopped at
/// its cap and the fuse looked at no pair of faces (§202.3).
///
/// With the box reaching the apex every fuse refuses by name instead. That is a
/// **gap** and not a limit (§16.10): the two solids share volume, so every rule
/// has an answer to give, and this pins that nothing here gives a wrong one.
#[test]
fn a_corner_up_cube_centred_on_a_pointed_cones_apex_refuses_rather_than_answering_wrong() {
    let cases: Vec<Case> = [0.0, 0.7]
        .into_iter()
        .map(|turn| Case {
            cone: THE_POINTED_CONE,
            block: a_corner_up_cube(4.0, 0.0, turn),
            solids: OVERLAPPING,
        })
        .collect();
    assert_eq!(
        tally(&cases),
        Tally {
            right: 0,
            refused: [("a point inside a piece of a face", 8)].into()
        }
    );
}

/// The closed forms every answer is held against have to say the same as the
/// model about the two inputs, or they check nothing.
#[test]
fn the_closed_forms_agree_with_the_model_about_the_frustum_and_the_blocks() {
    let policy = Policy::certified();
    for block in [
        a_leaning_block(0.375, -1.0, 2.0),
        a_corner_up_cube(6.0, 1.0, 0.7),
    ] {
        let case = Case {
            cone: THE_FRUSTUM,
            block,
            solids: OVERLAPPING,
        };
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = build(&mut model, &case);
        let mut checked = 0;
        for p in clear_grid_points(&case) {
            for (solid, truth) in [
                (cone, in_cone(case.cone, p)),
                (block, in_block(case.block, p)),
            ] {
                let said = model
                    .solid(solid)
                    .expect("this model made the key")
                    .classify_point(p, &policy, &mut ProvenanceLog::new());
                let want = if truth {
                    Containment::Inside
                } else {
                    Containment::Outside
                };
                assert_eq!(said, want, "at {p:?}");
                checked += 1;
            }
        }
        assert!(checked > 10_000, "only {checked} points were compared");
    }
}

/// How many solids each rule leaves, in the order [`RULES`] runs them.
type Solids = [usize; 4];

/// Two solids that overlap and neither holds the other: one piece each way.
const OVERLAPPING: Solids = [1, 1, 1, 1];

/// Two solids that never meet: both, nothing, and each untouched.
const APART: Solids = [2, 0, 1, 1];

/// Union, intersection, cone less block, block less cone.
const RULES: [(Rule, bool); 4] = [
    (Rule::Union, true),
    (Rule::Intersection, true),
    (Rule::Difference, true),
    (Rule::Difference, false),
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

#[derive(Clone, Copy, Debug)]
struct Case {
    cone: ConeShape,
    block: BoxShape,
    solids: Solids,
}

#[derive(Debug, PartialEq)]
struct Tally {
    right: usize,
    refused: BTreeMap<&'static str, usize>,
}

/// A block 12 by 10 by 15 whose low-`x` face is the plane
/// `x = lean · (z − crossing)`, the block on the `+x` side of it, then turned
/// about the cone's axis.
///
/// Only 10 wide and running from `z ≈ −6` to `z ≈ 9`, so its two walls square
/// to `y` stay clear of the double cone and its top and bottom cut it in
/// ellipses: the one steep face is the face under test.
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

/// A cube with its diagonal brought upright and its middle on the axis at
/// `z = middle`, turned about the axis.
fn a_corner_up_cube(side: f64, middle: f64, turn: f64) -> BoxShape {
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction");
    let stood = Isometry::rotation(&diagonal, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn");
    let placed = about_z(turn)
        .compose(&moved([0.0, 0.0, middle - side * 3.0f64.sqrt() / 2.0]))
        .compose(&stood);
    BoxShape {
        size: [side; 3],
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

/// Every way a fuse in this file refuses today.
const REFUSALS: [&str; 4] = [
    "a point inside a piece of a face",
    "order along a crossing",
    "Geometry(CannotNameWhichPieceOfACrossingThisIs)",
    "AWireWillNotLieFlatOnItsFace",
];

/// Runs every rule over every case. Panics on a wrong answer; counts the
/// right ones and the refusals by name.
fn tally(cases: &[Case]) -> Tally {
    let mut tally = Tally {
        right: 0,
        refused: BTreeMap::new(),
    };
    for case in cases {
        for (which, (rule, cone_first)) in RULES.into_iter().enumerate() {
            match fuse_and_check(case, rule, cone_first, case.solids[which]) {
                None => tally.right += 1,
                Some(name) => {
                    let known = REFUSALS
                        .into_iter()
                        .find(|it| *it == name)
                        .unwrap_or_else(|| panic!("a refusal no family here expects: {name}"));
                    *tally.refused.entry(known).or_default() += 1;
                }
            }
        }
    }
    println!("{tally:?}");
    tally
}

/// `None` when the fuse answered and the answer is right; the refusal's name
/// when it refused.
///
/// # Panics
///
/// When the fuse answers wrongly, which is what this file is looking for.
fn fuse_and_check(case: &Case, rule: Rule, cone_first: bool, want_solids: usize) -> Option<String> {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let (cone, block) = build(&mut model, case);
    let (arguments, tools) = if cone_first {
        ([cone], [block])
    } else {
        ([block], [cone])
    };
    let answer = match model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(answer) => answer,
        Err(Error::Undecidable { records }) => {
            let record = records.records().last().expect("a refusal names why");
            return Some(record.predicate().to_owned());
        }
        Err(other) => {
            let text = format!("{other:?}");
            return Some(text.split(' ').next().unwrap_or_default().to_owned());
        }
    };
    let what = format!("{rule:?} (cone first: {cone_first}) of {case:?}");
    let pieces: Vec<SolidKey> = answer.solids().to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|error| panic!("{what}: the answer does not validate: {error:?}"));
    assert_eq!(pieces.len(), want_solids, "{what}: how many solids");

    let c = case.cone;
    let cone_volume =
        core::f64::consts::PI * c.slope * c.slope * (c.far.powi(3) - c.near.powi(3)) / 3.0;
    let block_volume: f64 = case.block.size.iter().product();
    let shared = shared_volume(case);
    let want = match (rule, cone_first) {
        (Rule::Union, _) => cone_volume + block_volume - shared,
        (Rule::Intersection, _) => shared,
        (Rule::Difference, true) => cone_volume - shared,
        (Rule::Difference, false) => block_volume - shared,
        _ => panic!("a rule this file does not run"),
    };
    let how_fine = Deflection::of(0.002).expect("a positive length");
    let (mut got, mut bound) = (0.0, 0.0);
    for &piece in &pieces {
        let measured = model
            .solid(piece)
            .expect("this model made the key")
            .measure(how_fine, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|error| panic!("{what}: a piece does not measure: {error:?}"));
        got += measured.volume();
        bound += measured.volume_error();
    }
    assert!(
        bound.is_finite(),
        "{what}: a bound of {bound} checks nothing"
    );
    let slack = bound + 0.002 * cone_volume;
    assert!(
        (got - want).abs() <= slack,
        "{what}: volume {got}, where the shapes say {want} (allowed {slack})"
    );

    let mut decided = 0;
    for p in clear_grid_points(case) {
        let a = in_cone(case.cone, p);
        let b = in_block(case.block, p);
        let truth = match (rule, cone_first) {
            (Rule::Union, _) => a || b,
            (Rule::Intersection, _) => a && b,
            (Rule::Difference, true) => a && !b,
            (Rule::Difference, false) => b && !a,
            _ => panic!("a rule this file does not run"),
        };
        let mut inside = false;
        let mut sure = true;
        for &piece in &pieces {
            match model
                .solid(piece)
                .expect("this model made the key")
                .classify_point(p, &policy, &mut ProvenanceLog::new())
            {
                Containment::Inside => inside = true,
                Containment::Outside => {}
                Containment::OnBoundary
                | Containment::Undecidable { .. }
                | Containment::CannotFollowAFacesTrimming { .. }
                | Containment::CannotFireARayAtAFacesSurface { .. }
                | _ => sure = false,
            }
        }
        if sure {
            assert_eq!(inside, truth, "{what}: at {p:?}");
            decided += 1;
        }
    }
    // 8 000 and not the 10 000 this file first had: once a cone face's box
    // counted every place its curved sides can turn (§202.3), the fewest any
    // answer here decided fell to 8879. What moved was not the answer but where
    // the rays go. Measured on the frustum and side-6 corner-up cube's union,
    // which decided 9442: its cone face's box reaches z ≈ 17.74, where one cube
    // plane's ellipse turns far above the arc the face keeps; the solid's box
    // grows to hold it, its ray targets move with it, and 16040 of that
    // answer's 20780 undecided points then refused on the frustum's big cap,
    // whose trimming the ray caster cannot follow.
    assert!(
        decided > 8_000,
        "{what}: only {decided} grid points could be classified"
    );
    None
}

fn build(model: &mut Model, case: &Case) -> (SolidKey, SolidKey) {
    let cone = if case.cone.near == 0.0 {
        a_pointed_cone(model, case.cone.slope, case.cone.far)
    } else {
        a_frustum(model, case.cone.slope, case.cone.near, case.cone.far)
    };
    let [x, y, z] = case.block.size;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    let block = model
        .add_extrusion(&rectangle, z, &case.block.placed)
        .expect("a rectangle extrudes");
    (cone, block)
}

/// Points half a unit apart over `|x|, |y| ≤ 6`, `|z| ≤ 12`, nudged off the
/// lattice so none lands on an axis or a cap, and keeping only those further
/// than `0.02` from every face of either input.
fn clear_grid_points(case: &Case) -> impl Iterator<Item = Point> + '_ {
    (-12..=12).flat_map(move |i| {
        (-12..=12).flat_map(move |j| {
            (-24..=24).filter_map(move |k| {
                let p = Point::new(
                    f64::from(i) * 0.5 + 0.1,
                    f64::from(j) * 0.5 + 0.07,
                    f64::from(k) * 0.5 + 0.03,
                );
                (distance_to_a_boundary(case, p) >= 0.02).then_some(p)
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

/// Not the exact distance, but never more than it: the distance to the
/// nearest of the cone's surface, the two cap planes and the box's six planes.
fn distance_to_a_boundary(case: &Case, p: Point) -> f64 {
    let cone = case.cone;
    let r = p.x().hypot(p.y());
    let to_side = (cone.slope * p.z() - r).abs() / cone.slope.hypot(1.0);
    let to_caps = (p.z() - cone.near).abs().min((p.z() - cone.far).abs());
    let local = case.block.placed.inverse().apply_to_point(p).coordinates();
    let to_block = (0..3)
        .map(|i| local[i].abs().min((local[i] - case.block.size[i]).abs()))
        .fold(f64::INFINITY, f64::min);
    to_side.min(to_caps).min(to_block)
}

/// The volume the cone and the box share, summed over 800 × 800 vertical
/// columns across the cone's widest circle: along each column the cone is one
/// interval in `z`, and the box, being convex, is another.
fn shared_volume(case: &Case) -> f64 {
    let cone = case.cone;
    let reach = cone.slope * cone.far;
    let steps = 800;
    let h = 2.0 * reach / f64::from(steps);
    let inverse = case.block.placed.inverse();
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
                    if at[k] < 0.0 || at[k] > case.block.size[k] {
                        to = from;
                    }
                    continue;
                }
                let one = -at[k] / up[k];
                let other = (case.block.size[k] - at[k]) / up[k];
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
