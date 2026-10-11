//! A block less a bar, and a block less a ball, asked which points they hold
//! (`docs/decisions.md` §331.5, row Q610).
//!
//! **Each answer has one curved face used reversed**, the wall of the round
//! slot or of the dimple, and every side of that face is a piece of where its
//! surface crosses a plane of the block. So `classify_point` reads that face
//! through `holds.rs`'s `which_side_of_its_neighbour_a_side_keeps`, which says
//! which side of each plane the face keeps. Since §331.5 that function reads
//! `∇S` as the stored surface's own gradient whichever way the face is used. Put
//! back the turn of `∇S` for a face used reversed, which §331.5 took out, and
//! the slots' walls refuse (`CannotFollowAFacesTrimming`) while the dimples'
//! walls read as the rest of the ball and put points in the dimple inside the
//! block. Before these, the only test that drove a reversed face through that
//! function was `every_first_answer_says_which_points_it_holds`, on a
//! crescent's inner wall.
//!
//! **The oracle is the shape, worked out from the coordinates alone**: a point
//! is inside exactly when it is inside the block and not inside the bar or the
//! ball. Every point asked is at least [`CLEAR`] from every surface either
//! solid has, so none is on the skin and the answer is one or the other. The
//! points come in pairs, one each side of a plane the curved face is cut by,
//! or one each side of the curved face itself. The volume is checked against
//! the space taken out, worked by hand, within the error the measure publishes.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, ProfileBuilder, Rule, SolidKey};

/// The radius of every bar and ball here.
const RADIUS: f64 = 5.0;

/// The block runs from `−10` to `10` along `x` and `y`.
const HALF_WIDTH: f64 = 10.0;

/// The block runs from `0` to this up `z`.
const HEIGHT: f64 = 10.0;

/// Where every bar starts, and so the floor of every slot: halfway up the
/// block, so the floor is a plane the wall is cut by with the block on both
/// sides of it.
const FLOOR: f64 = 5.0;

/// How far every point asked is from every surface of the block and of the
/// bar or ball taken out of it.
const CLEAR: f64 = 0.4;

/// **A round slot down the block's side, three ways: half a pipe, less than
/// half, and a quarter at a corner.**
///
/// The bar stands from [`FLOOR`] to well over the top, so the slot's wall is a
/// piece of the pipe cut by the floor, the block's top, and the side of the
/// block the slot opens through, along two rulings of the pipe. For the
/// quarter, those are the two sides of the block that meet at the corner.
///
/// The space taken out is the slot's cross-section times its depth,
/// `HEIGHT − FLOOR`. Half a disc is `π·r²/2`, a quarter `π·r²/4`. The bar three
/// past the side cuts off the part of the disc past a chord three from its
/// centre, `r²·acos(3/5) − 3·4`, where the side meets the pipe at `(10, ±4)`.
#[test]
fn a_point_beside_a_round_slot_down_a_blocks_side_is_placed_as_worked_by_hand() {
    let slots = [
        Slot {
            name: "a half-round slot, its axis on the block's side",
            axis: [10.0, 0.0],
            cross_section: PI * RADIUS * RADIUS / 2.0,
            points: &[
                [8.0, 0.0, 4.0],
                [8.0, 0.0, 6.0],
                [8.0, 0.0, 9.0],
                [8.0, 0.0, 11.0],
                [4.0, 0.0, 9.0],
                [4.0, 0.0, 11.0],
                [9.5, 0.0, 7.0],
                [10.5, 0.0, 7.0],
                [9.5, 6.0, 7.0],
                [10.5, 6.0, 7.0],
                [4.5, 0.0, 7.0],
                [5.5, 0.0, 7.0],
                [9.0, 4.4, 7.0],
                [9.0, 5.6, 7.0],
            ],
        },
        Slot {
            name: "a shallow slot, its axis three past the block's side",
            axis: [13.0, 0.0],
            cross_section: RADIUS * RADIUS * (3.0_f64 / 5.0).acos() - 3.0 * 4.0,
            points: &[
                [9.0, 0.0, 4.0],
                [9.0, 0.0, 6.0],
                [9.0, 0.0, 9.0],
                [9.0, 0.0, 11.0],
                [7.0, 0.0, 9.0],
                [7.0, 0.0, 11.0],
                [9.5, 0.0, 7.0],
                [10.5, 0.0, 7.0],
                [9.5, 5.0, 7.0],
                [10.5, 5.0, 7.0],
                [8.5, 0.0, 7.0],
                [7.5, 0.0, 7.0],
            ],
        },
        Slot {
            name: "a quarter-round notch, its axis on the block's corner",
            axis: [10.0, 10.0],
            cross_section: PI * RADIUS * RADIUS / 4.0,
            points: &[
                [8.0, 8.0, 4.0],
                [8.0, 8.0, 6.0],
                [6.0, 6.0, 9.0],
                [6.0, 6.0, 11.0],
                [9.5, 4.0, 7.0],
                [10.5, 4.0, 7.0],
                [4.0, 9.5, 7.0],
                [4.0, 10.5, 7.0],
                [7.0, 7.0, 7.0],
                [6.0, 6.0, 7.0],
            ],
        },
    ];
    for Slot {
        name,
        axis: [x, y],
        cross_section,
        points,
    } in slots
    {
        let mut model = a_model();
        let block = a_block(&mut model);
        let bar = model
            .add_round_bar(
                RADIUS,
                3.0 * HEIGHT,
                &Isometry::translation(Vector::new(x, y, FLOOR)).expect("a shift"),
            )
            .expect("a bar");
        let in_the_bar = |[px, py, pz]: [f64; 3]| {
            let off_the_axis = (px - x).hypot(py - y);
            (
                off_the_axis < RADIUS && pz > FLOOR,
                (off_the_axis - RADIUS).abs().min((pz - FLOOR).abs()),
            )
        };
        let (model, answer) = the_block_less(model, block, bar, name);
        let removed = cross_section * (HEIGHT - FLOOR);
        is_placed_as_worked_by_hand(&model, answer, removed, points, in_the_bar, name);
    }
}

/// **A round dimple in the block's top edge, two ways.**
///
/// A ball whose centre is on the edge where the block's top meets its side
/// takes out a quarter of itself, `π·r³/3`. One whose centre is on the side,
/// three under the top, takes out half of what is left of it when the top cuts
/// off a cap two deep: the cap is `π·2²·(3r − 2)/3`, so half the rest is
/// `(4π·r³/3 − 52π/3)/2`. Its top meets the side at `(10, ±4, 10)`.
#[test]
fn a_point_beside_a_round_dimple_in_a_blocks_edge_is_placed_as_worked_by_hand() {
    let dimples = [
        Dimple {
            name: "a quarter of a ball, its centre on the block's top edge",
            centre: [10.0, 0.0, 10.0],
            removed: PI * RADIUS.powi(3) / 3.0,
            points: &[
                [9.5, 0.0, 9.5],
                [10.5, 0.0, 9.5],
                [9.5, 6.0, 9.5],
                [10.5, 6.0, 9.5],
                [6.0, 0.0, 9.5],
                [6.0, 0.0, 10.5],
                [4.0, 0.0, 9.5],
                [4.0, 0.0, 10.5],
                [7.0, 0.0, 7.0],
                [6.0, 0.0, 6.0],
                [9.0, 3.0, 7.0],
                [9.0, 4.0, 6.0],
            ],
        },
        Dimple {
            name: "half a ball less a cap, its centre on the block's side",
            centre: [10.0, 0.0, 7.0],
            removed: (4.0 * PI * RADIUS.powi(3) / 3.0 - 52.0 * PI / 3.0) / 2.0,
            points: &[
                [9.5, 0.0, 7.0],
                [10.5, 0.0, 7.0],
                [9.5, 6.0, 7.0],
                [10.5, 6.0, 7.0],
                [9.0, 0.0, 9.5],
                [9.0, 0.0, 10.5],
                [9.0, 5.0, 9.5],
                [9.0, 5.0, 10.5],
                [5.5, 0.0, 7.0],
                [4.5, 0.0, 7.0],
                [9.0, 0.0, 2.6],
                [9.0, 0.0, 1.5],
            ],
        },
    ];
    for Dimple {
        name,
        centre,
        removed,
        points,
    } in dimples
    {
        let mut model = a_model();
        let block = a_block(&mut model);
        let ball = model
            .add_ball(
                RADIUS,
                &Isometry::translation(Vector::new(centre[0], centre[1], centre[2]))
                    .expect("a shift"),
            )
            .expect("a ball");
        let in_the_ball = |point: [f64; 3]| {
            let from_the_centre = (0..3)
                .map(|axis| (point[axis] - centre[axis]).powi(2))
                .sum::<f64>()
                .sqrt();
            (from_the_centre < RADIUS, (from_the_centre - RADIUS).abs())
        };
        let (model, answer) = the_block_less(model, block, ball, name);
        is_placed_as_worked_by_hand(&model, answer, removed, points, in_the_ball, name);
    }
}

/// A bar of [`RADIUS`] standing from [`FLOOR`], taken out of the block, and the
/// points asked of what is left.
struct Slot {
    name: &'static str,
    /// Where the bar's axis is, as `x` and `y`.
    axis: [f64; 2],
    /// The area of the slot across, worked by hand.
    cross_section: f64,
    points: &'static [[f64; 3]],
}

/// A ball of [`RADIUS`] taken out of the block, and the points asked of what is
/// left.
struct Dimple {
    name: &'static str,
    centre: [f64; 3],
    /// The space the ball takes out of the block, worked by hand.
    removed: f64,
    points: &'static [[f64; 3]],
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_block(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [-HALF_WIDTH, -HALF_WIDTH],
        [HALF_WIDTH, -HALF_WIDTH],
        [HALF_WIDTH, HALF_WIDTH],
        [-HALF_WIDTH, HALF_WIDTH],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square is a profile");
    model
        .add_extrusion(&square, HEIGHT, &Isometry::identity())
        .expect("a square extrudes")
}

/// The block less `tool`, as one solid that validates and has exactly one
/// curved face, used reversed: the face these tests are about.
fn the_block_less(
    mut model: Model,
    block: SolidKey,
    tool: SolidKey,
    name: &str,
) -> (Model, SolidKey) {
    let policy = Policy::certified();
    let fused = model
        .fuse(
            &[block],
            &[tool],
            Rule::Difference,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{name}: {refused:?}"));
    let [answer] = fused.solids() else {
        panic!("{name} is one solid: {:?}", fused.solids());
    };
    let answer = *answer;
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|broken| panic!("{name} validates: {broken:?}"));
    let solid = model.solid(answer).expect("this model made the key");
    let curved: Vec<_> = solid
        .faces()
        .filter(|face| face.surface().as_plane().is_none())
        .collect();
    let [wall] = curved.as_slice() else {
        panic!("{name} has one curved face: {} of them", curved.len());
    };
    assert!(
        wall.orientation().is_reversed(),
        "{name}: the curved face is used reversed"
    );
    (model, answer)
}

/// Every point of `points` placed where the block less the tool puts it, and
/// the volume the block less `removed`.
///
/// `in_the_tool` says whether a point is inside the bar or the ball, and how
/// far it is from that solid's own surfaces.
fn is_placed_as_worked_by_hand(
    model: &Model,
    answer: SolidKey,
    removed: f64,
    points: &[[f64; 3]],
    in_the_tool: impl Fn([f64; 3]) -> (bool, f64),
    name: &str,
) {
    let policy = Policy::certified();
    let solid = model.solid(answer).expect("this model made the key");
    for &point in points {
        let (inside_the_tool, off_the_tool) = in_the_tool(point);
        let [x, y, z] = point;
        let inside_the_block =
            x.abs() < HALF_WIDTH && y.abs() < HALF_WIDTH && 0.0 < z && z < HEIGHT;
        let off_the_block = [
            HALF_WIDTH - x.abs(),
            HALF_WIDTH - y.abs(),
            z.abs(),
            (HEIGHT - z).abs(),
        ]
        .into_iter()
        .map(f64::abs)
        .fold(f64::INFINITY, f64::min);
        assert!(
            off_the_block >= CLEAR && off_the_tool >= CLEAR,
            "{name}: {point:?} is clear of every surface"
        );
        let wanted = if inside_the_block && !inside_the_tool {
            Containment::Inside
        } else {
            Containment::Outside
        };
        let placed = solid.classify_point(Point::new(x, y, z), &policy, &mut ProvenanceLog::new());
        assert_eq!(placed, wanted, "{name} at {point:?}");
    }

    let measured = solid
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a solid measures");
    let by_hand = 2.0 * HALF_WIDTH * 2.0 * HALF_WIDTH * HEIGHT - removed;
    assert!(
        (measured.volume() - by_hand).abs() <= measured.volume_error(),
        "{name}: measured {} ± {} against {by_hand} by hand",
        measured.volume(),
        measured.volume_error()
    );
}
