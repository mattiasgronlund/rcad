//! A face bounded by a **ring** — one closed curve drawn right round a cone,
//! with no seam, no pole step and no corner on the apex.
//!
//! A pointed cone inside a cube stood on its corner keeps a piece of nappe round
//! its apex. The cube's three upper planes each cut the cone in an arc, the
//! three arcs meet at three corners off the apex, and together they close right
//! round the longitude. `docs/decisions.md` §206.6 found `mesh.rs` refusing that
//! face — `AWireWillNotLieFlatOnItsFace`, on the wire at `z ≈ 2.559` — and it is
//! the last of §200.3's eight fuses to refuse, the other four having been fixed
//! by §206.1.
//!
//! **The walk's own gain is what read it wrong, and one step is the whole of
//! it.** The walk stops at the last corner rather than stepping back onto the
//! first, so the ring came back `−5.235987755982988` where a whole turn is
//! `6.283185307179586`. Counting the closing step — which `finish` was already
//! doing for `climbed`, one line further down — makes it `−6.283185307179586`
//! exactly. Once right round, and a ring is allowed to be.
//!
//! **Which side of the ring the face is on is not this file's question to
//! answer**, and it is not guessed: `which_pole_this_cap_reaches` reads the step
//! the wire takes in space against `∇S × ∇T`, and the apex either lies on the
//! material side or there is no bounded face at all and the refusal stands.
//!
//! Nothing here needs the mesher to invent a seam. The apex is where the cone's
//! own second coordinate runs out, exactly as it is for a pointed cone's own
//! face, and `cut_a_cap_up_to_its_pole` fans four patches up to it.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

/// Slope one half, apex at the origin, base cap at `z = 8`.
const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// The cube of §200.3: side 4, diagonal upright, middle on the apex.
const SIDE: f64 = 4.0;

/// Union, intersection, cone less cube, cube less cone — one solid each, since
/// the two overlap and neither holds the other.
const RULES: [(Rule, bool); 4] = [
    (Rule::Union, true),
    (Rule::Intersection, true),
    (Rule::Difference, true),
    (Rule::Difference, false),
];

/// All eight of §200.3's fuses answer, where four of them refused.
///
/// Four already answered on the base: the union and the cone less the cube keep
/// no piece of nappe round the apex, because the cube covers it. The other four
/// do, and those are the four this pins.
#[test]
fn every_fuse_of_a_pointed_cone_with_a_cube_round_its_apex_answers() {
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

/// The ring's face is fanned up to the apex, and the apex is **one** point of
/// the mesh however many patches meet there.
///
/// Four patches, four cuts down to the apex, and every one of them names the
/// same entry in the point table — which is what makes the triangles round the
/// tip a fan rather than four separate slivers with a hole between them. Each
/// patch's own pole side is that point used twice, and the triangle standing on
/// it covers nothing and is dropped, so no triangle of the mesh has two corners
/// at the apex.
#[test]
fn the_ring_is_fanned_up_to_one_point_at_the_apex() {
    let (model, answer) = fuse(0.0, Rule::Intersection, true);
    let piece = model
        .solid(answer[0])
        .expect("this model made the key")
        .mesh(how_fine(), &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the shared piece meshes");

    let at_the_apex: Vec<u32> = (0..piece.points().len())
        .filter(|&at| piece.points()[at].distance_to(Point::origin()) < 1e-12)
        .map(|at| u32::try_from(at).expect("a mesh this size"))
        .collect();
    assert_eq!(
        at_the_apex.len(),
        1,
        "the apex is one point of the table, and it is at {at_the_apex:?}"
    );

    let standing_there = piece
        .triangles()
        .iter()
        .filter(|triangle| triangle.corners().contains(&at_the_apex[0]))
        .count();
    assert!(
        standing_there >= 8,
        "only {standing_there} triangles reach the apex, where four quarter-turn patches \
         fanning up to it cannot make fewer than eight"
    );
}

/// The four new answers are the right size, checked against the two closed
/// forms and nothing else.
///
/// **Every truth here is a closed form, and the shared volume is not one** — it
/// is a column sum (§200.3). So the shared piece is never compared to a number
/// written down here. What is compared is the three identities that hold
/// whatever it is: the union and the shared piece together are the two solids,
/// and each difference with the shared piece is its own solid.
///
/// **A measured volume of any of these carries an infinite bound**, so the slack
/// below is chosen rather than published, and that is `measure.rs`'s own gap
/// rather than this file's: a cone face that reaches its apex gets a radius of
/// nothing, and `how_much_space_hides_under_a_flat_stand_in` is honest about
/// having no argument there (`radius_of_each_face` says what would close it).
/// A **bare** pointed cone measures `134.02461503321067` with a bound of `inf`
/// on the base, so this is not something the ring brought with it.
#[test]
fn the_four_new_answers_and_the_shared_piece_add_up_to_the_two_solids() {
    let cone_volume = core::f64::consts::PI * SLOPE * SLOPE * HEIGHT.powi(3) / 3.0;
    let cube_volume = SIDE * SIDE * SIDE;

    for turn in [0.0, 0.7] {
        let shared = volume_of(turn, Rule::Intersection, true);
        let both = volume_of(turn, Rule::Union, true);
        let cone_less_cube = volume_of(turn, Rule::Difference, true);
        let cube_less_cone = volume_of(turn, Rule::Difference, false);

        // The mesh is drawn through points on the surfaces, so a cone face
        // falls inside the material and every sum below falls short by the
        // shell under it. A bare cone at this deflection is `0.017` short of
        // its own `134.041`, and no answer here holds more cone than that.
        let slack = 0.05;
        for (what, got, want) in [
            (
                "the union and the shared piece",
                both + shared,
                cone_volume + cube_volume,
            ),
            (
                "the cone less the cube, and the shared piece",
                cone_less_cube + shared,
                cone_volume,
            ),
            (
                "the cube less the cone, and the shared piece",
                cube_less_cone + shared,
                cube_volume,
            ),
        ] {
            assert!(
                (got - want).abs() <= slack,
                "at turn {turn}: {what} come to {got}, where the shapes say {want}"
            );
        }
    }
}

/// The cone's wall outside the cube is **one** face, in the union either way
/// round and in the cone less the cube, and the union meshes from coarse to
/// fine.
///
/// The cube's planes cut the wall in arcs and no ruling, and no one edge runs
/// from the rim to the ring round the apex: the fewest edges across are two
/// arcs where the cube's planes cross the wall. While a seam of more than one
/// edge had to be a chain of rulings, the wall stayed in the nine pieces the
/// cube cut it into (queue row Q868). Those two arcs kept as the seam were
/// once refused by the mesher, `TwoPatchesOfTheMeshDrewTheSameSide`, which is
/// why the union is meshed here at each deflection Q868 measured it at.
#[test]
fn the_cones_wall_outside_the_cube_is_one_face_that_meshes() {
    for turn in [0.0, 0.7] {
        for (rule, cone_first) in [
            (Rule::Union, true),
            (Rule::Union, false),
            (Rule::Difference, true),
        ] {
            let (model, answer) = fuse(turn, rule, cone_first);
            let walls = model
                .solid(answer[0])
                .expect("this model made the key")
                .faces()
                .filter(|face| face.surface().as_plane().is_none())
                .count();
            assert_eq!(
                walls, 1,
                "{rule:?} (cone first: {cone_first}) at turn {turn}: how many walls"
            );
        }
        let (model, answer) = fuse(turn, Rule::Union, true);
        let solid = model.solid(answer[0]).expect("this model made the key");
        for fine in [0.5, 0.05, 0.005, 0.0005] {
            solid
                .mesh(
                    Deflection::of(fine).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| {
                    panic!("the union at turn {turn} does not mesh at {fine}: {why:?}")
                });
        }
    }
}

/// The intersection of §200.3's pair, or whichever rule is asked for.
fn fuse(turn: f64, rule: Rule, cone_first: bool) -> (Model, Vec<SolidKey>) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);

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
