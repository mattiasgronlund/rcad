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
//! # What `what_the_dynamic_range_check_is_a_margin_against.rs` covers
//!
//! That file landed the same day, after the same rscene sentence, and answers
//! the *other* question: **why** the number is `1e11`. It is a margin, and the
//! thresholds it is a margin against are measured there — a written feature
//! surviving to `2^52`, a constructed point giving up at `2^49`, and the
//! conditioning law that explains the gap. None of that is repeated here.
//!
//! What it holds the constant to are three inequalities against those
//! thresholds, and the tightest of them — a conditioning figure required to
//! land in `1e-6..1e-4` — leaves the bound free to sit anywhere between about
//! `4.5e9` and `4.5e11`. Measured: with the constant moved to `2e11`, twice the
//! spread handed to a certified caller, all three of its tests pass.
//!
//! It is also predicates from end to end. Nothing in it builds a solid, names a
//! [`Policy`], or calls `check`, `check_under_policy` or
//! `is_within_supported_range`, so nothing in it can see the accept/refuse edge
//! move. Measured: drop the `Surface::Cone` arm of `Model::check_dynamic_range`
//! — so that a certified sweep stops seeing a truncated cone's rim at all,
//! which is the rscene exposure exactly — and every other test in the suite
//! stays green.
//!
//! The older pins are looser still. `scale.rs`'s own unit tests accept a spread
//! of `1e11` and refuse one of `1e12`, and
//! `crates/rcad/tests/coordinates_that_span_too_wide_a_range.rs` accepts a
//! spread of about one and refuses one of about `1e12`. Neither says where the
//! frontier is, and neither goes near a cone.
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
//! # The frontier is in the same place whatever the coordinates round like
//!
//! A power of two has no mantissa, so a boundary decided by rounding can look
//! sharp at one and be blunt everywhere else; a sweep that samples only tidy
//! binary values reports a cliff because the other answer was never in it. Two
//! measurements today had that shape, so this one is taken at four heights —
//! `2^3`, `2^20`, `τ` and `e·2^18` — and the frontier lands within two ulps of
//! the published bound at all four. Rounding does not move it, and that is a
//! finding rather than an assumption.
//!
//! # The door this knocks on
//!
//! `SolidBuilder::build` describes the solid into a scratch model and runs
//! [`Model::validate`] over it, so a refusal here *is* the sweep's refusal —
//! this is the whole sweep and not one function, and it is the door an importer
//! comes through. The frustum is `tests/cones/mod.rs`'s, taken through
//! `a_frustum_under` and not `a_frustum`: the second hardwires
//! [`Policy::certified`] and `expect`s the build, which would turn the refusal
//! this test is about into a panic.

mod cones;

use rcad_geom::{DynamicRange, Scale};
use rcad_predicates::{Policy, ProvenanceLog, Severity};
use rcad_topo::{Error, Invalid, Model, Result, SolidKey};

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
/// a certified caller? The frustum's coordinates reach its own height and its
/// smallest feature is the near cap's circle, so its spread is that height over
/// that radius, and thinning the rim walks it across the frontier.
///
/// # Both kinds of coordinate, on purpose
///
/// "Just inside" and "just outside" are decided by rounding, and a power of two
/// has no mantissa to round. A frontier sampled only at tidy binary heights
/// could be sharp for those and blunt everywhere else, and pass forever while
/// the real edge moved. [`TALL_COORDINATES`] therefore carries two heights with
/// zero mantissa and two with every mantissa bit in use, and the frontier is
/// found separately at each.
///
/// # What it goes red for, measured
///
/// The bound at `2e11` reports a first refused spread of `200000000000.00003`
/// and the bound at `1e10` reports one of `10000000000.000002`, both against a
/// published `1e11`. Dropping the `Surface::Cone` arm of
/// `Model::check_dynamic_range` leaves no rim thin enough to be refused and the
/// opening bracket never closes.
#[test]
fn the_thinnest_rim_a_certified_frustum_may_have_is_the_published_bound() {
    assert_eq!(
        Policy::certified().dynamic_range_check(),
        Severity::Fail,
        "a certified caller is who this boundary protects"
    );

    for (what_kind_of_number, far) in TALL_COORDINATES {
        let (refused, accepted) = the_rims_either_side_of_the_frontier(far);

        let spread = the_spread_a_frustum_is_refused_for(far, refused)
            .expect("the refused side of a closed bracket is still refused");
        println!(
            "a frustum {far} tall, {what_kind_of_number}: the frontier is between rims {refused} \
             and {accepted}, and the first refused spread is {spread} against a published \
             {THE_PUBLISHED_SPREAD}"
        );

        assert!(
            spread > THE_PUBLISHED_SPREAD,
            "a frustum {far} tall is refused at a spread of {spread}, which the bound allows — the \
             kernel is refusing what it published as supported"
        );
        // Neighbouring `f64` are a relative `2⁻⁵²` apart and the spread is one
        // division away from the rim, so the first refused spread cannot clear
        // the bound by more than a few of those unless the frontier has moved.
        let no_further_than = THE_PUBLISHED_SPREAD * (1.0 + 8.0 * f64::EPSILON);
        assert!(
            spread <= no_further_than,
            "a frustum {far} tall is first refused at a spread of {spread}, past the \
             {no_further_than} that is one rim's worth of rounding above the published \
             {THE_PUBLISHED_SPREAD} — the frontier has moved"
        );

        the_frustum_on_the_accepted_side_is_a_solid_that_certifies_clean(far, accepted);
    }
}

/// How far up the axis the frustum's wide end sits, which is also its largest
/// coordinate, and what kind of number that is.
///
/// Two with a zero mantissa and two with every bit of one in use. τ and e
/// scaled by a power of two, so the mantissa is full and the exponent is still
/// somewhere ordinary.
const TALL_COORDINATES: [(&str, f64); 4] = [
    ("two to the third", 8.0),
    ("two to the twentieth", 1048576.0),
    ("τ, a full mantissa", core::f64::consts::TAU),
    (
        "e times 2^18, a full mantissa",
        core::f64::consts::E * 262144.0,
    ),
];

/// Out per unit up. A power of two, so `slope * height` is exact whatever the
/// height's mantissa is and every corner is certified on its own cone —
/// `tests/cones/mod.rs` says why that matters. Keeping this tidy is what lets
/// the *height* be untidy without changing which frontier is being walked.
const SLOPE: f64 = 0.5;

/// The two neighbouring rim radii the accept/refuse frontier lies between, for
/// a frustum `far` tall: the thickest rim refused, then the thinnest accepted.
///
/// # Panics
///
/// If the opening bracket does not straddle the frontier, which means the
/// bisection would be walking something else.
fn the_rims_either_side_of_the_frontier(far: f64) -> (f64, f64) {
    let mut accepted = far / 1e9;
    let mut refused = far / 1e13;
    assert!(
        the_spread_a_frustum_is_refused_for(far, accepted).is_none(),
        "a frustum {far} tall with a rim of {accepted} has to be accepted for this to be a bracket"
    );
    assert!(
        the_spread_a_frustum_is_refused_for(far, refused).is_some(),
        "a frustum {far} tall with a rim of {refused} has to be refused for this to be a bracket"
    );

    while accepted.to_bits() - refused.to_bits() > 1 {
        let between = f64::from_bits(refused.to_bits().midpoint(accepted.to_bits()));
        if the_spread_a_frustum_is_refused_for(far, between).is_some() {
            refused = between;
        } else {
            accepted = between;
        }
    }
    (refused, accepted)
}

/// The spread a certified build refused a frustum of this rim radius for, or
/// `None` if it accepted the solid.
///
/// # Panics
///
/// If the build refuses for any other reason. Every frustum here is a sound
/// solid, so a refusal about anything but its spread would mean the bisection
/// is walking a different frontier.
fn the_spread_a_frustum_is_refused_for(far: f64, rim: f64) -> Option<f64> {
    let mut model = Model::new(a_working_size(far));
    match a_frustum_with_this_rim(
        &mut model,
        far,
        rim,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(_) => None,
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => Some(ratio),
        Err(other) => panic!("a frustum {far} tall with a rim of {rim} is a solid, but {other:?}"),
    }
}

/// The frustum on the accepted side of the frontier is one a certified caller
/// can go on to use, and it needed nothing approximated.
///
/// Without this the frontier would still be found if the accepted side were let
/// through the range check and then refused by something else, which is a
/// different frontier wearing this one's clothes.
fn the_frustum_on_the_accepted_side_is_a_solid_that_certifies_clean(far: f64, rim: f64) {
    let mut model = Model::new(a_working_size(far));
    let mut log = ProvenanceLog::new();
    let solid = a_frustum_with_this_rim(&mut model, far, rim, &Policy::certified(), &mut log)
        .expect("the accepted side of the frontier is accepted");
    assert!(log.is_empty(), "certified outcomes are not recorded");

    model
        .validate(&Policy::certified(), &mut log)
        .expect("and the sweep says the same about it a second time");
    assert!(log.is_empty(), "still nothing approximated");
    assert_eq!(model.solid(solid).faces().count(), 3, "two caps and a wall");
}

fn a_working_size(far: f64) -> Scale {
    Scale::millimetres(far).expect("a frustum's height is a positive length")
}

/// A frustum about `+z` whose near cap circle has radius `rim`, built under
/// `policy`.
///
/// The shape is `tests/cones/mod.rs`'s, described the same way; all this adds is
/// the height that puts the near cap's circle at `rim`, and the check that it
/// really did.
///
/// # Errors
///
/// Whatever `SolidBuilder::build` makes of the description, which for a rim this
/// thin is [`Invalid::DynamicRangeTooWide`]. It comes back as an [`Err`] rather
/// than being `expect`ed away, because the refusal is what is being measured.
///
/// # Panics
///
/// If the rim asked for is not the rim described. The bisection walks the rim
/// and reads the frontier off the value it asked for, so a rim that rounded on
/// the way into the cap's height would report a frontier belonging to a
/// different shape.
fn a_frustum_with_this_rim(
    model: &mut Model,
    far: f64,
    rim: f64,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    let near = rim / SLOPE;
    assert_eq!(SLOPE * near, rim, "the rim asked for is the rim described");
    cones::a_frustum_under(model, SLOPE, near, far, policy, log)
}
