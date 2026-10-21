## 314. The room between two meridian circles, and what it loses — 2026-09-27

*Queue row Q200, named by Q117 and not probed there. Lane
`lane-q200-ring-less-tube-enclosed`, off `e7a437b4`. The code is
`two_meridian_circles`, `how_far_a_rounded_cross_product_turns` and
`how_far_a_picking_half_space_sits_off_a_point` in
`crates/rcad-geom/src/intersect.rs`; the tests are
`crates/rcad-geom/tests/the_room_between_two_meridian_circles.rs` (commit
`6ce15e67`).*

  314.1   The question
  314.2   Rounding R − r never mattered, and why
  314.3   What did cost something: the reading was twice too wide
  314.4   The turn the factor of two was hiding
  314.5   Why R − r is enclosed now
  314.6   The units
  314.7   What this leaves
  314.8   Enclosing R − r takes a whole f64 off the room
  314.9   Subtracting the turn takes a whole f64 off the room too

### 314.1 The question

A plane through a torus's axis cuts two circles, centred `R` either side of
the axis and `r` across. They are told apart by the half-space through the
centre facing `w × n`. Each circle is `R − r` clear of that surface, and the
picker is named only when what the picker loses is provably less than `R − r`.

The code took `R − r` as `Interval::exactly(R - r)`: the `f64` difference,
claimed exact. That subtraction is exact when `r ≥ R/2` (Sterbenz) and rounds
by up to half an ulp otherwise, so the margin could start half an ulp wide.
The row asked whether a torus exists whose offset rounding uses up the true
`R − r` and not its rounding, and which circle gets named on it.

### 314.2 Rounding R − r never mattered, and why

**No such torus exists for the code as it was**, and the reason is not luck.
The old margin was `fl(R − r) − q`, where `q` is the upper end of one interval
and so **one `f64`**. It named the pair when `fl(R − r) > q`. Rounding to
nearest is monotone and leaves an `f64` where it is, so `R − r ≤ q` would give
`fl(R − r) ≤ fl(q) = q`. Read the other way: `fl(R − r) > q` means
`R − r > q`, and `q` is at least the distance the picker is off. Put plainly,
no `f64` lies between `R − r` and its rounding, so a bound that is an `f64`
cannot fall in the gap.

The family searched is the only one where rounding `R − r` could matter at
all. It takes `r < R/2`, or the difference is exact, and then `R − r > R/2`.
The picker's offset is `w × n · c` rounded once from the exact dot, so it
misses by at most half an ulp of that dot. For half an ulp of the dot to reach
`R/2`, the centre has to sit about `2⁵²` ring radii out along `w × n`. The
tests put a torus on `z = 0` at `x = 2⁵³ + 8`, axis along `(1, 1, 0)`: the
picking normal `(h, −h, 0)` has nothing rounded in it, so the offset is the
only thing off, by `0.3431457505…`. Radii built so that
`R − r = fl(R − r) − 109·2⁻⁶²` sit the true room just under that distance and
its rounding just over it.

`a_picker_off_the_centre_by_all_of_the_room_is_refused_although_the_room_rounds_up_past_it`
is that torus. It is refused on the base and refused now. With `R − r` put
back to its rounding on top of this lane's other changes, it is still refused.
All 13 tests then in that file and `the_shapes_a_plane_cuts_a_torus_in.rs`
passed under that change.
That is 314.2's argument measured: there is one loss term in this family,
because the turn in 314.4 is exactly zero there.

### 314.3 What did cost something: the reading was twice too wide

`how_far_a_picking_half_space_sits_off_a_point` read the offset through an
interval dot. That interval ends at the two `f64` either side of the true dot,
and the offset is the nearer of them, so the far end is at least **twice** the
true rounding away. On the torus above, that reading is a whole unit against a
true `0.343`.

So a torus with `R − r` between the distance and twice the distance was
refused. That is a gap: the circles are apart and each lies wholly on its own
side. `a_picker_off_the_centre_by_more_than_half_the_room_still_names_both_circles`
is one: `R = 1`, `r = 0.4`, the picker `0.343` off centre. **On the base it is
refused**, because a unit is more than `0.6`. Now the reading comes from
`Exact::dot_of` minus the offset, which is exact, and it is named. When the
exact dot declines (a product below `2⁻⁹⁶⁹`), the reading falls back to the
interval, as before.

The helper has one other caller, `two_loops_either_side_of_a_pipe`. Its reading
gets tighter in the same way. It is still an upper bound on the distance, which
is all that caller's argument uses.

**2026-10-05, Q304: the pipe did not lean on the factor of two.** Its margin
reads `α` and `β` off the picker's normal as stored, so a turned normal is
charged in `β`, which the torus's `R − r` never was. Measured in
`crates/rcad-geom/tests/two_loops_either_side_of_a_pipe.rs`: a fat pipe along
`(1, 1, 0)` through `(0, 0, 2⁵³ + 8)`, a thin one through the same point,
widened one `f64` at a time to the widest named, and the room its nearer loop
leaves worked out to about a hundred bits. Tightest is a fallback picker,
where `β` is a third and not a rounding: along `(1, 0, 2)` the offset misses
by `0.485`, an interval read `1.414`, and the widest pipe named leaves
`3.8e-15`
(`the_widest_thin_pipe_named_through_a_leaning_picker_leaves_its_near_loop_clear`).
A picker turned `5.5e-17`, along `(1, −2, 2)`, leaves `6.6e-8`
(`the_widest_thin_pipe_named_through_a_turned_picker_leaves_its_near_loop_clear`).
All 324 directions with whole components up to 4 leave at least `1.7e-15`
(`every_widest_thin_pipe_named_leaves_its_near_loop_clear_of_the_picker`).

### 314.4 The turn the factor of two was hiding

The picker's normal is `w × n` in `f64`, and those products round, so the
picker can be turned a little off the true surface between the circles. A
surface through the centre turned by `θ` leaves each circle `R·cos θ − r`
clear. That loses `R·(1 − cos θ)`, at most `R·sin²θ`. With the old reading, the
factor of two covered it: a named pair had the distance under half of `R − r`,
and the turn is tiny. This section said `10⁻³²·R` without measuring it; the
two tori 314.9 measures have a bound of `7·10⁻³⁵` and `1.4·10⁻³⁴`. A reading
this tight no longer leaves that room, so the turn is now bounded on its own.
`across` is the true product plus a rounding `e`, so `sin θ ≤ ‖e‖/‖across‖`,
with `e` taken from `Exact::cross_of`. If that product cannot be worked out
exactly, the pair is refused.

`a_leaning_plane_through_the_axis_still_names_both_circles` guards the bound's
direction: a plane with normal `(1, −1, 1)` through an axis along `(1, 1, 0)`,
where `w × n` does round. **Its picker is not turned**, which this paragraph
used to say it was: `w × n` comes out `(p, −p, −2p)` with `p = fl(h·a)`,
parallel to the true `h·a·(1, −1, −2)`. Only the bound is nonzero there,
`((h·a − p)/p)²`, about `7·10⁻³⁵`, and the test now asserts its size. With the
exact product crossed the wrong way round, that test fails along with four
others in these two files, because `e` becomes twice the picker itself.
Re-measured on 2026-10-08 with 314.8's and 314.9's tests in the file: 17 run,
7 failed, those five and both new ones (314.8, 314.9).

### 314.5 Why R − r is enclosed now

The margin is now `R − r` less **two** bounds, the turn and the reading. Their
sum need not be an `f64`, so 314.2's argument does not carry to it: a sum can
fall between `R − r` and its rounding. `R − r` is therefore
`Interval::exactly(R) − Interval::exactly(r)`, whose lower end is at most the
true difference.

**No test failed if that enclosure was taken out**, when this was written. In
the family of 314.2 the turn is exactly zero, so there is one loss term again.
A torus where the enclosure decides the answer needs a nonzero turn of about
`10⁻³²·R`, plus a distance tuned to within that of `R − r`. Nobody built one.
That is a statement of what was not searched, not a claim that no such torus
exists.

**2026-10-08, Q302: one does now, and the other way round.** 314.8 is a torus
of 314.2's family, turn exactly zero, on which the enclosure decides the
answer by **refusing** a pair the `f64` difference names, and names rightly.
The enclosure's low end is a whole `f64` under the room whenever `R − r`
rounds up. What it was put in to keep out, a wrong pair, nobody has built.

### 314.6 The units

The old doc comment said the margin is `(R − r)·‖w × n‖`, which would be in the
picker's own reading units. The code compared `R − r` with
`how_far_a_picking_half_space_sits_off_a_point`, and that divides the reading
by `‖w × n‖` as stored. **Both sides were lengths.** The comparison was sound
and the prose was wrong, whatever length `w × n` has. The comment now says
lengths.

### 314.7 What this leaves

- A torus on which the outward `R − r` decides the answer (314.5). It needs a
  plane through the axis where `w × n` rounds, and a distance within `R·sin²θ`
  of `R − r`. **Answered, Q302 (2026-10-08), in 314.8**, and the need was
  wrong: on a plane where `w × n` does not round, a torus whose room is a
  quarter step under one `f64` past the picker is refused with `R − r`
  enclosed and named without. The turn's bound decides an answer the same way,
  in 314.9 (Q303). Both are gaps, filed together as one row to decide the
  margin's sign exactly.
- `select.rs` and `holds.rs` each keep their own copy of the meridian-circle
  lookup (§250.5), which names a circle from a corner and does not go through
  this function. Whether their margins have the same shape was not checked.
  **Answered, Q305 (2026-10-05): there is no second margin.** The two copies
  were one by Q117 (250.8), `the_picker_naming_the_circle_through` in
  `holds.rs`, and that lookup asked `intersect` for its pickers from its first
  commit (`6c9ab819`, in `select.rs`). For a plane through the axis,
  `intersect` reaches `two_meridian_circles`, so the lookup gets this
  section's pickers. Its only arithmetic of its own is
  `Picker::side_of_point(corner)`, which for `InFrontOf` is
  `HalfSpace::side_of_point`, exact about the four numbers the half-space
  holds. The sentence above was wrong when it was written.

### 314.8 Enclosing R − r takes a whole f64 off the room

*2026-10-08, queue rows Q302 and Q303, lane
`lane-q302-q303-a-torus-each-meridian-bound-decides`, off `7f9d0d3c`. Nothing
in `src/` changed; the tests are in
`crates/rcad-geom/tests/the_room_between_two_meridian_circles.rs`.*

The margin's low end is worked out one subtraction at a time, each rounded
down: `fl↓(fl↓(lo(R − r) − T) − O)`, where `T` is the top of `R·turned` and
`O` the top of the picker's distance, both an `f64`. Where nothing in `w × n`
rounds, `turned` is exactly `[0, 0]` and `T` drops out. `O` is at least `up`,
the `f64` at or above the true distance, and on 314.2's torus it is `up`
itself: a room of exactly the `f64` past `up` is named.

`lo(R − r)` is `R − r` rounded down. When `r < R/2` and `R − r` rounds up,
that is the `f64` below its rounding, a whole step under the true room. Take
`R = next(up) + 2⁻¹⁰` and `r = 2⁻¹⁰ + 2⁻⁵⁶`, where a step is `2⁻⁵⁴`: the true
room is a quarter step under `next(up)` and rounds to it, the enclosure starts
at `up`, and `up − up` is not above zero, so the pair is **refused**. The true
room clears the true distance by three quarters of a step and more, and
nothing is turned, so both circles lie wholly on their own sides of the
picker. **The answer exists and the refusal is a gap**, of class refuses/D:
the centre is `2⁵³ + 8` out and the tube `2⁻¹⁰` across, a spread of about
`9·10¹⁸` against §3.5's `1e11`.
`rounding_the_ring_less_the_tube_down_takes_a_whole_f64_off_the_room` pins
it, beside the torus of room exactly `next(up)`, which is named.

**The Q302 mutation fails that test and nothing else.** In
`two_meridian_circles`, `let margin = ring_radius` followed by
`- Interval::exactly(ring.tube_radius())` became
`let margin = Interval::exactly(ring.ring_radius() - ring.tube_radius())`.
`mise exec -- just t -p rcad-geom -E 'binary(merged) &
(test(the_room_between_two_meridian_circles) |
test(the_shapes_a_plane_cuts_a_torus_in))' --no-fail-fast` printed
`17 tests run: 16 passed, 1 failed`, the one being that test. The fifteen
tests that were in those two files before it all pass under the mutation, as
314.2's thirteen did.

So 314.5 had the danger the wrong way round. On this torus 314.2's argument
holds: one loss term, which is an `f64`, so the `f64` difference names the
pair, and names it rightly. Whether the enclosure ever keeps a wrong pair out
**is argued and not run**, so this section does not claim it never does. The
argument: with `T > 0`, the `f64` difference `F` gives `prev(F) − O` after
rounding down, and `prev(F)` is under the true `R − r`. That difference is of
two `f64`, so it is at least one step of `O`, and when `O` is near enough
`R − r > R/2` for a refusal to be close, that is about `R·2⁻⁵⁴`. `T` is far
smaller (314.9).

The fix the row asks for takes both this loss and 314.9's away: decide the
sign of `R − r − T − O` exactly, as a sum of four `f64`. `T` and `O` are
already upper bounds, so nothing else rounds.

### 314.9 Subtracting the turn takes a whole f64 off the room too

The turn's bound is tiny, but when it is positive `fl↓(room − T)` is the `f64`
below the room. So it costs a whole step, not `R·sin²θ`.

The torus has its axis along `(1, 5, 0)` through `(2⁵³ + 8, 0, 0)`, cut by the
plane through the axis with normal `(0, 0, 1 − 3·2⁻⁵³)`, kept as written by
`Plane::from_point_and_normal_as_written`. `w × n` is `(fl(w₂·z), −fl(w₁·z), 0)`
and both products round, so the bound is `1.4·10⁻³⁴` and costs `R` times that,
about `2·10⁻³⁵`. The true distance is just over the `f64` nearest it, so `up`
is the one after, `0.15535459447264185`, where a step is `2⁻⁵⁵`. A room two
`f64` past `up` is named, which shows `O` is `up` itself. A room one `f64` past
`up` is **refused**: less `T`, rounded down, it is `up`, and `up − up` is not
above zero. That room clears the true distance by more than a step, and the
turn costs under `2⁻¹⁰⁰` of it, so **the answer exists and the refusal is a
gap**, refuses/D for 314.8's reason.
`subtracting_a_turn_worth_ten_to_the_minus_35_takes_a_whole_f64_off_the_room`
pins it. Its other half, the room two past `up` named, is what makes the
refusal the turn's: `O` is at least `up` whatever happens.

**The Q303 mutation fails that test and nothing else.** Deleting the line
`- ring_radius * turned` from the margin, the same command printed
`17 tests run: 16 passed, 1 failed`, the one being that test.

Why this axis. Along `(1, 1, 0)` with a leaning normal `(0, 0, z < 1)` the turn
is positive, but `O` was never `up`: the first room named was three or more
steps past it. `O` comes from an interval quotient that steps one `f64` out
whenever the quotient is inexact, so it is usually the `f64` after `up`. The
lane before this one probed axes `(1, k, 0)` for `k` in 1, 2, 3, 5 and 7,
normals `(0, 0, 1 − j·2⁻⁵³)` for `j` from 1 to 4, and centres `2⁵³ + 8 + 2m`
for `m` from 0 to 29, walking five rooms up from the `f64` before `up`. The
first named came two past `up`, so that `O` was `up`, at 4 of the 600 pairs:
all on `(1, 5, 0)` at `j = 3`, with centres `2⁵³ + 8` plus 0, 8, 28 and 36.
This lane ran it again at `7c4da5ab`, the commit holding the probe since
deleted, with `mise exec -- just t probe_where_the_threshold_sits --no-capture`:
it printed `along=5 hist=[0, 0, 0, 4, 58, 58]`, and
`along=1 hist=[0, 0, 0, 0, 68, 52]` for `(1, 1, 0)`, 68 first named three past
`up` and 52 not within the five.

The turn's size, measured with the tests' own `f64` estimate of the bound: on
this torus `1.4·10⁻³⁴`, and on the leaning plane of 314.4 `6.9·10⁻³⁵`, which
costs `2.8·10⁻³⁴` at `R = 4`. Each test asserts its cost to within a factor of
ten.

Whether the turn's bound ever keeps a wrong pair out is not built. It could
matter only where one step of `O` is under `R·sin²θ`: a tube within a few
steps of its ring, `R − r` around `R·2⁻⁵⁰`, with the picker's distance within
a step of `R − r` and a real turn. That is a guess, not a measurement.
