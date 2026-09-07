//! A region two solids both have a face over, united either way round.
//!
//! §48, lane W1. A block two wide and a frame with a one-wide hole, flush
//! against each other on three planes. Every corner of both is a small whole
//! number, every wall is axis-aligned, and every meet of three of their planes
//! is a Cramer solve with a determinant of ±1 — so nothing here is a story
//! about arithmetic, and the pair used to unite to a solid one way round and to
//! an edge with one face along it the other.
//!
//! What made the difference is which of the two flush faces the rule kept. The
//! frame's own hole wall cuts the block's wall at `x = 1`; nothing of the block
//! sits on `x = 1`, so before this the frame's wall was left whole; and where
//! the answer took the region below `z = 1` from the frame's face and the
//! region above it from the block's, the two no longer met along one edge.
//! `fuse::cuts` cuts every face by every plane of both solids now, so both
//! sides of that line are diced the same and it no longer matters which copy
//! survives.
//!
//! The answer is checked against arithmetic done by hand rather than against
//! the other order, because agreeing with each other is what the two orders did
//! wrong here: one of them was right and the other was not.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
#[path = "pairs/mod.rs"]
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

/// The block, the frame, and the solid they make between them.
///
/// The block is `x ∈ [0, 2]`, `y ∈ [0, 1]`, `z ∈ [0, 2]`, so it holds 4. The
/// frame is a three by three plate one thick at `y ∈ [-2, 1]` with a one by one
/// hole at `x ∈ [1, 2]`, `y ∈ [-1, 0]`, so it holds `9 - 1 = 8`. The hole stops
/// at `y = 0` and the block starts there, so what they share is the plain box
/// `x ∈ [0, 2]`, `y ∈ [0, 1]`, `z ∈ [0, 1]`, which holds 2. The union holds
/// `8 + 4 - 2 = 10`.
///
/// Its skin, face by face: 8 under the plate, 6 over what is left of its top, 2
/// over the block, 3 down the `y = 0` wall — the hole's one square plus the
/// block's two above the plate — 1 at `x = 2`, 5 at `y = 1`, 4 at `x = 0`, 3 at
/// `y = -2`, 3 at `x = 3`, and three more square hole walls, which is 38.
const UNITED_VOLUME: f64 = 10.0;
const UNITED_SURFACE_AREA: f64 = 38.0;

#[test]
fn a_block_flush_against_a_frame_unites_to_the_same_solid_either_way_round() {
    let (model, keys) = a_block_and_a_frame_flush_against_it();

    for (arguments, tools) in [(keys[0], keys[1]), (keys[1], keys[0])] {
        let mut model = model.clone();
        let fused = model
            .fuse(
                &[arguments],
                &[tools],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a union of two solids on the lattice is decided");
        let solids = fused.solids().to_vec();

        assert_eq!(
            solids.len(),
            1,
            "the two overlap, so their union is one lump"
        );
        let solid = model.solid(solids[0]);
        assert_eq!(solid.shells().count(), 1, "nothing is hollowed out");
        let measure = solid
            .measure(
                Deflection::of(0.01).expect("a deflection"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a solid this size measures");
        assert_eq!(measure.volume(), UNITED_VOLUME);
        assert_eq!(measure.surface_area(), UNITED_SURFACE_AREA);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the answer is a model");
    }
}

fn a_block_and_a_frame_flush_against_it() -> (Model, Vec<SolidKey>) {
    let block = Placed {
        shape: Shape::Block { width: 2, depth: 1 },
        skew: Skew::none(),
        height: 2,
        at: [0, 0, 0],
    };
    let frame = Placed {
        shape: Shape::Frame { width: 3, depth: 3 },
        skew: Skew::none(),
        height: 1,
        at: [0, -2, 0],
    };
    Configuration {
        solids: vec![block, frame],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
    .build()
}
