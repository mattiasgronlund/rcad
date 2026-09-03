use std::collections::HashMap;

use rcad_geom::{Cone, Deflection, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};

use crate::mesh::{Mesh, MeshTriangle};
use crate::view::FaceView;
use crate::{Error, FaceKey, Invalid, Result, SolidView};

/// How much space a solid takes up, how much skin it has, where it balances,
/// and how far each of those three could be out.
///
/// **Three answers in three different units, and each error is in the unit of
/// the answer it belongs to** — a cubed one, a squared one and a length
/// (`docs/decisions.md` §25.4). Cramming a volume error into an
/// [`Approximation`](rcad_predicates::Approximation), which is a length by
/// §15.2, is the units mistake §12.4 says this kernel loses money on.
///
/// All three errors are exactly `0.0` for a solid whose faces are all flat: the
/// divergence-theorem sum over a polyhedron's own triangles is the answer and
/// not a picture of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    volume: f64,
    volume_error: f64,
    surface_area: f64,
    area_error: f64,
    centroid: Point,
    centroid_wobble: f64,
}

impl Measurement {
    /// How much space the solid encloses.
    #[must_use]
    pub fn volume(self) -> f64 {
        self.volume
    }

    /// How much more space the solid could enclose than [`Measurement::volume`]
    /// says — a **cubed** quantity, in the model's own unit.
    ///
    /// The triangles a curved face is cut into are drawn through points on the
    /// surface itself, so they lie inside the material and the sum over them is
    /// short of the real answer. What is missing is the shell between the
    /// triangles and the surface, and
    /// [`how_much_space_hides_under_a_flat_stand_in`] bounds it.
    ///
    /// Exactly `0.0` for a wholly flat solid, and never negative.
    #[must_use]
    pub fn volume_error(self) -> f64 {
        self.volume_error
    }

    /// The total area of all its faces.
    #[must_use]
    pub fn surface_area(self) -> f64 {
        self.surface_area
    }

    /// How much more skin the solid could have than
    /// [`Measurement::surface_area`] says — a **squared** quantity, in the
    /// model's own unit.
    ///
    /// Two things are missing and both are counted:
    /// [`how_much_more_surface_there_can_be`] for the curved faces, whose true
    /// surface bulges away from the flat triangles standing in for it, and
    /// [`rim_of_the_flat_faces`] for the flat ones, whose meshed outline is a
    /// polygon inscribed in a curve.
    ///
    /// Exactly `0.0` for a wholly flat solid, and never negative.
    #[must_use]
    pub fn area_error(self) -> f64 {
        self.area_error
    }

    /// The point the solid would balance on, if it were of even density.
    #[must_use]
    pub fn centroid(self) -> Point {
        self.centroid
    }

    /// How far [`Measurement::centroid`] could be from where the solid really
    /// balances — a **length**, in the model's own unit.
    ///
    /// The one of the three that is a length, so the one that is comparable
    /// with [`Policy::max_wobble`] (§25.4). See [`SolidView::measure`] for what
    /// that comparison is worth and who makes it.
    ///
    /// The missing material is no more than [`Measurement::volume_error`] of
    /// space, and none of it is further from the centroid than the solid
    /// reaches, so moving all of it to the far side shifts the balance point by
    /// at most `volume_error / volume` of that reach.
    ///
    /// Exactly `0.0` for a wholly flat solid.
    #[must_use]
    pub fn centroid_wobble(self) -> f64 {
        self.centroid_wobble
    }
}

impl SolidView<'_> {
    /// Measure the solid: volume, surface area and centroid in one pass, each
    /// with how far out it could be.
    ///
    /// Straight out of the divergence theorem. Each triangle and the hub make a
    /// tetrahedron whose signed volume is a determinant; add them all up and the
    /// parts outside the solid cancel, because the triangles form a closed
    /// surface. Faces used the wrong way round come out negative, which is why
    /// [`Orientation::sign`] has to be exactly ±1 — and it is.
    ///
    /// # Two paths, and a polyhedron still takes the exact one
    ///
    /// A solid whose faces are **all flat** is measured over the triangles
    /// [`FaceView::for_each_triangle`] hands out, exactly as it was in M0:
    /// `deflection` is ignored, no mesh is built, nothing is logged, and all
    /// three error terms come back exactly `0.0`. That is not an optimisation.
    /// Every metamorphic property in the fuse suite — inclusion–exclusion,
    /// idempotence, absorption — reads these numbers, and putting them
    /// downstream of a triangulator is the schedule risk §23.7 named before this
    /// work started. `a_flat_solids_measurement_is_bit_for_bit_what_it_was` is
    /// the test that says it did not happen.
    ///
    /// A solid with a curved face has no such fan — its caps fan into no
    /// triangles at all and its wall into two with no area — so it is measured
    /// over [`SolidView::mesh`], and the three error terms say how much of the
    /// answer the triangles could be hiding.
    ///
    /// # Which knob decides what
    ///
    /// `deflection` decides how fine the mesh is, and a coarse one is a request
    /// rather than a failure (§23.5). `policy` decides the one thing §23.5 gives
    /// it: whether a deflection under what the arithmetic can reach is refused
    /// or tagged, which is [`SolidView::mesh`]'s business and is why both travel
    /// through to it.
    ///
    /// **What `policy` deliberately does not do is refuse a measurement for
    /// being meshed.** [`Measurement::centroid_wobble`] is the one term
    /// comparable with [`Policy::max_wobble`], and that budget is 64 roundings
    /// at the working size — around `1e-12` for a 100 mm part. The finest mesh
    /// this kernel will make of a ball leaves a wobble many orders above it, so
    /// judging one against the other would refuse *every* curved solid at
    /// *every* deflection, and this signature would have replaced one blanket
    /// refusal with another. It would also let the deflection decide what the
    /// policy certifies, which is the conflation §23.5 exists to stop. The
    /// comparison is the caller's to make, on a number that is handed back for
    /// exactly that;
    /// `judging_a_meshed_centroid_against_the_policy_would_refuse_every_ball` is
    /// the demonstration rather than the paragraph.
    ///
    /// [`Orientation::sign`]: rcad_geom::Orientation::sign
    /// [`FaceView::for_each_triangle`]: crate::FaceView::for_each_triangle
    ///
    /// # Errors
    ///
    /// [`Error::VolumeTooLargeToMeasure`] or
    /// [`Error::SurfaceAreaTooLargeToMeasure`] when that answer is a real
    /// number the solid genuinely has and `f64` has no room for — a cube of
    /// side `1e300` encloses `1e900`. The solid is sound; the type is the
    /// limit, so this is bad input rather than a broken model (§4.1).
    ///
    /// For a curved solid, whatever [`SolidView::mesh`] refuses:
    /// [`Error::DeflectionBelowWhatAMeshCanReach`],
    /// [`Error::MoreMeshPointsThanASolidWillHold`], [`Error::CannotMeshAFace`]
    /// or [`Error::Undecidable`].
    ///
    /// [`Invalid::VolumeIsNotPositive`] when the solid encloses a negative
    /// amount of space, or none — the same refusal
    /// [`Model::validate`](crate::Model::validate) gives for the same bytes,
    /// with the same number in it.
    ///
    /// **This used to be an assert, and §20.1 settled it that way.** The
    /// argument then was that the only route to an inside-out solid was a bug
    /// in this crate, so a crash beat a corruption. `Model` is `Deserialize`,
    /// which is a second route: a stranger's bytes can carry a solid whose
    /// faces point inwards, `validate` already answers `Err` for them, and a
    /// caller's bytes may not panic. `measuring_a_solid_read_back_inside_out_is_refused_rather_than_crashing`
    /// is the demonstration.
    ///
    /// # Panics
    ///
    /// For a curved solid, everything [`SolidView::mesh`] panics on.
    pub fn measure(
        self,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Measurement> {
        if self.is_wholly_flat() {
            return self.measured_over_its_own_fan();
        }
        let mesh = self.mesh(deflection, policy, log)?;
        self.measured_over(&mesh)
    }

    /// The exact answer for a polyhedron, summed over the triangles its faces
    /// fan into.
    ///
    /// No quadrature. Planar faces do not need any (decisions.md §3.1). The
    /// triangles come in a fixed order, so the sums are bit for bit repeatable.
    fn measured_over_its_own_fan(self) -> Result<Measurement> {
        let hub = self.hub();
        let shifts = self.axis_shifts(hub);
        let (six_volumes, moment) = self.volume_and_moment(hub, shifts);

        let volume = shift(six_volumes / 6.0, shifts.iter().sum());
        if !volume.is_finite() {
            return Err(Error::VolumeTooLargeToMeasure);
        }
        if volume <= 0.0 {
            return Err(Invalid::VolumeIsNotPositive { volume }.into());
        }
        let surface_area = self.surface_area();
        if !surface_area.is_finite() {
            return Err(Error::SurfaceAreaTooLargeToMeasure);
        }

        let balance = [0, 1, 2].map(|axis| shift(moment[axis] / (4.0 * six_volumes), shifts[axis]));
        Ok(Measurement {
            volume,
            volume_error: 0.0,
            surface_area,
            area_error: 0.0,
            centroid: hub + Vector::from(balance),
            centroid_wobble: 0.0,
        })
    }

    /// The same three sums over a mesh, plus how much of the solid the mesh
    /// leaves out.
    ///
    /// The arithmetic is the fan's: hang the sums from a corner rather than the
    /// origin, shrink each axis by a power of two, and undo both on the way out.
    /// [`SolidView::volume_and_moment`] is where that is derived, and every word
    /// of it applies here — the triangles are a different set, not a different
    /// kind of thing.
    fn measured_over(self, mesh: &Mesh) -> Result<Measurement> {
        let hub = hub_of(mesh);
        let shifts = axis_shifts_over(mesh, hub);
        let (six_volumes, moment) = volume_and_moment_over(mesh, hub, shifts);

        let volume = shift(six_volumes / 6.0, shifts.iter().sum());
        if !volume.is_finite() {
            return Err(Error::VolumeTooLargeToMeasure);
        }
        if volume <= 0.0 {
            return Err(Invalid::VolumeIsNotPositive { volume }.into());
        }
        let skin = self.skin_of(mesh);
        if !skin.area.is_finite() {
            return Err(Error::SurfaceAreaTooLargeToMeasure);
        }

        let balance = [0, 1, 2].map(|axis| shift(moment[axis] / (4.0 * six_volumes), shifts[axis]));
        let centroid = hub + Vector::from(balance);
        Ok(Measurement {
            volume,
            volume_error: skin.volume_error,
            surface_area: skin.area,
            area_error: skin.area_error,
            centroid,
            centroid_wobble: (skin.volume_error / volume)
                * how_far_the_solid_reaches_from(mesh, centroid),
        })
    }

    /// How much skin the mesh has, and how much of the solid's own it misses.
    ///
    /// One walk over the faces rather than three, because the two error terms
    /// are both a share of the very area being summed.
    fn skin_of(self, mesh: &Mesh) -> Skin {
        let radii = radius_of_each_face(self, mesh);
        let deviation = mesh.deviation().magnitude();
        let mut skin = Skin {
            area: 0.0,
            area_error: 0.0,
            volume_error: 0.0,
        };
        for (face, area) in areas_of_each_face(mesh) {
            skin.area += area;
            let Some(radius) = radii[&face] else {
                continue;
            };
            skin.area_error += how_much_more_surface_there_can_be(area, radius, deviation);
            skin.volume_error +=
                how_much_space_hides_under_a_flat_stand_in(area, radius, deviation);
        }
        skin.area_error += deviation * rim_of_the_flat_faces(mesh, &radii);
        skin
    }

    /// The signed volume without the refusals, for
    /// [`Model::validate`](crate::Model::validate) to complain about. Infinite
    /// when the solid is too large to measure, which is a different complaint
    /// from a negative one and `validate` keeps the two apart.
    ///
    /// The fan, so this is for a wholly flat solid. A curved one has no fan and
    /// `validate` reads [`signed_volume_over`] instead.
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

/// How much skin a mesh has, and how much of its solid's own it leaves out.
struct Skin {
    area: f64,
    area_error: f64,
    volume_error: f64,
}

/// The corner the sums over a mesh hang from, picked exactly as
/// [`SolidView::hub`] picks one: the smallest, `x` first.
fn hub_of(mesh: &Mesh) -> Point {
    let mut smallest: Option<[f64; 3]> = None;
    for point in mesh.points() {
        let corner = point.coordinates();
        if smallest.is_none_or(|so_far| lexicographically_smaller(corner, so_far)) {
            smallest = Some(corner);
        }
    }
    Point::from(smallest.unwrap_or([0.0; 3]))
}

/// One power of two per axis over a mesh's points, for
/// [`SolidView::axis_shifts`]'s reasons.
fn axis_shifts_over(mesh: &Mesh, hub: Point) -> [i32; 3] {
    let mut widest = [0.0f64; 3];
    for point in mesh.points() {
        let offsets = (*point - hub).components();
        for (axis, so_far) in widest.iter_mut().enumerate() {
            *so_far = so_far.max(offsets[axis].abs());
        }
    }
    widest.map(shift_for)
}

/// Six times the signed volume of a mesh, and its first moment about `hub`.
///
/// The mesh's triangles in the order it stores them, which is fixed (§25.7), so
/// two runs give the same sum bit for bit.
fn volume_and_moment_over(mesh: &Mesh, hub: Point, shifts: [i32; 3]) -> (f64, [f64; 3]) {
    let mut six_volumes = 0.0;
    let mut moment = [0.0f64; 3];
    for triangle in mesh.triangles() {
        let [a, b, c] =
            corners_of(mesh, triangle).map(|corner| shrink_each_axis(corner - hub, shifts));
        let six_volume = a.dot(b.cross(c));
        six_volumes += six_volume;
        let offsets = [a, b, c].map(|offset| offset.components());
        for (axis, total) in moment.iter_mut().enumerate() {
            *total += six_volume * (offsets[0][axis] + offsets[1][axis] + offsets[2][axis]);
        }
    }
    (six_volumes, moment)
}

/// The signed volume of a mesh, for
/// [`Model::validate`](crate::Model::validate) to complain about — the curved
/// twin of [`SolidView::signed_volume`].
pub(crate) fn signed_volume_over(mesh: &Mesh) -> f64 {
    let hub = hub_of(mesh);
    let shifts = axis_shifts_over(mesh, hub);
    let (six_volumes, _) = volume_and_moment_over(mesh, hub, shifts);
    shift(six_volumes / 6.0, shifts.iter().sum())
}

/// Each face of the mesh and how much area its triangles have, in the order the
/// mesh stores them.
///
/// A face's triangles arrive together, so this is a walk and not a lookup.
fn areas_of_each_face(mesh: &Mesh) -> Vec<(FaceKey, f64)> {
    let triangles = mesh.triangles();
    let mut areas = Vec::new();
    let mut start = 0;
    while start < triangles.len() {
        let face = triangles[start].face();
        let mut end = start;
        while end < triangles.len() && triangles[end].face() == face {
            end += 1;
        }
        areas.push((face, area_of(mesh, &triangles[start..end])));
        start = end;
    }
    areas
}

/// The area of one face's triangles, measured in a scale of its own for the
/// reason [`SolidView::surface_area`] gives.
///
/// **Every triangle's area added, not the length of their vector sum.** A flat
/// face's triangle normals all point the same way and the two agree there; a
/// cylinder's wall goes right round and its vector areas cancel to nothing, so
/// the shorter sum would say a bar has no wall.
fn area_of(mesh: &Mesh, triangles: &[MeshTriangle]) -> f64 {
    let mut widest = 0.0f64;
    for triangle in triangles {
        let [a, b, c] = corners_of(mesh, triangle);
        for edge in [b - a, c - a] {
            for component in edge.components() {
                widest = widest.max(component.abs());
            }
        }
    }
    let across = shift_for(widest);

    let mut total = 0.0;
    for triangle in triangles {
        let [a, b, c] = corners_of(mesh, triangle);
        let along = shrink(b - a, across);
        let down = shrink(c - a, across);
        total += along.cross(down).length() * 0.5;
    }
    shift(total, 2 * across)
}

/// How much more surface a curved face can have than the `area` of the flat
/// triangles standing in for it.
///
/// **The derivation, for a face on a sphere of radius `r` cut to a deviation of
/// `d`.** Every point of every triangle is inside the material and no further
/// than `d` from the surface, so all of them sit between the radii `r − d` and
/// `r`. Projecting the triangles radially in onto the sphere of radius `r − d`
/// scales an area element by `cos θ · ((r − d) / ρ)²`, where `ρ ≥ r − d` is how
/// far out the element is and `θ` is how far its normal leans off the radial
/// direction — never more than one — so the solid angle the face covers is at
/// most `area / (r − d)²`. The true surface over that solid angle has area
/// `r²` times it, so it is at most `(r / (r − d))²` of `area`, and this is the
/// difference.
///
/// **A cylinder is covered by the same number and is not derived twice.** The
/// same argument about the axis gives `r / (r − d)`, which is the smaller of the
/// two for every `r > d`, so the sphere's factor bounds both. One expression
/// that is loose for pipes beats two that can disagree.
///
/// Infinite when the mesh may be a whole radius away from the surface: there is
/// then no shell to trap the difference in and nothing here bounds it. A
/// deflection that coarse is a legitimate request (§23.5) and this is what it
/// costs.
fn how_much_more_surface_there_can_be(area: f64, radius: f64, deviation: f64) -> f64 {
    let inner = radius - deviation;
    if inner <= 0.0 {
        return f64::INFINITY;
    }
    area * ((radius / inner) * (radius / inner) - 1.0)
}

/// How much space can hide between a curved face's triangles and the surface
/// they stand in for, over the `area` of those triangles.
///
/// The same shell as [`how_much_more_surface_there_can_be`], now measured rather
/// than projected. The face covers a solid angle of at most `area / (r − d)²`,
/// and the shell between the radii `r − d` and `r` over that angle holds
/// `(r³ − (r − d)³) / 3` per unit of it.
///
/// **Nothing is charged to a flat face**, and that is a claim rather than an
/// omission: a flat face is meshed in its own plane, so no space at all is
/// enclosed between the two skins there. The sliver over a flat face's *rim* —
/// between its inscribed outline and the curve that outline stands for — is
/// inside the neighbouring curved face's shell and already counted — which the
/// short wide bar in
/// `the_published_error_bounds_cover_what_a_mesh_of_a_bar_or_a_ball_misses`
/// checks against `πr²h`.
///
/// Infinite for a mesh a whole radius out, for the reason above.
fn how_much_space_hides_under_a_flat_stand_in(area: f64, radius: f64, deviation: f64) -> f64 {
    let inner = radius - deviation;
    if inner <= 0.0 {
        return f64::INFINITY;
    }
    area * (radius * radius * radius - inner * inner * inner) / (3.0 * inner * inner)
}

/// The total length of the mesh sides where a flat face meets another face.
///
/// **A flat face of a curved solid is not exact either, and this is why.** A
/// round bar's cap is a disc, and what the mesh draws is a polygon inscribed in
/// its rim — short of the disc by one circular segment per side. Each segment
/// fits inside a rectangle as long as its chord and as tall as the deviation, so
/// `deviation` times this length bounds the lot.
///
/// Counted once for each flat face along a side, which is what makes a bar's two
/// caps cost twice one rim. A side used twice by the *same* face is a seam and
/// is not a rim at all, so it is skipped by construction.
fn rim_of_the_flat_faces(mesh: &Mesh, radii: &HashMap<FaceKey, Option<f64>>) -> f64 {
    let mut first_seen: HashMap<(u32, u32), FaceKey> = HashMap::new();
    let mut rim = 0.0;
    for triangle in mesh.triangles() {
        let [first, second, third] = triangle.corners();
        for (from, to) in [(first, second), (second, third), (third, first)] {
            let side = (from.min(to), from.max(to));
            let Some(&already) = first_seen.get(&side) else {
                first_seen.insert(side, triangle.face());
                continue;
            };
            if already == triangle.face() {
                continue;
            }
            let length = mesh.points()[from as usize].distance_to(mesh.points()[to as usize]);
            for face in [already, triangle.face()] {
                if radii[&face].is_none() {
                    rim += length;
                }
            }
        }
    }
    rim
}

/// How far out of `centroid` any part of the solid can be: the furthest mesh
/// point, and the deviation on top, because the surface stands off the mesh by
/// that much again.
fn how_far_the_solid_reaches_from(mesh: &Mesh, centroid: Point) -> f64 {
    let furthest = mesh
        .points()
        .iter()
        .map(|point| point.distance_to(centroid))
        .fold(0.0, f64::max);
    furthest + mesh.deviation().magnitude()
}

/// How far each face of this solid bends, or `None` where it does not bend at
/// all.
///
/// # How small a cone face's radius really gets, and why the corners cannot say
///
/// **Read over the mesh's own points and not over the face's corners**, and the
/// case that forces it is a cone that ends in a point. Its cone face is bounded
/// by one circle and nothing else; the apex is where the surface's own second
/// coordinate runs out, not a vertex of the model, so **no corner of that face
/// is anywhere near it**. Charged against its corners the face would publish the
/// radius at its rim — four units on a cone of radius four — for a face that
/// reaches a radius of nothing at all. That is a bound claimed and not earned,
/// which is worse than no bound.
///
/// The mesh's points are the right domain anyway: what the two bounds above are
/// about is how far *these triangles* can be from the surface, and the triangles
/// are what the mesh holds.
///
/// **The offset-surface argument is what needs the smallest and not the
/// largest.** [`how_much_more_surface_there_can_be`] projects the mesh onto the
/// surface `deviation` inside the material and reads the area back out; for a
/// cone that inner surface is *the same cone shifted along its own axis*, so the
/// factor is the pipe's `R/(R − d)` with `R = slope·|v|·√(1 + slope²)` the radius
/// of normal curvature — and the sphere's `(R/(R − d))²` bounds that, exactly as
/// it bounds a pipe's. `R` varies over the face, so the smallest of it is the
/// one that bounds the lot.
///
/// **A face that reaches its apex gets zero, and the bounds then come back
/// infinite.** That is honest and it is a **gap**: within `deviation` of the
/// apex there is no inner offset cone at all — its own apex has moved
/// `deviation·√(1 + slope²)/slope` up the axis — so no argument of this shape
/// covers the tip, and the number this hands back must not pretend otherwise.
/// What closes it is splitting the face at `v = 2·deviation·√(1 + slope²)/slope`
/// and charging the tip its **whole** size rather than a share of it: the cone
/// above that split has `R ≥ 2·deviation`, and the tip below it holds at most
/// `4π·deviation²(1 + slope²)^{3/2}/slope` of area and
/// `(8π/3)·deviation³(1 + slope²)^{3/2}/slope` of volume, both of which are
/// bounded outright. That wants a bound taking two numbers rather than one, and
/// its consumer is a pointed cone somebody wants a measured volume for.
///
/// # Panics
///
/// On a surface this has not been taught. [`Surface`] grows a variant at every
/// milestone, and a surface measured by not being measured is worse than a stop.
fn radius_of_each_face(solid: SolidView<'_>, mesh: &Mesh) -> HashMap<FaceKey, Option<f64>> {
    let mut cones: HashMap<FaceKey, Cone> = HashMap::new();
    let mut radii: HashMap<FaceKey, Option<f64>> = solid
        .faces()
        .map(|face| {
            let radius = match face.surface() {
                Surface::Plane(_) => None,
                Surface::Sphere(ball) => Some(ball.radius()),
                Surface::Cylinder(pipe) => Some(pipe.radius()),
                // Worked out below, over the mesh's own points rather than over
                // the face's corners. A face with no triangles keeps the
                // infinity, and contributes no area either, so it is charged
                // nothing — which is right, because there is nothing there.
                Surface::Cone(cone) => {
                    cones.insert(face.key(), cone);
                    Some(f64::INFINITY)
                }
                // **The tube radius, and it needs no mesh to find.** The inner
                // offset surface of a torus is the same torus with a thinner
                // tube, which exists for every `d` below `r` — so unlike a
                // cone, a torus never runs out of an inner surface to project
                // onto, and unlike a ball its ratio is not one number. The area
                // element is `(R + r·cos v)·r`, so the offset's is
                // `(R + (r − d)cos v)(r − d)` and the ratio is
                // `[r/(r − d)]·[(R + r·cos v)/(R + (r − d)cos v)]`. The second
                // factor is largest at the outer equator, where it is
                // `(R + r)/(R + r − d)` — smaller than `r/(r − d)` because
                // `R + r` is bigger than `r` — so the whole ratio is under
                // `(r/(r − d))²`, which is the sphere's factor with `r` in it.
                Surface::Torus(ring) => Some(ring.tube_radius()),
                other => panic!("this crate cannot say how far {other:?} bends"),
            };
            (face.key(), radius)
        })
        .collect();

    for triangle in mesh.triangles() {
        let Some(cone) = cones.get(&triangle.face()) else {
            continue;
        };
        let along = cone.axis().direction();
        let steepness = cone.slope().hypot(1.0);
        for corner in triangle.corners() {
            let at = mesh.points()[corner as usize];
            let here = cone.radius_at(along.dot(at - cone.apex())) * steepness;
            let slot = radii
                .get_mut(&triangle.face())
                .expect("every face of the solid is in the table");
            *slot = Some(
                slot.expect("a cone face was put in with a radius")
                    .min(here),
            );
        }
    }
    radii
}

/// Where one mesh triangle's three corners are.
fn corners_of(mesh: &Mesh, triangle: &MeshTriangle) -> [Point; 3] {
    triangle.corners().map(|at| mesh.points()[at as usize])
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
            core::cmp::Ordering::Less => Some(true),
            core::cmp::Ordering::Greater => Some(false),
        })
        .unwrap_or(false)
}
