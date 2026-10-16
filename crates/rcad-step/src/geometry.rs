//! What a file's geometric entities are, in the kernel's own types.
//!
//! Nothing here builds topology and nothing here converts a unit. A number in
//! the file is the number in the model: §42.8 bakes world coordinates and §3.5
//! says the declared unit never takes part in arithmetic, so the only rounding
//! anything below does is the rounding a constructor does on its way in, and
//! each place that rounds says so.

use core::cell::Cell;
use core::f64::consts::FRAC_PI_2;

use rcad_geom::{Axis, Cone, Cylinder, Plane, Point, Sphere, Surface, Torus, Vector};

use crate::graph::{EntityId, Record, StepFile, Value};
use crate::names::{
    AXIS2_PLACEMENT_3D, CARTESIAN_POINT, CIRCLE, CONICAL_SURFACE, CYLINDRICAL_SURFACE, DIRECTION,
    EnumerationName, FALSE, Keyword, PLANE, SPHERICAL_SURFACE, TOROIDAL_SURFACE, TRUE,
};
use crate::refusal::Refusal;
use crate::token::LastDigit;

/// One walk over one file, and what it had to read loosely to get through.
///
/// The counter is the whole reason this is a struct rather than a handful of
/// free functions: `1` written where Part 21 wants `1.` is read as the real it
/// names, and a reader that does that quietly is a reader that heals. This
/// counts every one and [`Import`](crate::Import) publishes the count.
#[derive(Debug)]
pub(crate) struct Reading<'a> {
    file: &'a StepFile,
    radians_per_angle_unit: Cell<Option<f64>>,
    integers_read_as_reals: Cell<usize>,
}

impl<'a> Reading<'a> {
    pub(crate) fn of(file: &'a StepFile) -> Self {
        Self {
            file,
            radians_per_angle_unit: Cell::new(None),
            integers_read_as_reals: Cell::new(0),
        }
    }

    /// How many radians one of the file's plane-angle units is, once
    /// [`what_the_file_declared`](crate::units::what_the_file_declared) has
    /// read it. A `CONICAL_SURFACE` is the only thing that asks.
    pub(crate) fn angles_are_measured_in(&self, radians: Option<f64>) {
        self.radians_per_angle_unit.set(radians);
    }

    pub(crate) fn file(&self) -> &'a StepFile {
        self.file
    }

    pub(crate) fn integers_read_as_reals(&self) -> usize {
        self.integers_read_as_reals.get()
    }

    /// The surface a face sits on.
    ///
    /// Five kinds, and every other kind the file may hold is refused by the
    /// name the file gave it — which is §42.12 criterion 4 and, by the spike's
    /// count, the commonest thing this reader says.
    ///
    /// **A pipe's, a cone's and a ring's axis keep the direction the file
    /// wrote** when it is of length one to within what a turn leaves
    /// ([`Axis::as_written`]), as a plane's normal does ([`plane_through`]).
    /// This writer writes a seam's plane and the ring it cuts along one
    /// direction, and scaling the ring's axis again while keeping the plane's
    /// normal made them two directions an ulp apart: of 372 turned bends, 28
    /// were refused and 48 read back with the axis moved
    /// (`docs/decisions.md` §355.6).
    ///
    /// **A cone's slope may come from `corners`**, the face's corners as the
    /// file wrote them, which are read only for a cone whose half-angle the
    /// file may have cut ([`cone_from`]).
    pub(crate) fn surface_at(
        &self,
        face: EntityId,
        id: EntityId,
        corners: impl FnOnce() -> Option<Vec<Point>>,
    ) -> Result<Surface, Refusal> {
        let instance = self.instance(face, id)?;
        let keyword = instance.parts()[0].keyword();
        match Keyword::of(keyword) {
            Some(PLANE) => {
                let position = self.reference_at(id, PLANE, 1)?;
                let place = self.placement_at(id, position)?;
                let plane = plane_through(place.origin, place.axis)
                    .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
                Ok(Surface::Plane(plane))
            }
            Some(CYLINDRICAL_SURFACE) => Ok(Surface::Cylinder(self.pipe_at(id)?)),
            Some(SPHERICAL_SURFACE) => {
                let position = self.reference_at(id, SPHERICAL_SURFACE, 1)?;
                let place = self.placement_at(id, position)?;
                let radius = self.real_at(id, SPHERICAL_SURFACE, 2)?;
                let ball = Sphere::new(place.origin, radius)
                    .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
                Ok(Surface::Sphere(ball))
            }
            Some(CONICAL_SURFACE) => {
                let position = self.reference_at(id, CONICAL_SURFACE, 1)?;
                let place = self.placement_at(id, position)?;
                let radius = self.real_at(id, CONICAL_SURFACE, 2)?;
                let written = self.real_at(id, CONICAL_SURFACE, 3)?;
                let radians = self.radians_per_angle_unit.get().ok_or_else(|| {
                    Refusal::ALengthUnitThisKernelDoesNotHave {
                        id,
                        described: "a cone whose semi-angle is in a unit this reader cannot read"
                            .to_owned(),
                    }
                })?;
                let half_angle = HalfAngle {
                    radians: written * radians,
                    cut_at: if radians == 1.0 {
                        self.last_digit_at(id, CONICAL_SURFACE, 3)
                    } else {
                        None
                    },
                };
                Ok(Surface::Cone(cone_from(
                    id, &place, radius, half_angle, corners,
                )?))
            }
            Some(TOROIDAL_SURFACE) => {
                let position = self.reference_at(id, TOROIDAL_SURFACE, 1)?;
                let place = self.placement_at(id, position)?;
                let ring_radius = self.real_at(id, TOROIDAL_SURFACE, 2)?;
                let tube_radius = self.real_at(id, TOROIDAL_SURFACE, 3)?;
                let ring = Axis::as_written(place.origin, place.axis)
                    .and_then(|axis| Torus::new(axis, ring_radius, tube_radius))
                    .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
                Ok(Surface::Torus(ring))
            }
            _ => Err(Refusal::ASurfaceThisKernelDoesNotHold {
                face,
                surface: id,
                keyword: keyword.to_owned(),
            }),
        }
    }

    /// A `CYLINDRICAL_SURFACE`'s pipe.
    fn pipe_at(&self, id: EntityId) -> Result<Cylinder, Refusal> {
        let position = self.reference_at(id, CYLINDRICAL_SURFACE, 1)?;
        let place = self.placement_at(id, position)?;
        let radius = self.real_at(id, CYLINDRICAL_SURFACE, 2)?;
        Axis::as_written(place.origin, place.axis)
            .and_then(|axis| Cylinder::new(axis, radius))
            .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })
    }

    /// The pipe a `CIRCLE` named `'#n'` points at, where `#n` is a
    /// `CYLINDRICAL_SURFACE` that reads as a pipe and `keep` takes it.
    ///
    /// **For a circle drawn on a flat face whose own pipe starts off that
    /// face.** This writer writes such a circle where the kernel's walk centres
    /// it, and the pipe through that centre can walk it round another point, a
    /// few ulps away; so it writes the kernel's own pipe beside the circle and
    /// names the circle after it (`Writing::an_arc_of_a_circle_on_a_flat_face`,
    /// `docs/decisions.md` §249.8). A name is free text in Part 21, so any
    /// other name, and a pipe that does not read or that `keep` turns down,
    /// gives `None` and is not refused.
    ///
    /// A `1` read as `1.` in a pipe `keep` turns down is not counted, since
    /// that pipe takes no part in the model. One in a pipe it takes is: the
    /// pipe this writer names a circle after is one no face sits on, so
    /// nothing reads it again.
    pub(crate) fn pipe_a_circle_names(
        &self,
        circle: EntityId,
        keep: impl FnOnce(&Cylinder) -> bool,
    ) -> Option<Cylinder> {
        let name = self.record(circle, CIRCLE).ok()?.attribute(0)?.as_text()?;
        let named = EntityId(name.strip_prefix('#')?.parse().ok()?);
        let counted = self.integers_read_as_reals.get();
        let pipe = self.pipe_at(named).ok().filter(keep);
        if pipe.is_none() {
            self.integers_read_as_reals.set(counted);
        }
        pipe
    }

    /// An `AXIS2_PLACEMENT_3D`: where it is, which way it points, and which way
    /// its first coordinate runs.
    ///
    /// The reference direction is optional in the schema and is written `$`
    /// when it is left out. Nothing this reader does needs one that is not
    /// there — a circle's arc is picked by its two ends and the axis, and never
    /// by where the parametrisation starts — so an absent one is absent and not
    /// a default.
    pub(crate) fn placement_at(&self, from: EntityId, id: EntityId) -> Result<Placement, Refusal> {
        let instance = self.instance(from, id)?;
        if instance.part(AXIS2_PLACEMENT_3D.name()).is_none() {
            return Err(Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: AXIS2_PLACEMENT_3D.name(),
                found: instance.parts()[0].keyword().to_owned(),
            });
        }
        let at = self.reference_at(id, AXIS2_PLACEMENT_3D, 1)?;
        let origin = self.point_at(id, at)?;
        let axis = match self.optional_reference_at(id, AXIS2_PLACEMENT_3D, 2)? {
            Some(direction) => self.direction_at(id, direction)?,
            None => Vector::new(0.0, 0.0, 1.0),
        };
        let reference = match self.optional_reference_at(id, AXIS2_PLACEMENT_3D, 3)? {
            Some(direction) => Some(self.direction_at(id, direction)?),
            None => None,
        };
        Ok(Placement {
            origin,
            axis,
            reference,
        })
    }

    /// A `CARTESIAN_POINT`, in the file's own numbers and its own units.
    pub(crate) fn point_at(&self, from: EntityId, id: EntityId) -> Result<Point, Refusal> {
        let coordinates = self.three_numbers(from, id, CARTESIAN_POINT)?;
        Ok(Point::new(coordinates[0], coordinates[1], coordinates[2]))
    }

    /// A `CARTESIAN_POINT` read ahead of the walk, without counting an
    /// integer in it.
    ///
    /// For a corner of a face, which a cone's slope reads before the face is
    /// described ([`cone_from`]). Describing the face reads the same corner
    /// again and counts it then, so counting it here as well would count it
    /// twice in [`Import::integers_read_as_reals`](crate::Import).
    pub(crate) fn point_read_ahead_at(
        &self,
        from: EntityId,
        id: EntityId,
    ) -> Result<Point, Refusal> {
        let counted = self.integers_read_as_reals.get();
        let point = self.point_at(from, id);
        self.integers_read_as_reals.set(counted);
        point
    }

    /// A `DIRECTION`, stored as the file wrote it and not scaled to length one.
    ///
    /// Whatever needs a unit direction scales it itself, and says what that
    /// costs where it does it.
    pub(crate) fn direction_at(&self, from: EntityId, id: EntityId) -> Result<Vector, Refusal> {
        let ratios = self.three_numbers(from, id, DIRECTION)?;
        Ok(Vector::new(ratios[0], ratios[1], ratios[2]))
    }

    fn three_numbers(
        &self,
        from: EntityId,
        id: EntityId,
        keyword: Keyword,
    ) -> Result<[f64; 3], Refusal> {
        let instance = self.instance(from, id)?;
        let Some(record) = instance.part(keyword.name()) else {
            return Err(Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: keyword.name(),
                found: instance.parts()[0].keyword().to_owned(),
            });
        };
        let wrong = || Refusal::AnAttributeIsNotWhatTheSchemaWants {
            id,
            keyword: keyword.name(),
            position: 1,
            wanted: "list of three numbers",
        };
        let list = record
            .attribute(1)
            .and_then(Value::as_list)
            .ok_or_else(wrong)?;
        let [first, second, third] = list else {
            return Err(wrong());
        };
        let mut numbers = [0.0; 3];
        for (slot, value) in numbers.iter_mut().zip([first, second, third]) {
            *slot = self.number(id, 1, value)?;
        }
        Ok(numbers)
    }

    /// Which of several names an instance goes by, for the pairs where the
    /// schema has a supertype and a subtype carrying the same attributes —
    /// `ADVANCED_FACE` under `FACE_SURFACE`, `FACE_OUTER_BOUND` under
    /// `FACE_BOUND`, `LENGTH_MEASURE_WITH_UNIT` under `MEASURE_WITH_UNIT`.
    pub(crate) fn keyword_of(
        &self,
        from: EntityId,
        id: EntityId,
        either: &[Keyword],
    ) -> Result<Keyword, Refusal> {
        let held = self.instance(from, id)?;
        either
            .iter()
            .find(|keyword| held.part(keyword.name()).is_some())
            .copied()
            .ok_or_else(|| Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: either[0].name(),
                found: held.parts()[0].keyword().to_owned(),
            })
    }

    /// An attribute the schema says is a real.
    pub(crate) fn real_at(
        &self,
        id: EntityId,
        keyword: Keyword,
        position: usize,
    ) -> Result<f64, Refusal> {
        let value = self
            .record(id, keyword)?
            .attribute(position)
            .ok_or(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword: keyword.name(),
                position,
                wanted: "real",
            })?
            .clone();
        self.number(id, position, &value)
    }

    /// The place of the last digit a real attribute was written to, where the
    /// file wrote it as a real.
    fn last_digit_at(&self, id: EntityId, keyword: Keyword, position: usize) -> Option<LastDigit> {
        self.record(id, keyword).ok()?.last_digit_of(position)
    }

    /// The number a value stands for.
    ///
    /// **`1` is read as the real `1.`, and every one is counted.** Part 21
    /// writes a real with a decimal point and [`Value`] keeps the two apart on
    /// purpose, because the reader that parses does not know the schema. This
    /// one does: the schema says this attribute is a `REAL`, an integer and a
    /// real that name the same number *are* the same number, and reading it
    /// loses nothing and guesses nothing. Past `2^53` an `f64` can no longer
    /// hold the integer exactly and it is refused by name instead.
    ///
    /// A [`Value::Typed`] wrapper — `LENGTH_MEASURE(25.4)` — is unwrapped, once.
    /// It is how a `MEASURE_WITH_UNIT` writes its number and it wears the type
    /// name a `SELECT` needs, not a second value.
    fn number(&self, id: EntityId, position: usize, value: &Value) -> Result<f64, Refusal> {
        match value {
            Value::Real(number) => Ok(*number),
            Value::Integer(whole) => {
                if whole.unsigned_abs() > (1u64 << 53) {
                    return Err(Refusal::AnIntegerTooBigToBeTheRealItStandsFor {
                        id,
                        position,
                        value: *whole,
                    });
                }
                self.integers_read_as_reals
                    .set(self.integers_read_as_reals.get() + 1);
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "refused just above unless the integer is one an f64 holds exactly"
                )]
                Ok(*whole as f64)
            }
            Value::Typed { value, .. } => match value.as_ref() {
                Value::Real(number) => Ok(*number),
                Value::Integer(_) => self.number(id, position, value),
                _ => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                    id,
                    keyword: "a measure",
                    position,
                    wanted: "real",
                }),
            },
            Value::Text(_)
            | Value::Enumeration(_)
            | Value::Reference(_)
            | Value::List(_)
            | Value::Nothing
            | Value::Derived => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword: "a number",
                position,
                wanted: "real",
            }),
        }
    }

    /// An attribute the schema says is a `BOOLEAN`.
    pub(crate) fn flag_at(
        &self,
        id: EntityId,
        keyword: Keyword,
        position: usize,
    ) -> Result<bool, Refusal> {
        match self
            .record(id, keyword)?
            .attribute(position)
            .and_then(Value::as_enumeration)
            .and_then(EnumerationName::of)
        {
            Some(TRUE) => Ok(true),
            Some(FALSE) => Ok(false),
            _ => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword: keyword.name(),
                position,
                wanted: "boolean",
            }),
        }
    }

    /// An attribute the schema says points at another entity.
    pub(crate) fn reference_at(
        &self,
        id: EntityId,
        keyword: Keyword,
        position: usize,
    ) -> Result<EntityId, Refusal> {
        self.optional_reference_at(id, keyword, position)?.ok_or(
            Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword: keyword.name(),
                position,
                wanted: "reference",
            },
        )
    }

    /// The same, where the schema allows the attribute to be left out.
    pub(crate) fn optional_reference_at(
        &self,
        id: EntityId,
        keyword: Keyword,
        position: usize,
    ) -> Result<Option<EntityId>, Refusal> {
        match self.record(id, keyword)?.attribute(position) {
            Some(Value::Reference(to)) => Ok(Some(*to)),
            Some(Value::Nothing) => Ok(None),
            _ => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword: keyword.name(),
                position,
                wanted: "reference",
            }),
        }
    }

    /// An attribute the schema says is a list of references.
    pub(crate) fn references_at(
        &self,
        id: EntityId,
        keyword: Keyword,
        position: usize,
    ) -> Result<Vec<EntityId>, Refusal> {
        let wrong = || Refusal::AnAttributeIsNotWhatTheSchemaWants {
            id,
            keyword: keyword.name(),
            position,
            wanted: "list of references",
        };
        let list = self
            .record(id, keyword)?
            .attribute(position)
            .and_then(Value::as_list)
            .ok_or_else(wrong)?;
        list.iter()
            .map(|value| value.as_reference().ok_or_else(wrong))
            .collect()
    }

    pub(crate) fn record(&self, id: EntityId, keyword: Keyword) -> Result<&'a Record, Refusal> {
        let instance = self
            .file
            .instance(id)
            .ok_or(Refusal::AReferencePointsNowhere { from: id, to: id })?;
        instance
            .part(keyword.name())
            .ok_or_else(|| Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: keyword.name(),
                found: instance.parts()[0].keyword().to_owned(),
            })
    }

    fn instance(
        &self,
        from: EntityId,
        id: EntityId,
    ) -> Result<&'a crate::graph::Instance, Refusal> {
        self.file
            .instance(id)
            .ok_or(Refusal::AReferencePointsNowhere { from, to: id })
    }
}

/// Where a piece of geometry sits, which way it points, and where its first
/// coordinate runs from.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Placement {
    pub(crate) origin: Point,
    pub(crate) axis: Vector,
    pub(crate) reference: Option<Vector>,
}

/// The plane a `PLANE` placed at `origin` and facing along `axis` describes,
/// and the plane a `CIRCLE` or `ELLIPSE` placed there lies in.
///
/// **A direction already of length one to within what a turn leaves is kept
/// as the file wrote it**, and any other is scaled to length one
/// ([`Plane::from_point_and_normal_as_written`]). A face a turn placed has a
/// normal an ulp or two off length one, and scaling it again moved its last
/// bit on 1 509 of the writer fuzzer's 2026-09-26 crash files, most of them
/// normals no direction scaled to length one reads back as
/// (`docs/decisions.md` §313.3). A direction another writer wrote to seventeen
/// digits a few ulps from length one keeps its own bits too.
///
/// The offset is `origin` dotted with that normal and rounded once, so the
/// file's point decides the offset's last bit. The writer asks this same
/// function which point to write (`a_point_this_reader_reads_back_as`, in
/// `write.rs`), so that what one writes and the other reads cannot drift
/// apart.
pub(crate) fn plane_through(origin: Point, axis: Vector) -> rcad_geom::Result<Plane> {
    Plane::from_point_and_normal_as_written(origin, axis)
}

/// Whether `rcad_geom::walk` centres the circle each of `planes` crosses
/// `pipe` in at `centre`, to the bit.
///
/// The writer and the reader ask this of the same pipe and the same planes, so
/// that where the writer names a drawn circle after the pipe it writes beside
/// it, the reader takes that pipe (`docs/decisions.md` §249.8).
pub(crate) fn the_walk_centres_it_at(pipe: &Cylinder, planes: &[Plane], centre: Point) -> bool {
    let bits = |at: Point| at.coordinates().map(f64::to_bits);
    planes.iter().all(|flat| {
        where_the_walk_centres_a_circle(flat, pipe)
            .is_some_and(|walked| bits(walked) == bits(centre))
    })
}

/// Where `rcad_geom::walk` centres the circle `flat` crosses `pipe` in: the
/// pipe's own origin moved up its axis to `flat`, in plain `f64`, as
/// `walk.rs`'s `where_a_plane_cuts_a_pipe` works it out. `None` where the axis
/// runs along `flat` and meets it nowhere.
pub(crate) fn where_the_walk_centres_a_circle(flat: &Plane, pipe: &Cylinder) -> Option<Point> {
    let along = pipe.axis().direction();
    let leans = flat.normal().dot(along);
    if leans == 0.0 {
        return None;
    }
    let from = pipe.axis().origin();
    let up = (flat.offset() - flat.normal().dot(from - Point::origin())) / leans;
    Some(from + along * up)
}

/// A `CONICAL_SURFACE`'s half-angle, and how far the file may have cut it.
#[derive(Clone, Copy, Debug)]
struct HalfAngle {
    /// The file's number times its angle unit.
    radians: f64,
    /// The place of the last digit the file wrote the angle to, where it wrote
    /// it in radians and as a real. In any other unit the number this reader
    /// takes the tangent of is a product, and the file's digits are not its
    /// digits.
    cut_at: Option<LastDigit>,
}

/// A `CONICAL_SURFACE`'s cone, whose apex the file does not write down.
///
/// The half-angle arrives in radians, which for a file that writes degrees is
/// the file's own conversion factor times the number it wrote — one
/// multiplication, and it lands in the same `tan` below that rounds anyway.
///
/// STEP gives the radius at the placement's own origin and the half-angle;
/// [`Cone`] is the apex and the slope, the tangent of that angle, which
/// [`the_slope_of`] works out. So the apex is `origin − (radius / slope)·d̂`,
/// and **two roundings happen here and nowhere else in this reader**: the
/// slope — `tan`, which Rust does not bound, which is why [`Cone`] stores the
/// tangent rather than the angle — and the division. A cone written with its
/// apex at its placement, `radius = 0`, has no division: the offset is exactly
/// zero and the apex is exactly the origin the file wrote.
///
/// The direction is the file's own when it is of length one to within what a
/// turn leaves ([`Axis::as_written`]), and the axis through the apex is that
/// one moved there ([`Axis::moved_to`]). Building it again with [`Axis::new`]
/// would scale an already-scaled direction a second time, which moves its
/// last bit on about one direction in a hundred.
fn cone_from(
    id: EntityId,
    place: &Placement,
    radius: f64,
    half_angle: HalfAngle,
    corners: impl FnOnce() -> Option<Vec<Point>>,
) -> Result<Cone, Refusal> {
    let along = Axis::as_written(place.origin, place.axis)
        .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
    let slope = the_slope_of(&along, radius, half_angle, corners);
    let apex = if radius == 0.0 {
        place.origin
    } else {
        place.origin - along.direction() * (radius / slope)
    };
    along
        .moved_to(apex)
        .and_then(|axis| Cone::new(axis, slope))
        .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })
}

/// The tangent of the half-angle written, or the slope every corner of the
/// face agrees on where the half-angle written is that slope's angle cut to the
/// file's own digits.
///
/// **An exporter that cuts every real to eleven to thirteen digits moves a
/// cone's slope and none of its corners**, when the corners are short numbers
/// the cut leaves alone. A frustum of slope one half written to twelve digits
/// has the half-angle `0.463647609001`, which reads back as the slope
/// `0.5000000000002424`, and its corner `(7, 4, 1)` is then `1.30e-12` off the
/// cone, 52 times the room `Model::validate` gives a corner of a curved face.
/// Every one of the 48 frustum files of `a_fuse_read_back_from_rounded_text.rs`
/// was refused that way at 11 and 12 digits, and 40 at 13
/// (`docs/decisions.md` §366.7).
///
/// The corners' slope is taken only where all three hold, and the tangent of
/// the angle written is taken otherwise, as it always was:
///
/// - the angle is in radians and written as a real, and is strictly between
///   nought and a right angle, the half-angles a cone has;
/// - every corner off the placement's own plane puts the cone at the same `f64`
///   slope ([`a_slope_every_corner_agrees_on`]);
/// - the angle written is that slope's angle to within half a unit in the last
///   digit the file wrote it to, in exact arithmetic
///   ([`its_angle_is_written_as`]).
///
/// **No room is widened by it.** The corners are checked against the cone
/// this builds exactly as against any other, by `Model::validate`, and a corner
/// off it is refused as before.
fn the_slope_of(
    along: &Axis,
    radius: f64,
    half_angle: HalfAngle,
    corners: impl FnOnce() -> Option<Vec<Point>>,
) -> f64 {
    let written = half_angle.radians.tan();
    let Some(cut_at) = half_angle.cut_at else {
        return written;
    };
    if !(half_angle.radians > 0.0 && half_angle.radians < FRAC_PI_2) {
        return written;
    }
    let Some(corners) = corners() else {
        return written;
    };
    match a_slope_every_corner_agrees_on(along, radius, &corners) {
        Some(slope) if its_angle_is_written_as(slope, half_angle.radians, cut_at) => slope,
        Some(_) | None => written,
    }
}

/// The slope each corner puts the cone at, where every corner puts it at the
/// same `f64`.
///
/// A corner `ρ` from the axis and `h` along it from the placement is on the
/// STEP cone whose radius at the placement is `R` when `ρ = R + h·slope`, so on
/// its own it says `slope = (ρ − R) / h`. A corner on the placement's own
/// plane, `h = 0`, says nothing about the slope and is passed over: the apex
/// of a cone placed there, or a corner on the circle `R` is measured on.
///
/// `None` where no corner is left to say anything, where two say different
/// slopes, or where one says a slope of nought or below or no number at all.
/// **Agreeing means the same bits.** Corners in short numbers along an axis the
/// file wrote as a unit direction, as a frustum's are, work out to the same
/// `f64`; corners that work out an ulp apart are not given one of their two
/// answers, and the tangent of the angle written is taken instead.
fn a_slope_every_corner_agrees_on(along: &Axis, radius: f64, corners: &[Point]) -> Option<f64> {
    let mut agreed = None;
    for &corner in corners {
        let from_the_placement = corner - along.origin();
        let up = from_the_placement.dot(along.direction());
        if up == 0.0 {
            continue;
        }
        let out = along.direction().cross(from_the_placement).length();
        let slope = (out - radius) / up;
        if !(slope.is_finite() && slope > 0.0) || agreed.is_some_and(|earlier| earlier != slope) {
            return None;
        }
        agreed = Some(slope);
    }
    agreed
}

/// Whether `written` — the `f64` the file's text read as, with its last digit
/// at `last` — is `atan(slope)` rounded to that digit: whether the true angle
/// is within half a unit in it of the text.
///
/// **This is exact, and never asks `atan`**, which is the platform's and is
/// bounded by nothing Rust publishes. [`atan_between`] brackets the true angle
/// in `[lo, hi]`. The text, `D`, is not an `f64` either: `written` is the
/// nearest one to it, so `|D − written|` is at most half an ulp of `written`,
/// and one ulp is allowed for it. Half a unit, `5·10^(last − 1)`, is read by
/// Rust's parser, which rounds correctly, and taken one `f64` lower. With every
/// subtraction moved one `f64` the way that keeps it true, a yes here means
///
/// ```text
/// D − half ≤ written − (half − ulp) ≤ lo ≤ atan(slope) ≤ hi ≤ written + (half − ulp) ≤ D + half
/// ```
///
/// A half unit narrower than half the bracket is a no, and the tangent of the
/// angle written is taken. **Seventeen significant digits are always a no**:
/// half a unit in the seventeenth digit is less than an ulp of the angle, and
/// an ulp is taken off it. So a file that writes every angle whole reads as it
/// did, whatever its corners say.
fn its_angle_is_written_as(slope: f64, written: f64, last: LastDigit) -> bool {
    let (lo, hi) = atan_between(slope);
    let half_a_unit = format!("5e{}", i64::from(last.0) - 1)
        .parse::<f64>()
        .expect("Rust reads a number it wrote")
        .next_down();
    let room = (half_a_unit - (written.next_up() - written)).next_down();
    (written - lo).next_up() <= room && (hi - written).next_up() <= room
}

/// Two `f64` either side of `atan(slope)`, for a slope above nought.
///
/// **Every step is one operation IEEE 754 rounds correctly** — `+`, `−`, `×`,
/// `/` and `sqrt`, whose results Rust's own docs say are the exact answer
/// rounded — and each result is moved one `f64` outwards, past wherever the
/// exact answer can be. A slope above one is turned round first,
/// `atan(s) = π/2 − atan(1/s)`, with [`FRAC_PI_2`] below `π/2` and the next
/// `f64` above it (`frac_pi_2_and_the_next_f64_up_stand_either_side_of_a_right_angle`).
/// The rest is [`atan_below`] and [`atan_above`]. Over 100 001 slopes from a
/// thousandth to a thousand the two are at most 49 ulps of the angle apart
/// (`atan_between_holds_the_platforms_atan_over_slopes_from_a_thousandth_to_a_thousand`).
fn atan_between(slope: f64) -> (f64, f64) {
    assert!(
        slope.is_finite() && slope > 0.0,
        "a cone's slope is above nought and finite, not {slope}"
    );
    if slope <= 1.0 {
        return (atan_below(slope), atan_above(slope));
    }
    let (right_angle_below, right_angle_above) = (FRAC_PI_2, FRAC_PI_2.next_up());
    (
        (right_angle_below - atan_above((1.0 / slope).next_up())).next_down(),
        (right_angle_above - atan_below((1.0 / slope).next_down())).next_up(),
    )
}

/// A number no larger than `atan(x)`, for `x` from nought to a hair over one.
///
/// `atan(x) = 2·atan(x / (1 + √(1 + x²)))`, twice, takes `x` below
/// `tan(π/16) < 0.2`, and the halved argument is worked out low so that its
/// angle is no larger. [`series_below`] does the rest.
fn atan_below(x: f64) -> f64 {
    let halved = |x: f64| {
        let under = (1.0 + (1.0 + (x * x).next_up()).next_up().sqrt().next_up()).next_up();
        (x / under).next_down().max(0.0)
    };
    4.0 * series_below(halved(halved(x)))
}

/// A number no smaller than `atan(x)`, for `x` from nought to a hair over one:
/// [`atan_below`] with every rounding the other way.
fn atan_above(x: f64) -> f64 {
    let halved = |x: f64| {
        let under = (1.0 + (1.0 + (x * x).next_down()).next_down().sqrt().next_down()).next_down();
        (x / under).next_up()
    };
    4.0 * series_above(halved(halved(x)))
}

/// How many terms of `z − z³/3 + z⁵/5 − …` [`series_below`] adds up; the last
/// one is taken away. [`series_above`] stops one sooner, on a term added.
const TERMS_OF_THE_SERIES: u32 = 14;

/// A number no larger than `atan(z)`, for `0 ≤ z < 1/4`.
///
/// **The series alternates and its terms shrink**, so it stops below the angle
/// after a term taken away, and above it after a term added. The next term is
/// below `0.2^29/29 < 2·10^-22`, under an ulp of any angle this is asked of.
/// Each term and each sum is rounded down, and the powers of `z` are carried
/// as two bounds so that a term taken away can be taken at its largest.
fn series_below(z: f64) -> f64 {
    assert!((0.0..0.25).contains(&z), "the halved argument is {z}");
    let (square_below, square_above) = ((z * z).next_down().max(0.0), (z * z).next_up());
    let (mut power_below, mut power_above) = (z, z);
    let mut sum = z;
    for n in 1..TERMS_OF_THE_SERIES {
        power_below = (power_below * square_below).next_down().max(0.0);
        power_above = (power_above * square_above).next_up();
        let odd = f64::from(2 * n + 1);
        sum = if n % 2 == 1 {
            (sum - (power_above / odd).next_up()).next_down()
        } else {
            (sum + (power_below / odd).next_down().max(0.0)).next_down()
        };
    }
    sum
}

/// A number no smaller than `atan(z)`, for `0 ≤ z < 1/4`: [`series_below`]
/// one term shorter, so that it stops on a term added, and rounded up.
fn series_above(z: f64) -> f64 {
    assert!((0.0..0.25).contains(&z), "the halved argument is {z}");
    let (square_below, square_above) = ((z * z).next_down().max(0.0), (z * z).next_up());
    let (mut power_below, mut power_above) = (z, z);
    let mut sum = z;
    for n in 1..TERMS_OF_THE_SERIES - 1 {
        power_below = (power_below * square_below).next_down().max(0.0);
        power_above = (power_above * square_above).next_up();
        let odd = f64::from(2 * n + 1);
        sum = if n % 2 == 1 {
            (sum - (power_below / odd).next_down().max(0.0)).next_up()
        } else {
            (sum + (power_above / odd).next_up()).next_up()
        };
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `π/2` cut, not rounded, to 55 decimals, from its digits as published.
    const A_RIGHT_ANGLE_CUT: &str = "1.5707963267948966192313216916397514420985846996875529104";

    /// The exact value of an `f64` is a decimal of at most 52 places below
    /// one... and so 55 places print it whole, and two decimals of 55 places
    /// compare as their text does. [`FRAC_PI_2`] is below the cut, so below
    /// `π/2`; the next `f64` is above the cut by at least `10^-55`, so above
    /// `π/2`, which the cut is within `10^-55` of.
    #[test]
    fn frac_pi_2_and_the_next_f64_up_stand_either_side_of_a_right_angle() {
        let below = format!("{FRAC_PI_2:.55}");
        let above = format!("{:.55}", FRAC_PI_2.next_up());
        assert_eq!(below.len(), A_RIGHT_ANGLE_CUT.len());
        assert!(below.as_str() < A_RIGHT_ANGLE_CUT, "{below}");
        assert!(above.as_str() > A_RIGHT_ANGLE_CUT, "{above}");
    }

    /// Each angle worked out to 45 digits by `bc -l`. The nearest `f64` to the
    /// angle is strictly inside the bracket, which puts the angle strictly
    /// inside it too: the `f64` either side of the nearest one are further
    /// from the angle than a half step.
    #[test]
    fn atan_between_brackets_angles_worked_out_to_forty_five_digits() {
        for (slope, angle) in [
            (0.001, "0.000999999666666866666523809634920544011620934"),
            (0.25, "0.244978663126864154172082481211275810914144098"),
            (0.5, "0.463647609000806116214256231461214402028537054"),
            (1.0, "0.785398163397448309615660845819875721049292349"),
            (2.0, "1.107148717794090503017065460178537040070047645"),
            (3.0, "1.249045772398254425829917077281090123077829404"),
            (1000.0, "1.569796327128229752564797882004830898086963765"),
        ] {
            let nearest: f64 = angle.parse().expect("an angle bc wrote");
            let (lo, hi) = atan_between(slope);
            assert!(
                lo < nearest && nearest < hi,
                "atan({slope}) = {angle}, bracketed {lo}..{hi}"
            );
            assert!(
                hi - lo <= WIDEST_BRACKET_IN_ULPS * (nearest.next_up() - nearest),
                "atan({slope}) is bracketed {} ulps wide",
                (hi - lo) / (nearest.next_up() - nearest)
            );
        }
    }

    /// The widest [`atan_between`] is allowed to be, in ulps of the angle.
    /// `its_angle_is_written_as` says no where the unit the file wrote is
    /// narrower than this, so this is what puts fifteen digits out of its reach.
    const WIDEST_BRACKET_IN_ULPS: f64 = 64.0;

    /// The platform's `atan` is not what this rests on, but a bracket that
    /// left it more than an ulp outside would be a bracket worth looking at.
    /// Slopes drawn evenly in their logarithm from a thousandth to a thousand.
    #[test]
    fn atan_between_holds_the_platforms_atan_over_slopes_from_a_thousandth_to_a_thousand() {
        let mut widest: f64 = 0.0;
        for step in 0..=100_000 {
            let slope = 10f64.powf(-3.0 + 6.0 * f64::from(step) / 100_000.0);
            let platform = slope.atan();
            let (lo, hi) = atan_between(slope);
            assert!(
                lo <= platform.next_up() && platform.next_down() <= hi,
                "atan({slope}) is {platform}, bracketed {lo}..{hi}"
            );
            widest = widest.max((hi - lo) / (platform.next_up() - platform));
        }
        println!("100001 slopes from 1e-3 to 1e3: the widest bracket is {widest} ulps");
        assert!(widest <= WIDEST_BRACKET_IN_ULPS, "{widest} ulps");
    }

    #[test]
    fn atan_between_keeps_its_order_at_the_ends_of_the_f64() {
        for slope in [
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            1e-300,
            1e300,
            f64::MAX,
        ] {
            let (lo, hi) = atan_between(slope);
            assert!(
                -1e-300 < lo && lo <= hi && hi < FRAC_PI_2 + 1e-15,
                "atan({slope:e}) bracketed {lo}..{hi}"
            );
        }
    }

    /// Q827's frustum of slope one half, at the three digit counts an Open
    /// CASCADE exporter writes (`a_fuse_read_back_from_rounded_text.rs`).
    #[test]
    fn a_half_angle_cut_to_eleven_to_thirteen_digits_is_the_angle_of_slope_one_half() {
        for (text, last) in [
            ("0.46364760900", -11),
            ("0.463647609001", -12),
            ("0.4636476090008", -13),
        ] {
            let written: f64 = text.parse().expect("a real");
            assert!(
                its_angle_is_written_as(0.5, written, LastDigit(last)),
                "{text}"
            );
            assert!(
                written.tan() != 0.5,
                "{text} reads back as slope one half anyway"
            );
        }
        assert!(its_angle_is_written_as(
            2.0,
            1.107_148_717_79,
            LastDigit(-11)
        ));
    }

    #[test]
    fn a_half_angle_a_unit_off_in_its_last_digit_is_not_the_angle_of_that_slope() {
        assert!(!its_angle_is_written_as(
            0.5,
            0.463_647_609_002,
            LastDigit(-12)
        ));
        assert!(!its_angle_is_written_as(
            0.5,
            0.463_647_609_000,
            LastDigit(-12)
        ));
        assert!(!its_angle_is_written_as(
            0.500_000_1,
            0.463_647_609_001,
            LastDigit(-12)
        ));
    }

    /// The writer prints a half-angle in the fewest digits that read back as
    /// its `f64`, sixteen for `atan(0.5)`, and its slope must come back from
    /// the tangent bit for bit, as §366.2 has it. Half a unit in the sixteenth
    /// digit is narrower than an ulp, so the answer is no.
    #[test]
    fn a_half_angle_written_to_every_digit_an_f64_has_keeps_its_tangent() {
        let written = 0.5_f64.atan();
        assert_eq!(format!("{written:?}"), "0.4636476090008061");
        assert!(!its_angle_is_written_as(0.5, written, LastDigit(-16)));
        assert!(!its_angle_is_written_as(0.5, written, LastDigit(-17)));
    }

    fn up_z_from(x: f64, y: f64, z: f64) -> Axis {
        Axis::new(Point::new(x, y, z), Vector::new(0.0, 0.0, 1.0)).expect("an axis")
    }

    #[test]
    fn a_frustums_corners_agree_on_its_slope() {
        let apex = up_z_from(4.0, 4.0, -5.0);
        let corners = [
            Point::new(6.0, 4.0, -1.0),
            Point::new(7.0, 4.0, 1.0),
            Point::new(6.0, 4.0, -1.0),
        ];
        assert_eq!(
            a_slope_every_corner_agrees_on(&apex, 0.0, &corners),
            Some(0.5)
        );
    }

    /// Placed on its cap with the radius there, a pointed cone's corner on the
    /// cap says nothing, and its apex says the slope.
    #[test]
    fn a_cone_placed_on_its_cap_has_its_slope_from_its_apex() {
        let cap = up_z_from(0.0, 0.0, 8.0);
        let corners = [Point::new(4.0, 0.0, 8.0), Point::origin()];
        assert_eq!(
            a_slope_every_corner_agrees_on(&cap, 4.0, &corners),
            Some(0.5)
        );
    }

    #[test]
    fn a_cone_face_with_no_corner_off_its_placements_plane_has_no_slope_from_its_corners() {
        let apex = up_z_from(0.0, 0.0, 0.0);
        assert_eq!(
            a_slope_every_corner_agrees_on(&apex, 0.0, &[Point::origin()]),
            None
        );
        let cap = up_z_from(0.0, 0.0, 8.0);
        assert_eq!(
            a_slope_every_corner_agrees_on(&cap, 4.0, &[Point::new(4.0, 0.0, 8.0)]),
            None
        );
        assert_eq!(a_slope_every_corner_agrees_on(&cap, 4.0, &[]), None);
    }

    #[test]
    fn corners_that_say_two_slopes_or_none_have_no_slope_from_their_corners() {
        let apex = up_z_from(4.0, 4.0, -5.0);
        let two = [Point::new(6.0, 4.0, -1.0), Point::new(7.5, 4.0, 1.0)];
        assert_eq!(a_slope_every_corner_agrees_on(&apex, 0.0, &two), None);
        let below_the_apex = [Point::new(6.0, 4.0, -9.0)];
        assert_eq!(
            a_slope_every_corner_agrees_on(&apex, 0.0, &below_the_apex),
            None
        );
        let on_the_axis = [Point::new(4.0, 4.0, 1.0)];
        assert_eq!(
            a_slope_every_corner_agrees_on(&apex, 0.0, &on_the_axis),
            None
        );
    }

    #[test]
    fn the_slope_comes_from_the_corners_only_where_the_angle_was_cut_and_they_agree() {
        let apex = up_z_from(4.0, 4.0, -5.0);
        let corners = || Some(vec![Point::new(6.0, 4.0, -1.0), Point::new(7.0, 4.0, 1.0)]);
        let cut = HalfAngle {
            radians: 0.463_647_609_001,
            cut_at: Some(LastDigit(-12)),
        };
        assert_eq!(the_slope_of(&apex, 0.0, cut, corners), 0.5);
        let uncut = HalfAngle {
            cut_at: None,
            ..cut
        };
        assert_eq!(the_slope_of(&apex, 0.0, uncut, corners), cut.radians.tan());
        assert_eq!(the_slope_of(&apex, 0.0, cut, || None), cut.radians.tan());
        let backwards = HalfAngle {
            radians: -cut.radians,
            ..cut
        };
        assert_eq!(
            the_slope_of(&apex, 0.0, backwards, corners),
            backwards.radians.tan()
        );
    }
}
