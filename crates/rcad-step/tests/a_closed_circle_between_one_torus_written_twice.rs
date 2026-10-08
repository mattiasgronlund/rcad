//! A disc with a round rim — the outer half of a torus between two flat faces —
//! cut into two bands at the rim's outer equator, each band on its own
//! `TOROIDAL_SURFACE`, reads the same whether the two surfaces are written bit
//! for bit alike or one ulp apart.
//!
//! `docs/decisions.md` §222.2: the reader checked a closed `CIRCLE` between
//! curved faces against every face and named it by its own plane, but it had
//! no bound for a torus, so between one torus written twice a hair apart the
//! equator stayed `EdgeRun::RightRoundWhereItsFacesCross` — "where these two
//! nearly identical tori cross", which is nowhere near the equator. The
//! equator's own plane crosses either torus in exactly the equator, and the
//! check now has a bound for a circle square to a torus's axis.
//!
//! **It is the ring radius that is one ulp off, not the tube radius.** The two
//! flat faces touch the torus along its top and bottom circles, and a tube one
//! ulp fatter makes a flat face cut the torus in two circles about `1e-8`
//! apart rather than touch it in one. A ring one ulp wider only moves the
//! circle it touches by that ulp.
//!
//! The bit-identical spelling is the oracle: it already carried the equator's
//! plane, so the hair-apart one must come out with the same run.
//!
//! **The reader stores each circle round the ring the way `rcad-topo` does**
//! (`docs/decisions.md` §390.5). The equator by `Model::validate`'s second
//! reference, which stores it running down the ring angle; the circles where
//! the flat faces rest on the rim by the mesher's rule for faces that touch.
//! Stored by the file's `ORIENTED_EDGE` flags instead, both spellings were
//! refused `AClosedEdgeIsWalkedAgainstItsSurface` at the equator.
//!
//! # The one-ulp spelling is refused past the reader, at the lid
//!
//! **The upper band's torus one ulp wider no longer touches the lid at the
//! file's corner.** The lid rests on that torus round a circle one ulp out
//! from `(3, 0, 1)`, so at the corner the torus's gradient leans in by an ulp
//! and crosses the lid's normal to `(0, 1.07e-14, 0)`, where the bit-for-bit
//! spelling crosses to nothing. `Model::validate` and the mesher read that as
//! the two crossing, and store the circle by the latitude, while the side of
//! the lid the band lies on is read off its far corner, below the lid. The two
//! readings disagree, and the import is refused
//! `AClosedEdgeIsWalkedAgainstItsSurface`. Stored the other way, as where the
//! two touch, it was refused `AWireWillNotLieFlatOnItsFace` instead (a probe in
//! the reader, not kept). Neither is the reader's to settle, so that spelling
//! is pinned to the refusal it meets.

mod written_by_hand;

use std::f64::consts::PI;

use rcad_geom::{Deflection, EdgeRun, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{Error, Invalid};
use written_by_hand::{a_header, real};

const RING: f64 = 3.0;
const TUBE: f64 = 1.0;

#[test]
fn a_torus_written_twice_bit_for_bit_names_the_equator_by_its_plane() {
    the_equator_is_named_by_its_plane_and_meshed_round(RING);
}

#[test]
fn a_torus_written_twice_one_ulp_apart_is_refused_where_the_lid_rests_on_the_wider_ring() {
    let text = a_disc_with_a_round_rim_in_two_bands(RING.next_up());
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        matches!(
            &refused,
            Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. })
                if matches!(**why, Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface))
        ),
        "the disc with its upper band's ring one ulp wide came back {:?}",
        refused.map(|import| import.solids().len())
    );
}

fn the_equator_is_named_by_its_plane_and_meshed_round(upper_ring: f64) {
    let text = a_disc_with_a_round_rim_in_two_bands(upper_ring);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import =
        Import::read(&file, &policy, &mut log).expect("a disc with a round rim in two bands reads");
    let [(_, disc)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*disc).expect("this model made the key");

    let mut equator = None;
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if edge.start().point().coordinates()[2] != 0.0
                    || edge.start().key() != edge.end().key()
                {
                    continue;
                }
                equator = Some(edge.key());
                let EdgeRun::RightRoundWhereItCrosses { and } = edge.run() else {
                    panic!(
                        "the two bands are one torus, so the equator has to carry the surface \
                         that names it; it says {:?}",
                        edge.run()
                    );
                };
                let Surface::Plane(flat) = and else {
                    panic!("a circle round a torus's axis is named by its own plane, not {and:?}");
                };
                assert_eq!(
                    flat.normal().cross(Vector::new(0.0, 0.0, 1.0)).length(),
                    0.0,
                    "the plane is square to the torus's axis, not to {:?}",
                    flat.normal()
                );
                assert_eq!(
                    flat.signed_distance_to(Point::new(0.0, 0.0, 0.0)),
                    0.0,
                    "the plane is the equator's own"
                );
            }
        }
    }
    let equator = equator.expect("the rim has a closed circle at its outer equator");

    model
        .validate(&policy, &mut log)
        .expect("the model the reader built passes the full sweep");
    let mesh = solid
        .mesh(Deflection::of(0.01).expect("a length"), &policy, &mut log)
        .expect("the disc meshes");
    let allowed = mesh.deviation().magnitude();
    let along = mesh.points_along_edge(equator);
    assert!(
        along.len() > 8,
        "a whole circle of radius four at a hundredth came back with {} points",
        along.len()
    );
    for &at in along {
        let [x, y, z] = mesh.points()[at as usize].coordinates();
        let off = ((x * x + y * y).sqrt() - (RING + TUBE)).abs().max(z.abs());
        assert!(
            off <= allowed,
            "{:?} is {off} off the equator, past the published {allowed}",
            [x, y, z]
        );
    }

    holds_what_pappus_gives(&solid, &policy, &mut log);
}

/// The disc's volume against Pappus, on its cross-section: the rectangle
/// `0 ≤ ρ ≤ 3`, `|z| ≤ 1` has area `6` and centroid `ρ = 3/2`, so it sweeps
/// `18π`; the half of the tube's disc outside `ρ = 3` has area `π/2` and its
/// centroid `4/(3π)` past that line, so it sweeps `3π² + 4π/3`. The same
/// figure `rcad-topo`'s `a_disc_with_a_round_rim_in_two_bands_holds_what_pappus_gives`
/// holds the disc built face by face to.
fn holds_what_pappus_gives(
    solid: &rcad_topo::SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) {
    let pappus = 18.0 * PI + 3.0 * PI * PI + 4.0 * PI / 3.0;
    for deflection in [0.1, 0.03, 0.01] {
        let measured = solid
            .measure(Deflection::of(deflection).expect("a length"), policy, log)
            .unwrap_or_else(|why| panic!("the disc at {deflection} was refused {why:?}"));
        assert!(
            (measured.volume() - pappus).abs() <= measured.volume_error(),
            "at {deflection} the disc measured {} where Pappus gives {pappus}, allowed {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// A disc of radius `RING` and thickness `2·TUBE` centred on the origin, with
/// the outer half of the torus of ring `RING` and tube `TUBE` about the `z`
/// axis for its rim. The flat faces touch the torus along its top and bottom
/// circles, and the outer equator cuts the rim into two bands: the lower on a
/// torus of ring `RING`, the upper on one of `upper_ring`, each its own entity.
///
/// Both bands' seams lie on the one tube circle in the half-plane `y = 0`,
/// `x > 0`, turning from the bottom circle up through the equator to the top.
/// Each band's loop runs as a pipe's does: round its lower circle, up the seam,
/// back round its upper circle and down the seam again.
fn a_disc_with_a_round_rim_in_two_bands(upper_ring: f64) -> String {
    let circles = [(-TUBE, RING), (0.0, RING + TUBE), (TUBE, RING)];
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.,0.,1.));\n";
    text += "#21=DIRECTION('',(1.,0.,0.));\n";
    text += "#22=DIRECTION('',(0.,0.,-1.));\n";
    text += "#25=DIRECTION('',(0.,-1.,0.));\n";
    for (which, (level, radius)) in circles.iter().enumerate() {
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
    text += &format!("#123=CARTESIAN_POINT('',({},0.,0.));\n", real(RING));
    text += "#133=AXIS2_PLACEMENT_3D('',#123,#25,#21);\n";
    text += &format!("#143=CIRCLE('',#133,{});\n", real(TUBE));
    for seam in 0..2 {
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#143,.T.);\n",
            170 + seam,
            110 + seam,
            111 + seam
        );
    }

    text += "#201=AXIS2_PLACEMENT_3D('',#121,#20,#21);\n";
    for (band, ring) in [RING, upper_ring].iter().enumerate() {
        text += &format!(
            "#{}=TOROIDAL_SURFACE('',#201,{},{});\n",
            210 + band,
            real(*ring),
            real(TUBE)
        );
        let (below, above, seam) = (150 + band, 151 + band, 170 + band);
        let first = 220 + band * 4;
        text += &format!("#{first}=ORIENTED_EDGE('',*,*,#{below},.T.);\n");
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{seam},.T.);\n", first + 1);
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{above},.F.);\n", first + 2);
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{seam},.F.);\n", first + 3);
        text += &format!(
            "#{}=EDGE_LOOP('',(#{first},#{},#{},#{}));\n",
            230 + band,
            first + 1,
            first + 2,
            first + 3
        );
        text += &format!(
            "#{}=FACE_OUTER_BOUND('',#{},.T.);\n",
            235 + band,
            230 + band
        );
        text += &format!(
            "#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n",
            240 + band,
            235 + band,
            210 + band
        );
    }

    text += "#250=AXIS2_PLACEMENT_3D('',#120,#22,#21);\n";
    text += "#251=PLANE('',#250);\n";
    text += "#252=ORIENTED_EDGE('',*,*,#150,.F.);\n";
    text += "#253=EDGE_LOOP('',(#252));\n";
    text += "#254=FACE_OUTER_BOUND('',#253,.T.);\n";
    text += "#255=ADVANCED_FACE('',(#254),#251,.T.);\n";
    text += "#260=AXIS2_PLACEMENT_3D('',#122,#20,#21);\n";
    text += "#261=PLANE('',#260);\n";
    text += "#262=ORIENTED_EDGE('',*,*,#152,.T.);\n";
    text += "#263=EDGE_LOOP('',(#262));\n";
    text += "#264=FACE_OUTER_BOUND('',#263,.T.);\n";
    text += "#265=ADVANCED_FACE('',(#264),#261,.T.);\n";

    text += "#500=CLOSED_SHELL('',(#240,#241,#255,#265));\n";
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
