//! The union of a round bar and a block that miss each other, measured.
//!
//! **The answer is a decomposition difference and not a defect**, and this file
//! is the measurement behind that word. `docs/decisions.md` §47.7 carries the
//! row *"the fuse drops the seam of a curved wall it never touched"*, and §48.4
//! sent a lane to find out whether a fused result carrying such a face is a
//! validity hole. It is not. Five questions, all answered here:
//!
//! 1. **It answers.** A bar of radius two standing a unit clear above a block of
//!    side eight unites under `Policy::certified()` into two solids, with an
//!    empty provenance log — nothing had to be settled.
//! 2. **It validates.** `Model::validate` under a certified policy passes.
//! 3. **It measures right.** The block comes back bit for bit `512` and `384`.
//!    The bar comes back `50.20344246266871` against `50.20344246266874` going
//!    in, and `75.35168938062597` against `75.35168938062601` — the last few
//!    bits, because the wall is now cut into patches a different way, and the
//!    two sums agree to well inside `1e-12`.
//! 4. **The seam really is dropped, and that is the load-bearing number.** The
//!    bar's wall arrives as one wire of four coedges over three edges, with the
//!    seam used twice by that one wire — which is what makes it a seam
//!    (`build.rs:660`). It leaves as **two** wires of one coedge each over two
//!    edges, both of them cap circles that close on themselves. Nothing cuts the
//!    wall open along its longitude any more.
//! 5. **Classification still agrees.** Inside the bar, inside the block, in the
//!    gap between them and well away from both all answer as they did.
//!
//! **So why is a face that goes right round with no seam not the thing
//! `Error::ACurvedFaceGoesRightRoundWithNoSeam` exists to refuse?** Because that
//! refusal is only ever asked of a surface that *pinches*:
//! `Surface::where_it_pinches` (`rcad-geom`) answers `Some` for a sphere and
//! a cone and **`None` for a cylinder**, and `mesh.rs:513-519` guards the
//! refusal on it. The reason is written there: on a pipe a loop that does not go
//! right round bounds a patch one way and an *unbounded* region the other, a
//! solid's faces are bounded, so the material is the patch, and there is no
//! second case for a sign to tell apart. A wall bounded by two whole circles and
//! nothing else is not an ambiguous face — it is a **ring**, which the mesher
//! names and cuts across into four patches (`mesh.rs:535-538`), and which it
//! says in as many words is *"the one the fuse makes"*.
//!
//! **What §47.7's row should say.** Not a gap. A note: a union hands a disjoint
//! curved argument back as a *ring* rather than as the seamed wall it went in
//! as, which validates, measures to the last few bits and classifies the same,
//! and is the difference roadmap §11.2 says not to test for.
//!
//! Nothing here is a pin on wrong behaviour. It is a pin on a *decomposition*,
//! so a fuse that later keeps the seam would fail
//! `the_bars_wall_arrives_with_a_seam_and_leaves_a_ring` — and that failure
//! would be good news to be edited in, not a regression.

use std::collections::HashMap;

use rcad_geom::{Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, EdgeKey, Fused, Model, Rule, SolidKey};

const SIDE: f64 = 8.0;
const RADIUS: f64 = 2.0;
const HEIGHT: f64 = 4.0;

/// §48.4's load-bearing question, counted rather than inferred from a face
/// count.
///
/// The seam is dropped. Both argument orders drop it the same way, and the bar
/// the union hands back is a **ring**: two cap circles, each a closed curve
/// bounding the wall, and nothing running up the side between them.
#[test]
fn the_bars_wall_arrives_with_a_seam_and_leaves_a_ring() {
    let (mut model, block, bar) = a_bar_standing_clear_above_a_block();

    assert_eq!(
        how_the_wall_is_bounded(&model, bar),
        HowTheWallIsBounded {
            wires: 1,
            coedges: 4,
            edges: 3,
            edges_used_twice: 1,
            every_wire_closes_on_one_edge: false,
        },
        "a bar is built with its wall cut open by a seam its one wire uses twice"
    );

    let answer = unite_them(&mut model, block, bar, &mut ProvenanceLog::new());
    let a_ring = HowTheWallIsBounded {
        wires: 2,
        coedges: 2,
        edges: 2,
        edges_used_twice: 0,
        every_wire_closes_on_one_edge: true,
    };
    assert_eq!(
        how_the_wall_is_bounded(&model, the_one_with_a_round_wall(&model, &answer)),
        a_ring,
        "the union hands the bar back with nothing cutting its wall open"
    );

    let (mut other, block, bar) = a_bar_standing_clear_above_a_block();
    let other_way = other
        .fuse(
            &[bar],
            &[block],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the other order answers too");
    assert_eq!(
        how_the_wall_is_bounded(&other, the_one_with_a_round_wall(&other, &other_way)),
        a_ring,
        "which argument the bar is does not change what happens to its seam"
    );
}

/// The union answers, what it leaves is a sound model, and the two solids
/// measure what they measured going in.
///
/// The block is flat, so its measurement is an exact fan and has to come back
/// bit for bit. The bar's is a sum over a mesh, and a ring is cut into patches
/// differently from a seamed wall, so the last few bits move — nowhere near
/// enough to be a different answer.
#[test]
fn a_union_that_drops_the_seam_still_validates_and_measures() {
    let (mut model, block, bar) = a_bar_standing_clear_above_a_block();
    let block_before = measure(&model, block);
    let bar_before = measure(&model, bar);

    let mut log = ProvenanceLog::new();
    let answer = unite_them(&mut model, block, bar, &mut log);
    assert_eq!(answer.solids().len(), 2, "two solids that miss stay two");
    assert!(
        log.records().is_empty(),
        "nothing about two solids a whole unit apart had to be settled"
    );

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("what the union left is a sound model");

    let round = the_one_with_a_round_wall(&model, &answer);
    let flat = the_other_solid(&answer, round);

    let flat_after = measure(&model, flat);
    assert_eq!(
        (flat_after.volume(), flat_after.surface_area()),
        (block_before.volume(), block_before.surface_area()),
        "the block came through a fuse that never touched it, bit for bit"
    );

    let round_after = measure(&model, round);
    assert!(
        (round_after.volume() - bar_before.volume()).abs() < 1e-12,
        "the ring measures the bar: {} against {}",
        round_after.volume(),
        bar_before.volume()
    );
    assert!(
        (round_after.surface_area() - bar_before.surface_area()).abs() < 1e-12,
        "and the same area: {} against {}",
        round_after.surface_area(),
        bar_before.surface_area()
    );
    assert!(
        (round_after.volume() + flat_after.volume()
            - (bar_before.volume() + block_before.volume()))
        .abs()
            < 1e-12,
        "the union of two solids that miss measures the sum of the two"
    );
    assert!(
        (round_after.surface_area() + flat_after.surface_area()
            - (bar_before.surface_area() + block_before.surface_area()))
        .abs()
            < 1e-12,
        "and so does the skin"
    );
}

/// A point inside the bar, a point inside the block, a point in the gap between
/// them and a point well away from both, asked of what the union left.
#[test]
fn dropping_the_seam_does_not_move_anything_in_or_out() {
    let (mut model, block, bar) = a_bar_standing_clear_above_a_block();
    let answer = unite_them(&mut model, block, bar, &mut ProvenanceLog::new());
    let round = the_one_with_a_round_wall(&model, &answer);
    let flat = the_other_solid(&answer, round);

    let where_it_is = |solid, point| {
        model
            .solid(solid)
            .expect("this model made the key")
            .classify_point(point, &Policy::certified(), &mut ProvenanceLog::new())
    };
    for (what, point, in_the_round_one, in_the_flat_one) in [
        (
            "inside the bar",
            Point::new(4.0, 4.0, 10.0),
            Containment::Inside,
            Containment::Outside,
        ),
        (
            "inside the block",
            Point::new(4.0, 4.0, 4.0),
            Containment::Outside,
            Containment::Inside,
        ),
        (
            "in the gap between them",
            Point::new(4.0, 4.0, 8.5),
            Containment::Outside,
            Containment::Outside,
        ),
        (
            "well away from both",
            Point::new(20.0, 20.0, 20.0),
            Containment::Outside,
            Containment::Outside,
        ),
    ] {
        assert_eq!(where_it_is(round, point), in_the_round_one, "{what}, round");
        assert_eq!(where_it_is(flat, point), in_the_flat_one, "{what}, flat");
    }
}

/// A ring is not a dead end: it goes back into a fuse and comes out a ring.
///
/// This is what separates *"the seam is gone"* from *"the answer is damaged"*.
/// If a seamless wall could not be fused again, dropping the seam would be a
/// one-way door and §47.7's row would be a gap after all. It can, and the wall
/// it hands back is bounded exactly as it went in — so the shape is a fixed
/// point of the fuse and not something that loses a little more each time.
#[test]
fn a_wall_with_no_seam_fuses_again_and_stays_a_ring() {
    let (mut model, block, bar) = a_bar_standing_clear_above_a_block();
    let once = unite_them(&mut model, block, bar, &mut ProvenanceLog::new());
    let seamless = the_one_with_a_round_wall(&model, &once);
    let bounded_once = how_the_wall_is_bounded(&model, seamless);

    let another_block = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a second block, which the bar misses just as the first one did");
    let twice = unite_them(
        &mut model,
        another_block,
        seamless,
        &mut ProvenanceLog::new(),
    );

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a second union over a seamless wall leaves a sound model");
    assert_eq!(
        how_the_wall_is_bounded(&model, the_one_with_a_round_wall(&model, &twice)),
        bounded_once,
        "fusing a ring again gives a ring, not something narrower each time"
    );
}

/// The pair `fuses_that_must_agree.rs:1275` uses: a bar standing a whole unit
/// clear above a block, so what the two share is nothing.
fn a_bar_standing_clear_above_a_block() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let block = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block eight across");
    let bar = model
        .add_round_bar(
            RADIUS,
            HEIGHT,
            &Isometry::translation(Vector::new(4.0, 4.0, 9.0)).expect("a shift"),
        )
        .expect("a bar standing clear above it");
    (model, block, bar)
}

fn unite_them(model: &mut Model, block: SolidKey, bar: SolidKey, log: &mut ProvenanceLog) -> Fused {
    model
        .fuse(&[block], &[bar], Rule::Union, &Policy::certified(), log)
        .expect("two solids that miss each other unite")
}

/// What bounds a solid's one cylindrical face.
#[derive(Debug, PartialEq, Eq)]
struct HowTheWallIsBounded {
    wires: usize,
    coedges: usize,
    edges: usize,
    /// How many of those edges are used twice by the same wire. A seam is
    /// exactly that, and nothing else on a bar is.
    edges_used_twice: usize,
    /// Whether every wire is one closed curve that starts where it ended.
    every_wire_closes_on_one_edge: bool,
}

fn how_the_wall_is_bounded(model: &Model, solid: SolidKey) -> HowTheWallIsBounded {
    let mut walls = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Cylinder(_)));
    let wall = walls.next().expect("this solid has a cylindrical face");
    assert!(walls.next().is_none(), "and only the one");

    let mut uses: HashMap<EdgeKey, usize> = HashMap::new();
    let mut coedges = 0;
    let mut every_wire_closes_on_one_edge = true;
    for wire in wall.wires() {
        every_wire_closes_on_one_edge &= wire.closes_on_one_edge();
        for coedge in wire.coedges() {
            coedges += 1;
            *uses.entry(coedge.edge().key()).or_default() += 1;
        }
    }
    HowTheWallIsBounded {
        wires: wall.wire_count(),
        coedges,
        edges: uses.len(),
        edges_used_twice: uses.values().filter(|&&count| count == 2).count(),
        every_wire_closes_on_one_edge,
    }
}

/// Which of the answer's solids is the one carrying a round wall.
fn the_one_with_a_round_wall(model: &Model, answer: &Fused) -> SolidKey {
    let mut round: Vec<SolidKey> = answer
        .solids()
        .iter()
        .copied()
        .filter(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .any(|face| matches!(face.surface(), Surface::Cylinder(_)))
        })
        .collect();
    assert_eq!(
        round.len(),
        1,
        "exactly one of the answer's solids is round"
    );
    round.pop().expect("just counted one")
}

fn the_other_solid(answer: &Fused, one: SolidKey) -> SolidKey {
    let mut rest = answer
        .solids()
        .iter()
        .copied()
        .filter(|&solid| solid != one);
    let other = rest.next().expect("the answer has a second solid");
    assert!(rest.next().is_none(), "and only the two");
    other
}

fn measure(model: &Model, solid: SolidKey) -> rcad_topo::Measurement {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures")
}

/// A tenth of a millimetre, the same as `measuring.rs` asks for.
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
