//! A round post cut on a slant, with a block's edge lying on the slant
//! (Q2805, `docs/decisions.md` §422).

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

#[test]
fn a_round_post_cut_on_a_slant_under_a_blocks_edge_answers_every_rule_either_way_round() {
    let failed = every_rule_against_the_block(Post::Round);
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_square_post_cut_on_a_slant_under_a_blocks_edge_answers_every_rule_either_way_round() {
    let failed = every_rule_against_the_block(Post::Square);
    assert!(failed.is_empty(), "{failed:#?}");
}

#[derive(Clone, Copy, Debug)]
enum Post {
    Round,
    Square,
}

/// What goes wrong fusing the post with a block over `x ∈ [-3, 3]`,
/// `y ∈ [-3, 0]`, `z ∈ [2, 3]`, every rule both ways round, as the fuses'
/// names. The post stands from `z = 0` and is cut on top by `z = 2 + y`, so
/// its volume is its cross-section times 2, and the block's bottom edge
/// `y = 0`, `z = 2` lies on that cut: the two share no volume.
fn every_rule_against_the_block(post: Post) -> Vec<String> {
    let post_volume = match post {
        Post::Round => PI * 2.0,
        Post::Square => 4.0 * 2.0,
    };
    let block = 6.0 * 3.0;
    let mut failed = Vec::new();
    for rule in EVERY_RULE {
        for post_first in [true, false] {
            let by_hand = match (rule, post_first) {
                (Rule::Union, _) => post_volume + block,
                (Rule::Intersection, _) => 0.0,
                (Rule::Difference, true) => post_volume,
                (Rule::Difference, false) => block,
                _ => unreachable!("EVERY_RULE names three rules"),
            };
            let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
            let the_block = a_box(&mut model, [[-3.0, 3.0], [-3.0, 0.0], [2.0, 3.0]]);
            let upright = match post {
                Post::Round => model
                    .add_round_bar(1.0, 5.0, &Isometry::identity())
                    .expect("a bar"),
                Post::Square => a_box(&mut model, [[-1.0, 1.0], [-1.0, 1.0], [0.0, 5.0]]),
            };
            let under_the_slant = a_block_across_x(
                &mut model,
                &[[-2.0, -1.0], [2.0, -1.0], [2.0, 4.0], [-2.0, 0.0]],
            );
            let the_post = one_solid(
                &mut model,
                &[upright],
                &[under_the_slant],
                Rule::Intersection,
            );
            let (arguments, tools) = if post_first {
                (vec![the_post], vec![the_block])
            } else {
                (vec![the_block], vec![the_post])
            };
            let asked = format!("{post:?} {rule:?} post_first={post_first}");
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
