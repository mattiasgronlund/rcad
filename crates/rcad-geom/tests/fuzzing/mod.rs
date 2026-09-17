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
/// **[`Plane::moved_to`] and not [`Plane::from_coefficients`], for the reason
/// [`scaled_axis`] gives one function down.** Handing a stored normal back to
/// `from_coefficients` reduces and normalizes an already-normalized vector and
/// lands a bit away for most directions, so the plane that came out of here was
/// not the plane that went in written larger — it was that plane leaning very
/// slightly differently.
///
/// The nights of 2026-09-16 and 2026-09-17 both crashed on that, with the same
/// axis direction both times: a cone of slope `7.8e28` with its apex at
/// `3.6e103` against a plane square to its axis, which is one circle. The
/// rebuilt plane leaned `2.8e-17` off the axis, and a cone's own side leans only
/// `1/slope` — `1.3e-29` — so the rebuilt plane really was the steeper of the
/// two and really did cut both nappes. `intersect` named one circle for the pair
/// and a hyperbola in two branches for the rebuild, and it was right about both.
/// `fuzz/corpus/where_a_cone_or_a_torus_crosses` carries the two seeds, and
/// `rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal` counts how
/// often the rebuild moves a normal at all.
///
/// `Surface` is `#[non_exhaustive]` and this is another crate, so the wildcard
/// is forced. All five arms that exist are named above it, and the wildcard
/// panics: a sixth kind of surface must be scaled on purpose here, not skipped
/// quietly into a green run over a family that stopped growing (§48.2).
fn scaled(surface: &Surface, by: i32) -> Option<Surface> {
    let factor = 2f64.powi(by);
    match surface {
        Surface::Plane(flat) => flat
            .moved_to(exactly_scaled(flat.offset(), factor)?)
            .ok()
            .map(Surface::Plane),
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
///
/// **[`Axis::moved_to`] and not [`Axis::new`], which would tilt it.** Handing a
/// stored direction back to `Axis::new` scales an already-scaled vector and
/// lands a bit away for a quarter of all directions, so the pair that came out
/// of here was not the pair that went in written larger — it was that pair with
/// one axis leaning very slightly differently. The oracle above then reports a
/// scaling that changed the answer when what changed the answer was the tilt.
///
/// The night of 2026-09-14 crashed on that: two leaning pipes whose first sign
/// question is `1/√(a²+c²) − 1` for the axis direction's `a` and `c`, which the
/// rebuild carried from `−1.87e-17` to `+1.31e-16`. One spelling crossed in a
/// closed loop and the other in nothing, and the kernel was right about both.
/// `scaled(pair, 2⁰)` — a factor of one, no scaling at all — already flipped it.
fn scaled_axis(axis: Axis, factor: f64) -> Option<Axis> {
    axis.moved_to(scaled_place(axis.origin(), factor)?).ok()
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
/// [`written_as_numbers`] picks the exponents, and **every number
/// [`two_surfaces_from`] can hand back comes back out of here bit for bit** —
/// which is what lets the explainer print a line that rebuilds the seed it read.
///
/// # Panics
///
/// If a number is outside what five bytes reach at all, which means a seed was
/// asked for that the decoder could never have produced.
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
/// thirty-one bits, or the smallest step one byte can spell, whichever is the
/// larger.
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
/// # Wanting the largest whole number runs off both ends of the byte
///
/// That want is a preference and never a requirement: dropping a step divides
/// the whole number by thirty-two and names the very same number, so wherever
/// the largest spelling is past the end of the byte a smaller one is not. The
/// clamp is what picks it up, and refusing was a gap and not a limit (§16.10).
///
/// It fired on a seed already in the corpus. `9.332114885078756e-187`, out of
/// `fuzz/corpus/where_two_surfaces_cross/slow-unit-b9068e9ed0f147c54aec5b6b35af2f152303b761`,
/// wants a step of `-129` for a whole number of `136247104`; at `-128`, the
/// smallest step there is, it is `4257722` and reads back bit for bit. So `just
/// fuzz-triage` died every morning on the one case it was pointed at. The far
/// end does the same: `i32::MIN` with an exponent byte of `127` is
/// `-3.061802069160839e+200`, which wants a step of `128` and is spelled at
/// `127` as exactly `i32::MIN`.
///
/// **Every number [`a_number`] can hand back survives the clamp exactly**, and
/// not by luck. Such a number is `whole · 2^(5s)` for some `s` the byte already
/// holds, so scaling it to any other step in range multiplies it by a power of
/// two and lands on a whole number again — `2^-640` is the smallest of them and
/// nothing goes subnormal on the way. The test at the foot of this file walks
/// all two hundred and fifty-six exponent bytes.
///
/// # Panics
///
/// If a number is outside what five bytes reach at all — wider than `2^666`,
/// smaller than half of `2^-640`, infinite, or not a number — which means a seed
/// was asked for that no decoder could have produced. The check is on the whole
/// number the clamped step leaves, at both ends: too big and it will not fit in
/// thirty-one bits, too small and it rounds to nothing. So the clamp can never
/// turn a number this cannot spell into a quietly wrong one.
///
/// **Infinity gets to that check only because the step is clamped before
/// anything multiplies it by five.** `inf.log2()` is `inf`, so the cast
/// saturates the wanted step to `i32::MAX`, and `i32::MAX · 5` overflows — which
/// used to panic naming the arithmetic rather than the number, and only in a
/// build that checks its arithmetic at all. Clamped first, the step is `127`,
/// `inf / 2^635` is still `inf`, and infinity is refused for being too wide to
/// spell like any other number that is.
fn written_as_numbers(out: &mut Vec<u8>, numbers: &[f64]) {
    for number in numbers {
        let wanted = if *number == 0.0 {
            0
        } else {
            ((number.abs().log2() - 31.0) / f64::from(FIVES_PER_STEP)).ceil() as i32
        };
        let mut steps = wanted.clamp(i32::from(i8::MIN), i32::from(i8::MAX));
        if (number / 2f64.powi(steps * FIVES_PER_STEP)).abs() >= 2f64.powi(31) {
            steps += 1;
        }
        let steps = steps.clamp(i32::from(i8::MIN), i32::from(i8::MAX));
        let whole = (number / 2f64.powi(steps * FIVES_PER_STEP)).round();
        let spellable =
            (-2f64.powi(31)..2f64.powi(31)).contains(&whole) && (whole != 0.0) == (*number != 0.0);
        assert!(spellable, "{number:e} is past what five bytes reach");
        out.extend_from_slice(&(whole as i32).to_le_bytes());
        out.push(steps as u8);
    }
}

/// What [`written_as_numbers`] can spell, and what it is right to refuse.
///
/// Nothing else in the suite reaches this writer with a number the decoder made
/// — the corpus replays go the other way, bytes first — which is how a seed that
/// stopped `just fuzz-triage` dead sat in `fuzz/corpus` without a red test.
#[cfg(test)]
mod what_the_writer_can_spell {
    use super::{FIVES_PER_STEP, a_number, written_as_numbers};

    /// The number [`a_number`] reads out of the bytes [`written_as_numbers`] writes.
    fn written_and_read_back(number: f64) -> f64 {
        let mut bytes = Vec::new();
        written_as_numbers(&mut bytes, &[number]);
        a_number(&mut bytes.into_iter()).expect("five bytes back out")
    }

    #[test]
    fn every_number_the_decoder_can_hand_back_comes_back_bit_for_bit() {
        for whole in [0, 1, -1, 3, 12_345, i32::MAX, i32::MIN] {
            for byte in i8::MIN..=i8::MAX {
                let number = f64::from(whole) * 2f64.powi(i32::from(byte) * FIVES_PER_STEP);
                assert_eq!(
                    written_and_read_back(number).to_bits(),
                    number.to_bits(),
                    "{whole} times two to the five times {byte}, which is {number:e}"
                );
            }
        }
    }

    /// **One of a class of 21, not a one-off.** Running the explainer over all
    /// 4709 seeds in `fuzz/corpus` on `b013ac4` killed 65 of them, every one in
    /// `where_two_surfaces_cross` and every one on this assert, over 21 distinct
    /// numbers between `-2.191809349008403e-193` and `-1.4708981359843996e-185`
    /// — signed, so the rounding is walked on both sides here. The first of the
    /// two below is the seed `just fuzz-triage` opens every morning,
    /// `slow-unit-b9068e9e…`; it wants a step of `-129` and one byte stops at
    /// `-128`, where the same number is a whole `4257722` and not `136247104`.
    #[test]
    fn a_number_too_small_to_want_a_step_the_byte_holds_still_writes_down() {
        for number in [
            9.332114885078756e-187,
            -2.191809349008403e-193,
            -1.4708981359843996e-185,
            1.3914044109375144e-188,
            1.6514376035707827e-186,
            5.611031933461512e-191,
            3.5584042315626215e-186,
        ] {
            assert_eq!(
                written_and_read_back(number).to_bits(),
                number.to_bits(),
                "{number:e}"
            );
        }
    }

    /// The other end of the same want: `i32::MIN` at the largest exponent byte
    /// asks for a step of `128`, and is spelled at `127` exactly as it went in.
    #[test]
    fn the_widest_number_the_decoder_can_hand_back_still_writes_down() {
        let number = f64::from(i32::MIN) * 2f64.powi(127 * FIVES_PER_STEP);
        assert_eq!(written_and_read_back(number).to_bits(), number.to_bits());
    }

    #[test]
    #[should_panic(expected = "is past what five bytes reach")]
    fn a_number_wider_than_five_bytes_reach_is_still_a_broken_invariant() {
        written_as_numbers(&mut Vec::new(), &[1e300]);
    }

    #[test]
    #[should_panic(expected = "is past what five bytes reach")]
    fn a_number_finer_than_five_bytes_reach_is_still_a_broken_invariant() {
        written_as_numbers(&mut Vec::new(), &[2f64.powi(-700)]);
    }

    /// The one number that went from written down wrong to refused. `NaN as
    /// i32` is `0` in Rust, so the old writer measured no steps, compared
    /// nothing against the top of an `i32` — every comparison with a `NaN` is
    /// false — and put a plain zero in the repro line. The seed the explainer
    /// printed was then a different seed from the one it had read.
    #[test]
    #[should_panic(expected = "NaN is past what five bytes reach")]
    fn something_that_is_not_a_number_is_refused_instead_of_written_down_as_zero() {
        written_as_numbers(&mut Vec::new(), &[f64::NAN]);
    }

    /// `inf.log2()` is `inf`, so the saturating cast leaves the wanted step at
    /// `i32::MAX`. The clamp brings it inside a byte before anything multiplies
    /// it by five, so infinity reaches the same check every other number does
    /// and is refused for the same reason: `inf / 2^635` is still `inf`, which
    /// does not fit in thirty-one bits.
    #[test]
    #[should_panic(expected = "inf is past what five bytes reach")]
    fn an_infinite_number_is_refused_by_the_check_on_the_whole_number() {
        written_as_numbers(&mut Vec::new(), &[f64::INFINITY]);
    }

    #[test]
    #[should_panic(expected = "-inf is past what five bytes reach")]
    fn an_infinite_number_the_other_way_is_refused_the_same_way() {
        written_as_numbers(&mut Vec::new(), &[f64::NEG_INFINITY]);
    }

    /// **The one thing moving the clamp had to not do.** The bump exists to
    /// undo a `ceil` that is one short for every number whose `log2` is
    /// `31 + 5k`, and bumping a step that has *already* been clamped could walk
    /// back out of the byte — so this holds the writer to producing the very
    /// same five bytes the old one did, for every number the old one spelled,
    /// and to refusing exactly the numbers it refused.
    ///
    /// It passes for a reason and not by luck. Wherever the wanted step is
    /// inside the byte the two are the same code. Above the top the old one
    /// clamped to `127` whatever the bump did, and the new one bumps `127` to
    /// `128` and clamps back to `127`. Below the bottom the new bump cannot
    /// fire at all: a wanted step of `-129` or less means `log2` at most
    /// `-614`, and such a number divided by `2^-640` is at most `2^26`, which
    /// is nowhere near the `2^31` the bump asks about.
    #[test]
    fn no_number_changed_its_spelling_when_the_clamp_moved_ahead_of_the_bump() {
        let (mut spelled, mut refused) = (0, 0);
        for number in every_number_worth_trying() {
            let now = spelled_now(number);
            assert_eq!(now, spelled_before_the_clamp_moved(number), "{number:e}");
            if now.is_some() {
                spelled += 1;
            } else {
                refused += 1;
            }
        }
        assert!(
            spelled > 20_000 && refused > 20_000,
            "agreeing on {spelled} spelled and {refused} refused is not enough of either"
        );
    }

    /// The numbers to hold the two writers against each other on.
    ///
    /// Deliberate first: the seeds that made the clamp necessary, both ends of
    /// the exponent byte, every one of the 256 bytes crossed with whole numbers
    /// that reach both ends of an `i32`, the powers of two where `ceil` is one
    /// short, and the magnitudes either side of what five bytes reach. Then a
    /// fixed-seed walk over raw bit patterns, which is what gets the subnormals
    /// and the far exponents nothing was chosen for.
    fn every_number_worth_trying() -> Vec<f64> {
        let mut numbers = vec![
            0.0,
            -0.0,
            2.0,
            9.332114885078756e-187,
            -2.191809349008403e-193,
            -1.4708981359843996e-185,
            1.3914044109375144e-188,
            1.6514376035707827e-186,
            5.611031933461512e-191,
            3.5584042315626215e-186,
            1e300,
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
        ];
        for whole in [
            0,
            1,
            -1,
            3,
            -7,
            12_345,
            -12_345,
            1 << 30,
            i32::MAX,
            i32::MAX - 1,
            i32::MIN,
            i32::MIN + 1,
        ] {
            for byte in i8::MIN..=i8::MAX {
                numbers.push(f64::from(whole) * 2f64.powi(i32::from(byte) * FIVES_PER_STEP));
            }
        }
        for power in -1070..=1023 {
            let number = 2f64.powi(power);
            numbers.extend_from_slice(&[number, -number, number * 1.5, number * 0.75]);
        }

        let mut bits = 0x2545_f491_4f6c_dd1d_u64;
        for _ in 0..100_000 {
            bits ^= bits << 13;
            bits ^= bits >> 7;
            bits ^= bits << 17;
            numbers.push(f64::from_bits(bits));
        }
        numbers.retain(|number| number.is_finite());
        numbers
    }

    /// The five bytes the writer spells a number as, or `None` where it refuses.
    fn spelled_now(number: f64) -> Option<[u8; 5]> {
        let loud = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let written = std::panic::catch_unwind(|| {
            let mut bytes = Vec::new();
            written_as_numbers(&mut bytes, &[number]);
            bytes
        });
        std::panic::set_hook(loud);
        written
            .ok()
            .map(|bytes| bytes.try_into().expect("five bytes"))
    }

    /// The same, spelled the way it was before the clamp moved ahead of the bump.
    ///
    /// A copy of the old arithmetic rather than a reconstruction of it, with the
    /// assert turned into the `None`. It is safe to run here on anything finite:
    /// the widest finite `f64` wants a step near 199 and the smallest subnormal
    /// one near `-221`, so the multiply the old code did before its clamp only
    /// overflows on infinity, which [`every_number_worth_trying`] leaves out.
    fn spelled_before_the_clamp_moved(number: f64) -> Option<[u8; 5]> {
        let mut steps = if number == 0.0 {
            0
        } else {
            ((number.abs().log2() - 31.0) / f64::from(FIVES_PER_STEP)).ceil() as i32
        };
        if (number / 2f64.powi(steps * FIVES_PER_STEP)).abs() >= 2f64.powi(31) {
            steps += 1;
        }
        let steps = steps.clamp(i32::from(i8::MIN), i32::from(i8::MAX));
        let whole = (number / 2f64.powi(steps * FIVES_PER_STEP)).round();
        let spellable =
            (-2f64.powi(31)..2f64.powi(31)).contains(&whole) && (whole != 0.0) == (number != 0.0);
        spellable.then(|| {
            let mut bytes = [0; 5];
            bytes[..4].copy_from_slice(&(whole as i32).to_le_bytes());
            bytes[4] = steps as u8;
            bytes
        })
    }
}
