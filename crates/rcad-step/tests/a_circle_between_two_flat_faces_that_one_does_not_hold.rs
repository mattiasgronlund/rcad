//! Two files whose closed edge is a circle between two flat faces, where one
//! of the two does not hold the file's circle — `Walk::every_face_holds_the_curve`
//! (`crates/rcad-step/src/brep.rs`) says no, and the reader keeps the run's
//! own plane instead of naming a cylinder, as Q574 changed it to.
//!
//! **A bar of radius 3 from `z = 0` to `4`, its lid split by a disc tilted
//! through the circle's corner**, normal `(-0.1, 0, 1)` — Q386's shape,
//! written here as a file rather than built through `SolidBuilder`. The two
//! flat faces, the lid's annulus and the tilted disc, are on two different
//! planes, and the circle between them is the rim a pipe of radius 1 about
//! the bar's axis would cross in, but no pipe is written here: the edge's
//! only surface to go by is the two faces, and they put it in two places.
//!
//! **A slab whose lid and the disc on it sit on the very same plane, with the
//! circle between them tilted off that plane.** Two faces on one plane never
//! disagree about where a closed edge between them goes — see
//! `a_closed_circle_between_two_faces_on_the_one_plane_names_its_cylinder` in
//! `a_circle_a_file_draws_round_in_one_edge.rs` — so this is not Q386's shape:
//! both faces hold the circle's own shared plane, `z = TOP`, and it is the
//! `CIRCLE` entity's own placement that leans off it instead. The reader has
//! nothing to name the run by but that leaning plane, and the two identical
//! faces either side of it are flat.
//!
//! §336.3 of the decisions has what still depends on an edge's first face,
//! which neither of these two shapes is: here the two faces disagree outright
//! (or the run's own surface disagrees with both), not by an ordering.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{Error, Invalid};
use written_by_hand::{a_header, real, triple};

/// Q386's bar, written as a file: refused `TheSolidTheFileDescribesIsNotOne`,
/// wrapping `Invalid::ClosedEdgeBetweenTwoFlatFaces` — the plain variant, and
/// **not** `ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes`, which that
/// variant's own doc comment cites this very bar for. The difference is where
/// the curved surface comes from: the topo-level pin in
/// `an_edge_between_two_planes_its_curve_leaves.rs` hands
/// `SolidBuilder` a run that names the pipe directly
/// (`EdgeRun::RightRoundWhereItCrosses { and: pipe(1.0) }`), a surface that
/// is not either face's own. A reader has no such pipe to hand: with the lid's
/// annulus and the tilted disc both flat, `Walk::every_face_holds_the_curve`
/// can only name a cylinder when the circle lies on **every** face along it,
/// which two genuinely different planes can never both do, so the run keeps
/// the circle's own plane and `nothing_curved_says_where_it_bends` finds
/// nothing curved anywhere along the edge — the plain variant, same as the
/// slab below. `ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes` has no file
/// in this crate's tests reaching it; every reachable case it is for needs a
/// curved face actually along the edge, which is `AnArcBetweenTwoFlatFacesOnDifferentPlanes`'s
/// territory, not a closed circle's.
#[test]
fn a_bar_whose_lid_is_split_by_a_disc_tilted_through_the_rims_corner_is_not_a_solid_this_kernel_can_build()
 {
    let text = a_bar_with_its_lid_split_by_a_tilted_disc();
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        matches!(
            &refused,
            Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. })
                if matches!(**why, Error::Invalid(Invalid::ClosedEdgeBetweenTwoFlatFaces))
        ),
        "the bar with its lid split by a disc tilted through the rim's corner came back {:?}",
        refused.map(|import| import.solids().len())
    );
}

/// The slab whose disc sits on the lid's own plane, with the circle leaning
/// off it: refused `TheSolidTheFileDescribesIsNotOne`, wrapping
/// `Invalid::ClosedEdgeBetweenTwoFlatFaces` — both faces are the one plane,
/// and the run names a plane too, so nothing along the edge is curved.
#[test]
fn a_circle_leaning_off_the_one_plane_both_its_flat_faces_sit_on_is_not_a_solid_this_kernel_can_build()
 {
    let text = a_slab_with_a_circle_leaning_off_the_plane_both_its_faces_sit_on();
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        matches!(
            &refused,
            Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. })
                if matches!(**why, Error::Invalid(Invalid::ClosedEdgeBetweenTwoFlatFaces))
        ),
        "the slab with its circle leaning off the shared plane came back {:?}",
        refused.map(|import| import.solids().len())
    );
}

/// A bar of radius 3 up `z` from 0 to 4. The lid, at `z = 4`, is split by a
/// circle of radius 1 about the bar's axis into an annulus on the lid's own
/// plane and a disc on the plane through the circle's corner `(1, 0, 4)`
/// leaning a tenth along `x` — `Plane::from_point_and_normal((1, 0, 4),
/// (-0.1, 0, 1))` in `an_edge_between_two_planes_its_curve_leaves.rs`.
///
/// The side is one `CYLINDRICAL_SURFACE` with a seam up `x = 3, y = 0`; the
/// bottom and the annulus are `PLANE`s normal to `z`; the disc is a third
/// `PLANE`, tilted, through the circle's own corner vertex.
fn a_bar_with_its_lid_split_by_a_tilted_disc() -> String {
    // (level, radius), in the order the circle's own number range below walks:
    // the lid's inner rim, the lid's outer rim (and the side's top), and the
    // bar's foot (and the side's bottom).
    let rings: [(f64, f64); 3] = [(4.0, 1.0), (4.0, 3.0), (0.0, 3.0)];

    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.,0.,1.));\n";
    text += "#21=DIRECTION('',(1.,0.,0.));\n";
    text += "#22=DIRECTION('',(0.,0.,-1.));\n";
    text += &format!("#23=VECTOR('',#20,{});\n", real(1.0));

    for (which, (level, radius)) in rings.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            100 + which,
            triple([*radius, 0.0, *level])
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            120 + which,
            triple([0.0, 0.0, *level])
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

    // The seam up the side, from the foot's vertex #112 to the lid outer
    // rim's vertex #111.
    text += "#160=LINE('',#102,#23);\n";
    text += "#170=EDGE_CURVE('',#112,#111,#160,.T.);\n";

    // The side: a cylinder of radius 3 about the bar's own axis.
    text += "#211=CYLINDRICAL_SURFACE('',#131,3.);\n";
    text += "#241=ORIENTED_EDGE('',*,*,#152,.T.);\n";
    text += "#242=ORIENTED_EDGE('',*,*,#170,.T.);\n";
    text += "#243=ORIENTED_EDGE('',*,*,#151,.F.);\n";
    text += "#244=ORIENTED_EDGE('',*,*,#170,.F.);\n";
    text += "#245=EDGE_LOOP('',(#241,#242,#243,#244));\n";
    text += "#246=FACE_OUTER_BOUND('',#245,.T.);\n";
    text += "#253=ADVANCED_FACE('',(#246),#211,.T.);\n";

    // The bar's foot, a full disc of radius 3 at z = 0, normal −z.
    text += "#134=AXIS2_PLACEMENT_3D('',#122,#22,#21);\n";
    text += "#266=PLANE('',#134);\n";
    text += "#267=ORIENTED_EDGE('',*,*,#152,.F.);\n";
    text += "#268=EDGE_LOOP('',(#267));\n";
    text += "#269=FACE_OUTER_BOUND('',#268,.T.);\n";
    text += "#270=ADVANCED_FACE('',(#269),#266,.T.);\n";

    // The lid's annulus: the outer rim bounds it, the inner rim is its hole.
    text += "#261=PLANE('',#131);\n";
    text += "#262=ORIENTED_EDGE('',*,*,#151,.T.);\n";
    text += "#263=EDGE_LOOP('',(#262));\n";
    text += "#264=FACE_OUTER_BOUND('',#263,.T.);\n";
    text += "#280=ORIENTED_EDGE('',*,*,#150,.F.);\n";
    text += "#281=EDGE_LOOP('',(#280));\n";
    text += "#282=FACE_BOUND('',#281,.T.);\n";
    text += "#265=ADVANCED_FACE('',(#264,#282),#261,.T.);\n";

    // The disc, on the plane through the inner rim's own corner vertex
    // #100 = (1, 0, 4), leaning a tenth of its radius along x.
    text += "#136=DIRECTION('',(-0.1,0.,1.));\n";
    text += "#135=AXIS2_PLACEMENT_3D('',#100,#136,$);\n";
    text += "#290=PLANE('',#135);\n";
    text += "#291=ORIENTED_EDGE('',*,*,#150,.T.);\n";
    text += "#292=EDGE_LOOP('',(#291));\n";
    text += "#293=FACE_OUTER_BOUND('',#292,.T.);\n";
    text += "#294=ADVANCED_FACE('',(#293),#290,.T.);\n";

    text += "#500=CLOSED_SHELL('',(#253,#270,#265,#294));\n";
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

const CENTRE: [f64; 2] = [5.0, 5.0];
const RADIUS: f64 = 2.0;
const TOP: f64 = 2.0;

/// A 10 × 10 × 2 slab, exactly §176.2's and
/// `a_circle_a_file_draws_round_in_one_edge.rs`'s own
/// `a_slab_with_a_circle_drawn_round_on_top(TheOneTheFaceRoundItSitsOn)`,
/// except the circle's own `AXIS2_PLACEMENT_3D` leans a tenth along `y` —
/// `(0, -0.1, 1)` rather than `(0, 0, 1)` — while the disc keeps the lid's own
/// `PLANE` entity, `#461`, the way the one-plane variant there does. The lean
/// is across `y`, square to the circle's own reference direction `(1, 0, 0)`,
/// so the vertex at `(x + RADIUS, y, TOP)` stays exactly on the leaning
/// circle: that reference direction is already perpendicular to the leaning
/// normal, so it is the leaning circle's own first axis unchanged, and the
/// point at its parameter zero does not move.
fn a_slab_with_a_circle_leaning_off_the_plane_both_its_faces_sit_on() -> String {
    let corners = [
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, TOP],
        [10.0, 0.0, TOP],
        [10.0, 10.0, TOP],
        [0.0, 10.0, TOP],
    ];
    let edges: [[usize; 2]; 12] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    let faces: [([usize; 4], [f64; 3], usize); 6] = [
        ([0, 3, 2, 1], [0.0, 0.0, -1.0], 0),
        ([4, 5, 6, 7], [0.0, 0.0, 1.0], 4),
        ([0, 1, 5, 4], [0.0, -1.0, 0.0], 0),
        ([3, 7, 6, 2], [0.0, 1.0, 0.0], 3),
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0], 0),
        ([1, 2, 6, 5], [1.0, 0.0, 0.0], 1),
    ];
    let [x, y] = CENTRE;

    let mut text = a_header(".MILLI.");
    for (which, corner) in corners.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            100 + which,
            triple(*corner)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
    }
    text += &format!(
        "#120=CARTESIAN_POINT('',{});\n",
        triple([x + RADIUS, y, TOP])
    );
    text += "#125=VERTEX_POINT('',#120);\n";
    for (which, [from, to]) in edges.iter().enumerate() {
        let way: Vec<f64> = (0..3)
            .map(|axis| {
                let step = corners[*to][axis] - corners[*from][axis];
                if step == 0.0 { 0.0 } else { step.signum() }
            })
            .collect();
        text += &format!(
            "#{}=DIRECTION('',{});\n",
            340 + which,
            triple([way[0], way[1], way[2]])
        );
        text += &format!(
            "#{}=VECTOR('',#{},{});\n",
            320 + which,
            340 + which,
            real(1.0)
        );
        text += &format!(
            "#{}=LINE('',#{},#{});\n",
            300 + which,
            100 + from,
            320 + which
        );
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            380 + which,
            110 + from,
            110 + to,
            300 + which
        );
    }
    text += &format!("#600=CARTESIAN_POINT('',{});\n", triple([x, y, TOP]));
    text += "#601=DIRECTION('',(0.,-0.1,1.));\n";
    text += "#602=DIRECTION('',(1.,0.,0.));\n";
    text += "#603=AXIS2_PLACEMENT_3D('',#600,#601,#602);\n";
    text += &format!("#604=CIRCLE('',#603,{});\n", real(RADIUS));
    text += "#610=EDGE_CURVE('',#125,#125,#604,.T.);\n";

    for (which, (round, normal, origin)) in faces.iter().enumerate() {
        for step in 0..4 {
            let (from, to) = (round[step], round[(step + 1) % 4]);
            let (edge, forwards) = which_edge(&edges, from, to);
            text += &format!(
                "#{}=ORIENTED_EDGE('',*,*,#{},{});\n",
                400 + which * 4 + step,
                380 + edge,
                if forwards { ".T." } else { ".F." }
            );
        }
        let steps: Vec<String> = (0..4)
            .map(|step| format!("#{}", 400 + which * 4 + step))
            .collect();
        text += &format!("#{}=EDGE_LOOP('',({}));\n", 430 + which, steps.join(","));
        text += &format!(
            "#{}=FACE_OUTER_BOUND('',#{},.T.);\n",
            440 + which,
            430 + which
        );
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            470 + which,
            triple(corners[*origin])
        );
        text += &format!("#{}=DIRECTION('',{});\n", 480 + which, triple(*normal));
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#{},$);\n",
            460 + which,
            470 + which,
            480 + which
        );
        text += &format!("#{}=PLANE('',#{});\n", 450 + which, 460 + which);
        let bounds = if which == 1 {
            format!("#{},#711", 440 + which)
        } else {
            format!("#{}", 440 + which)
        };
        text += &format!(
            "#{}=ADVANCED_FACE('',({bounds}),#{},.T.);\n",
            490 + which,
            450 + which
        );
    }

    text += "#700=ORIENTED_EDGE('',*,*,#610,.F.);\n";
    text += "#710=EDGE_LOOP('',(#700));\n";
    text += "#711=FACE_BOUND('',#710,.T.);\n";
    text += "#720=ORIENTED_EDGE('',*,*,#610,.T.);\n";
    text += "#730=EDGE_LOOP('',(#720));\n";
    text += "#731=FACE_OUTER_BOUND('',#730,.T.);\n";
    // The disc keeps the lid's own PLANE, #461: the same shared plane on
    // both sides of the circle, which the circle's own placement above leans
    // off.
    text += "#732=PLANE('',#461);\n";
    text += "#733=ADVANCED_FACE('',(#731),#732,.T.);\n";

    text += "#500=CLOSED_SHELL('',(#490,#491,#492,#493,#494,#495,#733));\n";
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

fn which_edge(edges: &[[usize; 2]; 12], from: usize, to: usize) -> (usize, bool) {
    for (which, ends) in edges.iter().enumerate() {
        if *ends == [from, to] {
            return (which, true);
        }
        if *ends == [to, from] {
            return (which, false);
        }
    }
    panic!("the slab's loops walk only its own edges, and {from} to {to} is not one");
}
