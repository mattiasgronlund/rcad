//! Flat faces with an arc among their sides, asked about a point on the line
//! through two of their corners, given as a box a rounding wide
//! (`docs/decisions.md` §191.5, row Q2035).
//!
//! `holds.rs` winds such a face's own sides, each arc read as its chord, and
//! runs its half-line from the point towards one corner. A box on the line
//! from that corner through another straddles it, so that other corner's side
//! of the line has no sign, and the face was refused `Undecidable` with no
//! second aim — the refusal row Q1834 lifted from faces with straight sides
//! alone. How often the sides wind round a point does not depend on which way
//! the half-line runs, so the face tries every corner and then the middle of
//! every side, as a face with straight sides does.
//!
//! Two faces reach that walk. A round bar cut in two beside its axis, its half
//! shared with the block over `x < c` bitten by a ball whose circle on the cut
//! takes an arc out of the top of that rectangle: the steps that read a face as
//! the part in front of all its cuts say nothing about it. And the top of a
//! plate with a round tab and a round hole, which winds straight away because
//! of the hole's circle.
//!
//! A third face needs the middle of a side, and an exact point (row Q2075):
//! the cap of a prism, built face by face, whose two arcs have their chords on
//! the two diagonals of a square, asked about the point where the diagonals
//! cross. Every corner is on one of the two chords and the arcs share no end, so
//! no corner is an aim off both chords, and before the middles were tried the
//! face was refused `AFaceTrimmedByACurveThisCannotFollow`.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, FaceView, Model, OnAFace, ProfileBuilder, Rule, SolidBuilder, SolidKey, SolidView,
};

/// Where the plane cuts the bar: off the axis on either side, and through it.
/// The ball stays clear of the bar's wall for each.
const CUTS: [f64; 5] = [0.5, 0.25, 0.125, -0.5, 0.0];

/// Where the bar's bottom is and how tall it is.
const BARS: [(f64, f64); 3] = [(2.0, 2.0), (0.0, 1.0), (0.0, 2.0)];

/// A point a quarter of the way along either diagonal of the cut face's
/// rectangle, from either end, as a box a rounding wide, is inside the face,
/// and the face says so.
#[test]
fn a_half_bars_cut_face_bitten_by_a_ball_holds_a_box_on_either_diagonal() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            let mut model = a_model();
            let half = a_half_bar_bitten_by_a_ball(&mut model, cut, bottom, height);
            let view = model.solid(half).expect("this model made the key");
            let (face, [first, second, third, fourth]) =
                the_cut_face(view, cut, bottom, bottom + height);
            for [from, to] in [
                [first, third],
                [third, first],
                [second, fourth],
                [fourth, second],
            ] {
                let point = a_box_part_of_the_way(from, to, 4.0);
                let asked = format!("cut at x = {cut}, bar from z = {bottom}, {point:?}");
                assert_eq!(the_face_holds(view, face, point, &asked), OnAFace::Held);
            }
        }
    }
}

/// The middle of the rectangle, as a box a rounding wide, is on both its
/// diagonals, so a half-line aimed at any of its four corners runs on through
/// the opposite one, and it is still inside the face, clear of the bite.
#[test]
fn a_half_bars_cut_face_bitten_by_a_ball_holds_a_box_at_its_middle() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            let mut model = a_model();
            let half = a_half_bar_bitten_by_a_ball(&mut model, cut, bottom, height);
            let view = model.solid(half).expect("this model made the key");
            let (face, [first, _, third, _]) = the_cut_face(view, cut, bottom, bottom + height);
            let point = a_box_part_of_the_way(first, third, 2.0);
            let asked = format!("cut at x = {cut}, bar from z = {bottom}, {point:?}");
            assert_eq!(the_face_holds(view, face, point, &asked), OnAFace::Held);
        }
    }
}

/// A point a quarter of a diagonal past either end of it, as a box a rounding
/// wide, is on the same line and off the face, and the face says that too.
#[test]
fn a_half_bars_cut_face_bitten_by_a_ball_holds_no_box_past_a_corner_on_a_diagonal() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            let mut model = a_model();
            let half = a_half_bar_bitten_by_a_ball(&mut model, cut, bottom, height);
            let view = model.solid(half).expect("this model made the key");
            let (face, [first, second, third, fourth]) =
                the_cut_face(view, cut, bottom, bottom + height);
            for [from, to] in [
                [first, third],
                [third, first],
                [second, fourth],
                [fourth, second],
            ] {
                let point = a_box_part_of_the_way(from, to, 0.8);
                let asked = format!("cut at x = {cut}, bar from z = {bottom}, {point:?}");
                assert_eq!(the_face_holds(view, face, point, &asked), OnAFace::NotHeld);
            }
        }
    }
}

/// The top of a plate `10 × 10 × 2` with a round tab of radius 3 at `(10, 5)`
/// and a round hole of radius 1 at `(3, 5)`. The square's corners are exact,
/// but a box a rounding wide round a point of either diagonal still straddles
/// that diagonal, so the half-line aimed along it had no sign for the corner at
/// its other end. Either diagonal passes the hole `√2` from its centre.
#[test]
fn a_plates_top_with_a_round_tab_and_a_round_hole_holds_a_box_on_either_diagonal() {
    let mut model = a_model();
    let plate = a_box(&mut model, [[0.0, 10.0], [0.0, 10.0], [0.0, 2.0]]);
    let tab = model
        .add_round_bar(3.0, 2.0, &shift(10.0, 5.0, 0.0))
        .expect("a bar of radius three");
    let tabbed = the_one_solid(&mut model, plate, tab, Rule::Union);
    let hole = model
        .add_round_bar(1.0, 4.0, &shift(3.0, 5.0, -1.0))
        .expect("a bar of radius one");
    let holed = the_one_solid(&mut model, tabbed, hole, Rule::Difference);
    let view = model.solid(holed).expect("this model made the key");
    let top = view
        .faces()
        .find(|face| {
            matches!(face.surface(), Surface::Plane(flat) if flat.origin().coordinates()[2] == 2.0)
        })
        .expect("the plate has a top");
    let [first, second, third, fourth] = [
        [0.0, 0.0, 2.0],
        [10.0, 0.0, 2.0],
        [10.0, 10.0, 2.0],
        [0.0, 10.0, 2.0],
    ];
    for ([from, to], parts, held) in [
        ([first, third], 4.0, OnAFace::Held),
        ([third, first], 4.0, OnAFace::Held),
        ([second, fourth], 4.0, OnAFace::Held),
        ([fourth, second], 4.0, OnAFace::Held),
        ([first, third], 2.0, OnAFace::Held),
        ([first, third], 0.8, OnAFace::NotHeld),
        ([second, fourth], 0.8, OnAFace::NotHeld),
    ] {
        let point = a_box_part_of_the_way(from, to, parts);
        assert_eq!(
            the_face_holds(view, top, point, &format!("{point:?}")),
            held,
            "{point:?}"
        );
    }
}

/// The cap of a prism whose two arcs have their chords on the two diagonals of
/// the square `|x|, |y| ≤ 7`, asked about the origin, where the two chords
/// cross (row Q2075). Its four corners are the square's. The first arc runs
/// from `(−7, −7)` to `(7, 7)` the long way round the circle of radius 10 about
/// `(1, −1)`, outside the square below it and to its right. The top side runs
/// back along `y = 7` to `(−7, 7)`. The second arc runs from there to `(7, −7)`
/// round the circle of radius 26 about `(c, c)`, a shallow bow across the
/// square. The bottom side runs back along `y = −7`. Every corner is exact,
/// since `6, 8, 10` and `10, 24, 26` are whole.
///
/// Every corner is on one of the two chords and the two arcs share no end, so
/// the only aims off both chords are the middles of the top and the bottom
/// side, `(0, ±7)`.
///
/// With `c = 17` the second arc bows towards `(−1, −1)`. The face is the part of
/// the square inside its circle, and the part of the first arc's disc outside
/// the square below `y = x`. The origin is `17√2`, about `24.04`, from `(17,
/// 17)`, inside that circle, so the face holds it. With `c = −17` the arc bows
/// towards `(1, 1)`, and the face is the part of the square above `y = −x`
/// outside its circle, with the same piece outside the square. The origin is
/// inside that circle, so the face does not hold it. Either way the four points
/// one away from the origin along an axis, which are off both chords, answer
/// what the origin does.
#[test]
fn a_cap_whose_two_arcs_have_their_chords_on_the_diagonals_holds_where_they_cross_as_one_bows() {
    for (centre, held) in [(17.0, OnAFace::Held), (-17.0, OnAFace::NotHeld)] {
        let mut model = a_model();
        let prism = a_prism_with_two_arcs_on_the_diagonals(&mut model, centre);
        let view = model.solid(prism).expect("this model made the key");
        for z in [0.0, 2.0] {
            let cap = the_flat_face_at(view, z);
            let mut corners: Vec<[f64; 3]> = cap
                .wires()
                .flat_map(|wire| wire.coedges())
                .map(|side| side.start().point().coordinates())
                .collect();
            corners.sort_by(|one, other| one.partial_cmp(other).expect("corners are numbers"));
            assert_eq!(
                corners,
                [
                    [-7.0, -7.0, z],
                    [-7.0, 7.0, z],
                    [7.0, -7.0, z],
                    [7.0, 7.0, z]
                ],
                "the second arc about ({centre}, {centre}), the cap at z = {z}"
            );
            for [x, y] in [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]] {
                let asked = format!("the second arc about ({centre}, {centre}), ({x}, {y}, {z})");
                let point = PredicatePoint::from([x, y, z]);
                assert_eq!(the_face_holds(view, cap, point, &asked), held, "{asked}");
            }
        }
    }
}

/// The prism from `z = 0` to `z = 2` over the face
/// [`a_cap_whose_two_arcs_have_their_chords_on_the_diagonals_holds_where_they_cross_as_one_bows`]
/// describes, with the second arc about `(centre, centre)`, built face by face:
/// a fuse of bars and blocks leaves corners a rounding off the diagonals, or
/// splits an arc where another of its surfaces crosses it.
fn a_prism_with_two_arcs_on_the_diagonals(model: &mut Model, centre: f64) -> SolidKey {
    let a_plane = |[x, y, z]: [f64; 3], [along_x, along_y, along_z]: [f64; 3]| {
        Plane::from_point_and_normal(Point::new(x, y, z), Vector::new(along_x, along_y, along_z))
            .expect("an axis direction has length")
    };
    let bottom = a_plane([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
    let top = a_plane([0.0, 0.0, 2.0], [0.0, 0.0, 1.0]);
    let an_arc = |cap: Plane, [x, y]: [f64; 2]| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cap),
        picked_by: [Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(x, y, 0.0))
                .expect("a direction off the axis has length"),
        )]
        .into(),
    };
    let first_bows = [1.0, -1.0];
    let second_bows = [-centre, -centre];
    let a_pipe = |x: f64, y: f64, radius: f64| {
        let axis = Axis::new(Point::new(x, y, 0.0), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis has length");
        Surface::Cylinder(Cylinder::new(axis, radius).expect("a positive radius"))
    };

    let mut builder = SolidBuilder::new();
    let corners = [
        [-7.0, -7.0, 0.0],
        [7.0, 7.0, 0.0],
        [-7.0, 7.0, 0.0],
        [7.0, -7.0, 0.0],
        [-7.0, -7.0, 2.0],
        [7.0, 7.0, 2.0],
        [-7.0, 7.0, 2.0],
        [7.0, -7.0, 2.0],
    ]
    .map(|[x, y, z]| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a finite corner")
    });
    let edges = [
        (0, 1, an_arc(bottom, first_bows)),
        (1, 2, EdgeRun::Straight),
        (2, 3, an_arc(bottom, second_bows)),
        (3, 0, EdgeRun::Straight),
        (4, 5, an_arc(top, first_bows)),
        (5, 6, EdgeRun::Straight),
        (6, 7, an_arc(top, second_bows)),
        (7, 4, EdgeRun::Straight),
        (0, 4, EdgeRun::Straight),
        (1, 5, EdgeRun::Straight),
        (2, 6, EdgeRun::Straight),
        (3, 7, EdgeRun::Straight),
    ]
    .map(|(from, to, run)| {
        builder
            .add_edge(corners[from], corners[to], run)
            .expect("an edge between two corners")
    });

    let (forward, back) = (Orientation::Forward, Orientation::Reversed);
    let inside_the_second_pipe = if centre > 0.0 { forward } else { back };
    let faces = [
        (
            Surface::Plane(bottom),
            forward,
            [(3, back), (2, back), (1, back), (0, back)],
        ),
        (
            Surface::Plane(top),
            forward,
            [(4, forward), (5, forward), (6, forward), (7, forward)],
        ),
        (
            a_pipe(1.0, -1.0, 10.0),
            forward,
            [(0, forward), (9, forward), (4, back), (8, back)],
        ),
        (
            Surface::Plane(a_plane([0.0, 7.0, 0.0], [0.0, 1.0, 0.0])),
            forward,
            [(1, forward), (10, forward), (5, back), (9, back)],
        ),
        (
            a_pipe(centre, centre, 26.0),
            inside_the_second_pipe,
            [(2, forward), (11, forward), (6, back), (10, back)],
        ),
        (
            Surface::Plane(a_plane([0.0, -7.0, 0.0], [0.0, 1.0, 0.0])),
            forward,
            [(3, forward), (8, forward), (7, back), (11, back)],
        ),
    ];
    let mut skin = Vec::with_capacity(faces.len());
    for (surface, used, steps) in faces {
        let walked = steps.map(|(which, way)| {
            DraftStep::Along(builder.add_coedge(edges[which], way).expect("a coedge"))
        });
        let wire = builder.add_wire(&walked).expect("a wire that closes");
        skin.push((builder.add_face(surface, &[wire]).expect("a face"), used));
    }
    builder.add_shell(&skin).expect("a shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the prism about ({centre}, {centre}) is not sound: {why}"))
}

/// The face on the plane `z = height`, facing up or down.
fn the_flat_face_at(view: SolidView<'_>, height: f64) -> FaceView<'_> {
    view.faces()
        .find(|face| {
            let Surface::Plane(flat) = face.surface() else {
                return false;
            };
            flat.normal().components()[2].abs() == 1.0 && flat.origin().coordinates()[2] == height
        })
        .unwrap_or_else(|| panic!("a flat face at z = {height}"))
}

/// The bar of radius 1 from `z = bottom` up `height` shared with the block over
/// `x < cut`, less a ball of radius `0.3` at `(cut + 0.1, 0, top + 0.1)`. The
/// ball's circle on the cut has radius `√0.08` and its centre `0.1` above the
/// top, so it takes an arc out of the top of the cut face between
/// `y = ±√0.07`, and its poles are off the cut.
fn a_half_bar_bitten_by_a_ball(model: &mut Model, cut: f64, bottom: f64, height: f64) -> SolidKey {
    let bar = model
        .add_round_bar(1.0, height, &shift(0.0, 0.0, bottom))
        .expect("a bar of radius one");
    let block = a_box(model, [[-10.0, cut], [-10.0, 10.0], [-10.0, 10.0]]);
    let half = the_one_solid(model, bar, block, Rule::Intersection);
    let ball = model
        .add_ball(0.3, &shift(cut + 0.1, 0.0, bottom + height + 0.1))
        .expect("a ball of radius 0.3");
    the_one_solid(model, half, ball, Rule::Difference)
}

/// The half's face on the plane `x = cut`, and the four corners of the
/// rectangle the bar cuts that plane in, which are corners of the face too:
/// low `y` and high `y` at the bottom, then high `y` and low `y` at the top.
fn the_cut_face(
    view: SolidView<'_>,
    cut: f64,
    bottom: f64,
    top: f64,
) -> (FaceView<'_>, [[f64; 3]; 4]) {
    let face = view
        .faces()
        .find(|face| {
            let Surface::Plane(flat) = face.surface() else {
                return false;
            };
            flat.normal().components() == [1.0, 0.0, 0.0] && flat.offset() == cut
        })
        .expect("the half has the cut as a face");
    let corners: Vec<[f64; 3]> = face
        .wires()
        .flat_map(|wire| wire.coedges())
        .map(|side| side.start().point().coordinates())
        .collect();
    let widest = |z: f64, sign: f64| {
        *corners
            .iter()
            .filter(|corner| corner[2] == z && corner[1] * sign > 0.0)
            .max_by(|one, other| one[1].abs().total_cmp(&other[1].abs()))
            .unwrap_or_else(|| panic!("a corner at z = {z} on that side: {corners:?}"))
    };
    (
        face,
        [
            widest(bottom, -1.0),
            widest(bottom, 1.0),
            widest(top, 1.0),
            widest(top, -1.0),
        ],
    )
}

/// What `face` says of `point`, certified, having logged nothing on the way.
fn the_face_holds(
    view: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
    asked: &str,
) -> OnAFace {
    let mut log = ProvenanceLog::new();
    let said = view
        .face_holds_the_point(face.key(), point, &Policy::certified(), &mut log)
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    assert!(
        log.is_empty(),
        "{asked}: a certified answer logs nothing: {log:?}"
    );
    said
}

/// The point `1 / parts` of the way from `from` to `to`, as a box one rounding
/// wide either side along every axis the two differ on.
fn a_box_part_of_the_way(from: [f64; 3], to: [f64; 3], parts: f64) -> PredicatePoint {
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        if from[axis] == to[axis] {
            return Interval::exactly(from[axis]);
        }
        let near = ((parts - 1.0) * from[axis] + to[axis]) / parts;
        Interval::new(near.next_down(), near.next_up()).expect("a box round a finite number")
    }))
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

fn shift(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let rectangle =
        ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
    model
        .add_extrusion(&rectangle, z[1] - z[0], &shift(0.0, 0.0, z[0]))
        .expect("a positive height")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{rule:?}: {refused:?}"));
    let [solid] = fused.solids() else {
        panic!("{rule:?} is one solid: {:?}", fused.solids());
    };
    *solid
}
