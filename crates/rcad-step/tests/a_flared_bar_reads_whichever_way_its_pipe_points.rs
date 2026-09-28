//! A round bar that flares out into a cone, read from a file, whichever way the
//! file writes its pipe's axis and whichever face its shell lists first.
//!
//! The rim between the pipe and the cone is a circle both of them go right
//! round, and the reader writes it as one closed edge that names the circle's
//! own plane (`EdgeRun::RightRoundWhereItCrosses`). Its two faces are both
//! curved and differ, so the plane is read with one of them — and until
//! `docs/decisions.md` §336 that was whichever face the caller listed first.
//! `Model::validate` lists first the face whose coedge it is checking, so the
//! pipe's coedge was checked along the pipe's circle and the cone's along the
//! cone's, each winding by its own surface's axis. With the pipe's axis written
//! pointing down and the cone's up, the two could not both pass.
//!
//! Measured on the base `96da3252`: with the pipe's axis at `-z` the file was
//! refused `AClosedEdgeIsWalkedAgainstItsSurface` in both shell orders, and at
//! `+z` it read; the rim measured `6π` with the pipe listed first and was
//! refused `CannotYetMeasureHowLongThisCurveIs` with the cone first.
//!
//! Every number is exact in binary: the pipe has radius `3` from `z = 0` to
//! `z = 4`, and the cone has its apex at the origin and widens three for every
//! four up, to radius `6` at `z = 8`.

mod written_by_hand;

use core::f64::consts::PI;

use rcad_geom::{EdgeRun, Surface};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Model, Rule, SolidKey};
use written_by_hand::{a_header, real};

/// The rim between the pipe and the cone, as `(height, radius)`.
const RIM: (f64, f64) = (4.0, 3.0);
/// The flat top of the cone, as `(height, radius)`.
const TOP: (f64, f64) = (8.0, 6.0);
/// The flat bottom of the pipe, as `(height, radius)`.
const BOTTOM: (f64, f64) = (0.0, 3.0);

/// Which way the file writes the pipe's axis.
#[derive(Debug, Clone, Copy)]
enum PipeAxis {
    Up,
    Down,
}

/// Which face the file's shell lists first.
#[derive(Debug, Clone, Copy)]
enum FirstInTheShell {
    Cone,
    Pipe,
}

const EVERY_WAY_IT_IS_WRITTEN: [(PipeAxis, FirstInTheShell); 4] = [
    (PipeAxis::Up, FirstInTheShell::Cone),
    (PipeAxis::Up, FirstInTheShell::Pipe),
    (PipeAxis::Down, FirstInTheShell::Cone),
    (PipeAxis::Down, FirstInTheShell::Pipe),
];

/// The file reads, and the model it becomes passes the full sweep, however it
/// is written.
///
/// Fails on the base with the pipe's axis at `-z`, in both shell orders.
#[test]
fn a_flared_bar_is_one_sound_solid_however_the_file_writes_it() {
    for (axis, first) in EVERY_WAY_IT_IS_WRITTEN {
        let what = format!("the pipe's axis {axis:?}, the {first:?} listed first");
        let (model, _) = read(&a_flared_bar(axis, first), &what);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: the model does not validate: {why:?}"));
    }
}

/// The rim is a circle of radius `3`, so it is `6π` long, from whichever face
/// the model reaches it.
///
/// Fails on the base with the cone listed first: the plane was read with the
/// cone, and a cone's circle is not one the length has a closed form for.
#[test]
fn the_rim_between_the_pipe_and_the_cone_measures_six_pi_however_the_file_is_written() {
    let (_, radius) = RIM;
    let closed_form = 2.0 * PI * radius;
    for (axis, first) in EVERY_WAY_IT_IS_WRITTEN {
        let what = format!("the pipe's axis {axis:?}, the {first:?} listed first");
        let (model, key) = read(&a_flared_bar(axis, first), &what);
        let solid = model.solid(key).expect("this model made the key");
        let rims: Vec<_> = solid
            .faces()
            .flat_map(|face| face.wires())
            .flat_map(|wire| wire.coedges())
            .map(|coedge| coedge.edge())
            .filter(|edge| {
                matches!(
                    edge.run(),
                    EdgeRun::RightRoundWhereItCrosses {
                        and: Surface::Plane(_)
                    }
                )
            })
            .collect();
        assert_eq!(
            rims.len(),
            2,
            "{what}: the rim is one edge, reached once from each of its two faces"
        );
        for rim in rims {
            let length = rim
                .length()
                .unwrap_or_else(|why| panic!("{what}: the rim has no length: {why:?}"));
            assert!(
                (length.length() - closed_form).abs() <= length.error(),
                "{what}: the rim measured {} ± {}, and 6π is {closed_form}",
                length.length(),
                length.error()
            );
        }
    }
}

/// **The guard for row Q41.** Two round bars of radius `3` on one axis, from
/// `z = 0` to `4` and from `z = 2` to `6`, read from one file with their pipes'
/// axes written opposite ways, join into one sound solid of volume `54π`.
///
/// This passes on the base as well as with §336's change; it is here because
/// §336 measured Q41's change alone breaking it. The union's rims run between
/// the two bars' pipes, which are one surface written two ways, so which of the
/// two the rim's plane is read with is the question §336 settled.
#[test]
fn two_bars_on_one_axis_with_their_pipes_written_opposite_ways_join_into_one_sound_solid() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let text = two_bars_on_one_axis();
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let import = Import::read(&file, &policy, &mut log).expect("two round bars read");
    let [(_, lower), (_, upper)] = import.solids() else {
        panic!("two MANIFOLD_SOLID_BREPs are two solids");
    };
    let (lower, upper) = (*lower, *upper);
    let mut model = import.into_model();
    let joined = model
        .fuse(&[lower], &[upper], Rule::Union, &policy, &mut log)
        .expect("two bars on one axis join");
    let [solid] = joined.solids() else {
        panic!("two overlapping bars join into one solid");
    };
    model
        .validate(&policy, &mut log)
        .expect("the joined bars are a sound model");

    let deflection = 0.01;
    let measured = model
        .solid(*solid)
        .expect("the fuse made the key")
        .measure(
            rcad_geom::Deflection::of(deflection).expect("a length"),
            &policy,
            &mut log,
        )
        .expect("the joined bars measure");
    let closed_form = PI * 3.0 * 3.0 * 6.0;
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "the joined bars measured {} ± {}, and 54π is {closed_form}",
        measured.volume(),
        measured.volume_error()
    );
}

fn read(text: &str, what: &str) -> (Model, SolidKey) {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let import = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the file does not read: {why:?}"));
    let [(_, key)] = import.solids() else {
        panic!("{what}: one MANIFOLD_SOLID_BREP is one solid");
    };
    let key = *key;
    (import.into_model(), key)
}

/// The pipe from `BOTTOM` to `RIM`, the cone band from `RIM` to `TOP`, and a
/// flat disc on each end.
///
/// Each band's loop runs round its lower circle, up its seam, back round its
/// upper circle and down the seam again; each disc walks its circle the way
/// the band beside it does not. The cone's placement is the rim's, with its
/// axis up, whichever way the pipe's is written.
fn a_flared_bar(axis: PipeAxis, first: FirstInTheShell) -> String {
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.,0.,1.));\n";
    text += "#21=DIRECTION('',(1.,0.,0.));\n";
    text += "#22=DIRECTION('',(0.,0.,-1.));\n";
    text += "#24=DIRECTION('',(3.,0.,4.));\n";
    text += &format!("#23=VECTOR('',#24,{});\n", real(1.0));
    text += &format!("#26=VECTOR('',#20,{});\n", real(1.0));
    for (which, (level, radius)) in [RIM, TOP, BOTTOM].iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',({},0.,{}));\n",
            100 + which,
            real(*radius),
            real(*level)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
        text += &format!(
            "#{}=CARTESIAN_POINT('',(0.,0.,{}));\n",
            120 + which,
            real(*level)
        );
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#20,#21);\n",
            130 + which,
            120 + which
        );
        text += &format!(
            "#{}=CIRCLE('',#{},{});\n",
            140 + which,
            130 + which,
            real(*radius)
        );
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            150 + which,
            110 + which,
            110 + which,
            140 + which
        );
    }
    text += "#160=LINE('',#100,#23);\n";
    text += "#170=EDGE_CURVE('',#110,#111,#160,.T.);\n";
    text += "#161=LINE('',#102,#26);\n";
    text += "#171=EDGE_CURVE('',#112,#110,#161,.T.);\n";

    text += &format!(
        "#210=CONICAL_SURFACE('',#130,{},{});\n",
        real(RIM.1),
        real(0.75_f64.atan())
    );
    text += "#220=ORIENTED_EDGE('',*,*,#150,.T.);\n";
    text += "#221=ORIENTED_EDGE('',*,*,#170,.T.);\n";
    text += "#222=ORIENTED_EDGE('',*,*,#151,.F.);\n";
    text += "#223=ORIENTED_EDGE('',*,*,#170,.F.);\n";
    text += "#230=EDGE_LOOP('',(#220,#221,#222,#223));\n";
    text += "#235=FACE_OUTER_BOUND('',#230,.T.);\n";
    text += "#240=ADVANCED_FACE('',(#235),#210,.T.);\n";

    let pipe_axis = match axis {
        PipeAxis::Up => "#20",
        PipeAxis::Down => "#22",
    };
    text += &format!("#133=AXIS2_PLACEMENT_3D('',#122,{pipe_axis},#21);\n");
    text += &format!("#211=CYLINDRICAL_SURFACE('',#133,{});\n", real(RIM.1));
    text += "#241=ORIENTED_EDGE('',*,*,#152,.T.);\n";
    text += "#242=ORIENTED_EDGE('',*,*,#171,.T.);\n";
    text += "#243=ORIENTED_EDGE('',*,*,#150,.F.);\n";
    text += "#244=ORIENTED_EDGE('',*,*,#171,.F.);\n";
    text += "#245=EDGE_LOOP('',(#241,#242,#243,#244));\n";
    text += "#246=FACE_OUTER_BOUND('',#245,.T.);\n";
    text += "#253=ADVANCED_FACE('',(#246),#211,.T.);\n";

    text += "#261=PLANE('',#131);\n";
    text += "#262=ORIENTED_EDGE('',*,*,#151,.T.);\n";
    text += "#263=EDGE_LOOP('',(#262));\n";
    text += "#264=FACE_OUTER_BOUND('',#263,.T.);\n";
    text += "#265=ADVANCED_FACE('',(#264),#261,.T.);\n";

    text += "#134=AXIS2_PLACEMENT_3D('',#122,#22,#21);\n";
    text += "#266=PLANE('',#134);\n";
    text += "#267=ORIENTED_EDGE('',*,*,#152,.F.);\n";
    text += "#268=EDGE_LOOP('',(#267));\n";
    text += "#269=FACE_OUTER_BOUND('',#268,.T.);\n";
    text += "#270=ADVANCED_FACE('',(#269),#266,.T.);\n";

    text += match first {
        FirstInTheShell::Cone => "#500=CLOSED_SHELL('',(#240,#253,#265,#270));\n",
        FirstInTheShell::Pipe => "#500=CLOSED_SHELL('',(#253,#240,#265,#270));\n",
    };
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

/// Two round bars of radius `3` on the `z` axis, the lower from `z = 0` to `4`
/// with its pipe's axis written up, the upper from `z = 2` to `6` with its
/// pipe's axis written down.
fn two_bars_on_one_axis() -> String {
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.,0.,1.));\n";
    text += "#21=DIRECTION('',(1.,0.,0.));\n";
    text += "#22=DIRECTION('',(0.,0.,-1.));\n";
    text += &format!("#26=VECTOR('',#20,{});\n", real(1.0));
    text += &a_round_bar(1000, (0.0, 4.0), PipeAxis::Up);
    text += &a_round_bar(2000, (2.0, 6.0), PipeAxis::Down);
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

/// A round bar of radius `3` on the `z` axis from `bottom` to `top`, its
/// entities numbered from `base`: a pipe band, a disc on top facing up and one
/// underneath facing down.
fn a_round_bar(base: usize, (bottom, top): (f64, f64), axis: PipeAxis) -> String {
    let n = |offset: usize| base + offset;
    let mut text = String::new();
    for (which, level) in [bottom, top].iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',(3.,0.,{}));\n",
            n(which),
            real(*level)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", n(10 + which), n(which));
        text += &format!(
            "#{}=CARTESIAN_POINT('',(0.,0.,{}));\n",
            n(20 + which),
            real(*level)
        );
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#20,#21);\n",
            n(30 + which),
            n(20 + which)
        );
        text += &format!("#{}=CIRCLE('',#{},3.);\n", n(40 + which), n(30 + which));
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            n(50 + which),
            n(10 + which),
            n(10 + which),
            n(40 + which)
        );
    }
    text += &format!("#{}=LINE('',#{},#26);\n", n(60), n(0));
    text += &format!(
        "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
        n(61),
        n(10),
        n(11),
        n(60)
    );
    let pipe_axis = match axis {
        PipeAxis::Up => "#20",
        PipeAxis::Down => "#22",
    };
    text += &format!(
        "#{}=AXIS2_PLACEMENT_3D('',#{},{pipe_axis},#21);\n",
        n(70),
        n(20)
    );
    text += &format!("#{}=CYLINDRICAL_SURFACE('',#{},3.);\n", n(71), n(70));
    text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.T.);\n", n(72), n(50));
    text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.T.);\n", n(73), n(61));
    text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.F.);\n", n(74), n(51));
    text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.F.);\n", n(75), n(61));
    text += &format!(
        "#{}=EDGE_LOOP('',(#{},#{},#{},#{}));\n",
        n(76),
        n(72),
        n(73),
        n(74),
        n(75)
    );
    text += &format!("#{}=FACE_OUTER_BOUND('',#{},.T.);\n", n(77), n(76));
    text += &format!("#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n", n(78), n(77), n(71));

    text += &format!("#{}=PLANE('',#{});\n", n(80), n(31));
    text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.T.);\n", n(81), n(51));
    text += &format!("#{}=EDGE_LOOP('',(#{}));\n", n(82), n(81));
    text += &format!("#{}=FACE_OUTER_BOUND('',#{},.T.);\n", n(83), n(82));
    text += &format!("#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n", n(84), n(83), n(80));

    text += &format!(
        "#{}=AXIS2_PLACEMENT_3D('',#{},#22,#21);\n",
        n(85),
        n(20)
    );
    text += &format!("#{}=PLANE('',#{});\n", n(86), n(85));
    text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.F.);\n", n(87), n(50));
    text += &format!("#{}=EDGE_LOOP('',(#{}));\n", n(88), n(87));
    text += &format!("#{}=FACE_OUTER_BOUND('',#{},.T.);\n", n(89), n(88));
    text += &format!("#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n", n(90), n(89), n(86));
    text += &format!(
        "#{}=CLOSED_SHELL('',(#{},#{},#{}));\n",
        n(91),
        n(78),
        n(84),
        n(90)
    );
    text += &format!("#{}=MANIFOLD_SOLID_BREP('',#{});\n", n(92), n(91));
    text
}
