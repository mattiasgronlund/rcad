//! The box round the point straight across a ball-and-plane circle that starts
//! at a pole holds the true point across (§194.4, §194.5).
//!
//! `across_a_circle_that_starts_at_a_pole` in `fuse::arrange` is private, so
//! this does what it does with the same public calls in the same order: the
//! start comes from [`where_a_closed_crossing_starts`], the middle of the
//! circle from [`Plane::project`], and the point across from
//! `centre + (centre - from)`. A replication that rounded differently would
//! measure a different point, so nothing here is reworded.
//!
//! The true point across is worked out in double-double arithmetic, about 106
//! bits, from the numbers the ball and the plane store: the circle point
//! nearest the rounded start, mirrored through the true middle of the circle.
//! What is measured is how far the rounded point sits from it, in the widest
//! coordinate, against the half-width the kernel gives its box.

use core::f64::consts::{PI, TAU};

use rcad_geom::{
    Axis, Deflection, Isometry, POINT_ALONG_A_CROSSING_ROUNDOFF, Plane, Point, Scale, Sphere,
    Surface, SurfaceMeet, Vector, intersect, where_a_closed_crossing_starts,
};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
};
use rcad_topo::{Model, Rule, SolidKey};

/// Measured on this tree, the worst of 3 824 boxed points sits at about 0.005
/// of the box's half-width from the true point across: the bound holds with
/// some two hundred times to spare, and the search came nowhere near it.
#[test]
fn the_point_across_a_circle_that_starts_at_a_pole_lies_in_its_box() {
    let mut worst = Worst::default();
    let mut random = SplitMix(0x5eed_0194_0005);
    for _ in 0..20_000 {
        let (ball, flat) = a_sunk_ball_turned_on_its_side(&mut random);
        worst.take(ball, flat);
        let (ball, flat) = a_plane_through_a_pole(&mut random);
        worst.take(ball, flat);
    }
    let (ball, flat) = the_fixture_itself();
    worst.take(ball, flat);
    println!("{worst:#?}");
    assert!(
        worst.boxed > 1_000,
        "the search reached the boxed point across only {} times",
        worst.boxed
    );
    assert!(
        worst.boxed_ratio > 0.0,
        "every rounded point across is the true one to 106 bits, so this measured nothing"
    );
    assert!(
        worst.boxed_ratio <= 1.0,
        "the true point across is {} of the box's half-width from the rounded one, at {:?}",
        worst.boxed_ratio,
        worst.boxed_case
    );
}

/// The fixture of `a_turned_ball_and_block_closes_an_edge_at_a_pole.rs` at
/// sizes from a thousandth to a million: every union that answers is a sound
/// model.
///
/// **All 36 answer.** Eleven used to refuse, every one turned about `x` at a
/// size that is not a power of two, because the start of the circle sat
/// within one `atan2` rounding of the pole without being on the axis, and the
/// fuse and the mesh disagreed about whether that was a pole (§195.2); the
/// two tests below pin one of each refusal it came back as.
#[test]
fn a_ball_and_block_turned_on_their_side_unite_soundly_at_every_size() {
    let mut answered = 0;
    let mut refused = Vec::new();
    for size in [
        1.0 / 1024.0,
        0.001,
        0.03,
        0.5,
        1.0,
        3.0,
        7.0,
        64.0,
        1000.0,
        4096.0,
        65536.0,
        1.0e6,
    ] {
        for (direction, steps) in [([1, 0, 0], 16), ([1, 0, 0], 48), ([0, 1, 0], 16)] {
            match a_turned_ball_and_block_united(size, direction, steps) {
                Ok((model, _)) => {
                    model
                        .validate(&Policy::certified(), &mut ProvenanceLog::new())
                        .unwrap_or_else(|why| {
                            panic!(
                                "a ball of radius {size} and its block turned {steps} steps \
                                 about {direction:?} unite into a model that is not sound: {why}"
                            )
                        });
                    answered += 1;
                }
                Err(why) => refused.push(format!(
                    "radius {size:e}, {steps} steps about {direction:?}: {why}"
                )),
            }
        }
    }
    println!("{answered} unions answered; refused: {refused:#?}");
    assert_eq!(
        answered, 36,
        "not every turned ball and block unites; refused: {refused:#?}"
    );
}

/// Radius 3, a quarter turn about `x`: the circle's start is `4.4e-16` off the
/// ball's axis, so `atan2` reads its latitude as exactly the south pole and the
/// mesh calls it one, while the fuse, asking whether the longitude had a
/// direction there, found one and left the circle one closed edge. The mesh
/// then refused the ball's face as a loop whose corners coincide (§195.2).
#[test]
fn a_ball_and_block_turned_at_radius_three_unite_into_the_block_and_half_the_ball() {
    unites_into_the_block_and_half_the_ball(3.0);
}

/// Radius 0.001, the same turn: the start is `1e-20` off the axis, the same
/// disagreement, refused further on as a face that goes right round with no
/// seam (§195.2).
#[test]
fn a_ball_and_block_turned_at_radius_a_thousandth_unite_into_the_block_and_half_the_ball() {
    unites_into_the_block_and_half_the_ball(0.001);
}

/// One solid, sound, and as big as the block and the half of the ball
/// standing out of its floor: `125 s³ + (2/3)π s³`, measured from a mesh
/// inscribed in the ball, so a little under it.
fn unites_into_the_block_and_half_the_ball(size: f64) {
    let (model, solids) = a_turned_ball_and_block_united(size, [1, 0, 0], 16)
        .unwrap_or_else(|why| panic!("a ball of radius {size} and its block unite: {why}"));
    assert_eq!(solids.len(), 1, "the ball stands out of the block's floor");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the union at radius {size} is not sound: {why}"));
    let volume = model
        .solid(solids[0])
        .expect("the union made the key")
        .measure(
            Deflection::of(size / 1000.0).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the union measures")
        .volume();
    let closed_form = (125.0 + 2.0 / 3.0 * PI) * size.powi(3);
    let apart = (closed_form - volume) / closed_form;
    println!("radius {size}: measured {volume}, closed form {closed_form}, {apart:e} under");
    assert!(
        (0.0..0.001).contains(&apart),
        "at radius {size} the union measures {volume} against {closed_form}, {apart:e} under"
    );
}

/// A block of side `5 · size` with a ball of radius `size` sunk to its middle
/// in the floor at `(3, 3) · size`, both turned `steps` sixty-fourths of a turn
/// about `direction`, and united.
fn a_turned_ball_and_block_united(
    size: f64,
    direction: [i32; 3],
    steps: u32,
) -> rcad_topo::Result<(Model, Vec<SolidKey>)> {
    let mut model = Model::new(Scale::millimetres(20.0 * size).expect("a working size"));
    let block = model
        .add_cube(5.0 * size, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    let ball = model
        .add_ball(
            size,
            &Isometry::translation(Vector::new(3.0 * size, 3.0 * size, 0.0)).expect("a shift"),
        )
        .expect("a ball");
    let turn = a_turn(direction, steps);
    for key in [block, ball] {
        model
            .place_solid(key, &turn)
            .expect("this model made the key");
    }
    let mut log = ProvenanceLog::new();
    let fused = model.fuse(
        &[block],
        &[ball],
        Rule::Union,
        &Policy::certified(),
        &mut log,
    )?;
    assert!(
        log.records().is_empty(),
        "a certified union recorded {} decisions it could not prove",
        log.records().len()
    );
    let solids = fused.solids().to_vec();
    Ok((model, solids))
}

/// The worst the search found, and how often each way out was taken.
#[derive(Debug, Default)]
struct Worst {
    /// Pairs whose circle does not start at a pole, or that do not cross.
    not_at_a_pole: u32,
    /// Pairs whose point across is the other pole, which the kernel leaves whole.
    across_is_the_other_pole: u32,
    /// Pairs whose rounded point across both surfaces certify, used as it is.
    certified: u32,
    /// Pairs that get the box.
    boxed: u32,
    /// Distance from the true point across to the rounded one, over the box's
    /// half-width, at its worst.
    boxed_ratio: f64,
    boxed_case: Option<(Sphere, Plane)>,
    /// The same ratio where the rounded point was certified and no box is
    /// made: that point is on the circle, and this says how far along it.
    certified_ratio: f64,
    /// How far the start itself sits off the circle, over the start's own
    /// half-width `POINT_ALONG_A_CROSSING_ROUNDOFF · max(1, |from|∞)`.
    start_ratio: f64,
    /// How far the rounded mirror sits from the exact mirror of the rounded
    /// start, over the box's half-width: the part of the error this function
    /// adds.
    mirror_ratio: f64,
}

impl Worst {
    fn take(&mut self, ball: Sphere, flat: Plane) {
        let Some(case) = where_the_kernel_puts_it(ball, flat) else {
            self.not_at_a_pole += 1;
            return;
        };
        let (across, reach) = match case {
            Across::TheOtherPole => {
                self.across_is_the_other_pole += 1;
                return;
            }
            Across::Certified(across) => (across, None),
            Across::Boxed { across, reach } => (across, Some(reach)),
        };
        let from = start_of(ball, flat);
        let truth = the_true_point_across(ball, flat, from);
        let off_by = widest_gap(across, truth.across);
        let start_reach =
            POINT_ALONG_A_CROSSING_ROUNDOFF * from.largest_coordinate_magnitude().max(1.0);
        let box_reach = 2.0
            * POINT_ALONG_A_CROSSING_ROUNDOFF
            * [across, from, ball.centre()]
                .map(|point| point.largest_coordinate_magnitude())
                .into_iter()
                .fold(1.0, f64::max);
        self.start_ratio = self
            .start_ratio
            .max(widest_gap(from, truth.start) / start_reach);
        match reach {
            None => {
                self.certified += 1;
                self.certified_ratio = self.certified_ratio.max(off_by / box_reach);
            }
            Some(reach) => {
                assert_eq!(
                    reach, box_reach,
                    "the box's half-width is worked out the way the kernel does"
                );
                self.boxed += 1;
                self.mirror_ratio = self
                    .mirror_ratio
                    .max(widest_gap(across, truth.mirror_of_the_start) / reach);
                let ratio = off_by / reach;
                if ratio > self.boxed_ratio {
                    self.boxed_ratio = ratio;
                    self.boxed_case = Some((ball, flat));
                }
            }
        }
    }
}

enum Across {
    TheOtherPole,
    Certified(Point),
    Boxed { across: Point, reach: f64 },
}

/// `across_a_circle_that_starts_at_a_pole`, step for step, from the start the
/// fuse gets. `None` where the circle does not start at a pole.
fn where_the_kernel_puts_it(ball: Sphere, flat: Plane) -> Option<Across> {
    let pair = [Surface::Plane(flat), Surface::Sphere(ball)];
    let SurfaceMeet::Curves(runs) = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .ok()?
    else {
        return None;
    };
    let [run] = &runs[..] else {
        panic!("a plane crosses a ball in one circle, not {}", runs.len())
    };
    let from = where_a_closed_crossing_starts(run, [&pair[0], &pair[1]], &mut ProvenanceLog::new())
        .expect("a circle starts somewhere");
    let from_the_other_side =
        where_a_closed_crossing_starts(run, [&pair[1], &pair[0]], &mut ProvenanceLog::new())
            .expect("a circle starts somewhere");
    assert_eq!(
        from, from_the_other_side,
        "where a circle starts does not depend on which surface is asked first"
    );
    let at = middle_of(where_the_crossing_starts(ball, flat, from));
    Surface::Sphere(ball)
        .the_way_its_first_coordinate_runs_at(at)
        .is_none()
        .then_some(())?;

    let centre = flat.project(ball.centre());
    let across = centre + (centre - from);
    if Surface::Sphere(ball)
        .the_way_its_first_coordinate_runs_at(across)
        .is_none()
    {
        return Some(Across::TheOtherPole);
    }
    if is_on_both(ball, flat, across) {
        return Some(Across::Certified(across));
    }
    let largest = [across, from, ball.centre()]
        .map(|point| point.largest_coordinate_magnitude())
        .into_iter()
        .fold(1.0, f64::max);
    Some(Across::Boxed {
        across,
        reach: 2.0 * POINT_ALONG_A_CROSSING_ROUNDOFF * largest,
    })
}

fn start_of(ball: Sphere, flat: Plane) -> Point {
    let pair = [Surface::Plane(flat), Surface::Sphere(ball)];
    let SurfaceMeet::Curves(runs) = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("asked once already") else {
        panic!("crossed once already")
    };
    where_a_closed_crossing_starts(&runs[0], [&pair[0], &pair[1]], &mut ProvenanceLog::new())
        .expect("started once already")
}

/// `fuse::arrange::where_the_crossing_starts`: the start as it is where both
/// surfaces certify it, and a box round it otherwise.
fn where_the_crossing_starts(ball: Sphere, flat: Plane, from: Point) -> PredicatePoint {
    let exact = PredicatePoint::from(from.coordinates());
    if is_on_both(ball, flat, from) {
        return exact;
    }
    let reach = POINT_ALONG_A_CROSSING_ROUNDOFF * from.largest_coordinate_magnitude().max(1.0);
    PredicatePoint::Constructed(from.coordinates().map(|coordinate| {
        Interval::around(coordinate, reach).expect("a finite point and a positive reach")
    }))
}

/// `fuse::middle_of`.
fn middle_of(point: PredicatePoint) -> Point {
    let box_of_it = point.enclosure();
    let axis =
        |axis: usize| box_of_it[axis].lo() + (box_of_it[axis].hi() - box_of_it[axis].lo()) / 2.0;
    Point::new(axis(0), axis(1), axis(2))
}

fn is_on_both(ball: Sphere, flat: Plane, point: Point) -> bool {
    let exact = PredicatePoint::from(point.coordinates());
    ball.side_of(exact) == Decision::Zero && flat.side_of_point(exact) == Decision::Zero
}

fn widest_gap(rounded: Point, truth: [Dd; 3]) -> f64 {
    let rounded = rounded.coordinates();
    (0..3)
        .map(|axis| Dd::of(rounded[axis]).minus(truth[axis]).hi.abs())
        .fold(0.0, f64::max)
}

struct Truth {
    /// The circle point nearest the rounded start.
    start: [Dd; 3],
    /// The circle point straight across from it.
    across: [Dd; 3],
    /// The rounded start mirrored exactly through the circle's true middle.
    mirror_of_the_start: [Dd; 3],
}

/// The circle the stored ball and plane cross in, read at about 106 bits.
///
/// The plane is every `x` with `n·x = offset`, `n` the stored normal at the
/// length it is stored at; the ball is every `x` `radius` from its centre.
fn the_true_point_across(ball: Sphere, flat: Plane, from: Point) -> Truth {
    let normal = flat.normal().components().map(Dd::of);
    let offset = Dd::of(flat.offset());
    let centre = ball.centre().coordinates().map(Dd::of);
    let from = from.coordinates().map(Dd::of);
    let squared_normal = dot(normal, normal);

    let out_of_the_plane = |point: [Dd; 3]| dot(normal, point).minus(offset).over(squared_normal);
    let onto_the_plane = |point: [Dd; 3]| {
        let out = out_of_the_plane(point);
        [0, 1, 2].map(|axis| point[axis].minus(normal[axis].times(out)))
    };

    let middle = onto_the_plane(centre);
    let height = dot(normal, centre).minus(offset);
    let squared_radius = Dd::of(ball.radius())
        .times(Dd::of(ball.radius()))
        .minus(height.times(height).over(squared_normal));
    assert!(
        squared_radius.hi > 0.0,
        "a plane through a pole of a ball crosses it in a circle"
    );
    let radius = squared_radius.sqrt();

    let foot = onto_the_plane(from);
    let outward = [0, 1, 2].map(|axis| foot[axis].minus(middle[axis]));
    let length = dot(outward, outward).sqrt();
    assert!(length.hi > 0.0, "the start of a circle is not its middle");
    let step = [0, 1, 2].map(|axis| radius.times(outward[axis]).over(length));
    Truth {
        start: [0, 1, 2].map(|axis| middle[axis].plus(step[axis])),
        across: [0, 1, 2].map(|axis| middle[axis].minus(step[axis])),
        mirror_of_the_start: [0, 1, 2]
            .map(|axis| middle[axis].plus(middle[axis]).minus(from[axis])),
    }
}

fn dot(one: [Dd; 3], other: [Dd; 3]) -> Dd {
    one[0]
        .times(other[0])
        .plus(one[1].times(other[1]))
        .plus(one[2].times(other[2]))
}

/// The pair `a_turned_ball_and_block_closes_an_edge_at_a_pole.rs` unites: a
/// ball of radius one at `(3, 3)` on the floor of a block, turned a quarter
/// turn about `x`.
fn the_fixture_itself() -> (Sphere, Plane) {
    let turn = a_turn([1, 0, 0], 16);
    (
        Sphere::new(Point::new(3.0, 3.0, 0.0), 1.0)
            .expect("a ball")
            .transformed(&turn),
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("a floor")
            .transformed(&turn),
    )
}

/// A ball sunk to its middle in a floor, the pair turned a quarter or three
/// quarters about `x` or `y` and moved somewhere: the floor stands up along
/// the world's `z`, so its circle runs from pole to pole and starts at one.
fn a_sunk_ball_turned_on_its_side(random: &mut SplitMix) -> (Sphere, Plane) {
    let size = random.size();
    let centre = Point::new(
        random.between(0.0, 5.0) * size,
        random.between(0.0, 5.0) * size,
        0.0,
    );
    let ball = Sphere::new(centre, random.between(0.3, 3.0) * size).expect("a ball");
    let facing = if random.coin() { -1.0 } else { 1.0 };
    let floor = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, facing))
        .expect("a floor");
    let direction = if random.coin() { [1, 0, 0] } else { [0, 1, 0] };
    let turn = a_turn(direction, if random.coin() { 16 } else { 48 });
    let (ball, floor) = (ball.transformed(&turn), floor.transformed(&turn));
    if random.coin() {
        return (ball, floor);
    }
    let shift = Isometry::translation(Vector::new(
        random.signed() * random.size(),
        random.signed() * random.size(),
        random.signed() * random.size(),
    ))
    .expect("a shift");
    (ball.transformed(&shift), floor.transformed(&shift))
}

/// A plane through a pole of a ball with its normal square to `x`: its circle
/// starts at that pole wherever the rounding lets it.
fn a_plane_through_a_pole(random: &mut SplitMix) -> (Sphere, Plane) {
    let size = random.size();
    let far = random.size();
    let centre = Point::new(
        random.signed() * far,
        random.signed() * far,
        random.signed() * far,
    );
    let radius = random.between(0.1, 10.0) * size;
    let ball = Sphere::new(centre, radius).expect("a ball");
    let up = if random.coin() { 1.0 } else { -1.0 };
    let pole = centre + Vector::new(0.0, 0.0, up * radius);
    let lean = match random.below(3) {
        0 => random.signed() * (-random.between(1.0, 60.0)).exp2(),
        1 => random.signed(),
        _ => random.signed() * random.between(1.0, 8.0),
    };
    let facing = if random.coin() { 1.0 } else { -1.0 };
    let flat = Plane::from_point_and_normal(pole, Vector::new(0.0, facing, lean))
        .expect("a plane through a pole");
    (ball, flat)
}

/// `pairs::Turn::isometry`.
fn a_turn(direction: [i32; 3], steps: u32) -> Isometry {
    let axis = Axis::new(Point::origin(), Vector::from(direction.map(f64::from)))
        .expect("a direction with length");
    Isometry::rotation(&axis, TAU * f64::from(steps) / 64.0).expect("a finite rotation")
}

/// A deterministic search needs no seed file and replays by itself.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f64 {
        // 53 random bits make every f64 in [0, 1) on the 2^-53 grid.
        #[allow(clippy::cast_precision_loss)]
        let bits = (self.next() >> 11) as f64;
        bits / 9_007_199_254_740_992.0
    }

    fn between(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }

    fn signed(&mut self) -> f64 {
        self.between(-1.0, 1.0)
    }

    fn coin(&mut self) -> bool {
        self.next() & 1 == 1
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    /// Anywhere from a thousandth to a million.
    fn size(&mut self) -> f64 {
        10f64.powf(self.between(-3.0, 6.0))
    }
}

/// A number held as the unrounded sum of two `f64`, about 106 bits.
///
/// Each step below is the textbook one (Dekker; Hida, Li and Bailey), and
/// each is good to a few units of `2^-104` of the numbers it combines — some
/// sixty binary places below the `2^-43` the box is measured in.
#[derive(Clone, Copy, Debug)]
struct Dd {
    hi: f64,
    lo: f64,
}

impl Dd {
    fn of(value: f64) -> Self {
        Self { hi: value, lo: 0.0 }
    }

    fn plus(self, other: Self) -> Self {
        let (sum, error) = two_sum(self.hi, other.hi);
        let (low_sum, low_error) = two_sum(self.lo, other.lo);
        let (sum, error) = quick_two_sum(sum, error + low_sum);
        let (hi, lo) = quick_two_sum(sum, error + low_error);
        Self { hi, lo }
    }

    fn minus(self, other: Self) -> Self {
        self.plus(Self {
            hi: -other.hi,
            lo: -other.lo,
        })
    }

    fn times(self, other: Self) -> Self {
        let product = self.hi * other.hi;
        let error = self.hi.mul_add(other.hi, -product);
        let (hi, lo) = quick_two_sum(product, error + self.hi * other.lo + self.lo * other.hi);
        Self { hi, lo }
    }

    fn over(self, other: Self) -> Self {
        let first = self.hi / other.hi;
        let left = self.minus(other.times(Self::of(first)));
        let second = left.hi / other.hi;
        let left = left.minus(other.times(Self::of(second)));
        let third = left.hi / other.hi;
        let (hi, lo) = quick_two_sum(first, second);
        Self { hi, lo }.plus(Self::of(third))
    }

    fn sqrt(self) -> Self {
        let guess = Self::of(self.hi.sqrt());
        guess.plus(self.minus(guess.times(guess)).over(guess.plus(guess)))
    }
}

fn two_sum(one: f64, other: f64) -> (f64, f64) {
    let sum = one + other;
    let other_part = sum - one;
    (sum, (one - (sum - other_part)) + (other - other_part))
}

fn quick_two_sum(big: f64, small: f64) -> (f64, f64) {
    let sum = big + small;
    (sum, small - (sum - big))
}
