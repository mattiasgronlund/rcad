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

use rcad_geom::Point;
use rcad_predicates::{
    Decision, Point as PredicatePoint, Policy, ProvenanceLog, orient3d_tolerantly,
};

use crate::{FaceView, SolidView};

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
    /// The solid's faces are broken into triangles, which form a closed
    /// surface. A ray is fired from `point` towards a far-off target and every
    /// triangle it passes through is counted, `+1` for a crossing that leaves
    /// the material and `-1` for one that enters it. The total is one inside
    /// and zero outside. Every test on the way is an exact predicate from the
    /// cascade — no tolerances, no epsilon.
    ///
    /// A face whose plane holds `point` is asked separately whether its
    /// material holds it too, and a face that says yes puts the point on the
    /// boundary and that answer wins. One triangle of the fan holding the point
    /// is not that question: the fan overlaps itself and covers a hole twice
    /// over, once each way, so only the whole outline can say (see
    /// [`face_holds_the_point`]).
    ///
    /// The three corners of a triangle are coordinates stored in the model and
    /// stay exact. The ray's far target is picked from the solid's bounding box
    /// and is exact too. `point` is the only thing that may be a box, and it is
    /// handed to every predicate as it is — nothing here rounds it down to a
    /// representative coordinate first.
    ///
    /// If the ray meets an edge of a triangle exactly, the count is not to be
    /// trusted, so the whole ray is thrown away and the next of three fixed
    /// targets is tried. If all three graze something, the answer is
    /// [`Containment::Undecidable`].
    ///
    /// # Why the answer never wobbles
    ///
    /// The benchmark decisions.md §9.2 asks for compares a serial run against a
    /// parallel one bit for bit, so nothing here may depend on when or where it
    /// runs:
    ///
    /// - The three ray targets come from the solid's bounding box, and `min`
    ///   and `max` over the corner coordinates do not care what order they see
    ///   them in.
    /// - Faces, wires and triangles come out in stored order, which is the same
    ///   on every call.
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
        let Some((low, high)) = self.bounding_box() else {
            return Containment::Outside;
        };
        if is_outside_the_box(point, low, high) {
            return Containment::Outside;
        }

        let mut worst_enclosure = 0.0f64;
        for target in ray_targets(low, high) {
            let pass = self.count_crossings(point, target, policy, log);
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

    fn count_crossings(
        self,
        point: PredicatePoint,
        target: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Crossings {
        let mut tally = Crossings::default();
        for face in self.faces() {
            let mut on_the_plane = false;
            face.for_each_triangle(|corners| {
                match look_at_triangle(corners, point, target, policy, log) {
                    Hit::Missed => {}
                    Hit::WentThrough { leaving } => tally.winding += leaving,
                    Hit::StartsOnThePlane => on_the_plane = true,
                    Hit::Grazed { enclosure } => {
                        tally.grazed_something = true;
                        tally.worst_enclosure = tally.worst_enclosure.max(enclosure);
                    }
                }
            });
            if on_the_plane {
                match face_holds_the_point(face, point, target, policy, log) {
                    Some(true) => tally.on_boundary = true,
                    Some(false) => (),
                    None => tally.grazed_something = true,
                }
            }
        }
        tally
    }
}

/// Whether the whole of `point`'s box sits off one side of the solid's
/// bounding box.
///
/// This decides `Outside` without firing a ray, and it is allowed to because it
/// is a proof rather than a shortcut. Every triangle of the solid has its three
/// corners inside the bounding box, and an enclosure is a range the point is
/// guaranteed to be somewhere in, so a range lying entirely past one side of
/// the box puts the point outside the solid with nothing left over. No
/// tolerance is involved and no sign of a rounded quantity is read.
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
    /// [`face_holds_the_point`] is what answers it.
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

/// Whether the material of `face` holds `point`, which lies on the face's
/// plane, or `None` when a predicate could not settle it.
///
/// [`FaceView::for_each_triangle`] cannot answer this. Its triangles overlap
/// wherever the face is not convex and its spokes are edges of nothing, so one
/// triangle holding the point says nothing about the face: the fan over a frame
/// covers the frame's own hole twice, once each way, and the two only cancel
/// once they are added up.
///
/// So the outline itself is walked. A half-line is run from `point` towards the
/// first corner and the edges crossing it are counted, `+1` for one way round
/// and `-1` for the other; a hole subtracts itself because it runs the other
/// way, and the count is non-zero exactly on the material. Which side of that
/// half-line's line each corner falls is read as a half-open range — a corner
/// exactly on the line counts as below it — and that is what leaves no tie to
/// break and no second try to make. The one case a crossing count cannot see is
/// the point sitting on an edge, and that is looked for as we go.
///
/// Every test is an [`orient3d`](orient3d_tolerantly) against `target`, which
/// the caller has already shown is off the face's plane: had it been on the
/// plane, every edge test in [`look_at_triangle`] would have come out zero and
/// no triangle would have reported [`Hit::StartsOnThePlane`]. Three points on a
/// plane turn one way about its normal exactly when they turn that way about a
/// fourth point off it, so nothing is projected into the plane's own two
/// coordinates (decisions.md §15.5).
fn face_holds_the_point(
    face: FaceView<'_>,
    point: PredicatePoint,
    target: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Option<bool> {
    let wires: Vec<Vec<Point>> = face.wires().map(|wire| wire.points().collect()).collect();
    let aim = *wires.first()?.first()?;

    let mut winding = 0i64;
    for wire in &wires {
        let sides: Vec<Decision> = wire
            .iter()
            .map(|&corner| decide(corner, aim, target, point, policy, log))
            .collect();
        for (index, &from) in wire.iter().enumerate() {
            let next = (index + 1) % wire.len();
            let along = decide(from, wire[next], target, point, policy, log);
            match along {
                Decision::Zero => {
                    if lies_between(from, wire[next], point)? {
                        return Some(true);
                    }
                }
                Decision::Pos | Decision::Neg => (),
                Decision::Undecidable { .. } => return None,
                other => {
                    panic!("orient3d came back with a decision nothing here knows: {other:?}")
                }
            }
            winding += crossing(sides[index], sides[next], along)?;
        }
    }
    Some(winding != 0)
}

/// Whether `point`, already shown to be on the line through `from` and `to`, is
/// on the segment between them. `None` when its box straddles an end.
///
/// The comparison runs along whichever axis the segment reaches furthest, which
/// is decisions.md §17.4's rule for ordering points along a line and needs no
/// predicate of its own.
fn lies_between(from: Point, to: Point, point: PredicatePoint) -> Option<bool> {
    let (from, to) = (from.coordinates(), to.coordinates());
    let axis = (0..3)
        .max_by(|&one, &other| {
            (to[one] - from[one])
                .abs()
                .total_cmp(&(to[other] - from[other]).abs())
        })
        .expect("three axes");
    let (low, high) = (from[axis].min(to[axis]), from[axis].max(to[axis]));
    let span = point.enclosure()[axis];
    if span.lo() >= low && span.hi() <= high {
        Some(true)
    } else if span.hi() < low || span.lo() > high {
        Some(false)
    } else {
        None
    }
}

/// `1` or `-1` when the edge crosses the half-line, signed by which way it
/// crosses, and `0` when it does not.
///
/// `from_side` and `to_side` say which side of the half-line's line each end of
/// the edge falls on, and `along` says which side of the edge the point falls
/// on — which is what tells a crossing ahead of the point from one behind it.
fn crossing(from_side: Decision, to_side: Decision, along: Decision) -> Option<i64> {
    Some(
        match (above_the_line(from_side)?, above_the_line(to_side)?, along) {
            (false, true, Decision::Pos) => 1,
            (true, false, Decision::Neg) => -1,
            _ => 0,
        },
    )
}

/// Whether a corner is above the half-line's line, counting a corner exactly on
/// it as below.
///
/// Half-open on purpose. It is what gives every edge two ends on definite sides
/// and leaves the crossing count with no tie to break, however many corners the
/// line happens to run through.
fn above_the_line(side: Decision) -> Option<bool> {
    match side {
        Decision::Pos => Some(true),
        Decision::Neg | Decision::Zero => Some(false),
        Decision::Undecidable { .. } => None,
        other => panic!("orient3d came back with a decision nothing here knows: {other:?}"),
    }
}

/// Which side of the plane through `a`, `b` and `c` the point `d` is on.
///
/// `a`, `b` and `c` are always exact: a triangle corner read out of the model,
/// or a ray target worked out from the bounding box. `d` is the point being
/// classified and is the one that may be a box.
fn decide(
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
