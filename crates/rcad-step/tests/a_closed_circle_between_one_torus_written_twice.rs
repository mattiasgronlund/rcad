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
//! # Refused by the reader, both spellings alike
//!
//! **Neither spelling reaches the mesher yet, and the stop is the reader's.**
//! `f886614e` wrote this file when the mesher refused the lower band
//! `CornersCoincide`, and `rcad-topo` now meshes a band that goes right round
//! the ring: the same disc written face by face through `SolidBuilder` builds
//! and holds what Pappus gives
//! (`a_disc_with_a_round_rim_in_two_bands_holds_what_pappus_gives` in
//! `rcad-topo`, `docs/decisions.md` §390). Read from this file it is refused
//! `AClosedEdgeIsWalkedAgainstItsSurface` at the equator.
//!
//! The reader says which way a closed edge is stored from the way its curved
//! surface's first coordinate runs, and on a torus that is round the tube,
//! square to every circle round the ring; the travel dots it to exactly zero,
//! so the reader falls back to the file's own `ORIENTED_EDGE` flags
//! (`which_way_round_a_closed_edge_is_stored` in `brep.rs`). `Model::validate`
//! reads the second reference there, `∇C` crossed into that first direction,
//! which stores the equator running down the ring angle; the flags store it the
//! way its `CIRCLE` runs, up the ring angle, and the lower band's `.F.` use is
//! then read as walking it against its surface. With the reader
//! reading that second reference between two curved faces, as an uncommitted
//! probe, the bit-for-bit spelling got past the equator and was refused
//! `AWireWillNotLieFlatOnItsFace` on a band instead, and the one-ulp spelling
//! was refused too. Not traced, but the likeliest reason: the disc written
//! through `SolidBuilder` needed the lower band's use of the bottom circle, where
//! the flat face touches the torus, the other way round from this file's flag,
//! so the flags miss the mesher's rule for faces that touch as well.
//!
//! So this pins the refusal both spellings meet today. When the reader stores
//! these circles the way the mesher does, `git show
//! f886614e:crates/rcad-step/tests/a_closed_circle_between_one_torus_written_twice.rs`
//! is the test this turns back into — the equator named by its plane, the
//! sweep passed and the equator's points within the published deviation — with
//! the volume held to Pappus, `18π + 3π² + 4π/3`.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{Error, Invalid};
use written_by_hand::{a_header, real};

const RING: f64 = 3.0;
const TUBE: f64 = 1.0;

#[test]
fn a_torus_written_twice_bit_for_bit_is_refused_where_the_reader_guesses_the_equators_way() {
    the_reader_stores_the_equator_against_the_sweeps_rule(RING);
}

#[test]
fn a_torus_written_twice_one_ulp_apart_is_refused_where_the_reader_guesses_the_equators_way() {
    the_reader_stores_the_equator_against_the_sweeps_rule(RING.next_up());
}

fn the_reader_stores_the_equator_against_the_sweeps_rule(upper_ring: f64) {
    let text = a_disc_with_a_round_rim_in_two_bands(upper_ring);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        matches!(
            &refused,
            Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. })
                if matches!(**why, Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface))
        ),
        "the disc with its upper band's ring {upper_ring} came back {:?}",
        refused.map(|import| import.solids().len())
    );
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
