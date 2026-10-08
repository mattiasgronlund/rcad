//! A frustum's wall cut below a block's floor, by the plane of the block's
//! wall. `docs/decisions.md` §294.8 is where it was first met.
//!
//! The frustum is `cones::a_frustum(model, 0.5, 2, 6)`: the cone `r = z/2`
//! from `z = 2` to `z = 6`, its top rim three across. A block over the rim,
//! `x` from 2 to 5 and `z` from 5 to 8, cuts the cone's face along the plane of
//! its floor and along the plane of its wall at `x = 2`, and the fuse cuts a
//! face along the whole of each crossing it has with the face, not only the
//! part the other face covers. Below the floor the wall's plane meets the cone
//! in a hyperbola whose lowest point is `(2, 0, 4)`, so one piece of the
//! cone's face lies between that hyperbola and the circle at `z = 5`: two
//! corners, `(2, ±1.5, 5)`, the piece on the `x > 2`, `z < 5` side of both,
//! and nowhere more than one unit deep.
//!
//! **No point the fuse offered was inside that piece**, because every step it
//! took across a curved piece was one radius long, here `2.5`, and each such
//! step from `(2, 0, 5)` lands on the circle or outside the piece. With one
//! block, a side the piece shares with the block's face places it (§365.3,
//! Q712), so the frustum less that block answered. Placed against the block
//! and a second solid together — a cube far away, a block over the far rim,
//! or the rest of the top beside the block — the piece has no side to share
//! with the second solid, and its corners lie on the block's edge, where no
//! corner places it: each such fuse below was refused
//! `"a point inside a piece of a face"` under every rule (Q929). Half the step towards the side
//! of both planes the piece is on is inside it, and the fuse offers the same
//! steps at a half, a quarter, an eighth and a sixteenth of the length after
//! every other point.
//!
//! **Where the seam is does not matter.** Q718 first met the piece in
//! `a_broken_two_surface_corner_read_back_from_bytes.rs`, as the block over
//! the rim *by the seam*, against a block over the rim away from it at
//! `x = −1` that answered. The seam at `+x`, `−x` or `+y`, and the block over
//! `x = −2` on the far side, all leave the same piece. What made the block at
//! `x = −1` answer is how far below the floor its wall's plane cuts the cone:
//! that hyperbola's lowest point is `(−1, 0, 2)`, on the bottom rim, so its
//! piece reaches the bottom cap and has corners there to blend.
//!
//! Every volume here is a closed form, and each answer is held to it within
//! what `measure` says its mesh is good for. The frustum is
//! `π/12 · (6³ − 2³) = 52π/3`, and above `z = 5` it is `91π/12`. A block over
//! the rim past `x = d` takes `∫₅⁶ seg(z/2) dz = 2 (F(3) − F(2.5))`, where
//! `seg(r) = r² acos(d/r) − d √(r² − d²)` is the part of a circle of radius
//! `r` past `x = d` and `F(r) = r³/3 · acos(d/r) − 2dr/3 · √(r² − d²) + d³/3 ·
//! ln(r + √(r² − d²))` is what it sums to: `F′` is `seg` term for term, and a
//! Simpson sum over `z` in a throwaway script gave `1.983696871277089` against
//! `1.983696871277111` for `d = 2`, and `6.535991205209577` against
//! `6.535991205209529` for `d = 1`.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;

/// A block, as the corners of its footprint and the heights of its floor and
/// its top.
#[derive(Clone, Copy, Debug)]
struct Block {
    x: [f64; 2],
    y: [f64; 2],
    z: [f64; 2],
}

/// Over the frustum's rim past `x = 2`, from its floor at `z = 5` up past the
/// top cap.
const OVER_THE_RIM: Block = Block {
    x: [2.0, 5.0],
    y: [-5.0, 5.0],
    z: [5.0, 8.0],
};

/// The same block on the far side of the axis, past `x = −2`.
const OVER_THE_FAR_RIM: Block = Block {
    x: [-5.0, -2.0],
    y: [-5.0, 5.0],
    z: [5.0, 8.0],
};

/// Over the far rim past `x = −1`, whose wall's plane cuts the cone down to
/// the bottom rim.
const OVER_THE_FAR_RIM_DOWN_TO_THE_BOTTOM: Block = Block {
    x: [-5.0, -1.0],
    y: [-5.0, 5.0],
    z: [5.0, 8.0],
};

/// The rest of the top: with [`OVER_THE_RIM`], a block over the whole of the
/// frustum above `z = 5`.
const THE_REST_OF_THE_TOP: Block = Block {
    x: [-5.0, 2.0],
    y: [-5.0, 5.0],
    z: [5.0, 8.0],
};

/// A cube of side two, nowhere near the frustum.
const FAR_AWAY: Block = Block {
    x: [20.0, 22.0],
    y: [20.0, 22.0],
    z: [20.0, 22.0],
};

/// The seam up `+x`, `−x` and `+y`.
const EVERY_SEAM: [[f64; 2]; 3] = [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0]];

/// The frustum less one block over its rim, with the seam each of three ways
/// and the block on either side: what Q718 measured, which a side the piece
/// shares with the block placed before this file was written, and a point
/// inside it places since. Each is held to its closed form at a fine mesh.
#[test]
fn a_frustum_less_one_block_over_its_rim_holds_what_the_two_say_under_certified() {
    less_one_block_over_its_rim(&Policy::certified());
}

#[test]
fn a_frustum_less_one_block_over_its_rim_holds_what_the_two_say_under_tolerant() {
    less_one_block_over_its_rim(&Policy::tolerant(100.0));
}

fn less_one_block_over_its_rim(policy: &Policy) {
    let fine = Deflection::of(1e-4).expect("a fine mesh");
    for seam in EVERY_SEAM {
        holds(
            seam,
            &[],
            &[OVER_THE_RIM],
            Rule::Difference,
            policy,
            fine,
            &[the_frustum() - past(2.0)],
        );
    }
    let seam = EVERY_SEAM[0];
    for (block, volume) in [
        (OVER_THE_FAR_RIM, the_frustum() - past(2.0)),
        (
            OVER_THE_FAR_RIM_DOWN_TO_THE_BOTTOM,
            the_frustum() - past(1.0),
        ),
    ] {
        holds(
            seam,
            &[],
            &[block],
            Rule::Difference,
            policy,
            fine,
            &[volume],
        );
    }
}

/// **A second tool far away was enough to refuse every rule.** The piece
/// shares no side with the far cube, and its corners are on the edge of the
/// block over the rim, so neither placed it against the two tools together.
#[test]
fn a_frustum_and_a_block_over_its_rim_with_a_second_block_far_away_answer_every_rule() {
    let block = 3.0 * 10.0 * 3.0;
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (rule, volumes) in [
            (Rule::Union, vec![8.0, the_frustum() + block - past(2.0)]),
            (Rule::Intersection, vec![past(2.0)]),
            (Rule::Difference, vec![the_frustum() - past(2.0)]),
        ] {
            holds(
                EVERY_SEAM[0],
                &[],
                &[OVER_THE_RIM, FAR_AWAY],
                rule,
                &policy,
                coarse(),
                &volumes,
            );
        }
    }
}

/// A block over each rim takes two flats off the top of the frustum, and the
/// same piece is cut on each side.
#[test]
fn a_frustum_with_a_block_over_each_rim_answers_every_rule() {
    let block = 3.0 * 10.0 * 3.0;
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (rule, volumes) in [
            (
                Rule::Union,
                vec![the_frustum() + 2.0 * block - 2.0 * past(2.0)],
            ),
            (Rule::Intersection, vec![past(2.0), past(2.0)]),
            (Rule::Difference, vec![the_frustum() - 2.0 * past(2.0)]),
        ] {
            holds(
                EVERY_SEAM[0],
                &[],
                &[OVER_THE_RIM, OVER_THE_FAR_RIM],
                rule,
                &policy,
                coarse(),
                &volumes,
            );
        }
    }
}

/// The top of the frustum taken off by two blocks that meet along `x = 2`,
/// each order, and the same two blocks with one of them on the frustum's own
/// side of the fuse. Each piece is placed against a group of two.
#[test]
fn a_frustum_and_a_block_over_its_top_in_two_halves_answer_every_rule() {
    let top = 3.0 * 10.0 * 10.0;
    let above = 91.0 * PI / 12.0;
    let union = the_frustum() + top - above;
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for tools in [
            [OVER_THE_RIM, THE_REST_OF_THE_TOP],
            [THE_REST_OF_THE_TOP, OVER_THE_RIM],
        ] {
            for (rule, volume) in [
                (Rule::Union, union),
                (Rule::Intersection, above),
                (Rule::Difference, the_frustum() - above),
            ] {
                let seam = EVERY_SEAM[0];
                holds(seam, &[], &tools, rule, &policy, coarse(), &[volume]);
            }
        }
        for (argument, tool, intersection, difference) in [
            (
                THE_REST_OF_THE_TOP,
                OVER_THE_RIM,
                past(2.0),
                the_frustum() + 210.0 - above,
            ),
            (
                OVER_THE_RIM,
                THE_REST_OF_THE_TOP,
                above - past(2.0),
                the_frustum() + 90.0 - above,
            ),
        ] {
            for (rule, volume) in [
                (Rule::Union, union),
                (Rule::Intersection, intersection),
                (Rule::Difference, difference),
            ] {
                let seam = EVERY_SEAM[0];
                holds(
                    seam,
                    &[argument],
                    &[tool],
                    rule,
                    &policy,
                    coarse(),
                    &[volume],
                );
            }
        }
    }
}

/// `π/12 · (6³ − 2³)`.
fn the_frustum() -> f64 {
    52.0 * PI / 3.0
}

/// What a block over the rim past `x = d` takes from the frustum, `2 (F(3) −
/// F(2.5))` as the file's header has it.
fn past(d: f64) -> f64 {
    let f = |r: f64| {
        let leg = (r * r - d * d).sqrt();
        r.powi(3) / 3.0 * (d / r).acos() - 2.0 * d * r / 3.0 * leg
            + d.powi(3) / 3.0 * (r + leg).ln()
    };
    2.0 * (f(3.0) - f(2.5))
}

fn coarse() -> Deflection {
    Deflection::of(1e-3).expect("a fine enough mesh")
}

/// The frustum with its seam towards `seam` and `arguments` beside it, fused
/// with `tools` under `rule`, answers, validates, and has one solid for each
/// of `volumes`, which measure them in order of size.
fn holds(
    seam: [f64; 2],
    arguments: &[Block],
    tools: &[Block],
    rule: Rule,
    policy: &Policy,
    deflection: Deflection,
    volumes: &[f64],
) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let frustum = cones::a_frustum_seamed_towards(
        &mut model,
        0.5,
        2.0,
        6.0,
        seam,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a frustum");
    let mut every_argument = vec![frustum];
    every_argument.extend(arguments.iter().map(|&block| a_block(&mut model, block)));
    let every_tool: Vec<SolidKey> = tools
        .iter()
        .map(|&block| a_block(&mut model, block))
        .collect();
    let what = format!("the frustum seamed {seam:?} with {arguments:?}, {rule:?} {tools:?}");
    let answer = model
        .fuse(
            &every_argument,
            &every_tool,
            rule,
            policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} refused: {why:?}"));
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} does not validate: {why:?}"));
    let mut measured: Vec<(f64, f64)> = answer
        .solids()
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("the fuse made this key")
                .measure(deflection, policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} does not measure: {why:?}"));
            (measured.volume(), measured.volume_error())
        })
        .collect();
    measured.sort_by(|one, other| one.0.total_cmp(&other.0));
    assert_eq!(
        measured.len(),
        volumes.len(),
        "{what} gave {measured:?} where {volumes:?} was asked for"
    );
    for (&(volume, error), &asked) in measured.iter().zip(volumes) {
        assert!(
            (volume - asked).abs() <= error,
            "{what} holds {asked} and measures {volume} give or take {error}"
        );
    }
}

fn a_block(model: &mut Model, block: Block) -> SolidKey {
    let [wide, deep] = [block.x[1] - block.x[0], block.y[1] - block.y[0]];
    let footprint = ProfileBuilder::new(vec![[0.0, 0.0], [wide, 0.0], [wide, deep], [0.0, deep]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let at = Isometry::translation(Vector::new(block.x[0], block.y[0], block.z[0]))
        .expect("a plain translation");
    model
        .add_extrusion(&footprint, block.z[1] - block.z[0], &at)
        .expect("a block")
}
