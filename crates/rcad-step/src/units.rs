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
use crate::refusal::Refusal;

/// What one file declared.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Declared {
    pub(crate) unit: Unit,
    pub(crate) uncertainty: Option<f64>,
    pub(crate) how_many_uncertainties: usize,
}

/// Read the length unit and the declared uncertainty out of the file's
/// geometric contexts.
pub(crate) fn what_the_file_declared(reading: &Reading<'_>) -> Result<Declared, Refusal> {
    let mut unit: Option<Unit> = None;
    let mut uncertainty: Option<f64> = None;
    let mut how_many_uncertainties = 0;

    for context in reading.file().instances() {
        if context.part("GLOBAL_UNIT_ASSIGNED_CONTEXT").is_some() {
            let named = reading.references_at(context.id(), "GLOBAL_UNIT_ASSIGNED_CONTEXT", 0)?;
            for one in named {
                let Some(found) = length_unit_at(reading, one)? else {
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
            .part("GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT")
            .is_some()
        {
            let declared =
                reading.references_at(context.id(), "GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT", 0)?;
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
    if held.part("LENGTH_UNIT").is_none() {
        return Ok(None);
    }
    if held.part("SI_UNIT").is_some() {
        return si_length_unit_at(reading, id).map(Some);
    }
    if held.part("CONVERSION_BASED_UNIT").is_some() {
        return converted_length_unit_at(reading, id).map(Some);
    }
    Err(Refusal::ALengthUnitThisKernelDoesNotHave {
        id,
        described: held.parts()[0].keyword().to_owned(),
    })
}

fn si_length_unit_at(reading: &Reading<'_>, id: EntityId) -> Result<Unit, Refusal> {
    let record = reading.record(id, "SI_UNIT")?;
    let name = record.attribute(1).and_then(crate::Value::as_enumeration);
    if name != Some("METRE") {
        return Err(Refusal::ALengthUnitThisKernelDoesNotHave {
            id,
            described: format!("SI unit {}", name.unwrap_or("with no name")),
        });
    }
    match record.attribute(0) {
        Some(crate::Value::Nothing) => Ok(Unit::Metre),
        Some(crate::Value::Enumeration(prefix)) => match prefix.as_str() {
            "MICRO" => Ok(Unit::Micrometre),
            "MILLI" => Ok(Unit::Millimetre),
            "CENTI" => Ok(Unit::Centimetre),
            other => Err(Refusal::ALengthUnitThisKernelDoesNotHave {
                id,
                described: format!("{other} metre"),
            }),
        },
        _ => Err(Refusal::AnAttributeIsNotWhatTheSchemaWants {
            id,
            keyword: "SI_UNIT",
            position: 0,
            wanted: "prefix",
        }),
    }
}

fn converted_length_unit_at(reading: &Reading<'_>, id: EntityId) -> Result<Unit, Refusal> {
    let measure = reading.reference_at(id, "CONVERSION_BASED_UNIT", 1)?;
    let says = reading.keyword_of(
        id,
        measure,
        &["LENGTH_MEASURE_WITH_UNIT", "MEASURE_WITH_UNIT"],
    )?;
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
    if held.part("UNCERTAINTY_MEASURE_WITH_UNIT").is_none() {
        return Ok(None);
    }
    let of = reading.reference_at(id, "UNCERTAINTY_MEASURE_WITH_UNIT", 1)?;
    if length_unit_at(reading, of)?.is_none() {
        return Ok(None);
    }
    reading
        .real_at(id, "UNCERTAINTY_MEASURE_WITH_UNIT", 0)
        .map(Some)
}
