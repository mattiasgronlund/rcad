## 22. A pair that is not square to itself

*Recorded 2026-08-31.* §17.8 priced a new shape family at about fifty thousand
to one against more of the old corpus, and §19.6 carried "nothing here widens
the shape vocabulary" forward as the best-value work available. This widens it by
one family and one control. The family is the cheapest thing that changes the
**arrangement** rather than the numbers: one whole-number shear per solid, no new
outline, no new arithmetic on the way in.

### 22.1 What the corpus structurally could not draw

Every configuration family that existed before this one turns **both** solids by
the same rotation. `a_turned_pair` rotates the pair; `a_pair_somewhere_else`
scales and shifts it; `a_flush_pair`, `a_chain_of_fuses` and the rest leave it on
the axes. So however irrational the world coordinates get, the pair stays exactly
as square to *itself* as the lattice configuration it came from, and both solids'
wall planes have normals drawn from one orthonormal triple.

That has a consequence nobody had written down. Take a face whose normal `n` is
from that triple. Every line cut into it — where another plane meets it, and its
own boundary — runs along `n × m` for some other `m` in the triple, and there are
only **two** such directions. Three distinct lines through one point need three
distinct directions, because two distinct lines sharing a direction are parallel
and meet nowhere. So:

**No configuration in the corpus has ever put three concurrent cut lines on a
face.** Not on any seed, not at any scale, not under any rotation. It was not
rare; the generators could not express it.

A face's arrangement has to put the intersections it finds in order along a
line. Where two of them are the same point, that order is a **tie** rather than a
comparison, and no amount of precision settles a tie. That is the branch the
suite had never reached.

### 22.2 The family, and what stays exact

`a_skew_pair` leans each solid over by its own whole number: a corner at
`(x, y)` moves to `(x + a·y, y)`, with `a` drawn from `-2..=2` per solid and
shrinking toward `0`, which is the lattice configuration the case came from.

As a matrix that is `[[1, a], [0, 1]]`, determinant `1`, and the determinant does
three jobs at once. An affine map takes a simple polygon to a simple polygon and
a positive determinant keeps it anticlockwise, so a leaning `Shape` never needs
checking and never gets filtered — every draw builds. Whole numbers go to whole
numbers, so every corner reaching `ProfileBuilder` is exact. And the volume does
not change, so a leaning pair encloses what its lattice original enclosed.

What is **not** exact, and provably cannot be, is the tilted walls.
`build_side_face` takes a wall normal as `(along_y, -along_x, 0)` and
`Plane::from_point_and_normal` normalizes it. The only exactly representable unit
normals are the signed axis directions, and that is a proof rather than an
observation: an `f64` is a dyadic rational, so a unit normal with `f64`
components clears to whole numbers with `p₁² + p₂² + p₃² = 4^k`; squares are `0`
or `1` mod `4`, so all three must be even; so every solution descends to one of
`4^{k-1}`, and the descent bottoms out at `1 = 1 + 0 + 0`. Only `(±2^k, 0, 0)`
and its permutations survive.

So this family sits between the two kinds the suite already had. **Corners exact,
caps exact, tilted walls rounded.** `a_lattice_pair` has everything exact;
`a_turned_pair` has nothing exact. Nothing had ever separated the two effects.

Because of that it runs only what is exactly true whatever the coefficients are:
the two ways round a question agreeing bit for bit, and every plane of an answer
being a plane of an input. Inclusion and exclusion is not asserted, for the
reason §17.8 gives for turned pairs. Neither is transform invariance, and that
one is specific here — moving a plane slides it along its own normal by `n · t`,
exact when `n` is a signed unit axis and a rounding otherwise, so the property
would be comparing the generator's arithmetic against the kernel's rather than
the kernel against itself.

### 22.3 The control, and the ordering that nearly made it a lie

§17.7 settles that families are reported separately and never as one number, and
that the control is the row that earns its keep. The control here is
`a_pair_leaning_the_same_way`: **both** solids leaned by the same amount, which
is one affine map `S` of the whole lattice pair — same shapes, same overlaps,
same touchings, the same cut lines on every face in the same order. The lattice
family's claim is that a certified policy must decide all of that, and it refuses
zero. The one thing left between the two rows is that the walls are tilted.

**That claim was false when it was first written, and the code was what was
wrong.** The shear was being applied to the outline's own corners and the
placement added afterwards, which is `S(ring) + at` — the image of nothing. An
affine image of the pair is `S(ring + at) = S(ring) + S(at)`, so the shear has to
go on *after* the placement. Leaning the solid where it stands, not leaning it
and then standing it somewhere.

It was caught by a number that did not add up, not by reading the code. An affine
map preserves incidence exactly, so a control that is an affine image of the
lattice family must produce not-a-solid answers at about the lattice family's
rate — an answer is not a solid here almost always because two solids meet along
a whole edge and four faces would run along it. The control was reporting 1.9%
against the lattice family's 4.8%. With the ordering corrected it reports 3.1%,
and the remaining gap is the two caveats the file already carries for
`A_PAIR_ELSEWHERE_IS_NOT_A_SOLID`: this family draws one more number per case, so
it is not the same 128 configurations, and a fuse that refuses never gets far
enough to find out whether its answer was a solid.

Worth recording as its own paragraph because of what the wrong version did. It
built, it ran, it shrank properly, it reported a plausible number, and every
sentence written about it was wrong. A control that does not control is worse
than no control, because the subtraction still produces a figure.

### 22.4 The measurement

| Family | Corpus | Refused | Rate | Where |
|---|---|---|---|---|
| Lattice pairs, for reference | 420 fuses | 0 | 0% | `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing` |
| Both solids leaning **alike** | 420 fuses | 30 | **7.1%** | `how_often_a_certified_fuse_of_a_pair_leaning_the_same_way_refuses` |
| The two leaning **differently** | 420 fuses | 60 | **14.3%** | `how_often_a_certified_fuse_of_a_skew_pair_refuses` |

So the subtraction the control exists for: **tilting the walls costs 7.1 points
with the arrangement held fixed, and leaning the two solids differently costs
7.2 more.** Neither number was available before, and the headline alone would
have been the two added together with no way to tell which was which.

The threshold on the headline is **18%**, asserted by the test named above and
judged on the shipped corpus only, for the reason §17.7 gives about applying one
corpus's threshold to another. Fresh seeds at ten times the corpus come out at
15.5% and 15.6% for the headline and 7.7%, 7.8% and 8.0% for the control.

The breakdown is the half anybody can act on, and here it is the claim rather
than a footnote.

| What could not be decided | Leaning alike | Leaning differently |
|---|---|---|
| `"order along a line"` | 9 | **30** |
| `"side of plane"` | 18 | 21 |
| `"a point inside a piece of a face"` | 3 | 9 |

§17.7 records that every refusal the turned family ever produced named
`"side of plane"`, over fourteen seeds, with `"a point inside a piece of a face"`
turning up rarely and **no run ever naming a third cause**. Leaning the two
solids differently does not merely refuse more; it refuses for a *different
reason*, and that reason becomes the leading one. `"side of plane"` barely moves
between the two rows — 18 against 21 — while `"order along a line"` more than
triples. At soak scale the same thing is sharper and steadier: `"side of plane"`
runs at 5.1–6.4% of fuses in both rows, and `"order along a line"` goes from
1.2–2.2% in the control to 9.0–9.4% in the headline.

That is the arrangement, isolated. It is asserted rather than only printed:
`Tally::the_cause_named_most_often_is` fails the headline row if
`"order along a line"` ever stops being what these fuses mostly refuse on, since
at that point the family has quietly become its own control.

Two cautions, because the mechanism is tempting and is not proved here. The
control names `"order along a line"` nine times too, so concurrency is not the
only way to reach it and this section does not claim it is. And what the rate
says about real models is what §17.7 already says about its own: measured over a
corpus steered into coincidence, it bounds how bad the answer can be on
purpose-built input, and is evidence rather than a verdict.

### 22.5 The second slide was written, measured and taken out

The shear started as two slides — `x` by so much per step of `y`, then `y` by so
much per step of `x`, composing to `[[1, a], [b, 1 + ab]]`, still determinant
one. The argument for it was structural and sounded right: one slide leaves every
solid a wall along `(1, 0)`, so two solids always keep one pair of mutually
parallel wall directions, and only the second slide lets a pair share no wall
direction at all.

Measured, it does not do that. It squeezes. A grid of refusal rates over every
pair of leans settles which effect the number reports:

| first \ second | `[0,0]` | `[1,0]` | `[2,0]` | `[0,1]` | `[1,1]` | `[2,2]` |
|---|---|---|---|---|---|---|
| `[0,0]` | **0.0%** | 8.8% | 14.4% | 11.2% | 25.0% | 45.6% |
| `[1,0]` | 6.9% | 4.4% | 18.1% | 16.9% | 24.4% | 44.4% |
| `[2,0]` | 13.1% | 16.9% | 10.6% | 23.8% | 25.0% | 43.8% |
| `[0,1]` | 13.8% | 15.0% | 18.8% | 16.9% | 30.6% | 48.8% |
| `[1,1]` | 38.1% | 36.2% | 34.4% | 43.8% | **36.2%** | 56.9% |
| `[2,2]` | 61.2% | 61.2% | 58.1% | 66.9% | 64.4% | **76.9%** |

The diagonal is the control — both solids leaning alike, an affine image of a
lattice pair, no new arrangement anywhere. A solid leaned `[1, 1]` refuses 38.1%
against a flat partner and **36.2% against a partner leaning identically**, which
is the same number. The rate follows the lean of the single solid and not the
relation between the two. At `a = b = 2` the composition squeezes 34 to 1 against
5.83 to 1 for one slide of two, and the headline the two slides bought was 46.4%
with `"where three planes meet"` leading it — which is what a sliver looks like,
not what an arrangement looks like.

With one slide that cause disappears from the breakdown entirely.

Recorded because the argument for the second slide was a good one and it was
wrong, and only the grid could tell. §16.10's shape again: a locally valid
argument answering a subtly different question than the measurement asks. Two
of those in one section, counting §22.3.

### 22.6 The family found a defect on its first full run

`swapping_the_two_sides_of_a_skew_pair` failed the second time the whole suite
ran, and the cause is one line of arithmetic that every family before this one
was safe from by luck.

`Plane::from_point_and_normal` normalizes the vector it is handed, so what it
stores depends on that vector's **length** and not only on its direction:

```text
normal (1, 1, 0)  ->  0.7071067811865475
normal (2, 2, 0)  ->  0.7071067811865475
normal (3, 3, 0)  ->  0.7071067811865476
normal (6, 6, 0)  ->  0.7071067811865476
```

`build_side_face` takes a wall normal as `(along_y, -along_x, 0)` straight off
the profile, so the length is whatever that outline's edge happened to be. **Two
walls that are the same plane get coefficients an ulp apart when they stand on
edges of different lengths.** §17.1 makes a face *be* its plane, so those are two
different planes to this kernel, and a fuse over such a pair keeps whichever
belonged to the argument — which makes the answer depend on which side is which.
An Ell and a Comb, both leaned by `-1`, both at the origin, intersected: same
solid, same twelve faces, same corners to the bit, and one face on a plane
differing in the last place.

Every rectilinear family is safe from this and none of them is safe *by design*.
An axis-aligned normal of any length normalizes exactly — dividing `(0, 3, 0)` by
`3` is exact — and the turned families rotate both solids by one isometry, so
coplanar walls stay bit-identical however long their edges. Tilting a wall is
what breaks it, and until now nothing here could tilt one.

**A gap and not a limit** (§16.10). The answer exists and the arithmetic can
reach it: dividing the normal by its largest-magnitude component before
normalizing makes the result depend on the ratios alone, and the ratios of two
scalar multiples are the same real number and so round the same way. That is a
change to every plane this kernel stores, which is why it is recorded here and
not made here.

Two things stand in for the fix. `the_same_wall_built_from_a_longer_edge_is_a_different_plane`
pins the arithmetic and is written the way §19 writes them — it asserts what the
code does, so fixing the defect fails it loudly with the fix in hand. And the
commutativity property over skew pairs compares **the corners only**, which is
not much of a retreat: the partition is what §17.4's argument is about, a
splitter that ordered its cuts differently would move a corner, and what is given
up is the last bit of a coefficient nobody disagrees about geometrically. When
the plane is canonical the comparison goes back to every last bit, and the diff
is one word.

Worth saying plainly, because §17.8 predicted it and put a price on it: **the
family found this on its first full run of the suite, after three million fuses
over the old generators had found nothing.**

### 22.7 What this does not settle

- **§22.6's plane defect is recorded and not fixed.** It is a gap, the fix is
  named, and it touches every plane in the kernel. *(Fixed at the opening of
  M2, by exactly the fix named there — §23.8 item 1. Item 2 beside it is the
  second defect on the same line that this section could not see: a normal near
  `f64::MAX` overflowed its own length and came back as a direction of length
  zero. `swapping_the_two_sides_of_a_skew_pair` compares every last bit now and
  `HowMuchMustAgree` went with it, which is the one-word diff §22.6 predicted.)*
- **Whether the `"order along a line"` refusals are deserved.** §14.7's worry
  applies here exactly as it does to the turned family, and a rate cannot tell a
  timid filter from a genuine tie. What would tell is a hand-written pair with
  three cut lines provably through one point, asked whether the two orderings
  give the same solid. That is the next piece of work this family opens, and it
  is not done here. The family's headline is 14.3% against the turned family's
  21.0%, so if any of it is undeserved it is worth about as much as the defects
  §19 fixed.
- **The concurrency claim is an argument about the generators, not a test.**
  §22.1 proves the old families cannot reach three concurrent lines and the proof
  is short, but nothing asserts that the new family *does* reach it. What stands
  in for it is the refusal breakdown, which is a consequence and not the thing
  itself. §15.6's rule about published bounds needing tests points the same way
  here.
- **Nothing here leans a cap.** `add_extrusion` extrudes along `z`, so the top
  and bottom faces of both solids stay parallel to each other whatever the
  outlines do. An oblique prism needs a different builder. §17.8's generator
  families 4 to 6 — `a_hollow_pair`, `a_perforated_plate`,
  `a_pair_that_must_be_refused` — are still unbuilt.

---

