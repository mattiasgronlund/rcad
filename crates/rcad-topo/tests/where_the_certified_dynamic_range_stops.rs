//! Exactly which solids a certified policy refuses for spreading too wide, to
//! the last bit.
//!
//! [`DynamicRange::LARGEST_SUPPORTED_RATIO`] is a published bound — `1e11`,
//! written out in its own doc comment and in `docs/decisions.md` §3.5 — and
//! §15.6 says a published bound wants a test. This is that test, applied to a
//! refusal boundary rather than to an error bound.
//!
//! # Why this one is worth pinning to the bit
//!
//! A downstream consumer put it plainly: a truncated cone's rim, on a real
//! ruling, is swallowed by rscene's own near-apex ball and shaded 58.1° wrong,
//! and it needs a *tolerant* caller because rcad's certified range check
//! refuses that solid outright. The defect there is theirs. What is ours is the
//! consequence: **the range check is the only thing between a certified caller
//! and a wrong normal, and nothing in that tree would notice if it moved.** A
//! check no consumer can observe moving is a check that will move.
//!
//! Widening is the dangerous direction — a wider range hands a certified caller
//! the very solid that shades wrong — so both directions are pinned, and pinned
//! at the edge rather than in the comfortable middle.
//!
//! # What was already pinned, and what was not
//!
//! `scale.rs`'s own unit tests accept a spread of `1e11` and refuse one of
//! `1e12`. Everything between is free: the bound could be moved to `5e11` —
//! five times the spread handed to a certified caller — with the whole suite
//! green. `crates/rcad/tests/coordinates_that_span_too_wide_a_range.rs` is
//! wider still, accepting a spread of about one and refusing one of about
//! `1e12`. Neither says where the frontier actually is, and neither goes near a
//! cone.
//!
//! # The oracle
//!
//! [`THE_PUBLISHED_SPREAD`] is a second copy of the bound, written out here on
//! purpose. Asserting against `DynamicRange::LARGEST_SUPPORTED_RATIO` would
//! move with it and pin nothing at all.
//!
//! The frontier itself is found rather than assumed: bisect on the rim radius
//! of a frustum between one the kernel accepts and one it refuses until the two
//! are neighbouring `f64`, then read the spread the kernel itself reported for
//! the refused one. Two adjacent `f64` are a relative `2⁻⁵²` apart and the
//! spread is one division away from the rim, so a frontier in the published
//! place puts that spread a couple of ulps above the bound and no further.
//!
//! # The door this knocks on
//!
//! [`SolidBuilder::build`] describes the solid into a scratch model and runs
//! [`Model::validate`] over it, so a refusal here *is* the sweep's refusal —
//! this is the whole sweep and not one function, and it is the door an importer
//! comes through. The frustum is described face by face rather than borrowed
//! from `tests/cones/mod.rs`, which builds under a hardwired
//! [`Policy::certified`] and turns the refusal this test is about into a panic.

use rcad_geom::{
    Axis, Cone, DynamicRange, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog, Severity};
use rcad_topo::{DraftStep, Error, Invalid, Model, Result, SolidBuilder, SolidKey};

/// The widest spread a certified caller is allowed, copied out of
/// [`DynamicRange::LARGEST_SUPPORTED_RATIO`]'s doc comment.
///
/// The second copy is the point. If the constant moves, this file goes red
/// rather than following it.
const THE_PUBLISHED_SPREAD: f64 = 1e11;

/// The bound the code holds and the bound the doc comment publishes are the
/// same number.
#[test]
fn the_widest_spread_the_kernel_supports_is_a_hundred_billion() {
    assert_eq!(DynamicRange::LARGEST_SUPPORTED_RATIO, THE_PUBLISHED_SPREAD);
}

/// The bound is inclusive, and the first spread past it is refused — one ulp
/// past it, not a factor of ten.
#[test]
fn a_spread_of_exactly_the_published_bound_is_the_last_one_accepted() {
    let at_the_bound = DynamicRange::measure(THE_PUBLISHED_SPREAD, 1.0)
        .expect("a finite magnitude over a positive feature");
    assert_eq!(at_the_bound.ratio(), THE_PUBLISHED_SPREAD);
    assert!(
        at_the_bound.is_within_supported_range(),
        "the widest supported spread is supported"
    );

    let one_bit_past = DynamicRange::measure(THE_PUBLISHED_SPREAD.next_up(), 1.0)
        .expect("a finite magnitude over a positive feature");
    assert!(
        one_bit_past.ratio() > THE_PUBLISHED_SPREAD,
        "the next f64 up is a wider spread and nothing has rounded it back"
    );
    assert!(
        !one_bit_past.is_within_supported_range(),
        "one bit past the bound is past the bound"
    );
}

/// The rim a certified truncated cone stops being allowed at sits on the
/// published bound, to a couple of ulps.
///
/// This is the rscene shape and the rscene question: how thin may a truncated
/// cone's rim get before the kernel refuses the solid rather than handing it to
/// a certified caller? The frustum's coordinates reach
/// [`HOW_FAR_UP_THE_FAR_CAP_SITS`] and its smallest feature is the near cap's
/// circle, so its spread is that height over that radius, and thinning the rim
/// walks it across the frontier.
#[test]
fn the_thinnest_rim_a_certified_frustum_may_have_is_the_published_bound() {
    assert_eq!(
        Policy::certified().dynamic_range_check(),
        Severity::Fail,
        "a certified caller is who this boundary protects"
    );

    let mut accepted = A_RIM_WELL_INSIDE_THE_BOUND;
    let mut refused = A_RIM_WELL_PAST_THE_BOUND;
    assert!(
        the_spread_a_frustum_with_this_rim_is_refused_for(accepted).is_none(),
        "a rim of {accepted} has to be accepted for this to be a bracket"
    );
    assert!(
        the_spread_a_frustum_with_this_rim_is_refused_for(refused).is_some(),
        "a rim of {refused} has to be refused for this to be a bracket"
    );

    while accepted.to_bits() - refused.to_bits() > 1 {
        let between = f64::from_bits(refused.to_bits().midpoint(accepted.to_bits()));
        if the_spread_a_frustum_with_this_rim_is_refused_for(between).is_some() {
            refused = between;
        } else {
            accepted = between;
        }
    }

    let spread = the_spread_a_frustum_with_this_rim_is_refused_for(refused)
        .expect("the refused side of a closed bracket is still refused");
    println!(
        "the frontier is between rims {refused} and {accepted}; the first refused spread is \
         {spread} against a published {THE_PUBLISHED_SPREAD}, over by {} ulps of the bound",
        (spread - THE_PUBLISHED_SPREAD) / (THE_PUBLISHED_SPREAD.next_up() - THE_PUBLISHED_SPREAD)
    );

    assert!(
        spread > THE_PUBLISHED_SPREAD,
        "a refusal at a spread the bound allows is the kernel refusing what it published as \
         supported"
    );
    // Neighbouring `f64` are a relative `2⁻⁵²` apart and the spread is one
    // division away from the rim, so the first refused spread cannot clear the
    // bound by more than a few of those unless the frontier has moved.
    let no_further_than = THE_PUBLISHED_SPREAD * (1.0 + 8.0 * f64::EPSILON);
    assert!(
        spread <= no_further_than,
        "the first spread a certified build refuses is {spread}, past the {no_further_than} that \
         is one rim's worth of rounding above the published {THE_PUBLISHED_SPREAD} — the frontier \
         has moved"
    );
}

/// The frustum one bit inside the frontier is a solid a certified caller can go
/// on to use, and it needed nothing approximated.
///
/// Without this the test above would still pass if the accepted side were
/// accepted by the range check and then refused by something else, which is a
/// different frontier wearing this one's clothes.
#[test]
fn the_last_rim_the_bound_allows_builds_a_solid_that_certifies_clean() {
    let rim = THE_THINNEST_RIM_THE_BOUND_ALLOWS;
    let mut model = Model::new(a_working_size());
    let mut log = ProvenanceLog::new();
    let solid = a_frustum_with_this_rim(&mut model, rim, &Policy::certified(), &mut log)
        .expect("a rim on the bound is a rim the bound allows");
    assert!(log.is_empty(), "certified outcomes are not recorded");

    model
        .validate(&Policy::certified(), &mut log)
        .expect("and the sweep says the same about it a second time");
    assert!(log.is_empty(), "still nothing approximated");
    assert_eq!(model.solid(solid).faces().count(), 3, "two caps and a wall");

    assert!(
        the_spread_a_frustum_with_this_rim_is_refused_for(rim.next_down()).is_some(),
        "and one bit thinner is refused, which is what makes {rim} the last one"
    );
}

/// How far up the axis the frustum's wide end sits, which is also its largest
/// coordinate.
const HOW_FAR_UP_THE_FAR_CAP_SITS: f64 = 8.0;

/// Out per unit up. A power of two, so `slope * height` is exact and every
/// corner is certified on its own cone — `tests/cones/mod.rs` says why that
/// matters.
const SLOPE: f64 = 0.5;

/// A rim a hundred times thicker than the frontier, to open the bracket with.
const A_RIM_WELL_INSIDE_THE_BOUND: f64 = 1e-8;

/// A rim eighty times thinner than the frontier, to close it with.
const A_RIM_WELL_PAST_THE_BOUND: f64 = 1e-12;

/// [`HOW_FAR_UP_THE_FAR_CAP_SITS`] over [`THE_PUBLISHED_SPREAD`], which is the
/// thinnest rim the published bound leaves room for.
///
/// Both are powers of two times a small integer, so the division is exact and
/// this is the frontier itself rather than a number near it.
const THE_THINNEST_RIM_THE_BOUND_ALLOWS: f64 = HOW_FAR_UP_THE_FAR_CAP_SITS / THE_PUBLISHED_SPREAD;

/// The spread a certified build refused a frustum of this rim radius for, or
/// `None` if it accepted the solid.
///
/// # Panics
///
/// If the build refuses for any other reason. Every frustum here is a sound
/// solid, so a refusal about anything but its spread would mean the bisection
/// is walking a different frontier.
fn the_spread_a_frustum_with_this_rim_is_refused_for(rim: f64) -> Option<f64> {
    let mut model = Model::new(a_working_size());
    match a_frustum_with_this_rim(
        &mut model,
        rim,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(_) => None,
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => Some(ratio),
        Err(other) => panic!("a frustum with a rim of {rim} is a sound solid, but {other:?}"),
    }
}

fn a_working_size() -> Scale {
    Scale::millimetres(HOW_FAR_UP_THE_FAR_CAP_SITS).expect("eight millimetres is a working size")
}

/// A frustum about `+z` whose near cap circle has radius `rim`, described face
/// by face and built under `policy`.
///
/// The same shape `tests/cones/mod.rs` builds, and the same way, with the one
/// difference this file needs: the refusal comes back as an [`Err`] instead of
/// being `expect`ed away, because the refusal is what is being measured.
///
/// # Errors
///
/// Whatever [`SolidBuilder::build`] makes of the description, which for a rim
/// this thin is [`Invalid::DynamicRangeTooWide`].
fn a_frustum_with_this_rim(
    model: &mut Model,
    rim: f64,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    let near = rim / SLOPE;
    let far = HOW_FAR_UP_THE_FAR_CAP_SITS;
    assert!(0.0 < near && near < far, "a frustum has two different ends");
    assert_eq!(SLOPE * near, rim, "the rim asked for is the rim described");

    let skin = Cone::new(Axis::z(), SLOPE).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let bottom_corner = builder
        .add_vertex(Point::new(rim, 0.0, near))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(SLOPE * far, 0.0, far))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cone is straight, which is the whole of what a seam here is");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the cap plane cuts the cone in one closed curve");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the cap plane cuts the cone in one closed curve");

    let cap = |builder: &mut SolidBuilder, at: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder
            .add_coedge(edge, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap is a plane trimmed by its circle")
    };
    let bottom = cap(
        &mut builder,
        near,
        -1.0,
        bottom_circle,
        Orientation::Reversed,
    );
    let top = cap(&mut builder, far, 1.0, top_circle, Orientation::Forward);

    let round_the_wall: Vec<DraftStep> = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round the bottom, up the seam, back round the top, down the seam");
    let side = builder
        .add_face(Surface::Cone(skin), &[wire])
        .expect("a face on the cone");

    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder.build(model, policy, log)
}
