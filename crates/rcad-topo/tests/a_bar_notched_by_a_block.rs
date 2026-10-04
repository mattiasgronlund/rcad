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
//! plane of one of the block's faces.
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

use rcad_geom::{Isometry, Scale, Surface, Vector};
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
/// the plane that cut them, 150 faces refused here, where these add to 75: the
/// notch united `19` each way round and less the block `19`, the bores through
/// an edge `5` and through a corner `13`, `13` and `11`.
///
/// - **The notch united, `14`**: twelve patches of the wall have a ruling at
///   `x = 2, y = ±√21` on a side. The block's back face is the plane that cut
///   the wall there, and the union keeps none of it, so no face of the answer
///   holds that ruling and nothing says which side of it the patch keeps. A
///   gap: the fuse knows the plane and stores the ruling as straight. The other
///   two are the block's top and floor, below.
/// - **The block's top or floor keeping the outside of the pipe across a strip,
///   `2`, in the notch united and the block less the bar**: the strip
///   `|y| ≤ 2` crosses the bar's circle from side to side, so the part of the
///   plane in front of every cut is two pieces, one each side of the bar. A
///   gap: the face is the piece on its own side.
/// - **The bored corner's floor and top, `2`**: the square less a quarter of a
///   disc, pinned refused in `a_cap_trimmed_by_a_pipe_holds_its_points.rs`.
/// - **Four of the box through the caps' `6`**: each quarter of the wall inside
///   the box is cut by two of its sides, which meet outside the pipe, so the
///   part in front of both is two runs of rulings. A gap.
/// - **An arc against a plane that crosses its circle and holds neither of its
///   ends, the `2` in every other case**: a cap of the bar cut along a side of
///   the block, or a face of the block inside the bar. A gap.
/// - **The box through a cap and the wall's `3`** are its two caps, each a disc
///   with the box's end bitten out of its rim, which no cut keeps whole, and the
///   wall, which keeps the outside of the strip `|y| ≤ 2` where one of two flats
///   is read for a strip kept between them. Gaps, both.
const REFUSED: [(&str, Rule, bool, usize); 30] = [
    ("the notch", Rule::Union, true, 14),
    ("the notch", Rule::Union, false, 14),
    ("the notch", Rule::Difference, true, 2),
    ("the notch", Rule::Difference, false, 2),
    ("the notch", Rule::Intersection, true, 2),
    ("the notch", Rule::Intersection, false, 2),
    ("a bore through one edge", Rule::Union, true, 0),
    ("a bore through one edge", Rule::Union, false, 0),
    ("a bore through one edge", Rule::Difference, true, 0),
    ("a bore through one edge", Rule::Difference, false, 0),
    ("a bore through one edge", Rule::Intersection, true, 0),
    ("a bore through one edge", Rule::Intersection, false, 0),
    ("a bore through one corner", Rule::Union, true, 2),
    ("a bore through one corner", Rule::Union, false, 2),
    ("a bore through one corner", Rule::Difference, true, 0),
    ("a bore through one corner", Rule::Difference, false, 2),
    ("a bore through one corner", Rule::Intersection, true, 0),
    ("a bore through one corner", Rule::Intersection, false, 0),
    ("a box through the caps", Rule::Union, true, 0),
    ("a box through the caps", Rule::Union, false, 0),
    ("a box through the caps", Rule::Difference, true, 0),
    ("a box through the caps", Rule::Difference, false, 6),
    ("a box through the caps", Rule::Intersection, true, 6),
    ("a box through the caps", Rule::Intersection, false, 6),
    ("a box through a cap and the wall", Rule::Union, true, 3),
    ("a box through a cap and the wall", Rule::Union, false, 3),
    (
        "a box through a cap and the wall",
        Rule::Difference,
        true,
        3,
    ),
    (
        "a box through a cap and the wall",
        Rule::Difference,
        false,
        2,
    ),
    (
        "a box through a cap and the wall",
        Rule::Intersection,
        true,
        2,
    ),
    (
        "a box through a cap and the wall",
        Rule::Intersection,
        false,
        2,
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
