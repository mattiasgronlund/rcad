//! Classifying a point against a solid with a torus face used to refuse
//! outright, whatever the point was.
//!
//! `rcad-topo`'s own `where_the_ray_meets` turned away every torus face before
//! asking anything about the particular ray it was given —
//! `count_where_the_ray_meets_it` checked the surface's *kind* and returned
//! [`Containment::CannotFireARayAtAFacesSurface`] before building a single box.
//! `rcad_geom::where_a_line_meets_a_surface` asks the real question instead: a
//! torus needs its line's coordinates exact
//! (`Error::WhereALineMeetsARingNeedsTheLineExactly`) and only refuses then.
//!
//! [`SolidView::classify_point`] always hands over an exact point — "a corner
//! on the lattice", in the words `rcad-geom`'s own `where_a_line_meets_a_surface`
//! doc uses for one, as opposed to the box a fuse's arrangement builds — so its
//! ray's line is exact too, and switching to the general answer decides
//! classification that the surface-kind check used to refuse unconditionally.

use rcad_geom::{Point, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model};

mod rings;
use rings::a_quarter_bend;

/// A point through the hole of a quarter bend of pipe classifies `Outside`.
///
/// Before this closure, classifying *any* point against this solid refused
/// with [`Containment::CannotFireARayAtAFacesSurface`], because the ring face
/// alone was enough to turn every ray away before its line was ever looked at.
#[test]
fn a_point_through_the_hole_of_a_bend_is_decided_rather_than_refused() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_quarter_bend(&mut model, 4.0, 1.0);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bend");

    // Well inside the ring's hole: the ring has radius 4 and tube radius 1, so
    // its material starts at distance 3 from the axis, and this point is at
    // distance √2.
    let point = Point::new(1.0, 1.0, 0.0);
    let mut log = ProvenanceLog::new();
    let result = model
        .solid(solid)
        .classify_point(point, &Policy::certified(), &mut log);

    assert_eq!(
        result,
        Containment::Outside,
        "a point through the hole of the ring should classify Outside, not {result:?}"
    );
}
