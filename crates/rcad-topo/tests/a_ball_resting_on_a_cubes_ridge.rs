//! A ball resting exactly on the ridge of a turned cube.
//!
//! The cube is the 2 by 2 square extruded 2, turned `3π/4` about `y` through
//! the origin, so `(1, 0, 1)` goes below `z = 0` and its edge along `y` at the
//! origin is a ridge between two faces leaning 45 degrees either way. A ball of
//! radius 1 centred at `(0, 1, 1)` touches that ridge at `(0, 1, 0)`, which is
//! the ball's south pole, and shares nothing else with the cube: each face's
//! plane cuts the ball in a circle through the pole, and both circles lie
//! outside the faces. The union is the two of them, `8 + 4π/3` between them.
//!
//! The fuse cuts the ball's skin along both whole circles, into two caps and
//! the face between them, whose one wire is a figure eight through the pole.
//! Each circle is a closed edge with a piece of the ball on both sides and no
//! flat face along it. At the pole there is no longitude to order the edge by,
//! so each piece reads its way round against the flat lid that would close it
//! (`the_lid_a_circle_from_a_pole_bounds` in `fuse/assemble.rs`). The cap lies
//! on the cube's side of the plane and the middle face on the other, so their
//! lids point opposite ways, and **read each against its own lid, both pieces
//! walked the circle the same way round.** Meshing the middle face then drew
//! the side from the pole along the first circle as the cap does, and the union
//! was refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 }` at
//! either size and either way round (Q2485).
//!
//! Now the middle face is read again against the cap's lid, so the two walk
//! each circle opposite ways. Where a flat face lies along such a circle, as in
//! `a_ball_cut_through_its_pole.rs`, the lid each ball piece reads is that flat
//! face, and nothing is read again.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Fused, Model, ProfileBuilder, Rule};

/// The union is the cube and the ball apart: two solids that validate, one
/// measuring 8 and the other `4π/3`, each within the error its own mesh
/// declares, at either working size and either way round.
#[test]
fn uniting_them_keeps_the_cube_and_the_ball_as_two_solids() {
    for size in [100.0, 1.0] {
        for ball_first in [true, false] {
            let what = format!("size {size}, ball first {ball_first}");
            let (model, fused) = unite(size, ball_first);
            let fused = fused.unwrap_or_else(|why| panic!("{what}: the union is answered: {why}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: the union validates: {why}"));
            let mut measured: Vec<_> = fused
                .solids()
                .iter()
                .map(|&solid| {
                    model
                        .solid(solid)
                        .expect("this model made the key")
                        .measure(
                            Deflection::of(0.001).expect("a positive length"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .unwrap_or_else(|why| panic!("{what}: a solid measures: {why:?}"))
                })
                .collect();
            measured.sort_by(|one, other| one.volume().total_cmp(&other.volume()));
            let [ball, cube] = measured.as_slice() else {
                panic!("{what}: two solids, not {}", measured.len());
            };
            for (measured, closed_form, which) in
                [(ball, 4.0 * PI / 3.0, "ball"), (cube, 8.0, "cube")]
            {
                assert!(
                    (measured.volume() - closed_form).abs() <= measured.volume_error(),
                    "{what}: the {which} measures {}, closed form {closed_form}, declared {}",
                    measured.volume(),
                    measured.volume_error()
                );
            }
        }
    }
}

fn unite(size: f64, ball_first: bool) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let turn =
        Isometry::rotation(&Axis::y(), 3.0 * core::f64::consts::FRAC_PI_4).expect("a finite turn");
    let cube = model
        .add_extrusion(&square, 2.0, &turn)
        .expect("a positive height");
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(0.0, 1.0, 1.0)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let (one, other) = if ball_first {
        (ball, cube)
    } else {
        (cube, ball)
    };
    let policy = Policy::tolerant(model.scale().nominal_size());
    let fused = model.fuse(
        &[one],
        &[other],
        Rule::Union,
        &policy,
        &mut ProvenanceLog::new(),
    );
    (model, fused)
}
