use crate::expansion::{
    difference_of_two_products, expansion_sign, expansion_sum, product_is_exact, scale_expansion,
};
use crate::interval::determinant;
use crate::power_of_two::{exponent_that_brings_into_range, scaled_by_a_power_of_two};
use crate::{Decision, Error, Filtered, Interval, Point, Predicate, decide, orient3d};

/// Where three planes meet.
///
/// A plane is given as `[nx, ny, nz, d]`, meaning the points where
/// `nx*x + ny*y + nz*z + d` is zero. `rcad-geom` owns the real `Plane` type and
/// calls this with its coefficients; taking them as plain numbers keeps the
/// dependency running the right way round (decisions.md §7.2).
///
/// The answer is a [`Point::Constructed`] — a box the true meeting point is
/// somewhere inside — unless the meeting point is an `f64` triple and that can
/// be *proved*, in which case it is a [`Point::Exact`]. Cramer's rule needs a
/// division, and a division of intervals almost never lands on an `f64`, so
/// most boxes have real width.
///
/// That width is the whole point where it is real. A predicate over a
/// constructed point cannot fall through to exact arithmetic, so it can
/// genuinely come back [`Decision::Undecidable`] (decisions.md §2.5). What it
/// must never be is width the arithmetic invented: three walls that meet at a
/// lattice corner meet there whatever the walls are made of, and a box four
/// ulps wide round `(0.5, 1.5, 2)` refuses an ordering that has an answer
/// (decisions.md §131).
///
/// # The two levels, and why level 2 has to go first
///
/// Level 2 is the interval arithmetic below. Level 3 asks Cramer's rule again
/// through [`expansion`](crate::expansion), where nothing rounds, and it runs
/// only where level 2 left a side of the box with width — §122.2's ordering,
/// cheapest first.
///
/// Here that ordering is forced rather than preferred, and by something
/// stronger than cost: **level 3 has nothing to ask until level 2 has
/// answered.** Exact arithmetic here is `+`, `-` and `*` and has no division
/// (decisions.md §2.3), so it cannot produce a quotient — it can only check a
/// candidate, by multiplying it back through. Level 2's box is where the
/// candidates come from, and it is what makes the check sound: the true
/// coordinate is inside that box, so if exactly one `f64` in it multiplies back
/// to the numerator, that `f64` is the coordinate.
///
/// [`Point::Exact`]: crate::Point::Exact
///
/// # Errors
///
/// [`Error::PlanesDoNotMeetInAPoint`] when the three normals are coplanar — so
/// there is no single meeting point — or when they are close enough to coplanar
/// that we cannot bound one.
///
/// Coefficients that are uniformly tiny or uniformly enormous can make every
/// term of the determinant underflow or overflow even though the three planes
/// are perfectly well conditioned. That is decided anyway: each plane is
/// retried scaled by its own exact power of two
/// ([`meet_after_rescaling_each_plane`]), which moves it back into range
/// without changing which points satisfy it.
///
/// ```
/// # use rcad_predicates::{meet_of_three_planes, Point};
/// // x + y + z = 1, x = y, y = z. They meet at (1/3, 1/3, 1/3).
/// let meet = meet_of_three_planes(
///     [1.0, 1.0, 1.0, -1.0],
///     [1.0, -1.0, 0.0, 0.0],
///     [0.0, 1.0, -1.0, 0.0],
/// )
/// .unwrap();
/// assert!(meet.exact_coordinates().is_none());
/// assert!(meet.uncertainty() > 0.0, "a third is not an f64");
/// ```
///
/// [`Decision::Undecidable`]: crate::Decision::Undecidable
pub fn meet_of_three_planes(
    first: [f64; 4],
    second: [f64; 4],
    third: [f64; 4],
) -> Result<Point, Error> {
    let refused = match meet_of_three_planes_at_this_scale(first, second, third) {
        Ok(box_) => return Ok(pinned_down([first, second, third], box_)),
        Err(refused) => refused,
    };
    meet_after_rescaling_each_plane(first, second, third)
        .map(Ok)
        .unwrap_or(Err(refused))
}

/// Level 2: the box, from coefficients read exactly as given.
fn meet_of_three_planes_at_this_scale(
    first: [f64; 4],
    second: [f64; 4],
    third: [f64; 4],
) -> Result<[Interval; 3], Error> {
    let normals = [
        [
            Interval::exactly(first[0]),
            Interval::exactly(first[1]),
            Interval::exactly(first[2]),
        ],
        [
            Interval::exactly(second[0]),
            Interval::exactly(second[1]),
            Interval::exactly(second[2]),
        ],
        [
            Interval::exactly(third[0]),
            Interval::exactly(third[1]),
            Interval::exactly(third[2]),
        ],
    ];
    let offsets = [
        Interval::exactly(-first[3]),
        Interval::exactly(-second[3]),
        Interval::exactly(-third[3]),
    ];

    let volume = determinant(normals);
    if volume.contains_zero() {
        return Err(Error::PlanesDoNotMeetInAPoint {
            determinant_enclosure: volume.width(),
        });
    }

    let mut meet = [Interval::exactly(0.0); 3];
    for (axis, coordinate) in meet.iter_mut().enumerate() {
        let mut substituted = normals;
        for (row, offset) in substituted.iter_mut().zip(offsets) {
            row[axis] = offset;
        }
        *coordinate = determinant(substituted).divided_by(volume);
    }

    Ok(meet)
}

/// One more try, each plane moved by its own exact power of two. `None` when
/// one of the three has no such copy, or the rescaled copy is refused too.
///
/// A plane `[nx, ny, nz, d]` names the same points after every one of its four
/// coefficients is multiplied by the same nonzero number — that is what makes
/// it an equation rather than four independent numbers. So scaling plane `i`
/// by `sᵢ` changes nothing about where the three planes meet. What it changes
/// is whether the arithmetic can see that: [`meet_of_three_planes_at_this_scale`]
/// forms `normals[i]` and `offsets[i]` from that same plane's coefficients, so
/// scaling the plane scales both by exactly `sᵢ`, and every entry `substituted`
/// borrows from row `i` — whether from `normals[i]` or, for the column being
/// solved, from `offsets[i]` — scales by that same `sᵢ` too. A determinant is
/// linear in each of its rows, so both `volume` and every substituted
/// determinant scale by `s₁s₂s₃` together, and Cramer's ratio (the two divided
/// by each other) has that factor cancel exactly, row by row, whatever `s₁`,
/// `s₂` and `s₃` individually are — no shared exponent required, unlike
/// `orient3d`'s own retry, whose degree-3 homogeneity argument needs one
/// factor common to every coordinate.
///
/// Each plane picks its own exponent off its own largest coefficient
/// ([`exponent_that_brings_into_range`]), so three planes at three unrelated
/// scales are each brought into range on their own terms rather than forced
/// through one scale that might not fit all three at once. A plane that has no
/// exact scaled copy — an infinite or overflowing coefficient — sinks this
/// attempt and leaves the original refusal standing, which is correct: that
/// plane is not representable in range at any scale, so there is no copy to
/// have tried.
fn meet_after_rescaling_each_plane(
    first: [f64; 4],
    second: [f64; 4],
    third: [f64; 4],
) -> Option<Point> {
    let first = rescaled(first)?;
    let second = rescaled(second)?;
    let third = rescaled(third)?;
    let box_ = meet_of_three_planes_at_this_scale(first, second, third).ok()?;
    Some(pinned_down([first, second, third], box_))
}

/// `plane`, moved by the exact power of two that brings its own largest
/// coefficient into range. `None` when the plane is all zeros (nothing to
/// bring anywhere) or when moving it loses precision at either end of the
/// `f64` range.
fn rescaled(plane: [f64; 4]) -> Option<[f64; 4]> {
    let exponent = exponent_that_brings_into_range(plane)?;
    scaled_by_a_power_of_two(plane, exponent)
}

/// The point the box holds if level 3 can name it, and the box itself
/// otherwise.
fn pinned_down(planes: [[f64; 4]; 3], box_: [Interval; 3]) -> Point {
    match the_exact_point_the_box_holds(planes, box_) {
        Some(corner) => Point::Exact(corner),
        None => Point::Constructed(box_),
    }
}

/// Level 3: the corner the box holds, when that corner is an `f64` triple.
///
/// `None` when the meeting point is a fraction no `f64` names, when a product
/// would round, or when a side of the box is too wide to walk — the box is then
/// the honest answer and the caller hands it back.
///
/// # Why a check and not a division
///
/// Exact arithmetic here is `+`, `-` and `*` (decisions.md §2.3), so there is
/// no exact quotient to compute. What there is instead is a candidate and a
/// multiplication: for a side of the box holding the `f64` values `q₁ … qₙ`,
/// the coordinate is `qᵢ` exactly when `qᵢ · volume` is the substituted
/// determinant exactly, and both of those are expansions.
///
/// Two facts make that a proof rather than a guess. The true coordinate is
/// **inside** the box, because level 2's arithmetic is an enclosure — so if it
/// is an `f64` at all it is one of the `qᵢ`. And `volume` is **not zero**,
/// because level 2 certified that before it divided by it — so `q · volume`
/// takes a different value for every `q` and at most one candidate can match.
fn the_exact_point_the_box_holds(planes: [[f64; 4]; 3], box_: [Interval; 3]) -> Option<[f64; 3]> {
    let mut candidates = [[0.0; HOW_MANY_F64_ARE_WORTH_TRYING]; 3];
    let mut how_many = [0usize; 3];
    for (axis, side) in box_.into_iter().enumerate() {
        how_many[axis] = the_f64s_in(side, &mut candidates[axis])?;
    }

    // A side with one `f64` in it is that `f64`. The true coordinate is
    // somewhere inside the box and there is nowhere else in it to be, so a
    // box with one `f64` on every side needs no arithmetic at all.
    if how_many == [1, 1, 1] {
        return Some([candidates[0][0], candidates[1][0], candidates[2][0]]);
    }

    let normals = planes.map(|[nx, ny, nz, _]| [nx, ny, nz]);
    let mut volume = [0.0; DETERMINANT_ENTRIES];
    let volume_len = exact_determinant(normals, &mut volume)?;
    let volume = &volume[..volume_len];
    assert!(
        expansion_sign(volume) != Decision::Zero,
        "level 2 proved this determinant clear of zero before dividing by it, \
         and it encloses the exact one"
    );

    let mut corner = [0.0; 3];
    for (axis, coordinate) in corner.iter_mut().enumerate() {
        let mut substituted = normals;
        for (row, plane) in substituted.iter_mut().zip(planes) {
            row[axis] = -plane[3];
        }
        let mut numerator = [0.0; DETERMINANT_ENTRIES];
        let numerator_len = exact_determinant(substituted, &mut numerator)?;
        *coordinate = *candidates[axis][..how_many[axis]]
            .iter()
            .find(|&&candidate| {
                multiplies_back_to(candidate, volume, &numerator[..numerator_len])
            })?;
    }
    Some(corner)
}

/// How many `f64` a side of the box may hold before level 3 stops asking.
///
/// A **budget and not a bound**: a wider side can still hold an exact corner,
/// and this hands back the box for it rather than claiming there is none.
/// decisions.md §131 records that as what is still refused.
///
/// Sixteen is where it sits because a near miss is a handful of ulps wide, not
/// an interval: the §131 witness corner, which the old arithmetic landed on
/// exactly, comes out of level 2 six `f64` wide on its widest side. A side
/// wider than a handful is a coordinate level 2 genuinely did not pin down, and
/// walking it is unbounded work for a vanishing chance.
const HOW_MANY_F64_ARE_WORTH_TRYING: usize = 16;

/// Every `f64` in `side`, written into `out`, and how many there were.
///
/// `None` when there are more than [`HOW_MANY_F64_ARE_WORTH_TRYING`] of them,
/// or when an end of the range is not a finite number.
fn the_f64s_in(side: Interval, out: &mut [f64; HOW_MANY_F64_ARE_WORTH_TRYING]) -> Option<usize> {
    let (lo, hi) = (side.lo(), side.hi());
    if !(lo.is_finite() && hi.is_finite()) {
        return None;
    }
    let mut at = lo;
    for (used, slot) in out.iter_mut().enumerate() {
        *slot = at;
        if at >= hi {
            return Some(used + 1);
        }
        at = at.next_up();
    }
    None
}

/// Whether `candidate · volume` is `numerator`, exactly.
///
/// `false` rather than `None` when a product would round: an answer this cannot
/// multiply out is an answer it has not proved, and the two are the same thing
/// to the caller.
fn multiplies_back_to(candidate: f64, volume: &[f64], numerator: &[f64]) -> bool {
    if !volume
        .iter()
        .all(|&entry| product_is_exact(entry, candidate))
    {
        return false;
    }
    let mut scaled = [0.0; 2 * DETERMINANT_ENTRIES];
    let scaled_len = scale_expansion(volume, candidate, &mut scaled);

    let mut taken_away = [0.0; DETERMINANT_ENTRIES];
    for (slot, &entry) in taken_away.iter_mut().zip(numerator) {
        *slot = -entry;
    }

    let mut difference = [0.0; 3 * DETERMINANT_ENTRIES];
    let difference_len = expansion_sum(
        &scaled[..scaled_len],
        &taken_away[..numerator.len()],
        &mut difference,
    );
    expansion_sign(&difference[..difference_len]) == Decision::Zero
}

/// Which comes first along the line where the two `line` planes meet: the point
/// where `first` crosses it, or the point where `second` does.
///
/// The line is walked along `n₀ × n₁`, the cross product of the two `line`
/// normals in the order given. [`Decision::Pos`] when `first`'s point comes
/// first, [`Decision::Zero`] when the two are one point, [`Decision::Neg`] when
/// `second`'s does. Planes are written as for [`meet_of_three_planes`], and
/// which way `first` and `second` face does not matter.
///
/// # Why this and not the two meets
///
/// Each point is a meet of three planes and so a box, and where the two points
/// are a rounding apart the boxes overlap and cannot be put in order. Two
/// turned cubes a rounding apart stop there: a corner of one against the point
/// where the other's face crosses that corner's edge.
///
/// Walk the line as `p(t) = p₀ + t·(n₀ × n₁)`, with `first`'s point at `t₁`
/// and `second`'s at `t₂`. `second`'s value along the line is linear in `t`
/// and nothing at `t₂`, so at `t₁` it is `(t₁ − t₂)·(n₂ · (n₀ × n₁))`, and
/// `n₂ · (n₀ × n₁)` is `det(n₀, n₁, n₂)`. So `t₂ − t₁`, which is positive
/// exactly when `first`'s point comes first, has the sign of
/// `−side · det(n₀, n₁, n₂)`, where `side` is which side of `second` the point
/// `first` makes lies on — [`side_of_a_plane_where_three_planes_meet`]. Both
/// signs are worked out exactly where the intervals cannot tell, so two points
/// a rounding apart are still put in order and one point is still called one.
///
/// # Errors
///
/// [`Error::PlanesDoNotMeetInAPoint`] when `first` or `second` is proved to run
/// along the line, so that it crosses it nowhere or everywhere.
///
/// ```
/// # use rcad_predicates::{Decision, which_meet_comes_first_along_a_line};
/// // z = 0 and y = 0 meet in the x axis, walked along (0, 0, 1) × (0, 1, 0),
/// // which is -x. Walking that way, x = 2 comes before x = 1.
/// let line = [[0.0, 0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 0.0]];
/// assert_eq!(
///     which_meet_comes_first_along_a_line(line, [1.0, 0.0, 0.0, -2.0], [1.0, 0.0, 0.0, -1.0]),
///     Ok(Decision::Pos)
/// );
/// ```
pub fn which_meet_comes_first_along_a_line(
    line: [[f64; 4]; 2],
    first: [f64; 4],
    second: [f64; 4],
) -> Result<Decision, Error> {
    let normal = |[nx, ny, nz, _]: [f64; 4]| [nx, ny, nz];
    let into_second = orient3d(normal(line[0]), normal(line[1]), normal(second), [0.0; 3]);
    if into_second == Decision::Zero {
        return Err(Error::PlanesDoNotMeetInAPoint {
            determinant_enclosure: 0.0,
        });
    }
    let side = side_of_a_plane_where_three_planes_meet([line[0], line[1], first], second)?;
    Ok(match (side, into_second) {
        (undecided @ Decision::Undecidable { .. }, _)
        | (_, undecided @ Decision::Undecidable { .. }) => undecided,
        (Decision::Zero, _) => Decision::Zero,
        (Decision::Pos, Decision::Pos) | (Decision::Neg, Decision::Neg) => Decision::Neg,
        (Decision::Pos, Decision::Neg) | (Decision::Neg, Decision::Pos) => Decision::Pos,
        (_, Decision::Zero) => unreachable!("a second plane along the line was refused above"),
    })
}

/// Which side of `plane` the point where the `three` planes meet lies on.
///
/// Planes are written as for [`meet_of_three_planes`], and the side is the sign
/// of `nx*x + ny*y + nz*z + d` for `plane` at the meeting point: the same sign
/// `rcad-geom`'s `Plane::side_of_point` names.
///
/// # Why ask the planes and not the meet
///
/// The meet is a box, and a fourth plane that passes closer to the meeting
/// point than the box is wide straddles it, so the box cannot say which side
/// the point is on. That is the common case, not a rare one, whenever two
/// solids lie a rounding apart: a corner of one, three of its own planes,
/// against a face of the other that runs a rounding past it.
///
/// The planes can say. Stack the four as the rows of a four by four matrix `M`.
/// At the meeting point `p`, `M · (x, y, z, 1)` is `(0, 0, 0, s)`, where `s` is
/// the value this asks the sign of. Cramer's rule for the last unknown, which
/// is `1`, gives `s · det3 = det4`, where `det4` is `M`'s determinant and `det3`
/// the determinant of the three normals. So the sign of `s` is the sign of
/// `det4` times the sign of `det3`, and both are sums of products of the
/// sixteen coefficients. There is no division, so exact arithmetic can work
/// them out: level 3 runs, and a plane only a rounding off the meeting point is
/// still told apart from one through it.
///
/// A fourth plane whose normal is one of the three turned round, the way two
/// solids touching face to face stand, needs no branch of its own. Then two of
/// the four minors below are exactly zero, the third is `det3` turned round,
/// and `s` is the two offsets added, exactly.
///
/// # Errors
///
/// [`Error::PlanesDoNotMeetInAPoint`] when the three normals are proved to lie
/// in one plane, so there is no meeting point to ask about. Three normals that
/// only nearly do still meet, and are asked.
///
/// ```
/// # use rcad_predicates::{Decision, side_of_a_plane_where_three_planes_meet};
/// // x = 1, y = 0 and z = 0 meet at (1, 0, 0), which is on the far side of
/// // x + y + z = 2 from the plane's normal.
/// let three = [
///     [1.0, 0.0, 0.0, -1.0],
///     [0.0, 1.0, 0.0, 0.0],
///     [0.0, 0.0, 1.0, 0.0],
/// ];
/// assert_eq!(
///     side_of_a_plane_where_three_planes_meet(three, [1.0, 1.0, 1.0, -2.0]),
///     Ok(Decision::Neg)
/// );
/// ```
pub fn side_of_a_plane_where_three_planes_meet(
    three: [[f64; 4]; 3],
    plane: [f64; 4],
) -> Result<Decision, Error> {
    let question = FourPlanes {
        rows: [three[0], three[1], three[2], plane],
    };
    if question.the_three_normals_lie_in_one_plane() {
        return Err(Error::PlanesDoNotMeetInAPoint {
            determinant_enclosure: 0.0,
        });
    }
    Ok(decide(&question).0)
}

/// Three planes and a fourth, as the rows of one four by four matrix.
struct FourPlanes {
    rows: [[f64; 4]; 4],
}

impl FourPlanes {
    /// Whether the three normals' determinant is proved to be zero. Where a
    /// product would round nothing is proved, and the cascade cannot run level
    /// 3 either, so it comes back undecided rather than refused.
    fn the_three_normals_lie_in_one_plane(&self) -> bool {
        if !self.the_three_normals_turn().contains_zero() {
            return false;
        }
        let mut entries = [0.0; DETERMINANT_ENTRIES];
        exact_determinant(self.normals_without(3), &mut entries)
            .is_some_and(|len| expansion_sign(&entries[..len]) == Decision::Zero)
    }

    /// `det3`, in intervals.
    fn the_three_normals_turn(&self) -> Interval {
        determinant(
            self.normals_without(3)
                .map(|row| row.map(Interval::exactly)),
        )
    }

    /// The normals of every row but `left_out`, in order.
    fn normals_without(&self, left_out: usize) -> [[f64; 3]; 3] {
        let mut kept = [[0.0; 3]; 3];
        let rows = self
            .rows
            .iter()
            .enumerate()
            .filter(|&(row, _)| row != left_out);
        for (slot, (_, &[nx, ny, nz, _])) in kept.iter_mut().zip(rows) {
            *slot = [nx, ny, nz];
        }
        kept
    }

    /// The offset of `row`, with the sign its cofactor carries when `det4` is
    /// expanded along the offset column: `-`, `+`, `-`, `+` down the rows.
    fn signed_offset(&self, row: usize) -> f64 {
        let offset = self.rows[row][3];
        if row.is_multiple_of(2) {
            -offset
        } else {
            offset
        }
    }
}

impl Predicate for FourPlanes {
    fn name(&self) -> &'static str {
        "side of a plane where three planes meet"
    }

    /// Level 1 has no error bound worked out for it. This is asked where a box
    /// round the meet has already straddled the plane, so a filter would almost
    /// never answer, and a bound nobody needs is one more to get wrong.
    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    /// `det4 · det3`, which has the sign of `s`.
    fn on_intervals(&self) -> Interval {
        let det4 = (0..4)
            .map(|row| {
                Interval::exactly(self.signed_offset(row))
                    * determinant(
                        self.normals_without(row)
                            .map(|normal| normal.map(Interval::exactly)),
                    )
            })
            .fold(Interval::exactly(0.0), |total, term| total + term);
        det4 * self.the_three_normals_turn()
    }

    fn exactly(&self) -> Option<Decision> {
        let mut det3 = [0.0; DETERMINANT_ENTRIES];
        let det3_len = exact_determinant(self.normals_without(3), &mut det3)?;

        let mut det4 = [0.0; 4 * 2 * DETERMINANT_ENTRIES];
        let mut det4_len = 0;
        let mut merged = [0.0; 4 * 2 * DETERMINANT_ENTRIES];
        for row in 0..4 {
            let mut minor = [0.0; DETERMINANT_ENTRIES];
            let minor_len = exact_determinant(self.normals_without(row), &mut minor)?;
            let offset = self.signed_offset(row);
            if !minor[..minor_len]
                .iter()
                .all(|&entry| product_is_exact(entry, offset))
            {
                return None;
            }
            let mut term = [0.0; 2 * DETERMINANT_ENTRIES];
            let term_len = scale_expansion(&minor[..minor_len], offset, &mut term);
            det4_len = expansion_sum(&det4[..det4_len], &term[..term_len], &mut merged);
            core::mem::swap(&mut det4, &mut merged);
        }

        let turned_round = match expansion_sign(&det3[..det3_len]) {
            Decision::Pos => false,
            Decision::Neg => true,
            Decision::Zero => unreachable!(
                "side_of_a_plane_where_three_planes_meet refuses three normals in one plane \
                 before it asks"
            ),
            // An entry overflowed, so there is no sign to multiply by.
            Decision::Undecidable { .. } => return None,
        };
        match expansion_sign(&det4[..det4_len]) {
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos if turned_round => Some(Decision::Neg),
            Decision::Pos => Some(Decision::Pos),
            Decision::Neg if turned_round => Some(Decision::Pos),
            Decision::Neg => Some(Decision::Neg),
            Decision::Undecidable { .. } => None,
        }
    }
}

/// The exact determinant of a three by three matrix given as rows, written into
/// `total`, and how many entries it took.
///
/// Expanded along the third column, which is how [`determinant`] arranges the
/// interval version, so level 2 and level 3 evaluate the same expression.
///
/// `None` when a product would round, which is where exact arithmetic stops
/// being exact. Additions need no such check: they are exact for every pair of
/// `f64` that does not overflow, and an overflow reaches
/// [`expansion_sign`](crate::expansion::expansion_sign), which names no sign
/// for it.
fn exact_determinant(rows: [[f64; 3]; 3], total: &mut [f64; DETERMINANT_ENTRIES]) -> Option<usize> {
    let [first, second, third] = rows;
    let terms = [
        (
            difference_of_two_products(second[0], third[1], third[0], second[1])?,
            first[2],
        ),
        (
            difference_of_two_products(third[0], first[1], first[0], third[1])?,
            second[2],
        ),
        (
            difference_of_two_products(first[0], second[1], second[0], first[1])?,
            third[2],
        ),
    ];

    let mut total_len = 0;
    let mut merged = [0.0; DETERMINANT_ENTRIES];
    let mut term = [0.0; 8];
    for ((two_by_two, two_by_two_len), scale) in terms {
        let two_by_two = &two_by_two[..two_by_two_len];
        if !two_by_two
            .iter()
            .all(|&entry| product_is_exact(entry, scale))
        {
            return None;
        }
        let term_len = scale_expansion(two_by_two, scale, &mut term);
        total_len = expansion_sum(&total[..total_len], &term[..term_len], &mut merged);
        core::mem::swap(total, &mut merged);
    }
    Some(total_len)
}

/// Three two by two determinants of at most four entries, each doubled by its
/// own scaling and then summed.
const DETERMINANT_ENTRIES: usize = 24;

/// A number that every plane crossing `plane` in one line gives alike, boxed,
/// so that lines can be sorted by it: two planes whose boxes share no point
/// cross `plane` in two different lines.
///
/// It decides nothing on its own, and that is the point of it. Whether two
/// planes cross a third in the same line is four exact determinants of their
/// coefficients, and asking that of every pair of the four hundred planes
/// that cut one face of a pin header's body was the most expensive thing left
/// in cutting it (`docs/decisions.md` §200.1). Sorted by this, a line is
/// asked about only the few whose boxes share a point with its own, and every
/// line it is the same as is among them.
///
/// **Why the number is the line's and not the plane's.** Planes are
/// `[nx, ny, nz, d]` taken as vectors of four numbers. Every plane through the
/// line where `other` crosses `plane` is `a·plane + b·other` with `b` not
/// nothing, so taking away what each one has along `plane`,
/// `v = other·|plane|² − (other·plane)·plane`, leaves `b·|plane|²` times one
/// vector. `v vᵀ / |v|²` is then one matrix for all of them, whatever `a` and
/// `b` are, and the number is a fixed sum of its ten entries. Each step is
/// done on [`Interval`]s, which round outward, so the box holds the number
/// exactly.
///
/// `None` when `|v|²` cannot be told apart from nothing, which is `other`
/// being `plane` itself or near enough to it, or when the box is not finite.
///
/// ```
/// # use rcad_predicates::where_the_line_two_planes_cross_in_sorts;
/// // z = 0 meets x = 1 and x + z = 1 in the same line.
/// let floor = [0.0, 0.0, 1.0, 0.0];
/// let wall = where_the_line_two_planes_cross_in_sorts(floor, [1.0, 0.0, 0.0, -1.0]).unwrap();
/// let slope = where_the_line_two_planes_cross_in_sorts(floor, [1.0, 0.0, 1.0, -1.0]).unwrap();
/// assert!(wall.lo() <= slope.hi() && slope.lo() <= wall.hi());
/// ```
#[must_use]
pub fn where_the_line_two_planes_cross_in_sorts(
    plane: [f64; 4],
    other: [f64; 4],
) -> Option<Interval> {
    let dot = |one: [Interval; 4], two: [Interval; 4]| {
        one[0] * two[0] + one[1] * two[1] + one[2] * two[2] + one[3] * two[3]
    };
    let (plane, other) = (plane.map(Interval::exactly), other.map(Interval::exactly));
    let (across, along) = (dot(plane, plane), dot(other, plane));
    let v: [Interval; 4] = core::array::from_fn(|m| other[m] * across - along * plane[m]);
    let reciprocal = one_over(dot(v, v))?;
    let mut weights = WHAT_EACH_ENTRY_IS_WEIGHED_BY.iter();
    let mut sum = Interval::exactly(0.0);
    for row in 0..4 {
        for column in row..4 {
            let weight = weights
                .next()
                .expect("one weight for each of the ten entries on or above the diagonal");
            sum = sum + Interval::exactly(*weight) * v[row] * v[column];
        }
    }
    let number = sum * reciprocal;
    (number.lo().is_finite() && number.hi().is_finite()).then_some(number)
}

/// `1 / x` for every `x` in `positive`, boxed, or `None` when `positive` does
/// not lie wholly above nothing or does not end.
///
/// [`Interval`] has no division, and this needs none of its own: `1.0 / x` is
/// the `f64` nearest the true quotient, so the true quotient lies between the
/// two `f64` either side of it, and that holds where it overflows to infinity
/// or underflows to nothing too.
fn one_over(positive: Interval) -> Option<Interval> {
    if !(positive.lo() > 0.0 && positive.hi().is_finite()) {
        return None;
    }
    Interval::new(
        (1.0 / positive.hi()).next_down(),
        (1.0 / positive.lo()).next_up(),
    )
    .ok()
}

/// The square roots of the first ten primes: ten different numbers with
/// nothing to do with the lines being sorted, so that two different lines
/// rarely give the same sum. One that does costs four exact determinants and
/// no wrong answer.
const WHAT_EACH_ENTRY_IS_WEIGHED_BY: [f64; 10] = [
    1.414_213_562_373_095_1,
    1.732_050_807_568_877_2,
    2.236_067_977_499_79,
    2.645_751_311_064_590_7,
    3.316_624_790_355_4,
    3.605_551_275_463_989,
    4.123_105_625_617_661,
    4.358_898_943_540_674,
    4.795_831_523_312_719,
    5.385_164_807_134_504,
];

#[cfg(test)]
mod tests {
    use super::{
        meet_of_three_planes, scaled_by_a_power_of_two, side_of_a_plane_where_three_planes_meet,
        where_the_line_two_planes_cross_in_sorts, which_meet_comes_first_along_a_line,
    };
    use crate::{Decision, Error, Interval, Point};

    /// The repro decisions.md §39.7 item 7 names: three mutually perpendicular
    /// planes, every coefficient written at `1e-200`. The true meeting point is
    /// `(2, 3, 4)`, in whole numbers with nothing rounded anywhere else — only
    /// the uniform tiny scale stops the unscaled arithmetic seeing it, which is
    /// what makes this a gap rather than a limit.
    #[test]
    fn three_mutually_perpendicular_planes_written_at_1e_200_still_answer() {
        let s = 1e-200_f64;
        let meet = meet_of_three_planes(
            [s, 0.0, 0.0, -2.0 * s],
            [0.0, s, 0.0, -3.0 * s],
            [0.0, 0.0, s, -4.0 * s],
        )
        .expect("the meeting point exists and is representable");
        for (side, truth) in meet.enclosure().into_iter().zip([2.0, 3.0, 4.0]) {
            assert!(
                side.lo() <= truth && truth <= side.hi(),
                "{truth} is not inside [{}, {}]",
                side.lo(),
                side.hi()
            );
        }
    }

    /// Each plane is retried at the exact power of two that brings its own
    /// largest coefficient into range, not at one exponent shared by all
    /// three. Three unrelated exponents have to reach the same answer as one
    /// shared exponent, or the row-by-row cancellation the fix relies on is
    /// not what is actually happening.
    ///
    /// The two triples below name the same three planes — `x = 2`, `y = 3`,
    /// `z = 4` — each plane multiplied by its own exact power of two. Scaling
    /// by a power of two never touches a mantissa, so retrying brings each
    /// plane back to the one canonical copy with its largest coefficient in
    /// `[1, 2)`, and that copy does not depend on which power of two the row
    /// started at. Two experiments that start from different exponents and
    /// land on the same canonical copy have to answer bit for bit alike; if
    /// the cancellation needed a shared exponent the way `orient3d`'s does,
    /// scaling the three rows apart like this would change the answer instead.
    #[test]
    fn three_different_per_plane_exponents_give_the_same_box_as_one_shared_exponent() {
        let base = (
            [1.0, 0.0, 0.0, -2.0],
            [0.0, 1.0, 0.0, -3.0],
            [0.0, 0.0, 1.0, -4.0],
        );

        let shared = meet_of_three_planes(
            written_at(base.0, -700),
            written_at(base.1, -700),
            written_at(base.2, -700),
        )
        .expect("a shared exponent still leaves a gap the retry can close");

        let different = meet_of_three_planes(
            written_at(base.0, -700),
            written_at(base.1, -600),
            written_at(base.2, -800),
        )
        .expect("three different exponents leave the same gap open to the same retry");

        assert_eq!(
            shared, different,
            "each plane cancels its own scale row by row, so the exponents \
             the three planes were written at should not have to match"
        );
    }

    fn written_at(plane: [f64; 4], exponent: i32) -> [f64; 4] {
        scaled_by_a_power_of_two(plane, exponent)
            .expect("an exponent this modest always has an exact scaled copy")
    }

    /// A coefficient that is infinite has no scaled copy at any exponent
    /// (`scaled_by_a_power_of_two` refuses as soon as one is not `is_normal()`),
    /// so the retry sinks and the refusal it started from has to stand. An
    /// out-of-range plane is a limit, not a gap, and the fix must not blur
    /// that line.
    #[test]
    fn a_plane_with_an_infinite_coefficient_is_not_rescued_by_the_retry() {
        let outcome = meet_of_three_planes(
            [f64::INFINITY, 0.0, 0.0, -2.0],
            [0.0, 1.0, 0.0, -3.0],
            [0.0, 0.0, 1.0, -4.0],
        );
        assert!(
            matches!(outcome, Err(Error::PlanesDoNotMeetInAPoint { .. })),
            "an infinite coefficient must stay refused, got {outcome:?}"
        );
    }

    /// These divisions are all by one, so level 2 leaves one `f64` on every
    /// side of the box and there is nowhere else in it for the corner to be.
    /// A box with nothing in it but the answer **is** the answer, and saying so
    /// costs no exact arithmetic at all.
    #[test]
    fn three_axis_planes_meet_at_the_offsets_they_name() {
        let meet = meet_of_three_planes(
            [1.0, 0.0, 0.0, -2.0],
            [0.0, 1.0, 0.0, -3.0],
            [0.0, 0.0, 1.0, -4.0],
        )
        .unwrap();
        assert_eq!(meet, Point::Exact([2.0, 3.0, 4.0]));
        assert_eq!(meet.uncertainty(), 0.0);
    }

    /// §131's witness: the corner of a cube's top face where the cut `x = 0.5`
    /// meets the 45° cut `x + y = 2`.
    ///
    /// Written the way `Plane` stores it — a correctly rounded unit normal —
    /// which is what §121 changed. Cramer's numerator for the `y` here is
    /// `2c − 0.5c` for `c` the stored `1/√2`, and `1.5c` is an `f64` only when
    /// `c`'s significand is a multiple of four. The old `c` was one bit below
    /// the correctly rounded one and its significand was; the correctly rounded
    /// one's is not, so level 2 comes back four ulps wide where it used to come
    /// back exact.
    ///
    /// **That was luck in the last two bits and cannot be arranged back** — the
    /// nearest `f64` to `1/√2` is the nearest `f64` to `1/√2`. Level 3 gets the
    /// corner without any luck: `0.5`, `1.5` and `2` each multiply back through
    /// the determinant exactly, and no other `f64` in the box can.
    #[test]
    fn a_cut_at_45_degrees_through_a_lattice_corner_lands_on_the_corner() {
        assert_eq!(
            meet_of_three_planes(
                the_top_of_the_cube(),
                the_cut_across(),
                the_cut_at_45_degrees()
            ),
            Ok(Point::Exact([0.5, 1.5, 2.0]))
        );
    }

    /// The three walls of §131's witness, in the order `split_a_face` hands
    /// them over: the top of a cube of side two, the cut `y = 1.5`, and the cut
    /// `x + y = 2` written the way `Plane` stores it — a correctly rounded unit
    /// normal, which is what §121 changed.
    fn the_top_of_the_cube() -> [f64; 4] {
        [0.0, 0.0, 1.0, -2.0]
    }

    fn the_cut_across() -> [f64; 4] {
        [0.0, 1.0, 0.0, -1.5]
    }

    fn the_cut_at_45_degrees() -> [f64; 4] {
        let slope = (0.5f64).sqrt();
        [slope, slope, 0.0, -2.0 * slope]
    }

    /// The box level 2 leaves that corner in, and the measurement
    /// [`HOW_MANY_F64_ARE_WORTH_TRYING`](super::HOW_MANY_F64_ARE_WORTH_TRYING)
    /// is set against — so it is measured here rather than remembered.
    ///
    /// Six `f64` on the widest side is what a near miss looks like. It is not a
    /// near miss by luck either: the ordering that refused in `split_a_face`
    /// reported exactly this box, four ulps across, round a corner that is three
    /// whole numbers.
    #[test]
    fn level_two_leaves_that_corner_in_a_box_six_f64_wide() {
        let widths = super::meet_of_three_planes_at_this_scale(
            the_top_of_the_cube(),
            the_cut_across(),
            the_cut_at_45_degrees(),
        )
        .expect("three walls meeting at a corner")
        .map(|side| {
            let mut room = [0.0; super::HOW_MANY_F64_ARE_WORTH_TRYING];
            super::the_f64s_in(side, &mut room).expect("a narrow side")
        });
        assert_eq!(widths, [6, 4, 1]);
        assert!(
            widths.iter().max().expect("three sides") * 2 <= super::HOW_MANY_F64_ARE_WORTH_TRYING,
            "the budget has room over a real near miss"
        );
    }

    /// The budget is a budget: a side with exactly that many `f64` in it is
    /// still walked, and one more is handed back whole.
    #[test]
    fn a_side_one_f64_past_the_budget_is_not_walked() {
        let mut room = [0.0; super::HOW_MANY_F64_ARE_WORTH_TRYING];
        let mut hi = 1.0_f64;
        for _ in 1..super::HOW_MANY_F64_ARE_WORTH_TRYING {
            hi = hi.next_up();
        }
        assert_eq!(
            super::the_f64s_in(Interval::new(1.0, hi).expect("a range"), &mut room),
            Some(super::HOW_MANY_F64_ARE_WORTH_TRYING)
        );
        assert_eq!(
            super::the_f64s_in(
                Interval::new(1.0, hi.next_up()).expect("a range"),
                &mut room
            ),
            None
        );
    }

    #[test]
    fn a_meet_that_is_not_an_f64_comes_back_as_a_box_around_it() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();
        let Point::Constructed(box_) = meet else {
            panic!("a meet is always a constructed point");
        };
        for side in box_ {
            assert!(side.lo() <= 1.0 / 3.0 && 1.0 / 3.0 <= side.hi());
            assert!(side.width() > 0.0);
        }
        assert!(meet.uncertainty() < 1e-15);
    }

    #[test]
    fn two_parallel_planes_have_no_meeting_point() {
        let outcome = meet_of_three_planes(
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0, -1.0],
            [1.0, 0.0, 0.0, 0.0],
        );
        assert_eq!(
            outcome,
            Err(Error::PlanesDoNotMeetInAPoint {
                determinant_enclosure: 0.0
            })
        );
    }

    #[test]
    fn three_planes_sharing_a_line_have_no_single_meeting_point() {
        let outcome = meet_of_three_planes(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0, 0.0],
        );
        assert!(matches!(
            outcome,
            Err(Error::PlanesDoNotMeetInAPoint { .. })
        ));
    }

    /// Queue row Q397's witness: two cubes of side two, the second moved by
    /// `(2, 0.5, 0.25)`, both turned 1.1 about the axis through `(1, 0, 0)`
    /// along `(1, 2, 3)`. The turn left the first cube's face `x = 2` and the
    /// second's face `x = 0` facing exactly opposite ways, with offsets that
    /// sum to `2^-51`: a gap one rounding wide. These are the three planes of
    /// the first cube round one corner on that face, and the second cube's
    /// face, as `Plane::coefficients` hands them out.
    fn a_corner_a_rounding_short_of_the_other_cubes_face() -> ([[f64; 4]; 3], [f64; 4]) {
        let three = [
            [
                -0.593_456_917_111_829_7,
                0.004_012_095_033_902_652,
                -0.804_855_757_651_991_8,
                0.593_456_917_111_829_7,
            ],
            [
                -0.636_497_860_615_333_9,
                0.609_711_515_303_983_8,
                0.472_358_276_669_122_17,
                -1.363_502_139_384_666,
            ],
            [
                0.492_624_969_895_178_9,
                0.792_613_254_493_740_4,
                -0.359_283_826_294_219_95,
                -1.492_624_969_895_178_8,
            ],
        ];
        let the_other_face = [
            -0.492_624_969_895_178_9,
            -0.792_613_254_493_740_4,
            0.359_283_826_294_219_95,
            1.492_624_969_895_179_2,
        ];
        (three, the_other_face)
    }

    /// The meet's box straddles the other face, which is where the fuse gave
    /// up, and the planes say the corner is a rounding in front of it: the
    /// two offsets add to `2^-51`, and that is the whole of `s`.
    #[test]
    fn a_corner_a_rounding_short_of_a_plane_is_told_apart_from_one_on_it() {
        let (three, the_other_face) = a_corner_a_rounding_short_of_the_other_cubes_face();
        assert_eq!(three[2][3] + the_other_face[3], 2f64.powi(-51));

        let meet = meet_of_three_planes(three[0], three[1], three[2]).expect("a corner");
        let corner = meet.enclosure();
        let at_the_box = (0..3)
            .map(|axis| Interval::exactly(the_other_face[axis]) * corner[axis])
            .fold(Interval::exactly(the_other_face[3]), |total, term| {
                total + term
            });
        assert!(
            at_the_box.contains_zero(),
            "the box round the corner straddles the plane, so it cannot say: {at_the_box:?}"
        );

        assert_eq!(
            side_of_a_plane_where_three_planes_meet(three, the_other_face),
            Ok(Decision::Pos)
        );
    }

    /// The three planes' own order turns `det3` and `det4` round together, so
    /// the side does not move with it.
    #[test]
    fn the_order_the_three_planes_come_in_does_not_move_the_side() {
        let (three, the_other_face) = a_corner_a_rounding_short_of_the_other_cubes_face();
        for order in [
            [0, 1, 2],
            [1, 0, 2],
            [2, 1, 0],
            [0, 2, 1],
            [1, 2, 0],
            [2, 0, 1],
        ] {
            assert_eq!(
                side_of_a_plane_where_three_planes_meet(order.map(|at| three[at]), the_other_face),
                Ok(Decision::Pos),
                "{order:?}"
            );
        }
    }

    /// One of the three planes turned round passes through the meet, and `s`
    /// is then its two offsets added: exactly zero, not a rounding of it.
    #[test]
    fn one_of_the_three_turned_round_is_through_the_meet() {
        let (three, _) = a_corner_a_rounding_short_of_the_other_cubes_face();
        for plane in three {
            let turned_round = plane.map(|coefficient| -coefficient);
            assert_eq!(
                side_of_a_plane_where_three_planes_meet(three, turned_round),
                Ok(Decision::Zero)
            );
        }
    }

    /// The same corner against the other cube's face moved one rounding the
    /// other way, so that the two faces overlap by `2^-52`: the corner is then
    /// behind it.
    #[test]
    fn a_corner_a_rounding_past_a_plane_is_behind_it() {
        let (three, the_other_face) = a_corner_a_rounding_short_of_the_other_cubes_face();
        let mut moved = the_other_face;
        moved[3] = -three[2][3] - 2f64.powi(-52);
        assert_eq!(three[2][3] + moved[3], -(2f64.powi(-52)));
        assert_eq!(
            side_of_a_plane_where_three_planes_meet(three, moved),
            Ok(Decision::Neg)
        );
    }

    /// Queue row Q676's witness, one stage on from Q397's: along the edge the
    /// first two of the corner's planes cross in, the corner, where the first
    /// cube's face crosses the edge, against the point a rounding past it where
    /// the second cube's face does. The two meets' boxes overlap on every axis,
    /// so nothing read off them can put the two in order, and the planes can.
    ///
    /// Which way round is checked against the same face moved half a unit
    /// further from the corner, where the boxes are far apart and say it
    /// themselves. The corner is in front of the face, so that is a move back
    /// along the face's normal, and it cannot carry the face past the corner:
    /// the two orders are one order.
    #[test]
    fn a_corner_and_the_point_a_rounding_past_it_on_its_edge_are_put_in_order() {
        let (three, the_other_face) = a_corner_a_rounding_short_of_the_other_cubes_face();
        let line = [three[0], three[1]];
        let corner = meet_of_three_planes(line[0], line[1], three[2]).expect("a corner");
        let past_it = meet_of_three_planes(line[0], line[1], the_other_face).expect("a point");
        for (mine, theirs) in corner.enclosure().into_iter().zip(past_it.enclosure()) {
            assert!(
                mine.lo() <= theirs.hi() && theirs.lo() <= mine.hi(),
                "the two boxes overlap on every axis: {mine:?} {theirs:?}"
            );
        }

        let mut further_out = the_other_face;
        further_out[3] += 0.5;
        assert_eq!(
            side_of_a_plane_where_three_planes_meet(three, further_out),
            Ok(Decision::Pos),
            "the corner is still in front of the face"
        );
        let far_past_it = meet_of_three_planes(line[0], line[1], further_out).expect("a point");
        let [n0, n1] = line.map(|[nx, ny, nz, _]| [nx, ny, nz].map(Interval::exactly));
        let along = [
            n0[1] * n1[2] - n0[2] * n1[1],
            n0[2] * n1[0] - n0[0] * n1[2],
            n0[0] * n1[1] - n0[1] * n1[0],
        ];
        let how_far_on = (0..3)
            .map(|axis| (far_past_it.enclosure()[axis] - corner.enclosure()[axis]) * along[axis])
            .fold(Interval::exactly(0.0), |total, term| total + term);
        let the_boxes_say = how_far_on.sign();
        assert!(matches!(the_boxes_say, Decision::Pos | Decision::Neg));

        assert_eq!(
            which_meet_comes_first_along_a_line(line, three[2], further_out),
            Ok(the_boxes_say)
        );
        assert_eq!(
            which_meet_comes_first_along_a_line(line, three[2], the_other_face),
            Ok(the_boxes_say)
        );
    }

    /// Swapping the two points swaps which comes first, turning the line round
    /// does too, and turning either point's plane round moves nothing: the
    /// plane is the same plane and crosses the line at the same point.
    #[test]
    fn the_order_turns_with_the_line_and_the_two_points_and_nothing_else() {
        let (three, the_other_face) = a_corner_a_rounding_short_of_the_other_cubes_face();
        let line = [three[0], three[1]];
        let order = which_meet_comes_first_along_a_line(line, three[2], the_other_face)
            .expect("both cross the line");
        let the_other_way = match order {
            Decision::Pos => Decision::Neg,
            Decision::Neg => Decision::Pos,
            decided @ (Decision::Zero | Decision::Undecidable { .. }) => {
                panic!("a rounding apart is two points in some order, not {decided:?}")
            }
        };
        let turned = |plane: [f64; 4]| plane.map(|coefficient| -coefficient);
        assert_eq!(
            which_meet_comes_first_along_a_line(line, the_other_face, three[2]),
            Ok(the_other_way)
        );
        assert_eq!(
            which_meet_comes_first_along_a_line(
                [turned(line[0]), line[1]],
                three[2],
                the_other_face
            ),
            Ok(the_other_way)
        );
        assert_eq!(
            which_meet_comes_first_along_a_line([line[1], line[0]], three[2], the_other_face),
            Ok(the_other_way)
        );
        assert_eq!(
            which_meet_comes_first_along_a_line(line, turned(three[2]), the_other_face),
            Ok(order)
        );
        assert_eq!(
            which_meet_comes_first_along_a_line(line, three[2], turned(the_other_face)),
            Ok(order)
        );
    }

    /// A plane turned round crosses the line where it did, so the two points
    /// are one point: nothing, exactly, and not a rounding of it.
    #[test]
    fn a_plane_and_itself_turned_round_cross_the_line_at_one_point() {
        let (three, _) = a_corner_a_rounding_short_of_the_other_cubes_face();
        assert_eq!(
            which_meet_comes_first_along_a_line(
                [three[0], three[1]],
                three[2],
                three[2].map(|coefficient| -coefficient)
            ),
            Ok(Decision::Zero)
        );
    }

    /// `y = 1` runs along the x axis, which `z = 0` and `y = 0` meet in, so it
    /// crosses it nowhere, whichever of the two points it is asked for.
    #[test]
    fn a_plane_along_the_line_crosses_it_at_no_one_point() {
        let line = [[0.0, 0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 0.0]];
        let across = [1.0, 0.0, 0.0, -1.0];
        let along = [0.0, 1.0, 0.0, -1.0];
        for (first, second) in [(across, along), (along, across)] {
            assert_eq!(
                which_meet_comes_first_along_a_line(line, first, second),
                Err(Error::PlanesDoNotMeetInAPoint {
                    determinant_enclosure: 0.0
                })
            );
        }
    }

    #[test]
    fn three_normals_in_one_plane_have_no_meet_to_ask_about() {
        assert_eq!(
            side_of_a_plane_where_three_planes_meet(
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0, -1.0],
                ],
                [0.0, 0.0, 1.0, 0.0],
            ),
            Err(Error::PlanesDoNotMeetInAPoint {
                determinant_enclosure: 0.0
            })
        );
    }

    #[test]
    fn planes_too_close_to_parallel_to_bound_are_refused() {
        // The determinant is exactly -2^-60, but the products it comes from
        // round at 2^-53, so level 2 cannot tell it from zero.
        let nudge = 2f64.powi(-30);
        let outcome = meet_of_three_planes(
            [1.0 + nudge, 1.0, 0.0, -1.0],
            [1.0, 1.0 - nudge, 0.0, -1.0],
            [0.0, 0.0, 1.0, 0.0],
        );
        let Err(Error::PlanesDoNotMeetInAPoint {
            determinant_enclosure,
        }) = outcome
        else {
            panic!("expected a refusal, got {outcome:?}");
        };
        assert!(determinant_enclosure > 0.0, "the planes are not parallel");
    }

    /// Every plane through one line on a tilted face sorts the line where its
    /// boxes share a point, however the plane is written: turned round,
    /// scaled, or leaned about the line. Each is written with nothing rounded,
    /// so each is exactly a plane through the line. A plane moved off it by a
    /// ten-millionth sorts somewhere else. Moved by one `f64` step it may not,
    /// which costs four exact determinants and no wrong answer.
    #[test]
    fn every_plane_through_one_line_sorts_it_alike() {
        let face = [0.6, 0.0, 0.8, -1.0];
        let wall = [0.0, 1.0, 0.0, -2.0];
        let through_the_line = [
            wall,
            wall.map(|coefficient| -coefficient),
            wall.map(|coefficient| coefficient * 3.0),
            core::array::from_fn(|m| wall[m] + face[m]),
            core::array::from_fn(|m| 0.5 * wall[m] - 2.0 * face[m]),
        ];
        let sorted = |other: [f64; 4]| {
            where_the_line_two_planes_cross_in_sorts(face, other)
                .expect("none of these is the face's own plane")
        };
        let first = sorted(wall);
        for other in through_the_line {
            let it = sorted(other);
            assert!(
                it.lo() <= first.hi() && first.lo() <= it.hi(),
                "{other:?} crosses the face in the line y = 2 and sorts at {it:?}, not {first:?}"
            );
        }
        let shifted = sorted([0.0, 1.0, 0.0, -2.000_000_1]);
        assert!(
            shifted.lo() > first.hi() || shifted.hi() < first.lo(),
            "y = 2 and y = 2.0000001 are two lines, sorted at {shifted:?} and {first:?}"
        );
    }

    /// The face's own plane, written any way, crosses it in no line, and has
    /// nothing to sort by.
    #[test]
    fn the_faces_own_plane_has_no_line_to_sort() {
        let face = [0.0, 0.0, 1.0, -1.0];
        assert!(where_the_line_two_planes_cross_in_sorts(face, face).is_none());
        assert!(where_the_line_two_planes_cross_in_sorts(face, face.map(|c| -2.0 * c)).is_none());
    }
}
