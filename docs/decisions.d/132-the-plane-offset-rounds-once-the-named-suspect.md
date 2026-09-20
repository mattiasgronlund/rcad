## 132. The plane offset rounds once, the named suspect was innocent, and the KiCad refusal is a check that grants no room, measured 2026-09-18

*§130.6 and §131.4 named one suspect for the KiCad round trip —
`Plane::from_coefficients`'s offset being two roundings — and §131.4 was
explicit that nothing had run it. This ran it. **It is innocent: the STEP
reader never calls `from_coefficients` at all.** What the offset rounding does
cost is somewhere else entirely, and it is large: cutting
`from_point_and_normal` from five roundings to one takes the skew family from
54 refusals to 42 and puts `"side of plane"` back on the number that file's own
doc comments record. Both remaining tests still fail, for two causes this names
and neither of which is in this lane's write set.*

### 132.1 The suspect, and the two commands that clear it

`grep -rn "from_coefficients\|from_point_and_normal\|through_three_points"
--include=*.rs crates/rcad-step/src` returns seven lines. Every one of them is
`from_point_and_normal`. `grep -rn "from_coefficients" --include=*.rs crates/`
returns twenty-five more, all of them doc comments, tests and
`fuse/arrange.rs`. **There is no path from a STEP file to
`Plane::from_coefficients`**, so its offset cannot be what moves a vertex off
its plane on a write and a read back. A STEP `PLANE` is an `AXIS2_PLACEMENT_3D`
— a point and a direction — and both ends of the round trip go through
`from_point_and_normal`.

That is the first half. The second is a measurement of what the round trip
actually moves, from a throwaway probe under `crates/rcad-step/tests` that
imports `PinHeader_2x08_P2.54mm_Vertical.step`, writes it, reads it back and
compares the two models' planes coefficient by coefficient:

```
planes: 340 here, 340 there
moved normals: 0, moved offsets: 147, worst offset ulps: 2
```

**No normal moves at all.** §121 bought that: a stored unit normal handed back
to `scaled_to_length_one_from_anywhere` comes back bit for bit. 147 of 340
offsets move, by at most two ulps, and every one of them is a 45° chamfer whose
normal is `±1/√2`.

### 132.2 What actually refuses, and which side of the round trip it is on

§131.4 read the error message as the round-tripped model being broken. **It is
the other way round on that file.** The same probe runs the test's own
`cut_by_a_centered_box` on both sides and prints the two errors whole:

```
BEFORE: the model is broken: a vertex of a face is 0.000000000000004440892098500626 away from the face's plane
AFTER:  could not settle 1 decisions the answer depends on
```

`BEFORE` is the model straight out of the file. The round trip is the side that
gives the ordinary `Undecidable`. (On `PinHeader_1x11_P2.54mm_Vertical.step`,
which this lane's change adds to the failure, the two are the other way about —
same shape, opposite sides.)

The refusal is `derive_a_corner`'s (`crates/rcad-topo/src/fuse/bake.rs:629`).
A throwaway `eprintln!` there prints the corner, its box and every surface
round it:

```
corner Point([-0.32, -17.46, -2.8400000000000056]) width 7.105427357601002e-15
 plane [0.0, 1.0, 0.0, 17.46]                                                    distance 0.0
 plane [-1.0, 0.0, 0.0, -0.32]                                                   distance 0.0
 plane [1.7351675083568896e-18, 0.7071067811865476, -0.7071067811865476, 10.337901140947324]   distance 1.7763568394002505e-15
 plane [-0.7071067811865476, -1.7351675083568896e-18, -0.7071067811865476, -2.23445742854949]  distance 4.440892098500626e-15
```

Four planes meet there: two axis-aligned walls and two 45° chamfers carrying a
`1.7e-18` tilt the file wrote. Three of them are solved for the corner; the
fourth reads `4.44e-15` — ten ulps of `2.84` — off it, and
`side_of_surface` certifies that sign, so the corner is refused.

**`Model::validate` allows that same disagreement and the fuse does not.**
`what_writing_a_plane_down_costs` grants `MEASURING_SLACK ×
Plane::SIGNED_DISTANCE_ROUNDOFF × largest_coordinate`, which on a model whose
largest coordinate is `19.05` is `2.7e-13` — sixty times the miss. The model
imports and passes the full sweep. `the_place_the_model_points_at`'s own doc
comment states the rule this breaks: *"a solid `Model::validate` accepts is one
the fuse has to be able to bake"* (§75.6). It grants the allowance when it
**picks** among places, and the loop straight after it, which re-checks the
picked point against every surface, grants none.

### 132.3 Two fixes that are not it, each with the run that says so

**Raising `HOW_MANY_F64_ARE_WORTH_TRYING` does not close it.** The box above is
`7.105427357601002e-15` wide on its `z` side, which is exactly sixteen ulps of
`2.84` — seventeen `f64`, one past §131.3's budget of sixteen. That looks like
§131.5's deferred gap exactly, so the constant was set to `64` and the probe re-
run. The refusal is unchanged, distance and all. The reason is that the true
corner is not in the box: level 2's enclosure is centred on
`-2.8400000000000056` and half of it is eight ulps, while `-2.84` is twelve and
a half ulps away. The three planes chosen genuinely do not meet at `-2.84`.

**Making the offset round once does not close it either, and it halves the
miss.** With `from_point_and_normal` on the hundred-bit dot product the same
probe prints a different corner and a smaller number:

```
corner Point([0.32, -18.1, -2.8399999999999967]) width 7.105427357601002e-15
 ... distance -2.220446049250313e-15
```

`-2.84` is now inside the box, seven and a half ulps from the middle. Both
changes together — one-rounding offset *and* a budget of 64 — were run, and the
refusal still stands: the exact meet of those three planes is not an `f64`
triple, because the `1.7e-18` tilt makes it a quotient no `f64` names. That is
a **limit** of those three planes and not a gap. The gap is the fourth plane
being asked a question with no room in it.

### 132.4 What one rounding is worth, measured by taking the other half away

The change is two spellings, both in this lane's write set.
`Vector::dot_rounded_once` keeps each product exact and adds the three to a
hundred bits, so `from_point_and_normal`'s offset rounds once where
`Vector::dot` rounded five times. `Plane::offset_of` forms `d · unitᵢ` exactly
and divides that by the widest coefficient, rounding once where dividing first
and multiplying after rounded twice.

Proportionality survives, and for a better reason than before. The old doc
comment said dividing `d` by the coefficient first is *what makes two
proportional coefficient sets agree*. Now **nothing rounds before the
division**: `two_product` is exact, so scaling the plane by `s` scales
numerator and denominator by exactly `s` and the quotient is the same real
number. `proportional_coefficients_describe_the_same_plane_bit_for_bit` and
`a_planes_coefficients_written_at_any_scale_are_the_same_plane` both pass
unmodified.

The two halves were measured apart, by putting `Vector::dot` back and running
the two failing tests again. **`offset_of` alone changes nothing either test
can see** — `429 fuses, 54 refused (12.6%), 1 not a solid`, and the same one
pinned KiCad file stopped, which are §131.4's numbers to the fuse. **The whole
of the movement is `from_point_and_normal`:**

| | refused | "side of plane" | "a point inside a piece of a face" | not a solid |
|---|---|---|---|---|
| `381b7bef`, before §121 (file's own doc comments) | — | 12 | — | 3 |
| `8a63fa2a`, §121 | 63 (14.7%) | 24 | 9 | 0 |
| `5b02cca5`, §131 | 54 (12.6%) | 18 | 9 | 1 |
| here | **42 (9.8%)** | **12** | **3** | 1 |

`"side of plane"` lands back on the twelve that file records for the arithmetic
before §121, and `"order along a crossing"` — the count §22 is about — sits at
27 through all three of the last rows. **The offset's four extra roundings were
costing a tenth of this family's `"side of plane"` decisions**, and they were
costing them before §121 as well; §121 only made them visible by taking the
normal's rounding away from underneath.

`a_dot_product_rounded_once_keeps_a_term_three_products_and_two_adds_lose` is
the test §15.6 asks for on the arithmetic itself: `(1,1,1) · (2⁶⁰, 1, −2⁶⁰)` is
one, `Vector::dot` returns zero, and one rounding returns one.
`a_planes_offset_from_its_coefficients_rounds_once_and_not_twice` pins the
other spelling on a plane out of a 200 000-sample sweep in which the two
spellings differed 52 355 times — a quarter of ordinary coefficient sets, not a
contrived one.

### 132.5 The skew band is still not re-measurable, and this says what would make it so

`A_SKEW_PAIR_IS_NOT_A_SOLID` is `0.47%–2.10%`, which over 429 fuses is two to
nine. It reads **1** here, as it did on §131's tree, and the guard still fires.

**It is not widened.** §131.4 declined to re-measure on the grounds that one of
429 is a number with no argument behind it, and that is still true — a second
run of the same deterministic family is the same one, not a second opinion. What
this lane adds is that the direction of travel is now explained rather than
merely observed: three of the four rows above move together, and the one that
moves them is the plane being nearer the plane the file names. Fewer refusals
and fewer non-manifold results are the same fact about the same arithmetic.

That is an argument for *why the number fell*, and it is still not an argument
for *what the floor should be*, which is what a band needs. The band's job is
vacuity: it says this family still reaches the four-faces-along-one-edge
configuration it was built to reach. At one it still reaches it. Re-deriving the
floor wants the family generated at more than one seed — the width of the
distribution, not one draw from it — and **this lane did not do that and the
band is left failing.** Widening it to fit a single observation is the one move
that would waste §121, §130, §131 and this.

### 132.6 What is left

- **The KiCad round trip fails on two files now, where it failed on one.**
  `PinHeader_2x08_P2.54mm_Vertical.step` and
  `PinHeader_1x11_P2.54mm_Vertical.step`. §132.2 names the cause and it is a
  check in `crates/rcad-topo/src/fuse/bake.rs`, outside this lane's write set:
  `derive_a_corner` re-checks the corner it picked against every surface with a
  bare certified sign, while every other reading of that same disagreement —
  `check_flat_face`, `the_place_the_model_points_at` — grants `MEASURING_SLACK ×
  SIGNED_DISTANCE_ROUNDOFF × size` and reads it from the one constant so that
  the two cannot drift. The next step is to make the re-check read the same
  allowance, and it owes the argument that doing so does not let through the
  corner §70.3 is about; the two are told apart by *how far*, which is exactly
  what the allowance measures and the bare sign does not.
- **The skew vacuity band**, §132.5. The next step is to run the family at more
  than one seed.
- §131.5's own last bullet — a side of the box wider than the budget handed back
  whole — **still has no witness**, and §132.3 is the negative result: the one
  case that looked like it is not one.
- §120.6's table still owes the row §121.6 and §130.6 both ask for.

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-132-plane-offset 5b02cca5`; `mise exec -- just decision 121`, `130`,
`131`, `130.5`, `130.6`, `131.1`, `131.3`, `131.4`, `131.5`; `mise exec -- just
src from_coefficients crates/rcad-geom`, `offset_of crates/rcad-geom`,
`scaled_to_length_one_from_anywhere`, `MEASURING_SLACK`,
`what_writing_a_plane_down_costs`, `the_place_the_model_points_at`,
`uncertainty`, `as_exact_as_it_is`; the two `grep` commands of §132.1; a
throwaway probe test under `crates/rcad-step/tests`, an `eprintln!` in
`bake.rs`, a witness sweep under `crates/rcad-geom/tests` and one temporary
edit of `HOW_MANY_F64_ARE_WORTH_TRYING`, all four removed or reverted before
the commit; `mise exec -- just t
every_kicad_file_that_imports_round_trips_to_the_same_solid --no-capture`;
`mise exec -- just t how_often_a_certified_fuse_of_a_skew_pair_refuses
every_kicad_file_that_imports_round_trips_to_the_same_solid --no-fail-fast
--no-capture`, once with both halves of the change and once with
`from_point_and_normal` put back; `mise exec -- cargo build --workspace
--all-targets`; `mise exec -- just t --no-fail-fast`; `mise exec -- just gate`,
redirected to a file with `EXIT=$?` echoed after it and read back.

**The gate's exit code is read and not forecast.** `mise exec -- just gate`
into a file: the last line is `EXIT=100`, and the line above it is `gate: FAIL
first failure: test (exit 100)`. Every arm before `test` passed, `lint` and
`check-wildcards` among them. `mise exec -- just t --no-fail-fast`: **2 007
tests run, 2 005 passed, 2 failed**, and the two are the two named above.

**`## 132.` is taken here on instruction.** The pre-commit check
`git log --all --oneline -S'## 132. ' -- docs/decisions.md` returns `e4331d8d`,
a section of the same number on an unmerged branch. The session dispatching this
lane holds that branch, has agreed to renumber its own three, and told this lane
to take 132 anyway.

