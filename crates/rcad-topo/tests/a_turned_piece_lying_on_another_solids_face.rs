//! Two turned pairs where a piece of one solid lies on a face of the other, and
//! the fuse used to ask the ray where that piece was.
//!
//! Q259's probe drew both from `a_turned_pair`, the generator the turned
//! properties of `fuses_that_must_agree.rs` draw from, as turned pairs that
//! were not a solid where the same pair upright answers:
//!
//! - **A comb standing in a block**, draw #3513 at seed 102. A comb of two
//!   teeth, each two above its base, extruded two high, and a block four by
//!   four and three high at `(0, -2, 0)`, which holds the comb's base and the
//!   lower half of each tooth. Turned three quarters of a turn about `z`, the
//!   comb less the block was not a solid, `count: 3`, where upright it is the
//!   two tooth tips, volume 4.
//! - **A frame with a block over one corner**, draw #1160 at seed 101. A frame
//!   three by five and one high, and a block four by four and two high at
//!   `(2, 4, 0)`, which covers the frame's corner cell at `x = 2, y = 4`.
//!   Turned 29/64 of a turn about `(-3, -2, 3)`, their union was not a solid,
//!   `count: 3`, where upright it is one solid of volume 43.
//!
//! In both, pieces of the answer lie on the other solid's faces — the teeth's
//! cut ends on the block's `y = 2` face, both solids' floors on `z = 0` — and
//! the fuse fired the ray at such a piece before asking whether it lay on a
//! face. The ray reads a face as triangles through its **stored** corners,
//! which the turn leaves a rounding off the face's plane (§351), so it could
//! certify a flush piece `Inside` or `Outside`, and the piece was kept or
//! dropped as if it were not flush (Q432).
//!
//! **§351 fixed both** before anyone came back to them: `settle_against_each` in
//! `fuse/select.rs` now asks `coincident_face` before it fires the ray, for a
//! flat piece as well as a curved one. On that tree every fuse below, all three
//! rules both ways round, is the upright answer face for face.
//!
//! Put the ray back in front of `coincident_face` for a flat piece — asking
//! coincidence only after the ray says `OnBoundary` or `Undecidable`, as before
//! §351 — and six of these twelve fuses are not a solid again where upright
//! they answer, three of each pair, the two above among them at `count: 3`.
//!
//! The frame less the block used to be not a solid upright either, `count: 4`:
//! two walls of the frame met along one edge at the hole's corner, `x = 2, y =
//! 4`, and no `Model` held that. Since Q1913 it is one solid of volume 11, one
//! shell of 12 faces, validating both upright and turned: the frame ring cut
//! open at the corner and touching itself along `x = 2, y = 4`
//! (`a_frame_with_a_block_over_one_corner_fuses_turned_as_it_does_upright`).

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use std::collections::BTreeMap;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew, Turn};

#[test]
fn a_comb_standing_in_a_block_fuses_turned_as_it_does_upright() {
    every_fuse_is_the_upright_one(&a_comb_standing_in_a_block());
}

/// The frame less the block is one solid of volume 11, one shell of 12 faces,
/// validating, upright and turned (Q1913).
#[test]
fn a_frame_with_a_block_over_one_corner_fuses_turned_as_it_does_upright() {
    every_fuse_is_the_upright_one(&a_frame_with_a_block_over_one_corner());
}

/// All three rules, each solid first in turn, against the same fuse upright.
fn every_fuse_is_the_upright_one(configuration: &Configuration) {
    let upright = Configuration {
        turn: None,
        ..configuration.clone()
    };
    let turned = Built::from(configuration, &upright);
    let standing = Built::from(&upright, &upright);

    let mut disagreements = Vec::new();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (first, second) in [(0, 1), (1, 0)] {
            let answer = turned.fused(first, second, rule);
            let expected = standing.fused(first, second, rule);
            if answer != expected {
                disagreements.push(format!(
                    "{rule:?} with solid {first} first: turned {}, upright {}",
                    answer.summary(),
                    expected.summary()
                ));
            }
        }
    }
    assert!(
        disagreements.is_empty(),
        "the turned pair is not the upright one:\n{}",
        disagreements.join("\n")
    );
}

/// Seed 102's 3513th draw of the turned pairs.
fn a_comb_standing_in_a_block() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Comb {
                    teeth: 2,
                    tooth_height: 2,
                },
                skew: Skew::none(),
                height: 2,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Block { width: 4, depth: 4 },
                skew: Skew::none(),
                height: 3,
                at: [0, -2, 0],
            },
        ],
        turn: Some(Turn {
            direction: [0, 0, 2],
            steps: 48,
        }),
        elsewhere: Elsewhere::here(),
    }
}

/// Seed 101's 1160th draw of the turned pairs.
fn a_frame_with_a_block_over_one_corner() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Frame { width: 3, depth: 5 },
                skew: Skew::none(),
                height: 1,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Block { width: 4, depth: 4 },
                skew: Skew::none(),
                height: 2,
                at: [2, 4, 0],
            },
        ],
        turn: Some(Turn {
            direction: [-3, -2, 3],
            steps: 29,
        }),
        elsewhere: Elsewhere::here(),
    }
}

/// A configuration built, and each of its planes named by the plane it is
/// upright.
///
/// A fuse makes no plane of its own — every face of an answer lies on a plane of
/// an input, or on that plane written the other way round where a difference
/// turns a tool's face over, bit for bit either way (§17.1) — so naming an
/// answer's faces by their upright planes lets a turned answer and an upright
/// one compare exactly.
struct Built {
    model: Model,
    keys: Vec<SolidKey>,
    upright_plane: BTreeMap<[u64; 4], String>,
}

impl Built {
    fn from(configuration: &Configuration, upright: &Configuration) -> Self {
        let (model, keys) = configuration.build();
        let (standing, standing_keys) = upright.build();
        let mut upright_plane = BTreeMap::new();
        for (&solid, &standing_solid) in keys.iter().zip(&standing_keys) {
            let faces = model.solid(solid).expect("built here").faces();
            let standing_faces = standing.solid(standing_solid).expect("built here").faces();
            for (face, standing_face) in faces.zip(standing_faces) {
                let plane = plane_of(face.surface());
                let standing_plane = plane_of(standing_face.surface());
                for (plane, standing_plane) in [
                    (plane, standing_plane),
                    (plane.map(|c| -c), standing_plane.map(|c| -c)),
                ] {
                    let name = format!("{:?}", standing_plane.map(|c| c + 0.0));
                    if let Some(earlier) = upright_plane.insert(bits(plane), name.clone()) {
                        assert_eq!(earlier, name, "one turned plane from two upright ones");
                    }
                }
            }
        }
        Self {
            model,
            keys,
            upright_plane,
        }
    }

    fn fused(&self, first: usize, second: usize, rule: Rule) -> Answer {
        let mut model = self.model.clone();
        let fused = model.fuse(
            &[self.keys[first]],
            &[self.keys[second]],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let solids = match fused {
            Ok(fused) => fused.solids().to_vec(),
            Err(Error::ResultIsNotManifold { count }) => return Answer::NotASolid { count },
            Err(other) => return Answer::Refused(other.to_string()),
        };
        let mut written: Vec<Vec<Face>> = solids
            .iter()
            .map(|&solid| {
                let mut faces: Vec<Face> = model
                    .solid(solid)
                    .expect("the fuse made it")
                    .faces()
                    .map(|face| {
                        let mut corners: Vec<usize> =
                            face.wires().map(|wire| wire.points().count()).collect();
                        corners.sort_unstable();
                        Face {
                            upright_plane: self
                                .upright_plane
                                .get(&bits(plane_of(face.surface())))
                                .expect("every plane of an answer is a plane of an input")
                                .clone(),
                            orientation: format!("{:?}", face.orientation()),
                            corners,
                        }
                    })
                    .collect();
                faces.sort_unstable();
                faces
            })
            .collect();
        written.sort_unstable();
        Answer::Solids(written)
    }
}

fn plane_of(surface: rcad_geom::Surface) -> [f64; 4] {
    surface
        .as_plane()
        .expect("both solids are flat")
        .coefficients()
}

/// `-0.0` and `0.0` are the same coefficient, and their bits are not.
fn bits(plane: [f64; 4]) -> [u64; 4] {
    plane.map(|c| (c + 0.0).to_bits())
}

/// What one fuse gave.
#[derive(Debug, PartialEq)]
enum Answer {
    Solids(Vec<Vec<Face>>),
    NotASolid { count: usize },
    Refused(String),
}

impl Answer {
    fn summary(&self) -> String {
        match self {
            Answer::Solids(solids) => format!(
                "{} solids of {:?} faces",
                solids.len(),
                solids.iter().map(Vec::len).collect::<Vec<_>>()
            ),
            Answer::NotASolid { count } => format!("not a solid, count {count}"),
            Answer::Refused(why) => format!("refused: {why}"),
        }
    }
}

/// One face of an answer: the plane it lies on upright, which way round its
/// solid uses it, and how many corners each of its rings has.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Face {
    upright_plane: String,
    orientation: String,
    corners: Vec<usize>,
}
