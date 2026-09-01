//! One pair of surfaces in, and §24.18's criterion 3 out.
//!
//! **A property fuzzer and not a crash hunt** (§24.14). What goes in is a pair
//! of surfaces rather than a file, and what is asserted is the acceptance oracle
//! itself: every point the evaluator produces has to be on both the surfaces
//! that named it. Roadmap §11.3 says coverage-guided fuzzing works unusually
//! well here because the branches worth reaching *are* the degeneracy handlers,
//! and that is the whole of what this module is.
//!
//! # "Mutate, do not generate" does not apply here
//!
//! §24.14 says so out loud, so that nobody applies it anyway. The warning is
//! about NURBS, where random control points make garbage no kernel handles. A
//! ball is four numbers and a pipe is seven, and every draw from that space is a
//! valid surface.
//!
//! What [`two_surfaces_from`] does with the bytes is therefore a straight
//! decoding and not a repair: four bytes make a whole number, one more makes a
//! power of two to scale it by, and the pair that comes out is refused only when
//! it names no surface at all — a radius of nothing, a direction of nothing.
//! **That reaches from `2^-640` to `2^666` in one byte**, which is what puts the
//! range refusals §28.2 measures inside a mutation's reach: a fuzzer that could
//! only draw ordinary sizes would never see the one refusal the four planar
//! pairs have.
//!
//! # This file is the one authority
//!
//! `fuzz/fuzz_targets/where_two_surfaces_cross.rs` reaches across to this file
//! rather than copying it, so the property `cargo fuzz` explores and the
//! property the suite replays are the same lines of code.

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

use rcad_geom::{
    Axis, Cylinder, Deflection, POINT_ALONG_A_CROSSING_ROUNDOFF, Plane, Point, Sphere, Surface,
    SurfaceMeet, Vector, intersect, walk,
};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
};

/// How many bytes one number takes.
const BYTES_PER_NUMBER: usize = 5;

/// The most bytes a pair of surfaces reads.
///
/// One for the kinds, then seven numbers apiece, which is what a pipe takes.
pub const MOST_BYTES_A_PAIR_READS: usize = 1 + 2 * 7 * BYTES_PER_NUMBER;

/// How far a walk in here may stray, as a share of how big the **crossing** is.
///
/// A hundredth, which is ten times coarser than anything a mesher would ask for.
/// being fuzzed is whether the points are on the surfaces, and a finer walk buys
/// more of the same points at a cost that would stop the fuzzer covering
/// anything.
///
/// **A share of the crossing and not of the two surfaces**, and that difference
/// is the whole reason this fuzzes at a useful rate. A plane a hair off parallel
/// to a pipe cuts an ellipse a hundred orders of magnitude longer than either
/// shape is wide, and a deflection taken from the shapes would ask for the
/// million points `MOST_POINTS_ALONG_ONE_CROSSING` caps at — one draw in a
/// thousand costing more than the other nine hundred and ninety-nine together.
const HOW_FINE: f64 = 1e-2;

/// Two surfaces decoded from `bytes`, or `None` where they name no pair.
///
/// # What is refused, and why that is not a repair
///
/// A radius of nothing is not a ball; a direction of nothing points nowhere; a
/// plane whose normal is all zeroes is no plane. Those are the only refusals,
/// and each of them is the surface constructors' own — nothing here nudges a
/// number to make it acceptable, because a fuzzer that is quietly corrected
/// stops exploring the thing it was pointed at.
#[must_use]
pub fn two_surfaces_from(bytes: &[u8]) -> Option<[Surface; 2]> {
    let mut reading = bytes.iter().copied();
    let kinds = reading.next()?;
    let first = one_surface_from(kinds & 0b11, &mut reading)?;
    let second = one_surface_from((kinds >> 2) & 0b11, &mut reading)?;
    Some([first, second])
}

/// One surface of the kind `which` names.
fn one_surface_from(which: u8, reading: &mut impl Iterator<Item = u8>) -> Option<Surface> {
    match which % 3 {
        0 => {
            let coefficients = [
                a_number(reading)?,
                a_number(reading)?,
                a_number(reading)?,
                a_number(reading)?,
            ];
            Plane::from_coefficients(coefficients)
                .ok()
                .map(Surface::Plane)
        }
        1 => {
            let centre = a_place(reading)?;
            Sphere::new(centre, a_number(reading)?.abs())
                .ok()
                .map(Surface::Sphere)
        }
        _ => {
            let origin = a_place(reading)?;
            let direction =
                Vector::from([a_number(reading)?, a_number(reading)?, a_number(reading)?]);
            let axis = Axis::new(origin, direction).ok()?;
            Cylinder::new(axis, a_number(reading)?.abs())
                .ok()
                .map(Surface::Cylinder)
        }
    }
}

/// Three numbers as a point.
fn a_place(reading: &mut impl Iterator<Item = u8>) -> Option<Point> {
    Some(Point::from([
        a_number(reading)?,
        a_number(reading)?,
        a_number(reading)?,
    ]))
}

/// One number: a whole one from four bytes, times a power of two from a fifth.
///
/// Always finite, and spread across thirteen hundred binary orders of magnitude.
/// The fifth byte counts *fives* of powers of two, and the whole number carries
/// thirty-one bits, so consecutive exponents overlap and nothing between the two
/// ends is out of reach.
fn a_number(reading: &mut impl Iterator<Item = u8>) -> Option<f64> {
    let whole = i32::from_le_bytes([
        reading.next()?,
        reading.next()?,
        reading.next()?,
        reading.next()?,
    ]);
    let exponent = i32::from(reading.next()? as i8) * FIVES_PER_STEP;
    Some(f64::from(whole) * 2f64.powi(exponent))
}

/// How many powers of two one step of the exponent byte is worth.
const FIVES_PER_STEP: i32 = 5;

/// **Criterion 3, asked of one pair of surfaces.**
///
/// Name where they cross; where that is a curve, walk it; and of every point
/// that comes back ask the two halves of the acceptance oracle §28.4 settled.
///
/// A pair that refuses is not a failure — the whole point of `Error` here is
/// that giving up is an answer a caller can read. What would be a failure is a
/// point handed back as being on a surface it is provably off.
///
/// Hands back how many points were actually asked about, so a caller can tell a
/// property that held from one that never ran.
///
/// # Panics
///
/// If any point of any walked run is certified on the wrong side of either
/// surface that named it.
pub fn every_point_of_every_crossing_is_on_both_surfaces(pair: [Surface; 2]) -> usize {
    let mut log = ProvenanceLog::new();
    let mut checked = 0;
    let Ok(SurfaceMeet::Curves(runs)) =
        intersect(&pair[0], &pair[1], &Policy::certified(), &mut log)
    else {
        return checked;
    };
    for run in &runs {
        // The first walk is asked for as coarsely as a walk can be asked for, so
        // it costs a handful of points and comes back with the one number this
        // harness needs: how big the crossing itself is, which the published
        // deviation is a fixed share of.
        let roughly = Deflection::of(f64::MAX / 4.0).expect("a positive length");
        let Ok(sized) = walk(
            run,
            [&pair[0], &pair[1]],
            roughly,
            &Policy::certified(),
            &mut log,
        ) else {
            continue;
        };
        let how_big = sized.deviation().magnitude() / POINT_ALONG_A_CROSSING_ROUNDOFF;
        let Ok(deflection) = Deflection::of(how_big * HOW_FINE) else {
            continue;
        };

        let Ok(along_it) = walk(
            run,
            [&pair[0], &pair[1]],
            deflection,
            &Policy::certified(),
            &mut log,
        ) else {
            continue;
        };
        let bound = along_it.deviation().magnitude();
        for point in along_it.points() {
            let Some(box_it_is_in) = somewhere_within(*point, bound) else {
                continue;
            };
            checked += 1;
            for surface in &pair {
                let decision = side_of(surface, box_it_is_in);
                assert!(
                    !matches!(decision, Decision::Pos | Decision::Neg),
                    "this point came back {decision:?} against a surface it is on.\n                       point {point:?}\n  bound {bound:e}\n  pair {pair:?}"
                );
            }
        }
    }
    checked
}

/// The box a computed point is really known to be inside, or `None` where the
/// bound is not a length.
fn somewhere_within(point: Point, bound: f64) -> Option<PredicatePoint> {
    let sides: Option<[Interval; 3]> = point
        .coordinates()
        .iter()
        .map(|coordinate| Interval::new(coordinate - bound, coordinate + bound).ok())
        .collect::<Option<Vec<_>>>()
        .and_then(|sides| <[Interval; 3]>::try_from(sides).ok());
    sides.map(PredicatePoint::Constructed)
}

fn side_of(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => flat.side_of_point(point),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        other => panic!("nothing here can ask about {other:?}"),
    }
}

/// The bytes a pair of surfaces would be decoded from, for writing a corpus out.
///
/// The exponent is the smallest step of five that keeps the whole number inside
/// thirty-one bits, so a number that is a whole one times a power of two — which
/// every seed in the corpus is — comes back bit for bit.
///
/// # Panics
///
/// If a number cannot be written down that way, which means a seed was asked for
/// that the decoder could never have produced.
#[must_use]
pub fn bytes_for(kinds: [u8; 2], numbers: &[f64]) -> Vec<u8> {
    let mut out = vec![kinds[0] | (kinds[1] << 2)];
    for number in numbers {
        let steps = if *number == 0.0 {
            0
        } else {
            ((number.abs().log2() - 31.0) / f64::from(FIVES_PER_STEP)).ceil() as i32
        };
        assert!(
            (-128..=127).contains(&steps),
            "{number:e} is past what one byte of exponent reaches"
        );
        let whole = (number / 2f64.powi(steps * FIVES_PER_STEP)).round() as i32;
        out.extend_from_slice(&whole.to_le_bytes());
        out.push(steps as u8);
    }
    out
}
