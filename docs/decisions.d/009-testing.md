## 9. Testing

### 9.1 What M0 ships

`proptest`, `cargo-nextest`, `criterion`, `cargo-deny`. `cargo-fuzz` deferred to
M2 (§6).

**Generators: extruded random simple polygons, with holes.** Not random convex
hulls — those are always valid and always convex, so they never exercise a
non-convex wire or an inner loop, which is where wire orientation invariants
actually bite. Convex hulls are tempting because generation is easy, and that ease
is exactly why they test nothing interesting.

Shrinking matters more than generation (roadmap §11.3). Budget for it.

The two tests that matter most in M0:

- `orient3d` differentially against `robust` on adversarial inputs — near-
  degenerate, collinear, wide dynamic range.
- **Transform invariance**: an arbitrary rotation must not change volume. §11.2
  says start here, and it costs nothing.

### 9.2 The parallel benchmark measures point classification

Planar volume by the divergence theorem is a handful of flops per face. Fanning
that across rayon measures the scheduler, then invites a false conclusion about the
data model.

The workload is **point-in-solid classification over a sampled grid**: real work
per query, exercises the predicate cascade under concurrency, and it is the
decomposition-invariant oracle §11.2 needs anyway — so none of it is throwaway
benchmark code.

Two pass criteria:

1. ≥0.8× linear scaling to physical core count.
2. **Bit-identical results to the serial run.** This is the one that will actually
   find a bug — it catches any accidental order-dependence in the cascade or in
   provenance recording.

### 9.3 An assert that would pass just as well if it watched nothing

*Recorded 2026-09-06. **This is rscene's finding**, offered for this record and
approved. Nothing in it was re-measured here and the counts are theirs.*

Core principle 3 says assert invariants. The defect that comes with it is an
assert whose condition is already forced true by something else in the program.
It never fires, it reads as a guard, and it is indistinguishable from a working
one until the thing it is supposed to watch becomes possible. rscene found seven
of these across the two trees in one day.

**How the worst two were found is the part worth keeping, because it was not
reading the tests.** Reading an assert and asking whether it can fail is the
obvious method and it is the one that missed them: the condition is short, it
looks right, and nothing on the line says what else already forces it. What
worked was to **build the state the fix would produce and run the whole suite
against it.** An assert that watches nothing behaves identically either way; one
that watches something does not. So the suite answers a question the line cannot
be read for.

What came back was worth more than the asserts. **Two tests were asserting that
the defect happens.** They are green today, they would have gone red on the fix,
and a reader arriving at that red would have read it as this kernel regressing.
Neither is visible from its own name, and neither would have turned up by reading
asserts, because neither is an assert.

The instrument generalises past asserts, and it is cheap: for any change that
narrows what is reachable, construct the narrowed state *first* and run the whole
suite, because the tests that go red are exactly the ones leaning on the old
width. That is a different question from "does the fix pass the suite", which is
asked after the fix exists and by then the two kinds of red look alike.

### 9.4 Five corrections in a day, and none of them from a side re-reading itself

Also rscene's, from the same day, and also not re-taken here.

**Every one of the five corrections made across rcad and rscene on 2026-09-06
came from one side checking the other side's claim. None came from either side
re-reading its own code.**

§51.10 named the mechanism — a claim gets fixed when it acquires a reader with a
reason to be adversarial about it — and offered announcing a dependency as the
cheap way to manufacture one. This is the census that says the mechanism was
doing all of the work that day, not some of it. It is the difference between
**measured twice by the same session** and **checked by anyone else**: the first
re-runs the same misreading and the second does not, and the record should keep
the two apart when it says how a number came to be trusted.

Five, in one day, over two trees, is not a law and this section does not dress it
as one. What it is enough for is the ordering: when a claim in this file is
load-bearing and cheap to hand to the other tree, hand it over rather than read
it again, because the second reading is the one with the measured record of
finding nothing.

---

