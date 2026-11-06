//! A plane that a fuse bakes, a placement turns and a second fuse bakes again
//! comes back from its bytes a few last digits off what the file wrote, and
//! then stays put but for three planes of one axis (Q3220, from Q2826 and
//! `docs/decisions.md` §464.37).
//!
//! A cube is united with a second cube, the answer is placed turned about a
//! slanted axis, and the next round unites it with another cube. After each
//! round the model is written with `postcard` and read back. `Deserialize`
//! takes a plane through `Plane::from_coefficients`, which scales the four
//! numbers so the first three have length one. A normal that the placements
//! turned again and again is not exactly length one, so the read moves it.
//!
//! Every one of the 24 rounds answers, and any refusal fails. Until Q3224 each
//! axis stopped at its second or third round, refused `Undecidable` for a point
//! in the new cube's floor that a fan triangle of almost no area could not
//! place, so 7 of the 24 answered.
//!
//! The first test measures how far a read moves a plane, at each of eight
//! rounds on each of three slanted axes, and holds it to
//! [`MOST_STEPS_A_READ_MOVES_A_NUMBER`]. The second shows the move is made
//! once, but for the three planes it names. The third shows each union takes
//! up what its two solids do less their overlap.

use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const ROUNDS: usize = 8;

const AXES: [(f64, f64, f64); 3] = [(1.0, 2.0, 3.0), (0.3, -0.7, 0.2), (1.0, 1.0, 1.0)];

/// The largest move of any one number of a placed plane seen over the 24
/// rounds, in [`steps_apart`]'s steps.
const MOST_STEPS_A_READ_MOVES_A_NUMBER: f64 = 3.0;

/// How far a union and its overlap may add up to other than its two solids, in
/// `f64::EPSILON`s of what the two solids take up together. The worst of the 24
/// rounds is `3.73`, at axis `(1, 1, 1)`, round 3. It is not `0` because the
/// fuse puts each new corner where three stored planes meet, rounded, and the
/// union and the overlap round theirs apart.
const MOST_A_UNION_AND_ITS_OVERLAP_ARE_OUT: f64 = 8.0;

#[test]
fn a_read_moves_a_turned_plane_by_a_few_last_digits_and_no_more() {
    let mut most = 0.0f64;
    let mut moved = 0;
    for_each_round(|axis, round, model, _| {
        let read_back = written_and_read(model);
        let (written, read) = (its_planes(model), its_planes(&read_back));
        assert_eq!(written.len(), read.len(), "{axis:?}, round {round}");
        for (written, read) in written.iter().zip(&read) {
            let steps = steps_apart(written, read);
            moved += u32::from(steps > 0.0);
            most = most.max(steps);
        }
    });
    println!("the read moved {moved} planes, a number by at most {most} steps");
    assert!(
        most <= MOST_STEPS_A_READ_MOVES_A_NUMBER,
        "a read moved a number by {most} steps"
    );
}

/// **A plane read back is not always read back the same again.**
/// `Plane::from_coefficients` scales an already scaled normal to length one,
/// and `1 053` of `100 000` such normals come back with other bits
/// (`rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal`, in
/// `rcad-geom`). Three of the planes axis `(1, 2, 3)` makes at its fourth round
/// are among them, and the model keeps them for every round after. That is
/// wrong, and a row filed from Q3224 is to make a second read move nothing.
/// Until then this pins where it moves.
#[test]
fn a_second_read_moves_three_planes_of_one_axis_and_no_other() {
    for_each_round(|axis, round, model, _| {
        let once = written_and_read(model);
        let twice = written_and_read(&once);
        let (once, twice) = (its_planes(&once), its_planes(&twice));
        let moved = once
            .iter()
            .zip(&twice)
            .filter(|(one, two)| one != two)
            .count();
        let three_from_round_3 = axis == AXES[0] && round >= 3;
        assert_eq!(
            moved,
            if three_from_round_3 { 3 } else { 0 },
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

/// The most that any of the four numbers of `one` is from the same number of
/// `other`, in steps between neighbouring `f64`s as large as one, or as large
/// as the number where it is larger.
///
/// **Not in the number's own steps.** A face's plane is its stored plane moved
/// by the solid's placement, so a number of it near zero is a sum of numbers
/// about one and is rounded in their steps, not its own. Counted in its own,
/// an offset of `-0.00093`, axis `(1, 2, 3)` round 2, that a read moved by
/// `2.8e-17`, an eighth of a step at one, moved 256.
fn steps_apart(one: &Plane, other: &Plane) -> f64 {
    let (one, other) = (one.coefficients(), other.coefficients());
    one.iter()
        .zip(&other)
        .map(|(&one, &other)| {
            let size = one.abs().max(other.abs()).max(1.0);
            (one - other).abs() / (size.next_up() - size)
        })
        .fold(0.0, f64::max)
}
