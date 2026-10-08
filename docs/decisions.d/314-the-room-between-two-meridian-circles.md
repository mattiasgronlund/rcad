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
picker is turned a little off the true surface between the circles. A surface
through the centre turned by `θ` leaves each circle `R·cos θ − r` clear. That
loses `R·(1 − cos θ)`, at most `R·sin²θ`. With the old reading, the factor of
two covered it: a named pair had the distance under half of `R − r`, and the
turn is about `10⁻³²·R`. A reading this tight no longer leaves that room, so
the turn is now bounded on its own. `across` is the true product plus a
rounding `e`, so `sin θ ≤ ‖e‖/‖across‖`, with `e` taken from
`Exact::cross_of`. If that product cannot be worked out exactly, the pair is
refused.

`a_leaning_plane_through_the_axis_still_names_both_circles` guards the bound's
direction: a plane with normal `(1, −1, 1)` through an axis along `(1, 1, 0)`,
where `w × n` does round. With the exact product crossed the wrong way round,
that test fails along with four others in these two files, because `e` becomes
twice the picker itself.

### 314.5 Why R − r is enclosed now

The margin is now `R − r` less **two** bounds, the turn and the reading. Their
sum need not be an `f64`, so 314.2's argument does not carry to it: a sum can
fall between `R − r` and its rounding. `R − r` is therefore
`Interval::exactly(R) − Interval::exactly(r)`, whose lower end is at most the
true difference.

**No test fails if that enclosure is taken out.** In the family of 314.2 the
turn is exactly zero, so there is one loss term again. A torus where the
enclosure decides the answer needs a nonzero turn of about `10⁻³²·R`, plus a
distance tuned to within that of `R − r`. Nobody built one. That is a
statement of what was not searched, not a claim that no such torus exists.

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
  of `R − r`.
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
