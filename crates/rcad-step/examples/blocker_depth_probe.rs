//! What blocks the STEP corpus, counted per file rather than per first
//! refusal.
//!
//! Four lanes each priced a hypothesis by how many files a blocker stops
//! **first**, and §45.2 says why that number is an upper bound and never a
//! projection: a file freed of its first blocker usually meets a second one
//! right behind it. This runs [`rcad_step::probe`] — the walk that keeps going
//! past a refused face, bound or edge instead of stopping at the first one —
//! and turns what it finds into the number that actually prices a fix: not how
//! many files a reason stops first, but how many files have **only** that
//! reason and nothing else behind it.
//!
//! No assertions and no gate cost; §56.7 is why it is kept rather than thrown
//! away, the same argument that landed the three censuses beside it. Run it
//! with `RCAD_STEP_CORPUS` pointing at a directory holding `nist/` and
//! `kicad/`:
//!
//! ```text
//! RCAD_STEP_CORPUS=<path> mise exec -- cargo run -p rcad-step --release \
//!   --example blocker_depth_probe
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile, probe};

fn main() {
    let root = corpus();
    let mut findings: Vec<FileFinding> = Vec::new();
    for family in ["nist", "kicad"] {
        let mut files = files_under(&root.join(family));
        files.sort();
        for path in files {
            findings.push(probe_one_file(family, &path));
        }
    }
    report(&findings);
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

/// What the depth probe found for one file, and the independent check against
/// [`Import::read`] that keeps it honest.
struct FileFinding {
    family: &'static str,
    name: String,
    /// Whether `Import::read` imports this file today, checked directly and
    /// never inferred from an empty reason set.
    imports: bool,
    /// Every distinct reason a face, a bound or an edge was skipped, kept
    /// walking past. Empty for a file that imports.
    reasons: BTreeSet<String>,
    /// A reason that stopped the walk before it had anything left to skip —
    /// see `ProbeReport`'s own doc comment for what puts a reason here rather
    /// than in `reasons`.
    fatal: BTreeSet<String>,
}

fn probe_one_file(family: &'static str, path: &Path) -> FileFinding {
    let bytes = fs::read(path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
    let file = StepFile::read(&bytes).unwrap_or_else(|why| {
        panic!(
            "{}: every file in both corpora parses: {why}",
            path.display()
        )
    });

    let mut import_log = ProvenanceLog::new();
    let imports = Import::read(&file, &Policy::certified(), &mut import_log).is_ok();

    let mut probe_log = ProvenanceLog::new();
    let found = probe(&file, &Policy::certified(), &mut probe_log);

    let name = name_of(path);
    assert!(
        !imports || (found.reasons.is_empty() && found.fatal.is_empty()),
        "{name} imports under Import::read and the depth probe still found a reason — the \
         probe is not the same walk any more: reasons={:?} fatal={:?}",
        found.reasons,
        found.fatal
    );

    FileFinding {
        family,
        name,
        imports,
        reasons: found.reasons,
        fatal: found.fatal,
    }
}

fn report(findings: &[FileFinding]) {
    report_reason_sets(findings);
    report_depth_histogram(findings);
    report_pricing_table(findings);
    report_co_occurrence(findings);
    report_fatal_bucket(findings);
    report_family_union(findings);
}

/// Deliverable 1: per file, the set of distinct refusal reasons it produces.
fn report_reason_sets(findings: &[FileFinding]) {
    println!("=== 1) per-file distinct refusal reasons ===");
    for finding in findings {
        if finding.imports {
            println!("  {}/{}  imports", finding.family, finding.name);
            continue;
        }
        let mut all: Vec<&str> = finding
            .reasons
            .iter()
            .map(String::as_str)
            .chain(finding.fatal.iter().map(|reason| reason.as_str()))
            .collect();
        all.sort_unstable();
        let fatal_marker = if finding.fatal.is_empty() {
            ""
        } else {
            " [fatal]"
        };
        println!(
            "  {}/{}  {} reason(s){fatal_marker}: {}",
            finding.family,
            finding.name,
            all.len(),
            all.join(" | ")
        );
    }
}

/// Deliverable 2: how many files have exactly 0, 1, 2, 3, ... distinct reasons
/// — counting `reasons` only (the reasons the walk actually kept going past),
/// never `fatal`, because a fatal file is one the walk could not keep going
/// past at all and belongs in its own bucket (deliverable 5).
fn report_depth_histogram(findings: &[FileFinding]) {
    println!("\n=== 2) histogram: how many distinct reasons does a file have ===");
    let mut by_family: BTreeMap<&str, BTreeMap<usize, usize>> = BTreeMap::new();
    for finding in findings {
        *by_family
            .entry(finding.family)
            .or_default()
            .entry(finding.reasons.len())
            .or_default() += 1;
    }
    for (family, histogram) in &by_family {
        let total: usize = histogram.values().sum();
        println!("  {family} ({total} files):");
        for (depth, count) in histogram {
            println!("    {depth:>3} distinct reason(s): {count:>4} files");
        }
    }
}

/// Deliverable 3: for each reason, how many files have it and no other — what
/// fixing that reason alone is actually worth, beside the count of files it
/// stops first.
fn report_pricing_table(findings: &[FileFinding]) {
    println!("\n=== 3) the number that prices everything ===");
    println!("  files where a reason is the ONLY one this probe found, all files, both corpora:");
    let mut only: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        match (finding.reasons.len(), finding.fatal.len()) {
            (1, 0) => {
                let reason = finding.reasons.iter().next().expect("len is 1");
                *only.entry(reason.as_str()).or_default() += 1;
            }
            (0, 1) => {
                let reason = finding.fatal.iter().next().expect("len is 1");
                *only.entry(reason.as_str()).or_default() += 1;
            }
            _ => {}
        }
    }
    let mut sorted: Vec<(&str, usize)> = only.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (reason, count) in sorted {
        println!("    {count:>4}  {reason}");
    }
    println!(
        "  (put this beside the first-blocker baseline from \
         `just t how_often_a_real_file_imports --no-capture`, which this example never re-runs \
         — it only ever asks `probe`'s own question)"
    );
}

/// Deliverable 4: for the five commonest reasons, which other reasons show up
/// in the same file and how often.
fn report_co_occurrence(findings: &[FileFinding]) {
    println!("\n=== 4) co-occurrence of the five commonest reasons ===");
    let mut file_count: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        for reason in &finding.reasons {
            *file_count.entry(reason.as_str()).or_default() += 1;
        }
    }
    let mut ranked: Vec<(&str, usize)> = file_count.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let top_five: Vec<&str> = ranked.iter().take(5).map(|(reason, _)| *reason).collect();

    for reason in &top_five {
        let mut alongside: BTreeMap<&str, usize> = BTreeMap::new();
        let mut total = 0;
        for finding in findings {
            if !finding.reasons.contains(*reason) {
                continue;
            }
            total += 1;
            for other in &finding.reasons {
                if other != reason {
                    *alongside.entry(other.as_str()).or_default() += 1;
                }
            }
        }
        println!("  \"{reason}\" — {total} files:");
        let mut sorted: Vec<(&str, usize)> = alongside.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        if sorted.is_empty() {
            println!("    never with another reason in the same file");
        }
        for (other, count) in sorted {
            println!("    alongside {count:>4}x  {other}");
        }
    }
}

/// Deliverable 5: the fatal bucket, by reason and by file count, kept apart
/// from every other number in this report.
fn report_fatal_bucket(findings: &[FileFinding]) {
    println!("\n=== 5) fatal bucket (the walk could not keep going past these at all) ===");
    let mut by_reason: BTreeMap<&str, usize> = BTreeMap::new();
    let mut files_with_any_fatal = 0;
    for finding in findings {
        if !finding.fatal.is_empty() {
            files_with_any_fatal += 1;
        }
        for reason in &finding.fatal {
            *by_reason.entry(reason.as_str()).or_default() += 1;
        }
    }
    println!("  files with at least one fatal reason: {files_with_any_fatal}");
    let mut sorted: Vec<(&str, usize)> = by_reason.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (reason, count) in sorted {
        println!("    {count:>4}  {reason}");
    }
}

/// Whether a reason means "this kernel does not hold this curve, surface or
/// loop type" — the three [`rcad_step::Refusal`] variants named
/// `ASurfaceThisKernelDoesNotHold`, `ACurveThisKernelDoesNotHold` and
/// `ALoopThisReaderDoesNotWalk`, told apart from every other reason by the
/// exact phrase `kind_of` builds for each. `APlacementThisReaderCannotApply`
/// says "cannot apply", not "does not hold", and names a transform rather
/// than a curve/surface/loop type, so it is not in this family.
fn is_unheld_geometry_reason(reason: &str) -> bool {
    reason.starts_with("a surface this kernel does not hold: ")
        || reason.starts_with("a curve this kernel does not hold: ")
        || reason.starts_with("a loop this reader does not walk: ")
}

/// Deliverable 6: §171.3 says the whole B-spline family's worth "cannot be
/// computed from this run" because co-occurrence (deliverable 4) is pairwise,
/// not a union. This answers that question directly: for a nested set of
/// unheld-geometry reasons, how many files have **every** reason they hold —
/// `reasons` and `fatal` both — inside that set. Such a file is fully freed by
/// implementing exactly that set; a file with even one reason outside the set
/// is not freed by it, no matter how many reasons inside the set it also has.
///
/// A file that already imports contributes nothing here — there is nothing
/// left to free — and is filtered out before any set is compared against
/// (`Import::read` agreeing on this, per `probe_one_file`'s assertion).
fn report_family_union(findings: &[FileFinding]) {
    println!("\n=== 6) unheld-geometry family: nested union pricing (§171.3) ===");

    let mut all_family_reasons: BTreeSet<&str> = BTreeSet::new();
    for finding in findings {
        for reason in finding.reasons.iter().chain(finding.fatal.iter()) {
            if is_unheld_geometry_reason(reason) {
                all_family_reasons.insert(reason.as_str());
            }
        }
    }

    let s1: BTreeSet<&str> =
        BTreeSet::from(["a curve this kernel does not hold: B_SPLINE_CURVE_WITH_KNOTS"]);
    let mut s2 = s1.clone();
    s2.insert("a surface this kernel does not hold: B_SPLINE_SURFACE_WITH_KNOTS");
    let mut s3 = s2.clone();
    s3.insert("a surface this kernel does not hold: SURFACE_OF_REVOLUTION");
    let mut s4 = s3.clone();
    s4.insert("a surface this kernel does not hold: BOUNDED_SURFACE");
    let mut s5 = s4.clone();
    s5.insert("a loop this reader does not walk: VERTEX_LOOP");
    let s6 = all_family_reasons;

    println!("  S6 — every unheld-geometry reason this run emitted:");
    for reason in &s6 {
        println!("    {reason}");
    }

    let sets: [(&str, &BTreeSet<&str>); 6] = [
        ("S1 = {B_SPLINE_CURVE_WITH_KNOTS}", &s1),
        ("S2 = S1 + B_SPLINE_SURFACE_WITH_KNOTS", &s2),
        ("S3 = S2 + SURFACE_OF_REVOLUTION", &s3),
        ("S4 = S3 + BOUNDED_SURFACE", &s4),
        ("S5 = S4 + VERTEX_LOOP", &s5),
        ("S6 = every unheld-geometry reason (see list above)", &s6),
    ];

    let blocked_but_reasonless: usize = findings
        .iter()
        .filter(|finding| !finding.imports && finding.reasons.is_empty() && finding.fatal.is_empty())
        .count();
    if blocked_but_reasonless > 0 {
        println!(
            "  ({blocked_but_reasonless} file(s) do not import and yet the probe found no reason \
             and no fatal for them — excluded from every set below; report this, it means probe \
             and Import::read disagree on a file the assertion in probe_one_file did not catch)"
        );
    }

    println!("  files fully freed by holding every reason in the set:");
    for (label, set) in &sets {
        let mut by_family: BTreeMap<&str, usize> = BTreeMap::new();
        for finding in findings {
            if finding.imports || (finding.reasons.is_empty() && finding.fatal.is_empty()) {
                continue;
            }
            let all: Vec<&str> = finding
                .reasons
                .iter()
                .map(String::as_str)
                .chain(finding.fatal.iter().map(String::as_str))
                .collect();
            if all.iter().all(|reason| set.contains(reason)) {
                *by_family.entry(finding.family).or_default() += 1;
            }
        }
        let kicad = by_family.get("kicad").copied().unwrap_or(0);
        let nist = by_family.get("nist").copied().unwrap_or(0);
        println!("    {label}: kicad={kicad:>4}  nist={nist:>3}  combined={:>4}", kicad + nist);
    }

    println!(
        "\n  after S6 alone is implemented, the non-family reasons still held by files S6 does \
         not fully free:"
    );
    let mut leftover_histogram: BTreeMap<usize, usize> = BTreeMap::new();
    for finding in findings {
        if finding.imports {
            continue;
        }
        let non_family_count = finding
            .reasons
            .iter()
            .chain(finding.fatal.iter())
            .filter(|reason| !s6.contains(reason.as_str()))
            .count();
        if non_family_count > 0 {
            *leftover_histogram.entry(non_family_count).or_default() += 1;
        }
    }
    for (count, files) in &leftover_histogram {
        println!("    {count:>3} other reason(s) remaining: {files:>4} files");
    }
}
