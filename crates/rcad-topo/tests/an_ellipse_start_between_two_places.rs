//! A round bar cut by a tilted block, whose top crosses the bar's wall in an
//! ellipse that is walked from a start between two places the block's side cuts
//! it at: the arm that puts the one ahead of the start first, reached on an
//! ellipse (`docs/decisions.md` §244.8).
//!
//! # The shape
//!
//! A round bar of radius `1 + 2⁻⁵¹` up `z`, its axis through `(−16, 0, 0)`, and a
//! block on the square `x ∈ [−17, −13]`, `y ∈ [−6, 6]`, 4 tall, moved down 2 and
//! turned `−0.5` about `x`. The turn leaves `x` alone, so the block's side is
//! exactly `x = −17`, and its top leans across the bar and cuts the wall in an
//! ellipse.
//!
//! An ellipse where a plane cuts a pipe is walked from an end of its short axis:
//! the centre, where the axis meets the plane, plus the bar's radius along the
//! way square to both the axis and the plane's normal. Here that way is `−x`, so
//! the start is `−16 − (1 + 2⁻⁵¹)`, which rounds to `−17`: on the block's side,
//! `2⁻⁵¹` inside the bar. The side cuts the ellipse at two places `y ≈ ±2⁻²⁵`,
//! one either side of the start.
//!
//! # Why it reaches the arm
//!
//! The start and both places have `x = −17` exactly and the top's normal has no
//! `x`, so seen along that normal the three are on one line and sweep exactly
//! nothing. Neither place's box holds the start and the two boxes are apart, so
//! `the_one_the_start_is` in `fuse/arrange.rs` answers from its `(false, false)`
//! arm, which asks `the_one_ahead_of_the_start`. Neither place is the one the
//! start is a rounding off, since the step to each has a sign along the way the
//! ellipse runs, so `the_start_is_a_rounding_off` puts neither first before
//! that.
//!
//! # What the arm reads, and what it is checked against
//!
//! The arm puts first the place whose step from the start has a positive dot
//! with `∇S × ∇T` at the start. The row asked for two checks: the sweep from the
//! start to a third point across the ellipse, and `det(n, from − c, P − c)` with
//! `c` the ellipse's centre. On an ellipse the second is not the arm's reading
//! everywhere, but at an end of the short axis it is, and here it is bit for bit.

use rcad_geom::{
    Axis, Deflection, DynamicRange, Isometry, MeetOfThree, Point, Scale, Surface, SurfaceMeet,
    Vector, intersect, meet_of_three_surfaces, where_a_closed_crossing_starts,
    which_comes_first_round_a_crossing,
};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Fused, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// The bar's radius, `1 + 2⁻⁵¹`.
const RADIUS: f64 = 1.0 + 2.0 * f64::EPSILON;

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The input the arm needs, with either surface first: the start is exactly
/// `(−17, 0, c_z)`, the end of the short axis on the side `x = −17`, with `c`
/// the centre worked out as `where_a_plane_cuts_a_pipe` works it out. The two
/// places have `x = −17` exactly and `y` boxes either side of the start's `0`,
/// so neither holds the start and the two are apart, and the sweep through all
/// three is exactly zero.
#[test]
fn the_tops_ellipse_starts_on_the_side_between_the_two_places_the_side_cuts_it_at() {
    let [top, side, wall] = the_top_the_side_and_the_wall();
    let centre = where_the_axis_meets_the_top(&top, &wall);
    assert_eq!(centre[..2], [-16.0, 0.0]);
    let [below, above] = the_two_places(&top, &side, &wall);
    for place in [below, above] {
        let x = place.enclosure()[0];
        assert_eq!([x.lo(), x.hi()], [-17.0, -17.0], "{place:?}");
    }
    assert!(below.enclosure()[1].hi() < 0.0, "{below:?}");
    assert!(above.enclosure()[1].lo() > 0.0, "{above:?}");
    for pair in [[&top, &wall], [&wall, &top]] {
        let start = the_walked_start(pair);
        assert_eq!(start, [-17.0, 0.0, centre[2]], "{pair:?}");
        assert_eq!(
            which_comes_first_round_a_crossing(pair, PredicatePoint::Exact(start), below, above),
            Ok(Decision::Zero),
            "{pair:?}"
        );
    }
}

/// What the arm reads, worked out here with its arithmetic, and checked against
/// the sweep to a third point and against the centre.
///
/// The arm's run is `n × ∇wall` at the start, top first, where `∇wall` is
/// `|w|²(x − o) − (w·(x − o))w` as `gradient_at` has it. Every step of that is
/// exact here, so the order the arithmetic is done in cannot move it, and it is
/// the exact `(−1, 0, 0)`. The step to the place below `y = 0` then reads `Pos`
/// and the one above reads `Neg`, so the arm puts the place below first.
///
/// `det(n, from − c, P − c)` is `(n × (from − c)) · (P − c)`, and since the
/// start is an end of the short axis, `∇wall` there is `|w|²(from − c)`: here
/// `n × (from − c)` is the run bit for bit, and the det gives the same signs.
///
/// The third point is `(−15, 0, c_z)`, the other end of the short axis and
/// where the bar's seam crosses the top, give or take a rounding. Top first,
/// the sweep puts the place below before it and it before the place above, so
/// going round the three come below, across, above, and the arm's order is
/// the only one of the two that agrees. With the wall first every sign turns
/// over and the order runs the other way.
#[test]
fn the_one_ahead_of_the_start_comes_before_the_point_across_the_ellipse_and_the_other_after_it() {
    let [top, side, wall] = the_top_the_side_and_the_wall();
    let Surface::Plane(flat) = top else {
        panic!("the top is a plane: {top:?}")
    };
    let normal = flat.normal().components().map(Interval::exactly);
    let centre = where_the_axis_meets_the_top(&top, &wall);
    let [below, above] = the_two_places(&top, &side, &wall);
    let start = the_walked_start([&top, &wall]);

    let gradient = the_walls_gradient_at(&wall, start);
    assert_eq!(gradient, [-1.0, 0.0, 0.0].map(Interval::exactly));
    let runs = cross(normal, gradient);
    let off_the_centre = [0, 1, 2].map(|axis| Interval::exactly(start[axis] - centre[axis]));
    assert_eq!(runs, cross(normal, off_the_centre));

    let step_to = |place: PredicatePoint| {
        [0, 1, 2].map(|axis| place.enclosure()[axis] - Interval::exactly(start[axis]))
    };
    let from_the_centre = |place: PredicatePoint| {
        [0, 1, 2].map(|axis| place.enclosure()[axis] - Interval::exactly(centre[axis]))
    };
    let turned_over = cross(gradient, normal);
    for (place, top_first, wall_first) in [
        (below, Decision::Pos, Decision::Neg),
        (above, Decision::Neg, Decision::Pos),
    ] {
        assert_eq!(dot(step_to(place), runs).sign(), top_first, "{place:?}");
        assert_eq!(
            dot(cross(normal, off_the_centre), from_the_centre(place)).sign(),
            top_first,
            "{place:?}"
        );
        assert_eq!(
            dot(step_to(place), turned_over).sign(),
            wall_first,
            "{place:?}"
        );
    }

    let across = PredicatePoint::Exact([-15.0, 0.0, centre[2]]);
    let from = PredicatePoint::Exact(start);
    for (pair, first) in [
        ([&top, &wall], Decision::Pos),
        ([&wall, &top], Decision::Neg),
    ] {
        assert_eq!(
            which_comes_first_round_a_crossing(pair, from, below, across),
            Ok(first),
            "{pair:?}"
        );
        assert_eq!(
            which_comes_first_round_a_crossing(pair, from, across, above),
            Ok(first),
            "{pair:?}"
        );
    }
}

/// The pin: under either policy, every rule with either solid first gets past
/// the order of the two places without a record of it. The intersection and
/// the block less the bar answer one solid that validates and holds its closed
/// form, with nothing recorded at all. The union and the bar less the block
/// keep the fin the bar leaves outside `x = −17`, which is too thin to hold
/// beside coordinates of 17: under `certified` each is refused
/// `DynamicRangeTooWide`, and under `tolerant` each carries one record, of the
/// range check, and answers one solid that holds its closed form.
///
/// **The closed forms.** The block's side `x = −17` is `2⁻⁵¹` inside the bar's
/// wall, its other three sides are clear of the bar, and its bottom is under
/// the bar's bottom everywhere over the bar's disc. So the two share the part
/// of the bar under the block's top. Over the disc the top stands at
/// `h ∓ y tan ½`, with `h = 4 / cos ½ − 2` where the axis meets it, between
/// `2.01` and `3.10` and so inside the bar's `10`. The `y` term sums to nothing
/// over a disc centred on `y = 0`, so what they share is `π r² (4 / cos ½ − 2)`,
/// `8.0361` with `r = 1 + 2⁻⁵¹`. The block holds `4 · 12 · 4 = 192` and the bar
/// `10 π r²`, so the union is `192 + 10 π r²` less what they share, `215.380`,
/// and the block less the bar is `192` less it, `183.964`. The fin of the bar
/// outside `x = −17` holds about `1.7e-22`, far under any of their error bars,
/// so the closed forms leave it out.
///
/// **The fin.** The block's side cuts the bar's bottom cap along a chord
/// `2⁻²⁴` long, `2⁻²⁵` either side of `y = 0`, and the piece of the cap
/// outside the block, which the union and the bar less the block both keep, is
/// the short arc beyond that chord: `2⁻⁵¹` thick. That is under a rounding of
/// coordinates of 17, past §3.5's `10¹⁶`, and `the_narrowest_a_face_is` in
/// `validate.rs` reads a face thinner than a rounding as one rounding, so the
/// spread comes out `17 / (17 · 2⁻⁵²)`, `2⁵²` exactly (queue rows Q2237 and
/// Q2257). The tolerant union validates under `tolerant` alone, and is refused
/// the same under `certified` (§68).
///
/// **The bar less the block**, `10 π r²` less what they share, `23.380`, and
/// the fin, answers one solid under `tolerant` that validates under `tolerant`
/// alone. The fin's wall and the block's side are drawn on the same three
/// points, back to back, and the sum `Model::validate` and `measure` read
/// keeps such a pair (`docs/decisions.md` §466). Until Q2790 `measure` meshed
/// with `SolidView::mesh`, which refuses the pair, and was refused
/// `TwoPatchesOfTheMeshDrewTheSameSide`. Until Q2774 the fuse was refused
/// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` at the last rung
/// of the fold ladder (Q1869), which was Q1972's refusal and then Q2262's.
///
/// **Why this says the arm gave an order.** Every fuse here goes through
/// `the_one_ahead_of_the_start`, and goes there from `the_one_the_start_is`'s
/// `(false, false)` arm: a run of each of the twelve fuses under `gdb` with no
/// change to the source counted six hits of each function and six of the arm,
/// none of the `(true, true)` arm that answers `Undecidable`, and none of the
/// other place `the_one_ahead_of_the_start` is called from. That is a run and
/// not a test. The rest is this file: the sweep is a proved zero, neither place
/// is a rounding off the start, the boxes are apart and neither holds the
/// start, which is the only way to that arm. Had the arm left the pair
/// `Undecidable`, a certified fuse would have refused `Undecidable` with an
/// `order along a crossing` record, where the ones that hold the fin are
/// refused only once their answer is put together.
///
/// Before Q1860's walk fix (`10517374`) every one of the twelve was refused
/// `CannotNameWhichPieceOfACrossingThisIs` out of `rcad-geom` while its answer
/// was written: where the block's side cuts the top's ellipse, the walk kept
/// both arcs, the long one on the top and a sliver `2⁻⁵¹` outside the block
/// (Q1860's trace).
#[test]
fn every_fuse_with_the_tilted_block_answers_its_closed_form() {
    let shared = std::f64::consts::PI * RADIUS * RADIUS * (4.0 / 0.5_f64.cos() - 2.0);
    let bar = std::f64::consts::PI * RADIUS * RADIUS * 10.0;
    let block = 4.0 * 12.0 * 4.0;
    for (policy, certified) in [(Policy::certified(), true), (Policy::tolerant(25.0), false)] {
        for rule in EVERY_RULE {
            for bar_first in [true, false] {
                let case = format!("{policy:?} {rule:?}, bar first {bar_first}");
                let mut log = ProvenanceLog::new();
                let (mut model, the_bar, the_block) = the_bar_and_the_block();
                let fused = fuse_the_bar_and_a_block(
                    &mut model,
                    [the_bar, the_block],
                    rule,
                    bar_first,
                    &policy,
                    &mut log,
                );
                let holds_the_fin = it_holds_the_fin(rule, bar_first);
                if holds_the_fin && certified {
                    assert_refused_for_the_fin(&fused, &case);
                    assert!(log.records().is_empty(), "{case}: {:#?}", log.records());
                    continue;
                }
                if holds_the_fin && fused.is_ok() {
                    assert_only_the_range_is_recorded(&log, &case);
                } else {
                    assert!(log.records().is_empty(), "{case}: {:#?}", log.records());
                }
                let closed_form = match (rule, bar_first) {
                    (Rule::Union, _) => block + bar - shared,
                    (Rule::Intersection, _) => shared,
                    (Rule::Difference, false) => block - shared,
                    (Rule::Difference, true) => bar - shared,
                    _ => unreachable!("{case}: EVERY_RULE names three rules"),
                };
                assert_one_solid_holding(&model, fused, closed_form, shared, &policy, &case);
                if holds_the_fin {
                    assert_refused_for_the_fin(
                        &model.validate(&Policy::certified(), &mut ProvenanceLog::new()),
                        &case,
                    );
                }
            }
        }
    }
}

/// The same bar and a block on the same square standing straight up from
/// `z = −1` to `z = 11`, so that it holds the whole bar but the fin `2⁻⁵¹`
/// thick the bar leaves outside `x = −17`. Under either policy, the
/// intersection and the block less the bar, with either solid first, answer
/// one solid that validates and holds its closed form, with nothing recorded.
/// The union and the bar less the block, the fin alone, keep the fin: under
/// `certified` each is refused `DynamicRangeTooWide` as in the test above, and
/// under `tolerant` each carries one record, of the range check, answers one
/// solid that holds its closed form, and validates under `tolerant` alone
/// (§68).
///
/// **The closed forms.** The block holds `4 · 12 · 12 = 576` and the bar
/// `10 π r²`, all of it inside the block but the fin. The fin is the piece of
/// the bar's disc beyond a chord `2⁻²⁴` long, `2⁻⁵¹` thick, two thirds of the
/// chord times its thickness, over the bar's height of 10: about `1.76e-22`,
/// far under any of the other error bars. So the union is `576`, the
/// intersection `10 π r²`, and the block less the bar `576 − 10 π r²`.
///
/// **The fin alone** holds no double in `x`, so every point of every mesh of
/// it lies on `x = −17` and encloses exactly `0`, and `measure` gives it `0`
/// give or take `4.3e-19`, over two thousand times what it holds. That cannot
/// tell the fin from any other sliver, so its faces and corners are checked
/// as well ([`assert_it_is_the_fin`]). From Q2089 until Q2291 it was refused
/// `CannotTellWhichWayASolidsFacesPoint` with `enclosed: 0.0`, since no mesh
/// could tell which way its faces point. `validate.rs` now tells it from its
/// corner `(−17, −2⁻²⁵, 0)`, on its bottom and on the block's side, below its
/// top and `2⁻¹⁰²` inside the bar's wall in `r²`. The answer was there, so the
/// refusal was a gap and not a limit (§16.10).
///
/// **The fin's ends.** Its bottom and top faces are each a chord and an arc
/// between the same two corners, `(−17, ±2⁻²⁵)`, which Q1928 met with the
/// bar's rim corner `(−17, ∓6.1e-17)` as a third, every corner on one line.
/// The triangulator hands such a face back with no triangles, and until the mesher
/// drew them itself (Q1928) the chord and both halves of the arc round each were
/// used by one triangle only, so the unions and the fin alone were refused
/// `Invalid(FacesDoNotAgreeWhichWayTheyFace)` by
/// `every_side_of_the_skin_is_used_both_ways` on the coarsest mesh.
///
/// Before Q1860's walk fix (`10517374`) the intersection and the block less the
/// bar were refused `CannotNameWhichPieceOfACrossingThisIs` before any mesh,
/// and the certified union `Undecidable` on two `side of a half-space` records
/// pushed by `which_side_of` in `rcad-geom`'s `walk.rs`, under
/// `Conic::the_stretch_this_run_covers`, while its volume was checked on its
/// mesh.
#[test]
fn every_fuse_with_the_upright_block_answers_its_closed_form() {
    let bar = std::f64::consts::PI * RADIUS * RADIUS * 10.0;
    let block = 4.0 * 12.0 * 12.0;
    let fin = 2.0 / 3.0 * f64::powi(2.0, -24) * f64::powi(2.0, -51) * 10.0;
    for (policy, certified) in [(Policy::certified(), true), (Policy::tolerant(25.0), false)] {
        for rule in EVERY_RULE {
            for bar_first in [true, false] {
                let case = format!("{policy:?} {rule:?}, bar first {bar_first}");
                let mut log = ProvenanceLog::new();
                let (mut model, the_bar, the_block) = the_bar_and_an_upright_block();
                let fused = fuse_the_bar_and_a_block(
                    &mut model,
                    [the_bar, the_block],
                    rule,
                    bar_first,
                    &policy,
                    &mut log,
                );
                let holds_the_fin = it_holds_the_fin(rule, bar_first);
                if holds_the_fin && certified {
                    assert_refused_for_the_fin(&fused, &case);
                    assert!(log.records().is_empty(), "{case}: {:#?}", log.records());
                    continue;
                }
                if holds_the_fin && fused.is_ok() {
                    assert_only_the_range_is_recorded(&log, &case);
                } else {
                    assert!(log.records().is_empty(), "{case}: {:#?}", log.records());
                }
                let closed_form = match (rule, bar_first) {
                    (Rule::Union, _) => block,
                    (Rule::Intersection, _) => bar,
                    (Rule::Difference, false) => block - bar,
                    (Rule::Difference, true) => {
                        assert_it_is_the_fin(&model, &fused, [the_bar, the_block], &case);
                        fin
                    }
                    _ => unreachable!("{case}: EVERY_RULE names three rules"),
                };
                assert_one_solid_holding(&model, fused, closed_form, bar, &policy, &case);
                if holds_the_fin {
                    assert_refused_for_the_fin(
                        &model.validate(&Policy::certified(), &mut ProvenanceLog::new()),
                        &case,
                    );
                }
            }
        }
    }
}

/// Whether the fuse of the bar and a block under `rule` keeps the fin the bar
/// leaves outside `x = −17`: the union, and the bar less the block.
fn it_holds_the_fin(rule: Rule, bar_first: bool) -> bool {
    matches!(
        (rule, bar_first),
        (Rule::Union, _) | (Rule::Difference, true)
    )
}

/// Checks that `refused` is `DynamicRangeTooWide` for the fin: `17` over one
/// rounding of `17`, which is what `the_narrowest_a_face_is` reads a face
/// thinner than a rounding as.
fn assert_refused_for_the_fin<T: std::fmt::Debug>(refused: &rcad_topo::Result<T>, case: &str) {
    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if *ratio == 1.0 / f64::EPSILON
        ),
        "{case}: {refused:?}"
    );
}

/// Checks that the one record in `log` is the range check's, the warning a
/// tolerant policy gives for the fin.
fn assert_only_the_range_is_recorded(log: &ProvenanceLog, case: &str) {
    assert!(
        log.records().len() == 1 && log.records()[0].predicate() == "dynamic range check",
        "{case}: {:#?}",
        log.records()
    );
}

/// Checks that `fused` is the fin alone: one solid whose faces are one piece
/// each of the bar's wall and its two caps and of the block's side `x = −17`,
/// and none of the block's other faces, with every corner on that side and
/// within `2⁻²⁵` of `y = 0`. What the fin holds is far under the error bar
/// `measure` gives it, so its volume cannot say this.
fn assert_it_is_the_fin(
    model: &Model,
    fused: &rcad_topo::Result<Fused>,
    [bar, block]: [SolidKey; 2],
    case: &str,
) {
    let fused = fused
        .as_ref()
        .unwrap_or_else(|why| panic!("{case}: {why:?}"));
    let [fin] = fused.solids()[..] else {
        panic!("{case}: {:?}", fused.solids())
    };
    let what_each_face_became = |key: SolidKey| -> Vec<(Surface, Vec<FaceKey>)> {
        model
            .solid(key)
            .expect("this model made the key")
            .faces()
            .map(|face| {
                let images = fused
                    .images_of_face(face.key())
                    .expect("a face of what was fused");
                (face.surface(), images.to_vec())
            })
            .collect()
    };
    let mut wanted: Vec<FaceKey> = Vec::new();
    for (surface, images) in what_each_face_became(bar) {
        assert_eq!(images.len(), 1, "{case}: the bar's {surface:?}");
        wanted.extend(images);
    }
    for (surface, images) in what_each_face_became(block) {
        let the_side = matches!(
            surface,
            Surface::Plane(flat) if flat.normal().components() == [-1.0, 0.0, 0.0]
        );
        assert_eq!(
            images.len(),
            usize::from(the_side),
            "{case}: the block's {surface:?}"
        );
        wanted.extend(images);
    }
    let fin = model.solid(fin).expect("the fuse made it");
    let faces: Vec<FaceKey> = fin.faces().map(|face| face.key()).collect();
    assert!(
        faces.len() == wanted.len() && wanted.iter().all(|face| faces.contains(face)),
        "{case}: {faces:?}, where {wanted:?} are wanted"
    );
    let corners: Vec<Point> = fin
        .faces()
        .flat_map(|face| face.wires().flat_map(|wire| wire.points()))
        .collect();
    let half_the_chord = f64::powi(2.0, -25);
    assert!(
        corners
            .iter()
            .all(|corner| corner.x() == -17.0 && corner.y().abs() <= half_the_chord),
        "{case}: {corners:?}"
    );
}

/// Checks that `fused` is one solid holding `closed_form` within the error bar
/// `measure` publishes, and that the model validates. The bar must be under
/// half of `shared`, the piece the bar and the block share, so that an answer
/// that left that piece out or counted it twice is off by more than its bar.
fn assert_one_solid_holding(
    model: &Model,
    fused: rcad_topo::Result<Fused>,
    closed_form: f64,
    shared: f64,
    policy: &Policy,
    case: &str,
) {
    let fused = fused.unwrap_or_else(|why| panic!("{case}: {why:?}"));
    let [solid] = fused.solids()[..] else {
        panic!("{case}: {:?}", fused.solids())
    };
    let measured = model
        .solid(solid)
        .expect("the fuse made it")
        .measure(
            Deflection::of(0.01).expect("a positive deflection"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{case}: {why:?}"));
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "{case}: {closed_form} wanted, {measured:?}"
    );
    assert!(
        measured.volume_error() < shared / 2.0,
        "{case}: {shared} shared, {measured:?}"
    );
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{case}: {why:?}"));
}

/// A bar `2⁻⁴⁰` wider than `1`, less the tilted block, leaves a fin outside
/// `x = −17` that is `2⁻⁴⁰` thick and `2.7e-6` across. Under `tolerant` the
/// fuse answers one solid that validates and holds `10 π r²` less what the bar
/// and the block share, worked out as the test above works it, within 30 s;
/// under `certified` it is refused `DynamicRangeTooWide`. The ellipse's start
/// is `−17 − 2⁻⁴⁰`, off the block's side, so the walk is not refused with or
/// without Q1860's walk fix.
///
/// **The fin is too thin for the mesh to split.** The block's flat strip of
/// side between the fin's two corners and the bar's wall over it are drawn on
/// the same three points, back to back, on the side from
/// `(−17, −1.3486991523489155e-6, 0)` to `(−17, 1.3486991523489157e-6, 2.558)`
/// (Q1869). Until Q2774 that was a fold like any other, `drawn 4, used 4`.
/// The strip's edges are all straight, so no finer mesh split it: before Q1869
/// the ladder in `validate.rs` asked mesh after mesh, and on `main` at
/// `a14d3018` both policies were still walking it after 465 s, the fold the
/// same at every rung from 5 down to `1.2e-3` and 24672 points. From Q1869 the
/// ladder ended thirteen halvings under its top, at `6.1e-4` here, and the
/// fuse was refused that fold. Since Q2774 the sum `Model::validate` reads
/// keeps two faces' triangles drawn back to back (`docs/decisions.md` §466),
/// and the fuse answers. The 30 s bound stays, so that the old wait cannot
/// come back unseen.
///
/// **Its measure sums over the same mesh**, the pair kept, since Q2790. Until
/// then `SolidView::measure` meshed with `SolidView::mesh`, which refuses the
/// pair, and was refused `TwoPatchesOfTheMeshDrewTheSameSide`.
///
/// **Only the tolerant fuse gets a mesh.** The fin's piece of the bar's
/// bottom cap bows `2⁻⁴⁰` off its chord, a spread of `17 / 2⁻⁴⁰`, `1.87e13`,
/// past §3.5's `10¹¹`, so under `certified` the fuse is refused
/// `DynamicRangeTooWide` before any mesh is asked for (queue rows Q2237 and
/// Q2257). Under `tolerant` that spread is a warning.
#[test]
fn a_fin_too_thin_for_the_mesh_to_split_answers_in_seconds_under_tolerant() {
    let radius = 1.0 + f64::powi(2.0, -40);
    let shared = std::f64::consts::PI * radius * radius * (4.0 / 0.5_f64.cos() - 2.0);
    let the_bar_less_the_block = std::f64::consts::PI * radius * radius * 10.0 - shared;
    for (policy, certified) in [(Policy::certified(), true), (Policy::tolerant(25.0), false)] {
        let (answer, answered) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let (mut model, bar, block) = the_block_and_a_bar_of_radius(radius);
            let fused = model.fuse(
                &[bar],
                &[block],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            );
            answer
                .send((model, fused))
                .expect("the test waits for the answer");
        });
        let (model, fused) = answered
            .recv_timeout(std::time::Duration::from_secs(30))
            .unwrap_or_else(|_| panic!("{policy:?}: still meshing after 30 s"));
        if certified {
            assert!(
                matches!(
                    fused,
                    Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio }))
                        if ratio > DynamicRange::LARGEST_SUPPORTED_RATIO
                ),
                "{policy:?}: {fused:?}"
            );
        } else {
            assert_one_solid_holding(
                &model,
                fused,
                the_bar_less_the_block,
                shared,
                &policy,
                &format!("{policy:?}"),
            );
        }
    }
}

fn fuse_the_bar_and_a_block(
    model: &mut Model,
    [bar, block]: [SolidKey; 2],
    rule: Rule,
    bar_first: bool,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    if bar_first {
        model.fuse(&[bar], &[block], rule, policy, log)
    } else {
        model.fuse(&[block], &[bar], rule, policy, log)
    }
}

fn the_bar_and_an_upright_block() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let bar = model
        .add_round_bar(
            RADIUS,
            10.0,
            &Isometry::translation(Vector::new(-16.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius and height");
    let square = ProfileBuilder::new(vec![
        [-17.0, -6.0],
        [-13.0, -6.0],
        [-13.0, 6.0],
        [-17.0, 6.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let block = model
        .add_extrusion(
            &square,
            12.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a finite shift"),
        )
        .expect("a positive height");
    (model, bar, block)
}

/// The block's top, the plane whose normal points mostly up, its side
/// `x = −17`, and the bar's wall, each as the model places it.
fn the_top_the_side_and_the_wall() -> [Surface; 3] {
    let (model, bar, block) = the_bar_and_the_block();
    let surfaces = |key: SolidKey| -> Vec<Surface> {
        model
            .solid(key)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .collect()
    };
    let block = surfaces(block);
    let plane_facing = |facing: fn([f64; 3]) -> bool| {
        let mut found = block.iter().filter(
            |surface| matches!(surface, Surface::Plane(flat) if facing(flat.normal().components())),
        );
        let (Some(plane), None) = (found.next(), found.next()) else {
            panic!("one face of the block faces that way: {block:?}")
        };
        *plane
    };
    let top = plane_facing(|normal| normal[2] > 0.5);
    let side = plane_facing(|normal| normal == [-1.0, 0.0, 0.0]);
    let bar = surfaces(bar);
    let wall = *bar
        .iter()
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a round bar has a wall");
    [top, side, wall]
}

/// The centre of the ellipse the top cuts the wall in, worked out the way
/// `where_a_plane_cuts_a_pipe` in `rcad-geom`'s `walk.rs` works it out.
fn where_the_axis_meets_the_top(top: &Surface, wall: &Surface) -> [f64; 3] {
    let (Surface::Plane(flat), Surface::Cylinder(pipe)) = (top, wall) else {
        panic!("a plane and a pipe: {top:?}, {wall:?}")
    };
    let normal = flat.normal();
    let along = pipe.axis().direction();
    let up_the_axis =
        (flat.offset() - normal.dot(pipe.axis().origin() - Point::origin())) / normal.dot(along);
    (pipe.axis().origin() + along * up_the_axis).coordinates()
}

/// The two places the block's side cuts the top's ellipse at, the one below
/// `y = 0` first.
fn the_two_places(top: &Surface, side: &Surface, wall: &Surface) -> [PredicatePoint; 2] {
    let MeetOfThree::At(places) = meet_of_three_surfaces(
        [side, top, wall],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the three meet") else {
        panic!("the side, the top and the wall meet at places")
    };
    let [one, other] = places[..] else {
        panic!("two places: {places:?}")
    };
    if one.enclosure()[1].hi() < 0.0 {
        [one, other]
    } else {
        [other, one]
    }
}

fn the_walked_start(pair: [&Surface; 2]) -> [f64; 3] {
    let SurfaceMeet::Curves(runs) = intersect(
        pair[0],
        pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the top crosses the wall") else {
        panic!("the top crosses the wall in a curve")
    };
    let [ellipse] = &runs[..] else {
        panic!("one ellipse: {runs:?}")
    };
    where_a_closed_crossing_starts(ellipse, pair, &mut ProvenanceLog::new())
        .expect("an ellipse has a start")
        .coordinates()
}

/// `|w|²(x − o) − (w·(x − o))w`, the gradient `fuse/arrange.rs`'s `gradient_at`
/// takes of a pipe, in the same arithmetic.
fn the_walls_gradient_at(wall: &Surface, at: [f64; 3]) -> [Interval; 3] {
    let Surface::Cylinder(pipe) = wall else {
        panic!("a pipe: {wall:?}")
    };
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components().map(Interval::exactly);
    let step = [0, 1, 2].map(|axis| Interval::exactly(at[axis]) - Interval::exactly(origin[axis]));
    let (squared, shadow) = (dot(along, along), dot(along, step));
    [0, 1, 2].map(|axis| squared * step[axis] - shadow * along[axis])
}

fn cross(one: [Interval; 3], other: [Interval; 3]) -> [Interval; 3] {
    [0, 1, 2].map(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        one[next] * other[last] - one[last] * other[next]
    })
}

fn dot(one: [Interval; 3], other: [Interval; 3]) -> Interval {
    (0..3)
        .map(|axis| one[axis] * other[axis])
        .fold(Interval::exactly(0.0), |so_far, term| so_far + term)
}

fn the_bar_and_the_block() -> (Model, SolidKey, SolidKey) {
    the_block_and_a_bar_of_radius(RADIUS)
}

fn the_block_and_a_bar_of_radius(radius: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let bar = model
        .add_round_bar(
            radius,
            10.0,
            &Isometry::translation(Vector::new(-16.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius and height");
    let square = ProfileBuilder::new(vec![
        [-17.0, -6.0],
        [-13.0, -6.0],
        [-13.0, 6.0],
        [-17.0, 6.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let placement = Isometry::translation(Vector::new(0.0, 0.0, -2.0))
        .expect("a finite shift")
        .compose(&Isometry::rotation(&Axis::x(), -0.5).expect("a finite turn"));
    let block = model
        .add_extrusion(&square, 4.0, &placement)
        .expect("a positive height");
    (model, bar, block)
}
