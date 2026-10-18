//! A round bar with a ball on the middle of its top cap, turned.
//!
//! The bar is `add_round_bar` of radius 5 and height 10. The ball is
//! `add_ball` of radius 6 moved to `(0, 0, 10)`, of radius 6 moved to
//! `(0, 0, 12)`, or of radius 5.5 moved to `(0, 0, 10)`. Unplaced, the ball's
//! centre is on the bar's axis and the two cross in one circle below the cap,
//! and every rule answers one solid of the volume worked out by hand. So does a
//! bar of height 30 with a ball of 6 through its middle at `(0, 0, 15)`, which
//! crosses it in two circles, except that the difference is two solids.
//!
//! Both are placed by the turn of `1.1` about the line through `(1, 0, −1)`
//! along `(1, 2, 3)`, the ball by that turn or by two halves of it. In the
//! stored numbers the ball's centre then sits about `1.1e−15` off the pipe's
//! axis, so the two cross in two loops a rounding off two circles, and every
//! rule is refused `ResultIsNotManifold { count: 1 }` (row Q2129). That the
//! ball is on the cap does not matter: the bar it goes through is refused the
//! same way.
//!
//! **The first step that goes wrong is in `rcad-geom`, and it is a gap.**
//! `where_a_ball_reaches_across_a_pipe` takes its way across the pipe from
//! `out − along·(out·along)`. When the centre is a rounding off the axis that
//! vector is rounding too: here it is `4.4e−15` long, and its direction points
//! back down the axis, `−0.99997` of the way. So both seeds it hands the marcher
//! sit inside the pipe, near neither loop, and `where_a_closed_crossing_starts`
//! refuses each loop `CannotNameWhichPieceOfACrossingThisIs`. The arrangement
//! starts no node on a loop it cannot name, so the pipe and the ball are left
//! uncut, and the answer comes out with an edge that has one face along it.
//!
//! The loops are there, and the answer is the unturned one turned. Taking the
//! part along the axis off that vector a second time leaves it square to the
//! axis, and with that one line every rule here answers, turned, the volume it
//! answers unturned. That line is outside this row's files, so the turned test
//! pins the refusal until it lands.
//!
//! Refusing by name where the arrangement starts no node is not the fix
//! either. Passing on every refusal there but "this crossing is not closed"
//! fails three other tests in this crate, which answer because the loop is
//! left uncut: two with a ball a bar holds round the one circle the two only
//! touch along, refused `CannotFollowACrossingWhereItsSurfacesAreTangent`, and
//! a turned frustum cut along its seam under a tolerant policy, refused
//! `CannotNameWhichPieceOfACrossingThisIs`.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Measurement, Model, Rule};

const BAR_RADIUS: f64 = 5.0;

/// A round bar of radius 5 standing on the origin, and a ball centred on its
/// axis.
#[derive(Clone, Copy, Debug)]
struct Shape {
    bar_height: f64,
    radius: f64,
    height: f64,
}

const SHAPES: [Shape; 4] = [
    Shape {
        bar_height: 10.0,
        radius: 6.0,
        height: 10.0,
    },
    Shape {
        bar_height: 10.0,
        radius: 6.0,
        height: 12.0,
    },
    Shape {
        bar_height: 10.0,
        radius: 5.5,
        height: 10.0,
    },
    Shape {
        bar_height: 30.0,
        radius: 6.0,
        height: 15.0,
    },
];

#[test]
fn a_bar_and_a_ball_on_its_axis_unturned_measure_what_every_rule_should() {
    for shape in SHAPES {
        for (rule, solids, closed_form) in shape.what_each_rule_leaves() {
            let what = format!("the {rule:?} of {shape:?}, unturned");
            let measured = shape
                .fused(&[], &[], rule)
                .unwrap_or_else(|why| panic!("{what} was refused: {why:?}"));
            assert_eq!(measured.len(), solids, "{what}: how many solids");
            let volume: f64 = measured.iter().map(|each| each.volume()).sum();
            let declared: f64 = measured.iter().map(|each| each.volume_error()).sum();
            assert!(
                (volume - closed_form).abs() <= declared,
                "{what}: measured {volume}, closed form {closed_form}, declared {declared}"
            );
        }
    }
}

#[test]
fn a_bar_and_a_ball_on_its_axis_turned_are_refused_while_the_seeds_point_along_the_bar() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");
    for shape in SHAPES {
        for ball_turns in [vec![turn], vec![half, half]] {
            for (rule, _, _) in shape.what_each_rule_leaves() {
                let what = format!(
                    "the {rule:?} of {shape:?}, the ball turned in {} steps",
                    ball_turns.len()
                );
                let refused = shape
                    .fused(&[turn], &ball_turns, rule)
                    .map(|measured| measured.len())
                    .expect_err(&what);
                assert!(
                    matches!(refused, Error::ResultIsNotManifold { count: 1 }),
                    "{what}: {refused:?}"
                );
            }
        }
    }
}

impl Shape {
    /// What each solid of the bar and the ball fused under `rule` measures,
    /// under `Policy::certified()`, once the answer validates.
    fn fused(
        self,
        bar_turns: &[Isometry],
        ball_turns: &[Isometry],
        rule: Rule,
    ) -> rcad_topo::Result<Vec<Measurement>> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(BAR_RADIUS, self.bar_height, &Isometry::identity())
            .expect("a bar");
        let ball = model
            .add_ball(
                self.radius,
                &Isometry::translation(Vector::new(0.0, 0.0, self.height)).expect("a move"),
            )
            .expect("a ball");
        for placed in bar_turns {
            model
                .place_solid(bar, placed)
                .expect("this model made the key");
        }
        for placed in ball_turns {
            model
                .place_solid(ball, placed)
                .expect("this model made the key");
        }
        let answer = model.fuse(
            &[bar],
            &[ball],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )?;
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("the {rule:?} does not validate: {why}"));
        Ok(answer
            .solids()
            .iter()
            .map(|solid| {
                model
                    .solid(*solid)
                    .expect("this model made the key")
                    .measure(
                        Deflection::of(0.01).expect("a positive length"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("the {rule:?} measures: {why:?}"))
            })
            .collect())
    }

    /// Each rule, with how many solids it leaves and their volume.
    fn what_each_rule_leaves(self) -> [(Rule, usize, f64); 3] {
        let bar = PI * BAR_RADIUS * BAR_RADIUS * self.bar_height;
        let ball = 4.0 / 3.0 * PI * self.radius.powi(3);
        let shared = self.the_ball_inside_the_bar();
        let through = self.height + self.radius < self.bar_height;
        [
            (Rule::Union, 1, bar + ball - shared),
            (Rule::Intersection, 1, shared),
            (Rule::Difference, if through { 2 } else { 1 }, bar - shared),
        ]
    }

    /// The part of the ball that lies in the bar, slice by slice across the
    /// axis, for a ball that reaches no lower than the bottom cap and crosses
    /// the bar's side below the top cap.
    ///
    /// Measured from the centre along the axis, the two cross at `±c`, with
    /// `c = √(radius² − 5²)`. Further out than `c` the ball is the narrower and
    /// each slice is the ball's; nearer in, each slice is the bar's.
    fn the_ball_inside_the_bar(self) -> f64 {
        let r = self.radius;
        let c = (r * r - BAR_RADIUS * BAR_RADIUS).sqrt();
        let top = (self.bar_height - self.height).min(r);
        assert!(
            self.height >= r && top >= -c,
            "{self:?} is not one this slices"
        );
        let up_to = |t: f64| PI * (r * r * t - t.powi(3) / 3.0);
        let below = up_to(-c) - up_to(-r);
        let middle = PI * BAR_RADIUS * BAR_RADIUS * (top.min(c) + c);
        let above = if top > c { up_to(top) - up_to(c) } else { 0.0 };
        below + middle + above
    }
}
