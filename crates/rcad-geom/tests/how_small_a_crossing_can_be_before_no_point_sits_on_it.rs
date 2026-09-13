//! Two pipes whose crossing is narrower than the numbers where it sits.
//!
//! `docs/decisions.md` §94. The pair is the one behind the corpus seed
//! `fuzz/corpus/where_two_surfaces_cross/slow-unit-b9068e9ed0f147c54aec5b6b35af2f152303b761`,
//! written out here rather than decoded so that what it is can be read.
//!
//! A pipe of radius `2147482112` standing along `z` through `x = 1191182336`,
//! and a pipe of radius `4.36e-106` lying along `y` through the origin. The thin
//! one goes in one side of the fat one and out the other, so they cross in two
//! closed loops, each about as wide as the thin pipe is — `4.36e-106` — sitting
//! at `y = ±1.787e9`. The numbers there are `2.38e-7` apart. Each loop is
//! narrower than one step of the grid it lives on by ninety-nine orders of
//! magnitude, so no point that can be written down is on both pipes at once, and
//! there is nothing to walk.
//!
//! # Why this is a limit and not a gap
//!
//! §16.10 wants a demonstration rather than a paragraph, so
//! [`the_same_two_pipes_with_a_fatter_thin_one_are_walked`] is it: nothing about
//! the pair changes but the thin pipe's radius, and at `0.0625` — where the loop
//! finally spans a quarter of a million grid steps rather than `1e-99` of one —
//! the same call walks it and hands back points. The arithmetic reaches the
//! answer as soon as there is an answer to reach.
//!
//! # What it cost before
//!
//! The marcher took the seed anyway and crawled: each step landed shorter than
//! the last by a constant factor, so the walk converged on a point instead of
//! going round a loop, and the only thing that stopped it was
//! `MOST_POINTS_ALONG_ONE_CROSSING` a million points later. That was
//! **63 seconds** of every `just gate` run, for two runs of one seed, and it came
//! back [`Error::TooManyPointsAlongOneCrossing`] — which was a true sentence
//! about what the marcher did and told nobody why. §91.3 landed the seed and
//! said this was the next thing to chase.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Deflection, Error, POINT_ALONG_A_CROSSING_ROUNDOFF, Point, Surface,
    SurfaceMeet, Vector, intersect, walk,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// Where the two loops sit, and how far apart the numbers are there.
///
/// Both worked out below rather than asserted from memory, so that a reader can
/// see the one is smaller than the other.
const WHERE_THE_LOOPS_SIT: f64 = 1786830731.7039194;

/// How wide the thin pipe of the seed is, and so how wide each loop is.
const THE_SEEDS_THIN_RADIUS: f64 = 4.3601508761683463e-106;

#[test]
fn a_loop_narrower_than_the_numbers_where_it_sits_is_refused_by_name() {
    let apart = WHERE_THE_LOOPS_SIT.next_up() - WHERE_THE_LOOPS_SIT;
    assert!(
        THE_SEEDS_THIN_RADIUS < apart,
        "this test is about a loop narrower than the grid it sits on, and this one is \
         {THE_SEEDS_THIN_RADIUS:e} wide against a grid {apart:e} across"
    );

    let pair = two_pipes_crossing_in_a_loop(THE_SEEDS_THIN_RADIUS);
    let runs = the_runs_of(&pair);
    assert_eq!(
        runs.len(),
        2,
        "the naming half has to reach the marcher for this test to be about the marcher"
    );

    for run in &runs {
        let refusal = walk_it(&pair, run, f64::MAX / 4.0).expect_err("nothing here can be walked");
        assert!(
            matches!(
                refusal,
                Error::ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces
            ),
            "expected the crossing to be turned away at its seed, and got {refusal:?}"
        );
    }
}

/// The same pair, one number different, and it walks.
///
/// §16.10's demonstration that the refusal above is a limit: the loop is refused
/// because it is smaller than the numbers where it sits, and a loop that is not
/// is walked by the same code with nothing else changed.
#[test]
fn the_same_two_pipes_with_a_fatter_thin_one_are_walked() {
    let mut narrowest_that_walked = f64::INFINITY;
    let mut worst_residual = 0.0_f64;

    for radius in [0.0625, 0.125, 1.0] {
        let apart = WHERE_THE_LOOPS_SIT.next_up() - WHERE_THE_LOOPS_SIT;
        assert!(
            radius > apart,
            "a loop of {radius:e} is not wider than the {apart:e} grid it sits on"
        );

        let pair = two_pipes_crossing_in_a_loop(radius);
        let runs = the_runs_of(&pair);
        assert_eq!(runs.len(), 2, "the same two loops, one on each side");

        for run in &runs {
            let (points, deviation) = points_along(&pair, run);
            assert!(
                !points.is_empty(),
                "a loop {radius:e} wide came back with no points"
            );
            narrowest_that_walked = narrowest_that_walked.min(radius);

            // The closed form: every point of this crossing is on the thin pipe,
            // whose axis is the y axis, and on the fat one. `deviation` is what
            // the walk publishes about how far off the crossing a point may sit,
            // and a point that far off the crossing is at most that far off
            // either surface that makes it.
            for point in &points {
                for surface in &pair {
                    let residual = how_far_off(surface, *point).abs();
                    assert!(
                        residual <= deviation,
                        "a point came back {residual:e} off a surface the walk said it was \
                         within {deviation:e} of"
                    );
                    worst_residual = worst_residual.max(residual);
                }
            }
        }
    }

    assert!(
        narrowest_that_walked.is_finite(),
        "no radius walked, so this demonstrated nothing"
    );
    assert!(
        worst_residual > 0.0,
        "every point sat exactly on both surfaces, so the published bound was never asked \
         for anything"
    );
}

/// The seed's fat pipe, and a thin one across it of the radius asked for.
fn two_pipes_crossing_in_a_loop(thin_radius: f64) -> [Surface; 2] {
    let fat = Cylinder::new(
        Axis::new(
            Point::from([1191182336.0, 0.0, 0.0]),
            Vector::from([7.234461236871035e-292, 1.2701238075670642e-101, 1.0]),
        )
        .expect("a direction"),
        2147482112.0,
    )
    .expect("a radius");
    let thin = Cylinder::new(
        Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([0.0, 1.0, 0.0]))
            .expect("a direction"),
        thin_radius,
    )
    .expect("a radius");
    [Surface::Cylinder(fat), Surface::Cylinder(thin)]
}

fn the_runs_of(pair: &[Surface; 2]) -> Vec<rcad_geom::NamedRun> {
    let mut log = ProvenanceLog::new();
    match intersect(&pair[0], &pair[1], &Policy::certified(), &mut log) {
        Ok(SurfaceMeet::Curves(runs)) => runs,
        other => panic!("these two pipes cross in curves, and this said {other:?}"),
    }
}

fn walk_it(
    pair: &[Surface; 2],
    run: &rcad_geom::NamedRun,
    room: f64,
) -> rcad_geom::Result<rcad_geom::PointsAlongACrossing> {
    let mut log = ProvenanceLog::new();
    let deflection = Deflection::of(room).expect("a positive length");
    walk(
        run,
        [&pair[0], &pair[1]],
        deflection,
        &Policy::certified(),
        &mut log,
    )
}

/// One run walked the way the fuzz harness walks one: coarsely first, to find
/// out how big the crossing is, then as a fixed share of that.
fn points_along(pair: &[Surface; 2], run: &rcad_geom::NamedRun) -> (Vec<Point>, f64) {
    let sized = walk_it(pair, run, f64::MAX / 4.0).expect("a loop this wide is walked");
    let how_big = sized.deviation().magnitude() / POINT_ALONG_A_CROSSING_ROUNDOFF;
    let along = walk_it(pair, run, how_big * 1e-2).expect("the same loop, more finely");
    (along.points().to_vec(), along.deviation().magnitude())
}

/// How far `at` sits off `surface`, signed, worked out here rather than asked of
/// the crate, so that the check is against the closed form and not against the
/// arithmetic under test.
fn how_far_off(surface: &Surface, at: Point) -> f64 {
    let Surface::Cylinder(pipe) = surface else {
        panic!("this file is about two pipes");
    };
    let along = pipe.axis().direction();
    let out = at - pipe.axis().origin();
    let sideways = out - along * (out.dot(along) / along.length_squared());
    sideways.length() - pipe.radius()
}
