use rcad_geom::Point;
use rcad_predicates::{Decision, Point as ExactPoint, Policy, ProvenanceLog, orient3d_tolerantly};

use crate::SolidView;

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
    /// # How it works
    ///
    /// The solid's faces are broken into triangles, which form a closed
    /// surface. A ray is fired from `point` towards a far-off target and every
    /// triangle it passes through is counted, `+1` for a crossing that leaves
    /// the material and `-1` for one that enters it. The total is one inside
    /// and zero outside. Every test on the way is an exact predicate from the
    /// cascade — no tolerances, no epsilon.
    ///
    /// A triangle whose plane holds `point`, with the point inside the
    /// triangle, means the point is on the boundary and that answer wins.
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
    pub fn classify_point(
        self,
        point: Point,
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
        point: Point,
        target: Point,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Crossings {
        let mut tally = Crossings::default();
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                match look_at_triangle(corners, point, target, policy, log) {
                    Hit::Missed => {}
                    Hit::WentThrough { leaving } => tally.winding += leaving,
                    Hit::PointIsOnIt => tally.on_boundary = true,
                    Hit::Grazed { enclosure } => {
                        tally.grazed_something = true;
                        tally.worst_enclosure = tally.worst_enclosure.max(enclosure);
                    }
                }
            });
        }
        tally
    }
}

fn is_outside_the_box(point: Point, low: Point, high: Point) -> bool {
    let point = point.coordinates();
    let low = low.coordinates();
    let high = high.coordinates();
    (0..3).any(|axis| point[axis] < low[axis] || point[axis] > high[axis])
}

/// Three far-off points to aim at, in turn.
///
/// Each is well clear of the solid and none is square-on to a coordinate axis,
/// because an axis-aligned ray runs straight along the edges of an axis-aligned
/// box and grazes everything. The multipliers are arbitrary; any fixed set of
/// three that point in different directions does the same job.
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
    /// The point being classified lies on this triangle.
    PointIsOnIt,
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
    point: Point,
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
        Decision::Zero => Hit::PointIsOnIt,
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

fn decide(
    a: Point,
    b: Point,
    c: Point,
    d: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    orient3d_tolerantly(
        ExactPoint::from(a.coordinates()),
        ExactPoint::from(b.coordinates()),
        ExactPoint::from(c.coordinates()),
        ExactPoint::from(d.coordinates()),
        policy,
        log,
    )
}
