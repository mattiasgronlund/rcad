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
/// solid. **The shape-carrying rest of that list is counted rather than
/// forgotten**: [`Import::shapes_this_reader_did_not_take`] names every
/// shape-carrying entity the file held that this reader is not the reader for,
/// with how many of each. Styling is not shape-carrying and is not on that
/// list either — this reader parses none of it, counted or not, so a file
/// full of colour and no geometry reports zero shapes walked past. A
/// `BREP_WITH_VOIDS` is *not* on that list: it is a solid, this kernel holds
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
    ///
    /// **Does not count styling.** Colour, tolerance and annotation entities
    /// carry no shape and never appear here, at any count — this reader does
    /// not parse them, so their absence from this map says nothing about
    /// whether the file had any.
    #[must_use]
    pub fn shapes_this_reader_did_not_take(&self) -> &BTreeMap<&'static str, usize> {
        &self.walked_past
    }
}

/// The largest coordinate magnitude of any corner the file's solids actually
/// have, which is what [`Scale::nominal_size`] is set from.
///
/// **Walks `VERTEX_POINT`, not every `CARTESIAN_POINT` in the file.** A file
/// holds many points that name no corner of anything: a `LINE`'s own
/// defining point (some tools put it near a vertex it supports, not on it),
/// a `PLANE`'s placement origin, a `CIRCLE`'s centre. None of those bound the
/// solid, so a file that carries one far from every real corner must not move
/// the nominal size — measured on
/// `PinHeader_2x01_P2.54mm_Vertical.step`, whose `LINE` reference point sits
/// at `8.646666666667` on one axis while every vertex on that axis tops out
/// at `8.54`, a 1.2% difference with no shape behind it.
/// `a_lines_own_reference_point_does_not_move_the_nominal_size` is the test.
///
/// **Chosen because it scales exactly.** A part written twice, once with every
/// coordinate multiplied by a power of two, gets a nominal size multiplied by
/// exactly that power of two — so the tolerance `Model::validate` allows a flat
/// face scales with the part and the two imports answer the same question. An
/// average or a diagonal would round. `a_part_written_at_a_power_of_two_scale_is_exactly_the_same_part_at_that_scale`
/// is the test.
fn biggest_coordinate_in(file: &StepFile) -> f64 {
    let mut biggest = 0.0_f64;
    for vertex in file.instances_named("VERTEX_POINT") {
        let Some(point) = vertex
            .part("VERTEX_POINT")
            .and_then(|record| record.attribute(1))
            .and_then(Value::as_reference)
            .and_then(|id| file.instance(id))
        else {
            continue;
        };
        let Some(coordinates) = point
            .part("CARTESIAN_POINT")
            .and_then(|record| record.attribute(1))
            .and_then(Value::as_list)
        else {
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

#[cfg(test)]
mod tests {
    use super::biggest_coordinate_in;
    use crate::graph::StepFile;

    /// A file with one corner at `(1.0, 2.0, 3.0)` and, unconnected to any
    /// vertex, a lone `CARTESIAN_POINT` far past it — the same shape a real
    /// file's `LINE` reference point takes: a defining point for a curve,
    /// sitting near but not on the corner it supports.
    fn a_file_whose_only_corner_is_small(far_point: [f64; 3]) -> String {
        format!(
            "ISO-10303-21;\n\
             HEADER;\n\
             FILE_DESCRIPTION((''),'2;1');\n\
             FILE_NAME('','',(''),(''),'','','');\n\
             FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\n\
             ENDSEC;\n\
             DATA;\n\
             #1=CARTESIAN_POINT('',(1.0,2.0,3.0));\n\
             #2=VERTEX_POINT('',#1);\n\
             #3=CARTESIAN_POINT('',({:?},{:?},{:?}));\n\
             ENDSEC;\n\
             END-ISO-10303-21;\n",
            far_point[0], far_point[1], far_point[2]
        )
    }

    /// The measured defect: `PinHeader_2x01_P2.54mm_Vertical.step` holds a
    /// `LINE` whose own defining point is `(5.333333333333E-02,
    /// 5.333333333333E-02, 8.646666666667)`, sitting near but not on the
    /// vertex `(0.16, 0.16, 8.54)` it supports. Before this fix,
    /// `Scale::nominal_size` came back `8.646666666667`; the part's own
    /// corners top out at `8.54`.
    #[test]
    fn a_lines_own_reference_point_does_not_move_the_nominal_size() {
        let text = a_file_whose_only_corner_is_small([
            5.333_333_333_333E-02,
            5.333_333_333_333E-02,
            8.646_666_666_667,
        ]);
        let file = StepFile::read(text.as_bytes()).expect("a small hand-written file reads");

        assert_eq!(
            biggest_coordinate_in(&file),
            3.0,
            "the biggest coordinate must come from the one corner the file has, not from a \
             point that names no vertex"
        );
    }

    /// A `CARTESIAN_POINT` with no `VERTEX_POINT` at all — a placement origin,
    /// say — is walked past the same way.
    #[test]
    fn a_point_with_no_vertex_at_all_does_not_move_the_nominal_size() {
        let text = a_file_whose_only_corner_is_small([1000.0, 1000.0, 1000.0]);
        let file = StepFile::read(text.as_bytes()).expect("a small hand-written file reads");

        assert_eq!(biggest_coordinate_in(&file), 3.0);
    }
}
