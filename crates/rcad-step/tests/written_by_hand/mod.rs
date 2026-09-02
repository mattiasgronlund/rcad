//! STEP files written here, so that what a test measures can be checked
//! against numbers nobody read off the kernel.
//!
//! Every file below is a whole AP203 `manifold_solid_brep`: header, units,
//! geometric context, closed shell, advanced faces, edge loops, oriented edges,
//! edge curves and vertex points. The point of writing them out rather than
//! exporting them from anywhere is that the closed form is known — a block of
//! sides `a`, `b`, `c` encloses `abc` and no measurement of it is being compared
//! against another measurement.

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

fn triple(numbers: [f64; 3]) -> String {
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
