//! What a file's geometric entities are, in the kernel's own types.
//!
//! Nothing here builds topology and nothing here converts a unit. A number in
//! the file is the number in the model: §42.8 bakes world coordinates and §3.5
//! says the declared unit never takes part in arithmetic, so the only rounding
//! anything below does is the rounding a constructor does on its way in, and
//! each place that rounds says so.

use core::cell::Cell;

use rcad_geom::{Axis, Cone, Cylinder, Plane, Point, Sphere, Surface, Torus, Vector};

use crate::graph::{EntityId, Record, StepFile, Value};
use crate::refusal::Refusal;

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
    pub(crate) fn surface_at(&self, face: EntityId, id: EntityId) -> Result<Surface, Refusal> {
        let instance = self.instance(face, id)?;
        let keyword = instance.parts()[0].keyword();
        match keyword {
            "PLANE" => {
                let position = self.reference_at(id, "PLANE", 1)?;
                let place = self.placement_at(id, position)?;
                let plane = Plane::from_point_and_normal(place.origin, place.axis)
                    .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
                Ok(Surface::Plane(plane))
            }
            "CYLINDRICAL_SURFACE" => {
                let position = self.reference_at(id, "CYLINDRICAL_SURFACE", 1)?;
                let place = self.placement_at(id, position)?;
                let radius = self.real_at(id, "CYLINDRICAL_SURFACE", 2)?;
                let pipe = Axis::new(place.origin, place.axis)
                    .and_then(|axis| Cylinder::new(axis, radius))
                    .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
                Ok(Surface::Cylinder(pipe))
            }
            "SPHERICAL_SURFACE" => {
                let position = self.reference_at(id, "SPHERICAL_SURFACE", 1)?;
                let place = self.placement_at(id, position)?;
                let radius = self.real_at(id, "SPHERICAL_SURFACE", 2)?;
                let ball = Sphere::new(place.origin, radius)
                    .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
                Ok(Surface::Sphere(ball))
            }
            "CONICAL_SURFACE" => {
                let position = self.reference_at(id, "CONICAL_SURFACE", 1)?;
                let place = self.placement_at(id, position)?;
                let radius = self.real_at(id, "CONICAL_SURFACE", 2)?;
                let written = self.real_at(id, "CONICAL_SURFACE", 3)?;
                let radians = self.radians_per_angle_unit.get().ok_or_else(|| {
                    Refusal::ALengthUnitThisKernelDoesNotHave {
                        id,
                        described: "a cone whose semi-angle is in a unit this reader cannot read"
                            .to_owned(),
                    }
                })?;
                Ok(Surface::Cone(cone_from(id, &place, radius, written * radians)?))
            }
            "TOROIDAL_SURFACE" => {
                let position = self.reference_at(id, "TOROIDAL_SURFACE", 1)?;
                let place = self.placement_at(id, position)?;
                let ring_radius = self.real_at(id, "TOROIDAL_SURFACE", 2)?;
                let tube_radius = self.real_at(id, "TOROIDAL_SURFACE", 3)?;
                let ring = Axis::new(place.origin, place.axis)
                    .and_then(|axis| Torus::new(axis, ring_radius, tube_radius))
                    .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
                Ok(Surface::Torus(ring))
            }
            other => Err(Refusal::ASurfaceThisKernelDoesNotHold {
                face,
                surface: id,
                keyword: other.to_owned(),
            }),
        }
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
        if instance.part("AXIS2_PLACEMENT_3D").is_none() {
            return Err(Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: "AXIS2_PLACEMENT_3D",
                found: instance.parts()[0].keyword().to_owned(),
            });
        }
        let keyword = "AXIS2_PLACEMENT_3D";
        let at = self.reference_at(id, keyword, 1)?;
        let origin = self.point_at(id, at)?;
        let axis = match self.optional_reference_at(id, keyword, 2)? {
            Some(direction) => self.direction_at(id, direction)?,
            None => Vector::new(0.0, 0.0, 1.0),
        };
        let reference = match self.optional_reference_at(id, keyword, 3)? {
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
        let coordinates = self.three_numbers(from, id, "CARTESIAN_POINT")?;
        Ok(Point::new(coordinates[0], coordinates[1], coordinates[2]))
    }

    /// A `DIRECTION`, stored as the file wrote it and not scaled to length one.
    ///
    /// Whatever needs a unit direction scales it itself, and says what that
    /// costs where it does it.
    pub(crate) fn direction_at(&self, from: EntityId, id: EntityId) -> Result<Vector, Refusal> {
        let ratios = self.three_numbers(from, id, "DIRECTION")?;
        Ok(Vector::new(ratios[0], ratios[1], ratios[2]))
    }

    fn three_numbers(
        &self,
        from: EntityId,
        id: EntityId,
        keyword: &'static str,
    ) -> Result<[f64; 3], Refusal> {
        let instance = self.instance(from, id)?;
        let Some(record) = instance.part(keyword) else {
            return Err(Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: keyword,
                found: instance.parts()[0].keyword().to_owned(),
            });
        };
        let wrong = || Refusal::AnAttributeIsNotWhatTheSchemaWants {
            id,
            keyword,
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
        either: &[&'static str],
    ) -> Result<&'static str, Refusal> {
        let held = self.instance(from, id)?;
        either
            .iter()
            .find(|keyword| held.part(keyword).is_some())
            .copied()
            .ok_or_else(|| Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: either[0],
                found: held.parts()[0].keyword().to_owned(),
            })
    }

    /// An attribute the schema says is a real.
    pub(crate) fn real_at(
        &self,
        id: EntityId,
        keyword: &'static str,
        position: usize,
    ) -> Result<f64, Refusal> {
        let value = self
            .record(id, keyword)?
            .attribute(position)
            .ok_or(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword,
                position,
                wanted: "real",
            })?
            .clone();
        self.number(id, position, &value)
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
        keyword: &'static str,
        position: usize,
    ) -> Result<bool, Refusal> {
        match self
            .record(id, keyword)?
            .attribute(position)
            .and_then(Value::as_enumeration)
        {
            Some("T") => Ok(true),
            Some("F") => Ok(false),
            _ => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword,
                position,
                wanted: "boolean",
            }),
        }
    }

    /// An attribute the schema says points at another entity.
    pub(crate) fn reference_at(
        &self,
        id: EntityId,
        keyword: &'static str,
        position: usize,
    ) -> Result<EntityId, Refusal> {
        self.optional_reference_at(id, keyword, position)?.ok_or(
            Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword,
                position,
                wanted: "reference",
            },
        )
    }

    /// The same, where the schema allows the attribute to be left out.
    pub(crate) fn optional_reference_at(
        &self,
        id: EntityId,
        keyword: &'static str,
        position: usize,
    ) -> Result<Option<EntityId>, Refusal> {
        match self.record(id, keyword)?.attribute(position) {
            Some(Value::Reference(to)) => Ok(Some(*to)),
            Some(Value::Nothing) => Ok(None),
            _ => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
                id,
                keyword,
                position,
                wanted: "reference",
            }),
        }
    }

    /// An attribute the schema says is a list of references.
    pub(crate) fn references_at(
        &self,
        id: EntityId,
        keyword: &'static str,
        position: usize,
    ) -> Result<Vec<EntityId>, Refusal> {
        let wrong = || Refusal::AnAttributeIsNotWhatTheSchemaWants {
            id,
            keyword,
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

    pub(crate) fn record(
        &self,
        id: EntityId,
        keyword: &'static str,
    ) -> Result<&'a Record, Refusal> {
        let instance = self
            .file
            .instance(id)
            .ok_or(Refusal::AReferencePointsNowhere { from: id, to: id })?;
        instance
            .part(keyword)
            .ok_or_else(|| Refusal::AnEntityIsNotTheKindTheSchemaWants {
                id,
                wanted: keyword,
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

/// A `CONICAL_SURFACE`'s cone, whose apex the file does not write down.
///
/// The half-angle arrives in radians, which for a file that writes degrees is
/// the file's own conversion factor times the number it wrote — one
/// multiplication, and it lands in the same `tan` below that rounds anyway.
///
/// STEP gives the radius at the placement's own origin and the half-angle;
/// [`Cone`] is the apex and the tangent of that angle. So the apex is
/// `origin − (radius / tan θ)·d̂`, and **two roundings happen here and nowhere
/// else in this reader**: `tan` — which Rust does not bound, which is why
/// [`Cone`] stores the tangent rather than the angle — and the division. A cone
/// written with its apex at its placement, `radius = 0`, has neither: the
/// offset is exactly zero and the apex is exactly the origin the file wrote.
fn cone_from(
    id: EntityId,
    place: &Placement,
    radius: f64,
    half_angle: f64,
) -> Result<Cone, Refusal> {
    let along = Axis::new(place.origin, place.axis)
        .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })?;
    let slope = half_angle.tan();
    let apex = if radius == 0.0 {
        place.origin
    } else {
        place.origin - along.direction() * (radius / slope)
    };
    Axis::new(apex, along.direction())
        .and_then(|axis| Cone::new(axis, slope))
        .map_err(|why| Refusal::TheGeometryIsNotAWorkingShape { id, why })
}
