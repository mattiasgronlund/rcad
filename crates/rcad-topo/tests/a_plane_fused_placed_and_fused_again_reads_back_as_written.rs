//! A plane that a fuse bakes, a placement turns and a second fuse bakes again
//! comes back from its bytes with the four numbers the file wrote, and a
//! second read moves it no more than the first (Q3220 and Q3226, from Q2826
//! and `docs/decisions.md` §464.37).
//!
//! A cube is united with a second cube, the answer is placed turned about a
//! slanted axis, and the next round unites it with another cube. After each
//! round the model is written with `postcard` and read back. A normal that the
//! placements turned again and again is not exactly length one. Until Q3226
//! `Deserialize` took a plane through `Plane::from_coefficients`, which scales
//! the first three numbers to length one again, so the first read moved 2 320
//! planes by up to three last digits, and a second read moved three planes of
//! axis `(1, 2, 3)` again from its fourth round on. It keeps a normal that is
//! within `Plane::NORMAL_LENGTH_ROUNDOFF` of length one now, as `Axis`'s
//! `Deserialize` keeps a direction (§355.6).
//!
//! Every one of the 24 rounds answers, and any refusal fails. Until Q3224 each
//! axis stopped at its second or third round, refused `Undecidable` for a point
//! in the new cube's floor that a fan triangle of almost no area could not
//! place, so 7 of the 24 answered.
//!
//! The first test shows a read gives back every plane as written, at each of
//! eight rounds on each of three slanted axes. The second shows a second read
//! gives back what the first did. The third shows each union takes up what its
//! two solids do less their overlap.

use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const ROUNDS: usize = 8;

const AXES: [(f64, f64, f64); 3] = [(1.0, 2.0, 3.0), (0.3, -0.7, 0.2), (1.0, 1.0, 1.0)];

/// How far a union and its overlap may add up to other than its two solids, in
/// `f64::EPSILON`s of what the two solids take up together. The worst of the 24
/// rounds is `3.73`, at axis `(1, 1, 1)`, round 3. It is not `0` because the
/// fuse puts each new corner where three stored planes meet, rounded, and the
/// union and the overlap round theirs apart.
const MOST_A_UNION_AND_ITS_OVERLAP_ARE_OUT: f64 = 8.0;

/// Until Q3226 this read moved 2 320 of the planes, a number of one by up to
/// three steps between neighbouring `f64`s at one.
#[test]
fn a_read_gives_back_every_plane_a_turn_left_as_written() {
    let (mut planes, mut moved) = (0, 0);
    for_each_round(|axis, round, model, _| {
        let read_back = written_and_read(model);
        let (written, read) = (its_planes(model), its_planes(&read_back));
        assert_eq!(written.len(), read.len(), "{axis:?}, round {round}");
        planes += written.len();
        moved += planes_that_differ(&written, &read);
    });
    println!("the read moved {moved} of {planes} planes");
    assert_eq!(moved, 0, "a read moved {moved} of {planes} planes");
}

/// **A plane read back is read back the same again.** Until Q3226 three
/// of the planes axis `(1, 2, 3)` makes at its fourth round moved on the
/// second read, and the model kept them for every round after:
/// `Plane::from_coefficients` scales an already scaled normal to length one,
/// and `1 053` of `100 000` such normals come back with other bits
/// (`rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal`, in
/// `rcad-geom`).
#[test]
fn a_second_read_moves_no_plane() {
    for_each_round(|axis, round, model, _| {
        let once = written_and_read(model);
        let twice = written_and_read(&once);
        assert_eq!(
            planes_that_differ(&its_planes(&once), &its_planes(&twice)),
            0,
            "{axis:?}, round {round}: planes moved on the second read"
        );
    });
}

#[test]
fn every_union_takes_up_what_its_two_solids_do_less_their_overlap() {
    let mut worst = 0.0f64;
    for_each_round(|axis, round, model, united| {
        let mut overlapped = model.clone();
        let shared = overlapped
            .fuse(
                &[united.kept],
                &[united.other],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{axis:?}, round {round}: the overlap {why:?}"));
        let overlap: f64 = shared
            .solids()
            .iter()
            .map(|&key| volume_of(&overlapped, key))
            .sum();
        let (kept, other) = (
            volume_of(model, united.kept),
            volume_of(model, united.other),
        );
        let out = (volume_of(model, united.union) + overlap - kept - other).abs()
            / (f64::EPSILON * (kept + other));
        worst = worst.max(out);
    });
    println!("a union and its overlap are out by at most {worst} eps of the two solids");
    assert!(
        worst <= MOST_A_UNION_AND_ITS_OVERLAP_ARE_OUT,
        "a union and its overlap are out by {worst} eps of the two solids"
    );
}

/// The keys one round's union was made of and made, in the model the round
/// leaves: the solid kept from the round before, the new cube, and their
/// union, placed turned.
struct United {
    kept: SolidKey,
    other: SolidKey,
    union: SolidKey,
}

fn for_each_round(mut check: impl FnMut((f64, f64, f64), usize, &Model, &United)) {
    for (x, y, z) in AXES {
        let axis =
            Axis::new(Point::new(0.5, -0.25, 0.75), Vector::new(x, y, z)).expect("a slanted axis");
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let mut kept = a_cube(&mut model, 0.0);
        for round in 0..ROUNDS {
            let other = a_cube(&mut model, 1.0 + 0.8 * round as f64);
            let fused = model
                .fuse(
                    &[kept],
                    &[other],
                    Rule::Union,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("axis {x} {y} {z}, round {round}: {why:?}"));
            let &[union] = fused.solids() else {
                panic!(
                    "axis {x} {y} {z}, round {round}: {} solids",
                    fused.solids().len()
                );
            };
            model
                .place_solid(
                    union,
                    &Isometry::rotation(&axis, 0.2 + 0.05 * round as f64).expect("a finite angle"),
                )
                .expect("this model made the key");
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("axis {x} {y} {z}, round {round}: {why}"));
            check((x, y, z), round, &model, &United { kept, other, union });
            kept = union;
        }
    }
    println!("{} rounds answered", AXES.len() * ROUNDS);
}

fn written_and_read(model: &Model) -> Model {
    let bytes = postcard::to_allocvec(model).expect("a model writes out");
    postcard::from_bytes(&bytes).expect("a model that validated reads back")
}

fn a_cube(model: &mut Model, shift: f64) -> SolidKey {
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(
                0.5 * shift,
                0.3 + 0.07 * shift,
                0.4 - 0.05 * shift,
            ))
            .expect("a finite shift"),
        )
        .expect("this model made the key");
    cube
}

/// Every solid here is flat, so `measure` takes no mesh and the deflection is
/// never read.
fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid measures")
        .volume()
}

fn its_planes(model: &Model) -> Vec<Plane> {
    model
        .solids()
        .flat_map(|solid| solid.faces())
        .filter_map(|face| {
            let Surface::Plane(plane) = face.surface() else {
                return None;
            };
            Some(plane)
        })
        .collect()
}

/// How many of the planes in `one` hold other bits than the one in `other`
/// in the same place.
fn planes_that_differ(one: &[Plane], other: &[Plane]) -> usize {
    let bits = |plane: &Plane| plane.coefficients().map(f64::to_bits);
    one.iter()
        .zip(other)
        .filter(|(one, other)| bits(one) != bits(other))
        .count()
}
