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
//! answers `None` for and pins what the fuse does with each. Each one is
//! refused before `rings_round`, and each test names the stage: that was
//! read with a breakpoint on `select::choose`, on `rings_round` and on
//! `holds_the_other_rings` (Q597), which the two tests above do reach. The
//! one probe that is answered, two holed bars turned a quarter, reaches
//! `holds_the_other_rings` too, but not by the family it was built for (Q616).

use std::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point,
    Scale, Surface, Torus, Vector,
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
/// Written out the way `rings.rs` writes a bend: the wall used reversed, every
/// number turned exactly, and each arc named by the half-spaces and the pipe of
/// the ring's own radius that pick it.
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
        (wall_face, Orientation::Reversed),
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
/// **That is not reached.** Every rule, both ways round, is refused in
/// `select::choose` while a point is found inside a piece of a cap, and the
/// three corners its record names are those three on `x = 0`.
/// `join_up_what_is_one_face` is never reached (a breakpoint on it, and on
/// `select::choose`, under the union, Q616).
#[test]
fn two_holed_bars_whose_corners_lie_on_one_line_are_refused_while_a_piece_is_placed() {
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = two_bars_holed_through_the_origin(&mut model, [6.0, 0.0]);
        let refused = fused_in_order(&mut model, pair, rule, swapped);
        assert_undecidable_on(
            &refused,
            "a point inside a piece of a face",
            &format!("{rule:?}, swapped {swapped}"),
        );
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
/// puts it there is refused before this.
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
/// **Neither gets that far**, because this pair, the first fuse each needs,
/// is refused under every rule, both ways round, in `select::choose` while a
/// point is found inside a piece of the block's top: the record names the two
/// corners where the top's chord meets the cone. Moving the wall to `x = −11`
/// or the top above the cone's base is refused the same way (Q616).
#[test]
fn a_cone_cut_by_a_block_wall_off_its_axis_is_refused_while_a_piece_is_placed() {
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cone = cones::a_pointed_cone(&mut model, 1.0, 20.0);
        let block = a_block(
            &mut model,
            &[[-12.0, -24.0], [24.0, -24.0], [24.0, 24.0], [-12.0, 24.0]],
            -1.0,
            19.0,
        );
        let refused = fused_in_order(&mut model, [cone, block], rule, swapped);
        assert_undecidable_on(
            &refused,
            "a point inside a piece of a face",
            &format!("{rule:?}, swapped {swapped}"),
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
/// so the bow tie cap would have a hole, and a block flush with it would join
/// its pieces into a face with two rings that may enclose no area: the bow
/// tie, if its corners cancelled, and the circle. Taking a
/// ball's pocket out of the bow tie instead is refused as well, with
/// `AFaceTrimmedByACurveThisCannotFollow`.
///
/// **Refused before any of that.** Cutting the holed bar to the bow tie is
/// refused under every rule, both ways round, while a face is cut into cells
/// in `fuse/arrange.rs` (Q616).
#[test]
fn a_holed_bar_cut_to_a_bow_tie_cap_is_refused_while_a_face_is_cut_into_cells() {
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let round = model
            .add_round_bar(10.0, 4.0, &Isometry::identity())
            .expect("a bar of radius ten");
        let through = model
            .add_round_bar(2.0, 6.0, &at(0.0, 4.0, -1.0))
            .expect("a bar of radius two");
        let holed = the_one_solid(&mut model, round, through, Rule::Difference);
        let prism = a_block(&mut model, &BOW_TIE_SIDES, -1.0, 6.0);
        let refused = fused_in_order(&mut model, [holed, prism], rule, swapped);
        assert!(
            matches!(refused, Err(Error::CannotTellTheCellsOfAFaceApart { .. })),
            "{rule:?}, swapped {swapped}: {refused:?}"
        );
    }
}

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
/// **Refused before any of that**, under every rule, both ways round, while
/// `fuse/arrange.rs` reads which way the crossing runs where the two walls
/// touch — the same gap §191.5 measures for bars touching along a line (Q616).
#[test]
fn a_bar_touching_a_tubes_inner_wall_is_refused_while_the_touch_is_followed() {
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
        let refused = fused_in_order(&mut model, [tube, inside], rule, swapped);
        assert_undecidable_on(
            &refused,
            "which way a crossing runs",
            &format!("{rule:?}, swapped {swapped}"),
        );
    }
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

fn assert_undecidable_on(refused: &rcad_topo::Result<Vec<SolidKey>>, predicate: &str, what: &str) {
    let Err(Error::Undecidable { records }) = refused else {
        panic!("{what}: {refused:?}");
    };
    assert_eq!(
        records.records().first().map(|record| record.predicate()),
        Some(predicate),
        "{what}: {refused:?}"
    );
}
