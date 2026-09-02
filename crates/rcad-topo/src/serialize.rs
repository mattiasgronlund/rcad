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
//!
//! **A file that names an entity the arenas do not hold is refused here**, not
//! left for [`Model::validate`] to find. Every level of the model points at the
//! level below by slot, and reading those slots wholesale is what let a
//! stranger's bytes build a solid naming a shell that was not there — a model
//! the STEP writer and the sweep both crashed on rather than refused
//! (`ModelData::check_keys_resolve`).
//! It is the one thing about a model that costs no arithmetic to decide, so
//! nothing downstream has to carry a `Policy` to ask it.
//!
//! **So is a file whose shape is impossible**, which is a different question
//! and was open until the one above was closed. Every key in a face's wire
//! list can resolve while the list is empty; a wire's steps can all name
//! coedges that are there and still not chain up. Those files read back
//! cleanly and then panicked whichever public method next assumed the shape,
//! so `Model::check_shape_is_possible` asks the three questions that cost no
//! arithmetic either — a face has a wire, a wire closes, an edge is walked
//! twice the opposite ways round. Everything else the sweep asks needs a
//! [`Policy`](rcad_predicates::Policy) and stays in [`Model::validate`].

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
///
/// **Three since M4's first lane**, where the same field narrowed from a
/// `Surface` to a [`Picker`](rcad_geom::Picker) (§33). Both write a tag and then
/// four or seven numbers, so a version 2 file read at this shape is again bytes
/// that parse rather than bytes that fail — and worse than that, the flat arm
/// now holds a [`HalfSpace`](rcad_geom::HalfSpace) whose normal is *not* scaled
/// to length one. A version 2 file's plane read as a half-space would come back
/// describing the same flat surface, so the tag is the only thing that can catch
/// it, and it cannot on its own: `Surface::Plane` and `Picker::InFrontOf` are
/// both the leading arm. Hence this number.
pub const FORMAT_VERSION: u32 = 3;

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
        payload
            .data
            .check_keys_resolve()
            .map_err(D::Error::custom)?;
        let model = Model::from_parts(payload.data);
        model.check_shape_is_possible().map_err(D::Error::custom)?;
        Ok(model)
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
    use rcad_geom::{
        Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
        Surface, Vector,
    };
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
            Picker::from(Cylinder::new(beside_the_ball, 2.0).expect("a pipe of radius two"));
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

    /// **A file naming an entity the arenas do not hold does not become a
    /// `Model` at all.**
    ///
    /// This is what closes the family rather than patching one site of it.
    /// Deserializing is the only route to a model whose stored slots resolve to
    /// nothing — no builder can make one and nothing in the crate removes an
    /// entity — so a refusal here is what lets every accessor downstream keep
    /// its `assert!` and every walk of the model keep indexing its arena.
    ///
    /// A cube with one corner taken out, written out and read back: the bytes
    /// are well formed and the version is right, and it is still refused.
    #[test]
    fn a_file_naming_a_corner_that_is_not_there_never_becomes_a_model() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        let corner = model
            .data()
            .vertices
            .keys()
            .next()
            .expect("a cube has corners");
        model.data_mut().vertices.remove(corner);

        let bytes = postcard::to_allocvec(&model).expect("a model writes out");

        assert!(
            postcard::from_bytes::<Model>(&bytes).is_err(),
            "a file whose edges name a corner that is not there must not read back"
        );
    }

    /// **A file whose keys all resolve and whose shape is still impossible does
    /// not become a `Model` either.**
    ///
    /// The three shapes are the three a public method asserts about: a face
    /// with no wires, which
    /// [`FaceView::outer_wire`](crate::FaceView::outer_wire) has nothing to
    /// hand back for; a wire whose steps do not chain up, and an edge walked by
    /// a third coedge, which are what [`Model::place_solid`]'s
    /// after-the-mutation checks assert and cannot have caused.
    ///
    /// Each is built by breaking a sound cube one way, so the bytes are well
    /// formed, the version is right and every stored key names something that
    /// is there — the previous door catches none of them.
    #[test]
    fn a_file_whose_shape_is_impossible_never_becomes_a_model() {
        assert!(
            a_cube_broken_by(|model| {
                let face = model.data().faces.keys().next().expect("a cube has faces");
                model.data_mut().faces[face].wires.clear();
            })
            .is_err(),
            "a face with no wires read back, and FaceView::outer_wire has \
             nothing to hand back for it"
        );
        assert!(
            a_cube_broken_by(|model| {
                let wire = model.data().wires.keys().next().expect("a cube has wires");
                model.data_mut().wires[wire].steps.swap(0, 1);
            })
            .is_err(),
            "a wire that does not close read back, and Model::place_solid \
             asserts that it does"
        );
        assert!(
            a_cube_broken_by(|model| {
                let edge = model.data().edges.keys().next().expect("a cube has edges");
                model.add_coedge(edge, Orientation::Forward);
            })
            .is_err(),
            "an edge walked by three coedges read back, and Model::place_solid \
             asserts there are two"
        );
    }

    /// A sound cube, broken one way, written out and read back.
    fn a_cube_broken_by(break_it: impl FnOnce(&mut Model)) -> postcard::Result<Model> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        break_it(&mut model);

        let bytes = postcard::to_allocvec(&model).expect("a model writes out");
        postcard::from_bytes::<Model>(&bytes)
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

    /// **Why the version had to move, in bytes, and why the tag cannot catch
    /// it.**
    ///
    /// A version 2 file holds `Surface::Plane` where a version 3 file holds
    /// `Picker::InFrontOf`. Both are the leading arm of their enum, so both
    /// write the same tag byte, and both follow it with four numbers — so the
    /// old bytes parse cleanly at the new shape and come back a *different*
    /// solid rather than an error. §4.1 ranks that below a crash, and
    /// [`FORMAT_VERSION`] is the only thing standing between the two.
    ///
    /// What differs is the four numbers themselves, and the assertion below is
    /// that they really do: a plane scales its normal to length one and a
    /// half-space stores what it was handed, so the same normal written both
    /// ways is the same flat surface in different bits.
    #[test]
    fn a_plane_and_a_half_space_facing_the_same_way_are_the_same_bytes_saying_different_numbers() {
        let facing = Vector::new(3.0, -1.0, 0.0);
        let plane =
            Plane::from_point_and_normal(Point::origin(), facing).expect("a normal with length");
        let half_space =
            HalfSpace::through_a_point_facing(Point::origin(), facing).expect("a normal");

        let old_shape =
            postcard::to_allocvec(&Surface::Plane(plane)).expect("a surface writes out");
        let new_shape =
            postcard::to_allocvec(&Picker::InFrontOf(half_space)).expect("a picker writes out");

        assert_eq!(FORMAT_VERSION, 3);
        assert_eq!(
            old_shape[0], new_shape[0],
            "both are the leading arm, so the tag byte cannot tell them apart"
        );
        assert_eq!(
            old_shape.len(),
            new_shape.len(),
            "four numbers either way, so the old bytes parse at the new shape"
        );
        assert_ne!(
            old_shape, new_shape,
            "and say something different, which is the whole reason for the version"
        );
        assert_eq!(
            half_space.normal(),
            facing,
            "a half-space keeps the normal it was handed"
        );
        assert_ne!(
            plane.normal(),
            facing,
            "and a plane scales it to length one"
        );
    }
}
