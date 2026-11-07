//! Slivers a rounding wide round a line where two blocks built as one solid
//! touch, placed by the edge rules of `fuse/select.rs` (Q2921).

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

/// Two blocks built as one solid, `[0, 3]²` from `z = −1` to `2` and `[3, 6]²`
/// from `z = 0` to `1`, touch along `x = y = 3`, so four faces stand round
/// that line. A right-angled triangle from `z = 0.5` to `1.5`, with legs `dx`
/// and `dy` and its long side from lower left to upper right through
/// `(3, 3)`, has a leaning wall a rounding off that line, and every face round
/// it cuts a sliver no point can be found in. A quarter of the triangle is in
/// each block, so the blocks and the triangle share `3·dx·dy/16`.
///
/// Where the slivers' sides start inside an edge up that line, or run along
/// one, the edge rules read them, and an answer read past the edge stands only
/// where no other face of the blocks reaches the line there. The first block's
/// face `x = 3` holds the whole line along the second block's top edge on
/// `x = 3`, and ends a rounding short of where the wall crosses that line, so
/// only its corners can show that it keeps off.
///
/// `S − T` is left out: it is refused as not manifold.
#[test]
fn a_triangle_whose_long_side_passes_where_two_blocks_touch_fuses_to_its_volume() {
    let mut wrong = Vec::new();
    for (dx, dy) in [
        (2.0, 1.0),
        (3.0, 1.0),
        (3.0, 2.0),
        (1.0, 2.0),
        (1.0, 3.0),
        (2.0, 3.0),
        (5.0, 2.0),
        (5.0, 3.0),
        (4.0, 3.0),
        (3.0, 4.0),
    ] {
        let lower_left = [3.0 - dx / 2.0, 3.0 - dy / 2.0];
        let upper_right = [3.0 + dx / 2.0, 3.0 + dy / 2.0];
        for (corner, outline) in [
            (
                "lower right",
                [lower_left, [upper_right[0], lower_left[1]], upper_right],
            ),
            (
                "upper left",
                [lower_left, upper_right, [lower_left[0], upper_right[1]]],
            ),
        ] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let blocks = two_blocks_touching_along_a_line(&mut model);
            let triangle = a_prism(&mut model, &outline, [0.0, 0.0, 0.5]);
            let blocks_volume = 27.0 + 9.0;
            let triangle_volume = dx * dy / 2.0;
            let shared = 3.0 * dx * dy / 16.0;
            for (one, other, rule, volume) in [
                (
                    blocks,
                    triangle,
                    Rule::Union,
                    blocks_volume + triangle_volume - shared,
                ),
                (blocks, triangle, Rule::Intersection, shared),
                (
                    triangle,
                    blocks,
                    Rule::Union,
                    blocks_volume + triangle_volume - shared,
                ),
                (triangle, blocks, Rule::Intersection, shared),
                (triangle, blocks, Rule::Difference, triangle_volume - shared),
            ] {
                let mut copy = model.clone();
                let said = copy
                    .fuse(
                        &[one],
                        &[other],
                        rule,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .map(|fused| {
                        let solids = fused.solids().to_vec();
                        copy.validate(&Policy::certified(), &mut ProvenanceLog::new())
                            .expect("a fuse leaves a sound model");
                        volume_of(&copy, &solids)
                    });
                let first = if one == blocks { "blocks" } else { "triangle" };
                let gave = match said {
                    Ok((measured, error)) if (measured - volume).abs() <= error + 1e-12 => continue,
                    Ok((measured, _)) => format!("{measured}"),
                    Err(refusal) => name_of(&refusal),
                };
                wrong.push(format!(
                    "slope {dy}/{dx}, right angle at the {corner}, {rule:?} from the {first}: \
                     {gave}, not {volume}"
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The two blocks, each built corner by corner and face by face into one shell.
fn two_blocks_touching_along_a_line(model: &mut Model) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let mut faces = Vec::new();
    for ([x0, x1], [y0, y1], [z0, z1]) in [
        ([0.0, 3.0], [0.0, 3.0], [-1.0, 2.0]),
        ([3.0, 6.0], [3.0, 6.0], [0.0, 1.0]),
    ] {
        let mut corners = Vec::new();
        for z in [z0, z1] {
            for y in [y0, y1] {
                for x in [x0, x1] {
                    corners.push([x, y, z]);
                }
            }
        }
        let vertices: Vec<_> = corners
            .iter()
            .map(|&[x, y, z]| builder.add_vertex(Point::new(x, y, z)).expect("a corner"))
            .collect();
        let mut edges = std::collections::BTreeMap::new();
        for face in [
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
        ] {
            let mut steps = Vec::new();
            for (at, &from) in face.iter().enumerate() {
                let to = face[(at + 1) % face.len()];
                let ends = (from.min(to), from.max(to));
                let edge = *edges.entry(ends).or_insert_with(|| {
                    builder
                        .add_edge(vertices[ends.0], vertices[ends.1], EdgeRun::Straight)
                        .expect("an edge")
                });
                let way = if from < to {
                    Orientation::Forward
                } else {
                    Orientation::Reversed
                };
                steps.push(DraftStep::Along(
                    builder.add_coedge(edge, way).expect("a coedge"),
                ));
            }
            let wire = builder.add_wire(&steps).expect("a wire");
            let at = |index: usize| {
                let [x, y, z] = corners[face[index]];
                Point::new(x, y, z)
            };
            let plane = Plane::through_three_points(at(0), at(1), at(2)).expect("three corners");
            faces.push(
                builder
                    .add_face(Surface::Plane(plane), &[wire])
                    .expect("a face"),
            );
        }
    }
    let shell: Vec<_> = faces
        .into_iter()
        .map(|face| (face, Orientation::Forward))
        .collect();
    builder.add_shell(&shell).expect("the faces close");
    let blocks = builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("two blocks touching along a line");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("two blocks touching along a line are a sound solid");
    blocks
}

/// The prism one long on `outline`, its floor at `at`.
fn a_prism(model: &mut Model, outline: &[[f64; 2]], [x, y, z]: [f64; 3]) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(
            &outline,
            1.0,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a positive length")
}

/// The variant's name, which is what its `Debug` starts with.
fn name_of(refusal: &Error) -> String {
    let debug = format!("{refusal:?}");
    let end = debug
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(debug.len());
    debug[..end].to_owned()
}

/// The volume of `solids` together, and how far off it may be.
fn volume_of(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    solids.iter().fold((0.0, 0.0), |(volume, error), &solid| {
        let measured = model
            .solid(solid)
            .expect("made")
            .measure(
                Deflection::of(0.1).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("measures");
        (volume + measured.volume(), error + measured.volume_error())
    })
}
