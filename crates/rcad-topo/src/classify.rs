//! Inside a solid, outside it, on its skin, or honestly not sure.
//!
//! # Two ways in, one body
//!
//! [`SolidView::classify_point`] takes a point with exact coordinates, which is
//! every point M0 ever had. [`SolidView::classify_uncertain_point`] takes one
//! that may be known only as a box — the meet of three planes, which is what
//! the fuse of decisions.md §1.7 asks about. The first is the second with an
//! exact point handed in. That is the whole arrangement, and it is deliberate:
//! §1.7's third exit criterion asks for classification *reused*, and a second
//! ray caster that took boxes would agree with this one right up until it did
//! not.
//!
//! # Where the fourth answer starts happening
//!
//! decisions.md §12.5 records that the enclosure and the provenance this
//! returns "carry no signal today ... and cannot until classification meets
//! constructed points". That is exactly right, and this is where it stops being
//! true. Ask about an exact point under a certified policy **on a model whose
//! coordinates the arithmetic can actually multiply**, and every `orient3d`
//! falls through to exact arithmetic and proves its sign: nothing is ever
//! recorded, so the log stays empty, and the only way left to
//! [`Containment::Undecidable`] is all three rays meeting an edge dead on,
//! which reports an enclosure of zero. Both fields are carried and neither
//! says anything.
//!
//! That qualification is not decoration, but the example that used to be here
//! was wrong twice over and is worth keeping as a correction. It said a `1e140`
//! cube returns `Undecidable` with an enclosure of infinity and thirty-six
//! records, because the ray target overflowed. Measured: it returns `Inside`
//! with an **empty log**, and has since `a23d73d` taught the predicates to
//! rescale; the ray target does not overflow until a side of `7.7e307`, and
//! what overflowed at `1e140` was inside `orient3d`. `docs/decisions.md` §20.7
//! records both mistakes rather than quietly dropping them, and §16.11 — which
//! carries the same wrong cause — is corrected in place.
//!
//! Ask about a point that is only known to lie inside a box and both start
//! meaning something. Exact arithmetic cannot run on a coordinate nobody has,
//! so a box straddling a face's plane leaves the predicate genuinely undecided
//! and the answer is [`Containment::Undecidable`] carrying the width that
//! straddled. Under a tolerant policy the same query resolves instead, and the
//! log picks up a record of what was accepted to get there.

use rcad_geom::{
    EdgeRun, MeetsAt, Plane, Point, Surface, WhereALineMeetsIt, where_a_line_meets_a_surface,
};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, orient3d_of_points,
    orient3d_tolerantly,
};

use crate::validate::side_of_surface;
use crate::{Error, FaceKey, FaceView, OnAFace, SolidView, WireView};

/// Where a point sits relative to a solid.
///
/// The fourth answer is the whole reason this is not a `bool`. A ray that
/// grazes an edge exactly, or a predicate that overflows, leaves the question
/// genuinely open, and saying so is better than picking a side (decisions.md
/// §2.1).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Containment {
    /// Inside the material.
    Inside,
    /// Exactly on one of the solid's faces, edges or corners.
    OnBoundary,
    /// Outside the material.
    Outside,
    /// Could not tell.
    Undecidable {
        /// How wide the widest enclosure that straddled zero was.
        ///
        /// Zero when no predicate was in doubt and the geometry was: every ray
        /// tried met an edge or a corner dead on, which is exact and still
        /// tells us nothing; or a ray met a flat face whose stored corners and
        /// stored plane put the point on different sides, or the point is
        /// beside such a face, and another face was too near for the plane
        /// alone to place it.
        enclosure: f64,
    },
    /// One face of the solid is trimmed in a way
    /// [`SolidView::face_holds_the_point`] cannot follow, so no ray was ever
    /// counted. A point another face holds is [`Containment::OnBoundary`]
    /// instead, as it is on the same solid without that face (Q1722). A box
    /// whose every ray another face grazes, as a box straddling a flat face is
    /// grazed by it, is [`Containment::Undecidable`]: no answer from the
    /// trimming would have counted those rays either (Q1727). The face whose
    /// trimming nothing follows throws no ray away where it grazes it only for
    /// want of showing it holds none of the box (Q1740).
    ///
    /// Deliberately not [`Containment::Undecidable`]. That one means every
    /// predicate was asked and none could prove an answer; this means the
    /// question was never put. Reading the second as the first would let a
    /// caller loosen a policy and expect it to help, and nothing here would get
    /// better for it (`docs/decisions.md` §4.2 keeps the two apart).
    ///
    /// This is what is left of `NeedsFlatFaces`, which said the same about every
    /// solid with a curved face on it. A bar, a ball and a block with a round
    /// hole through it are counted like any polyhedron now; what is left is a
    /// face trimmed by a *piece* of a curve rather than by whole ones (§35).
    CannotFollowAFacesTrimming {
        /// Which face.
        face: FaceKey,
    },
    /// One face of the solid sits on a surface no ray can be fired at, so no
    /// ray was ever counted.
    ///
    /// **A torus, and nothing else.** Where a ray meets a plane, a ball, a pipe
    /// or a cone is answered whatever the ray's own numbers are.
    /// `rcad_geom::where_a_line_meets_a_surface` answers a torus too, but only
    /// for a line whose coordinates are exact, and [`SolidView::classify_point`]
    /// gives it one — "a corner on the lattice", in that function's own words —
    /// only when the point being classified and the ray built from it both land
    /// on numbers a line can be written out of exactly. Most rays do not, and
    /// then an exact point beside the point is fired from instead, once no face
    /// of the solid is shown to meet the box round both (`docs/decisions.md`
    /// §323). This is what is left: a point no point beside it speaks for,
    /// since a face meets it or every such ray grazes something, and whose own
    /// rays only the ring threw away, for want of being fired at. A point
    /// another face holds is [`Containment::OnBoundary`], and a box whose every
    /// ray another face grazes is [`Containment::Undecidable`], as they are on
    /// the same solid without the ring, even where the box meets the ring too
    /// (Q1716, Q1739). So is a box the ring's own sign cannot place and its
    /// face holds some of, as the same box on a ball's wall is: the ring
    /// grazes every ray from it before any is fired, so firing one would not
    /// answer it (`docs/decisions.md` §323.3, Q1763).
    ///
    /// Deliberately its own answer and not
    /// [`Containment::CannotFollowAFacesTrimming`], which says a *trimming*
    /// could not be followed, and not [`Containment::Undecidable`], which says
    /// every predicate was asked and none could prove an answer. Here the
    /// question was never answered, and no policy makes it go away
    /// (`docs/decisions.md` §4.2).
    CannotFireARayAtAFacesSurface {
        /// Which face.
        face: FaceKey,
    },
}

impl SolidView<'_> {
    /// Say whether `point` is inside the solid, outside it, or on its skin.
    ///
    /// This is [`SolidView::classify_uncertain_point`] with a point whose
    /// coordinates are exact, and there is no other difference between them.
    ///
    /// # Errors
    ///
    /// Nothing here fails; not being able to tell is an answer, not a failure.
    #[must_use]
    pub fn classify_point(
        self,
        point: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Containment {
        self.classify_uncertain_point(PredicatePoint::from(point.coordinates()), policy, log)
    }

    /// Say whether a point known only to lie inside a box is inside the solid,
    /// outside it, or on its skin.
    ///
    /// `point` may have exact coordinates or be the meet of three planes, which
    /// is a box and never an `f64` triple in general. The box is where the
    /// undecided answer comes from: exact arithmetic cannot run on it, so a box
    /// straddling a face's plane straddles zero in the predicate too and stays
    /// straddling however much precision is thrown at it (decisions.md §2.5).
    ///
    /// # How it works
    ///
    /// A ray is fired from `point` towards a far-off target, and every place it
    /// passes through the skin is counted, `+1` for a crossing that leaves the
    /// material and `-1` for one that enters it. The total is one inside and
    /// zero outside.
    ///
    /// **Where those places are is asked two ways, and the second is the general
    /// one.** A flat face whose every side is straight fans into triangles that
    /// cover it, and a ray against a triangle is four `orient3d`s over
    /// coordinates the model holds — nothing is built and a polyhedron is
    /// decided at the exact level every time. Every other face — every curved
    /// one, and every flat one with a curved side, which a round cap is — is
    /// asked where the ray meets its *surface*, which is one equation in one
    /// unknown of degree at most two, and then whether that place is on the
    /// face. Those roots are the only thing here that is built rather than read,
    /// and they are built as boxes.
    ///
    /// A face whose surface holds `point` is asked separately whether its
    /// material holds it too, and a face that says yes puts the point on the
    /// boundary and that answer wins. One triangle of the fan holding the point
    /// is not that question: the fan overlaps itself and covers a hole twice
    /// over, once each way, so only the whole outline can say (see
    /// [`SolidView::face_holds_the_point`]).
    ///
    /// **A triangle is the face only where it agrees with the face's stored
    /// plane**, which is the face (`docs/decisions.md` §17.1), and a turned
    /// solid stores its corners a rounding off that plane. So where a ray's
    /// count turns on which side of a triangle the point is, the stored plane
    /// is asked too; where the two part, no count is trusted and the plane next
    /// to the point places it, or nothing does and the answer is
    /// [`Containment::Undecidable`] (`SolidView::what_its_plane_says_beside_it`).
    /// A point beside a face whose corners and plane part about it is placed
    /// the same way before any ray is fired, whether a ray's line would meet
    /// that face or not (`SolidView::what_a_face_it_is_beside_says`).
    ///
    /// The ray's far target is picked from [`SolidView::box_around_it`] and is
    /// exact. `point` is the only thing the *caller* may hand over as a box, and
    /// it is handed to every predicate as it is — nothing here rounds it down to
    /// a representative coordinate first.
    ///
    /// If the ray meets an edge of a triangle exactly, or a root cannot be told
    /// from the point it started at, the count is not to be trusted, so the
    /// whole ray is thrown away and the next of three fixed targets is tried. If
    /// all three graze something, the answer is [`Containment::Undecidable`].
    /// **A root the face holds none of is no graze**, wherever along the ray it
    /// is, and nor is a point a surface's own sign cannot place, where that
    /// face holds none of it: neither puts anything on the face.
    ///
    /// # Why the answer never wobbles
    ///
    /// The benchmark decisions.md §9.2 asks for compares a serial run against a
    /// parallel one bit for bit, so nothing here may depend on when or where it
    /// runs:
    ///
    /// - The three ray targets come from the solid's own box, and the `min` and
    ///   `max` that build it do not care what order they see a face in.
    /// - Faces, wires and triangles come out in stored order, which is the same
    ///   on every call, and the two roots of a quadratic come out in the order
    ///   the sign in front of the square root puts them.
    /// - The count is an integer. There is no float sum whose result could
    ///   depend on the order it was added up in.
    /// - The model is only ever read. There is no cache to warm and nothing to
    ///   share between calls.
    ///
    /// # Errors
    ///
    /// Nothing here fails; not being able to tell is an answer, not a failure.
    /// Anything the cascade could not certify of what the answer rests on lands
    /// in `log`, and a fully certified classification leaves it untouched.
    ///
    /// **No answer rests on a ray thrown away**, so what that ray could not
    /// tell is kept out of `log` wherever a later ray, an exact point beside
    /// this one or a face's plane places the point. Where none does, it is why
    /// none did, and it lands there. A fuse refuses, under a certified policy,
    /// any placement whose log holds a record, and two quarter bends whose caps
    /// touch were refused for a point whose first ray could not tell `side of
    /// torus` on a cap's rim and whose next placed it
    /// (`tests/a_ray_thrown_away_leaves_no_record.rs`, Q2267, Q2407).
    #[must_use]
    pub fn classify_uncertain_point(
        self,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Containment {
        let Some((low, high)) = self.box_around_it() else {
            return Containment::Outside;
        };
        if is_outside_the_box(point, low, high)
            && self
                .box_opened_out_to_where_its_planes_meet(low, high)
                .is_some_and(|(low, high)| is_outside_the_box(point, low, high))
        {
            return Containment::Outside;
        }
        if let Some(placed) = self.what_a_face_it_is_beside_says(point, policy, log) {
            return placed;
        }

        let mut worst_enclosure = 0.0f64;
        let mut asked_beside = false;
        let mut thrown_away = ProvenanceLog::new();
        for target in ray_targets(low, high) {
            let mut this_ray = ProvenanceLog::new();
            let pass = self.count_crossings(point, target, policy, &mut this_ray);
            if pass.on_boundary {
                this_ray.drain_into(log);
                return Containment::OnBoundary;
            }
            if let Some(face) = pass.unfollowed_at
                && !pass.is_thrown_away_without_them()
            {
                thrown_away.drain_into(log);
                this_ray.drain_into(log);
                return Containment::CannotFollowAFacesTrimming { face };
            }
            if let Some(face) = pass.unfired_at {
                if !asked_beside {
                    asked_beside = true;
                    if let Some(placed) =
                        self.what_a_point_beside_it_says(point, low, high, policy, log)
                    {
                        return placed;
                    }
                }
                if !pass.is_thrown_away_without_them() {
                    thrown_away.drain_into(log);
                    this_ray.drain_into(log);
                    return Containment::CannotFireARayAtAFacesSurface { face };
                }
            }
            if let Some(face) = pass.parted_at {
                if let Some(placed) = self.what_its_plane_says_beside_it(face, point, policy, log) {
                    return placed;
                }
                thrown_away.drain_into(log);
                this_ray.drain_into(log);
                return Containment::Undecidable { enclosure: 0.0 };
            }
            if !pass.grazed_something {
                this_ray.drain_into(log);
                return if pass.winding == 0 {
                    Containment::Outside
                } else {
                    Containment::Inside
                };
            }
            worst_enclosure = worst_enclosure.max(pass.worst_enclosure);
            this_ray.drain_into(&mut thrown_away);
        }
        thrown_away.drain_into(log);
        Containment::Undecidable {
            enclosure: worst_enclosure,
        }
    }

    /// What a flat face whose corners and stored plane read the point
    /// differently says of it, where the point is beside that face; `None` when
    /// it is beside no such face, and the ray is to count it.
    ///
    /// **The ray asks a face's stored plane only where its line passes through
    /// one of that face's triangles**, and it finds those by the triangles'
    /// sides, which are the corners'. The line from a point in one face's
    /// sliver within a sliver's depth of an edge can pass through the
    /// neighbouring face's triangle instead, or through neither, and then the
    /// plane was never asked and the corners' answer came back where the stored
    /// planes say the other (`docs/decisions.md` §346.6, Q997). So every flat
    /// face whose corners and plane part about the point is found first
    /// ([`its_corners_and_plane_part_at`]), whatever the line meets.
    ///
    /// **Such a face matters only beside the point.** Its plane runs on past
    /// its edges, and so does the sliver between that plane and its triangles'
    /// planes. A point in that sliver far from the face, as the cap a turned
    /// block's face cuts off a ball `0.3` past the block is, is left to the
    /// ray, which reads it the same both ways. A face is beside the point
    /// unless it holds none of the point's box stretched to its plane. Where
    /// two faces part about the point both are taken as beside it: that is the
    /// strip between their corners' edge and their planes' meeting, which
    /// neither face's corners hold and only their planes place.
    ///
    /// Each face beside the point is asked what its plane says
    /// ([`SolidView::what_its_plane_says_beside_it`]), and the first that can
    /// show it answers. If none can, the answer is [`Containment::Undecidable`].
    fn what_a_face_it_is_beside_says(
        self,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Option<Containment> {
        let parted: Vec<FaceView<'_>> = self
            .faces()
            .filter(|&face| {
                let surface = face.surface();
                surface.as_plane().is_some_and(|flat| {
                    every_side_of_it_is_straight(face)
                        && its_corners_and_plane_part_at(face, flat, point)
                })
            })
            .collect();
        let beside: Vec<FaceView<'_>> = if parted.len() > 1 {
            parted
        } else {
            parted
                .into_iter()
                .filter(|&face| self.is_beside(face, point, policy, log))
                .collect()
        };
        if beside.is_empty() {
            return None;
        }
        Some(
            beside
                .into_iter()
                .find_map(|face| self.what_its_plane_says_beside_it(face.key(), point, policy, log))
                .unwrap_or(Containment::Undecidable { enclosure: 0.0 }),
        )
    }

    /// Whether `face` may hold a point of `point`'s box stretched to its plane:
    /// anything short of a proof that it holds none.
    ///
    /// **The proof is about the face its stored planes bound, not the outline
    /// its corners draw**, which a turn leaves a rounding inside it at a
    /// corner. So the stretched box is grown first by as far as that can be
    /// (`SolidView::grown_to_where_its_planes_put_its_corners`), and only the
    /// outline's holding none of the grown box is taken as the proof. Asked of
    /// the stretched box alone, a point a rounding past a turned cube's corner,
    /// which every stored plane puts inside, was not beside a face whose
    /// corners stop short of it, and the ray counted it `Outside` by the
    /// corners (Q1156, Q2631).
    fn is_beside(
        self,
        face: FaceView<'_>,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> bool {
        let Some(reach) = stretched_to_the_plane(&face.plane(), point)
            .and_then(|reach| self.grown_to_where_its_planes_put_its_corners(face, reach))
        else {
            return true;
        };
        !matches!(
            self.what_it_holds_of_the_box(face, reach, policy, log),
            Ok(OnAFace::NotHeld)
        )
    }

    /// What every point of `point`'s box is, read off one exact point beside
    /// it, or `None` when that cannot be shown.
    ///
    /// **A ring needs its ray's line exact**, and a ray's line is exact only
    /// where the point is exact *and* so is its step to the target. A box has
    /// no exact line at all, and most exact points have none either, so a point
    /// against a solid with a torus face was refused whatever the solid was
    /// (`docs/decisions.md` §243.5). The answer exists: the point is somewhere,
    /// and wherever it is it is inside the solid or outside it.
    ///
    /// **One point speaks for a box only when no face meets the box.** A box is
    /// convex, so any two of its points are joined by a segment inside it; a
    /// segment that crosses no face does not go in or out of the solid, so both
    /// its ends are inside or both are outside. [`SolidView::no_face_meets`] is
    /// that proof, made of signs decided about the box — and a sign decided
    /// about a box holds for every point in it, which is the contract the ray's
    /// own roots are asked under already.
    ///
    /// So for each target an exact point whose step to it is exact is built
    /// beside the box ([`a_point_an_exact_ray_leaves_for`]), the box is grown
    /// to take that point in, and if no face meets the grown box the point is
    /// classified with an ordinary ray: what it gets, every point of the box
    /// gets. A target whose grown box a face meets, or whose ray grazes
    /// something, hands over to the next, and what it could not tell goes with
    /// it: only the target that answers puts anything in `log` (Q2407).
    ///
    /// **Except where its ray parts at a flat face** ([`Hit::ReadingsPart`]).
    /// [`SolidView::no_face_meets`] reads such a face by its triangles, so the
    /// grown box can be clear of them while the face's stored plane, which is
    /// the face, passes through it over the face: a point a rounding off a
    /// turned block's side, and a point beside it a few spacings away in the
    /// sliver between the side's triangles and its plane. The ray from the
    /// point beside leaves out the crossing it parts at, so its count is one
    /// off, and it put points on the wrong side of a turned block's side
    /// (`tests/a_point_beside_a_turned_flat_face.rs`, Q1068). So the face's
    /// plane places `point` itself, as it does where a ray from `point` parts
    /// ([`SolidView::what_its_plane_says_beside_it`]), and where it cannot the
    /// next target is tried.
    fn what_a_point_beside_it_says(
        self,
        point: PredicatePoint,
        low: Point,
        high: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Option<Containment> {
        for target in ray_targets(low, high) {
            let Some(beside) = a_point_an_exact_ray_leaves_for(point, target) else {
                continue;
            };
            let mut trial = ProvenanceLog::new();
            if !self.no_face_meets(the_box_round_both(point, beside), policy, &mut trial) {
                continue;
            }
            let pass = self.count_crossings(beside, target, policy, &mut trial);
            if pass.unfollowed_at.is_some() {
                continue;
            }
            assert!(
                !pass.on_boundary,
                "{beside:?} is in a box round {point:?} that no face of the solid meets, and a \
                 ray from it says it is on the solid's skin"
            );
            if pass.unfired_at.is_some() {
                continue;
            }
            if let Some(face) = pass.parted_at {
                let Some(placed) =
                    self.what_its_plane_says_beside_it(face, point, policy, &mut trial)
                else {
                    continue;
                };
                trial.drain_into(log);
                return Some(placed);
            }
            if !pass.grazed_something {
                trial.drain_into(log);
                return Some(if pass.winding == 0 {
                    Containment::Outside
                } else {
                    Containment::Inside
                });
            }
        }
        None
    }

    /// What every point of `point`'s box is, read off the stored plane of
    /// `face` alone, or `None` when another face comes too near for that to be
    /// shown.
    ///
    /// **Asked where the ray met `face` and its corners and its stored plane
    /// read the point differently** ([`Hit::ReadingsPart`]). The fuse cuts along
    /// the stored plane, which is the face (`docs/decisions.md` §17.1), and the
    /// ray fans the face's stored corners, which a turn leaves a rounding off
    /// that plane (§75.6). A point in the sliver between the two was put on one
    /// side of the face by the ray and on the other by the plane, or the ray
    /// could not settle its sign at all where the plane could: a ball whose cap
    /// a turned block's face takes off is a rounding deep, and every point of it
    /// is in that sliver (§346.6). Asked too of a face the point is beside
    /// whose corners and plane part about it, wherever the ray's line goes
    /// ([`SolidView::what_a_face_it_is_beside_says`]), and of a face where the
    /// ray from an exact point beside the point's box parted
    /// ([`SolidView::what_a_point_beside_it_says`]).
    ///
    /// **The plane speaks for the point when no other face is near.** The box is
    /// stretched along an axis until it holds a point of the plane as well
    /// ([`stretched_to_the_plane`]). If `face` holds every point of the
    /// stretched box and no other face holds any, the skin inside that box is
    /// `face` and nothing else, so the segment from the point to the plane
    /// reaches the skin only at the plane, where the face's material is on its
    /// inner side. The point is inside exactly when it is on that side.
    ///
    /// **No other flat face may part from its own plane about the stretched
    /// box**, even one whose plane the box is clear of. `face`'s corners say
    /// how far it reaches, and by its edge a neighbour's plane and corners can
    /// put the box on opposite sides: the box is then held by `face`'s corners
    /// and past the edge the stored planes meet along, and `face`'s plane
    /// placed it where the neighbour's plane says the other (Q997). Every
    /// sign here is decided about a whole box, so nothing is a tolerance; where
    /// a sign cannot be decided, or another face is near, the answer is `None`
    /// and the caller does not fall back on the ray it already distrusts.
    fn what_its_plane_says_beside_it(
        self,
        face: FaceKey,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Option<Containment> {
        let face = self
            .faces()
            .find(|used| used.key() == face)
            .expect("a ray parted at a face of this solid");
        let flat = face
            .surface()
            .as_plane()
            .copied()
            .expect("only a flat face's corners and plane can read a point differently");
        let reach = stretched_to_the_plane(&flat, point)?;
        let mut trial = ProvenanceLog::new();
        if !matches!(
            self.what_this_face_holds(face, reach, policy, &mut trial),
            Ok(OnAFace::Held)
        ) {
            return None;
        }
        for other in self.faces().filter(|other| other.key() != face.key()) {
            let surface = other.surface();
            if surface.as_plane().is_some_and(|other_flat| {
                every_side_of_it_is_straight(other)
                    && its_corners_and_plane_part_at(other, other_flat, reach)
            }) {
                return None;
            }
            let side = surface.as_plane().map_or_else(
                || side_of_surface(&surface, reach),
                |other_flat| side_of_a_flat_face(other_flat, reach),
            );
            if matches!(side, Decision::Pos | Decision::Neg) {
                continue;
            }
            if !matches!(
                self.what_this_face_holds(other, reach, policy, &mut trial),
                Ok(OnAFace::NotHeld)
            ) {
                return None;
            }
        }
        let inner = if face.orientation().is_reversed() {
            Decision::Pos
        } else {
            Decision::Neg
        };
        let placed = match side_of_a_flat_face(&flat, point) {
            Decision::Zero => Containment::OnBoundary,
            side if side == inner => Containment::Inside,
            Decision::Pos | Decision::Neg => Containment::Outside,
            Decision::Undecidable { .. } | _ => return None,
        };
        trial.drain_into(log);
        Some(placed)
    }

    /// Whether no face of this solid holds any point of `point`'s box.
    ///
    /// A face is kept clear of the box by the same description the ray counts
    /// it by. A flat face with straight sides is its fanned triangles, and the
    /// fan covers the face, so the box has to miss every one of them
    /// ([`misses_the_triangle`]), or the ones it meets have to add up to
    /// nothing there ([`SolidView::its_fan_adds_up_to_nothing_over`]). Every
    /// other face is the part of its surface it holds, so the box has to be off
    /// the surface, or the face has to hold none of the box.
    ///
    /// **Clear of a turned face's triangles is not clear of its stored
    /// plane**, which the triangles are a rounding off. A box a rounding off
    /// such a face, over it, can miss every triangle and still hold points on
    /// both sides of the plane. A ray from a point of the box between the two
    /// parts at the face wherever its line passes through it, and
    /// [`SolidView::what_a_point_beside_it_says`] then asks the plane rather
    /// than the ray (Q1068).
    fn no_face_meets(
        self,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> bool {
        self.faces().all(|face| {
            if face.surface().as_plane().is_some() && every_side_of_it_is_straight(face) {
                let mut clear = true;
                face.for_each_triangle(|corners| {
                    clear &= misses_the_triangle(corners, point, policy, log);
                });
                return clear || self.its_fan_adds_up_to_nothing_over(face, point, policy, log);
            }
            match side_of_surface(&face.surface(), point) {
                Decision::Pos | Decision::Neg => true,
                Decision::Zero | Decision::Undecidable { .. } | _ => matches!(
                    self.what_this_face_holds(face, point, policy, log),
                    Ok(OnAFace::NotHeld)
                ),
            }
        })
    }

    /// Whether the triangles of `face`'s fan that meet `point`'s box add up to
    /// nothing anywhere in it, for a flat face whose every corner is on its
    /// stored plane.
    ///
    /// **The fan of a face that is not convex reaches outside it.** Every wire
    /// is fanned from one corner ([`FaceView::for_each_triangle`]), so an
    /// L-shaped face fanned from the end of one arm puts triangles over the
    /// notch between the arms, and they come in pairs wound opposite ways. The
    /// ray counts each with its sign, so a ray through the notch is counted in
    /// and out again; a box in the notch meets those triangles and still meets
    /// nothing the ray counts, and no direction keeps it apart from a triangle
    /// it really is in.
    ///
    /// **Where every corner is on the stored plane, what the fan adds up to at a
    /// place on the plane is the winding of the wires round it**, the same for
    /// whichever corner it is fanned from, and that winding is what
    /// [`SolidView::what_this_face_holds`] counts. So where it says
    /// [`OnAFace::NotHeld`] of the box — a sign decided about every point of
    /// it, §323.3's contract — the fan adds up to nothing at every place where
    /// the box meets the plane, and a segment inside the box crosses the face
    /// as often one way as the other. Every corner is asked by the plane's own
    /// predicate, which is exact for a stored corner.
    ///
    /// **A face whose corners are off its stored plane is not answered.** A
    /// turn leaves them a rounding off it (§75.6), and then the two triangles
    /// of a pair are not in one plane: between them is a sliver a rounding
    /// thick that one of them counts and the other does not, and a box that
    /// straddles the plane takes it in.
    fn its_fan_adds_up_to_nothing_over(
        self,
        face: FaceView<'_>,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> bool {
        let flat = face
            .surface()
            .as_plane()
            .copied()
            .expect("only a flat face is fanned");
        let every_corner_on_the_plane = face.wires().flat_map(WireView::points).all(|corner| {
            flat.side_of_point(PredicatePoint::from(corner.coordinates())) == Decision::Zero
        });
        every_corner_on_the_plane
            && matches!(
                self.what_this_face_holds(face, point, policy, log),
                Ok(OnAFace::NotHeld)
            )
    }

    /// One ray's tally.
    ///
    /// A face whose trimming nothing can follow, or whose surface the ray
    /// cannot be fired at, does not stop the count: it is noted in the tally,
    /// and every other face is still asked whether it holds the point or
    /// grazes the ray, since neither depends on that face (Q1716, Q1722).
    ///
    /// Each face is tallied on its own and then added in, so that a graze is
    /// known to stand or to be for want of an answer the face held back
    /// ([`Crossings::add`], Q1727).
    fn count_crossings(
        self,
        point: PredicatePoint,
        target: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Crossings {
        let mut tally = Crossings::default();
        for face in self.faces() {
            let mut its_own = Crossings::default();
            if face.surface().as_plane().is_some() && every_side_of_it_is_straight(face) {
                self.count_a_polygon(face, &mut its_own, point, target, policy, log);
            } else {
                self.count_where_the_ray_meets_it(face, &mut its_own, point, target, policy, log);
            }
            tally.add(its_own);
        }
        tally
    }

    /// What the ray does to one flat face whose every side is straight: its
    /// triangles, and then the outline when the point turns out to be on its
    /// plane.
    ///
    /// **Nothing here is built**, which is the whole reason this route is kept
    /// beside the general one rather than folded into it. Every corner is a
    /// coordinate the model holds and every test is an `orient3d` over four of
    /// them, so a polyhedron asked about an exact point is decided at the exact
    /// level every time and records nothing. Working out *where* the ray met the
    /// plane, which the general route has to, is a division, and what it hands
    /// the predicate after it is a box.
    ///
    /// It only answers about a polygon. [`FaceView::for_each_triangle`] fans a
    /// wire's corners, and a wire that is one closed curve has one corner and
    /// fans into nothing at all — a round cap would be invisible to every ray.
    /// A round *hole* is worse than invisible: its one corner fans into a
    /// triangle with no area, so the outer polygon's own fan covers the hole and
    /// the ray is counted through material that is not there.
    ///
    /// **The corners are checked against the stored plane wherever the ray's
    /// count depends on them.** Whether a triangle the ray's line passes
    /// through is met ahead of the point is asked of the triangle, and asked
    /// again of the face's stored plane: the ray from the point to its target
    /// crosses that plane when the two are on opposite sides of it. Where both
    /// answers are decided and differ, or only the plane's is decided, the
    /// count is not trusted and the face is named instead
    /// ([`SolidView::what_its_plane_says_beside_it`]).
    fn count_a_polygon(
        self,
        face: FaceView<'_>,
        tally: &mut Crossings,
        point: PredicatePoint,
        target: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) {
        let flat = face
            .surface()
            .as_plane()
            .copied()
            .expect("a polygon is counted only on a flat face");
        let stored_side = |side: Decision| match side {
            Decision::Pos => Some(true),
            Decision::Neg => Some(false),
            Decision::Zero | Decision::Undecidable { .. } | _ => None,
        };
        let crosses_the_plane_ahead = stored_side(side_of_a_flat_face(&flat, point))
            .zip(stored_side(
                flat.side_of_point(PredicatePoint::from(target.coordinates())),
            ))
            .map(|(from, to)| from != to);
        let mut on_the_plane = false;
        face.for_each_triangle(|corners| {
            match look_at_triangle(corners, point, target, crosses_the_plane_ahead, policy, log) {
                Hit::Missed => {}
                Hit::WentThrough { leaving } => tally.winding += leaving,
                Hit::StartsOnThePlane => on_the_plane = true,
                Hit::Grazed { enclosure } => tally.grazed(enclosure),
                Hit::ReadingsPart => tally.parted(face.key()),
            }
        });
        if on_the_plane {
            self.ask_whether_the_face_holds_it(face, tally, point, policy, log);
        }
    }

    /// What the ray does to one face, by where it meets that face's surface.
    ///
    /// Every curved face goes this way, and so does a flat one with a curved
    /// side — a bar's cap is a plane and a circle, and it has no polygon for
    /// [`SolidView::count_a_polygon`] to fan.
    ///
    /// **The surface first and the trimming second**, and the split is what
    /// keeps this out of `(u, v)`. Where a straight ray meets a plane, a ball or
    /// a pipe is one equation in one unknown of degree at most two — §31.2's row
    /// 11 — and its roots are the only thing in the whole of classification that
    /// is built rather than read, so they are built as boxes and handed on as
    /// boxes. Whether a root is on the *face* is then
    /// [`SolidView::face_holds_the_point`], which is signs and nothing else.
    ///
    /// Which way the ray crosses is row 12, `v · ∇S`, taken at the root. A face
    /// used reversed turns it round, exactly as a triangle's does.
    fn count_where_the_ray_meets_it(
        self,
        face: FaceView<'_>,
        tally: &mut Crossings,
        point: PredicatePoint,
        target: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) {
        let surface = face.surface();
        let from = point.enclosure();
        let along = gap_enclosing(from, target);

        match side_of_surface(&surface, point) {
            // The point is on this face's surface, so the ray starts there and
            // crosses nothing. Whether it is on the *face* is the outline's
            // question, the same as a flat face's.
            Decision::Zero => {
                self.ask_whether_the_face_holds_it(face, tally, point, policy, log);
            }
            Decision::Pos | Decision::Neg => (),
            Decision::Undecidable { enclosure } => {
                let held = self.what_it_holds_of_the_box(face, point, policy, log);
                tally.grazed_unless_it_holds_none(held, face.key(), enclosure);
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }

        // `+1` where the surface's own gradient points out of the material and
        // `-1` where the face is used the other way round, which is the same
        // turn a triangle of a reversed face gets.
        let outward: i64 = if face.orientation().is_reversed() {
            -1
        } else {
            1
        };
        let steps = match where_a_line_meets_a_surface(from, along, &surface, policy, log) {
            Ok(WhereALineMeetsIt::Nowhere) => return,
            // The ray's line lies in the surface itself, so nothing here counts
            // a crossing rather than a run alongside it — a graze, the same way
            // a plane's own near-parallel ray already was.
            Ok(WhereALineMeetsIt::MoreThanAPoint) => {
                tally.grazed(0.0);
                return;
            }
            Ok(WhereALineMeetsIt::At(steps)) => steps,
            // `WhereALineMeetsIt` grows a variant at every milestone; whatever
            // the new one says, nothing this ray found can be trusted with it.
            Ok(_) => {
                tally.grazed(0.0);
                return;
            }
            // A torus needs its line exact (`Error::WhereALineMeetsARingNeedsTheLineExactly`)
            // and refuses everywhere else too, which is what
            // `Containment::CannotFireARayAtAFacesSurface` already means: no ray
            // was ever put to this face's surface. A corner sitting exactly on
            // the lattice gives an exact line, reaches `Ok` above, and is
            // answered rather than refused. The other faces are still counted:
            // one that holds the point puts it on the skin, and one that grazes
            // the ray throws it away, whatever this face would have added
            // (Q1716, Q1739). So does this face's own graze above, if it made
            // one: every graze in its tally is from before this firing, which
            // could not have undone it (Q1763).
            Err(_) if matches!(surface, Surface::Torus(_)) => {
                tally.unfired(face.key());
                return;
            }
            // A plane, a ball, a pipe or a cone that could not tell is what
            // `Meeting::CouldNotTell` already meant: nothing this ray says can be
            // trusted, so it is grazed and the next of three targets gets a go.
            Err(_) => {
                tally.grazed(0.0);
                return;
            }
        };
        for step in steps {
            let meeting = match step {
                MeetsAt::Here(t) => where_it_lands(t, from, along),
                // A touch no `f64` names is not a meeting and not a miss — it is
                // grazed the same as a discriminant that straddled zero.
                // `MeetsAt` grows a variant at every milestone too, and an
                // unknown one gets the same treatment.
                MeetsAt::MaybeATouch { .. } | _ => Meeting::CouldNotTell,
            };
            let at = match meeting {
                Meeting::Ahead { at } => at,
                Meeting::Behind => continue,
                Meeting::Straddles { at } => {
                    let held = self.what_it_holds_of_the_box(
                        face,
                        PredicatePoint::Constructed(at),
                        policy,
                        log,
                    );
                    tally.grazed_unless_it_holds_none(held, face.key(), 0.0);
                    continue;
                }
                Meeting::CouldNotTell => {
                    tally.grazed(0.0);
                    continue;
                }
            };
            let x = PredicatePoint::Constructed(at);
            match self.what_this_face_holds(face, x, policy, log) {
                Ok(OnAFace::Held) => (),
                Ok(OnAFace::NotHeld) => continue,
                Ok(OnAFace::OnItsBoundary) => {
                    tally.grazed(0.0);
                    continue;
                }
                Err(Error::Undecidable { mut records }) => {
                    records.drain_into(log);
                    tally.grazed(0.0);
                    continue;
                }
                // Nothing this face says about the ray can be counted, and the
                // other faces are still asked: one that holds the point puts it
                // on the skin whatever this face would have added (Q1722).
                Err(_) => {
                    tally.unfollowed(face.key());
                    return;
                }
            }
            match dot_enclosing(along, gradient_at(&surface, at)).sign() {
                Decision::Pos => tally.winding += outward,
                Decision::Neg => tally.winding -= outward,
                Decision::Zero | Decision::Undecidable { .. } | _ => tally.grazed(0.0),
            }
        }
    }

    /// Put what the face says about a point on its own surface into the tally.
    ///
    /// The point is on the surface either way this is reached — a triangle of a
    /// polygon reported it, or the surface's own predicate did — and what is
    /// left is the outline's question.
    fn ask_whether_the_face_holds_it(
        self,
        face: FaceView<'_>,
        tally: &mut Crossings,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) {
        let held = self.what_this_face_holds(face, point, policy, log);
        tally.take_in(held, face.key());
    }

    /// What `face` holds of `point`'s box. Only [`OnAFace::NotHeld`] says
    /// anything about the whole box: that it can neither put a point on the
    /// skin through `face` nor be where the ray crosses it, whichever side of
    /// the face's surface each of its points is on.
    ///
    /// **It rests on §323.3's contract**: [`OnAFace::NotHeld`] is a sign decided
    /// about every point of the box, which is what the ray already drops a root
    /// ahead of the point on. Asked where the surface's own sign cannot place a
    /// point, and where a root cannot be told from the point it started at. A
    /// point of an upper bar's wall, against a lower bar on the same pipe, is
    /// both: the pipe's sign straddles the point, and one root of every ray is a
    /// rounding either side of it. Neither is on the lower bar's wall, and each
    /// threw every ray away until Q1080. What the face was asked goes into `log`
    /// only when it holds none of the box.
    fn what_it_holds_of_the_box(
        self,
        face: FaceView<'_>,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> crate::Result<OnAFace> {
        let mut trial = ProvenanceLog::new();
        let held = self.what_this_face_holds(face, point, policy, &mut trial);
        if matches!(held, Ok(OnAFace::NotHeld)) {
            trial.drain_into(log);
        }
        held
    }
}

/// Whether the whole of `point`'s box sits off one side of the box from `low`
/// to `high`.
///
/// `classify_uncertain_point` decides `Outside` from this without firing a
/// ray, and it is allowed to because it is a proof rather than a shortcut
/// **when the box holds every point of every face**, and an enclosure is a
/// range the point is guaranteed to be somewhere in, so a range lying entirely
/// past one side of the box puts the point outside the solid with nothing left
/// over. No tolerance is involved and no sign of a rounded quantity is read.
///
/// [`SolidView::box_around_it`] alone is not that box: it holds a flat face's
/// stored corners, and a turned solid's stored planes, which are its faces,
/// meet a rounding past them (Q1157). So the answer is taken only where the
/// point is past that box opened out to where the planes meet as well
/// (`SolidView::box_opened_out_to_where_its_planes_meet`). A disc across a
/// pipe's or a cone's axis tipped a hair off a coordinate axis needs nothing
/// more here: `box_around_it` works out its reach without the cancellation
/// that once left it short (Q1332, Q1363).
///
/// A box that overlaps the solid's bounding box proves nothing either way, so
/// it falls through and the rays decide it. So does a coordinate with no
/// enclosure around it at all — an infinity or a `NaN` encloses to every real
/// number — and the rays then come back undecided, which is the honest answer
/// to a point that cannot be bounded.
fn is_outside_the_box(point: PredicatePoint, low: Point, high: Point) -> bool {
    let point = point.enclosure();
    let low = low.coordinates();
    let high = high.coordinates();
    (0..3).any(|axis| point[axis].hi() < low[axis] || point[axis].lo() > high[axis])
}

/// Whether some triangle of `face`'s fan and its stored plane `flat` do not
/// both put every point of `point`'s box on the same side.
///
/// **A side is read against a point well in front of the plane**, which every
/// triangle's own plane puts in front too: a turn leaves the corners a
/// rounding off the plane, so each triangle is tilted from it by about a
/// rounding, and a triangle of a hole or of a fan that doubles back is wound
/// the other way round, which reading against that point undoes. A triangle
/// with no area has no side and is passed over, and so is one no point in
/// front of the plane can be found for ([`a_point_in_front_of`]); the ray
/// still asks the plane wherever its line passes through either.
///
/// A point exactly on a triangle's plane is no parting, since every stored
/// corner is one and the outline is what places it (`look_at_triangle`). A box
/// that both the triangle and the plane straddle is left to the ray, which
/// grazes it there as it always has. Every other case short of both settling
/// the same side is a parting: the box holds a point the two readings place
/// differently, or may.
fn its_corners_and_plane_part_at(face: FaceView<'_>, flat: &Plane, point: PredicatePoint) -> bool {
    let by_the_plane = in_front_of_it(side_of_a_flat_face(flat, point), Decision::Pos);
    let mut parted = false;
    face.for_each_triangle(|corners| {
        let [a, b, c] = corners.map(|corner| PredicatePoint::from(corner.coordinates()));
        if parted {
            return;
        }
        let Some(ahead) = a_point_in_front_of(flat, corners) else {
            return;
        };
        let front = orient3d_of_points(a, b, c, ahead);
        if front == Decision::Zero {
            return;
        }
        let by_the_triangle = in_front_of_it(orient3d_of_points(a, b, c, point), front);
        parted = match (by_the_triangle, by_the_plane) {
            (Reading::On, _) | (Reading::Unsettled, Reading::Unsettled) => false,
            (Reading::Side(one), Reading::Side(other)) => one != other,
            (Reading::Unsettled, Reading::Side(_) | Reading::On)
            | (Reading::Side(_), Reading::On | Reading::Unsettled) => true,
        };
    });
    parted
}

/// What one plane says of a point, against the sign it gives a point in
/// front of it.
#[derive(Clone, Copy)]
enum Reading {
    /// Off the plane: in front of it, or behind.
    Side(bool),
    /// Exactly on it.
    On,
    /// A box the plane straddles.
    Unsettled,
}

fn in_front_of_it(side: Decision, front: Decision) -> Reading {
    match side {
        Decision::Pos | Decision::Neg => Reading::Side(side == front),
        Decision::Zero => Reading::On,
        Decision::Undecidable { .. } => Reading::Unsettled,
        other => panic!("a side came back with a decision nothing here knows: {other:?}"),
    }
}

/// A point a step along `flat`'s normal from the first of `corners`, where the
/// plane puts it in front, or `None`.
///
/// The step is a thousandth of the largest coordinate of the three corners or
/// of the plane's offset, far more than the rounding a corner sits off its
/// plane and far less than the top of the range, which a step as long as the
/// coordinates overflows past at `1e308`. A triangle whose corners are all the
/// origin gets no step, and has no area to read a side by either.
fn a_point_in_front_of(flat: &Plane, corners: [Point; 3]) -> Option<PredicatePoint> {
    let normal = flat.normal().components();
    let reach = corners
        .iter()
        .flat_map(|corner| corner.coordinates())
        .fold(flat.offset().abs(), |far, part| far.max(part.abs()))
        / 1024.0;
    let at = corners[0].coordinates();
    let ahead = PredicatePoint::from(core::array::from_fn::<f64, 3, _>(|axis| {
        at[axis] + normal[axis] * reach
    }));
    (flat.side_of_point(ahead) == Decision::Pos).then_some(ahead)
}

/// Three far-off points to aim at, in turn.
///
/// Each is well clear of the solid and none is square-on to a coordinate axis,
/// because an axis-aligned ray runs straight along the edges of an axis-aligned
/// box and grazes everything. The multipliers are arbitrary; any fixed set of
/// three that point in different directions does the same job.
///
/// Nothing here looks at the point being classified, which is what keeps aiming
/// and deciding apart (decisions.md §15.5). A point known only as a box has no
/// coordinates to aim from, and none are invented for it: the targets come from
/// the solid alone, and the box itself is what every predicate is handed.
fn ray_targets(low: Point, high: Point) -> [Point; 3] {
    let low = low.coordinates();
    let high = high.coordinates();
    let span = (0..3)
        .map(|axis| high[axis] - low[axis])
        .fold(0.0, f64::max);
    let reach = if span > 0.0 { span } else { 1.0 };

    [
        Point::new(
            high[0] + reach,
            high[1] + reach * 0.618_033_988_749_895,
            high[2] + reach * 0.414_213_562_373_095_1,
        ),
        Point::new(
            low[0] - reach * 0.673_612_029_183_215_6,
            high[1] + reach * 1.324_717_957_244_746,
            low[2] - reach * 0.309_016_994_374_947_45,
        ),
        Point::new(
            high[0] + reach * 0.292_893_218_813_452_54,
            low[1] - reach * 1.732_050_807_568_877_2,
            high[2] + reach * 0.809_016_994_374_947_5,
        ),
    ]
}

/// A point with exact coordinates, next to the middle of `point`'s box, whose
/// step to `target` is exact too — or `None` when the one this builds is not.
///
/// **The step is rounded first and the point is read back off it**: `d` is the
/// step from the box's middle `m` to the target `t`, rounded, and the point is
/// `t − d`. Where `|t| ≥ |m|` that subtraction is exact (Dekker's), so the step
/// from the point to the target is `d` itself, and the point is within half an
/// ulp of `d` of the middle. The step is checked rather than trusted, because a
/// middle further out than the target need not give one.
fn a_point_an_exact_ray_leaves_for(point: PredicatePoint, target: Point) -> Option<PredicatePoint> {
    let enclosure = point.enclosure();
    let target = target.coordinates();
    let mut beside = [0.0; 3];
    for axis in 0..3 {
        let span = enclosure[axis];
        let middle = span.lo() + (span.hi() - span.lo()) / 2.0;
        let there = target[axis] - (target[axis] - middle);
        let step = Interval::exactly(target[axis]) - Interval::exactly(there);
        if !there.is_finite() || step.lo() != step.hi() {
            return None;
        }
        beside[axis] = there;
    }
    Some(PredicatePoint::from(beside))
}

/// The smallest box holding both `point`'s box and `beside`.
///
/// Both have finite ends by the time this is asked: a box with an infinite end
/// has no finite middle, and [`a_point_an_exact_ray_leaves_for`] turns it away.
fn the_box_round_both(point: PredicatePoint, beside: PredicatePoint) -> PredicatePoint {
    let [one, other] = [point.enclosure(), beside.enclosure()];
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        Interval::new(
            one[axis].lo().min(other[axis].lo()),
            one[axis].hi().max(other[axis].hi()),
        )
        .expect("two boxes with finite ends span a box")
    }))
}

/// `point`'s box stretched along one axis until it holds a point of `flat`
/// too, or `None` when no stretch could be proved to reach the plane. A point
/// the plane holds comes back as the box it is, and a box of no width as the
/// `f64` triple in it, as a stretched box does.
///
/// The axis is the one the plane's normal has most of, and the box keeps its
/// other two sides. Each of its two ends across that axis is moved out until
/// the plane's own sign puts that end's face on its own side of the plane, or
/// on it. Every line along the axis through the box then crosses the plane
/// between the two faces, so the box holds a point of the plane as well as
/// every point it held. The first try past the point is where the plane
/// crosses the axis through the middle of the box, worked out in `f64`;
/// nothing about it is trusted, and a try the sign does not prove is passed
/// over. `fuse::select` stretches a flat piece's candidate with this too,
/// before asking the piece's curved partners about it (§346.6, Q671).
pub(crate) fn stretched_to_the_plane(
    flat: &Plane,
    point: PredicatePoint,
) -> Option<PredicatePoint> {
    if side_of_a_flat_face(flat, point) == Decision::Zero {
        return Some(as_exact_as_it_is(point.enclosure()));
    }
    let normal = flat.normal().components();
    let along = (0..3)
        .max_by(|&one, &other| normal[one].abs().total_cmp(&normal[other].abs()))
        .expect("three axes");
    let mut boxed = point.enclosure();
    let middle = boxed.map(|span| span.lo() + (span.hi() - span.lo()) / 2.0);
    let crossing = (0..3)
        .filter(|&axis| axis != along)
        .fold(flat.offset(), |rest, axis| {
            rest - normal[axis] * middle[axis]
        })
        / normal[along];
    let low = an_end_on_its_side(flat, boxed, along, boxed[along].lo(), crossing, -1.0)?;
    let high = an_end_on_its_side(flat, boxed, along, boxed[along].hi(), crossing, 1.0)?;
    boxed[along] = Interval::new(low, high).expect("each end only ever moves away from the other");
    Some(as_exact_as_it_is(boxed))
}

/// Where along `along` the face of `boxed` on the `way` end is on that end's
/// side of `flat`, or on it: `own_end` when its face already is, and otherwise
/// the first of a run of tries from `crossing`, or from `own_end` when the
/// crossing is not past it, each step twice the one before. `way` is `-1.0` for
/// the low end and `1.0` for the high one. `None` when no try is proved.
fn an_end_on_its_side(
    flat: &Plane,
    boxed: [Interval; 3],
    along: usize,
    own_end: f64,
    crossing: f64,
    way: f64,
) -> Option<f64> {
    let ends_side = if (flat.normal().components()[along] > 0.0) == (way > 0.0) {
        Decision::Pos
    } else {
        Decision::Neg
    };
    let is_on_its_side = |end: f64| {
        let mut face = boxed;
        face[along] = Interval::exactly(end);
        let side = side_of_a_flat_face(flat, PredicatePoint::Constructed(face));
        side == ends_side || side == Decision::Zero
    };
    if is_on_its_side(own_end) {
        return Some(own_end);
    }
    let size = boxed.iter().fold(flat.offset().abs(), |largest, side| {
        largest.max(side.lo().abs()).max(side.hi().abs())
    });
    let mut step = size.max(f64::MIN_POSITIVE) * f64::EPSILON;
    let mut end = if (crossing - own_end) * way > 0.0 {
        crossing
    } else {
        own_end + way * step
    };
    for _ in 0..64 {
        if !end.is_finite() {
            return None;
        }
        if is_on_its_side(end) {
            return Some(end);
        }
        end += way * step;
        step *= 2.0;
    }
    None
}

/// Which side of `flat` a point is on, asked with each coordinate the plane
/// cannot see narrowed to one end of its box.
///
/// **A plane whose normal has no part along an axis cannot tell apart two
/// points that differ only along it**, so narrowing that coordinate changes
/// nothing about the answer and everything about whether it can be given: a
/// box that is one double in the other two coordinates becomes an exact point,
/// and the plane's own predicate settles it exactly. A segment up through the
/// cap a turned block's face takes off a ball is such a box, a fraction of a
/// spacing from the plane, and asked as a box its sign is never settled
/// (`docs/decisions.md` §346.6, item 4). `fuse::select` builds those segments
/// and asks a flat piece's plane about its candidates through this.
pub(crate) fn side_of_a_flat_face(flat: &Plane, point: PredicatePoint) -> Decision {
    let PredicatePoint::Constructed(mut boxed) = point else {
        return flat.side_of_point(point);
    };
    for (axis, part) in flat.normal().components().into_iter().enumerate() {
        if part == 0.0 {
            boxed[axis] = Interval::exactly(boxed[axis].lo());
        }
    }
    flat.side_of_point(as_exact_as_it_is(boxed))
}

/// A box of no width is the `f64` triple in it, which every predicate can then
/// settle exactly; any other box is handed on as the box it is.
fn as_exact_as_it_is(boxed: [Interval; 3]) -> PredicatePoint {
    if boxed.iter().all(|side| side.lo() == side.hi()) {
        return PredicatePoint::from(boxed.map(|side| side.lo()));
    }
    PredicatePoint::Constructed(boxed)
}

/// Whether no point of `point`'s box is on the triangle `corners`.
///
/// **Off the triangle's plane is not the only way to miss it.** A box beside a
/// block's side, on the plane through that side and nowhere near the side
/// itself, is on the plane of both its triangles, and asking only the plane
/// refused it (`docs/decisions.md` §323.3). So the box and the triangle are
/// first held up against each coordinate axis in turn: where the triangle's
/// corners all lie below the box's lower end, or all above its upper end, the
/// two are apart. That compares numbers the model holds with the box's own
/// ends, nothing is rounded, and it is a sign decided about the whole box —
/// the contract §323.3 rests on. It goes first because it is exact and cheap,
/// and a box it settles leaves nothing in `log`.
///
/// **A box inside a triangle's own bounding box can still be off it**, where
/// the triangle's sides are not lined up with the axes: beside the top of a
/// block turned about the upright, in a corner of the square round it. So the
/// two are held up next against the nine directions square to a side of the
/// triangle and to a coordinate axis ([`apart_across_a_side`]). With the three
/// axes and the plane's own normal those are the thirteen directions along one
/// of which a box and a triangle that do not touch are always apart. The nine
/// here are rounded and their projections widened, so a box still refused
/// touches the triangle or is within a rounding of touching it.
///
/// A box a triangle touches can still miss the face, where the face is not
/// convex and its fan reaches outside it; that is the whole outline's
/// question, and [`SolidView::its_fan_adds_up_to_nothing_over`] asks it.
fn misses_the_triangle(
    [a, b, c]: [Point; 3],
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> bool {
    let reach = point.enclosure();
    let corners = [a, b, c].map(|corner| corner.coordinates());
    let apart_along = |axis: usize| {
        corners.iter().all(|corner| corner[axis] < reach[axis].lo())
            || corners.iter().all(|corner| corner[axis] > reach[axis].hi())
    };
    (0..3).any(apart_along)
        || apart_across_a_side(corners, reach)
        || matches!(
            decide(a, b, c, point, policy, log),
            Decision::Pos | Decision::Neg
        )
}

/// Whether, along a direction square to a side of the triangle `corners` and
/// to one coordinate axis, every corner is below every point of the box
/// `reach`, or every corner above.
///
/// **The direction does not have to be the exact one, and is not.** Any vector
/// `v` at all keeps the two apart when every corner's `v · x` is below the
/// least `v · x` of the box: the triangle is the corners' convex hull, so each
/// of its points is below that too, and a point in both would have to be
/// below itself. So the side is a rounded `f64` difference and each direction
/// is that side crossed with an axis, which is a swap and a sign; only the
/// two projections have to hold for every point, and they are worked out in
/// [`Interval`] arithmetic, which rounds every step outward. A direction whose
/// projections overlap decides nothing and the next is asked; one with a part
/// that is not finite is not a direction and is passed over.
fn apart_across_a_side(corners: [[f64; 3]; 3], reach: [Interval; 3]) -> bool {
    (0..3).any(|side| {
        let [from, to] = [corners[side], corners[(side + 1) % 3]];
        let [x, y, z]: [f64; 3] = core::array::from_fn(|axis| to[axis] - from[axis]);
        [[0.0, z, -y], [-z, 0.0, x], [y, -x, 0.0]]
            .into_iter()
            .any(|across| apart_along_the_direction(across, corners, reach))
    })
}

/// Whether the corners of a triangle all project below the box `reach` along
/// `across`, or all above it, by projections that hold for every point.
fn apart_along_the_direction(
    across: [f64; 3],
    corners: [[f64; 3]; 3],
    reach: [Interval; 3],
) -> bool {
    if !across.iter().all(|part| part.is_finite()) {
        return false;
    }
    let across = exactly(across);
    let of_the_box = dot_enclosing(across, reach);
    let of_the_corners = corners.map(|corner| dot_enclosing(across, exactly(corner)));
    of_the_corners
        .iter()
        .all(|shadow| shadow.hi() < of_the_box.lo())
        || of_the_corners
            .iter()
            .all(|shadow| shadow.lo() > of_the_box.hi())
}

/// What one triangle had to say about the ray.
enum Hit {
    /// The ray does not pass through this triangle.
    Missed,
    /// The ray passes through it. `+1` on the way out of the material, `-1` on
    /// the way in.
    WentThrough { leaving: i64 },
    /// The point being classified lies on this triangle's plane, so the ray
    /// meets that plane only where it starts and crosses nothing here. Whether
    /// the point is on the *face* is a question about the whole outline, and
    /// [`SolidView::face_holds_the_point`] is what answers it.
    StartsOnThePlane,
    /// The ray met an edge or a corner of the triangle, or a predicate could
    /// not decide. Nothing this ray says can be trusted.
    Grazed { enclosure: f64 },
    /// The triangle and the face's stored plane disagree about whether the ray
    /// meets the face ahead of the point, or only the plane can say. The point
    /// is in the sliver between the corners and the plane, and no ray counted
    /// by the corners can be trusted about it.
    ReadingsPart,
}

/// Running totals for one ray.
#[derive(Default)]
struct Crossings {
    winding: i64,
    on_boundary: bool,
    grazed_something: bool,
    /// Whether a face whose trimming was followed wherever the ray asked it
    /// grazed the ray: one the ray was fired at, or a ring it could not be
    /// fired at grazing from its own sign before the firing.
    a_graze_that_stands: bool,
    worst_enclosure: f64,
    /// The first flat face whose corners and stored plane read the point
    /// differently ([`Hit::ReadingsPart`]).
    parted_at: Option<FaceKey>,
    /// The first face whose surface the ray could not be fired at, so that
    /// its crossings are not in `winding`.
    unfired_at: Option<FaceKey>,
    /// The first face whose trimming nothing could follow where the ray asked
    /// it, so that no count of this ray is to be trusted, and no other ray
    /// gets past it unless a graze that stands threw it away.
    unfollowed_at: Option<FaceKey>,
}

impl Crossings {
    /// Add in one face's own tally.
    ///
    /// **Only a graze that stands throws the ray away whatever the answers
    /// held back would add.** A face whose trimming nothing follows grazes a
    /// ray where it cannot show it holds none of the box the ray starts from
    /// or meets it in, and that is for want of the answer it refused;
    /// following its trimming might have put the box off it. Every other
    /// graze stands. A ring the ray could not be fired at grazes it only
    /// before the firing, from its own sign and what its face holds of the
    /// point, since [`SolidView::count_where_the_ray_meets_it`] stops at the
    /// refusal; a ray fired at the ring could not undo that graze, so it
    /// throws the ray away as a ball's own sign would, and a box on the ring's
    /// wall is [`Containment::Undecidable`], as one on a ball's wall is
    /// (`docs/decisions.md` §323.3, Q1763). Until Q1763 it was left out, and
    /// such a box was refused.
    fn add(&mut self, face: Self) {
        let Self {
            winding,
            on_boundary,
            grazed_something,
            a_graze_that_stands: _,
            worst_enclosure,
            parted_at,
            unfired_at,
            unfollowed_at,
        } = face;
        let its_trimming_was_followed = unfollowed_at.is_none();
        self.winding += winding;
        self.on_boundary |= on_boundary;
        self.grazed_something |= grazed_something;
        self.a_graze_that_stands |= grazed_something && its_trimming_was_followed;
        self.worst_enclosure = self.worst_enclosure.max(worst_enclosure);
        self.parted_at = self.parted_at.or(parted_at);
        self.unfired_at = self.unfired_at.or(unfired_at);
        self.unfollowed_at = self.unfollowed_at.or(unfollowed_at);
    }

    /// Nothing this ray says can be trusted any more.
    fn grazed(&mut self, enclosure: f64) {
        self.grazed_something = true;
        self.worst_enclosure = self.worst_enclosure.max(enclosure);
    }

    /// The ray met `face` where its corners and its stored plane part.
    fn parted(&mut self, face: FaceKey) {
        self.parted_at.get_or_insert(face);
    }

    /// The ray could not be fired at `face`'s surface.
    fn unfired(&mut self, face: FaceKey) {
        self.unfired_at.get_or_insert(face);
    }

    /// Whether this ray is thrown away whatever the faces it could not be
    /// fired at, or whose trimming nothing could follow, would add to it: a
    /// graze that stands threw it away (Q1716, Q1727). That holds where the
    /// box straddles the sign of the ring left unfired, too (Q1739), and where
    /// that ring's own graze is the one (Q1763). A ray fired at the ring would
    /// add crossings, grazes or a trimming nothing follows, and a ray a graze
    /// that stands threw away is thrown away whichever it added.
    fn is_thrown_away_without_them(&self) -> bool {
        self.a_graze_that_stands && self.parted_at.is_none()
    }

    /// Take in what a face said about a point on its own surface.
    ///
    /// On the face or on its boundary both put the point on the solid's skin,
    /// because a face's boundary is part of the face. A sign nobody could settle
    /// is a graze and the next ray gets a go; a face whose trimming nothing can
    /// follow is noted, and another ray gets past it only where a graze that
    /// stands threw this one away.
    fn take_in(&mut self, held: crate::Result<OnAFace>, face: FaceKey) {
        match held {
            Ok(OnAFace::Held | OnAFace::OnItsBoundary) => self.on_boundary = true,
            Ok(OnAFace::NotHeld) => (),
            Err(Error::Undecidable { .. }) => self.grazed(0.0),
            Err(_) => self.unfollowed(face),
        }
    }

    /// Take in what a face holds of a box its own sign could not place, or of
    /// a root that could not be told from the point the ray started at.
    ///
    /// Only none of the box lets the ray by, and anything else grazes it. **A
    /// face whose trimming nothing follows is noted as well**: its graze is
    /// for want of the answer it refused, and [`Crossings::add`] keeps such a
    /// graze from throwing the ray away. Taken as a graze that stands, it
    /// threw away a ray no other face grazed, and a box on the ball under the
    /// floor of a half ball whose cap nothing follows came back
    /// [`Containment::Undecidable`], where the half ball alone puts it outside
    /// (Q1740).
    fn grazed_unless_it_holds_none(
        &mut self,
        held: crate::Result<OnAFace>,
        face: FaceKey,
        enclosure: f64,
    ) {
        match held {
            Ok(OnAFace::NotHeld) => (),
            Ok(OnAFace::Held | OnAFace::OnItsBoundary) | Err(Error::Undecidable { .. }) => {
                self.grazed(enclosure);
            }
            Err(_) => {
                self.grazed(enclosure);
                self.unfollowed(face);
            }
        }
    }

    /// Nothing can follow `face`'s trimming where the ray asked it.
    fn unfollowed(&mut self, face: FaceKey) {
        self.unfollowed_at.get_or_insert(face);
    }
}

/// Work out what the ray from `point` towards `target` does to one triangle.
///
/// Three tests say whether the ray's line passes inside the triangle: for each
/// edge, the signed volume of the box on that edge and the ray direction. They
/// have to agree in sign. A fourth test — which side of the triangle's plane
/// the point is on — then says whether the meeting is ahead of the point or
/// behind it.
///
/// `crosses_the_plane_ahead` is the same question put to the face's stored
/// plane, or `None` when the plane cannot say. Where the plane says and the
/// triangle's certified sign says otherwise, or cannot say, the answer is
/// [`Hit::ReadingsPart`], before any policy is asked to settle the triangle's
/// sign. A point exactly on the triangle's plane is still asked of the
/// outline: every stored corner is one, and the outline is what says it is
/// on the skin.
fn look_at_triangle(
    [a, b, c]: [Point; 3],
    point: PredicatePoint,
    target: Point,
    crosses_the_plane_ahead: Option<bool>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Hit {
    let mut positives = 0;
    let mut negatives = 0;
    let mut zeros = 0;
    for [from, to] in [[a, b], [b, c], [c, a]] {
        match decide(from, to, target, point, policy, log) {
            Decision::Pos => positives += 1,
            Decision::Neg => negatives += 1,
            Decision::Zero => zeros += 1,
            Decision::Undecidable { enclosure } => return Hit::Grazed { enclosure },
            // `Decision` grows a variant at every milestone. Whatever the new
            // one means, guessing is worse than stopping.
            other => panic!("orient3d came back with a decision nothing here knows: {other:?}"),
        }
    }

    if positives > 0 && negatives > 0 {
        return Hit::Missed;
    }
    if positives == 0 && negatives == 0 {
        return Hit::Grazed { enclosure: 0.0 };
    }
    let leaving = if positives > 0 { 1 } else { -1 };

    let by_the_corners = orient3d_of_points(
        PredicatePoint::from(a.coordinates()),
        PredicatePoint::from(b.coordinates()),
        PredicatePoint::from(c.coordinates()),
        point,
    );
    if let Some(ahead_by_the_plane) = crosses_the_plane_ahead {
        let parted = match by_the_corners {
            Decision::Pos | Decision::Neg => {
                ((by_the_corners == Decision::Pos) == (leaving == 1)) != ahead_by_the_plane
            }
            Decision::Undecidable { .. } => true,
            Decision::Zero => false,
            other => panic!("orient3d came back with a decision nothing here knows: {other:?}"),
        };
        if parted {
            return Hit::ReadingsPart;
        }
    }
    let side_of_the_plane = if matches!(by_the_corners, Decision::Undecidable { .. }) {
        decide(a, b, c, point, policy, log)
    } else {
        by_the_corners
    };
    match side_of_the_plane {
        Decision::Zero => Hit::StartsOnThePlane,
        Decision::Pos | Decision::Neg => {
            if zeros > 0 {
                Hit::Grazed { enclosure: 0.0 }
            } else if (side_of_the_plane == Decision::Pos) == (leaving == 1) {
                Hit::WentThrough { leaving }
            } else {
                Hit::Missed
            }
        }
        Decision::Undecidable { enclosure } => Hit::Grazed { enclosure },
        other => panic!("orient3d came back with a decision nothing here knows: {other:?}"),
    }
}

/// Whether every side of `face` is a straight line, so that its corners fan into
/// triangles that cover it.
pub(crate) fn every_side_of_it_is_straight(face: FaceView<'_>) -> bool {
    face.wires().all(|wire| {
        wire.coedges()
            .all(|coedge| coedge.edge().stored_run() == &EdgeRun::Straight)
    })
}

/// Where a ray meets one surface, at one of the steps
/// [`rcad_geom::where_a_line_meets_a_surface`] found along the line — turned
/// from a step measured along the whole line into a fact about the *ray*, which
/// only looks ahead of where it started.
enum Meeting {
    /// Ahead of the point being classified, somewhere in this box.
    Ahead {
        /// Where it landed.
        at: [Interval; 3],
    },
    /// At the point itself or behind it, so the ray crosses nothing here.
    Behind,
    /// The root's box straddles the point: somewhere in this box, and nobody
    /// can say whether ahead of the point or behind it.
    Straddles {
        /// Where it landed.
        at: [Interval; 3],
    },
    /// A touch no `f64` names, or a step of a kind nothing here knows. Nothing
    /// this ray says can be trusted.
    CouldNotTell,
}

/// The step from `from` to `there`, as a box.
fn gap_enclosing(from: [Interval; 3], there: Point) -> [Interval; 3] {
    let there = there.coordinates();
    core::array::from_fn(|axis| Interval::exactly(there[axis]) - from[axis])
}

/// What a root at `t` along the ray comes to.
///
/// A root at exactly zero is where the ray *starts*, which is the point sitting
/// on this face's own surface: the ray crosses nothing there, and whether the
/// point is on the face is asked separately. A root whose box merely covers zero
/// is a different thing — nobody can say whether the crossing is ahead — and it
/// is a graze unless the face holds none of the box it lands in.
fn where_it_lands(t: Interval, from: [Interval; 3], along: [Interval; 3]) -> Meeting {
    if t.hi() < 0.0 || (t.lo() == 0.0 && t.hi() == 0.0) {
        return Meeting::Behind;
    }
    let at = core::array::from_fn(|axis| from[axis] + t * along[axis]);
    if t.lo() <= 0.0 {
        return Meeting::Straddles { at };
    }
    Meeting::Ahead { at }
}

/// Which way the surface faces outwards at `at`, as a box.
///
/// A polynomial in the stored numbers with no normalization anywhere in it, so
/// the sign of anything dotted with it is the sign of the real thing (§31.2's
/// row 12).
///
/// # Panics
///
/// On a surface [`Surface`] has not grown yet.
fn gradient_at(surface: &Surface, at: [Interval; 3]) -> [Interval; 3] {
    match surface {
        Surface::Plane(flat) => exactly(flat.normal().components()),
        // **Out of the middle and not towards it.** `gap_enclosing(a, b)` is the
        // step *from* `a` *to* `b`, so the step from the centre to the point is
        // the one turned round — the same way round `rcad_geom::where_a_line_meets_a_surface`
        // reads it. Taken the other way this is the gradient of the
        // negated expression, and then a ray that enters a curved face counts as
        // leaving it. It cancels for a solid whose faces are *all* curved, which
        // is why a ball and a bar's own wall never showed it: what shows it is a
        // ray that crosses one curved face and one flat one, which is a ray
        // through a bar's wall and out of its cap (§36.13).
        Surface::Sphere(ball) => negated(gap_enclosing(at, ball.centre())),
        // The gradient of `|x × w|² − slope²(x·w)²`, halved, read out of the
        // middle rather than towards it for the reason the pipe's arm gives.
        Surface::Cone(cone) => {
            let direction = exactly(cone.axis().direction().components());
            let out = negated(gap_enclosing(at, cone.apex()));
            let up = dot_enclosing(out, direction);
            let squared = dot_enclosing(direction, direction);
            let slope = Interval::exactly(cone.slope());
            let steepness = Interval::exactly(1.0) + slope * slope;
            core::array::from_fn(|axis| squared * out[axis] - steepness * up * direction[axis])
        }
        Surface::Cylinder(pipe) => {
            let direction = exactly(pipe.axis().direction().components());
            let out = negated(gap_enclosing(at, pipe.axis().origin()));
            let up = dot_enclosing(out, direction);
            // `|d|²`, and it is **not** one. `Cylinder::side_of` decides
            // `|(x − o) × d|² − r²` about the direction *as stored*, whose
            // length is one only to within the rounding `Axis::new` left in it
            // (`Cylinder`'s own doc says so). The gradient of that expression is
            // `2(|d|²(x − o) − ((x − o)·d)d)`, and dropping the factor would
            // leave this the gradient of a slightly different expression from
            // the one the sign is read from — which is the whole thing this is
            // written to avoid.
            let squared = dot_enclosing(direction, direction);
            core::array::from_fn(|axis| squared * out[axis] - up * direction[axis])
        }
        // `(w·w)(x·x + R² − r² − 2R²)x + 2R²(x·w)w` for `x = at − centre`: the
        // quartic's own gradient, divided by four so nothing is normalized. It
        // points out of the tube, which is the way a torus faces — the
        // standard's, the reader's and `validate.rs`'s reading, and the
        // mesher's since it stopped taking the chart's turn for it
        // (`docs/decisions.md` §329, §361).
        Surface::Torus(ring) => {
            let direction = exactly(ring.axis().direction().components());
            let out = negated(gap_enclosing(at, ring.centre()));
            let up = dot_enclosing(out, direction);
            let squared = dot_enclosing(direction, direction);
            let big = Interval::exactly(ring.ring_radius());
            let small = Interval::exactly(ring.tube_radius());
            let bulge = dot_enclosing(out, out) - big * big - small * small;
            core::array::from_fn(|axis| {
                squared * bulge * out[axis]
                    + Interval::exactly(2.0) * big * big * up * direction[axis]
            })
        }
        other => panic!("this crate cannot say which way {other:?} faces"),
    }
}

/// The same box the other way round.
fn negated(gap: [Interval; 3]) -> [Interval; 3] {
    gap.map(core::ops::Neg::neg)
}

/// Three stored numbers as a box of no width at all.
fn exactly(value: [f64; 3]) -> [Interval; 3] {
    value.map(Interval::exactly)
}

/// `u · v` as a box.
fn dot_enclosing(u: [Interval; 3], v: [Interval; 3]) -> Interval {
    (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
        so_far + u[axis] * v[axis]
    })
}

/// Which side of the plane through `a`, `b` and `c` the point `d` is on.
///
/// `a`, `b` and `c` are always exact: a triangle corner read out of the model,
/// or a ray target worked out from the bounding box. `d` is the point being
/// classified and is the one that may be a box.
pub(crate) fn decide(
    a: Point,
    b: Point,
    c: Point,
    d: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    orient3d_tolerantly(
        PredicatePoint::from(a.coordinates()),
        PredicatePoint::from(b.coordinates()),
        PredicatePoint::from(c.coordinates()),
        d,
        policy,
        log,
    )
}
