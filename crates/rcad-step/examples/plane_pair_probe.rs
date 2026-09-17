//! Are the plane/plane edges the reader refuses actually two *different*
//! planes?
//!
//! An edge lies on both of its faces, so an edge between two distinct planes
//! lies in their intersection, which is a straight line — §24.3's rule that an
//! edge is named by the two surfaces either side of it, in its simplest case.
//! `run_along` nevertheless refuses such an edge when the file spelled its
//! curve as something the reader does not hold, and this counts how much that
//! costs.
//!
//! Two numbers, neither of which the pinned sweep can show because it stops
//! each file at its first refusal:
//!
//! - how many corpus files have **every** unheld-curve edge sitting open
//!   between two distinct `PLANE`s, so that one rule would answer the whole
//!   file;
//! - how many plane/plane pairs are the *same* plane both sides — coincident,
//!   naming no line at all, which a fix would have to keep refusing rather
//!   than let through silently.
//!
//! The answers, at the tree this landed on: **0 files** and **161 of 663
//! pairs**. The coincident ones are real and not a bug in the comparison —
//! `Nidec_Copal_SH-7010C.step` models its top as many small coplanar wedge
//! faces, so adjacent wedges genuinely share an edge with one plane on both
//! sides. Both numbers are why the plane/plane rule was measured and not
//! built: §45.2 again, an upper bound that delivered nothing.
//!
//! Shares lane A's census walk with `examples/spline_edge_census.rs`,
//! generalised here to carry each face's surface **entity id** rather than
//! just its keyword, so a `PLANE`/`PLANE` pair can be resolved to real
//! geometry and not just named.
//!
//! Being outside the crate, this cannot reach the private `Reading`/`Plane`
//! the reader itself resolves planes through, so it reads the geometry
//! straight off the file and the two must be kept in step: a `PLANE`
//! names an `AXIS2_PLACEMENT_3D`, whose first attribute is a `CARTESIAN_POINT`
//! (the origin) and whose second is a `DIRECTION` (the axis, defaulting to
//! `(0,0,1)` when absent, same as the reader). The axis is scaled to unit
//! length and the offset is `axis · origin`. Two planes are read as the same
//! one when their unit normals agree to a part in 10¹⁰ (either sign) and
//! their offsets agree to the same tolerance with the matching sign — loose
//! enough to catch a file that names one plane twice, and, going by the
//! report below, never once mistaken for two different walls of a real part.
//!
//! `RCAD_STEP_CORPUS=<path> mise exec -- cargo run -p rcad-step --release \
//!  --example plane_pair_probe`

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_step::{EntityId, StepFile, Value};

const HELD_CURVES: [&str; 3] = ["LINE", "CIRCLE", "ELLIPSE"];
const SAME_PLANE_TOLERANCE: f64 = 1e-10;

fn main() {
    let root = corpus_root();
    let mut totals = Totals::default();
    for family in ["nist", "kicad"] {
        let mut files = files_under(&root.join(family));
        files.sort();
        for path in files {
            probe_one_file(family, &path, &mut totals);
        }
    }
    totals.print();
}

fn corpus_root() -> PathBuf {
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

/// What one unheld-curve edge came out as, once its two faces are looked at.
///
/// Exhaustive on purpose: three outcomes, and a fourth would be a different
/// question.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PlanePair {
    /// Two distinct faces, both `PLANE`, geometrically different planes —
    /// the case that names a straight line.
    Distinct,
    /// Two distinct faces, both `PLANE`, the same plane both sides. No line is
    /// named, so this one can never be let through.
    Coincident,
    /// Anything else: not exactly two distinct faces, a face whose surface
    /// keyword is not `PLANE`, or a `PLANE` whose placement would not
    /// resolve.
    NotTwoPlanes,
}

#[derive(Default)]
struct Totals {
    files_seen: usize,
    edges_by_classification: BTreeMap<&'static str, usize>,
    /// file -> (has at least one open unheld edge, every open unheld edge so
    /// far is a distinct plane pair)
    per_file: BTreeMap<String, (bool, bool)>,
    coincident_examples: Vec<String>,
}

fn probe_one_file(family: &str, path: &Path, totals: &mut Totals) {
    totals.files_seen += 1;
    let label = format!("{family}/{}", name_of(path));
    let bytes = fs::read(path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
    let Ok(file) = StepFile::read(&bytes) else {
        return;
    };

    let edge_faces = faces_of_every_edge(&file);
    let entry = totals
        .per_file
        .entry(label.clone())
        .or_insert((false, true));

    for instance in file.instances() {
        let Some(edge_curve) = instance.part("EDGE_CURVE") else {
            continue;
        };
        let id = instance.id();
        let Some(curve_keyword) = reference_at(edge_curve, 3).and_then(|to| keyword_of(&file, to))
        else {
            continue;
        };
        if HELD_CURVES.contains(&curve_keyword) {
            continue;
        }
        let ends = (reference_at(edge_curve, 1), reference_at(edge_curve, 2));
        let (Some(start), Some(end)) = ends else {
            continue;
        };
        if start == end {
            continue;
        }
        entry.0 = true;

        let classification = classify(&file, &edge_faces, id);
        *totals
            .edges_by_classification
            .entry(classification.name())
            .or_default() += 1;
        if classification != PlanePair::Distinct {
            entry.1 = false;
        }
        if classification == PlanePair::Coincident {
            totals.coincident_examples.push(format!("{label} {id}"));
        }
    }
}

impl PlanePair {
    fn name(self) -> &'static str {
        match self {
            PlanePair::Distinct => "distinct plane pair",
            PlanePair::Coincident => "coincident plane pair (same plane twice)",
            PlanePair::NotTwoPlanes => "not a plane/plane pair",
        }
    }
}

fn classify(
    file: &StepFile,
    edge_faces: &BTreeMap<EntityId, Vec<(EntityId, EntityId)>>,
    edge_curve: EntityId,
) -> PlanePair {
    let Some(occurrences) = edge_faces.get(&edge_curve) else {
        return PlanePair::NotTwoPlanes;
    };
    let mut by_face: BTreeMap<EntityId, EntityId> = BTreeMap::new();
    for (face, surface) in occurrences {
        by_face.entry(*face).or_insert(*surface);
    }
    let surfaces: Vec<EntityId> = by_face.into_values().collect();
    let [surface_a, surface_b] = surfaces.as_slice() else {
        return PlanePair::NotTwoPlanes;
    };
    let (Some("PLANE"), Some("PLANE")) =
        (keyword_of(file, *surface_a), keyword_of(file, *surface_b))
    else {
        return PlanePair::NotTwoPlanes;
    };
    let (Some(plane_a), Some(plane_b)) = (
        plane_geometry(file, *surface_a),
        plane_geometry(file, *surface_b),
    ) else {
        return PlanePair::NotTwoPlanes;
    };
    if same_plane(plane_a, plane_b) {
        PlanePair::Coincident
    } else {
        PlanePair::Distinct
    }
}

/// A plane's unit normal and offset: every point `p` on it has `normal · p =
/// offset`.
fn plane_geometry(file: &StepFile, plane_id: EntityId) -> Option<([f64; 3], f64)> {
    let plane = file.instance(plane_id)?.part("PLANE")?;
    let placement_id = reference_at(plane, 1)?;
    let placement = file.instance(placement_id)?.part("AXIS2_PLACEMENT_3D")?;
    let origin_id = reference_at(placement, 1)?;
    let origin = three_reals(file, origin_id, "CARTESIAN_POINT")?;
    let axis = match reference_at(placement, 2) {
        Some(direction_id) => three_reals(file, direction_id, "DIRECTION")?,
        None => [0.0, 0.0, 1.0],
    };
    let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if !length.is_finite() || length == 0.0 {
        return None;
    }
    let normal = [axis[0] / length, axis[1] / length, axis[2] / length];
    let offset = normal[0] * origin[0] + normal[1] * origin[1] + normal[2] * origin[2];
    if !offset.is_finite() {
        return None;
    }
    Some((normal, offset))
}

fn same_plane(
    (normal_a, offset_a): ([f64; 3], f64),
    (normal_b, offset_b): ([f64; 3], f64),
) -> bool {
    let same_direction = (0..3)
        .all(|axis| (normal_a[axis] - normal_b[axis]).abs() < SAME_PLANE_TOLERANCE)
        && (offset_a - offset_b).abs() < SAME_PLANE_TOLERANCE;
    let opposite_direction = (0..3)
        .all(|axis| (normal_a[axis] + normal_b[axis]).abs() < SAME_PLANE_TOLERANCE)
        && (offset_a + offset_b).abs() < SAME_PLANE_TOLERANCE;
    same_direction || opposite_direction
}

/// Every `EDGE_CURVE` id found while walking every `ADVANCED_FACE`, mapped to
/// the face id and that face's surface **entity id** for each occurrence —
/// lane A's `faces_of_every_edge` (`spline_edge_census.rs`), generalised to
/// carry the surface id rather than just its keyword.
fn faces_of_every_edge(file: &StepFile) -> BTreeMap<EntityId, Vec<(EntityId, EntityId)>> {
    let mut edge_faces: BTreeMap<EntityId, Vec<(EntityId, EntityId)>> = BTreeMap::new();
    for instance in file.instances() {
        let Some(face_record) = instance
            .part("ADVANCED_FACE")
            .or_else(|| instance.part("FACE_SURFACE"))
        else {
            continue;
        };
        let face_id = instance.id();
        let Some(surface_id) = reference_at(face_record, 2) else {
            continue;
        };
        let Some(bounds) = references_at(face_record, 1) else {
            continue;
        };
        for bound in bounds {
            let Some(bound_instance) = file.instance(bound) else {
                continue;
            };
            let Some(bound_record) = bound_instance
                .part("FACE_OUTER_BOUND")
                .or_else(|| bound_instance.part("FACE_BOUND"))
            else {
                continue;
            };
            let Some(loop_id) = reference_at(bound_record, 1) else {
                continue;
            };
            let Some(loop_instance) = file.instance(loop_id) else {
                continue;
            };
            let Some(loop_record) = loop_instance.part("EDGE_LOOP") else {
                continue;
            };
            let Some(oriented_edges) = references_at(loop_record, 1) else {
                continue;
            };
            for oriented_edge in oriented_edges {
                let Some(oriented_instance) = file.instance(oriented_edge) else {
                    continue;
                };
                let Some(oriented_record) = oriented_instance.part("ORIENTED_EDGE") else {
                    continue;
                };
                let Some(edge_curve) = reference_at(oriented_record, 3) else {
                    continue;
                };
                edge_faces
                    .entry(edge_curve)
                    .or_default()
                    .push((face_id, surface_id));
            }
        }
    }
    edge_faces
}

fn keyword_of(file: &StepFile, id: EntityId) -> Option<&'static str> {
    use std::collections::HashSet;
    use std::sync::Mutex;
    static SEEN: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);
    let keyword = file.instance(id)?.parts()[0].keyword();
    let mut guard = SEEN.lock().unwrap();
    let set = guard.get_or_insert_with(HashSet::new);
    if let Some(found) = set.get(keyword) {
        return Some(found);
    }
    let leaked: &'static str = Box::leak(keyword.to_owned().into_boxed_str());
    set.insert(leaked);
    Some(leaked)
}

fn three_reals(file: &StepFile, id: EntityId, keyword: &str) -> Option<[f64; 3]> {
    let record = file.instance(id)?.part(keyword)?;
    let Value::List(items) = record.attribute(1)? else {
        return None;
    };
    let [a, b, c] = items.as_slice() else {
        return None;
    };
    Some([a.as_real()?, b.as_real()?, c.as_real()?])
}

fn reference_at(record: &rcad_step::Record, position: usize) -> Option<EntityId> {
    match record.attribute(position)? {
        Value::Reference(to) => Some(*to),
        Value::Integer(_)
        | Value::Real(_)
        | Value::Text(_)
        | Value::Enumeration(_)
        | Value::List(_)
        | Value::Typed { .. }
        | Value::Nothing
        | Value::Derived
        | _ => None,
    }
}

fn references_at(record: &rcad_step::Record, position: usize) -> Option<Vec<EntityId>> {
    let Value::List(items) = record.attribute(position)? else {
        return None;
    };
    items
        .iter()
        .map(|value| match value {
            Value::Reference(to) => Some(*to),
            Value::Integer(_)
            | Value::Real(_)
            | Value::Text(_)
            | Value::Enumeration(_)
            | Value::List(_)
            | Value::Typed { .. }
            | Value::Nothing
            | Value::Derived
            | _ => None,
        })
        .collect()
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

impl Totals {
    fn print(&self) {
        println!("files seen: {}", self.files_seen);
        println!("\nopen unheld-curve edges by classification:");
        for (name, count) in &self.edges_by_classification {
            println!("  {count:>4}  {name}");
        }

        let qualifying: BTreeSet<&String> = self
            .per_file
            .iter()
            .filter(|(_, (has_any, all_distinct_plane_pair))| *has_any && *all_distinct_plane_pair)
            .map(|(name, _)| name)
            .collect();
        println!(
            "\nfiles where every open unheld-curve edge is a distinct plane/plane pair: {}",
            qualifying.len()
        );
        for name in &qualifying {
            println!("  {name}");
        }

        println!(
            "\ncoincident plane/plane pairs found (same plane both sides), {} of them:",
            self.coincident_examples.len()
        );
        for at in &self.coincident_examples {
            println!("  {at}");
        }
    }
}
