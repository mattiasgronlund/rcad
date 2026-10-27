//! What rcad-brep needs from rcad when an OCCT solid has four faces along one
//! edge (`docs/decisions.md` §469), reached with `rcad` as the only dependency.
//!
//! The shape is the one rcad-brep's brief drew: an L plate whose square hole
//! has a corner on the L's inner corner. Going round the line up through that
//! corner, the quarters are plate, air, plate, air, and four walls meet along
//! it. **Written as one edge with four faces along it, `SolidBuilder` refuses
//! it. Written as two edges on that line, two walls each, it builds**, with
//! either pairing that has each pair walking the line opposite ways. The fuse
//! already writes a solid that touches itself along a line this way (§39.4),
//! and it is what rcad-brep splits such an edge into.
//!
//! The first test is what stops the others being a tautology: the same faces
//! written with one edge up the corner are refused, so the other two build
//! because of how the line is written, and not because the builder takes
//! anything. Every corner is on whole or half numbers, so the volumes are exact.

use std::collections::BTreeMap;

use rcad::{
    Deflection, DraftEdge, DraftFace, DraftStep, DraftVertex, DraftWire, EdgeRun, Model,
    Orientation, Plane, Point, Policy, ProvenanceLog, Rule, Scale, SolidBuilder, SolidKey, Surface,
    TopologyError, Vector,
};

/// The plate seen from above: the L's outline, then its hole, each walked with
/// the plate on its left. It stands from `z = 0` to `z = 1`.
const CORNERS: [[f64; 2]; 9] = [
    [0.0, 0.0],
    [4.0, 0.0],
    [4.0, 2.0],
    [2.0, 2.0],
    [2.0, 4.0],
    [0.0, 4.0],
    [1.0, 1.0],
    [2.0, 1.0],
    [1.0, 2.0],
];
const OUTLINE: [usize; 6] = [0, 1, 2, 3, 4, 5];
const HOLE: [usize; 4] = [6, 8, 3, 7];

/// `(2, 2)`, the L's inner corner and a corner of the hole.
const WHERE_THEY_TOUCH: usize = 3;

/// A four by four square, less the two by two notch and the one by one hole.
const THE_PLATES_VOLUME: f64 = 11.0;

/// How the four walls along the line up through `(2, 2)` are shared out among
/// the edges on it.
#[derive(Clone, Copy, Debug)]
enum UpTheCorner {
    OneEdge,
    /// The notch's two walls on one edge and the hole's two on the other, so
    /// each edge has a quarter of air between its walls.
    TwoEdgesPairedAcrossTheAir,
    /// Each edge has a quarter of the plate between its walls.
    TwoEdgesPairedAcrossThePlate,
}

const THE_TWO_PAIRINGS: [UpTheCorner; 2] = [
    UpTheCorner::TwoEdgesPairedAcrossTheAir,
    UpTheCorner::TwoEdgesPairedAcrossThePlate,
];

#[test]
fn an_l_plate_whose_hole_touches_its_inner_corner_is_refused_with_one_edge_up_that_corner() {
    let refused = the_plate(UpTheCorner::OneEdge).build(
        &mut a_model(),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(
            refused,
            Err(TopologyError::ADraftEdgeIsNotWalkedByTwoCoedges { count: 4, .. })
        ),
        "{refused:?}"
    );
}

#[test]
fn the_same_plate_with_two_edges_up_that_corner_builds_whichever_way_its_walls_are_paired() {
    for pairing in THE_TWO_PAIRINGS {
        let mut model = a_model();
        let plate = the_plate(pairing)
            .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{pairing:?}: {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{pairing:?}: {why:?}"));
        assert_eq!(
            volume_of(&model, plate),
            (THE_PLATES_VOLUME, 0.0),
            "{pairing:?}"
        );
    }
}

/// The box holds a half by half by half cube of the plate either side of the
/// line, in the two quarters that meet along it.
#[test]
fn a_box_cut_across_the_line_the_plate_touches_itself_along_takes_off_what_it_holds() {
    for pairing in THE_TWO_PAIRINGS {
        let mut model = a_model();
        let plate = the_plate(pairing)
            .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{pairing:?}: {why:?}"));
        let cutter = a_box([1.5, 1.5, 0.5], [2.5, 2.5, 1.5])
            .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a box");
        let fused = model
            .fuse(
                &[plate],
                &[cutter],
                Rule::Difference,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{pairing:?}: {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{pairing:?}: {why:?}"));
        let [cut] = fused.solids() else {
            panic!("{pairing:?}: {:?}", fused.solids());
        };
        assert_eq!(
            volume_of(&model, *cut),
            (THE_PLATES_VOLUME - 2.0 * 0.5 * 0.5 * 0.5, 0.0),
            "{pairing:?}"
        );
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn the_plate(pairing: UpTheCorner) -> SolidBuilder {
    a_prism(&CORNERS, &[&OUTLINE, &HOLE], [0.0, 1.0], |at, wall| {
        if at == WHERE_THEY_TOUCH {
            pairing.edge_for(wall)
        } else {
            0
        }
    })
}

impl UpTheCorner {
    /// Which of the edges up `(2, 2)` the wall from corner `wall.0` to corner
    /// `wall.1` runs along.
    fn edge_for(self, wall: (usize, usize)) -> usize {
        let notch = [(2, 3), (3, 4)];
        let hole = [(8, 3), (3, 7)];
        assert!(
            notch.contains(&wall) || hole.contains(&wall),
            "{wall:?} is not one of the four walls up (2, 2)"
        );
        match self {
            Self::OneEdge => 0,
            Self::TwoEdgesPairedAcrossTheAir => usize::from(hole.contains(&wall)),
            Self::TwoEdgesPairedAcrossThePlate => usize::from(wall == (3, 4) || wall == (8, 3)),
        }
    }
}

fn a_box(low: [f64; 3], high: [f64; 3]) -> SolidBuilder {
    let corners = [
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ];
    a_prism(&corners, &[&[0, 1, 2, 3]], [low[2], high[2]], |_, _| 0)
}

fn volume_of(model: &Model, solid: SolidKey) -> (f64, f64) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.1).expect("a tenth of a millimetre"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid measures");
    (measured.volume(), measured.volume_error())
}

/// A prism standing from `bottom` to `top` over `wires`, each a ring of
/// `corners` walked with the solid on its left seen from above.
///
/// `which_edge_up(corner, wall)` says which of the edges up `corner` the wall
/// from corner `wall.0` to corner `wall.1` runs along. Every wall that names
/// the same one shares one edge.
fn a_prism(
    corners: &[[f64; 2]],
    wires: &[&[usize]],
    [bottom, top]: [f64; 2],
    which_edge_up: impl Fn(usize, (usize, usize)) -> usize,
) -> SolidBuilder {
    let mut builder = SolidBuilder::new();
    let up = corners.len();
    let drafted = [bottom, top]
        .into_iter()
        .flat_map(|z| corners.iter().map(move |&[x, y]| Point::new(x, y, z)))
        .map(|point| builder.add_vertex(point).expect("a finite corner"))
        .collect();
    let mut prism = Prism {
        builder,
        corners: drafted,
        edges: BTreeMap::new(),
    };

    let mut top_wires = Vec::new();
    let mut bottom_wires = Vec::new();
    for ring in wires {
        let next = |i: usize| ring[(i + 1) % ring.len()];
        let round_the_top: Vec<DraftStep> = (0..ring.len())
            .map(|i| prism.step(ring[i] + up, next(i) + up, 0))
            .collect();
        top_wires.push(prism.wire(&round_the_top));
        let round_the_bottom: Vec<DraftStep> = (0..ring.len())
            .rev()
            .map(|i| prism.step(next(i), ring[i], 0))
            .collect();
        bottom_wires.push(prism.wire(&round_the_bottom));
    }
    let mut shell = vec![
        prism.face([0.0, 0.0, top], [0.0, 0.0, 1.0], &top_wires),
        prism.face([0.0, 0.0, bottom], [0.0, 0.0, -1.0], &bottom_wires),
    ];
    for ring in wires {
        for i in 0..ring.len() {
            let wall = (ring[i], ring[(i + 1) % ring.len()]);
            let (from, to) = wall;
            let steps = [
                prism.step(to + up, from + up, 0),
                prism.step(from + up, from, which_edge_up(from, wall)),
                prism.step(from, to, 0),
                prism.step(to, to + up, which_edge_up(to, wall)),
            ];
            let around = prism.wire(&steps);
            let ([x0, y0], [x1, y1]) = (corners[from], corners[to]);
            shell.push(prism.face([x0, y0, bottom], [y1 - y0, x0 - x1, 0.0], &[around]));
        }
    }
    prism.builder.add_shell(&shell).expect("every face once");
    prism.builder
}

struct Prism {
    builder: SolidBuilder,
    corners: Vec<DraftVertex>,
    edges: BTreeMap<(usize, usize, usize), (usize, DraftEdge)>,
}

impl Prism {
    /// One step from corner `from` to corner `to`, along edge number `which` of
    /// those between the two, which the first step along it makes.
    fn step(&mut self, from: usize, to: usize, which: usize) -> DraftStep {
        let between = (from.min(to), from.max(to), which);
        let (edge, orientation) = match self.edges.get(&between) {
            Some(&(start, edge)) if start == from => (edge, Orientation::Forward),
            Some(&(_, edge)) => (edge, Orientation::Reversed),
            None => {
                let edge = self
                    .builder
                    .add_edge(self.corners[from], self.corners[to], EdgeRun::Straight)
                    .expect("two different corners");
                self.edges.insert(between, (from, edge));
                (edge, Orientation::Forward)
            }
        };
        DraftStep::Along(
            self.builder
                .add_coedge(edge, orientation)
                .expect("an edge of this builder"),
        )
    }

    fn wire(&mut self, steps: &[DraftStep]) -> DraftWire {
        self.builder
            .add_wire(steps)
            .expect("each ring of steps here closes")
    }

    fn face(
        &mut self,
        through: [f64; 3],
        normal: [f64; 3],
        wires: &[DraftWire],
    ) -> (DraftFace, Orientation) {
        let [x, y, z] = through;
        let [nx, ny, nz] = normal;
        let plane = Plane::from_point_and_normal(Point::new(x, y, z), Vector::new(nx, ny, nz))
            .expect("a wall has some length");
        let face = self
            .builder
            .add_face(Surface::Plane(plane), wires)
            .expect("wires of this builder");
        (face, Orientation::Forward)
    }
}
