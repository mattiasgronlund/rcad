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
    /// [`FaceView::for_each_triangle`] hands out. Each triangle and the origin
    /// make a tetrahedron whose signed volume is a determinant; add them all up
    /// and the parts outside the solid cancel, because the face triangles form
    /// a closed surface. Faces used the wrong way round come out negative,
    /// which is why [`Orientation::sign`] has to be exactly ±1 — and it is.
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
        let mut six_volumes = 0.0;
        let mut moment = [0.0f64; 3];
        let mut surface_area = 0.0;

        for face in self.faces() {
            let mut vector_area = Vector::zero();
            face.for_each_triangle(|[a, b, c]| {
                let six_volume = triple_product(a, b, c);
                six_volumes += six_volume;
                let corners = [a.coordinates(), b.coordinates(), c.coordinates()];
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
            centroid: Point::from(moment.map(|axis| axis / (4.0 * six_volumes))),
        }
    }

    /// The signed volume sum without the assertion, for
    /// [`Model::validate`](crate::Model::validate) to complain about.
    pub(crate) fn signed_volume(self) -> f64 {
        let mut six_volumes = 0.0;
        for face in self.faces() {
            face.for_each_triangle(|[a, b, c]| {
                six_volumes += triple_product(a, b, c);
            });
        }
        six_volumes / 6.0
    }
}

/// `a · (b × c)`: six times the signed volume of the tetrahedron on the origin
/// and these three corners.
fn triple_product(a: Point, b: Point, c: Point) -> f64 {
    let (a, b, c) = (
        a - Point::origin(),
        b - Point::origin(),
        c - Point::origin(),
    );
    a.dot(b.cross(c))
}
