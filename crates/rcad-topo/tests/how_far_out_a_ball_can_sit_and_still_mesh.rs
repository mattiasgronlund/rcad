//! How far out a ball can sit and still mesh — and why "how far out" is the
//! wrong question.
//!
//! A downstream consumer reported that a unit ball at `1.0808639105689192e16`
//! refuses to mesh and one at `1.170935903116329e16` — *further out* — meshes,
//! and asked whether refusal is monotone in where a solid stands. It is not,
//! and the way it is not is worse than a threshold that leans the wrong way:
//! **the answer alternates between meshing and refusing at every single ulp of
//! the placement, right across a whole binade.** Measured below at 32 meshes
//! and 32 refusals over 64 consecutive placements.
//!
//! So the sentence this file exists to forbid is *"past this distance nothing
//! meshes"*. There is no such distance. There is a **band**, and inside the
//! band the last bit of the placement decides.
//!
//! # The mechanism, which is a tie and not a magnitude
//!
//! A ball of radius `r` centred at `c` reaches its poles at `c - r` and
//! `c + r`. Those are the two boundary corners the mesher walks. Out where the
//! ulp of `c` is `u`, three things can happen, and the ratio `r / u` is the
//! whole of it:
//!
//! - `r / u > 0.5` — the poles land in different ulp buckets from `c`. Two
//!   corners, and a mesh.
//! - `r / u < 0.5` — both poles round back onto `c`. The mesh lands flat,
//!   enclosing nothing, and the mesher refuses it. **This does not depend on
//!   `c` at all.**
//! - `r / u == 0.5` — the poles sit *exactly halfway* between two doubles.
//!   Round-half-to-even breaks the tie, and which way it breaks is decided by
//!   the last bit of `c`'s mantissa. Even, and both poles round *inward* onto
//!   `c`: a flat mesh, refused. Odd, and they round *outward* to `c - u` and
//!   `c + u`: a mesh.
//!
//! That third case is the alternating band, and it is not a knife edge: `u` is
//! constant across a binade, so `r / u == 0.5` holds for **every** placement in
//! one factor-of-two stretch. For a unit ball that stretch is
//! `[2^53, 2^54)` — nine quadrillion units wide — and the answer flips at each
//! of its four and a half quadrillion representable points **but the first**.
//! At `2^53` itself the ulp below is half the one above, so the near pole is not
//! a tie at all and lands where it was made, and the ball meshes half flattened
//! ([`at_the_bottom_of_the_band_the_ball_meshes_half_flattened`]).
//!
//! # The ulp of 2 was half the story
//!
//! Both reported placements sit in `[2^53, 2^54)` and so both have an ulp of
//! exactly 2. That is real and it is necessary: it is what makes a radius of 1
//! *exactly* half an ulp, which is what puts the poles on a tie at all. But it
//! is not sufficient and on its own it explains nothing, because it is equally
//! true of the placement that meshes and the placement that refuses. What
//! separates those two is
//! [`the_ulp_sets_up_the_tie_and_the_last_bit_of_the_placement_breaks_it`]:
//! `1.0808639105689192e16` has an even mantissa and `1.170935903116329e16` has
//! an odd one.
//!
//! # This is a limit and not a gap
//!
//! §16.10 asks which. Where the ball spans one ulp, every point of its surface
//! rounds into `{c - u, c, c + u}`; the shape is not representable as a mesh
//! with distinct vertices, so the answer does not exist rather than being
//! missed. The family of fixes that would otherwise apply is the exact
//! power-of-two rescale that `orient2d` and `orient3d` use — and it is the case
//! `power_of_two.rs` writes off: the coordinates are near the top of the
//! integer range while the differences that matter are one ulp of them, so
//! lifting the differences pushes the coordinates out of range and no single
//! exponent does both. A mesh is handed back in the model's own coordinates in
//! any case, so there is nowhere for a scaled copy to live.
//!
//! **The refusal is therefore right, and so is the mesh.** Nothing here is a
//! bug report. What is worth writing down is that the two are interleaved, and
//! that the thing they interleave on is not distance.
//!
//! # The same edge, and why a sweep by doubling reports it as a cliff
//!
//! `crates/rcad-geom/tests/what_the_dynamic_range_check_is_a_margin_against.rs`
//! — added on `main` in `f98ef8f`, after this branch's base — reaches the same
//! `2^53` from the other side. It puts a wedge at `c` with an edge `c + f` and
//! sweeps `f = c / 2^s`: the feature survives to `s = 52` and is gone from
//! `s = 53`. That is this file's band start, said as a spread instead of as a
//! ratio, because `f = c / 2^53` **is** half an ulp of `c`.
//!
//! **It reports a cliff, and this file reports an alternation, and both
//! measurements are right.** Its four coordinates are `1`, `2^10`, `2^20` and
//! `2^40` — every one a power of two, so every one has a mantissa of zero. All
//! four fall on the *even* side of the tie, where the feature rounds inward and
//! is indeed gone. A coordinate with an odd mantissa keeps it a whole binade
//! longer, which
//! [`a_coordinate_with_an_odd_mantissa_keeps_a_feature_a_binade_longer`]
//! measures in plain `f64` with no kernel in the way.
//!
//! Nothing there needs correcting. Its verdict is that `1e11` is an enormous
//! margin against a cliff at `9.0e15`, and one more binade of head-room on half
//! the coordinates does not touch that. What is worth keeping is only that
//! "gone above `2^53`" is a statement about even mantissas, and a consumer that
//! reads it as a distance will be surprised by a solid that meshes further out
//! than one that did not.
//!
//! **And a threshold cannot produce the reported pair at all.** Both placements
//! are above `2^53`, so a cliff there predicts both refuse; what happened is
//! that the lower one refuses and the higher one meshes. That inversion is the
//! finding, and it is the tie-break and not the threshold that makes it.
//!
//! # What is not claimed
//!
//! §16.10 scopes a verdict to the family swept. This is balls, placed along one
//! axis, meshed under [`Policy::certified()`]. The parity law is checked on the
//! `z` axis as well as `x` and holds on both, with the same refusal on both,
//! although a different coordinate collapses on each
//! ([`the_law_and_its_refusal_travel_to_another_axis`]).

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model};

/// The reported placement that refuses. Its mantissa's last bit is 0.
const EVEN: f64 = 1.080_863_910_568_919_2e16;

/// The reported placement that meshes, further out than [`EVEN`]. Its
/// mantissa's last bit is 1.
const ODD: f64 = 1.170_935_903_116_329e16;

/// Coarse enough that no deflection floor is in play at these placements, so
/// every answer below is about the geometry and not about the number asked for.
///
/// A hundred thousand units on a ball one unit across sounds absurd, and out
/// here it is the only kind of request that gets past
/// [`Error::DeflectionBelowWhatAMeshCanReach`] —
/// [`what_a_certified_caller_has_to_ask_for_out_there`] measures the floor.
const COARSE: f64 = 1e5;

/// The headline: a ball that refuses has a ball further out that meshes.
///
/// This is the whole of what the consumer saw, re-measured. It is one sentence
/// long and it is the one assertion in this file that must never be deleted:
/// while it stands, nobody can write "past this distance nothing meshes"
/// without it going red.
#[test]
fn a_ball_that_refuses_has_a_ball_further_out_that_meshes() {
    const { assert!(ODD > EVEN, "the one that meshes is the one further out") };
    assert!(
        !meshes(1.0, EVEN, COARSE),
        "a unit ball at {EVEN:e} refuses: {}",
        what_happened(1.0, EVEN, COARSE)
    );
    assert!(
        meshes(1.0, ODD, COARSE),
        "a unit ball at {ODD:e}, which is {} units further out, meshes",
        ODD - EVEN
    );
}

/// Both reported placements have an ulp of exactly 2, so the ulp is not what
/// tells them apart. The last bit of the mantissa is.
///
/// The arithmetic is done here in plain `f64`, with no kernel in the way, so
/// that the mechanism is visible without trusting anything this workspace
/// computes: at [`EVEN`] the ball's two poles are *the same double as its
/// centre*, and at [`ODD`] they are two doubles a full ulp either side.
#[test]
fn the_ulp_sets_up_the_tie_and_the_last_bit_of_the_placement_breaks_it() {
    assert_eq!(ulp_of(EVEN), 2.0, "the ulp out at {EVEN:e}");
    assert_eq!(
        ulp_of(ODD),
        2.0,
        "the ulp out at {ODD:e}, which is the same"
    );

    assert_eq!(EVEN.to_bits() % 2, 0, "{EVEN:e} has an even mantissa");
    assert_eq!(EVEN - 1.0, EVEN, "so the near pole rounds onto the centre");
    assert_eq!(EVEN + 1.0, EVEN, "and so does the far pole");

    assert_eq!(ODD.to_bits() % 2, 1, "{ODD:e} has an odd mantissa");
    assert_eq!(
        ODD - 1.0,
        ODD - 2.0,
        "so the near pole rounds a whole ulp in"
    );
    assert_eq!(ODD + 1.0, ODD + 2.0, "and the far pole a whole ulp out");
}

/// The `2^53` spread where a written feature disappears is where it disappears
/// *from an even mantissa*. An odd one keeps it to `2^54`.
///
/// The same arithmetic as the test above, done the way
/// `what_the_dynamic_range_check_is_a_margin_against.rs` on `main` sets it up —
/// a feature `c / 2^s` written into a coordinate — so that the two can be laid
/// side by side. That file sweeps `c` over `1`, `2^10`, `2^20` and `2^40`, and
/// the point here is that all four have a mantissa of zero: `c + c / 2^53` is a
/// tie that rounds inward at every one of them, so the sweep sees a clean cliff
/// at `2^53` and could not have seen anything else.
///
/// Set the last bit of `c` and the tie rounds outward instead. The feature is
/// still there at `2^53` and only gone at `2^54`. Checked at four coordinates a
/// decade apart so it is not one place on the number line, and in plain `f64`
/// with nothing of this workspace in the way.
///
/// `c / 2^53` is *just over* half an ulp when `c` is not a power of two, so on
/// its own that pair of lines would also pass under plain round-to-nearest with
/// no tie-break at all. The exact half-ulp is therefore checked as well, and
/// that one only goes the way it does because ties round to even.
///
/// **This does not move that file's verdict** — `1e11` is still four and a half
/// decades below the nearer of the two cliffs. It is only that "gone above
/// `2^53`" is about half the coordinates and not all of them, which is the same
/// fact as this file's alternating band.
#[test]
fn a_coordinate_with_an_odd_mantissa_keeps_a_feature_a_binade_longer() {
    for even in [1.0f64, 1024.0, 1_048_576.0, 1_099_511_627_776.0] {
        assert_eq!(even.to_bits() % 2, 0, "{even} is a power of two");
        let odd = f64::from_bits(even.to_bits() | 1);

        assert_eq!(
            even + even / two_to_the(53),
            even,
            "half an ulp on {even} rounds back down, so the feature is gone"
        );
        assert_ne!(
            odd + odd / two_to_the(53),
            odd,
            "the same half ulp on {odd} rounds up, so the feature is still there"
        );
        assert_eq!(
            odd + odd / two_to_the(54),
            odd,
            "a quarter ulp is under the tie and no last bit saves it"
        );

        assert_eq!(
            even + ulp_of(even) / 2.0,
            even,
            "and the tie exactly, which is the mechanism rather than the shape \
             the other file writes it in: an even mantissa rounds it inward"
        );
        assert_eq!(
            odd + ulp_of(odd) / 2.0,
            odd + ulp_of(odd),
            "and an odd one rounds the same tie outward"
        );
    }
}

/// The refusal is [`Error::PlacingItTookAwayTheSpaceItsMeshEncloses`], which
/// is the collapse above said as what it costs the mesh.
///
/// Named rather than counted, per §15.10: a different refusal appearing here
/// would mean the placement stopped mattering for the reason this file says it
/// does, and a test that only asked "did it refuse" would not notice.
///
/// **It was [`rcad_mesh::CannotTriangulate::CornersCoincide`] until
/// `docs/decisions.md` §379.** The mesher read each point of a placed ball back
/// from where it lands, and the middle of the seam came back on the axis between
/// the poles. It now reads each point where it was made, so the ball is cut into
/// a sound octahedron, and every point of it then lands on the plane
/// `x = EVEN`: a mesh that encloses nothing, refused by name.
#[test]
fn what_the_refusal_out_there_is_called() {
    assert_eq!(
        what_happened(1.0, EVEN, COARSE),
        "placing it took away the space its mesh encloses",
        "the octahedron the ball is meshed as encloses 4/3 where it was made \
         and lands flat"
    );
}

/// [`Model::validate`] and [`rcad_topo::SolidView::measure`] mesh through the
/// same call and give the same name, so none of the three calls a sound ball
/// malformed.
///
/// Read without the refusal, the flat mesh reached the other two as
/// `Invalid::VolumeIsNotPositive { volume: 0.0 }`, which says the solid's faces
/// do not point outwards (`docs/decisions.md` §333.6). The ball is the kernel's
/// own `add_ball` and they do. Both hand the mesher's refusal on unchanged:
/// `validate` retries a finer mesh only for a refusal a finer mesh may clear,
/// and this is not one.
#[test]
fn mesh_validate_and_measure_all_refuse_the_flat_ball_by_one_name() {
    let (model, ball) = a_ball_out_there(1.0, EVEN);
    let deflection = Deflection::of(COARSE).expect("a positive length");
    let view = model.solid(ball).expect("this model made the key");

    let meshed = view.mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new());
    let validated = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
    let measured = view.measure(deflection, &Policy::certified(), &mut ProvenanceLog::new());

    for (call, answer) in [
        ("mesh", meshed.map(|_| ())),
        ("validate", validated),
        ("measure", measured.map(|_| ())),
    ] {
        let Err(Error::PlacingItTookAwayTheSpaceItsMeshEncloses {
            enclosed_where_made,
            enclosed_where_placed,
        }) = answer
        else {
            panic!("{call} of a unit ball at {EVEN:e} answered {answer:?}");
        };
        assert_eq!(
            (enclosed_where_made, enclosed_where_placed),
            (4.0 / 3.0, 0.0),
            "{call}: the octahedron of a unit ball encloses 4/3 where it is made \
             and nothing where it lands"
        );
    }
}

/// Sixty-four placements one ulp apart: thirty-two mesh and thirty-two refuse,
/// strictly alternating.
///
/// This is the finding. A single threshold and a genuinely non-monotone region
/// are different things and only one of them is surprising — so the walk is
/// asserted **rung by rung against the parity rule**, not summed into a count.
/// A run of two meshes or two refusals anywhere in it fails, which is what
/// stops "it alternates" degrading into "roughly half of them work".
///
/// **The walk starts one ulp above `2^53`**, because `2^53` itself has an even
/// mantissa and meshes: the ulp below it is 1, so its near pole is no tie
/// ([`at_the_bottom_of_the_band_the_ball_meshes_half_flattened`]). Before
/// `docs/decisions.md` §379 the mesher read the ball's points back from where
/// they landed and refused it, and the walk started there.
#[test]
fn the_answer_alternates_at_every_ulp_across_the_band() {
    let two_53 = 9_007_199_254_740_992.0f64;
    assert_eq!(ulp_of(two_53), 2.0, "2^53 is where the ulp becomes 2");
    assert!(
        meshes(1.0, two_53, COARSE),
        "the bottom of the band meshes, half flattened"
    );

    let mut meshed = 0;
    let mut refused = 0;
    for step in 1..=64 {
        let out = two_53 + f64::from(step) * 2.0;
        let by_parity = out.to_bits() % 2 == 1;
        assert_eq!(
            meshes(1.0, out, COARSE),
            by_parity,
            "at {out:.1} the last bit of the mantissa is {}, so it should {}, \
             and it {}",
            out.to_bits() % 2,
            if by_parity { "mesh" } else { "refuse" },
            what_happened(1.0, out, COARSE)
        );
        if by_parity {
            meshed += 1;
        } else {
            refused += 1;
        }
    }
    assert_eq!(
        (meshed, refused),
        (32, 32),
        "strictly alternating, so half each"
    );
}

/// The band is exactly one binade wide, and outside it the placement stops
/// mattering — in opposite directions on the two sides.
///
/// Below `2^53` the ulp is 1, a unit radius is a whole ulp, and every placement
/// meshes whatever its last bit. At or above `2^54` the ulp is 4, a unit radius
/// is a quarter of one, and every placement refuses whatever its last bit. Only
/// in between is the radius *exactly* half an ulp, and only there does a tie
/// need breaking.
///
/// Four placements a side, both parities represented on each, because the claim
/// is that parity stops being read and one sample cannot say that.
#[test]
fn outside_the_band_the_last_bit_of_the_placement_stops_mattering() {
    let two_53 = 9_007_199_254_740_992.0f64;
    let two_54 = 18_014_398_509_481_984.0f64;

    for out in [two_53 - 4.0, two_53 - 3.0, two_53 - 2.0, two_53 - 1.0] {
        assert_eq!(ulp_of(out), 1.0, "just under 2^53 the ulp is 1");
        assert!(
            meshes(1.0, out, COARSE),
            "a radius of a whole ulp clears the tie at {out:.1}, whose last bit \
             is {}: {}",
            out.to_bits() % 2,
            what_happened(1.0, out, COARSE)
        );
    }

    for out in [two_54, two_54 + 4.0, two_54 + 8.0, two_54 + 12.0] {
        assert_eq!(ulp_of(out), 4.0, "from 2^54 the ulp is 4");
        assert!(
            !meshes(1.0, out, COARSE),
            "a radius of a quarter ulp is under the tie at {out:.1}, whose last \
             bit is {}, so nothing can save it",
            out.to_bits() % 2
        );
    }

    let last_in_the_band = two_54 - 2.0;
    assert_eq!(ulp_of(last_in_the_band), 2.0, "still the ulp-2 binade");
    assert_eq!(last_in_the_band.to_bits() % 2, 1, "and an odd mantissa");
    assert!(
        meshes(1.0, last_in_the_band, COARSE),
        "the furthest-out unit ball that meshes at all sits at {last_in_the_band:.1}"
    );
}

/// It is `radius / ulp` against a half, and either side of the half the last
/// bit is not consulted.
///
/// The same two placements, with the radius moved instead of the placement.
/// Nine tenths of a half-ulp refuses at both, eleven tenths meshes at both, and
/// only the exact half splits them. That is what makes this a tie and not a
/// magnitude — a story about "very large coordinates" would not have a seam
/// this thin.
#[test]
fn a_radius_either_side_of_half_an_ulp_does_not_need_the_tie_broken() {
    for out in [EVEN, ODD] {
        for radius in [0.4, 0.5, 0.9] {
            assert!(
                !meshes(radius, out, COARSE),
                "a ball of radius {radius} is under half an ulp at {out:e}, so it \
                 collapses whatever the last bit is"
            );
        }
        for radius in [1.1, 2.0, 4.0] {
            assert!(
                meshes(radius, out, COARSE),
                "a ball of radius {radius} is over half an ulp at {out:e}, so it \
                 stands whatever the last bit is"
            );
        }
    }
    assert!(!meshes(1.0, EVEN, COARSE), "and exactly a half splits them");
    assert!(meshes(1.0, ODD, COARSE));
}

/// A quarter radius at the *same* two places refuses at both — but a quarter
/// radius a quarter of the way out splits them exactly as before.
///
/// The consumer reported "same at a quarter radius" and the first half of that
/// is not what this tree does: quartering the radius alone takes `r / u` from a
/// half to an eighth, which is the region where the placement is not read at
/// all. What survives quartering is the *ratio*, so the quarter-radius twin of
/// the reported pair is at a quarter of the distance, and there the split is
/// back.
#[test]
fn a_quarter_radius_needs_a_quarter_of_the_distance_to_split_the_same_way() {
    assert!(
        !meshes(0.25, EVEN, COARSE) && !meshes(0.25, ODD, COARSE),
        "an eighth of an ulp collapses at both placements"
    );

    let even_quarter = EVEN / 4.0;
    let odd_quarter = ODD / 4.0;
    assert_eq!(
        ulp_of(even_quarter),
        0.5,
        "a quarter of the way out, ulp 0.5"
    );
    assert_eq!(ulp_of(odd_quarter), 0.5);
    assert_eq!(
        even_quarter.to_bits() % 2,
        0,
        "dividing by four keeps the bit"
    );
    assert_eq!(odd_quarter.to_bits() % 2, 1);
    assert!(
        !meshes(0.25, even_quarter, COARSE),
        "the same split, scaled"
    );
    assert!(meshes(0.25, odd_quarter, COARSE));
}

/// Every radius has a band of its own, and the alternation is in all of them.
///
/// The test above scales the reported pair and gets the split back, which is two
/// points at one radius and settles nothing on its own. This is the general
/// claim: for a radius `r` the tie is `ulp(c) == 2 * r`, and the ulp is
/// `2^(e - 52)` across `[2^e, 2^(e + 1))`, so **each radius has exactly one
/// binade where the placement is read at all** and it sits at
/// `e = 53 + log2(r)`. Quartering the radius does not weaken the tie argument;
/// it moves the band down two binades, which is why the same behaviour turns up
/// at a quarter of the distance and not at the same distance.
///
/// Sixteen consecutive placements from one ulp above the bottom of each band,
/// four radii two binades apart, asserted against parity one placement at a
/// time. The bands do not overlap: at `2^51` a unit ball meshes and a quarter
/// ball alternates, and at `2^53` it is the other way round.
///
/// **The bottom of every band meshes, whatever its last bit**, for the reason
/// [`at_the_bottom_of_the_band_the_ball_meshes_half_flattened`] measures at two
/// of them: the ulp below `2^e` is half the one above, so the near pole lands
/// where it was made. Until `docs/decisions.md` §379 the bottom was refused with
/// the rest of the even rungs, and the walk started there.
#[test]
fn each_radius_has_its_own_band_and_they_all_alternate() {
    for (radius, exponent) in [(0.25f64, 51u64), (0.5, 52), (1.0, 53), (2.0, 54)] {
        let bottom = two_to_the(exponent);
        assert_eq!(
            ulp_of(bottom),
            2.0 * radius,
            "a ball of radius {radius} is exactly half an ulp from 2^{exponent}"
        );
        assert!(
            meshes(radius, bottom, COARSE),
            "radius {radius} at the bottom of its band, 2^{exponent}: {}",
            what_happened(radius, bottom, COARSE)
        );

        for step in 1..=16 {
            let out = bottom + f64::from(step) * ulp_of(bottom);
            let by_parity = out.to_bits() % 2 == 1;
            assert_eq!(
                meshes(radius, out, COARSE),
                by_parity,
                "radius {radius} at {out:.1}, last bit {}: {}",
                out.to_bits() % 2,
                what_happened(radius, out, COARSE)
            );
        }
    }

    let two_51 = two_to_the(51);
    assert!(
        meshes(1.0, two_51, COARSE) && meshes(1.0, two_51 + 0.5, COARSE),
        "a unit ball is four ulps wide down at 2^51, so it is nowhere near a tie \
         and its last bit is not read"
    );
    let two_53 = two_to_the(53);
    assert!(
        !meshes(0.25, two_53, COARSE) && !meshes(0.25, two_53 + 2.0, COARSE),
        "and a quarter ball is an eighth of an ulp up at 2^53, which is under \
         every tie, so its last bit is not read either"
    );
}

/// The parity law holds on the `z` axis too, and so does the refusal, although
/// a different coordinate collapses.
///
/// A point of the unit ball moved out to [`EVEN`] and back has `0` in the
/// coordinate the ball was moved along, whatever it had there before.
///
/// - **Along `x`** the poles `(0, 0, ±1)` do not use that coordinate. They come
///   back where they were, but the middle of the seam between them, `(1, 0, 0)`
///   to within a rounding, comes back on the axis between the poles.
/// - **Along `z`** the poles do use it. Both come back as the centre, and the
///   seam's middle comes back at height `0` with them.
///
/// **Until `docs/decisions.md` §379 that difference decided the refusal.** The
/// mesher read each point of a placed ball back by that round trip, so along
/// `x` the triangulator found two corners in one place, and up `z` the wire read
/// as one place in the ball's own two coordinates and would not lie flat. It now
/// reads each point where it was made, cuts the ball into a sound octahedron
/// either way, and asks the whole mesh whether it still encloses space where it
/// lands — and on both axes it does not.
///
/// The three points are checked here with the kernel's own [`Isometry`].
#[test]
fn the_law_and_its_refusal_travel_to_another_axis() {
    let north = Point::new(0.0, 0.0, 1.0);
    let south = Point::new(0.0, 0.0, -1.0);
    let the_seams_middle = Point::new(1.0, 0.0, 0.0);

    let along_x = Isometry::translation(Vector::new(EVEN, 0.0, 0.0)).expect("a finite placement");
    assert_eq!(
        out_and_back(&along_x, north),
        north,
        "along x the poles stay"
    );
    assert_eq!(out_and_back(&along_x, south), south);
    assert_eq!(
        out_and_back(&along_x, the_seams_middle),
        Point::origin(),
        "and the seam's middle lands on the axis between them"
    );

    let up_z = Isometry::translation(Vector::new(0.0, 0.0, EVEN)).expect("a finite placement");
    assert_eq!(
        out_and_back(&up_z, north),
        Point::origin(),
        "up z the poles land on the centre"
    );
    assert_eq!(out_and_back(&up_z, south), Point::origin());
    assert_eq!(
        out_and_back(&up_z, the_seams_middle),
        the_seams_middle,
        "and the seam's middle stays at height 0 with them"
    );

    assert_eq!(
        what_happened(1.0, EVEN, COARSE),
        "placing it took away the space its mesh encloses",
        "along x the ball lands flat"
    );
    assert_eq!(
        along_z(EVEN),
        "placing it took away the space its mesh encloses",
        "and up z it lands flat too, refused by the same name"
    );
    assert_eq!(along_z(ODD), "meshed", "and the odd placement still meshes");
}

/// What a certified caller has to ask for out there, which is nothing like
/// `1e-3`.
///
/// The consumer's sweep was reported as running from `1e-3` to `1e300`. In this
/// tree the bottom of that range never reaches the geometry at either
/// placement: a point out at `1e16` rounds by tens of units when it is placed,
/// so `room_a_mesh_has_to_work_in` turns away anything finer with
/// [`Error::DeflectionBelowWhatAMeshCanReach`] before a triangle is cut. Both
/// placements refuse at `1e-3`, for that reason and not for the parity one.
///
/// The floors are printed rather than pinned to the digit — they are a rounding
/// distance and would move if the evaluator's own bound moved — but the decade
/// is asserted, because "tens of units" versus "thousandths" is the whole point
/// and a floor that quietly fell to `1e-3` would make this file's other tests
/// mean something different.
#[test]
fn what_a_certified_caller_has_to_ask_for_out_there() {
    for out in [EVEN, ODD] {
        let Err(Error::DeflectionBelowWhatAMeshCanReach { closest, .. }) =
            try_to_mesh(1.0, out, 1e-3)
        else {
            panic!("a thousandth of a unit is under the floor at {out:e}");
        };
        println!("the floor for a unit ball at {out:e} is {closest}");
        assert!(
            (10.0..1e3).contains(&closest),
            "the floor out at {out:e} is tens of units, not thousandths: {closest}"
        );
    }

    assert_eq!(
        what_happened(1.0, ODD, 1e-3),
        "the deflection is under the floor",
        "so the placement that meshes still refuses a thousandth"
    );
}

/// Monotone in the deflection asked for, at both placements, over the range
/// that reaches the geometry at all.
///
/// The consumer was careful to say their monotonicity-in-deflection result held
/// everywhere they measured, and it holds here: once a rung works every coarser
/// rung works. The point of measuring it again is that the two axes are
/// independent — the placement alternates and the deflection does not — and a
/// file that pinned only the first could be read as saying the mesher is ragged
/// in general. It is not.
#[test]
fn monotone_in_the_deflection_even_where_it_is_not_monotone_in_the_placement() {
    let ladder = [47.0, 50.0, 1e2, 1e3, 1e6, 1e12, 1e100, 1e300];

    let mut worked = false;
    for deflection in ladder {
        let meshed = meshes(1.0, ODD, deflection);
        assert!(
            !worked || meshed,
            "at {ODD:e} a deflection of {deflection:e} refused although a finer \
             one meshed: {}",
            what_happened(1.0, ODD, deflection)
        );
        worked |= meshed;
    }
    assert!(worked, "the coarse end of the ladder meshes at {ODD:e}");

    for deflection in ladder {
        assert!(
            !meshes(1.0, EVEN, deflection),
            "at {EVEN:e} no deflection helps, because the placement flattens \
             whatever mesh the ball is cut into"
        );
    }
}

/// The mesh that comes back at the placement that works is twice the ball
/// across, and that is inside the deflection it was asked for.
///
/// Worth measuring because a reader could take the alternation for a bug on the
/// meshing side rather than the refusing side. It is not: at these placements a
/// unit ball's poles are a full ulp — two units — either side of the centre once
/// they are rounded, so the mesh spans four units where the ball spans two. The
/// caller asked to be allowed a hundred thousand units of stray, so an answer
/// one unit off is well inside its budget and the mesher is right to hand it
/// back. This is what a limit looks like from the side that does not refuse.
#[test]
fn the_mesh_that_comes_back_out_there_is_twice_the_ball_across() {
    let (model, ball) = a_ball_out_there(1.0, ODD);
    let mesh = model
        .solid(ball)
        .expect("this model made the key")
        .mesh(
            Deflection::of(COARSE).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the odd placement meshes");

    let across = mesh
        .points()
        .iter()
        .map(|point| point.x())
        .fold(f64::NEG_INFINITY, f64::max)
        - mesh
            .points()
            .iter()
            .map(|point| point.x())
            .fold(f64::INFINITY, f64::min);
    println!("a ball 2 units across meshes {across} units across at {ODD:e}");
    assert_eq!(
        across, 4.0,
        "the poles round a whole ulp out, so the mesh is an ulp wider each side \
         than the ball it stands for"
    );
    assert!(
        across - 2.0 < COARSE,
        "and being a unit off is inside the {COARSE:e} the caller allowed"
    );
}

/// At the bottom of a band the ball meshes half flattened, and that mesh is an
/// honest one: it encloses space, it is inside the deviation it claims, and no
/// triangle of it lies on another.
///
/// At `2^53` the ulp below is 1 and the ulp above is 2, so a unit ball's near
/// side, `c − 1`, is a double and lands where it was made, while its far side
/// `c + 1` is a tie and rounds to even, onto `c`. The octahedron the ball is
/// meshed as keeps its near half and has its far corner pressed into the
/// square through its middle: a square pyramid, closed, with half the
/// octahedron's volume. A ball of radius a quarter at `2^51` is the same
/// pyramid at a quarter the size.
///
/// The three things checked are the three a mesh of a solid promises and the
/// flat mesh at [`EVEN`] broke one of (`docs/decisions.md` §379.2). It is the
/// same kind of answer as [`ODD`]'s, which is twice the ball across
/// ([`the_mesh_that_comes_back_out_there_is_twice_the_ball_across`]): inside
/// the deflection asked for, closed, and enclosing space. Distances and the
/// volume are worked out about the centre, where every point here is exact.
#[test]
fn at_the_bottom_of_the_band_the_ball_meshes_half_flattened() {
    for (radius, exponent) in [(1.0f64, 53u64), (0.25, 51)] {
        let out = two_to_the(exponent);
        let mesh = try_to_mesh(radius, out, COARSE).expect("the bottom of the band meshes");
        let about_the_centre: Vec<[f64; 3]> = mesh
            .points()
            .iter()
            .map(|point| {
                let [x, y, z] = point.coordinates();
                [x - out, y, z]
            })
            .collect();
        let triangles: Vec<[[f64; 3]; 3]> = mesh
            .triangles()
            .iter()
            .map(|triangle| {
                triangle
                    .corners()
                    .map(|corner| about_the_centre[corner as usize])
            })
            .collect();

        let reach_in_x = about_the_centre
            .iter()
            .map(|point| point[0])
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), x| {
                (low.min(x), high.max(x))
            });
        assert_eq!(
            reach_in_x,
            (-radius, 0.0),
            "radius {radius} at 2^{exponent}: the near side lands where it was made \
             and the far side on the centre"
        );

        let volume: f64 = triangles
            .iter()
            .map(|[a, b, c]| dot(*a, cross(*b, *c)) / 6.0)
            .sum();
        let the_octahedrons = 4.0 / 3.0 * radius.powi(3);
        assert_eq!(
            volume,
            the_octahedrons / 2.0,
            "radius {radius} at 2^{exponent}: half the octahedron the same deflection \
             gives where nothing rounds"
        );

        for (index, one) in triangles.iter().enumerate() {
            let normal = cross(minus(one[1], one[0]), minus(one[2], one[0]));
            assert_ne!(
                dot(normal, normal),
                0.0,
                "radius {radius} at 2^{exponent}: triangle {index} has no area"
            );
            for other in &triangles[index + 1..] {
                assert!(
                    !on_the_same_three_places(*one, *other),
                    "radius {radius} at 2^{exponent}: two triangles lie on top of each \
                     other, {one:?} and {other:?}"
                );
            }
        }

        let furthest = furthest_a_sampled_point_of_the_ball_sits_from(&triangles, radius);
        println!(
            "radius {radius} at 2^{exponent}: volume {volume} against the ball's {}, \
             furthest sampled point {furthest} against a deviation of {}",
            4.0 / 3.0 * core::f64::consts::PI * radius.powi(3),
            mesh.deviation().magnitude()
        );
        assert_eq!(
            furthest, radius,
            "radius {radius} at 2^{exponent}: the far pole sits one radius from the \
             square it was pressed into"
        );
        assert!(
            furthest <= mesh.deviation().magnitude(),
            "radius {radius} at 2^{exponent}: {furthest} off is inside the {} the mesh \
             claims",
            mesh.deviation().magnitude()
        );
    }
}

/// The furthest any of a grid of points on the ball of `radius` about the
/// origin sits from the nearest of `triangles`.
fn furthest_a_sampled_point_of_the_ball_sits_from(triangles: &[[[f64; 3]; 3]], radius: f64) -> f64 {
    let rows = 64;
    let mut furthest = 0.0f64;
    for row in 0..=rows {
        let latitude = core::f64::consts::PI * (f64::from(row) / f64::from(rows) - 0.5);
        for column in 0..2 * rows {
            let longitude = core::f64::consts::PI * f64::from(column) / f64::from(rows);
            let on_the_ball = [
                radius * latitude.cos() * longitude.cos(),
                radius * latitude.cos() * longitude.sin(),
                radius * latitude.sin(),
            ];
            let nearest = triangles
                .iter()
                .map(|triangle| how_far_from_a_triangle(on_the_ball, *triangle))
                .fold(f64::INFINITY, f64::min);
            furthest = furthest.max(nearest);
        }
    }
    furthest
}

fn how_far_from_a_triangle(point: [f64; 3], [a, b, c]: [[f64; 3]; 3]) -> f64 {
    let nearest_side = how_far_from_a_side(point, a, b)
        .min(how_far_from_a_side(point, b, c))
        .min(how_far_from_a_side(point, c, a));
    let normal = cross(minus(b, a), minus(c, a));
    let square = dot(normal, normal);
    let along_the_normal = dot(minus(point, a), normal) / square;
    let dropped = minus(point, normal.map(|component| component * along_the_normal));
    let inside = [(a, b), (b, c), (c, a)]
        .into_iter()
        .all(|(from, to)| dot(cross(minus(to, from), minus(dropped, from)), normal) >= 0.0);
    if inside {
        return length(minus(point, dropped));
    }
    nearest_side
}

fn how_far_from_a_side(point: [f64; 3], from: [f64; 3], to: [f64; 3]) -> f64 {
    let along = minus(to, from);
    let part = (dot(minus(point, from), along) / dot(along, along)).clamp(0.0, 1.0);
    length(minus(
        point,
        [0, 1, 2].map(|axis| from[axis] + part * along[axis]),
    ))
}

fn on_the_same_three_places(one: [[f64; 3]; 3], other: [[f64; 3]; 3]) -> bool {
    let sorted = |mut corners: [[f64; 3]; 3]| {
        corners.sort_by(|a, b| {
            a.iter()
                .zip(b)
                .map(|(x, y)| x.total_cmp(y))
                .find(|order| order.is_ne())
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        corners.map(|corner| corner.map(f64::to_bits))
    };
    sorted(one) == sorted(other)
}

fn minus(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|axis| a[axis] - b[axis])
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn meshes(radius: f64, out: f64, deflection: f64) -> bool {
    try_to_mesh(radius, out, deflection).is_ok()
}

fn what_happened(radius: f64, out: f64, deflection: f64) -> String {
    match try_to_mesh(radius, out, deflection) {
        Ok(_) => "meshed".to_owned(),
        Err(why) => what_it_refused(&why),
    }
}

fn along_z(out: f64) -> String {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(0.0, 0.0, out)).expect("a finite placement"),
        )
        .expect("a ball out there");
    match model.solid(ball).expect("this model made the key").mesh(
        Deflection::of(COARSE).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(_) => "meshed".to_owned(),
        Err(why) => what_it_refused(&why),
    }
}

/// `point` moved out by `placement` and back by its inverse, the way the mesher
/// read a point of its table back into the frame a placed face is worked in
/// until `docs/decisions.md` §379.
fn out_and_back(placement: &Isometry, point: Point) -> Point {
    placement
        .inverse()
        .apply_to_point(placement.apply_to_point(point))
}

fn try_to_mesh(
    radius: f64,
    out: f64,
    deflection: f64,
) -> Result<rcad_topo::Mesh, rcad_topo::Error> {
    let (model, ball) = a_ball_out_there(radius, out);
    model.solid(ball)?.mesh(
        Deflection::of(deflection).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// The ball, in a model whose working size is read off the ball and not off how
/// far away it is.
///
/// §25.3: the working size is what the shape is, and a solid placed a long way
/// out is still the size it was built. Sizing the model to the placement would
/// change what a deflection means and put a second moving part into every
/// measurement here.
fn a_ball_out_there(radius: f64, out: f64) -> (Model, rcad_topo::SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0 * radius).expect("a working size"));
    let ball = model
        .add_ball(
            radius,
            &Isometry::translation(Vector::new(out, 0.0, 0.0)).expect("a finite placement"),
        )
        .expect("a ball out there");
    (model, ball)
}

/// A refusal as a name, so a change of refusal fails rather than passing as
/// "still refused".
///
/// [`Error`] is `#[non_exhaustive]`, so the wildcard is unavoidable; the
/// variants this file has actually seen are spelled out above it, and reaching
/// the wildcard is a finding.
fn what_it_refused(why: &Error) -> String {
    match why {
        Error::DeflectionBelowWhatAMeshCanReach { .. } => {
            "the deflection is under the floor".to_owned()
        }
        Error::PlacingItTookAwayTheSpaceItsMeshEncloses { .. } => {
            "placing it took away the space its mesh encloses".to_owned()
        }
        Error::CannotMeshAFace {
            why: rcad_mesh::CannotTriangulate::CornersCoincide { .. },
            ..
        } => "the triangulator found two corners in the same place".to_owned(),
        Error::CannotMeshAFace { why, .. } => format!("the triangulator refused: {why:?}"),
        Error::AWireWillNotLieFlatOnItsFace { .. } => {
            "a wire will not lie flat on its face".to_owned()
        }
        Error::Geometry(what) => format!("the geometry layer refused: {what:?}"),
        Error::Undecidable { .. } => "a predicate could not settle a sign".to_owned(),
        other => format!("an unexpected refusal: {other:?}"),
    }
}

/// How far apart two neighbouring doubles are at `x`.
fn ulp_of(x: f64) -> f64 {
    f64::from_bits(x.to_bits() + 1) - x
}

/// `2^exponent`, built from the bits so that no rounding of its own enters a
/// measurement about rounding.
fn two_to_the(exponent: u64) -> f64 {
    assert!(exponent <= 1023, "2^{exponent} is not a normal f64");
    f64::from_bits((1023 + exponent) << 52)
}
