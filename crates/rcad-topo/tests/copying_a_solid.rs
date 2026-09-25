//! A solid copied inside its own model: the same shape, and nothing shared.
//!
//! `rcad_testing` runs OpenCASCADE's DRAW test corpus against this kernel, and
//! DRAW's `copy` and `tcopy` are nearly always a copy that is then moved. Before
//! `Model::copy_solid` there was nothing to copy with, so the harness gave the
//! same key a second name and moving one moved both.
//!
//! **The oracle is the original itself.** Everything a walk down from a solid
//! reads — placements, surfaces, runs, corners and tags — is written out with
//! `{:?}`, which for an `f64` prints every bit and tells `-0.0` from `0.0`, so
//! two descriptions that match are the same numbers and not merely close ones.
//! A copy that has not been moved must read back as its original does, and
//! measure the same bit for bit. Once one of them is moved, the other must read
//! back as it did before, and the moved one as a clone of the model where the
//! original was moved instead.

mod cones;
mod rings;

use std::collections::HashSet;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey, WireStepView};

/// The radius of the ring the other tests bend a tube round.
const THE_RING: f64 = 4.0;

#[test]
fn a_copy_reads_back_the_same_as_its_original_down_to_the_last_bit() {
    for (what, build) in every_kind_of_solid() {
        let mut model = a_model();
        let original = build(&mut model);
        let which_model = model.which_model_this_is();

        let copy = model.copy_solid(original).expect("this model made the key");

        assert_ne!(copy, original, "{what}: the copy is a solid of its own");
        assert_eq!(
            described(&model, copy),
            described(&model, original),
            "{what}: the copy reads back differently from its original"
        );
        assert_eq!(
            measured(&model, copy),
            measured(&model, original),
            "{what}: the copy measures differently from its original"
        );
        assert!(
            keys_under(&model, copy).is_disjoint(&keys_under(&model, original)),
            "{what}: the copy shares an entity with its original"
        );
        assert_eq!(
            model.which_model_this_is(),
            which_model,
            "{what}: a copy is one more solid in the same model"
        );
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!("{what}: the model does not validate after a copy: {why}")
            });
    }
}

#[test]
fn moving_the_copy_leaves_the_original_where_it_was() {
    for (what, build) in every_kind_of_solid() {
        let mut model = a_model();
        let original = build(&mut model);
        let before = (described(&model, original), measured(&model, original));
        let mut reference = model.clone();
        reference
            .place_solid(original, &a_movement())
            .expect("a clone keeps its original's keys");

        let copy = model.copy_solid(original).expect("this model made the key");
        model
            .place_solid(copy, &a_movement())
            .expect("this model made the key");

        assert_eq!(
            (described(&model, original), measured(&model, original)),
            before,
            "{what}: moving the copy moved the original"
        );
        assert_eq!(
            (described(&model, copy), measured(&model, copy)),
            (
                described(&reference, original),
                measured(&reference, original)
            ),
            "{what}: the moved copy is not the original moved"
        );
    }
}

#[test]
fn moving_the_original_leaves_the_copy_where_it_was() {
    for (what, build) in every_kind_of_solid() {
        let mut model = a_model();
        let original = build(&mut model);
        let before = (described(&model, original), measured(&model, original));
        let mut reference = model.clone();
        reference
            .place_solid(original, &a_movement())
            .expect("a clone keeps its original's keys");

        let copy = model.copy_solid(original).expect("this model made the key");
        model
            .place_solid(original, &a_movement())
            .expect("this model made the key");

        assert_eq!(
            (described(&model, copy), measured(&model, copy)),
            before,
            "{what}: moving the original moved the copy"
        );
        assert_eq!(
            (described(&model, original), measured(&model, original)),
            (
                described(&reference, original),
                measured(&reference, original)
            ),
            "{what}: the original moved somewhere else once it had a copy"
        );
    }
}

/// A fuse cannot tell a copy from a second solid built from the same numbers:
/// the answers read back the same bit for bit, whichever rule is asked.
#[test]
fn a_solid_fused_with_its_copy_answers_as_with_one_built_again() {
    for (what, build) in every_kind_a_fuse_takes_with_its_copy() {
        let mut model = a_model();
        let original = build(&mut model);
        let built_again = build(&mut model);
        let copy = model.copy_solid(original).expect("this model made the key");

        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let (with_the_copy, from_the_copy) = fused(&model, original, copy, rule, what);
            let (with_one_built_again, from_one_built_again) =
                fused(&model, original, built_again, rule, what);
            let read_back = |model: &Model, solids: &[SolidKey]| {
                solids
                    .iter()
                    .map(|&solid| (described(model, solid), measured(model, solid)))
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                read_back(&with_the_copy, &from_the_copy),
                read_back(&with_one_built_again, &from_one_built_again),
                "{what}: {rule:?} with the copy answers differently from {rule:?} with one \
                 built again"
            );
        }
    }
}

/// A turned cube is not here, and the reason is the fuse's rather than the
/// copy's: the fuse writes the turned corners into its answer rounded, and the
/// answer measures `8.000000000000002` against the cube's `8` — the same with
/// the copy as with a second cube built and turned the same way, which the test
/// above holds bit for bit.
#[test]
fn a_solid_united_with_its_unmoved_copy_is_the_solid_again() {
    for (what, build) in every_kind_a_fuse_takes_with_its_copy()
        .into_iter()
        .filter(|&(what, _)| what != A_TURNED_CUBE)
    {
        let mut model = a_model();
        let original = build(&mut model);
        let copy = model.copy_solid(original).expect("this model made the key");

        let (united, pieces) = fused(&model, original, copy, Rule::Union, what);

        assert_eq!(pieces.len(), 1, "{what}: one solid in, one solid out");
        let (found, found_bound) = volume_of(&united, pieces[0]);
        let (want, want_bound) = volume_of(&model, original);
        assert!(
            (found - want).abs() <= found_bound + want_bound,
            "{what}: the union has volume {found} against the solid's {want}, beyond the bound {}",
            found_bound + want_bound
        );
        united
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: the union does not validate: {why}"));
    }
}

#[test]
fn a_solid_less_its_unmoved_copy_is_nothing() {
    for (what, build) in every_kind_a_fuse_takes_with_its_copy() {
        let mut model = a_model();
        let original = build(&mut model);
        let copy = model.copy_solid(original).expect("this model made the key");

        let (_, pieces) = fused(&model, original, copy, Rule::Difference, what);

        assert!(
            pieces.is_empty(),
            "{what}: a solid less its own copy left {} solids behind",
            pieces.len()
        );
    }
}

/// Every corner tagged `2e-9`, every edge `1e-9` and one face `1e-9`, which is
/// the most slop each may carry and keep a corner no better known than the
/// edges through it.
#[test]
fn a_copy_carries_every_tag_of_its_original_and_tagging_it_leaves_the_original_alone() {
    let mut model = a_model();
    let original = a_cube(&mut model, 2.0);
    let corner = Approximation::new(2e-9, Cause::ImportedUncertainty);
    let side = Approximation::new(1e-9, Cause::ImportedUncertainty);
    for vertex in model
        .vertices()
        .map(|vertex| vertex.key())
        .collect::<Vec<_>>()
    {
        model
            .set_vertex_approximation(vertex, corner)
            .expect("a corner may carry more slop than its edges");
    }
    for edge in model.edges().map(|edge| edge.key()).collect::<Vec<_>>() {
        model
            .set_edge_approximation(edge, side)
            .expect("an edge may carry as much slop as its corners");
    }
    let one_face = model.faces().next().expect("a cube has faces").key();
    model
        .set_face_approximation(one_face, side)
        .expect("a face may carry as much slop as its edges");
    let worst = model.worst_approximation();

    let copy = model.copy_solid(original).expect("this model made the key");

    assert_eq!(described(&model, copy), described(&model, original));
    assert_eq!(
        model.worst_approximation().to_bits(),
        worst.to_bits(),
        "a copy's tags are ones the model already held"
    );

    let before = described(&model, original);
    let a_corner_of_the_copy = model
        .solid(copy)
        .expect("this model made the key")
        .faces()
        .next()
        .expect("a cube has faces")
        .outer_wire()
        .coedges()
        .next()
        .expect("a wire has sides")
        .start()
        .key();
    model
        .set_vertex_approximation(
            a_corner_of_the_copy,
            Approximation::new(3e-9, Cause::ImportedUncertainty),
        )
        .expect("a corner may carry more slop than its edges");
    assert_eq!(
        described(&model, original),
        before,
        "tagging a corner of the copy tagged the original's"
    );
}

#[test]
fn a_key_from_another_model_copies_nothing() {
    let mut model = a_model();
    a_cube(&mut model, 2.0);
    let mut elsewhere = a_model();
    let stranger = a_cube(&mut elsewhere, 2.0);
    let counts = counts_of(&model);

    let refused = model.copy_solid(stranger);

    assert!(
        matches!(refused, Err(Error::KeyFromAnotherModel { entity: "solid" })),
        "expected a key from another model to be refused, got {refused:?}"
    );
    assert_eq!(counts_of(&model), counts, "a refused copy wrote something");
}

#[test]
fn a_copy_of_a_copy_is_the_original_again() {
    for (what, build) in every_kind_of_solid() {
        let mut model = a_model();
        let original = build(&mut model);

        let copy = model.copy_solid(original).expect("this model made the key");
        let copy_of_the_copy = model.copy_solid(copy).expect("this model made the key");

        assert_eq!(
            (
                described(&model, copy_of_the_copy),
                measured(&model, copy_of_the_copy)
            ),
            (described(&model, original), measured(&model, original)),
            "{what}: a copy of the copy is not the original"
        );
        assert!(
            keys_under(&model, copy_of_the_copy).is_disjoint(&keys_under(&model, copy)),
            "{what}: a copy of the copy shares an entity with the copy"
        );
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: the model does not validate: {why}"));
    }
}

/// The one shape here a certified fuse will not take with its copy, pinned so
/// that closing the gap moves a test rather than a sentence.
///
/// **The refusal is the fuse's and not the copy's**: a second bend built from
/// the same numbers is refused in the same words — "a point in a piece that
/// every solid could place", asked at `(3, 0, 0)`, where the tube's inside
/// meets the `+x` cap.
#[test]
fn a_quarter_bend_and_its_copy_are_refused_as_two_bends_built_alike_are() {
    let mut model = a_model();
    let (what, build) = A_QUARTER_BEND;
    let bend = build(&mut model);
    let built_again = build(&mut model);
    let copy = model.copy_solid(bend).expect("this model made the key");

    for rule in [Rule::Union, Rule::Difference] {
        let with_the_copy = refused(&model, bend, copy, rule, what);
        let with_one_built_again = refused(&model, bend, built_again, rule, what);
        assert!(
            matches!(with_the_copy, Error::Undecidable { .. }),
            "{rule:?}: expected the arithmetic to give up, got {with_the_copy}"
        );
        assert_eq!(
            format!("{with_the_copy:?}"),
            format!("{with_one_built_again:?}"),
            "{rule:?}: the copy is refused differently from a bend built again"
        );
    }
}

type Build = fn(&mut Model) -> SolidKey;

/// Every shape a copy has to get right.
fn every_kind_of_solid() -> impl Iterator<Item = (&'static str, Build)> {
    every_kind_a_fuse_takes_with_its_copy()
        .into_iter()
        .chain([A_QUARTER_BEND])
}

/// A torus face, which a fuse with its copy refuses (see
/// `a_quarter_bend_and_its_copy_are_refused_as_two_bends_built_alike_are`).
const A_QUARTER_BEND: (&str, Build) = ("a quarter bend", |model| {
    rings::a_quarter_bend(model, THE_RING, 1.0)
});

const A_TURNED_CUBE: &str = "a cube turned before it is copied";

/// Flat, each curved surface but the torus, a ball's poles, a cone's apex, the
/// answer of a fuse, a solid with a hollow inside it, and one that was moved
/// before it was copied.
fn every_kind_a_fuse_takes_with_its_copy() -> [(&'static str, Build); 8] {
    [
        ("a cube", |model| a_cube(model, 2.0)),
        ("a ball", |model| {
            model
                .add_ball(1.0, &Isometry::identity())
                .expect("a ball of radius one")
        }),
        ("a round bar", |model| {
            model
                .add_round_bar(1.0, 3.0, &Isometry::identity())
                .expect("a bar of radius one")
        }),
        ("a frustum", |model| cones::a_frustum(model, 0.5, 2.0, 4.0)),
        ("a pointed cone", |model| {
            cones::a_pointed_cone(model, 0.5, 4.0)
        }),
        (
            "two overlapping cubes made one",
            two_overlapping_cubes_made_one,
        ),
        ("a cube with a hollow in it", a_cube_with_a_hollow_in_it),
        (A_TURNED_CUBE, |model| {
            let cube = a_cube(model, 2.0);
            model
                .place_solid(cube, &a_movement())
                .expect("this model made the key");
            cube
        }),
    ]
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_cube(model: &mut Model, side: f64) -> SolidKey {
    model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube")
}

fn a_cube_at(model: &mut Model, side: f64, at: Vector) -> SolidKey {
    let cube = a_cube(model, side);
    model
        .place_solid(
            cube,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("this model made the key");
    cube
}

fn two_overlapping_cubes_made_one(model: &mut Model) -> SolidKey {
    let one = a_cube_at(model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = a_cube_at(model, 2.0, Vector::new(1.0, 1.0, 1.0));
    only_solid_of(
        model
            .fuse(
                &[one],
                &[other],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("two lattice cubes unite")
            .solids(),
    )
}

/// A cube of side four with a cube of side two taken out of its middle: one
/// solid, two shells.
fn a_cube_with_a_hollow_in_it(model: &mut Model) -> SolidKey {
    let block = a_cube_at(model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let bubble = a_cube_at(model, 2.0, Vector::new(1.0, 1.0, 1.0));
    let hollow = only_solid_of(
        model
            .fuse(
                &[block],
                &[bubble],
                Rule::Difference,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a lattice cube less a smaller one inside it")
            .solids(),
    );
    assert_eq!(
        model
            .solid(hollow)
            .expect("this model made the key")
            .shells()
            .count(),
        2,
        "the outside of the block and the skin of the hollow"
    );
    hollow
}

fn only_solid_of(solids: &[SolidKey]) -> SolidKey {
    let [only] = solids else {
        panic!("expected one solid, got {}", solids.len());
    };
    *only
}

fn a_movement() -> Isometry {
    let axis = Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
        .expect("a direction with length");
    Isometry::rotation(&axis, 0.7).expect("a rotation")
}

/// Everything a walk down from the solid reads, in the order it reads it.
fn described(model: &Model, solid: SolidKey) -> Vec<String> {
    let solid = model.solid(solid).expect("this model made the key");
    let mut said = vec![format!(
        "solid at {:?} {:?}",
        solid.location(),
        solid.orientation()
    )];
    for shell in solid.shells() {
        said.push(format!(
            "shell at {:?} {:?}, {} faces",
            shell.location(),
            shell.orientation(),
            shell.face_count()
        ));
        for face in shell.faces() {
            said.push(format!(
                "face on {:?} at {:?} {:?}, slop {:?}",
                face.surface(),
                face.location(),
                face.orientation(),
                face.approximation()
            ));
            for wire in face.wires() {
                said.push(format!(
                    "wire at {:?} {:?}, {} steps",
                    wire.location(),
                    wire.orientation(),
                    wire.step_count()
                ));
                for step in wire.steps() {
                    said.push(match step {
                        WireStepView::Along(coedge) => {
                            let edge = coedge.edge();
                            format!(
                                "along {:?} of an edge {:?} running {:?}, slop {:?}, \
                                 from {:?} slop {:?} to {:?} slop {:?}",
                                coedge.orientation_on_its_edge(),
                                edge.orientation(),
                                edge.run(),
                                edge.approximation(),
                                coedge.start().point(),
                                coedge.start().approximation(),
                                coedge.end().point(),
                                coedge.end().approximation()
                            )
                        }
                        WireStepView::Pole(vertex) => format!(
                            "pole at {:?} slop {:?}",
                            vertex.point(),
                            vertex.approximation()
                        ),
                        _ => panic!("a kind of step this test does not know"),
                    });
                }
            }
        }
    }
    said
}

/// The measure at a hundredth, or the refusal, as text.
fn measured(model: &Model, solid: SolidKey) -> String {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
    format!("{measured:?}")
}

/// The volume at a hundredth, and the bound on how far it is out.
fn volume_of(model: &Model, solid: SolidKey) -> (f64, f64) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the solid measures");
    assert!(
        measured.volume_error().is_finite(),
        "an endless bound checks nothing"
    );
    (measured.volume(), measured.volume_error())
}

/// Fuse on a clone, certified, and hand back the clone and the answer's solids.
/// A certified answer that logged anything is a failure.
fn fused(
    model: &Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    what: &str,
) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{what}: {rule:?} with its copy refused: {why}"));
    assert!(
        log.is_empty(),
        "{what}: a certified {rule:?} with its copy logged {} records",
        log.len()
    );
    let solids = answer.solids().to_vec();
    (model, solids)
}

/// Fuse on a clone, certified, and hand back why it was refused.
fn refused(model: &Model, one: SolidKey, other: SolidKey, rule: Rule, what: &str) -> Error {
    let fused = model.clone().fuse(
        &[one],
        &[other],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    match fused {
        Ok(answer) => panic!(
            "{what}: {rule:?} answered {} solids where it was expected to refuse",
            answer.solids().len()
        ),
        Err(why) => why,
    }
}

/// Every key a walk down from the solid reaches, of every kind.
fn keys_under(model: &Model, solid: SolidKey) -> HashSet<String> {
    let solid = model.solid(solid).expect("this model made the key");
    let mut keys = HashSet::new();
    for shell in solid.shells() {
        keys.insert(format!("{:?}", shell.key()));
        for face in shell.faces() {
            keys.insert(format!("{:?}", face.key()));
            for wire in face.wires() {
                keys.insert(format!("{:?}", wire.key()));
                for step in wire.steps() {
                    match step {
                        WireStepView::Along(coedge) => {
                            keys.insert(format!("{:?}", coedge.key()));
                            keys.insert(format!("{:?}", coedge.edge().key()));
                            keys.insert(format!("{:?}", coedge.start().key()));
                            keys.insert(format!("{:?}", coedge.end().key()));
                        }
                        WireStepView::Pole(vertex) => {
                            keys.insert(format!("{:?}", vertex.key()));
                        }
                        _ => panic!("a kind of step this test does not know"),
                    }
                }
            }
        }
    }
    keys
}

fn counts_of(model: &Model) -> [usize; 7] {
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
