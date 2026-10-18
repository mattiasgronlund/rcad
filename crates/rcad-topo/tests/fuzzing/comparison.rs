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
//!
//! **The test that `same_answer` can say no lives here too**, at the bottom,
//! because every binary that pulls this file in runs it. Before, only the
//! fuzz side had one, so a `same_answer` that agreed with everything still
//! passed every law in `fuses_that_must_agree.rs`.

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
    /// Right round the closed curve where the face crosses `and`, and whether
    /// the wire walks it the way the edge is stored running.
    ///
    /// **The fuse writes this** for a rim whose faces either side are the one
    /// curved surface, which say nothing about where the rim runs: a ball cut
    /// in two by the plane of a block's face, the block clear of the ball
    /// (`docs/decisions.md` §217.1). `and` is the only place that plane is
    /// written down in the answer, so leaving it out would call two balls cut
    /// by two different planes through one corner the same answer.
    RightRoundWhereItCrosses { and: WhatItSitsOn, forward: bool },
}

/// A picker, as numbers: `a·x + b·y + c·z ≥ d` for a flat one, the surface
/// itself for a curved one.
#[derive(Clone, Debug)]
pub enum Picking {
    InFrontOf([f64; 4]),
    Outside(WhatItSitsOn),
    InsideAPipe(WhatItSitsOn),
    /// Everything on or in front of a flat surface that passes exactly
    /// through a point: the point, then the way it faces.
    InFrontOfAFlatThrough([f64; 3], [f64; 3]),
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
            EdgeRun::RightRoundWhereItCrosses { and } => Run::RightRoundWhereItCrosses {
                and: WhatItSitsOn::of(&and),
                forward: edge.orientation() == Orientation::Forward,
            },
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
            Run::RightRoundWhereItCrosses { and, forward } => {
                let mut written = vec![3];
                written.extend(and.written_out());
                written.push(u64::from(*forward));
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
            Picker::InFrontOfAFlatThrough(flat) => Picking::InFrontOfAFlatThrough(
                flat.through().coordinates(),
                flat.facing().components(),
            ),
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
            Picking::InFrontOfAFlatThrough(through, facing) => core::iter::once(3)
                .chain(through.map(bits))
                .chain(facing.map(bits))
                .collect(),
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
/// ring visits a corner twice. A face ring that does — an ell covering a
/// frame's far corner leaves the frame's top face a C whose two ends touch
/// there, upright or turned (`the_turned_pair_whose_union_lost_a_face_one_way_round.rs`,
/// `docs/decisions.md` §351.8) — is written down as its two halves by the two
/// orders, and the comparison above then reports a face that differs when
/// the two answers are the same solid. `docs/decisions.md` §149 is the
/// counterexample that cost a session, §151 is this.
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

/// A triangle used the other way round, or with one corner a single ulp over,
/// is a different answer. The same triangle walked from another corner is not.
///
/// The last half is there so this cannot pass on a `same_answer` that refuses
/// everything.
#[test]
fn two_different_answers_are_refused() {
    let triangle = |reversed, far_corner: f64, first: usize| {
        let corners = [[0.0, 0.0, 0.0], [far_corner, 0.0, 0.0], [0.0, 1.0, 0.0]];
        Answer {
            solids: vec![vec![Face {
                sits: WhatItSitsOn::Flat([0.0, 0.0, 1.0, 0.0]),
                reversed,
                wires: vec![
                    (0..3)
                        .map(|step| Step {
                            at: corners[(first + step) % 3],
                            run: Run::Straight,
                        })
                        .collect(),
                ],
            }]],
        }
    };
    let answer = triangle(false, 1.0, 0);

    assert!(
        same_answer(
            &answer,
            &triangle(true, 1.0, 0),
            Rule::Union,
            "turning the face over"
        )
        .is_err(),
        "a face used the other way round passed for the same answer"
    );
    assert!(
        same_answer(
            &answer,
            &triangle(false, 1.0f64.next_up(), 0),
            Rule::Union,
            "one ulp"
        )
        .is_err(),
        "a corner one ulp away passed for the same answer"
    );
    assert_eq!(
        same_answer(
            &answer,
            &triangle(false, 1.0, 2),
            Rule::Union,
            "starting elsewhere"
        ),
        Ok(()),
        "the same triangle walked from another corner"
    );
}

/// A ball whose one rim is cut by another plane through the same corner, or
/// walked the other way round, is a different answer.
///
/// The corner and the ball are all the rest of the face says, so the plane in
/// the run is the only thing that tells the first apart and the direction the
/// only thing that tells the second (`a_rim_that_names_its_cutting_surface.rs`).
#[test]
fn two_rims_cut_by_different_planes_or_walked_different_ways_are_refused() {
    let a_ball_with_one_rim = |cut_by: [f64; 4], forward| Answer {
        solids: vec![vec![Face {
            sits: WhatItSitsOn::Ball([0.0, 0.0, 0.0], 1.0),
            reversed: false,
            wires: vec![vec![Step {
                at: [0.0, -1.0, 0.0],
                run: Run::RightRoundWhereItCrosses {
                    and: WhatItSitsOn::Flat(cut_by),
                    forward,
                },
            }]],
        }]],
    };
    let answer = a_ball_with_one_rim([0.0, 1.0, 0.0, -1.0], true);

    assert!(
        same_answer(
            &answer,
            &a_ball_with_one_rim([0.0, 1.0, 1.0, -1.0], true),
            Rule::Union,
            "another plane through the corner"
        )
        .is_err(),
        "a rim cut by another plane through the same corner passed for the same answer"
    );
    assert!(
        same_answer(
            &answer,
            &a_ball_with_one_rim([0.0, 1.0, 0.0, -1.0], false),
            Rule::Union,
            "walking the rim the other way"
        )
        .is_err(),
        "a rim walked the other way round passed for the same answer"
    );
    assert_eq!(
        same_answer(
            &answer,
            &a_ball_with_one_rim([0.0, 1.0, 0.0, -1.0], true),
            Rule::Union,
            "nothing"
        ),
        Ok(()),
        "the same rim"
    );
}
