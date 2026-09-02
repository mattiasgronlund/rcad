//! A file's solids become a [`Model`], or the file is refused by name.

use std::collections::BTreeMap;

use rcad_geom::{Point, Scale, Unit, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, SolidKey};

use crate::brep::solid_from;
use crate::geometry::Reading;
use crate::graph::{EntityId, StepFile, Value};
use crate::refusal::Refusal;
use crate::units::what_the_file_declared;

/// What a STEP file's solids came to.
///
/// # What it takes, and what it walks past
///
/// It takes every `MANIFOLD_SOLID_BREP` in the file, in file order, each as one
/// solid. A STEP file holds much else — annotations, tolerances, styling,
/// tessellated pictures, loose curves and surfaces — and none of that is a
/// solid. **What is walked past is counted rather than forgotten**:
/// [`Import::shapes_this_reader_did_not_take`] names every shape-carrying entity
/// the file held that this reader is not the reader for, with how many of each.
/// A `BREP_WITH_VOIDS` is *not* on that list: it is a solid, this kernel holds
/// one shell to a solid, and it is refused by name.
///
/// # World coordinates, and no placement
///
/// §42.8 decided this and it is measured on both sides: every solid is stored
/// with the file's own numbers at the identity. So a file that places one shape
/// at several occurrences is **refused** rather than imported at the wrong
/// coordinates — see [`Refusal::APlacementThisReaderCannotApply`] for what
/// closes that.
///
/// # Nothing is healed
///
/// No vertex is merged with a nearby one, no gap is sewn, no loop is reordered
/// and no tolerance is taken from the file's declared uncertainty (§42.9 option
/// A). The uncertainty is read and published — [`Import::declared_uncertainty`]
/// — and no predicate anywhere is asked a different question because of it.
#[derive(Debug)]
pub struct Import {
    model: Model,
    solids: Vec<(EntityId, SolidKey)>,
    unit: Unit,
    uncertainty: Option<f64>,
    how_many_uncertainties: usize,
    biggest_coordinate: f64,
    integers_read_as_reals: usize,
    walked_past: BTreeMap<&'static str, usize>,
}

impl Import {
    /// Read every solid the file holds into one [`Model`].
    ///
    /// # Errors
    ///
    /// A [`Refusal`] naming the entity that stopped it. Every one of them is
    /// documented on the variant, with whether it is a limit or a gap.
    pub fn read(
        file: &StepFile,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Self, Refusal> {
        let reading = Reading::of(file);
        let declared = what_the_file_declared(&reading)?;
        reading.angles_are_measured_in(declared.radians_per_angle_unit);
        nothing_is_placed_somewhere_this_reader_cannot_move_it(&reading)?;

        let biggest_coordinate = biggest_coordinate_in(file);
        let scale =
            Scale::new(declared.unit, biggest_coordinate).map_err(|_| Refusal::TheFileHasNoSize)?;
        let mut model = Model::new(scale);

        let breps: Vec<EntityId> = file
            .instances()
            .filter(|held| held.part("MANIFOLD_SOLID_BREP").is_some())
            .map(crate::Instance::id)
            .collect();
        if let Some(voided) = file.instances_named("BREP_WITH_VOIDS").next() {
            return Err(Refusal::ASolidWithAVoidInIt { id: voided.id() });
        }
        if breps.is_empty() {
            return Err(Refusal::TheFileHoldsNoSolid);
        }

        let mut solids = Vec::with_capacity(breps.len());
        for brep in breps {
            solids.push((brep, solid_from(&reading, brep, &mut model, policy, log)?));
        }

        Ok(Self {
            model,
            solids,
            unit: declared.unit,
            uncertainty: declared.uncertainty,
            how_many_uncertainties: declared.how_many_uncertainties,
            biggest_coordinate,
            integers_read_as_reals: reading.integers_read_as_reals(),
            walked_past: what_this_reader_walked_past(file),
        })
    }

    /// The model the file's solids were written into.
    #[must_use]
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// The model, to keep.
    #[must_use]
    pub fn into_model(self) -> Model {
        self.model
    }

    /// Every solid, with the `#id` of the `MANIFOLD_SOLID_BREP` it came from.
    #[must_use]
    pub fn solids(&self) -> &[(EntityId, SolidKey)] {
        &self.solids
    }

    /// The length unit the file declared, which is the unit the model's
    /// coordinates are in.
    ///
    /// **Nothing was converted to get here** and no number in the model differs
    /// from the number in the file. A file written in inches comes back in
    /// inches (§3.5, §42.12 criterion 6).
    #[must_use]
    pub fn declared_unit(&self) -> Unit {
        self.unit
    }

    /// The largest uncertainty the file declared, in [`Import::declared_unit`],
    /// or `None` if it declared none.
    ///
    /// **Read and reported, never acted on.** Every predicate the import asked
    /// was asked exactly what it would have been asked had the file said
    /// nothing.
    #[must_use]
    pub fn declared_uncertainty(&self) -> Option<f64> {
        self.uncertainty
    }

    /// How many `UNCERTAINTY_MEASURE_WITH_UNIT`s the file declared for lengths.
    #[must_use]
    pub fn how_many_uncertainties_were_declared(&self) -> usize {
        self.how_many_uncertainties
    }

    /// Whether the file asks for more precision than its own numbers can carry.
    ///
    /// The coordinates are `f64`, so two neighbouring ones at the model's
    /// largest magnitude are `ε` of it apart, and nothing written in the file
    /// can be more precise than that. A declared uncertainty *below* that
    /// spacing is a promise the file's own numbers do not keep, and this says so
    /// (§42.12 criterion 7). It is one comparison of two numbers the file wrote
    /// and it changes nothing: no tolerance moves either way.
    #[must_use]
    pub fn asks_for_more_precision_than_its_numbers_carry(&self) -> bool {
        self.uncertainty
            .is_some_and(|declared| declared < self.biggest_coordinate * f64::EPSILON)
    }

    /// How many times an attribute the schema says is a real was written as an
    /// integer, and read as the real it names.
    ///
    /// Part 21 writes a real with a decimal point, and
    /// [`StepFile`](crate::StepFile) keeps `1` and `1.` apart because a reader
    /// that parses does not know the schema. This one does. Nothing is guessed
    /// and nothing is lost — an integer and a real that name one number are one
    /// number — but a reader that did it quietly would be a reader that heals,
    /// so the count is published. The spike found `step-io` doing this 865 times
    /// in the NIST AP242 files.
    #[must_use]
    pub fn integers_read_as_reals(&self) -> usize {
        self.integers_read_as_reals
    }

    /// Shape-carrying entities the file held that this reader is not the reader
    /// for, with how many of each.
    ///
    /// A tessellated export, a surface model, a loose curve set: real geometry,
    /// none of it a solid. An importer that quietly saw nothing in a file made
    /// entirely of them would be the silent drop §42.11 exists to avoid, so
    /// they are counted and handed back.
    #[must_use]
    pub fn shapes_this_reader_did_not_take(&self) -> &BTreeMap<&'static str, usize> {
        &self.walked_past
    }
}

/// The largest coordinate magnitude anywhere in the file, which is what
/// [`Scale::nominal_size`] is set from.
///
/// **Chosen because it scales exactly.** A part written twice, once with every
/// coordinate multiplied by a power of two, gets a nominal size multiplied by
/// exactly that power of two — so the tolerance `Model::validate` allows a flat
/// face scales with the part and the two imports answer the same question. An
/// average or a diagonal would round. `a_part_written_at_twice_the_size_is_the_same_part_twice_the_size`
/// is the test.
fn biggest_coordinate_in(file: &StepFile) -> f64 {
    let mut biggest = 0.0_f64;
    for point in file.instances_named("CARTESIAN_POINT") {
        let Some(coordinates) = point.parts()[0].attribute(1).and_then(Value::as_list) else {
            continue;
        };
        for number in coordinates {
            if let Value::Real(written) = number {
                biggest = biggest.max(written.abs());
            }
        }
    }
    biggest
}

/// The shape-carrying entities this reader is not the reader for.
///
/// Named here rather than discovered by a caller wondering where its geometry
/// went. `BREP_WITH_VOIDS` is deliberately absent: it is a solid and it is
/// refused by name.
const SHAPES_THIS_READER_DOES_NOT_TAKE: [&str; 7] = [
    "SHELL_BASED_SURFACE_MODEL",
    "GEOMETRIC_CURVE_SET",
    "GEOMETRIC_SET",
    "TESSELLATED_GEOMETRIC_SET",
    "TESSELLATED_CURVE_SET",
    "COMPLEX_TRIANGULATED_SURFACE_SET",
    "TRIANGULATED_SURFACE_SET",
];

fn what_this_reader_walked_past(file: &StepFile) -> BTreeMap<&'static str, usize> {
    let census = file.how_many_of_each_type();
    SHAPES_THIS_READER_DOES_NOT_TAKE
        .into_iter()
        .filter_map(|keyword| census.get(keyword).map(|&count| (keyword, count)))
        .collect()
}

/// Refuse a file that puts geometry somewhere this reader would have to move it
/// to.
///
/// Three entities can, and each is refused **only when it actually moves
/// something**: a `NEXT_ASSEMBLY_USAGE_OCCURRENCE` always does, because an
/// occurrence is a copy and importing the shape once would be N − 1 solids
/// short; a `MAPPED_ITEM` and an `ITEM_DEFINED_TRANSFORMATION` do only when
/// their placements are not the identity, and a file whose transformations are
/// all identities has nothing to move and is imported.
fn nothing_is_placed_somewhere_this_reader_cannot_move_it(
    reading: &Reading<'_>,
) -> Result<(), Refusal> {
    if let Some(occurrence) = reading
        .file()
        .instances_named("NEXT_ASSEMBLY_USAGE_OCCURRENCE")
        .next()
    {
        return Err(Refusal::APlacementThisReaderCannotApply {
            id: occurrence.id(),
            keyword: "NEXT_ASSEMBLY_USAGE_OCCURRENCE",
        });
    }
    for moved in reading
        .file()
        .instances_named("ITEM_DEFINED_TRANSFORMATION")
    {
        let from = reading.reference_at(moved.id(), "ITEM_DEFINED_TRANSFORMATION", 2)?;
        let to = reading.reference_at(moved.id(), "ITEM_DEFINED_TRANSFORMATION", 3)?;
        if from != to && !(is_the_identity(reading, from)? && is_the_identity(reading, to)?) {
            return Err(Refusal::APlacementThisReaderCannotApply {
                id: moved.id(),
                keyword: "ITEM_DEFINED_TRANSFORMATION",
            });
        }
    }
    for mapped in reading.file().instances_named("MAPPED_ITEM") {
        let source = reading.reference_at(mapped.id(), "MAPPED_ITEM", 1)?;
        let target = reading.reference_at(mapped.id(), "MAPPED_ITEM", 2)?;
        let origin = reading.reference_at(source, "REPRESENTATION_MAP", 0)?;
        // A mapped item both of whose ends are placements is placing a shape.
        // One whose ends are not is placing a **view**: NIST's files map a
        // shape representation from a `CAMERA_MODEL_D3`, which is where a
        // picture is looked at from, and the solids that picture shows are the
        // same ones already read at their own coordinates. Nothing physical is
        // lost by not following it, and following it would put the part in a
        // second place it is not.
        let places_a_shape = [origin, target].into_iter().all(|end| {
            reading
                .file()
                .instance(end)
                .is_some_and(|held| held.part("AXIS2_PLACEMENT_3D").is_some())
        });
        if places_a_shape
            && !(is_the_identity(reading, origin)? && is_the_identity(reading, target)?)
        {
            return Err(Refusal::APlacementThisReaderCannotApply {
                id: mapped.id(),
                keyword: "MAPPED_ITEM",
            });
        }
    }
    Ok(())
}

/// Whether an `AXIS2_PLACEMENT_3D` moves anything: at the origin, pointing up
/// `z`, with its first coordinate along `x`.
///
/// An absent direction is the schema's own default and is the identity's, so
/// this is an equality of exact numbers and never a near-enough.
fn is_the_identity(reading: &Reading<'_>, id: EntityId) -> Result<bool, Refusal> {
    let place = reading.placement_at(id, id)?;
    Ok(place.origin == Point::origin()
        && place.axis == Vector::new(0.0, 0.0, 1.0)
        && place.reference.unwrap_or(Vector::new(1.0, 0.0, 0.0)) == Vector::new(1.0, 0.0, 0.0))
}
