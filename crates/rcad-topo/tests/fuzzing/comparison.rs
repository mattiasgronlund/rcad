//! The both-orders comparison, shared rather than copied.
//!
//! `tests/fuses_that_must_agree.rs` and `tests/fuzzing/fuse.rs` both ask a
//! union or an intersection with its two sides swapped and hold the two
//! answers to each other bit for bit — the first of that file's laws to reach
//! a fuzzer's own solids too (§129.5 item 3). Reached by `#[path]` from both,
//! so there is one copy of the comparison rather than two that can drift apart
//! the way `docs/decisions.md` §129.5 found them.
//!
//! What moved here is the geometry of one answer and the two ways it gets
//! compared: [`Answer`] and the pieces it is built from, [`answer_of`], and
//! [`same_answer`]. What stayed behind in each file is everything that needs
//! proptest's `TestCaseError` or a fuzz-only counter, plus `moved_by` on every
//! type here — `fuses_that_must_agree.rs` adds those as their own `impl`
//! blocks over the types this module declares, which needs no more than an
//! ordinary `use`: they live in the same binary once `#[path]` has pulled this
//! file in, so it is not a foreign type.
//!
//! `same_answer`'s error is a plain [`String`]. Neither side needs to wrap
//! it the same way: `fuses_that_must_agree.rs` turns it into a
//! `TestCaseError` and the fuzz side leaves it as `String` and panics on it.

use rcad_geom::{EdgeRun, Orientation, Picker, Surface};
use rcad_topo::{EdgeView, Model, Rule, SolidKey};

/// The geometry of an answer, and nothing about which slot anything landed in.
#[derive(Clone, Debug)]
pub struct Answer {
    pub solids: Vec<Vec<Face>>,
}

/// One face: the surface it sits on, which way round its solid uses it, and its
/// rings of steps — each a corner and what the edge from it runs along.
///
/// **Which way round rather than which way it faces**, because a curved face has
/// no one normal to read. For a flat face the two say the same thing: the
/// outward normal is the stored plane's normal times the orientation's sign, so
/// two faces with the same plane bits agree about the normal exactly when they
/// agree about the orientation. That stays true under a movement as well, since
/// a quarter turn is a swap and a negation and multiplying by `±1` commutes
/// with both.
#[derive(Clone, Debug)]
pub struct Face {
    pub sits: WhatItSitsOn,
    pub reversed: bool,
    pub wires: Vec<Vec<Step>>,
}

/// One step round a wire: the corner it leaves from and what it runs along to
/// the next.
///
/// **The run is here because the corners are not enough** (§165). A ring of two
/// corners reads the same walked either way, a round wire has one corner and
/// no order at all, and a quarter of a ball and three quarters of it have the
/// same faces with the same corners — only which half-meridian each edge
/// follows tells them apart.
#[derive(Clone, Debug)]
pub struct Step {
    pub at: [f64; 3],
    pub run: Run,
}

/// What an edge runs along, as numbers.
#[derive(Clone, Debug)]
pub enum Run {
    /// Straight to the next corner, which is all there is to say about it.
    Straight,
    /// Right round a closed crossing and back to the corner it left, and
    /// whether the wire walks it the way the edge is stored running.
    ///
    /// **Which way a closed edge is stored running is not arbitrary**: it is
    /// the way the curved surface's own longitude takes it, the fuse writes it
    /// that way and `Model::validate` reads it back
    /// (`check_closed_edges_are_walked_the_way_their_surfaces_wind`). So which
    /// way round the wire goes is part of the answer, and a round wire has no
    /// second corner to say it.
    RightRound { forward: bool },
    /// One piece of where the face crosses `and`, picked out by `picked_by`.
    PartOf {
        and: WhatItSitsOn,
        picked_by: Vec<Picking>,
    },
}

/// A picker, as numbers: `a·x + b·y + c·z ≥ d` for a flat one, the surface
/// itself for a curved one.
#[derive(Clone, Debug)]
pub enum Picking {
    InFrontOf([f64; 4]),
    Outside(WhatItSitsOn),
    InsideAPipe(WhatItSitsOn),
}

impl Run {
    #[must_use]
    pub fn of(edge: EdgeView<'_>) -> Run {
        match edge.run() {
            EdgeRun::Straight => Run::Straight,
            EdgeRun::RightRoundWhereItsFacesCross => Run::RightRound {
                forward: edge.orientation() == Orientation::Forward,
            },
            EdgeRun::PartOfWhereItCrosses { and, picked_by } => Run::PartOf {
                and: WhatItSitsOn::of(&and),
                picked_by: picked_by.iter().map(Picking::of).collect(),
            },
            // Named rather than left to the wildcard, because it exists (§40).
            // It gets no `Run` shape: the fuse never writes it — `assemble`
            // still names a closed rim `RightRoundWhereItsFacesCross`, and only
            // `SolidBuilder` can say which surface cut one (§216) — so a shape
            // here would be one nothing reaches (§13).
            EdgeRun::RightRoundWhereItCrosses { .. } => {
                panic!("the fuse does not write a rim that names its cutting surface; §216")
            }
            other => panic!("this file does not know how to write {other:?} down"),
        }
    }

    /// The numbers, tagged. The pickers are sorted: they are a set, and the
    /// order they were listed in is the arena's business.
    #[must_use]
    pub fn written_out(&self) -> Vec<u64> {
        match self {
            Run::Straight => vec![0],
            Run::RightRound { forward } => vec![1, u64::from(*forward)],
            Run::PartOf { and, picked_by } => {
                let mut pickers: Vec<Vec<u64>> =
                    picked_by.iter().map(Picking::written_out).collect();
                pickers.sort_unstable();
                let mut written = vec![2];
                written.extend(and.written_out());
                written.push(pickers.len() as u64);
                written.extend(pickers.into_iter().flatten());
                written
            }
        }
    }
}

impl Picking {
    #[must_use]
    pub fn of(picker: &Picker) -> Picking {
        match picker {
            Picker::InFrontOf(flat) => Picking::InFrontOf(flat.coefficients()),
            Picker::OutsideABall(ball) => {
                Picking::Outside(WhatItSitsOn::of(&Surface::Sphere(*ball)))
            }
            Picker::OutsideAPipe(pipe) => {
                Picking::Outside(WhatItSitsOn::of(&Surface::Cylinder(*pipe)))
            }
            Picker::InsideAPipe(pipe) => {
                Picking::InsideAPipe(WhatItSitsOn::of(&Surface::Cylinder(*pipe)))
            }
            other => panic!("this file does not know how to write {other:?} down"),
        }
    }

    #[must_use]
    pub fn written_out(&self) -> Vec<u64> {
        match self {
            Picking::InFrontOf(coefficients) => {
                core::iter::once(0).chain(coefficients.map(bits)).collect()
            }
            Picking::Outside(surface) => core::iter::once(1).chain(surface.written_out()).collect(),
            Picking::InsideAPipe(surface) => {
                core::iter::once(2).chain(surface.written_out()).collect()
            }
        }
    }
}

/// What a face sits on, kept as the numbers that name it.
///
/// Numbers rather than a `Surface` because `Answer::moved_by` has to move a
/// plane's four coefficients, and because the whole point of the comparison is
/// that nothing about those numbers changed.
#[derive(Clone, Debug)]
pub enum WhatItSitsOn {
    /// A plane, as its four coefficients.
    Flat([f64; 4]),
    /// A ball's skin: where its middle is and how big it is.
    Ball([f64; 3], f64),
    /// A pipe's skin: a point on its axis, the way that axis points, and how
    /// big it is.
    Pipe([f64; 3], [f64; 3], f64),
    /// A cone's skin: its apex, the way its axis points, and how fast it
    /// widens.
    Cone([f64; 3], [f64; 3], f64),
    /// A ring's skin: its centre, the way its axis points, and its two radii.
    Ring([f64; 3], [f64; 3], f64, f64),
}

impl WhatItSitsOn {
    #[must_use]
    pub fn of(surface: &Surface) -> WhatItSitsOn {
        match surface {
            Surface::Plane(flat) => WhatItSitsOn::Flat(flat.coefficients()),
            Surface::Sphere(ball) => WhatItSitsOn::Ball(ball.centre().coordinates(), ball.radius()),
            Surface::Cylinder(pipe) => WhatItSitsOn::Pipe(
                pipe.axis().origin().coordinates(),
                pipe.axis().direction().components(),
                pipe.radius(),
            ),
            Surface::Cone(cone) => WhatItSitsOn::Cone(
                cone.apex().coordinates(),
                cone.axis().direction().components(),
                cone.slope(),
            ),
            Surface::Torus(ring) => WhatItSitsOn::Ring(
                ring.centre().coordinates(),
                ring.axis().direction().components(),
                ring.ring_radius(),
                ring.tube_radius(),
            ),
            other => panic!("this file does not know how to write {other:?} down"),
        }
    }

    /// The numbers, with a tag so two different kinds of surface can never
    /// write themselves out the same way.
    #[must_use]
    pub fn written_out(&self) -> Vec<u64> {
        match self {
            WhatItSitsOn::Flat(coefficients) => {
                core::iter::once(0).chain(coefficients.map(bits)).collect()
            }
            WhatItSitsOn::Ball(centre, radius) => core::iter::once(1)
                .chain(centre.map(bits))
                .chain([bits(*radius)])
                .collect(),
            WhatItSitsOn::Pipe(origin, direction, radius) => core::iter::once(2)
                .chain(origin.map(bits))
                .chain(direction.map(bits))
                .chain([bits(*radius)])
                .collect(),
            WhatItSitsOn::Cone(apex, direction, slope) => core::iter::once(3)
                .chain(apex.map(bits))
                .chain(direction.map(bits))
                .chain([bits(*slope)])
                .collect(),
            WhatItSitsOn::Ring(centre, direction, big, small) => core::iter::once(4)
                .chain(centre.map(bits))
                .chain(direction.map(bits))
                .chain([bits(*big), bits(*small)])
                .collect(),
        }
    }
}

/// The geometry of one fuse's answer, read straight out of the model.
#[must_use]
pub fn answer_of(model: &Model, solids: &[SolidKey]) -> Answer {
    Answer {
        solids: solids
            .iter()
            .map(|&solid| {
                model
                    .solid(solid)
                    .expect("this model made the key")
                    .faces()
                    .map(|face| Face {
                        sits: WhatItSitsOn::of(&face.surface()),
                        reversed: face.orientation() == Orientation::Reversed,
                        wires: face
                            .wires()
                            .map(|wire| {
                                wire.coedges()
                                    .map(|coedge| Step {
                                        at: coedge.start().point().coordinates(),
                                        run: Run::of(coedge.edge()),
                                    })
                                    .collect()
                            })
                            .collect(),
                    })
                    .collect()
            })
            .collect(),
    }
}

impl Answer {
    /// The answer written out in one fixed order, so that two answers agree
    /// exactly when the arena order is the only thing between them.
    ///
    /// A ring of steps is turned to its own smallest rotation rather than
    /// sorted: which step comes first is arbitrary, but which step follows
    /// which is the winding, and the winding is part of the answer.
    #[must_use]
    pub fn canonical(&self) -> Vec<Vec<Vec<u64>>> {
        let mut solids: Vec<Vec<Vec<u64>>> = self
            .solids
            .iter()
            .map(|faces| {
                let mut faces: Vec<Vec<u64>> = faces.iter().map(written_out).collect();
                faces.sort_unstable();
                faces
            })
            .collect();
        solids.sort_unstable();
        solids
    }

    fn face_count(&self) -> usize {
        self.solids.iter().map(Vec::len).sum()
    }
}

/// One face as numbers: its surface, which way round it is used, its outline,
/// and then its holes in sorted order.
///
/// **The outline stays first and only the holes are sorted.** Which wire is
/// first is part of the face in this kernel, and a round wire has no winding
/// to carry it instead (§163).
fn written_out(face: &Face) -> Vec<u64> {
    let mut written = face.sits.written_out();
    written.push(u64::from(face.reversed));

    let mut wires = face.wires.iter().map(|wire| turned_to_its_smallest(wire));
    let outline = wires.next().expect("a face has at least one wire");
    let mut holes: Vec<Vec<u64>> = wires.collect();
    holes.sort_unstable();
    written.push(face.wires.len() as u64);
    for wire in core::iter::once(outline).chain(holes) {
        written.push(wire.len() as u64);
        written.extend(wire);
    }
    written
}

/// A ring written down from whichever of its corners makes the smallest list.
///
/// **The smallest rotation, not the first smallest corner.** Starting at the
/// first corner equal to the smallest one is a canonical form only while no
/// ring visits a corner twice. A face ring that does — and a turned pair makes
/// one, through the point the turn holds still — is written down as its two
/// halves by the two orders, and the comparison above then reports a face that
/// differs when the two answers are the same solid. `docs/decisions.md` §149
/// is the counterexample that cost a session, §151 is this.
///
/// Each step is its corner followed by its run (§165), and a step is turned as
/// one piece, so a corner never lands beside another corner's run.
#[must_use]
pub fn turned_to_its_smallest(wire: &[Step]) -> Vec<u64> {
    let steps: Vec<Vec<u64>> = wire
        .iter()
        .map(|step| {
            step.at
                .map(bits)
                .into_iter()
                .chain(step.run.written_out())
                .collect()
        })
        .collect();
    (0..steps.len())
        .map(|start| -> Vec<u64> {
            (0..steps.len())
                .flat_map(|step| steps[(start + step) % steps.len()].iter().copied())
                .collect()
        })
        .min()
        .unwrap_or_default()
}

/// A number's bits, with the two zeroes counted as one.
///
/// `-0.0` and `0.0` name the same point and the same plane, and which one an
/// arithmetic path lands on is not part of the answer. Everything else is
/// compared bit for bit.
#[must_use]
pub fn bits(number: f64) -> u64 {
    if number == 0.0 {
        0u64
    } else {
        number.to_bits()
    }
}

/// Insist two answers are the same one, bit for bit.
///
/// # Errors
///
/// How the two differ, when they do.
pub fn same_answer(
    expected: &Answer,
    found: &Answer,
    rule: Rule,
    what_changed: &str,
) -> Result<(), String> {
    if expected.canonical() == found.canonical() {
        return Ok(());
    }

    let mut why = format!("{what_changed} changed the answer to a {rule:?}:\n");
    why.push_str(&format!(
        "  {} solids and {} faces one way, {} solids and {} faces the other\n",
        expected.solids.len(),
        expected.face_count(),
        found.solids.len(),
        found.face_count()
    ));
    for (position, (one, other)) in expected
        .canonical()
        .iter()
        .zip(found.canonical().iter())
        .enumerate()
    {
        for (face, (mine, theirs)) in one.iter().zip(other.iter()).enumerate() {
            if mine != theirs {
                why.push_str(&format!("  solid {position} face {face} differs\n"));
                return Err(why);
            }
        }
    }
    Err(why)
}
