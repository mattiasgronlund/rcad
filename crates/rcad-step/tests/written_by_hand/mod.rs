//! STEP files written here, so that what a test measures can be checked
//! against numbers nobody read off the kernel.
//!
//! Every file below is a whole AP203 `manifold_solid_brep`: header, units,
//! geometric context, closed shell, advanced faces, edge loops, oriented edges,
//! edge curves and vertex points. The point of writing them out rather than
//! exporting them from anywhere is that the closed form is known — a block of
//! sides `a`, `b`, `c` encloses `abc` and no measurement of it is being compared
//! against another measurement.
//!
//! Every test file here takes what it needs from this module and leaves the
//! rest, so what is unused in one is used in another.
#![allow(dead_code)]

/// A block with one corner at the origin and the opposite corner at
/// `(sides[0], sides[1], sides[2])`, written in `unit`.
///
/// `unit` is an `SI_UNIT` prefix as Part 21 writes one: `"$"` for metres,
/// `".MILLI."` for millimetres. The numbers are written exactly as handed over,
/// so a caller doubling them writes a file whose every coordinate is exactly
/// twice the first one's.
pub fn a_block(sides: [f64; 3], unit: &str) -> String {
    let [a, b, c] = sides;
    let corners = [
        [0.0, 0.0, 0.0],
        [a, 0.0, 0.0],
        [a, b, 0.0],
        [0.0, b, 0.0],
        [0.0, 0.0, c],
        [a, 0.0, c],
        [a, b, c],
        [0.0, b, c],
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
    // Each loop runs the way a right hand turns about the face's own outward
    // normal, which is what both STEP and this kernel ask of an outer bound.
    let faces: [([usize; 4], [f64; 3], usize); 6] = [
        ([0, 3, 2, 1], [0.0, 0.0, -1.0], 0),
        ([4, 5, 6, 7], [0.0, 0.0, 1.0], 4),
        ([0, 1, 5, 4], [0.0, -1.0, 0.0], 0),
        ([3, 7, 6, 2], [0.0, 1.0, 0.0], 3),
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0], 0),
        ([1, 2, 6, 5], [1.0, 0.0, 0.0], 1),
    ];

    let mut text = a_header(unit);
    for (which, corner) in corners.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            100 + which,
            triple(*corner)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
    }
    for (which, [from, to]) in edges.iter().enumerate() {
        let way = direction_from(corners[*from], corners[*to]);
        text += &format!("#{}=DIRECTION('',{});\n", 340 + which, triple(way));
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
        text += &format!(
            "#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n",
            490 + which,
            440 + which,
            450 + which
        );
    }
    let skin: Vec<String> = (0..6).map(|which| format!("#{}", 490 + which)).collect();
    text += &format!("#500=CLOSED_SHELL('',({}));\n", skin.join(","));
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

/// The header, the three units and the geometric context every file below
/// shares.
pub fn a_header(unit: &str) -> String {
    format!(
        "ISO-10303-21;\n\
         HEADER;\n\
         FILE_DESCRIPTION((''),'2;1');\n\
         FILE_NAME('','',(''),(''),'','','');\n\
         FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\n\
         ENDSEC;\n\
         DATA;\n\
         #1=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT({unit},.METRE.));\n\
         #2=(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.));\n\
         #3=(NAMED_UNIT(*)SOLID_ANGLE_UNIT()SI_UNIT($,.STERADIAN.));\n\
         #4=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-7),#1,'','');\n\
         #5=(GEOMETRIC_REPRESENTATION_CONTEXT(3)\
         GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#4))\
         GLOBAL_UNIT_ASSIGNED_CONTEXT((#1,#2,#3))REPRESENTATION_CONTEXT('',''));\n"
    )
}

/// Which of the twelve edges runs between two corners, and whether it runs the
/// way this loop is walking it.
fn which_edge(edges: &[[usize; 2]; 12], from: usize, to: usize) -> (usize, bool) {
    for (which, ends) in edges.iter().enumerate() {
        if *ends == [from, to] {
            return (which, true);
        }
        if *ends == [to, from] {
            return (which, false);
        }
    }
    panic!("the block's loops walk only its own edges, and {from} to {to} is not one");
}

/// A unit direction along one of the block's own axes, so nothing rounds.
fn direction_from(from: [f64; 3], to: [f64; 3]) -> [f64; 3] {
    let mut way = [0.0; 3];
    for axis in 0..3 {
        way[axis] = (to[axis] - from[axis]).signum() * f64::from(u8::from(to[axis] != from[axis]));
    }
    way
}

/// Three numbers as Part 21 writes a list of three `REAL`s, each by [`real`].
pub fn triple(numbers: [f64; 3]) -> String {
    format!(
        "({},{},{})",
        real(numbers[0]),
        real(numbers[1]),
        real(numbers[2])
    )
}

/// A number as Part 21 writes a `REAL`: always a decimal point, and an exponent
/// that carries one in front of it.
///
/// `{:?}` gives the shortest text that reads back as the same `f64`, which is
/// what makes a file written here and read back bit for bit the same numbers.
/// What it does not do is put a `.` in front of an `e`, and Part 21 has no
/// literal without one.
pub fn real(number: f64) -> String {
    let written = format!("{number:?}");
    match written.split_once('e') {
        None => written,
        Some((mantissa, exponent)) => {
            let mantissa = if mantissa.contains('.') {
                mantissa.to_owned()
            } else {
                format!("{mantissa}.")
            };
            format!("{mantissa}E{exponent}")
        }
    }
}

/// A plate whose top and bottom are quarter-discs of `radius`, standing
/// `height` tall on the origin, written in `unit`.
///
/// **Written because a block cannot fail at the things a real file fails at.**
/// It has a cylindrical face, two arcs between two corners each — which is what
/// makes an edge need a picking half-space at all — two flat faces the cylinder
/// crosses, and a seam up each side. Every number in it is exact for any radius
/// and height an `f64` holds: the corners sit at `(r, 0, z)` and `(0, r, z)`, so
/// `x² + y² − r²` cancels to nothing, and the arcs' plane is `z = const` with
/// its corners exactly on it.
///
/// **A quarter and not a half, and the reason it used to give is gone.** It read
/// that a half-disc's three corners — the apex and the two on the rim — are
/// collinear, that `check_flat_face` reads a wire's winding off its corners
/// alone, and so that the area a half-disc's arc encloses is invisible and the
/// face is refused as winding against its own normal, while a quarter's three
/// corners make a triangle. The middle step of that is no longer true:
/// `rcad-topo`'s `winding_of` winds the corners **plus the points
/// `rcad_geom::walk` puts along every curved step**, which is the same boundary
/// the mesher draws, so an arc's area is not invisible to it and collinear
/// corners are not a refusal. `a_file_bounded_by_two_half_circles.rs` is a whole
/// file that turns on exactly that.
///
/// So nothing about the winding forces a quarter here now. It stays one because
/// `πr²h/4` is what `a_quarter_disc_plate_written_by_hand_measures_what_its_numbers_say`
/// checks against and because `a_pie` shares its far corner at `(0, r, z)`
/// between the two shapes it writes — a half-turn arm wants a corner of its own,
/// which is the arm the module note below is still missing.
pub fn a_quarter_disc_plate(radius: f64, height: f64, unit: &str) -> String {
    a_pie(radius, height, unit, true)
}

/// The same plate cut down to the segment between the arc and its chord, so
/// each flat face is bounded by two steps: an arc and a straight.
///
/// **Written for a gap that is now closed, and it imports.** Two steps used to
/// be too few for `SolidBuilder::add_wire` and for `Invalid::WireTooShort` — the
/// reason 59 of the 323 KiCad files and 16 of the 44 NIST ones stopped first —
/// and both rules now take an arc and its chord (§44.11 item 3). Behind that sat
/// a second thing: this file's arc is a **quarter** turn, `rcad_geom::walk`
/// never steps an arc more than a quarter at a time and used to allow a floor of
/// one step, and the sweep meshes as coarsely as a mesh gets, so the arc
/// contributed no corner between its two ends and the flat face fanned into two.
/// The floor is now `FEWEST_STEPS_THAT_SHOW_A_BEND`, which is two, and this file
/// goes all the way through. **So this is the shape that proves the two-step
/// rule from bytes**, which is what nothing did before.
///
/// What is still missing is the *half*-turn arm — the far corner at
/// `(−r, 0, z)`, which `a_pie` currently shares with the quarter-disc, a chord
/// face facing `[0, −1, 0]`, and a row in
/// `every_file_this_kernel_wrote_itself_imports` measuring `πr²h/2`. That the
/// half-turn *solid* holds up is demonstrated already:
/// `a_face_bounded_by_two_steps.rs` builds it face by face and sweeps it with
/// nothing approximated.
pub fn a_plate_bounded_by_an_arc_and_its_chord(radius: f64, height: f64, unit: &str) -> String {
    a_pie(radius, height, unit, false)
}

/// A plate whose flat top and bottom are **bounded by two half-circles**: the
/// upper half of a circle of `outer` radius, closed by the lower half of a
/// circle of `inner` radius, with a straight along the `x` axis at each end.
///
/// A half-round plate with a half-round rib under it. Both half-circles bulge
/// away from the axis they are cut by, so both walls are cylinders with their
/// material inside them and every face is written the way it sits — nothing in
/// the file is turned round.
///
/// **All four corners sit on the `x` axis.** A polygon through four collinear
/// corners sweeps exactly nothing, so the whole of this face's area is in how
/// its two arcs bulge: `π(outer² + inner²)/2`, times `height` for the solid.
/// That is what makes the file worth writing — the STEP door adds each arc's
/// bulge and `Model::validate` reads the corners alone, and this is a shape the
/// two disagree about.
pub fn a_plate_bounded_by_two_half_circles(
    outer: f64,
    inner: f64,
    height: f64,
    unit: &str,
) -> String {
    let corners = [
        [outer, 0.0, 0.0],
        [inner, 0.0, 0.0],
        [-inner, 0.0, 0.0],
        [-outer, 0.0, 0.0],
        [outer, 0.0, height],
        [inner, 0.0, height],
        [-inner, 0.0, height],
        [-outer, 0.0, height],
    ];
    // An arc runs from its first corner to its second the way the circle turns,
    // so `v0 → v3` is the half over `y > 0` and `v2 → v1` the half under it.
    let edges: [Edge; 12] = [
        (0, 3, Some(960)),
        (3, 2, None),
        (2, 1, Some(961)),
        (1, 0, None),
        (4, 7, Some(962)),
        (7, 6, None),
        (6, 5, Some(963)),
        (5, 4, None),
        (0, 4, None),
        (1, 5, None),
        (2, 6, None),
        (3, 7, None),
    ];
    let faces: [(Loop, Face); 6] = [
        (
            vec![(3, false), (2, false), (1, false), (0, false)],
            Face::Flat([0.0, 0.0, -1.0], [0.0, 0.0, 0.0]),
        ),
        (
            vec![(4, true), (5, true), (6, true), (7, true)],
            Face::Flat([0.0, 0.0, 1.0], [0.0, 0.0, height]),
        ),
        (
            vec![(0, true), (11, true), (4, false), (8, false)],
            Face::Round(outer),
        ),
        (
            vec![(2, true), (9, true), (6, false), (10, false)],
            Face::Round(inner),
        ),
        (
            vec![(3, true), (8, true), (7, false), (9, false)],
            Face::Flat([0.0, -1.0, 0.0], [0.0, 0.0, 0.0]),
        ),
        (
            vec![(1, true), (10, true), (5, false), (11, false)],
            Face::Flat([0.0, -1.0, 0.0], [0.0, 0.0, 0.0]),
        ),
    ];

    let mut text = a_header(unit);
    for (which, corner) in corners.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            900 + which,
            triple(*corner)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 910 + which, 900 + which);
    }
    for (which, radius, z) in [
        (0_usize, outer, 0.0),
        (1, inner, 0.0),
        (2, outer, height),
        (3, inner, height),
    ] {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            920 + which,
            triple([0.0, 0.0, z])
        );
        text += &format!(
            "#{}=DIRECTION('',{});\n",
            930 + which,
            triple([0.0, 0.0, 1.0])
        );
        text += &format!(
            "#{}=DIRECTION('',{});\n",
            940 + which,
            triple([1.0, 0.0, 0.0])
        );
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#{},#{});\n",
            950 + which,
            920 + which,
            930 + which,
            940 + which
        );
        text += &format!(
            "#{}=CIRCLE('',#{},{});\n",
            960 + which,
            950 + which,
            real(radius)
        );
    }
    for (which, (from, to, arc)) in edges.iter().enumerate() {
        let curve = match arc {
            Some(circle) => *circle,
            None => {
                let way = direction_from(corners[*from], corners[*to]);
                text += &format!("#{}=DIRECTION('',{});\n", 970 + which, triple(way));
                text += &format!(
                    "#{}=VECTOR('',#{},{});\n",
                    985 + which,
                    970 + which,
                    real(1.0)
                );
                text += &format!(
                    "#{}=LINE('',#{},#{});\n",
                    1000 + which,
                    900 + from,
                    985 + which
                );
                1000 + which
            }
        };
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            1020 + which,
            910 + from,
            910 + to,
            curve
        );
    }
    for (which, (round, sits_on)) in faces.iter().enumerate() {
        let mut steps = Vec::new();
        for (step, (edge, forwards)) in round.iter().enumerate() {
            let id = 1100 + which * 8 + step;
            text += &format!(
                "#{id}=ORIENTED_EDGE('',*,*,#{},{});\n",
                1020 + edge,
                if *forwards { ".T." } else { ".F." }
            );
            steps.push(format!("#{id}"));
        }
        text += &format!("#{}=EDGE_LOOP('',({}));\n", 1160 + which, steps.join(","));
        text += &format!(
            "#{}=FACE_OUTER_BOUND('',#{},.T.);\n",
            1170 + which,
            1160 + which
        );
        let (origin, axis) = match sits_on {
            Face::Flat(normal, at) => (*at, *normal),
            Face::Round(_) => ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        };
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            1180 + which,
            triple(origin)
        );
        text += &format!("#{}=DIRECTION('',{});\n", 1190 + which, triple(axis));
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#{},$);\n",
            1200 + which,
            1180 + which,
            1190 + which
        );
        text += &match sits_on {
            Face::Flat(_, _) => format!("#{}=PLANE('',#{});\n", 1210 + which, 1200 + which),
            Face::Round(radius) => format!(
                "#{}=CYLINDRICAL_SURFACE('',#{},{});\n",
                1210 + which,
                1200 + which,
                real(*radius)
            ),
        };
        text += &format!(
            "#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n",
            1220 + which,
            1170 + which,
            1210 + which
        );
    }
    let skin: Vec<String> = (0..faces.len())
        .map(|which| format!("#{}", 1220 + which))
        .collect();
    text += &format!("#1300=CLOSED_SHELL('',({}));\n", skin.join(","));
    text += "#1301=MANIFOLD_SOLID_BREP('',#1300);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

fn a_pie(radius: f64, height: f64, unit: &str, through_the_middle: bool) -> String {
    let mut corners = vec![[radius, 0.0, 0.0]];
    if through_the_middle {
        corners.push([0.0, 0.0, 0.0]);
    }
    corners.push([0.0, radius, 0.0]);
    let below = corners.len();
    for step in 0..below {
        corners.push([corners[step][0], corners[step][1], height]);
    }

    let (edges, faces): (Vec<Edge>, Vec<(Loop, Face)>) = if through_the_middle {
        (
            vec![
                (0, 2, Some(630)),
                (2, 1, None),
                (1, 0, None),
                (3, 5, Some(631)),
                (5, 4, None),
                (4, 3, None),
                (0, 3, None),
                (2, 5, None),
                (1, 4, None),
            ],
            vec![
                (
                    vec![(2, false), (1, false), (0, false)],
                    Face::Flat([0.0, 0.0, -1.0], [0.0, 0.0, 0.0]),
                ),
                (
                    vec![(3, true), (4, true), (5, true)],
                    Face::Flat([0.0, 0.0, 1.0], [0.0, 0.0, height]),
                ),
                (
                    vec![(6, true), (5, false), (8, false), (2, true)],
                    Face::Flat([0.0, -1.0, 0.0], [0.0, 0.0, 0.0]),
                ),
                (
                    vec![(1, true), (8, true), (4, false), (7, false)],
                    Face::Flat([-1.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
                ),
                (
                    vec![(0, true), (7, true), (3, false), (6, false)],
                    Face::Round(radius),
                ),
            ],
        )
    } else {
        (
            vec![
                (0, 1, Some(630)),
                (1, 0, None),
                (2, 3, Some(631)),
                (3, 2, None),
                (0, 2, None),
                (1, 3, None),
            ],
            vec![
                (
                    vec![(1, false), (0, false)],
                    Face::Flat([0.0, 0.0, -1.0], [0.0, 0.0, 0.0]),
                ),
                (
                    vec![(2, true), (3, true)],
                    Face::Flat([0.0, 0.0, 1.0], [0.0, 0.0, height]),
                ),
                // The segment's material is the arc side, `x + y >= r`, so out
                // of it is *towards* the axis. This read `[1, 1, 0]` until the
                // two-step rule let the file get far enough to be judged, and
                // then the sweep said `OuterWireWindsAgainstTheNormal`.
                (
                    vec![(4, true), (3, false), (5, false), (1, true)],
                    Face::Flat([-1.0, -1.0, 0.0], [radius, 0.0, 0.0]),
                ),
                (
                    vec![(0, true), (5, true), (2, false), (4, false)],
                    Face::Round(radius),
                ),
            ],
        )
    };

    let mut text = a_header(unit);
    for (which, corner) in corners.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            600 + which,
            triple(*corner)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 610 + which, 600 + which);
    }
    for (which, z) in [(0_usize, 0.0), (1_usize, height)] {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            620 + which,
            triple([0.0, 0.0, z])
        );
        text += &format!(
            "#{}=DIRECTION('',{});\n",
            624 + which,
            triple([0.0, 0.0, 1.0])
        );
        text += &format!(
            "#{}=DIRECTION('',{});\n",
            626 + which,
            triple([1.0, 0.0, 0.0])
        );
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#{},#{});\n",
            628 + which,
            620 + which,
            624 + which,
            626 + which
        );
        text += &format!(
            "#{}=CIRCLE('',#{},{});\n",
            630 + which,
            628 + which,
            real(radius)
        );
    }
    for (which, (from, to, arc)) in edges.iter().enumerate() {
        let curve = match arc {
            Some(circle) => *circle,
            None => {
                let way = direction_from(corners[*from], corners[*to]);
                text += &format!("#{}=DIRECTION('',{});\n", 660 + which, triple(way));
                text += &format!(
                    "#{}=VECTOR('',#{},{});\n",
                    670 + which,
                    660 + which,
                    real(1.0)
                );
                text += &format!(
                    "#{}=LINE('',#{},#{});\n",
                    680 + which,
                    600 + from,
                    670 + which
                );
                680 + which
            }
        };
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            690 + which,
            610 + from,
            610 + to,
            curve
        );
    }
    for (which, (round, sits_on)) in faces.iter().enumerate() {
        let mut steps = Vec::new();
        for (step, (edge, forwards)) in round.iter().enumerate() {
            let id = 700 + which * 8 + step;
            text += &format!(
                "#{id}=ORIENTED_EDGE('',*,*,#{},{});\n",
                690 + edge,
                if *forwards { ".T." } else { ".F." }
            );
            steps.push(format!("#{id}"));
        }
        text += &format!("#{}=EDGE_LOOP('',({}));\n", 740 + which, steps.join(","));
        text += &format!(
            "#{}=FACE_OUTER_BOUND('',#{},.T.);\n",
            746 + which,
            740 + which
        );
        let (origin, axis) = match sits_on {
            Face::Flat(normal, at) => (*at, *normal),
            Face::Round(_) => ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        };
        text += &format!("#{}=CARTESIAN_POINT('',{});\n", 752 + which, triple(origin));
        text += &format!("#{}=DIRECTION('',{});\n", 758 + which, triple(axis));
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#{},$);\n",
            764 + which,
            752 + which,
            758 + which
        );
        text += &match sits_on {
            Face::Flat(_, _) => format!("#{}=PLANE('',#{});\n", 770 + which, 764 + which),
            Face::Round(radius) => format!(
                "#{}=CYLINDRICAL_SURFACE('',#{},{});\n",
                770 + which,
                764 + which,
                real(*radius)
            ),
        };
        text += &format!(
            "#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n",
            776 + which,
            746 + which,
            770 + which
        );
    }
    let skin: Vec<String> = (0..faces.len())
        .map(|which| format!("#{}", 776 + which))
        .collect();
    text += &format!("#800=CLOSED_SHELL('',({}));\n", skin.join(","));
    text += "#801=MANIFOLD_SOLID_BREP('',#800);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

/// One edge: the corners it runs between, and the circle it runs along where it
/// is an arc rather than a straight.
type Edge = (usize, usize, Option<usize>);

/// One loop: the edges it walks, each with whether it walks it forwards.
type Loop = Vec<(usize, bool)>;

/// What one face of a pie-slice plate sits on.
enum Face {
    /// A plane, given its outward normal and a point on it.
    Flat([f64; 3], [f64; 3]),
    /// The cylinder about the `z` axis, given its radius.
    Round(f64),
}
