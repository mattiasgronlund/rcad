//! A round post cut on a slant, with a block's edge lying on the slant
//! (Q2805, `docs/decisions.md` §422).
//!
//! The slant, the block's bottom and the block's side all hold the edge, so
//! on the post's pipe the slant meets the pipe where the edge does twice over:
//! once with the bottom and once with the side. Those two meets are one point,
//! worked out two ways into one box, and every fuse was refused on their order
//! until the three planes were read as holding one line.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

#[test]
fn a_round_post_cut_on_a_slant_under_a_blocks_edge_answers_every_rule_either_way_round() {
    let failed = every_rule_on_two_apart(
        |model| a_box(model, [[-3.0, 3.0], [-3.0, 0.0], [2.0, 3.0]]),
        6.0 * 3.0,
        |model| {
            let upright = model
                .add_round_bar(1.0, 5.0, &Isometry::identity())
                .expect("a bar");
            a_post_cut_on_a_slant(model, upright)
        },
        PI * 2.0,
    );
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_square_post_cut_on_a_slant_under_a_blocks_edge_answers_every_rule_either_way_round() {
    let failed = every_rule_on_two_apart(
        |model| a_box(model, [[-3.0, 3.0], [-3.0, 0.0], [2.0, 3.0]]),
        6.0 * 3.0,
        |model| {
            let upright = a_box(model, [[-1.0, 1.0], [-1.0, 1.0], [0.0, 5.0]]);
            a_post_cut_on_a_slant(model, upright)
        },
        4.0 * 2.0,
    );
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The post standing from `z = 0`, cut on top by `z = 2 + y`, so its volume
/// is its cross-section times 2. The block over `x ∈ [-3, 3]`, `y ∈ [-3, 0]`,
/// `z ∈ [2, 3]` has its bottom edge `y = 0`, `z = 2` on that cut.
fn a_post_cut_on_a_slant(model: &mut Model, upright: SolidKey) -> SolidKey {
    let under_the_slant =
        a_block_across_x(model, &[[-2.0, -1.0], [2.0, -1.0], [2.0, 4.0], [-2.0, 0.0]]);
    one_solid(model, &[upright], &[under_the_slant], Rule::Intersection)
}

/// The same with a bar of radius `1.5` along `z` through `y = 0.5`, cut on
/// top by `z = 0.5 + y`, and a slab over `x ∈ [-3, 3]`, `y ∈ [-0.5, 1.5]`,
/// `z ∈ [2, 3]` whose edge `y = 1.5`, `z = 2` lies on that cut and meets the
/// bar's pipe at `x = ±√1.25` (Q2722). Q2722's slab started at `y = −0.3`,
/// whose volume `6 × 1.8` no `f64` holds.
#[test]
fn a_round_bar_cut_on_a_slant_under_a_slabs_edge_answers_every_rule_either_way_round() {
    let failed = every_rule_on_two_apart(
        |model| a_box(model, [[-3.0, 3.0], [-0.5, 1.5], [2.0, 3.0]]),
        12.0,
        a_bar_cut_by_z_half_plus_y,
        PI * 1.5 * 1.5 * 4.0,
    );
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The same with the part of the bar's disc with `x ≤ −1.3` cut through the
/// slab (Q2722).
#[test]
fn a_round_bar_cut_on_a_slant_under_a_holed_slabs_edge_answers_every_rule_either_way_round() {
    let r: f64 = 1.5;
    let the_hole_across = r * r * (1.3 / r).acos() - 1.3 * (r * r - 1.3 * 1.3).sqrt();
    let failed = every_rule_on_two_apart(
        |model| {
            let the_slab = a_box(model, [[-3.0, 3.0], [-0.5, 1.5], [2.0, 3.0]]);
            let placed =
                Isometry::translation(Vector::new(0.0, 0.5, -2.0)).expect("a finite shift");
            let bar = model.add_round_bar(1.5, 6.0, &placed).expect("a bar");
            let the_box = a_box(model, [[-2.0, -1.3], [-1.5, 2.5], [-3.0, 5.0]]);
            let the_hole = one_solid(model, &[bar], &[the_box], Rule::Intersection);
            one_solid(model, &[the_slab], &[the_hole], Rule::Difference)
        },
        12.0 - the_hole_across,
        a_bar_cut_by_z_half_plus_y,
        PI * 1.5 * 1.5 * 4.0,
    );
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A bar of radius `1.5` along `z` through `y = 0.5`, from `z = −3` up to
/// `z = 0.5 + y`: its volume is its cross-section times `4`.
fn a_bar_cut_by_z_half_plus_y(model: &mut Model) -> SolidKey {
    let placed = Isometry::translation(Vector::new(0.0, 0.5, -3.0)).expect("a finite shift");
    let bar = model.add_round_bar(1.5, 7.0, &placed).expect("a bar");
    let under_the_slant = a_block_across_x(
        model,
        &[[-2.0, -4.0], [3.0, -4.0], [3.0, 3.5], [-2.0, -1.5]],
    );
    one_solid(model, &[bar], &[under_the_slant], Rule::Intersection)
}

/// What goes wrong fusing what `over` makes, of volume `over_volume`, with
/// what `under` makes, of volume `under_volume`, every rule both ways round,
/// as the fuses' names. The two share no volume.
fn every_rule_on_two_apart(
    over: impl Fn(&mut Model) -> SolidKey,
    over_volume: f64,
    under: impl Fn(&mut Model) -> SolidKey,
    under_volume: f64,
) -> Vec<String> {
    let mut failed = Vec::new();
    for rule in EVERY_RULE {
        for under_first in [true, false] {
            let by_hand = match (rule, under_first) {
                (Rule::Union, _) => under_volume + over_volume,
                (Rule::Intersection, _) => 0.0,
                (Rule::Difference, true) => under_volume,
                (Rule::Difference, false) => over_volume,
                _ => unreachable!("EVERY_RULE names three rules"),
            };
            let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
            let the_one_over = over(&mut model);
            let the_one_under = under(&mut model);
            let (arguments, tools) = if under_first {
                (vec![the_one_under], vec![the_one_over])
            } else {
                (vec![the_one_over], vec![the_one_under])
            };
            let asked = format!("{rule:?} under_first={under_first}");
            if let Some(wrong) = check(&mut model, &arguments, &tools, rule, by_hand) {
                println!("{asked}: {wrong}");
                failed.push(asked);
            }
        }
    }
    failed
}

/// The box over `x`, `y` and `z`, each a low and a high end.
fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let rectangle =
        ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            z[1] - z[0],
            &Isometry::translation(Vector::new(0.0, 0.0, z[0])).expect("a finite shift"),
        )
        .expect("a positive height")
}

/// `profile`, drawn in `y` and `z`, pushed across `x ∈ [-3, 3]`.
fn a_block_across_x(model: &mut Model, profile: &[[f64; 2]]) -> SolidKey {
    let profile = ProfileBuilder::new(profile.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a profile");
    let about_x = Isometry::quarter_turns(&Axis::x(), 1).expect("a quarter turn");
    let about_z = Isometry::quarter_turns(&Axis::z(), 1).expect("a quarter turn");
    let back = Isometry::translation(Vector::new(-3.0, 0.0, 0.0)).expect("a finite shift");
    model
        .add_extrusion(&profile, 6.0, &back.compose(&about_z.compose(&about_x)))
        .expect("a positive height")
}

/// The one solid fusing `arguments` and `tools` by `rule` gives.
fn one_solid(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> SolidKey {
    let fused = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a fuse that makes the shape");
    let [one] = fused.solids() else {
        panic!("{} solids where one was meant", fused.solids().len());
    };
    *one
}

/// What is wrong with fusing `arguments` and `tools` by `rule`: a refusal, a
/// result that does not validate, or a volume further from `by_hand` than the
/// measure's own bound. `None` when nothing is.
fn check(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
    by_hand: f64,
) -> Option<String> {
    let policy = Policy::certified();
    let fused = match model.fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(fused) => fused.solids().to_vec(),
        Err(refused) => return Some(format!("refused {refused:?}")),
    };
    if let Err(broken) = model.validate(&policy, &mut ProvenanceLog::new()) {
        return Some(format!("does not validate {broken:?}"));
    }
    let (mut volume, mut bound) = (0.0, 0.0);
    for solid in fused {
        let measured = match model.solid(solid).expect("a key").measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        ) {
            Ok(measured) => measured,
            Err(refused) => return Some(format!("measure refused {refused:?}")),
        };
        volume += measured.volume();
        bound += measured.volume_error();
    }
    ((volume - by_hand).abs() > bound).then(|| format!("{volume} ± {bound} against {by_hand}"))
}
