## 78. A corner a few ulps from where its own planes cross, recorded 2026-09-09

§75.2 made `the_place_the_model_points_at` choose between the places three
surfaces meet by asking which of them the model's own corner sits in. It asked
that question only when there was more than one place to choose from. One place
and it returned that place, unasked:

    if let [only] = places {
        return Some(*only);
    }

That is every corner of every flat solid, because three planes meet at one point
or at none. The test §70.3 asked for was in the code and switched off across the
whole polyhedral half of the kernel.

§75.6 called making it universal "one line", said the line was not taken for a
reason that was measurement and not taste, and gave three reasons. All three were
wrong; §75.6 now carries the corrections at the sentences that were wrong. This
is the unit that found that out by taking the line, and the two numbers it turns
on are: bit for bit the line costs **30 solids the kernel had already accepted**,
and with the stored corner given the room that writing it down cost it costs
**nothing at all** — no published rate moved by a single fuse.

### 78.1 A test skipped exactly where there was nothing to choose

Two different questions were being run together, and only one of them goes quiet
when there is a single place.

*Which of these places is the corner* has no content when there is one place.
That is what §75.2 wrote down — "one place and the model is not consulted at all
— the surfaces gave a single answer and there is nothing to choose" — and it is
true of the arithmetic.

*Is this triple talking about this corner at all* has exactly as much content as
ever. It is the question §70.3 asked for, and it was the one being skipped.

The corner §70.3 describes is the one that shows the difference: three surfaces
at a grazing angle, a corner close to all three and far from where they cross.
The three surfaces do meet, at one place, and that place is nowhere near the
corner the model wrote down. The guard took it without a word, and
`derive_a_corner` then wrote the meet into the answer as if the model had agreed.

### 78.2 What asking bit for bit cost, and what the number 33 is

Take the guard out and change nothing else. **30 solids the kernel had already
accepted are refused, and 3 that were already refused change the reason they
give.** The skew corpus goes from 51 refused of 429 to 81 — 11.9% to 18.9%, over
the 18% §22.4 publishes as its threshold, so the suite goes red and prints the
number itself. That is where the counts in this section come from: the
`how_often_*_refuses` suites assert their own rates, so a failure message is a
measurement.

**33 is the size of the "where three surfaces meet" bucket. It is not a count of
newly-refused fuses.** That distinction was got wrong twice inside this unit —
once in a commit message and once in a doc comment, both corrected — which is why
it is spelled out rather than left to arithmetic. `WHERE_THREE_SURFACES_MEET` is
what a corner nothing could place reports. 30 of the 33 are fuses that used to
answer and now report it; 3 are fuses that already refused for some other reason
and now refuse for this one. A reader who takes 33 as "33 more refusals" is off
by three, and — worse than the three — has the wrong picture of what moved.

### 78.3 All thirty are gaps, by §16.10's test

§16.10 splits a refusal two ways. A **limit** is where the answer does not exist;
a **gap** is where it does and the arithmetic could not reach it. Defer a limit,
fix a gap. All 30 are gaps, and there are three separate things to show for it.

**The answer exists, and no fat box is hiding it.** The triple places the corner
in a box **one ulp wide**, and both boxes in the comparison — the meet's and the
stored corner's — are certified enclosures of true points. Neither is wide enough
to be papering over the disagreement. The two points genuinely differ.

**Nothing here is undecidable.** `side_of_surface(surface, stored)` over all 99
planes of the 33 corners gives **18 `Neg`, 15 `Pos`, 66 `Zero`** — and 18 + 15 =
33, exactly one plane per corner coming back with a decided sign. A decided sign
is a proof that the stored corner is off that plane, and the other two planes
hold it exactly. Not one `Undecidable` in 99. The arithmetic could see the
disagreement perfectly well; it was the comparison that threw the answer away,
after which `derive_a_corner` had no placement left to report and said so.

**And the answer thrown away was the right one.** It is the meet of the three
planes, which is what §17.1 says a corner is. The thing a few ulps out of place is
the corner the model wrote down, not the meet — §75.2's "the model's corner never
places the point" cuts this way as well as the way it was written.

Gaps, then, to be fixed rather than deferred.

### 78.4 Where the few ulps come from, worked out exactly

Not "a few ulps, probably rounding". Exact rational arithmetic over the six
situations, which is small enough to do by hand.

`build_side_face` pins a wall's plane through the **start** of its bottom edge.
The offset is computed to make `n·p = off` hold at that one point. The corner at
the *other* end of that edge inherits whatever `n·p` rounds to there, so it lies
on the plane only up to a rounding — and the meet the three planes solve for sits
a few ulps from the corner the model stored.

The residual is exactly **±1/2⁵⁴, ±1/2⁵³ or ±1/2⁵²** in every one of the six, and
the true meet lies **0.28 to 1.12 ulps** from the stored corner. Three of the six
are under one ulp. That is the whole of the disagreement: it is the width of the
number and nothing else.

**The normal is not the culprit, and that was checked rather than assumed.** The
tilted walls carry `(±1, ∓2, 0)/√5`, and the obvious suspect is the unit-length
scaling doubling a rounding on the way in. It does not: `2·fl(1/√5)` and
`fl(2/√5)` are equal bit for bit. The normal is as exact as an irrational normal
gets, and the miss is entirely in the offset having been pinned at one end of an
edge.

### 78.5 The shape of the thirty-three

The 33 are **11 configurations × 3 rules** — union, intersection, difference.
Every one of them fails while baking an **input** solid, never the answer, which
is why the rule makes no difference: the same corner of the same input is being
re-derived whatever is being asked of it. So the 33 are really 11 facts wearing
three hats each.

Under those 11 are **six distinct (corner, three-plane) situations**, and all six
look the same:

- the corner is at `z = 0`, on the bottom cap, at small whole-number coordinates;
- the three planes are the floor, one axis-aligned wall, and one tilted wall
  whose normal is `(±1, ∓2, 0)/√5` — no other tilted normal appears in any of the
  six, and a `skew ±1` wall appears in none of them;
- the stored corner's box has **zero width in all three axes**. It is an exact
  `f64` point, which is why "widen the meet's box" was never the fix here;
- the miss is on the **x axis only**, and how far follows from §78.4 with no new
  measurement. `build_side_face` (`crates/rcad-topo/src/build.rs:908`) pins the
  wall through `ring[index]` with normal `(along_y, −along_x, 0)`, so the tilted
  wall's `|n_x| = 1/√5` and the whole residual is taken up along x:
  `|Δx| = |r| / |n_x| = |r|·√5`. The three residuals give **1.241e-16, 2.483e-16
  and 4.965e-16**.

**Say which ulp.** The corners sit at small whole-number coordinates in `[2, 4)`,
where an ulp is `2⁻⁵¹ = 4.441e-16`, and **in ulps of the corner's own coordinate
the three misses are 0.2795, 0.5590 and 1.1180** — exactly §78.4's "0.28 to 1.12
ulps, three of the six under one". In ulps of 1.0, where an ulp is
`2⁻⁵² = 2.220e-16`, the same three misses are **0.559, 1.118 and 2.236**. The two
units differ by a factor of two, so a bare "n ulps" here says nothing.

What stood in this bullet was "1.11e-16 to 4.44e-16 — 1 to 4 ulps of 1.0", and it
was wrong twice over. It scaled the residual by **2** rather than √5 — 2 is
`1/|n_x|` for no normal in the six — and then counted the result in units of
`f64::EPSILON / 2 = 2⁻⁵³ = 1.11e-16`, which is half an ulp of 1.0 and not an ulp
of anything.

**`|skew| = 2` is necessary and nowhere near sufficient.** 99 of the 143
configurations in the corpus contain a solid with a ±2 shear, and **11 of those
99 hold a corner the bit-for-bit guard could not place**. Those 11 times the three
rules are the whole 33-entry bucket — they are not 11 refusals on top of it, and
the bucket is not 33 newly-refused fuses either: 30 of the 33 are fuses that used
to answer and 3 were already refusing for another cause (§78.2). The tilted wall
is the setting, not the cause; what picks out the 11 is which edge of which face
got pinned where, and that is not a property of the shear.

Across every call the guard used to cover: **10 632 single-place calls, 33
misses, 0.31%.**

### 78.6 The room, read from `validate.rs` rather than copied

The fix widens the stored corner's box on every side by

    MEASURING_SLACK × Plane::SIGNED_DISTANCE_ROUNDOFF × L

and takes `MEASURING_SLACK` from `validate.rs` — which became `pub(crate)` for
it, value unchanged at 16.0 — rather than writing 16.0 down a second time.

That is the point of the fix and not an implementation detail. `check_flat_face`
grants a corner exactly this allowance against exactly this disagreement, and **a
solid `Model::validate` accepts has to be one the fuse can bake.** Two constants
that happen to be equal make that true by coincidence, and a coincidence drifts:
the next person to tune one has no way of knowing the other was leaning on it.

`L` is **the corner's own size, floored at 1.0** — not the model's
`largest_coordinate()`, the way `Model::what_writing_a_plane_down_costs` measures
it. Two reasons, and the second is the one with a price.

- There is no model here to ask. `the_place_the_model_points_at` is handed two
  points, and threading a model down to it would be threading it for this alone.
- Model-wide would hand a corner at the origin the room a corner a kilometre away
  earned. One feature far out and the test is loosened everywhere.

The price is real and worth naming: a small corner on a face whose *other*
corners are huge was rounded by the big face's arithmetic and is granted the
small corner's room. That triple then says nothing about it, and another has to
place it; if none does, the fuse refuses. A refusal and never a wrong answer,
which is the direction to be wrong in.

**This is a tolerance, and the doc comment says so.** §75.2's "needs no
tolerance, no constant and no policy" was true of what it was written about —
comparing two certified boxes round two readings of one point. The model's corner
and the model's surfaces are not two readings of one point. Both boxes have zero
width when the arithmetic is exact, so *any* rounding at all is a miss, and there
is no way to compare them without granting something. What keeps this from being
a modelling tolerance is which number it is: the width of the arithmetic, not a
distance anybody chose. `Scale::RELATIVE_LINEAR_TOLERANCE` is `1e-9`
(`crates/rcad-geom/src/scale.rs:69`); the room granted here is
`16 × 4 × f64::EPSILON = 1.421e-14` of `L`. That is **70 369× wider — not quite
five orders of magnitude** — and it would swallow a corner somebody placed off
its plane on purpose. §78.7 does the same division one subsection down with
`1e-10`, a tenth as large, and gets 7 037×; the two agree because they are the
same ratio ten apart.

**"Seven orders of magnitude" stood here, and it is right about a different pair
of quantities.** `allowed_wobble` (`crates/rcad-topo/src/validate.rs:1034`) reads
`RELATIVE_LINEAR_TOLERANCE` against the largest coordinate, so it is about `1e-9`
*relative*; a certified predicate errs at about `f64::EPSILON`, `2.2e-16`
relative. That ratio is 4.5e6 — seven orders to the nearest power of ten, six and
a half exactly. Neither quantity in it is the room this fix grants, so the two
comparisons cannot stand in for one another.

### 78.7 The factor, swept and not argued

The 16 is not reasoned to. The corpus was re-run at each factor.

| factor × `SIGNED_DISTANCE_ROUNDOFF` | `WHERE_THREE_SURFACES_MEET` | skew refusal rate |
|---|---|---|
| 0 (bit for bit) | 33 | 18.9% |
| 0.25 | 6 | 13.3% |
| 0.5 | 0 | 11.9% |
| 1 | 0 | 11.9% |
| 16 (shipped) | 0 | 11.9% |

A gate agent re-derived the two boundary rows — 0.25 and 0.5 — independently,
which is what makes the step between them a measurement rather than a typo.

So the corpus needs **between 1 and 2 ε of `L`**, which is *less* than one bare
`SIGNED_DISTANCE_ROUNDOFF`. Everything above 0.5 buys nothing that this corpus
can see, and is headroom.

The question worth asking is what the headroom costs, and the answer is that it
is four orders of magnitude short of anything anyone would mind. A corner
somebody placed `1e-10` off its plane is about **7 000×** the room 16 grants.
Between "swallows a rounding" and "swallows a placement" there are four orders of
magnitude of nothing at all, and 16 sits near the bottom of that gap — closer to
the measured need than to the first thing it would be wrong to swallow.

**No published number moved.** Every cell of the skew tally, and every
`how_often_*` rate, came back to the value it had before any of this. Making the
test universal is not a refusal-rate change; it is a hole closed at no measured
cost.

### 78.8 What a published bound is worth if nothing re-derives it

This part was on nobody's list, and it is the part worth reading twice.

While measuring the above, the corpus was found to have grown from **423 to 429**
per family. The first guess was the generator, which is the wrong guess §38.8
made and §39.6 corrected, and it was wrong again: the six new cases are two
proptest regression seeds, checked in at `01e62c1` and `9b3a5fd`. The seed files
*are* part of the corpus, so adding one seed grows every family at once.

**Eleven figures across six doc comments were stale.** All six
`*_IS_NOT_A_SOLID` counts had fallen, and they had fallen *together*, at one
commit: `b4890f1`, "cuts: a solid's own planes cut its own faces". The direction
is right — fewer answers are not solids because fewer answers are wrong — and
**no band fired**.

Not because the bands were unusually wide. End to end they were lattice
`0.02..=0.10` at 5×, flush `0.10..=0.28` at 2.8×, elsewhere `0.02..=0.10` at 5×,
turned `0.015..=0.09` at 6×, skew `0.005..=0.03` at 6× and leaning-alike
`0.012..=0.08` at 6.67×. The replacements span 4.5×, so a sentence condemning the
width condemns those too — and the flush band, kept as it was, is the *narrowest*
of the six.

What was wrong is **where the count sat inside the band**, which is a different
number and the one that matters. Leaning-alike stood at 17 of 423 — 4.02% —
against a lower end of 1.2%, so its count could fall to **30% of itself** before
the band said a word. It fell to 41%, landed at 1.63%, and the band stayed quiet
because 1.63% is still above 1.2%. A band drawn by the convention below has its
lower end at two thirds of the count it was drawn around, so it speaks at any
fall past 67%. That is the whole of the difference, and the fall factors in §78.9
show it: of the six families, exactly two fell past two thirds — leaning-alike at
0.41 and skew at 0.375 — and those are the two the archived re-run names below.

They were re-derived to the convention this file already states for them,
"roughly two thirds to three times", which makes every span 4.5×.
`A_FLUSH_PAIR_IS_NOT_A_SOLID` is a deliberate exception, kept tighter at
`0.10..=0.28`: the convention applied to it would have gone from 43–120 permitted
fuses to 41–184, and no measurement asked for that loosening. A convention is not
a reason to widen a band that is doing its job.

The test of the new bands is whether they would have caught what got past the old
ones, and that was measured on the archived trees rather than argued: under the
new bands, **leaning-alike and skew would have fired at `b4890f1`**, and two more
families came within one fuse of firing.

**What that sentence means is the convention and not the shipped numbers**, and
the difference is worth spelling out because the shipped numbers say the
opposite. `A_PAIR_LEANING_ALIKE_IS_NOT_A_SOLID` ships as `0.011..=0.049`, which
is the convention drawn around **today's** 7 of 429 — and the pre-`b4890f1` rate,
17 of 423 or 4.02%, sits inside it. Run the shipped band on the archived tree and
nothing fires; it cannot, because it was drawn around the count after the fall.
The claim is about the convention: a band drawn by it around the count *at the
time* has its lower end at 2.68%, and `b4890f1`'s 1.65% is below that.

One more line, because the name misleads:
`share_of_answers_that_were_not_solids_must_be_within` divides `not_a_solid` by
`self.fuses`, not by the number of answers. A fuse that refused never got far
enough for its answer to be judged, so the denominator counts cases the numerator
structurally cannot reach. It is a share of fuses. §22.3's caveats say as much in
prose about one family; the name says the opposite everywhere.

### 78.9 §22.3's defect is not back, measured once and by nothing in the tree

§22.3 records a shear applied as `S(ring) + at` instead of `S(ring + at)` — the
image of nothing — whose symptom was a not-a-solid quietly becoming an ordinary
solid. It was caught by a rate that did not add up. A rate is a weak instrument
for it: the argument is that an affine map preserves incidence **exactly**, and a
rate can only ever say that two numbers are close.

So it was checked directly this time — **once, in a lane's scratchpad, and
nothing in this repository re-derives it.** That is worth saying plainly, because
it is §78.8's own subject two subsections up and this section would rather own it
than smooth it. `a_pair_leaning_the_same_way()`
(`crates/rcad-topo/tests/pairs/mod.rs:481`) is called from exactly one place,
`crates/rcad-topo/tests/fuses_that_must_agree.rs:1423`, and builds each
configuration **once**. The doubled build — each configuration again with
`Skew::none()` — never left the scratchpad it was written in. Everything in the
rest of this subsection is one lane's measurement at three commits, not a
standing property of the tree, and §78.11 lists it with the rest.

What that measurement said: build each leaning-alike configuration as drawn and
again with `Skew::none()`, and the cell §22.3's bug lived in is **0 of 429 at all
three commits**, as is its mirror. So, on this corpus, at those commits, measured
once: the shear turned no not-a-solid into a solid and no solid into a
not-a-solid. **"Incidence is preserved case by case" is what that supports — on
this corpus, at these commits, measured once.** It is not a property anything
re-checks, and the argument that an affine map preserves incidence exactly is
still an argument.

The rate did move, and it moved in a **different cell**, which is why the two
numbers are not the contradiction they look like. Leaning-alike's not-a-solid
rate fell from **4.0% to 1.6%**, and on identical configurations the unsheared
build gives 12 not-a-solid answers where the sheared gives 7. **The missing 5 are
exactly the cases the sheared fuse refused** — 12 = 7 + 5, with nothing
unaccounted for. §22.3's cell is not-a-solid → **solid**; these 5 are
not-a-solid → **refused**. A refused fuse never gets far enough for its answer to
be judged (§78.8's denominator again), so the shear lowers the not-a-solid count
by refusing, not by turning a not-a-solid into a solid. That is the opposite
failure from §22.3's and it is benign.

The fall was not proportional across families, and the split falls along the line
anyone would draw in advance. **These factors are counts, not rates**: the
not-a-solid count at `b4890f1` over the count at its parent, both on the same
423-fuse corpus. Lattice 0.70, flush 0.85, elsewhere 0.71, turned 0.76 — against
leaning-alike 0.41 and skew 0.375. **The two tilted-wall families fell about
twice as far as the axis-aligned ones.**

Turned shows why the unit has to be named. As counts it is 13 over 17, **0.76**.
Today's rate over the parent's rate is a different question with a different
answer: turned stands at 14 of 429 now, and `14/429 ÷ 17/423` is **0.81**. The
figures above are the first of those.

### 78.10 Two open items, named so nobody re-derives them

Both were left open deliberately, and both are written out in enough detail to
pick up cold. **The first is closed now, by §80, and it closed the opposite way
round from the way this subsection guessed the question.** **The second is
closed by §82, and it was not open when this was written.**

**One fuse went the wrong way at `b4890f1`.** Of today's 7 leaning-alike
not-a-solid cases, six are members of the old 17 and one is new:

    Union of
      Frame { width: 5, depth: 3 }, height 2, at [0, 0, 0]
      Comb { teeth: 1, tooth_height: 3 }, height 3, at [-1, 1, 2]
      shear x_per_y: -1

It answered at `cc959c1`, sheared and unsheared. It is not-a-solid now, sheared
and unsheared, so the shear has nothing to do with it. `b4890f1`'s stated
direction is that fewer answers are not solids because fewer answers are wrong,
and this one went the other way. **Nobody has asked which of §16.10's two it is,
and this section does not guess** — it may be a limit, it may be a gap, and
saying which needs the same kind of work §78.3 did for the 30. Deferred because
of the shape of the outcome: not-a-solid is a refusal, so nothing corrupt is
written while it stands. What it costs is that the 7 is not fully attributed.

**Closed 2026-09-09 by §80, and the paragraph above is left standing because
what it got wrong is the useful part.** Only one side of `b4890f1` is one of
§16.10's two, and that is the side this subsection asked about. Today's refusal
is a **limit**: nothing was undecidable at any policy, and a union with four
faces along one edge is not something a `Model` can hold (§17.4's first row).
The old behaviour is neither of the two, because it was not a refusal — it was a
**missed refusal**, an answer written to a configuration §17.7 refuses, for one
of the two shapes that relation comes in while the same build refused the other.
Two solids with a contact neither of them recorded, right volume, `validate`
happy, because §4.3's checks are local and nothing was locally wrong. §16.10
sorts refusals and does not reach a wrong answer; §80.1 declines to stretch it
and says why. So "this one went the other way" is true of the tally and wrong
about the direction; a tally counts refusals, not mistakes. §80.4 has the
mechanism — neither touching edge contained the other, and the pieces they were
cut into did not line up, so the manifold check was never asked the right
question — and §80.8 corrects `cc959c1` to `c66bdca`, which is `b4890f1`'s
actual parent. §80.9
carries the one thing that stayed open, and it is a possibility with no consumer
rather than a defect.

**`Assembly::commit` self-checks nothing in a release build.** `was_sound` is

    cfg!(debug_assertions) && model.validate(policy, &mut ProvenanceLog::new()).is_ok()

so in release it is `false` and the `if was_sound { … }` block below it — the one
whose `expect` reads "a fuse writes a valid model or does not write at all" —
never runs. The doc comment above it explains *why the check is conditional on
the model having been sound before the write*, which is a good reason and a
different one; nothing there tells a reader the whole check is debug-only.
Related in shape to §66: a promise the doc makes that the release build does not
keep. Not changed here because it is a real decision with a cost on both sides —
a full `validate` per commit is the quadratic sweep §4.3 exists to avoid — and it
is not this lane's to make.

**Closed 2026-09-09 by §82, and the paragraph above is left standing for the
same reason the one above it is.** Not a real decision with a cost on both sides:
**§66.3 had already decided it**, two days earlier and about this exact line,
and §4.1's middle row and §4.3's first bullet had both carried the amendment
since. What this paragraph did was read §4.3 as it stood *before* §66 amended
it, and so re-open a settled question — "related in shape to §66" was closer to
the truth than it reads, because it *is* §66, unimplemented.

The observation itself is the useful part and none of it is wrong: `was_sound`
really was a silencer and not a check, and the doc comment really did explain
the conditional without mentioning the gate. §82.1 keeps both. Where the
paragraph guessed at the cost it also guessed low-and-high at once — the sweep
is not the quadratic one §4.3 is about *per operation*, and it is worse than
§66.3 assumed on a model the fuse did not build. §82.7 measures both halves, and
what went in is a local check for that reason.

### 78.11 The figures in this section that nothing re-derives

§78.8's whole subject is a published number nobody re-derived, so this section
closes by naming its own. A gate agent worked back through §78 from the tree and
could not reach the following: each needs instrumentation that was never checked
in, or a build of an archived commit. Read them as one lane's measurement on one
day, not as standing properties.

- **`18 Neg, 15 Pos, 66 Zero`** over the 99 planes of the 33 corners (§78.3).
- **The residual being exactly ±1/2⁵⁴, ±1/2⁵³ or ±1/2⁵² in *every* one of the
  six**, and the six situations sharing the shape §78.5 lists (§78.4, §78.5). The
  arithmetic from a residual to a miss *is* re-derivable and is written out in
  §78.5; the residuals themselves are not.
- **`10 632` single-place calls, 33 misses, 0.31%** (§78.5).
- **`99 of the 143`** configurations holding a ±2 shear, and the 11 among them
  (§78.5).
- **The bit-for-bit cost, 30 + 3 = 33** (§78.2). It was a suite failure message
  when it was taken, and the guard it was taken without is back in, so nothing
  prints it now.
- **Every row of §78.7's sweep table.** The `0.25` and `0.5` rows were re-derived
  independently by a gate agent, which is what makes the step between them a
  measurement; no row is re-run by anything in the tree.
- **`12 = 7 + 5` and the `0 of 429` cell at all three commits** (§78.9) — the
  doubled build that produced them is not in the repository.
- **The six fall factors** — lattice 0.70, flush 0.85, elsewhere 0.71, turned
  0.76, leaning-alike 0.41, skew 0.375 (§78.9).
- **Which families the new bands would have fired on at `b4890f1`**, and the two
  that came within one fuse (§78.8). That needs archived trees.
- **`1.665e-16`, the worst margin over 72 coordinates** (§75.6, and this unit
  leans on it). The test is in the tree; the margin is not, because the test
  ships `1e-12` of slack and reaching the figure means editing it to `0.0`.

What *is* re-derived, every run: the shipped `how_often_*` refusal rates and the
six `*_IS_NOT_A_SOLID` bands. Those suites assert their own numbers, so a green
`just gate` is the statement that they have not moved. Nothing above is in that
set.

