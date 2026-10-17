//! §42.12 criterion 10: the import rate, family by family and never blended,
//! with the reason every refusal gave.
//!
//! Two corpora and a control, and the three are three different questions.
//!
//! - **The control** is files this test writes itself, out of the shapes this
//!   kernel already builds. It has to come in at **100 %**. It is what makes the
//!   other two numbers mean anything: a rate with no control cannot tell a
//!   reader that refuses real files from a reader that refuses everything.
//! - **`nist/`** — 44 ASME Y14.5 GD&T certification parts, built to exercise
//!   every surface a CAD system might mis-tolerance.
//! - **`kicad/`** — 323 connector and switch housings out of KiCad's own
//!   footprint library.
//!
//! The two real corpora disagree by a factor of eight about what a real file
//! holds (§42's spike) and are never added together. Neither is in the tree;
//! point `RCAD_STEP_CORPUS` at a directory holding `nist/` and `kicad/` and this
//! runs. Without it, the corpus half **fails** rather than skips — green has
//! to mean the corpus ran (decisions.md §44.10) — while the control half,
//! which needs no corpus, still runs either way.

mod written_by_hand;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_geom::{Deflection, EdgeRun, Plane, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile, Value};
use written_by_hand::{a_block, a_quarter_disc_plate};

/// Every file this kernel wrote itself imports, and the model it makes is
/// sound.
///
/// A block and a quarter-disc plate, each at eleven sizes over eleven orders of
/// magnitude and in each of the four units this kernel names by an `SI_UNIT`
/// prefix. The plate is the half that matters: it has a cylindrical face, two
/// arcs and the half-spaces that say which arc, which is the whole of what a
/// block cannot exercise.
///
/// **One refusal here is one bug**, and it is what makes the two rates below
/// mean anything: a rate with no control cannot tell a reader that refuses real
/// files from a reader that refuses everything (§42.12 criterion 10).
///
/// The plate is **measured** as well as swept, because a sound solid and the
/// solid the file describes are two claims and the sweep only makes the first.
/// A quarter-disc of radius `r` and height `h` encloses `πr²h/4`, which is a
/// number nothing in this kernel supplied.
#[test]
fn every_file_this_kernel_wrote_itself_imports() {
    let mut written = 0;
    for power in -5..6 {
        let size = f64::from(2.0_f32).powi(power);
        for unit in ["$", ".MILLI.", ".CENTI.", ".MICRO."] {
            for (what, text) in [
                ("a block", a_block([size, size * 2.0, size * 4.0], unit)),
                ("a plate", a_quarter_disc_plate(size, size * 2.0, unit)),
            ] {
                let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
                let mut log = ProvenanceLog::new();
                let import = Import::read(&file, &Policy::certified(), &mut log)
                    .unwrap_or_else(|why| panic!("{what} of size {size} in {unit}: {why}"));
                import
                    .model()
                    .validate(&Policy::certified(), &mut log)
                    .expect("and every imported model validates under a certified policy");
                if what == "a plate" {
                    let [(_, key)] = import.solids() else {
                        panic!("one MANIFOLD_SOLID_BREP is one solid");
                    };
                    let measured = import
                        .model()
                        .solid(*key)
                        .expect("this model made the key")
                        .measure(
                            Deflection::for_scale(import.model().scale()),
                            &Policy::certified(),
                            &mut log,
                        )
                        .expect("a plate measures");
                    let space = core::f64::consts::PI * size * size * (size * 2.0) / 4.0;
                    assert!(
                        (measured.volume() - space).abs() <= measured.volume_error(),
                        "a plate of radius {size} encloses {space} and this one says {} give \
                         or take {}",
                        measured.volume(),
                        measured.volume_error()
                    );
                }
                written += 1;
            }
        }
    }
    println!("control: {written} / {written} imported");
    assert_eq!(
        written, 88,
        "the control must not shrink without somebody saying so"
    );
}

/// A build stopped for one reason is published as **that** reason, so the
/// `miss=` column above measures the corner of the refusal beside it.
///
/// Two things are broken in one plate, and the point is which one gets
/// published. The pipe is widened by a nanometre, which leaves a corner
/// provably off a curved face for anything scanning for one; and the bottom
/// face's plane is turned to face along `y`, which the sweep reaches first —
/// `check_faces` walks the bottom face before the round one — and stops at.
///
/// Before `arms0` the mapping scanned every curved face of any refused build
/// whatever the build had said, so this file was published as a corner off a
/// pipe, with a `miss=` measuring a corner that had nothing to do with why the
/// build failed. The first half of the test is the control that keeps the trap
/// baited: take the turned plane away and the corner off the pipe really is
/// what stops it.
#[test]
fn a_build_stopped_by_a_leaning_face_is_not_published_as_a_corner_off_a_pipe() {
    let widened = one_line_changed(
        &a_quarter_disc_plate(2.0, 4.0, ".MILLI."),
        "#774=CYLINDRICAL_SURFACE('',#768,2.0);",
        "#774=CYLINDRICAL_SURFACE('',#768,2.000000001);",
    );
    let what_the_widening_alone_does = refusal_for(&widened);
    assert!(
        matches!(
            what_the_widening_alone_does,
            Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn { .. }
        ),
        "the widened pipe has to be a corner off a curved face on its own, or this proves \
         nothing about which of two reasons wins: {what_the_widening_alone_does}"
    );

    let leaning = one_line_changed(
        &widened,
        "#758=DIRECTION('',(0.0,0.0,-1.0));",
        "#758=DIRECTION('',(0.0,1.0,0.0));",
    );
    let Refusal::TheSolidTheFileDescribesIsNotOne { why, .. } = refusal_for(&leaning) else {
        panic!("a corner off its face's plane is the sweep's refusal about the whole solid");
    };
    assert!(
        matches!(
            &*why,
            rcad_topo::Error::Invalid(rcad_topo::Invalid::VertexOffItsFacePlane { .. })
        ),
        "the sweep stopped at the turned plane and the refusal has to say so: {why}"
    );
}

/// `text` with `from` swapped for `to`, and a panic if `from` was not there.
///
/// `str::replace` on a line that has moved does nothing and says nothing, which
/// would leave a test passing on a file it did not mean to write.
fn one_line_changed(text: &str, from: &str, to: &str) -> String {
    assert!(
        text.contains(from),
        "this fixture no longer holds {from}, so the test is breaking something else"
    );
    text.replace(from, to)
}

/// What the mapping refused a hand-written file for, and a panic if it imports.
fn refusal_for(text: &str) -> Refusal {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    match Import::read(&file, &Policy::certified(), &mut log) {
        Ok(import) => panic!(
            "this was supposed to be refused, and instead it imported {} solids",
            import.solids().len()
        ),
        Err(why) => why,
    }
}

/// A real file measures what its own name says it should.
///
/// **The independent figure this lane needs, and it is not this kernel's.**
/// Every other check here compares one measurement with another; this one
/// compares a measurement against a number KiCad's own library wrote into the
/// file name before anybody ran anything. A `PinHeader_2x15_P2.54mm` is fifteen
/// pitches of 2.54 mm long and two across, and a
/// `LED_Rectangular_W5.0mm_H2.0mm` is five by two — so a mapping that read a
/// coordinate as a radius, or a millimetre as an inch, or dropped a face, would
/// come out with a part the wrong size and say so here.
///
/// A nanometre of slack, which is a hundredth of what these files declare their
/// own uncertainty to be and a ten-millionth of the smallest thing they hold.
#[test]
fn a_real_file_measures_what_its_own_name_says() {
    let root = corpus();
    let mut checked = 0;
    let mut dip_switches: Vec<(String, u32, f64)> = Vec::new();
    for path in files_under(&root.join("kicad")) {
        let claims = what_the_name_says(&name_of(&path));
        let switches = how_many_dip_switches(&name_of(&path));
        if claims.is_empty() && switches.is_none() {
            continue;
        }
        let bytes = fs::read(&path).expect("a file the walk just listed");
        let file = StepFile::read(&bytes).expect("every file in the corpus parses");
        let mut log = ProvenanceLog::new();
        let Ok(import) = Import::read(&file, &Policy::certified(), &mut log) else {
            continue;
        };
        let across = how_big_it_is(&import);
        if let Some(switches) = switches {
            let family = which_dip_switch_family(&name_of(&path))
                .expect("a name the switch count was read out of has a family too");
            dip_switches.push((family, switches, across[0]));
        }
        for (axis, expected) in claims {
            assert!(
                (across[axis] - expected).abs() < 1e-9,
                "{} says it is {expected} mm across axis {axis} and it measures {}",
                name_of(&path),
                across[axis]
            );
            checked += 1;
        }
    }
    // A DIP switch's name says how many switches it has and what pitch they sit
    // at, and says nothing about how much housing there is either side. So what
    // its family checks is the **difference**: one more switch is one more
    // 2.54 mm, whatever the ends are worth.
    //
    // Within **one family**, and that is the fix rather than the design. Sorting
    // by count alone put two *different* housings with the same count next to
    // each other and demanded they be the same length, which is the one thing
    // the paragraph above says a name does not promise. It went unnoticed while
    // only one housing per count imported.
    dip_switches.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    let mut switch_steps = 0;
    for pair in dip_switches.windows(2) {
        let [(family, fewer, shorter), (next_family, more, longer)] = pair else {
            unreachable!("windows(2) hands back two");
        };
        if family != next_family {
            continue;
        }
        let expected = f64::from(more - fewer) * 2.54;
        assert!(
            (longer - shorter - expected).abs() < 1e-9,
            "a {family} with {more} switches against one with {fewer} should be \
             {expected} mm longer and is {}",
            longer - shorter
        );
        switch_steps += 1;
        checked += 1;
    }

    println!(
        "{checked} dimensions checked against the names the library wrote, \
         {switch_steps} of them steps between DIP switches of one family"
    );
    assert!(
        switch_steps >= 15,
        "only {switch_steps} DIP steps were compared, so the family grouping has \
         thrown away the half of this test that measures a pitch"
    );
    assert!(
        checked >= 24,
        "only {checked} dimensions were checked, which is too few for this to be \
         saying anything"
    );
}

/// How many switches a DIP switch's own name says it has.
fn how_many_dip_switches(name: &str) -> Option<u32> {
    let rest = name.strip_prefix("SW_DIP_SPSTx")?;
    name.contains("_P2.54mm").then_some(())?;
    rest.split_once('_')?.0.parse().ok()
}

/// Which DIP switch housing a file's name puts it in.
///
/// Only two files from the same housing may be compared, because a name says
/// how many switches and at what pitch and nothing about the ends. The corpus
/// holds four: a piano-key body carrying its own size, the CTS piano-key
/// series, a low-profile slide and a full-height slide. What tells them apart
/// is the style word after the count and the two markers that may follow. The
/// size a name carries cannot be part of the key, because that is the thing
/// that grows with the count; the pitch and the row spacing are the same in all
/// four and would tell nobody anything.
fn which_dip_switch_family(name: &str) -> Option<String> {
    let after_count = name.strip_prefix("SW_DIP_SPSTx")?.split_once('_')?.1;
    let style = after_count.split('_').next()?;
    let series = if name.contains("_CTS_") { " CTS" } else { "" };
    let profile = if name.contains("_LowProfile") {
        " low profile"
    } else {
        ""
    };
    Some(format!("{style}{series}{profile}"))
}

/// How far a whole import reaches along each axis.
fn how_big_it_is(import: &Import) -> [f64; 3] {
    let mut least = [f64::INFINITY; 3];
    let mut most = [f64::NEG_INFINITY; 3];
    for (_, key) in import.solids() {
        let Some((low, high)) = import
            .model()
            .solid(*key)
            .expect("this model made the key")
            .box_around_it()
        else {
            continue;
        };
        for axis in 0..3 {
            least[axis] = least[axis].min(low.coordinates()[axis]);
            most[axis] = most[axis].max(high.coordinates()[axis]);
        }
    }
    [0, 1, 2].map(|axis| most[axis] - least[axis])
}

/// What a KiCad file's own name claims about how big the part is, as `(axis,
/// millimetres)` pairs.
///
/// One family, and the ones that were tried and dropped are worth naming: a
/// `LED_Rectangular_W3.0mm_H2.0mm` measures **four** millimetres across,
/// because the name is about the body and the leads reach wider than it. A name
/// that describes a part of the part is not a claim about the whole one, and a
/// check built on it would have failed for a reason that is nobody's mistake.
///
/// **`P2.54mm` is a pitch, and a pitch is not a width.** For a plain
/// `_Vertical` pin header the claim `axis 0 = rows * 2.54` holds because the
/// rows genuinely sit one pitch apart — measured on every `1x*` and `2x*`
/// `_Vertical` file in the corpus that imports, exactly, `assert_eq!`-tight.
/// For an `_SMD` header it does not: an SMD row's solder legs splay out to a
/// fixed width regardless of pitch or row count, which
/// `an_smd_pin_headers_width_across_its_rows_does_not_scale_with_row_count`
/// below measures directly off ten files' own `VERTEX_POINT` entities at
/// **exactly** 5.0 mm for one row and 5.0 + 2.54 mm for two, never `rows *
/// 2.54` (2.54 mm and 5.08 mm). So axis 0 is only claimed for the plain
/// family; axis 1 — the spacing along a row, which the pitch does fix — is
/// claimed for both.
fn what_the_name_says(name: &str) -> Vec<(usize, f64)> {
    if let Some(rest) = name.strip_prefix("PinHeader_")
        && name.contains("_P2.54mm_Vertical")
        && let Some((rows, rest)) = rest.split_once('x')
        && let Some((columns, _)) = rest.split_once('_')
        && let (Ok(rows), Ok(columns)) = (rows.parse::<u32>(), columns.parse::<u32>())
    {
        let along_a_row = (1, f64::from(columns) * 2.54);
        if name.contains("_SMD") {
            return vec![along_a_row];
        }
        return vec![(0, f64::from(rows) * 2.54), along_a_row];
    }
    Vec::new()
}

/// The width an `_SMD` pin header keeps across its rows, no matter how many
/// pins or rows it has — settled by measuring the file directly rather than
/// assumed, because the whole `_SMD` family is refused by the curved-face
/// check before `Import` ever sees its dimensions (`RCAD_STEP_CORPUS`'s
/// `kicad/` corpus has no `_SMD` pin header import today).
///
/// This reads `VERTEX_POINT` straight off [`StepFile`] — no [`Import`], no
/// dependence on the sweep — because the point of the test is to pin the
/// geometric fact down independently of whether the mapping can reach it yet.
///
/// Ten files, five one-row and five two-row, every one exact: a one-row `_SMD`
/// header is **5.0 mm** across and a two-row one is **7.54 mm** — `5.0 + 2.54`,
/// one row's fixed splay plus one more pitch for the second row's centre.
/// Neither number is `rows * 2.54` (2.54 mm, 5.08 mm), which is what a plain
/// `_Vertical` header of the same row count measures instead. That gap is why
/// `what_the_name_says` does not claim axis 0 for an `_SMD` file: the file
/// really does measure what it measures, and the name's `2.54mm` is a pitch
/// that never governs this axis.
#[test]
fn an_smd_pin_headers_width_across_its_rows_does_not_scale_with_row_count() {
    let root = corpus();
    let kicad = root.join("kicad");
    let one_row = [
        "PinHeader_1x06_P2.54mm_Vertical_SMD_Pin1Right.step",
        "PinHeader_1x08_P2.54mm_Vertical_SMD_Pin1Left.step",
        "PinHeader_1x13_P2.54mm_Vertical_SMD_Pin1Right.step",
        "PinHeader_1x20_P2.54mm_Vertical_SMD_Pin1Right.step",
        "PinHeader_1x36_P2.54mm_Vertical_SMD_Pin1Left.step",
    ];
    let two_row = [
        "PinHeader_2x06_P2.54mm_Vertical_SMD.step",
        "PinHeader_2x13_P2.54mm_Vertical_SMD.step",
        "PinHeader_2x20_P2.54mm_Vertical_SMD.step",
        "PinHeader_2x27_P2.54mm_Vertical_SMD.step",
        "PinHeader_2x34_P2.54mm_Vertical_SMD.step",
    ];
    let mut checked = 0;
    for name in one_row {
        checked += 1;
        assert_vertex_width(&kicad.join(name), 5.0);
    }
    for name in two_row {
        checked += 1;
        assert_vertex_width(&kicad.join(name), 5.0 + 2.54);
    }
    assert_eq!(checked, 10, "this test has to run over the files it names");
}

/// The extent, along axis 0, of every `VERTEX_POINT` a file holds — read
/// straight off its `CARTESIAN_POINT`s, bypassing `Import` entirely.
fn assert_vertex_width(path: &Path, expected: f64) {
    let bytes = fs::read(path).unwrap_or_else(|why| panic!("{}: {why}", path.display()));
    let file = StepFile::read(&bytes).expect("every file in the corpus parses");
    let mut least = f64::INFINITY;
    let mut most = f64::NEG_INFINITY;
    for vertex in file.instances_named("VERTEX_POINT") {
        let Some(at) = vertex
            .part("VERTEX_POINT")
            .and_then(|part| part.attribute(1))
            .and_then(Value::as_reference)
        else {
            continue;
        };
        let Some(x) = file
            .instance(at)
            .and_then(|point| point.part("CARTESIAN_POINT"))
            .and_then(|part| part.attribute(1))
            .and_then(Value::as_list)
            .and_then(|coordinates| coordinates.first())
            .and_then(Value::as_real)
        else {
            continue;
        };
        least = least.min(x);
        most = most.max(x);
    }
    let width = most - least;
    assert_eq!(
        width,
        expected,
        "{} should measure {expected} mm across axis 0 and measures {width}",
        path.display()
    );
}

/// What one corpus came to.
#[derive(Default)]
struct Tally {
    imported: usize,
    solids: usize,
    refused: BTreeMap<String, usize>,
    surfaces_wanted: BTreeMap<String, usize>,
    curves_wanted: BTreeMap<String, usize>,
    integers_read_as_reals: usize,
    files_needing_that: usize,
    walked_past: BTreeMap<&'static str, usize>,
    units: BTreeMap<String, usize>,
    over_precise: usize,
    /// How many files got a whole solid described and handed to
    /// `SolidBuilder`, which is the only denominator the corner-miss bucket
    /// below may be read against. A file the reader stopped at a `B_SPLINE`
    /// never had a corner judged at all, and counting it in would make the
    /// bucket a statement about which shapes this reader holds.
    reached_the_solid_builder: usize,
    corner_misses_in_spacings: Vec<f64>,
    imported_names: BTreeSet<String>,
    /// Every refused file's own reason, one entry each — never a rate, so a
    /// reader can tell which file moved (§44.10) and check §44.6's D trigger
    /// and §44.11 item 6 against measured per-file data rather than a count.
    refused_files: Vec<RefusedFile>,
}

/// One refused file's reason: the filename, and, where the refusal was a
/// corner missing its surface, how far off in that corner's own coordinate
/// spacings.
///
/// **The miss belongs to the reason beside it.** It is read off the refusal
/// that is being printed and off nothing else, so a file whose build stopped
/// for a wire or a winding gets no miss at all rather than one measured on some
/// other corner of the same file (`arms0`).
struct RefusedFile {
    name: String,
    reason: String,
    corner_miss_in_spacings: Option<f64>,
}

#[test]
fn how_often_a_real_file_imports() {
    let root = corpus();

    let mut nist_imports: BTreeSet<String> = BTreeSet::new();
    let mut kicad_imports: BTreeSet<String> = BTreeSet::new();
    let mut every_corner_miss: Vec<f64> = Vec::new();
    let mut planes_between_curved_faces = PlanesBetweenCurvedFaces::default();
    for source in ["nist", "kicad"] {
        let files = files_under(&root.join(source));
        assert!(
            !files.is_empty(),
            "{source} holds no STEP files, so this test would pass by doing nothing"
        );
        let mut tally = Tally::default();
        for path in &files {
            let bytes = fs::read(path).expect("a file the walk just listed");
            let file = StepFile::read(&bytes).expect("every file in both corpora parses");
            let mut log = ProvenanceLog::new();
            match Import::read(&file, &Policy::certified(), &mut log) {
                Ok(import) => {
                    tally.imported += 1;
                    tally.reached_the_solid_builder += 1;
                    tally.imported_names.insert(name_of(path));
                    tally.solids += import.solids().len();
                    *tally
                        .units
                        .entry(import.declared_unit().symbol().to_owned())
                        .or_default() += 1;
                    if import.integers_read_as_reals() > 0 {
                        tally.files_needing_that += 1;
                        tally.integers_read_as_reals += import.integers_read_as_reals();
                    }
                    if import.asks_for_more_precision_than_its_numbers_carry() {
                        tally.over_precise += 1;
                    }
                    for (keyword, count) in import.shapes_this_reader_did_not_take() {
                        *tally.walked_past.entry(keyword).or_default() += count;
                    }
                    planes_between_curved_faces.saw(&name_of(path), import.model());
                    import
                        .model()
                        .validate(&Policy::certified(), &mut log)
                        .unwrap_or_else(|why| {
                            panic!(
                                "{} imported and then failed the sweep, which is a defect and \
                                 not a refusal: {why}",
                                name_of(path)
                            )
                        });
                }
                Err(why) => {
                    let reason = grouped(&why);
                    *tally.refused.entry(reason.clone()).or_default() += 1;
                    let mut corner_miss_in_spacings = None;
                    match &why {
                        Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn {
                            off_by,
                            coordinate_spacing,
                            ..
                        }
                        | Refusal::ACornerIsNotOnThePlaneOfTheArcItEndsOn {
                            off_by,
                            coordinate_spacing,
                            ..
                        } => {
                            tally.reached_the_solid_builder += 1;
                            let miss = off_by / coordinate_spacing;
                            tally.corner_misses_in_spacings.push(miss);
                            corner_miss_in_spacings = Some(miss);
                        }
                        // The other two refusals a whole described solid can
                        // earn. `ThisEntityOfTheSolidWasRefused` is the loose
                        // one: the mapping makes it both for a draft entity
                        // `SolidBuilder` would not take and for a
                        // `SolidBuilder::build` that came back with an index,
                        // so a file counted here may have stopped one call
                        // short of `build`.
                        Refusal::ThisEntityOfTheSolidWasRefused { .. }
                        | Refusal::TheSolidTheFileDescribesIsNotOne { .. } => {
                            tally.reached_the_solid_builder += 1;
                        }
                        Refusal::ASurfaceThisKernelDoesNotHold { keyword, .. } => {
                            *tally.surfaces_wanted.entry(keyword.clone()).or_default() += 1;
                        }
                        Refusal::ACurveThisKernelDoesNotHold { keyword, .. } => {
                            *tally.curves_wanted.entry(keyword.clone()).or_default() += 1;
                        }
                        _ => {}
                    }
                    tally.refused_files.push(RefusedFile {
                        name: name_of(path),
                        reason,
                        corner_miss_in_spacings,
                    });
                }
            }
        }
        report(source, &files, &tally);
        every_corner_miss.extend(&tally.corner_misses_in_spacings);
        if source == "kicad" {
            kicad_imports = tally.imported_names;
        } else if source == "nist" {
            nist_imports = tally.imported_names;
        }
    }
    assert_a_corner_off_its_surface_is_two_problems(&every_corner_miss);
    assert_pinned_nist_imports(&nist_imports);
    assert_pinned_kicad_imports(&kicad_imports);
    planes_between_curved_faces.assert_every_one_was_checked();
}

/// Every arc an imported file names by a plane between two curved faces, and
/// the ones that could not have been checked.
///
/// `rcad_geom::walk` pairs such a plane with one of the faces and walks where
/// the two cross, without asking the other face (`docs/decisions.md` §189.3,
/// widened to every curved face by §220.4). So the plane is only safe to write
/// where the reader has checked the file's curve lies on every face. The check
/// is gone by the time the model is built, so this asks what it would have had
/// to be true of:
///
/// - a circle can lie on any ball;
/// - on a pipe, a circle's plane is square to the axis, and an ellipse's plane
///   leans and cuts **every** face along the edge in the one ellipse: every
///   face a pipe of one radius, each axis crossing the plane at one point and
///   leaning off it the same way;
/// - on a cone, the plane is square to the axis;
/// - on a torus, the plane is square to the axis bit for bit, or holds the
///   axis — the two families the walk tells apart.
///
/// Anything else is a plane nobody checked. Until §220.4 the reader wrote one
/// beside every arc between two curved faces, ellipses where two pipes cross
/// among them, and from then until §222.2's two gaps were closed it wrote one
/// only for a circle on a ball or a pipe.
///
/// A closed curve between two curved faces that are not one surface bit for
/// bit is held to the same check since §219.6's closed branch was brought in
/// line with the arc's, and `closed` counts how many of the checked ones those
/// are. `kinds` counts the checked ones that only the §222.2 bounds pass, as
/// `[arcs, closed]`.
#[derive(Default)]
struct PlanesBetweenCurvedFaces {
    checked: usize,
    closed: usize,
    kinds: BTreeMap<&'static str, [usize; 2]>,
    unchecked: Vec<String>,
    the_most_a_square_pipe_leans: f64,
}

/// The sine of the angle between a plane's normal and a pipe's axis.
fn how_far_a_pipe_leans_off_square(normal: Vector, axis: Vector) -> f64 {
    let normal = normal.normalized().expect("a plane's normal has a length");
    let axis = axis.normalized().expect("an axis has a length");
    normal.cross(axis).length()
}

/// How far a surface's axis may lean off the normal of a plane a circle on it
/// lies in, as a sine, and how far two ellipses' numbers may differ, relative
/// to their size, and still be read as one.
///
/// Not a modelling tolerance and not what the reader checks — the reader's
/// check is `every_face_holds_the_curve`, in the reader. This only tells a
/// circle's plane from an ellipse's, which lean by a large angle or not at
/// all: the corpus's worst square pipe leans `2.7e-15`, printed beside the
/// count, and the mitre named under [`A_PLANE_BETWEEN_CURVED_FACES`] leans
/// `0.707`.
const A_PIPE_SQUARE_TO_ITS_CIRCLE_LEANS_AT_MOST: f64 = 1e-12;

impl PlanesBetweenCurvedFaces {
    fn saw(&mut self, file: &str, model: &rcad_topo::Model) {
        for edge in model.edges() {
            let (and, closed) = match edge.run() {
                EdgeRun::PartOfWhereItCrosses { and, .. } => (and, false),
                EdgeRun::RightRoundWhereItCrosses { and } => (and, true),
                EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross | _ => continue,
            };
            let Some(flat) = and.as_plane() else {
                continue;
            };
            let faces: Vec<Surface> = edge
                .neighbouring_faces()
                .map(|face| model.face(face).expect("a face this edge names").surface())
                .collect();
            let all_curved = faces.len() >= 2 && faces.iter().all(|f| f.as_plane().is_none());
            if !all_curved || faces.windows(2).all(|pair| pair[0] == pair[1]) {
                continue;
            }
            let normal = flat.normal();
            let square = |axis: Vector| {
                how_far_a_pipe_leans_off_square(normal, axis)
                    <= A_PIPE_SQUARE_TO_ITS_CIRCLE_LEANS_AT_MOST
            };
            let mut kind = None;
            let a_circle_on_it_could_be_on_every_face = faces.iter().all(|face| {
                if let Surface::Cylinder(pipe) = face {
                    let lean = how_far_a_pipe_leans_off_square(normal, pipe.axis().direction());
                    if square(pipe.axis().direction()) {
                        self.the_most_a_square_pipe_leans =
                            self.the_most_a_square_pipe_leans.max(lean);
                        return true;
                    }
                    kind = Some("an ellipse on a pipe");
                    return one_ellipse_on_every_pipe(flat, &faces);
                }
                if let Surface::Cone(cone) = face {
                    kind = kind.or(Some("a circle on a cone"));
                    return square(cone.axis().direction());
                }
                if let Surface::Torus(ring) = face {
                    kind = kind.or(Some("a circle on a torus"));
                    let up = ring.axis().direction();
                    return normal.cross(up) == Vector::zero()
                        || normal.normalized().expect("a normal").dot(up).abs()
                            <= A_PIPE_SQUARE_TO_ITS_CIRCLE_LEANS_AT_MOST;
                }
                matches!(face, Surface::Sphere(_))
            });
            if a_circle_on_it_could_be_on_every_face {
                self.checked += 1;
                self.closed += usize::from(closed);
                if let Some(kind) = kind {
                    self.kinds.entry(kind).or_default()[usize::from(closed)] += 1;
                }
            } else {
                self.unchecked
                    .push(format!("{file}: {faces:?} and {and:?}"));
            }
        }
    }

    /// Floored as well as checked, so a corpus that stopped reaching such an
    /// arc would not leave this green over nothing.
    fn assert_every_one_was_checked(&self) {
        println!(
            "planes between two curved faces: {} a check could pass ({} of them closed), {} it \
             could not; the most a square pipe leans is {:e}; passed only by §222.2's bounds, as \
             [arcs, closed]: {:?}",
            self.checked,
            self.closed,
            self.unchecked.len(),
            self.the_most_a_square_pipe_leans,
            self.kinds
        );
        assert!(
            self.unchecked.is_empty(),
            "{} arcs name a plane between curved faces no circle check covers, the first: {}",
            self.unchecked.len(),
            self.unchecked[0]
        );
        assert!(
            self.checked >= A_PLANE_BETWEEN_CURVED_FACES,
            "only {} imported arcs name a plane between two curved faces, so this check looked at \
             almost nothing",
            self.checked
        );
    }
}

/// Whether `flat` cuts every face in the one ellipse: every face a pipe of one
/// radius, whose axis crosses `flat` at one point and leans off it the same
/// way, which gives every pipe the same centre, the same two semi-axes and the
/// same long direction.
fn one_ellipse_on_every_pipe(flat: &Plane, faces: &[Surface]) -> bool {
    let normal = flat
        .normal()
        .normalized()
        .expect("a plane's normal has a length");
    let Some(ellipses) = faces
        .iter()
        .map(|face| {
            let Surface::Cylinder(pipe) = face else {
                return None;
            };
            let line = pipe.axis();
            let along = line.direction();
            let reach = (flat.offset() - flat.normal().dot(line.origin() - Point::origin()))
                / flat.normal().dot(along);
            Some((
                line.origin() + along * reach,
                along - normal * along.dot(normal),
                along.dot(normal).abs(),
                pipe.radius(),
            ))
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let (centre, long_way, square, radius) = ellipses[0];
    let near = |one: f64, other: f64, size: f64| {
        (one - other).abs() <= A_PIPE_SQUARE_TO_ITS_CIRCLE_LEANS_AT_MOST * size.max(1.0)
    };
    ellipses.iter().all(|(there, way, how_square, how_wide)| {
        near(
            centre.distance_to(*there),
            0.0,
            centre.largest_coordinate_magnitude(),
        ) && near(long_way.cross(*way).length(), 0.0, 1.0)
            && near(square, *how_square, 1.0)
            && near(radius, *how_wide, radius)
    })
}

/// How many imported arcs have to name a plane between two curved faces.
///
/// Measured at 848 on 2026-09-28, with 0 a check could not have passed, by
///
/// ```text
/// mise exec -- just t -p rcad-step --no-capture -E 'test(=how_often_a_real_file_imports)'
/// ```
///
/// and floored at three quarters. Before §220.4 the same files held 848, and 494
/// of them were planes no check could have passed — the first a mitre where two
/// pipes of `Relay_DPDT_FCL_FTR-B4C.step` cross in an ellipse at 45°. From
/// §220.4 until the ellipse got a bound there were 354, every one a circle; the
/// 334 more are ellipses where pipes of one radius meet, that mitre among them,
/// which the reader now checks rather than naming by a face. It was 688 until
/// the reader worked an ellipse's sizes out from its pipe rather than reading
/// the file's twelve digits (`docs/decisions.md` §335); the 160 more are
/// ellipses too, and the files that import did not change.
const A_PLANE_BETWEEN_CURVED_FACES: usize = 636;

/// A corner off the surface it is meant to be on is **two** problems, and both
/// corpora together have to hold both or the report is describing a corpus this
/// is not being run on.
///
/// Under one spacing the file's numbers are as close to the surface as an `f64`
/// can be written and the refusal is about what a `Decision` can certify, not
/// about the file. Over one spacing — and the tail runs into the tens of
/// thousands — the file carries a modelling tolerance somebody chose, and no
/// arithmetic anywhere would make those corners land on their surfaces.
///
/// **Asked of both corpora together and no longer of each, which is `arms0`'s
/// finding and not a weakening slipped in.** It used to hold per source, on a
/// population that included refusals the mapping had *substituted*: a build
/// stopped for a winding or a leaning face was published as a corner off a
/// curved face, scanned for after the fact. With the substitution gone, `nist`
/// keeps four such files and all four are a modelling tolerance. One corpus of
/// 44 files holding no near-miss is a fact about that corpus; the claim the
/// report makes is about the refusal.
fn assert_a_corner_off_its_surface_is_two_problems(misses: &[f64]) {
    let within_a_spacing = misses.iter().filter(|miss| **miss < 1.0).count();
    assert!(
        within_a_spacing > 0 && within_a_spacing < misses.len(),
        "{within_a_spacing} of {} corner misses are inside one spacing, and both halves have to \
         be there or this is not the corpus it was measured on",
        misses.len()
    );
    let worst = misses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        worst > 1000.0,
        "a worst corner of {worst} spacings means no file anywhere here carries a modelling \
         tolerance, and they do"
    );
}

/// The one NIST file that imports today, out of 44.
///
/// **Pinned by name, not by rate** (§44.10), the same reason
/// [`KICAD_FILES_THAT_IMPORT`] is. The crate doc's table at
/// `crates/rcad-step/src/lib.rs` used to say 0 of 44; this is the file that
/// moved it.
const NIST_FILES_THAT_IMPORT: &[&str] = &["nist_ctc_03_asme1_ap242-e2.stp"];

/// Every NIST file that imports must be [`NIST_FILES_THAT_IMPORT`], and every
/// one of those must import — checked as a set for the same reason
/// [`assert_pinned_kicad_imports`] is.
fn assert_pinned_nist_imports(imported: &BTreeSet<String>) {
    let pinned: BTreeSet<&str> = NIST_FILES_THAT_IMPORT.iter().copied().collect();
    let got: BTreeSet<&str> = imported.iter().map(String::as_str).collect();
    let newly_importing: Vec<&str> = got.difference(&pinned).copied().collect();
    let stopped_importing: Vec<&str> = pinned.difference(&got).copied().collect();
    assert!(
        newly_importing.is_empty() && stopped_importing.is_empty(),
        "the nist files that import moved: {} file(s) now import and are not pinned: \
         {newly_importing:?}; {} pinned file(s) stopped importing: {stopped_importing:?}",
        newly_importing.len(),
        stopped_importing.len()
    );
}

/// The 114 KiCad files that import today, out of 323.
///
/// **Pinned by name, not by rate** (§44.10): a rate cannot tell a reader which
/// file moved. This list moves only by an edit to this constant, made by
/// whoever's change moved it. Regenerate it with
/// `RCAD_STEP_CORPUS=<path> mise exec -- cargo test -p rcad-step --release \
/// --test how_often_a_real_file_imports -- --nocapture`, which prints the
/// failing assertion's "now imports" and "stopped importing" sets when this
/// list is stale.
///
/// **It was 46 at `d23c700` and the curved-rounding lane moved it by fifty**,
/// with nothing dropping out and no file that already imported changing the
/// size it measures. The fifty are the ones a corner off a curved face was
/// stopping first, now that a corner within `NEAREST_F64_POINT_ROUNDOFF` of its
/// surface is read as one no writer could have improved on rather than one that
/// is off: every `Fuse_*`, both `LED_D2.0mm_*_FlatTop`, the `_Horizontal` and
/// `_Vertical_SMD` pin headers, four more relays, and the twelve
/// `SW_DIP_SPSTx*_Piano_10.8x*`.
///
/// **Fifty is not the 88 §44.6 named, and the difference is not measured
/// here.** §43.5's 88 counts files with a corner off a curved face, not files
/// that corner stopped *first*, so a file freed of it may meet a second blocker
/// — 29 `B_SPLINE_CURVE_WITH_KNOTS` and 13 `B_SPLINE_SURFACE_WITH_KNOTS` are
/// waiting in this same corpus.
///
/// **The two-step wire lane then moved it by one, and that is a correction to
/// §44.3 rather than a disappointing result.** §44.3 priced *a face bounded by
/// an arc and its chord is a face* at 75 files — the 59 KiCad and 16 NIST whose
/// **first** refusal was a two-step wire. Lifting it emptied that row of both
/// tables, and exactly one file came all the way through:
/// `LED_SideEmitter_Rectangular_W4.5mm_H1.6mm`. The other 74 met their second
/// blocker the same run — *a corner is not on the curved face the file says it
/// is on* went 103 to 160 here and 5 to 11 in the NIST table, and
/// `B_SPLINE_CURVE_WITH_KNOTS` went 29 to 30 and 15 to 25. So §44.11 item 5's
/// warning — a count of files a blocker stops **first** is an upper bound on
/// what removing it is worth, never a projection — governs the two-step item
/// exactly as it governs the curved one, and 75 was that upper bound.
///
/// **The closed-edge lane then moved it from 97 to 110** (§115): the nine
/// `LED_D1.8mm_*_Horizontal`, three more Finder relays and `SW_NKK_BB15AH`,
/// with nothing dropping out. Those thirteen were being refused as *a face
/// walks a closed edge against the way its surface winds*, which the reader was
/// earning for itself by handing `ORIENTED_EDGE.orientation` through on an edge
/// with one corner, where it means nothing.
///
/// **Sixteen files carried that reason and thirteen came through**, which is
/// §44.11 item 5 again and this time from both sides: two met a later blocker
/// the same run, and `LED_D5.0mm-3` still had the closed-edge reason itself.
/// That last one is the open end of §115 — the reader's hint and the certified
/// rule disagree about that file, and which of them is right is not measured.
///
/// **The equal-pipes lane then moved it from 110 to 112**, with nothing dropping
/// out: `Relay_DPDT_Omron_G6K-2P` and `Relay_DPDT_Omron_G6K-2P-Y`. Two pipes of
/// one width whose axes meet cross in two ellipses, and `walk` used to hand that
/// pair to the marcher, which walked into the point where the four branches meet
/// and gave up saying the two surfaces were *too nearly parallel to follow* — a
/// reason that was not true of them. Six KiCad files were being stopped first by
/// exactly that, measured with `blocker_depth_probe`; two came all the way
/// through and the other four met a later blocker the same run, which is §44.11
/// item 5 yet again. `Relay_SPST_SANYOU_SRD_Series_Form_A` is one of the four
/// and its next blocker is a pipe tangent to a ball at a pin's rounded end.
///
/// **The homogeneous-cylinder lane then moved it from 112 to 114**, with nothing
/// dropping out: `Relay_SPDT_Finder_32.21-x000` and
/// `Relay_SPST_Finder_32.21-x300`. Both hold the identical pair of 0.3 mm bores
/// crossing at 45°, which the equal-pipes family above should have caught and
/// did not: `Cylinder::side_of` compared `|(p−o)×w|²` against a bare `r²`, so
/// the width it certified was `r/‖w‖`, and `1/√2` is not an `f64` — the two
/// bores were 1.5e-17 apart in width and crossed in a genuine quartic with two
/// necks that narrow to about that. Making both sides of the comparison degree
/// two in the direction makes a pipe `r` wide however its direction rounded, and
/// the pair is the family again (`docs/decisions.md` §118.10).
///
/// **The touching-circle lane then moved it from 114 to 140**, with nothing
/// dropping out: 25 relays and `Fuseholder_Cylinder-5x20mm_Stelvio-Kontek_PTF78_Horizontal_Open`.
/// Where a ball meets another curved face along a circle the file names, the
/// reader now checks that circle lies on both faces and names the run by the
/// circle's plane, and the walk crosses the plane with the ball instead of
/// following two surfaces that only touch (`docs/decisions.md` §189.3). Which
/// edge stopped each of the 26 before is not measured here;
/// `Relay_SPST_SANYOU_SRD_Series_Form_A` is the one the equal-pipes paragraph
/// above named as stopped by a pipe tangent to a ball at a pin's rounded end.
///
/// **The dome-slit lane then moved it from 140 to 160**, with nothing dropping
/// out: 20 `LED_D3.0mm_Horizontal` and `LED_D5.0mm_Horizontal` files. Each
/// dome's seam, read in this kernel's chart of the ball, is a slit the wire
/// runs out along and back, and the mesher now cuts the dome in two on from
/// the slit's inner end instead of handing the triangulator the same points
/// twice.
///
/// **The pole lane then moved it from 160 to 161**, with nothing dropping out:
/// `LED_D5.0mm-3`, the §115 file named above. The mesher now puts in a ball's
/// pole when a circle passes within a rounding of it, and calls a walked point
/// that close to a pole the pole (`docs/decisions.md` §198). Which of those two
/// freed this file is not measured, and the solid it imports as is checked by
/// `Model::validate` and nothing else.
///
/// **Turning the mesher's torus faces moved it from 161 to 164**, with nothing
/// dropping out: `Relay_1-Form-A_Schrack-RYII_RM5mm`,
/// `Relay_1-Form-B_Schrack-RYII_RM5mm` and `Relay_SPDT_Finder_34.51_Vertical`,
/// each with torus faces used forwards, which the mesher faced into the tube
/// and refused `FacesDoNotAgreeWhichWayTheyFace` (`docs/decisions.md` §329.4,
/// §361).
///
/// **Charging a circle's lean off a torus's tube once moved it from 164 to
/// 165**, with nothing dropping out: `Relay_1-Form-C_Schrack-RYII_RM3.2mm`,
/// whose two pin-bend circles read `19.43` roundings off their tube circles
/// against the sixteen allowed until the torus arm's `2` became `√2`
/// (`docs/decisions.md` §387).
const KICAD_FILES_THAT_IMPORT: &[&str] = &[
    "Fuse_0402_1005Metric.step",
    "Fuse_0603_1608Metric.step",
    "Fuse_0805_2012Metric.step",
    "Fuse_1206_3216Metric.step",
    "Fuse_1210_3225Metric.step",
    "Fuseholder_Cylinder-5x20mm_Stelvio-Kontek_PTF78_Horizontal_Open.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O1.27mm_Z1.6mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O1.27mm_Z4.9mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O1.27mm_Z8.2mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O3.81mm_Z1.6mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O3.81mm_Z4.9mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O3.81mm_Z8.2mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O6.35mm_Z1.6mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O6.35mm_Z4.9mm.step",
    "LED_D1.8mm_W1.8mm_H2.4mm_Horizontal_O6.35mm_Z8.2mm.step",
    "LED_D2.0mm_W4.0mm_H2.8mm_FlatTop.step",
    "LED_D2.0mm_W4.8mm_H2.5mm_FlatTop.step",
    "LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step",
    "LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step",
    "LED_D3.0mm_Horizontal_O3.81mm_Z10.0mm.step",
    "LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step",
    "LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step",
    "LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step",
    "LED_D3.0mm_Horizontal_O6.35mm_Z2.0mm.step",
    "LED_D3.0mm_Horizontal_O6.35mm_Z6.0mm.step",
    "LED_D5.0mm-3.step",
    "LED_D5.0mm_Horizontal_O1.27mm_Z15.0mm.step",
    "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step",
    "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm_Clear.step",
    "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm_IRBlack.step",
    "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm_IRGrey.step",
    "LED_D5.0mm_Horizontal_O1.27mm_Z9.0mm.step",
    "LED_D5.0mm_Horizontal_O3.81mm_Z15.0mm.step",
    "LED_D5.0mm_Horizontal_O3.81mm_Z3.0mm.step",
    "LED_D5.0mm_Horizontal_O3.81mm_Z9.0mm.step",
    "LED_D5.0mm_Horizontal_O6.35mm_Z15.0mm.step",
    "LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step",
    "LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step",
    "LED_Rectangular_W3.0mm_H2.0mm.step",
    "LED_Rectangular_W3.9mm_H1.8mm.step",
    "LED_Rectangular_W3.9mm_H1.9mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O1.27mm_Z1.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O1.27mm_Z3.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O1.27mm_Z5.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O3.81mm_Z1.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O3.81mm_Z3.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O3.81mm_Z5.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O6.35mm_Z1.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O6.35mm_Z3.0mm.step",
    "LED_Rectangular_W5.0mm_H2.0mm_Horizontal_O6.35mm_Z5.0mm.step",
    "LED_Rectangular_W5.0mm_H5.0mm.step",
    "LED_SideEmitter_Rectangular_W4.5mm_H1.6mm.step",
    "PinHeader_1x03_P2.54mm_Horizontal.step",
    "PinHeader_1x04_P2.54mm_Vertical.step",
    "PinHeader_1x06_P2.54mm_Vertical_SMD_Pin1Right.step",
    "PinHeader_1x08_P2.54mm_Vertical_SMD_Pin1Left.step",
    "PinHeader_1x10_P2.54mm_Horizontal.step",
    "PinHeader_1x11_P2.54mm_Vertical.step",
    "PinHeader_1x13_P2.54mm_Vertical_SMD_Pin1Right.step",
    "PinHeader_1x15_P2.54mm_Vertical_SMD_Pin1Left.step",
    "PinHeader_1x17_P2.54mm_Horizontal.step",
    "PinHeader_1x18_P2.54mm_Vertical.step",
    "PinHeader_1x20_P2.54mm_Vertical_SMD_Pin1Right.step",
    "PinHeader_1x22_P2.54mm_Vertical_SMD_Pin1Left.step",
    "PinHeader_1x24_P2.54mm_Horizontal.step",
    "PinHeader_1x25_P2.54mm_Vertical.step",
    "PinHeader_1x27_P2.54mm_Vertical_SMD_Pin1Right.step",
    "PinHeader_1x29_P2.54mm_Vertical_SMD_Pin1Left.step",
    "PinHeader_1x31_P2.54mm_Horizontal.step",
    "PinHeader_1x32_P2.54mm_Vertical.step",
    "PinHeader_1x34_P2.54mm_Vertical_SMD_Pin1Right.step",
    "PinHeader_1x36_P2.54mm_Vertical_SMD_Pin1Left.step",
    "PinHeader_1x38_P2.54mm_Horizontal.step",
    "PinHeader_1x39_P2.54mm_Vertical.step",
    "PinHeader_2x01_P2.54mm_Vertical.step",
    "PinHeader_2x04_P2.54mm_Horizontal.step",
    "PinHeader_2x06_P2.54mm_Vertical_SMD.step",
    "PinHeader_2x08_P2.54mm_Vertical.step",
    "PinHeader_2x11_P2.54mm_Horizontal.step",
    "PinHeader_2x13_P2.54mm_Vertical_SMD.step",
    "PinHeader_2x15_P2.54mm_Vertical.step",
    "PinHeader_2x18_P2.54mm_Horizontal.step",
    "PinHeader_2x20_P2.54mm_Vertical_SMD.step",
    "PinHeader_2x22_P2.54mm_Vertical.step",
    "PinHeader_2x25_P2.54mm_Horizontal.step",
    "PinHeader_2x27_P2.54mm_Vertical_SMD.step",
    "PinHeader_2x29_P2.54mm_Vertical.step",
    "PinHeader_2x32_P2.54mm_Horizontal.step",
    "PinHeader_2x34_P2.54mm_Vertical_SMD.step",
    "PinHeader_2x36_P2.54mm_Vertical.step",
    "PinHeader_2x39_P2.54mm_Horizontal.step",
    "Relay_1-Form-A_Schrack-RYII_RM5mm.step",
    "Relay_1-Form-B_Schrack-RYII_RM5mm.step",
    "Relay_1-Form-C_Schrack-RYII_RM3.2mm.step",
    "Relay_3PST_COTO_3650.step",
    "Relay_3PST_COTO_3660.step",
    "Relay_DPDT_FCL_FTR-B4C.step",
    "Relay_DPDT_Finder_30.22.step",
    "Relay_DPDT_Finder_40.52.step",
    "Relay_DPDT_Hongfa_HF115F-2Z-x4.step",
    "Relay_DPDT_Kemet_EC2_NJ.step",
    "Relay_DPDT_Kemet_EC2_NJ_DoubleCoil.step",
    "Relay_DPDT_Kemet_EC2_NU.step",
    "Relay_DPDT_Kemet_EC2_NU_DoubleCoil.step",
    "Relay_DPDT_Kemet_UA2-NE_NU_NJ.step",
    "Relay_DPDT_Omron_G2RL.step",
    "Relay_DPDT_Omron_G5V-2.step",
    "Relay_DPDT_Omron_G6A.step",
    "Relay_DPDT_Omron_G6AK.step",
    "Relay_DPDT_Omron_G6H-2.step",
    "Relay_DPDT_Omron_G6K-2P-Y.step",
    "Relay_DPDT_Omron_G6K-2P.step",
    "Relay_DPDT_Panasonic_JW2.step",
    "Relay_DPDT_Schrack-RT2-FormC-Dual-Coil_RM5mm.step",
    "Relay_DPDT_Schrack-RT2-FormC_RM5mm.step",
    "Relay_DPST_Schrack-RT2-FormA_RM5mm.step",
    "Relay_SPDT_CUI_SR5.step",
    "Relay_SPDT_Finder_32.21-x000.step",
    "Relay_SPDT_Finder_34.51_Vertical.step",
    "Relay_SPDT_Finder_36.11.step",
    "Relay_SPDT_Finder_40.31.step",
    "Relay_SPDT_Finder_40.51.step",
    "Relay_SPDT_HJR-4102.step",
    "Relay_SPDT_Omron-G5LE-1.step",
    "Relay_SPDT_Omron_G6E.step",
    "Relay_SPDT_Omron_G6EK.step",
    "Relay_SPDT_SANYOU_SRD_Series_Form_C.step",
    "Relay_SPDT_Schrack-RT1-FormC_RM5mm.step",
    "Relay_SPDT_StandexMeder_SIL_Form1C.step",
    "Relay_SPST_Finder_32.21-x300.step",
    "Relay_SPST_Omron_G5NB.step",
    "Relay_SPST_SANYOU_SRD_Series_Form_A.step",
    "Relay_SPST_SANYOU_SRD_Series_Form_B.step",
    "Relay_SPST_Schrack-RT1-16A-FormA_RM5mm.step",
    "Relay_SPST_Schrack-RT1-FormA_RM5mm.step",
    "Relay_SPST_StandexMeder_SIL_Form1A.step",
    "Relay_SPST_StandexMeder_SIL_Form1B.step",
    "Relay_StandexMeder_DIP_HighProfile.step",
    "Relay_StandexMeder_DIP_LowProfile.step",
    "Relay_StandexMeder_UMS.step",
    "SW_DIP_SPSTx01_Piano_10.8x4.1mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx02_Piano_10.8x6.64mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx02_Piano_CTS_Series194-2MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx03_Piano_10.8x9.18mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx03_Piano_CTS_Series194-3MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx04_Piano_10.8x11.72mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx04_Piano_CTS_Series194-4MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx05_Piano_10.8x14.26mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx05_Piano_CTS_Series194-5MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx06_Piano_10.8x16.8mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx06_Piano_CTS_Series194-6MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx07_Piano_10.8x19.34mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx07_Piano_CTS_Series194-7MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx08_Piano_10.8x21.88mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx08_Piano_CTS_Series194-8MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx09_Piano_10.8x24.42mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx09_Piano_CTS_Series194-9MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx10_Piano_10.8x26.96mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx10_Piano_CTS_Series194-10MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx11_Piano_10.8x29.5mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx11_Piano_CTS_Series194-11MSTN_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx12_Piano_10.8x32.04mm_W7.62mm_P2.54mm.step",
    "SW_DIP_SPSTx12_Piano_CTS_Series194-12MSTN_W7.62mm_P2.54mm.step",
    "SW_NKK_BB15AH.step",
];

/// Every KiCad file that imports must be one of [`KICAD_FILES_THAT_IMPORT`],
/// and every one of those must import — checked as a set so a failure names
/// what appeared and what disappeared, never just a count.
fn assert_pinned_kicad_imports(imported: &BTreeSet<String>) {
    let pinned: BTreeSet<&str> = KICAD_FILES_THAT_IMPORT.iter().copied().collect();
    let got: BTreeSet<&str> = imported.iter().map(String::as_str).collect();
    let newly_importing: Vec<&str> = got.difference(&pinned).copied().collect();
    let stopped_importing: Vec<&str> = pinned.difference(&got).copied().collect();
    assert!(
        newly_importing.is_empty() && stopped_importing.is_empty(),
        "the kicad files that import moved since d23c700 (docs/decisions.md §44.10): \
         {} file(s) now import and are not pinned: {newly_importing:?}; \
         {} pinned file(s) stopped importing: {stopped_importing:?}",
        newly_importing.len(),
        stopped_importing.len()
    );
}

fn report(source: &str, files: &[PathBuf], tally: &Tally) {
    let denominator = files.len();
    #[expect(
        clippy::cast_precision_loss,
        reason = "a percentage of a few hundred files"
    )]
    let rate = 100.0 * tally.imported as f64 / denominator as f64;
    println!(
        "=== {source}: {} / {denominator} imported = {rate:.1}%",
        tally.imported
    );
    println!("    {} solids", tally.solids);
    println!("    units declared: {:?}", tally.units);
    println!(
        "    {} files wrote an integer where the schema wants a real, {} times in all",
        tally.files_needing_that, tally.integers_read_as_reals
    );
    println!(
        "    {} files ask for more precision than their own numbers carry",
        tally.over_precise
    );
    println!(
        "    shapes this reader did not take: {:?}",
        tally.walked_past
    );
    let mut reasons: Vec<_> = tally.refused.iter().collect();
    reasons.sort_by_key(|&(name, count)| (std::cmp::Reverse(*count), name.clone()));
    for (reason, count) in reasons {
        println!("    {count:>4}  {reason}");
    }
    if !tally.surfaces_wanted.is_empty() {
        println!("    surfaces asked for: {:?}", tally.surfaces_wanted);
    }
    if !tally.curves_wanted.is_empty() {
        println!("    curves asked for: {:?}", tally.curves_wanted);
    }
    println!(
        "    {} files got a whole solid described and handed to SolidBuilder",
        tally.reached_the_solid_builder
    );
    report_the_corner_misses(
        &tally.corner_misses_in_spacings,
        tally.reached_the_solid_builder,
    );
    report_per_file_refusals(source, &tally.refused_files);
}

/// One line per refused file: its family, its name, the reason `Import::read`
/// gave first, and — for a corner missing its surface — the miss in that
/// corner's own coordinate spacings. §44.6's D trigger and §44.11 item 6 both
/// need this rather than a count: whether a family's rate hides one file or
/// many, and whether a lumped row double-counted another, are both questions
/// a rate cannot answer and a per-file list can.
fn report_per_file_refusals(source: &str, refused: &[RefusedFile]) {
    let mut sorted: Vec<&RefusedFile> = refused.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    for file in sorted {
        match file.corner_miss_in_spacings {
            Some(miss) => {
                println!(
                    "    refused  {source}  {}  {}  miss={miss:.3} spacings",
                    file.name, file.reason
                );
            }
            None => println!("    refused  {source}  {}  {}", file.name, file.reason),
        }
    }
}

/// How far the worst corner of each refused file is off the curved surface the
/// file says it is on, in that corner's own coordinate spacings.
///
/// **Counted against the files that got as far as `SolidBuilder`** and never
/// against the whole corpus. A file the reader stopped at a `B_SPLINE` never had
/// a corner judged, so putting it in the denominator would turn this into a
/// statement about which shapes this reader holds.
///
/// **The split is the finding and it is a split**, so both halves are printed
/// and a single median never stands alone. That both halves exist at all is
/// asked of the two corpora together, in
/// [`assert_a_corner_off_its_surface_is_two_problems`].
fn report_the_corner_misses(misses: &[f64], reached_the_solid_builder: usize) {
    if misses.is_empty() {
        return;
    }
    assert!(
        misses.len() <= reached_the_solid_builder,
        "{} files were refused for a corner off a surface and only {reached_the_solid_builder} \
         got as far as the builder, so the bucket is being counted against the wrong thing",
        misses.len()
    );
    let mut sorted = misses.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |part: f64| {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "an index into a few hundred"
        )]
        #[expect(clippy::cast_sign_loss, reason = "part is between zero and one")]
        #[expect(clippy::cast_precision_loss, reason = "a few hundred files")]
        let position = (part * (sorted.len() - 1) as f64) as usize;
        sorted[position]
    };
    let within_a_spacing = sorted.iter().filter(|miss| **miss < 1.0).count();
    println!(
        "    the worst corner of each of the {} files refused for one, out of the \
         {reached_the_solid_builder} that reached the builder, in its own coordinate spacings: \
         least {:.3}, quartile {:.3}, median {:.3}, upper quartile {:.3}, most {:.1}",
        sorted.len(),
        at(0.0),
        at(0.25),
        at(0.5),
        at(0.75),
        at(1.0)
    );
    println!(
        "      {within_a_spacing} of them are inside one spacing — as close as an f64 can be \
         written — and {} are a modelling tolerance",
        sorted.len() - within_a_spacing
    );
}

/// The name a refusal is counted under, which is the variant and never the
/// entity it names.
fn grouped(why: &Refusal) -> String {
    match why {
        Refusal::TheFileHoldsNoSolid => "the file holds no solid".to_owned(),
        Refusal::ASolidWithAVoidInIt { .. } => "a solid with a void in it".to_owned(),
        Refusal::ASurfaceThisKernelDoesNotHold { keyword, .. } => {
            format!("a surface this kernel does not hold: {keyword}")
        }
        Refusal::ACurveThisKernelDoesNotHold { keyword, .. } => {
            format!("a curve this kernel does not hold: {keyword}")
        }
        Refusal::ALoopThisReaderDoesNotWalk { keyword, .. } => {
            format!("a loop this reader does not walk: {keyword}")
        }
        Refusal::AFaceThatDoesNotSayWhichBoundIsTheOuterOne { .. } => {
            "a face that does not say which bound is the outer one".to_owned()
        }
        Refusal::AnArcThatStartsAndEndsInTheSamePlace { .. } => {
            "an arc that starts and ends in the same place".to_owned()
        }
        Refusal::AnArcThisReaderCouldNotName { .. } => {
            "an arc this reader could not name".to_owned()
        }
        Refusal::ALengthUnitThisKernelDoesNotHave { described, .. } => {
            format!("a length unit this kernel does not have: {described}")
        }
        Refusal::TheFileDeclaresNoLengthUnit => "the file declares no length unit".to_owned(),
        Refusal::TheFileDeclaresTwoDifferentLengthUnits { .. } => {
            "the file declares two different length units".to_owned()
        }
        Refusal::APlacementThisReaderCannotApply { keyword, .. } => {
            format!("a placement this reader cannot apply: {keyword}")
        }
        Refusal::AnEntityIsNotTheKindTheSchemaWants { wanted, found, .. } => {
            format!("a {found} where the schema wants a {wanted}")
        }
        Refusal::AnAttributeIsNotWhatTheSchemaWants {
            keyword, wanted, ..
        } => format!("{keyword}'s attribute is not the {wanted} the schema wants"),
        Refusal::AnIntegerTooBigToBeTheRealItStandsFor { .. } => {
            "an integer too big to be the real it stands for".to_owned()
        }
        Refusal::AReferencePointsNowhere { .. } => "a reference points nowhere".to_owned(),
        Refusal::TheGeometryIsNotAWorkingShape { why, .. } => {
            format!("the geometry is not a working shape: {why}")
        }
        Refusal::ThisEntityOfTheSolidWasRefused { why, .. }
        | Refusal::TheSolidTheFileDescribesIsNotOne { why, .. } => {
            format!(
                "the builder or the sweep: {}",
                first_line(&format!("{why}"))
            )
        }
        Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn { .. } => {
            "a corner is not on the curved face the file says it is on".to_owned()
        }
        Refusal::ACornerIsNotOnThePlaneOfTheArcItEndsOn { .. } => {
            "a corner is not on the plane of the arc it ends on".to_owned()
        }
        Refusal::TheFileHasNoSize => "the file has no size".to_owned(),
        // `Refusal` is `#[non_exhaustive]` and grows; a new variant that is not
        // named here would otherwise be counted under somebody else's name.
        _ => format!("unnamed: {why}"),
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or(text).to_owned()
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!(
            "RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/ (see \
             this file's header) — this test does not skip on a missing corpus."
        )
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

fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}
