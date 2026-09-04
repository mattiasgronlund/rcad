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
    Axis, Cone, Cylinder, Deflection, POINT_ALONG_A_CROSSING_ROUNDOFF, Plane, Point, Result,
    Sphere, Surface, SurfaceMeet, Torus, Vector, intersect, walk,
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

/// What one pair was actually asked.
///
/// Every field counts comparisons *reached*, never draws seen. A pair that
/// refuses is not a failure here — giving up is an answer a caller can read —
/// so without these a harness could not tell an oracle that held from one that
/// skipped every draw it was given.
#[derive(Clone, Copy, Debug, Default)]
pub struct Asked {
    /// Walked points checked against both the surfaces that named them.
    pub points: usize,
    /// Pairs named both ways round where at least one order named something.
    pub orders: usize,
    /// Pairs named at two scales where both scales were decided.
    pub scalings: usize,
}

impl Asked {
    /// What two pairs were asked between them.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self {
            points: self.points + other.points,
            orders: self.orders + other.orders,
            scalings: self.scalings + other.scalings,
        }
    }
}

/// **Every oracle this harness has, asked of one pair of surfaces.**
///
/// The fuzz target and the suite's replay both call this and nothing narrower,
/// so a property added here is explored by `cargo fuzz` and replayed in CI
/// without either being edited. That is the file header's one-authority rule
/// applied to the oracles and not only to the decoder.
///
/// # Why turning is not among them
///
/// A pair turned by a rotation was the obvious third oracle and it is
/// deliberately absent. Turning changes what an exactly tangent pair crosses
/// in — legitimately, because the turned coordinates are a rounding away and
/// the surfaces really do come apart — so the property is only true away from a
/// knife edge. `the_same_pair_turned_names_the_same_kind_of_crossing.rs` has the
/// filter for that, and §50.9 records that the filter is known to miss a ball
/// tangent along a whole circle inside a pipe. Adding a turning oracle on top of
/// a guard that is known to miss cases would report that one miss on every
/// degenerate draw the corpus holds, and bury whatever else it found.
///
/// # Panics
///
/// If any oracle below fails.
pub fn every_oracle_holds_of(pair: [Surface; 2]) -> Asked {
    Asked {
        points: every_point_of_every_crossing_is_on_both_surfaces(pair),
        orders: naming_a_pair_does_not_depend_on_which_way_round_it_is_asked(&pair),
        scalings: writing_a_pair_at_another_power_of_two_does_not_change_it(&pair),
    }
}

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
        Surface::Cone(cone) => cone.side_of(point),
        Surface::Torus(ring) => ring.side_of(point),
        other => panic!("nothing here can ask about {other:?}"),
    }
}

/// **Which way round the pair is asked does not change the answer.**
///
/// `intersect(a, b)` and `intersect(b, a)` are one question, and the pair table
/// sends them down different rows to answer it.
///
/// What is compared is the *kind* of answer and not its geometry. Two correct
/// runs may describe one curve from either surface's point of view, and
/// insisting on the same bits would assert a choice rather than a fact. Every
/// refusal compares equal to every other for the same reason: which one a path
/// reaches is a property of the path.
///
/// **The sharp half is one order answering where the other gives up.** That says
/// the answer exists and the arithmetic did not reach it from one side, which is
/// §16.10's gap rather than a limit — and no amount of walking points can see
/// it, because the refusing order never produces a point to check. That is the
/// hole §50.9 sent this oracle to close.
///
/// # Panics
///
/// If the two orders disagree about what the pair crosses in.
fn naming_a_pair_does_not_depend_on_which_way_round_it_is_asked(pair: &[Surface; 2]) -> usize {
    let one_way = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let other_way = intersect(
        &pair[1],
        &pair[0],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert_eq!(
        answer_of(&one_way),
        answer_of(&other_way),
        "asking this pair the other way round changed the answer.\n  first {:?}\n  second {:?}\n  pair {pair:?}",
        one_way.as_ref().err(),
        other_way.as_ref().err(),
    );
    usize::from(one_way.is_ok() || other_way.is_ok())
}

/// What a pair crosses in, or that it was refused.
///
/// Every refusal reads the same, because which one a path reaches is a property
/// of the path and not of the pair.
fn answer_of(met: &Result<SurfaceMeet>) -> (&'static str, usize) {
    match met {
        Ok(named) => kind_of(named),
        Err(_) => ("refused", 0),
    }
}

/// What a pair crosses in, at the coarseness the two relation oracles compare.
///
/// [`SurfaceMeet`] is `#[non_exhaustive]` and this is another crate, so the
/// wildcard is forced. Every arm that exists is named above it, and the
/// wildcard panics rather than folding a new arm into an old one — a new kind of
/// crossing silently compared equal to `Nothing` is exactly how these oracles
/// would go quiet.
fn kind_of(met: &SurfaceMeet) -> (&'static str, usize) {
    match met {
        SurfaceMeet::Nothing => ("nothing", 0),
        SurfaceMeet::TouchAtOnePoint { .. } => ("one point", 0),
        SurfaceMeet::Curves(runs) => ("curves", runs.len()),
        SurfaceMeet::SameSurface => ("the same surface", 0),
        other => panic!("unhandled crossing: {other:?}"),
    }
}

/// **The same pair written at another power of two crosses in the same kind of
/// thing.**
///
/// Multiplying every coordinate by `2^k` is exact until a number leaves the
/// range, and it carries the crossing to the scaled crossing, so the kind of
/// answer has to survive it. [`exactly_scaled`] divides back and compares, which
/// refuses the scaling outright wherever it would have overflowed or rounded in
/// the subnormals — so the two runs really are the same geometry rather than a
/// near copy of it, which is the argument
/// `scaling_a_surface_must_not_change_its_answer.rs` makes for its own samples.
///
/// **A pair that refuses at either scale is skipped, and that is not a
/// weakening.** §30.5's first row is a known gap: a product leaves `f64`'s range
/// while naming, and the same shapes written back on the lattice are named.
/// Asserting across a refusal would re-report that one gap on every draw and
/// drown out whatever else is here. What is left is the sharp case — both scales
/// decided, and disagreeing.
///
/// # Panics
///
/// If the two scales disagree about what the pair crosses in.
fn writing_a_pair_at_another_power_of_two_does_not_change_it(pair: &[Surface; 2]) -> usize {
    let Ok(here) = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        return 0;
    };

    let mut compared = 0;
    for by in [-1, 1] {
        let (Some(first), Some(second)) = (scaled(&pair[0], by), scaled(&pair[1], by)) else {
            continue;
        };
        let Ok(there) = intersect(
            &first,
            &second,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) else {
            continue;
        };
        compared += 1;
        assert_eq!(
            kind_of(&here),
            kind_of(&there),
            "writing this pair at 2^{by} changed what it crosses in.\n  pair {pair:?}"
        );
    }
    compared
}

/// The same surface with every length multiplied by `2^by`, or `None` where that
/// is not exact.
///
/// **A cone's slope and a plane's normal do not scale, and every other number
/// here does.** A slope is a ratio of two lengths and a normal is a direction;
/// scaling either would tilt the surface rather than resize it, and the oracle
/// above would then be comparing two different shapes and calling their
/// agreement a property.
///
/// `Surface` is `#[non_exhaustive]` and this is another crate, so the wildcard
/// is forced. All five arms that exist are named above it, and the wildcard
/// panics: a sixth kind of surface must be scaled on purpose here, not skipped
/// quietly into a green run over a family that stopped growing (§48.2).
fn scaled(surface: &Surface, by: i32) -> Option<Surface> {
    let factor = 2f64.powi(by);
    match surface {
        Surface::Plane(flat) => {
            let [a, b, c, d] = flat.coefficients();
            Plane::from_coefficients([a, b, c, exactly_scaled(d, factor)?])
                .ok()
                .map(Surface::Plane)
        }
        Surface::Sphere(ball) => Sphere::new(
            scaled_place(ball.centre(), factor)?,
            exactly_scaled(ball.radius(), factor)?,
        )
        .ok()
        .map(Surface::Sphere),
        Surface::Cylinder(pipe) => Cylinder::new(
            scaled_axis(pipe.axis(), factor)?,
            exactly_scaled(pipe.radius(), factor)?,
        )
        .ok()
        .map(Surface::Cylinder),
        Surface::Cone(cone) => Cone::new(scaled_axis(cone.axis(), factor)?, cone.slope())
            .ok()
            .map(Surface::Cone),
        Surface::Torus(ring) => Torus::new(
            scaled_axis(ring.axis(), factor)?,
            exactly_scaled(ring.ring_radius(), factor)?,
            exactly_scaled(ring.tube_radius(), factor)?,
        )
        .ok()
        .map(Surface::Torus),
        other => panic!("nothing here can scale {other:?}"),
    }
}

/// The same axis moved to where scaling puts it, pointing the same way.
fn scaled_axis(axis: Axis, factor: f64) -> Option<Axis> {
    Axis::new(scaled_place(axis.origin(), factor)?, axis.direction()).ok()
}

fn scaled_place(point: Point, factor: f64) -> Option<Point> {
    let mut moved = [0.0; 3];
    for (out, coordinate) in moved.iter_mut().zip(point.coordinates()) {
        *out = exactly_scaled(coordinate, factor)?;
    }
    Some(Point::from(moved))
}

/// `value · factor`, or `None` where that product is not exact.
///
/// `factor` is a power of two, so dividing it back out is exact too and the
/// comparison catches every way the multiplication can lose something:
/// overflow to infinity, underflow to zero, and rounding in the subnormals.
fn exactly_scaled(value: f64, factor: f64) -> Option<f64> {
    let scaled = value * factor;
    (scaled.is_finite() && scaled / factor == value).then_some(scaled)
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
