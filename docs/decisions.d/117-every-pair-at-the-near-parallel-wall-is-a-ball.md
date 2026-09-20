## 117. Every pair at the near-parallel wall is a ball on a pipe or a pair of crossed pipes, measured 2026-09-17

§113.5 said the example's "transverse" was the default arm of a `match`
reporting itself as a verdict, and that a cylinder/sphere case in
`evaluate_surfaces` is what closes §109.1's hole. This is that case, and the
census with a test behind it.

**Every count in this section is `44c59a7e`'s, and the merged tree's are not the
same — §117.6.** The run below is kept exactly as it printed rather than
restated, because it is the measurement §117.1 to §117.5 reason about; the
conclusions all survived, and only the counts moved.

Command, from a worktree of `44c59a7e`:

```text
RCAD_STEP_CORPUS=<corpus> mise exec -- cargo run -q -p rcad-step --release \
  --example near_parallel_which_pair
```

```text
=== files whose only probe reason is the near-parallel crossing ===
  found 24
  and 0 more where it is one reason among several

=== summary over the 24 files ===
  ThisEntityOfTheSolidWasRefused (an #id to read a pair from): 24
  TheSolidTheFileDescribesIsNotOne (no entity, whole-solid form):      0
  anything else (succeeded, or a different refusal):                   0
  of the 24 with a named entity:
    TANGENT — touch without crossing, a limit:  18
    TRANSVERSE — they really cross, a GAP:       0
    THEY NEVER MEET — no crossing to follow:     0
    UNDECIDED — no test here knows the pair:     6
    no surface pair readable from the file:      0
```

**Not one of the 24 is a plain crossing the marcher failed to follow.** The
number the example used to print — every file transverse, every file a GAP —
was worth exactly nothing, and §113.5 was right to refuse to read it.

### 117.1 Eighteen are a ball capping a pipe, and the numbers are exact

All 18 are cylinder/sphere, and all 18 come back with the ball's centre
`0.000000000000` from the pipe's axis and the ball's radius equal to the
pipe's — three radii across the 18, `0.2`, `0.25` and `0.5`, and no other
value. That is the capsule end §113.5 worked out by hand for one file, holding
for every one of them: the ball and the pipe touch all the way round a circle
and never cross, so there is no crossing for the marcher to follow. **A limit,
not a gap**, and now on evidence rather than on a default arm.

The test is one line of algebra rather than a special case. Put the pipe's
axis along `z`; a ball of radius `rs` whose centre sits `e` from that axis
meets a pipe wall of radius `rc` where

```text
(z - h)² = rs² - rc² - e² + 2·rc·e·cos t
```

going round the pipe by `t`. The right-hand side runs between `rs² - (rc+e)²`
and `rs² - (rc-e)²`, so the whole question is where `rs` sits against `rc + e`
and `|rc - e|`: equal to either, the right-hand side just reaches zero and the
two touch without crossing; between them or above both, a real crossing curve
exists; below both, they never meet. Those five are every case, because `rs`
is one real number and is either equal to one of two values, between them,
above both, or below both — so the ball-and-pipe family is answered in full
and never lands in `CannotTell`. With `e = 0` the two values are the same
number and the touch runs all the way round; with `e > 0` they differ and a
touch is a single point. The eighteen are all `e = 0`.

### 117.2 The six are two equal pipes crossing at right angles, and that one is not decided

The other 6 are cylinder/cylinder, axes `90.000000` degrees apart, equal radii,
and — with the distance between the two axes computed properly — axes that pass
through each other. Two pipes of the same radius `r` whose axes cross meet in
**two whole ellipses**: with the axes along `z` and `x`, the walls meet where
`x² = z²` and `y² = r² - x²`. The two ellipses cross each other at `(0, ±r, 0)`,
which is exactly where the two walls' normals line up — both `(0, ±r, 0)`.

So a crossing is there to follow everywhere except at two points, and at those
two points the walls touch and four branches of the curve meet. Neither "they
touch without crossing" nor "they really cross" is the truth about that pair.
It is `Verdict::CannotTell`, printed as UNDECIDED, and the message says which
shape and why. **It is not folded into transverse**, which is the whole point
of §113.5 and of this section.

That is a claim about this test and not about the arithmetic (§16.10): the
answer exists, and a marcher that splits the curve at the two singular points
could follow all four branches. Nobody is building that yet. What is recorded
here is that six of the corpus's twenty-four refusals are that shape and not a
gap in a filter.

**§118 corrects the paragraph above.** Measured on `90006e7b`: the pair is
exactly degenerate rather than nearly so, the answer is two ellipses in closed
form, and the fix family is not a marcher. The counts in this section stand.

### 117.3 The `bool` was the bug, and the cylinder/cylinder arm had it twice

`PairEvidence` carried `confirmed_limit: bool`. Every pair no arm of
`evaluate_surfaces` knew got `false`, and `false` printed as "TRANSVERSE — a
gap". A `bool` has no room for "no test here knows this pair", so the default
arm and a measured crossing came out as the same word. It is now a four-way
`Verdict` — `Tangent`, `Transverse`, `NeverMeet`, `CannotTell` — and the
summary prints all four with their own lines, so a count of undecided pairs can
never again be read as a count of gaps.

**The same fault was in the cylinder/cylinder arm and was not about spheres at
all.** That arm called a pair transverse whenever the axes were more than
`DEGENERATE_DEG` from parallel. Two pipes leaning on each other can touch along
a curve, so "not parallel" is not "crossing", and the inference was unsound.
Worse, the distance it printed as evidence was one pipe's **origin** measured
against the other pipe's axis — whichever point the file happened to write
down, not a property of the pair. For the six it printed `0.400000000000`
against a touch at `0.200000000000`, for pairs whose axes in fact pass through
each other at distance `0`. Leaning axes now get the distance between two
lines, and for them `d > r1 + r2` is never meeting, `d = r1 + r2` is a touch at
one point, and less is a crossing — no second radius case, because no pipe can
hide inside another it is not parallel to.

Finding a second copy of a fault in the arm next door is the ordinary outcome
of fixing the first, and it is why the fix was a type and not an extra arm.

### 117.4 The count is 24 today where §107.1 counted 44, and this does not say why

§107.1 and §113.5 both speak of 44 files. Run today against `44c59a7e`, the
example finds **24** files whose only probe reason is the marcher giving up on
a crossing, and **0** where that reason appears among others — so the missing
twenty do not report it at all any more, rather than reporting it alongside
something else. §113's own test already counts 24, so the two agree.

What moved is not measured here and this section does not guess. §112 fixed a
near-tangency subtraction that was a gap in the same window, which is the
obvious candidate and is exactly the sort of thing that should be checked
rather than assumed. The example now prints the "among others" count on every
run so the question is answerable from the output instead of from memory.

### 117.5 The translation §110 could not chase does not reach this census

§110 records that the two `Surface`s the refusal carries cannot be matched back
to the file's own surfaces, because the sweep holds them a translation away
from where the reader wrote them — a pipe at `(0, 0, 1)` against faces at
`(23.3, ±4.5, ·)`, and so on. That is still open and still `rcad-topo`'s.

It does not touch these numbers. This example never uses the refusal's
surfaces: it reads the refused `EDGE_CURVE`'s two owning faces out of the
file's own topology, so both sit in the frame the file wrote them in, and every
quantity it compares is a distance or an angle **between the two** — never a
position. Whatever translation is applied to the pair is applied to both halves
and cancels. That is why a census could be finished while §110 stays open.

*"Still open" was already false when this merged: §116 landed in the same wave
and moves the pair back into the model's frame. §117.6. The argument above is
untouched and is the better one — this census never reads the refusal's
surfaces, so it held either way.*

### 117.6 Measured again on the merged tree: 26 files, not 24

*Written at merge time, 2026-09-17, by the dispatcher rather than by the lane.
Everything above is a worktree of `44c59a7e`; this is `6b0badfa`, which is this
section together with §115 and §116.*

§115 landed in the same merge and rescued thirteen more KiCad files into
importing. Two of those get **further** than they used to and reach the
near-parallel wall for the first time, so the wall's set **grew**:

| | `44c59a7e`, above | `6b0badfa`, merged |
|---|---|---|
| files whose only reason is the near-parallel crossing | 24 | **26** |
| tangent — touch without crossing, a limit | 18 | **20** |
| transverse — they really cross, a gap | 0 | 0 |
| they never meet | 0 | 0 |
| undecided — no test here knows the pair | 6 | 6 |

Three routes agree on 26: the example's own count, the set
`every_file_that_stops_at_the_tangency_wall_names_the_edge_it_was_following`
selects, and the refusal tally in `how_often_a_real_file_imports`'s log. §115's
own rates are unmoved by the merge — kicad 110/323, nist 1/44, re-derived on
this tree rather than carried across from the lane's.

**Every conclusion above survives the move.** Both newcomers are tangent, so
§117.1's ball-capping-a-pipe family gained two and lost none; the six undecided
are the same six, still two equal pipes crossing at right angles; and transverse
is still empty, which is the claim §113.5 actually wanted.

**This is the stale-claim shape of §16.10 happening inside a single merge**, for
the second time in two days — §112 moved §113's fixture the same way yesterday.
A lane measures on the base it was cut from, and its number is a fact about that
tree and no other. Where a number *is* the point of a section, it has to be
re-derived on the tree that is actually pushed, by somebody who is not the lane
that produced it. The count above is the audit lane's, not Lane C's, for exactly
that reason.

