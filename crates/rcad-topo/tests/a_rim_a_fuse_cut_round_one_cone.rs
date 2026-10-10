//! A fuse cutting a curved wall in two, and the rim it leaves saying which
//! plane cut it.
//!
//! §216.2 built [`EdgeRun::RightRoundWhereItCrosses`] and closed the *builder's*
//! half of §214.4(d): a rim whose two faces are the one surface names nothing
//! between them, and a run that names the cutting surface instead is what the
//! mesher needs to draw the ellipse. The fuse kept writing
//! [`EdgeRun::RightRoundWhereItsFacesCross`] for every closed run, and this file
//! is the fuse half.
//!
//! # The draw, and why it still reaches this at all
//!
//! `a_frustum_and_a_block_that_do_not_touch.rs`'s draw, which is the smallest
//! one that reaches this at all: a frustum and a block standing clear of each
//! other — the block's nearest corner is `√2` from the axis and the cone never
//! gets wider than `1`, so no face of the one ever shares a point with a face
//! of the other. Since §400 the fuse cuts two solids against each other only
//! where it cannot prove they do not meet
//! (`fuse::cuts::solids_within_reach_of_each_other`, about line 552), and that
//! proof still fails on this draw: the block's lower cap is a flat face with no
//! side on a round one, so `two_faces_provably_apart`'s band check (about line
//! 615) is not asked of it, and the two whole-solid boxes still touch at the
//! block's near corner, which the box check alone cannot see past. So the pair
//! is still joined for cutting, and the answer still carries **the cone face
//! cut in two**, with a circle between the halves and the one cone either side
//! of it — moving the block out to `1.5` on each axis instead of `1`, clear of
//! the box as well as the cone, stops the cut. That is the pair that names
//! nothing, arriving out of a fuse rather than out of a builder.
//!
//! The same draw round a round bar is what says the substitution is not
//! unconditional: the bar's *cap* circles have a cylinder one side and a plane
//! the other, and those two really do say where the circle runs, so they keep
//! the run they had.
//!
//! # A rim like this bounds a wire of one step
//!
//! A fuse's curved wall comes back with **no seam** — each band is a face with
//! two wires of one coedge each, one per closed boundary — so every wire the
//! new run appears in is a one-step wire. That is the shape
//! `Model::wire_closes_on_one_edge` is asked about, and a rim the sweep does
//! not read as closed is `Invalid::WireTooShort { count: 1 }` on a sound
//! answer. The assertions below all run the whole sweep over the fused model
//! for that reason.

use rcad_geom::{EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, EdgeView, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

mod cones;

use cones::a_frustum;

/// The frustum of §214.5's smallest draw: a power-of-two slope, so every corner
/// sits exactly on the cone (`tests/cones/mod.rs` says why).
const SLOPE: f64 = 0.125;
const NEAR: f64 = 1.0;
const FAR: f64 = 8.0;

/// How far up the block starts, which is the height its lower cap plane cuts
/// both walls at.
const CUT: f64 = NEAR + 1.0;

/// The wall of a frustum, cut in two by a plane no face of the answer sits on,
/// says which plane cut it.
#[test]
fn a_frustums_wall_cut_in_two_by_a_tool_plane_names_the_plane_that_cut_it() {
    let (mut model, cone, block) = a_solid_and_a_block_clear_of_it(a_frustum_of_the_draw);
    let runs = what_the_fuse_wrote(&mut model, cone, block);

    assert!(
        runs.contains(&EdgeRun::RightRoundWhereItCrosses {
            and: the_blocks_lower_cap(),
        }),
        "the cone face came back cut in two, so the rim between the halves has the one cone \
         either side of it and names nothing; it has to say which plane cut it instead. The \
         answer's runs were {runs:?}"
    );
}

/// The two cap circles of that same frustum keep the run they had.
///
/// They are the pair that really does name where the circle runs — a cone one
/// side and a plane the other — so the substitution must not reach them.
#[test]
fn the_cap_circles_of_that_frustum_still_say_their_two_faces_cross() {
    let (mut model, cone, block) = a_solid_and_a_block_clear_of_it(a_frustum_of_the_draw);
    let runs = what_the_fuse_wrote(&mut model, cone, block);

    let crossings = runs
        .iter()
        .filter(|run| **run == EdgeRun::RightRoundWhereItsFacesCross)
        .count();
    assert_eq!(
        crossings, 4,
        "a frustum has two cap circles and each is walked twice — once by its cap and once by the \
         band of wall beside it — so the answer carries four runs that name their own two faces. \
         Its runs were {runs:?}"
    );
}

/// A round bar's cap circle is the same shape a crate down and must not move
/// either.
///
/// The bar stands where the frustum stood, with the same block clear of it, and
/// its two cap circles come back saying what they said. **Its wall is not cut at
/// all**, which is worth writing down beside the frustum's: the block's lower
/// cap plane crosses this cylinder as squarely as it crosses that cone, and the
/// answer carries four runs and no more. So a cylinder does not reach the new
/// run through this draw, and only the cone does.
#[test]
fn a_round_bars_cap_circle_still_says_its_two_faces_cross() {
    let (mut model, bar, block) = a_solid_and_a_block_clear_of_it(a_round_bar_of_the_draw);
    let runs = what_the_fuse_wrote(&mut model, bar, block);

    let crossings = runs
        .iter()
        .filter(|run| **run == EdgeRun::RightRoundWhereItsFacesCross)
        .count();
    assert_eq!(
        crossings, 4,
        "the bar's two cap circles have a cylinder one side and a plane the other, which says \
         where they run, so they keep the run they had — twice each, once from the cap and once \
         from the band of wall beside it. Its runs were {runs:?}"
    );
    assert_eq!(
        runs.len(),
        4,
        "nothing else is on the bar at all: no rim, and so nothing for the new run to be written \
         on. Its runs were {runs:?}"
    );
}

/// [`EdgeView::is_closed`] is the one copy of the rule and answers `true` for
/// the rim the fuse just wrote.
///
/// §216.2 records what storing a closed rim as
/// [`EdgeRun::PartOfWhereItCrosses`] would have cost: *not closed* at all six
/// of that rule's askers. This asks it of a rim a fuse built rather than of one
/// a test wrote down.
#[test]
fn the_rim_the_fuse_cut_is_closed() {
    let (mut model, cone, block) = a_solid_and_a_block_clear_of_it(a_frustum_of_the_draw);
    let fused = model
        .fuse(
            &[cone],
            &[block],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two solids that miss each other unite");
    let solid = model
        .solid(fused.solids()[0])
        .expect("this model made the key");

    let mut asked = 0;
    for edge in every_edge_of(solid) {
        if !matches!(edge.run(), EdgeRun::RightRoundWhereItCrosses { .. }) {
            continue;
        }
        asked += 1;
        assert!(
            edge.is_closed().expect("this crate knows this kind of run"),
            "a rim that names the plane that cut it is a whole closed curve"
        );
    }
    assert!(
        asked > 0,
        "no edge of the answer carried the new run at all"
    );
}

/// The rim the fuse wrote can be described again through [`SolidBuilder`].
///
/// A solid the kernel can produce has to be sayable at the kernel's other door,
/// and the rule that decides it is one rule at two doors — §44.11.
/// `Model::wire_closes_on_one_edge` names both closing kinds, and
/// `SolidBuilder::closes_on_one_edge` is the same question asked of a
/// description. A rim like this bounds a wire of **one** coedge, so a builder
/// that reads only the older closing kind refuses the wire outright with
/// [`Error::ADraftWireIsTooShort`](rcad_topo::Error::ADraftWireIsTooShort) — a
/// solid the fuse writes and nobody can write down.
///
/// The run comes out of the fuse rather than being typed here, so this is that
/// rim and not a hand-made one that looks like it.
#[test]
fn the_rim_the_fuse_cut_can_be_described_again_at_the_builders_door() {
    let (mut model, cone, block) = a_solid_and_a_block_clear_of_it(a_frustum_of_the_draw);
    let runs = what_the_fuse_wrote(&mut model, cone, block);
    let rim = runs
        .iter()
        .find(|run| matches!(run, EdgeRun::RightRoundWhereItCrosses { .. }))
        .expect("no edge of the answer carried the new run at all")
        .clone();

    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(0.0, 0.0, CUT))
        .expect("a finite point");
    let edge = builder
        .add_edge(corner, corner, rim)
        .expect("a rim is a whole closed curve, so one corner twice is an edge");
    let coedge = builder
        .add_coedge(edge, Orientation::Forward)
        .expect("an edge of this build");
    let wire = builder.add_wire(&[DraftStep::Along(coedge)]);

    assert!(
        wire.is_ok(),
        "one coedge on a rim that runs right round a curve is a closed wire, and \
         the model's door already says so; the builder's door answered {wire:?}"
    );
}

/// Every run the fuse wrote on the first solid of the answer, once per coedge.
fn what_the_fuse_wrote(model: &mut Model, solid: SolidKey, block: SolidKey) -> Vec<EdgeRun> {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[solid], &[block], Rule::Union, &policy, &mut log)
        .expect("two solids that miss each other unite");
    model
        .validate(&policy, &mut log)
        .expect("a fuse hands back a sound model or refuses");
    assert_eq!(
        fused.solids().len(),
        2,
        "the two never touch, so a union of them is the two of them"
    );
    let answer = model
        .solid(fused.solids()[0])
        .expect("this model made the key");
    every_edge_of(answer).map(EdgeView::run).collect()
}

fn every_edge_of(solid: rcad_topo::SolidView<'_>) -> impl Iterator<Item = EdgeView<'_>> {
    solid
        .faces()
        .flat_map(|face| face.wires().collect::<Vec<_>>())
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .map(|coedge| coedge.edge())
}

fn a_frustum_of_the_draw(model: &mut Model) -> SolidKey {
    a_frustum(model, SLOPE, NEAR, FAR)
}

/// A bar standing over the frustum's span and narrower than it ever gets, so
/// the block clear of the one is clear of the other — and clear by more than a
/// tangent: a bar as wide as the frustum's far end would have the block's
/// `x = 1` wall exactly along its side, which is a plane `side of plane` cannot
/// decide against a cylinder.
fn a_round_bar_of_the_draw(model: &mut Model) -> SolidKey {
    model
        .add_round_bar(
            SLOPE * FAR / 2.0,
            FAR - NEAR,
            &Isometry::translation(Vector::new(0.0, 0.0, NEAR)).expect("a finite shift"),
        )
        .expect("a bar one across and seven tall")
}

/// `made` and a block standing clear of it, the block of §214.5's draw.
///
/// Its nearest upright edge is `(1, 1)` from the axis, `√2` out, and neither
/// solid ever gets wider than `SLOPE * FAR`, which is `1`.
fn a_solid_and_a_block_clear_of_it(
    made: fn(&mut Model) -> SolidKey,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = made(&mut model);
    let side = SLOPE * FAR;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    let block = model
        .add_extrusion(
            &rectangle,
            FAR,
            &Isometry::translation(Vector::new(1.0, 1.0, CUT)).expect("a finite shift"),
        )
        .expect("a block is a solid");
    (model, solid, block)
}

/// The block's lower cap, turned to point out of the block, which is the
/// surface the answer's face carries and so the one the rim names.
fn the_blocks_lower_cap() -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, CUT), Vector::new(0.0, 0.0, -1.0))
            .expect("a normal with a length is a plane"),
    )
}
