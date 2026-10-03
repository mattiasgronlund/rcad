//! Two circles round a torus's tube, each leaning `1.04e-13` off the tube
//! circle it is drawn on, and the file that holds them imports.
//!
//! `kicad/Relay_1-Form-C_Schrack-RYII_RM3.2mm.step` bends each of its two
//! pins round a quarter of a torus, and closes each bend with a `CIRCLE` of
//! the tube's radius round the tube: edge `#1374`, circle `#1375`, on torus
//! `#1558`, and edge `#1567`, circle `#1568`, on torus `#1588`. Each circle's
//! axis leans `1.04e-13` off the direction round the ring, so it is its tube
//! circle turned by that much.
//!
//! The reader's bound for a circle round a tube charged that turn as
//! `2·r·sin φ`, twice what it costs a point for a small turn, and read the
//! two circles as `19.43` and `19.31` roundings off against an allowance of
//! sixteen, so the file was refused. A point moves `2·r·sin(φ/2)` when its
//! circle turns by `φ`, which is at most `√2·r·sin φ` for every turn the
//! bound can be handed (`docs/decisions.md` §387; Q563).
//!
//! The numbers below are the file's own lines, so a corpus that drifted from
//! them fails here rather than passing on some other circle.

use std::path::PathBuf;
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};

const FILE: &str = "Relay_1-Form-C_Schrack-RYII_RM3.2mm.step";

/// Each circle's placement and its torus's, as the file writes them. Both
/// circles turn about a direction within `1.04e-13` of the **opposite** of
/// the way round the ring at their centre, so the turn the bound charges is
/// past a right angle and reads its sine as that of the small one.
const LINES: [&str; 12] = [
    "#1375 = CIRCLE('',#1376,0.25);",
    "#1377 = CARTESIAN_POINT('',(-0.9,1.56,12.05));",
    "#1378 = DIRECTION('',(-1.,-1.039781074449E-013,0.E+000));",
    "#1558 = TOROIDAL_SURFACE('',#1559,0.75,0.25);",
    "#1560 = CARTESIAN_POINT('',(-0.9,2.31,12.05));",
    "#1561 = DIRECTION('',(0.E+000,0.E+000,1.));",
    "#1568 = CIRCLE('',#1569,0.25);",
    "#1570 = CARTESIAN_POINT('',(-0.15,5.31,12.05));",
    "#1571 = DIRECTION('',(1.033119736301E-013,-1.,0.E+000));",
    "#1588 = TOROIDAL_SURFACE('',#1589,0.75,0.25);",
    "#1590 = CARTESIAN_POINT('',(-0.9,5.31,12.05));",
    "#1591 = DIRECTION('',(0.E+000,0.E+000,1.));",
];

#[test]
fn a_relay_whose_pin_bends_round_two_tori_imports() {
    let text = fs::read(corpus().join("kicad").join(FILE)).expect("the relay is in the corpus");
    let written = String::from_utf8_lossy(&text);
    for line in LINES {
        assert!(
            written.lines().any(|at| at == line),
            "{FILE} no longer carries `{line}`, so this test is not reading its circles"
        );
    }

    let file = StepFile::read(&text).expect("the relay parses");
    let mut log = ProvenanceLog::new();
    if let Err(why) = Import::read(&file, &Policy::certified(), &mut log) {
        panic!(
            "{FILE} does not import ({why}: {why:?}); its circles #1375 and #1568 lean 1.04e-13 \
             off the tube circles of their tori"
        );
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
