## 131. The three-plane meet gets an exact tier, and it buys back two of §130's four, measured 2026-09-18

*§130.5 named three answers this kernel started giving differently after §121
rounded a plane's normal correctly, and §130.6 left them open because "what the
cut set in those two tests should be" looked like a decision. It was not one.
The corner those tests refuse is `(0.5, 1.5, 2)`, which is three whole numbers,
and §16.10's test says an answer that exists and that the arithmetic could not
reach is a **gap** — so the fix is the arithmetic and not the test. Cramer's
rule now has a third level. Two of the four failures are gone; the other two
were never this cause, and §131.4 says what they are instead.*

### 131.1 What was built

`meet_of_three_planes` was one level and is now two. `meet_of_three_planes_at_this_scale`
is exactly the interval arithmetic that was there before and hands back the box
rather than a `Point`; `the_exact_point_the_box_holds` is the new level, which
asks Cramer's rule again through `crate::expansion`, where nothing rounds; and
`pinned_down` is the one line that puts them together — the corner if level 3
named it, the box if it did not. Both entry points run it, the plain one and
`meet_after_rescaling_each_plane`, so a meet rescued by a power of two is
sharpened on the same terms as one that was never out of range.

**Level 3 checks rather than divides.** Exact arithmetic here is `+`, `-` and
`*` (§2.3), so there is no exact quotient to compute — but there is a candidate
and a multiplication. For a side of the box holding the `f64` values `q₁ … qₙ`,
the coordinate is `qᵢ` exactly when `qᵢ · volume` is the substituted determinant
exactly, and both of those are expansions. Two facts make that a proof and not a
guess: the true coordinate is **inside** the box, because level 2's arithmetic
is an enclosure, so if it is an `f64` at all it is one of the `qᵢ`; and `volume`
is **not zero**, because level 2 certified that before it divided by it, so
`q · volume` takes a different value for every `q` and at most one candidate can
match. The second of those is asserted rather than assumed — a broken internal
invariant panics (§2.1's half of the rule that is not about callers).

The two by two determinant both `orient3d` and this expand along a column into
moved to `expansion::difference_of_two_products`, which is where the only copy
of it belongs; `orient3d::minor` is now one line calling it.

A side of the box holding one `f64` needs no arithmetic at all: there is nowhere
else in the box for the corner to be. That is the same reading
`fuse::as_exact_as_it_is` and `de_boor::enclosed` already take of a zero-width
enclosure, and it is now taken at the meet itself, where every caller gets it
rather than the two that remembered to ask.

### 131.2 The ordering, and why it is forced by something other than cost

§122.2 settled that level 2 answers first and level 3 runs only where it did not
commit, and settled it with a witness on which the other ordering is **wrong**
rather than slow. This site inherits that ordering and owes no witness of its
own, because it cannot have one: **level 3 here has nothing to ask until level 2
has answered.** It has no division, so it cannot produce a quotient — only check
one. Level 2's box is where the candidates come from and is what makes the check
sound. An ordering that ran level 3 first would not be a worse answer; it would
not be an algorithm.

"Did not commit" is read here as **a side of the box with width in it**. A
zero-width side is a commitment and is taken as one; a side four ulps across is
level 2 saying it does not know which `f64` this is, which is the same shape as
an enclosure that straddles zero.

### 131.3 The corner, and the arithmetic that lost it

The witness is `split_a_face` cutting the top of a cube of side two with
`y = 1.5` and `x + y = 2`, in the order it hands them over —
`[0, 0, 1, −2]`, `[0, 1, 0, −1.5]`, `[c, c, 0, −2c]` for `c` the stored `1/√2`.
Level 2 comes back with the sides holding **6, 4 and 1** `f64`
(`level_two_leaves_that_corner_in_a_box_six_f64_wide`), which is the box the
refusal §130.5 records printed: `[0.4999999999999999, 0.5000000000000003]`,
`[1.4999999999999996, 1.5000000000000002]`, `[2, 2]`. Level 3 names
`(0.5, 1.5, 2)` out of it.

§130.5's mechanism holds and this lane did not re-derive it: the numerator is
`1.5c`, an `f64` only when `c`'s significand is a multiple of four, and the
correctly rounded `c` is `0x3fe6a09e667f3bcd` where the old one was `…cc`. What
is worth adding is that **this corner is reached by more than one trio and level
2 does not lose all of them.** The same cube's same three cut lines meet at the
same corner through `x = 0.5` instead of `y = 1.5`, and
`[1, 0, 0, −0.5]`, `[c, c, 0, −2c]`, `[0, 0, 1, −2]` comes out of level 2 with
one `f64` on every side — measured, by the same width probe. So the refusal was
never "this corner is not reachable in `f64`"; it was one trio of three losing
it, which is exactly what a gap looks like from the inside.

`HOW_MANY_F64_ARE_WORTH_TRYING` is 16, and it is a **budget and not a bound**: a
wider side can still hold an exact corner and this hands back the box for it.
Six is what a real near miss measures, so the budget has room over the only one
we have. `a_side_one_f64_past_the_budget_is_not_walked` pins both ends of it
(§15.6).

### 131.4 What came back, what did not, and the two causes that were never one

§130.6 read all four failures as one cause. Two of them were.

`mise exec -- just t the_pieces_of_a_face_add_up_to_the_area_it_started_with
presenting_the_same_cuts_the_other_way_round_gives_the_very_same_pieces
how_often_a_certified_fuse_of_a_skew_pair_refuses
every_kicad_file_that_imports_round_trips_to_the_same_solid --no-fail-fast`, on
`8a63fa2a`: **4 tests run, 0 passed, 4 failed**. The same command on this
branch: **4 tests run, 2 passed, 2 failed**. The two `arrange` oracles pass as
written; neither was touched.

**The skew band moved towards its floor and did not reach it.** `mise exec --
just t how_often_a_certified_fuse_of_a_skew_pair_refuses --no-capture`:

```
§22 headline: skew pairs, every rule: 429 fuses, 54 refused (12.6%), 1 not a solid (0.2%)
    9 (2.10% of all fuses) named "a point inside a piece of a face"
    27 (6.29% of all fuses) named "order along a crossing"
    18 (4.20% of all fuses) named "side of plane"
```

§130.5 measured 63 refused (14.7%) and 0 not a solid on the base. So nine
refusals are gone and one not-a-solid is back, against the three
`A_SKEW_PAIR_IS_NOT_A_SOLID` was derived from. **The band still fails, and it is
not re-measured here** — a band is re-measured when you can say why the new
number is right, and 1 of 429 is a number with no argument behind it.

**It is not the meet that is still losing them.** Level 3 was instrumented to
print every time it asked and declined, and over one run of that family it
declined **43 374 times** — on boxes one to four `f64` wide, every one of them a
corner of two solids leaning at unrelated angles, whose coordinates are algebraic
irrationals and are not `f64`. Those declines are correct. Three planes are the
only route `where_three_planes_meet` takes to a plane-plane-plane corner
(`three_surfaces.rs:450`), so there is no second path for this family's corners
to be losing exactness on. Whatever the other two not-a-solid answers were, they
are not a corner the meet could name and did not.

**`PinHeader_2x08_P2.54mm_Vertical.step` is a different cause, and the error
message says so.** `mise exec -- just t
every_kicad_file_that_imports_round_trips_to_the_same_solid --no-capture` on
this branch:

```
round trip: both sides refuse to fuse, but for different reasons: the model is
broken: a vertex of a face is # away from the face's plane vs could not settle
# decisions the answer depends on
```

A vertex off its own plane after a write and a read back is not an undecided
ordering; it is the plane moving under a vertex that did not. §130.6 already
names the suspect one line up from it: **`from_coefficients`'s offset is two
roundings and could be one.** `offset_of` computes `-(d / widest) * unit_there`,
and on a plane that is already unit the round trip evaluates
`(offset / nᵢ) · nᵢ`, which two roundings need not bring back. That is a
hypothesis with the error message behind it and **not a measurement** — nothing
in this lane ran it — and `crates/rcad-geom/src/plane.rs` was outside this lane's
write set, so it is named rather than fixed.

**The whole suite, and the exit code read out of a file.** `mise exec -- just t
--no-fail-fast`: **2 005 tests run, 2 003 passed, 2 failed**, and the two are
the two above. `mise exec -- just gate` redirected to a file with its exit code
echoed after it: `gate: FAIL  first failure: test (exit 100)`, and the file's
last line is `EXIT=100`. That is read and not forecast. Nothing else in the gate
moved: every `check-*` arm passed, and `lint` and `check-wildcards` passed after
one `cargo fmt --all` and one `needless_range_loop` rewrite that a first gate run
caught.

### 131.5 What is left

- The skew vacuity band, at 1 of 429 against a floor of 2. Not this lane's to
  re-measure, and §131.4 says why.
- `PinHeader_2x08_P2.54mm_Vertical.step`, and the one-rounding offset §131.4
  points at. A lane, with §130.6's bullet, not a gate.
- `Plane::meet_of_three`'s doc comment (`plane.rs:497`) now says "The answer is a
  constructed point", which this makes false in the same way §130.5 item 3's two
  sentences are false. Outside this lane's write set and not edited.
- A side of the box wider than sixteen `f64` is still handed back whole even
  where an exact corner is hiding in it. A **gap** by §16.10 and deferred, in the
  shape §122.4 defers its own: the answer can exist and the budget declines to
  look for it. The family of fixes is not one member wide here — a tighter level 2
  would shrink the box, and Newton's step on the two expansions would name the
  quotient without walking anything — so it needs an argument before it needs
  code, and no witness of it has been produced.

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-130-exact-meet 8a63fa2a`; `mise exec -- just decision 122`, `122.1`,
`122.2`, `122.4`, `130`, `130.5`, `130.6`; `mise exec -- just src Point
crates/rcad-predicates`, `from_coefficients crates/rcad-geom`, `offset_of
crates/rcad-geom`, `as_exact_as_it_is`; the four-test command of §131.4 on
`8a63fa2a` and again here; `mise exec -- just t -p rcad-predicates
--no-fail-fast` (175 tests, all passing); two throwaway `eprintln!` probes in
`planes.rs`, one to find the witness trio and one to count the declines, both
removed before the commit; `mise exec -- cargo fmt --all`; `mise exec -- just t
--no-fail-fast`; `mise exec -- just gate`, redirected to a file with `EXIT=$?`
echoed after it and read back.

**The base's four is taken on the brief's word and was not re-run here as a
whole-suite count.** What this lane measured on `8a63fa2a` is the four named
tests, by name: 0 passed, 4 failed.

