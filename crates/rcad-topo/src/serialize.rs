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
///
/// Two since M3's gate, where an edge's `picked_by` widened from a plane to any
/// surface (decisions.md §24.5). A plane writes as four numbers and a surface
/// writes as a tag and then its own numbers, so a version 1 file read at this
/// shape would take a plane's leading coefficient for the tag — bytes that parse
/// into a *different* solid rather than bytes that fail, which is the one
/// outcome §4.1 ranks below a crash.
pub const FORMAT_VERSION: u32 = 2;

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

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Cylinder, EdgeRun, Isometry, Plane, Point, Scale, Surface, Vector};
    use rcad_predicates::{Policy, ProvenanceLog};

    use super::FORMAT_VERSION;
    use crate::Model;

    /// A ball whose seam is picked out by a *curved* half-space writes out and
    /// reads back holding the same one.
    ///
    /// The widened field is the whole of what moved the version, so this is the
    /// round trip §24.18's thirteenth criterion asks for. The pipe swallows the
    /// `+x` half of the meridian and leaves the `-x` half outside it, so it
    /// really does pick one — the model that comes back validates, which means
    /// it meshes, which means the seam it names could be found.
    #[test]
    fn a_seam_picked_out_by_a_pipe_comes_back_picked_out_by_the_same_pipe() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");

        let beside_the_ball = Axis::new(Point::new(3.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length");
        let picker =
            Surface::Cylinder(Cylinder::new(beside_the_ball, 2.0).expect("a pipe of radius two"));
        let edge = model.data().edges.keys().next().expect("the seam");
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = &mut model.data_mut().edges[edge].run
        else {
            panic!("a ball's seam is named by a crossing")
        };
        picked_by[0] = picker;

        let bytes = postcard::to_allocvec(&model).expect("a model writes out");
        let after: Model = postcard::from_bytes(&bytes).expect("and reads back");
        after
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a ball whose poles sit outside the pipe beside it is sound");

        let edge = after.data().edges.keys().next().expect("the seam");
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = &after.data().edges[edge].run else {
            panic!("a ball's seam is named by a crossing")
        };
        assert_eq!(picked_by.as_slice(), [picker]);
    }

    /// Every run a computed crossing actually hands back, written out and read
    /// in.
    ///
    /// The test above puts a curved half-space on an edge by hand, which is the
    /// widened field exercised at its widest. This one asks the other question —
    /// what does the intersector *really* produce — and the answer is a run
    /// whose `and` is a pipe and whose `picked_by` is a plane, which nothing had
    /// round-tripped. A ball reaching into the side of a pipe leaves two loops
    /// and names each of them that way.
    ///
    /// An `EdgeRun` on its own rather than one on a model, because a model
    /// holding this run needs corners on the crossing and putting them there is
    /// M4's fuse (§29.13). What the format has to carry is the run.
    #[test]
    fn a_run_the_intersector_hands_back_writes_out_and_reads_in_unchanged() {
        let ball = Surface::Sphere(rcad_geom::Sphere::new(Point::origin(), 3.0).expect("a ball"));
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
                2.0,
            )
            .expect("a pipe"),
        );

        let met = rcad_geom::intersect(
            &ball,
            &pipe,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball threaded on a pipe is named");
        let rcad_geom::SurfaceMeet::Curves(runs) = met else {
            panic!("this pair crosses in two loops, and came back {met:?}")
        };
        assert_eq!(runs.len(), 2, "two loops either side of the centre plane");

        for run in runs {
            let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                panic!("a loop of a quartic is picked out by a half-space")
            };
            assert!(and.as_plane().is_none(), "the second surface is the pipe");
            let bytes = postcard::to_allocvec(run.run()).expect("a run writes out");
            let after: EdgeRun = postcard::from_bytes(&bytes).expect("and reads back");
            let EdgeRun::PartOfWhereItCrosses {
                and: back,
                picked_by: pickers,
            } = &after
            else {
                panic!("it came back a different kind of run")
            };
            assert_eq!(back, and);
            assert_eq!(pickers.as_slice(), picked_by.as_slice());
        }
    }

    /// Why the version had to move, in bytes.
    ///
    /// A plane and a surface holding that plane are not the same run of bytes,
    /// so a version 1 file read at this shape would come apart — quietly, where
    /// the leading coefficient happens to read as a valid tag. The number is
    /// what stops that, and it is only worth having if the shapes really differ.
    #[test]
    fn a_plane_and_a_surface_holding_that_plane_are_different_bytes() {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis is a unit normal");
        let bare = postcard::to_allocvec(&plane).expect("a plane writes out");
        let wrapped = postcard::to_allocvec(&Surface::Plane(plane)).expect("a surface writes out");

        assert_eq!(FORMAT_VERSION, 2);
        assert_ne!(bare, wrapped);
        assert_eq!(wrapped.len(), bare.len() + 1, "one byte of tag in front");
    }
}
