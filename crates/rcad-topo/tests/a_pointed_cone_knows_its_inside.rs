//! A pointed cone's `classify_point`, held against the cone itself.
//!
//! `docs/decisions.md` §200.3 found it answering `Outside` at points inside the
//! cone and refusing `CannotFollowAFacesTrimming` at 3444 more over the grid
//! below. **Two causes, and this file covers the one fixed so far.**
//!
//! 1. **The cone face could not say which side of its cap it lies on.** Its one
//!    corner sits on the cap, and that corner is all `the_side_this_face_lies_on`
//!    reads, so every point asked of the face refused. And once it could say,
//!    the side alone would have held the *other* half of the cone too — the
//!    whole of it lies below the cap. Fixed in `holds.rs`: a face on a cone
//!    keeps to the half its corners are on, and a cone face trimmed by one
//!    closed curve round the axis is the piece reaching the apex.
//! 2. **`SolidView::box_around_it` stops at `z = 4`**, half way up. A cone
//!    face's box is opened out from its corners' shadows on the axis, and a
//!    pointed cone's one corner is on its cap; the apex is not a corner. So
//!    every point below `z = 4` is outside the box and answers `Outside` before
//!    any ray is fired. That is in `view.rs`, which this lane could not edit,
//!    so the points §200.3 named — `(0, 0, 1)`, `(0.1, 0.07, 1.03)` and
//!    `(−1.4, 0.07, 3.03)` — are not here yet, and neither is the fuse that
//!    goes wrong for the same reason.

use rcad_geom::{Point, Scale};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, OnAFace, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// The skin of the cone holds the half it is on and not the half mirrored
/// through the apex, which every plane below the cap puts on the same side.
#[test]
fn the_skin_of_a_pointed_cone_holds_its_own_half_and_not_the_other() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    let view = model.solid(cone).expect("this model made the key");
    let skin = view
        .faces()
        .find(|face| face.surface().as_plane().is_none())
        .expect("a pointed cone has a curved face")
        .key();
    let holds = |at: Point| {
        view.face_holds_the_point(
            skin,
            PredicatePoint::from(at.coordinates()),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the skin of a pointed cone says")
    };

    assert_eq!(holds(Point::new(2.0, 0.0, 4.0)), OnAFace::Held);
    assert_eq!(holds(Point::new(0.0, 1.0, 2.0)), OnAFace::Held);
    assert_eq!(holds(Point::new(-2.0, 0.0, -4.0)), OnAFace::NotHeld);
    assert_eq!(holds(Point::new(0.0, -1.0, -2.0)), OnAFace::NotHeld);
    assert_eq!(holds(Point::origin()), OnAFace::OnItsBoundary);
}

/// Every point of a half-unit grid from half way up the cone to past its top
/// is answered, and answered right.
///
/// **Only from half way up**, because below `z = 4` the solid's box is wrong
/// (the second cause at the top of the file) and the answer there is still
/// `Outside` at every point. Before the first fix, every point here whose ray
/// met the skin was refused, which was 3444 of them.
#[test]
fn every_point_from_half_way_up_a_pointed_cone_is_answered_right() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);

    let tally = classify_the_grid(&model, cone, &policy, |p| p.z() > HEIGHT / 2.0);

    assert_eq!(
        tally,
        Tally {
            inside: 876,
            outside: 9749,
            wrong: 0,
            refused: 0,
        }
    );
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Tally {
    inside: usize,
    outside: usize,
    wrong: usize,
    refused: usize,
}

/// Points half a unit apart over `|x|, |y| ≤ 6`, `|z| ≤ 12`, nudged off the
/// lattice so none lands on the axis, the cap or the skin — the grid §200.3
/// counted over — classified against the cone and held against its closed
/// form: inside exactly where `0 < z < 8` and the distance from the axis is
/// under `z / 2`.
fn classify_the_grid(
    model: &Model,
    cone: SolidKey,
    policy: &Policy,
    keep: impl Fn(Point) -> bool,
) -> Tally {
    let view = model.solid(cone).expect("this model made the key");
    let mut tally = Tally::default();
    for i in -12..=12 {
        for j in -12..=12 {
            for k in -24..=24 {
                let p = Point::new(
                    f64::from(i) * 0.5 + 0.1,
                    f64::from(j) * 0.5 + 0.07,
                    f64::from(k) * 0.5 + 0.03,
                );
                if !keep(p) {
                    continue;
                }
                let inside = 0.0 < p.z() && p.z() < HEIGHT && p.x().hypot(p.y()) < SLOPE * p.z();
                match view.classify_point(p, policy, &mut ProvenanceLog::new()) {
                    Containment::Inside if inside => tally.inside += 1,
                    Containment::Outside if !inside => tally.outside += 1,
                    Containment::Inside | Containment::Outside | Containment::OnBoundary => {
                        tally.wrong += 1;
                    }
                    Containment::Undecidable { .. }
                    | Containment::CannotFollowAFacesTrimming { .. }
                    | Containment::CannotFireARayAtAFacesSurface { .. }
                    | _ => tally.refused += 1,
                }
            }
        }
    }
    tally
}
