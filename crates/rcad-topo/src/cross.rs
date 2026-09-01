//! Where two faces of a model cross, with the bookkeeping the geometry half
//! cannot name.
//!
//! `docs/decisions.md` §24.9 is §23.6 arriving a second time: two separately
//! settled decisions that turn out to be jointly impossible. A meet has to carry
//! which *input edges* a piece of it runs along, `EdgeKey` lives here, and
//! `rcad-geom` sits below this crate and cannot name one. The same two-level
//! shape resolves it — `rcad-geom` answers with a topology-free
//! [`SurfaceMeet`], and this wraps it.
//!
//! # Deliberately thin
//!
//! `along` and nothing else. §27.3 settled that the fuse's own `FaceMeet` — a
//! plane–plane meet whose ends are `MeetPoint`s naming input vertices — did not
//! move and did not change, and that **M4 is where a fuse over curved crossings
//! is the first consumer that has to merge the two**. Building the merged type
//! here would be settling that twice, which is §13 rule 4's hazard exactly.
//! `pub(crate)`, per §24.18's fourteenth criterion.
//!
//! # Nothing in this crate calls it yet
//!
//! Its consumer is named and it is not in this milestone: a fuse over curved
//! crossings, which is M4's, and §24.19 keeps `classify` and `fuse` gated on
//! `is_wholly_flat` until then. §24.4 puts the split at M3's gate anyway,
//! because `rcad-geom` cannot name an `EdgeKey` and the two halves have to be
//! settled together or twice. The tests below are what exercise it in the
//! meantime.
#![allow(dead_code)]

use rcad_geom::{SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::SmallVec;

use crate::view::FaceView;
use crate::{EdgeKey, Result};

/// Where two faces cross, and which edges the model already holds along it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CrossedFaces {
    met: SurfaceMeet,
    along: SmallVec<[EdgeKey; 1]>,
}

impl CrossedFaces {
    /// What the two surfaces do, with no topology in it.
    pub(crate) fn met(&self) -> &SurfaceMeet {
        &self.met
    }

    /// The edges the model already holds along this crossing.
    ///
    /// An edge is on it when **both** faces use it, which is what makes it a
    /// piece of their crossing that has already been named once. Empty for two
    /// faces that share no edge, which is most pairs in a fuse — and for two
    /// faces of two different models, which share nothing by definition.
    pub(crate) fn along(&self) -> &[EdgeKey] {
        &self.along
    }
}

impl<'m> FaceView<'m> {
    /// Cross this face's surface with `other`'s.
    ///
    /// **Both locations are composed once, here.** `FaceView::surface` is
    /// `surface.transformed(&location)` on every read, and that function walks a
    /// plane's offset an ulp per read and rescales an almost-unit normal; an
    /// intersector that read its own faces would pay it per pair, per fuse
    /// (§19.1, §24.8). So what goes down to `rcad-geom` is two `Surface`s and
    /// never two `FaceView`s.
    ///
    /// # Errors
    ///
    /// Whatever [`intersect`] makes of the pair, unchanged. A crossing that
    /// could not be named is not a crossing this can bookkeep.
    ///
    pub(crate) fn crossed_with(
        self,
        other: FaceView<'m>,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<CrossedFaces> {
        let met = intersect(&self.surface(), &other.surface(), policy, log)?;
        Ok(CrossedFaces {
            met,
            along: self.edges_shared_with(other),
        })
    }

    /// The edges both faces use, once each, in the first face's stored order.
    fn edges_shared_with(self, other: FaceView<'m>) -> SmallVec<[EdgeKey; 1]> {
        let theirs: Vec<EdgeKey> = other.edge_keys().collect();
        let mut shared: SmallVec<[EdgeKey; 1]> = SmallVec::new();
        for edge in self.edge_keys() {
            if theirs.contains(&edge) && !shared.contains(&edge) {
                shared.push(edge);
            }
        }
        shared
    }

    /// Every edge this face runs along, in wire order and with repeats — a wire
    /// that closes on one edge uses it twice.
    fn edge_keys(self) -> impl Iterator<Item = EdgeKey> + 'm {
        self.wires()
            .flat_map(|wire| wire.coedges().map(|coedge| coedge.edge().key()))
    }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{EdgeRun, Isometry, Plane, Point, Scale, Surface, SurfaceMeet, Vector};
    use rcad_predicates::{Policy, ProvenanceLog};

    use crate::{Model, SolidKey};

    fn a_round_bar() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(3.0, 10.0, &Isometry::identity())
            .expect("a bar of radius three");
        (model, bar)
    }

    /// §24.18's second criterion, asked of the shape M3 exists to compute.
    ///
    /// A round bar's cap circle is an edge the model **declares**. Crossing the
    /// two faces either side of it names the same thing from the surfaces alone,
    /// with nothing declared — and the crossing knows the edge is already there,
    /// which is the whole of what `along` adds.
    #[test]
    fn crossing_the_two_faces_along_a_cap_circle_names_the_circle_and_the_edge() {
        let (model, bar) = a_round_bar();
        let faces: Vec<_> = model.solid(bar).faces().collect();
        let wall = faces
            .iter()
            .find(|face| matches!(face.surface(), Surface::Cylinder(_)))
            .expect("a bar has a wall");
        let cap = faces
            .iter()
            .find(|face| face.surface().as_plane().is_some())
            .expect("a bar has two caps");

        let mut log = ProvenanceLog::new();
        let crossed = wall
            .crossed_with(*cap, &Policy::certified(), &mut log)
            .expect("a pipe and a plane square to it cross in a circle");
        assert!(log.is_empty(), "nothing here needs approximating: {log:?}");

        let SurfaceMeet::Curves(runs) = crossed.met() else {
            panic!(
                "a cap plane cuts a bar's wall in one circle: {:?}",
                crossed.met()
            );
        };
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run(), &EdgeRun::RightRoundWhereItsFacesCross);

        assert_eq!(
            crossed.along().len(),
            1,
            "the model already holds this circle as one edge: {:?}",
            crossed.along()
        );
        let already = model.edge(crossed.along()[0]);
        assert_eq!(
            already.run(),
            &EdgeRun::RightRoundWhereItsFacesCross,
            "the edge the model declared and the crossing just named are the same run"
        );
    }

    /// Two faces that share no edge name a crossing with no bookkeeping on it,
    /// which is what most pairs in a fuse are.
    #[test]
    fn two_faces_that_share_no_edge_name_a_crossing_along_nothing() {
        let (model, bar) = a_round_bar();
        let caps: Vec<_> = model
            .solid(bar)
            .faces()
            .filter(|face| face.surface().as_plane().is_some())
            .collect();
        assert_eq!(caps.len(), 2, "a bar has two caps");

        let crossed = caps[0]
            .crossed_with(caps[1], &Policy::certified(), &mut ProvenanceLog::new())
            .expect("two parallel planes are named without arithmetic");
        assert_eq!(crossed.met(), &SurfaceMeet::Nothing);
        assert!(crossed.along().is_empty());
    }

    /// A ball's seam has the same sphere on both sides, so crossing the face
    /// with itself is two dimensional — and the run on the edge is the only
    /// thing that says which meridian it is.
    #[test]
    fn crossing_a_balls_one_face_with_itself_is_the_same_surface_twice() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let ball = model
            .add_ball(3.0, &Isometry::identity())
            .expect("a ball of radius three");
        let skin = model
            .solid(ball)
            .faces()
            .next()
            .expect("a ball has one face");

        let crossed = skin
            .crossed_with(skin, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a sphere against itself is named without arithmetic");
        assert_eq!(crossed.met(), &SurfaceMeet::SameSurface);
        assert_eq!(
            crossed.along().len(),
            1,
            "the seam is the one edge both sides of this face use"
        );

        // And what the seam actually runs along comes off the edge, not off the
        // pair: the meridian plane, which no crossing of the sphere with itself
        // could ever name.
        let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal");
        let EdgeRun::PartOfWhereItCrosses { and, .. } = model.edge(crossed.along()[0]).run() else {
            panic!("a ball's seam is part of where its face crosses a meridian");
        };
        assert_eq!(and, &Surface::Plane(meridian));
    }
}
