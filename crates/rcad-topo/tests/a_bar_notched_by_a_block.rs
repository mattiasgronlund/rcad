//! A round bar and a block fused every way round, and every face of the answer
//! asked which points it holds (`docs/decisions.md` §167).
//!
//! **The fuse splits a bar's wall into patches of one pipe.** Where a plane of
//! the block crosses the pipe, the wall is cut along the whole of that crossing
//! — the circle where the block's top crosses it, the rulings where its sides
//! do — and not only along the part the block reaches. So a wall face's side
//! often has another patch of the same pipe across it, and nothing about the
//! pipe says which side of that side the face keeps. What does is the plane the
//! wall was cut along: an arc carries it in its run, and a ruling lies on the
//! plane of one of the block's faces, or on a plane square to `x` or `y` that
//! holds its two ends where the answer keeps none of that face.
//!
//! **The oracle is the solid, worked out from the coordinates alone**: the bar
//! is every point less than its radius from its axis between its two ends, the
//! block is every point strictly between its bounds, and the answer is the two
//! taken by the rule. A point on one of those surfaces is on a face of the
//! answer exactly when a step off the surface either way lands once inside the
//! answer and once outside, and then exactly one face on that surface holds it.
//! Every point asked is further than [`CLEAR`] from every other surface, so it
//! is on no edge and both steps land clear of everything. Nothing is read from
//! the faces but which surface each lies on.
//!
//! A face that refuses is named, and how many refuse in each fuse is pinned in
//! [`REFUSED`], with the reason each class is left in the comment beside it. A
//! refusal is allowed to hide an answer, never to stand beside a wrong one: the
//! faces that do answer still have to agree with the solid.

use core::f64::consts::PI;

use rcad_geom::{Axis, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

/// How far a point asked is from every surface it is not on, and so from every
/// edge of the answer.
const CLEAR: f64 = 0.4;

/// How far off its surface each step that settles a point lands.
const STEP: f64 = 0.2;

/// How many faces refuse in each fuse, by shape, rule and which went first.
///
/// Before the arcs and rulings between two patches of one pipe were read off
/// the plane that cut them, 150 faces refused here, and 75 after. Since a flat
/// face the cuts cannot follow winds its own chords and arcs instead (row
/// Q1002), these add to 39, and every one of them is a curved face: the 36 that
/// moved were all flat, and each now answers what the solid says at every point
/// asked. They were the block's top and floor across the notch, keeping the
/// outside of the pipe in two pieces; the bored corner's floor and top, the
/// square less a quarter of a disc; and the arcs against a plane that crosses
/// their circle and holds neither end, in the caps of the bar cut along a side
/// of the block, the faces of the block inside the bar, and the caps of the box
/// through a cap and the wall, each a disc with the box's end bitten out.
///
/// The notch united refused 12 faces each way round until row Q975, and these
/// added to 15 after it. Those were the twelve patches of the wall with a ruling
/// at `x = 2, y = ±√21` on a side: the block's back face is the plane that cut
/// the wall there, and the union keeps none of it, so no face of the answer
/// holds that ruling. Each is now read against the planes square to `x` and to
/// `y` that hold the ruling's two ends, `x = 2` and `y = √21` as rounded, and
/// `x = 2` proves each patch the part of the pipe in front of its cuts. They
/// answer 110 points more held and 43 more not held than they were asked
/// before, each the solid's answer.
///
/// - **The box through the caps** used to refuse 4 faces each way round: each
///   quarter of the wall inside the box is cut by two of its sides, which meet
///   outside the pipe, so the part in front of both is two runs of rulings.
///   `holds.rs`'s `one_of_two_runs_between_two_cuts_meeting_off_the_pipe`
///   (row Q1252) now answers that shape, so none of the four refuse any more.
/// - **The box through a cap and the wall's `1`** is the wall, which keeps the
///   outside of the strip `|y| ≤ 2` where one of two flats is read for a strip
///   kept between them. A gap.
const REFUSED: [(&str, Rule, bool, usize); 30] = [
    ("the notch", Rule::Union, true, 0),
    ("the notch", Rule::Union, false, 0),
    ("the notch", Rule::Difference, true, 0),
    ("the notch", Rule::Difference, false, 0),
    ("the notch", Rule::Intersection, true, 0),
    ("the notch", Rule::Intersection, false, 0),
    ("a bore through one edge", Rule::Union, true, 0),
    ("a bore through one edge", Rule::Union, false, 0),
    ("a bore through one edge", Rule::Difference, true, 0),
    ("a bore through one edge", Rule::Difference, false, 0),
    ("a bore through one edge", Rule::Intersection, true, 0),
    ("a bore through one edge", Rule::Intersection, false, 0),
    ("a bore through one corner", Rule::Union, true, 0),
    ("a bore through one corner", Rule::Union, false, 0),
    ("a bore through one corner", Rule::Difference, true, 0),
    ("a bore through one corner", Rule::Difference, false, 0),
    ("a bore through one corner", Rule::Intersection, true, 0),
    ("a bore through one corner", Rule::Intersection, false, 0),
    ("a box through the caps", Rule::Union, true, 0),
    ("a box through the caps", Rule::Union, false, 0),
    ("a box through the caps", Rule::Difference, true, 0),
    ("a box through the caps", Rule::Difference, false, 0),
    ("a box through the caps", Rule::Intersection, true, 0),
    ("a box through the caps", Rule::Intersection, false, 0),
    ("a box through a cap and the wall", Rule::Union, true, 1),
    ("a box through a cap and the wall", Rule::Union, false, 1),
    (
        "a box through a cap and the wall",
        Rule::Difference,
        true,
        1,
    ),
    (
        "a box through a cap and the wall",
        Rule::Difference,
        false,
        0,
    ),
    (
        "a box through a cap and the wall",
        Rule::Intersection,
        true,
        0,
    ),
    (
        "a box through a cap and the wall",
        Rule::Intersection,
        false,
        0,
    ),
];

/// Every face of every fuse of a bar and a block says, of every point asked on
/// its surface, what the solid says — or refuses by its own name, as many
/// times as [`REFUSED`] says.
#[test]
fn every_face_of_a_bar_fused_with_a_block_holds_what_the_solid_says() {
    let mut refused = Vec::new();
    let mut totals = [0; 3];
    for shape in SHAPES {
        for rule in [Rule::Union, Rule::Difference, Rule::Intersection] {
            for bar_first in [true, false] {
                let found = ask_every_face(&shape, rule, bar_first);
                println!(
                    "{} {rule:?} bar first {bar_first}: {} faces, {} refused, {} points held, {} not",
                    shape.name, found.faces, found.refused, found.held, found.not_held
                );
                refused.push((shape.name, rule, bar_first, found.refused));
                totals[0] += found.held;
                totals[1] += found.not_held;
                totals[2] += found.refused;
            }
        }
    }
    println!(
        "in all: {} points held, {} points not held, {} faces refused",
        totals[0], totals[1], totals[2]
    );
    assert!(
        totals[0] >= 1000 && totals[1] >= 10_000,
        "enough points were answered to mean something: {totals:?}"
    );
    assert_eq!(refused, REFUSED);
}

/// The notch's wall, asked the same way as the pipe faces above are, once the
/// block that notches it is turned a few degrees about the bar's own axis
/// (`docs/decisions.md` §167, row Q1039).
///
/// **Only the pipe's faces are asked.** The turned block's own six faces are
/// trimmed by a different family (`what_a_flat_face_holds` in `holds.rs`),
/// which this row does not touch; asking them too would not say anything
/// about the one it does.
///
/// Before Q975, a ruling between two patches of one pipe that no flat face of
/// the solid holds is read against the planes square to `x` or `y` that hold
/// both its ends exactly. Before the block is turned, its own back face is
/// one of those square planes, so it is the plane the fuse cut along as well
/// as the one this fallback finds. Turned, the back face still stands along
/// the bar's axis — rotating about that axis cannot tip a vertical face over
/// — so the ruling it cuts is still a vertical line at one fixed point of the
/// circle, and that point's `x` and `y` are still shared bit for bit between
/// the ruling's two ends, stored once and reused for both. So the fallback
/// still finds a plane through it, and this turned notch answers every wall
/// face the same as the one standing square to the axes does: no refusal.
#[test]
fn every_wall_face_of_the_notch_united_holds_what_the_solid_says_with_the_block_turned_a_little() {
    let bar = Bar {
        radius: 5.0,
        axis: [0.0, 0.0],
        ends: [0.0, 4.0],
    };
    let low = [2.0, -2.0, 1.0];
    let high = [10.0, 2.0, 3.0];
    let size: [f64; 3] = core::array::from_fn(|axis| high[axis] - low[axis]);

    let turn_axis = Axis::new(
        Point::new(bar.axis[0], bar.axis[1], 0.0),
        Vector::new(0.0, 0.0, 1.0),
    )
    .expect("the bar's own axis is a fine turning axis");
    let turn =
        Isometry::rotation(&turn_axis, 7.0_f64.to_radians()).expect("7 degrees is a finite angle");
    let placement = turn.compose(&at(low));
    let placed_back = placement.inverse();

    let inside_the_turned_block = |point: [f64; 3]| {
        let local = placed_back.apply_to_point(Point::from(point)).coordinates();
        (0..3).all(|axis| 0.0 < local[axis] && local[axis] < size[axis])
    };

    let heights: Vec<f64> = [bar.ends[0], bar.ends[1], low[2], high[2]]
        .into_iter()
        .flat_map(|z| [z - 0.5, z + 0.5])
        .chain([(bar.ends[0] + bar.ends[1]) / 2.0])
        .collect();
    let pythagorean = [
        [5.0, 0.0],
        [4.0, 3.0],
        [3.0, 4.0],
        [0.0, 5.0],
        [-3.0, 4.0],
        [-4.0, 3.0],
        [-5.0, 0.0],
        [-4.0, -3.0],
        [-3.0, -4.0],
        [0.0, -5.0],
        [3.0, -4.0],
        [4.0, -3.0],
    ];
    let turned_points = [10.0, 100.0, 190.0, 280.0, 350.0].map(|degrees: f64| {
        let radians = degrees * PI / 180.0;
        [5.0 * radians.cos(), 5.0 * radians.sin()]
    });

    let mut totals = [0usize; 3];
    for bar_first in [true, false] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bar_solid = bar.build(&mut model);
        let rectangle = ProfileBuilder::new(vec![
            [0.0, 0.0],
            [size[0], 0.0],
            [size[0], size[1]],
            [0.0, size[1]],
        ])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
        let block_solid = model
            .add_extrusion(&rectangle, size[2], &placement)
            .expect("a positive height");

        let [one, other] = if bar_first {
            [bar_solid, block_solid]
        } else {
            [block_solid, bar_solid]
        };
        let mut log = ProvenanceLog::new();
        let answer = model
            .fuse(
                &[one],
                &[other],
                Rule::Union,
                &Policy::certified(),
                &mut log,
            )
            .unwrap_or_else(|why| panic!("the turned notch's union is refused: {why:?}"));
        assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");

        let mut pipe_faces: Vec<(SolidKey, FaceKey)> = Vec::new();
        for &solid in answer.solids() {
            for face in model.solid(solid).expect("this model made the key").faces() {
                if matches!(face.surface(), Surface::Cylinder(_)) {
                    pipe_faces.push((solid, face.key()));
                }
            }
        }
        assert!(
            !pipe_faces.is_empty(),
            "the notch keeps part of the bar's wall"
        );

        let mut refusing: Vec<FaceKey> = Vec::new();
        for [x, y] in pythagorean.into_iter().chain(turned_points) {
            for &z in &heights {
                let point = [x, y, z];
                let clear_of_the_bars_caps =
                    bar.ends.iter().all(|&end| (point[2] - end).abs() > CLEAR);
                let local = placed_back.apply_to_point(Point::from(point)).coordinates();
                let clear_of_the_block = (0..3).all(|axis| {
                    local[axis].abs() > CLEAR && (local[axis] - size[axis]).abs() > CLEAR
                });
                if !clear_of_the_bars_caps || !clear_of_the_block {
                    continue;
                }

                let off = [x / bar.radius, y / bar.radius, 0.0];
                let away = core::array::from_fn(|axis| point[axis] + STEP * off[axis]);
                let back = core::array::from_fn(|axis| point[axis] - STEP * off[axis]);
                let inside = |p: [f64; 3]| bar.holds(p) || inside_the_turned_block(p);
                let on_the_skin = inside(away) != inside(back);

                let mut holding = 0;
                let mut a_face_refused = false;
                for &(solid, face) in &pipe_faces {
                    match holds(&model, solid, face, point) {
                        Ok(OnAFace::Held) => holding += 1,
                        Ok(OnAFace::NotHeld) => (),
                        Ok(other) => panic!(
                            "bar first {bar_first}: {point:?} is clear of every edge and {face:?} says {other:?}"
                        ),
                        Err(named) => {
                            assert_eq!(named, face, "a refusal names the face asked about");
                            a_face_refused = true;
                            if !refusing.contains(&face) {
                                refusing.push(face);
                            }
                        }
                    }
                }
                let expected = usize::from(on_the_skin);
                let agrees = if a_face_refused {
                    holding <= expected
                } else {
                    holding == expected
                };
                assert!(
                    agrees,
                    "bar first {bar_first}: {point:?} on the pipe is {} the skin, and {holding} faces hold it",
                    if on_the_skin { "on" } else { "off" }
                );
                if !a_face_refused {
                    if on_the_skin {
                        totals[0] += 1;
                    } else {
                        totals[1] += 1;
                    }
                }
            }
        }
        totals[2] += refusing.len();
        println!(
            "the notch turned, union, bar first {bar_first}: {} wall faces, {} refused",
            pipe_faces.len(),
            refusing.len()
        );
    }
    println!(
        "in all: {} points held, {} points not held, {} wall faces refused",
        totals[0], totals[1], totals[2]
    );
    assert_eq!(
        totals,
        [220, 86, 0],
        "a turned back face still stands along the bar's axis, so no wall face should refuse"
    );
}

/// What one fuse's faces said, counted.
struct Found {
    faces: usize,
    refused: usize,
    held: usize,
    not_held: usize,
}

fn ask_every_face(shape: &Shape, rule: Rule, bar_first: bool) -> Found {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = shape.bar.build(&mut model);
    let block = shape.block.build(&mut model);
    let [one, other] = if bar_first {
        [bar, block]
    } else {
        [block, bar]
    };
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{} {rule:?} is refused: {why:?}", shape.name));
    assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");

    let surfaces = shape.surfaces();
    let mut on: Vec<Vec<(SolidKey, FaceKey)>> = vec![Vec::new(); surfaces.len()];
    let mut faces = 0;
    for &solid in answer.solids() {
        for face in model.solid(solid).expect("this model made the key").faces() {
            let at = surfaces
                .iter()
                .position(|known| known.is(&face.surface()))
                .unwrap_or_else(|| panic!("a face on a surface of neither: {:?}", face.surface()));
            on[at].push((solid, face.key()));
            faces += 1;
        }
    }

    let inside = |point: [f64; 3]| {
        let [in_the_bar, in_the_block] = [shape.bar.holds(point), shape.block.holds(point)];
        match rule {
            Rule::Union => in_the_bar || in_the_block,
            Rule::Difference if bar_first => in_the_bar && !in_the_block,
            Rule::Difference => in_the_block && !in_the_bar,
            Rule::Intersection => in_the_bar && in_the_block,
            other => panic!("a rule this test does not know: {other:?}"),
        }
    };
    let mut refusing: Vec<FaceKey> = Vec::new();
    let mut found = Found {
        faces,
        refused: 0,
        held: 0,
        not_held: 0,
    };
    for (at, surface) in surfaces.iter().enumerate() {
        for point in surface.points_clear_of(&surfaces, shape) {
            let off = surface.off_it(point, &shape.bar);
            let away = core::array::from_fn(|axis| point[axis] + STEP * off[axis]);
            let back = core::array::from_fn(|axis| point[axis] - STEP * off[axis]);
            let on_the_skin = inside(away) != inside(back);
            let mut holding = 0;
            let mut a_face_refused = false;
            for &(solid, face) in &on[at] {
                match holds(&model, solid, face, point) {
                    Ok(OnAFace::Held) => holding += 1,
                    Ok(OnAFace::NotHeld) => (),
                    Ok(other) => panic!(
                        "{} {rule:?} bar first {bar_first}: {point:?} is clear of every edge and {face:?} says {other:?}",
                        shape.name
                    ),
                    Err(named) => {
                        assert_eq!(named, face, "a refusal names the face asked about");
                        a_face_refused = true;
                        if !refusing.contains(&face) {
                            refusing.push(face);
                        }
                    }
                }
            }
            let expected = usize::from(on_the_skin);
            let agrees = if a_face_refused {
                holding <= expected
            } else {
                holding == expected
            };
            assert!(
                agrees,
                "{} {rule:?} bar first {bar_first}: {point:?} on {surface:?} is {} the skin, and {holding} faces hold it",
                shape.name,
                if on_the_skin { "on" } else { "off" }
            );
            if !a_face_refused {
                if on_the_skin {
                    found.held += 1;
                } else {
                    found.not_held += 1;
                }
            }
        }
    }
    found.refused = refusing.len();
    found
}

/// What `face` says of `point`, or the face a refusal by name names.
fn holds(
    model: &Model,
    solid: SolidKey,
    face: FaceKey,
    point: [f64; 3],
) -> Result<OnAFace, FaceKey> {
    let mut log = ProvenanceLog::new();
    let answer = model
        .solid(solid)
        .expect("this model made the key")
        .face_holds_the_point(
            face,
            rcad_predicates::Point::from(point),
            &Policy::certified(),
            &mut log,
        );
    assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
    match answer {
        Ok(found) => Ok(found),
        Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: named }) => Err(named),
        Err(other) => panic!("{point:?} on {face:?} is refused some other way: {other:?}"),
    }
}

/// A bar and a block, each standing square to the axes.
struct Shape {
    name: &'static str,
    bar: Bar,
    block: Block,
}

const SHAPES: [Shape; 5] = [
    Shape {
        name: "the notch",
        bar: Bar {
            radius: 5.0,
            axis: [0.0, 0.0],
            ends: [0.0, 4.0],
        },
        block: Block {
            low: [2.0, -2.0, 1.0],
            high: [10.0, 2.0, 3.0],
        },
    },
    Shape {
        name: "a bore through one edge",
        bar: Bar {
            radius: 3.0,
            axis: [0.0, 5.0],
            ends: [-1.0, 5.0],
        },
        block: Block {
            low: [0.0, 0.0, 0.0],
            high: [10.0, 10.0, 4.0],
        },
    },
    Shape {
        name: "a bore through one corner",
        bar: Bar {
            radius: 3.0,
            axis: [0.0, 0.0],
            ends: [-1.0, 5.0],
        },
        block: Block {
            low: [0.0, 0.0, 0.0],
            high: [10.0, 10.0, 4.0],
        },
    },
    Shape {
        name: "a box through the caps",
        bar: Bar {
            radius: 5.0,
            axis: [0.0, 0.0],
            ends: [0.0, 4.0],
        },
        block: Block {
            low: [-4.0, -4.0, -1.0],
            high: [4.0, 4.0, 5.0],
        },
    },
    Shape {
        name: "a box through a cap and the wall",
        bar: Bar {
            radius: 5.0,
            axis: [0.0, 0.0],
            ends: [0.0, 4.0],
        },
        block: Block {
            low: [2.0, -2.0, -1.0],
            high: [10.0, 2.0, 5.0],
        },
    },
];

impl Shape {
    /// The bar's pipe, then the planes of its caps and of the block's six
    /// faces.
    fn surfaces(&self) -> Vec<OneSurface> {
        let mut surfaces = vec![OneSurface::Pipe];
        surfaces.extend(self.bar.ends.map(|at| OneSurface::Flat { axis: 2, at }));
        for axis in 0..3 {
            for at in [self.block.low[axis], self.block.high[axis]] {
                surfaces.push(OneSurface::Flat { axis, at });
            }
        }
        surfaces
    }
}

/// A round bar standing up `z`.
struct Bar {
    radius: f64,
    axis: [f64; 2],
    ends: [f64; 2],
}

impl Bar {
    fn build(&self, model: &mut Model) -> SolidKey {
        let [x, y] = self.axis;
        let [bottom, top] = self.ends;
        model
            .add_round_bar(self.radius, top - bottom, &at([x, y, bottom]))
            .expect("a bar of positive size")
    }

    fn holds(&self, point: [f64; 3]) -> bool {
        let [bottom, top] = self.ends;
        self.squared_from_the_axis(point) < self.radius * self.radius
            && bottom < point[2]
            && point[2] < top
    }

    fn squared_from_the_axis(&self, point: [f64; 3]) -> f64 {
        let [x, y] = [point[0] - self.axis[0], point[1] - self.axis[1]];
        x * x + y * y
    }
}

/// A block between two corners.
struct Block {
    low: [f64; 3],
    high: [f64; 3],
}

impl Block {
    fn build(&self, model: &mut Model) -> SolidKey {
        let [x, y, z] = core::array::from_fn(|axis| self.high[axis] - self.low[axis]);
        let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
        model
            .add_extrusion(&rectangle, z, &at(self.low))
            .expect("a positive height")
    }

    fn holds(&self, point: [f64; 3]) -> bool {
        (0..3).all(|axis| self.low[axis] < point[axis] && point[axis] < self.high[axis])
    }
}

/// One of the surfaces a face of the answer can lie on.
#[derive(Debug, Clone, Copy)]
enum OneSurface {
    /// The bar's pipe.
    Pipe,
    /// The plane where coordinate `axis` is `at`.
    Flat { axis: usize, at: f64 },
}

impl OneSurface {
    /// Whether a face's surface, moved into place, is this one.
    fn is(&self, surface: &Surface) -> bool {
        match (self, surface) {
            (Self::Pipe, Surface::Cylinder(_)) => true,
            (Self::Flat { axis, at }, Surface::Plane(flat)) => {
                let normal = flat.normal().components();
                let along = normal[*axis];
                (along == 1.0 || along == -1.0)
                    && (0..3).all(|other| other == *axis || normal[other] == 0.0)
                    && flat.offset() * along == *at
            }
            (Self::Pipe | Self::Flat { .. }, _) => false,
        }
    }

    /// A direction off this surface at `point`.
    fn off_it(&self, point: [f64; 3], bar: &Bar) -> [f64; 3] {
        match self {
            Self::Pipe => {
                let [x, y] = [point[0] - bar.axis[0], point[1] - bar.axis[1]];
                let length = x.hypot(y);
                [x / length, y / length, 0.0]
            }
            Self::Flat { axis, .. } => core::array::from_fn(|other| f64::from(other == *axis)),
        }
    }

    /// Points on this surface further than [`CLEAR`] from every other one of
    /// `surfaces`. On the pipe they are at the corners of 3-4-5 triangles,
    /// which are on it exactly for a radius of 5 and to within rounding for 3,
    /// and a few angles more; on a plane they are a grid across it.
    fn points_clear_of(&self, surfaces: &[OneSurface], shape: &Shape) -> Vec<[f64; 3]> {
        let bar = &shape.bar;
        let [bottom, top] = bar.ends;
        let heights: Vec<f64> = [bottom, top, shape.block.low[2], shape.block.high[2]]
            .into_iter()
            .flat_map(|z| [z - 0.5, z + 0.5])
            .chain([(bottom + top) / 2.0])
            .collect();
        let candidates: Vec<[f64; 3]> = match self {
            Self::Pipe => {
                let pythagorean = [
                    [5.0, 0.0],
                    [4.0, 3.0],
                    [3.0, 4.0],
                    [0.0, 5.0],
                    [-3.0, 4.0],
                    [-4.0, 3.0],
                    [-5.0, 0.0],
                    [-4.0, -3.0],
                    [-3.0, -4.0],
                    [0.0, -5.0],
                    [3.0, -4.0],
                    [4.0, -3.0],
                ];
                let turned = [10.0, 100.0, 190.0, 280.0, 350.0].map(|degrees: f64| {
                    let turn = degrees * PI / 180.0;
                    [5.0 * turn.cos(), 5.0 * turn.sin()]
                });
                pythagorean
                    .into_iter()
                    .chain(turned)
                    .flat_map(|[x, y]| {
                        let scale = bar.radius / 5.0;
                        let [x, y] = [x * scale + bar.axis[0], y * scale + bar.axis[1]];
                        heights.iter().map(move |&z| [x, y, z])
                    })
                    .collect()
            }
            Self::Flat { axis, at } => {
                let across = [
                    -6.5, -5.5, -4.5, -3.5, -2.5, -1.5, -1.0, -0.5, 0.5, 1.0, 1.5, 2.5, 3.0, 3.5,
                    4.5, 5.5, 6.5, 7.5, 8.5, 9.5, 10.5,
                ];
                let [first, second] = match axis {
                    0 => [1, 2],
                    1 => [0, 2],
                    _ => [0, 1],
                };
                let mut grid = Vec::new();
                for one in across {
                    for other in across.iter().copied().chain(heights.iter().copied()) {
                        let mut point = [0.0; 3];
                        point[*axis] = *at;
                        point[first] = one;
                        point[second] = other;
                        grid.push(point);
                    }
                }
                grid
            }
        };
        candidates
            .into_iter()
            .filter(|point| {
                surfaces
                    .iter()
                    .all(|other| other.is_this(self) || other.how_far(*point, bar) > CLEAR)
            })
            .collect()
    }

    fn is_this(&self, other: &OneSurface) -> bool {
        match (self, other) {
            (Self::Pipe, Self::Pipe) => true,
            (
                Self::Flat { axis, at },
                Self::Flat {
                    axis: theirs,
                    at: there,
                },
            ) => axis == theirs && at == there,
            (Self::Pipe, Self::Flat { .. }) | (Self::Flat { .. }, Self::Pipe) => false,
        }
    }

    /// How far `point` is from this surface, taken as going on for ever.
    fn how_far(&self, point: [f64; 3], bar: &Bar) -> f64 {
        match self {
            Self::Pipe => (bar.squared_from_the_axis(point).sqrt() - bar.radius).abs(),
            Self::Flat { axis, at } => (point[*axis] - at).abs(),
        }
    }
}

fn at(shift: [f64; 3]) -> Isometry {
    let [x, y, z] = shift;
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
