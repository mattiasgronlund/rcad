//! Measures a question `run_along` (`src/brep.rs`) never asks: when a corpus
//! file puts a curve on an edge that this kernel does not hold — everything
//! but `LINE`, `CIRCLE`, `ELLIPSE` — what are the two faces either side of
//! that edge?
//!
//! This walks the entity graph itself, at `StepFile` level, so one file's
//! refusal never hides the rest of that file the way `Import::read` would.
//! `docs/decisions.md` §24.3: an edge is named by the two surfaces either
//! side of it, and never by a stored curve — so if those two surfaces are
//! both kinds this kernel already holds (`PLANE`, `CYLINDRICAL_SURFACE`,
//! `SPHERICAL_SURFACE`, `CONICAL_SURFACE`, `TOROIDAL_SURFACE`,
//! `src/geometry.rs:56-120`), the curve the file wrote may be redundant.
//!
//! Run with `RCAD_STEP_CORPUS` pointing at a directory holding `nist/` and
//! `kicad/`:
//! `RCAD_STEP_CORPUS=<path> mise exec -- cargo run -p rcad-step --release \
//!  --example spline_edge_census`

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_step::{EntityId, StepFile, Value};

const HELD_CURVES: [&str; 3] = ["LINE", "CIRCLE", "ELLIPSE"];
const HELD_SURFACES: [&str; 5] = [
    "PLANE",
    "CYLINDRICAL_SURFACE",
    "SPHERICAL_SURFACE",
    "CONICAL_SURFACE",
    "TOROIDAL_SURFACE",
];

fn main() {
    let root = corpus_root();
    let mut report = Report::default();
    for family in ["nist", "kicad"] {
        let mut files = files_under(&root.join(family));
        files.sort();
        for path in files {
            census_one_file(family, &path, &mut report);
        }
    }
    report.print();
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

/// What the forward walk from every `ADVANCED_FACE` found for one edge.
enum FacesEitherSide {
    /// The usual case: exactly two faces use this edge, with these two
    /// surface keywords, sorted so the pair reads the same whichever face
    /// the file listed first.
    Two([String; 2]),
    /// Some other number of distinct faces were found — 0, 1, or 3+ — or a
    /// face was found but its own surface could not be resolved. Named
    /// rather than folded into the pair count, so that nothing is silently
    /// dropped and the buckets add back up to the total.
    NotTwo {
        distinct_faces: usize,
        unresolved_surface: bool,
    },
}

/// One edge whose curve is a kind `run_along` does not hold.
struct UnheldEdge {
    curve_keyword: String,
    closed: bool,
    faces: FacesEitherSide,
}

/// One file's findings, rolled into the corpus-wide report at the end.
#[derive(Default)]
struct Report {
    files_seen: usize,
    files_failed_to_parse: Vec<String>,
    broken_curve_ref: Vec<String>,
    broken_vertices: Vec<String>,
    unheld_edges: Vec<(String, UnheldEdge)>, // (family/file, edge)
}

fn census_one_file(family: &str, path: &Path, report: &mut Report) {
    report.files_seen += 1;
    let name = name_of(path);
    let label = format!("{family}/{name}");
    let bytes = fs::read(path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
    let file = match StepFile::read(&bytes) {
        Ok(file) => file,
        Err(why) => {
            report.files_failed_to_parse.push(format!("{label}: {why}"));
            return;
        }
    };

    let edge_faces = faces_of_every_edge(&file);

    for instance in file.instances() {
        let Some(edge_curve) = instance.part("EDGE_CURVE") else {
            continue;
        };
        let id = instance.id();
        let curve_keyword = reference_at(edge_curve, 3).and_then(|to| instance_keyword(&file, to));
        let Some(curve_keyword) = curve_keyword else {
            report.broken_curve_ref.push(format!("{label} #{}", id.0));
            continue;
        };
        if HELD_CURVES.contains(&curve_keyword) {
            continue;
        }

        let ends = (reference_at(edge_curve, 1), reference_at(edge_curve, 2));
        let (Some(start), Some(end)) = ends else {
            report.broken_vertices.push(format!("{label} #{}", id.0));
            continue;
        };
        let closed = start == end;

        let occurrences: &[(EntityId, Option<&'static str>)] =
            edge_faces.get(&id).map_or(&[], Vec::as_slice);
        let mut by_face: BTreeMap<EntityId, Option<&'static str>> = BTreeMap::new();
        for (face, surface) in occurrences {
            by_face.entry(*face).or_insert(*surface);
        }
        let faces = match by_face.len() {
            2 if by_face.values().all(Option::is_some) => {
                let mut keywords: Vec<&str> = by_face.into_values().map(Option::unwrap).collect();
                keywords.sort_unstable();
                FacesEitherSide::Two([keywords[0].to_owned(), keywords[1].to_owned()])
            }
            distinct_faces => FacesEitherSide::NotTwo {
                distinct_faces,
                unresolved_surface: by_face.values().any(Option::is_none),
            },
        };

        report.unheld_edges.push((
            label.clone(),
            UnheldEdge {
                curve_keyword: curve_keyword.to_owned(),
                closed,
                faces,
            },
        ));
    }
}

/// Every `EDGE_CURVE` id found while walking every `ADVANCED_FACE`, mapped to
/// the face id and that face's surface keyword for each occurrence — one
/// entry per `ORIENTED_EDGE` that names it, so a seam edge a face walks twice
/// shows up twice under the same face id.
///
/// A broken link anywhere in one face's own chain — bounds, loop, an
/// `ORIENTED_EDGE`'s `edge_element`, the surface reference — drops that one
/// occurrence and nothing else; it is why an edge can come out of this with
/// fewer than the two occurrences a sound file would give it.
fn faces_of_every_edge(
    file: &StepFile,
) -> BTreeMap<EntityId, Vec<(EntityId, Option<&'static str>)>> {
    let mut edge_faces: BTreeMap<EntityId, Vec<(EntityId, Option<&'static str>)>> = BTreeMap::new();
    for instance in file.instances() {
        let Some(face_record) = instance
            .part("ADVANCED_FACE")
            .or_else(|| instance.part("FACE_SURFACE"))
        else {
            continue;
        };
        let face_id = instance.id();
        let surface_keyword =
            reference_at(face_record, 2).and_then(|to| instance_keyword(file, to));

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
                    .push((face_id, surface_keyword));
            }
        }
    }
    edge_faces
}

/// The keyword of an entity's first-written part, leaked to `'static` so the
/// census can hold it past the `StepFile` it came from — each file is read
/// and dropped in turn, but a keyword like `PLANE` is one of a small, fixed
/// vocabulary, so leaking each distinct spelling once is the whole cost.
fn instance_keyword(file: &StepFile, id: EntityId) -> Option<&'static str> {
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

fn is_bspline(curve_keyword: &str) -> bool {
    curve_keyword.starts_with("B_SPLINE_CURVE")
}

impl Report {
    fn print(&self) {
        println!("files seen: {}", self.files_seen);
        if !self.files_failed_to_parse.is_empty() {
            println!(
                "files that would not parse at StepFile level ({}):",
                self.files_failed_to_parse.len()
            );
            for line in &self.files_failed_to_parse {
                println!("  {line}");
            }
        }
        println!(
            "EDGE_CURVE instances with a broken curve reference (keyword unknown): {}",
            self.broken_curve_ref.len()
        );
        for at in &self.broken_curve_ref {
            println!("  {at}");
        }
        println!(
            "EDGE_CURVE instances with a broken vertex reference (closed unknown): {}",
            self.broken_vertices.len()
        );
        for at in &self.broken_vertices {
            println!("  {at}");
        }

        println!("\nunheld-curve edges found: {}", self.unheld_edges.len());

        let mut resolved_pair_histogram: BTreeMap<[String; 2], usize> = BTreeMap::new();
        let mut bspline_pair_histogram: BTreeMap<[String; 2], usize> = BTreeMap::new();
        let mut not_two: BTreeMap<(usize, bool), usize> = BTreeMap::new();
        let mut closed_count = 0usize;
        let mut open_count = 0usize;
        let mut resolved_count = 0usize;

        // file -> (saw at least one unheld edge, every resolved one so far is
        // between two held surfaces, saw an anomalous/unresolved one)
        let mut per_file: BTreeMap<&str, (bool, bool, bool)> = BTreeMap::new();

        for (label, edge) in &self.unheld_edges {
            let entry = per_file.entry(label).or_insert((false, true, false));
            entry.0 = true;
            match &edge.faces {
                FacesEitherSide::Two(pair) => {
                    resolved_count += 1;
                    *resolved_pair_histogram.entry(pair.clone()).or_insert(0) += 1;
                    if is_bspline(&edge.curve_keyword) {
                        *bspline_pair_histogram.entry(pair.clone()).or_insert(0) += 1;
                    }
                    if edge.closed {
                        closed_count += 1;
                    } else {
                        open_count += 1;
                    }
                    let both_held = HELD_SURFACES.contains(&pair[0].as_str())
                        && HELD_SURFACES.contains(&pair[1].as_str());
                    if !both_held {
                        entry.1 = false;
                    }
                }
                FacesEitherSide::NotTwo {
                    distinct_faces,
                    unresolved_surface,
                } => {
                    *not_two
                        .entry((*distinct_faces, *unresolved_surface))
                        .or_insert(0) += 1;
                    entry.1 = false;
                    entry.2 = true;
                }
            }
        }

        println!("  resolved to exactly two faces, both surfaces known: {resolved_count}");
        println!(
            "  not exactly two distinct faces, or a face's surface unresolved \
             (distinct_faces, unresolved_surface) -> count:"
        );
        for ((distinct, unresolved), count) in &not_two {
            println!("    ({distinct}, unresolved_surface={unresolved}) -> {count}");
        }

        println!("\n1) histogram of (surface, surface) over every resolved unheld edge:");
        for (pair, count) in &resolved_pair_histogram {
            println!("  {} / {} -> {count}", pair[0], pair[1]);
        }
        println!("\n1b) same, restricted to a B-spline curve:");
        for (pair, count) in &bspline_pair_histogram {
            println!("  {} / {} -> {count}", pair[0], pair[1]);
        }

        let qualifying: BTreeSet<&&str> = per_file
            .iter()
            .filter(|(_, (has_any, all_held, has_anomaly))| *has_any && *all_held && !*has_anomaly)
            .map(|(name, _)| name)
            .collect();
        println!(
            "\n2) files where every unheld-curve edge sits between two of the five held \
             surfaces: {}",
            qualifying.len()
        );
        for name in &qualifying {
            println!("  {name}");
        }

        println!(
            "\n3) closed/open split of resolved unheld edges: closed={closed_count} \
             open={open_count}"
        );

        let cannot_find: usize = self
            .unheld_edges
            .iter()
            .filter(|(_, edge)| {
                matches!(
                    edge.faces,
                    FacesEitherSide::NotTwo {
                        distinct_faces: 0,
                        ..
                    }
                )
            })
            .count();
        println!(
            "\n4) unheld-curve edges where no face at all could be found for either side: \
             {cannot_find}"
        );
    }
}
