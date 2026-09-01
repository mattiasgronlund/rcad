//! Writing a model out and reading it back.
//!
//! **This format is unstable and will break.** It has a version number in it
//! and refuses anything that does not match, so a stale file fails loudly
//! rather than being read as nonsense. Do not archive anything in it. `rkyv`
//! and a format worth keeping are a job for M2, once the data model has stopped
//! moving (decisions.md §6).
//!
//! [`Model`] implements `serde`'s `Serialize` and `Deserialize`, so any
//! self-describing or compact format works. `postcard` is what the tests use:
//!
//! ```
//! use rcad_geom::{Deflection, Scale};
//! use rcad_predicates::{Policy, ProvenanceLog};
//! use rcad_topo::Model;
//!
//! let scale = Scale::millimetres(100.0)?;
//! let mut model = Model::new(scale);
//! let cube = model.add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())?;
//! // A cube is a polyhedron, so this deflection is ignored and the fan is summed.
//! let how_fine = Deflection::for_scale(scale);
//! let before = model
//!     .solid(cube)
//!     .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())?;
//!
//! let bytes = postcard::to_allocvec(&model)?;
//! let read_back: Model = postcard::from_bytes(&bytes)?;
//!
//! read_back.validate(&Policy::certified(), &mut ProvenanceLog::new())?;
//! let solid = read_back.solids().next().expect("the cube came back");
//! assert_eq!(
//!     solid.measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())?,
//!     before
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Note what the round trip proves: going through `serde` means `slotmap`'s
//! internal layout never reaches the file, which is what settles the tension
//! decisions.md §1.3 leaves open between using `slotmap` and the roadmap's
//! claim that the model is flat arrays.
//!
//! The keys a caller is holding do **not** survive the trip. The model that
//! comes back is a different model with its own identity, and in debug builds
//! using an old key against it panics.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::{Model, ModelData};

/// The version stamped into every model written out.
///
/// Reading anything else fails with
/// [`Error::WrongFormatVersion`](crate::Error::WrongFormatVersion). It goes up
/// whenever the stored shape changes, which in M0 and M1 means often.
pub const FORMAT_VERSION: u32 = 1;

impl Serialize for Model {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Written {
            version: FORMAT_VERSION,
            data: self.data(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let payload = Payload::deserialize(deserializer)?;
        if payload.version != FORMAT_VERSION {
            return Err(D::Error::custom(crate::Error::WrongFormatVersion {
                found: payload.version,
                expected: FORMAT_VERSION,
            }));
        }
        Ok(Model::from_parts(payload.data))
    }
}

/// A model on its way out. Borrows, so writing does not copy the arenas.
#[derive(Serialize)]
struct Written<'m> {
    version: u32,
    data: &'m ModelData,
}

/// A model on its way in.
#[derive(Deserialize)]
struct Payload {
    version: u32,
    data: ModelData,
}
