//! A plate perforated in a checkerboard of triangles: three holes meeting
//! along one line, with the plate between them.
//!
//! The plate is `[0, 6]² × [0, 1]`. The holes are prisms from `z = −1` to
//! `z = 2` over the triangles `(3, 3) (4, 4) (3, 4)` and `(3, 3) (2, 3)
//! (2, 2)` and the square `[3, 4] × [2, 3]`, so each has a corner on the line
//! up through `(3, 3)` and the answer's volume is `36 − 0.5 − 0.5 − 1 = 34`.
//! Round that line six walls meet, at 0°, 45°, 90°, 180°, 225° and 270°, and
//! the plate fills the wedges from 0° to 45°, 90° to 180° and 225° to 270°.
//! The walls opposite each other lie on one plane, `y = 3`, `x = 3` or
//! `y = x`, and face one way.
//!
//! **The fuse used to join each two opposite walls into one face** (queue row
//! Q2918, `docs/decisions.md` §470.4). It paired the six darts along the line
//! by plane, and the joining then took the line out of each pair, as it does
//! between two pieces of one flat region. So the walls on `y = 3`, `x = 3` and
//! `y = x` each came back as one face running straight through the line, three
//! faces crossing each other where no edge runs, 13 faces where there are 16.
//! The line is now three edges, each between the two walls either side of one
//! wedge of plate.

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, DraftVertex, DraftWire, Error, FaceKey, FaceView, Invalid,
    Model, ProfileBuilder, Rule, SolidBuilder, SolidKey, SolidView,
};

#[test]
fn a_plate_less_three_holes_meeting_at_one_corner_keeps_the_line_as_three_edges() {
    let mut model = a_model();
    let plate = a_plate(&mut model);
    let holes = three_holes(&mut model);
    let answer = one_solid(&mut model, plate, &holes, Rule::Difference);
    is_the_plate_less_the_holes(&model, answer, 34.0, 6);
}

#[test]
fn a_plate_less_three_holes_one_at_a_time_in_any_order_keeps_the_line_as_three_edges() {
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut model = a_model();
        let mut plate = a_plate(&mut model);
        let holes = three_holes(&mut model);
        for hole in order {
            plate = one_solid(&mut model, plate, &[holes[hole]], Rule::Difference);
        }
        is_the_plate_less_the_holes(&model, plate, 34.0, 6);
    }
}

/// The answer's line is an edge of the solid it is fused with next, six faces
/// round one edge of an input, and a box far from it leaves it as it was.
///
/// The box notches the plate's far corner from `z = 0.5` up, so the plate has
/// nine faces: its six, the notch's floor and the notch's two walls. The
/// plane of that floor cuts every wall of the plate, those along the line
/// included, and the joining takes the cuts back out. Where it kept the
/// corner the cuts leave on the line at `(3, 3, 0.5)`, at which six edges
/// meet, the line came back as six edges (Q2918).
#[test]
fn the_plate_less_three_holes_less_a_far_corner_keeps_the_line_as_three_edges() {
    let mut model = a_model();
    let plate = a_plate(&mut model);
    let holes = three_holes(&mut model);
    let perforated = one_solid(&mut model, plate, &holes, Rule::Difference);
    let corner = a_prism(
        &mut model,
        [[5.0, 5.0], [7.0, 5.0], [7.0, 7.0], [5.0, 7.0]],
        [0.0, 0.0, 0.5],
        1.0,
    );
    let answer = one_solid(&mut model, perforated, &[corner], Rule::Difference);
    is_the_plate_less_the_holes(&model, answer, 33.5, 9);
}

/// The answer the fuse used to write, described by hand: each two opposite
/// walls one face running straight through the line, and `Model::validate`
/// passed it (§470.4).
#[test]
fn three_walls_running_through_each_other_where_no_edge_is_are_refused() {
    let mut model = a_model();

    let built = the_plate_less_the_holes_drawn_by_hand(
        &mut model,
        &[
            &[[2.0, 3.0], [3.0, 3.0], [4.0, 3.0]],
            &[[3.0, 2.0], [3.0, 3.0], [3.0, 4.0]],
            &[[4.0, 4.0], [3.0, 3.0], [2.0, 2.0]],
        ],
    );

    let Err(Error::Invalid(Invalid::TwoFlatFacesRunThroughEachOtherWhereNeitherHasAnEdge { at })) =
        built
    else {
        panic!("expected the sweep to refuse the crossing walls by name, got {built:?}");
    };
    assert!(
        at.x() == 3.0 && at.y() == 3.0,
        "the walls cross on the line up through (3, 3), not at {at:?}"
    );
    assert_eq!(
        model.solids().count(),
        0,
        "a refusal leaves the model as it was"
    );
}

/// The same plate with the line written as three edges, each between the walls
/// either side of one wedge of plate, as the fuse writes it now. Every wall
/// meets the line in an edge, so nothing runs through anything.
#[test]
fn the_same_walls_split_at_the_line_with_an_edge_between_each_two_build() {
    let mut model = a_model();

    let built = the_plate_less_the_holes_drawn_by_hand(
        &mut model,
        &[
            &[[3.0, 3.0], [4.0, 3.0]],
            &[[4.0, 4.0], [3.0, 3.0]],
            &[[3.0, 3.0], [3.0, 4.0]],
            &[[2.0, 3.0], [3.0, 3.0]],
            &[[3.0, 3.0], [2.0, 2.0]],
            &[[3.0, 2.0], [3.0, 3.0]],
        ],
    )
    .unwrap_or_else(|why| panic!("the plate with the line as three edges is refused: {why}"));

    is_the_plate_less_the_holes(&model, built, 34.0, 6);
}

/// The plate less its three holes, faces listed by their corners, with the
/// walls along the line up through `(3, 3)` walked along `walls_on_the_line`.
///
/// Each wall is walked with the plate on its left, so it faces right. Two walls
/// walking one stretch of the line back to back, one after the other, share an
/// edge there.
fn the_plate_less_the_holes_drawn_by_hand(
    model: &mut Model,
    walls_on_the_line: &[&[[f64; 2]]],
) -> rcad_topo::Result<SolidKey> {
    let mut drawing = Drawing::default();
    let outline = [[0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0]];
    let holes_walked_against_the_outline: [&[[f64; 2]]; 3] = [
        &[[3.0, 3.0], [3.0, 4.0], [4.0, 4.0]],
        &[[3.0, 3.0], [2.0, 2.0], [2.0, 3.0]],
        &[[3.0, 3.0], [4.0, 3.0], [4.0, 2.0], [3.0, 2.0]],
    ];
    let mut top = vec![outline.to_vec()];
    top.extend(holes_walked_against_the_outline.map(<[[f64; 2]]>::to_vec));
    drawing.flat(&top, 1.0, Vector::new(0.0, 0.0, 1.0));
    let bottom: Vec<Vec<[f64; 2]>> = top
        .iter()
        .map(|walk| walk.iter().rev().copied().collect())
        .collect();
    drawing.flat(&bottom, 0.0, Vector::new(0.0, 0.0, -1.0));
    for side in 0..4 {
        drawing.wall(&[outline[side], outline[(side + 1) % 4]]);
    }
    for wall in [
        &[[3.0, 4.0], [4.0, 4.0]][..],
        &[[2.0, 2.0], [2.0, 3.0]],
        &[[4.0, 3.0], [4.0, 2.0]],
        &[[4.0, 2.0], [3.0, 2.0]],
    ]
    .into_iter()
    .chain(walls_on_the_line.iter().copied())
    {
        drawing.wall(wall);
    }
    drawing.build(model)
}

/// Flat faces written into a [`SolidBuilder`] by their corners.
///
/// Each corner is made once. An edge is made by the first face to walk it and
/// taken by the next one to walk it the other way, and then by nothing else,
/// so a stretch walked by four faces is two edges.
#[derive(Default)]
struct Drawing {
    builder: SolidBuilder,
    corners: Vec<(Point, DraftVertex)>,
    edges: Vec<([Point; 2], DraftEdge)>,
    faces: Vec<(DraftFace, Orientation)>,
}

impl Drawing {
    /// A face on the plane at `height` facing `out`, its first walk the
    /// outside and the rest holes.
    fn flat(&mut self, walks: &[Vec<[f64; 2]>], height: f64, out: Vector) {
        let walks: Vec<Vec<Point>> = walks
            .iter()
            .map(|walk| {
                walk.iter()
                    .map(|&[x, y]| Point::new(x, y, height))
                    .collect()
            })
            .collect();
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, height), out).expect("a direction");
        self.face(plane, &walks);
    }

    /// An upright wall from `z = 0` to `z = 1` along `path`, facing to the
    /// right of the way the path runs.
    fn wall(&mut self, path: &[[f64; 2]]) {
        let [x, y] = path[0];
        let [next_x, next_y] = path[1];
        let out = Vector::new(next_y - y, x - next_x, 0.0);
        let plane = Plane::from_point_and_normal(Point::new(x, y, 0.0), out).expect("a direction");
        let along = path.iter().map(|&[x, y]| Point::new(x, y, 0.0));
        let back = path.iter().rev().map(|&[x, y]| Point::new(x, y, 1.0));
        self.face(plane, &[along.chain(back).collect()]);
    }

    fn face(&mut self, plane: Plane, walks: &[Vec<Point>]) {
        let wires: Vec<DraftWire> = walks.iter().map(|walk| self.wire(walk)).collect();
        let face = self
            .builder
            .add_face(Surface::Plane(plane), &wires)
            .expect("wires of this builder");
        self.faces.push((face, Orientation::Forward));
    }

    fn wire(&mut self, walk: &[Point]) -> DraftWire {
        let steps: Vec<DraftStep> = (0..walk.len())
            .map(|at| {
                let [from, to] = [walk[at], walk[(at + 1) % walk.len()]];
                let (edge, orientation) = self.edge(from, to);
                DraftStep::Along(
                    self.builder
                        .add_coedge(edge, orientation)
                        .expect("an edge of this builder"),
                )
            })
            .collect();
        self.builder.add_wire(&steps).expect("a closed walk")
    }

    fn edge(&mut self, from: Point, to: Point) -> (DraftEdge, Orientation) {
        if let Some(at) = self.edges.iter().position(|(ends, _)| *ends == [to, from]) {
            let (_, edge) = self.edges.swap_remove(at);
            return (edge, Orientation::Reversed);
        }
        let ends = [self.corner(from), self.corner(to)];
        let edge = self
            .builder
            .add_edge(ends[0], ends[1], EdgeRun::Straight)
            .expect("two corners apart");
        self.edges.push(([from, to], edge));
        (edge, Orientation::Forward)
    }

    fn corner(&mut self, at: Point) -> DraftVertex {
        if let Some(&(_, corner)) = self.corners.iter().find(|(point, _)| *point == at) {
            return corner;
        }
        let corner = self.builder.add_vertex(at).expect("a finite point");
        self.corners.push((at, corner));
        corner
    }

    fn build(mut self, model: &mut Model) -> rcad_topo::Result<SolidKey> {
        self.builder.add_shell(&self.faces)?;
        self.builder
            .build(model, &Policy::certified(), &mut ProvenanceLog::new())
    }
}

/// Whether `solid` is the plate less its three holes, the line up through
/// `(3, 3)` three edges, each between the walls either side of one wedge of
/// plate, and no wall reaching across the line.
fn is_the_plate_less_the_holes(
    model: &Model,
    solid: SolidKey,
    volume: f64,
    faces_of_the_plate: usize,
) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a valid solid");
    let view = model.solid(solid).expect("this model made the key");
    let measured = view
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid this size measures");
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error() + 1e-12,
        "the answer holds {} where it should hold {volume}",
        measured.volume(),
    );
    for wall in view
        .faces()
        .filter(|&face| on_a_plane_through_the_line(face))
    {
        let ways: Vec<u32> = corners_of(wall).filter_map(way_off_the_line).collect();
        assert!(
            ways.windows(2).all(|pair| pair[0] == pair[1]),
            "a wall reaches across the line up through (3, 3), off it at {ways:?} degrees: {:?}",
            corners_of(wall).collect::<Vec<_>>(),
        );
    }
    let mut on_the_line: Vec<[u32; 2]> = edges_on_the_line(view)
        .map(|faces| {
            let mut ways = faces.map(|face| {
                let wall = view
                    .faces()
                    .find(|wall| wall.key() == face)
                    .expect("an edge's faces are the solid's");
                corners_of(wall)
                    .find_map(way_off_the_line)
                    .expect("a wall along the line reaches off it")
            });
            ways.sort_unstable();
            ways
        })
        .collect();
    on_the_line.sort_unstable();
    assert_eq!(
        on_the_line,
        [[0, 45], [90, 180], [225, 270]],
        "the line up through (3, 3) should be three edges, each between the walls \
         either side of one wedge of plate, by the way each wall leaves it in degrees"
    );
    assert_eq!(
        view.faces().count(),
        faces_of_the_plate + 10,
        "{faces_of_the_plate} faces of the plate and ten walls"
    );
}

/// The two faces along each edge from `(3, 3, 0)` to `(3, 3, 1)`, each edge
/// once.
fn edges_on_the_line(view: SolidView<'_>) -> impl Iterator<Item = [FaceKey; 2]> + '_ {
    let mut seen = Vec::new();
    view.faces()
        .flat_map(|face| face.wires().flat_map(|wire| wire.coedges()))
        .map(|coedge| coedge.edge())
        .filter(|edge| {
            [edge.start().point(), edge.end().point()]
                .iter()
                .all(|end| end.x() == 3.0 && end.y() == 3.0)
        })
        .filter(move |edge| {
            let new = !seen.contains(&edge.key());
            seen.push(edge.key());
            new
        })
        .map(|edge| {
            let faces: Vec<FaceKey> = edge.neighbouring_faces().collect();
            <[FaceKey; 2]>::try_from(faces).expect("an edge of a valid solid has two faces")
        })
}

/// Whether `face` is an upright wall on a plane through the line up through
/// `(3, 3)`: seen from above, every corner of it lies on one line through
/// that point.
fn on_a_plane_through_the_line(face: FaceView<'_>) -> bool {
    let off: Vec<[f64; 2]> = corners_of(face)
        .map(|corner| [corner.x() - 3.0, corner.y() - 3.0])
        .collect();
    face.outward_normal().z() == 0.0
        && off.iter().all(|one| {
            off.iter()
                .all(|other| one[0] * other[1] == one[1] * other[0])
        })
}

fn corners_of(face: FaceView<'_>) -> impl Iterator<Item = Point> + '_ {
    face.wires().flat_map(|wire| wire.points())
}

/// Which way, in whole degrees, `point` lies from the line up through
/// `(3, 3)`; `None` on the line. Every corner here is a whole number or a
/// half, so the angle is one of the eight multiples of 45° exactly.
fn way_off_the_line(point: Point) -> Option<u32> {
    let (across, up) = (point.x() - 3.0, point.y() - 3.0);
    if across == 0.0 && up == 0.0 {
        return None;
    }
    let degrees = up.atan2(across).to_degrees().rem_euclid(360.0).round();
    Some(degrees as u32)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_plate(model: &mut Model) -> SolidKey {
    a_prism(
        model,
        [[0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0]],
        [0.0, 0.0, 0.0],
        1.0,
    )
}

fn three_holes(model: &mut Model) -> [SolidKey; 3] {
    [
        a_prism(
            model,
            [[3.0, 3.0], [4.0, 4.0], [3.0, 4.0]],
            [0.0, 0.0, -1.0],
            3.0,
        ),
        a_prism(
            model,
            [[3.0, 3.0], [2.0, 3.0], [2.0, 2.0]],
            [0.0, 0.0, -1.0],
            3.0,
        ),
        a_prism(
            model,
            [[3.0, 2.0], [4.0, 2.0], [4.0, 3.0], [3.0, 3.0]],
            [0.0, 0.0, -1.0],
            3.0,
        ),
    ]
}

/// `one` fused with `others` under `rule`, where that is one solid.
fn one_solid(model: &mut Model, one: SolidKey, others: &[SolidKey], rule: Rule) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            others,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
    let &[solid] = fused.solids() else {
        panic!("{rule:?} answers {:?}, not one solid", fused.solids());
    };
    solid
}

/// An outline stood `length` long along `z`, moved by `at`.
fn a_prism<const N: usize>(
    model: &mut Model,
    outline: [[f64; 2]; N],
    [x, y, z]: [f64; 3],
    length: f64,
) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(
            &outline,
            length,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a positive length")
}
