//! Where two pipes running the same way, off the coordinate directions and far
//! from the origin, cross — every answer checked against the true two lines.
//!
//! §184.6 left this family unswept, and §184.7's defect lived in it: the window
//! around the middle of the two lines missed the true middle 119 763 times in
//! 279 636 before `67893d98`. §184.3's sweep could not be reused here, because
//! its side check assumes the two lines sit at `thin_z ± R2`, which is only true
//! along a coordinate direction.
//!
//! # How the true lines are found
//!
//! From the numbers the two pipes store, with nothing rounded. Write `o` and `c`
//! for the two axis origins, `w` for the shared direction, `g = c − o`,
//! `W = w·w`, `A = ‖g × w‖²` and `K = A + W·(r₁² − r₂²)`. The two lines are
//! `o + α·s ± β·(w × g)`, where `s` is the step from one axis across to the
//! other, `α = K / 2A` and `β² = D / (4A²W)` with `D = 4AW·r₁² − K²`. So the
//! pair crosses in two lines exactly when `D > 0`.
//!
//! A half-space `n·p ≥ h` whose normal is exactly square to `w` reads the two
//! lines, multiplied by `2A`, as `a ± √(D/W)·b` with `a = 2A·(n·o − h) + K·(n·g)`
//! and `b = n·(w × g)`. Every one of those is a polynomial in stored `f64`s, so
//! [`Dyadic`] holds it exactly and the sign of `a ± √(D/W)·b` is decided by
//! comparing `a²W` against `b²D`. Nothing the kernel rounded is read.
//!
//! # What counts as wrong
//!
//! A refusal is not wrong; it is counted apart. An answer is wrong when it is not
//! two runs where the pair crosses in two lines, when a run is not cut out of the
//! second pipe by one flat half-space, when that half-space's normal is not
//! exactly square to the pipes, or when the two half-spaces do not each hold
//! exactly one of the two true lines, a different one each.

#![forbid(unsafe_code)]

mod soak;

use std::cmp::Ordering;
use std::collections::BTreeMap;

use proptest::prelude::RngCore;
use proptest::test_runner::TestRng;
use rcad_geom::{
    Axis, Cylinder, EdgeRun, Error, HalfSpace, NamedRun, Picker, Point, Surface, SurfaceMeet,
    Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// How many pairs the shipped sweep draws. Each is asked both ways round.
const PAIRS: u32 = 4000;

/// Every answer the kernel gives for a pair far out and off the axes is right.
///
/// Also prints, and answers §184.6's question with, how many askings have at
/// least one exactly-square normal through a stored axis origin whose stored
/// offset separates the two true lines.
#[test]
fn every_answer_for_two_leaning_pipes_far_out_names_the_true_two_lines() {
    let mut rng = soak::seeded();
    let mut tally = Tally::default();
    for _ in 0..soak::cases(PAIRS) {
        let pair = a_leaning_pair_far_out(&mut rng);
        for first in [0, 1] {
            tally.ask(&pair[first], &pair[1 - first]);
        }
    }
    tally.report();

    assert!(
        tally.wrong.is_empty(),
        "{} of {} answers are wrong; the first: {}",
        tally.wrong.len(),
        tally.answered,
        tally.wrong[0]
    );
    assert!(
        tally.answered > tally.asked / 2,
        "a sweep that is mostly refused checks few answers: {} of {} answered",
        tally.answered,
        tally.asked
    );
    assert!(
        tally.not_two_lines < tally.asked / 5,
        "the family is drawn to cross in two lines, and only the draws on a \
         knife edge should round out of it: {} of {}",
        tally.not_two_lines,
        tally.asked
    );
}

/// The checker agrees with a pair worked by hand on the lattice, so a sweep
/// finding nothing wrong is not a checker that finds nothing wrong.
///
/// Two pipes along `z`, radius 5, axes through the origin and `(6, 0, 0)`, cross
/// in the lines `x = 3, y = ±4`. `y ≥ 0` holds one, `y ≤ 0` the other; `x ≥ 0`
/// holds both, and `y ≥ 4` holds one on its surface and none strictly — the
/// line `y = 4` reads zero and belongs to it, `y = −4` does not.
#[test]
fn the_checker_reads_two_pipes_along_z_the_way_a_hand_does() {
    let along = Vector::new(0.0, 0.0, 1.0);
    let one = a_pipe([0.0, 0.0, 0.0], along, 5.0);
    let other = a_pipe([6.0, 0.0, 0.0], along, 5.0);
    let truth = TrueLines::of(&one, &other);
    assert_eq!(truth.d.sign(), Ordering::Greater);

    let y_up = HalfSpace::new(Vector::new(0.0, 1.0, 0.0), 0.0).expect("a half-space");
    let x_up = HalfSpace::new(Vector::new(1.0, 0.0, 0.0), 0.0).expect("a half-space");
    let y_past_four = HalfSpace::new(Vector::new(0.0, 1.0, 0.0), 4.0).expect("a half-space");
    let held = |half_space: HalfSpace| truth.which_lines_it_holds(half_space).expect("square");

    assert_eq!(held(y_up), [true, false]);
    assert_eq!(held(y_up.reversed()), [false, true]);
    assert_eq!(held(x_up), [true, true]);
    assert_eq!(held(y_past_four), [true, false]);
    assert!(truth.separates(y_up) && !truth.separates(x_up) && !truth.separates(y_past_four));

    let leaning = HalfSpace::new(Vector::new(0.0, 1.0, 1e-300), 0.0).expect("a half-space");
    assert!(truth.which_lines_it_holds(leaning).is_none());
}

/// The check refuses the three ways a far-out answer could go wrong while still
/// looking like two runs: a half-space holding both lines, the same line picked
/// twice, and a run cut out of the wrong pipe.
///
/// §182's defect was the first of these — a plane holding both lines and a
/// reverse holding neither — and the sweep found none of it on the tree before
/// §182's fix either, so this is what shows the sweep would have said so.
#[test]
fn the_checker_refuses_an_answer_that_holds_both_lines_or_one_twice() {
    let mut rng = soak::seeded();
    let [one, other] = a_leaning_pair_far_out(&mut rng);
    let truth = TrueLines::of(&one, &other);
    let Ok(meet @ SurfaceMeet::Curves(_)) = intersect(
        &Surface::Cylinder(one),
        &Surface::Cylinder(other),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        panic!("the first pair the sweep draws is named");
    };
    assert_eq!(truth.agrees_with(&meet, &other), Ok(()));

    let SurfaceMeet::Curves(runs) = &meet else {
        unreachable!("matched above")
    };
    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = runs[0].run() else {
        panic!("a picked piece of a crossing");
    };
    let [Picker::InFrontOf(picker)] = picked_by[..] else {
        panic!("one flat picker");
    };
    let everything_in_front = HalfSpace::new(
        picker.normal(),
        picker.offset() - 1e12 * picker.normal().length(),
    )
    .expect("a half-space");
    let answer = |pickers: [HalfSpace; 2], and: Cylinder| {
        SurfaceMeet::Curves(
            pickers
                .into_iter()
                .map(|half_space| {
                    NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                        and: Surface::Cylinder(and),
                        picked_by: [Picker::InFrontOf(half_space)].into_iter().collect(),
                    })
                    .expect("a picked run")
                })
                .collect(),
        )
    };

    assert_eq!(
        truth.agrees_with(&answer([picker, picker.reversed()], other), &other),
        Ok(()),
        "rebuilt as the kernel builds it"
    );
    assert!(
        truth
            .agrees_with(
                &answer([everything_in_front, picker.reversed()], other),
                &other
            )
            .is_err()
    );
    assert!(
        truth
            .agrees_with(&answer([picker, picker], other), &other)
            .is_err()
    );
    assert!(
        truth
            .agrees_with(&answer([picker, picker.reversed()], one), &other)
            .is_err()
    );
}

/// A pair of pipes: the first written first.
type Pair = [Cylinder; 2];

/// Two pipes along one direction with no zero-length component to speak of,
/// their first axis origin between `1e3` and `1e8` from the origin, crossing in
/// two lines by construction.
///
/// The second origin is worked out in `f64` and so rounds; the truth is taken
/// from what was stored, so a rounding only moves where the lines are.
fn a_leaning_pair_far_out(rng: &mut TestRng) -> Pair {
    let raw = Vector::new(a_component(rng), a_component(rng), a_component(rng));
    let far = 10f64.powf(3.0 + 5.0 * unit(rng));
    let here = [0, 1, 2].map(|_| far * signed(rng) * (0.1 + 0.9 * unit(rng)));

    let fat = 10f64.powf(-7.0 + 11.0 * unit(rng));
    let thin = fat * 10f64.powf(-4.0 + 8.0 * unit(rng));
    let [least, most] = [(fat - thin).abs(), fat + thin];
    let share = match rng.next_u32() % 4 {
        0 => 10f64.powf(-12.0 * unit(rng)),
        1 => 1.0 - 10f64.powf(-12.0 * unit(rng)),
        _ => unit(rng),
    };
    let apart = least + (most - least) * share;

    let one = a_pipe(here, raw, fat);
    let along = one.axis().direction();
    let across = along
        .cross(Vector::new(
            signed(rng) * (0.1 + unit(rng)),
            signed(rng) * (0.1 + unit(rng)),
            signed(rng) * (0.1 + unit(rng)),
        ))
        .normalized()
        .expect("a random vector off the pipes");
    let slide = if rng.next_u32().is_multiple_of(4) {
        0.0
    } else {
        far * signed(rng) * unit(rng)
    };
    let there = Point::from(here) + across * apart + along * slide;
    let other = a_pipe(there.coordinates(), raw, thin);
    assert_eq!(
        one.axis().direction(),
        other.axis().direction(),
        "one raw direction stores one direction"
    );
    [one, other]
}

fn a_pipe(origin: [f64; 3], along: Vector, radius: f64) -> Cylinder {
    Cylinder::new(
        Axis::new(Point::from(origin), along).expect("a direction"),
        radius,
    )
    .expect("a radius")
}

/// A component well away from zero: a small whole number, or a mantissa drawn
/// off the lattice.
fn a_component(rng: &mut TestRng) -> f64 {
    let size = if rng.next_u32().is_multiple_of(2) {
        f64::from(1 + rng.next_u32() % 20)
    } else {
        0.05 + unit(rng)
    };
    size * signed(rng)
}

fn unit(rng: &mut TestRng) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let drawn = (rng.next_u64() >> 11) as f64;
    drawn / 9_007_199_254_740_992.0
}

fn signed(rng: &mut TestRng) -> f64 {
    if rng.next_u32().is_multiple_of(2) {
        1.0
    } else {
        -1.0
    }
}

#[derive(Default)]
struct Tally {
    asked: usize,
    answered: usize,
    named_in_runs: usize,
    refused: BTreeMap<String, usize>,
    refused_two_lines: usize,
    wrong: Vec<String>,
    not_two_lines: usize,
    some_stored_plane_separates: usize,
    refused_though_a_stored_plane_separates: usize,
}

impl Tally {
    fn ask(&mut self, one: &Cylinder, other: &Cylinder) {
        self.asked += 1;
        let truth = TrueLines::of(one, other);
        if truth.d.sign() != Ordering::Greater {
            self.not_two_lines += 1;
        }
        let separable = truth.some_stored_plane_separates(one, other);
        if separable {
            self.some_stored_plane_separates += 1;
        }

        let answer = intersect(
            &Surface::Cylinder(*one),
            &Surface::Cylinder(*other),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        match answer {
            Ok(meet) => {
                self.answered += 1;
                if matches!(meet, SurfaceMeet::Curves(_)) {
                    self.named_in_runs += 1;
                }
                if let Err(why) = truth.agrees_with(&meet, other) {
                    self.wrong.push(format!("{why}: {one:?} then {other:?}"));
                }
            }
            Err(refusal) => {
                *self.refused.entry(what_refused(&refusal)).or_default() += 1;
                if truth.d.sign() == Ordering::Greater {
                    self.refused_two_lines += 1;
                }
                if separable {
                    self.refused_though_a_stored_plane_separates += 1;
                }
            }
        }
    }

    fn report(&self) {
        let refused: usize = self.refused.values().sum();
        println!(
            "asked {}, answered {} ({} in runs), refused {} {:?} ({} of them two \
             true lines), wrong {}; not two lines {}; some exactly-square normal \
             through a stored origin separates in {} askings, {} of them refused",
            self.asked,
            self.answered,
            self.named_in_runs,
            refused,
            self.refused,
            self.refused_two_lines,
            self.wrong.len(),
            self.not_two_lines,
            self.some_stored_plane_separates,
            self.refused_though_a_stored_plane_separates,
        );
        for wrong in self.wrong.iter().take(5) {
            println!("wrong: {wrong}");
        }
    }
}

fn what_refused(refusal: &Error) -> String {
    let named = format!("{refusal:?}");
    named
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// The two lines two pipes along one stored direction cross in, held as the
/// exact polynomials the module comment names.
struct TrueLines {
    o: [Dyadic; 3],
    g: [Dyadic; 3],
    w: [Dyadic; 3],
    w_cross_g: [Dyadic; 3],
    two_a: Dyadic,
    k: Dyadic,
    big_w: Dyadic,
    d: Dyadic,
}

impl TrueLines {
    fn of(one: &Cylinder, other: &Cylinder) -> Self {
        let o = exactly(one.axis().origin().coordinates());
        let c = exactly(other.axis().origin().coordinates());
        let w = exactly(one.axis().direction().components());
        let g = [0, 1, 2].map(|i| c[i].minus(&o[i]));
        let w_cross_g = cross(&w, &g);
        let a = dot(&w_cross_g, &w_cross_g);
        let big_w = dot(&w, &w);
        let [r1, r2] = [one.radius(), other.radius()].map(Dyadic::of);
        let k = a.plus(&big_w.times(&r1.times(&r1).minus(&r2.times(&r2))));
        let d = Dyadic::of(4.0)
            .times(&a)
            .times(&big_w)
            .times(&r1)
            .times(&r1)
            .minus(&k.times(&k));
        Self {
            o,
            g,
            w,
            w_cross_g,
            two_a: a.plus(&a),
            k,
            big_w,
            d,
        }
    }

    /// Which of the two lines `n·p ≥ h` holds, or `None` where its normal is
    /// not exactly square to the lines and so holds only part of each.
    fn which_lines_it_holds(&self, half_space: HalfSpace) -> Option<[bool; 2]> {
        assert_eq!(self.d.sign(), Ordering::Greater, "two lines to hold");
        let n = exactly(half_space.normal().components());
        if dot(&n, &self.w).sign() != Ordering::Equal {
            return None;
        }
        let a = self
            .two_a
            .times(&dot(&n, &self.o).minus(&Dyadic::of(half_space.offset())))
            .plus(&self.k.times(&dot(&n, &self.g)));
        let b = dot(&n, &self.w_cross_g);
        Some([b.clone(), b.negated()].map(|b| self.sign_of(&a, &b) != Ordering::Less))
    }

    /// The sign of `a + √(D/W)·b`.
    fn sign_of(&self, a: &Dyadic, b: &Dyadic) -> Ordering {
        let [sa, sb] = [a.sign(), b.sign()];
        if sb == Ordering::Equal || sa == sb {
            return if sa == Ordering::Equal { sb } else { sa };
        }
        if sa == Ordering::Equal {
            return sb;
        }
        match a
            .times(a)
            .times(&self.big_w)
            .minus(&b.times(b).times(&self.d))
            .sign()
        {
            Ordering::Greater => sa,
            Ordering::Less => sb,
            Ordering::Equal => Ordering::Equal,
        }
    }

    fn separates(&self, half_space: HalfSpace) -> bool {
        matches!(
            (
                self.which_lines_it_holds(half_space),
                self.which_lines_it_holds(half_space.reversed())
            ),
            (Some([a, b]), Some([c, d])) if a != b && c != d && a != c
        )
    }

    /// Whether some normal exactly square to the pipes — each of the three
    /// written by swapping two of the direction's components and negating one —
    /// through either stored axis origin, with the offset
    /// [`HalfSpace::through_a_point_facing`] stores, separates the two lines.
    fn some_stored_plane_separates(&self, one: &Cylinder, other: &Cylinder) -> bool {
        if self.d.sign() != Ordering::Greater {
            return false;
        }
        let [x, y, z] = one.axis().direction().components();
        let normals = [
            Vector::new(0.0, z, -y),
            Vector::new(-z, 0.0, x),
            Vector::new(y, -x, 0.0),
        ];
        [one.axis().origin(), other.axis().origin()]
            .into_iter()
            .flat_map(|anchor| normals.map(|normal| (anchor, normal)))
            .filter(|(_, normal)| *normal != Vector::zero())
            .any(|(anchor, normal)| {
                HalfSpace::through_a_point_facing(anchor, normal)
                    .is_ok_and(|half_space| self.separates(half_space))
            })
    }

    /// `Ok` where `meet` is the right answer for these two lines, cut out of
    /// `other`.
    fn agrees_with(&self, meet: &SurfaceMeet, other: &Cylinder) -> Result<(), String> {
        let crosses = self.d.sign();
        let SurfaceMeet::Curves(runs) = meet else {
            return match (meet, crosses) {
                (SurfaceMeet::Nothing, Ordering::Less) => Ok(()),
                _ => Err(format!(
                    "answered {meet:?} where D is {crosses:?} than zero"
                )),
            };
        };
        if crosses != Ordering::Greater {
            return Err(format!(
                "answered {} runs where D is {crosses:?}",
                runs.len()
            ));
        }
        let [first, second] = &runs[..] else {
            return Err(format!("answered {} runs for two lines", runs.len()));
        };
        let [first, second] = [first, second].map(|run| match run.run() {
            EdgeRun::PartOfWhereItCrosses { and, picked_by }
                if *and == Surface::Cylinder(*other) =>
            {
                match picked_by[..] {
                    [Picker::InFrontOf(half_space)] => Ok(half_space),
                    _ => Err(format!("picked by {picked_by:?}")),
                }
            }
            elsewise => Err(format!("a run {elsewise:?}")),
        });
        let [first, second] = [first?, second?];
        let held = [first, second].map(|half_space| {
            self.which_lines_it_holds(half_space)
                .ok_or_else(|| format!("{half_space:?} is not exactly square to the pipes"))
        });
        let [first_holds, second_holds] = [held[0].clone()?, held[1].clone()?];
        let one_each = |[a, b]: [bool; 2]| a != b;
        if one_each(first_holds) && one_each(second_holds) && first_holds != second_holds {
            Ok(())
        } else {
            Err(format!(
                "{first:?} holds {first_holds:?} and {second:?} holds {second_holds:?} of the \
                 two true lines"
            ))
        }
    }
}

fn exactly(xyz: [f64; 3]) -> [Dyadic; 3] {
    xyz.map(Dyadic::of)
}

fn dot(one: &[Dyadic; 3], other: &[Dyadic; 3]) -> Dyadic {
    one[0]
        .times(&other[0])
        .plus(&one[1].times(&other[1]))
        .plus(&one[2].times(&other[2]))
}

fn cross(one: &[Dyadic; 3], other: &[Dyadic; 3]) -> [Dyadic; 3] {
    [(1, 2), (2, 0), (0, 1)].map(|(i, j)| one[i].times(&other[j]).minus(&one[j].times(&other[i])))
}

/// A whole number times a power of two, held with nothing rounded: every `f64`
/// is one, and sums and products of them stay one.
///
/// `magnitude` is little-endian 32-bit limbs with no zero limb on top; zero is
/// the empty magnitude.
#[derive(Clone, Debug)]
struct Dyadic {
    negative: bool,
    magnitude: Vec<u32>,
    exponent: i64,
}

impl Dyadic {
    fn of(value: f64) -> Self {
        assert!(value.is_finite(), "{value} is not a number to hold exactly");
        let bits = value.to_bits();
        let raw_exponent = i64::try_from((bits >> 52) & 0x7ff).expect("eleven bits");
        let fraction = bits & ((1 << 52) - 1);
        let (mantissa, exponent) = if raw_exponent == 0 {
            (fraction, -1074)
        } else {
            (fraction | (1 << 52), raw_exponent - 1075)
        };
        #[allow(clippy::cast_possible_truncation)]
        let magnitude = vec![mantissa as u32, (mantissa >> 32) as u32];
        Self::tidy(value.is_sign_negative(), magnitude, exponent)
    }

    fn tidy(negative: bool, mut magnitude: Vec<u32>, mut exponent: i64) -> Self {
        while magnitude.last() == Some(&0) {
            magnitude.pop();
        }
        if magnitude.is_empty() {
            return Self {
                negative: false,
                magnitude,
                exponent: 0,
            };
        }
        let zero_limbs = magnitude.iter().take_while(|limb| **limb == 0).count();
        magnitude.drain(..zero_limbs);
        exponent += 32 * i64::try_from(zero_limbs).expect("a limb count");
        Self {
            negative,
            magnitude,
            exponent,
        }
    }

    fn sign(&self) -> Ordering {
        match (self.magnitude.is_empty(), self.negative) {
            (true, _) => Ordering::Equal,
            (false, true) => Ordering::Less,
            (false, false) => Ordering::Greater,
        }
    }

    fn negated(&self) -> Self {
        Self::tidy(!self.negative, self.magnitude.clone(), self.exponent)
    }

    fn minus(&self, other: &Self) -> Self {
        self.plus(&other.negated())
    }

    fn plus(&self, other: &Self) -> Self {
        if self.magnitude.is_empty() {
            return other.clone();
        }
        if other.magnitude.is_empty() {
            return self.clone();
        }
        let exponent = self.exponent.min(other.exponent);
        let mine = shifted_up(&self.magnitude, self.exponent - exponent);
        let theirs = shifted_up(&other.magnitude, other.exponent - exponent);
        if self.negative == other.negative {
            return Self::tidy(self.negative, added(&mine, &theirs), exponent);
        }
        match compared(&mine, &theirs) {
            Ordering::Equal => Self::tidy(false, Vec::new(), 0),
            Ordering::Greater => Self::tidy(self.negative, subtracted(&mine, &theirs), exponent),
            Ordering::Less => Self::tidy(other.negative, subtracted(&theirs, &mine), exponent),
        }
    }

    fn times(&self, other: &Self) -> Self {
        let mut product = vec![0u32; self.magnitude.len() + other.magnitude.len()];
        for (i, mine) in self.magnitude.iter().enumerate() {
            let mut carry = 0u64;
            for (j, theirs) in other.magnitude.iter().enumerate() {
                let sum = u64::from(product[i + j]) + u64::from(*mine) * u64::from(*theirs) + carry;
                #[allow(clippy::cast_possible_truncation)]
                {
                    product[i + j] = sum as u32;
                }
                carry = sum >> 32;
            }
            #[allow(clippy::cast_possible_truncation)]
            {
                product[i + other.magnitude.len()] = carry as u32;
            }
        }
        Self::tidy(
            self.negative != other.negative,
            product,
            self.exponent + other.exponent,
        )
    }
}

fn shifted_up(magnitude: &[u32], bits: i64) -> Vec<u32> {
    let bits = usize::try_from(bits).expect("shifted up, never down");
    let (limbs, within) = (bits / 32, bits % 32);
    let mut shifted = vec![0u32; limbs];
    let mut carry = 0u32;
    for limb in magnitude {
        if within == 0 {
            shifted.push(*limb);
        } else {
            shifted.push((limb << within) | carry);
            carry = limb >> (32 - within);
        }
    }
    shifted.push(carry);
    shifted
}

fn compared(one: &[u32], other: &[u32]) -> Ordering {
    let top = |limbs: &[u32]| {
        limbs
            .iter()
            .rposition(|limb| *limb != 0)
            .map_or(0, |i| i + 1)
    };
    let [one, other] = [&one[..top(one)], &other[..top(other)]];
    one.len()
        .cmp(&other.len())
        .then_with(|| one.iter().rev().cmp(other.iter().rev()))
}

fn added(one: &[u32], other: &[u32]) -> Vec<u32> {
    let mut sum = Vec::with_capacity(one.len().max(other.len()) + 1);
    let mut carry = 0u64;
    for i in 0..one.len().max(other.len()) {
        let total =
            u64::from(*one.get(i).unwrap_or(&0)) + u64::from(*other.get(i).unwrap_or(&0)) + carry;
        #[allow(clippy::cast_possible_truncation)]
        sum.push(total as u32);
        carry = total >> 32;
    }
    #[allow(clippy::cast_possible_truncation)]
    sum.push(carry as u32);
    sum
}

/// `larger − smaller`, which the caller has already compared.
fn subtracted(larger: &[u32], smaller: &[u32]) -> Vec<u32> {
    let mut difference = Vec::with_capacity(larger.len());
    let mut borrow = 0i64;
    for (i, limb) in larger.iter().enumerate() {
        let mut total = i64::from(*limb) - i64::from(*smaller.get(i).unwrap_or(&0)) - borrow;
        borrow = i64::from(total < 0);
        if total < 0 {
            total += 1 << 32;
        }
        difference.push(u32::try_from(total).expect("one limb"));
    }
    assert_eq!(borrow, 0, "the larger was larger");
    difference
}
