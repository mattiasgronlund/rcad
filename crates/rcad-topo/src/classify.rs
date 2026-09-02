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

use rcad_geom::{Cone, Cylinder, EdgeRun, Point, Sphere, Surface};
use rcad_predicates::expansion::{product_is_exact, two_product};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, orient3d_tolerantly,
};

use crate::validate::side_of_surface;
use crate::{Error, FaceKey, FaceView, OnAFace, SolidView};

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
        /// tells us nothing.
        enclosure: f64,
    },
    /// One face of the solid is trimmed in a way
    /// [`SolidView::face_holds_the_point`] cannot follow, so no ray was ever
    /// counted.
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
    /// The ray's far target is picked from [`SolidView::box_around_it`] and is
    /// exact. `point` is the only thing the *caller* may hand over as a box, and
    /// it is handed to every predicate as it is — nothing here rounds it down to
    /// a representative coordinate first.
    ///
    /// If the ray meets an edge of a triangle exactly, or a root cannot be told
    /// from the point it started at, the count is not to be trusted, so the
    /// whole ray is thrown away and the next of three fixed targets is tried. If
    /// all three graze something, the answer is [`Containment::Undecidable`].
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
    /// Anything the cascade could not certify lands in `log`, and a fully
    /// certified classification leaves it untouched.
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
        if is_outside_the_box(point, low, high) {
            return Containment::Outside;
        }

        let mut worst_enclosure = 0.0f64;
        for target in ray_targets(low, high) {
            let pass = match self.count_crossings(point, target, policy, log) {
                Ok(pass) => pass,
                Err(face) => return Containment::CannotFollowAFacesTrimming { face },
            };
            if pass.on_boundary {
                return Containment::OnBoundary;
            }
            if !pass.grazed_something {
                return if pass.winding == 0 {
                    Containment::Outside
                } else {
                    Containment::Inside
                };
            }
            worst_enclosure = worst_enclosure.max(pass.worst_enclosure);
        }
        Containment::Undecidable {
            enclosure: worst_enclosure,
        }
    }

    /// One ray's tally, or which face's trimming stopped it being counted.
    ///
    /// A face nothing can follow is not something another ray gets past, so it
    /// comes straight back rather than being counted as a graze.
    fn count_crossings(
        self,
        point: PredicatePoint,
        target: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Crossings, FaceKey> {
        let mut tally = Crossings::default();
        for face in self.faces() {
            if face.surface().as_plane().is_some() && every_side_of_it_is_straight(face) {
                self.count_a_polygon(face, &mut tally, point, target, policy, log)?;
            } else {
                self.count_where_the_ray_meets_it(face, &mut tally, point, target, policy, log)?;
            }
        }
        Ok(tally)
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
    fn count_a_polygon(
        self,
        face: FaceView<'_>,
        tally: &mut Crossings,
        point: PredicatePoint,
        target: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<(), FaceKey> {
        let mut on_the_plane = false;
        face.for_each_triangle(|corners| {
            match look_at_triangle(corners, point, target, policy, log) {
                Hit::Missed => {}
                Hit::WentThrough { leaving } => tally.winding += leaving,
                Hit::StartsOnThePlane => on_the_plane = true,
                Hit::Grazed { enclosure } => tally.grazed(enclosure),
            }
        });
        if on_the_plane {
            self.ask_whether_the_face_holds_it(face, tally, point, policy, log)?;
        }
        Ok(())
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
    ) -> Result<(), FaceKey> {
        let surface = face.surface();
        let from = point.enclosure();
        let along = gap_enclosing(from, target);

        match side_of_surface(&surface, point) {
            // The point is on this face's surface, so the ray starts there and
            // crosses nothing. Whether it is on the *face* is the outline's
            // question, the same as a flat face's.
            Decision::Zero => {
                self.ask_whether_the_face_holds_it(face, tally, point, policy, log)?;
            }
            Decision::Pos | Decision::Neg => (),
            Decision::Undecidable { enclosure } => tally.grazed(enclosure),
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
        for step in where_the_ray_meets(&surface, from, along) {
            let at = match step {
                Meeting::Ahead { at } => at,
                Meeting::Behind => continue,
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
                Err(_) => return Err(face.key()),
            }
            match dot_enclosing(along, gradient_at(&surface, at)).sign() {
                Decision::Pos => tally.winding += outward,
                Decision::Neg => tally.winding -= outward,
                Decision::Zero | Decision::Undecidable { .. } | _ => tally.grazed(0.0),
            }
        }
        Ok(())
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
    ) -> Result<(), FaceKey> {
        let held = self.what_this_face_holds(face, point, policy, log);
        tally.take_in(held, face.key())
    }
}

/// Whether the whole of `point`'s box sits off one side of the solid's
/// bounding box.
///
/// This decides `Outside` without firing a ray, and it is allowed to because it
/// is a proof rather than a shortcut. Every point of every face is inside
/// [`SolidView::box_around_it`] — which is what that box is for and what
/// [`SolidView::box_around_its_corners`] is not — and an enclosure is a range
/// the point is guaranteed to be somewhere in, so a range lying entirely past
/// one side of the box puts the point outside the solid with nothing left over.
/// No tolerance is involved and no sign of a rounded quantity is read.
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
}

/// Running totals for one ray.
#[derive(Default)]
struct Crossings {
    winding: i64,
    on_boundary: bool,
    grazed_something: bool,
    worst_enclosure: f64,
}

impl Crossings {
    /// Nothing this ray says can be trusted any more.
    fn grazed(&mut self, enclosure: f64) {
        self.grazed_something = true;
        self.worst_enclosure = self.worst_enclosure.max(enclosure);
    }

    /// Take in what a face said about a point on its own surface.
    ///
    /// On the face or on its boundary both put the point on the solid's skin,
    /// because a face's boundary is part of the face. A sign nobody could settle
    /// is a graze and the next ray gets a go; a face whose trimming nothing can
    /// follow is not something another ray gets past, so it stops the walk.
    fn take_in(&mut self, held: crate::Result<OnAFace>, face: FaceKey) -> Result<(), FaceKey> {
        match held {
            Ok(OnAFace::Held | OnAFace::OnItsBoundary) => self.on_boundary = true,
            Ok(OnAFace::NotHeld) => (),
            Err(Error::Undecidable { .. }) => self.grazed(0.0),
            Err(_) => return Err(face),
        }
        Ok(())
    }
}

/// Work out what the ray from `point` towards `target` does to one triangle.
///
/// Three tests say whether the ray's line passes inside the triangle: for each
/// edge, the signed volume of the box on that edge and the ray direction. They
/// have to agree in sign. A fourth test — which side of the triangle's plane
/// the point is on — then says whether the meeting is ahead of the point or
/// behind it.
fn look_at_triangle(
    [a, b, c]: [Point; 3],
    point: PredicatePoint,
    target: Point,
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

    let side_of_the_plane = decide(a, b, c, point, policy, log);
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
fn every_side_of_it_is_straight(face: FaceView<'_>) -> bool {
    face.wires().all(|wire| {
        wire.coedges()
            .all(|coedge| coedge.edge().run() == &EdgeRun::Straight)
    })
}

/// Where a ray meets one surface, at one of the two roots.
///
/// A plane has one root and fills its second place with [`Meeting::Behind`],
/// which is what "there is nothing to count here" already means to every reader
/// of the loop.
enum Meeting {
    /// Ahead of the point being classified, somewhere in this box.
    Ahead {
        /// Where it landed.
        at: [Interval; 3],
    },
    /// At the point itself or behind it, so the ray crosses nothing here.
    Behind,
    /// The root's box straddles the point, or a coefficient of the quadratic
    /// did. Nothing this ray says can be trusted.
    CouldNotTell,
}

/// The step from `from` to `there`, as a box.
fn gap_enclosing(from: [Interval; 3], there: Point) -> [Interval; 3] {
    let there = there.coordinates();
    core::array::from_fn(|axis| Interval::exactly(there[axis]) - from[axis])
}

/// Where the ray from `from` along `along` meets `surface`.
///
/// **One equation in one unknown, and nearly the same one three times** —
/// §31.2's row 11. Put `x = from + t·along` into `normal · x − offset` for a
/// plane, into `|x − centre|² − radius²` for a ball, or into
/// `|(x − origin) × direction|² − radius²` for a pipe. The last two come out
/// `a·t² + b·t + k`, the pipe being the ball with the axis crossed out of every
/// vector first; the plane comes out with no square in it and so has one root
/// rather than two. All three expressions are the ones [`side_of_surface`]
/// already decides signs of, so the surface a root is worked out from and the
/// surface it is later tested against are the same one written the same way.
///
/// **A plane reaches here and it is not a mistake.** A flat face whose every
/// side is straight is counted by its triangles, but a bar's cap is a plane and
/// a circle and has no triangles to count (§35.4), so it comes this way.
///
/// The roots are the only thing in the whole of classification that is *built*
/// rather than read out of a model, and they are built as boxes and handed on as
/// boxes. A ray that grazes the surface leaves a discriminant that straddles
/// zero, and that is [`Meeting::CouldNotTell`] rather than a guess at a tangent.
///
/// # Panics
///
/// On a surface kind this has not been taught. [`Surface`] grows a variant at
/// every milestone.
fn where_the_ray_meets(
    surface: &Surface,
    from: [Interval; 3],
    along: [Interval; 3],
) -> [Meeting; 2] {
    if let Surface::Plane(flat) = surface {
        let normal = exactly(flat.normal().components());
        let steepness = dot_enclosing(along, normal);
        if steepness.contains_zero() {
            return [Meeting::CouldNotTell, Meeting::Behind];
        }
        let height = dot_enclosing(from, normal) - Interval::exactly(flat.offset());
        return [
            where_it_lands(quotient(-height, steepness), from, along),
            Meeting::Behind,
        ];
    }
    let (a, b, k) = match surface {
        Surface::Sphere(ball) => how_a_ray_meets_a_ball(ball, from, along),
        Surface::Cylinder(pipe) => how_a_ray_meets_a_pipe(pipe, from, along),
        Surface::Cone(cone) => how_a_ray_meets_a_cone(cone, from, along),
        other => panic!("this crate cannot fire a ray at {other:?}"),
    };

    let twice_a = Interval::exactly(2.0) * a;
    if twice_a.contains_zero() {
        return [Meeting::CouldNotTell, Meeting::CouldNotTell];
    }
    // **A point already on the surface is a root, and the quadratic formula
    // loses it.** With `k` exactly zero the two roots are `0` and `−b/a`, and
    // reading them off `(−b ± √(b²))/2a` instead costs the first one its
    // exactness: `√` of an interval steps outwards, so the root that ought to be
    // nothing at all comes back as a box straddling zero, and every point on a
    // bar's wall grazes its own face. Factoring is not an optimisation here, it
    // is the difference between an answer and a refusal.
    if k.lo() == 0.0 && k.hi() == 0.0 {
        return [
            Meeting::Behind,
            where_it_lands(quotient(-b, a), from, along),
        ];
    }
    let discriminant = b * b - Interval::exactly(4.0) * a * k;
    if discriminant.hi() < 0.0 {
        return [Meeting::Behind, Meeting::Behind];
    }
    if discriminant.contains_zero() {
        return [Meeting::CouldNotTell, Meeting::CouldNotTell];
    }
    let spread = root_enclosing(discriminant);
    [-b - spread, -b + spread].map(|top| where_it_lands(quotient(top, twice_a), from, along))
}

/// `a`, `b` and `k` of the quadratic a ray puts a ball to.
fn how_a_ray_meets_a_ball(
    ball: &Sphere,
    from: [Interval; 3],
    along: [Interval; 3],
) -> (Interval, Interval, Interval) {
    let out = negated(gap_enclosing(from, ball.centre()));
    let radius = Interval::exactly(ball.radius());
    (
        dot_enclosing(along, along),
        Interval::exactly(2.0) * dot_enclosing(along, out),
        dot_enclosing(out, out) - radius * radius,
    )
}

/// `a`, `b` and `k` of the quadratic a ray puts a pipe to.
///
/// The same three as a ball's, with the axis direction crossed out of both
/// vectors first — which is exactly how [`Cylinder`]'s own side predicate
/// measures the distance from the axis, so the two agree by construction rather
/// than by a rounding.
fn how_a_ray_meets_a_pipe(
    pipe: &Cylinder,
    from: [Interval; 3],
    along: [Interval; 3],
) -> (Interval, Interval, Interval) {
    let direction = exactly(pipe.axis().direction().components());
    let out = cross_enclosing(
        negated(gap_enclosing(from, pipe.axis().origin())),
        direction,
    );
    let across = cross_enclosing(along, direction);
    let radius = Interval::exactly(pipe.radius());
    (
        dot_enclosing(across, across),
        Interval::exactly(2.0) * dot_enclosing(across, out),
        dot_enclosing(out, out) - radius * radius,
    )
}

/// `a`, `b` and `k` of the quadratic a ray puts a cone to.
///
/// The pipe's three with the axial term put back: `‖x × w‖² − slope²(x·w)²`
/// against the pipe's `‖x × w‖² − r²`, which is the same cross product read the
/// same way, so the surface a root is worked out from is the one
/// [`side_of_surface`] is later asked about.
///
/// **`a` can be below zero here and cannot for a ball or a pipe**, because
/// `slope²(D·w)²` can beat `‖D × w‖²` — every ray pointing further into the cone
/// than its own rulings do. The quadratic is solved the same way either way; the
/// caller's comment says what changes.
fn how_a_ray_meets_a_cone(
    cone: &Cone,
    from: [Interval; 3],
    along: [Interval; 3],
) -> (Interval, Interval, Interval) {
    let direction = exactly(cone.axis().direction().components());
    let out = negated(gap_enclosing(from, cone.apex()));
    let across = cross_enclosing(along, direction);
    let sideways = cross_enclosing(out, direction);
    let slope = Interval::exactly(cone.slope());
    let spread = slope * slope;
    let up_the_ray = dot_enclosing(along, direction);
    let up_the_gap = dot_enclosing(out, direction);
    (
        dot_enclosing(across, across) - spread * up_the_ray * up_the_ray,
        Interval::exactly(2.0)
            * (dot_enclosing(across, sideways) - spread * up_the_ray * up_the_gap),
        dot_enclosing(sideways, sideways) - spread * up_the_gap * up_the_gap,
    )
}

/// What a root at `t` along the ray comes to.
///
/// A root at exactly zero is where the ray *starts*, which is the point sitting
/// on this face's own surface: the ray crosses nothing there, and whether the
/// point is on the face is asked separately. A root whose box merely covers zero
/// is a different thing — nobody can say whether the crossing is ahead — and it
/// is a graze.
fn where_it_lands(t: Interval, from: [Interval; 3], along: [Interval; 3]) -> Meeting {
    if t.hi() < 0.0 || (t.lo() == 0.0 && t.hi() == 0.0) {
        return Meeting::Behind;
    }
    if t.lo() <= 0.0 {
        return Meeting::CouldNotTell;
    }
    Meeting::Ahead {
        at: core::array::from_fn(|axis| from[axis] + t * along[axis]),
    }
}

/// Which way the surface faces outwards at `at`, as a box.
///
/// A polynomial in the stored numbers with no normalization anywhere in it, so
/// the sign of anything dotted with it is the sign of the real thing (§31.2's
/// row 12).
///
/// # Panics
///
/// On a surface this has not been taught, for the reason
/// [`where_the_ray_meets`] gives.
fn gradient_at(surface: &Surface, at: [Interval; 3]) -> [Interval; 3] {
    match surface {
        Surface::Plane(flat) => exactly(flat.normal().components()),
        // **Out of the middle and not towards it.** `gap_enclosing(a, b)` is the
        // step *from* `a` *to* `b`, so the step from the centre to the point is
        // the one turned round — the same way round `where_the_ray_meets` reads
        // it a few lines up. Taken the other way this is the gradient of the
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

/// `u × v` as a box.
fn cross_enclosing(u: [Interval; 3], v: [Interval; 3]) -> [Interval; 3] {
    core::array::from_fn(|axis| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        u[next] * v[after] - u[after] * v[next]
    })
}

/// `√value` as a box, for a value already shown to be above zero.
///
/// **A step outwards only where one is needed**, the same care `rcad-geom`'s own
/// enclosure takes and for the same reason: widening every root costs the exact
/// cases their exactness, and an exact case here is a ray that meets a surface at
/// a place `f64` can write down. `f64::sqrt` is correctly rounded, so squaring
/// what it hands back through [`two_product`] — which is exact — says which side
/// of the true root it landed, and only then is a step taken.
fn root_enclosing(value: Interval) -> Interval {
    let lo = rounded_down(value.lo().max(0.0));
    let hi = rounded_up(value.hi());
    Interval::new(lo.max(0.0), hi).unwrap_or(Interval::ENTIRE)
}

/// The largest `f64` at or below `√value`.
fn rounded_down(value: f64) -> f64 {
    let root = value.sqrt();
    if squared_is_at_most(root, value) {
        root
    } else {
        root.next_down()
    }
}

/// The smallest `f64` at or above `√value`.
fn rounded_up(value: f64) -> f64 {
    let root = value.sqrt();
    if squared_is_at_most(root, value) && !squared_is_exactly(root, value) {
        root.next_up()
    } else {
        root
    }
}

/// Whether `root · root` is at or below `value`, exactly.
fn squared_is_at_most(root: f64, value: f64) -> bool {
    if !product_is_exact(root, root) {
        return false;
    }
    let (high, low) = two_product(root, root);
    high < value || (high == value && low <= 0.0)
}

/// Whether `root · root` is exactly `value`.
fn squared_is_exactly(root: f64, value: f64) -> bool {
    if !product_is_exact(root, root) {
        return false;
    }
    let (high, low) = two_product(root, root);
    high == value && low == 0.0
}

/// `over / under` as a box, for a divisor already shown to be clear of zero.
///
/// **A step outwards only where one is needed**, which is the same care
/// `rcad-geom`'s own enclosure takes about roots and for the same reason: a ray
/// that starts exactly on a plane divides a numerator of nothing, and a widened
/// nothing is a box straddling zero, which reads as a ray that might be about to
/// cross the very face it is standing on.
fn quotient(over: Interval, under: Interval) -> Interval {
    let ends = [
        divided(over.lo(), under.lo()),
        divided(over.lo(), under.hi()),
        divided(over.hi(), under.lo()),
        divided(over.hi(), under.hi()),
    ];
    let lo = ends.iter().map(|end| end.0).fold(f64::INFINITY, f64::min);
    let hi = ends
        .iter()
        .map(|end| end.1)
        .fold(f64::NEG_INFINITY, f64::max);
    Interval::new(lo, hi).unwrap_or(Interval::ENTIRE)
}

/// `over / under` as the two ends of a box holding it.
///
/// The two ends are the same number when the division came out exact, and that
/// is *proved* rather than assumed: the quotient is multiplied back out through
/// [`two_product`], which is exact, and compared against what was divided.
fn divided(over: f64, under: f64) -> (f64, f64) {
    let quotient = over / under;
    if !quotient.is_finite() {
        return (f64::NEG_INFINITY, f64::INFINITY);
    }
    if product_is_exact(quotient, under) {
        let (high, low) = two_product(quotient, under);
        if high == over && low == 0.0 {
            return (quotient, quotient);
        }
    }
    (quotient.next_down(), quotient.next_up())
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
