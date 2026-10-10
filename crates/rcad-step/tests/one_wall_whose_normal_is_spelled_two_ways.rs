//! One leaning wall, written as two faces whose `PLANE`s spell its normal two
//! ways, reads and fuses as the same file spelled one way (`docs/decisions.md`
//! §355.5, queue row Q498).
//!
//! The reader keeps a normal as the file wrote it when it is within
//! `Plane::NORMAL_LENGTH_ROUNDOFF` of length one (§355.1). The `f64` just below
//! `1/√2` and the one just above both are, so `(s, 0, s)` and `(s', 0, s')` are
//! both kept, and a file that writes one of them on each half of one wall gets
//! two planes whose four numbers differ in the last bit. Where the wall sits on
//! `x + z = 2` the two are still one plane, since `2s` and `2s'` are exact; on
//! `x + z = 3` the offsets round, and they are two parallel planes a rounding
//! apart.
//!
//! **Neither changes an answer.** The fuse joins two faces of an input that
//! share an edge and face the same way on parallel planes into one face, on the
//! smaller of the two planes (§17.4 amended 2026-08-31, §426), so the block
//! spelled two ways fuses exactly as the block spelled the smaller way: every
//! rule, both orders, every face's plane and every corner to the bit.

mod written_by_hand;

use core::f64::consts::FRAC_1_SQRT_2;

use rcad_geom::{Deflection, Isometry, PlaneAgreement, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Model, Rule, SolidKey};
use written_by_hand::{a_header, real, triple};

/// The `f64` just below `1/√2`, which `Plane::from_point_and_normal` never
/// stores: it scales `(1, 0, 1)` to [`JUST_ABOVE`] (§22.6).
const JUST_BELOW: f64 = 0.707_106_781_186_547_5;

/// The `f64` just above `1/√2`, and the nearest one to it.
const JUST_ABOVE: f64 = FRAC_1_SQRT_2;

/// A wall whose offset is exact for both spellings, and one whose offset rounds.
const REACHES: [f64; 2] = [2.0, 3.0];

/// The two ways round the two spellings can go on the wall's two halves.
const TWO_WAYS: [[f64; 2]; 2] = [[JUST_BELOW, JUST_ABOVE], [JUST_ABOVE, JUST_BELOW]];

#[test]
fn both_spellings_of_one_leaning_wall_are_kept_as_the_file_wrote_them() {
    assert_eq!(
        JUST_BELOW.to_bits() + 1,
        JUST_ABOVE.to_bits(),
        "the two spellings are neighbouring f64"
    );
    for (reach, the_two_planes) in REACHES
        .into_iter()
        .zip([PlaneAgreement::SameWay, PlaneAgreement::Different])
    {
        let (model, block) = read(&a_block_with_a_leaning_wall(
            reach,
            [JUST_BELOW, JUST_ABOVE],
        ));
        let walls: Vec<rcad_geom::Plane> = model
            .solid(block)
            .expect("the import made the key")
            .faces()
            .filter_map(|face| face.surface().as_plane().copied())
            .filter(|plane| plane.normal().components()[0] > 0.0)
            .collect();
        let [one, other] = walls[..] else {
            panic!("the leaning wall is read as two faces, not {}", walls.len())
        };
        assert_eq!(
            [one.normal().components(), other.normal().components()],
            [[JUST_BELOW, 0.0, JUST_BELOW], [JUST_ABOVE, 0.0, JUST_ABOVE]],
            "on x + z = {reach}, both normals are near enough length one to be kept"
        );
        assert_eq!(
            one.agreement_with(&other),
            the_two_planes,
            "on x + z = {reach}, the offsets are {} and {}",
            one.coefficients()[3],
            other.coefficients()[3]
        );
    }
}

/// Every fuse of the block spelled two ways with a cube crossing its wall is the
/// fuse of the block spelled the smaller way, to the last bit of every corner,
/// and measures what the shape does worked out by hand.
#[test]
fn a_wall_spelled_two_ways_fuses_as_it_does_spelled_the_smaller_way() {
    for reach in REACHES {
        let smaller = a_block_with_a_leaning_wall(reach, [JUST_BELOW, JUST_BELOW]);
        for spellings in TWO_WAYS {
            let both = a_block_with_a_leaning_wall(reach, spellings);
            for (rule, order, by_hand) in every_fuse_and_what_it_gives(reach) {
                let answer = fused(&both, reach, rule, order);
                assert_eq!(
                    answer,
                    fused(&smaller, reach, rule, order),
                    "x + z = {reach}, spelled {spellings:?}, {rule:?} {order:?}"
                );
                assert_eq!(
                    (answer.faces, answer.volume),
                    by_hand,
                    "x + z = {reach}, spelled {spellings:?}, {rule:?} {order:?}"
                );
            }
        }
    }
}

/// Against the block spelled the larger way, the wall's own four numbers differ
/// in every answer, as they must, since the joined face keeps the smaller plane.
/// The face counts agree everywhere, and so do the volumes but one: on
/// `x + z = 3`, where the larger spelling's plane is a rounding further out, the
/// overlap of block and cube measures one ulp more than the hand-worked
/// `0.158203125` the two spellings give.
#[test]
fn a_wall_spelled_two_ways_has_the_faces_and_volumes_it_has_spelled_the_larger_way_but_one() {
    let mut differing = Vec::new();
    for reach in REACHES {
        let larger = a_block_with_a_leaning_wall(reach, [JUST_ABOVE, JUST_ABOVE]);
        for spellings in TWO_WAYS {
            let both = a_block_with_a_leaning_wall(reach, spellings);
            for (rule, order, _) in every_fuse_and_what_it_gives(reach) {
                let answer = fused(&both, reach, rule, order);
                let spelled_one_way = fused(&larger, reach, rule, order);
                assert_eq!(
                    answer.faces, spelled_one_way.faces,
                    "x + z = {reach}, spelled {spellings:?}, {rule:?} {order:?}"
                );
                if answer.volume != spelled_one_way.volume {
                    differing.push(format!(
                        "x + z = {reach}, {rule:?} {order:?}: {:?} against {:?}",
                        answer.volume, spelled_one_way.volume
                    ));
                }
            }
        }
    }
    let one_way_round = [
        "x + z = 3, Intersection BlockFirst: 0.158203125 against 0.15820312500000003",
        "x + z = 3, Intersection CubeFirst: 0.158203125 against 0.15820312500000003",
    ];
    assert_eq!(differing, [one_way_round, one_way_round].concat());
}

/// Which of the two goes in as the argument and which as the tool.
#[derive(Clone, Copy, Debug)]
enum Order {
    BlockFirst,
    CubeFirst,
}

/// What a fuse handed back, in its bits.
#[derive(Debug, PartialEq)]
struct Answer {
    faces: usize,
    volume: f64,
    planes: Vec<[u64; 4]>,
    corners: Vec<[u64; 3]>,
}

/// Every rule both ways round, with the face count and volume each gives,
/// worked out from the shape.
///
/// The block's end is a trapezoid `reach` wide at the floor and `reach − 1` at
/// the top, one high and two deep, so it holds `2·reach − 1`. The cube reaches
/// `0.625` into the block, starting `0.0625` above its floor, and the wall cuts
/// that corner off as a right triangle of side `0.5625`, one deep: the overlap
/// is `0.5625² / 2`. The union has the block's six faces and the cube's six, the
/// overlap is a prism of five, the block less the cube keeps its six and takes
/// four of the cube's, and the cube less the block keeps its six and takes the
/// wall.
fn every_fuse_and_what_it_gives(reach: f64) -> [(Rule, Order, (usize, f64)); 6] {
    let block = 2.0 * reach - 1.0;
    let overlap = 0.5625 * 0.5625 / 2.0;
    [
        (Rule::Union, Order::BlockFirst, (12, block + 1.0 - overlap)),
        (Rule::Union, Order::CubeFirst, (12, block + 1.0 - overlap)),
        (Rule::Intersection, Order::BlockFirst, (5, overlap)),
        (Rule::Intersection, Order::CubeFirst, (5, overlap)),
        (Rule::Difference, Order::BlockFirst, (10, block - overlap)),
        (Rule::Difference, Order::CubeFirst, (7, 1.0 - overlap)),
    ]
}

/// The block read from `text`, fused with a cube of side one that crosses its
/// wall and both halves of it.
///
/// The cube runs from `(reach − 0.625, 0.5, 0.0625)` to one further along each
/// axis. The diagonal between the wall's two halves enters it through its face
/// at `y = 0.5` and leaves through its face at `x = reach − 0.625`, each well
/// inside the face, so no corner of the cube is on the diagonal and no edge of
/// the cube crosses it.
fn fused(text: &str, reach: f64, rule: Rule, order: Order) -> Answer {
    let policy = Policy::certified();
    let (mut model, block) = read(text);
    let cube = model
        .add_cube(1.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side one fits beside the block");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(reach - 0.625, 0.5, 0.0625))
                .expect("a finite shift"),
        )
        .expect("this model made the key");
    let (arguments, tools) = match order {
        Order::BlockFirst => ([block], [cube]),
        Order::CubeFirst => ([cube], [block]),
    };
    let fused = model
        .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?} {order:?} on x + z = {reach}: {why:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the answer is sound");
    let [solid] = fused.solids() else {
        panic!(
            "{rule:?} {order:?} on x + z = {reach} is one solid, not {}",
            fused.solids().len()
        )
    };
    the_bits_of(&model, *solid)
}

fn the_bits_of(model: &Model, key: SolidKey) -> Answer {
    let solid = model.solid(key).expect("the fuse made the key");
    let mut planes: Vec<[u64; 4]> = solid
        .faces()
        .filter_map(|face| face.surface().as_plane().copied())
        .map(|plane| plane.coefficients().map(f64::to_bits))
        .collect();
    planes.sort_unstable();
    let mut corners: Vec<[u64; 3]> = solid
        .faces()
        .flat_map(|face| face.wires().collect::<Vec<_>>())
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .map(|coedge| coedge.start().point().coordinates().map(f64::to_bits))
        .collect();
    corners.sort_unstable();
    corners.dedup();
    let volume = solid
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid measures")
        .volume();
    Answer {
        faces: solid.faces().count(),
        volume,
        planes,
        corners,
    }
}

/// The model `text` makes, checked sound, and the one solid in it.
fn read(text: &str) -> (Model, SolidKey) {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote parses");
    let policy = Policy::certified();
    let import =
        Import::read(&file, &policy, &mut ProvenanceLog::new()).expect("the block imports");
    import
        .model()
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the block read back is sound");
    let [(_, block)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let block = *block;
    (import.into_model(), block)
}

/// A block two deep along `y` and one high whose end wall leans: its floor runs
/// from `x = 0` to `x = reach` and its top to `x = reach − 1`, so the wall is
/// `x + z = reach`.
///
/// The wall is written as two triangles either side of its diagonal from
/// `(reach, 0, 0)` to `(reach − 1, 2, 1)`, each on a `PLANE` of its own, placed
/// at `(reach, 0, 0)` and facing along `(n, 0, n)` for the `n` that `spellings`
/// gives it.
fn a_block_with_a_leaning_wall(reach: f64, spellings: [f64; 2]) -> String {
    let corners = [
        [0.0, 0.0, 0.0],
        [reach, 0.0, 0.0],
        [reach - 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 2.0, 0.0],
        [reach, 2.0, 0.0],
        [reach - 1.0, 2.0, 1.0],
        [0.0, 2.0, 1.0],
    ];
    let edges: [[usize; 2]; 13] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
        [1, 6],
    ];
    // Each loop runs the way a right hand turns about the face's outward normal.
    let faces: [(&[usize], [f64; 3]); 7] = [
        (&[0, 4, 5, 1], [0.0, 0.0, -1.0]),
        (&[3, 2, 6, 7], [0.0, 0.0, 1.0]),
        (&[0, 1, 2, 3], [0.0, -1.0, 0.0]),
        (&[4, 7, 6, 5], [0.0, 1.0, 0.0]),
        (&[0, 3, 7, 4], [-1.0, 0.0, 0.0]),
        (&[1, 5, 6], [spellings[0], 0.0, spellings[0]]),
        (&[1, 6, 2], [spellings[1], 0.0, spellings[1]]),
    ];

    let mut text = a_header(".MILLI.");
    for (which, corner) in corners.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            100 + which,
            triple(*corner)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
    }
    for (which, [from, to]) in edges.iter().enumerate() {
        // The reader takes a `LINE` as straight between the edge's two corners
        // and reads nothing else of it, so its direction need not have length one.
        let way = [0, 1, 2].map(|axis| corners[*to][axis] - corners[*from][axis]);
        text += &format!("#{}=DIRECTION('',{});\n", 140 + which, triple(way));
        text += &format!(
            "#{}=VECTOR('',#{},{});\n",
            160 + which,
            140 + which,
            real(1.0)
        );
        text += &format!(
            "#{}=LINE('',#{},#{});\n",
            180 + which,
            100 + from,
            160 + which
        );
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            200 + which,
            110 + from,
            110 + to,
            180 + which
        );
    }
    for (which, (round, normal)) in faces.iter().enumerate() {
        let mut steps = Vec::new();
        for (step, &from) in round.iter().enumerate() {
            let to = round[(step + 1) % round.len()];
            let (edge, forwards) = edges
                .iter()
                .enumerate()
                .find_map(|(edge, ends)| {
                    (*ends == [from, to])
                        .then_some((edge, true))
                        .or((*ends == [to, from]).then_some((edge, false)))
                })
                .expect("every step of a loop is one of the block's edges");
            let id = 300 + which * 4 + step;
            text += &format!(
                "#{id}=ORIENTED_EDGE('',*,*,#{},{});\n",
                200 + edge,
                if forwards { ".T." } else { ".F." }
            );
            steps.push(format!("#{id}"));
        }
        text += &format!("#{}=EDGE_LOOP('',({}));\n", 340 + which, steps.join(","));
        text += &format!(
            "#{}=FACE_OUTER_BOUND('',#{},.T.);\n",
            350 + which,
            340 + which
        );
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            360 + which,
            triple(corners[round[0]])
        );
        text += &format!("#{}=DIRECTION('',{});\n", 370 + which, triple(*normal));
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#{},$);\n",
            380 + which,
            360 + which,
            370 + which
        );
        text += &format!("#{}=PLANE('',#{});\n", 390 + which, 380 + which);
        text += &format!(
            "#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n",
            400 + which,
            350 + which,
            390 + which
        );
    }
    let skin: Vec<String> = (0..faces.len())
        .map(|which| format!("#{}", 400 + which))
        .collect();
    text += &format!("#500=CLOSED_SHELL('',({}));\n", skin.join(","));
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
