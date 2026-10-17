//! What comes back from crossing two surfaces.
//!
//! A crossing is **named and never carried** (`docs/decisions.md` §24.3): what
//! comes out is the two surfaces, the half-spaces that pick one piece of them,
//! and two ends. No curve is built, because a curve stored beside the surfaces
//! that named it is a second authority on where the edge is, with no tiebreak at
//! the first ulp of disagreement.
//!
//! Nothing here names a topology key, which is what lets it live in this crate.
//! `rcad-topo` wraps a [`SurfaceMeet`] with the bookkeeping it owes — which
//! input edges a piece runs along, and the history behind it — because that half
//! does name keys and cannot come down here (§24.9).

use rcad_predicates::{Interval, Point as PredicatePoint};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::error::{Error, Result};
use crate::point::Vector;
use crate::{Isometry, Picker, Surface};

/// Where two surfaces cross.
///
/// Four arms covering five outcomes. Touching at a point and being the same
/// surface are **answers about the geometry** and not errors: roadmap §7.4 lists
/// both in the degeneracy catalogue as cases to handle, and what a fuse does
/// with them is the fuse's decision (§24.9).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum SurfaceMeet {
    /// They do not meet at all.
    Nothing,
    /// They touch at one point and cross nowhere — a plane tangent to a ball,
    /// a ball resting against a pipe.
    TouchAtOnePoint {
        /// Where they touch.
        at: PredicatePoint,
    },
    /// One curve, or two.
    ///
    /// Two is not a curiosity: a plane parallel to a pipe's axis and inside it
    /// cuts two straight lines, and a ball crossing a pipe off-axis can leave
    /// two closed loops. **A partly undecidable answer refuses whole** (§24.10):
    /// a caller handed one of two curves has no way to know the second existed.
    ///
    /// A `Vec` and not the two-deep `SmallVec` §24.9 wrote down. Two
    /// [`NamedRun`]s inline come to over five hundred bytes, and every arm of an
    /// enum is as wide as its widest — so [`SurfaceMeet::Nothing`], which is
    /// what most pairs of faces in a fuse answer, would cost that much to hand
    /// back. Saving one allocation on the rare answer by widening the common one
    /// is backwards (§27.3).
    Curves(Vec<NamedRun>),
    /// The crossing is two-dimensional, because the two surfaces are one.
    SameSurface,
}

/// One curve of a crossing: what it runs along, and where it starts and ends.
///
/// An [`EdgeRun`] and two ends and nothing else, so a fuse drops it into
/// topology without translating it — a result that needed translating to become
/// an edge would be the second authority §24.3 refuses.
///
/// The ends are [`PredicatePoint`]s rather than plain points because a computed
/// end usually is one: where a cap plane cuts a pipe the answer is a box, and
/// saying so is what lets a predicate downstream answer
/// [`Decision::Undecidable`](rcad_predicates::Decision::Undecidable) honestly
/// instead of being handed a rounded witness (§2.5).
#[derive(Clone, Debug, PartialEq)]
pub struct NamedRun {
    run: EdgeRun,
    ends: [PredicatePoint; 2],
    stored_this_far_out: f64,
}

impl NamedRun {
    /// The piece of curve `run` names, running from `ends[0]` to `ends[1]`.
    ///
    /// # Errors
    ///
    /// [`Error::AClosedRunDoesNotComeBackToWhereItStarted`] if `run` goes
    /// right round a closed curve and the two ends are not the same point. A
    /// closed run comes back to where it started by definition, so a pair of
    /// different ends on one names a curve that does not exist. **Both closed
    /// variants**, which [`EdgeRun::it_comes_back_to_where_it_started`] is the
    /// one list of.
    pub fn new(run: EdgeRun, ends: [PredicatePoint; 2]) -> Result<Self> {
        if run.it_comes_back_to_where_it_started() && ends[0] != ends[1] {
            return Err(Error::AClosedRunDoesNotComeBackToWhereItStarted {
                start: Box::new(ends[0]),
                end: Box::new(ends[1]),
            });
        }
        Ok(Self {
            run,
            ends,
            stored_this_far_out: 0.0,
        })
    }

    /// The whole of what `run` names, with no ends to cut it at.
    ///
    /// **A pair of surfaces names a curve and nothing on it.** The circle where
    /// a plane cuts a ball has no start; the line where two planes cross has no
    /// end at either side. Ends are what topology adds — the vertices that cut a
    /// curve into edges — and [`intersect`](crate::intersect) is handed two
    /// surfaces and no topology, so it has none to give.
    ///
    /// Both ends come back as the box that says *nothing at all is known about
    /// where this is*. That is **not** the same as an end that could not be
    /// worked out, and no route through [`intersect`](crate::intersect) confuses
    /// them: a partly undecidable answer refuses whole (`docs/decisions.md`
    /// §24.10), so an end a predicate gave up on comes back as `Err` and never
    /// inside a run. The type does not carry that — [`NamedRun::new`] is `pub`
    /// and checks only the closed-run rule, and `Interval::ENTIRE` is `pub` too,
    /// so a caller can build the confusion by hand. Nothing in this crate does
    /// (§39).
    ///
    /// A crossing can still come in more than one piece with no ends anywhere —
    /// a plane running down the inside of a pipe cuts two endless lines — and
    /// then the half-spaces are the whole of what tells them apart.
    ///
    /// # Errors
    ///
    /// [`Error::ACrossingPieceHasNoEndsAndNothingPickingIt`] if `run` is a
    /// [`EdgeRun::PartOfWhereItCrosses`] with nothing picking it. That variant
    /// exists to say *which* piece of a crossing this is, and with no ends
    /// and no half-space it says nothing at all. Bad caller input rather
    /// than a broken invariant: `picked_by` is a public field of a public
    /// enum variant, so a caller can build the confusion by hand without
    /// going through this crate's own naming logic at all.
    pub fn the_whole_of(run: EdgeRun) -> Result<Self> {
        if let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = &run
            && picked_by.is_empty()
        {
            return Err(Error::ACrossingPieceHasNoEndsAndNothingPickingIt);
        }
        Ok(Self {
            run,
            ends: [nowhere_in_particular(); 2],
            stored_this_far_out: 0.0,
        })
    }

    /// The same run, whose surfaces and ends were stored `shift` further out
    /// than they are now and were moved in by taking it off.
    ///
    /// **A number keeps the rounding of the place it was stored at.** A corner
    /// a file wrote a million out sits up to a rounding at a million,
    /// `1.16e-10`, off the circle its two faces cut, and moving the corner and
    /// the faces in together leaves it that far off a circle that is now of
    /// radius 7 round the origin. [`walk`](crate::walk) tells which of a
    /// crossing's circles a run is on by how far its ends are off each,
    /// against a bound that is a multiple of the size the circle was worked
    /// out from, and that bound grows by this shift's largest coordinate. So a
    /// run moved in is held no tighter than where it was stored, where the
    /// circle's size counted the million. `rcad-topo`'s mesher moves a face
    /// stored far out in until its numbers are small, and says so with this:
    /// without it the quarter bend of a ring of 8 moved a million out was
    /// refused [`Error::CannotNameWhichPieceOfACrossingThisIs`] once it had
    /// been through a file (queue row Q2218).
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `shift` has a component that is not a finite
    /// number.
    pub fn moved_in_from(self, shift: Vector) -> Result<Self> {
        if !shift.is_finite() {
            return Err(Error::NotFinite {
                what: "shift a run was moved in from",
            });
        }
        let [x, y, z] = shift.components().map(f64::abs);
        Ok(Self {
            stored_this_far_out: x.max(y).max(z),
            ..self
        })
    }

    /// The largest coordinate of the shift [`NamedRun::moved_in_from`] says
    /// this run was moved in by, and zero for a run worked out where it was
    /// stored.
    pub(crate) fn how_far_out_it_was_stored(&self) -> f64 {
        self.stored_this_far_out
    }

    /// Which piece of which crossing this is.
    #[must_use]
    pub fn run(&self) -> &EdgeRun {
        &self.run
    }

    /// Where it starts and where it ends, in that order. One point twice for a
    /// run that goes right round.
    #[must_use]
    pub fn ends(&self) -> [PredicatePoint; 2] {
        self.ends
    }

    /// Whether this is the whole of what its surfaces name, so its ends say
    /// nothing about where it is.
    ///
    /// See [`NamedRun::the_whole_of`].
    #[must_use]
    pub fn is_the_whole_curve(&self) -> bool {
        self.ends.iter().all(|end| *end == nowhere_in_particular())
    }
}

/// The box that says nothing at all is known about where a point is.
pub(crate) fn nowhere_in_particular() -> PredicatePoint {
    PredicatePoint::Constructed([Interval::ENTIRE; 3])
}

/// What an edge runs along between its two ends.
///
/// The two faces either side of an edge each sit on a surface, and those two
/// surfaces name where the edge is with no arithmetic at all — the same
/// argument decisions.md §15.9 makes for a pair of planes naming their line.
/// What the pair does *not* say is which piece of the crossing this edge is,
/// and that is the whole of what this enum adds.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EdgeRun {
    /// Straight between its two ends, which are the whole of it.
    Straight,
    /// Right round the closed curve where the faces either side of it cross,
    /// so it starts and ends at the same vertex.
    ///
    /// A cap of a round bar is this: the cylinder crosses the cap's plane in
    /// one circle, and the edge is the whole of that circle.
    ///
    /// **This says "one closed curve", and everything named with it has one.** A
    /// plane cuts a ball in a circle, a plane cuts a pipe in an ellipse, two
    /// balls cross in a circle, and a degree-four crossing that leaves a single
    /// loop is one too.
    ///
    /// Where a quartic leaves **two** disjoint loops this variant is not what
    /// says so: each loop comes back as a [`EdgeRun::PartOfWhereItCrosses`] with
    /// the half-space that holds it, which is the whole of what `picked_by` is
    /// for. §27.3 filed that as a gap while nothing built a quartic; M3's second
    /// stage built them and named them that way, so this claim is true again
    /// (§29.13). What is still outside the vocabulary is a crossing that runs
    /// *through itself*, and that is refused by name rather than said badly.
    RightRoundWhereItsFacesCross,
    /// Part of the curve where the face also crosses `and`, cut down to this
    /// one piece by `picked_by`.
    ///
    /// A ball's seam is why this exists, and it is worth saying exactly why
    /// neither simpler variant can hold it.
    ///
    /// The seam runs pole to pole down one meridian, so the faces either side
    /// of it are the *same* sphere — and a sphere crossed with itself names
    /// nothing at all. `and` is the missing second surface: the plane the
    /// meridian lies in.
    ///
    /// The endpoints do not finish the job either, and this is the case that
    /// forced the design. A plane crosses a sphere in one circle; cutting a
    /// circle at two points leaves **two** arcs with the same two ends. So
    /// naming the crossing and naming the ends still names two things.
    /// `picked_by` settles it: half-spaces this edge lies in, each one exact,
    /// each answering with a `Decision` rather than with a rounded witness
    /// point somebody had to construct.
    PartOfWhereItCrosses {
        /// The second surface, when the faces either side do not name it.
        and: Surface,
        /// What this edge lies on or inside of, enough of it to pick one piece.
        ///
        /// **Not only planes.** A piece of a crossing that can only be told from
        /// its sibling by which side of a ball or a pipe it is on could not be
        /// named while this held planes — two circles where a ball crosses a
        /// pipe are the case M3 reaches, and
        /// [`SideOfSurface`](rcad_predicates::SideOfSurface) already answers
        /// exactly for both curved kinds (§24.5).
        ///
        /// **And not a [`Surface`] either**, which is what M4 changed. A piece
        /// that runs on for ever needs a picking normal exactly square to the
        /// direction it runs along, and a [`Plane`](crate::Plane) rounds that
        /// squareness away because it scales every normal to length one
        /// (§32.3). A [`HalfSpace`](crate::HalfSpace) does not, and it can be
        /// one only because it is never a face's geometry — which is what
        /// [`Picker`] says in the type system rather than in a comment (§33).
        picked_by: SmallVec<[Picker; 1]>,
    },
    /// Right round the closed curve where the face crosses `and`, so it starts
    /// and ends at the same vertex — for when the faces either side name no
    /// crossing at all: the same surface, as a frustum's split wall or a ball's
    /// seam, or, since Q1118, two different curved surfaces whose names
    /// `rcad-topo`'s `the_name_each_closed_curve_goes_by` has joined into one.
    ///
    /// **A frustum's wall split in two by a leaning plane is why this exists.**
    /// Both bands are on the one cone, so the pair either side of the rim
    /// between them names nothing, exactly as a ball's seam does; and the rim is
    /// closed, so [`EdgeRun::PartOfWhereItCrosses`] — which is *part* of a
    /// crossing, cut down by `picked_by` — is not what it is either. Without
    /// this the rim could only be said as
    /// [`EdgeRun::RightRoundWhereItsFacesCross`], and a reader asked where it
    /// runs has one point on a cone and nothing else. One point on a cone does
    /// not name an ellipse: it names the circle through it, which is a
    /// **different curve**, and that wrong answer is measured at 1.45 mm on a
    /// 100 mm solid by `rcad-topo/tests/a_leaning_cut_round_one_cone.rs`.
    ///
    /// **A dome on a shaft of the dome's own radius is the same with two
    /// different faces either side.** The rim where the cap's plane crosses
    /// the ball and the pipe is a closed curve under two names, one from each
    /// face, and `rcad-topo`'s fuse joins them before this is written only
    /// where the plane and the two curved surfaces meet in more than a point
    /// (`meet_of_three_surfaces`), so `and` is that plane rather than the
    /// pipe's or the ball's own partner. A frustum against a bar of the
    /// frustum's narrow radius, and two pointed cones base to base, are the
    /// same join with a cone standing in for the ball (Q1118).
    ///
    /// So `and` is the missing second surface, the same job it does in
    /// [`EdgeRun::PartOfWhereItCrosses`], and no `picked_by`: the edge is the
    /// whole of the closed curve rather than a piece of it, which is the whole
    /// difference between the two.
    ///
    /// **Last in the list, and that is not tidiness.** A [`Model`] is saved with
    /// postcard, which writes an enum as its variant's *index*, so putting this
    /// third would have moved `PartOfWhereItCrosses` from 2 to 3 and silently
    /// changed the meaning of every saved byte string carrying an edge run.
    /// Measured: put third, the checked-in writer-fuzz seed `two_cubes_that_write`
    /// stopped decoding at all —
    /// `rcad-step/tests/every_model_the_writer_fuzzer_kept_is_written_again.rs`
    /// went red on `decoded: 0`. A new variant goes on the end.
    ///
    /// **This closes**, and `rcad-topo`'s `EdgeView::is_closed` is the one list
    /// that has to say so. `docs/decisions.md` §216.
    ///
    /// [`Model`]: https://docs.rs/rcad-topo
    RightRoundWhereItCrosses {
        /// The second surface, which the faces either side do not name.
        and: Surface,
    },
}

impl EdgeRun {
    /// Whether this run goes right round a closed curve, so that its two ends
    /// are one point and the curve it names has to be a closed one.
    ///
    /// **Two variants close, and the one list of them is here.** Both
    /// [`NamedRun::new`] and `walk`'s `where_a_plane_cuts_a_cone` tested only
    /// [`EdgeRun::RightRoundWhereItsFacesCross`] by name, so a run written the
    /// §216 way slipped past each of them — a closed run with two different
    /// ends built without complaint, and an open unbounded parabola handed back
    /// for a run the caller had already been told closes
    /// (`docs/decisions.md` §219.7).
    pub(crate) fn it_comes_back_to_where_it_started(&self) -> bool {
        match self {
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. } => {
                true
            }
            EdgeRun::Straight | EdgeRun::PartOfWhereItCrosses { .. } => false,
        }
    }

    /// This run, shifted by `by`.
    ///
    /// **A run is written in the frame its face is stored in**, so anything that
    /// works a face out somewhere else has to bring the run along or it picks a
    /// piece of a crossing that is not there. `rcad-topo`'s mesher shifts a face
    /// stored far from the origin until its numbers are small, and this is how
    /// the run goes with it.
    ///
    /// A shift keeps every picking normal bit for bit, which
    /// [`HalfSpace::shifted`] is for and [`EdgeRun::transformed`] cannot
    /// promise. `rcad-topo`'s mesher wants that one.
    ///
    /// [`HalfSpace::shifted`]: crate::HalfSpace::shifted
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `by` has a component that is not a finite number.
    pub fn shifted(&self, by: Vector) -> Result<Self> {
        Ok(self.transformed(&Isometry::translation(by)?))
    }

    /// This run, moved by `movement`.
    ///
    /// [`EdgeRun::Straight`] and [`EdgeRun::RightRoundWhereItsFacesCross`] hold
    /// no geometry of their own and come back unchanged; they name the crossing
    /// of the two faces either side, and moving those faces moves the crossing.
    /// [`EdgeRun::PartOfWhereItCrosses`] carries a second surface and its
    /// pickers, and [`EdgeRun::RightRoundWhereItCrosses`] carries the second
    /// surface alone; each of those moves by its own type's rule.
    ///
    /// **The one thing this loses is the exact squareness a picker on an endless
    /// piece was written with**, and [`HalfSpace::transformed`] sets out why —
    /// including which of the fixes is a limit there and a gap here. This is
    /// where the gap is: the direction the piece runs along is the crossing of
    /// the face's surface with `and`, so a fix could write a fresh normal
    /// exactly square to the *moved* direction rather than turning the old one.
    /// Nothing calls that due yet, so it is not built (§13).
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        match self {
            EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross => self.clone(),
            EdgeRun::RightRoundWhereItCrosses { and } => EdgeRun::RightRoundWhereItCrosses {
                and: and.transformed(movement),
            },
            EdgeRun::PartOfWhereItCrosses { and, picked_by } => EdgeRun::PartOfWhereItCrosses {
                and: and.transformed(movement),
                picked_by: picked_by
                    .iter()
                    .map(|picker| picker.transformed(movement))
                    .collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use rcad_predicates::Interval;
    use smallvec::smallvec;

    use super::{EdgeRun, Error, NamedRun, PredicatePoint, SurfaceMeet};
    use crate::{
        Axis, Cylinder, HalfSpace, Isometry, Picker, Plane, Point, Sphere, Surface, Vector,
    };

    fn a_pole(up: f64) -> PredicatePoint {
        PredicatePoint::from([0.0, 0.0, up])
    }

    /// The vocabulary has to hold what the kernel already builds, or the split
    /// moved the type somewhere it cannot do its job.
    ///
    /// A ball's seam is the hardest of the three: both faces along it are the
    /// same sphere, both its ends are poles, and only the meridian plane and the
    /// half-space picking one side of it say which half of that circle it is.
    #[test]
    fn a_balls_seam_is_a_named_run_between_its_two_poles() {
        let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal");
        let facing_x =
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis has length");
        let seam = NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(meridian),
                picked_by: smallvec![Picker::from(facing_x)],
            },
            [a_pole(3.0), a_pole(-3.0)],
        )
        .expect("a picked run does not have to come back to where it started");

        let SurfaceMeet::Curves(runs) = SurfaceMeet::Curves(vec![seam.clone()]) else {
            panic!("just built one");
        };
        assert_eq!(runs.as_slice(), core::slice::from_ref(&seam));
        assert_eq!(runs[0].ends(), [a_pole(3.0), a_pole(-3.0)]);
        let EdgeRun::PartOfWhereItCrosses { and, picked_by } = runs[0].run() else {
            panic!("a seam is part of where its face crosses a meridian");
        };
        assert_eq!(and, &Surface::Plane(meridian));
        assert_eq!(picked_by.as_slice(), [Picker::from(facing_x)]);
    }

    /// The half-space that picks a piece may be curved, which is the whole of
    /// what §24.5 widened.
    #[test]
    fn a_piece_of_a_crossing_can_be_picked_out_by_a_ball_or_a_pipe() {
        for picker in [
            Picker::from(Sphere::new(Point::origin(), 1.0).expect("a radius")),
            Picker::from(Cylinder::new(Axis::z(), 1.0).expect("a radius")),
        ] {
            let run = EdgeRun::PartOfWhereItCrosses {
                and: Surface::Sphere(Sphere::new(Point::origin(), 3.0).expect("a radius")),
                picked_by: smallvec![picker],
            };
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = &run else {
                unreachable!("just built one");
            };
            assert_eq!(picked_by.as_slice(), [picker]);
        }
    }

    /// A run that goes right round comes back to where it started, and a pair
    /// of different ends on one names a curve that is not there.
    ///
    /// `Model` derives `Deserialize`, so nothing between a caller's bytes and
    /// this constructor guarantees the two ends agree — this has to be an
    /// `Err`, not a crash, or a hand-built model can bring the kernel down
    /// (CLAUDE.md's rule on panics vs. bad caller input).
    #[test]
    fn a_closed_run_with_two_different_ends_is_bad_caller_input() {
        assert_eq!(
            NamedRun::new(
                EdgeRun::RightRoundWhereItsFacesCross,
                [a_pole(1.0), a_pole(-1.0)],
            ),
            Err(Error::AClosedRunDoesNotComeBackToWhereItStarted {
                start: Box::new(a_pole(1.0)),
                end: Box::new(a_pole(-1.0)),
            })
        );
    }

    /// Two surfaces name a curve and nothing on it, so what comes back from
    /// crossing them has no ends — and that has to be tellable from an end
    /// somebody failed to work out.
    ///
    /// It is: a run with no ends says so, and a run whose end was undecided
    /// never reaches a caller, because §24.10 refuses such an answer whole.
    #[test]
    fn a_run_a_pair_of_surfaces_names_has_no_ends_and_says_so() {
        let circle = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");
        assert!(circle.is_the_whole_curve());
        assert_eq!(circle.ends()[0], circle.ends()[1]);
        assert_eq!(circle.ends()[0].uncertainty(), f64::INFINITY);
        assert_eq!(circle.ends()[0].exact_coordinates(), None);

        let between_two_poles =
            NamedRun::new(EdgeRun::RightRoundWhereItsFacesCross, [a_pole(3.0); 2])
                .expect("the same point twice does come back to where it started");
        assert!(!between_two_poles.is_the_whole_curve());
    }

    /// A piece of a crossing with no ends and nothing picking it says nothing
    /// about which piece it is, and two of those are the same run twice.
    ///
    /// `picked_by` is a public field of a public enum variant, so a caller
    /// can build this by hand with no help from this crate at all — bad
    /// caller input, and `Err` rather than a crash.
    #[test]
    fn a_piece_of_a_crossing_with_neither_ends_nor_a_half_space_is_bad_caller_input() {
        let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal");
        assert_eq!(
            NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(meridian),
                picked_by: smallvec![],
            }),
            Err(Error::ACrossingPieceHasNoEndsAndNothingPickingIt)
        );
    }

    /// Moving a run takes its second surface and every one of its pickers along,
    /// and leaves the two variants that hold no geometry alone.
    ///
    /// A run left half moved names a piece of a crossing that is not there —
    /// the picker would still be cutting the crossing where the face used to be
    /// — so this checks every piece against what its own type does, rather than
    /// only that something changed.
    #[test]
    fn moving_a_run_moves_its_second_surface_and_every_one_of_its_pickers() {
        let movement = Isometry::rotation(&Axis::z(), 0.7)
            .expect("a finite angle about a finite axis")
            .compose(&Isometry::translation(Vector::new(2.5, -7.25, 11.0)).expect("a shift"));
        let meridian = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal"),
        );
        let pickers = [
            Picker::from(
                HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                    .expect("the x axis has length"),
            ),
            Picker::from(Sphere::new(Point::origin(), 1.0).expect("a radius")),
            Picker::from(Cylinder::new(Axis::z(), 1.0).expect("a radius")),
        ];

        let moved = EdgeRun::PartOfWhereItCrosses {
            and: meridian,
            picked_by: pickers.into_iter().collect(),
        }
        .transformed(&movement);
        let EdgeRun::PartOfWhereItCrosses { and, picked_by } = &moved else {
            unreachable!("a picked piece stays a picked piece");
        };
        assert_eq!(and, &meridian.transformed(&movement));
        for (was, is) in pickers.iter().zip(picked_by.as_slice()) {
            assert_eq!(&was.transformed(&movement), is);
        }

        for plain in [EdgeRun::Straight, EdgeRun::RightRoundWhereItsFacesCross] {
            assert_eq!(plain.transformed(&movement), plain);
        }
    }

    /// §16.10 for [`EdgeRun::shifted`] going through [`EdgeRun::transformed`]:
    /// a movement with no turn still moves a whole run by the shift and nothing
    /// else.
    ///
    /// `rcad-topo`'s mesher shifts a face stored far from the origin into a
    /// frame where its numbers are small, and brings the run along. That whole
    /// frame rests on the shift being exact, so a shift that had quietly started
    /// paying for a turn of the identity would understate what a mesh point
    /// costs — and nothing in the mesher would say so.
    #[test]
    fn shifting_a_run_is_moving_it_by_a_movement_with_no_turn() {
        let by = Vector::new(2.5, -7.25, 11.0);
        let run = EdgeRun::PartOfWhereItCrosses {
            and: Surface::Sphere(Sphere::new(Point::new(1.0, 2.0, 3.0), 3.0).expect("a radius")),
            picked_by: smallvec![
                Picker::from(HalfSpace::new(Vector::new(0.0, 5.0, -3.0), 2.5).expect("a normal")),
                Picker::from(Cylinder::new(Axis::z(), 1.0).expect("a radius")),
            ],
        };
        assert_eq!(
            run.shifted(by).expect("a finite shift"),
            run.transformed(&Isometry::translation(by).expect("a finite shift"))
        );
    }

    /// An end of a computed crossing is usually a box rather than a point, and
    /// the type has to say so or a predicate downstream is handed a rounded
    /// witness and answers confidently about the wrong place (§2.5).
    #[test]
    fn an_end_that_is_only_known_to_be_inside_a_box_stays_a_box() {
        let somewhere = PredicatePoint::Constructed([
            Interval::new(0.5, 0.5 + f64::EPSILON).expect("a range"),
            Interval::exactly(0.0),
            Interval::exactly(0.0),
        ]);
        let run = NamedRun::new(EdgeRun::Straight, [somewhere, somewhere])
            .expect("a straight run never has to come back to where it started");
        assert!(run.ends()[0].uncertainty() > 0.0);
        assert_eq!(run.ends()[0].exact_coordinates(), None);
    }
}
