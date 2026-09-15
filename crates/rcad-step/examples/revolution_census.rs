//! How many corpus files use `SURFACE_OF_REVOLUTION` or
//! `SURFACE_OF_LINEAR_EXTRUSION` — both refused by name in `geometry.rs` —
//! for a profile curve this kernel already holds exactly?
//!
//! A `LINE` swept about an axis is a cylinder, a cone, or a plane, depending
//! on how the line sits against the axis. A `CIRCLE` swept about an axis is a
//! sphere or a torus, depending on where its centre sits. This walks every
//! file in the corpus at parse level — `StepFile`'s own graph, never
//! `Import`, which stops at its first refusal — and counts which of those
//! outcomes each instance lands on.
//!
//! This produces numbers, not a fix, and a count of files a blocker stops
//! first is an upper bound on what removing it is worth, never a projection
//! (`docs/decisions.md` §45.2).
//!
//! The answer, at the tree this landed on: **all 50 `SURFACE_OF_REVOLUTION`
//! instances in the corpus have a `B_SPLINE_CURVE_WITH_KNOTS` profile**, and
//! none is a `LINE` or a `CIRCLE`. No surface the kernel already holds is
//! hiding behind that refusal. KiCad's exporter writes revolved fillet
//! profiles as degree-6 clamped Béziers, which is why.
//!
//! Run with:
//! ```text
//! RCAD_STEP_CORPUS=<path> mise exec -- cargo run -p rcad-step --release \
//!     --example revolution_census
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_geom::{Point, Vector};
use rcad_step::{EntityId, Instance, StepFile, Value};

/// How small a number with no unit — a sine, a cosine, or one length divided
/// by another — has to be before this calls it zero.
///
/// Every test here is a handful of arithmetic steps (a cross product, a dot
/// product, one division) on unit vectors, so accumulated rounding stays near
/// `1e-15`. `1e-9` sits six orders above that — nowhere near what rounding
/// alone produces — and nine orders below a real geometric offset in a
/// millimetre-scale part, so it separates the two without being fooled by
/// either.
const DECIDE: f64 = 1e-9;

/// Above [`DECIDE`] but below this is reported as a near miss and never
/// folded into a decision either way.
const NEAR_MISS: f64 = 1e-6;

fn main() {
    let root = corpus_root();
    let mut report = Report::default();
    for source in ["nist", "kicad"] {
        for path in files_under(&root.join(source)) {
            census_one_file(&path, &mut report);
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
    found.sort();
    found
}

#[derive(Default)]
struct Report {
    files_scanned: usize,
    files_failed_to_parse: Vec<PathBuf>,
    revolutions: Vec<Sweep>,
    extrusions: Vec<Extrusion>,
}

/// One `SURFACE_OF_REVOLUTION`, and what its profile curve and axis worked
/// out to.
struct Sweep {
    file: PathBuf,
    surface_id: EntityId,
    curve_label: String,
    profile_keyword: String,
    axis_id: EntityId,
    outcome: Outcome,
}

enum Outcome {
    Cylinder {
        sin_to_axis: f64,
        radius: f64,
        exact: bool,
    },
    Plane {
        cos_to_axis: f64,
        exact: bool,
    },
    Cone {
        angle_degrees: f64,
        offset_ratio: f64,
        exact: bool,
    },
    Sphere {
        offset_ratio: f64,
        exact: bool,
    },
    Torus {
        offset_ratio: f64,
        offset: f64,
        radius: f64,
    },
    /// Neither parallel nor perpendicular to the axis, and not meeting it
    /// either: sweeping this line around the axis gives a hyperboloid of one
    /// sheet, which is none of the five surfaces this kernel holds.
    SkewLine {
        offset_ratio: f64,
    },
    /// Parallel to the axis and sitting on it: a zero-radius cylinder, which
    /// is a line, not a surface.
    DegenerateOnAxis,
    OtherProfile(String),
    Unresolvable(String),
}

impl Outcome {
    fn is_a_held_kind(&self) -> bool {
        matches!(
            self,
            Outcome::Cylinder { .. }
                | Outcome::Plane { .. }
                | Outcome::Cone { .. }
                | Outcome::Sphere { .. }
                | Outcome::Torus { .. }
        )
    }

    fn describe(&self) -> String {
        match self {
            Outcome::Cylinder {
                sin_to_axis,
                radius,
                exact,
            } => format!(
                "cylinder (line parallel to axis, sin={sin_to_axis:.3e}{}; radius {radius:.6})",
                exactness_note(*exact)
            ),
            Outcome::Plane { cos_to_axis, exact } => format!(
                "plane (line perpendicular to axis, cos={cos_to_axis:.3e}{})",
                exactness_note(*exact)
            ),
            Outcome::Cone {
                angle_degrees,
                offset_ratio,
                exact,
            } => format!(
                "cone (line meets axis at {angle_degrees:.3} degrees, offset ratio \
                 {offset_ratio:.3e}{})",
                exactness_note(*exact)
            ),
            Outcome::Sphere {
                offset_ratio,
                exact,
            } => format!(
                "sphere (circle centre on axis, offset ratio {offset_ratio:.3e}{})",
                exactness_note(*exact)
            ),
            Outcome::Torus {
                offset_ratio,
                offset,
                radius,
            } => format!(
                "torus (circle centre off axis by {offset:.6}, ratio {offset_ratio:.3e}; \
                 circle radius {radius:.6})"
            ),
            Outcome::SkewLine { offset_ratio } => format!(
                "UNCLASSIFIABLE: line is skew to the axis (offset ratio {offset_ratio:.3e}) \
                 — a hyperboloid of one sheet, not one of the five held kinds"
            ),
            Outcome::DegenerateOnAxis => {
                "UNCLASSIFIABLE: line lies on the axis itself — zero-radius, degenerate".to_owned()
            }
            Outcome::OtherProfile(keyword) => format!("profile is {keyword}, not LINE or CIRCLE"),
            Outcome::Unresolvable(reason) => format!("UNRESOLVABLE: {reason}"),
        }
    }
}

fn exactness_note(exact: bool) -> &'static str {
    if exact {
        ", exactly"
    } else {
        ", within rounding"
    }
}

/// One `SURFACE_OF_LINEAR_EXTRUSION`, reported separately from the
/// revolutions per the task's own instruction.
struct Extrusion {
    file: PathBuf,
    surface_id: EntityId,
    curve_label: String,
    outcome: ExtrusionOutcome,
}

enum ExtrusionOutcome {
    /// A line extruded is a plane.
    Plane,
    /// A circle extruded is a cylinder.
    Cylinder,
    OtherProfile(String),
    Unresolvable(String),
}

impl ExtrusionOutcome {
    fn is_a_held_kind(&self) -> bool {
        matches!(self, ExtrusionOutcome::Plane | ExtrusionOutcome::Cylinder)
    }

    fn describe(&self) -> String {
        match self {
            ExtrusionOutcome::Plane => "plane (LINE extruded)".to_owned(),
            ExtrusionOutcome::Cylinder => "cylinder (CIRCLE extruded)".to_owned(),
            ExtrusionOutcome::OtherProfile(keyword) => {
                format!("profile is {keyword}, not LINE or CIRCLE")
            }
            ExtrusionOutcome::Unresolvable(reason) => format!("UNRESOLVABLE: {reason}"),
        }
    }
}

fn census_one_file(path: &Path, report: &mut Report) {
    let bytes = fs::read(path).expect("a file the walk just listed");
    let file = match StepFile::read(&bytes) {
        Ok(file) => file,
        Err(_) => {
            report.files_failed_to_parse.push(path.to_path_buf());
            return;
        }
    };
    report.files_scanned += 1;

    for instance in file.instances_named("SURFACE_OF_REVOLUTION") {
        report
            .revolutions
            .push(census_revolution(path, &file, instance));
    }
    for instance in file.instances_named("SURFACE_OF_LINEAR_EXTRUSION") {
        report
            .extrusions
            .push(census_extrusion(path, &file, instance));
    }
}

fn census_revolution(path: &Path, file: &StepFile, instance: &Instance) -> Sweep {
    let surface_id = instance.id();
    let record = instance
        .part("SURFACE_OF_REVOLUTION")
        .expect("the named iterator only yields instances that hold this part");
    let curve_id = record.attribute(1).and_then(Value::as_reference);
    let axis_id = record.attribute(2).and_then(Value::as_reference);

    let Some(curve_id) = curve_id else {
        return unresolvable_sweep(
            path,
            surface_id,
            axis_id.unwrap_or(surface_id),
            "SURFACE_OF_REVOLUTION has no swept-curve reference".to_owned(),
        );
    };
    let Some(axis_id) = axis_id else {
        return unresolvable_sweep(
            path,
            surface_id,
            surface_id,
            "SURFACE_OF_REVOLUTION has no axis reference".to_owned(),
        );
    };

    let profile = resolve_profile(file, curve_id);
    let curve_label = profile.label();
    let profile_keyword = profile.terminal.keyword();

    let outcome = match resolve_axis(file, axis_id) {
        Err(reason) => Outcome::Unresolvable(reason),
        Ok(axis) => match &profile.terminal {
            Terminal::Line(line_id) => match resolve_line(file, *line_id) {
                Ok(line) => classify_line(&line, &axis),
                Err(reason) => Outcome::Unresolvable(reason),
            },
            Terminal::Circle(circle_id) => match resolve_circle(file, *circle_id) {
                Ok(circle) => classify_circle(&circle, &axis),
                Err(reason) => Outcome::Unresolvable(reason),
            },
            Terminal::Other(keyword) => Outcome::OtherProfile(keyword.clone()),
            Terminal::Dangling => {
                Outcome::Unresolvable(format!("swept curve {curve_id} is not defined"))
            }
        },
    };

    Sweep {
        file: path.to_path_buf(),
        surface_id,
        curve_label,
        profile_keyword,
        axis_id,
        outcome,
    }
}

fn unresolvable_sweep(
    path: &Path,
    surface_id: EntityId,
    axis_id: EntityId,
    reason: String,
) -> Sweep {
    Sweep {
        file: path.to_path_buf(),
        surface_id,
        curve_label: "<malformed>".to_owned(),
        profile_keyword: "<malformed>".to_owned(),
        axis_id,
        outcome: Outcome::Unresolvable(reason),
    }
}

fn census_extrusion(path: &Path, file: &StepFile, instance: &Instance) -> Extrusion {
    let surface_id = instance.id();
    let record = instance
        .part("SURFACE_OF_LINEAR_EXTRUSION")
        .expect("the named iterator only yields instances that hold this part");
    let curve_id = record.attribute(1).and_then(Value::as_reference);

    let Some(curve_id) = curve_id else {
        return Extrusion {
            file: path.to_path_buf(),
            surface_id,
            curve_label: "<malformed>".to_owned(),
            outcome: ExtrusionOutcome::Unresolvable(
                "SURFACE_OF_LINEAR_EXTRUSION has no swept-curve reference".to_owned(),
            ),
        };
    };

    let profile = resolve_profile(file, curve_id);
    let curve_label = profile.label();
    let outcome = match &profile.terminal {
        Terminal::Line(_) => ExtrusionOutcome::Plane,
        Terminal::Circle(_) => ExtrusionOutcome::Cylinder,
        Terminal::Other(keyword) => ExtrusionOutcome::OtherProfile(keyword.clone()),
        Terminal::Dangling => {
            ExtrusionOutcome::Unresolvable(format!("swept curve {curve_id} is not defined"))
        }
    };

    Extrusion {
        file: path.to_path_buf(),
        surface_id,
        curve_label,
        outcome,
    }
}

/// What a swept-curve reference ultimately names, unwrapping `TRIMMED_CURVE`
/// on the way — a trim is not itself a profile shape.
enum Terminal {
    Line(EntityId),
    Circle(EntityId),
    Other(String),
    Dangling,
}

impl Terminal {
    fn keyword(&self) -> String {
        match self {
            Terminal::Line(_) => "LINE".to_owned(),
            Terminal::Circle(_) => "CIRCLE".to_owned(),
            Terminal::Other(keyword) => keyword.clone(),
            Terminal::Dangling => "<dangling>".to_owned(),
        }
    }
}

struct Profile {
    trace: Vec<String>,
    terminal: Terminal,
}

impl Profile {
    fn label(&self) -> String {
        self.trace.join(" -> ")
    }
}

fn resolve_profile(file: &StepFile, mut id: EntityId) -> Profile {
    let mut trace = Vec::new();
    loop {
        let Some(instance) = file.instance(id) else {
            trace.push(format!("{id} (undefined)"));
            return Profile {
                trace,
                terminal: Terminal::Dangling,
            };
        };
        if let Some(basis_id) = instance
            .part("TRIMMED_CURVE")
            .and_then(|record| record.attribute(1))
            .and_then(Value::as_reference)
        {
            trace.push("TRIMMED_CURVE".to_owned());
            id = basis_id;
            continue;
        }
        if instance.part("LINE").is_some() {
            trace.push("LINE".to_owned());
            return Profile {
                trace,
                terminal: Terminal::Line(id),
            };
        }
        if instance.part("CIRCLE").is_some() {
            trace.push("CIRCLE".to_owned());
            return Profile {
                trace,
                terminal: Terminal::Circle(id),
            };
        }
        let keyword = instance.parts()[0].keyword().to_owned();
        trace.push(keyword.clone());
        return Profile {
            trace,
            terminal: Terminal::Other(keyword),
        };
    }
}

struct Axis {
    origin: Point,
    direction: Vector,
}

fn resolve_axis(file: &StepFile, id: EntityId) -> Result<Axis, String> {
    let instance = file
        .instance(id)
        .ok_or_else(|| format!("axis {id} is not defined"))?;
    let record = instance.part("AXIS1_PLACEMENT").ok_or_else(|| {
        format!(
            "axis {id} is not an AXIS1_PLACEMENT (it is {})",
            instance.parts()[0].keyword()
        )
    })?;
    let location_id = record
        .attribute(1)
        .and_then(Value::as_reference)
        .ok_or_else(|| format!("axis {id} has no location reference"))?;
    let origin = cartesian_point(file, location_id)
        .ok_or_else(|| format!("axis {id}'s location {location_id} is not a readable point"))?;
    let raw_direction = match record.attribute(2) {
        Some(Value::Reference(direction_id)) => {
            direction(file, *direction_id).ok_or_else(|| {
                format!("axis {id}'s direction {direction_id} is not a readable direction")
            })?
        }
        _ => Vector::new(0.0, 0.0, 1.0),
    };
    let direction = raw_direction
        .normalized()
        .map_err(|why| format!("axis {id}'s direction is not usable: {why:?}"))?;
    Ok(Axis { origin, direction })
}

struct Line {
    point: Point,
    direction: Vector,
}

fn resolve_line(file: &StepFile, id: EntityId) -> Result<Line, String> {
    let record = file
        .instance(id)
        .and_then(|instance| instance.part("LINE"))
        .ok_or_else(|| format!("{id} is not a readable LINE"))?;
    let point_id = record
        .attribute(1)
        .and_then(Value::as_reference)
        .ok_or_else(|| format!("LINE {id} has no point reference"))?;
    let point = cartesian_point(file, point_id)
        .ok_or_else(|| format!("LINE {id}'s point {point_id} is not a readable point"))?;
    let vector_id = record
        .attribute(2)
        .and_then(Value::as_reference)
        .ok_or_else(|| format!("LINE {id} has no direction reference"))?;
    let raw_direction = resolve_vector_direction(file, vector_id).ok_or_else(|| {
        format!("LINE {id}'s direction {vector_id} is not a readable VECTOR or DIRECTION")
    })?;
    let direction = raw_direction
        .normalized()
        .map_err(|why| format!("LINE {id}'s direction is not usable: {why:?}"))?;
    Ok(Line { point, direction })
}

fn resolve_vector_direction(file: &StepFile, id: EntityId) -> Option<Vector> {
    let instance = file.instance(id)?;
    if let Some(record) = instance.part("VECTOR") {
        let direction_id = record.attribute(1).and_then(Value::as_reference)?;
        return direction(file, direction_id);
    }
    if instance.part("DIRECTION").is_some() {
        return direction(file, id);
    }
    None
}

struct Circle {
    center: Point,
    radius: f64,
}

fn resolve_circle(file: &StepFile, id: EntityId) -> Result<Circle, String> {
    let record = file
        .instance(id)
        .and_then(|instance| instance.part("CIRCLE"))
        .ok_or_else(|| format!("{id} is not a readable CIRCLE"))?;
    let placement_id = record
        .attribute(1)
        .and_then(Value::as_reference)
        .ok_or_else(|| format!("CIRCLE {id} has no placement reference"))?;
    let placement_record = file
        .instance(placement_id)
        .and_then(|instance| instance.part("AXIS2_PLACEMENT_3D"))
        .ok_or_else(|| {
            format!("CIRCLE {id}'s placement {placement_id} is not an AXIS2_PLACEMENT_3D")
        })?;
    let location_id = placement_record
        .attribute(1)
        .and_then(Value::as_reference)
        .ok_or_else(|| format!("CIRCLE {id}'s placement has no location reference"))?;
    let center = cartesian_point(file, location_id)
        .ok_or_else(|| format!("CIRCLE {id}'s centre {location_id} is not a readable point"))?;
    let radius = record
        .attribute(2)
        .and_then(as_f64)
        .ok_or_else(|| format!("CIRCLE {id} has no readable radius"))?;
    Ok(Circle { center, radius })
}

fn cartesian_point(file: &StepFile, id: EntityId) -> Option<Point> {
    let record = file.instance(id)?.part("CARTESIAN_POINT")?;
    let [x, y, z] = three_reals(record.attribute(1)?)?;
    Some(Point::new(x, y, z))
}

fn direction(file: &StepFile, id: EntityId) -> Option<Vector> {
    let record = file.instance(id)?.part("DIRECTION")?;
    let [x, y, z] = three_reals(record.attribute(1)?)?;
    Some(Vector::new(x, y, z))
}

fn three_reals(value: &Value) -> Option<[f64; 3]> {
    let list = value.as_list()?;
    let [a, b, c] = list else {
        return None;
    };
    Some([as_f64(a)?, as_f64(b)?, as_f64(c)?])
}

fn as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Real(number) => Some(*number),
        Value::Integer(whole) => Some(as_f64_lossy(*whole)),
        Value::Typed { value, .. } => as_f64(value),
        Value::Text(_)
        | Value::Enumeration(_)
        | Value::Reference(_)
        | Value::List(_)
        | Value::Nothing
        | Value::Derived
        | _ => None,
    }
}

fn as_f64_lossy(whole: i64) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a measuring script's coordinates, not a certified conversion; \
                  files this corpus holds never write a length as an integer \
                  past the range an f64 carries exactly"
    )]
    let converted = whole as f64;
    converted
}

/// Is `direction` parallel to the axis (a cylinder), perpendicular to it (a
/// plane), or does it meet the axis at some other angle (a cone)?
///
/// Parallel: the cross product of the two unit directions has near-zero
/// length — `sin` of the angle between them is near zero. Perpendicular: the
/// dot product is near zero — `cos` of the angle is near zero. Otherwise the
/// two lines (the swept line, and the axis) are checked for whether they
/// actually meet: the scalar triple product of the offset between them and
/// their common perpendicular gives the signed distance between the two
/// lines as lines in space. Near zero, they meet, and revolving gives a cone.
/// Away from zero, they are skew, and revolving gives a hyperboloid of one
/// sheet, which this kernel does not hold.
fn classify_line(line: &Line, axis: &Axis) -> Outcome {
    let sin_to_axis = line.direction.cross(axis.direction).length();
    let cos_to_axis = line.direction.dot(axis.direction);

    if sin_to_axis <= DECIDE {
        let offset = line.point - axis.origin;
        let along_axis = offset.dot(axis.direction);
        let radius = (offset - axis.direction * along_axis).length();
        let scale = offset.length().max(1.0);
        if radius / scale <= DECIDE {
            return Outcome::DegenerateOnAxis;
        }
        return Outcome::Cylinder {
            sin_to_axis,
            radius,
            exact: sin_to_axis == 0.0,
        };
    }

    if cos_to_axis.abs() <= DECIDE {
        return Outcome::Plane {
            cos_to_axis,
            exact: cos_to_axis == 0.0,
        };
    }

    let common_perpendicular = line
        .direction
        .cross(axis.direction)
        .normalized()
        .expect("sin_to_axis was checked above DECIDE, so this cross product is not near zero");
    let offset = line.point - axis.origin;
    let signed_distance = offset.dot(common_perpendicular);
    let scale = offset.length().max(1.0);
    let offset_ratio = (signed_distance / scale).abs();
    let angle_degrees = sin_to_axis.atan2(cos_to_axis).to_degrees();

    if offset_ratio <= DECIDE {
        Outcome::Cone {
            angle_degrees,
            offset_ratio,
            exact: signed_distance == 0.0,
        }
    } else {
        Outcome::SkewLine { offset_ratio }
    }
}

/// Does the circle's centre sit on the axis (a sphere) or off it (a torus)?
///
/// The centre's distance to the axis line, divided by a length scale built
/// from the circle's own radius and its distance from the axis origin, is
/// compared against [`DECIDE`] the same way the line tests are.
fn classify_circle(circle: &Circle, axis: &Axis) -> Outcome {
    let offset = circle.center - axis.origin;
    let along_axis = offset.dot(axis.direction);
    let radius_to_axis = (offset - axis.direction * along_axis).length();
    let scale = circle.radius.abs().max(offset.length()).max(1.0);
    let offset_ratio = radius_to_axis / scale;

    if offset_ratio <= DECIDE {
        Outcome::Sphere {
            offset_ratio,
            exact: radius_to_axis == 0.0,
        }
    } else {
        Outcome::Torus {
            offset_ratio,
            offset: radius_to_axis,
            radius: circle.radius,
        }
    }
}

impl Report {
    fn print(&self) {
        println!("files parsed: {}", self.files_scanned);
        if !self.files_failed_to_parse.is_empty() {
            println!(
                "files that failed to parse: {}",
                self.files_failed_to_parse.len()
            );
            for path in &self.files_failed_to_parse {
                println!("  {}", path.display());
            }
        }

        self.print_inventory();
        self.print_histogram();
        self.print_line_breakdown();
        self.print_circle_breakdown();
        self.print_rollup();
        self.print_extrusions();
    }

    fn print_inventory(&self) {
        println!("\n== 1. Every SURFACE_OF_REVOLUTION instance ==");
        for sweep in &self.revolutions {
            println!(
                "{}  {}  curve={} ({})  axis={}  -> {}",
                sweep.file.display(),
                sweep.surface_id,
                sweep.curve_label,
                sweep.profile_keyword,
                sweep.axis_id,
                sweep.outcome.describe()
            );
        }
        println!(
            "total SURFACE_OF_REVOLUTION instances: {}",
            self.revolutions.len()
        );
    }

    fn print_histogram(&self) {
        println!("\n== 2. Profile keyword histogram ==");
        let mut histogram: BTreeMap<&str, usize> = BTreeMap::new();
        for sweep in &self.revolutions {
            *histogram.entry(sweep.profile_keyword.as_str()).or_insert(0) += 1;
        }
        for (keyword, count) in &histogram {
            println!("{keyword}: {count}");
        }
    }

    fn print_line_breakdown(&self) {
        println!("\n== 3. LINE profiles ==");
        let lines: Vec<&Sweep> = self
            .revolutions
            .iter()
            .filter(|sweep| sweep.profile_keyword == "LINE")
            .collect();
        println!("total LINE profiles: {}", lines.len());
        for bucket in [
            "Cylinder (parallel)",
            "Plane (perpendicular)",
            "Cone (meets axis)",
            "SkewLine (unclassifiable)",
            "DegenerateOnAxis (unclassifiable)",
            "Unresolvable",
        ] {
            let count = lines
                .iter()
                .filter(|sweep| {
                    matches!(
                        (bucket, &sweep.outcome),
                        ("Cylinder (parallel)", Outcome::Cylinder { .. })
                            | ("Plane (perpendicular)", Outcome::Plane { .. })
                            | ("Cone (meets axis)", Outcome::Cone { .. })
                            | ("SkewLine (unclassifiable)", Outcome::SkewLine { .. })
                            | (
                                "DegenerateOnAxis (unclassifiable)",
                                Outcome::DegenerateOnAxis
                            )
                            | ("Unresolvable", Outcome::Unresolvable(_))
                    )
                })
                .count();
            println!("  {bucket}: {count}");
        }
        print_near_misses("line-vs-axis", &lines);
    }

    fn print_circle_breakdown(&self) {
        println!("\n== 4. CIRCLE profiles ==");
        let circles: Vec<&Sweep> = self
            .revolutions
            .iter()
            .filter(|sweep| sweep.profile_keyword == "CIRCLE")
            .collect();
        println!("total CIRCLE profiles: {}", circles.len());
        for bucket in [
            "Sphere (centre on axis)",
            "Torus (centre off axis)",
            "Unresolvable",
        ] {
            let count = circles
                .iter()
                .filter(|sweep| {
                    matches!(
                        (bucket, &sweep.outcome),
                        ("Sphere (centre on axis)", Outcome::Sphere { .. })
                            | ("Torus (centre off axis)", Outcome::Torus { .. })
                            | ("Unresolvable", Outcome::Unresolvable(_))
                    )
                })
                .count();
            println!("  {bucket}: {count}");
        }
        print_near_misses("circle-centre-vs-axis", &circles);
    }

    fn print_rollup(&self) {
        println!("\n== 5. Files fully classifiable into the five held kinds ==");
        let mut per_file: BTreeMap<&Path, Vec<&Sweep>> = BTreeMap::new();
        for sweep in &self.revolutions {
            per_file
                .entry(sweep.file.as_path())
                .or_default()
                .push(sweep);
        }
        let mut fully_classifiable: Vec<&Path> = Vec::new();
        let mut not_fully: Vec<&Path> = Vec::new();
        for (file, sweeps) in &per_file {
            if sweeps.iter().all(|sweep| sweep.outcome.is_a_held_kind()) {
                fully_classifiable.push(file);
            } else {
                not_fully.push(file);
            }
        }
        println!(
            "files with at least one SURFACE_OF_REVOLUTION: {}",
            per_file.len()
        );
        println!(
            "of those, every SURFACE_OF_REVOLUTION classifiable into a held kind: {}",
            fully_classifiable.len()
        );
        for file in &fully_classifiable {
            println!("  {}", file.display());
        }
        println!(
            "of those, at least one SURFACE_OF_REVOLUTION not classifiable: {}",
            not_fully.len()
        );
        for file in &not_fully {
            println!("  {}", file.display());
        }
    }

    fn print_extrusions(&self) {
        println!("\n== 6. SURFACE_OF_LINEAR_EXTRUSION (secondary, kept separate) ==");
        println!(
            "total SURFACE_OF_LINEAR_EXTRUSION instances: {}",
            self.extrusions.len()
        );
        for extrusion in &self.extrusions {
            println!(
                "{}  {}  curve={}  -> {}",
                extrusion.file.display(),
                extrusion.surface_id,
                extrusion.curve_label,
                extrusion.outcome.describe()
            );
        }
        let classifiable = self
            .extrusions
            .iter()
            .filter(|extrusion| extrusion.outcome.is_a_held_kind())
            .count();
        println!("classifiable into a held kind: {classifiable}");
    }
}

fn print_near_misses(what: &str, sweeps: &[&Sweep]) {
    let ratios: Vec<f64> = sweeps
        .iter()
        .filter_map(|sweep| match &sweep.outcome {
            Outcome::Cylinder { sin_to_axis, .. } => Some(*sin_to_axis),
            Outcome::Plane { cos_to_axis, .. } => Some(cos_to_axis.abs()),
            Outcome::Cone { offset_ratio, .. } | Outcome::SkewLine { offset_ratio } => {
                Some(*offset_ratio)
            }
            Outcome::Sphere { offset_ratio, .. } | Outcome::Torus { offset_ratio, .. } => {
                Some(*offset_ratio)
            }
            Outcome::DegenerateOnAxis | Outcome::OtherProfile(_) | Outcome::Unresolvable(_) => None,
        })
        .filter(|ratio| *ratio > DECIDE && *ratio <= NEAR_MISS)
        .collect();
    println!(
        "  near misses for {what} (ratio between {DECIDE:e} and {NEAR_MISS:e}, decided \
         the \"not\" way but close): {}",
        ratios.len()
    );
    for ratio in &ratios {
        println!("    {ratio:e}");
    }
}
