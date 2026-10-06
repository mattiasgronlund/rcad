//! A bar across the wall of a tube, united with it: a joined flat face whose
//! outline and hole each run along two pipes.
//!
//! The tube is a bar of radius 7 and height 4 less a bar of radius 1 standing
//! through it. The bar has radius 5 and stands at `(−4, −4)`, so its circle
//! crosses the tube's outer circle at `(0, −7)` and `(−7, 0)` and its inner
//! circle at `(0, −1)` and `(−1, 0)`: it pokes out past the wall on one side
//! and into the hole on the other, and leaves most of the hole open.
//!
//! United, each cap is the tube's ring and the bar's disk joined into one face.
//! Its outline is an arc of the outer pipe and an arc of the bar, and its hole
//! an arc of the inner pipe and an arc of the bar. **Each ring has two corners,
//! so neither encloses any area between its corners**, and neither runs along
//! one surface, so the region it bounds is not simply the inside of a pipe.
//! Before the fix that was a panic in `fuse/assemble.rs`, both ways round: *"2
//! rings of a joined face enclose no area between their corners, and the regions
//! they bound cannot say which of them is the outline"*. Q576 found it; Q359 is
//! the same panic for rings along one pipe, and §191.5 is the family.
//!
//! # The rings `holds_the_other_rings` still leaves unplaced
//!
//! Where it answers `None` for two rings of one joined face, `rings_round`
//! panics with that same message. The rest of this file builds the families it
//! answers `None` for and pins what the fuse does with each. Each one that is
//! refused is refused before `rings_round`, and each test names the stage: that was
//! read with a breakpoint on `select::choose`, on `rings_round` and on
//! `holds_the_other_rings` (Q597), which the two tests above do reach. Two
//! holed bars turned a quarter are answered and reach `holds_the_other_rings`
//! too, but not by the family they were built for (Q616). The same two bars
//! unturned, the probe for that family, were refused before `rings_round`
//! until Q897 and are answered now. The probe for an arc along a cone's
//! parabola, a tube and a cone cut by one leaning plane, is refused before
//! `rings_round` under every rule (Q634), and so is the probe for two arcs
//! along two cones' hyperbolas, a cut cone less a second cone on its axis,
//! because two different cones could not be crossed (Q636), then because the
//! arrangement could not order points round the circle they cross in (Q1122),
//! and now because a point cannot be tested against the cut cone's face
//! (Q1129).
//! The family of
//! corners that sweep no area and are not on one line is built with corners
//! that cancel exactly, face by face, and the fuse that would join its cap
//! puts a corner on its arc wherever one of its own walls would cross it, so
//! the ring `rings_round` is handed winds (Q641).

use std::f64::consts::PI;

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, NamedRun, Orientation, Picker,
    Plane, Point, Scale, Surface, SurfaceMeet, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};
use smallvec::smallvec;

mod rings;

/// With both caps flush, both are joined, and the union is the tube and the
/// bar less what they share, whichever solid goes first.
#[test]
fn a_bar_across_a_tubes_wall_united_with_it_holds_both() {
    for bar_first in [false, true] {
        let (mut model, tube, bar) = a_tube_and_a_bar_across_its_wall(0.0, 4.0);
        let [one, other] = if bar_first { [bar, tube] } else { [tube, bar] };
        let answer = the_one_solid(&mut model, one, other, Rule::Union);
        let shared = 4.0 * (what_the_bar_shares_with(7.0) - what_the_bar_shares_with(1.0));
        let volume = 4.0 * 48.0 * PI + 4.0 * 25.0 * PI - shared;
        assert_holds(&model, answer, volume, "the union");
    }
}

/// With the bar standing a unit lower, only the top cap is flush and joined.
#[test]
fn a_bar_across_a_tubes_wall_and_below_it_united_with_it_holds_both() {
    let (mut model, tube, bar) = a_tube_and_a_bar_across_its_wall(-1.0, 5.0);
    let answer = the_one_solid(&mut model, tube, bar, Rule::Union);
    let shared = 4.0 * (what_the_bar_shares_with(7.0) - what_the_bar_shares_with(1.0));
    let volume = 4.0 * 48.0 * PI + 5.0 * 25.0 * PI - shared;
    assert_holds(&model, answer, volume, "the union");
}

fn a_tube_and_a_bar_across_its_wall(bottom: f64, height: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outer = model
        .add_round_bar(7.0, 4.0, &Isometry::identity())
        .expect("a bar of radius seven");
    let through = model
        .add_round_bar(1.0, 6.0, &at(0.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
    let bar = model
        .add_round_bar(5.0, height, &at(-4.0, -4.0, bottom))
        .expect("a bar of radius five");
    (model, tube, bar)
}

/// The area the bar's disk shares with a disk of `radius` about the tube's axis:
/// two circular segments either side of the chord between the two crossings.
fn what_the_bar_shares_with(radius: f64) -> f64 {
    let (bar, apart) = (5.0_f64, 32.0_f64.sqrt());
    let segment = |mine: f64, theirs: f64| {
        let half_angle = ((apart * apart + mine * mine - theirs * theirs) / (2.0 * apart * mine))
            .clamp(-1.0, 1.0)
            .acos();
        mine * mine * (half_angle - half_angle.sin() * half_angle.cos())
    };
    segment(radius, bar) + segment(bar, radius)
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
    let [solid] = answer.solids() else {
        panic!("{rule:?} is one solid, not {:?}", answer.solids());
    };
    *solid
}

fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}

/// **A ring along one torus.** A block whose top and bottom are square to a
/// bend's axis would cut its wall in two circles, but the block's four sides
/// are parallel to the axis and off it, and `intersect` refuses such a plane
/// against a torus as a quartic before a single face is cut — even where, as
/// here for the sides at `±6`, the plane misses the torus entirely. Every rule,
/// both ways round.
///
/// The quarter bend is `rings::a_quarter_bend` with ring 4 and tube 1; the
/// block is the square from `(−6, −6)` to `(6, 6)` stood from `z = 0` to `z = 2`.
/// Refused in `cuts::gather`, so it tests nothing about `rings_round`.
#[test]
fn a_block_square_to_a_bends_axis_is_refused_before_anything_is_cut() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for block_first in [false, true] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let bend = rings::a_quarter_bend(&mut model, 4.0, 1.0);
            let square =
                ProfileBuilder::new(vec![[-6.0, -6.0], [6.0, -6.0], [6.0, 6.0], [-6.0, 6.0]])
                    .build(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a square");
            let block = model
                .add_extrusion(&square, 2.0, &Isometry::identity())
                .expect("a block");
            let [one, other] = if block_first {
                [block, bend]
            } else {
                [bend, block]
            };
            let refused = model.fuse(
                &[one],
                &[other],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(
                    refused,
                    Err(Error::Geometry(
                        rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                    ))
                ),
                "{rule:?}, block first {block_first}: {refused:?}"
            );
        }
    }
}

/// **A ring along one torus, and an arc along a torus beside straight
/// sides.** The top halves, `z ≥ 0`, of the two half bends of one ring, ring 4
/// and tube 1: one from `+x` round through `+y` to `−x`, the other from `−x`
/// round through `−y` to `+x`. Their caps meet flush on `y = 0`, and their
/// floors are two half rings of `z = 0`, each a ring of four corners all on
/// the `x` axis — outer arc, straight side, inner arc, straight side — so
/// neither floor encloses any area between its corners.
///
/// United, the two floors would join into the whole flat ring between the
/// circles of radius 3 and 5, each a ring along the torus alone, which is the
/// case `holds_the_other_rings` answers `None` for. **That is not reached.**
/// Every rule, both ways round, is refused while the first solid's cap is cut
/// into arcs (`which_way_an_edge_runs_round`, from
/// `arrange::cut_the_crossings_into_arcs`): the cap's arc is half of one of the
/// two circles `y = 0` cuts the torus in, and the arrangement has been taught to
/// read a picker only on a ball's circle or a cone's circle or ellipse. A gap by
/// that function's own account. Whether the union panics once it is closed is
/// not shown here; the breakpoint on `select::choose` was never reached.
#[test]
fn the_top_halves_of_two_half_bends_are_refused_while_a_cap_is_cut_into_arcs() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for turned_first in [false, true] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [one, other] = [turned_first, !turned_first].map(|turned| {
                the_top_half_of_a_half_bend(&mut model, turned)
                    .unwrap_or_else(|why| panic!("the top half of a half bend: {why:?}"))
            });
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("both halves are sound");
            // Pappus: a half disk of area π/2 whose centroid is 4 from the
            // axis, carried half a turn round it, π·4.
            for half in [one, other] {
                assert_measures(
                    &model,
                    half,
                    PI / 2.0 * PI * 4.0,
                    "the top half of a half bend",
                );
            }
            let refused = model.fuse(
                &[one],
                &[other],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) = refused else {
                panic!("{rule:?}, turned first {turned_first}: {refused:?}");
            };
            let first = model.solid(one).expect("this model made the key");
            assert!(
                first.faces().any(|mine| mine.key() == face),
                "{rule:?}, turned first {turned_first}: the refusal names a face of the second \
                 solid"
            );
        }
    }
}

fn assert_measures(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The top half, `z ≥ 0`, of a half bend about `+z` with ring 4 and tube 1,
/// from `+x` round through `+y` to `−x`; `turned`, half a turn round, from `−x`
/// through `−y` to `+x`. Four faces: the half torus wall, the half ring of
/// `z = 0` under it, and a half disk of `y = 0` at each end.
///
/// Written out the way `rings.rs` writes a bend: the wall used forwards, as a
/// torus faces out of its tube (`docs/decisions.md` §361), every number turned
/// exactly, and each arc named by the half-spaces and the pipe of the ring's
/// own radius that pick it. No edge here is closed, so there is no circle to
/// turn over: the wall's loop already runs right-handed about the way out of
/// the tube, anticlockwise seen from above.
fn the_top_half_of_a_half_bend(model: &mut Model, turned: bool) -> rcad_topo::Result<SolidKey> {
    let (ring, tube) = (4.0, 1.0);
    let way = if turned { -1.0 } else { 1.0 };
    let at = |x: f64| Point::new(written_out(way * x), 0.0, 0.0);
    let facing =
        |x: f64, y: f64, z: f64| Vector::new(written_out(way * x), written_out(way * y), z);
    let wall =
        Surface::Torus(Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring"));
    let floor = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let caps = Plane::from_point_and_normal(Point::origin(), facing(0.0, -1.0, 0.0))
        .expect("the y axis is a unit normal");
    let across = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let pipe = Cylinder::new(Axis::z(), ring).expect("the ring's own radius");
    let side =
        |normal: Vector| Picker::from(HalfSpace::new(normal, 0.0).expect("a normal with a length"));

    let mut builder = SolidBuilder::new();
    let outer_start = builder.add_vertex(at(ring + tube))?;
    let inner_start = builder.add_vertex(at(ring - tube))?;
    let outer_end = builder.add_vertex(at(-(ring + tube)))?;
    let inner_end = builder.add_vertex(at(-(ring - tube)))?;

    let along_the_floor = |inside: Picker| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(floor),
        picked_by: smallvec![inside, side(facing(0.0, 1.0, 0.0))],
    };
    let over_the_tube = |towards: f64| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(across),
        picked_by: smallvec![
            side(facing(towards, 0.0, 0.0)),
            side(Vector::new(0.0, 0.0, 1.0))
        ],
    };
    let outer = builder.add_edge(
        outer_start,
        outer_end,
        along_the_floor(Picker::OutsideAPipe(pipe)),
    )?;
    let inner = builder.add_edge(
        inner_start,
        inner_end,
        along_the_floor(Picker::InsideAPipe(pipe)),
    )?;
    let over_the_start = builder.add_edge(outer_start, inner_start, over_the_tube(1.0))?;
    let over_the_end = builder.add_edge(inner_end, outer_end, over_the_tube(-1.0))?;
    let under_the_start = builder.add_edge(inner_start, outer_start, EdgeRun::Straight)?;
    let under_the_end = builder.add_edge(outer_end, inner_end, EdgeRun::Straight)?;

    let floor_wire = a_wire(
        &mut builder,
        &[
            (under_the_start, Orientation::Reversed),
            (inner, Orientation::Forward),
            (under_the_end, Orientation::Reversed),
            (outer, Orientation::Reversed),
        ],
    )?;
    let floor_face = builder.add_face(Surface::Plane(floor), &[floor_wire])?;
    let start_wire = a_wire(
        &mut builder,
        &[
            (under_the_start, Orientation::Forward),
            (over_the_start, Orientation::Forward),
        ],
    )?;
    let start_cap = builder.add_face(Surface::Plane(caps), &[start_wire])?;
    let end_wire = a_wire(
        &mut builder,
        &[
            (under_the_end, Orientation::Forward),
            (over_the_end, Orientation::Forward),
        ],
    )?;
    let end_cap = builder.add_face(Surface::Plane(caps), &[end_wire])?;
    let wall_wire = a_wire(
        &mut builder,
        &[
            (outer, Orientation::Forward),
            (over_the_end, Orientation::Reversed),
            (inner, Orientation::Reversed),
            (over_the_start, Orientation::Reversed),
        ],
    )?;
    let wall_face = builder.add_face(wall, &[wall_wire])?;
    builder.add_shell(&[
        (floor_face, Orientation::Forward),
        (start_cap, Orientation::Forward),
        (end_cap, Orientation::Forward),
        (wall_face, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// `−0.0` written as `+0.0`, which is how a person writing a zero out writes it.
fn written_out(number: f64) -> f64 {
    if number == 0.0 { 0.0 } else { number }
}

fn a_wire(
    builder: &mut SolidBuilder,
    steps: &[(DraftEdge, Orientation)],
) -> rcad_topo::Result<rcad_topo::DraftWire> {
    let mut along = Vec::with_capacity(steps.len());
    for &(edge, turn) in steps {
        along.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    builder.add_wire(&along)
}

mod cones;

/// **No corner of the other rings placed.** Two bars of radius 10 and height
/// 4 standing at `(−6, 0)` and `(6, 0)`, each less a bar of radius 2 through
/// the origin. United, the joined cap's outline would be an arc of each big
/// bar with its two corners where their circles cross, `(0, ±8)`, and its
/// hole the small circle, whose one corner the fuse puts at `(0, −2)`: on the
/// chord of the outline's arcs and inside both big bars, so
/// `holds_the_other_rings` would pass it over and place nothing.
///
/// **Every rule, both ways round, now answers**, and holds what the circles
/// say: the same volumes as the pair turned a quarter, below. Each used to be
/// refused in `select::choose` on `a point inside a piece of a face`, its
/// record naming those three corners on `x = 0` (Q616), until `select` offered
/// rungs nearer the places the line square to each arc's chord crosses its
/// circle (`docs/decisions.md` §365.6, Q897). Over the six fuses,
/// `join_up_what_is_one_face` is reached 18 times and `holds_the_other_rings`
/// 8 times (breakpoints, Q897), and `rings_round` does not panic. Where the
/// fuse put the hole's corner, and what `holds_the_other_rings` answered, was
/// not read.
#[test]
fn two_holed_bars_whose_corners_lie_on_one_line_are_answered_under_every_rule() {
    let lens = 200.0 * 0.6_f64.acos() - 96.0;
    let hole = 4.0 * PI;
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = two_bars_holed_through_the_origin(&mut model, [6.0, 0.0]);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, pair, rule, swapped);
        let area = match rule {
            Rule::Union => 200.0 * PI - lens - hole,
            Rule::Intersection => lens - hole,
            Rule::Difference => 100.0 * PI - lens,
            other => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), 4.0 * area, &what);
    }
}

/// The same two holed bars turned a quarter, standing at `(0, ∓6)`. The fuse
/// puts the hole's corner at `(0, 2)` this time, off the chord `y = 0` through
/// the outline's corners `(±8, 0)`, so the hole's corner is placed by how the
/// outline's arcs bulge and nothing is left unplaced. Every rule, both ways
/// round, is answered and holds what the circles say.
///
/// Under the union `holds_the_other_rings` is reached four times, once for
/// each of the two rings of each cap, and answers `Some(false)` for each hole
/// and `Some(true)` for each outline (read off its return with gdb, Q616). So
/// family 5 needs the hole's corner on the chord, and the probe above that
/// puts it there was refused before this until Q897; what it reaches now is
/// read there.
#[test]
fn two_holed_bars_turned_a_quarter_are_answered_with_the_hole_placed_by_the_outlines_bulge() {
    let lens = 200.0 * 0.6_f64.acos() - 96.0;
    let hole = 4.0 * PI;
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = two_bars_holed_through_the_origin(&mut model, [0.0, 6.0]);
        let answer = fused_in_order(&mut model, pair, rule, swapped)
            .unwrap_or_else(|why| panic!("{rule:?}, swapped {swapped}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("{rule:?}, swapped {swapped}: {answer:?}");
        };
        let area = match rule {
            Rule::Union => 200.0 * PI - lens - hole,
            Rule::Intersection => lens - hole,
            Rule::Difference => 100.0 * PI - lens,
            other => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(
            &model,
            solid,
            4.0 * area,
            &format!("{rule:?}, swapped {swapped}"),
        );
    }
}

/// **An arc along a cone's hyperbola.** A pointed cone, apex at the origin,
/// slope 1, base at `z = 20`, and a block from `x = −12` to `24`, `y = ±24`,
/// `z = −1` to `18`. The block's wall `x = −12` cuts the cone in a hyperbola,
/// so the piece of that wall inside the cone is a ring of two corners: an arc
/// of the hyperbola and a chord along the block's top. That is the ring a
/// joined face on `x = −12` would hand `holds_the_other_rings`, which answers
/// `None` for a hyperbola arc. Two nested cups of two cones on that wall, and a
/// block flush with it to join its pieces, were the probe; a ball's pocket
/// in place of the second cone was the other.
///
/// **This pair, the first fuse each needs, is answered under every rule, both
/// ways round**, and each answer holds what the cone and the block say. It
/// used to be refused in `select::choose` on `a point inside a piece of a
/// face`, its record naming the two corners where the block's top meets the
/// cone, `(−12, ±√180, 18)` (Q616). No point the search builds is found inside
/// one piece with those corners, though each such piece holds a double
/// (§365.2, Q635), and that piece is now placed by the side it shares with a
/// piece of the other solid's face instead (§365.3, Q712). Moving the wall to
/// `x = −11` or the top above the cone's base was refused the same way, and
/// has not been run since. The probe of two cones is refused at its second
/// fuse (the next test, Q636); the ball's pocket has not been run since, so
/// whether it now reaches `holds_the_other_rings` is not known.
///
/// The part of the cone the block holds is the cone up to `z = 18`, `1944π`,
/// less what lies past the wall, `∫₁₂¹⁸ (z² acos(12/z) − 12√(z² − 144)) dz`,
/// which is `1944 acos(2/3) − 144√180 + 576 ln((18 + √180)/12)`.
#[test]
fn a_cone_cut_by_a_block_wall_off_its_axis_is_answered_under_every_rule() {
    let cone = 8000.0 * PI / 3.0;
    let block = 36.0 * 48.0 * 19.0;
    let shared = what_the_block_holds_of_the_cone();
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = a_cone_and_a_block_across_it(&mut model);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, pair, rule, swapped);
        let volume = match (rule, swapped) {
            (Rule::Union, _) => cone + block - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, false) => cone - shared,
            (Rule::Difference, true) => block - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), volume, &what);
    }
}

/// **Two cups along two cones' hyperbolas on one face.** The cone the block
/// above holds, less a second pointed cone of slope 1 on the same axis, apex
/// at `z = 2` and base at `z = 16`. On the wall `x = −12` the second cone's
/// cup reaches down to `(−12, 0, 14)` and is `√52` either side at its base,
/// inside the first's, which reaches down to `(−12, 0, 12)` and is `√112`
/// either side at that height: the two cups nest. United with the block
/// `[−12, −11] × [10, 12] × [17.25, 17.75]`, flush with that wall, the wall's
/// pieces would join into one face whose outline and hole each run along a
/// hyperbola between two corners, which `holds_the_other_rings` answers `None`
/// for.
///
/// **That is not reached: the cut cone less the second cone is refused under
/// every rule, both ways round**, and each of Q1122 and Q1129 took it one step
/// further on. The two surfaces cross: a cone here is both nappes, and the
/// first's upper nappe meets the second's lower one in the circle of radius 1
/// at `z = 1`, which `intersect` names now — it used to refuse every pair of
/// two different cones in `cuts::gather` (a breakpoint on `intersect`, Q636).
/// That circle is on the first cone's face and below the second's apex, so the
/// two faces never meet. The arrangement of the first cone's face refused it
/// next, `ACrossingThisCannotBeOrderedAlong`, because two cones were not a pair
/// `a_pair_that_crosses_in_a_conic` in `fuse/arrange.rs` listed (Q1122), and
/// then `CannotTellTheCellsOfAFaceApart`, because the circle's walks could not
/// be placed round the cone's axis against the face's own outline (Q1129).
/// Both are answered now, and every rule is refused
/// `AFaceTrimmedByACurveThisCannotFollow` on the first cone's face, which
/// comes from `holds.rs`, where a point is tested against a face; which of its
/// tests gives up was not traced. **So the nested cups still do not reach
/// `rings_round`**, and the flush block is not built.
///
/// Both solids are measured first: the cut cone holds what the test above
/// says, and the second cone `2744π/3`.
#[test]
fn a_cut_cone_less_a_second_cone_on_its_axis_is_refused_before_its_cups_are_joined() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [cut, second] = a_cut_cone_and_a_second_cone_on_its_axis(&mut model);
    assert_holds(
        &model,
        cut,
        what_the_block_holds_of_the_cone(),
        "the cut cone",
    );
    assert_holds(&model, second, 2744.0 * PI / 3.0, "the second cone");

    let first_skin = Cone::new(Axis::z(), 1.0).expect("a positive slope");
    let second_skin = first_skin.transformed(&at(0.0, 0.0, 2.0));
    assert_eq!(
        [
            first_skin.point_at(0.0, 1.0),
            second_skin.point_at(0.0, -1.0)
        ],
        [Point::new(1.0, 0.0, 1.0); 2],
        "both cones hold the point one up and one out, the second on its lower nappe"
    );
    let named = rcad_geom::intersect(
        &Surface::Cone(first_skin),
        &Surface::Cone(second_skin),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert_eq!(
        named,
        Ok(SurfaceMeet::Curves(vec![
            NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
                .expect("a whole circle needs nothing picking it")
        ])),
        "one slope, two apexes: the one circle halfway between them"
    );

    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = a_cut_cone_and_a_second_cone_on_its_axis(&mut model);
        let refused = fused_in_order(&mut model, pair, rule, swapped);
        assert!(
            matches!(
                refused,
                Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. })
            ),
            "{rule:?}, swapped {swapped}: {refused:?}"
        );
    }
}

/// The pointed cone of slope 1 with its base at `z = 20`, and the block from
/// `x = −12` to `24`, `y = ±24`, `z = −1` to `18`, cone first.
fn a_cone_and_a_block_across_it(model: &mut Model) -> [SolidKey; 2] {
    let cone = cones::a_pointed_cone(model, 1.0, 20.0);
    let block = a_block(
        model,
        &[[-12.0, -24.0], [24.0, -24.0], [24.0, 24.0], [-12.0, 24.0]],
        -1.0,
        19.0,
    );
    [cone, block]
}

/// The part of [`a_cone_and_a_block_across_it`]'s cone that its block holds,
/// as the test of the pair works it out. A quadrature of the slices agrees to
/// 1e-11 (Q636).
fn what_the_block_holds_of_the_cone() -> f64 {
    let past_the_wall = 1944.0 * (2.0_f64 / 3.0).acos() - 144.0 * 180.0_f64.sqrt()
        + 576.0 * ((18.0 + 180.0_f64.sqrt()) / 12.0).ln();
    1944.0 * PI - past_the_wall
}

/// The cone the block holds, and the second cone, its apex moved up to `z = 2`.
fn a_cut_cone_and_a_second_cone_on_its_axis(model: &mut Model) -> [SolidKey; 2] {
    let [cone, block] = a_cone_and_a_block_across_it(model);
    let cut = the_one_solid(model, cone, block, Rule::Intersection);
    let second = cones::a_pointed_cone(model, 1.0, 14.0);
    model
        .place_solid(second, &at(0.0, 0.0, 2.0))
        .expect("this model made the key");
    [cut, second]
}

/// **An arc along a cone's parabola.** The plane `z = x + 4` leans at the
/// slope of a pointed cone of slope 1, apex at the origin and base at
/// `z = 10`, so it cuts the cone in a parabola. A tube about the same axis,
/// radius 12 less radius 4 from `z = −10` to `20`, and the cone are each cut
/// to the far side of that plane by a wedge built face by face, the only door
/// to a plane leaning along `x` (extrusions run along `z`, and a quarter turn
/// is not exact). United, the two faces on the plane would join into one whose
/// outline is the outer pipe's ellipse, one corner, and whose hole is an arc of
/// the inner pipe's ellipse and an arc of the parabola meeting at `(0, ±4, 4)`,
/// two corners. Neither encloses any area between its corners, and the hole
/// runs along a parabola, which `holds_the_other_rings` answers `None` for.
///
/// **That is not reached: every rule, both ways round, is refused before
/// `rings_round`, on `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`.** Cone first
/// used to stop earlier, on `AFaceTrimmedByACurveThisCannotFollow` naming a
/// face of the tube; sharing the arc picker between `holds.rs` and
/// `arrange.rs` (Q1033) takes it one step further, to the same three-surface
/// meet as tube first. The hole's corners are where the cone, the inner pipe
/// and the leaning plane meet, and `rcad-geom` works out a meet of two curved
/// surfaces and a plane about one axis only when the plane is square to that
/// axis (a gap, `docs/decisions.md` §31.2). Two other shapes were run on the
/// way, and their set-up fuses refused as well (Q634): the bar of radius 12
/// cut to this side less the cut cone, and the cut cone less a bar of radius
/// 1 up its axis, both the same three-surface refusal since Q1033 (Q1092 had
/// the second one on `AFaceTrimmedByACurveThisCannotFollow`, before it).
///
/// Both solids are measured before they are fused. The tube holds the ring's
/// area times its mean height, `128π · 16 = 2048π`. The cone holds
/// `1000π/3 − (2000/3)·atan(1/2) + 416/9`, of which `320π/3 − 512/9` lies inside
/// the inner pipe; the union is then `2048π + 320π/3 − 512/9`, and the
/// intersection the rest of the cone. All four agree with a quadrature of the
/// slices to 1e-12 (Q634).
#[test]
fn a_tube_and_a_cone_cut_by_one_leaning_plane_are_refused_before_their_rings_are_placed() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [tube, cone] = a_tube_and_a_cone_cut_by_one_leaning_plane(&mut model);
    assert_holds(&model, tube, 2048.0 * PI, "the cut tube");
    let the_cut_cone = 1000.0 * PI / 3.0 - 2000.0 / 3.0 * 0.5_f64.atan() + 416.0 / 9.0;
    assert_holds(&model, cone, the_cut_cone, "the cut cone");

    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = a_tube_and_a_cone_cut_by_one_leaning_plane(&mut model);
        let refused = fused_in_order(&mut model, pair, rule, swapped);
        let first = if swapped { "cone" } else { "tube" };
        assert!(
            matches!(
                refused,
                Err(Error::Geometry(
                    rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                ))
            ),
            "{rule:?}, {first} first: {refused:?}"
        );
    }
}

/// The tube and the cone, each cut to the far side of the leaning plane, tube
/// first. The bar is cut before it is holed: holed first, cutting it is
/// refused (the next test).
fn a_tube_and_a_cone_cut_by_one_leaning_plane(model: &mut Model) -> [SolidKey; 2] {
    let outer = model
        .add_round_bar(12.0, 30.0, &at(0.0, 0.0, -10.0))
        .expect("a bar of radius twelve");
    let cone = cones::a_pointed_cone(model, 1.0, 10.0);
    let [bar, cone] = [outer, cone].map(|whole| {
        let wedge = the_wedge_past_the_leaning_plane(model)
            .unwrap_or_else(|why| panic!("the wedge: {why:?}"));
        the_one_solid(model, whole, wedge, Rule::Intersection)
    });
    let through = model
        .add_round_bar(4.0, 34.0, &at(0.0, 0.0, -12.0))
        .expect("a bar of radius four");
    let tube = the_one_solid(model, bar, through, Rule::Difference);
    [tube, cone]
}

/// The prism `z ≥ x + 4`, `x ≥ −14`, `z ≤ 22`, `−14 ≤ y ≤ 14`: a right
/// triangle in the `xz`-plane with corners `(−14, −10)`, `(−14, 22)` and
/// `(18, 22)`, run along `y`. Its face on `z = x + 4` faces `(1, 0, −1)`, and
/// every other face lies clear of the tube and the cone.
fn the_wedge_past_the_leaning_plane(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let mut builder = SolidBuilder::new();
    let [low, high, far] = [[-14.0, -10.0], [-14.0, 22.0], [18.0, 22.0]];
    let mut both_ends = |[x, z]: [f64; 2]| {
        Ok::<_, rcad_topo::Error>([
            builder.add_vertex(Point::new(x, -14.0, z))?,
            builder.add_vertex(Point::new(x, 14.0, z))?,
        ])
    };
    let [low_front, low_back] = both_ends(low)?;
    let [high_front, high_back] = both_ends(high)?;
    let [far_front, far_back] = both_ends(far)?;
    let mut straight = |start, end| builder.add_edge(start, end, EdgeRun::Straight);
    let low_across = straight(low_front, low_back)?;
    let high_across = straight(high_front, high_back)?;
    let far_across = straight(far_front, far_back)?;
    let front_leaning = straight(low_front, far_front)?;
    let front_top = straight(high_front, far_front)?;
    let front_side = straight(low_front, high_front)?;
    let back_leaning = straight(low_back, far_back)?;
    let back_top = straight(high_back, far_back)?;
    let back_side = straight(low_back, high_back)?;

    let leaning = a_flat_face(
        &mut builder,
        Point::new(0.0, 0.0, 4.0),
        Vector::new(1.0, 0.0, -1.0),
        &[
            (front_leaning, Reversed),
            (low_across, Forward),
            (back_leaning, Forward),
            (far_across, Reversed),
        ],
    )?;
    let side = a_flat_face(
        &mut builder,
        Point::new(-14.0, 0.0, 0.0),
        Vector::new(-1.0, 0.0, 0.0),
        &[
            (front_side, Forward),
            (high_across, Forward),
            (back_side, Reversed),
            (low_across, Reversed),
        ],
    )?;
    let top = a_flat_face(
        &mut builder,
        Point::new(0.0, 0.0, 22.0),
        Vector::new(0.0, 0.0, 1.0),
        &[
            (front_top, Forward),
            (far_across, Forward),
            (back_top, Reversed),
            (high_across, Reversed),
        ],
    )?;
    let front = a_flat_face(
        &mut builder,
        Point::new(0.0, -14.0, 0.0),
        Vector::new(0.0, -1.0, 0.0),
        &[
            (front_leaning, Forward),
            (front_top, Reversed),
            (front_side, Reversed),
        ],
    )?;
    let back = a_flat_face(
        &mut builder,
        Point::new(0.0, 14.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        &[
            (back_side, Forward),
            (back_top, Forward),
            (back_leaning, Reversed),
        ],
    )?;
    builder.add_shell(&[
        (leaning, Forward),
        (side, Forward),
        (top, Forward),
        (front, Forward),
        (back, Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn a_flat_face(
    builder: &mut SolidBuilder,
    point: Point,
    outward: Vector,
    steps: &[(DraftEdge, Orientation)],
) -> rcad_topo::Result<rcad_topo::DraftFace> {
    let plane = Plane::from_point_and_normal(point, outward).expect("a normal with a length");
    let wire = a_wire(builder, steps)?;
    builder.add_face(Surface::Plane(plane), &[wire])
}

/// **A tube cut by a block whose floor crosses it and whose top clears it is
/// refused**, both ways round: the tube of the test above, holed before it is
/// cut, and the block `[−14, 14] × [−14, 14] × [0, 22]`. The block's top,
/// `z = 22`, is two above the tube's, yet the fuse cuts it along the circles of
/// radius 4 and 12 where the two pipes would cross it, and cannot find a point
/// inside the ring between them to place it by (Q634, with a print in
/// `select::choose`). Nothing of the tube reaches `z = 22`, so there is nothing
/// there to cut along. The wedge above, holding the holed tube, refuses the
/// same way with its top at 21, 22, 24 or 30. A bar of radius 12 gets the
/// circle of radius 12 on the block's top as well, and is answered; so is the
/// tube against a block reaching down past it, `z = −30` to `22`.
#[test]
fn a_tube_cut_by_a_block_whose_top_clears_it_is_refused_on_circles_its_walls_never_reach() {
    for swapped in [false, true] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = model
            .add_round_bar(12.0, 30.0, &at(0.0, 0.0, -10.0))
            .expect("a bar of radius twelve");
        let through = model
            .add_round_bar(4.0, 34.0, &at(0.0, 0.0, -12.0))
            .expect("a bar of radius four");
        let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
        let block = a_block(
            &mut model,
            &[[-14.0, -14.0], [14.0, -14.0], [14.0, 14.0], [-14.0, 14.0]],
            0.0,
            22.0,
        );
        let refused = fused_in_order(&mut model, [tube, block], Rule::Intersection, swapped);
        let Err(Error::Undecidable { records }) = &refused else {
            panic!("swapped {swapped}: {refused:?}");
        };
        let [record] = records.records() else {
            panic!("swapped {swapped}: {records:?}");
        };
        assert_eq!(record.predicate(), "a point inside a piece of a face");
        let corners: Vec<Option<[f64; 3]>> = record
            .inputs()
            .iter()
            .map(|corner| corner.exact_coordinates())
            .collect();
        assert_eq!(
            corners,
            [Some([0.0, 4.0, 22.0]), Some([0.0, 12.0, 22.0])],
            "swapped {swapped}"
        );
    }
}

/// **Corners that sweep no area and are not on one line.** A bar of radius 10
/// cut to the region above three sides: `(6, −8)` to `(2, −9)`, on to
/// `(−2, −7)`, on to `(−6, −8)`. The cap that is left runs the long way round
/// the circle from `(−6, −8)` to `(6, −8)` and back along those three sides,
/// so its four corners, as written, go round a bow tie whose two halves cancel
/// exactly, and they are not on one line. Every rule, both ways round, is
/// answered and holds what the circle and the prism say.
///
/// **But the corners do not come back as written.** Three of the four are an
/// ulp or two off (`(6, −8)` as `(5.999999999999999, −8.000000000000002)`),
/// so the bow tie the fuse writes does not cancel: a ring with these corners
/// winds one way or the other and is placed by that, not by
/// `holds_the_other_rings`. Family 1 needs corners that cancel exactly after
/// the fuse has written them. This cap is also one face that nothing joins,
/// so it never reaches `rings_round`; the next test is the probe that would.
#[test]
fn a_bar_cut_to_a_bow_tie_cap_is_answered_with_corners_that_no_longer_cancel() {
    let [a, b, c, d] = BOW_TIE_CORNERS;
    let swept = [[a, b], [b, c], [c, d], [d, a]]
        .iter()
        .map(|[p, q]| p[0] * q[1] - p[1] * q[0])
        .sum::<f64>();
    assert_eq!(swept, 0.0, "the bow tie's two halves cancel");
    let above_the_chord = 100.0 * PI - (100.0 * 0.8_f64.acos() - 48.0);
    let (round, sides) = (400.0 * PI, 6.0 * 2280.0);
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let round_bar = model
            .add_round_bar(10.0, 4.0, &Isometry::identity())
            .expect("a bar of radius ten");
        let prism = a_block(&mut model, &BOW_TIE_SIDES, -1.0, 6.0);
        let answer = fused_in_order(&mut model, [round_bar, prism], rule, swapped)
            .unwrap_or_else(|why| panic!("{rule:?}, swapped {swapped}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("{rule:?}, swapped {swapped}: {answer:?}");
        };
        let shared = 4.0 * above_the_chord;
        let volume = match (rule, swapped) {
            (Rule::Union, _) => round + sides - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, false) => round - shared,
            (Rule::Difference, true) => sides - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        let what = format!("{rule:?}, swapped {swapped}");
        assert_holds(&model, solid, volume, &what);
        if rule == Rule::Intersection {
            let mut on_top: Vec<[f64; 3]> = Vec::new();
            for face in model.solid(solid).expect("this model made the key").faces() {
                for wire in face.wires() {
                    for coedge in wire.coedges() {
                        let at = coedge.start().point().coordinates();
                        if at[2] == 4.0 && !on_top.contains(&at) {
                            on_top.push(at);
                        }
                    }
                }
            }
            let written = BOW_TIE_CORNERS.map(|[x, y]| {
                *on_top
                    .iter()
                    .find(|at| (at[0] - x).abs() < 1e-9 && (at[1] - y).abs() < 1e-9)
                    .unwrap_or_else(|| panic!("{what}: no corner near ({x}, {y}) in {on_top:?}"))
            });
            let [a, b, c, d] = written;
            let swept = [[a, b], [b, c], [c, d], [d, a]]
                .iter()
                .map(|[p, q]| p[0] * q[1] - p[1] * q[0])
                .sum::<f64>();
            assert_ne!(written, BOW_TIE_CORNERS.map(|[x, y]| [x, y, 4.0]), "{what}");
            assert_ne!(swept, 0.0, "{what}: {written:?}");
        }
    }
}

/// The same bar with a bar of radius 2 through `(0, 4)` taken out of it first,
/// so the bow tie cap has a hole, and a block flush with it would join its
/// pieces into a face with two rings that may enclose no area: the bow tie, if
/// its corners cancelled, and the circle. Taking a ball's pocket out of the bow
/// tie instead is refused, with `AFaceTrimmedByACurveThisCannotFollow`.
///
/// **Cutting the holed bar to the bow tie is answered**, under every rule both
/// ways round. It was refused `CannotTellTheCellsOfAFaceApart` while the bar's
/// floor was cut into cells in `fuse/arrange.rs` (Q616): the hole's circle
/// agrees with every piece of the floor about every closed curve, and the piece
/// above the bow tie has the bar's arc for a side, so its corners — the bow
/// tie, which winds no times round the hole — were not its shape. Counted with
/// its arc, which bulges up past the hole from the chord `y = −8`, it winds
/// once round the hole and every piece below the bow tie none (queue row Q1053).
///
/// The hole takes `16π` out of what the bar and the prism share and nothing out
/// of the prism, so the bar less the prism is what it was without the hole, the
/// prism less the bar holds `16π` more, and the union is the same.
#[test]
fn a_holed_bar_cut_to_a_bow_tie_cap_is_answered_under_every_rule() {
    let above_the_chord = 100.0 * PI - (100.0 * 0.8_f64.acos() - 48.0);
    let (round, sides, hole) = (400.0 * PI, 6.0 * 2280.0, 16.0 * PI);
    let shared = 4.0 * above_the_chord - hole;
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let whole = model
            .add_round_bar(10.0, 4.0, &Isometry::identity())
            .expect("a bar of radius ten");
        let through = model
            .add_round_bar(2.0, 6.0, &at(0.0, 4.0, -1.0))
            .expect("a bar of radius two");
        let holed = the_one_solid(&mut model, whole, through, Rule::Difference);
        let prism = a_block(&mut model, &BOW_TIE_SIDES, -1.0, 6.0);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [holed, prism], rule, swapped);
        let volume = match (rule, swapped) {
            (Rule::Union, _) => round - hole + sides - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, false) => round - hole - shared,
            (Rule::Difference, true) => sides - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), volume, &what);
    }
}

/// **Corners that cancel exactly, written by hand: the fuse cuts the arc
/// wherever a wall of the cap would cross it, and the ring it hands
/// `rings_round` winds.** A bar of radius 20 and height 4, cut to the piece
/// between its arc from `(20, 0)` to `(0, 20)` and a staircase down through
/// `(0, 18)`, `(8, 18)`, `(8, 14)`, `(12, 14)` and `(12, 0)`, built face by face
/// ([`a_bar_cut_to`]). The area under the staircase is
/// `8·18 + 4·14 = 200`, the same as under the arc's chord `x + y = 20`, so the
/// seven corners sweep no area, and they are not on one line. Every corner is a
/// whole number and every wall square to an axis, so nothing written rounds:
/// read back off the model, the cap's corners are the seven as written, and
/// their sweep is exactly 0. The cap holds what the arc's segment holds,
/// `100π − 200`.
///
/// Against a bar of radius 2 at `(15, 8)`, flush with both of its caps or
/// standing on its top, the cap is cut in two along that bar's circle, and
/// where the two pieces are one face again the fuse joins them, so the cap
/// reaches `rings_round`. **It does not reach it with these corners.** The
/// fuse also cuts the bar's wall along every wall of the staircase that would
/// cross it, `x = 8` at `(8, √336)`, `x = 12` at `(12, 16)`, `y = 14` at
/// `(√204, 14)` and `y = 18` at `(√76, 18)`, though none of those walls reaches
/// the arc. Where pieces of one pipe meet, the corner stays
/// (`what_is_left` in `fuse/assemble.rs`), so the joined cap comes back with
/// eleven corners, four of them on the arc. Their sweep is what the corners
/// on the arc add over its chord, so the ring winds with the plane and is the
/// outline by that, before any ring enclosing nothing is looked at. Over the
/// twelve fuses `rings_round` is reached four times, and each time one ring
/// winds with the plane and `holds_the_other_rings` is never asked
/// (breakpoints on `outline = Some(ring)` and on the closure that asks it,
/// Q641).
///
/// **So a ring of one arc and straight sides does not reach `rings_round` with
/// its corners cancelling**, wherever the fuse cuts the arc along every wall
/// the way it does here. If every side ran on the centre's side of every
/// chord between corners on the arc, the corners would go round a polygon
/// that does not cross itself, which sweeps some area. So some side runs
/// between a chord and its arc. The line along it meets the chord's line only
/// once, so it leaves that piece of the disk through the arc, and the fuse
/// puts a corner there: the arc that chord stood for is not one arc of the
/// ring after all. Nor can the arc be longer than half its circle with no
/// corner on it: the bar cut to the step from `(−8, −6)` along `y = −6` to
/// `(−1, −6)`, down to `(−1, −8)` and along `y = −8` to `(6, −8)`, the arc
/// running the long way round, built face by face, is refused
/// `AFaceTrimmedByACurveThisCannotFollow` against a bar of radius 2 standing
/// through it, under every rule both ways round
/// (`a_step_cap_whose_arc_runs_the_long_way_round_is_refused_by_the_next_fuse`,
/// `docs/decisions.md` §224.1). The same step
/// cut out of a bar by a prism comes back with a corner on the arc at
/// `(−1, √99)`, where `x = −1` crosses it, and one at `(8, −6)`, where
/// `y = −6` does (Q641's probes). Not tried: sides along other pipes, which
/// the line argument does not cover.
///
/// What each fuse answers is pinned: the union, the intersection and both
/// differences, both ways round, with the second bar in both places. Five of
/// the twelve are refused `ResultIsNotManifold { count: 1 }`, among them the
/// flush union with the second bar handed over first, whose other order is
/// answered.
#[test]
fn a_staircase_cap_whose_corners_cancel_is_cut_on_its_arc_by_its_own_walls() {
    let as_written = STAIRCASE.map(|[x, y]| [x, y, 4.0]);
    let staircase_volume = 4.0 * (100.0 * PI - 200.0);
    {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let staircase = a_bar_cut_to(&mut model, 20.0, &STAIRCASE)
            .unwrap_or_else(|why| panic!("the staircase: {why:?}"));
        assert_holds(&model, staircase, staircase_volume, "the staircase");
        let corners = the_outline_of_the_cap(&model, staircase, as_written[0]);
        assert_eq!(corners, as_written, "read back as written");
        assert_eq!(swept_by(&corners), 0.0, "the staircase's corners cancel");
    }

    let on_the_arc = [
        [8.0, 336.0_f64.sqrt()],
        [12.0, 16.0],
        [204.0_f64.sqrt(), 14.0],
        [76.0_f64.sqrt(), 18.0],
    ];
    for on_top in [false, true] {
        for (rule, swapped) in every_rule_both_ways() {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let staircase = a_bar_cut_to(&mut model, 20.0, &STAIRCASE)
                .unwrap_or_else(|why| panic!("the staircase: {why:?}"));
            let pin = if on_top {
                model.add_round_bar(2.0, 2.0, &at(15.0, 8.0, 4.0))
            } else {
                model.add_round_bar(2.0, 4.0, &at(15.0, 8.0, 0.0))
            }
            .expect("a bar of radius two");
            let what = format!("{rule:?}, swapped {swapped}, on top {on_top}");
            let answer = fused_in_order(&mut model, [staircase, pin], rule, swapped);
            let pin_volume = if on_top { 8.0 * PI } else { 16.0 * PI };
            let with_the_cap = match (on_top, rule, swapped) {
                (false, Rule::Union, false) | (true, Rule::Difference, false) => {
                    let solid = the_only(answer, &what);
                    assert_holds(&model, solid, staircase_volume, &what);
                    solid
                }
                (false, Rule::Intersection, true) | (true, Rule::Difference, true) => {
                    assert_holds(&model, the_only(answer, &what), pin_volume, &what);
                    continue;
                }
                (false, Rule::Difference, true) | (true, Rule::Intersection, _) => {
                    let solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
                    assert_eq!(solids, Vec::new(), "{what}: no solid is shared");
                    continue;
                }
                (false, Rule::Union | Rule::Intersection, _)
                | (false, Rule::Difference, false)
                | (true, Rule::Union, _) => {
                    assert!(
                        matches!(answer, Err(Error::ResultIsNotManifold { count: 1, .. })),
                        "{what}: {answer:?}"
                    );
                    continue;
                }
                (_, other, _) => panic!("not a rule this probes: {other:?}"),
            };
            let written = the_outline_of_the_cap(&model, with_the_cap, as_written[0]);
            for corner in as_written {
                assert!(
                    written.contains(&corner),
                    "{what}: {corner:?} is not in {written:?}"
                );
            }
            for [x, y] in on_the_arc {
                assert!(
                    written
                        .iter()
                        .any(|at| (at[0] - x).abs() < 1e-12 && (at[1] - y).abs() < 1e-12),
                    "{what}: no corner at ({x}, {y}) in {written:?}"
                );
            }
            assert_eq!(written.len(), 11, "{what}: {written:?}");
            assert!(swept_by(&written) > 1.0, "{what}: {written:?}");
        }
    }
}

/// The staircase of the test above against a bar of radius 2 at `(15, 3)`
/// standing through it, height 6 from `z = −1`, is answered under every rule
/// both ways round. It was refused `CannotTellTheCellsOfAFaceApart`, for the
/// reason the holed bar cut to Q616's bow tie was
/// (`a_holed_bar_cut_to_a_bow_tie_cap_is_answered_under_every_rule`): the
/// circle's walk on a cap agrees with every piece of it, and the piece it is in
/// has the arc for a side, whose corners sweep no area. Counted with the arc it
/// winds once round the circle (queue row Q1053).
///
/// The bar is `24π` and shares `16π` with the staircase, all of the bar
/// between its caps, so the bar less the staircase is its two ends, `4π` each.
#[test]
fn a_staircase_cap_whose_corners_cancel_is_given_a_hole_under_every_rule() {
    let staircase_volume = 4.0 * (100.0 * PI - 200.0);
    let (pin, shared) = (24.0 * PI, 16.0 * PI);
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let staircase = a_bar_cut_to(&mut model, 20.0, &STAIRCASE)
            .unwrap_or_else(|why| panic!("the staircase: {why:?}"));
        let through = model
            .add_round_bar(2.0, 6.0, &at(15.0, 3.0, -1.0))
            .expect("a bar of radius two");
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [staircase, through], rule, swapped);
        match (rule, swapped) {
            (Rule::Union, _) => assert_holds(
                &model,
                the_only(answer, &what),
                staircase_volume + pin - shared,
                &what,
            ),
            (Rule::Intersection, _) => {
                assert_holds(&model, the_only(answer, &what), shared, &what);
            }
            (Rule::Difference, false) => assert_holds(
                &model,
                the_only(answer, &what),
                staircase_volume - shared,
                &what,
            ),
            (Rule::Difference, true) => {
                for end in the_two_smaller_first(&model, answer, &what) {
                    assert_holds(&model, end, 4.0 * PI, &what);
                }
            }
            (other, _) => panic!("not a rule this probes: {other:?}"),
        }
    }
}

/// The step of [`LONG_STEP`] holds what the circle above the chord `y = −8`
/// holds, `75π + 50` on its cap, and its four corners, read back as written,
/// sweep no area. Its arc runs three quarters of the way round with no corner
/// on it, and no fuse that would join its cap gets that far: taking a bar of
/// radius 2 at `(3, 2)` out of it is refused while its cap is cut, under every
/// rule, both ways round (`docs/decisions.md` §224.1: an arc longer than half
/// its circle has its centre in front of any picker that holds the whole arc).
#[test]
fn a_step_cap_whose_arc_runs_the_long_way_round_is_refused_by_the_next_fuse() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let step = a_bar_cut_to(&mut model, 10.0, &LONG_STEP)
        .unwrap_or_else(|why| panic!("the step: {why:?}"));
    assert_holds(&model, step, 4.0 * (75.0 * PI + 50.0), "the step");
    let corners = the_outline_of_the_cap(&model, step, [6.0, -8.0, 4.0]);
    assert_eq!(
        corners,
        LONG_STEP.map(|[x, y]| [x, y, 4.0]),
        "read back as written"
    );
    assert_eq!(swept_by(&corners), 0.0, "the step's corners cancel");

    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let step = a_bar_cut_to(&mut model, 10.0, &LONG_STEP)
            .unwrap_or_else(|why| panic!("the step: {why:?}"));
        let through = model
            .add_round_bar(2.0, 6.0, &at(3.0, 2.0, -1.0))
            .expect("a bar of radius two");
        let refused = fused_in_order(&mut model, [step, through], rule, swapped);
        let Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) = refused else {
            panic!("{rule:?}, swapped {swapped}: {refused:?}");
        };
        let the_step = model.solid(step).expect("this model made the key");
        assert!(
            the_step.faces().any(|mine| mine.key() == face),
            "{rule:?}, swapped {swapped}: the refusal names a face of the step"
        );
    }
}

/// What the corners sweep round `z`, twice over: the shoelace sum.
fn swept_by(corners: &[[f64; 3]]) -> f64 {
    (0..corners.len())
        .map(|at| {
            let [p, q] = [corners[at], corners[(at + 1) % corners.len()]];
            p[0] * q[1] - p[1] * q[0]
        })
        .sum()
}

/// The corners of the outline of `solid`'s face on `z = 4` that holds the
/// corner `start`, as its ring runs from there.
fn the_outline_of_the_cap(model: &Model, solid: SolidKey, start: [f64; 3]) -> Vec<[f64; 3]> {
    let solid = model.solid(solid).expect("this model made the key");
    let mut outlines = solid.faces().filter_map(|face| {
        let corners: Vec<[f64; 3]> = face
            .outer_wire()
            .points()
            .map(|point| point.coordinates())
            .collect();
        let from = corners.iter().position(|&corner| corner == start)?;
        corners
            .iter()
            .all(|corner| corner[2] == 4.0)
            .then(|| [&corners[from..], &corners[..from]].concat())
    });
    let outline = outlines.next().expect("a cap holds the staircase's corner");
    assert!(outlines.next().is_none(), "one cap holds it");
    outline
}

/// A round bar of `radius` about the `z` axis, height 4, cut to the piece
/// whose cap runs round `corners`: along the bar's circle anticlockwise from
/// the first to the second, picked by the side of the line between them that
/// is on the right going from the first to the second, then straight from
/// corner to corner back to the first. Built face by face, so every corner is
/// where it is written.
fn a_bar_cut_to(
    model: &mut Model,
    radius: f64,
    corners: &[[f64; 2]],
) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let mut builder = SolidBuilder::new();
    let mut low = Vec::with_capacity(corners.len());
    let mut high = Vec::with_capacity(corners.len());
    for &[x, y] in corners {
        low.push(builder.add_vertex(Point::new(x, y, 0.0))?);
        high.push(builder.add_vertex(Point::new(x, y, 4.0))?);
    }
    let top = Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
        .expect("a unit normal");
    let bottom = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("a unit normal");
    let [[from_x, from_y], [to_x, to_y]] = [corners[0], corners[1]];
    let the_arc = Picker::InFrontOf(
        HalfSpace::new(
            Vector::new(to_y - from_y, from_x - to_x, 0.0),
            from_x * to_y - from_y * to_x,
        )
        .expect("a normal with a length"),
    );
    let next = |index: usize| (index + 1) % corners.len();
    let mut up = Vec::with_capacity(corners.len());
    let mut along_the_top = Vec::with_capacity(corners.len());
    let mut along_the_bottom = Vec::with_capacity(corners.len());
    for index in 0..corners.len() {
        up.push(builder.add_edge(low[index], high[index], EdgeRun::Straight)?);
        let [run_high, run_low] = if index == 0 {
            [top, bottom].map(|plane| EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(plane),
                picked_by: smallvec![the_arc],
            })
        } else {
            [EdgeRun::Straight, EdgeRun::Straight]
        };
        along_the_top.push(builder.add_edge(high[index], high[next(index)], run_high)?);
        along_the_bottom.push(builder.add_edge(low[index], low[next(index)], run_low)?);
    }
    let mut faces = Vec::with_capacity(corners.len() + 2);
    let top_steps: Vec<_> = along_the_top.iter().map(|&edge| (edge, Forward)).collect();
    let top_wire = a_wire(&mut builder, &top_steps)?;
    faces.push(builder.add_face(Surface::Plane(top), &[top_wire])?);
    let bottom_steps: Vec<_> = along_the_bottom
        .iter()
        .rev()
        .map(|&edge| (edge, Reversed))
        .collect();
    let bottom_wire = a_wire(&mut builder, &bottom_steps)?;
    faces.push(builder.add_face(Surface::Plane(bottom), &[bottom_wire])?);
    for index in 0..corners.len() {
        let wire = a_wire(
            &mut builder,
            &[
                (along_the_bottom[index], Forward),
                (up[next(index)], Forward),
                (along_the_top[index], Reversed),
                (up[index], Reversed),
            ],
        )?;
        let surface = if index == 0 {
            Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"))
        } else {
            let ([x, y], [to_x, to_y]) = (corners[index], corners[next(index)]);
            Surface::Plane(
                Plane::from_point_and_normal(
                    Point::new(x, y, 0.0),
                    Vector::new(to_y - y, x - to_x, 0.0),
                )
                .expect("a side with a length"),
            )
        };
        faces.push(builder.add_face(surface, &[wire])?);
    }
    let shell: Vec<_> = faces.into_iter().map(|face| (face, Forward)).collect();
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The corners of the staircase cap, in the order its ring runs: the arc of
/// radius 20 from `(20, 0)` to `(0, 20)`, then the six straight sides.
const STAIRCASE: [[f64; 2]; 7] = [
    [20.0, 0.0],
    [0.0, 20.0],
    [0.0, 18.0],
    [8.0, 18.0],
    [8.0, 14.0],
    [12.0, 14.0],
    [12.0, 0.0],
];

/// The bar of radius 10 cut to the step of the test above it, its arc the long
/// way round from `(6, −8)` to `(−8, −6)`: the polygon through its corners
/// crosses itself where the chord meets the riser, and the two halves cancel.
const LONG_STEP: [[f64; 2]; 4] = [[6.0, -8.0], [-8.0, -6.0], [-1.0, -6.0], [-1.0, -8.0]];

/// **A closed side that is one of several.** A tube, radius 7 less a bar of
/// radius 3 through it, and a bar of radius 1 at `(2, 0)` inside its hole,
/// touching the inner wall along the line through `(3, 0)` — the seam corner
/// of both circles. United, the hole left in the cap would be one ring
/// through `(3, 0)` twice: the inner circle and the small bar's circle, each a
/// closed side of the same ring, which `holds_the_other_rings` answers `None`
/// for. A ring can hold a closed side among others only by coming back to that
/// side's one corner, so this family needs a touch at a corner, and a touch
/// along a line is how two round solids make one.
///
/// **The touch is followed now** (queue row Q649): the line runs up the bars'
/// axis, and the small bar lies inside the hole's pipe on both sides of it. It
/// used to be refused `Undecidable` on *which way a crossing runs* under every
/// rule (Q616), the gap §191.5 measures for bars touching along a line.
///
/// So the intersection is no solid, the tube less the bar is the tube, and the
/// bar less the tube is the bar. **The union is the tube and the bar as they
/// stand**, two lumps. It used to be refused `ResultIsNotManifold { count: 4 }`
/// on the touching line, kept on both sides in both solids, until queue row
/// Q760 parted that edge into one per input face in `fuse/assemble.rs`.
///
/// **The ring above is still unreached.** The two caps on each end plane meet
/// only at the touching line's corner and share no edge, so nothing joins them
/// into one face, and each lump keeps its own cap.
#[test]
fn a_bar_touching_a_tubes_inner_wall_is_answered_under_every_rule() {
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = model
            .add_round_bar(7.0, 4.0, &Isometry::identity())
            .expect("a bar of radius seven");
        let through = model
            .add_round_bar(3.0, 6.0, &at(0.0, 0.0, -1.0))
            .expect("a bar of radius three");
        let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
        let inside = model
            .add_round_bar(1.0, 4.0, &at(2.0, 0.0, 0.0))
            .expect("a bar of radius one");
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [tube, inside], rule, swapped);
        match (rule, swapped) {
            (Rule::Union, _) => {
                let [bar, tube] = the_two_smaller_first(&model, answer, &what);
                assert_holds(&model, tube, 4.0 * 40.0 * PI, "the tube in the union");
                assert_holds(&model, bar, 4.0 * PI, "the bar in the union");
            }
            (Rule::Intersection, _) => assert_eq!(
                answer.unwrap_or_else(|why| panic!("{what} refused: {why}")),
                Vec::new(),
                "{what}: the bar is in the hole, so the two share no solid"
            ),
            (Rule::Difference, false) => assert_holds(
                &model,
                the_only(answer, &what),
                4.0 * 40.0 * PI,
                "the tube less the bar",
            ),
            (Rule::Difference, true) => assert_holds(
                &model,
                the_only(answer, &what),
                4.0 * PI,
                "the bar less the tube",
            ),
            (other, _) => panic!("not a rule this probes: {other:?}"),
        }
    }
}

fn the_only(answer: rcad_topo::Result<Vec<SolidKey>>, what: &str) -> SolidKey {
    let solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
    assert_eq!(solids.len(), 1, "{what}: {solids:?}");
    solids[0]
}

/// The two solids of an answer, the one with less inside it first.
fn the_two_smaller_first(
    model: &Model,
    answer: rcad_topo::Result<Vec<SolidKey>>,
    what: &str,
) -> [SolidKey; 2] {
    let solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
    let mut two: [SolidKey; 2] = solids
        .try_into()
        .unwrap_or_else(|solids| panic!("{what}: two lumps, got {solids:?}"));
    let volume = |solid: &SolidKey| {
        model
            .solid(*solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} does not measure: {why}"))
            .volume()
    };
    two.sort_by(|one, other| volume(one).total_cmp(&volume(other)));
    two
}

/// The corners of the bow tie cap, in the order its ring runs: two on the
/// circle of radius 10, then two inside it.
const BOW_TIE_CORNERS: [[f64; 2]; 4] = [[-6.0, -8.0], [6.0, -8.0], [2.0, -9.0], [-2.0, -7.0]];

/// A region of area 2280 whose sides through `(2, −9)` and `(−2, −7)` are
/// three of the bow tie's, carried out past the circle of radius 10.
const BOW_TIE_SIDES: [[f64; 2]; 6] = [
    [30.0, -2.0],
    [30.0, 30.0],
    [-30.0, 30.0],
    [-30.0, -14.0],
    [-2.0, -7.0],
    [2.0, -9.0],
];

fn every_rule_both_ways() -> impl Iterator<Item = (Rule, bool)> {
    [Rule::Union, Rule::Intersection, Rule::Difference]
        .into_iter()
        .flat_map(|rule| [false, true].map(move |swapped| (rule, swapped)))
}

/// Two bars of radius 10 and height 4 at `−apart` and `apart`, each less a bar
/// of radius 2 standing through the origin.
fn two_bars_holed_through_the_origin(model: &mut Model, [x, y]: [f64; 2]) -> [SolidKey; 2] {
    [-1.0, 1.0].map(|side| {
        let whole = model
            .add_round_bar(10.0, 4.0, &at(side * x, side * y, 0.0))
            .expect("a bar of radius ten");
        let hole = model
            .add_round_bar(2.0, 6.0, &at(0.0, 0.0, -1.0))
            .expect("a bar of radius two");
        the_one_solid(model, whole, hole, Rule::Difference)
    })
}

fn a_block(model: &mut Model, outline: &[[f64; 2]], bottom: f64, height: f64) -> SolidKey {
    let profile = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(&profile, height, &at(0.0, 0.0, bottom))
        .expect("a block")
}

fn fused_in_order(
    model: &mut Model,
    [one, other]: [SolidKey; 2],
    rule: Rule,
    swapped: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let [one, other] = if swapped { [other, one] } else { [one, other] };
    model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}
