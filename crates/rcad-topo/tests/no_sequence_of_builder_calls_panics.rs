//! Scripts of builder calls nobody wrote, and a cube described in every order.
//!
//! `docs/decisions.md` §42.12 criterion 2: *no sequence of builder calls panics;
//! every misuse is a named `Err`*. §42.5's rule is that once the builder is
//! public every arena invariant is caller input, and §4.1's tier table says
//! caller input is an `Err` and never a panic. This is that rule checked over
//! inputs nobody chose.
//!
//! **Two families, reported separately** (§17.7, and the testing skill's
//! generator rules), and neither is a percentage of the other.
//!
//! *Scripts nobody wrote* draws a list of calls and runs them, each handle taken
//! modulo how many of that kind the builder has handed out. Almost every script
//! describes nonsense. The property is not that it builds — it is that nothing
//! panics, that a refused build leaves all seven arenas exactly as it found
//! them, and that whatever the model does end up holding passes the sweep.
//!
//! **No random script ever builds a solid, and that is reported rather than
//! hidden**: on the shipped corpus and on a two-hundredfold soak the count of
//! builds accepted is zero, because a shell a draw stumbles into always has
//! edges nothing walks twice. That is why the third family exists — *a cube
//! with a drawn surface* is structurally perfect every time, so it reaches
//! `Model::validate`, which is where a panic on a caller's geometry would hide.
//!
//! *The same cube in any order* is the **control**. Six faces of the unit cube
//! in a drawn order, each loop started at a drawn corner, is the same solid
//! every time and is exactly representable end to end: `Policy::certified()`
//! has to decide all of it, the volume has to come back exactly one, and **one
//! refusal is one bug**. Without it, a green run of the first family would be
//! consistent with a builder that refuses everything.
//!
//! **The counting.** The first family reports how many calls it got accepted,
//! how many refused, and how many builds it got through, and asserts a floor on
//! each. A generator that bounced off the first check every time would leave
//! the accepted count at zero and every assertion above it green — which is the
//! testing skill's first way a suite goes vacuous.
//!
//! The draw is a deterministic sequence, so a number here is one number and not
//! a range. `RCAD_SOAK_FACTOR` multiplies how many are drawn.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftCoedge, DraftFace, DraftStep, DraftVertex, DraftWire, Model, SolidBuilder};

/// How many scripts, and how many orders of the cube, before the soak factor.
const DRAWS: usize = 512;

/// Nothing panics, a refused build writes nothing, and what does get written
/// passes the sweep.
#[test]
fn a_script_nobody_wrote_is_refused_rather_than_crashing() {
    let policy = Policy::certified();
    let mut draw = Draw(0x5EED_1234_5678_9ABC);
    let mut tally = Tally::default();

    for _ in 0..how_many() {
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let mut described = Described::default();
        let mut builder = SolidBuilder::new();

        for _ in 0..draw.upto(40) + 1 {
            match described.one_call(&mut builder, &mut draw) {
                Some(Ok(())) => tally.accepted += 1,
                Some(Err(_)) => tally.refused += 1,
                None => tally.skipped += 1,
            }
        }

        let before = sizes(&model);
        tally.builds += 1;
        match builder.build(&mut model, &policy, &mut log) {
            Ok(_) => {
                tally.builds_accepted += 1;
                assert_eq!(model.solid_count(), 1);
                model.validate(&policy, &mut log).expect(
                    "a build the builder accepted left a model the sweep refuses, \
                     which is the two halves of the checking disagreeing",
                );
            }
            Err(_) => assert_eq!(
                sizes(&model),
                before,
                "a refused build wrote something into the model"
            ),
        }
    }

    tally.report("scripts nobody wrote");
}

/// A cube with one face's surface and one corner drawn, so the description is
/// always structurally perfect and almost never geometrically right.
///
/// This is the family that reaches `Model::validate` — the random scripts above
/// never do, because a shell they stumble into has edges nothing walks twice,
/// and the builder's own checks stop them first. What it puts in front of the
/// sweep is a solid whose topology is a cube and whose geometry is not: a face
/// claiming to be a ball, a corner off its own plane, a wire winding the wrong
/// way. Meshing one of those is where a panic would hide.
///
/// **Either answer is a pass and neither may be a crash.** The drawn surface is
/// sometimes the face's own, and then the sweep must accept; usually it is not,
/// and then the sweep must name what is wrong.
#[test]
fn a_cube_with_a_drawn_surface_on_one_face_is_named_rather_than_crashing() {
    let policy = Policy::certified();
    let mut draw = Draw(0x0BAD_C0DE_1234_5EED);
    let (mut accepted, mut refused) = (0usize, 0usize);

    for _ in 0..how_many() / 4 {
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        // Each knob is off about half the time, so the plain cube — which the
        // sweep must accept — is drawn as often as the broken ones.
        let which_face = draw.upto(6);
        let which_surface = draw.upto(10);
        let moved_corner = (
            if draw.upto(2) == 0 { draw.upto(8) } else { 8 },
            a_lattice_point(draw.upto(27)),
        );

        let mut builder = SolidBuilder::new();
        let corners: Vec<DraftVertex> = CORNERS
            .iter()
            .enumerate()
            .map(|(which, &[x, y, z])| {
                let point = if which == moved_corner.0 {
                    moved_corner.1
                } else {
                    Point::new(x, y, z)
                };
                builder
                    .add_vertex(point)
                    .expect("a lattice point is finite")
            })
            .collect();
        let sides: Vec<_> = SIDES
            .iter()
            .map(|&[from, to]| builder.add_edge(corners[from], corners[to], EdgeRun::Straight))
            .collect();
        // A moved corner can land on a neighbour, and a straight edge of no
        // length is refused there. That is the right answer and not this
        // family's subject, so the draw is thrown away rather than forced.
        let Ok(sides) = sides.into_iter().collect::<rcad_topo::Result<Vec<_>>>() else {
            continue;
        };

        let mut faces = Vec::new();
        for (which, (through, normal, walk)) in FACES.into_iter().enumerate() {
            let steps: Vec<DraftStep> = walk
                .into_iter()
                .map(|(side, forwards)| {
                    DraftStep::Along(
                        builder
                            .add_coedge(sides[side], turn(forwards))
                            .expect("the side is here"),
                    )
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("a face of a cube closes");
            let surface = if which == which_face && which_surface < 5 {
                a_surface(which_surface)
            } else {
                Surface::Plane(
                    Plane::from_point_and_normal(
                        Point::new(through[0], through[1], through[2]),
                        Vector::new(normal[0], normal[1], normal[2]),
                    )
                    .expect("an axis is a unit normal"),
                )
            };
            faces.push((
                builder
                    .add_face(surface, &[wire])
                    .expect("a face has its wire"),
                Orientation::Forward,
            ));
        }
        builder.add_shell(&faces).expect("six faces make one shell");

        match builder.build(&mut model, &policy, &mut log) {
            Ok(_) => {
                accepted += 1;
                model
                    .validate(&policy, &mut log)
                    .expect("a build the builder accepted left a model the sweep refuses");
            }
            Err(_) => {
                refused += 1;
                assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
            }
        }
    }

    println!("a cube with a drawn surface: {accepted} accepted, {refused} refused, none crashed");
    assert!(
        accepted > 0,
        "every draw was refused, so the accepting half was never exercised"
    );
    assert!(
        refused > 0,
        "no draw was refused, so the sweep is not looking at the geometry at all"
    );
}

/// The control: the same cube however it is described.
///
/// Six faces in a drawn order, each loop started at a drawn corner. Every one
/// of them is the unit cube, so a certified policy has to decide all of it, the
/// log has to come back empty, and the volume has to be exactly one.
#[test]
fn the_same_cube_described_in_any_order_is_the_same_cube() {
    let policy = Policy::certified();
    let mut draw = Draw(0x1234_5EED_9ABC_5678);
    let mut built = 0;

    for _ in 0..how_many() {
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let mut builder = SolidBuilder::new();

        let corners: Vec<DraftVertex> = CORNERS
            .iter()
            .map(|&[x, y, z]| {
                builder
                    .add_vertex(Point::new(x, y, z))
                    .expect("a corner of the unit cube")
            })
            .collect();
        let sides: Vec<_> = SIDES
            .iter()
            .map(|&[from, to]| {
                builder
                    .add_edge(corners[from], corners[to], EdgeRun::Straight)
                    .expect("a side of the unit cube")
            })
            .collect();

        let mut order: Vec<usize> = (0..6).collect();
        for position in (1..6).rev() {
            order.swap(position, draw.upto(position + 1));
        }
        let mut faces = Vec::new();
        for &which in &order {
            let (through, normal, walk) = FACES[which];
            let mut walk = walk.to_vec();
            walk.rotate_left(draw.upto(4));
            let steps: Vec<DraftStep> = walk
                .into_iter()
                .map(|(side, forwards)| {
                    DraftStep::Along(
                        builder
                            .add_coedge(sides[side], turn(forwards))
                            .expect("the side is here"),
                    )
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("a face of a cube closes");
            let flat = Plane::from_point_and_normal(
                Point::new(through[0], through[1], through[2]),
                Vector::new(normal[0], normal[1], normal[2]),
            )
            .expect("an axis is a unit normal");
            faces.push((
                builder
                    .add_face(Surface::Plane(flat), &[wire])
                    .expect("a face has its wire"),
                Orientation::Forward,
            ));
        }
        builder.add_shell(&faces).expect("six faces make one shell");

        let cube = builder
            .build(&mut model, &policy, &mut log)
            .expect("the unit cube, whichever order it was described in");
        assert!(
            log.is_empty(),
            "a cube on the unit lattice needs nothing approximated"
        );
        let measured = model
            .solid(cube)
            .expect("this model made the key")
            .measure(Deflection::for_scale(model.scale()), &policy, &mut log)
            .expect("a polyhedron measures over its own fan");
        assert_eq!(measured.volume(), 1.0);
        assert_eq!(measured.surface_area(), 6.0);
        assert_eq!(measured.centroid(), Point::new(0.5, 0.5, 0.5));
        built += 1;
    }

    println!("the same cube in any order: {built} descriptions, all accepted");
    assert_eq!(
        built,
        how_many(),
        "the control is not allowed a single refusal"
    );
}

/// What the handles a script has been given so far, and the next call over
/// them.
#[derive(Default)]
struct Described {
    vertices: Vec<DraftVertex>,
    edges: Vec<rcad_topo::DraftEdge>,
    coedges: Vec<DraftCoedge>,
    wires: Vec<DraftWire>,
    faces: Vec<DraftFace>,
}

impl Described {
    /// `None` when the draw named a kind of handle nothing has handed out yet,
    /// so nothing was called at all.
    fn one_call(
        &mut self,
        builder: &mut SolidBuilder,
        draw: &mut Draw,
    ) -> Option<rcad_topo::Result<()>> {
        match draw.upto(6) {
            0 => Some(
                builder
                    .add_vertex(a_lattice_point(draw.upto(27)))
                    .map(|handle| self.vertices.push(handle)),
            ),
            1 => {
                let start = *pick(&self.vertices, draw)?;
                let end = *pick(&self.vertices, draw)?;
                Some(
                    builder
                        .add_edge(start, end, a_run(draw.upto(3)))
                        .map(|handle| self.edges.push(handle)),
                )
            }
            2 => {
                let edge = *pick(&self.edges, draw)?;
                Some(
                    builder
                        .add_coedge(edge, turn(draw.upto(2) == 0))
                        .map(|handle| self.coedges.push(handle)),
                )
            }
            3 => {
                let mut steps = Vec::new();
                for _ in 0..draw.upto(6) {
                    if draw.upto(4) == 0 {
                        steps.push(DraftStep::Pole(*pick(&self.vertices, draw)?));
                    } else {
                        steps.push(DraftStep::Along(*pick(&self.coedges, draw)?));
                    }
                }
                Some(
                    builder
                        .add_wire(&steps)
                        .map(|handle| self.wires.push(handle)),
                )
            }
            4 => {
                let mut wires = Vec::new();
                for _ in 0..draw.upto(3) {
                    wires.push(*pick(&self.wires, draw)?);
                }
                Some(
                    builder
                        .add_face(a_surface(draw.upto(5)), &wires)
                        .map(|handle| self.faces.push(handle)),
                )
            }
            _ => {
                let mut faces = Vec::new();
                for _ in 0..draw.upto(8) {
                    faces.push((*pick(&self.faces, draw)?, turn(draw.upto(2) == 0)));
                }
                Some(builder.add_shell(&faces))
            }
        }
    }
}

/// What a family actually did, so a green run cannot mean it did nothing.
#[derive(Default)]
struct Tally {
    accepted: usize,
    refused: usize,
    skipped: usize,
    builds: usize,
    builds_accepted: usize,
}

impl Tally {
    fn report(&self, family: &str) {
        let Tally {
            accepted,
            refused,
            skipped,
            builds,
            builds_accepted,
        } = *self;
        println!(
            "{family}: {builds} scripts, {accepted} calls accepted, {refused} refused, \
             {skipped} with nothing to name yet, {builds_accepted} builds accepted"
        );
        assert!(builds > 0, "no script ran at all");
        assert!(
            accepted > builds,
            "the draw never got past the first call, so nothing below it was \
             reached: {accepted} accepted over {builds} scripts"
        );
        assert!(
            refused > 0,
            "the draw never got a call refused, so the refusals were never \
             exercised"
        );
    }
}

/// The seven arena sizes, so a refused build can be shown to have written none
/// of them.
fn sizes(model: &Model) -> [usize; 7] {
    [
        model.vertex_count(),
        model.edge_count(),
        model.coedge_count(),
        model.wire_count(),
        model.face_count(),
        model.shell_count(),
        model.solid_count(),
    ]
}

fn pick<'a, T>(handles: &'a [T], draw: &mut Draw) -> Option<&'a T> {
    if handles.is_empty() {
        return None;
    }
    handles.get(draw.upto(handles.len()))
}

fn turn(forwards: bool) -> Orientation {
    if forwards {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}

/// A corner of a `3 × 3 × 3` whole-number lattice, so a script's coordinates
/// are exact and nothing is refused for arithmetic that was never the point.
fn a_lattice_point(which: usize) -> Point {
    let along = |axis: u32| f64::from(u32::try_from(which).unwrap_or(0) / 3u32.pow(axis) % 3);
    Point::new(along(0), along(1), along(2))
}

fn a_run(which: usize) -> EdgeRun {
    match which {
        0 => EdgeRun::Straight,
        1 => EdgeRun::RightRoundWhereItsFacesCross,
        _ => EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                    .expect("the y axis is a unit normal"),
            ),
            picked_by: smallvec::SmallVec::new(),
        },
    }
}

/// One of the three kinds of surface the kernel holds, on the same lattice.
fn a_surface(which: usize) -> Surface {
    let flat = |through: Point, normal: Vector| {
        Surface::Plane(
            Plane::from_point_and_normal(through, normal).expect("an axis is a unit normal"),
        )
    };
    match which {
        0 => flat(Point::origin(), Vector::new(0.0, 0.0, 1.0)),
        1 => flat(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0)),
        2 => flat(Point::new(0.0, 1.0, 0.0), Vector::new(0.0, 1.0, 0.0)),
        3 => Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a ball of radius one")),
        _ => Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a pipe of radius one")),
    }
}

/// How many of each family to draw. `RCAD_SOAK_FACTOR` multiplies it, the way
/// the rest of the suite does.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number above zero, or
/// to one whose product is more than a run can count. Both used to run the
/// shipped corpus or less and say nothing about it: a factor that did not parse
/// fell back to one, and a product that wrapped drew **nought** while the
/// operator read the number as the wide one (§48.2, §50.9). Measured: at a
/// factor of 36 028 797 018 963 968 this drew no cubes at all and the run went
/// green.
fn how_many() -> usize {
    let Ok(factor) = std::env::var("RCAD_SOAK_FACTOR") else {
        return DRAWS;
    };
    let factor: usize = factor
        .parse()
        .unwrap_or_else(|_| panic!("RCAD_SOAK_FACTOR has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "RCAD_SOAK_FACTOR has to be at least one");
    DRAWS
        .checked_mul(factor)
        .unwrap_or_else(|| panic!("{DRAWS} draws times {factor} is more than one run can count"))
}

/// A deterministic sequence, so a number reported here is one number and not a
/// range.
struct Draw(u64);

impl Draw {
    fn upto(&mut self, most: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        if most == 0 {
            return 0;
        }
        usize::try_from(self.0 >> 33).unwrap_or(0) % most
    }
}

/// The eight corners of the unit cube, `(x, y, z)` bit by bit.
const CORNERS: [[f64; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0],
];

/// The twelve sides: the bottom ring, the top ring, then the four uprights.
const SIDES: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

/// A point on the plane, its outward normal, and the four sides of its loop
/// with `true` for walking a side the way it is stored.
///
/// Every loop runs anticlockwise seen from outside, which is anticlockwise
/// about the outward normal.
type AFace = ([f64; 3], [f64; 3], [(usize, bool); 4]);
const FACES: [AFace; 6] = [
    (
        [0.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [(3, false), (2, false), (1, false), (0, false)],
    ),
    (
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [(4, true), (5, true), (6, true), (7, true)],
    ),
    (
        [0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [(0, true), (9, true), (4, false), (8, false)],
    ),
    (
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [(1, true), (10, true), (5, false), (9, false)],
    ),
    (
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [(2, true), (11, true), (6, false), (10, false)],
    ),
    (
        [0.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [(3, true), (8, true), (7, false), (11, false)],
    ),
];
