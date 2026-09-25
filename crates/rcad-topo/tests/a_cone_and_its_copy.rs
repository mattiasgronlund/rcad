//! A cone and a second copy of it, built from the same numbers, united and
//! shared: both answers are the cone.
//!
//! `docs/decisions.md` §222.3 found every one of these refused, on "a point in
//! a piece that every solid could place", in `fuse_laws_over_a_cone.rs`. These
//! are the two smallest shapes it refused: one frustum from §49.6's grid and the
//! pointed cone.

mod cones;

use rcad_geom::{Deflection, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use cones::{a_frustum, a_pointed_cone};

/// The frustum of §49.6's grid with slope one half, caps at two and four up.
///
/// Refused `Undecidable` on "a point in a piece that every solid could place":
/// the cap piece's candidate is on the copy's cap, and the copy's cap is read
/// by winding the polygon of its corners — one corner, which winds round
/// nothing — so no face of the copy holds it.
#[test]
#[ignore = "a cone's cap is wound by its one corner and holds nothing: Undecidable on a point in \
            a piece that every solid could place"]
fn a_frustum_united_with_its_copy_is_the_frustum() {
    let mut model = a_model();
    let one = a_frustum(&mut model, 0.5, 2.0, 4.0);
    let copy = a_frustum(&mut model, 0.5, 2.0, 4.0);
    is_the_same_solid(&model, one, &fused(&model, one, copy, Rule::Union));
}

#[test]
#[ignore = "a cone's cap is wound by its one corner and holds nothing: Undecidable on a point in \
            a piece that every solid could place"]
fn a_frustum_shared_with_its_copy_is_the_frustum() {
    let mut model = a_model();
    let one = a_frustum(&mut model, 0.5, 2.0, 4.0);
    let copy = a_frustum(&mut model, 0.5, 2.0, 4.0);
    is_the_same_solid(&model, one, &fused(&model, one, copy, Rule::Intersection));
}

/// A pointed cone has no seam, and was refused just the same.
#[test]
#[ignore = "a cone's cap is wound by its one corner and holds nothing: Undecidable on a point in \
            a piece that every solid could place"]
fn a_pointed_cone_united_with_its_copy_is_the_cone() {
    let mut model = a_model();
    let one = a_pointed_cone(&mut model, 0.5, 4.0);
    let copy = a_pointed_cone(&mut model, 0.5, 4.0);
    is_the_same_solid(&model, one, &fused(&model, one, copy, Rule::Union));
}

#[test]
#[ignore = "a cone's cap is wound by its one corner and holds nothing: Undecidable on a point in \
            a piece that every solid could place"]
fn a_pointed_cone_shared_with_its_copy_is_the_cone() {
    let mut model = a_model();
    let one = a_pointed_cone(&mut model, 0.5, 4.0);
    let copy = a_pointed_cone(&mut model, 0.5, 4.0);
    is_the_same_solid(&model, one, &fused(&model, one, copy, Rule::Intersection));
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// Fuse a clone of `model` certified, and hand back the answer's model and
/// solids. A certified answer that logged anything is a failure.
fn fused(model: &Model, one: SolidKey, copy: SolidKey, rule: Rule) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[copy], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{rule:?} of a cone and its copy refused: {why:?}"));
    assert!(
        log.is_empty(),
        "a certified {rule:?} logged {} records",
        log.len()
    );
    let solids = answer.solids().to_vec();
    (model, solids)
}

/// The answer is one solid with the cone's corners bit for bit, and a volume
/// within the sum of the two bounds `measure` published.
fn is_the_same_solid(model: &Model, cone: SolidKey, answer: &(Model, Vec<SolidKey>)) {
    let (answer_model, pieces) = answer;
    assert_eq!(pieces.len(), 1, "one cone in, one solid out");
    assert_eq!(corners_of(answer_model, pieces[0]), corners_of(model, cone));
    let (found, found_bound) = volume_of(answer_model, pieces[0]);
    let (want, want_bound) = volume_of(model, cone);
    assert!(
        (found - want).abs() <= found_bound + want_bound,
        "volume {found} against {want}, beyond the bound {}",
        found_bound + want_bound
    );
}

/// Every corner of every face, as bits, sorted and without repeats — a set,
/// because the fuse drops a seam it never cut and a seam corner is listed twice.
fn corners_of(model: &Model, solid: SolidKey) -> Vec<[u64; 3]> {
    let mut corners: Vec<[u64; 3]> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for at in wire.points() {
                corners.push(at.coordinates().map(f64::to_bits));
            }
        }
    }
    corners.sort_unstable();
    corners.dedup();
    corners
}

fn volume_of(model: &Model, solid: SolidKey) -> (f64, f64) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a cone measures");
    assert!(
        measured.volume_error().is_finite(),
        "an endless bound checks nothing"
    );
    (measured.volume(), measured.volume_error())
}
