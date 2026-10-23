//! A stepped shaft cut in two on a plane beside or through its axis, both
//! halves turned about that axis, and the halves fused again by every rule
//! (`docs/decisions.md` §191.5, rows Q2228, Q2241, Q2487, Q2488, Q2613, Q2619
//! and Q2657).
//!
//! **The shape.** The shaft is a bar of radius 2 from `z = 0` to `z = 1`
//! united with a bar of radius 1 from `z = 1` to `z = 2`, so `5π` in all. One
//! copy less the block `x < c` is the half cut off, and another shared with
//! that block is the half kept, so the two lie flush along the plane `x = c`.
//! Both are then turned by the same angle about the shaft's axis, `z`.
//!
//! **What it holds, by hand.** The part of a disk of radius `r` on the side
//! `x < c` of a chord is `r² acos(−c / r) + c √(r² − c²)`, so the kept half is
//! that for radius 2 and for radius 1, one unit high each, and the half cut off
//! is the rest of `5π`. The halves only touch, so they unite into one solid of
//! both volumes, share nothing, and either less the other is itself.
//!
//! **What was wrong before Q2228.** Cutting the shaft splits its bottom face
//! where the narrow bar's pipe would cross it, and the fuse joins the two
//! pieces back into one face. That face's one ring runs round an arc of radius
//! 2 and back along the chord, through four corners all on the chord's line.
//! Unturned, those corners are exactly on one line, the polygon through them
//! sweeps nothing, and the ring, the only one, was taken as the outline.
//! Turned, the turn rounds them off the line, the polygon sweeps a sliver
//! whose sign is the rounding's, and where that came out against the plane
//! the ring was taken as a hole. With no outline left, `rings_round` in
//! `fuse/assemble.rs` panicked "none of its rings winds with the plane or
//! bounds a region holding the others", under both policies. Now a ring with
//! a curved side is not placed by its corners alone: where every ring winds
//! against the plane, the outline is among those with a curved side, and the
//! one such ring is it.
//!
//! **What was wrong before Q2241.** Bored from below before it is cut, the
//! kept half's bottom is that D and a hole. A round hole has one corner and
//! encloses nothing, and `rings_round` took a ring enclosing nothing as the
//! outline before it looked at a ring winding against the plane with a
//! curved side. So cut on `x = 0.5` and turned by 0.4, 1 or 2.5, the kept half
//! less the other was written with the hole as its bottom's outline and the D
//! as a hole, under both policies. Nothing refused it: the volume is a sum
//! over every ring, which does not care which is first, and
//! `Model::validate` does not check which of two rings holds the other where
//! one has fewer than three corners. Now both kinds of ring are candidates,
//! and the region each bounds says which is the outline.
//!
//! **What was wrong before Q2487 and Q2488.** Boring left corners on the
//! circles of the shaft where some surface of the bore crosses them, each a
//! rounding off that surface. A round hole across the narrow bar's radius no
//! longer leaves one (Q2619, below), nor does a square hole on the whole
//! shaft (Q2657, below), but the halves cut from the shaft with a square hole
//! still keep them. A cut then met that surface again there, as a
//! place where three surfaces meet whose box held the corner or lay within a
//! rounding of it, and asked the corner about the surface instead. A round
//! bore across the narrow bar's radius was refused `order along a crossing`
//! at the hole's corner, and a square one, turned or shifted, `side of plane`
//! where a side of the hole meets the wide bar's circle, under the certified
//! policy. Now a corner with two surfaces round it, a sample of the curve
//! they share, is taken as the one place where a third surface crosses that
//! curve within what writing the corner down cost.
//!
//! **What was wrong before Q2613 and Q2619.** A round bore across the narrow
//! bar's radius stops at `z = 0.5`, and the narrow bar starts at `z = 1`, so
//! the two pipes share no point of the solid. As whole pipes they cross in two
//! lines without end, at `(−0.955, ±0.2966)`, and no plane cuts such a line
//! off, so every face asked whether the lines lie wholly past a plane the
//! other pipe's faces lie behind, and every face said no. Boring split both
//! pipes in two faces along those lines, and put a corner where they cross
//! the circles at `z = 0`, `0.5`, `1` and `2`. Now each face is also told the
//! sides of planes every face on its own surface lies in, and leaves out a
//! crossing whose faces lie in a side that shares no point with one of those:
//! here `z ≤ 0.5` and `z ≥ 1`.
//!
//! **What was wrong before Q2657.** The square hole's side planes, run on past
//! the hole, cross the shaft's circles far from it: on the bottom and the
//! annulus at `(±1.977, ±0.3)`, `(−1.8, ±0.872)` and `(−1.2, ±1.6)`, and on the
//! narrow bar's circles at `(±0.954, ±0.3)`. Boring left a corner at each,
//! though no face of the hole is there. The wide bar's pipe was joined back up
//! with a seam kept along the ruling at `(−1.977, −0.3)`, and that seam's
//! corner on each rim kept the rims in pieces at all of those crossings. Now a
//! pipe band whose rings all go right round closes them up, as a cone's did,
//! and needs no seam. The halves cut from the bored shaft still keep every one
//! of those corners.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The turns about `z` the halves are placed with: unturned, and the three
/// the row found panicking.
const TURNS: [f64; 4] = [0.0, 0.4, 1.0, 2.5];

/// The stepped shaft cut on `x = 0.5`, beside its axis and through both bars.
#[test]
fn a_stepped_shaft_cut_beside_its_axis_and_turned_fuses_with_its_other_half() {
    for turn in TURNS {
        every_fuse_of_the_halves_answers(
            &format!("cut at x = 0.5, turned {turn}"),
            0.5,
            CutOffFrom::TheSteppedShaft,
            &turned(turn),
        );
    }
}

/// The stepped shaft cut through its axis on `x = 0`, then shifted by
/// `(7, −5, 2.5)` and turned about `z`, so it is turned about a line it no
/// longer stands on.
#[test]
fn a_stepped_shaft_cut_through_its_axis_shifted_and_turned_fuses_with_its_other_half() {
    for turn in TURNS {
        every_fuse_of_the_halves_answers(
            &format!("cut at x = 0, shifted, turned {turn}"),
            0.0,
            CutOffFrom::TheSteppedShaft,
            &shifted_and_turned(turn),
        );
    }
}

/// The half cut off is of a plain bar of radius 2 from `z = 0` to `z = 2`, and
/// the half kept is of the stepped shaft, both cut on `x = 0.5`.
#[test]
fn a_bars_half_and_a_stepped_shafts_half_turned_fuse_by_every_rule() {
    for turn in TURNS {
        every_fuse_of_the_halves_answers(
            &format!("a bar's half against the shaft's, turned {turn}"),
            0.5,
            CutOffFrom::AWideBar,
            &turned(turn),
        );
    }
}

/// The stepped shaft bored from below with a bar of radius 0.3 about
/// `(−1.5, 0)`, clear of the narrow bar's radius, then cut and turned as above:
/// the kept half's bottom is the D with a round hole of one corner, and every
/// fuse keeps the D as that bottom's outline.
#[test]
fn a_stepped_shaft_bored_with_a_round_hole_cut_and_turned_keeps_the_d_as_its_bottoms_outline() {
    let bore = Bore::ARoundHole {
        centre: [-1.5, 0.0],
        radius: 0.3,
    };
    for turn in TURNS {
        every_fuse_of_the_bored_halves_answers(
            &format!("a round hole, cut at x = 0.5, turned {turn}"),
            0.5,
            bore,
            &turned(turn),
        );
        every_fuse_of_the_bored_halves_answers(
            &format!("a round hole, cut at x = 0, shifted, turned {turn}"),
            0.0,
            bore,
            &shifted_and_turned(turn),
        );
    }
}

/// The stepped shaft bored from below with a block 0.6 on a side about
/// `(−1.5, 0)`, then cut and turned as above: every fuse answers with the D as
/// its bottom's outline, under both policies. Turned, or cut through the axis
/// and shifted, every fuse under the certified policy was refused
/// `Undecidable` at `side of plane`, on a point where the plane of one of the
/// hole's sides meets the wide bar's pipe on the bottom's plane (Q2488).
#[test]
fn a_stepped_shaft_bored_with_a_square_hole_cut_and_turned_keeps_the_d_as_its_bottoms_outline() {
    for turn in TURNS {
        every_fuse_of_the_bored_halves_answers(
            &format!("a square hole, cut at x = 0.5, turned {turn}"),
            0.5,
            Bore::ASquareHole,
            &turned(turn),
        );
        every_fuse_of_the_bored_halves_answers(
            &format!("a square hole, cut at x = 0, shifted, turned {turn}"),
            0.0,
            Bore::ASquareHole,
            &shifted_and_turned(turn),
        );
    }
}

/// Bored with a bar whose circle straddles the circle of radius 1 the narrow
/// bar stands on, the cut on either plane keeps the part of the bored shaft
/// on `x < cut`, with the whole hole in it, under the certified policy.
/// Boring split the narrow bar's pipe and the bore's where the two would cross
/// as whole pipes, leaving the hole two corners on that circle, and the cut
/// was refused `Undecidable` at `order along a crossing`, on one of those two
/// corners, for each of these six bores (Q2487). Since Q2619 boring leaves no
/// such corner, and since Q2657 neither does a square hole, but the halves of
/// the shaft with a square hole keep theirs, and bring them to the fuse of the
/// two halves.
#[test]
fn a_stepped_shaft_bored_across_the_narrow_bars_radius_is_cut_in_half() {
    for (centre, radius) in [
        ([-1.0, 0.0], 0.3),
        ([-1.0, 0.1], 0.3),
        ([-1.2, 0.0], 0.3),
        ([-1.0, 0.0], 0.25),
        ([-1.0, 0.0], 0.4),
        ([-0.9, 0.3], 0.3),
    ] {
        let bore = Bore::ARoundHole { centre, radius };
        for cut in [0.5, 0.0] {
            let asked = format!("a bore of radius {radius} about {centre:?}, cut at x = {cut}");
            let mut model = a_model();
            let shaft = a_stepped_shaft(&mut model);
            let bored = bore.cut_into(&mut model, shaft);
            let block = a_box(&mut model, [[-10.0, cut], [-10.0, 10.0], [-10.0, 10.0]]);
            let mut log = ProvenanceLog::new();
            let kept = model
                .fuse(
                    &[bored],
                    &[block],
                    Rule::Intersection,
                    &Policy::certified(),
                    &mut log,
                )
                .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            assert!(
                log.is_empty(),
                "{asked}: a certified fuse logs nothing: {log:?}"
            );
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{asked}: {broken:?}"));
            let [solid] = kept.solids() else {
                panic!("{asked}: one solid, not {:?}", kept.solids());
            };
            holds(
                &model,
                *solid,
                a_stepped_shafts_part_left_of(cut) - bore.volume(),
                &asked,
            );
        }
    }
}

/// Bored with a bar of radius 0.3 about `(−1, 0)`, from below the shaft to
/// `z = 0.5`, the shaft keeps the narrow bar's pipe and the bore's pipe one
/// face each, and no corner where the two pipes would cross. The bore stops
/// half a unit below the narrow bar, so the two pipes meet nowhere on the
/// solid: as whole pipes they cross on two lines without end, at
/// `(−0.955, ±0.2966)`. Boring split both pipes along those lines, and the
/// circles at `z = 0`, `0.5`, `1` and `2` where they cross them (Q2613,
/// Q2619).
#[test]
fn a_stepped_shaft_bored_across_the_narrow_bars_radius_keeps_each_pipe_one_face() {
    let mut model = a_model();
    let shaft = a_stepped_shaft(&mut model);
    let bored = Bore::ARoundHole {
        centre: [-1.0, 0.0],
        radius: 0.3,
    }
    .cut_into(&mut model, shaft);
    let view = model.solid(bored).expect("this model made the key");
    let faces_on_a_pipe_of = |radius: f64| {
        view.faces()
            .filter(
                |face| matches!(face.surface(), Surface::Cylinder(pipe) if pipe.radius() == radius),
            )
            .count()
    };
    assert_eq!(
        [faces_on_a_pipe_of(1.0), faces_on_a_pipe_of(0.3)],
        [1, 1],
        "faces on the narrow bar's pipe and on the bore's"
    );
    let where_the_pipes_would_cross = |corner: &Point| {
        (corner.x() + 0.955).abs() < 1e-3 && (corner.y().abs() - 0.2966).abs() < 1e-3
    };
    for face in view.faces() {
        for wire in face.wires() {
            let corners: Vec<Point> = wire.points().collect();
            assert!(
                !corners.iter().any(where_the_pipes_would_cross),
                "face {:?} on {:?} has a corner where the pipes would cross: {corners:?}",
                face.key(),
                face.surface()
            );
        }
    }
}

/// The first of those bores, radius 0.3 about `(−1, 0)`, then cut and turned
/// as the round and square holes are: every fuse answers with the D as its
/// bottom's outline, under both policies.
#[test]
fn a_stepped_shaft_bored_across_the_narrow_bars_radius_cut_and_turned_keeps_the_d_as_its_bottoms_outline()
 {
    let bore = Bore::ARoundHole {
        centre: [-1.0, 0.0],
        radius: 0.3,
    };
    for turn in TURNS {
        every_fuse_of_the_bored_halves_answers(
            &format!("a hole across the narrow bar's radius, cut at x = 0.5, turned {turn}"),
            0.5,
            bore,
            &turned(turn),
        );
        every_fuse_of_the_bored_halves_answers(
            &format!("a hole across the narrow bar's radius, cut at x = 0, shifted, turned {turn}"),
            0.0,
            bore,
            &shifted_and_turned(turn),
        );
    }
}

/// Bored with the square hole, the shaft keeps no corner on its circles where
/// a side plane of the hole, run on past the hole, crosses them: on the
/// bottom and the annulus at `(±1.977, ±0.3)`, `(−1.8, ±0.872)` and
/// `(−1.2, ±1.6)`, and on the narrow bar's circles at `(±0.954, ±0.3)`. No
/// face of the hole is there. Boring put a corner at every one of them
/// (Q2657), and those corners are what Q2488's refusal met.
#[test]
fn a_stepped_shaft_bored_with_a_square_hole_keeps_no_corner_where_the_holes_side_planes_cross_its_circles()
 {
    let mut model = a_model();
    let shaft = a_stepped_shaft(&mut model);
    let bored = Bore::ASquareHole.cut_into(&mut model, shaft);
    let view = model.solid(bored).expect("this model made the key");
    let on_a_circle_and_a_side_plane = |corner: &Point| {
        let radius = corner.x().hypot(corner.y());
        let on_a_circle = (radius - 1.0).abs() < 1e-6 || (radius - 2.0).abs() < 1e-6;
        let on_a_side_plane = (corner.x() + 1.8).abs() < 1e-6
            || (corner.x() + 1.2).abs() < 1e-6
            || (corner.y().abs() - 0.3).abs() < 1e-6;
        on_a_circle && on_a_side_plane
    };
    for face in view.faces() {
        for wire in face.wires() {
            let corners: Vec<Point> = wire.points().collect();
            assert!(
                !corners.iter().any(on_a_circle_and_a_side_plane),
                "face {:?} on {:?} has a corner where a side plane of the hole crosses a circle: \
                 {corners:?}",
                face.key(),
                face.surface()
            );
        }
    }
}

fn turned(turn: f64) -> [Isometry; 1] {
    [Isometry::rotation(&Axis::z(), turn).expect("a finite turn")]
}

fn shifted_and_turned(turn: f64) -> [Isometry; 2] {
    [
        Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a finite shift"),
        Isometry::rotation(&Axis::z(), turn).expect("a finite turn"),
    ]
}

/// What is bored up into the stepped shaft from its bottom before it is cut,
/// from below the shaft to `z = 0.5`, on the kept side of both cuts.
#[derive(Clone, Copy, Debug)]
enum Bore {
    Nothing,
    /// A bar of `radius` about `centre`.
    ARoundHole {
        centre: [f64; 2],
        radius: f64,
    },
    /// A block 0.6 on a side about `(−1.5, 0)`.
    ASquareHole,
}

impl Bore {
    fn cut_into(self, model: &mut Model, shaft: SolidKey) -> SolidKey {
        let bit = match self {
            Bore::Nothing => return shaft,
            Bore::ARoundHole {
                centre: [x, y],
                radius,
            } => model
                .add_round_bar(
                    radius,
                    1.5,
                    &Isometry::translation(Vector::new(x, y, -1.0)).expect("a finite shift"),
                )
                .expect("a bar of positive size"),
            Bore::ASquareHole => a_box(model, [[-1.8, -1.2], [-0.3, 0.3], [-1.0, 0.5]]),
        };
        the_one_solid(model, shaft, bit, Rule::Difference)
    }

    /// The volume it takes out of the shaft, from `z = 0` to `z = 0.5`.
    fn volume(self) -> f64 {
        match self {
            Bore::Nothing => 0.0,
            Bore::ARoundHole { radius, .. } => PI * radius * radius * 0.5,
            Bore::ASquareHole => 0.6 * 0.6 * 0.5,
        }
    }
}

/// What the half cut off is cut from.
#[derive(Clone, Copy, Debug)]
enum CutOffFrom {
    TheSteppedShaft,
    /// A bar of radius 2 from `z = 0` to `z = 2`.
    AWideBar,
}

impl CutOffFrom {
    fn build(self, model: &mut Model) -> SolidKey {
        match self {
            CutOffFrom::TheSteppedShaft => a_stepped_shaft(model),
            CutOffFrom::AWideBar => a_bar(model, 2.0, 2.0, 0.0),
        }
    }

    /// The volume of the half cut off, on `x > cut`, by hand.
    fn volume_right_of(self, cut: f64) -> f64 {
        match self {
            CutOffFrom::TheSteppedShaft => 5.0 * PI - a_stepped_shafts_part_left_of(cut),
            CutOffFrom::AWideBar => 2.0 * (4.0 * PI - a_bars_part_left_of(2.0, cut)),
        }
    }
}

/// The half of `cut_off_from` on `x > cut` and the stepped shaft's half on
/// `x < cut`, each placed by every movement in `moves` in turn, fused by every
/// rule either way round under both policies: one solid of the volume worked
/// out by hand, or nothing where the two only touch, and a certified fuse logs
/// nothing.
fn every_fuse_of_the_halves_answers(
    route: &str,
    cut: f64,
    cut_off_from: CutOffFrom,
    moves: &[Isometry],
) {
    every_fuse_answers(route, cut, cut_off_from, Bore::Nothing, moves);
}

/// [`every_fuse_of_the_halves_answers`] with the kept half cut from the
/// stepped shaft less `bore`, and the half cut off from the stepped shaft.
fn every_fuse_of_the_bored_halves_answers(route: &str, cut: f64, bore: Bore, moves: &[Isometry]) {
    every_fuse_answers(route, cut, CutOffFrom::TheSteppedShaft, bore, moves);
}

/// Every fuse of the halves answers, and every face of the answer with a hole
/// has an outline spreading as wide as any hole.
fn every_fuse_answers(
    route: &str,
    cut: f64,
    cut_off_from: CutOffFrom,
    bore: Bore,
    moves: &[Isometry],
) {
    let kept = a_stepped_shafts_part_left_of(cut) - bore.volume();
    let cut_off = cut_off_from.volume_right_of(cut);
    for (policy_name, policy) in [
        ("certified", Policy::certified()),
        ("tolerant", Policy::tolerant(100.0)),
    ] {
        for (rule, kept_first, volume) in [
            (Rule::Union, true, Some(kept + cut_off)),
            (Rule::Union, false, Some(kept + cut_off)),
            (Rule::Intersection, true, None),
            (Rule::Intersection, false, None),
            (Rule::Difference, true, Some(kept)),
            (Rule::Difference, false, Some(cut_off)),
        ] {
            let asked =
                format!("{route}, {rule:?}, the kept half first {kept_first}, {policy_name}");
            let mut model = a_model();
            let halves = the_two_halves(&mut model, cut, cut_off_from, bore);
            for movement in moves {
                for half in halves {
                    model
                        .place_solid(half, movement)
                        .expect("this model made the key");
                }
            }
            let [kept_half, cut_off_half] = halves;
            let [one, other] = if kept_first {
                [kept_half, cut_off_half]
            } else {
                [cut_off_half, kept_half]
            };
            let mut log = ProvenanceLog::new();
            let fused = model
                .fuse(&[one], &[other], rule, &policy, &mut log)
                .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            if policy_name == "certified" {
                assert!(
                    log.is_empty(),
                    "{asked}: a certified fuse logs nothing: {log:?}"
                );
            }
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{asked}: {broken:?}"));
            match volume {
                Some(volume) => {
                    let [solid] = fused.solids() else {
                        panic!("{asked}: one solid, not {:?}", fused.solids());
                    };
                    holds(&model, *solid, volume, &asked);
                    every_outline_spreads_as_wide_as_its_holes(&model, *solid, &asked);
                }
                None => assert!(
                    fused.solids().is_empty(),
                    "{asked}: nothing, not {:?}",
                    fused.solids()
                ),
            }
        }
    }
}

/// The stepped shaft's volume on `x < cut`.
fn a_stepped_shafts_part_left_of(cut: f64) -> f64 {
    a_bars_part_left_of(2.0, cut) + a_bars_part_left_of(1.0, cut)
}

/// The area of a disk of `radius` about the origin on the side `x < cut`, the
/// volume of that part of a bar one unit high.
fn a_bars_part_left_of(radius: f64, cut: f64) -> f64 {
    radius * radius * (-cut / radius).acos() + cut * (radius * radius - cut * cut).sqrt()
}

/// The part on `x < cut` of the stepped shaft less `bore`, then
/// `cut_off_from`'s on `x > cut`.
fn the_two_halves(
    model: &mut Model,
    cut: f64,
    cut_off_from: CutOffFrom,
    bore: Bore,
) -> [SolidKey; 2] {
    let below = [[-10.0, cut], [-10.0, 10.0], [-10.0, 10.0]];
    let one = cut_off_from.build(model);
    let shaft = a_stepped_shaft(model);
    let other = bore.cut_into(model, shaft);
    let [block, its_copy] = [a_box(model, below), a_box(model, below)];
    [
        the_one_solid(model, other, its_copy, Rule::Intersection),
        the_one_solid(model, one, block, Rule::Difference),
    ]
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// A bar of radius 2 from `z = 0` to `z = 1` united with one of radius 1 from
/// `z = 1` to `z = 2`.
fn a_stepped_shaft(model: &mut Model) -> SolidKey {
    let wide = a_bar(model, 2.0, 1.0, 0.0);
    let narrow = a_bar(model, 1.0, 1.0, 1.0);
    the_one_solid(model, wide, narrow, Rule::Union)
}

fn a_bar(model: &mut Model, radius: f64, height: f64, bottom: f64) -> SolidKey {
    model
        .add_round_bar(
            radius,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite shift"),
        )
        .expect("a bar of positive size")
}

fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let outline = ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("an outline");
    model
        .add_extrusion(
            &outline,
            z[1] - z[0],
            &Isometry::translation(Vector::new(0.0, 0.0, z[0])).expect("a finite shift"),
        )
        .expect("a positive height")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{rule:?}: {refused:?}"));
    let [solid] = fused.solids() else {
        panic!("{rule:?} is one solid: {:?}", fused.solids());
    };
    *solid
}

/// Every face of `solid` with a hole has corners on its outline at least as
/// far apart as those of any hole. The D's chord is near four long and a
/// bore's corners under one apart, so a bore written as the outline, with the
/// D a hole, shows; two rings of one corner each, as a whole disk with a
/// round hole has, pass either way.
fn every_outline_spreads_as_wide_as_its_holes(model: &Model, solid: SolidKey, asked: &str) {
    let spread = |points: Vec<Point>| -> f64 {
        points
            .iter()
            .flat_map(|one| points.iter().map(move |other| one.distance_to(*other)))
            .fold(0.0, f64::max)
    };
    for face in model.solid(solid).expect("this model made the key").faces() {
        let mut wires = face.wires().map(|wire| spread(wire.points().collect()));
        let outline = wires.next().expect("a face has an outline");
        for hole in wires {
            assert!(
                outline >= hole,
                "{asked}: face {:?} has an outline spreading {outline} and a hole spreading {hole}",
                face.key()
            );
        }
    }
}

fn holds(model: &Model, solid: SolidKey, volume: f64, asked: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{asked}: {} ± {} against {volume}",
        measured.volume(),
        measured.volume_error()
    );
}
