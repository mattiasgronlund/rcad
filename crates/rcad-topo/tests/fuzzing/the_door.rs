//! What `Deserialize for Model` promised about the numbers it lets through.
//!
//! `docs/decisions.md` §44.5. Three targets ask this question — the writer's,
//! the fuse's, and anything later that decodes a stranger's `Model` — and they
//! all reach across to this one file rather than carrying a copy. Two of them
//! carried a copy each until this file existed, which is two places to update
//! when the door grows a fourth promise and one of them to forget.
//!
//! It lives in `rcad-topo` because `rcad-topo` owns the door
//! (`crates/rcad-topo/src/serialize.rs`).

// Shared between several test binaries and several fuzz targets.
#![allow(dead_code)]

use rcad_topo::Model;

/// Every corner is at a point, and every amount of slop is a length.
///
/// The second one is the one that costs something if it is wrong:
/// `not_provably_off` keeps any magnitude above zero and widens the corner it
/// is asked about into an interval that straddles every surface, so an infinite
/// tag is the certified check switched off rather than loosened. A model
/// carrying one *passes* `Model::validate` and reaches whatever comes next with
/// its corners nowhere near its faces, which is why this is asked before
/// `validate` runs and not after — `validate` is the thing that would be
/// fooled.
///
/// # Panics
///
/// If either promise is broken.
pub fn every_number_the_door_promised_is_real(model: &Model) {
    for vertex in model.vertices() {
        assert!(
            vertex.point().is_finite(),
            "a corner that is at no point came through the door: {:?}",
            vertex.point()
        );
        assert_it_is_a_length(vertex.approximation().map(|tag| tag.magnitude()));
    }
    for edge in model.edges() {
        assert_it_is_a_length(edge.approximation().map(|tag| tag.magnitude()));
    }
    for face in model.faces() {
        assert_it_is_a_length(face.approximation().map(|tag| tag.magnitude()));
    }
}

fn assert_it_is_a_length(magnitude: Option<f64>) {
    if let Some(magnitude) = magnitude {
        assert!(
            magnitude.is_finite() && magnitude >= 0.0,
            "a slop of {magnitude} came through the door, and it is not a length"
        );
    }
}
