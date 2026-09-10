//! A ball and a ring are measured from their own centre, not from a corner.
//!
//! `how_far_the_points_of_reach` says how far out a face gets, and everything
//! about how finely that face can be meshed is that number times a constant. A
//! **pipe** and a **cone** already read it off where their **edges run**; a
//! **ball** and a **ring** used to read it off where their **corners sit**, as
//! `corner + radius` and `corner + R + r`.
//!
//! A corner is a place on the surface and not the middle of it, so that answer
//! moved when the trimming moved and the middle did not. Both shapes here show
//! it, and neither is checked against a number recorded from a previous run:
//!
//! - **A ball a quarter of a radius off the origin** used to be given exactly
//!   the floor of one sitting on it, bit for bit, and so did one half a radius
//!   off and one a whole radius off. A unit ball's poles are a unit out whatever
//!   the ball is doing, so `corner` stuck at `1` until the centre had gone
//!   further out than that, and the four balls were charged one number.
//! - **A bend of pipe whose tube is fatter** used to be given exactly the floor
//!   of a thinner one round the same ring. A bend's two corners sit on the inner
//!   equator at `R − r`, so `corner + R + r` is `2R` for every tube there is,
//!   and the tube fell out of the answer altogether.
//!
//! Measured from the centre both of those go away, and the second gets an oracle
//! with no kernel in it: a ring reaches `R + r` and nothing else, so a bend's
//! floor divided by `R + r` is one constant across every tube.
//!
//! # What the corner charge cost, and in which direction
//!
//! It read **too low by at most two**: any two points of a bounded surface are
//! within `2 · radius` of each other in each coordinate, so a face reaches at
//! most `corner + 2 · radius` while it was charged `corner + radius`, and the
//! ratio is worst when the corner is on the origin.
//!
//! **No shape this kernel builds today reaches that**, and both of these read
//! too *high* instead. A ball's poles sit at `centre ± radius · axis`, so one of
//! the two has a largest coordinate at least the centre's; a bend's corners sit
//! at `centre + (R − r)` along each of two square directions, so between them
//! the same holds. Either way `corner + radius ≥ |centre| + radius`, which is
//! already the whole surface — the charge was above the truth, by two on a unit
//! ball at the origin and by eight fifths on a bend of ring four and tube one.
//! Reading it too low wants a face whose corners **all** sit nearer the origin
//! than its centre does, which takes a trimmed ball or a trimmed ring, and the
//! only doors to one are a fuse and a hand-written solid.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Mesh, Model, SolidKey};

mod rings;

/// The ring every bend here is bent round.
const THE_RING: f64 = 4.0;

/// A ball moved off the origin is meshed to a different floor than one on it.
///
/// The move is a quarter of the radius, which is well inside the distance the
/// poles were covering: under the corner charge these two floors were the same
/// `f64`, and that is what this catches.
///
/// **The floor grows in step with the move**, and the second assertion is what
/// says so. `how_far_a_mesh_point_of_this_face_rounds` charges the *placed*
/// reach once and the *stored* reach once; only the first sees the move, and it
/// sees it linearly, so equal moves buy equal steps.
#[test]
fn a_ball_moved_off_the_origin_is_not_given_the_floor_of_one_sitting_on_it() {
    let on_it = how_fine_a_ball_a_way_out_is_cut(0.0);
    let a_quarter_out = how_fine_a_ball_a_way_out_is_cut(0.25);
    let half_out = how_fine_a_ball_a_way_out_is_cut(0.5);
    let a_whole_radius_out = how_fine_a_ball_a_way_out_is_cut(1.0);

    assert!(
        on_it < a_quarter_out,
        "a unit ball at the origin is cut to {on_it:e} and one a quarter of a radius out to \
         {a_quarter_out:e}, so moving it did not move what it is charged"
    );
    assert_eq!(
        a_quarter_out - on_it,
        half_out - a_quarter_out,
        "the first quarter of a radius cost {:e} and the second {:e}, so the charge does not \
         follow the centre",
        a_quarter_out - on_it,
        half_out - a_quarter_out
    );
    assert_eq!(
        a_whole_radius_out - half_out,
        2.0 * (half_out - a_quarter_out),
        "half a radius cost {:e} and a quarter of one {:e}",
        a_whole_radius_out - half_out,
        half_out - a_quarter_out
    );
}

/// A fatter tube round the same ring is meshed to a coarser floor, and to the
/// one `R + r` asks for.
///
/// The oracle is Pappus's, near enough: a point of a ring is `R + r` from its
/// centre at the furthest and there is nothing else to know, so the floor
/// divided by `R + r` is one number for every tube. Under the corner charge it
/// was `2R` for every tube instead, and all three of these were one `f64`.
///
/// The three agree to within an `f64`'s last place and not exactly, because the
/// division is done here and the multiplication in the kernel.
#[test]
fn a_bend_with_a_fatter_tube_is_not_given_the_floor_of_a_thinner_one() {
    let floors = [1.0, 2.0, 3.0].map(how_fine_a_bend_with_this_tube_is_cut);

    assert!(
        floors[0] < floors[1] && floors[1] < floors[2],
        "bends of tube one, two and three round a ring of {THE_RING} are cut to {floors:?}, \
         so how fat the tube is did not reach the floor"
    );
    for (tube, floor) in [1.0, 2.0, 3.0].into_iter().zip(floors) {
        let for_each_unit_of_reach = floor / (THE_RING + tube);
        let and_for_the_thinnest = floors[0] / (THE_RING + 1.0);
        assert!(
            (for_each_unit_of_reach - and_for_the_thinnest).abs()
                <= 2.0 * f64::EPSILON * and_for_the_thinnest,
            "a bend of tube {tube} is cut to {floor:e}, which is {for_each_unit_of_reach:e} for \
             each unit of the {} it reaches, where the thinnest gives {and_for_the_thinnest:e}",
            THE_RING + tube
        );
    }
}

/// How fine a mesh of a unit ball whose centre is `out` along x can be cut.
fn how_fine_a_ball_a_way_out_is_cut(out: f64) -> f64 {
    let mut model = Model::new(a_working_size());
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(out, 0.0, 0.0)).expect("a finite move"),
        )
        .expect("a ball of radius one");
    how_fine_a_mesh_of_it_is_cut(&model, ball)
}

/// How fine a mesh of a quarter bend of [`THE_RING`] and this tube can be cut.
fn how_fine_a_bend_with_this_tube_is_cut(tube: f64) -> f64 {
    let mut model = Model::new(a_working_size());
    let bend = rings::a_quarter_bend(&mut model, THE_RING, tube);
    how_fine_a_mesh_of_it_is_cut(&model, bend)
}

/// The floor, taken off the public refusal and off nothing else.
///
/// The same reading `how_far_apart_the_two_floors_under_a_mesh_are.rs` takes:
/// ask for a deflection three hundred decades under anything and the mesher
/// hands back the closest it could have cut.
fn how_fine_a_mesh_of_it_is_cut(model: &Model, solid: SolidKey) -> f64 {
    let refused: Result<Mesh, Error> = model.solid(solid).expect("this model made the key").mesh(
        Deflection::of(1e-300).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Err(Error::DeflectionBelowWhatAMeshCanReach { closest, .. }) = refused else {
        panic!("a deflection under every floor this solid has was answered with {refused:?}");
    };
    closest
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}
