//! A face bounded by a **ring** of arcs that also has a hole: a frustum's band
//! between its own near cap circle and a ring of arcs a cube's upper planes cut.
//!
//! `docs/decisions.md` §208.4 left this open, and called the missing piece
//! *"which of the two pieces the ring cuts the surface into the hole sits
//! in"*. On a surface whose longitude wraps there is nothing to choose. Two
//! loops that each go right round cut the chart into three stretches — below
//! both, between them, above both — and only the one **between** them has both
//! loops in its boundary. A face is bounded by all of its wires, so a face with
//! two such wires is the band and no other stretch, whatever a fuse makes of it.
//! That is the shape `cut_a_ring_across` has cut since a bar's wall between two
//! planes, and the ring of arcs only had to be let in.
//!
//! **Letting it in found a second defect in the cutting, and one that was
//! already there.** A ring is cut at four quarter turns taken off the *outer*
//! loop's own first corner, and each quarter is matched to the corner of the
//! inner loop nearest it. Both loops run a whole turn, but each starts wherever
//! the pair that cut it started, so the two spans of longitude are offset: this
//! frustum's inner loop runs `-2.66` to `3.58` where its outer runs `0` to
//! `6.28`. The quarter at `4.71` fell off the end of the inner loop, so the
//! nearest corner to it was whichever corner the loop ended on — and a patch
//! then took a sliver of the inner loop where it took a quarter of the outer
//! one. That is a boundary folded across itself, and the triangulator said so:
//! `CannotMeshAFace { why: BoundaryCrossesItself { at: 113 } }`. Bringing each
//! quarter into the inner loop's own turn first is the whole fix.
//!
//! A bar through a block was cut that way too, and nothing had caught it: its
//! inner loop's corners came back `[1, 2, 3, 3]` for the four quarters, so one
//! patch had no width along the bottom at all and the next spanned half the
//! turn. That is what `cut_off_what_goes_too_far_round`'s own argument says no
//! patch may do, and it is why
//! `a_curved_pair_obeys_inclusion_and_exclusion_inside_the_error_the_four_declare`
//! reads a different share of its tolerance now.
//!
//! **A face with one wire right round beside one that is not was refused**,
//! and that was a gap rather than a limit: the answer exists — the winding
//! loop cuts the chart in two and the other wire lies in one of the halves —
//! but `cut_a_cap_up_to_its_pole` handed back patches with one loop each and
//! had nowhere to put an island. **Since Q2811 that gap is closed on a cone's
//! own cap round its apex**: `a_cap_on_a_ball_or_a_cone_with_islands_in_it`
//! (`mesh.rs`) now puts an island there, and `a_tapered_shaft_with_a_cross_hole.rs`
//! fuses a pointed cone with a ball poking out of its side, with a bar leaning
//! through it, and with a pin pushed through it square to its axis, and every
//! one answers under every rule. `mesh.rs`'s
//! `a_curved_face_with_a_hole_is_refused_rather_than_asserted_against` still
//! demonstrates the refusal reaching a shape no builder here makes, arrived at
//! through `Deserialize`-shaped arena surgery instead.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_frustum;

/// Slope one half, cap circles at `z = 2` and `z = 8`.
const SLOPE: f64 = 0.5;
const NEAR: f64 = 2.0;
const FAR: f64 = 8.0;

/// The cube of §200.3: side 4, diagonal upright, middle on the axis at `z = 0`,
/// so its three upper planes cut the frustum's band in three arcs that close
/// right round it and its lower ones cut the nappe the frustum has no face on.
const SIDE: f64 = 4.0;

/// Union, intersection, frustum less cube, cube less frustum.
const RULES: [(Rule, bool); 4] = [
    (Rule::Union, true),
    (Rule::Intersection, true),
    (Rule::Difference, true),
    (Rule::Difference, false),
];

/// Every rule answers, where the base refused all eight.
///
/// On `main` at `c23b960` each of these came back
/// `Error::Undecidable`, the last record naming `AWireWillNotLieFlatOnItsFace`:
/// the band's ring of arcs is not its face's one wire, and only a face's one
/// wire was allowed to come back a whole turn on.
#[test]
fn every_fuse_of_a_frustum_with_a_cube_round_its_apex_answers() {
    for turn in [0.0, 0.7] {
        for (rule, cone_first) in RULES {
            let (model, answer) = fuse(turn, rule, cone_first);
            assert_eq!(
                answer.len(),
                1,
                "{rule:?} (cone first: {cone_first}) at turn {turn}: how many solids"
            );
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| {
                    panic!("{rule:?} (cone first: {cone_first}) at turn {turn}: {why:?}")
                });
        }
    }
}

/// The band meshes, and its measured size is the one the shapes say.
///
/// **Neither the shared piece nor the two differences has a closed form here**,
/// so none of them is compared to a number written down. What is compared is the
/// three identities that hold whatever the shared piece is, each of which needs
/// only the frustum's and the cube's own closed forms.
///
/// Every mesh is drawn through points *on* the cone, so the band falls inside
/// the material and each sum comes up short by the shell under it. **The slack
/// is that shell, measured here rather than chosen**: a bare frustum at this
/// deflection is short of its own closed form by some amount, no answer below
/// holds more cone face than a whole frustum does, and twice that is the
/// allowance.
#[test]
fn the_band_and_the_shared_piece_add_up_to_the_frustum_and_the_cube() {
    let frustum = core::f64::consts::PI * SLOPE * SLOPE * (FAR.powi(3) - NEAR.powi(3)) / 3.0;
    let cube = SIDE * SIDE * SIDE;
    let slack = 2.0 * (frustum - a_bare_frustums_measured_volume());

    for turn in [0.0, 0.7] {
        let shared = volume_of(turn, Rule::Intersection, true);
        let both = volume_of(turn, Rule::Union, true);
        let cone_less_cube = volume_of(turn, Rule::Difference, true);
        let cube_less_cone = volume_of(turn, Rule::Difference, false);

        for (what, got, want) in [
            (
                "the union and the shared piece",
                both + shared,
                frustum + cube,
            ),
            (
                "the frustum less the cube, and the shared piece",
                cone_less_cube + shared,
                frustum,
            ),
            (
                "the cube less the frustum, and the shared piece",
                cube_less_cone + shared,
                cube,
            ),
        ] {
            assert!(
                (got - want).abs() <= slack,
                "at turn {turn}: {what} come to {got}, where the shapes say {want}"
            );
        }
    }
}

/// The shared piece's own bound is finite and small, which is what says the band
/// was cut into patches rather than merely handed over.
///
/// A frustum keeps its apex off every face, so nothing here meets
/// `measure.rs`'s cone-tip gap (§208.4) and the bound is a real one. Before the
/// change this piece could not be reached at all; the first mesh of it that the
/// fuse let through came back `BoundaryCrossesItself`, which is a refusal and
/// not a bound.
#[test]
fn the_shared_piece_measures_with_a_bound_worth_having() {
    let (model, answer) = fuse(0.7, Rule::Intersection, true);
    let measured = model
        .solid(answer[0])
        .expect("this model made the key")
        .measure(how_fine(), &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the shared piece measures");
    assert!(
        measured.volume_error().is_finite() && measured.volume_error() < 0.01,
        "the shared piece declares {}, where a band clear of the apex owes a \
         bound of its own",
        measured.volume_error()
    );
    assert!(
        measured.volume() > 2.0 && measured.volume() < 2.6,
        "the shared piece measures {}, where the cube's upper corner over the \
         frustum's band is about 2.3",
        measured.volume()
    );
}

/// The frustum of §200.3's family and the cube stood on its corner round its
/// apex, fused by `rule`.
fn fuse(turn: f64, rule: Rule, cone_first: bool) -> (Model, Vec<SolidKey>) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_frustum(&mut model, SLOPE, NEAR, FAR);

    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction");
    let stood = Isometry::rotation(&diagonal, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn");
    let down = Isometry::translation(Vector::new(0.0, 0.0, -SIDE * 3.0f64.sqrt() / 2.0))
        .expect("a finite move");
    let placed = Isometry::rotation(&Axis::z(), turn)
        .expect("a finite turn")
        .compose(&down)
        .compose(&stood);
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [SIDE, 0.0], [SIDE, SIDE], [0.0, SIDE]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let cube = model
        .add_extrusion(&square, SIDE, &placed)
        .expect("a square extrudes");

    let (arguments, tools) = if cone_first {
        ([cone], [cube])
    } else {
        ([cube], [cone])
    };
    let answer = model
        .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| {
            panic!("{rule:?} (cone first: {cone_first}) at turn {turn}: {why:?}")
        });
    let solids = answer.solids().to_vec();
    (model, solids)
}

/// The frustum on its own, measured the same way, so that the shell a cone face
/// loses to its own mesh is a number this file has run rather than one it quotes.
fn a_bare_frustums_measured_volume() -> f64 {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_frustum(&mut model, SLOPE, NEAR, FAR);
    model
        .solid(cone)
        .expect("this model made the key")
        .measure(how_fine(), &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a frustum measures")
        .volume()
}

fn volume_of(turn: f64, rule: Rule, cone_first: bool) -> f64 {
    let (model, answer) = fuse(turn, rule, cone_first);
    answer
        .iter()
        .map(|&piece| {
            model
                .solid(piece)
                .expect("this model made the key")
                .measure(how_fine(), &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a piece measures")
                .volume()
        })
        .sum()
}

fn how_fine() -> Deflection {
    Deflection::of(0.002).expect("a positive length")
}
