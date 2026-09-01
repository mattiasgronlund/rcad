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

use rcad_predicates::Point as PredicatePoint;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::Surface;

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
}

impl NamedRun {
    /// The piece of curve `run` names, running from `ends[0]` to `ends[1]`.
    ///
    /// # Panics
    ///
    /// If `run` goes right round a closed curve and the two ends are not the
    /// same point. A closed run comes back to where it started by definition, so
    /// a pair of different ends on one names a curve that does not exist, and
    /// §4.1 ranks the crash above handing that on.
    #[must_use]
    pub fn new(run: EdgeRun, ends: [PredicatePoint; 2]) -> Self {
        assert!(
            run != EdgeRun::RightRoundWhereItsFacesCross || ends[0] == ends[1],
            "a run that goes right round comes back to where it started, but {:?} and {:?} are \
             different points",
            ends[0],
            ends[1]
        );
        Self { run, ends }
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
    /// **This says "one closed curve", and the four crossings that can be named
    /// exactly all have one.** A plane cuts a ball in a circle, a plane cuts a
    /// pipe in an ellipse, two balls cross in a circle. A quartic is where it
    /// stops being enough: a ball crossing a pipe off-axis can leave two
    /// disjoint closed loops, and then this variant names both at once. The
    /// half-spaces of [`EdgeRun::PartOfWhereItCrosses`] are what would tell them
    /// apart, and nothing builds a quartic yet, so the gap is named here rather
    /// than filled (§27.3).
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
        /// Surfaces this edge lies on or outside of, enough of them to pick one
        /// piece.
        ///
        /// **Not only planes.** A piece of a crossing that can only be told from
        /// its sibling by which side of a ball or a pipe it is on could not be
        /// named while this held planes — two circles where a ball crosses a
        /// pipe are the case M3 reaches, and
        /// [`SideOfSurface`](rcad_predicates::SideOfSurface) already answers
        /// exactly for both curved kinds (§24.5).
        picked_by: SmallVec<[Surface; 1]>,
    },
}

#[cfg(test)]
mod tests {
    use rcad_predicates::Interval;
    use smallvec::smallvec;

    use super::{EdgeRun, NamedRun, PredicatePoint, SurfaceMeet};
    use crate::{Axis, Cylinder, Plane, Point, Sphere, Surface, Vector};

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
        let facing_x = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis is a unit normal");
        let seam = NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(meridian),
                picked_by: smallvec![Surface::Plane(facing_x)],
            },
            [a_pole(3.0), a_pole(-3.0)],
        );

        let SurfaceMeet::Curves(runs) = SurfaceMeet::Curves(vec![seam.clone()]) else {
            panic!("just built one");
        };
        assert_eq!(runs.as_slice(), core::slice::from_ref(&seam));
        assert_eq!(runs[0].ends(), [a_pole(3.0), a_pole(-3.0)]);
        let EdgeRun::PartOfWhereItCrosses { and, picked_by } = runs[0].run() else {
            panic!("a seam is part of where its face crosses a meridian");
        };
        assert_eq!(and, &Surface::Plane(meridian));
        assert_eq!(picked_by.as_slice(), [Surface::Plane(facing_x)]);
    }

    /// The half-space that picks a piece may be curved, which is the whole of
    /// what §24.5 widened.
    #[test]
    fn a_piece_of_a_crossing_can_be_picked_out_by_a_ball_or_a_pipe() {
        for picker in [
            Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a radius")),
            Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a radius")),
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

    /// A run that goes right round comes back to where it started, and a pair of
    /// different ends on one names a curve that is not there.
    #[test]
    #[should_panic(expected = "comes back to where it started")]
    fn a_closed_run_with_two_different_ends_is_a_bug() {
        let _ = NamedRun::new(
            EdgeRun::RightRoundWhereItsFacesCross,
            [a_pole(1.0), a_pole(-1.0)],
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
        let run = NamedRun::new(EdgeRun::Straight, [somewhere, somewhere]);
        assert!(run.ends()[0].uncertainty() > 0.0);
        assert_eq!(run.ends()[0].exact_coordinates(), None);
    }
}
