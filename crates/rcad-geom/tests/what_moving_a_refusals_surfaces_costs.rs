//! What `Error::with_its_surfaces_moved` does, and what it costs.
//!
//! A refusal that carries two surfaces carries them in the frame whoever asked
//! was working in. A layer that worked in a frame of its own — `rcad-topo`'s
//! mesher is the one in this workspace — has to move them back before it hands
//! the refusal on, or it publishes numbers belonging to no model at all
//! (`docs/decisions.md` §113.6, and §116 for the argument).
//!
//! # Why the cost is measured here and not asserted away
//!
//! Moving a surface is arithmetic, so a surface moved out of a frame and back
//! into it is not always the surface that went in. That matters to exactly one
//! plan: §110.4 wanted to match a refusal's pair against a table of the faces a
//! file named, **by value**, and a match by value wants the bits. So this file
//! says which movements give the bits back and which do not, with numbers,
//! rather than leaving the next lane to find out.

use rcad_geom::{
    Axis, Cylinder, Error, Isometry, Plane, Point, Sphere, Surface, Vector,
    WhereFollowingACrossingGaveUp,
};

fn a_ball_and_a_pipe() -> Box<[Surface; 2]> {
    Box::new([
        Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a ball of radius one")),
        Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("the y axis"),
                1.0,
            )
            .expect("a pipe of radius one"),
        ),
    ])
}

fn tangency(surfaces: Box<[Surface; 2]>) -> Error {
    Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
        surfaces,
        at: WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
    }
}

/// Both surfaces move, and the rest of the refusal is left alone.
#[test]
fn moving_a_tangency_refusal_moves_both_of_its_surfaces_and_nothing_else() {
    let shift = Isometry::translation(Vector::new(1024.0, 2048.0, 4096.0)).expect("a shift");
    let moved = tangency(a_ball_and_a_pipe()).with_its_surfaces_moved(&shift);

    let Error::CannotFollowACrossingWhereItsSurfacesAreTangent { surfaces, at } = moved else {
        panic!("moving a refusal does not change which refusal it is")
    };
    assert_eq!(
        at,
        WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
        "which question gave up is not a fact about where anything is"
    );

    let [ball, pipe] = *surfaces;
    let Surface::Sphere(ball) = ball else {
        panic!("a moved ball is a ball")
    };
    let Surface::Cylinder(pipe) = pipe else {
        panic!("a moved pipe is a pipe")
    };
    assert_eq!(ball.centre(), Point::new(1024.0, 2048.0, 4096.0));
    assert_eq!(ball.radius(), 1.0, "a movement is not a scaling");
    assert_eq!(pipe.axis().origin(), Point::new(1024.0, 2048.0, 4096.0));
    assert_eq!(pipe.radius(), 1.0);
}

/// The stepping wall carries the same pair and is moved the same way.
///
/// The two are one defect and not two: they were split into two variants by
/// §110.1 on the day the field landed, and a fix that moved one of them would
/// leave the other publishing the mesher's own frame.
#[test]
fn the_stepping_wall_carries_the_same_pair_and_is_moved_the_same_way() {
    let shift = Isometry::translation(Vector::new(3.0, 0.0, 0.0)).expect("a shift");
    let moved = Error::CannotFollowACrossingBecauseNoStepStaysOnIt {
        surfaces: a_ball_and_a_pipe(),
        shortenings: 50,
    }
    .with_its_surfaces_moved(&shift);

    let Error::CannotFollowACrossingBecauseNoStepStaysOnIt {
        surfaces,
        shortenings,
    } = moved
    else {
        panic!("moving a refusal does not change which refusal it is")
    };
    assert_eq!(shortenings, 50, "how many tries it took is not a place");
    let Surface::Sphere(ball) = surfaces[0] else {
        panic!("a moved ball is a ball")
    };
    assert_eq!(ball.centre(), Point::new(3.0, 0.0, 0.0));
}

/// A refusal that says nothing about where anything is comes back untouched.
#[test]
fn a_refusal_carrying_no_surface_is_handed_back_as_it_was() {
    let shift = Isometry::translation(Vector::new(7.0, 7.0, 7.0)).expect("a shift");
    for refusal in [
        Error::PointsAreInALine,
        Error::ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces,
        Error::TheCrossingOfTheseTwoSurfacesIsNotClosed,
        Error::TooManyPointsAlongOneCrossing { most: 4 },
    ] {
        assert_eq!(
            refusal.clone().with_its_surfaces_moved(&shift),
            refusal,
            "{refusal} says nothing about where anything is"
        );
    }
}

/// **What a round trip costs, measured.** A shift down the axes gives the bits
/// back; a turn does not, and this says by how much.
///
/// It matters to one plan and not to the fix: §110.4 wanted to match a
/// refusal's pair against a table of the faces a file named, **by value**. The
/// mesher moves a *placed* face's pair by the face's own placement, which is
/// the exact inverse of the frame it worked in rather than that frame inverted
/// (`back_out_of_the_frame_this_face_is_meshed_in`, and the ulp its doc comment
/// measures) — so what strays here is the surface's own round trip and nothing
/// about the movement.
///
/// **The ball is not at the origin, and that is the whole test.** A ball
/// centred on the origin comes back bit for bit through a turn as well, because
/// turning the origin is `R·0 = 0` however awkward `R` is, and a first version
/// of this test measured `0` for both rows and concluded turns were free.
#[test]
fn what_a_surface_moved_out_of_a_frame_and_back_into_it_costs() {
    let awkward = Point::new(3.0, 5.0, 7.0);
    let a_ball_somewhere = || {
        Box::new([
            Surface::Sphere(Sphere::new(awkward, 1.0).expect("a ball of radius one")),
            Surface::Plane(
                Plane::from_point_and_normal(awkward, Vector::new(0.0, 0.0, 1.0))
                    .expect("the z axis is a unit normal"),
            ),
        ])
    };

    let straight_along_the_axes =
        Isometry::translation(Vector::new(1024.0, 2048.0, 4096.0)).expect("a shift");
    let turned_as_well = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("an axis"),
        0.25,
    )
    .expect("a quarter of a radian")
    .compose(&straight_along_the_axes);

    for (what, movement, gives_the_bits_back) in [
        ("a shift down the axes", straight_along_the_axes, true),
        ("a turn with a shift on it", turned_as_well, false),
    ] {
        let there_and_back = tangency(a_ball_somewhere())
            .with_its_surfaces_moved(&movement)
            .with_its_surfaces_moved(&movement.inverse());
        let Error::CannotFollowACrossingWhereItsSurfacesAreTangent { surfaces, .. } = there_and_back
        else {
            panic!("still the same refusal")
        };
        let Surface::Sphere(ball) = surfaces[0] else {
            panic!("still a ball")
        };
        let strayed = (ball.centre() - awkward).length();

        println!("{what}: the ball's centre came back {strayed:e} from where it started");
        assert_eq!(
            strayed == 0.0,
            gives_the_bits_back,
            "{what}: came back {strayed:e} off, and the claim was {gives_the_bits_back}"
        );
        assert!(
            strayed < 1e-11,
            "{what}: a round trip is a rounding and not a move, and this one strayed {strayed:e}"
        );
    }
}
