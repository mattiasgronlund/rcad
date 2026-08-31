use rcad_geom::{Point, Vector};

use crate::SolidView;

/// How much space a solid takes up, how much skin it has, and where it
/// balances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    volume: f64,
    surface_area: f64,
    centroid: Point,
}

impl Measurement {
    /// How much space the solid encloses.
    #[must_use]
    pub fn volume(self) -> f64 {
        self.volume
    }

    /// The total area of all its faces.
    #[must_use]
    pub fn surface_area(self) -> f64 {
        self.surface_area
    }

    /// The point the solid would balance on, if it were of even density.
    #[must_use]
    pub fn centroid(self) -> Point {
        self.centroid
    }
}

impl SolidView<'_> {
    /// Measure the solid: volume, surface area and centroid in one pass.
    ///
    /// Straight out of the divergence theorem, over the triangles
    /// [`FaceView::for_each_triangle`] hands out. Each triangle and the hub make
    /// a tetrahedron whose signed volume is a determinant; add them all up and
    /// the parts outside the solid cancel, because the face triangles form a
    /// closed surface. Faces used the wrong way round come out negative, which
    /// is why [`Orientation::sign`] has to be exactly ±1 — and it is.
    ///
    /// No quadrature. Planar faces do not need any (decisions.md §3.1).
    ///
    /// The triangles come in a fixed order, so the sums are bit for bit
    /// repeatable.
    ///
    /// [`FaceView::for_each_triangle`]: crate::FaceView::for_each_triangle
    /// [`Orientation::sign`]: rcad_geom::Orientation::sign
    ///
    /// # Panics
    ///
    /// If the solid encloses no space, or an infinite amount of it. A closed
    /// solid with its faces pointing outwards cannot, and
    /// [`Model::validate`](crate::Model::validate) says so, which makes this a
    /// broken model rather than bad input.
    #[must_use]
    pub fn measure(self) -> Measurement {
        let hub = self.hub();
        let mut six_volumes = 0.0;
        let mut moment = [0.0f64; 3];
        let mut surface_area = 0.0;

        for face in self.faces() {
            let mut vector_area = Vector::zero();
            face.for_each_triangle(|[a, b, c]| {
                let six_volume = triple_product(hub, [a, b, c]);
                six_volumes += six_volume;
                let corners = [a - hub, b - hub, c - hub].map(|corner| corner.components());
                for (axis, total) in moment.iter_mut().enumerate() {
                    *total += six_volume * (corners[0][axis] + corners[1][axis] + corners[2][axis]);
                }
                vector_area = vector_area + (b - a).cross(c - a) * 0.5;
            });
            surface_area += vector_area.length();
        }

        assert!(
            six_volumes.is_finite() && six_volumes > 0.0,
            "a solid must enclose a positive amount of space, this one gives {}",
            six_volumes / 6.0
        );

        Measurement {
            volume: six_volumes / 6.0,
            surface_area,
            centroid: hub + Vector::from(moment.map(|axis| axis / (4.0 * six_volumes))),
        }
    }

    /// The signed volume sum without the assertion, for
    /// [`Model::validate`](crate::Model::validate) to complain about.
    pub(crate) fn signed_volume(self) -> f64 {
        let hub = self.hub();
        let mut six_volumes = 0.0;
        for face in self.faces() {
            face.for_each_triangle(|[a, b, c]| {
                six_volumes += triple_product(hub, [a, b, c]);
            });
        }
        six_volumes / 6.0
    }

    /// The corner the divergence-theorem sums hang from: the one with the
    /// smallest coordinates, `x` first.
    ///
    /// **A corner of the solid rather than the world origin**, for the reason
    /// [`triple_product`] gives, and **the smallest one rather than the first
    /// one the arena hands over**, because the arena's order is not part of the
    /// answer. Two routes through a fuse can reach the same solid with its faces
    /// in different orders, and a measurement that depended on which face came
    /// first would come back different for a solid that is the same — the
    /// property `fuses_that_must_agree.rs` calls decomposition invariance.
    ///
    /// The origin when the solid has no triangles at all. Nothing is summed in
    /// that case and the callers refuse the answer anyway.
    fn hub(self) -> Point {
        let mut smallest: Option<[f64; 3]> = None;
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                for corner in corners {
                    let corner = corner.coordinates();
                    if smallest.is_none_or(|so_far| lexicographically_smaller(corner, so_far)) {
                        smallest = Some(corner);
                    }
                }
            });
        }
        Point::from(smallest.unwrap_or([0.0; 3]))
    }
}

/// Whether `corner` sorts before `so_far`, comparing `x`, then `y`, then `z`.
fn lexicographically_smaller(corner: [f64; 3], so_far: [f64; 3]) -> bool {
    corner
        .iter()
        .zip(&so_far)
        .find_map(|(mine, theirs)| match mine.total_cmp(theirs) {
            core::cmp::Ordering::Equal => None,
            order => Some(order.is_lt()),
        })
        .unwrap_or(false)
}

/// Six times the signed volume of the tetrahedron on `hub` and these three
/// corners.
///
/// **`hub` is a corner of the solid, not the world origin.** The sum is
/// the same in exact arithmetic whatever the apex is, and wildly different in
/// `f64`: hung from the origin, a solid `2^m` away adds up terms of size
/// `2^{3m}` to reach a volume of order one, so past about `m = 28` the spacing
/// of the terms is wider than the answer and what comes back has no correct
/// digits and can be negative. Hung from a corner of the solid, every term is
/// the size of the solid, and the three subtractions here are exact by
/// Sterbenz's lemma whenever the coordinates share an exponent — which is the
/// case that used to be worst. `docs/decisions.md` §16.11 is the question this
/// answers: a model far from the origin has a perfectly ordinary
/// `DynamicRange`, because a ratio cannot see where a solid *is*.
fn triple_product(hub: Point, [a, b, c]: [Point; 3]) -> f64 {
    let (a, b, c) = (a - hub, b - hub, c - hub);
    a.dot(b.cross(c))
}
