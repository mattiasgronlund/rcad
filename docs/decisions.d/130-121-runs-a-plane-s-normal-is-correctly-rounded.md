## 130. §121 runs: a plane's normal is correctly rounded, and the ulp it gained cost a corner, measured 2026-09-18

*§121 booked the plane half of §120 and said what the fix owes. This is the fix
and the bill. The arithmetic change is four lines; what it moved is seven tests,
four of which needed a **new witness** rather than a new number, and three of
which are not pins at all but answers this kernel now gives differently. Those
three are named in §130.5 and are **not** closed here.*

### 130.1 What changed

`Plane::from_point_and_normal` and `Plane::from_coefficients` were
`reduced_by_largest_component` and then `normalized_as` — two divisions and a
rounded square root between them, §121.2's 2 759-of-16 680. Both now call
`Vector::scaled_to_length_one_from_anywhere`, the same function §120 gave
`Axis::new`: an exact power-of-two rescale, a length carried to about a hundred
bits, and one division per component.

`reduced_by_largest_component` had no other caller and is gone. What it was also
doing for `from_coefficients` was giving the offset a divisor that two
proportional coefficient sets agree on, and `Plane::offset_of` does that instead:

```rust
-(d / widest) * unit_there
```

`widest` is the largest-magnitude of `a`, `b`, `c` and `unit_there` is the unit
normal's component beside it, so `unit_there / widest` is `1 / |(a, b, c)|` and
the length is never formed. Dividing `d` by the coefficient **first** is the
whole of the §22.6 property: `d/m` and `s·d/s·m` are the same real number and so
the same `f64`, and the unit normal is already the same for both. Multiplying `d`
by `unit_there` first would not be — that product moves with `s`.

The offset is two roundings where the old one was a division by a length that had
already rounded twice. It is not correctly rounded, and it could be: the
hundred-bit length is a private local in `point.rs` and exposing it is a change to
a file this lane did not own. Named here rather than done.

**The bare `normalized_as` in `Plane::basis` and the five sites beside it
(§121.6's second bullet) was not touched** — another lane holds it. `basis()`
reads the stored normal, so what it computes moves wherever the stored normal
moved; the arithmetic at that line is untouched.

### 130.2 The test that fails without it

`the_same_wall_built_from_a_longer_edge_is_the_same_plane`, in
`crates/rcad-topo/tests/fuses_that_must_agree.rs`. §120.5 named it as §120's tell
and §121.5 corrected that: it could not have moved for §120 and it does move for
this. It now asserts the stored coefficient is `FRAC_1_SQRT_2` itself, and
separately that the ulp below it — `0.707_106_781_186_547_5`, what a plane stored
until today — is one bit down, so a slide back is something somebody has to look
at. Run against the old constructor it reads `0.7071067811865475` and fails.

### 130.3 The four witnesses that dissolved, and the two counts that moved

§120.7's warning, arriving. Four tests were built on a number the change made
uninteresting, and each needed a new witness found by search:

| test | what it needs | old | new |
| --- | --- | --- | --- |
| `a_conic_the_intersector_names_is_refused_here_because_the_two_ask_it_differently` | `f64` dot cancels, exact dot does not | pipe `(2,0,3)`, plane `(−12,−12,8)` | plane `(−12,−7,8)` |
| `a_cross_product_of_unit_triples_can_round_to_zero` | two stored normals whose `f64` cross is zero and whose true cross is not | `(1,2,6)` and its `next_up` | `(2,6,7)` and its `next_up` |
| `a_plane_a_rounded_dot_calls_flush_with_an_axis_is_not` | a normal the constructor hands **straight back**, square to an axis in `f64` only | one triple | another, found in 200 000 draws |
| `a_rounded_apex_and_a_rounded_lean_together_call_a_closed_ellipse_two_rulings` | the same, plus an apex an ulp off the plane | four numbers | four others, found in 400 000 draws |

The first one has now dissolved **twice** — §120 killed `(1,0,3)`/`(−12,−12,4)`
and this killed its replacement — and the gap it witnesses has not moved either
time. The last two dissolved for a reason worth naming: a hand-written witness in
those files has to be a **fixed point** of the constructor, or the test cannot say
"the numbers below are the ones the doors read", and the old stored normals are
not fixed points of a correctly rounded normalize.

Two counts moved, both towards the fix:

| | before | after |
| --- | --- | --- |
| normals that move when a plane is rebuilt from its own coefficients, of 100 000 | 24 733 | **1 053** |
| the three normals written square to `(1, 3, 5)`, read in `f64` after storing | `0.0`, `0.0`, `1.1e−16` | `0.0`, `4.4e−16`, `−1.1e−16` |

The first is `Plane::moved_to`'s published number and
`rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal` counts it; a
twentieth of the footgun is still a footgun, and `moved_to` is still how to scale
a plane. The second is
`a_normal_written_square_to_a_direction_stops_being_square_once_a_plane_stores_it`,
whose absolute `f64::EPSILON` bound held by luck and is now
`SIGNED_DISTANCE_ROUNDOFF × largest coordinate` — the bound this file already
publishes for exactly that dot product. **The §29.8 limit it demonstrates is
untouched**: all three are still not square once stored, at the exact level,
which is the assertion that matters.

### 130.4 The corpus did not move by name, and the round trip did

`mise exec -- just t how_often_a_real_file_imports --no-capture`:

```
=== nist: 1 / 44 imported = 2.3%
=== kicad: 114 / 323 imported = 35.3%
```

`assert_pinned_kicad_imports` compares sets and fails both ways (§44.10), and it
passed — the same 114 files, not merely as many.
`every_file_that_stops_at_the_tangency_wall_names_the_edge_it_was_following`
reads **24**, unmoved.

**`every_kicad_file_that_imports_round_trips_to_the_same_solid` moved by one, and
downwards.** 44 of 114 where it was 45, and the by-name pin says which:
`PinHeader_2x08_P2.54mm_Vertical.step` stopped. Nothing was gained. Its line in
the breakdown reads *"both sides refuse to fuse, but for different reasons: the
model is broken: a vertex of a face is # away from the face's plane vs could not
settle # decisions the answer depends on"* — the two sides used to refuse the
same way and now do not, so the round trip no longer counts. A by-name pin
catching one file in one direction is §44.10 working; **it is reported and not
re-pinned.**

### 130.5 What this bought, and the three answers it changed

§17.1's claim holds: `proportional_coefficients_describe_the_same_plane_bit_for_bit`
and `a_planes_coefficients_written_at_any_scale_are_the_same_plane` pass, and a
correctly rounded normalize keeps the same-ray property by construction
(§120.1) — the offset keeps it because `d` meets the coefficient before anything
else.

**Three things this kernel now answers differently, and none of them is a pin.**
They are the reason this section is `measured` and not `fixed`:

1. **A 45° cut through a lattice corner is no longer decided.**
   `the_pieces_of_a_face_add_up_to_the_area_it_started_with` and
   `presenting_the_same_cuts_the_other_way_round_gives_the_very_same_pieces` cut
   the top of a cube of side two with `x = 0.5`, `y = 1.5` and `x + y = 2` —
   three cut lines through one point, which is §22's whole subject. The corner
   `(0.5, 1.5, 2)` used to come back from `meet_of_three_planes` as a
   **zero-width** box and now comes back four ulps wide, so ordering it against
   the same corner named exactly is `Undecidable` and `split_a_face` refuses.

   The mechanism is arithmetic and was measured, not guessed. Cramer's numerator
   for that `y` is `2c − 0.5c` for `c` the stored `1/√2`; `1.5c` is exactly
   representable only when `c`'s mantissa is a multiple of four. The old `c` is
   `0x3fe6a09e667f3bcc`, mantissa `6369051672525772`, a multiple of four. The
   correctly rounded `c` is `0x3fe6a09e667f3bcd`, mantissa `6369051672525773`,
   which is not. **The old answer was luck in the last two bits and the new one
   cannot be arranged**: the nearest `f64` to `1/√2` is the nearest `f64` to
   `1/√2`. This is a third horn of §32's "canonical or square, not both" —
   canonical *and* closest costs exactness at rational corners of irrational
   walls.

2. **The skew family stopped producing not-a-solid answers at all.** `mise exec
   -- just t how_often_a_certified_fuse_of_a_skew_pair_refuses --no-capture`:

   ```
   §22 headline: skew pairs, every rule: 429 fuses, 63 refused (14.7%), 0 not a solid (0.0%)
       9 (2.10% of all fuses) named "a point inside a piece of a face"
       30 (6.99% of all fuses) named "order along a crossing"
       24 (5.59% of all fuses) named "side of plane"
   ```

   `A_SKEW_PAIR_IS_NOT_A_SOLID` is `0.47%–2.10%` and 0 is outside it, so the
   vacuity guard fires. The refusal ceiling `SKEW_REFUSALS_MUST_STAY_BELOW` of
   18% still holds at 14.7%, and `"order along a crossing"` is still the leading
   cause at 30 — the number §22 records and the one that says this family is
   still reaching the branch it was built for. What moved is the other two:
   `"side of plane"` is 24 where this file records 12, and
   `"a point inside a piece of a face"` is 9. **The before-column here is what
   the file's own doc comments record, not something this lane ran**, and it is
   worth re-measuring on the base tree before anything is concluded from it.

   That the not-a-solid count went to zero is consistent with item 1: four faces
   along one edge needs two edges that are the same edge *exactly*, and a
   configuration whose corners stopped landing exactly stops producing it.

3. **`validate.rs:2855` and `surface.rs:305` now say something false.** Both
   describe a plane as dividing by its largest component. Neither is in this
   lane's write set and neither was edited. `validate.rs`'s is the doc comment on
   `a_plane_through_a_point_does_not_put_that_point_exactly_on_itself`, which
   still passes; `surface.rs`'s is the sentence in `how_it_sorts_against` that
   earns §32.5's trap for each of the five surfaces.

### 130.6 What is left

- The two `arrange` oracle tests and the skew vacuity band are **failing on this
  branch**. They are not pins to update: item 1 above is a real answer the kernel
  now gives, and turning either oracle into "this refuses now" would gut it. What
  the cut set in those two tests should be, or whether a triple point with an
  irrational wall through it is now a named gap, is a decision and not a lane's
  to take alone.
- `PinHeader_2x08_P2.54mm_Vertical.step`'s round trip, same reason.
- The `from_coefficients` offset is two roundings and could be one (§130.1).
- The two false sentences in §130.5 item 3.
- §120.6's table owes the row §121.6's last bullet asks for: after this,
  `the_old_way`'s 2 759 of 16 680 stops describing anything live and becomes a
  measurement of the arithmetic `Plane` ran until 2026-09-18. It is recorded here
  rather than in §120.6, because §120 is not this lane's section to edit.

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-121-plane-normalize 381b7bef`; `mise exec -- just decision 121`,
`121.1`–`121.6`, `120.6`–`120.8`; `mise exec -- just src
scaled_to_length_one_from_anywhere`, `moved_to`; `mise exec -- cargo build
--workspace --all-targets`; `mise exec -- just test` (2002 tests, 11 failing
after the arithmetic change); `mise exec -- just t --no-fail-fast`; `mise exec --
just t <name> --no-capture` for each failure; three throwaway sweeps written into
the test modules of `along_a_crossing.rs`, `walk.rs` and `arrange.rs`, run, and
reverted with `git checkout --`; a scratchpad `rustc` probe for the two mantissas
in §130.5 item 1; `mise exec -- just t how_often_a_real_file_imports
--no-capture`; `mise exec -- just t
every_file_that_stops_at_the_tangency_wall_names_the_edge_it_was_following
--no-capture`; `mise exec -- just t
every_kicad_file_that_imports_round_trips_to_the_same_solid --no-capture`.

