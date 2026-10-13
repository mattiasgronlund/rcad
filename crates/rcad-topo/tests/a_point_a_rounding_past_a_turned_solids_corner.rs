//! A point a rounding past a turned cube's outermost corner, which every one of
//! the cube's stored planes puts inside (`docs/decisions.md` §17.1, §346.7,
//! Q1157).
//!
//! `classify_uncertain_point` answers `Outside` without firing a ray for a point
//! past the box [`SolidView::box_around_it`] draws, and that box holds a flat
//! face's stored corners. The face is its stored plane, though, and a turn
//! leaves a corner a rounding off where its three planes meet. So the cube the
//! planes bound can poke a rounding past that box at its outermost corner, and
//! a point in that tip came back `Outside` where every plane put it inside.
//! `SolidView::shares_space_with` read the same box the same way, and called a
//! block resting a spacing above that box `Apart` from the cube (Q1331).
//!
//! A face on a pipe or a cone is held in that box by the discs across its
//! axis, and a disc's reach along a coordinate axis was worked out as
//! `r · √(1 − a²)`, which keeps nothing but rounding where the axis is a hair
//! off that coordinate axis. A bar or a frustum tipped `1e-9` reached a
//! billionth of its radius past the box, and points there came back `Outside`
//! where every stored surface put them inside (Q1332), and a block resting
//! just above the box was called `Apart` from the bar (Q1363). The box itself
//! now works the reach out as `r · √(b² + c²)/|a|` from the axis's other two
//! parts, rounded outward (Q1363).
//!
//! Every answer here is checked against the stored surfaces' exact signs.
//! Every solid here is convex, so it holds a point exactly when each of its
//! surfaces puts the point on the solid's side.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};
use rcad_topo::{Containment, FaceView, Model, Rule, SharedSpace, SolidKey, SolidView};

mod cones;

/// A point one spacing above the top of a cube's box, turned `2.5` about
/// `(1, 1, 1)`, under the corner whose three planes meet above that box. The
/// box alone put it outside.
#[test]
fn a_point_above_a_turned_cubes_box_that_every_plane_puts_inside_is_inside() {
    let (model, cube) = a_cube_of_side_2_turned([1.0, 1.0, 1.0], 2.5);
    let view = model.solid(cube).expect("this model made the key");
    let point = [
        -0.891_818_517_366_445_4,
        0.490_293_696_637_200_8,
        1.401_524_820_729_244_6,
    ];
    let (_, high) = view.box_around_it().expect("a cube has a box");
    assert_eq!(high.coordinates()[2].next_up(), point[2]);
    assert_eq!(
        what_the_stored_surfaces_say(view, point),
        Containment::Inside
    );
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        assert_eq!(
            view.classify_point(
                Point::new(point[0], point[1], point[2]),
                &policy,
                &mut ProvenanceLog::new()
            ),
            Containment::Inside,
            "{policy:?}"
        );
    }
}

/// A block of side 1 standing one spacing above the box of the cube above,
/// over the corner whose three planes meet above that box (Q1331).
///
/// The two boxes miss, and the boxes alone answered `Apart` both ways round,
/// which is a proof that no point is in both. Yet a point three spacings above
/// the cube's box, so two into the block, is inside both by every stored plane
/// of each. Both are convex, so that is exact, and `Shared` is the whole
/// answer. No corner of either is inside the other and no two faces share a
/// plane, so the search finds nothing: `NothingFound`, which claims nothing.
#[test]
fn a_block_one_spacing_above_a_turned_cubes_box_is_not_called_apart_from_it() {
    let policy = Policy::certified();
    let (mut model, cube) = a_cube_of_side_2_turned([1.0, 1.0, 1.0], 2.5);
    let block = model
        .add_cube(1.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side 1");
    model
        .place_solid(
            block,
            &Isometry::translation(Vector::new(-1.4, 0.0, 1.401_524_820_729_244_6))
                .expect("a finite shift"),
        )
        .expect("this model made the key");
    let cube = model.solid(cube).expect("this model made the key");
    let block = model.solid(block).expect("this model made the key");

    let (_, top_of_the_cube) = cube.box_around_it().expect("a cube has a box");
    let (bottom_of_the_block, _) = block.box_around_it().expect("a cube has a box");
    assert_eq!(
        top_of_the_cube.coordinates()[2].next_up(),
        bottom_of_the_block.coordinates()[2]
    );
    let in_both = [
        -0.891_818_517_366_446,
        0.490_293_696_637_201_5,
        1.401_524_820_729_245,
    ];
    assert_eq!(
        what_the_stored_surfaces_say(cube, in_both),
        Containment::Inside
    );
    assert_eq!(
        what_the_stored_surfaces_say(block, in_both),
        Containment::Inside
    );

    for [one, other] in [[cube, block], [block, cube]] {
        assert_eq!(
            one.shares_space_with(other, &policy, &mut ProvenanceLog::new()),
            SharedSpace::NothingFound
        );
    }
}

/// Forty cubes of side 2, five axes by eight angles. At every corner outermost
/// along an axis, every `f64` point within six spacings of the box round where
/// that corner's three planes meet, past the cube's box and put inside by every
/// stored plane, is classified.
///
/// The box alone answered all 3897 of them `Outside`. Now 1 is `Inside` and
/// 3811 are `Undecidable`. The 85 still `Outside` are answered by the ray,
/// past the box: one face's corners and plane part about each, and
/// `is_beside` lets it through because that face's outline, read at the plane,
/// holds none of the point's box stretched there, so the corners count it
/// (Q1156's first shape, §346.7).
#[test]
fn how_often_a_point_past_a_turned_cubes_box_that_every_plane_puts_inside_is_answered_outside() {
    let axes = [
        [1.0, 2.0, 3.0],
        [3.0, -1.0, 2.0],
        [1.0, 1.0, 1.0],
        [-2.0, 1.0, 5.0],
        [5.0, 3.0, -1.0],
    ];
    let angles = [0.3, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0];
    let (mut points, mut inside, mut undecided, mut outside) = (0, 0, 0, 0);
    for axis in axes {
        for angle in angles {
            let (model, cube) = a_cube_of_side_2_turned(axis, angle);
            let view = model.solid(cube).expect("this model made the key");
            for point in points_past_its_box_every_plane_puts_inside(view) {
                points += 1;
                let answer = view.classify_point(
                    Point::new(point[0], point[1], point[2]),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                if answer == Containment::Inside {
                    inside += 1;
                } else if matches!(answer, Containment::Undecidable { .. }) {
                    undecided += 1;
                } else {
                    assert_eq!(answer, Containment::Outside, "{point:?}");
                    outside += 1;
                }
            }
        }
    }
    println!(
        "{points} points past the box that every plane puts inside: \
         {inside} Inside, {undecided} Undecidable, {outside} Outside"
    );
    assert_eq!((points, inside, undecided, outside), (3897, 1, 3811, 85));
}

/// A point a billionth above the top cap's middle of a bar and of a frustum,
/// each tipped `1e-9` about `x`, under the top rim's highest point (Q1332).
///
/// The tip leaves the axis's `z` part exactly one, so a box that works a
/// disc's reach along `z` out as `r · √(1 − 1)` has the top cap reach no
/// higher than its middle. It truly reaches a billionth of its radius higher
/// on one side, and that box alone put these points outside. The box holds
/// them now (Q1363).
#[test]
fn a_point_above_a_bar_or_a_frustum_tipped_a_billionth_that_its_surfaces_put_inside_is_inside() {
    for (shape, point) in [
        (Tipped::Bar, [0.0, 0.499_999_999, 1.000_000_000_4]),
        (Tipped::Frustum, [0.0, 0.999_999_996, 4.000_000_000_9]),
    ] {
        let (model, solid, _) = shape.built([1.0, 0.0, 0.0], 1e-9);
        let view = model.solid(solid).expect("this model made the key");
        let (_, high) = view.box_around_it().expect("a solid has a box");
        assert!(point[2] <= high.coordinates()[2], "{shape:?}: {high:?}");
        assert_eq!(
            what_the_stored_surfaces_say(view, point),
            Containment::Inside,
            "{shape:?}"
        );
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            assert_eq!(
                view.classify_point(
                    Point::new(point[0], point[1], point[2]),
                    &policy,
                    &mut ProvenanceLog::new()
                ),
                Containment::Inside,
                "{shape:?} {policy:?}"
            );
        }
    }
}

/// Forty bars and frustums, each tipped about one of four axes square to its
/// own by one of five small angles. Near each rim, every point sampled that
/// every stored surface puts inside is counted, and those past the solid's box
/// are classified.
///
/// While the box worked a disc's reach out as `r · √(1 − a²)`, a tip under
/// about `1e-7` left it short of the discs at the rims, and 23639 of these
/// points were past it. The box alone answered every one of them `Outside`,
/// and Q1332 opened the box out for `classify_uncertain_point` alone, after
/// which all 23639 came back `Inside`. Now the box itself holds all 172800
/// (Q1363), so none is past it and that step is gone. Classified with it gone,
/// all 172800 came back `Inside` too; that run takes six seconds, so it is not
/// kept here.
#[test]
fn how_often_a_point_past_a_tipped_bar_or_frustums_box_that_its_surfaces_put_inside_is_answered_outside()
 {
    let axes = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, -1.0, 0.0],
    ];
    let angles = [1e-9, 1e-8, 3e-8, 1e-7, 1e-6];
    let (mut points, mut past_the_box) = (0, 0);
    let (mut inside, mut undecided, mut outside) = (0, 0, 0);
    for shape in [Tipped::Bar, Tipped::Frustum] {
        for axis in axes {
            for angle in angles {
                let (model, solid, tip) = shape.built(axis, angle);
                let view = model.solid(solid).expect("this model made the key");
                let (low, high) = view.box_around_it().expect("a solid has a box");
                let (low, high) = (low.coordinates(), high.coordinates());
                for point in points_near_its_rims_its_surfaces_put_inside(view, shape, &tip) {
                    points += 1;
                    if (0..3).all(|axis| low[axis] <= point[axis] && point[axis] <= high[axis]) {
                        continue;
                    }
                    past_the_box += 1;
                    let answer = view.classify_point(
                        Point::new(point[0], point[1], point[2]),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                    if answer == Containment::Inside {
                        inside += 1;
                    } else if matches!(answer, Containment::Undecidable { .. }) {
                        undecided += 1;
                    } else {
                        assert_eq!(answer, Containment::Outside, "{shape:?} {point:?}");
                        outside += 1;
                    }
                }
            }
        }
    }
    println!(
        "{points} points near a rim that every surface puts inside, {past_the_box} past the box: \
         {inside} Inside, {undecided} Undecidable, {outside} Outside"
    );
    assert_eq!(
        (points, past_the_box, inside, undecided, outside),
        (172_800, 0, 0, 0, 0)
    );
}

/// A block of side 1 resting two ten-billionths above the box of the bar
/// above, tipped `1e-9` about `x`, over the side of the top cap that the tip
/// lifts (Q1363).
///
/// The bar's box stopped at the cap's middle in `z`, where the cap truly
/// reaches a billionth higher at `y = 1`, so the two boxes missed and the boxes
/// alone answered `Apart` both ways round. Yet the block's corner at
/// `(−0.5, 0.5)` is under the cap by half a billionth less two
/// ten-billionths, and every stored surface of the bar puts it inside. A
/// corner of one solid strictly inside the other is material of both in one
/// place, so `Shared` is the whole answer.
#[test]
fn a_block_just_above_a_tipped_bars_box_is_not_called_apart_from_it() {
    let policy = Policy::certified();
    let (model, bar, block) = a_block_just_above_a_tipped_bar();
    let bar = model.solid(bar).expect("this model made the key");
    let block = model.solid(block).expect("this model made the key");

    let (_, top_of_the_bar) = bar.box_around_it().expect("a bar has a box");
    let (bottom_of_the_block, _) = block.box_around_it().expect("a cube has a box");
    let corner = [-0.5, 0.5, 1.000_000_000_2];
    assert_eq!(bottom_of_the_block.coordinates(), corner);
    assert_eq!(
        what_the_stored_surfaces_say(bar, corner),
        Containment::Inside
    );
    let in_both = [0.0, 0.75, 1.000_000_000_4];
    assert_eq!(
        what_the_stored_surfaces_say(bar, in_both),
        Containment::Inside
    );
    assert_eq!(
        what_the_stored_surfaces_say(block, in_both),
        Containment::Inside
    );

    for [one, other] in [[bar, block], [block, bar]] {
        assert_eq!(
            one.shares_space_with(other, &policy, &mut ProvenanceLog::new()),
            SharedSpace::Shared
        );
    }
    assert!(
        top_of_the_bar.coordinates()[2] >= in_both[2],
        "the bar's box stops short of what its surfaces hold: {top_of_the_bar:?}"
    );
}

/// The fuse of the block and the bar above, under every rule both ways round.
///
/// While the bar's box stopped short, all six gave `ResultIsNotManifold {
/// count: 1 }`. Once the pair was seen, all six refused at the order along
/// the line where the bar's wall meets the plane of the block's side `y =
/// 0.5`: the block's bottom and the bar's top cut that line about three
/// ten-billionths apart, and the order's enclosure, `4.4e-7`, was wider than
/// that (§346.7). `fuse/arrange.rs` puts the two in order now, by the way the
/// loop that plane cuts the wall in runs at each
/// (`along_the_stretch_between_the_two`) and by how far up the bar's axis
/// each is (`Arrangement::as_the_pipes_axis_says`); with either taken out the
/// other still answered all six, and with both out all six were refused as
/// before. Each fuse now gives one solid that holds what it holds by hand,
/// and logs nothing.
#[test]
fn every_fuse_of_a_block_just_above_a_tipped_bar_holds_what_it_holds_by_hand() {
    let policy = Policy::certified();
    let (model, bar, block) = a_block_just_above_a_tipped_bar();
    let sliver = the_sliver_the_block_and_the_bar_share();
    for (rule, bar_first, by_hand) in [
        (Rule::Union, true, PI + 1.0 - sliver),
        (Rule::Union, false, PI + 1.0 - sliver),
        (Rule::Intersection, true, sliver),
        (Rule::Intersection, false, sliver),
        (Rule::Difference, true, PI - sliver),
        (Rule::Difference, false, 1.0 - sliver),
    ] {
        let what = format!("{rule:?} bar first {bar_first}");
        let [one, other] = if bar_first {
            [bar, block]
        } else {
            [block, bar]
        };
        let mut model = model.clone();
        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(&[one], &[other], rule, &policy, &mut log)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        assert!(log.is_empty(), "{what}: logged {log:?}");
        let [solid] = fused.solids() else {
            panic!("{what}: {} solids", fused.solids().len());
        };
        let measured = model
            .solid(*solid)
            .expect("the fuse made the key")
            .measure(
                Deflection::for_scale(model.scale()),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a solid the fuse made can be measured");
        let (volume, error) = (measured.volume(), measured.volume_error());
        assert!(
            (volume - by_hand).abs() <= error,
            "{what}: measured {volume} ± {error}, by hand {by_hand}"
        );
    }
}

/// The volume both the bar and the block above hold.
///
/// Tipped `1e-9` about `x`, the bar's top cap is the plane `z = (1 + y sin θ) /
/// cos θ` for `θ = 1e-9`, which is `1 + 1e-9 · y` to within `1e-18`, and the
/// block's bottom is at `z = 1 + 2e-10`. Over the block's foot, where `y` is at
/// least `0.5`, the two are `1e-9 · y − 2e-10` apart, and that is the sliver's
/// thickness. The part of the foot under the cap is `|x| ≤ 0.5`, `0.5 ≤ y ≤
/// √(1 − x²)`; the tip moves the wall about `1e-9` in `y` there, where the
/// sliver is under `1e-9` thick, which moves the volume by under `1e-18`. So
/// the sliver holds `∫∫ (1e-9 · y − 2e-10) dy dx`, which is `1e-9 / 3 − 2e-10 ·
/// (√3/4 + π/6 − 1/2)`, about `2.42e-10`.
fn the_sliver_the_block_and_the_bar_share() -> f64 {
    let under_the_cap = 3.0_f64.sqrt() / 4.0 + PI / 6.0 - 0.5;
    1e-9 / 3.0 - 2e-10 * under_the_cap
}

/// The bar of [`Tipped::Bar`] tipped `1e-9` about `x`, and a block of side 1
/// from `(−0.5, 0.5)` to `(0.5, 1.5)` across, its bottom at
/// `1.000_000_000_2`.
fn a_block_just_above_a_tipped_bar() -> (Model, SolidKey, SolidKey) {
    let (mut model, bar, _) = Tipped::Bar.built([1.0, 0.0, 0.0], 1e-9);
    let block = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side 1");
    model
        .place_solid(
            block,
            &Isometry::translation(Vector::new(-0.5, 0.5, 1.000_000_000_2))
                .expect("a finite shift"),
        )
        .expect("this model made the key");
    (model, bar, block)
}

/// Every `f64` point near a corner outermost along an axis that is past the
/// box along that axis and that every stored plane puts inside.
fn points_past_its_box_every_plane_puts_inside(view: SolidView<'_>) -> Vec<[f64; 3]> {
    let (low, high) = view.box_around_it().expect("a cube has a box");
    let (low, high) = (low.coordinates(), high.coordinates());
    let faces: Vec<FaceView<'_>> = view.faces().collect();
    let mut found = Vec::new();
    for corner in corners_of(&faces) {
        let around: Vec<Plane> = faces
            .iter()
            .filter(|face| {
                face.outer_wire()
                    .points()
                    .any(|at| at.coordinates() == corner)
            })
            .map(|face| face.plane())
            .collect();
        assert_eq!(around.len(), 3, "a cube's corner has three faces round it");
        let meet = Plane::meet_of_three(&around[0], &around[1], &around[2])
            .expect("a cube's three planes at a corner meet in a point")
            .enclosure();
        for axis in 0..3 {
            for (bound, is_past) in [
                (high[axis], f64::gt as fn(&f64, &f64) -> bool),
                (low[axis], f64::lt),
            ] {
                if corner[axis] != bound {
                    continue;
                }
                let near: [Vec<f64>; 3] = core::array::from_fn(|along| {
                    f64s_between(meet[along].lo(), meet[along].hi(), 6)
                        .into_iter()
                        .filter(|&at| along != axis || is_past(&at, &bound))
                        .collect()
                });
                for &x in &near[0] {
                    for &y in &near[1] {
                        for &z in &near[2] {
                            if what_the_stored_surfaces_say(view, [x, y, z]) == Containment::Inside
                            {
                                found.push([x, y, z]);
                            }
                        }
                    }
                }
            }
        }
    }
    found
}

fn corners_of(faces: &[FaceView<'_>]) -> Vec<[f64; 3]> {
    let mut corners: Vec<[f64; 3]> = Vec::new();
    for face in faces {
        for at in face.outer_wire().points() {
            if !corners.contains(&at.coordinates()) {
                corners.push(at.coordinates());
            }
        }
    }
    corners
}

/// Every `f64` from `spacings` below `lo` to `spacings` above `hi`.
fn f64s_between(lo: f64, hi: f64, spacings: usize) -> Vec<f64> {
    let (mut at, mut end) = (lo, hi);
    for _ in 0..spacings {
        at = at.next_down();
        end = end.next_up();
    }
    let mut every = Vec::new();
    while at <= end {
        every.push(at);
        at = at.next_up();
    }
    every
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Side {
    Inner,
    Outer,
    On,
}

/// Every solid here is convex — a cube, a block, a bar and a frustum — so it
/// holds a point exactly when every stored surface puts the point on the
/// solid's side.
fn what_the_stored_surfaces_say(view: SolidView<'_>, point: [f64; 3]) -> Containment {
    let sides: Vec<Side> = view
        .faces()
        .map(|face| side_of_its_surface(face, point))
        .collect();
    if sides.contains(&Side::Outer) {
        Containment::Outside
    } else if sides.contains(&Side::On) {
        Containment::OnBoundary
    } else {
        Containment::Inside
    }
}

fn side_of_its_surface(face: FaceView<'_>, point: [f64; 3]) -> Side {
    let outer = if face.orientation().is_reversed() {
        Decision::Neg
    } else {
        Decision::Pos
    };
    let at = PredicatePoint::from(point);
    let surface = face.surface();
    let side = if let Surface::Cylinder(pipe) = surface {
        pipe.side_of(at)
    } else if let Surface::Cone(cone) = surface {
        cone.side_of(at)
    } else {
        face.plane().side_of_point(at)
    };
    assert!(
        matches!(side, Decision::Pos | Decision::Neg | Decision::Zero),
        "an exact point is always placed: {side:?}"
    );
    if side == Decision::Zero {
        Side::On
    } else if side == outer {
        Side::Outer
    } else {
        Side::Inner
    }
}

#[derive(Clone, Copy, Debug)]
enum Tipped {
    /// A round bar of radius 1 from `z = 0` to `z = 1`.
    Bar,
    /// The frustum of a cone of slope a half about `z`, from `z = 2`, where
    /// its radius is 1, to `z = 4`, where it is 2.
    Frustum,
}

impl Tipped {
    /// The solid, tipped `angle` about `axis` through the origin, and the tip.
    fn built(self, axis: [f64; 3], angle: f64) -> (Model, SolidKey, Isometry) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let tip = Isometry::rotation(
            &Axis::new(Point::origin(), Vector::new(axis[0], axis[1], axis[2])).expect("an axis"),
            angle,
        )
        .expect("a finite turn");
        let solid = match self {
            Self::Bar => model
                .add_round_bar(1.0, 1.0, &tip)
                .expect("a bar of radius 1"),
            Self::Frustum => {
                let frustum = cones::a_frustum(&mut model, 0.5, 2.0, 4.0);
                model
                    .place_solid(frustum, &tip)
                    .expect("this model made the key");
                frustum
            }
        };
        (model, solid, tip)
    }

    /// Each cap's rim as its radius, its height up the untipped axis, and
    /// which way along the axis the solid lies from it.
    fn rims(self) -> [(f64, f64, f64); 2] {
        match self {
            Self::Bar => [(1.0, 0.0, 1.0), (1.0, 1.0, -1.0)],
            Self::Frustum => [(1.0, 2.0, 1.0), (2.0, 4.0, -1.0)],
        }
    }
}

/// Points just inside each rim of the untipped solid, `inward` of the radius
/// in from the rim and `inward` along the axis in from the cap, half a degree
/// apart round it, tipped with the solid — those that every stored surface
/// puts inside.
fn points_near_its_rims_its_surfaces_put_inside(
    view: SolidView<'_>,
    shape: Tipped,
    tip: &Isometry,
) -> Vec<[f64; 3]> {
    let mut found = Vec::new();
    for (radius, height, towards_the_middle) in shape.rims() {
        for step in 0..720 {
            let round = f64::from(step) * core::f64::consts::TAU / 720.0;
            for inward in [1e-13, 1e-11, 1e-9] {
                let near_the_rim = Point::new(
                    radius * (1.0 - inward) * round.cos(),
                    radius * (1.0 - inward) * round.sin(),
                    height + towards_the_middle * inward,
                );
                let point = tip.apply_to_point(near_the_rim).coordinates();
                if what_the_stored_surfaces_say(view, point) == Containment::Inside {
                    found.push(point);
                }
            }
        }
    }
    found
}

/// A cube of side 2 about the origin, turned `angle` about `axis` through it.
fn a_cube_of_side_2_turned(axis: [f64; 3], angle: f64) -> (Model, SolidKey) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let cube = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side 2");
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(axis[0], axis[1], axis[2])).expect("an axis"),
        angle,
    )
    .expect("a finite turn");
    let centred = Isometry::translation(Vector::new(-1.0, -1.0, -1.0)).expect("a finite shift");
    model
        .place_solid(cube, &turn.compose(&centred))
        .expect("this model made the key");
    (model, cube)
}
