## 11. Deliberately deferred

| Thing | Until | Why |
|---|---|---|
| Bump arena in signatures | M2.5 | §1.4 — nothing to measure yet |
| Symbolic perturbation | Never, unless a caller asks | §1.5 — competes with `Undecidable` |
| `rkyv` zero-copy format | M2 | §6 — model still churning |
| `cargo-fuzz` | M2 | §6 — needs a format worth fuzzing |
| `rayon` inside the library | M2 | §7.3 — tessellation is the first real need |
| `ProvenanceSink` streaming | On demand | §5 — no caller yet |
| Branded lifetimes for cross-model keys | Indefinitely | Debug-checked model id is the pragmatic fix |
| A `&Policy` on `where_a_closed_crossing_starts` | A consumer that fuses tolerantly | §13 — the gap is real and nothing named is standing on it |
| `check-front-door` (a check that no test or bench under `crates/rcad` imports `rcad_geom` directly) | On demand | §13 rule 3 — the one violation found is already fixed by hand; nothing yet needs it enforced |

**Amended 2026-08-30.** §1.7 inserts M2.5, and the planar fuse is where scratch
allocation first becomes measurable — candidate pair lists, split pieces, image
maps, per-face working sets, all short-lived. The bump arena row therefore moves
from M4 to M2.5; §1.4's reason for deferring it, "nothing to measure yet",
expires there rather than at M4. `rkyv`, `cargo-fuzz` and `rayon` stay at M2:
all three are gated on tessellation and a settled format, and M2.5 changes
neither.

**Amended 2026-08-30, while planning M2.5.** The bump-arena row moves back out
to M4, and the paragraph above is wrong. It reasoned from the allocating end —
the fuse allocates, therefore the arena has arrived — and never named anything
inside M2.5 that would consume a `Bump` or any figure the arena would have to
beat. §1.7's eight exit criteria are what M2.5 is judged on and not one of them
mentions allocation, so the criteria win over the amended table. §13 rule 3 is
the general form: no named consumer inside the current milestone, no build, and
a viral `'bump` lifetime threaded through every fuse signature for a win nobody
has a target for is the largest possible version of the mistake that rule
exists to stop. M4 is where the consumer appears — a fuse over curved
intersections, run over the many-tool batches roadmap §7.2 is written for, is
the first scratch big enough for §1.4's "measure rather than assume" to come
back with a number instead of noise.

**Amended 2026-09-06: `where_a_closed_crossing_starts` keeps handing a certified
answer to a tolerant caller, and that is deferred rather than fixed.**

`rcad_geom::walk::where_a_closed_crossing_starts` is `pub`, is re-exported from
`rcad`, and takes no `&Policy`. It builds a `Policy::certified()` of its own and
asks the half-spaces under it, so a caller who asked for tolerance gets a
certified refusal on the tie and never learns its budget was not spent. The
function's own doc comment carries the reasoning and was narrowed twice on
2026-09-06 by rscene reading it back; this section records the decision, not a
second version of the argument, and where the two differ the doc comment is the
one that was checked.

**It is a gap and not a limit** (§16.10): the answer exists and the arithmetic
reaches it — the same half-spaces decide it under a tolerant policy when `walk`
hands one down — so nothing here is being called impossible. What is missing is a
policy argument.

**Why it is deferred, and this is §13 and not a claim that nobody can get
there.** Reachability was checked against the public API rather than against
today's callers, which is the check §16.10 asks for, and it came back *reachable*:
the function is re-exported and anyone may call it. Inside the kernel the only
callers are in `rcad-topo`'s `fuse::arrange`, so importing and meshing do not
reach it — `Import::read` builds and validates, and `SolidView::mesh` never asks
which piece of a crossing a run is. A backtrace census over the STEP corpus found
every undecided half-space arriving through `walk`, from `Model::validate`
checking which way a flat face's wire winds, and none through this door. **This
lane took no census and re-measured nothing**; the count and the paths were the
doc comment's, which published 31 undecided half-spaces, all through `walk`.
**§118.8 has since emptied that population** — the closed form for two equal
pipes on crossing axes took the whole family off the quartic route, so the
corpus now leaves nothing undecided anywhere and the census above is a fact
about a tree this one no longer is. The deferral's argument does not rest on
the number, and the residual below is why.

**The honest residual, which the deferral does not make go away.** A census is a
fact about the paths one corpus took, not about the door. An application that
fuses tolerantly *does* sit on this gap, and neither the kernel nor a drawing
layer can see that it does, because the fuse happens before the model reaches
either of them. A display layer's own policy is not evidence about its
application's: rscene takes `&Model` and never mutates one, so its tolerance is
spent on meshing and never on arranging a crossing, and that says nothing about
whatever built the model.

**What would make it worth building: a named consumer that fuses tolerantly.**
That is the trigger and there is no other. Until then the cost is not the policy
argument, it is that closing it changes a `pub` signature that `fuse::arrange`
and half a dozen test files call, for a caller nobody has named — which is
exactly the trade §13 exists to settle. The doc comment tells such a caller to
check itself against the door rather than against the census, and that sentence
is the deferral's mitigation.

**Amended 2026-09-07: a `check-front-door` recipe is deferred, not built, and
the gap it would close is real.** The rule is that a consumer of `rcad` goes
through the front door — no test or bench under `crates/rcad` should import
`rcad_geom`, or any crate underneath the facade, directly. One did:
`crates/rcad/tests/coordinates_that_span_too_wide_a_range.rs` reached past the
facade for `DynamicRange`, which the facade did not re-export. `5692a7f` added
the re-export and `50a5fb7` swapped the import for it. Measured before the
pair, `grep -rn "rcad_geom::" crates/rcad/tests crates/rcad/benches` gives 1
hit — that file, importing `DynamicRange`; measured at `50a5fb7`, 0.

**Nothing prevents it recurring.** No recipe in `justfile`, no lint, and no
test asserts that count stays at 0 — a future test file can add the same
import and `just default` will not see it. The lane that found and fixed the
gap declined to build the recipe, citing §13 rule 3: no named consumer beyond
the one violation it would have caught, no build. Mattias's answer, asked for
directly: defer the recipe, record the gap. So the gap is known, the check is
deliberately not built, and the answer to "can this recur" is yes.

---

