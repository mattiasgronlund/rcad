//! Two bars of radius 1/4 whose axes meet at a slant (Q1646).
//!
//! The first bar runs along `x` from `through − (2, 0, 0)`, 4 long. The second
//! is turned onto `x` and then by `degrees` about `z`, and is either 32 long and
//! centred on `through = (11, 0, 0)`, so it runs right through the first, or 2
//! long and starts at `through = (11, −3.25, 0.5)`, so its near cap passes
//! through where the axes meet. Each test runs all six fuses — three rules,
//! each bar first in turn — and every one is still refused, each by the variant
//! pinned here.
//!
//! **What they should come to**, with `r = 1/4` and `a` the angle: the first
//! bar is `A = π/4`. What two pipes of one width share where their axes cross at
//! `a` is `16r³/(3 sin a)`, Steinmetz's `16r³/3` at a right angle, which is
//! `1/(12 sin a)` here. Through `(11, 0, 0)` the second bar is `B = 2π` and
//! runs well past that solid both ways, so the shared part is `I = 1/(12 sin
//! a)`. Through `(11, −3.25, 0.5)` the second bar is `B = π/8` and starts at
//! the meeting point. Turning every point half a turn about that point keeps
//! both axes where they are, so it keeps the shared solid too, and it swaps the
//! two sides of the cap: the cap cuts the solid in two equal halves, and `I =
//! 1/(24 sin a)`. In both, the union is `A + B − I`, the first less the second
//! `A − I` and the second less the first `B − I`.

use rcad_geom::{Axis, Isometry};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

/// The second bar runs right through the first at 50°, 60°, 70° or 100°.
///
/// **Every fuse is refused telling the cells of a pipe apart, the way the
/// square Steinmetz cross is** (Q1622,
/// `two_bars_of_one_radius_whose_axes_meet_are_refused_telling_the_cells_of_a_pipe_apart`).
/// Before Q1646 these were refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`,
/// where each bar's far cap cuts its own pipe in a circle and the arrangement
/// could not prove that circle misses the other pipe; and `a_half_of_an_ellipse`
/// did not know a half picked by flat surfaces that do not face along the
/// pipes' stored directions. Both are fixed in `fuse/arrange.rs`.
///
/// What stops them now was traced on the first pipe at 50°. The cap circle at
/// `x = 9` is to be matched with one of the four walks the half ellipses cut
/// the pipe into, and which one is read off how often each walk goes round the
/// axis, counted where it crosses a half-plane through the axis. Of the six
/// half-planes tried, two run along the pipe's own axis, and each of the other
/// four runs through a node: the two points the halves meet at sit on `±z`,
/// and the middles of the four halves on `±y`. So every count is `None`. A
/// half-plane across `(0, 1, 1)` misses them all and counts the four walks
/// `0, 1, 0, −1`; with it the union gets one step on and is refused
/// `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` in `fuse/assemble.rs`. But
/// diagonal half-planes answer three fuses of a cone beside a block wrongly
/// (Q1623, `a_cone_shared_with_its_union_with_a_block_is_the_cone`), so they
/// are not in.
///
/// **A gap, by §16.10**: the cells are there and each walk is in one of them.
/// The family of fixes is the half-plane: choose one clear of every node by an
/// argument that holds, which is Q1622's, followed by naming an edge out of a
/// point where the two pipes are tangent, which is also Q1622's.
#[test]
fn a_long_bar_through_another_at_a_slant_is_refused_telling_the_cells_of_a_pipe_apart() {
    for degrees in [50.0, 60.0, 70.0, 100.0] {
        for (rule, first_first) in six_fuses() {
            let said = fuse(degrees, [11.0, 0.0, 0.0], rule, first_first);
            assert!(
                matches!(said, Err(Error::CannotTellTheCellsOfAFaceApart { .. })),
                "{degrees}° {rule:?} (first first: {first_first}): {said:?}"
            );
        }
    }
}

/// The second bar runs right through the first at 120°. Its pickers are its
/// stored directions, so what `a_half_of_an_ellipse` learned in Q1646 does
/// not touch it. Its stored direction is `(−0.5, 0.866…, −2.2e−16)`, a hair off
/// the flat `z = 0`.
///
/// **Every fuse gets past the arrangement and is refused making the faces into
/// triangles or building their wires**: `CannotMeshAFace { why: CornersCoincide
/// }`, `AWireWillNotLieFlatOnItsFace` or `ACurvedFaceGoesRightRoundWithNoSeam`,
/// as pinned below. Before Q1646 all six were refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, at the far caps. Not traced
/// further. A lead, not run: the pipe's direction is off `z = 0`, so the two
/// points the halves meet at are off the `±z` half-planes and the count of how
/// often each walk goes round the axis settles, where at 50° it cannot; and
/// the way out of those two points is read in `fuse/assemble.rs` off the cross
/// of the two pipes' gradients, which is nought there but for rounding.
///
/// **A gap, by §16.10**: the answer is `1/(12 sin 120°)` shared, inside the
/// range a model holds.
#[test]
fn a_long_bar_through_another_at_120_degrees_is_refused_meshing_or_building_a_wire() {
    for (rule, first_first) in six_fuses() {
        let said = fuse(120.0, [11.0, 0.0, 0.0], rule, first_first);
        let pinned = match (rule, first_first) {
            (Rule::Union, true) | (Rule::Intersection, false) | (Rule::Difference, true) => {
                matches!(
                    said,
                    Err(Error::CannotMeshAFace {
                        why: rcad_mesh::CannotTriangulate::CornersCoincide { .. },
                        ..
                    })
                )
            }
            (Rule::Union | Rule::Difference, false) => {
                matches!(said, Err(Error::AWireWillNotLieFlatOnItsFace { .. }))
            }
            (Rule::Intersection, true) => {
                matches!(said, Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { .. }))
            }
            (_, _) => unreachable!("six_fuses names three rules and no other"),
        };
        assert!(pinned, "{rule:?} (first first: {first_first}): {said:?}");
    }
}

/// The second bar starts where the axes meet, at 50°, 60°, 70°, 100° or 120°.
///
/// **Every fuse is refused**: with the first bar first
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, and with the second first
/// `Undecidable`, asking which side of a pipe a box round `(11, −3.25, 0.25)`
/// is on. Neither moved in Q1646, and neither is traced to the function that
/// asks. At 120° both stand since Q1645 named the crossing there; before it,
/// `intersect` refused all six `CannotNameWhichPieceOfACrossingThisIs`.
///
/// The second bar's near cap is square to its axis through the meeting point
/// `c`, so it holds both points `c ± r·ẑ` where the two pipes touch, `(11,
/// −3.25, 0.25)` and `(11, −3.25, 0.75)`: there the cap and both pipes meet,
/// two of the three tangent to each other. At 50° to 100° the second pipe's
/// stored direction lies in `z = 0`, so both points are exact in `f64`; at 120°
/// it is a hair off `z = 0`, as the test above says.
///
/// **A gap, by §16.10**: the three surfaces meet exactly at two points known
/// from stored numbers, as the node where the four halves meet is
/// (`place_where_two_halves_meet` in `fuse/arrange.rs`). Naming the cap's meet
/// with the two pipes as that node is the family of fixes; the cells of the
/// pipes then meet the half-planes of the test above.
#[test]
fn a_bar_starting_where_it_crosses_another_at_a_slant_is_refused() {
    for degrees in [50.0, 60.0, 70.0, 100.0, 120.0] {
        for (rule, first_first) in six_fuses() {
            let said = fuse(degrees, [11.0, -3.25, 0.5], rule, first_first);
            let pinned = if first_first {
                matches!(
                    said,
                    Err(Error::Geometry(
                        rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                    ))
                )
            } else {
                matches!(said, Err(Error::Undecidable { .. }))
            };
            assert!(
                pinned,
                "{degrees}° {rule:?} (first first: {first_first}): {said:?}"
            );
        }
    }
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

fn fuse(degrees: f64, through: [f64; 3], rule: Rule, first_first: bool) -> Result<usize, Error> {
    let mut model =
        Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [first, second] = two_bars(&mut model, degrees, through);
    let (arguments, tools) = if first_first {
        ([first], [second])
    } else {
        ([second], [first])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().len())
}

fn two_bars(model: &mut Model, degrees: f64, through: [f64; 3]) -> [SolidKey; 2] {
    let onto_x = Isometry::quarter_turns(&Axis::y(), 1).expect("a turn about y");
    let first_at = Isometry::translation(rcad_geom::Vector::new(
        through[0] - 2.0,
        through[1],
        through[2],
    ))
    .expect("a finite move")
    .compose(&onto_x);
    let first = model.add_round_bar(0.25, 4.0, &first_at).expect("a bar");
    let turned = Isometry::rotation(&Axis::z(), f64::to_radians(degrees))
        .expect("a turn about z")
        .compose(&onto_x);
    let along = turned.apply_to_vector(rcad_geom::Vector::new(0.0, 0.0, 1.0));
    let (start, length) = if through[1] == 0.0 {
        (
            [
                through[0] - 16.0 * along.x(),
                through[1] - 16.0 * along.y(),
                through[2] - 16.0 * along.z(),
            ],
            32.0,
        )
    } else {
        (through, 2.0)
    };
    let second_at = Isometry::translation(rcad_geom::Vector::new(start[0], start[1], start[2]))
        .expect("a finite move")
        .compose(&turned);
    let second = model
        .add_round_bar(0.25, length, &second_at)
        .expect("a bar");
    [first, second]
}
