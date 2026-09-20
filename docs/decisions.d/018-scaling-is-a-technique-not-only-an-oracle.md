## 18. Scaling is a technique, not only an oracle

*Recorded 2026-08-31, after the §16.10 claim audit reached `rcad-predicates`.*

M0 wrote a metamorphic oracle: scale a predicate's inputs by `2ᵏ` and the answer
must not change (§14.2). It has been one of the sharpest tests in the tree ever
since, and §14.1 explains why — the two runs take different paths through the
dangerous code.

Nobody noticed that it is also **a way to compute**.

### 18.1 What was being refused, and why it was a gap

```
orient2d([1e-200,0], [0,1e-200], [0,0])   → Undecidable
   all six coordinates × 2^600 (exact)     → Pos, at Level::Float
orient3d at 1e300                          → Undecidable
   all twelve coordinates × 2^-700 (exact) → Pos
```

By §16.9's test these are **gaps, not limits**: the sign exists and exact
arithmetic reaches it. `orient2d` is homogeneous of degree 2 in its coordinates
and `orient3d` of degree 3, so scaling every input by `s > 0` multiplies the
determinant by `s^d` and cannot change its sign. A power of two rounds nothing.

The refusal was excused by the sentence §16.11 corrected in three files and
could not reach in two more — `orient2d.rs` and `orient3d.rs` were **frozen**
under §15.9 at the time. The freeze that made this crate trustworthy is exactly
what let a retired sentence go on doing load-bearing work inside it. That is a
finding about process, not about arithmetic, and it is the argument for auditing
a crate *when its freeze lifts* rather than treating the freeze as a warranty.

The crate's own test asserted *"there is no honest sign to report"*, and the
property refuting it was three files away being used as an oracle.

### 18.2 What was built

A shared helper in `rcad-predicates`, called by `orient2d` and `orient3d` at the
entry point both the plain and tolerant routes funnel through.

- **Lazy.** First pass exactly as before; the retry runs only on `Undecidable`.
  Zero cost on the path that already worked. Retrying at the entry point re-runs
  the whole cascade, which is necessary rather than tidy: the `1e-200` case fails
  at *all three* levels — subnormal products, collapsed intervals, overflowing
  expansions — so a fix inside `on_floats` would not have been one.
- **Normalised on the largest difference**, not the largest coordinate, because
  these predicates subtract before they multiply.
- **A self-check that decides limit from gap.** Every scaled coordinate must
  still be normal, or the retry is discarded and the original refusal stands.
  Coordinates near `1e300` with differences near `1e-300` cannot both be brought
  into range, and that model is a genuine limit that must keep being refused.
  The guard makes the boundary a property of the code rather than of a comment —
  which is the whole of §16.10 applied forwards.
- **Nothing is marked.** No `Level` variant, nothing in provenance. A rescued
  answer is certified in exactly the sense an ordinary one is, and §5's empty log
  has to keep meaning what it says.
- **Two predicates only.** The surfaces would each need their own parameters
  scaled — a radius, a plane offset — and their gap is plausible rather than
  demonstrated. §13 rule 3.

**The bounds survive untouched**, and this is the part that had to be checked
rather than asserted (§16.3). `value` and the permanent are sums of the *same*
degree-`d` products, so an exact scaling moves both by exactly `s^d` and the
comparison `value > BOUND × permanent` is scale-invariant. Asserted bit for bit
in tests over full-mantissa coordinates, not argued in a paragraph.

### 18.3 The oracle nearly went quiet, and two probes already had

Self-normalisation is precisely the change that could make both runs of the
scaling oracle take the same path, turning §14.1 question 2's answer from "yes"
into a tautology that still passes. It was checked *before* the change landed.

It was not blinded — the retry only follows a refusal, and a wrongly certified
level-1 answer is not a refusal. But making that **measurable rather than
argued** found two probes that were already dead: `orient3d`'s overflow search
looked only at the minors, when it overflows in the height products; and
`orient2d`'s never overflowed anything at all, because shifts of 300–400 on
coordinates below `2²³` cannot. That run had been reporting 197,000 healthy
certified samples and testing nothing.

The suite now counts how many samples actually reach the dangerous path and
fails if that count collapses. **An oracle that cannot say how much it is
testing is one nobody will notice going quiet** — §16.9, arriving for the third
time in two milestones, and the first time it was caught before it cost anything.

### 18.4 A point that is not a point

`Interval::exactly(NaN)` maps to `ENTIRE`, justified by *"an enclosure of
everything can never certify a sign"*. False: a zero coefficient annihilates it,
and `orient3d` on a point with a `NaN` coordinate returned a certified `Neg`
under `Policy::certified()` with an empty log.

Not *unsound* — certification needs the value constant over every real in that
coordinate, and it is — but a kernel whose premise is "prove it or admit you
cannot" must not answer confidently about a point that does not exist.

Guarded at the point of use in all six production predicates, not at
construction: enforcing finiteness inside `Point` means private variants, `From`
becoming `TryFrom`, and 80 call sites growing error handling for a case that
cannot arise in them. Refusing to *answer* costs one check where the predicate
already reads its inputs, and matches `product_is_normal`, which is a
point-of-use guard too.

A constructed point holding an unbounded interval is deliberately left alone.
Every interval the public API can build denotes a non-empty set of reals, so a
sign certified across it is a true statement about every point in the box. `NaN`
differs because there is no point at all.

Surfaces turned out to be able to hold non-finite parameters — `transformed` is
infallible and overflows — so a cylinder whose axis origin had overflowed
certified `Pos` about an ordinary finite point. Also guarded; the infallible
`transformed` is not fixed.

### 18.5 The net under the bounds had 7× of slack

§15.6 says a published bound needs a test that could fail. Mutation testing
showed `ORIENT3D_ERROR_BOUND` — `7ε` — could be cut to **`1ε`** with the entire
suite green. §16.3's bound was wrong by nine roundings to seven; **this net would
not have caught it.**

Searching rather than guessing — random exponents, full mantissas, then hill
climbing over `|value − exact| / permanent`, about 10⁸ candidates — found `6.286ε`,
cross-checked against exact rational arithmetic. The net now fails below `6.25ε`.
The measured figure is in the doc comment, so the next reader knows how much
slack remains instead of re-measuring.

`ORIENT2D_ERROR_BOUND` produced a scare worth recording: a search found `3.360ε`
against a `3ε` constant. It is not a forward error bound on `value` — it bounds
the two products against the permanent, and the final subtraction's rounding is
absorbed because `|value|` never exceeds the permanent. Shewchuk's filter working
as designed and as its own doc comment says. The test now asserts the quantity
the bound is actually about.

### 18.6 What this does not fix

- **The large end.** `orient2d` and `orient3d` never fail there. The `1e140`
  cube's `classify_point` failure is a ray target derived from the bounding box,
  in `rcad-topo` — §16.11 carries it with a hypothesis attached. *(Both halves
  of that sentence are wrong; §20.7 measured it. There is no failure left, and
  the ray target was never the cause.)*
- **`product_is_exact` reads like an overflow gate and is not one.** It is true
  for `1e200` squared and true for anything times zero. It is safe today only
  because `expansion_sum` and `scale_expansion` drop an entry only when it
  compares equal to zero, and no infinity or `NaN` ever does — so `expansion_sign`
  is the real backstop. That fact had never been written down; it is now, on
  `two_diff`, with a test. Two call sites bypass the gate entirely and lean on
  `expansion_sign` alone. The doc is correct; the call sites are what mislead.
  *(§21 closed the gate. The last two sentences are wrong and §21.3 measured
  them: no call site bypasses it.)*
- **`Approximation` magnitudes and `two_diff`'s "always exact"** were both stated
  more strongly than they held and are fixed, but neither was the load-bearing
  kind of wrong.

### 18.7 The refusal rate does not move, and I had written that it would

*Corrected 2026-08-31, after measuring.*

As first written this section was headed *"The refusal rate moves, and it moves
the right way"*, and said the change "certifies cases that previously came back
`Undecidable`, so that number improves". **Measured across `a23d73d`, it does
not move at all.**

| | before | after |
|---|---|---|
| Turned pairs, the headline family | 105 of 414 = **25.4%** | 105 of 414 = **25.4%** |
| Lattice, sharing coordinates, elsewhere (controls) | 0 | 0 |

Every refusal is `"side of plane"`, before and after; the catalogue table is
identical row for row; no operation that succeeded now refuses; no answer
changed value. The harness is `Tally` in
`crates/rcad-topo/tests/fuses_that_must_agree.rs`, and it counts §15.8's number
— per operation, `Err(Error::Undecidable)` only, under `Policy::certified()`.

**The change is real; the corpus cannot see it.** Verified rather than assumed:
copying `main`'s `when_it_cannot_tell.rs` into a worktree at the previous commit
fails four tests, so `a23d73d` genuinely certifies cases its predecessor
refused. None of them lie inside §1.7's corpus, which contains no coordinates
anywhere near `1e-170` or `1e200`. The corpus is built from shapes at ordinary
working sizes, which is what makes it a good corpus for its own purpose and
blind to this one.

**Two things worth carrying.** A delta of `0/414` bounds the corpus-level
improvement near zero; it does **not** show the change is inert, and those are
different statements. And §1.7's criterion 8 measures refusals over one corpus
at one range of magnitudes — a kernel-wide "refusal rate" it is not, and nothing
should quote it as one.

**Why this correction is in the record rather than a quiet edit.** The claim
that the number improves was written because it was obviously true — more cases
certify, so fewer refuse. It is obviously true and it is not what the number
does, because the number is over a corpus that never contained the cases. That
is §16.10's shape exactly: a locally valid argument answering a subtly different
question than the measurement asks. Written by the same head that wrote §16.10,
in the same session, one commit after using it to find two panics.

---

