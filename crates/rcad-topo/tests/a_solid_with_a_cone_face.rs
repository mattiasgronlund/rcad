//! Solids with a face on a cone, built through the public door.
//!
//! Two shapes, and the difference between them is the whole of what a cone adds
//! to this kernel.
//!
//! A **frustum** — a cone with its tip cut off — has the same structure as a
//! round bar: two cap circles, a seam up one ruling, and one face on the curved
//! surface. Nothing about it is singular.
//!
//! A **pointed cone** ends at its apex, where the surface pinches to a point and
//! has no normal. That is the same species of shape as a ball's pole, and it is
//! answered with the same machinery: the mesher's `where_this_surface_pinches`
//! says a cone pinches at `v = 0`, and `cut_a_cap_up_to_its_pole` cuts the one
//! loop of the face up to it exactly as it cuts a ball's cap. **A cone's apex
//! needs no seam and no `WireStep::Pole`**, which is where it differs from a
//! ball: a ball's face goes right round *both* coordinates and has to be cut
//! open along a meridian to be a rectangle at all, while a cone's face is
//! bounded below by its own base circle and the apex is one end of the chart
//! rather than a hole in it. So a pointed cone is two faces, one edge and one
//! vertex, and the apex is not a vertex of the model.
//!
//! # Everything here is exact on purpose, and a cone makes that harder
//!
//! `Model::validate` asks that every corner of a curved face be **certified** on
//! its surface, or else be inside the width of the number. A bar gets the first
//! for free: its corner sits at `(r, 0, z)` and `x² + y² − r²` cancels whatever
//! `r` is. A cone's corner has to satisfy `radius = slope · height`, a relation
//! between three stored numbers, so one of them is a product that has to round
//! to nothing. Every cone below has a slope that is a **power of two**, which
//! makes `slope · height` exact for every height `f64` can hold.
//!
//! A slope of `0.1` does not, and the last two tests here are that pair: the
//! corner at `0.1 · 3.0` misses the cone by one coordinate spacing and is
//! **built**, because no `f64` puts it nearer; a corner a hundred times the
//! width of the number off is still refused.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder};

mod cones;

use cones::{a_frustum, a_pointed_cone};

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// A frustum built face by face is a sound solid, and it measures its closed
/// form.
///
/// `V = πh(R² + Rr + r²)/3` and the curved skin is `π(R + r)·slant`, with the
/// two caps on top. **Compared against the closed form and the kernel's own
/// published bound**, not against a number recorded from a previous run: a
/// measurement checked against itself checks nothing.
#[test]
fn a_frustum_measures_the_closed_form_within_the_published_bound() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    // Slope a half, so every corner's radius is an exact product. The cap
    // circles are at radius two and four, four apart up the axis.
    let solid = a_frustum(&mut model, 0.5, 4.0, 8.0);
    model.validate(&policy, &mut log).expect("a sound frustum");

    let (small, big, height) = (2.0f64, 4.0f64, 4.0f64);
    let measured = model
        .solid(solid)
        .measure(
            Deflection::of(0.002).expect("a positive length"),
            &policy,
            &mut log,
        )
        .expect("a frustum measures");

    let volume = core::f64::consts::PI * height * (big * big + big * small + small * small) / 3.0;
    let slant = (height * height + (big - small) * (big - small)).sqrt();
    let area = core::f64::consts::PI * ((big + small) * slant + big * big + small * small);

    println!(
        "frustum: volume {} against {volume} with a bound of {}, area {} against {area} with \
         a bound of {}",
        measured.volume(),
        measured.volume_error(),
        measured.surface_area(),
        measured.area_error(),
    );
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the mesh's volume {} misses πh(R² + Rr + r²)/3 = {volume} by more than the bound {}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.surface_area() - area).abs() <= measured.area_error(),
        "the mesh's area {} misses π(R + r)·slant + πR² + πr² = {area} by more than the bound {}",
        measured.surface_area(),
        measured.area_error()
    );
    // A frustum about `+z` between `z = 4` and `z = 8` has its centroid on the
    // axis, and up the axis at `h(R² + 2Rr + 3r²)/(4(R² + Rr + r²))` from the
    // wide end — which for these numbers is `4 + 4 − 4·(16 + 16 + 12)/(4·28)`.
    let up = height * (big * big + 2.0 * big * small + 3.0 * small * small)
        / (4.0 * (big * big + big * small + small * small));
    let centroid = Point::new(0.0, 0.0, 8.0 - up);
    assert!(
        measured.centroid().distance_to(centroid) <= measured.centroid_wobble(),
        "the centroid {:?} is not {centroid:?} to within {}",
        measured.centroid(),
        measured.centroid_wobble()
    );
}

/// A cone that ends in a point is a sound solid, it meshes, and its volume and
/// lateral area are `πr²h/3` and `πr√(r² + h²)`.
///
/// # Two things about it that a frustum does not have, and both are reported
/// rather than papered over
///
/// **The published error bound on it is infinite.** The bound the kernel
/// publishes for a curved face is an offset-surface argument: project the mesh
/// onto the surface `deviation` inside the material and read the area back out.
/// For a cone that inner surface is the same cone shifted
/// `deviation·√(1 + slope²)/slope` up its own axis — so **within that distance
/// of the apex there is no inner surface at all**, and the smallest radius of
/// normal curvature over the face is zero. `how_tightly_a_cone_bends_over`
/// carries the derivation and names what would close it.
///
/// **And it will not mesh below about a four-hundredth of its own size.** A
/// cone's local radius shrinks to nothing at the apex, so the chord a triangle
/// there may have shrinks with it — while the triangulator's packing guard,
/// which is what makes the refinement terminate at all, will not put two points
/// closer than an eighth of the step the *whole patch* asked for. Measured on
/// this cone, radius four and height eight:
///
/// | deflection | what came back |
/// |---|---|
/// | 0.2, 0.1, 0.05, 0.02, 0.01 | a mesh |
/// | 0.005 | `RanOutOfRoomBeforeMeetingTheDeflection`, still straying 0.0070 |
/// | 0.002 | `RanOutOfRoomBeforeMeetingTheDeflection`, still straying 0.0035 |
///
/// That is the **existing** refusal doing its job on a shape it was written
/// before — a named refusal about the request, not a wrong answer — and
/// `a_pointed_cone_asked_for_a_finer_mesh_than_its_apex_allows_is_refused_by_name`
/// pins it so the limit cannot move unnoticed.
///
/// So this checks the closed form against a **stated share of the deflection**
/// rather than dressing an unpublished number as a bound.
#[test]
fn a_pointed_cone_meshes_and_measures_its_closed_form() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let solid = a_pointed_cone(&mut model, 0.5, 8.0);
    model
        .validate(&policy, &mut log)
        .expect("a sound pointed cone");

    let (radius, height) = (4.0f64, 8.0f64);
    let deflection = 0.01;
    let measured = model
        .solid(solid)
        .measure(
            Deflection::of(deflection).expect("a positive length"),
            &policy,
            &mut log,
        )
        .expect("a pointed cone meshes at a hundredth");

    let volume = core::f64::consts::PI * radius * radius * height / 3.0;
    let area = core::f64::consts::PI
        * (radius * (radius * radius + height * height).sqrt() + radius * radius);
    println!(
        "pointed cone at a deflection of {deflection}: volume {} against {volume}, area {} \
         against {area}, published bounds {} and {}",
        measured.volume(),
        measured.surface_area(),
        measured.volume_error(),
        measured.area_error(),
    );

    assert!(
        !measured.volume_error().is_finite() && !measured.area_error().is_finite(),
        "a face reaching its apex has no offset surface within a deviation of it, and the \
         published bound says so rather than pretending"
    );

    // A statement about *this test* and not a published bound — see the doc
    // above. The mesh is inscribed in a convex solid, so its volume is below the
    // true one and by no more than the deflection times the solid's skin.
    let room = deflection * area;
    assert!(
        measured.volume() <= volume && volume - measured.volume() < room,
        "the mesh's volume {} is inside πr²h/3 = {volume} by more than {room}",
        measured.volume()
    );
    // **Area is not one-sided and a cone is where that shows.** An inscribed
    // triangulation of a ruled surface can have *more* area than the surface —
    // Schwarz's lantern is the classical example, and a cone is a lantern with
    // one radius shrinking. So this is a two-sided comparison where the volume's
    // is one-sided, and asserting `<=` here would be asserting something false.
    assert!(
        (measured.surface_area() - area).abs() < room,
        "the mesh's area {} misses πr√(r² + h²) + πr² = {area} by more than {room}",
        measured.surface_area()
    );
}

/// How fine a pointed cone will mesh, as a measurement rather than a sentence.
///
/// The table in the test above, asserted: a hundredth of the solid's size comes
/// back with a mesh and a four-hundredth comes back with the refusal that says
/// there is nowhere left to put a point. **What it must not do is come back with
/// a mesh that quietly misses**, which is what the `strays` in the refusal is
/// there to rule out.
#[test]
fn a_pointed_cone_asked_for_a_finer_mesh_than_its_apex_allows_is_refused_by_name() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let solid = a_pointed_cone(&mut model, 0.5, 8.0);

    for coarse in [0.2, 0.1, 0.05, 0.02, 0.01] {
        assert!(
            model
                .solid(solid)
                .measure(
                    Deflection::of(coarse).expect("a positive length"),
                    &policy,
                    &mut log
                )
                .is_ok(),
            "a pointed cone meshes at a deflection of {coarse}"
        );
    }
    for fine in [0.005, 0.002] {
        let refused = model.solid(solid).measure(
            Deflection::of(fine).expect("a positive length"),
            &policy,
            &mut log,
        );
        match refused {
            Err(why) => println!("a pointed cone at a deflection of {fine}: {why}"),
            Ok(measured) => panic!(
                "a deflection of {fine} is finer than a cone's apex allows and this answered \
                 with a mesh measuring {}",
                measured.volume()
            ),
        }
    }
}

const A_SLOPE_WHOSE_PRODUCT_ROUNDS: f64 = 0.1;
const HOW_FAR_UP_THE_RIM_SITS: f64 = 3.0;

/// A corner that is a *rounding* off the cone it claims to sit on is built.
///
/// **This test said the opposite until the curved rounding landed**, and the
/// reversal is the point of it. `0.1 · 3.0` is `0.30000000000000004`, one
/// coordinate spacing above `0.3`; the exact predicate certifies that corner
/// off the cone `√(x² + y²) = 0.1z` and always will, because no `f64` on that
/// ray is on the cone at all. Refusing it refused a corner nobody could have
/// written any nearer, which is what 88 of the 161 KiCad files with a corner off
/// a curved face were refused for.
///
/// Nothing here is nudged: the miss is what ordinary `f64` multiplication does
/// to `slope · height`, which is why this pair is worth having beside the
/// synthetic pair in `validate.rs`.
#[test]
fn a_cone_whose_corner_is_a_rounding_off_it_is_built() {
    let mut model = Model::new(a_working_size());
    let rim = A_SLOPE_WHOSE_PRODUCT_ROUNDS * HOW_FAR_UP_THE_RIM_SITS;
    assert_ne!(
        rim, 0.3,
        "a tenth times three does not land on three tenths, which is the whole point"
    );
    assert_eq!(
        (rim - 0.3).abs(),
        f64::from_bits(0.3f64.to_bits() + 1) - 0.3,
        "and it misses by exactly one coordinate spacing"
    );

    a_cone_capped_at_a_rim_of(&mut model, rim)
        .expect("a corner one spacing off is one no writer could improve on");
}

/// And the half that keeps the allowance from being a tolerance.
///
/// A hundred times the width of the number is a corner somebody chose — the
/// budget at this size is `16 · f64::EPSILON · 3`, about `1.1e-14` — and the
/// sweep still says so rather than accepting a face whose own boundary is off
/// it.
#[test]
fn a_cone_whose_corner_is_further_off_than_writing_it_down_could_explain_is_refused_by_the_sweep() {
    let mut model = Model::new(a_working_size());
    let rim = A_SLOPE_WHOSE_PRODUCT_ROUNDS * HOW_FAR_UP_THE_RIM_SITS + 1.0e-12;

    let refused = a_cone_capped_at_a_rim_of(&mut model, rim);
    assert!(
        refused.is_err(),
        "a corner a thousandth of a millionth off its own cone is one somebody \
         chose, and this was accepted"
    );
}

/// A cone under a flat cap, with the one circle they share put at `rim`.
///
/// The cap's plane is square to the axis at the rim's own height, so moving the
/// rim across leaves the cap exactly where it was and the cone's face is the
/// only thing with an opinion.
fn a_cone_capped_at_a_rim_of(
    model: &mut Model,
    rim: f64,
) -> Result<rcad_topo::SolidKey, rcad_topo::Error> {
    let far = HOW_FAR_UP_THE_RIM_SITS;
    let skin = Cone::new(Axis::z(), A_SLOPE_WHOSE_PRODUCT_ROUNDS).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(rim, 0.0, far))
        .expect("a finite point");
    let circle = builder
        .add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed edge");
    let plane = Plane::from_point_and_normal(Point::new(0.0, 0.0, far), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let cap_coedge = builder
        .add_coedge(circle, Orientation::Forward)
        .expect("an edge of this build");
    let cap_wire = builder
        .add_wire(&[DraftStep::Along(cap_coedge)])
        .expect("one closed edge is a wire");
    let cap = builder
        .add_face(Surface::Plane(plane), &[cap_wire])
        .expect("a cap");
    let side_coedge = builder
        .add_coedge(circle, Orientation::Reversed)
        .expect("an edge of this build");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("one closed edge is a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone");
    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");

    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A cone-faced solid written out and read back is the same solid.
#[test]
fn a_model_holding_a_cone_reads_back_at_the_version_that_was_already_there() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let solid = a_frustum(&mut model, 0.5, 4.0, 8.0);
    let how_fine = Deflection::of(0.01).expect("a positive length");
    let before = model
        .solid(solid)
        .measure(how_fine, &policy, &mut log)
        .expect("a frustum measures");

    assert_eq!(
        rcad_topo::FORMAT_VERSION,
        3,
        "a fourth variant on the end of `Surface` takes the next tag and moves nobody, so the \
         version stays where §33 left it — `a_cone_takes_the_next_tag_and_moves_nobody` in \
         `rcad-geom` is the demonstration"
    );

    let bytes = postcard::to_allocvec(&model).expect("a model writes out");
    let after: Model = postcard::from_bytes(&bytes).expect("and reads back");
    after.validate(&policy, &mut log).expect("still sound");

    let back = after.solids().next().expect("the frustum came back");
    assert_eq!(
        back.measure(how_fine, &policy, &mut log)
            .expect("and still measures"),
        before
    );
}

/// A cone's apex is a vertex and has no size, so the dynamic-range check does
/// not read a radius of zero off it.
///
/// Left to itself the smallest radius of a pointed cone face is nothing at all —
/// the apex — and the sweep would refuse every pointed cone ever built for
/// having an infinite ratio between its largest coordinate and its smallest
/// feature. What the radius stands in for is a **closed edge**, whose chord is
/// zero and whose real size is the circle it runs round, and a closed edge never
/// sits at an apex.
#[test]
fn a_pointed_cone_is_not_refused_for_having_a_feature_of_no_size() {
    let mut model = Model::new(a_working_size());
    a_pointed_cone(&mut model, 0.5, 8.0);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the apex is a corner and a corner has no size");
}

/// A cone-faced solid can be turned and still measures the same.
///
/// §19's rule, over a surface kind that did not exist when it was written: where
/// a solid sits may not change what it measures.
#[test]
fn a_frustum_turned_about_its_own_axis_measures_what_it_did_before() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let solid = a_frustum(&mut model, 0.5, 4.0, 8.0);
    let how_fine = Deflection::of(0.01).expect("a positive length");
    let before = model
        .solid(solid)
        .measure(how_fine, &policy, &mut log)
        .expect("a frustum measures");

    let turned = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_2)
        .expect("a quarter turn about z");
    let mut moved = Model::new(a_working_size());
    let there = a_frustum(&mut moved, 0.5, 4.0, 8.0);
    moved.place_solid(there, &turned);
    let after = moved
        .solid(there)
        .measure(how_fine, &policy, &mut log)
        .expect("and so does the turned one");

    println!(
        "a frustum turned a quarter turn: volume {} against {}, area {} against {}",
        after.volume(),
        before.volume(),
        after.surface_area(),
        before.surface_area(),
    );
    // **Not bit for bit, and the cone is not why.** A turned face is meshed in a
    // frame of its own (§41.11 item 19) and its points are worked out from
    // different numbers, so the two meshes are different meshes of the same
    // solid. What §19 asks is that they measure the same *solid*, and the
    // published bounds are what says how closely two meshes of one solid have to
    // agree.
    assert!(
        (after.volume() - before.volume()).abs() <= after.volume_error() + before.volume_error(),
        "two meshes of one frustum measure {} and {}, further apart than their own bounds",
        after.volume(),
        before.volume()
    );
    assert!(
        (after.surface_area() - before.surface_area()).abs()
            <= after.area_error() + before.area_error(),
        "two meshes of one frustum have areas {} and {}, further apart than their own bounds",
        after.surface_area(),
        before.surface_area()
    );
}
