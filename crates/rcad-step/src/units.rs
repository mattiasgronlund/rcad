//! What the file says its numbers mean, and how sure it says they are.
//!
//! **Nothing here multiplies anything.** [`Scale`](rcad_geom::Scale) carries a
//! unit and never takes part in arithmetic, and two models in different units
//! are two different models (`docs/decisions.md` §3.5) — so a file written in
//! inches comes back in inches, with the numbers the file wrote, and a unit this
//! kernel has no name for is refused rather than converted into one it does.
//! That is §42.12 criterion 6, and the reason it costs nothing is the decision
//! and not the arithmetic: a conversion would round, and there is no conversion.

use rcad_geom::Unit;

use crate::geometry::Reading;
use crate::graph::EntityId;
use crate::names::{
    CENTI, CONVERSION_BASED_UNIT, EnumerationName, GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT,
    GLOBAL_UNIT_ASSIGNED_CONTEXT, LENGTH_MEASURE_WITH_UNIT, LENGTH_UNIT, MEASURE_WITH_UNIT, METRE,
    MICRO, MILLI, PLANE_ANGLE_MEASURE_WITH_UNIT, PLANE_ANGLE_UNIT, RADIAN, SI_UNIT,
    UNCERTAINTY_MEASURE_WITH_UNIT,
};
use crate::refusal::Refusal;

/// What one file declared.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Declared {
    pub(crate) unit: Unit,
    /// How many radians one of the file's own plane-angle units is, or `None`
    /// where the file declares one this reader cannot read.
    ///
    /// **The file's own number, not this reader's.** A `CONVERSION_BASED_UNIT`
    /// called `'DEGREE'` writes the factor out —
    /// `PLANE_ANGLE_MEASURE(0.0174532925199433)` — and using that rather than a
    /// `PI / 180.0` computed here keeps the reader from disagreeing with the
    /// file about what a degree is. One `CONICAL_SURFACE` attribute in the whole
    /// of this reader needs it, and `None` is only a refusal if a file holds
    /// one.
    pub(crate) radians_per_angle_unit: Option<f64>,
    pub(crate) uncertainty: Option<f64>,
    pub(crate) how_many_uncertainties: usize,
}

/// Read the length unit and the declared uncertainty out of the file's
/// geometric contexts.
pub(crate) fn what_the_file_declared(reading: &Reading<'_>) -> Result<Declared, Refusal> {
    let mut unit: Option<Unit> = None;
    let mut radians_per_angle_unit: Option<f64> = None;
    let mut uncertainty: Option<f64> = None;
    let mut how_many_uncertainties = 0;

    for context in reading.file().instances() {
        if context.part(GLOBAL_UNIT_ASSIGNED_CONTEXT.name()).is_some() {
            let named = reading.references_at(context.id(), GLOBAL_UNIT_ASSIGNED_CONTEXT, 0)?;
            for one in named {
                let Some(found) = length_unit_at(reading, one)? else {
                    radians_per_angle_unit =
                        radians_per_angle_unit.or(how_big_an_angle_unit_is(reading, one));
                    continue;
                };
                match unit {
                    None => unit = Some(found),
                    Some(already) if already == found => {}
                    Some(already) => {
                        return Err(Refusal::TheFileDeclaresTwoDifferentLengthUnits {
                            first: already,
                            second: found,
                        });
                    }
                }
            }
        }
        if context
            .part(GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT.name())
            .is_some()
        {
            let declared =
                reading.references_at(context.id(), GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT, 0)?;
            for one in declared {
                let Some(measure) = length_uncertainty_at(reading, one)? else {
                    continue;
                };
                how_many_uncertainties += 1;
                uncertainty = Some(uncertainty.map_or(measure, |worst: f64| worst.max(measure)));
            }
        }
    }

    Ok(Declared {
        unit: unit.ok_or(Refusal::TheFileDeclaresNoLengthUnit)?,
        radians_per_angle_unit,
        uncertainty,
        how_many_uncertainties,
    })
}

/// The [`Unit`] a `NAMED_UNIT` names, or `None` if it is not a length at all.
///
/// Two spellings, and both are matched **exactly**. An `SI_UNIT` is a prefix and
/// a name, so `.MILLI.` `.METRE.` is a millimetre and nothing rounds. A
/// `CONVERSION_BASED_UNIT` is a number and another unit, and the pair is looked
/// up rather than multiplied: `25.4` millimetres is an inch because the two
/// numbers are equal as `f64`, not because anything was scaled. A prefix or a
/// factor outside the table is refused by name and carries what the file said.
fn length_unit_at(reading: &Reading<'_>, id: EntityId) -> Result<Option<Unit>, Refusal> {
    let Some(held) = reading.file().instance(id) else {
        return Ok(None);
    };
    if held.part(LENGTH_UNIT.name()).is_none() {
        return Ok(None);
    }
    if held.part(SI_UNIT.name()).is_some() {
        return si_length_unit_at(reading, id).map(Some);
    }
    if held.part(CONVERSION_BASED_UNIT.name()).is_some() {
        return converted_length_unit_at(reading, id).map(Some);
    }
    Err(Refusal::ALengthUnitThisKernelDoesNotHave {
        id,
        described: held.parts()[0].keyword().to_owned(),
    })
}

fn si_length_unit_at(reading: &Reading<'_>, id: EntityId) -> Result<Unit, Refusal> {
    let record = reading.record(id, SI_UNIT)?;
    let name = record.attribute(1).and_then(crate::Value::as_enumeration);
    if name.and_then(EnumerationName::of) != Some(METRE) {
        return Err(Refusal::ALengthUnitThisKernelDoesNotHave {
            id,
            described: format!("SI unit {}", name.unwrap_or("with no name")),
        });
    }
    match record.attribute(0) {
        Some(crate::Value::Nothing) => Ok(Unit::Metre),
        Some(crate::Value::Enumeration(prefix)) => match EnumerationName::of(prefix) {
            Some(MICRO) => Ok(Unit::Micrometre),
            Some(MILLI) => Ok(Unit::Millimetre),
            Some(CENTI) => Ok(Unit::Centimetre),
            _ => Err(Refusal::ALengthUnitThisKernelDoesNotHave {
                id,
                described: format!("{prefix} metre"),
            }),
        },
        _ => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
            id,
            keyword: SI_UNIT.name(),
            position: 0,
            wanted: "prefix",
        }),
    }
}

fn converted_length_unit_at(reading: &Reading<'_>, id: EntityId) -> Result<Unit, Refusal> {
    let measure = reading.reference_at(id, CONVERSION_BASED_UNIT, 1)?;
    let says = reading.keyword_of(id, measure, &[LENGTH_MEASURE_WITH_UNIT, MEASURE_WITH_UNIT])?;
    let factor = reading.real_at(measure, says, 0)?;
    let of = reading.reference_at(measure, says, 1)?;
    let base = length_unit_at(reading, of)?.ok_or(Refusal::ALengthUnitThisKernelDoesNotHave {
        id,
        described: "a conversion from something that is not a length".to_owned(),
    })?;

    // Every row is an exact `f64` pair, and the match is an equality and not a
    // multiplication. Two spellings of an inch that differ in the last bit are
    // two different units, and this says so rather than rounding them together.
    let table = [
        (1.0, base, base),
        (25.4, Unit::Millimetre, Unit::Inch),
        (2.54, Unit::Centimetre, Unit::Inch),
        (0.0254, Unit::Metre, Unit::Inch),
        (304.8, Unit::Millimetre, Unit::Foot),
        (30.48, Unit::Centimetre, Unit::Foot),
        (0.3048, Unit::Metre, Unit::Foot),
        (12.0, Unit::Inch, Unit::Foot),
    ];
    table
        .into_iter()
        .find(|&(number, from, _)| number == factor && from == base)
        .map(|(_, _, becomes)| becomes)
        .ok_or_else(|| Refusal::ALengthUnitThisKernelDoesNotHave {
            id,
            described: format!("{factor} {}", base.symbol()),
        })
}

/// How many radians one of this unit is, when it is a plane angle at all.
///
/// Radians are one, and a `CONVERSION_BASED_UNIT` from radians is whatever
/// number the file wrote. Anything else — an angle unit converted from
/// something that is not a radian, or a measure this reader cannot read —
/// is `None`, and `None` only refuses a file that has a cone in it.
fn how_big_an_angle_unit_is(reading: &Reading<'_>, id: EntityId) -> Option<f64> {
    let held = reading.file().instance(id)?;
    held.part(PLANE_ANGLE_UNIT.name())?;
    if held.part(SI_UNIT.name()).is_some() {
        let name = reading
            .record(id, SI_UNIT)
            .ok()?
            .attribute(1)
            .and_then(crate::Value::as_enumeration)?;
        return (EnumerationName::of(name) == Some(RADIAN)).then_some(1.0);
    }
    let measure = reading.reference_at(id, CONVERSION_BASED_UNIT, 1).ok()?;
    let says = reading
        .keyword_of(
            id,
            measure,
            &[PLANE_ANGLE_MEASURE_WITH_UNIT, MEASURE_WITH_UNIT],
        )
        .ok()?;
    let factor = reading.real_at(measure, says, 0).ok()?;
    let of = reading.reference_at(measure, says, 1).ok()?;
    (how_big_an_angle_unit_is(reading, of)? == 1.0).then_some(factor)
}

/// The value of an `UNCERTAINTY_MEASURE_WITH_UNIT`, when it is a length.
///
/// **Read and reported, never acted on.** M5 heals nothing, so a file that says
/// its numbers are good to a micron does not thereby buy a micron of tolerance
/// anywhere: every predicate is asked the same question it would have been
/// asked if the file had said nothing (§42.9 option A).
fn length_uncertainty_at(reading: &Reading<'_>, id: EntityId) -> Result<Option<f64>, Refusal> {
    let Some(held) = reading.file().instance(id) else {
        return Ok(None);
    };
    if held.part(UNCERTAINTY_MEASURE_WITH_UNIT.name()).is_none() {
        return Ok(None);
    }
    let of = reading.reference_at(id, UNCERTAINTY_MEASURE_WITH_UNIT, 1)?;
    if length_unit_at(reading, of)?.is_none() {
        return Ok(None);
    }
    reading
        .real_at(id, UNCERTAINTY_MEASURE_WITH_UNIT, 0)
        .map(Some)
}
