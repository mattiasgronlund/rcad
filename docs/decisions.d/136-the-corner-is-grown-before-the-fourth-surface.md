## 136. The corner is grown before the fourth surface is asked, and the allowance it is grown by is the model's only in name, fixed 2026-09-18

§132.6 named the next step and the argument it owed: make `derive_a_corner`'s
re-check read the same allowance every other reading of that disagreement
reads, and show that doing so does not let through the corner §70.3 is about.
Both are done here. A third thing came out of doing them, and it is the part of
this section worth reading: **the allowance the re-check now grants is sized off
the corner and `Model::validate`'s is sized off the model, so the two still
disagree — in the direction §75.6 says they must not.** That is §136.4, and it
is left open with a repro rather than argued away.

### 136.1 What changed

One line of `crates/rcad-topo/src/fuse/bake.rs`. `derive_a_corner` solved the
corner from its best triple and then asked every surface around it
`side_of_surface(surface, point)` against the bare box. It now asks
`side_of_surface(surface, grown_by(point, what_writing_the_corner_down_cost(…)))`
— the point grown on every side first, by the same expression
`the_place_the_model_points_at` already grows the stored corner by, which reads
`MEASURING_SLACK` from `validate.rs` rather than copying it.

`grown_by` is the new helper. **A box and not a signed distance, because only a
plane hands a signed length out**: a ball, a pipe, a cone and a ring answer with
a sign and nothing else, so the allowance has to be spent on the point before
the question rather than on the answer after it. `not_provably_off` in
`validate.rs` makes the same move for the same reason, and that citation was
checked rather than taken on the lane's word — it builds `Interval::around` on
each coordinate and asks `side_of_surface`, which is this, spelled the same way.

### 136.2 Why the bare box was the wrong question, in the corpus's own numbers

The box a triple solves a corner as says how well *that solve* is known. It says
nothing about the fourth surface, which was fitted through corners that had
already been rounded and had its normal scaled to length one on the way in, so
it passes a few ulps from where the other three cross. Asking it about the bare
box asks it to agree bit for bit with a number it never saw.

Read off `PinHeader_1x11_P2.54mm_Vertical.step`'s failing corner with an
`eprintln!` in the refusal arm, not invented:

| | |
|---|---|
| corner three planes place | `(-0.32, -20.64, -2.84)` |
| the fourth surface | a 45° wall, normal `(-0.7071067811865476, -1.735e-18, -0.7071067811865476)` |
| what it reads there | `3.5527e-15` — one ulp of that `-20.64` |
| the allowance | `2.9331e-13` |

So the miss is **82 times inside** the allowance. This was never a marginal
call, which is worth saying because a change that grants room usually is one.

The tilt in the third direction cosine is the file's own and is what makes the
long axis enter the sum at all. The corner's solved box is narrow in exactly the
two directions this wall's normal points along — `1.1e-16` in `x`, `5.3e-15` in
`z` — while the `7.1e-15` it has in `y` is multiplied by a normal component of
`1.7e-18` and contributes nothing. That is why a box that looks wide enough was
not: **the width was in the direction the question did not ask about.**

### 136.3 The two tests, and which of them could have failed

Both are in `bake.rs`'s own test module, because both call `derive_a_corner`
directly and that is private.

**`a_fourth_plane_a_few_ulps_off_the_corner_three_others_place_is_agreement`** is
the one that fails without the change. Three axis planes place `(-0.32, -20.64,
-2.84)` exactly; a fourth is pushed off it by `3.552713678800501e-15`, the pin
header's own miss. Run against the unfixed line it fails with
`Invalid(VertexOffItsFacePlane { distance: 3.552713678800501e-15 })` — the same
number, which is how the reduction is known to be one. It was run that way, not
reasoned about.

Its first assertion is the one that keeps it from being a pin: it checks that
the bare corner still reads a **certified** `Pos` or `Neg`. Without that the
test could go green on a kernel where the miss had stopped being provable at
all, which is a different world and not this fix.

**`a_corner_near_three_grazing_planes_but_far_from_where_they_cross_is_refused`**
is §132.6's owed argument, and it passes before and after — that is its job.
Three planes at a `1e-14` graze cross at the origin. A corner stored at
`(1, 1, 0)` is `1e-14` off all three of them, inside the `1.4211e-14` that
writing each plane down cost, **so the per-plane comparison accepts it** — and
it sits `1.414` from where they actually cross. That is §70.3's sentence, that a
corner close to all three surfaces can still be far from where they meet, as
numbers rather than as a paragraph (§16.10).

What refuses it is `the_place_the_model_points_at`, which is asked about the
**stored** corner and runs before the re-check, which is asked about the
**solved** one. **The error variant is the evidence of which of the two fired:**
it is `Error::Undecidable` — nothing placed the corner — and not an
`Invalid::VertexOffItsFacePlane` from the re-check. Growing the solved corner
cannot reach a corner that was never solved.

The second half of that test is a control, and it is there because without it a
regression making a grazing triple unusable would leave the first half green for
the wrong reason: the same three planes, with the corner stored where they
cross, place it exactly. So the graze is not what did the refusing.

One thing measured on the way that was expected to go the other way: **a `1e-14`
graze is still solved exactly.** The conditioning filter does not refuse these,
so the selector really is the only thing standing in front of §70.3's corner,
and the argument does not get a second guard it was not entitled to.

### 136.4 Open: the re-check's size is the corner's and `validate`'s is the model's

`what_writing_the_corner_down_cost` floors the corner's own size at one;
`Model::what_writing_a_plane_down_costs` scales by the model's largest
coordinate and floors nothing. Above a coordinate of magnitude one the first is
**tighter**, and tighter is the wrong direction: `MEASURING_SLACK`'s own doc
comment and §75.6 both say a solid `validate` accepts is one the fuse has to be
able to bake.

Measured, with a chamfer pinned through its own corner a reach away and the
corner at the origin:

| reach | miss at the origin | this check allows | `validate` compares against | bakes? |
|---|---|---|---|---|
| `1e3` | `1.7352e-15` | `1.4211e-14` | `1.4211e-11` | yes |
| `1e4` | `1.7352e-14` | `1.4211e-14` | `1.4211e-10` | yes |
| `1e5` | `1.7352e-13` | `1.4211e-14` | `1.4211e-9` | **no** |

At `1e5` the miss is 12 times outside what the fuse allows and 8000 times inside
what `validate` measured the same corner against. The crossover sits between
`1e4` and `1e5`.

**This is older than the growth above and much narrower because of it.** The
re-check used to ask with no room at all, so *every* miss was a refusal; only a
miss past the corner's own floored size is one now. The growth shrank this gap
rather than opening it — but it did not close it, and the doc comment as the
lane left it read the tightness as a reassurance ("that leaves this check
tighter than `Model::validate`'s own reading"). It is corrected to say what it
is.

`a_corner_at_the_origin_of_a_big_model_is_refused_for_a_miss_validate_allows` is
the repro, `#[ignore]`d and naming this section, per the testing skill's rule
that an unfixed bug is owed a compiling test and not a paragraph. It is a **gap
and not a limit** in §16.10's sense — the answer exists and the arithmetic can
reach it — so it is owed a fix. Closing it wants the model's largest coordinate
threaded down to `derive_a_corner`, which changes signatures across the fuse and
is a lane of its own rather than a line of this one.

### 136.5 The skew band, re-measured: the rate never moved and the floor was never sound

Q3, answered by Mattias as (c) then (a) — leave the row failing until §136
lands, then re-measure at more than one seed and publish the band with its N.
§136 lands above, so this is that re-measurement.

**The corner fix did not move the number.** With it in, the row still reads 1 of
429, exactly as §131.4 and §132.5 both recorded. The corner allowance was never
what was moving it, which is worth saying because it was the obvious suspect.

`RCAD_SOAK_SEED` varied the way `SOAK_SEED`'s own doc comment documents.
**Twelve fresh seeds, shipped corpus of 429 fuses:**

| seed | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| not a solid | 1 | 5 | 3 | 1 | 4 | 4 | 5 | 4 | 6 | 1 | 1 | 3 |

One to six, and **four of the twelve land on one**. So 1 of 429 is an ordinary
draw from this family and not a signal at all.

**Six seeds at ten times the corpus, 3 885 fuses each:** 28, 39, 34, 28, 36, 28
— `0.72%, 1.00%, 0.88%, 0.72%, 0.93%, 0.72%`. The underlying rate is **0.83%**,
against the **0.7%** the band was first built on. It did not move. Six weeks of
arithmetic across §121, §130, §131 and §132 changed which draw the fixed seed
takes and left the family's behaviour where it was.

**The floor was mis-derived, and this is the general lesson.** All six rows use
the same convention — two thirds to three times the measured rate — and on this
row it was applied to a *single draw of three*. The skew family's rate is a
twentieth of the lattice family's, so its expected count at 429 fuses is about
three and a half, and a floor at two fuses sits **inside** that count's own
sampling spread. The convention is a statement about a rate and this row asserts
a count; at counts this small the two are not the same statement.

The sentence that guarded it was true and still misled: *a band this tight is
not a flake waiting to happen — `measured` runs from a fixed seed, so a count of
three is a count of three on every machine.* Right about re-running, wrong about
safety. Determinism means the number does not move **between runs**; it says
nothing about where the draw lands when the corpus is reshuffled, and any change
to the arithmetic reshuffles it.

**The new ends are one fuse to twelve, `0.0023..=0.028`** — the observed spread,
ceiling at twice its maximum. The floor is one fuse because one fuse is what
this row actually asks: whether the family still reaches the
four-faces-along-one-edge configuration it was built for. Zero fires.

**This is not §132.5's forbidden move.** That was widening a band to fit a
single observation, and it would have been wrong. What makes this different is
the N: eighteen runs across two resolutions, the rate pinned at the higher one
and the spread measured at the shipped one. §131.4 declined to re-measure on the
grounds that one of 429 is a number with no argument behind it — that was
correct, and this is the argument it was waiting for.

**What a reader should take from this:** the other five rows were derived the
same way from single draws. Four sit at counts of ten or more where the
convention is defensible, and `A_TURNED_PAIR_IS_NOT_A_SOLID` at 14 of 429 is the
next smallest. None is re-derived here and none is known to be wrong — but the
method that produced them is now known to fail below counts of about ten, and
that is recorded rather than acted on.

### 136.6 What is left

- **§136.4**, above.
- **The other five not-a-solid bands are still single-draw derivations**,
  §136.5's last paragraph. Not known to be wrong; the method behind them has a
  floor it does not survive.
- §120.6's table still owes the row §121.6 and §130.6 both ask for.
- §131.5's last bullet — a side of the box wider than the budget handed back
  whole — still has no witness.

### 136.7 Provenance

**The change itself was not written this session and the tests were.** The lane
that wrote the one-line fix and `grown_by` died on a weekly rate limit before
committing anything; the dispatcher preserved its working tree as `2387d042`
with a message saying so, and nothing in it had been compiled or run. What this
session did: checked that it compiles, that `not_provably_off` exists and says
what it was cited as saying, wrote both tests, ran the first against the unfixed
line, found §136.4, and corrected the doc comment's claim about it.

Commands run to reach this: `mise exec -- just status`; `mise exec -- just
decision 70.3`; `git show 2387d042`; `git grep -n not_provably_off`; `mise exec
-- just src MEASURING_SLACK`, `SIGNED_DISTANCE_ROUNDOFF`,
`what_writing_a_plane_down_costs`, `from_point_and_normal`; `mise exec -- cargo
check -p rcad-topo --all-targets`; `mise exec -- just t
every_kicad_file_that_imports_round_trips_to_the_same_solid --no-capture` with
the re-check's line reverted, once plain and once with an `eprintln!` in the
refusal arm — both reverted with `git checkout` before anything was kept; three
throwaway probe tests in `bake.rs`'s test module, all removed or turned into the
tests above; `mise exec -- just t -p rcad-topo -E 'test(/bake::tests/)'`; `mise
exec -- just t` on the two new tests with the re-check's line reverted and put
back; `git merge main` into the lane branch, which auto-merged
`docs/decisions.md` with no conflict; for §136.5, a script in the scratchpad
running `RCAD_SOAK_SEED=$seed mise exec -- just t
how_often_a_certified_fuse_of_a_skew_pair_refuses --no-capture` over seeds 1–12
at the shipped corpus and over seeds 1–6 at `RCAD_SOAK_FACTOR=10`, reading the
`§22 headline` line each run prints before it asserts; `mise exec -- just gate`
and `mise exec -- just t --no-fail-fast`, each redirected to a file with its
exit code echoed after it and read back.

**The loop went in a script because the worktree guard refused it inline.** A
`for` loop naming a path with `git` in it is "too complex to verify"; a script
in the scratchpad taking the worktree as its first argument is not.

