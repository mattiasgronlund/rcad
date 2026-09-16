//! Of the 44 corpus files whose only probe reason is "this crossing runs
//! where its two surfaces are too nearly parallel to follow it"
//! (`docs/decisions.md` §107.1), a census already found *a* tangent face pair
//! in every one of them and called all 44 LIMIT. **That census never checked
//! the tangent pair it found is the pair that actually refused** — a file's
//! `reasons` set is one deduplicated string however many pairs contributed
//! it, so a file could hold a genuinely tangent pair *and* a transverse pair
//! the marcher gave up on early, and the census would look identical either
//! way.
//!
//! This runs [`rcad_step::Import::read`] instead of [`rcad_step::probe`], so
//! it sees the actual [`Refusal`] the reader stopped at rather than a bag of
//! reason strings. `Refusal::ThisEntityOfTheSolidWasRefused` carries the `#id`
//! of the one entity the builder was working on when it gave up — read that,
//! find which two faces it sits between, and check tangency on *that* pair.
//!
//! Run with `RCAD_STEP_CORPUS` pointing at a directory holding `nist/` and
//! `kicad/`:
//!
//! ```text
//! RCAD_STEP_CORPUS=<path> mise exec -- cargo run -p rcad-step --release \
//!   --example near_parallel_which_pair
//! ```
//!
//! # What that run printed, on 2026-09-16
//!
//! ```text
//! === summary over the 44 files ===
//!   ThisEntityOfTheSolidWasRefused (an #id to read a pair from): 0
//!   TheSolidTheFileDescribesIsNotOne (no entity, whole-solid form):     44
//!   anything else (succeeded, or a different refusal):                   0
//! ```
//!
//! All 44 of them, and every single one the same line:
//!
//! ```text
//!   TheSolidTheFileDescribesIsNotOne: solid #15, why: geometry: this crossing
//!   runs where its two surfaces are too nearly parallel to follow it
//! ```
//!
//! So **this plan does not work, and the reason is a design one and not a
//! bug.** `Refusal::ThisEntityOfTheSolidWasRefused` names an entity only when
//! `SolidBuilder::which_entity_the_builder_meant` (`brep.rs:902`) recognizes
//! the `rcad_topo::Error` it was handed, and that function names six
//! `ADraft…` variants and answers `None` to everything else. The near-parallel
//! crossing is none of the six: it is
//! `rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent`, a unit
//! variant carrying no fields, and it arrives wrapped as
//! `rcad_topo::Error::Geometry` — which is what the `geometry:` in every line
//! above is. Inside `SolidBuilder::build` the one step that can produce an
//! `Error::Geometry` at all is `scratch.validate`, so the refusal is the
//! sweep's and the sweep does not say which pair of surfaces it was looking
//! at. `what_stopped_the_build` (`brep.rs:727`) therefore takes the `None`
//! branch and publishes the whole-solid form, naming the `MANIFOLD_SOLID_BREP`
//! — `#15` in all 44, which are all KiCad files from one writer — and nothing
//! narrower.
//!
//! **The census's hole is therefore still open, and this example cannot close
//! it.** `pair_for_entity` and `evaluate_surfaces` below are never reached: 0
//! files took the `ThisEntityOfTheSolidWasRefused` arm, so the tangency test
//! was applied to no pair at all. They are kept, rather than deleted, because
//! they are the half that runs the moment an `#id` does come back. Closing the
//! hole needs the refusal to carry the two surfaces the marcher gave up
//! between — a kernel change in `rcad-geom`/`rcad-topo`, not a reading of the
//! STEP text — and until then "the tangent pair the census found is the pair
//! that refused" is unchecked, so all 44 LIMIT verdicts rest on it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{EntityId, Import, Refusal, StepFile, Value, probe};

/// `ProbeReport::reasons`' own text for this refusal, confirmed against
/// `near_parallel_census.rs`'s own constant of the same name.
const TARGET_REASON: &str = "the builder or the sweep: geometry: this crossing runs where its two surfaces are too \
     nearly parallel to follow it";

/// Relative slack allowed when comparing an axis-to-plane or axis-to-axis
/// distance against the radius sum or difference a full-line tangency needs.
const RADIUS_RELATIVE_TOL: f64 = 1.0e-9;

/// Degrees short of the exact tangent configuration still counted as a
/// confirmed match, same threshold `near_parallel_census.rs` uses.
const DEGENERATE_DEG: f64 = 2.0;

fn main() {
    let root = corpus();
    let mut qualifying: Vec<(&'static str, String, StepFile)> = Vec::new();
    for family in ["nist", "kicad"] {
        let mut files = files_under(&root.join(family));
        files.sort();
        for path in files {
            if let Some(file) = qualifies(&path) {
                qualifying.push((family, name_of(&path), file));
            }
        }
    }

    println!("=== files whose only probe reason is the near-parallel crossing ===");
    println!("  found {}", qualifying.len());

    let mut named_entity = 0usize;
    let mut whole_solid = 0usize;
    let mut other = 0usize;
    let mut confirmed_tangent = 0usize;
    let mut transverse_gap = 0usize;

    for (family, name, file) in &qualifying {
        let mut log = ProvenanceLog::new();
        let outcome = Import::read(file, &Policy::certified(), &mut log);
        println!("\n  {family}/{name}");
        match outcome {
            Ok(_) => {
                println!("    Import::read succeeded — this file no longer refuses at all");
                other += 1;
            }
            Err(Refusal::ThisEntityOfTheSolidWasRefused { solid, entity, .. }) => {
                named_entity += 1;
                println!(
                    "    ThisEntityOfTheSolidWasRefused: solid {solid}, entity {entity} ({})",
                    keyword_of(file, entity)
                );
                match pair_for_entity(file, entity) {
                    Some((a, b, kind_a, kind_b)) => {
                        let evidence = evaluate_pair(file, a, b);
                        println!("    refusing pair: {a} ({kind_a}) / {b} ({kind_b})");
                        println!("    {}", evidence.detail);
                        if evidence.confirmed_limit {
                            confirmed_tangent += 1;
                            println!("    verdict: TANGENT — a genuine limit");
                        } else {
                            transverse_gap += 1;
                            println!("    verdict: TRANSVERSE — a gap, not a limit");
                        }
                    }
                    None => println!(
                        "    could not read a surface pair for entity {entity} from the file's \
                         own text"
                    ),
                }
            }
            Err(Refusal::TheSolidTheFileDescribesIsNotOne { solid, why }) => {
                whole_solid += 1;
                println!("    TheSolidTheFileDescribesIsNotOne: solid {solid}, why: {why}");
            }
            Err(other_refusal) => {
                other += 1;
                println!("    a different refusal entirely: {other_refusal}");
            }
        }
    }

    println!("\n=== summary over the {} files ===", qualifying.len());
    println!("  ThisEntityOfTheSolidWasRefused (an #id to read a pair from): {named_entity}");
    println!("  TheSolidTheFileDescribesIsNotOne (no entity, whole-solid form):     {whole_solid}");
    println!("  anything else (succeeded, or a different refusal):                   {other}");
    if named_entity > 0 {
        println!(
            "  of the {named_entity} with a named entity: {confirmed_tangent} confirmed \
             tangent, {transverse_gap} transverse (a GAP)"
        );
    }
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!("RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/.")
    });
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "RCAD_STEP_CORPUS points at {}, which is not a directory",
        root.display()
    );
    root
}

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_under(&path));
            continue;
        }
        let is_step = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("stp") || extension.eq_ignore_ascii_case("step")
            });
        if is_step {
            found.push(path);
        }
    }
    found
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// The file, parsed, if `probe` finds it refuses for exactly the one reason
/// this census is about and nothing else.
fn qualifies(path: &Path) -> Option<StepFile> {
    let bytes = fs::read(path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
    let file = StepFile::read(&bytes)
        .unwrap_or_else(|why| panic!("{}: every file in the corpus parses: {why}", path.display()));
    let mut log = ProvenanceLog::new();
    let found = probe(&file, &Policy::certified(), &mut log);
    let only_this =
        found.fatal.is_empty() && found.reasons.len() == 1 && found.reasons.contains(TARGET_REASON);
    only_this.then_some(file)
}

fn keyword_of(file: &StepFile, id: EntityId) -> &str {
    file.instance(id)
        .and_then(|instance| instance.parts().first())
        .map_or("<unknown>", |record| record.keyword())
}

/// The two faces `entity` sits between, read from the file's own topology —
/// works whether `entity` names an `ADVANCED_FACE` directly, or an edge/loop
/// that sits on exactly two of them.
fn pair_for_entity(
    file: &StepFile,
    entity: EntityId,
) -> Option<(EntityId, EntityId, String, String)> {
    let faces = faces_of(file);
    if faces.contains_key(&entity) {
        return None;
    }
    let owners = edge_curve_owners(file, &faces);
    if let Some(two) = owners.get(&entity)
        && two.len() == 2
    {
        let mut both = two.iter().copied();
        let a = both.next()?;
        let b = both.next()?;
        return Some((
            a,
            b,
            kind_name(&faces[&a]).to_owned(),
            kind_name(&faces[&b]).to_owned(),
        ));
    }
    None
}

fn evaluate_pair(file: &StepFile, a: EntityId, b: EntityId) -> PairEvidence {
    let faces = faces_of(file);
    let (Some(surface_a), Some(surface_b)) = (faces.get(&a), faces.get(&b)) else {
        return PairEvidence::uncomputable("a face's surface did not parse");
    };
    evaluate_surfaces(surface_a, surface_b)
}

fn faces_of(file: &StepFile) -> BTreeMap<EntityId, SurfaceGeom> {
    let mut faces = BTreeMap::new();
    for instance in file.instances() {
        let Some(record) = instance
            .part("ADVANCED_FACE")
            .or_else(|| instance.part("FACE_SURFACE"))
        else {
            continue;
        };
        let Some(surface_id) = reference_at(record, 2) else {
            continue;
        };
        let Some(surface) = surface_of(file, surface_id) else {
            continue;
        };
        faces.insert(instance.id(), surface);
    }
    faces
}

/// Which two faces (there are never more than two for a manifold shell) name
/// each `EDGE_CURVE`.
fn edge_curve_owners(
    file: &StepFile,
    faces: &BTreeMap<EntityId, SurfaceGeom>,
) -> BTreeMap<EntityId, BTreeSet<EntityId>> {
    let mut owners: BTreeMap<EntityId, BTreeSet<EntityId>> = BTreeMap::new();
    for &face_id in faces.keys() {
        let Some(instance) = file.instance(face_id) else {
            continue;
        };
        let Some(record) = instance
            .part("ADVANCED_FACE")
            .or_else(|| instance.part("FACE_SURFACE"))
        else {
            continue;
        };
        let Some(bounds) = record.attribute(1).and_then(Value::as_list) else {
            continue;
        };
        for bound_value in bounds {
            let Some(bound_id) = bound_value.as_reference() else {
                continue;
            };
            for edge_curve_id in edge_curves_of_bound(file, bound_id) {
                owners.entry(edge_curve_id).or_default().insert(face_id);
            }
        }
    }
    owners
}

fn edge_curves_of_bound(file: &StepFile, bound_id: EntityId) -> Vec<EntityId> {
    let mut found = Vec::new();
    let Some(instance) = file.instance(bound_id) else {
        return found;
    };
    let Some(record) = instance
        .part("FACE_BOUND")
        .or_else(|| instance.part("FACE_OUTER_BOUND"))
    else {
        return found;
    };
    let Some(loop_id) = reference_at(record, 1) else {
        return found;
    };
    let Some(loop_instance) = file.instance(loop_id) else {
        return found;
    };
    let Some(loop_record) = loop_instance.part("EDGE_LOOP") else {
        return found;
    };
    let Some(oriented_edges) = loop_record.attribute(1).and_then(Value::as_list) else {
        return found;
    };
    for oriented_edge_value in oriented_edges {
        let Some(oriented_edge_id) = oriented_edge_value.as_reference() else {
            continue;
        };
        let Some(oriented_edge_instance) = file.instance(oriented_edge_id) else {
            continue;
        };
        let Some(oriented_edge_record) = oriented_edge_instance.part("ORIENTED_EDGE") else {
            continue;
        };
        if let Some(edge_curve_id) = reference_at(oriented_edge_record, 3) {
            found.push(edge_curve_id);
        }
    }
    found
}

/// What one face's surface entity says, read straight from its own
/// attributes — never the kernel's `Surface` type.
enum SurfaceGeom {
    Plane {
        origin: [f64; 3],
        normal: [f64; 3],
    },
    Cylinder {
        origin: [f64; 3],
        axis: [f64; 3],
        radius: f64,
    },
    Other(String),
}

fn kind_name(surface: &SurfaceGeom) -> &str {
    match surface {
        SurfaceGeom::Plane { .. } => "PLANE",
        SurfaceGeom::Cylinder { .. } => "CYLINDRICAL_SURFACE",
        SurfaceGeom::Other(keyword) => keyword,
    }
}

fn surface_of(file: &StepFile, id: EntityId) -> Option<SurfaceGeom> {
    let instance = file.instance(id)?;
    let record = instance.parts().first()?;
    match record.keyword() {
        "PLANE" => {
            let position = reference_at(record, 1)?;
            let (origin, axis) = placement_of(file, position)?;
            Some(SurfaceGeom::Plane {
                origin,
                normal: axis,
            })
        }
        "CYLINDRICAL_SURFACE" => {
            let position = reference_at(record, 1)?;
            let (origin, axis) = placement_of(file, position)?;
            let radius = number_at(record, 2)?;
            Some(SurfaceGeom::Cylinder {
                origin,
                axis,
                radius,
            })
        }
        other => Some(SurfaceGeom::Other(other.to_owned())),
    }
}

fn placement_of(file: &StepFile, id: EntityId) -> Option<([f64; 3], [f64; 3])> {
    let instance = file.instance(id)?;
    let record = instance.part("AXIS2_PLACEMENT_3D")?;
    let point_id = reference_at(record, 1)?;
    let origin = three_numbers_of(file, point_id, "CARTESIAN_POINT")?;
    let axis = match record.attribute(2) {
        Some(Value::Reference(direction_id)) => three_numbers_of(file, *direction_id, "DIRECTION")?,
        _ => [0.0, 0.0, 1.0],
    };
    Some((origin, axis))
}

fn three_numbers_of(file: &StepFile, id: EntityId, keyword: &str) -> Option<[f64; 3]> {
    let instance = file.instance(id)?;
    let record = instance.part(keyword)?;
    let list = record.attribute(1)?.as_list()?;
    let [a, b, c] = list else { return None };
    Some([number_of(a)?, number_of(b)?, number_of(c)?])
}

fn reference_at(record: &rcad_step::Record, position: usize) -> Option<EntityId> {
    record.attribute(position)?.as_reference()
}

fn number_at(record: &rcad_step::Record, position: usize) -> Option<f64> {
    number_of(record.attribute(position)?)
}

fn number_of(value: &Value) -> Option<f64> {
    match value {
        Value::Real(number) => Some(*number),
        Value::Integer(number) => Some(*number as f64),
        Value::Text(_)
        | Value::Enumeration(_)
        | Value::Reference(_)
        | Value::List(_)
        | Value::Typed { .. }
        | Value::Nothing
        | Value::Derived
        | _ => None,
    }
}

struct PairEvidence {
    detail: String,
    confirmed_limit: bool,
}

impl PairEvidence {
    fn uncomputable(why: &str) -> Self {
        PairEvidence {
            detail: why.to_owned(),
            confirmed_limit: false,
        }
    }
}

fn evaluate_surfaces(a: &SurfaceGeom, b: &SurfaceGeom) -> PairEvidence {
    match (a, b) {
        (SurfaceGeom::Plane { normal: n1, .. }, SurfaceGeom::Plane { normal: n2, .. }) => {
            let Some(angle) = angle_deg(*n1, *n2) else {
                return PairEvidence::uncomputable("a plane's normal did not normalize");
            };
            let folded = angle.min(180.0 - angle);
            PairEvidence {
                detail: format!("normals {folded:.6} deg from parallel"),
                confirmed_limit: folded < DEGENERATE_DEG,
            }
        }
        (
            SurfaceGeom::Plane { origin, normal },
            SurfaceGeom::Cylinder {
                origin: cylinder_origin,
                axis,
                radius,
            },
        )
        | (
            SurfaceGeom::Cylinder {
                origin: cylinder_origin,
                axis,
                radius,
            },
            SurfaceGeom::Plane { origin, normal },
        ) => {
            let (Some(angle), Some(unit_normal)) = (angle_deg(*axis, *normal), normalize(*normal))
            else {
                return PairEvidence::uncomputable("the axis or the normal did not normalize");
            };
            let deviation = (angle - 90.0).abs();
            let distance = dot(sub(*cylinder_origin, *origin), unit_normal).abs();
            let matches_radius = close(distance, *radius);
            PairEvidence {
                detail: format!(
                    "axis {deviation:.6} deg short of lying in the plane; axis-to-plane \
                     distance {distance:.12} vs radius {radius:.12}"
                ),
                confirmed_limit: deviation < DEGENERATE_DEG && matches_radius,
            }
        }
        (
            SurfaceGeom::Cylinder {
                origin: o1,
                axis: a1,
                radius: r1,
            },
            SurfaceGeom::Cylinder {
                origin: o2,
                axis: a2,
                radius: r2,
            },
        ) => {
            let (Some(angle), Some(unit_a1)) = (angle_deg(*a1, *a2), normalize(*a1)) else {
                return PairEvidence::uncomputable("an axis did not normalize");
            };
            let folded = angle.min(180.0 - angle);
            let distance = length(cross(unit_a1, sub(*o2, *o1)));
            let matches_external = close(distance, r1 + r2);
            let matches_internal = close(distance, (r1 - r2).abs());
            PairEvidence {
                detail: format!(
                    "axes {folded:.6} deg from parallel; axis-to-axis distance {distance:.12} \
                     vs r1+r2={:.12} / |r1-r2|={:.12}",
                    r1 + r2,
                    (r1 - r2).abs()
                ),
                confirmed_limit: folded < DEGENERATE_DEG && (matches_external || matches_internal),
            }
        }
        _ => PairEvidence::uncomputable(
            "not one of plane/plane, plane/cylinder or cylinder/cylinder",
        ),
    }
}

fn close(measured: f64, target: f64) -> bool {
    (measured - target).abs() <= RADIUS_RELATIVE_TOL * target.abs().max(1.0)
}

fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let len = length(v);
    (len.is_finite() && len > 0.0).then(|| [v[0] / len, v[1] / len, v[2] / len])
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}

fn angle_deg(a: [f64; 3], b: [f64; 3]) -> Option<f64> {
    let unit_a = normalize(a)?;
    let unit_b = normalize(b)?;
    let cosine = dot(unit_a, unit_b).clamp(-1.0, 1.0);
    Some(cosine.acos().to_degrees())
}
