//! The 24 turns that take a cube onto itself move each coordinate onto another
//! and negate some, so turning a point by one of them rounds nothing. This file
//! pins that every one of them is turned that way, however it was reached.
//!
//! **Why it matters.** A bar turned onto `x` by `Isometry::rotation(&Axis::y(),
//! FRAC_PI_2)` comes out pointing along `(1, 0, 2⁻⁵²)`, so it misses the centre
//! of a ball it was meant to pass through by `20·2⁻⁵²`, and the two cross in two
//! loops that are not circles
//! (`a_bar_turned_by_frac_pi_2_leans_off_the_balls_centre_and_is_answered_under_every_rule`).
//! `Isometry::quarter_turns` names the quarter turn exactly, so the bar lies
//! along `x` and passes through the centre, where the two cross in two circles
//! instead.
//!
//! The oracle is the signed swap of coordinates itself, compared bit for bit.
//! No tolerance appears here except in
//! [`a_point_on_an_axis_through_anywhere_stays_within_two_epsilon_of_itself`],
//! which is the one published bound in `quarter_turns` and is checked against
//! its own derivation.

use core::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2};

use rcad_geom::{Axis, Error, Isometry, Point, Vector};

/// Which way each axis is sent, as the nine numbers of the turn's matrix:
/// `matrix[row][column]` is component `row` of where axis `column` goes.
type Matrix = [[i8; 3]; 3];

/// `quarter_turns` about `x`, `y` and `z` in every combination: all 24 turns of
/// a cube and nothing else, each one exact, and each one still exact after its
/// inverse undoes it.
///
/// "All 24" is checked against a list made without any turn at all: the
/// signed swaps of three coordinates whose determinant is `+1`.
#[test]
fn composing_quarter_turns_about_x_y_and_z_reaches_all_24_turns_and_each_is_exact() {
    let reached = every_turn_composing_quarter_turns_reaches();
    let mut matrices: Vec<Matrix> = reached.iter().map(|(matrix, _)| *matrix).collect();
    matrices.sort_unstable();
    assert_eq!(
        matrices,
        every_signed_swap_that_does_not_mirror(),
        "composing quarter turns reached other turns than the 24 of a cube"
    );

    let mut numbers = Numbers::seeded();
    for (matrix, turn) in &reached {
        let shift = Isometry::translation(numbers.a_vector()).expect("a finite shift");
        let placed = shift.compose(turn);
        for _ in 0..1000 {
            let vector = numbers.a_vector();
            assert_turned_exactly(&placed, *matrix, vector);
            let point = Point::new(vector.x(), vector.y(), vector.z());
            let swapped = swapped_by(*matrix, vector.components());
            let by = shift.translation_part().components();
            let expected = [0, 1, 2].map(|axis| swapped[axis] + by[axis]);
            assert_eq!(
                placed.apply_to_point(point).coordinates().map(f64::to_bits),
                expected.map(f64::to_bits),
                "{matrix:?} then a shift moved {point:?} by more than one addition per coordinate"
            );

            let undone = placed.inverse();
            assert_turned_exactly(&undone, transposed(*matrix), vector);
            assert_eq!(
                undone.compose(&placed).apply_to_point(point),
                point,
                "{matrix:?} composed with its own inverse moved {point:?}"
            );
        }
    }
}

/// A long chain of turns of a cube composed one after another is still a turn
/// of a cube, stored the way that turn is always stored.
///
/// Multiplying stored coefficients rounds, and two thousand products of
/// coefficients a rounding off `1/√2` would leave the four no longer of one
/// size — no longer recognisably a turn of a cube, and turned by a formula
/// that rounds. So the stored coefficients are checked to be exactly `±1`,
/// `±1/√2` or `±1/2` at every step.
#[test]
fn two_thousand_quarter_turns_in_a_row_are_still_a_turn_of_a_cube() {
    let quarter_turns = [Axis::x(), Axis::y(), Axis::z()].map(|axis| {
        [1, -1].map(|quarters| Isometry::quarter_turns(&axis, quarters).expect("an axis along x"))
    });
    let mut numbers = Numbers::seeded();
    let mut chain = Isometry::identity();
    for step in 0..2000 {
        let next = &quarter_turns[(numbers.next() % 3) as usize][(numbers.next() % 2) as usize];
        chain = if numbers.next().is_multiple_of(2) {
            next.compose(&chain)
        } else {
            chain.compose(next)
        };
        let stored = chain.rotation_quaternion();
        let size = stored.iter().copied().map(f64::abs).fold(0.0, f64::max);
        assert!(
            [1.0, FRAC_1_SQRT_2, 0.5].contains(&size)
                && stored.iter().all(|c| *c == 0.0 || c.abs() == size),
            "after {step} quarter turns the stored turn is {stored:?}"
        );
        assert_turned_exactly(&chain, matrix_of(&chain), numbers.a_vector());
    }
}

/// Every one of the 48 sign patterns a file can write a turn of a cube as, at
/// sizes from `1e-200` to `1e200`, is read as that turn, turned exactly, stored
/// one way, and written out and read back as the same numbers.
///
/// `q` and `−q` name one turn, so the 48 patterns are the 24 turns twice.
#[test]
fn every_sign_pattern_a_file_can_hold_is_read_as_its_turn_and_reads_back_bit_for_bit() {
    let mut numbers = Numbers::seeded();
    let mut matrices = Vec::new();
    for pattern in every_sign_pattern_of_a_turn_of_a_cube() {
        let shift = numbers.a_vector().components();
        let at_one = Isometry::try_from((shift, pattern)).expect("a turn of a cube");
        let matrix = matrix_of(&at_one);
        matrices.push(matrix);
        for size in [
            1.0,
            3.0,
            0.1,
            FRAC_1_SQRT_2,
            0.707_106_781_186_547_5,
            1e-200,
            1e200,
        ] {
            let written = pattern.map(|sign| sign * size);
            let read = Isometry::try_from((shift, written)).expect("a turn of a cube");
            assert_eq!(
                read.rotation_quaternion().map(f64::to_bits),
                at_one.rotation_quaternion().map(f64::to_bits),
                "{pattern:?} written at {size:e} is stored differently from at one"
            );
            let bytes = postcard::to_allocvec(&read).expect("a movement writes out");
            let read_again: Isometry = postcard::from_bytes(&bytes).expect("and reads back");
            assert_eq!(
                read_again.rotation_quaternion().map(f64::to_bits),
                read.rotation_quaternion().map(f64::to_bits),
                "{pattern:?} written at {size:e} came back as other numbers"
            );
            assert_eq!(
                read_again.translation_part().components().map(f64::to_bits),
                shift.map(f64::to_bits),
                "{pattern:?} lost its shift on the way through a file"
            );
            for _ in 0..50 {
                assert_turned_exactly(&read_again, matrix, numbers.a_vector());
            }
        }
    }
    matrices.sort_unstable();
    let mut each_twice: Vec<Matrix> = every_signed_swap_that_does_not_mirror()
        .into_iter()
        .flat_map(|matrix| [matrix, matrix])
        .collect();
    each_twice.sort_unstable();
    assert_eq!(
        matrices, each_twice,
        "the 48 sign patterns are not the 24 turns of a cube twice over"
    );
}

/// A quarter turn about an axis through `(5, 5, 0)` is that turn and not the
/// one about the parallel axis through the origin: it carries the shift that
/// puts the axis back, every point on the axis stays where it is bit for bit,
/// and it agrees with `Isometry::rotation` about the same axis to within the
/// rounding that one carries.
#[test]
fn a_quarter_turn_about_an_axis_off_the_origin_keeps_its_shift() {
    let axis = Axis::new(Point::new(5.0, 5.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("an axis");
    let turn = Isometry::quarter_turns(&axis, 1).expect("an axis along z");
    assert_eq!(turn.translation_part(), Vector::new(10.0, 0.0, 0.0));
    for height in [-3.5, 0.0, 1e-300, 7.0, 1e300] {
        let on_the_axis = Point::new(5.0, 5.0, height);
        assert_eq!(turn.apply_to_point(on_the_axis), on_the_axis);
    }
    assert_eq!(
        turn.apply_to_point(Point::new(6.0, 5.0, 2.0)),
        Point::new(5.0, 6.0, 2.0)
    );

    let rounded = Isometry::rotation(&axis, FRAC_PI_2).expect("a finite angle");
    let mut numbers = Numbers::seeded();
    for _ in 0..1000 {
        let point = numbers.a_point();
        let apart = turn
            .apply_to_point(point)
            .distance_to(rounded.apply_to_point(point));
        assert!(
            apart <= 1e-15 * (10.0 + point.largest_coordinate_magnitude()),
            "{point:?} lands {apart:e} away from where rotation(FRAC_PI_2) puts it"
        );
    }

    let three_back = Isometry::quarter_turns(&axis, -3).expect("an axis along z");
    assert_eq!(three_back, turn, "three quarters back is one forward");
    assert_eq!(
        Isometry::quarter_turns(&axis, 4).expect("an axis along z"),
        Isometry::identity(),
        "four quarter turns are none"
    );
}

/// §16.10 for `quarter_turns`' "within twice the one subtraction per
/// coordinate": `2·f64::EPSILON` times the largest coordinate of the point the
/// axis is written through.
///
/// The derivation: the shift is `fl(oᵢ − rᵢ)` with `rᵢ = ±oⱼ` turned exactly,
/// which slips by `eᵢ`, at most half an ulp of a number no bigger than `2L`, so
/// `|eᵢ| ≤ ε·L`. A point `p` on the axis is turned exactly and the true sum
/// `turned(p)ᵢ + oᵢ − rᵢ` is `pᵢ` itself, so what is computed is `fl(pᵢ + eᵢ)`,
/// and since `pᵢ` is a number the nearest one to `pᵢ + eᵢ` is within `|eᵢ|`
/// of it: `2|eᵢ|` from `pᵢ` at most.
///
/// It also asserts some points did move, so the bound is not tested on draws
/// that happen to be exact.
#[test]
fn a_point_on_an_axis_through_anywhere_stays_within_two_epsilon_of_itself() {
    let mut numbers = Numbers::seeded();
    let (mut moved, mut worst) = (0, 0.0_f64);
    for _ in 0..20_000 {
        let through = numbers.a_point();
        let which = (numbers.next() % 3) as usize;
        let mut along = [0.0; 3];
        along[which] = if numbers.next().is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        let axis = Axis::new(through, Vector::from(along)).expect("an axis along x, y or z");
        let quarters = (numbers.next() % 3) as i32 + 1;
        let turn = Isometry::quarter_turns(&axis, quarters).expect("an axis along x, y or z");

        let mut on_the_axis = through.coordinates();
        on_the_axis[which] = numbers.a_point().x();
        let on_the_axis = Point::from(on_the_axis);
        let landed = turn.apply_to_point(on_the_axis);

        let largest = through.largest_coordinate_magnitude();
        for (now, was) in landed
            .coordinates()
            .into_iter()
            .zip(on_the_axis.coordinates())
        {
            let slipped = (now - was).abs();
            assert!(
                slipped <= 2.0 * f64::EPSILON * largest,
                "{quarters} quarter turns about {axis:?} moved {on_the_axis:?} to {landed:?}"
            );
            if slipped > 0.0 {
                moved += 1;
                worst = worst.max(slipped / (2.0 * f64::EPSILON * largest));
            }
        }
    }
    println!(
        "{moved} coordinates moved, the worst by {:.1}% of the bound",
        worst * 100.0
    );
    assert!(
        moved > 1000,
        "only {moved} coordinates of points on an axis moved at all"
    );
}

/// About an axis that does not run exactly along `x`, `y` or `z` a quarter turn
/// rounds, so it is refused by name rather than handed back as something it is
/// not.
#[test]
fn quarter_turns_about_a_leaning_axis_are_refused_by_name() {
    for direction in [
        Vector::new(1.0, 1.0, 0.0),
        Vector::new(1e-300, 0.0, 1.0),
        Vector::new(1.0, 2.0, 3.0),
    ] {
        let leaning = Axis::new(Point::origin(), direction).expect("an axis");
        assert_eq!(
            Isometry::quarter_turns(&leaning, 1),
            Err(Error::QuarterTurnsNeedAnAxisAlongXYOrZ {
                direction: leaning.direction()
            })
        );
    }
}

/// §16.10 for the doc's "no `f64` `j` has `fl(j·2j) = 1`", which is why no
/// stored quaternion put through Rodrigues' form turns `z` exactly onto `x`.
///
/// A quarter turn about `y` stored as `(w, 0, j, 0)` sends `z` to
/// `(fl(2j·w), 0, 1 − fl(j·2j))` through Rodrigues' form, whatever `w` is.
/// `2j` is exact and rounding never reverses an order, so `fl(j·2j)` never
/// falls as `|j|` grows, and the `j` where it is exactly `1` would be one run
/// of neighbours. The run is empty: the last `j` below it and the first above
/// it are neighbours.
///
/// Then `rotation(&Axis::y(), FRAC_PI_2)` is shown landing where that says,
/// and `quarter_turns` landing on `x`.
#[test]
fn rodrigues_form_cannot_turn_z_exactly_onto_x_whatever_the_stored_numbers() {
    let twice_its_square = |j: f64| j * (2.0 * j);
    let mut below = FRAC_1_SQRT_2;
    while twice_its_square(below) >= 1.0 {
        below = f64::from_bits(below.to_bits() - 1);
    }
    let mut above = FRAC_1_SQRT_2;
    while twice_its_square(above) <= 1.0 {
        above = f64::from_bits(above.to_bits() + 1);
    }
    assert_eq!(
        above.to_bits() - below.to_bits(),
        1,
        "some j between {below:e} and {above:e} has fl(j·2j) = 1"
    );
    assert!(twice_its_square(below) < 1.0 && twice_its_square(above) > 1.0);

    let straight_up = Vector::new(0.0, 0.0, 1.0);
    let rounded = Isometry::rotation(&Axis::y(), FRAC_PI_2).expect("a finite angle");
    let [_, _, j, _] = rounded.rotation_quaternion();
    assert_eq!(
        rounded.apply_to_vector(straight_up),
        Vector::new(1.0, 0.0, 1.0 - twice_its_square(j)),
        "rotation(FRAC_PI_2) about y no longer turns z the way Rodrigues' form does"
    );
    assert_eq!(
        rounded.apply_to_vector(straight_up).z(),
        2f64.powi(-52),
        "the lean Q1245 traced"
    );
    assert_eq!(
        Isometry::quarter_turns(&Axis::y(), 1)
            .expect("y is a coordinate axis")
            .apply_to_vector(straight_up),
        Vector::new(1.0, 0.0, 0.0)
    );
}

/// Every turn composing quarter turns about `x`, `y` and `z` reaches, starting
/// from none, each with the matrix it turns by.
fn every_turn_composing_quarter_turns_reaches() -> Vec<(Matrix, Isometry)> {
    let quarter_turns = [Axis::x(), Axis::y(), Axis::z()]
        .map(|axis| Isometry::quarter_turns(&axis, 1).expect("an axis along x, y or z"));
    let mut reached = vec![(matrix_of(&Isometry::identity()), Isometry::identity())];
    let mut next = 0;
    while next < reached.len() {
        let (_, from) = reached[next];
        for quarter in &quarter_turns {
            let turned = quarter.compose(&from);
            let matrix = matrix_of(&turned);
            if reached.iter().all(|(seen, _)| *seen != matrix) {
                reached.push((matrix, turned));
            }
        }
        next += 1;
    }
    reached
}

/// The matrix `turn` turns by, read off where it sends the three axes, which
/// must each land exactly on an axis.
fn matrix_of(turn: &Isometry) -> Matrix {
    let mut matrix = [[0; 3]; 3];
    for column in 0..3 {
        let mut axis = [0.0; 3];
        axis[column] = 1.0;
        let landed = turn.apply_to_vector(Vector::from(axis)).components();
        for row in 0..3 {
            matrix[row][column] = if landed[row] == 1.0 {
                1
            } else if landed[row] == -1.0 {
                -1
            } else {
                assert!(
                    landed[row] == 0.0,
                    "{turn:?} sends axis {column} to {landed:?}, whose component {row} is not a \
                     whole number"
                );
                0
            };
        }
    }
    matrix
}

/// The turn that undoes `matrix`, which for a turn is its transpose.
fn transposed(matrix: Matrix) -> Matrix {
    let mut flipped = [[0; 3]; 3];
    for (row, entries) in matrix.iter().enumerate() {
        for (column, entry) in entries.iter().enumerate() {
            flipped[column][row] = *entry;
        }
    }
    flipped
}

/// The 24 matrices that swap three coordinates about and negate some without
/// mirroring, made from the swaps and the signs alone.
fn every_signed_swap_that_does_not_mirror() -> Vec<Matrix> {
    let swaps = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut every = Vec::new();
    for swap in swaps {
        for signs in 0..8 {
            let mut matrix = [[0; 3]; 3];
            for row in 0..3 {
                matrix[row][swap[row]] = if signs >> row & 1 == 0 { 1 } else { -1 };
            }
            if determinant(matrix) == 1 {
                every.push(matrix);
            }
        }
    }
    every.sort_unstable();
    assert_eq!(every.len(), 24, "a cube has 24 turns");
    every
}

fn determinant(m: Matrix) -> i32 {
    let m = m.map(|row| row.map(i32::from));
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Every `[w, i, j, k]` of `-1`, `0` and `1` with one, two or four that are not
/// zero: 8, 24 and 16 of them.
fn every_sign_pattern_of_a_turn_of_a_cube() -> Vec<[f64; 4]> {
    let mut every = Vec::new();
    for code in 0..81 {
        let pattern = [0, 1, 2, 3].map(|place| f64::from((code / 3_i32.pow(place)) % 3 - 1));
        let non_zero = pattern.iter().filter(|sign| **sign != 0.0).count();
        if matches!(non_zero, 1 | 2 | 4) {
            every.push(pattern);
        }
    }
    assert_eq!(every.len(), 48, "8 + 24 + 16 sign patterns");
    every
}

/// `vector` swapped about and negated by `matrix`, which rounds nothing.
fn swapped_by(matrix: Matrix, vector: [f64; 3]) -> [f64; 3] {
    matrix.map(|row| {
        let from = row
            .iter()
            .position(|entry| *entry != 0)
            .expect("each row of a turn of a cube has one entry");
        if row[from] < 0 {
            -vector[from]
        } else {
            vector[from]
        }
    })
}

fn assert_turned_exactly(turn: &Isometry, matrix: Matrix, vector: Vector) {
    assert_eq!(
        turn.apply_to_vector(vector).components().map(f64::to_bits),
        swapped_by(matrix, vector.components()).map(f64::to_bits),
        "{matrix:?} turned {vector:?} other than by swapping its components"
    );
}

/// The same numbers every run, spread over sixty octaves so a turn that
/// rounded would show at some of them.
struct Numbers(u64);

impl Numbers {
    fn seeded() -> Self {
        Self(0x9e37_79b9_7f4a_7c15)
    }

    fn a_vector(&mut self) -> Vector {
        Vector::new(self.a_number(), self.a_number(), self.a_number())
    }

    fn a_point(&mut self) -> Point {
        Point::new(self.a_number(), self.a_number(), self.a_number())
    }

    fn a_number(&mut self) -> f64 {
        let bits = self.next();
        let mantissa = (bits >> 11) as f64 / (1u64 << 53) as f64;
        let octave = ((bits >> 3) % 60) as i32 - 30;
        let sign = if bits & 1 == 0 { 1.0 } else { -1.0 };
        sign * (1.0 + mantissa) * 2.0f64.powi(octave)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
