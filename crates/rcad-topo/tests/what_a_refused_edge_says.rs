//! The reason an edge is refused for having no length, in the words a caller
//! reads.
//!
//! [`Error::ADraftEdgeHasNoLength`] used to say the edge "is straight and has no
//! length", and that stopped being true when the rule stopped being about
//! straightness. What `SolidBuilder::add_edge` asks is whether the two ends are
//! the whole of what says where the edge runs: [`EdgeRun::Straight`] says so and
//! is refused, the two runs that name a curve do not and are kept, and a kind of
//! run this kernel has never seen is not taken for one of those.
//!
//! So a caller handed a straight edge back gets told about straightness by a
//! message that would say the same thing about a run nobody has written yet.
//! Reading the arithmetic out of the message and putting the rule in is the
//! whole of this test: it says what was asked, not one case of it.

use rcad_geom::{EdgeRun, Point};
use rcad_topo::{Error, SolidBuilder};

#[test]
fn an_edge_refused_for_having_no_length_says_what_was_actually_asked() {
    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::origin())
        .expect("the origin is finite");

    let refused = builder
        .add_edge(corner, corner, EdgeRun::Straight)
        .expect_err("one corner twice, run straight between them, is no edge");

    assert_eq!(
        refused.to_string(),
        "the edge described at position 0 has both ends in one place, and its \
         run does not say it comes back there",
        "the refusal is about the ends being the whole of what says where the \
         edge runs, and not about the one run that is straight"
    );
    assert!(
        matches!(refused, Error::ADraftEdgeHasNoLength { edge: 0 }),
        "and it names which edge of the description: got {refused:?}"
    );
}
