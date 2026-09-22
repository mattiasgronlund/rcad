//! A `CIRCLE` a real file puts on a pipe is off that pipe in exact arithmetic,
//! and the file still has to import.
//!
//! This is the demonstration behind `brep.rs`'s `A_CIRCLE_MAY_MISS_ITS_FACE_BY`
//! (`docs/decisions.md` §16.10): the reader decides whether a file's circle
//! lies on the faces along its edge with a width, not with a certified sign,
//! because for these numbers the sign is not zero. A check asking "is every
//! point of the circle exactly on the pipe" gets "no", with certainty, for a
//! circle every modeller meant to be on it.
//!
//! Each case reads the file's own lines, so the numbers below are the file's
//! and not a copy that could drift from it.

use std::path::PathBuf;
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};

/// A circle and the pipe along its edge, as one file writes them.
struct CircleOnAPipe {
    file: &'static str,
    lines: [&'static str; 4],
    circle_centre: [f64; 3],
    pipe_origin: [f64; 3],
    leaning: [f64; 3],
}

/// The LED body's circle `#316` on pipe `#324`, and the relay's `#1359` on
/// `#1365` — the second is the furthest off its face of any circle in a file
/// that imports.
const CASES: [CircleOnAPipe; 2] = [
    CircleOnAPipe {
        file: "LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step",
        lines: [
            "#318 = CARTESIAN_POINT('',(1.27,-5.07,10.));",
            "#319 = DIRECTION('',(-5.551115123126E-017,-1.,0.E+000));",
            "#326 = CARTESIAN_POINT('',(1.27,-1.27,10.));",
            "#327 = DIRECTION('',(-5.551115123126E-017,-1.,-0.E+000));",
        ],
        circle_centre: [1.27, -5.07, 10.0],
        pipe_origin: [1.27, -1.27, 10.0],
        leaning: [-5.551_115_123_126e-17, -1.0, 0.0],
    },
    CircleOnAPipe {
        file: "Relay_3PST_COTO_3650.step",
        lines: [
            "#1361 = CARTESIAN_POINT('',(-1.54,2.302,11.06));",
            "#1362 = DIRECTION('',(2.22044604925E-16,1.,-0.));",
            "#1367 = CARTESIAN_POINT('',(-1.54,-13.462,11.06));",
            "#1368 = DIRECTION('',(2.22044604925E-16,1.,0.));",
        ],
        circle_centre: [-1.54, 2.302, 11.06],
        pipe_origin: [-1.54, -13.462, 11.06],
        leaning: [2.220_446_049_25e-16, 1.0, 0.0],
    },
];

/// The circle's centre is off the pipe's axis, exactly, and the file imports.
///
/// The centre and the pipe's origin share `x` and `z`, so the line between
/// them runs along `y` alone, and its cross product with the pipe's direction
/// has one term that is not zero: `Δy · dx`, a product of two numbers that are
/// not zero. So the centre is off the axis by `|Δy · dx| / |d|` — `2.1e-16`
/// for the LED, `3.5e-15` for the relay — and the circle, which has the pipe's
/// radius and turns about the pipe's direction, is that far off the pipe at
/// every point. No rounding in this test decides that: the two zero terms are
/// differences of equal numbers, and a product of two nonzero doubles of
/// these sizes cannot round to zero.
#[test]
fn a_circle_every_modeller_meant_on_its_pipe_is_off_it_and_the_file_imports() {
    for case in CASES {
        let path = corpus().join("kicad").join(case.file);
        let text = fs::read(&path).expect("the file is in the pinned corpus");
        let written = String::from_utf8_lossy(&text);
        for line in case.lines {
            assert!(
                written.lines().any(|at| at == line),
                "{} no longer carries `{line}`, so this test is not reading its circle",
                case.file
            );
        }

        let [cx, cy, cz] = case.circle_centre;
        let [ox, oy, oz] = case.pipe_origin;
        let [dx, dy, dz] = case.leaning;
        assert_eq!((cx - ox, cz - oz, dz), (0.0, 0.0, 0.0));
        let along_y = cy - oy;
        assert!(along_y != 0.0 && dx != 0.0 && dy != 0.0);
        let off_the_axis = (along_y * dx).abs() / (dx * dx + dy * dy).sqrt();
        assert!(
            off_the_axis > 1e-16,
            "{}: the centre is {off_the_axis} off the pipe's axis",
            case.file
        );

        let file = StepFile::read(&text).expect("every file in the corpus parses");
        let mut log = ProvenanceLog::new();
        if let Err(why) = Import::read(&file, &Policy::certified(), &mut log) {
            panic!(
                "{} does not import ({why}), and its circle is only {off_the_axis} off its pipe",
                case.file
            );
        }
    }
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!("RCAD_STEP_CORPUS is not set; this test does not skip on a missing corpus")
    });
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "RCAD_STEP_CORPUS points at {}, which is not a directory",
        root.display()
    );
    root
}
