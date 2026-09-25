//! Associativity, idempotence and absorption, run over solids with a cone face.
//!
//! §221.5's third bullet: nothing had run these three laws over a cone.
//! `fuses_that_must_agree.rs` runs commutativity over a frustum up through a
//! block, and `what_the_answered_block_fuses_actually_say.rs` checks inclusion,
//! exclusion and the two ways round by volume — but neither asks whether
//! `(A ∪ B) ∪ C` is `A ∪ (B ∪ C)`, whether `A ∪ A` is `A`, or whether
//! `A ∪ (A ∩ B)` is `A`, with a cone in any of them.
//!
//! # The shapes
//!
//! Three families, each built the way the file that owns it builds it:
//!
//! - frustums up through a block's floor, from §49.6's grid
//!   (`pairs::FrustumAndBlock`);
//! - the sixteen frustum-and-block rows of
//!   `what_the_answered_block_fuses_actually_say.rs`, drawn with the same seed;
//! - a cone that ends in a point (`cones::a_pointed_cone`) through a block's
//!   floor, out through its roof, and swallowed whole by it.
//!
//! # What "the same answer" means here
//!
//! Two answers agree when they have the same number of solids, the same corners
//! **bit for bit**, and volumes within the sum of `measure`'s own bounds.
//!
//! The corners are compared as a set and not as a list, for the reason
//! `what_the_answered_block_fuses_actually_say.rs` gives at length: the fuse
//! drops the seam of a curved wall it never touched, so a frustum goes in with
//! its two seam corners listed twice each and comes out with them listed once.
//! Same points, same solid.
//!
//! The volumes are compared within a bound rather than bit for bit because two
//! ways of reaching the same solid can mesh its cone from different places — a
//! frustum with its seam and one without are the same solid and different
//! meshes. How many compared volumes came out bit-identical anyway is printed.
//!
//! # A refusal is counted, not failed
//!
//! A fuse that refuses on either side is tallied by name and printed, and is not
//! a failure — except where the two sides of one law disagree about whether
//! there is an answer at all. That is an asymmetry and it fails: whether a
//! union of three solids exists cannot depend on which two were united first.
//!
//! **Most of what is asked here is refused**, and the counts are printed so that
//! stays visible. Every law but associativity makes the two sides share faces —
//! `A ∩ B` shares its skin with `A` — and two copies of one curved face is
//! §36.16's row, which `fuses_that_must_agree.rs` pins for a bar. So each test
//! asserts a floor on what it compared, measured on the day it was written, and
//! a drop in it is a regression rather than a draw (§19.1).
//!
//! The five `#[ignore]`d tests at the bottom are the asymmetries this found, one
//! per shape, each still standing.

mod cones;
mod pairs;

use std::collections::BTreeMap;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

use cones::{a_frustum, a_pointed_cone};
use pairs::{FrustumAndBlock, WORKING_SIZE};

/// What two copies of one cone face come to today, whatever the law asks.
const TWO_COPIES_OF_ONE_CURVED_FACE: &str =
    "Undecidable on a point in a piece that every solid could place";

/// `A ∪ A'` and `A ∩ A'`, where `A'` is a frustum or a pointed cone built again
/// from the same numbers.
///
/// **Every one of them is refused, on two copies of one curved face**, the same
/// refusal `fuses_that_must_agree.rs` pins for `A \ A` over a bar. A pointed cone
/// has no seam, and a ball — which also has none — answers all three questions
/// there, so this says §36.16's row is wider than "a surface that closes round a
/// seam": a seamless cone refuses too.
///
/// The law is still checked on anything that answers, and a refusal of any other
/// kind fails, so the day these answer is the day the law is asserted.
#[test]
fn a_cone_with_a_copy_of_itself_is_refused_rather_than_united_or_shared() {
    let report = idempotence_of_the_cone_alone(&every_pair());
    for name in report.refused.keys() {
        assert_eq!(
            name, TWO_COPIES_OF_ONE_CURVED_FACE,
            "a cone and a copy of it refused for a new reason"
        );
    }
    report.print_and_check("idempotence, a cone and a copy of it", 0);
}

/// `U ∪ U'` and `U ∩ U'` are both `U`, where `U` is a block united with a cone
/// and `U'` is the same union made again in the same model.
///
/// What answers is the pointed cone the block swallows, where `U` is the block
/// and has no cone face left; every `U` that keeps one refuses.
#[test]
fn a_block_and_cone_united_with_a_copy_of_that_union_is_that_union() {
    idempotence_of_a_union_with_a_cone(&every_pair())
        .print_and_check("idempotence, a block-and-cone union and a copy of it", 2);
}

/// `A ∪ (A ∩ B)` is `A`, with `A` the block.
#[test]
fn a_block_united_with_what_it_shares_with_a_cone_is_the_block() {
    absorption(
        &every_pair(),
        Which::TheBlock,
        Rule::Intersection,
        Rule::Union,
    )
    .print_and_check("absorption A ∪ (A ∩ B), A the block", 5);
}

/// `A ∪ (A ∩ B)` is `A`, with `A` the cone.
///
/// Only the rows whose block misses the frustum answer, so `A ∩ B` is empty and
/// this is `A ∪ ∅` — the corners and the volume still have to come back.
#[test]
fn a_cone_united_with_what_it_shares_with_a_block_is_the_cone() {
    absorption(
        &every_pair(),
        Which::TheCone,
        Rule::Intersection,
        Rule::Union,
    )
    .print_and_check("absorption A ∪ (A ∩ B), A the cone", 4);
}

/// `A ∩ (A ∪ B)` is `A`, with `A` the block.
#[test]
fn a_block_shared_with_its_union_with_a_cone_is_the_block() {
    absorption(
        &every_pair(),
        Which::TheBlock,
        Rule::Union,
        Rule::Intersection,
    )
    .print_and_check("absorption A ∩ (A ∪ B), A the block", 4);
}

/// `A ∩ (A ∪ B)` is `A`, with `A` the cone.
#[test]
fn a_cone_shared_with_its_union_with_a_block_is_the_cone() {
    absorption(
        &every_pair(),
        Which::TheCone,
        Rule::Union,
        Rule::Intersection,
    )
    .print_and_check("absorption A ∩ (A ∪ B), A the cone", 1);
}

/// `(A ∪ B) ∪ C` is `A ∪ (B ∪ C)`, with `A` a block, `B` a cone and `C` a
/// second cube, over every trio where the two groupings agree about whether
/// there is an answer.
#[test]
fn uniting_a_block_a_cone_and_a_cube_does_not_depend_on_which_two_go_first() {
    associativity(&TRIOS_THAT_AGREE_ABOUT_ANSWERING)
        .print_and_check("associativity, a block, a cone and a cube", 3);
}

/// A frustum up through a block's floor, and a small cube wholly inside the
/// block, above the frustum.
///
/// `(block ∪ frustum) ∪ cube` answers. `block ∪ (frustum ∪ cube)` unites the
/// frustum and the cube, which never touch, into two pieces, and then that with
/// the block comes back [`rcad_topo::Error::ResultIsNotManifold`] — an answer no
/// model can hold, on one grouping of a union whose other grouping is a solid.
#[test]
#[ignore = "associativity: frustum through a block, cube inside the block — one grouping answers, \
            the other is ResultIsNotManifold"]
fn a_frustum_a_block_and_a_cube_inside_the_block_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: on_the_grid(9, 2, [3, 5]),
        third_side: 3.0,
        third_at: [0.5, 0.5, 4.5],
    }])
    .print_and_check("associativity, a cube inside the block", 1);
}

/// A pointed cone the block swallows whole, and a cube cutting through both.
///
/// `(block ∪ cone) ∪ cube` answers, since `block ∪ cone` is the block.
///
/// `cone ∪ cube` used to be refused, undecided on a point inside a piece of a
/// face, and answers now:
/// [`a_cube_wall_parallel_to_a_pointed_cones_axis_cuts_it_and_both_ways_answer`].
/// What still refuses is uniting that answer with the block, with
/// [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`] on the answer's
/// cone face — the part with `x < 3.5`, round the apex, whose one wire is a long
/// arc of the cap circle and the hyperbola arc the wall cut, between
/// `(3.5, 1.064, 5)` and `(3.5, 4.936, 5)`. That is items 4 and 5 of §222.3
/// again: the fuse cannot take back in a face of its own answer bounded by a
/// hyperbola arc.
#[test]
#[ignore = "associativity: pointed cone inside a block, cube cutting both — (cone ∪ cube) ∪ block \
            is AFaceTrimmedByACurveThisCannotFollow on the answer's cone face, bounded by a \
            hyperbola arc; the other grouping answers"]
fn a_swallowed_pointed_cone_and_a_cube_through_it_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: a_pointed_cone_in_a_block_six_across(1.0),
        third_side: 6.0,
        third_at: [3.5, 0.5, 0.5],
    }])
    .print_and_check("associativity, a cube through a swallowed cone", 1);
}

/// The pointed cone of the two tests either side, and the cube whose wall
/// `x = 3.5` runs half a unit off the cone's axis, parallel to it.
///
/// That wall cuts the cone in a hyperbola, and the part of the wall inside the
/// cone is a piece bounded by one arc of it and the chord the cap's plane cuts
/// across — two corners, and until `select.rs` counted the bulge of a
/// hyperbola arc on the arc's own half of the cone, no point inside it, so
/// `cone ∪ cube` was Undecidable on a point inside a piece of a face.
///
/// Union and intersection both answer now, and what they measure adds up to
/// what the two inputs measure, within `measure`'s own bounds.
#[test]
fn a_cube_wall_parallel_to_a_pointed_cones_axis_cuts_it_and_both_ways_answer() {
    let mut model = a_model();
    let (_, cone) = a_pointed_cone_in_a_block_six_across(1.0).build_into(&mut model);
    let cube = model
        .add_cube(6.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(3.5, 0.5, 0.5)).expect("a finite shift"),
        )
        .expect("this model made the key");

    let (united_model, united) =
        fused(&model, &[cone], &[cube], Rule::Union).expect("cone ∪ cube answers");
    let (shared_model, shared) =
        fused(&model, &[cone], &[cube], Rule::Intersection).expect("cone ∩ cube answers");
    assert_eq!((united.len(), shared.len()), (1, 1));

    let where_the_cap_crosses_the_wall =
        [1.063_508_326_896_291_5, 4.936_491_673_103_708].map(|y| [3.5, y, 5.0].map(f64::to_bits));
    let shared_corners = corners_of(&shared_model, &shared);
    for corner in where_the_cap_crosses_the_wall {
        assert!(
            shared_corners.contains(&corner),
            "the intersection's corners {shared_corners:?} miss {corner:?}"
        );
    }

    let [united, shared, cone, cube] = [
        volume_of(&united_model, &united),
        volume_of(&shared_model, &shared),
        volume_of(&model, &[cone]),
        volume_of(&model, &[cube]),
    ];
    let off_by = (united.volume + shared.volume) - (cone.volume + cube.volume);
    let bound = united.bound + shared.bound + cone.bound + cube.bound;
    assert!(
        off_by.abs() <= bound,
        "union {} and intersection {} against cone {} and cube {}: off by {off_by:e} beyond {bound:e}",
        united.volume,
        shared.volume,
        cone.volume,
        cube.volume
    );
}

/// A pointed cone the block swallows whole, and a small cube grazing the cone's
/// wall near the block's corner.
///
/// `(block ∪ cone) ∪ cube` answers. `cone ∪ cube` is refused with
/// [`rcad_topo::Error::ACurvedFaceGoesRightRoundWithNoSeam`], and the refusal is
/// right to stop there, though not for the reason its name gives.
///
/// The cube's three planes through its corner `(2.5, 2.5, 2.5)` cut the cone's
/// face **past the cube** as well: the plane `x = 2.5` along a whole hyperbola
/// branch through `(2.5, 3, 2)`, `y = 2.5` likewise, and `z = 2.5` right round.
/// The union keeps every piece outside the cube as a face of its own, so one of
/// them is the patch round the apex bounded by those three arcs, with corners
/// `(2.5, 3.559, 2.5)`, `(3.559, 2.5, 2.5)` and `(2.5, 2.5, 2.414)`, and every
/// edge of it has the cone on both sides. Measuring the answer meshes that
/// patch, and `mesh.rs`'s `which_pole_this_cap_reaches` looks for the surface
/// across its first edge among the faces' surfaces, finds only the cone, and
/// gives up — which is where the refusal is raised.
///
/// Reading that surface off the edge's run instead (`EdgeRun::PartOfWhereItCrosses`
/// names the plane) meshes it, and the union then answers with those extra
/// corners on it — and uniting that answer with the block is refused with
/// `AFaceMeetsItsNeighbourOnItsOwnSurface`, and `A ∪ (A ∩ B)` over the cone
/// whose apex is at `z = 3` answers with a corner `(3, 4.5, 6)` the cone does
/// not have. So the mesher is not the fault: the answer should not carry pieces
/// of one cone as separate faces, and until the fuse joins them the refusal is
/// the better of the two outcomes.
#[test]
#[ignore = "associativity: pointed cone inside a block, small cube grazing it — cone ∪ cube keeps \
            the cone's face cut into pieces by the cube's planes past the cube, and the piece \
            round the apex cannot be meshed (ACurvedFaceGoesRightRoundWithNoSeam)"]
fn a_swallowed_pointed_cone_and_a_cube_grazing_it_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: a_pointed_cone_in_a_block_six_across(1.0),
        third_side: 2.0,
        third_at: [0.5, 0.5, 0.5],
    }])
    .print_and_check("associativity, a cube grazing a swallowed cone", 1);
}

/// A frustum and the block that cuts a piece out of it, and a cube far from
/// both.
///
/// `block ∪ (frustum ∪ cube)` answers. `(block ∪ frustum) ∪ cube` refuses at
/// the second union with [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`]:
/// the fuse answered `block ∪ frustum` and then cannot take that answer back
/// in, even to unite it with a cube that touches nothing.
#[test]
#[ignore = "associativity: frustum cut by a block, cube far away — (block ∪ frustum) ∪ cube is \
            AFaceTrimmedByACurveThisCannotFollow, the other grouping answers"]
fn a_frustum_cut_by_a_block_and_a_far_cube_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: Pair::BesideABlock {
            slope: 0.5,
            near: 4.0,
            far: 9.0,
        },
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    }])
    .print_and_check("associativity, a cube far from a cut frustum", 1);
}

/// The same frustum and block, and a small cube standing in the frustum.
///
/// Refuses the same way as the far cube does, so a cube that does touch the
/// answer changes nothing about which grouping refuses.
#[test]
#[ignore = "associativity: frustum cut by a block, cube inside the frustum — (block ∪ frustum) ∪ \
            cube is AFaceTrimmedByACurveThisCannotFollow, the other grouping answers"]
fn a_frustum_cut_by_a_block_and_a_cube_in_the_frustum_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: Pair::BesideABlock {
            slope: 0.5,
            near: 4.0,
            far: 9.0,
        },
        third_side: 2.0,
        third_at: [3.5, 3.5, 0.5],
    }])
    .print_and_check("associativity, a cube in a cut frustum", 1);
}

/// A cube and a cone-faced solid.
#[derive(Clone, Copy, Debug)]
enum Pair {
    /// `pairs::FrustumAndBlock`, built the way that family builds it.
    Frustum(FrustumAndBlock),
    /// One row of `what_the_answered_block_fuses_actually_say.rs`'s family: a
    /// frustum about `z` with its apex at the origin, and beside it a box
    /// `slope · far` square and `far` tall with its low corner at
    /// `(1, 1, near + 1)`.
    BesideABlock { slope: f64, near: f64, far: f64 },
    /// A cube of `side` at the origin, and a cone that ends in a point with its
    /// apex at `apex`, widening by `slope` per unit up to a cap `far` above it.
    Pointed {
        side: f64,
        slope: f64,
        far: f64,
        apex: [f64; 3],
    },
}

/// A pair and a third cube, for associativity.
#[derive(Clone, Copy, Debug)]
struct Trio {
    pair: Pair,
    third_side: f64,
    third_at: [f64; 3],
}

/// Frustums up through a block's floor, three slopes, and one off the diagonal.
const FRUSTUMS: [Pair; 4] = [
    on_the_grid(6, 2, [3, 3]),
    on_the_grid(8, 4, [4, 4]),
    on_the_grid(10, 8, [5, 5]),
    on_the_grid(9, 2, [3, 5]),
];

/// A pointed cone with its apex below the block's floor, poking up through it;
/// one with its apex inside the block, poking out through the roof; and one
/// the block swallows whole.
const POINTED: [Pair; 3] = [
    a_pointed_cone_in_a_block_six_across(-1.0),
    a_pointed_cone_in_a_block_six_across(3.0),
    a_pointed_cone_in_a_block_six_across(1.0),
];

/// Trios on which the two groupings either both answer or both refuse.
///
/// Three answer both ways: a far cube with a frustum, a cube on the block's roof
/// with a frustum, and a far cube with a swallowed pointed cone. The rest refuse
/// both ways and are here so a change that makes one grouping answer is seen.
const TRIOS_THAT_AGREE_ABOUT_ANSWERING: [Trio; 7] = [
    Trio {
        pair: on_the_grid(6, 2, [3, 3]),
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    },
    Trio {
        pair: on_the_grid(6, 2, [3, 3]),
        third_side: 6.0,
        third_at: [0.0, 0.0, 6.0],
    },
    Trio {
        pair: a_pointed_cone_in_a_block_six_across(1.0),
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    },
    Trio {
        pair: on_the_grid(8, 2, [4, 4]),
        third_side: 8.0,
        third_at: [5.0, 0.5, 2.5],
    },
    Trio {
        pair: on_the_grid(8, 2, [4, 4]),
        third_side: 8.0,
        third_at: [5.0, 0.5, -2.5],
    },
    Trio {
        pair: on_the_grid(6, 2, [3, 3]),
        third_side: 4.0,
        third_at: [3.5, 0.5, -3.5],
    },
    Trio {
        pair: a_pointed_cone_in_a_block_six_across(-1.0),
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    },
];

/// A frustum of slope `1 / up_per_unit_wider` from a cap of radius one a unit
/// below the floor of a block `side` across, to a cap of radius two, with its
/// axis at `(at, at)`: one row of §49.6's eighty-four-configuration grid.
const fn on_the_grid(side: u32, up_per_unit_wider: u32, at: [u32; 2]) -> Pair {
    Pair::Frustum(FrustumAndBlock {
        side,
        narrow: 1,
        wide: 2,
        up_per_unit_wider,
        at,
    })
}

/// A pointed cone of slope one half and height four, standing in a block six
/// across with its axis at `(3, 3)` and its apex at `z`.
const fn a_pointed_cone_in_a_block_six_across(z: f64) -> Pair {
    Pair::Pointed {
        side: 6.0,
        slope: 0.5,
        far: 4.0,
        apex: [3.0, 3.0, z],
    }
}

/// Every pair: the grid frustums, the pointed cones, and the sixteen rows
/// `what_the_answered_block_fuses_actually_say.rs` draws.
fn every_pair() -> Vec<Pair> {
    let mut every: Vec<Pair> = FRUSTUMS.iter().chain(POINTED.iter()).copied().collect();
    every.extend(the_rows_the_answered_block_family_draws());
    every
}

/// Copied from `what_the_answered_block_fuses_actually_say.rs` with its seed, so
/// these are that family's rows and not a family of this file's own.
fn the_rows_the_answered_block_family_draws() -> Vec<Pair> {
    let mut draw = Draw(0x5EED);
    (0..16)
        .map(|_| {
            let slope = 2.0_f64.powi(i32::try_from(draw.next() % 3).expect("a small number") - 3);
            let far = draw.whole(6, 12);
            let near = draw.whole(1, 4);
            Pair::BesideABlock { slope, near, far }
        })
        .collect()
}

struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }
}

/// What one law over one family saw.
#[derive(Default)]
struct Report {
    compared: usize,
    volumes_bit_identical: usize,
    refused: BTreeMap<String, usize>,
    findings: Vec<String>,
}

impl Report {
    fn print_and_check(self, law: &str, at_least: usize) {
        println!(
            "{law}: {} compared, {} with bit-identical volumes, refusals {:?}",
            self.compared, self.volumes_bit_identical, self.refused
        );
        for finding in &self.findings {
            println!("  finding: {finding}");
        }
        assert!(
            self.findings.is_empty(),
            "{law}: {} findings, printed above",
            self.findings.len()
        );
        assert!(
            self.compared >= at_least,
            "{law}: {} comparisons where {at_least} were made the day this was written — a \
             fall is a regression; re-derive it rather than relax it (§19.1)",
            self.compared
        );
    }

    fn refused(&mut self, name: &str) {
        *self.refused.entry(name.to_owned()).or_default() += 1;
    }

    /// Compare an answer against what the law says it is, and write down any
    /// difference as a finding.
    fn compare(&mut self, what: &str, found: (&Model, &[SolidKey]), want: (&Model, &[SolidKey])) {
        self.compared += 1;
        let (found_model, found_pieces) = found;
        let (want_model, want_pieces) = want;
        if found_pieces.len() != want_pieces.len() {
            self.findings.push(format!(
                "{what}: {} solids where the law says {}",
                found_pieces.len(),
                want_pieces.len()
            ));
            return;
        }
        let found_corners = corners_of(found_model, found_pieces);
        let want_corners = corners_of(want_model, want_pieces);
        if found_corners != want_corners {
            let extra: Vec<[f64; 3]> = found_corners
                .iter()
                .filter(|it| !want_corners.contains(it))
                .map(|it| it.map(f64::from_bits))
                .collect();
            let missing: Vec<[f64; 3]> = want_corners
                .iter()
                .filter(|it| !found_corners.contains(it))
                .map(|it| it.map(f64::from_bits))
                .collect();
            self.findings.push(format!(
                "{what}: corners differ — {extra:?} extra and {missing:?} missing"
            ));
        }
        let found_volume = volume_of(found_model, found_pieces);
        let want_volume = volume_of(want_model, want_pieces);
        if found_volume.volume.to_bits() == want_volume.volume.to_bits() {
            self.volumes_bit_identical += 1;
        }
        let off_by = found_volume.volume - want_volume.volume;
        let bound = found_volume.bound + want_volume.bound;
        if off_by.abs() > bound {
            self.findings.push(format!(
                "{what}: volume {} against {}, off by {off_by:e} beyond the bound {bound:e}",
                found_volume.volume, want_volume.volume
            ));
        }
    }
}

/// `A ∪ A'` and `A ∩ A'` are both `A` — and so answer or refuse together.
fn idempotence_of_the_cone_alone(pairs: &[Pair]) -> Report {
    let mut report = Report::default();
    for &pair in pairs {
        let (model, one, copy) = pair.the_cone_and_a_copy_of_it();
        let mut answered = Vec::new();
        for rule in [Rule::Union, Rule::Intersection] {
            match fused(&model, &[one], &[copy], rule) {
                Ok((answer, pieces)) => {
                    report.compare(
                        &format!("{pair:?}: {rule:?} of the cone and a copy of it"),
                        (&answer, &pieces),
                        (&model, &[one]),
                    );
                    answered.push(true);
                }
                Err(name) => {
                    report.refused(&name);
                    answered.push(false);
                }
            }
        }
        if answered[0] != answered[1] {
            report.findings.push(format!(
                "{pair:?}: asymmetry — the union of the cone and a copy of it answered: {}, the \
                 intersection answered: {}",
                answered[0], answered[1]
            ));
        }
    }
    report
}

/// `U ∪ U'` and `U ∩ U'` are both `U` — and so answer or refuse together.
fn idempotence_of_a_union_with_a_cone(pairs: &[Pair]) -> Report {
    let mut report = Report::default();
    for &pair in pairs {
        let mut model = a_model();
        let (block, cone) = pair.build_into(&mut model);
        let (block_again, cone_again) = pair.build_into(&mut model);
        let (model, united) = match fused(&model, &[block], &[cone], Rule::Union) {
            Ok(answer) => answer,
            Err(name) => {
                report.refused(&format!("making U: {name}"));
                continue;
            }
        };
        let Ok((model, united_again)) = fused(&model, &[block_again], &[cone_again], Rule::Union)
        else {
            report.findings.push(format!(
                "{pair:?}: asymmetry — the same union answered once and refused the second time"
            ));
            continue;
        };
        let mut answered = Vec::new();
        for rule in [Rule::Union, Rule::Intersection] {
            match fused(&model, &united, &united_again, rule) {
                Ok((answer, pieces)) => {
                    report.compare(
                        &format!("{pair:?}: {rule:?} of a block-and-cone union and a copy of it"),
                        (&answer, &pieces),
                        (&model, &united),
                    );
                    answered.push(true);
                }
                Err(name) => {
                    report.refused(&name);
                    answered.push(false);
                }
            }
        }
        if answered[0] != answered[1] {
            report.findings.push(format!(
                "{pair:?}: asymmetry — the union of U and a copy of it answered: {}, the \
                 intersection answered: {}",
                answered[0], answered[1]
            ));
        }
    }
    report
}

/// Which of the pair is `A` in an absorption law.
#[derive(Clone, Copy, Debug)]
enum Which {
    TheBlock,
    TheCone,
}

/// `A ∪ (A ∩ B)` is `A` when `inner` is intersection and `outer` union, and
/// `A ∩ (A ∪ B)` is `A` the other way round.
fn absorption(pairs: &[Pair], which: Which, inner: Rule, outer: Rule) -> Report {
    let mut report = Report::default();
    for &pair in pairs {
        let mut model = a_model();
        let (block, cone) = pair.build_into(&mut model);
        let (a, b) = match which {
            Which::TheBlock => (block, cone),
            Which::TheCone => (cone, block),
        };
        let (after_inner, inner_pieces) = match fused(&model, &[a], &[b], inner) {
            Ok(answer) => answer,
            Err(name) => {
                report.refused(&format!("A {inner:?} B: {name}"));
                continue;
            }
        };
        match fused(&after_inner, &[a], &inner_pieces, outer) {
            Ok((answer, pieces)) => report.compare(
                &format!("{pair:?}: A = {which:?}, A {outer:?} (A {inner:?} B)"),
                (&answer, &pieces),
                (&model, &[a]),
            ),
            Err(name) => report.refused(&name),
        }
    }
    report
}

/// `(A ∪ B) ∪ C` is `A ∪ (B ∪ C)`, with `A` the block, `B` the cone and `C` a
/// second cube.
fn associativity(trios: &[Trio]) -> Report {
    let mut report = Report::default();
    for &trio in trios {
        let mut model = a_model();
        let (block, cone) = trio.pair.build_into(&mut model);
        let third = model
            .add_cube(
                trio.third_side,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a whole-number cube");
        let [x, y, z] = trio.third_at;
        model
            .place_solid(
                third,
                &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
            )
            .expect("this model made the key");

        let left = united_in_turn(&model, &[block], &[cone], &[third]);
        let right = united_in_turn(&model, &[cone], &[third], &[block]);
        match (left, right) {
            (Ok((left_model, left_pieces)), Ok((right_model, right_pieces))) => report.compare(
                &format!("{trio:?}: block ∪ (cone ∪ third) against (block ∪ cone) ∪ third"),
                (&right_model, &right_pieces),
                (&left_model, &left_pieces),
            ),
            (Err(left), Err(right)) => {
                report.refused(&left);
                report.refused(&right);
            }
            (Ok(_), Err(right)) => report.findings.push(format!(
                "{trio:?}: asymmetry — (block ∪ cone) ∪ third answered and block ∪ (cone ∪ \
                 third) refused at {right}"
            )),
            (Err(left), Ok(_)) => report.findings.push(format!(
                "{trio:?}: asymmetry — block ∪ (cone ∪ third) answered and (block ∪ cone) ∪ \
                 third refused at {left}"
            )),
        }
    }
    report
}

/// Unite the first two, then unite that with the third; a refusal at either
/// step comes back named.
fn united_in_turn(
    model: &Model,
    first: &[SolidKey],
    second: &[SolidKey],
    third: &[SolidKey],
) -> Result<(Model, Vec<SolidKey>), String> {
    let (model, two) = match fused(model, first, second, Rule::Union) {
        Ok(answer) => answer,
        Err(name) => return Err(format!("the first union: {name}")),
    };
    match fused(&model, &two, third, Rule::Union) {
        Ok(answer) => Ok(answer),
        Err(name) => Err(format!("the second union: {name}")),
    }
}

impl Pair {
    /// Build the block and then the cone into `model`, and hand back both.
    fn build_into(self, model: &mut Model) -> (SolidKey, SolidKey) {
        let policy = Policy::certified();
        match self {
            Pair::Frustum(pair) => {
                let block = model
                    .add_cube(f64::from(pair.side), &policy, &mut ProvenanceLog::new())
                    .expect("a whole-number cube");
                let frustum = a_frustum(model, pair.slope(), pair.near(), pair.far());
                let drop = Vector::new(
                    f64::from(pair.at[0]),
                    f64::from(pair.at[1]),
                    -1.0 - pair.near(),
                );
                model
                    .place_solid(
                        frustum,
                        &Isometry::translation(drop).expect("a whole-number shift"),
                    )
                    .expect("this model made the key");
                (block, frustum)
            }
            Pair::BesideABlock { slope, near, far } => {
                let frustum = a_frustum(model, slope, near, far);
                let side = slope * far;
                let rectangle =
                    ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
                        .build(&policy, &mut ProvenanceLog::new())
                        .expect("a rectangle with four corners");
                let block = model
                    .add_extrusion(
                        &rectangle,
                        far,
                        &Isometry::translation(Vector::new(1.0, 1.0, near + 1.0))
                            .expect("a finite shift"),
                    )
                    .expect("a box");
                (block, frustum)
            }
            Pair::Pointed {
                side,
                slope,
                far,
                apex,
            } => {
                let block = model
                    .add_cube(side, &policy, &mut ProvenanceLog::new())
                    .expect("a whole-number cube");
                let cone = a_pointed_cone(model, slope, far);
                model
                    .place_solid(
                        cone,
                        &Isometry::translation(Vector::new(apex[0], apex[1], apex[2]))
                            .expect("a whole-number shift"),
                    )
                    .expect("this model made the key");
                (block, cone)
            }
        }
    }

    /// The cone-faced solid alone, built twice into one fresh model.
    fn the_cone_and_a_copy_of_it(self) -> (Model, SolidKey, SolidKey) {
        let mut model = a_model();
        let (one, copy) = match self {
            Pair::Frustum(pair) => (
                a_frustum(&mut model, pair.slope(), pair.near(), pair.far()),
                a_frustum(&mut model, pair.slope(), pair.near(), pair.far()),
            ),
            Pair::BesideABlock { slope, near, far } => (
                a_frustum(&mut model, slope, near, far),
                a_frustum(&mut model, slope, near, far),
            ),
            Pair::Pointed { slope, far, .. } => (
                a_pointed_cone(&mut model, slope, far),
                a_pointed_cone(&mut model, slope, far),
            ),
        };
        (model, one, copy)
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"))
}

/// The model an answer is in and the answer's solids, or the name of why there
/// was no answer.
type Side = Result<(Model, Vec<SolidKey>), String>;

/// Fuse a copy of `model`. A certified answer that recorded a decision it could
/// not prove is a failure, as it is in `fuses_that_must_agree.rs`.
fn fused(model: &Model, arguments: &[SolidKey], tools: &[SolidKey], rule: Rule) -> Side {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    match model.fuse(arguments, tools, rule, &Policy::certified(), &mut log) {
        Ok(answer) => {
            assert!(
                log.is_empty(),
                "a certified {rule:?} recorded {} decisions it could not prove",
                log.len()
            );
            let solids = answer.solids().to_vec();
            Ok((model, solids))
        }
        Err(why) => Err(the_name_of(&why)),
    }
}

/// The error's variant without its fields, so refusals tally by kind — and for
/// an `Undecidable`, the decision it gave up on.
fn the_name_of(why: &rcad_topo::Error) -> String {
    if let rcad_topo::Error::Undecidable { records } = why {
        let last = records
            .records()
            .last()
            .map_or("nothing named", |record| record.predicate());
        return format!("Undecidable on {last}");
    }
    let written = format!("{why:?}");
    written
        .split(['{', '('])
        .next()
        .expect("split always hands back one piece")
        .trim()
        .to_owned()
}

/// Every corner of every face of every piece, as bits, sorted and without
/// repeats.
fn corners_of(model: &Model, pieces: &[SolidKey]) -> Vec<[u64; 3]> {
    let mut corners: Vec<[u64; 3]> = Vec::new();
    for &piece in pieces {
        for face in model.solid(piece).expect("this model made the key").faces() {
            for wire in face.wires() {
                for at in wire.points() {
                    corners.push(at.coordinates().map(f64::to_bits));
                }
            }
        }
    }
    corners.sort_unstable();
    corners.dedup();
    corners
}

/// A volume `measure` gave, all pieces together, and the bound it published.
struct Volume {
    volume: f64,
    bound: f64,
}

fn volume_of(model: &Model, pieces: &[SolidKey]) -> Volume {
    let how_fine = Deflection::of(0.01).expect("a positive length");
    let mut total = Volume {
        volume: 0.0,
        bound: 0.0,
    };
    for &piece in pieces {
        let it = model
            .solid(piece)
            .expect("this model made the key")
            .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("every piece of these answers measures");
        assert!(
            it.volume_error().is_finite(),
            "a bound of {} on the volume checks nothing",
            it.volume_error()
        );
        total.volume += it.volume();
        total.bound += it.volume_error();
    }
    total
}
