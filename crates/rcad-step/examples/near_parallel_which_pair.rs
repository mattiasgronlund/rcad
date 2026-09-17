//! Of the corpus files whose only probe reason is "this crossing runs
//! where its two surfaces are too nearly parallel to follow it"
//! (`docs/decisions.md` §107.1 counted 44 of them; this run finds 24 that
//! report it at all, and none at all where it is one reason among several —
//! §117 records that and does not claim to know why), a census already found
//! *a* tangent face pair in every one and called them all LIMIT. **That census never checked
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
//! # What that run printed, on 2026-09-17
//!
//! ```text
//! === summary over the 24 files ===
//!   ThisEntityOfTheSolidWasRefused (an #id to read a pair from): 24
//!   TheSolidTheFileDescribesIsNotOne (no entity, whole-solid form):      0
//!   anything else (succeeded, or a different refusal):                   0
//!   of the 24 with a named entity:
//!     TANGENT — touch without crossing, a limit:  18
//!     TRANSVERSE — they really cross, a GAP:       0
//!     THEY NEVER MEET — no crossing to follow:     0
//!     UNDECIDED — no test here knows the pair:     6
//!     no surface pair readable from the file:      0
//! ```
//!
//! **Not one of the 24 is a plain crossing the marcher failed to follow.**
//! Eighteen are a ball capping a pipe — same radius, the ball's centre exactly
//! on the pipe's axis, so the two touch all the way round and there is no
//! crossing to follow. Six are two pipes of the same radius crossing at right
//! angles with their axes actually meeting, which is the one shape
//! [`evaluate_pipe_against_pipe`] refuses to call either way.
//!
//! # Why the earlier "all transverse, a GAP" was not a measurement
//!
//! Before this, [`evaluate_surfaces`] knew plane/plane, plane/cylinder and
//! cylinder/cylinder, carried a `bool` called `confirmed_limit`, and answered
//! `false` for everything else — which printed as "TRANSVERSE — a gap".
//! Cylinder/sphere was none of the three, so every one of the eighteen was the
//! default arm of a `match` reporting itself as a verdict.
//! `docs/decisions.md` §113.5 caught that; §117 records what the numbers are
//! once a test actually runs. The `bool` is now a [`Verdict`] with a fourth
//! case, so "no test here knows this pair" can no longer come out as "they
//! really cross".
//!
//! The six had a second version of the same fault. The cylinder/cylinder arm
//! called a pair transverse whenever the axes were more than
//! [`DEGENERATE_DEG`] from parallel, and the distance it printed beside that
//! verdict was one pipe's **origin** measured against the other's axis — the
//! point the file happened to write down, not a property of the pair. It read
//! `0.400000000000` against a touch at `0.200000000000` for pairs whose axes
//! in fact pass through each other, distance `0`.
//!
//! # What the refusal carries now, and why that is still not an `#id`
//!
//! `docs/decisions.md` §110 gave the refusal the two `Surface`s the marcher
//! gave up between, and which of its questions gave up. Turning those two
//! surfaces back into two `#id`s was tried and **does not work**: matched
//! against every surface this reader wrote down for the file, the pair scores
//! **0 matches**. The surfaces the sweep is holding are a translation away
//! from the ones the reader wrote. Naming the pair by `#id` that way needs
//! whatever carries that translation, which is `rcad-topo`'s and not this
//! crate's; §110 records the measurement and leaves the cause open.
//!
//! This example does not need it. It reads both faces out of the file's own
//! topology, so both sit in the same frame the file wrote them in and the
//! translation cancels between them — every number below is a distance or an
//! angle *between the two*, and never a position.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{EntityId, Import, Refusal, StepFile, Value, probe};

/// How `ProbeReport::reasons`' own text for this refusal **starts**, the same
/// two prefixes `near_parallel_census.rs` selects on and for the same reason:
/// the message now ends with which of the marcher's questions gave up, so a
/// whole-string match would select nothing (`docs/decisions.md` §110).
const TANGENCY_REASON_STARTS: &str = "the builder or the sweep: geometry: this crossing runs where its two surfaces are too \
     nearly parallel to follow it";

/// The stepping wall, which used to share the message above.
const STEPPING_REASON_STARTS: &str =
    "the builder or the sweep: geometry: no step along this crossing stayed on it";

/// Relative slack allowed when comparing an axis-to-plane or axis-to-axis
/// distance against the radius sum or difference a full-line tangency needs.
const RADIUS_RELATIVE_TOL: f64 = 1.0e-9;

/// Degrees short of the exact tangent configuration still counted as a
/// confirmed match, same threshold `near_parallel_census.rs` uses.
const DEGENERATE_DEG: f64 = 2.0;

fn main() {
    let root = corpus();
    let mut qualifying: Vec<(&'static str, String, StepFile)> = Vec::new();
    let mut among_others = 0usize;
    let mut what_else_they_say: BTreeSet<String> = BTreeSet::new();
    for family in ["nist", "kicad"] {
        let mut files = files_under(&root.join(family));
        files.sort();
        for path in files {
            match how_it_refuses(&path) {
                HowItRefuses::OnlyTheMarcherGaveUp(file) => {
                    qualifying.push((family, name_of(&path), file));
                }
                HowItRefuses::TheMarcherGaveUpAmongOthers(others) => {
                    among_others += 1;
                    what_else_they_say.extend(others);
                }
                HowItRefuses::SomethingElse => {}
            }
        }
    }

    println!("=== files whose only probe reason is the near-parallel crossing ===");
    println!("  found {}", qualifying.len());
    println!(
        "  and {among_others} more where it is one reason among several, so which entity the \
         reader stopped at proves nothing — not classified below"
    );
    for reason in &what_else_they_say {
        println!("    those also refuse for: {reason}");
    }

    let mut named_entity = 0usize;
    let mut whole_solid = 0usize;
    let mut other = 0usize;
    let mut confirmed_tangent = 0usize;
    let mut transverse_gap = 0usize;
    let mut never_meet = 0usize;
    let mut cannot_tell = 0usize;
    let mut no_pair_readable = 0usize;

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
                        match evidence.verdict {
                            Verdict::Tangent => {
                                confirmed_tangent += 1;
                                println!(
                                    "    verdict: TANGENT — they touch without crossing, a \
                                     genuine limit"
                                );
                            }
                            Verdict::Transverse => {
                                transverse_gap += 1;
                                println!(
                                    "    verdict: TRANSVERSE — they really cross, so this is a gap"
                                );
                            }
                            Verdict::NeverMeet => {
                                never_meet += 1;
                                println!(
                                    "    verdict: THEY NEVER MEET — there is no crossing to follow"
                                );
                            }
                            Verdict::CannotTell => {
                                cannot_tell += 1;
                                println!(
                                    "    verdict: UNDECIDED — no test here measured this pair"
                                );
                            }
                        }
                    }
                    None => {
                        no_pair_readable += 1;
                        println!(
                            "    could not read a surface pair for entity {entity} from the \
                             file's own text"
                        );
                    }
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
        println!("  of the {named_entity} with a named entity:");
        println!("    TANGENT — touch without crossing, a limit:  {confirmed_tangent}");
        println!("    TRANSVERSE — they really cross, a GAP:      {transverse_gap}");
        println!("    THEY NEVER MEET — no crossing to follow:    {never_meet}");
        println!("    UNDECIDED — no test here knows the pair:    {cannot_tell}");
        println!("    no surface pair readable from the file:     {no_pair_readable}");
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

/// What `probe` says about one file, as far as this census cares.
enum HowItRefuses {
    /// The marcher giving up on a crossing, and nothing else. These are the
    /// files the census classifies: the one entity the reader stopped at can
    /// only be the marcher's, because there is nothing else to blame.
    OnlyTheMarcherGaveUp(StepFile),
    /// The marcher gave up, but the file refuses for other reasons too, so the
    /// entity the reader stopped at need not be the marcher's one. Carries
    /// those other reasons.
    TheMarcherGaveUpAmongOthers(Vec<String>),
    /// The marcher never gave up on this file.
    SomethingElse,
}

fn how_it_refuses(path: &Path) -> HowItRefuses {
    let bytes = fs::read(path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
    let file = StepFile::read(&bytes)
        .unwrap_or_else(|why| panic!("{}: every file in the corpus parses: {why}", path.display()));
    let mut log = ProvenanceLog::new();
    let found = probe(&file, &Policy::certified(), &mut log);
    if !found
        .reasons
        .iter()
        .any(|reason| is_the_marcher_giving_up(reason))
    {
        return HowItRefuses::SomethingElse;
    }
    let others: Vec<String> = found
        .reasons
        .iter()
        .filter(|reason| !is_the_marcher_giving_up(reason))
        .map(|reason| reason.to_string())
        .collect();
    if found.fatal.is_empty() && others.is_empty() {
        return HowItRefuses::OnlyTheMarcherGaveUp(file);
    }
    HowItRefuses::TheMarcherGaveUpAmongOthers(others)
}

/// Whether one of `probe`'s reason strings is the marcher giving up on a
/// crossing, under either of the two names that wall now has.
fn is_the_marcher_giving_up(reason: &str) -> bool {
    reason.starts_with(TANGENCY_REASON_STARTS) || reason.starts_with(STEPPING_REASON_STARTS)
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
        return PairEvidence::cannot_tell("a face's surface did not parse");
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
    Sphere {
        centre: [f64; 3],
        radius: f64,
    },
    Other(String),
}

fn kind_name(surface: &SurfaceGeom) -> &str {
    match surface {
        SurfaceGeom::Plane { .. } => "PLANE",
        SurfaceGeom::Cylinder { .. } => "CYLINDRICAL_SURFACE",
        SurfaceGeom::Sphere { .. } => "SPHERICAL_SURFACE",
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
        "SPHERICAL_SURFACE" => {
            let position = reference_at(record, 1)?;
            let (centre, _) = placement_of(file, position)?;
            let radius = number_at(record, 2)?;
            Some(SurfaceGeom::Sphere { centre, radius })
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
    verdict: Verdict,
}

/// What a pair's own numbers say once a test that knows that pair has run.
///
/// The fourth case is the whole point of this type. It used to be a `bool`,
/// and every pair no test here knew came back `false` — which printed as
/// "TRANSVERSE — a gap". That is the default arm of a `match` reporting
/// itself as a measurement, and it is what `docs/decisions.md` §113.5 caught.
/// `CannotTell` is a result, not a failure, and it must never be folded into
/// `Transverse`.
enum Verdict {
    /// The two surfaces touch without crossing, so there is no crossing for
    /// the marcher to follow. The refusal is a limit.
    Tangent,
    /// The two surfaces really do cross, so a crossing is there and the
    /// marcher failed to follow it. The refusal is a gap.
    Transverse,
    /// The two surfaces have no point in common at all.
    NeverMeet,
    /// No test here knows this pair of shapes.
    CannotTell,
}

impl PairEvidence {
    fn cannot_tell(why: &str) -> Self {
        PairEvidence {
            detail: why.to_owned(),
            verdict: Verdict::CannotTell,
        }
    }
}

fn evaluate_surfaces(a: &SurfaceGeom, b: &SurfaceGeom) -> PairEvidence {
    match (a, b) {
        (SurfaceGeom::Plane { normal: n1, .. }, SurfaceGeom::Plane { normal: n2, .. }) => {
            let Some(angle) = angle_deg(*n1, *n2) else {
                return PairEvidence::cannot_tell("a plane's normal did not normalize");
            };
            let folded = angle.min(180.0 - angle);
            PairEvidence {
                detail: format!("normals {folded:.6} deg from parallel"),
                verdict: if folded < DEGENERATE_DEG {
                    Verdict::Tangent
                } else {
                    Verdict::Transverse
                },
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
                return PairEvidence::cannot_tell("the axis or the normal did not normalize");
            };
            let deviation = (angle - 90.0).abs();
            let distance = dot(sub(*cylinder_origin, *origin), unit_normal).abs();
            let matches_radius = close(distance, *radius);
            PairEvidence {
                detail: format!(
                    "axis {deviation:.6} deg short of lying in the plane; axis-to-plane \
                     distance {distance:.12} vs radius {radius:.12}"
                ),
                verdict: if deviation < DEGENERATE_DEG && matches_radius {
                    Verdict::Tangent
                } else {
                    Verdict::Transverse
                },
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
        ) => evaluate_pipe_against_pipe(*o1, *a1, *r1, *o2, *a2, *r2),
        (
            SurfaceGeom::Cylinder {
                origin,
                axis,
                radius: pipe_radius,
            },
            SurfaceGeom::Sphere {
                centre,
                radius: ball_radius,
            },
        )
        | (
            SurfaceGeom::Sphere {
                centre,
                radius: ball_radius,
            },
            SurfaceGeom::Cylinder {
                origin,
                axis,
                radius: pipe_radius,
            },
        ) => evaluate_ball_against_pipe(*centre, *ball_radius, *origin, *axis, *pipe_radius),
        _ => PairEvidence::cannot_tell(
            "no test here knows this pair: it is not plane/plane, plane/cylinder, \
             cylinder/cylinder or cylinder/sphere",
        ),
    }
}

/// Whether two pipes touch without crossing, really cross, or never meet.
///
/// Parallel axes and leaning axes are two different questions, and the
/// distance that answers one does not answer the other.
///
/// **Parallel.** The pipes meet in straight lines, and the whole question is
/// the distance `d` between the two axes against `r1 + r2` and `|r1 - r2|`:
/// equal to either, they touch along one line; between them, they cross in two
/// lines; outside them, they never meet. Same three cases as the ball and the
/// pipe below, for the same reason.
///
/// **Leaning.** `d` now means the distance between two lines that are not
/// parallel, which is a different formula — the old one measured one pipe's
/// origin against the other's axis, which is whatever point the file happened
/// to write down and not a property of the pair at all. Either pipe runs off
/// to infinity along its own axis and so leaves any bounded distance from the
/// other's axis, so once the two walls reach each other at the common
/// perpendicular they must cut: `d > r1 + r2` is never meeting, `d = r1 + r2`
/// is a touch at exactly that one point, and anything less is a real crossing.
/// Radii do not have a second case here — no pipe can hide inside another it
/// is not parallel to.
///
/// The one leaning pair this does not answer is two pipes of the same radius
/// `r` whose axes actually cross. Put those axes along `z` and `x`: the walls
/// meet where `x² = z²` and `y² = r² - x²`, which is two whole ellipses, and
/// the two ellipses cross each other at `(0, ±r, 0)` — which is exactly where
/// the two walls' normals line up, `(0, ±r, 0)` for both. So a crossing is
/// there to follow everywhere but at two points, and at those two points the
/// walls touch and four branches meet. Neither "touch without crossing" nor
/// "really cross" is the truth about that pair, so it is `CannotTell`, named
/// rather than swept into `Transverse`.
fn evaluate_pipe_against_pipe(
    o1: [f64; 3],
    a1: [f64; 3],
    r1: f64,
    o2: [f64; 3],
    a2: [f64; 3],
    r2: f64,
) -> PairEvidence {
    let (Some(angle), Some(unit_a1), Some(unit_a2)) =
        (angle_deg(a1, a2), normalize(a1), normalize(a2))
    else {
        return PairEvidence::cannot_tell("an axis did not normalize");
    };
    let folded = angle.min(180.0 - angle);
    let touch_outside = r1 + r2;
    let between_axes = cross(unit_a1, unit_a2);
    let leaning = length(between_axes);
    if folded < DEGENERATE_DEG {
        let distance = length(cross(unit_a1, sub(o2, o1)));
        let touch_inside = (r1 - r2).abs();
        let detail = format!(
            "axes {folded:.6} deg from parallel; axis-to-axis distance {distance:.12} vs a touch \
             from outside at {touch_outside:.12} / from inside at {touch_inside:.12}"
        );
        let verdict = if close(distance, touch_outside) || close(distance, touch_inside) {
            Verdict::Tangent
        } else if distance > touch_outside || distance < touch_inside {
            Verdict::NeverMeet
        } else {
            Verdict::Transverse
        };
        return PairEvidence { detail, verdict };
    }
    let distance = dot(sub(o2, o1), between_axes).abs() / leaning;
    let detail = format!(
        "axes {folded:.6} deg from parallel, leaning on each other; the two axes pass \
         {distance:.12} apart vs a touch at {touch_outside:.12}"
    );
    let verdict = if close(distance, touch_outside) {
        Verdict::Tangent
    } else if distance > touch_outside {
        Verdict::NeverMeet
    } else if close(distance, 0.0) && close(r1, r2) {
        return PairEvidence::cannot_tell(
            "two leaning pipes of the same radius whose axes cross meet in two whole ellipses \
             that cross each other at the two points where the walls touch — a crossing is \
             there to follow, except at those two points, and this test does not answer for \
             them",
        );
    } else {
        Verdict::Transverse
    };
    PairEvidence { detail, verdict }
}

/// Whether a ball and a pipe touch without crossing, really cross, or never
/// meet, from the only two numbers that settle it.
///
/// Put the pipe's axis along `z`. A ball of radius `rs` whose centre sits a
/// distance `e` from that axis meets the pipe wall of radius `rc` where
/// `(z - h)² = rs² - rc² - e² + 2·rc·e·cos t`, going round the pipe by `t`.
/// The right-hand side runs between `rs² - (rc + e)²` and `rs² - (rc - e)²`,
/// so the whole question is where `rs` sits against `rc + e` and `|rc - e|`:
///
/// - `rs` equal to either one: the right-hand side just reaches zero and the
///   two surfaces **touch without crossing**. With the centre on the axis
///   (`e = 0`) the two values are the same number, `rc`, and the touch runs
///   all the way round — a ball capping a pipe, which is the capsule end
///   `docs/decisions.md` §113.5 worked out by hand. With the centre off the
///   axis they are different numbers and the touch is a single point.
/// - `rs` between them, or above `rc + e`: the right-hand side is positive
///   somewhere, so a real crossing curve exists — **they really cross**.
/// - `rs` below `|rc - e|`: the right-hand side is negative everywhere and
///   **they never meet**.
///
/// Those five are every case, because `rs`, being one real number, is either
/// equal to one of the two values, between them, above both, or below both.
/// So this answers every ball-and-pipe pair, and never `CannotTell` — bar an
/// axis that will not normalize, which is a broken file and not a shape.
fn evaluate_ball_against_pipe(
    centre: [f64; 3],
    ball_radius: f64,
    pipe_origin: [f64; 3],
    pipe_axis: [f64; 3],
    pipe_radius: f64,
) -> PairEvidence {
    let Some(unit_axis) = normalize(pipe_axis) else {
        return PairEvidence::cannot_tell("the pipe's axis did not normalize");
    };
    let off_axis = length(cross(unit_axis, sub(centre, pipe_origin)));
    let touch_outside = pipe_radius + off_axis;
    let touch_inside = (pipe_radius - off_axis).abs();
    let where_it_would_touch = if close(off_axis, 0.0) {
        "the ball's centre is on the pipe's axis, so a touch runs all the way round"
    } else {
        "the ball's centre is off the pipe's axis, so a touch is one point"
    };
    let detail = format!(
        "ball centre {off_axis:.12} from the pipe's axis; ball radius {ball_radius:.12} vs a \
         touch from outside at {touch_outside:.12} / from inside at {touch_inside:.12} — \
         {where_it_would_touch}"
    );
    let verdict = if close(ball_radius, touch_outside) || close(ball_radius, touch_inside) {
        Verdict::Tangent
    } else if ball_radius < touch_inside {
        Verdict::NeverMeet
    } else {
        Verdict::Transverse
    };
    PairEvidence { detail, verdict }
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
