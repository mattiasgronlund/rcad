use rcad_geom::{Point, Vector};

use crate::view::FaceView;
use crate::{Error, Result, SolidView};

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
    /// # Errors
    ///
    /// [`Error::MeasureNeedsFlatFaces`] for a solid with a curved face. The fan
    /// is a polyhedron's answer and a round bar is not a polyhedron — asked
    /// anyway it sums to zero, which is not a small error but no answer at all.
    /// Asked over **every** face before anything is summed, for the reason
    /// [`SolidView::is_wholly_flat`] gives: which answer comes back must not
    /// depend on the order the faces were reached.
    ///
    /// [`Error::VolumeTooLargeToMeasure`] or
    /// [`Error::SurfaceAreaTooLargeToMeasure`] when that answer is a real
    /// number the solid genuinely has and `f64` has no room for — a cube of
    /// side `1e300` encloses `1e900`. The solid is sound; the type is the
    /// limit, so this is bad input rather than a broken model (§4.1).
    ///
    /// # Panics
    ///
    /// If the solid encloses a negative amount of space, or none. A closed
    /// solid with its faces pointing outwards cannot, and
    /// [`Model::validate`](crate::Model::validate) says so, which makes this a
    /// broken model rather than bad input.
    pub fn measure(self) -> Result<Measurement> {
        if !self.is_wholly_flat() {
            return Err(Error::MeasureNeedsFlatFaces);
        }
        let hub = self.hub();
        let shifts = self.axis_shifts(hub);
        let (six_volumes, moment) = self.volume_and_moment(hub, shifts);

        assert!(
            six_volumes > 0.0,
            "a solid must enclose a positive amount of space, this one gives {}",
            six_volumes / 6.0
        );

        let volume = shift(six_volumes / 6.0, shifts.iter().sum());
        if !volume.is_finite() {
            return Err(Error::VolumeTooLargeToMeasure);
        }
        let surface_area = self.surface_area();
        if !surface_area.is_finite() {
            return Err(Error::SurfaceAreaTooLargeToMeasure);
        }

        let balance = [0, 1, 2].map(|axis| shift(moment[axis] / (4.0 * six_volumes), shifts[axis]));
        Ok(Measurement {
            volume,
            surface_area,
            centroid: hub + Vector::from(balance),
        })
    }

    /// The signed volume without the refusals, for
    /// [`Model::validate`](crate::Model::validate) to complain about. Infinite
    /// when the solid is too large to measure, which is a different complaint
    /// from a negative one and `validate` keeps the two apart.
    pub(crate) fn signed_volume(self) -> f64 {
        let hub = self.hub();
        let shifts = self.axis_shifts(hub);
        let (six_volumes, _) = self.volume_and_moment(hub, shifts);
        shift(six_volumes / 6.0, shifts.iter().sum())
    }

    /// Six times the signed volume, and the first moment about the hub, both in
    /// the units `shifts` picked rather than the model's.
    ///
    /// **The offsets are from a corner of the solid, not the world origin.**
    /// The sum is the same in exact arithmetic whatever the apex is, and wildly
    /// different in `f64`: hung from the origin, a solid `2^m` away adds up
    /// terms of size `2^{3m}` to reach a volume of order one, so past about
    /// `m = 28` the spacing of the terms is wider than the answer and what
    /// comes back has no correct digits and can be negative. Hung from a corner
    /// of the solid, every term is the size of the solid, and the three
    /// subtractions are exact by Sterbenz's lemma whenever the coordinates
    /// share an exponent — which is the case that used to be worst. A rescale
    /// does not stand in for this: it moves where the terms sit and cannot put
    /// back digits that cancellation has already taken.
    ///
    /// Every offset is then between one and two, so no triple product here can
    /// overflow or fall subnormal whatever the solid's shape. The overflow that
    /// used to happen inside this loop now happens in [`shift`] on the way out,
    /// where it means the answer does not fit rather than that the sum went
    /// wrong.
    fn volume_and_moment(self, hub: Point, shifts: [i32; 3]) -> (f64, [f64; 3]) {
        let mut six_volumes = 0.0;
        let mut moment = [0.0f64; 3];
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                let [a, b, c] = corners.map(|corner| shrink_each_axis(corner - hub, shifts));
                let six_volume = a.dot(b.cross(c));
                six_volumes += six_volume;
                let offsets = [a, b, c].map(|offset| offset.components());
                for (axis, total) in moment.iter_mut().enumerate() {
                    *total += six_volume * (offsets[0][axis] + offsets[1][axis] + offsets[2][axis]);
                }
            });
        }
        (six_volumes, moment)
    }

    /// The total area of the faces, each measured in a scale of its own.
    ///
    /// **A different scaling from the one the volume uses, because a different
    /// thing is being measured.** A determinant scales by the product of its
    /// three column scales, so the volume can afford one power of two per axis;
    /// a length does not, because `x² + y² + z²` only comes apart if every
    /// component was scaled the same way. So each face is measured whole, in a
    /// single power of two.
    ///
    /// **Per face rather than per solid**, because the alternative loses small
    /// faces. A tower one unit across and `1e200` tall scaled to its own height
    /// would push its top and bottom to `1e-400`, which is zero, and the two
    /// units of area they carry would vanish. Measured in its own scale, each
    /// face keeps every bit it had.
    fn surface_area(self) -> f64 {
        let mut total = 0.0;
        for face in self.faces() {
            let across = shift_for(largest_edge_component(&face));
            let mut vector_area = Vector::zero();
            face.for_each_triangle(|[a, b, c]| {
                let along = shrink(b - a, across);
                let down = shrink(c - a, across);
                vector_area = vector_area + along.cross(down) * 0.5;
            });
            total += shift(vector_area.length(), 2 * across);
        }
        total
    }

    /// One power of two per axis, so the widest corner offset along each lands
    /// between one and two.
    ///
    /// **Exact powers of two, which is what makes this free.** Scaling an axis
    /// scales every term of the determinant that uses it by the same amount,
    /// and floating-point addition commutes with an exact scaling, so the sums
    /// come back bit for bit what they would have been — and so does what
    /// [`shift`] undoes them to. `docs/decisions.md` §18: a rescale is a
    /// technique and not only an oracle.
    ///
    /// **One per axis rather than one for the solid**, because a solid need not
    /// be the same size in all three directions. A tower one unit across and
    /// `1e200` tall, scaled by its height, would have its two short axes land
    /// at `2^-664`, and their product with the third is `2^-1328` — zero, and a
    /// volume of nothing for a tower that plainly holds some.
    fn axis_shifts(self, hub: Point) -> [i32; 3] {
        let mut widest = [0.0f64; 3];
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                for corner in corners {
                    let offsets = (corner - hub).components();
                    for (axis, so_far) in widest.iter_mut().enumerate() {
                        *so_far = so_far.max(offsets[axis].abs());
                    }
                }
            });
        }
        widest.map(shift_for)
    }

    /// The corner the divergence-theorem sums hang from: the one with the
    /// smallest coordinates, `x` first.
    ///
    /// **A corner of the solid rather than the world origin**, for the reason
    /// [`Self::volume_and_moment`] gives, and **the smallest one rather than
    /// the first one the arena hands over**, because the arena's order is not
    /// part of the answer. Two routes through a fuse can reach the same solid
    /// with its faces in different orders, and a measurement that depended on
    /// which face came first would come back different for a solid that is the
    /// same — the property `fuses_that_must_agree.rs` calls decomposition
    /// invariance.
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

/// The widest component of any edge of any of `face`'s triangles.
///
/// The face's own extent, not its distance from the hub. A small face a long
/// way out has small edges, and scaling it by how far away it is would flush
/// its area to nothing.
fn largest_edge_component(face: &FaceView<'_>) -> f64 {
    let mut widest = 0.0f64;
    face.for_each_triangle(|[a, b, c]| {
        for edge in [b - a, c - a] {
            for component in edge.components() {
                widest = widest.max(component.abs());
            }
        }
    });
    widest
}

/// The exponent that brings `value` to between one and two, or `0` when there
/// is no exponent to read: `value` is zero, subnormal, or not a number.
///
/// A solid whose extent along an axis is subnormal is §19.6's low-end limit and
/// scaling does not reach it.
fn shift_for(value: f64) -> i32 {
    if !value.is_normal() {
        return 0;
    }
    i32::try_from(value.to_bits() >> 52 & 0x7ff).expect("eleven bits") - 1023
}

/// `offset` with each axis shrunk by its own power of two.
fn shrink_each_axis(offset: Vector, shifts: [i32; 3]) -> Vector {
    let components = offset.components();
    Vector::from([0, 1, 2].map(|axis| shift(components[axis], -shifts[axis])))
}

/// `offset` with every axis shrunk by the same power of two.
fn shrink(offset: Vector, by: i32) -> Vector {
    let components = offset.components();
    Vector::from(components.map(|component| shift(component, -by)))
}

/// `value * 2^exponent`, in steps small enough to stay in range.
///
/// **In steps, because `2^exponent` need not be representable when the answer
/// is.** A slab `2^350` across and `2^300` thick encloses `2^1000`, which fits,
/// while the `2^1050` that would scale it back in one multiplication does not.
/// Every step here moves the same way, so the running value walks towards the
/// answer and never past it, which leaves overflow meaning what it should:
/// `f64` has no room for the number.
fn shift(value: f64, exponent: i32) -> f64 {
    let mut shifted = value;
    let mut left = exponent;
    while left != 0 {
        let step = left.clamp(-512, 512);
        shifted *= two_to_the(step);
        left -= step;
    }
    shifted
}

/// `2^exponent`, exactly, for an exponent an `f64` can hold.
fn two_to_the(exponent: i32) -> f64 {
    f64::from_bits(u64::try_from(1023 + exponent).expect("a normal exponent") << 52)
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
