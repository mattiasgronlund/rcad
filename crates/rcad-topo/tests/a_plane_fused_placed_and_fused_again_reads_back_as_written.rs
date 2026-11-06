//! A plane that a fuse bakes, a placement turns and a second fuse bakes again
//! comes back from its bytes a few last digits off what the file wrote, and
//! then stays put (Q3220, from Q2826 and `docs/decisions.md` §464.37).
//!
//! A cube is united with a second cube, the answer is placed turned about a
//! slanted axis, and the next round unites it with another cube. After each
//! round the model is written with `postcard` and read back. `Deserialize`
//! takes a plane through `Plane::from_coefficients`, which scales the four
//! numbers so the first three have length one. A normal that the placements
//! turned again and again is not exactly length one, so the read moves it.
//!
//! Each axis stops at the first round whose fuse is refused `Undecidable`: the
//! second or third round on all three, so 7 of the 24 rounds answer. The read
//! is measured on those that do.
//!
//! The first test measures how far, in steps between neighbouring `f64`s, at
//! each of eight rounds on each of three slanted axes, and holds it to
//! [`MOST_STEPS_A_READ_MOVES_A_NUMBER`]. The second shows the move is made
//! once: a model read back and written again reads back bit for bit.

use rcad_geom::{Axis, Isometry, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

const ROUNDS: usize = 8;

/// The largest move of any one stored number seen over the 24 rounds.
const MOST_STEPS_A_READ_MOVES_A_NUMBER: u64 = 4;

const AXES: [(f64, f64, f64); 3] = [(1.0, 2.0, 3.0), (0.3, -0.7, 0.2), (1.0, 1.0, 1.0)];

#[test]
fn a_read_moves_a_turned_plane_by_a_few_last_digits_and_no_more() {
    let mut most = 0;
    let mut moved = 0;
    for_each_round(|axis, round, model| {
        let read_back = written_and_read(model);
        let (written, read) = (its_planes(model), its_planes(&read_back));
        assert_eq!(written.len(), read.len(), "{axis:?}, round {round}");
        for (written, read) in written.iter().zip(&read) {
            let steps = steps_apart(written, read);
            moved += u32::from(steps > 0);
            most = most.max(steps);
        }
    });
    println!("the read moved {moved} planes, a number by at most {most} steps");
    assert!(
        most <= MOST_STEPS_A_READ_MOVES_A_NUMBER,
        "a read moved a number by {most} steps"
    );
}

#[test]
fn a_model_read_back_and_written_again_reads_back_as_written() {
    for_each_round(|axis, round, model| {
        let once = written_and_read(model);
        let twice = written_and_read(&once);
        assert_eq!(
            its_planes(&once),
            its_planes(&twice),
            "{axis:?}, round {round}: a plane moved on the second read"
        );
    });
}

fn for_each_round(mut check: impl FnMut((f64, f64, f64), usize, &Model)) {
    let mut rounds_answered = 0;
    for (x, y, z) in AXES {
        let axis =
            Axis::new(Point::new(0.5, -0.25, 0.75), Vector::new(x, y, z)).expect("a slanted axis");
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let mut kept = a_cube(&mut model, 0.0);
        for round in 0..ROUNDS {
            let other = a_cube(&mut model, 1.0 + 0.8 * round as f64);
            let fused = match model.fuse(
                &[kept],
                &[other],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) {
                Ok(fused) => fused,
                Err(Error::Undecidable { .. }) => {
                    println!("axis {x} {y} {z}: round {round} is refused Undecidable");
                    break;
                }
                Err(why) => panic!("axis {x} {y} {z}, round {round}: {why:?}"),
            };
            rounds_answered += 1;
            let [one] = fused.solids() else {
                panic!(
                    "axis {x} {y} {z}, round {round}: {} solids",
                    fused.solids().len()
                );
            };
            kept = *one;
            model
                .place_solid(
                    kept,
                    &Isometry::rotation(&axis, 0.2 + 0.05 * round as f64).expect("a finite angle"),
                )
                .expect("this model made the key");
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("axis {x} {y} {z}, round {round}: {why}"));
            check((x, y, z), round, &model);
        }
    }
    println!(
        "{rounds_answered} of {} rounds answered",
        AXES.len() * ROUNDS
    );
    assert!(
        rounds_answered >= AXES.len(),
        "{rounds_answered} rounds answered"
    );
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

fn its_planes(model: &Model) -> Vec<Plane> {
    model
        .solids()
        .flat_map(|solid| solid.faces())
        .filter_map(|face| match face.surface() {
            Surface::Plane(plane) => Some(plane),
            _ => None,
        })
        .collect()
}

/// The most steps between neighbouring `f64`s that any of the four stored
/// numbers of `one` is from the same number of `other`.
fn steps_apart(one: &Plane, other: &Plane) -> u64 {
    let (one, other) = (one.coefficients(), other.coefficients());
    one.iter()
        .zip(&other)
        .map(|(one, other)| {
            let (one, other) = (ordered(*one), ordered(*other));
            one.abs_diff(other)
        })
        .max()
        .expect("four numbers")
}

/// `number`'s bits as a count that rises with the number, so that two
/// neighbours differ by one.
fn ordered(number: f64) -> i64 {
    let bits = number.to_bits() as i64;
    if bits < 0 { i64::MIN - bits } else { bits }
}
