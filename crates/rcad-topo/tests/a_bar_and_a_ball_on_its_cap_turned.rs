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
//! stored numbers the ball's centre then sits a rounding off the pipe's axis,
//! so the two cross in two loops a rounding off two circles, and turned, every
//! rule answers what it answers unturned.
//!
//! **Every rule was refused `ResultIsNotManifold { count: 1 }` until Q2330**,
//! the bar through the ball as much as the ball on the cap (Q2129). In
//! `rcad-geom`, `where_a_ball_reaches_across_a_pipe` took its way across the
//! pipe from `out − along·(out·along)`, and with the centre a rounding off the
//! axis that is rounding too: `4.4e−15` long, and pointing back down the axis,
//! `−0.99997` of the way. So both seeds it handed the marcher sat inside the
//! pipe, near neither loop, and `where_a_closed_crossing_starts` refused each
//! loop `CannotNameWhichPieceOfACrossingThisIs`. The arrangement starts no
//! node on a loop it cannot name, so the pipe and the ball were left uncut,
//! and the answer came out with an edge that had one face along it. Taking the
//! part along the axis off that vector a second time leaves it square to the
//! axis, which is the fix; `rcad-geom`'s
//! `a_ball_a_rounding_off_a_pipes_axis.rs` tests the seeds themselves.
//!
//! Refusing by name where the arrangement starts no node was not the fix
//! either. Passing on every refusal there but "this crossing is not closed"
//! fails three other tests in this crate, which answer because the loop is
//! left uncut: two with a ball a bar holds round the one circle the two only
//! touch along, refused `CannotFollowACrossingWhereItsSurfacesAreTangent`, and
//! a turned frustum cut along its seam under a tolerant policy, refused
//! `CannotNameWhichPieceOfACrossingThisIs`.
//!
//! # A rounded rod: a ball of the bar's own radius on its cap
//!
//! A bar of radius 1 or 2.5 and one longer than that, with a ball of its
//! radius centred on its cap, both turned by one of six angles about one of
//! two lines (Q3131). Upright the cap, the bar's side and the ball share the
//! rim exactly, and every rule answers. Turned, the stored numbers part them:
//! with a radius of 1 the ball's centre is on the axis exactly but a few
//! roundings off the cap, so the ball's circle on the cap is a ring `1e−32`
//! inside the rim; with a radius of 2.5 it is a rounding off the axis too, and
//! the two circles cross, leaving two lunes. **All 96 fuses were refused**,
//! certified and under `Policy::tolerant(100.0)` alike, until Q3165: in
//! `rcad-geom`, a tolerant policy now takes two circles a plane cuts within
//! its wobble of each other as one, so the cap, the side and the ball share
//! the rim again. Certified, the refusals stay, and they are limits: the ring
//! and the lunes are `1e16` and more times smaller than the rod.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::expansion::{expansion_sum, scale_expansion};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Measurement, Model, Rule, SolidKey};

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
        shape.measures_what_every_rule_should(&[], &[], "unturned");
    }
}

#[test]
fn a_bar_and_a_ball_on_its_axis_turned_measure_what_every_rule_should() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");
    for shape in SHAPES {
        for ball_turns in [vec![turn], vec![half, half]] {
            shape.measures_what_every_rule_should(
                &[turn],
                &ball_turns,
                &format!("the ball turned in {} steps", ball_turns.len()),
            );
        }
    }
}

impl Shape {
    /// Every rule fuses the bar placed by `bar_turns` and the ball by
    /// `ball_turns` into as many solids as it should, of the volume worked out
    /// by hand within the error the measure declares.
    fn measures_what_every_rule_should(
        self,
        bar_turns: &[Isometry],
        ball_turns: &[Isometry],
        placed: &str,
    ) {
        for (rule, solids, closed_form) in self.what_each_rule_leaves() {
            let what = format!("the {rule:?} of {self:?}, {placed}");
            let measured = self
                .fused(bar_turns, ball_turns, rule)
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

#[test]
fn a_rod_with_a_ball_of_its_radius_on_its_cap_turned_fuses_under_a_tolerant_policy() {
    for working_size in [100.0, 10.0] {
        let policy = Policy::tolerant(working_size);
        for rod in RoundedRod::every_one() {
            for (first, rule, closed_form) in rod.what_each_fuse_leaves() {
                let what = format!("the {rule:?} with the {first:?} first of {rod:?}, {policy:?}");
                let (mut model, bar, ball) = rod.built();
                let [one, other] = match first {
                    First::Bar => [bar, ball],
                    First::Ball => [ball, bar],
                };
                let answer = model
                    .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what} was refused: {why:?}"));
                let [solid] = answer.solids() else {
                    panic!("{what}: {} solids, not one", answer.solids().len());
                };
                model
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what} does not validate: {why}"));
                let measured = model
                    .solid(*solid)
                    .expect("this model made the key")
                    .measure(
                        Deflection::of(0.01).expect("a positive length"),
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{what} measures: {why:?}"));
                let (volume, declared) = (measured.volume(), measured.volume_error());
                assert!(
                    (volume - closed_form).abs() <= declared,
                    "{what}: measured {volume}, closed form {closed_form}, declared {declared}"
                );
            }
        }
    }
}

/// §3.5 says a model whose largest coordinate is more than `1e11` times its
/// smallest feature is past what `f64` can be asked to hold, and that a
/// feature `1e16` times smaller than the coordinates is already gone from the
/// stored numbers. Each rod's smallest feature is worked out exactly from the
/// numbers its surfaces store, and is past both — `8.2e−34` to `6.6e−32` for
/// a radius of 1, a range of `3.6e31` and up, and `8.1e−19` to `3.1e−16` for a
/// radius of 2.5, a range of `1.7e16` and up — so a refusal under the
/// certified policy is a limit, not a gap (§16.10). Every exact answer keeps
/// the cap's ring or its two lunes, and those are the features measured.
#[test]
fn a_rod_with_a_ball_on_its_cap_turned_is_refused_certified_for_a_gap_1e11_below_its_size() {
    for rod in RoundedRod::every_one() {
        for (first, rule, _) in rod.what_each_fuse_leaves() {
            let what = format!("the {rule:?} with the {first:?} first of {rod:?}");
            let (mut model, bar, ball) = rod.built();
            let [one, other] = match first {
                First::Bar => [bar, ball],
                First::Ball => [ball, bar],
            };
            let refused = model.fuse(
                &[one],
                &[other],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(refused.is_err(), "{what} was answered certified");
        }
        let (model, bar, ball) = rod.built();
        let feature = rod.its_smallest_feature(&model, bar, ball);
        let range = rod.reach(&model, ball) / feature;
        assert!(
            range > 1e11,
            "{rod:?}: a feature of {feature:e} is a range of only {range:e}, inside §3.5's 1e11"
        );
        assert!(
            range > 1e16,
            "{rod:?}: a feature of {feature:e} is a range of {range:e}, short of the 1e16 at \
             which §3.5 says it is gone from the stored numbers"
        );
    }
}

/// Which of the two goes first in a fuse.
#[derive(Clone, Copy, Debug)]
enum First {
    Bar,
    Ball,
}

/// A round bar of radius `radius` and `radius + 1` long standing on the
/// origin, and a ball of the same radius centred on the middle of its top cap,
/// both placed by the turn of `degrees` about the line through the origin
/// along `about` (Q3131).
///
/// Upright, the cap, the bar's side and the ball share the bar's rim. Turned,
/// the stored numbers part them by roundings, and the cap keeps a ring or two
/// lunes between the ball's circle and the rim, `1e−16` wide or less.
#[derive(Clone, Copy, Debug)]
struct RoundedRod {
    radius: f64,
    degrees: f64,
    about: [f64; 3],
}

impl RoundedRod {
    /// The 24 rods: both radii, six turns, about two lines.
    fn every_one() -> Vec<RoundedRod> {
        let mut rods = Vec::new();
        for radius in [1.0, 2.5] {
            for degrees in [7.0, 13.0, 30.0, 45.0, 61.0, 89.0] {
                for about in [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]] {
                    rods.push(RoundedRod {
                        radius,
                        degrees,
                        about,
                    });
                }
            }
        }
        rods
    }

    /// A model holding the bar and the ball, and the two.
    fn built(self) -> (Model, SolidKey, SolidKey) {
        let [x, y, z] = self.about;
        let turn = Isometry::rotation(
            &Axis::new(Point::new(0.0, 0.0, 0.0), Vector::new(x, y, z)).expect("a direction"),
            self.degrees.to_radians(),
        )
        .expect("a finite angle");
        let length = self.radius + 1.0;
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(self.radius, length, &turn)
            .expect("a bar");
        let on_the_cap =
            turn.compose(&Isometry::translation(Vector::new(0.0, 0.0, length)).expect("a move"));
        let ball = model.add_ball(self.radius, &on_the_cap).expect("a ball");
        (model, bar, ball)
    }

    /// Each fuse, with the volume it leaves: the lower half of the ball is
    /// inside the bar and the upper half outside it.
    fn what_each_fuse_leaves(self) -> [(First, Rule, f64); 4] {
        let r = self.radius;
        let bar = PI * r * r * (r + 1.0);
        let half = 2.0 / 3.0 * PI * r.powi(3);
        [
            (First::Bar, Rule::Union, bar + half),
            (First::Bar, Rule::Intersection, half),
            (First::Bar, Rule::Difference, bar - half),
            (First::Ball, Rule::Difference, half),
        ]
    }

    /// The narrowest thing the cap keeps between the ball's circle and the
    /// rim, as the stored numbers place them.
    ///
    /// Where the ball's centre is off the bar's axis, the two circles cross
    /// and leave two lunes about as wide as the centre is off the axis. Where
    /// it is on the axis exactly, as `2 × direction` is for a radius of 1, the
    /// two circles share a centre, and the ball's is narrower than the rim by
    /// `r − √(r² − e²)`, for `e` how far the centre is off the cap.
    fn its_smallest_feature(self, model: &Model, bar: SolidKey, ball: SolidKey) -> f64 {
        let surfaces = |solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .map(|face| face.surface())
                .collect::<Vec<_>>()
        };
        let of_the_bar = surfaces(bar);
        let pipe = of_the_bar
            .iter()
            .find_map(|surface| {
                let Surface::Cylinder(pipe) = surface else {
                    return None;
                };
                Some(*pipe)
            })
            .expect("a bar has a side");
        let cap = of_the_bar
            .iter()
            .filter_map(|surface| {
                let Surface::Plane(plane) = surface else {
                    return None;
                };
                Some(*plane)
            })
            .max_by(|one, other| one.offset().total_cmp(&other.offset()))
            .expect("a bar has caps");
        let centre = self.the_balls_centre(model, ball);

        let along = exact(pipe.axis().direction().components());
        let (origin, middle) = (pipe.axis().origin().coordinates(), centre.coordinates());
        let from_the_axis: [Vec<f64>; 3] =
            std::array::from_fn(|axis| plus(&[middle[axis]], &[-origin[axis]]));
        let square_to_the_axis = cross(&from_the_axis, &along);
        let off_the_axis_squared =
            rounded(&dot(&square_to_the_axis, &square_to_the_axis)) / rounded(&dot(&along, &along));
        if off_the_axis_squared > 0.0 {
            return off_the_axis_squared.sqrt();
        }

        let normal = exact(cap.normal().components());
        let above_the_cap = plus(
            &dot(&normal, &exact(centre.coordinates())),
            &[-cap.offset()],
        );
        let e = rounded(&above_the_cap) / rounded(&dot(&normal, &normal)).sqrt();
        let r = self.radius;
        e * e / (r + (r * r - e * e).sqrt())
    }

    /// How far the model reaches from the origin, at the most: the ball's
    /// centre's largest coordinate and its radius, as §3.5 measures it.
    fn reach(self, model: &Model, ball: SolidKey) -> f64 {
        let centre = self.the_balls_centre(model, ball).coordinates();
        centre
            .iter()
            .fold(0.0, |most, each| f64::max(most, each.abs()))
            + self.radius
    }

    fn the_balls_centre(self, model: &Model, ball: SolidKey) -> Point {
        model
            .solid(ball)
            .expect("this model made the key")
            .faces()
            .find_map(|face| {
                let Surface::Sphere(skin) = face.surface() else {
                    return None;
                };
                Some(skin.centre())
            })
            .expect("a ball has a skin")
    }
}

/// Each of three numbers as an expansion of one entry.
fn exact(numbers: [f64; 3]) -> [Vec<f64>; 3] {
    numbers.map(|each| vec![each])
}

/// The exact sum of two expansions.
fn plus(one: &[f64], other: &[f64]) -> Vec<f64> {
    let mut sum = vec![0.0; one.len() + other.len()];
    let used = expansion_sum(one, other, &mut sum);
    sum.truncate(used);
    sum
}

/// The exact product of two expansions.
fn times(one: &[f64], other: &[f64]) -> Vec<f64> {
    other.iter().fold(vec![0.0], |sum, each| {
        let mut product = vec![0.0; 2 * one.len()];
        let used = scale_expansion(one, *each, &mut product);
        product.truncate(used);
        plus(&sum, &product)
    })
}

fn dot(one: &[Vec<f64>; 3], other: &[Vec<f64>; 3]) -> Vec<f64> {
    (0..3).fold(vec![0.0], |sum, axis| {
        plus(&sum, &times(&one[axis], &other[axis]))
    })
}

fn cross(one: &[Vec<f64>; 3], other: &[Vec<f64>; 3]) -> [Vec<f64>; 3] {
    std::array::from_fn(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        plus(
            &times(&one[next], &other[last]),
            &times(&times(&one[last], &other[next]), &[-1.0]),
        )
    })
}

/// An expansion added up in `f64`, smallest entry first. The tests divide by
/// what it gives and ask the answer against `1e11`, not to the last bit.
fn rounded(expansion: &[f64]) -> f64 {
    expansion.iter().sum()
}
