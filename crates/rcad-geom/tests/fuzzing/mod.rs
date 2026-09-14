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
//! # This file is the one authority, and this is what it cannot draw
//!
//! `fuzz/fuzz_targets/where_two_surfaces_cross.rs` reaches across to this file
//! rather than copying it, so the property `cargo fuzz` explores and the
//! property the suite replays are the same lines of code.
//!
//! **No byte string reaches a cone or a torus through
//! [`two_surfaces_from`].** It reads two
//! bits per surface and asks `which % 3`, so a plane, a ball and a pipe are the
//! whole of what it can name. Measured over the checked-in corpus: 744 of its
//! 825 seeds decode to a pair, and those 1488 surfaces are 232 planes, 423 balls
//! and 833 pipes. Bits 4 to 7 of the kinds byte are read by nothing at all.
//!
//! **The three it does draw are not drawn evenly.** `which % 3` sends both 0 and
//! 3 to a plane, so uniform bytes ask for a plane twice as often as for a ball
//! or a pipe. Measured over the soak's 200 000 draws, of which 199 943 decode:
//! 200 003 planes, 99 925 balls, 99 958 pipes.
//!
//! # A second decoder, whose bytes cannot avoid a cone or a torus
//!
//! [`two_surfaces_with_a_cone_or_a_torus_from`] is the other half, and it is a
//! second byte format rather than a widening of the first. The 830 seeds behind
//! the paragraph above keep meaning exactly what they meant, because nothing
//! reads them differently — the new format has a corpus of its own and a fuzz
//! target of its own, `fuzz/fuzz_targets/where_a_cone_or_a_torus_crosses.rs`.
//!
//! One slot of its pair is *forced* to be a cone or a torus by the decoder and
//! not by a filter over what it drew, so there is no byte string it turns into a
//! pair without one. The other slot is free across all five kinds.
//!
//! The hand-written family in
//! `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` stays where it
//! is and keeps its own row, and holds the measurements behind leaving the first
//! layout alone. It is a table of fixed `Surface` values that no fuzzer can
//! mutate; this is a byte format that a fuzzer can.

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

/// The most bytes a pair with a cone or a torus in it reads.
///
/// One for the kinds, then eight numbers apiece, which is what a ring takes —
/// a torus on both sides being the longest thing
/// [`two_surfaces_with_a_cone_or_a_torus_from`] can be asked for.
pub const MOST_BYTES_A_CONE_OR_TORUS_PAIR_READS: usize = 1 + 2 * 8 * BYTES_PER_NUMBER;

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

/// Two surfaces decoded from `bytes`, **one of which is always a cone or a
/// torus**, or `None` where they name no pair.
///
/// # What the kinds byte says
///
/// | bits | what they pick |
/// |---|---|
/// | 0 | which slot is the forced one: 0 the first, 1 the second |
/// | 1 | what the forced slot is: 0 a cone, 1 a torus |
/// | 2 to 7 | the free slot's kind, as `(kinds >> 2) % 5` |
///
/// `0` is a plane, `1` a ball, `2` a pipe, `3` a cone, `4` a ring. The numbers
/// follow in slot order — the first slot's, then the second's — so which slot is
/// forced changes how many bytes each side eats: 4 for a plane, 4 for a ball, 7
/// for a pipe, 7 for a cone and 8 for a ring.
///
/// **Every bit of the kinds byte is read**, which is the one thing the first
/// format does not do — bits 4 to 7 of its kinds byte are read by nothing.
///
/// # The free slot is nearly, and not exactly, even
///
/// `kinds >> 2` runs over 0 to 63 and 64 is not a multiple of 5, so a uniform
/// byte asks for a plane, a ball, a pipe or a cone 13 times in 64 and for a ring
/// 12. That is an 8 % lean against the ring and it is left alone: a
/// coverage-guided fuzzer finds its way by feedback rather than by the byte
/// distribution, and squaring the residue up would cost a rejected value — a
/// byte string that decodes to nothing at all — for the whole of the gain.
/// §51.3's own note on `which % 3` is the same sentence about a 2:1 lean.
///
/// # What is refused, and why that is not a repair
///
/// The same rule the first decoder keeps: nothing here nudges a number to make
/// it acceptable. A cone's slope is taken through `.abs()` the way a ball's and
/// a pipe's radii are, because a sign there is a spelling of the same shape and
/// not a shape — but a ring's two radii are handed over as they were drawn, so
/// [`Torus::new`] refuses a tube that is as wide as its ring rather than this
/// swapping them behind the fuzzer's back.
///
/// **There is no byte string this hands back a pair without a cone or a torus
/// in it.** The forced slot is built by the cone arm or the ring arm and by
/// nothing else, so the invariant is the decoder's shape rather than a filter
/// over what it drew.
#[must_use]
pub fn two_surfaces_with_a_cone_or_a_torus_from(bytes: &[u8]) -> Option<[Surface; 2]> {
    let mut reading = bytes.iter().copied();
    let kinds = reading.next()?;
    let forced = if kinds & 0b10 == 0 { CONE } else { TORUS };
    let free = (kinds >> 2) % 5;
    let [first, second] = if kinds & 0b1 == 0 {
        [forced, free]
    } else {
        [free, forced]
    };
    Some([
        one_of_five_surfaces_from(first, &mut reading)?,
        one_of_five_surfaces_from(second, &mut reading)?,
    ])
}

/// What the second decoder's forced slot may be.
const CONE: u8 = 3;
const TORUS: u8 = 4;

/// One surface of the kind `which` names, across all five kinds.
///
/// The first three are exactly the first decoder's, asked for by the same
/// number, so the two formats read a plane, a ball and a pipe out of identical
/// bytes and only the kinds byte in front of them differs.
///
/// # Panics
///
/// If `which` is above 4. Every caller reduces modulo 5 first, so reaching that
/// is a broken invariant in here and not bad input from outside.
fn one_of_five_surfaces_from(which: u8, reading: &mut impl Iterator<Item = u8>) -> Option<Surface> {
    match which {
        0..=2 => one_surface_from(which, reading),
        CONE => {
            let axis = an_axis(reading)?;
            Cone::new(axis, a_number(reading)?.abs())
                .ok()
                .map(Surface::Cone)
        }
        TORUS => {
            let axis = an_axis(reading)?;
            Torus::new(axis, a_number(reading)?, a_number(reading)?)
                .ok()
                .map(Surface::Torus)
        }
        other => unreachable!("{other} is not one of the five kinds"),
    }
}

/// A place and a direction, six numbers, or `None` where that is no axis.
fn an_axis(reading: &mut impl Iterator<Item = u8>) -> Option<Axis> {
    let origin = a_place(reading)?;
    let direction = Vector::from([a_number(reading)?, a_number(reading)?, a_number(reading)?]);
    Axis::new(origin, direction).ok()
}

/// One surface of the kind `which % 3` names: a plane for 0 and for 3, a ball
/// for 1, a pipe for 2.
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
///
/// [`Interval::around`] and not [`Interval::new`], because `coordinate - bound`
/// rounds to nearest and nearest rounds the box *inwards*. A point far from the
/// origin walked along a small crossing is where that bites: the bound goes
/// under half an ulp of the coordinate, the side collapses, and this hands back
/// a box that does not hold the point it is a box around.
fn somewhere_within(point: Point, bound: f64) -> Option<PredicatePoint> {
    let sides: Option<[Interval; 3]> = point
        .coordinates()
        .iter()
        .map(|coordinate| Interval::around(*coordinate, bound).ok())
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
    written_as_numbers(&mut out, numbers);
    out
}

/// The bytes a pair with a cone or a torus in it would be decoded from.
///
/// `kinds` is what each slot is — 0 a plane, 1 a ball, 2 a pipe, 3 a cone, 4 a
/// ring — and the numbers follow in slot order. The forced slot is the first one
/// of the two that is a cone or a ring, which is a free choice only when both
/// are, and then the two spellings decode to the same pair anyway.
///
/// # Panics
///
/// If neither slot is a cone or a ring, which is a pair
/// [`two_surfaces_with_a_cone_or_a_torus_from`] could never hand back, or if a
/// number cannot be written down in five bytes.
#[must_use]
pub fn bytes_for_a_cone_or_torus_pair(kinds: [u8; 2], numbers: &[f64]) -> Vec<u8> {
    assert!(
        kinds.iter().all(|kind| *kind <= TORUS),
        "{kinds:?} names a kind that does not exist"
    );
    let forced_slot = kinds
        .iter()
        .position(|kind| *kind >= CONE)
        .expect("a pair with a cone or a torus in it");
    let free = kinds[1 - forced_slot];
    let mut out = vec![
        u8::try_from(forced_slot).expect("a slot is 0 or 1")
            | u8::from(kinds[forced_slot] == TORUS) << 1
            | free << 2,
    ];
    written_as_numbers(&mut out, numbers);
    out
}

/// Every number appended as the five bytes [`a_number`] reads back.
///
/// The exponent is the smallest step of five that keeps the whole number inside
/// thirty-one bits.
///
/// **`ceil` alone gets that wrong by one for every number whose `log2` is
/// `31 + 5k`**, and `2.0` is one of them. The division then lands on exactly
/// `2^31`, which is one past the top of an `i32`, and `as i32` saturates
/// silently to `2147483647` — so a radius asked for as `2.0` used to be written
/// out as `1.9999999990686774`, against a doc comment promising it came back bit
/// for bit. Nothing had ever called this with such a number, which is why the
/// promise stood. Measuring the whole number and stepping once more is the whole
/// of the fix, and it fires only where the old arithmetic had already overflowed.
///
/// # Panics
///
/// If a number cannot be written down that way, which means a seed was asked for
/// that no decoder could have produced.
fn written_as_numbers(out: &mut Vec<u8>, numbers: &[f64]) {
    for number in numbers {
        let mut steps = if *number == 0.0 {
            0
        } else {
            ((number.abs().log2() - 31.0) / f64::from(FIVES_PER_STEP)).ceil() as i32
        };
        if (number / 2f64.powi(steps * FIVES_PER_STEP)).abs() >= 2f64.powi(31) {
            steps += 1;
        }
        assert!(
            (-128..=127).contains(&steps),
            "{number:e} is past what one byte of exponent reaches"
        );
        let whole = (number / 2f64.powi(steps * FIVES_PER_STEP)).round() as i32;
        out.extend_from_slice(&whole.to_le_bytes());
        out.push(steps as u8);
    }
}
