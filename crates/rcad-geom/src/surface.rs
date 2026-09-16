//! The surfaces a face can sit on.

use core::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::{Point, Vector};
use crate::{Cone, Cylinder, Plane, Sphere, Torus};

/// The surface a face sits on.
///
/// A concrete list, not a trait object and not a type parameter. Making the
/// model generic over a `Geometry` trait would push a type parameter through
/// every signature in topology to buy a milestone boundary, and
/// `docs/decisions.md` §3.1 turned that down.
///
/// M0 built solids out of [`Plane`] only, and [`Sphere`] and [`Cylinder`] were
/// here as types so the enum was shaped right and the sign predicates had
/// somewhere to land. **Topology accepts all five now**: a round bar has a face
/// on a cylinder, a ball has one on a sphere, a taper has one on a cone and a
/// ring has one on a torus.
///
/// **Four of the five are quadrics and the fifth is not**, and almost every
/// place a torus is refused traces to that. A straight line meets a quadric
/// twice and a torus four times; a plane cuts a quadric in a conic and a torus
/// in a quartic. [`Torus`] says what follows.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Surface {
    /// A flat surface.
    Plane(Plane),
    /// A ball's skin.
    Sphere(Sphere),
    /// An endless round pipe's skin.
    Cylinder(Cylinder),
    /// A double cone's skin, pinched to nothing at its apex.
    Cone(Cone),
    /// A ring's skin.
    Torus(Torus),
}

impl Surface {
    /// The plane, if this is one. `None` for everything else.
    ///
    /// M0's face construction and its volume, area and centroid sums are planar
    /// only, and this is how they say so.
    #[must_use]
    pub fn as_plane(&self) -> Option<&Plane> {
        match self {
            Surface::Plane(plane) => Some(plane),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_) => {
                None
            }
        }
    }

    /// The point at `(u, v)` in this surface's own two coordinates.
    ///
    /// What each of the two *means* is the surface's own business — a distance
    /// across a plane, an angle round a cylinder, a latitude on a ball. The one
    /// thing they have in common is
    /// [`Surface::furthest_a_unit_step_goes`], which is what lets a mesher size
    /// a step without knowing which kind of surface it is holding.
    ///
    /// Exact only where the surface's own trigonometry is, which is nowhere
    /// interesting on a curved one. **No corner of a face is ever made this
    /// way**: corners come from exact constructions and are certified onto
    /// their surface by a predicate. This is for filling in between them, where
    /// the deviation is the thing to be bounded rather than avoided.
    #[must_use]
    pub fn point_at(&self, u: f64, v: f64) -> Point {
        match self {
            Surface::Plane(plane) => plane.point_at(u, v),
            Surface::Sphere(sphere) => sphere.point_at(u, v),
            Surface::Cylinder(cylinder) => cylinder.point_at(u, v),
            Surface::Cone(cone) => cone.point_at(u, v),
            Surface::Torus(torus) => torus.point_at(u, v),
        }
    }

    /// How far a point can move by going into this surface's two coordinates
    /// and back out again, as a multiple of the largest number involved.
    ///
    /// The size to multiply by is the largest magnitude among the point's three
    /// coordinates, the numbers the surface places itself with — a plane's
    /// offset, a ball's centre, a pipe's axis origin — and the radius. It is
    /// absolute in that size and never relative to the answer, for the reason
    /// [`Plane::SIGNED_DISTANCE_ROUNDOFF`] opens with: a coordinate on its way
    /// through zero still carries the rounding of the numbers it came from.
    ///
    /// Where the number comes from, on a pipe, with `u = f64::EPSILON / 2` and
    /// `L` the size above:
    ///
    /// - `point - axis origin` rounds each component, at most `u` of something
    ///   no bigger than `2L`.
    /// - Each of the three dot products against a unit direction is three
    ///   products and two sums on terms adding to at most `2√3·L`, so it is out
    ///   by about `12u·L`.
    /// - For a point on the skin the pair the angle is read from sits `radius`
    ///   from zero, so moving each of the two by `12u·L` turns the angle by
    ///   about `17u·L / radius`, and `atan2` adds a rounding of its own.
    /// - Turning back into a point multiplies that angle by the radius, which
    ///   cancels it back to `17u·L`. Then `cos` and `sin` round, and seven more
    ///   arithmetic steps each round something no bigger than about `4L`.
    ///
    /// That comes to under `100u·L`, which is `50 · f64::EPSILON · L`. The
    /// constant is five times it, and the room is for the trigonometry: Rust
    /// does not say how accurate `sin`, `cos` and `atan2` are, so this holds for
    /// a libm that is a few ulps out rather than one that is perfect.
    ///
    /// A ball's chain is the same shape with one term fewer — nothing runs
    /// along an axis — and a plane's is shorter still, with no trigonometry in
    /// it at all, so the pipe's number covers all three.
    ///
    /// The worst a search over balls and pipes across forty orders of magnitude
    /// of size can find is `2.4 · f64::EPSILON · L` — the ordinary gap between
    /// a worst case and a drawn one, because roundings mostly cancel.
    /// `a_round_trip_through_a_surfaces_own_two_coordinates.rs` measures it and
    /// prints the figure.
    ///
    /// **Turned the other way** — a coordinate handed to [`Surface::point_at`]
    /// and read back out — the same bound applies once it is converted into
    /// that coordinate, by dividing by how far a step in it actually carries
    /// you *there*. Round a pipe that is the radius. Round a ball it is the
    /// radius of the ring the point sits on, which shrinks to nothing at a
    /// pole — and that is the honest reason a pole's longitude is not
    /// determined, rather than a shortcoming of the arithmetic.
    ///
    /// What is **not** promised: bit equality in either direction, or that
    /// `coordinates_of(point_at(u, v))` is `(u, v)` for a `u` outside
    /// `(-π, π]`, which it is not, or at a pole, where it is not defined.
    pub const ROUND_TRIP_ROUNDOFF: f64 = 256.0 * f64::EPSILON;

    /// Where `point` lands in this surface's own two coordinates —
    /// [`Surface::point_at`] the other way round.
    ///
    /// Bare numbers, no [`Decision`](rcad_predicates::Decision) and no
    /// [`Result`](crate::Result). This decides nothing and cannot fail, and a
    /// `(u, v)` is a working space rather than an answer, so handing back
    /// something a caller could branch on would invite exactly the decision on
    /// projected coordinates the kernel is built not to take
    /// (`docs/decisions.md` §25.2).
    ///
    /// **It answers about the nearest point on the surface**, so a point off
    /// the surface gets its projection's coordinates. The axis of a pipe and
    /// the centre of a ball have no nearest point; the answer there is `(0, 0)`
    /// on a ball and `(0, along the axis)` on a pipe, which is what the
    /// arithmetic gives without a branch. Each surface's own `coordinates_of`
    /// says what its two coordinates mean and where they are not determined.
    #[must_use]
    pub fn coordinates_of(&self, point: Point) -> (f64, f64) {
        match self {
            Surface::Plane(plane) => plane.plane_coordinates_of(point),
            Surface::Sphere(sphere) => sphere.coordinates_of(point),
            Surface::Cylinder(cylinder) => cylinder.coordinates_of(point),
            Surface::Cone(cone) => cone.coordinates_of(point),
            Surface::Torus(torus) => torus.coordinates_of(point),
        }
    }

    /// Which way a point moves when this surface's **first** coordinate goes up,
    /// at `at`.
    ///
    /// [`Surface::point_at`] turns a pair of coordinates into a point,
    /// [`Surface::coordinates_of`] comes back, and this is the third of the set:
    /// it says which way the first of the two runs. `rcad-topo` calls it the
    /// surface's *longitude*, and it is the direction a closed edge is stored
    /// running along (`docs/decisions.md` §36.18) — the round-the-pipe angle on
    /// a pipe or a cone, the longitude on a ball, the angle round the **tube**
    /// on a bend of pipe, and the first of [`Plane::basis`]'s two directions on
    /// a flat one.
    ///
    /// **A direction and not a rate.** Nothing here is scaled to length one, so
    /// only the answer's direction means anything and its length does not. That
    /// is what keeps every arm a polynomial in the numbers the surface stores,
    /// with no square root anywhere in it (`docs/decisions.md` §31.2), which is
    /// what a caller reading the sign of a dot product needs. A caller that
    /// wants a *rate* wants [`Surface::furthest_a_unit_step_goes`].
    ///
    /// **This is meant to be the one spelling of it.** `rcad-topo` works the
    /// same direction out for itself in `validate.rs`, as
    /// `the_way_it_is_parametrised_about` crossed into `gradient_of`, and keeps
    /// a second `gradient_of` in `mesh.rs` besides. Two spellings of one
    /// question is how two answers come to disagree (`docs/decisions.md`
    /// §23.8), so those are to call this. Rewiring them is not this method's
    /// own change.
    ///
    /// # Where there is no answer
    ///
    /// `None`, in exactly the places each surface's own `coordinates_of` says
    /// its first coordinate is not determined by the geometry:
    ///
    /// - a ball's two **poles**, where every longitude runs through the one
    ///   point — which is [`Surface::ROUND_TRIP_ROUNDOFF`]'s own reason, that a
    ///   step in longitude carries nothing at all there;
    /// - a cone's **apex**, where the second coordinate pinches the first away
    ///   to nothing;
    /// - a point on the **axis** of a pipe, a cone or a bend, which has no
    ///   nearest point on the surface to answer about.
    ///
    /// A pipe and a bend pinch nowhere, so a point on either always has one, and
    /// a plane always has one.
    ///
    /// The question is about a point **on** the surface. A point off it gets its
    /// nearest point's answer wherever the arithmetic has one, the same way
    /// [`Surface::coordinates_of`] does.
    #[must_use]
    pub fn the_way_its_first_coordinate_runs_at(&self, at: Point) -> Option<Vector> {
        let running = match self {
            Surface::Plane(flat) => flat.basis().0,
            // A ball has no axis of its own to be wound about, so the frame is
            // the world's — [`Sphere::point_at`] says so in its arithmetic.
            Surface::Sphere(ball) => Vector::new(0.0, 0.0, 1.0).cross(at - ball.centre()),
            // `w × (at − origin)` is `w` crossed into the way the surface faces,
            // with the part of it along `w` dropped for costing nothing: `w`
            // crossed into itself is zero.
            Surface::Cylinder(pipe) => pipe.axis().direction().cross(at - pipe.axis().origin()),
            Surface::Cone(cone) => cone.axis().direction().cross(at - cone.apex()),
            Surface::Torus(ring) => Surface::the_way_a_bend_winds_round_its_tube_at(ring, at),
        };
        (running != Vector::zero()).then_some(running)
    }

    /// Which way a bend of pipe's first coordinate runs, which is round its
    /// **tube** and not round its ring.
    ///
    /// The other four wind about one fixed direction, so crossing that into the
    /// way the surface faces is the whole of it. A bend does not: the direction
    /// its tube winds about is the tangent to the centre circle, and that turns
    /// as you go round the ring. So this is the general form written out — the
    /// tangent `x × w`, crossed into the way the surface faces.
    ///
    /// Handed the bend's own axis instead of that tangent, the answer at a
    /// corner on a meridian circle would be square to the circle and the sign a
    /// caller reads off it would be exactly zero.
    fn the_way_a_bend_winds_round_its_tube_at(ring: &Torus, at: Point) -> Vector {
        let (centre, along) = (ring.centre(), ring.axis().direction());
        let out = at - centre;
        let (big, small) = (ring.ring_radius(), ring.tube_radius());
        // The gradient of `(w·w)(x·x + R² − r²)² − 4R²‖x × w‖²` at
        // `x = at − centre`, divided by four.
        let bulge = out.dot(out) - big * big - small * small;
        let facing = out * (along.dot(along) * bulge) + along * (2.0 * big * big * along.dot(out));
        out.cross(along).cross(facing)
    }

    /// How far in space one unit of `u`, and one of `v`, can carry you.
    ///
    /// An **upper bound**, not a rate, and the difference is why it is named
    /// this way. On a plane a unit is a unit; along a cylinder a unit is a
    /// unit, and round one of radius `r` it is exactly `r`. On a ball a step in
    /// longitude covers `r` at the equator and less and less towards the poles,
    /// reaching nothing at all — so `r` is the most it can be and almost never
    /// what it is.
    ///
    /// That asymmetry is not a defect waiting to be fixed. Triangulating a
    /// cylinder in `(r·u, v)` scales its domain into an *exact* isometry,
    /// because a cylinder unrolls flat. A ball does not unroll at all, so no
    /// scaling makes its domain faithful and the crowding near its poles is
    /// real. It costs triangles rather than correctness, because what governs
    /// correctness is the proven deviation and not the shape of the triangles.
    ///
    /// # A cone's first coordinate has no ceiling, and infinity is the honest
    /// answer
    ///
    /// A step of one radian round a cone carries `slope · v`, which grows
    /// without limit as the cone widens, so **no finite number bounds it over
    /// the whole surface** and this hands back [`f64::INFINITY`]. That is not a
    /// gap to be closed by a cleverer bound: the quantity being bounded is
    /// unbounded.
    ///
    /// What a mesher wants is a scaling for one *patch*, and a patch is
    /// bounded — [`Cone::radius_at`] at the patch's own furthest reach is the
    /// number, and `rcad-mesh` works it out from the corners it was handed.
    /// Along a ruling a unit of `v` carries exactly `√(1 + slope²)`, which is
    /// finite and is the same everywhere, so the second half of the pair is a
    /// bound like the others.
    #[must_use]
    pub fn furthest_a_unit_step_goes(&self) -> (f64, f64) {
        match self {
            Surface::Plane(_) => (1.0, 1.0),
            Surface::Sphere(sphere) => (sphere.radius(), sphere.radius()),
            Surface::Cylinder(cylinder) => (cylinder.radius(), 1.0),
            Surface::Cone(cone) => (f64::INFINITY, cone.slope().hypot(1.0)),
            // Round the tube first, because that is the coordinate a face on a
            // bend of pipe wraps in — see [`Torus::point_at`].
            Surface::Torus(ring) => (ring.tube_radius(), ring.ring_radius() + ring.tube_radius()),
        }
    }

    /// Where this surface sorts against another, on the numbers each one
    /// stores.
    ///
    /// **Two questions need a fixed order over a *pair or triple* of surfaces,
    /// and neither may invent its own.** Where three surfaces meet has to name a
    /// corner the same way whichever face asked, or a fuse's two arguments
    /// number the same corner differently (`docs/decisions.md` §31.3); and a
    /// closed crossing has to start at the same place whichever of its two faces
    /// walks it, or the one vertex §31.3 puts on it lands in two places.
    ///
    /// Sorting on the stored numbers is sound for exactly the reason §32.5's
    /// trap gives: two faces that are the same plane hold the same four numbers.
    /// Each of the five earns that for itself, and none of the five is left out:
    /// a [`Plane`](crate::Plane)'s normal and an [`Axis`](crate::Axis)'s
    /// direction are scaled to length one from the largest component, so two
    /// spellings that differ only in how long the vector was written hold the
    /// same bits, and a [`Sphere`]'s centre has nothing to scale. A pipe, a cone
    /// and a bend take theirs from the axis they sit on.
    ///
    /// It is **not** sound for a [`HalfSpace`](crate::HalfSpace), and that one
    /// is exempt rather than unfinished: a half-space has no canonical form at
    /// all, two of them describing one flat surface may hold quite different
    /// numbers, and nothing anywhere may compare two for equality or key on one
    /// (§33.8). A half-space is not a [`Surface`] and cannot reach here.
    ///
    /// **[`f64::total_cmp`](f64::total_cmp) tells `-0.0` from `0.0`**, so "the
    /// same four numbers" has to mean the same bits and not the same values.
    /// [`Plane`](crate::Plane), [`Axis`](crate::Axis) and [`Sphere`] each fold a
    /// negative zero out of every number they store — see the note on any of the
    /// three for what a single one costs.
    ///
    /// Planes sort before balls and balls before pipes, which is what puts the
    /// two flat ones of a triple at the front. **That is why this is public and
    /// not `pub(crate)`**: it is how a reader is to pick which of the two
    /// surfaces either side of a closed edge names the way that edge runs, and
    /// `rcad-topo`'s `the_curved_one_of` already asks its own copy of the same
    /// bits. The two have to land on the same surface or a file gets imported
    /// walked against a rule it was read by. Nothing in `rcad-step` asks yet —
    /// the reader that will is the rest of this work, and until it exists the
    /// wider spelling carries no caller of its own.
    ///
    /// **A pipe, a cone and a bend written with the axis pointing the other way
    /// sort apart here, on purpose.** The two spellings are one surface, and
    /// `rcad-topo`'s `name_of_the_surface_either_way_round` is where that is
    /// folded, because folding it *here* would make two different bit patterns
    /// compare **equal** — and a sort whose ties are broken by the order the
    /// caller handed them in is the thing this exists to stop. What the fold
    /// costs nothing to leave out is measured rather than argued:
    /// `turning_a_stored_axis_round_moves_no_corner_and_renumbers_none` sweeps
    /// 12 960 triples of pipes, cones and bends with every stored axis turned
    /// round and gets the same corners in the same order, bit for bit, so the
    /// row they sort into never mattered.
    #[must_use]
    pub fn how_it_sorts_against(&self, other: &Surface) -> Ordering {
        let (mine, my_numbers) = self.the_numbers_it_is_written_with();
        let (theirs, their_numbers) = other.the_numbers_it_is_written_with();
        mine.cmp(&theirs).then_with(|| {
            my_numbers
                .iter()
                .zip(their_numbers.iter())
                .map(|(mine, theirs)| mine.total_cmp(theirs))
                .find(|order| order.is_ne())
                .unwrap_or(Ordering::Equal)
        })
    }

    /// Which kind this is, and the numbers it places and sizes itself with.
    ///
    /// **Every number a surface holds goes in**, and a torus is why the row is
    /// eight long rather than seven: two tori about one axis with the same ring
    /// radius and different tubes are two surfaces, and a row that left the tube
    /// out would sort them equal and name one corner for both.
    fn the_numbers_it_is_written_with(&self) -> (u8, [f64; 8]) {
        match self {
            Surface::Plane(flat) => {
                let [a, b, c, d] = flat.coefficients();
                (0, [a, b, c, d, 0.0, 0.0, 0.0, 0.0])
            }
            Surface::Sphere(ball) => {
                let [x, y, z] = ball.centre().coordinates();
                (1, [x, y, z, ball.radius(), 0.0, 0.0, 0.0, 0.0])
            }
            Surface::Cylinder(pipe) => {
                let [x, y, z] = pipe.axis().origin().coordinates();
                let [u, v, w] = pipe.axis().direction().components();
                (2, [x, y, z, u, v, w, pipe.radius(), 0.0])
            }
            Surface::Cone(cone) => {
                let [x, y, z] = cone.apex().coordinates();
                let [u, v, w] = cone.axis().direction().components();
                (3, [x, y, z, u, v, w, cone.slope(), 0.0])
            }
            Surface::Torus(ring) => {
                let [x, y, z] = ring.centre().coordinates();
                let [u, v, w] = ring.axis().direction().components();
                (
                    4,
                    [x, y, z, u, v, w, ring.ring_radius(), ring.tube_radius()],
                )
            }
        }
    }

    /// The same surface after `movement` is applied.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Surface {
        match self {
            Surface::Plane(plane) => Surface::Plane(plane.transformed(movement)),
            Surface::Sphere(sphere) => Surface::Sphere(sphere.transformed(movement)),
            Surface::Cylinder(cylinder) => Surface::Cylinder(cylinder.transformed(movement)),
            Surface::Cone(cone) => Surface::Cone(cone.transformed(movement)),
            Surface::Torus(ring) => Surface::Torus(ring.transformed(movement)),
        }
    }
}

impl From<Plane> for Surface {
    fn from(plane: Plane) -> Self {
        Surface::Plane(plane)
    }
}

impl From<Sphere> for Surface {
    fn from(sphere: Sphere) -> Self {
        Surface::Sphere(sphere)
    }
}

impl From<Cylinder> for Surface {
    fn from(cylinder: Cylinder) -> Self {
        Surface::Cylinder(cylinder)
    }
}

impl From<Cone> for Surface {
    fn from(cone: Cone) -> Self {
        Surface::Cone(cone)
    }
}

impl From<Torus> for Surface {
    fn from(torus: Torus) -> Self {
        Surface::Torus(torus)
    }
}

#[cfg(test)]
mod tests {
    use rcad_predicates::{Policy, ProvenanceLog};

    use super::*;
    use crate::three_surfaces::{MeetOfThree, meet_of_three_surfaces};
    use crate::{Axis, Point, Vector};

    #[test]
    fn only_a_plane_answers_to_being_asked_for_a_plane() {
        let plane =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0)).unwrap();
        assert_eq!(Surface::from(plane).as_plane(), Some(&plane));

        let ball = Sphere::new(Point::origin(), 1.0).unwrap();
        assert_eq!(Surface::from(ball).as_plane(), None);

        let pipe = Cylinder::new(Axis::z(), 1.0).unwrap();
        assert_eq!(Surface::from(pipe).as_plane(), None);
    }

    /// Stepping the first coordinate up moves the point the way
    /// [`Surface::the_way_its_first_coordinate_runs_at`] said it would, on all
    /// five kinds.
    ///
    /// The check is on the *direction* and nothing else, because that is all the
    /// answer claims to be: a small step forwards in the first coordinate,
    /// minus the point itself, has to point the same way — a positive dot — and
    /// a step backwards the other. A dot rather than an angle, so no tolerance
    /// is chosen anywhere and nothing is scaled.
    ///
    /// Comparing against a *step* rather than against a hand-written formula is
    /// the point: a formula copied out of `point_at` would pass whatever
    /// `point_at` did, and what is being checked is that the two agree.
    #[test]
    fn stepping_the_first_coordinate_up_moves_the_way_this_says_it_will() {
        let step = 1.0e-6;
        let mut checked = 0;
        for (surface, places) in every_surface_with_places_on_it() {
            for (first, second) in places {
                let here = surface.point_at(first, second);
                let running = surface
                    .the_way_its_first_coordinate_runs_at(here)
                    .unwrap_or_else(|| {
                        panic!("{surface:?} has no first coordinate at ({first}, {second})")
                    });
                let forwards = surface.point_at(first + step, second) - here;
                let backwards = surface.point_at(first - step, second) - here;
                assert!(
                    forwards.dot(running) > 0.0,
                    "{surface:?} at ({first}, {second}): a step up the first coordinate goes \
                     {forwards:?}, which is not the way {running:?} points"
                );
                assert!(
                    backwards.dot(running) < 0.0,
                    "{surface:?} at ({first}, {second}): a step down the first coordinate goes \
                     {backwards:?}, which is not against {running:?}"
                );
                checked += 1;
            }
        }
        assert!(checked > 100, "a sweep this small proves little: {checked}");
    }

    /// A ball's poles and a cone's apex have no first coordinate, and say so.
    ///
    /// These are the two places on a surface where it pinches — every longitude
    /// of a ball runs through its pole, and a cone's angle means nothing at its
    /// apex — and both are written out exactly here rather than stepped towards,
    /// because `Sphere::point_at` lands a hair off a pole (`cos(π/2)` is
    /// `6.1e-17`) and a hair off is a place that *does* have a first coordinate.
    #[test]
    fn a_balls_pole_and_a_cones_apex_have_no_first_coordinate() {
        let ball = Sphere::new(Point::new(1.0, 2.0, 3.0), 5.0).unwrap();
        for pole in [Point::new(1.0, 2.0, 8.0), Point::new(1.0, 2.0, -2.0)] {
            assert_eq!(
                Surface::from(ball).the_way_its_first_coordinate_runs_at(pole),
                None,
                "a pole of {ball:?}"
            );
        }

        let cone = Cone::new(
            Axis::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 0.0, 1.0)).unwrap(),
            0.5,
        )
        .unwrap();
        assert_eq!(
            Surface::from(cone).the_way_its_first_coordinate_runs_at(cone.apex()),
            None
        );

        let pipe = Cylinder::new(Axis::z(), 2.0).unwrap();
        assert_eq!(
            Surface::from(pipe).the_way_its_first_coordinate_runs_at(Point::new(0.0, 0.0, 7.0)),
            None,
            "a point on a pipe's axis has no nearest point on its skin"
        );
    }

    #[test]
    fn moving_a_surface_moves_whichever_surface_it_is() {
        let shift = Isometry::translation(Vector::new(0.0, 0.0, 4.0)).unwrap();

        let ball = Sphere::new(Point::origin(), 1.0).unwrap();
        let moved = Surface::from(ball).transformed(&shift);
        assert_eq!(
            moved,
            Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 1.0).unwrap())
        );
    }

    /// A pipe, a cone and a bend written with the axis pointing the other way
    /// are the same surface, and every corner they take part in comes back at
    /// the same bits and in the same order.
    ///
    /// The reason to expect it is that all three sign expressions are **even**
    /// in the stored direction — `‖(p − o) × w‖²` and `slope²((p − o)·w)²` both
    /// take `w` twice — so negating it negates two factors of every product and
    /// nothing at all of any sum. Expecting it is not measuring it, and this
    /// sweeps the whole of [`meet_of_three_surfaces`] rather than the
    /// expression: two planes and one curved surface, and one plane against two
    /// quadrics that bend the same way, with every subset of the stored axes
    /// turned round. Bit for bit, points in order, refusals against refusals.
    ///
    /// **It is what makes folding the key safe.** A corner is named by its three
    /// surfaces and by `which` of their meeting points it is, and `which` counts
    /// along the line the two planes cross in. Two spellings of one pipe would
    /// be named alike by a folded key, so if either spelling moved a corner or
    /// renumbered one, the fold would put two different corners under one name —
    /// which is worse than the two names it fixes.
    #[test]
    fn turning_a_stored_axis_round_moves_no_corner_and_renumbers_none() {
        let (compared, placed, differed) = every_triple_against_its_turned_selves();
        assert!(
            compared > 10_000,
            "a sweep this small proves little: {compared} comparisons"
        );
        // A refusal matching a refusal says nothing about where a corner lands,
        // so the sweep is only worth its name if most of it placed corners.
        assert!(
            placed > compared / 2,
            "{placed} of {compared} comparisons placed a corner at all"
        );
        assert_eq!(differed, 0, "of {compared} comparisons");
    }

    /// How many triples were compared against a turned-round spelling of
    /// themselves, how many of those placed at least one corner, and how many
    /// came back with a different answer.
    fn every_triple_against_its_turned_selves() -> (usize, usize, usize) {
        let mut compared = 0;
        let mut placed = 0;
        let mut differed = 0;
        let mut check = |triple: [Surface; 3], turned: [Surface; 3]| {
            compared += 1;
            let (written, turned) = (what_they_meet_at(&triple), what_they_meet_at(&turned));
            if written.as_ref().is_ok_and(|places| !places.is_empty()) {
                placed += 1;
            }
            if written != turned {
                differed += 1;
            }
        };

        for curved in every_curved_surface() {
            for (one, other) in every_pair_of_planes() {
                check(
                    [one, other, curved],
                    [one, other, with_its_axis_turned_round(&curved)],
                );
            }
        }
        for (one, other) in every_pair_that_bends_the_same_way() {
            for flat in every_plane() {
                for turn_the_first in [false, true] {
                    for turn_the_second in [false, true] {
                        if !turn_the_first && !turn_the_second {
                            continue;
                        }
                        let turn = |surface: &Surface, turning| {
                            if turning {
                                with_its_axis_turned_round(surface)
                            } else {
                                *surface
                            }
                        };
                        check(
                            [flat, one, other],
                            [
                                flat,
                                turn(&one, turn_the_first),
                                turn(&other, turn_the_second),
                            ],
                        );
                    }
                }
            }
        }
        (compared, placed, differed)
    }

    /// What three surfaces meet at, as something two answers can be compared by
    /// without a tolerance anywhere: each point's enclosure as the bits of its
    /// two ends, in the order the meet put them, and a refusal as its own name.
    fn what_they_meet_at(surfaces: &[Surface; 3]) -> Result<Vec<[u64; 6]>, String> {
        let [first, second, third] = surfaces;
        let answer = meet_of_three_surfaces(
            [first, second, third],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        match answer {
            Ok(MeetOfThree::At(places)) => Ok(places
                .iter()
                .map(|place| {
                    let sides = place.enclosure();
                    [
                        sides[0].lo().to_bits(),
                        sides[0].hi().to_bits(),
                        sides[1].lo().to_bits(),
                        sides[1].hi().to_bits(),
                        sides[2].lo().to_bits(),
                        sides[2].hi().to_bits(),
                    ]
                })
                .collect()),
            Ok(other) => Err(format!("{other:?}")),
            Err(gave_up) => Err(format!("{gave_up:?}")),
        }
    }

    fn with_its_axis_turned_round(surface: &Surface) -> Surface {
        match surface {
            Surface::Cylinder(pipe) => Surface::Cylinder(
                Cylinder::new(pipe.axis().reversed(), pipe.radius()).expect("a turned pipe"),
            ),
            Surface::Cone(cone) => Surface::Cone(
                Cone::new(cone.axis().reversed(), cone.slope()).expect("a turned cone"),
            ),
            Surface::Torus(ring) => Surface::Torus(
                Torus::new(
                    ring.axis().reversed(),
                    ring.ring_radius(),
                    ring.tube_radius(),
                )
                .expect("a turned bend"),
            ),
            Surface::Plane(_) | Surface::Sphere(_) => {
                panic!("{surface:?} stores no axis to turn round")
            }
        }
    }

    fn every_curved_surface() -> Vec<Surface> {
        let mut all = Vec::new();
        for axis in every_axis() {
            for radius in [1.0, 2.0, 3.0] {
                all.push(Surface::Cylinder(
                    Cylinder::new(axis, radius).expect("a radius"),
                ));
            }
            for slope in [0.5, 2.0] {
                all.push(Surface::Cone(Cone::new(axis, slope).expect("a slope")));
            }
            all.push(Surface::Torus(
                Torus::new(axis, 3.0, 1.0).expect("a ring and a tube"),
            ));
        }
        all
    }

    /// Pairs a plane can be asked about without the quartic: two pipes about one
    /// line's worth of direction, or two cones of one slope about it.
    fn every_pair_that_bends_the_same_way() -> Vec<(Surface, Surface)> {
        let mut pairs = Vec::new();
        for direction in every_direction() {
            let places = [
                Point::origin(),
                Point::new(1.0, 0.0, 0.0),
                Point::new(0.0, 2.0, -1.0),
            ];
            for (here, there) in [(0, 1), (0, 2), (1, 2)] {
                let mine = Axis::new(places[here], direction).expect("a direction");
                let theirs = Axis::new(places[there], direction).expect("a direction");
                for radius in [1.0, 3.0] {
                    pairs.push((
                        Surface::Cylinder(Cylinder::new(mine, radius).expect("a radius")),
                        Surface::Cylinder(Cylinder::new(theirs, 2.0).expect("a radius")),
                    ));
                }
                pairs.push((
                    Surface::Cone(Cone::new(mine, 0.5).expect("a slope")),
                    Surface::Cone(Cone::new(theirs, 0.5).expect("a slope")),
                ));
            }
        }
        pairs
    }

    /// Every kind of surface, with places on it to ask about, in that surface's
    /// own two coordinates.
    ///
    /// A cone's and a ball's lists keep away from the apex and the poles, which
    /// have no answer and are checked on their own.
    fn every_surface_with_places_on_it() -> Vec<(Surface, Vec<(f64, f64)>)> {
        let angles = [-3.0, -1.5, -0.4, 0.0, 0.7, 2.2, 3.1];
        let mut all = Vec::new();
        for axis in every_axis() {
            let mut round_and_along = Vec::new();
            for round in angles {
                for along in [-2.0, 1.0, 4.0] {
                    round_and_along.push((round, along));
                }
            }
            all.push((
                Surface::Cylinder(Cylinder::new(axis, 2.0).expect("a radius")),
                round_and_along.clone(),
            ));
            all.push((
                Surface::Cone(Cone::new(axis, 0.5).expect("a slope")),
                round_and_along,
            ));
            let mut over_and_round = Vec::new();
            for over in angles {
                for round in [-2.0, 0.3, 2.5] {
                    over_and_round.push((over, round));
                }
            }
            all.push((
                Surface::Torus(Torus::new(axis, 3.0, 1.0).expect("two radii")),
                over_and_round,
            ));
        }
        let mut round_and_up = Vec::new();
        for round in angles {
            for up in [-1.2, -0.3, 0.0, 0.9] {
                round_and_up.push((round, up));
            }
        }
        all.push((
            Surface::Sphere(Sphere::new(Point::new(1.0, -2.0, 0.5), 3.0).expect("a radius")),
            round_and_up,
        ));
        all.push((
            Surface::Plane(
                Plane::from_point_and_normal(Point::new(0.0, 1.0, 2.0), Vector::new(1.0, 2.0, 3.0))
                    .expect("a normal with length"),
            ),
            vec![(0.0, 0.0), (1.5, -2.0), (-3.0, 4.0)],
        ));
        all
    }

    fn every_axis() -> Vec<Axis> {
        let mut axes = Vec::new();
        for origin in [
            Point::origin(),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 2.0, -1.0),
        ] {
            for direction in every_direction() {
                axes.push(Axis::new(origin, direction).expect("a direction"));
            }
        }
        axes
    }

    fn every_direction() -> [Vector; 4] {
        [
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(1.0, 1.0, 0.0),
            Vector::new(1.0, 2.0, 3.0),
        ]
    }

    fn every_pair_of_planes() -> Vec<(Surface, Surface)> {
        let all = every_plane();
        let mut pairs = Vec::new();
        for (here, one) in all.iter().enumerate() {
            for other in all.iter().skip(here + 1) {
                pairs.push((*one, *other));
            }
        }
        pairs
    }

    fn every_plane() -> Vec<Surface> {
        let mut all = Vec::new();
        for normal in [
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(1.0, 1.0, 0.0),
            Vector::new(0.0, 1.0, -1.0),
            Vector::new(1.0, -2.0, 1.0),
        ] {
            for through in [
                Point::origin(),
                Point::new(0.0, 0.0, 2.0),
                Point::new(1.0, 2.0, 3.0),
            ] {
                all.push(Surface::Plane(
                    Plane::from_point_and_normal(through, normal).expect("a normal with length"),
                ));
            }
        }
        all
    }
}
