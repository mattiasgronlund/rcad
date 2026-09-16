//! Of the corpus files whose only refusal reason is "this crossing runs where
//! its two surfaces are too nearly parallel to follow it"
//! (`docs/decisions.md` §107.1's 44), how many are a real curve the arithmetic
//! ran out of angle on, and how many are two surfaces genuinely touching along
//! their whole shared line — a **gap** versus a **limit**, in the words
//! `Error::CannotFollowACrossingWhereItsSurfacesAreTangent`'s own doc comment
//! uses (`crates/rcad-geom/src/error.rs:490`).
//!
//! # How
//!
//! First, [`rcad_step::probe`] finds the files whose `ProbeReport::reasons` is
//! exactly one string — [`TARGET_REASON`], the reader's own wrapped form of
//! that message — and whose `fatal` is empty. Same walk `blocker_depth_probe`
//! uses, just filtered to one reason.
//!
//! Second, for each of those files, this reads the file's own text — never the
//! kernel's internals — to find every pair of `ADVANCED_FACE`s that share an
//! edge (through `FACE_BOUND`/`FACE_OUTER_BOUND` → `EDGE_LOOP` →
//! `ORIENTED_EDGE` → `EDGE_CURVE`, an `EDGE_CURVE` two distinct faces both
//! reference), and for the four surface-kind pairs whose tangency condition is
//! arithmetic on `DIRECTION` and `CARTESIAN_POINT` entities alone —
//! plane/plane, plane/cylinder, plane/cone, cylinder/cylinder — works out how
//! close that pair sits to the configuration where the two surfaces touch
//! along their whole shared line rather than crossing it.
//!
//! # What this can and cannot conclude
//!
//! **`probe` cannot say which pair refused, so finding a tangent pair in a file
//! does not make that file's refusal a limit.** `ProbeReport::reasons` is a
//! `BTreeSet<String>` filled by `report.reasons.insert(kind_of(&why))`, and
//! `kind_of` groups by the kind of thing that refused and drops the `#id`
//! entirely. One entry therefore covers any number of distinct edges, and the
//! six raise sites of this one error (`march.rs:286`, `:324`, `:328`, `:346`,
//! `:416` and `along_a_crossing.rs:326`) all collapse to the same text. A
//! genuinely tangent pair and a transverse pair the marcher gave up on early
//! are indistinguishable from this output.
//!
//! So there is exactly one way this census can earn LIMIT without naming the
//! pair: when **every** computable adjacent pair in the file is tangent,
//! whichever one refused was tangent. Anything less is
//! [`VerdictKind::TangencyNotAttributed`] — a tangency is present, and nothing
//! says it is the one that refused.
//!
//! The reverse direction is stronger. A file where **no** computable adjacent
//! pair is anywhere near a full-line tangency, and where every pair was
//! computable, holds no geometry that explains its refusal as a genuine
//! tangency at all, which leans gap.
//!
//! The control at the end of the run is what prices the whole criterion: it
//! counts how many corpus files that *don't* have this refusal also hold a
//! confirmed tangency. A criterion that fires on those just as readily
//! separates nothing.
//!
//! Two cylinders that are coaxial with the same radius are **one cylinder split
//! at a seam**, not two tangent surfaces, so they are counted separately and
//! never confirm a tangency.
//!
//! **Every number computed here is read straight out of the file's own
//! `DIRECTION`, `CARTESIAN_POINT` and real-valued attributes — no unit
//! conversion, no kernel geometry.** A cone's semi-angle is one exception
//! worth naming: it is printed assuming the file's angle unit is radians,
//! because working out the file's actual angle unit needs
//! `rcad_step::units`, which is private, and every cone in this corpus that
//! reaches this census is flagged with that assumption spelled out rather than
//! silently trusted.
//!
//! Run with `RCAD_STEP_CORPUS` pointing at a directory holding `nist/` and
//! `kicad/`:
//!
//! ```text
//! RCAD_STEP_CORPUS=<path> mise exec -- cargo run -p rcad-step --release \
//!   --example near_parallel_census
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{EntityId, StepFile, Value, probe};

/// `ProbeReport::reasons`' own text for this refusal — `kind_of` in
/// `crates/rcad-step/src/import.rs` wraps `Error::CannotFollowACrossingWhereItsSurfacesAreTangent`'s
/// message in "the builder or the sweep: geometry: …", and that wrapped form
/// is what actually lands in the set, confirmed against
/// `blocker_depth_probe`'s own pricing-table output before this constant was
/// written.
const TARGET_REASON: &str = "the builder or the sweep: geometry: this crossing runs where its two surfaces are too \
     nearly parallel to follow it";

/// Degrees short of the exact tangent configuration still counted as
/// "sitting on it" rather than "genuinely crossing at an angle".
const DEGENERATE_DEG: f64 = 2.0;

/// Degrees short of the exact tangent configuration past which a pair is
/// confidently a real, well-conditioned crossing rather than a candidate.
const GENERIC_DEG: f64 = 10.0;

/// Relative slack allowed when comparing an axis-to-plane or axis-to-axis
/// distance against the radius sum or difference a full-line tangency needs.
const RADIUS_RELATIVE_TOL: f64 = 1.0e-9;

fn main() {
    let root = corpus();
    let mut qualifying: Vec<(&'static str, String, FileCensus)> = Vec::new();
    let mut other_files = 0usize;
    let mut other_files_holding_a_tangency = 0usize;
    for family in ["nist", "kicad"] {
        let mut files = files_under(&root.join(family));
        files.sort();
        for path in files {
            let (file, only_this_reason) = read_and_probe(&path);
            let census = census_one_file(&file);
            if only_this_reason {
                qualifying.push((family, name_of(&path), census));
            } else {
                other_files += 1;
                if census.confirmed_tangent_pairs > 0 {
                    other_files_holding_a_tangency += 1;
                }
            }
        }
    }

    assert!(
        !qualifying.is_empty(),
        "no corpus file matched TARGET_REASON, so the string this census filters on has drifted \
         from what `kind_of` writes — every number below would be a count of nothing"
    );

    println!("=== files whose only probe reason is the near-parallel crossing ===");
    println!(
        "  found {} (docs/decisions.md §107.1 puts this at 44; this run's own count is the one \
         that matters)",
        qualifying.len()
    );

    let mut earned_limit = 0usize;
    let mut not_attributed = 0usize;
    let mut leans_gap = 0usize;
    let mut uncertain = 0usize;
    let mut holding_a_tangency = 0usize;
    for (family, name, census) in &qualifying {
        println!("\n  {family}/{name}");
        println!(
            "    faces read: {}   adjacent pairs found: {}   of those, computable: {}   \
             tangencies confirmed: {}   pairs that are one surface split at a seam: {}",
            census.faces_read,
            census.pairs_found,
            census.pairs_computable,
            census.confirmed_tangent_pairs,
            census.same_surface_pairs
        );
        match &census.closest {
            Some(evidence) => {
                let label = if evidence.confirmed_tangency {
                    "a confirmed tangent pair (not necessarily the pair that refused)"
                } else {
                    "closest pair"
                };
                println!("    {label}: {} / {}", evidence.kinds.0, evidence.kinds.1);
                println!("    {}", evidence.detail);
            }
            None => println!("    no computable pair found in this file"),
        }
        println!("    verdict: {}", census.kind.label());
        if census.confirmed_tangent_pairs > 0 {
            holding_a_tangency += 1;
        }
        match census.kind {
            VerdictKind::EveryPairTangent => earned_limit += 1,
            VerdictKind::TangencyNotAttributed => not_attributed += 1,
            VerdictKind::NoTangencyAnywhere => leans_gap += 1,
            VerdictKind::Uncertain => uncertain += 1,
        }
    }

    println!("\n=== summary over the {} files ===", qualifying.len());
    println!(
        "  LIMIT earned without naming the pair (every computable pair tangent): {earned_limit}"
    );
    println!(
        "  a tangency is present but nothing says it is the pair that refused:   {not_attributed}"
    );
    println!("  no computable pair is near a tangency, and coverage was complete:     {leans_gap}");
    println!("  uncertain (this file's own numbers did not settle it):                {uncertain}");

    println!("\n=== control: does \"this file holds a tangency\" pick the 44 out at all? ===");
    let of_the_44 = percent(holding_a_tangency, qualifying.len());
    let of_the_rest = percent(other_files_holding_a_tangency, other_files);
    println!(
        "  of the {} qualifying files, {holding_a_tangency} hold a confirmed tangency ({of_the_44:.1}%)",
        qualifying.len()
    );
    println!(
        "  of the {other_files} corpus files that refuse for something else or not at all, \
         {other_files_holding_a_tangency} hold one too ({of_the_rest:.1}%)"
    );
    if of_the_rest > 20.0 {
        println!(
            "  so holding a tangency is ordinary in this corpus and separates nothing: it cannot \
             carry a LIMIT verdict on its own"
        );
    }
}

fn percent(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    100.0 * part as f64 / whole as f64
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

/// The file, parsed, and whether `probe` finds it refuses for exactly the one
/// reason this census is about and nothing else.
fn read_and_probe(path: &Path) -> (StepFile, bool) {
    let bytes = fs::read(path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
    let file = StepFile::read(&bytes)
        .unwrap_or_else(|why| panic!("{}: every file in the corpus parses: {why}", path.display()));
    let mut log = ProvenanceLog::new();
    let found = probe(&file, &Policy::certified(), &mut log);
    let only_this =
        found.fatal.is_empty() && found.reasons.len() == 1 && found.reasons.contains(TARGET_REASON);
    (file, only_this)
}

/// What this file's own text says about how its adjacent face pairs sit —
/// never about which of them refused, which `probe` does not report.
struct FileCensus {
    faces_read: usize,
    pairs_found: usize,
    pairs_computable: usize,
    confirmed_tangent_pairs: usize,
    /// Pairs that are one cylinder split at a seam: coaxial, same radius,
    /// touching everywhere because they are the same surface. Not a tangency.
    same_surface_pairs: usize,
    closest: Option<PairEvidence>,
    kind: VerdictKind,
}

#[derive(Clone, Copy)]
enum VerdictKind {
    /// Every computable pair in the file is tangent, so whichever one refused
    /// was tangent — the one verdict this census can earn without naming the
    /// pair that refused.
    EveryPairTangent,
    /// A tangency is present, and nothing in `probe`'s output says it is the
    /// pair that refused.
    TangencyNotAttributed,
    /// Every pair was computable and none is near a tangency.
    NoTangencyAnywhere,
    Uncertain,
}

impl VerdictKind {
    fn label(self) -> &'static str {
        match self {
            VerdictKind::EveryPairTangent => {
                "LIMIT — every computable pair in this file is tangent, so whichever one refused \
                 was tangent"
            }
            VerdictKind::TangencyNotAttributed => {
                "NOT ATTRIBUTED — a tangency is present, but probe's deduplicated reason cannot \
                 say it is the pair that refused"
            }
            VerdictKind::NoTangencyAnywhere => {
                "LEANS GAP — every pair was computable and none is near a tangency, so this \
                 file's own geometry does not explain the refusal as one"
            }
            VerdictKind::Uncertain => "UNCERTAIN — this file's own numbers did not settle it",
        }
    }
}

fn census_one_file(file: &StepFile) -> FileCensus {
    let faces = faces_of(file);
    let owners = edge_curve_owners(file, &faces);
    let pairs = adjacent_face_pairs(&owners);

    let mut evaluated: Vec<PairEvidence> = Vec::new();
    for (a, b) in &pairs {
        let (Some(surface_a), Some(surface_b)) = (faces.get(a), faces.get(b)) else {
            continue;
        };
        evaluated.push(evaluate_pair(surface_a, surface_b));
    }
    let pairs_computable = evaluated.iter().filter(|e| e.closeness.is_some()).count();
    let confirmed_tangent_pairs = evaluated.iter().filter(|e| e.confirmed_tangency).count();
    let same_surface_pairs = evaluated.iter().filter(|e| e.same_surface).count();

    // The pair shown is the tangent one, when there is one, because that is the
    // evidence a reader wants to check. It is **not** a claim that this is the
    // pair that refused: `probe` groups its reasons by kind and drops the
    // `#id`, so nothing here can name the pair.
    let closest = evaluated
        .iter()
        .filter(|e| e.confirmed_tangency)
        .min_by(|a, b| a.closeness.unwrap().total_cmp(&b.closeness.unwrap()))
        .or_else(|| {
            evaluated
                .iter()
                .filter(|e| e.closeness.is_some())
                .min_by(|a, b| a.closeness.unwrap().total_cmp(&b.closeness.unwrap()))
        })
        .cloned();

    let every_pair_tangent = pairs_computable > 0
        && pairs_computable == pairs.len()
        && confirmed_tangent_pairs == pairs_computable;
    let nothing_near_a_tangency = pairs_computable == pairs.len()
        && evaluated
            .iter()
            .all(|e| e.closeness.is_some_and(|degrees| degrees > GENERIC_DEG));
    let kind = if every_pair_tangent {
        VerdictKind::EveryPairTangent
    } else if confirmed_tangent_pairs > 0 {
        VerdictKind::TangencyNotAttributed
    } else if nothing_near_a_tangency {
        VerdictKind::NoTangencyAnywhere
    } else {
        VerdictKind::Uncertain
    };

    FileCensus {
        faces_read: faces.len(),
        pairs_found: pairs.len(),
        pairs_computable,
        confirmed_tangent_pairs,
        same_surface_pairs,
        closest,
        kind,
    }
}

/// Every `ADVANCED_FACE` (or `FACE_SURFACE`) this file names, with the
/// geometry its own surface entity carries.
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
/// each `EDGE_CURVE`, over every face this file has geometry for.
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

/// Every distinct pair of faces that share exactly one `EDGE_CURVE`'s two
/// owners — a shared edge between two faces of a manifold shell always has
/// exactly two, one per face, walked opposite ways.
fn adjacent_face_pairs(
    owners: &BTreeMap<EntityId, BTreeSet<EntityId>>,
) -> BTreeSet<(EntityId, EntityId)> {
    let mut pairs = BTreeSet::new();
    for faces in owners.values() {
        if faces.len() != 2 {
            continue;
        }
        let mut both = faces.iter().copied();
        let first = both.next().expect("len is 2");
        let second = both.next().expect("len is 2");
        pairs.insert(if first < second {
            (first, second)
        } else {
            (second, first)
        });
    }
    pairs
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
    Cone {
        axis: [f64; 3],
        /// The semi-angle exactly as the file wrote it — not yet turned into
        /// degrees or confirmed to be in radians. See the module doc comment.
        semi_angle_raw: f64,
    },
    Sphere,
    Torus,
    Other(String),
}

fn kind_name(surface: &SurfaceGeom) -> &str {
    match surface {
        SurfaceGeom::Plane { .. } => "PLANE",
        SurfaceGeom::Cylinder { .. } => "CYLINDRICAL_SURFACE",
        SurfaceGeom::Cone { .. } => "CONICAL_SURFACE",
        SurfaceGeom::Sphere => "SPHERICAL_SURFACE",
        SurfaceGeom::Torus => "TOROIDAL_SURFACE",
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
        "CONICAL_SURFACE" => {
            let position = reference_at(record, 1)?;
            let (_, axis) = placement_of(file, position)?;
            let semi_angle_raw = number_at(record, 3)?;
            Some(SurfaceGeom::Cone {
                axis,
                semi_angle_raw,
            })
        }
        "SPHERICAL_SURFACE" => Some(SurfaceGeom::Sphere),
        "TOROIDAL_SURFACE" => Some(SurfaceGeom::Torus),
        other => Some(SurfaceGeom::Other(other.to_owned())),
    }
}

/// An `AXIS2_PLACEMENT_3D`'s origin and axis — `(0,0,1)` when the file leaves
/// the axis out, exactly as `rcad_step::geometry::Reading::placement_at`
/// treats it. The reference direction (the placement's third attribute) is
/// never read: nothing this census computes needs which way the first
/// coordinate runs, only where the surface sits and which way its axis or
/// normal points.
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

/// What this census could work out about one candidate pair, and how close it
/// sits to a configuration where the two surfaces touch along their whole
/// shared line rather than crossing it at an angle.
#[derive(Clone)]
struct PairEvidence {
    kinds: (String, String),
    detail: String,
    /// Degrees short of the exact tangent configuration. Smaller is more
    /// suspicious. `None` when this pair's kinds are not one of the four this
    /// census computes an angle for, or a `DIRECTION` failed to normalize.
    closeness: Option<f64>,
    /// Whether the file's own radius and distance numbers also match a
    /// full-line tangency, not just the angle — the strongest evidence this
    /// census can produce about a pair, which is still not evidence about
    /// *which* pair refused.
    confirmed_tangency: bool,
    /// Whether the two faces carry the same surface — a cylinder split at a
    /// seam. They touch everywhere, which is not a tangency of two surfaces.
    same_surface: bool,
}

impl PairEvidence {
    fn uncomputable(kinds: (String, String), why: &str) -> Self {
        PairEvidence {
            kinds,
            detail: why.to_owned(),
            closeness: None,
            confirmed_tangency: false,
            same_surface: false,
        }
    }
}

fn evaluate_pair(a: &SurfaceGeom, b: &SurfaceGeom) -> PairEvidence {
    let kinds = (kind_name(a).to_owned(), kind_name(b).to_owned());
    match (a, b) {
        (SurfaceGeom::Plane { normal: n1, .. }, SurfaceGeom::Plane { normal: n2, .. }) => {
            plane_plane(kinds, *n1, *n2)
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
        ) => plane_cylinder(kinds, *origin, *normal, *cylinder_origin, *axis, *radius),
        (
            SurfaceGeom::Plane { normal, .. },
            SurfaceGeom::Cone {
                axis,
                semi_angle_raw,
            },
        )
        | (
            SurfaceGeom::Cone {
                axis,
                semi_angle_raw,
            },
            SurfaceGeom::Plane { normal, .. },
        ) => plane_cone(kinds, *normal, *axis, *semi_angle_raw),
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
        ) => cylinder_cylinder(kinds, *o1, *a1, *r1, *o2, *a2, *r2),
        _ => PairEvidence::uncomputable(
            kinds,
            "not one of plane/plane, plane/cylinder, plane/cone or cylinder/cylinder",
        ),
    }
}

/// Two planes cross in a line whose direction is `n1 × n2`, which is zero
/// exactly when the planes are parallel — the only way two planes can fail to
/// have a well-defined crossing direction.
fn plane_plane(kinds: (String, String), n1: [f64; 3], n2: [f64; 3]) -> PairEvidence {
    let Some(angle) = angle_deg(n1, n2) else {
        return PairEvidence::uncomputable(kinds, "a plane's normal did not normalize");
    };
    let folded = angle.min(180.0 - angle);
    PairEvidence {
        kinds,
        detail: format!("normals {folded:.6} deg from parallel"),
        closeness: Some(folded),
        confirmed_tangency: false,
        same_surface: false,
    }
}

/// A cylinder touches a plane along its whole axis line exactly when the axis
/// lies in the plane (axis ⟂ normal, so the axis-to-normal angle is 90°) and
/// the axis sits exactly `radius` from the plane.
fn plane_cylinder(
    kinds: (String, String),
    plane_origin: [f64; 3],
    normal: [f64; 3],
    cylinder_origin: [f64; 3],
    axis: [f64; 3],
    radius: f64,
) -> PairEvidence {
    let (Some(angle), Some(unit_normal)) = (angle_deg(axis, normal), normalize(normal)) else {
        return PairEvidence::uncomputable(kinds, "the axis or the normal did not normalize");
    };
    let deviation = (angle - 90.0).abs();
    let distance = dot(sub(cylinder_origin, plane_origin), unit_normal).abs();
    let matches_radius = close(distance, radius);
    PairEvidence {
        kinds,
        detail: format!(
            "axis {deviation:.6} deg short of lying in the plane; axis-to-plane distance \
             {distance:.12} vs radius {radius:.12}"
        ),
        closeness: Some(deviation),
        confirmed_tangency: deviation < DEGENERATE_DEG && matches_radius,
        same_surface: false,
    }
}

/// A cone is tangent to a plane along one generator line when the plane's
/// normal sits at `90° − semi_angle` (or `90° + semi_angle`, the other side of
/// the double cone) from the axis. This does not check the plane's offset
/// against the apex, which the tangency also needs — see the module doc
/// comment — so this never confirms a limit on its own.
fn plane_cone(
    kinds: (String, String),
    normal: [f64; 3],
    axis: [f64; 3],
    semi_angle_raw: f64,
) -> PairEvidence {
    let Some(angle) = angle_deg(axis, normal) else {
        return PairEvidence::uncomputable(kinds, "the axis or the normal did not normalize");
    };
    let semi_angle_deg = semi_angle_raw.to_degrees();
    let target_near = (90.0 - semi_angle_deg).abs();
    let target_far = (90.0 + semi_angle_deg).abs();
    let deviation = (angle - target_near).abs().min((angle - target_far).abs());
    PairEvidence {
        kinds,
        detail: format!(
            "axis/normal angle {angle:.6} deg; cone semi-angle {semi_angle_deg:.6} deg \
             (assumed radians in the file, not confirmed); {deviation:.6} deg short of the \
             generator-tangent orientation (apex offset not checked, so this never confirms a \
             tangency by itself)"
        ),
        closeness: Some(deviation),
        confirmed_tangency: false,
        same_surface: false,
    }
}

/// Two cylinders touch along a full line exactly when their axes are parallel
/// and the distance between the two axis lines equals `r1 + r2` (external
/// tangency) or `|r1 − r2|` (one inside the other).
///
/// The `|r1 − r2|` test has a degenerate case that is not a tangency at all:
/// coaxial cylinders of the *same* radius, where the distance and `|r1 − r2|`
/// are both zero. Those two faces carry one surface, split at a seam — the
/// usual way a whole cylinder is written as two `ADVANCED_FACE`s — and they
/// touch everywhere for that reason and no other. Counted as
/// [`PairEvidence::same_surface`], never as a tangency.
fn cylinder_cylinder(
    kinds: (String, String),
    o1: [f64; 3],
    a1: [f64; 3],
    r1: f64,
    o2: [f64; 3],
    a2: [f64; 3],
    r2: f64,
) -> PairEvidence {
    let (Some(angle), Some(unit_a1)) = (angle_deg(a1, a2), normalize(a1)) else {
        return PairEvidence::uncomputable(kinds, "an axis did not normalize");
    };
    let folded = angle.min(180.0 - angle);
    let distance = length(cross(unit_a1, sub(o2, o1)));
    let matches_external = close(distance, r1 + r2);
    let matches_internal = close(distance, (r1 - r2).abs());
    let coaxial = folded < DEGENERATE_DEG && close(distance, 0.0);
    let same_surface = coaxial && close(r1, r2);
    let same_surface_note = if same_surface {
        " — one cylinder split at a seam, not two tangent surfaces"
    } else {
        ""
    };
    PairEvidence {
        kinds,
        detail: format!(
            "axes {folded:.6} deg from parallel; axis-to-axis distance {distance:.12} vs \
             r1+r2={:.12} / |r1-r2|={:.12}{same_surface_note}",
            r1 + r2,
            (r1 - r2).abs()
        ),
        closeness: Some(folded),
        confirmed_tangency: !same_surface
            && folded < DEGENERATE_DEG
            && (matches_external || matches_internal),
        same_surface,
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
