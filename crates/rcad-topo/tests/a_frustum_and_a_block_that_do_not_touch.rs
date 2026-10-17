//! A frustum and a block standing clear of each other, fused.
//!
//! The two solids do not reach each other at all, so a union of them is the two
//! of them side by side and there is nothing to work out, and that is what each
//! fuse below now answers. It used to refuse, and §214 is the whole story of
//! why: the fuse put the block's cap plane to the cone face whether or not the
//! solids met, so the answer carried the cone face cut in two with a circle
//! between them. Walking that circle handed `rcad_geom::walk` **the same cone
//! twice**, and `what_a_run_between_these_two_lies_on` had no arm for a pair
//! that is one surface: it fell through to *a cone crossing another curved
//! surface* and named a second curved surface a frustum and a block have not
//! got between them.
//!
//! §400's band proof (`ABandRoundAnAxis`, since Q896) is what stops the cut
//! from being tried at all: a cone or a pipe whose every side is a circle
//! square to its axis lies in a band, and the block's cap plane misses that
//! band here, so `fuse/cuts.rs`'s `two_faces_provably_apart` proves the cone
//! face and the block's faces apart without a box test on the round face. No
//! cut is put to the cone, no circle appears, and `walk` is never asked about
//! it.
//!
//! The draw here is the smallest of the three §214.5 measured, and that section
//! records that all three refused on `Union` both ways and on `Difference` with
//! the cone first, back before §400 — so those are the three fuses asked below.

use rcad_geom::{Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_frustum;

/// The slope, the two ends and the block of §214.5's smallest draw.
///
/// A power-of-two slope, so every corner of the cone sits exactly on it — the
/// note at the top of `tests/cones/mod.rs` says why that matters.
const SLOPE: f64 = 0.125;
const NEAR: f64 = 1.0;
const FAR: f64 = 8.0;

/// The frustum of `SLOPE`, `NEAR` and `FAR` and a block that stands clear of it.
///
/// The block's nearest corner is `(1, 1)` from the axis, `√2` out, and the widest
/// the cone ever gets is `SLOPE * FAR`, which is `1`. So the two never touch, and
/// the assertion below says so through the kernel rather than through this
/// paragraph.
fn a_frustum_and_a_block_clear_of_it() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_frustum(&mut model, SLOPE, NEAR, FAR);
    let side = SLOPE * FAR;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    let block = model
        .add_extrusion(
            &rectangle,
            FAR,
            &Isometry::translation(Vector::new(1.0, 1.0, NEAR + 1.0)).expect("a finite shift"),
        )
        .expect("a block is a solid");
    (model, cone, block)
}

/// Every one of the three fuses §214.5 says this draw refuses answers, hands
/// back a sound model, and says the right number of solids.
///
/// The pieces are the whole of what an answer about two solids that miss each
/// other can be: a union of them is the two of them, and taking either from the
/// other leaves the one it was taken from. Asserting only that it did not refuse
/// would pass on a fuse that answered with nothing.
#[test]
fn a_union_of_two_solids_that_miss_each_other_is_the_two_of_them() {
    let policy = Policy::certified();
    for (rule, cone_first, pieces_wanted) in [
        (Rule::Union, true, 2),
        (Rule::Union, false, 2),
        (Rule::Difference, true, 1),
    ] {
        let (mut model, cone, block) = a_frustum_and_a_block_clear_of_it();
        let (arguments, tools) = if cone_first {
            ([cone], [block])
        } else {
            ([block], [cone])
        };
        let mut log = ProvenanceLog::new();
        let answer = model
            .fuse(&arguments, &tools, rule, &policy, &mut log)
            .unwrap_or_else(|why| {
                panic!(
                    "these two solids do not touch, so {rule:?} with the cone \
                     {} has an answer: {why}",
                    if cone_first { "first" } else { "second" }
                )
            });
        assert_eq!(
            answer.solids().len(),
            pieces_wanted,
            "{rule:?} of a frustum and a block that never touch, asked with the cone \
             {}, came back in {} pieces rather than {pieces_wanted}",
            if cone_first { "first" } else { "second" },
            answer.solids().len()
        );
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .expect("a fuse hands back a sound model or refuses");
    }
}

/// The two solids really are clear of each other, said by the kernel.
///
/// Without this the test above could pass over a pair that overlaps, and the
/// draw it is pinned to would mean something else.
#[test]
fn the_block_of_that_draw_holds_no_point_of_the_frustum() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let (model, cone, _) = a_frustum_and_a_block_clear_of_it();
    let frustum = model.solid(cone).expect("the frustum is in the model");
    // The block's four upright edges, at every height the block reaches over
    // the cone. The nearest of them is the one at `(1, 1)`.
    for (x, y) in [(1.0, 1.0), (2.0, 1.0), (1.0, 2.0), (2.0, 2.0)] {
        for z in [NEAR + 1.0, 4.0, FAR] {
            assert_eq!(
                frustum.classify_point(Point::new(x, y, z), &policy, &mut log),
                rcad_topo::Containment::Outside,
                "the block's upright edge at ({x}, {y}) has to be clear of the cone \
                 at {z} for this draw to be the disjoint one §214.5 names"
            );
        }
    }
}
