## 50. The endless bend — the unit after nothing round was ever checked, recorded 2026-09-04

*Written before any lane was dispatched. Every file and line it cites was checked
against the tree at `4b3ecda` by opening the file, not by trusting the report that
named it. Three citations changed when that was done and one of them changed the
scope of a lane, which is §48's preamble earning its place a second time.*

### 50.1 The name, and what the unit is

Called **the endless bend**, after the thing the kernel cannot do: put points in
order along a curve that has no ends. §47.7 named it, §48.7 held it out with the
reason *"it wants `arrange.rs`, which the claim audit is reading"*, and §48.10's
spike then passed on it. It has been the missing capability for three units and
it is now the spine of one.

**`the cut` stays held**, a third time. It has still never been earned.

The unit is two things that turn out to be one. Ordering along an endless bend is
the capability; **cone against pipe** — §48.7's *"this is the next unit's
headline"* — is the consumer that proves it, because a cone and a pipe on one
axis cross in exactly the kind of curve that has no ends. Shipping the ordering
without the consumer would be a `HowItRuns` variant nothing constructs.

### 50.2 Five corrections, before any lane reads them

**1. §49.3 describes the second latent defect as something the function does not
do.** It says `three_planes_faced_the_one_way` *"canonicalises to `-0.0` where its
sibling adds `+ 0.0`"*. Opening `arrange.rs:396-411`: the function does not
canonicalise to anything. It calls `flat.reversed()` when the first nonzero
coefficient is negative, and reversing a plane whose *other* coefficients are
`0.0` is what **produces** `-0.0`. The sibling is `exactly_at`
(`assemble.rs:1882-1889`), which maps `-0.0 → 0.0` with a comment saying why, and
the key that tells the two apart is `in_the_order_a_float_compares`
(`arrange.rs:419-422`), a `total_cmp`-equivalent ordering.

The defect is real and §49.3 was right that it is one. But a lane briefed to
"stop it canonicalising to `-0.0`" would look for a canonicalisation that is not
there. The fix is on the *producing* side or the *ordering* side, and which one is
the lane's to decide.

**2. `HowItRuns` has moved.** §48.6 cites `fuse/arrange.rs:448-467`; it is now
`arrange.rs:495-513`, with the same two variants. Ordinary drift, recorded because
§48.6 is where the third variant was decided and it is the paragraph a lane will
be sent to read.

**3. The §49.8 directive was priced as a lane and it is not one.** *"What else in
`arrange.rs` and `assemble.rs` decides a shape by comparing a sum of products to
zero"* sounds like a sweep of two 3 000-line files. Counted: **20 sites in
production code** — 12 in `arrange.rs` (`:358, 408, 1264, 1306, 1337, 1864, 1949,
3529, 3648`, plus `:419`), 8 in `assemble.rs` (`:248, 1122, 1129, 1135, 1666,
1826, 1887`) — and most already go through `Interval::sign()` or return a
`Decision` rather than comparing floats. The raw `== 0.0` sites are `assemble.rs`
`:1887`, `:248` and `:1122`. **This is a gate charge, not a lane**, and it is
moved to §50.7 accordingly. It also collided head-on with the ordering lane, which
writes the same file; making it a gate charge dissolves the collision rather than
scheduling around it, and buys independence — an audit of new code by the lane
that wrote it is what §49.7 predicts will go wrong.

**4. Lane A's scope is bounded by an arm in another crate, and the arm says so
itself.** `assemble.rs:2118-2123` returns `false` — *this crossing holds no
straight line* — for a cone against a ball, a pipe or another cone, and its
comment states the whole hazard:

> ***`false` rests on `intersect` and not on the geometry*** *… two cones that
> share an apex meet along rulings, which are lines. No such pair reaches here …
> so this arm is right for as long as that refusal stands, and is the arm to
> revisit the day it lifts.*

This unit lifts part of that refusal. **Coaxial cone against pipe is safe**: they
cross in circles, which hold no lines, so `false` stays right. **Cone against cone
is not**, and naming it would turn that arm into a wrong answer of exactly
§49.8's shape — an arm believed safe because a refusal upstream holds, failing
into an answer rather than a refusal when the refusal lifts. So Lane A names the
coaxial cone-and-pipe case **and nothing else**, and the day cone-and-cone is
named, `assemble.rs:2118` is the first thing that changes.

**5. §50.10's prediction fired inside this section, before a single lane ran.**
Pruning the lane worktrees turned up two branches that were never merged and
that looked like stranded fixes, and this section was about to carry both into
the wave:

- `m5-predicates-scaling` — *"meet_of_three_planes retries scaled per plane on
  refusal"*, which is the case CLAUDE.md §16.10 cites by name: *"true of
  orderings and silent about scalings, and a representable answer was refused
  for it."* It is **already in main, byte for byte**.
  `git diff m5-predicates-scaling:crates/rcad-predicates/src/planes.rs
  main:crates/rcad-predicates/src/planes.rs` is empty. `meet_after_rescaling_each_plane`,
  `meet_of_three_planes_at_this_scale` and `rescaled` all stand at
  `planes.rs:48-160` with the branch's own doc wording. The fix reached main by
  another route and the branch's merge was never recorded, so `git
  merge-base --is-ancestor` still calls it unmerged and it reads, at a glance,
  like work somebody dropped.
- `rcad-fuzz` — **not** superseded, and not whole either. Main fixed the same
  overflow by a different route (`power_of_two_that_keeps_the_squares_in_range`,
  `point.rs:367`) rather than the branch's divide-by-the-largest-component, so
  the *fix* may be dead while the *tests* are not: the branch's
  `a_vector_too_huge_to_square_is_normalized_by_dividing_first` is in no file in
  main, and `crates/rcad-geom/tests/soak/mod.rs` does not exist in main at all.

**The prediction was that the next unit would build something the tree already
has, somewhere else. It took the planning session, not a lane.** The
countermeasure named there — grep before writing, not after review — is what
found it, one turn after the recommendation to carry both was made. An unmerged
branch is not evidence of unlanded work; it is evidence of an unrecorded merge,
and the two look identical from `git branch --no-merged`.

### 50.3 The sentence the unit earns

> **The fuse can put points in order along a curve that never ends, and a cone
> coaxial with a pipe is cut by it.**

Falsifiable two ways: `HowItRuns` has a third variant with a construction site
that is not a test, and `intersect.rs:181` no longer refuses the coaxial pair.

### 50.4 The shape of the wave, and what each lane may write

**Spike first, alone.** It is short, and it settles two things at once: whether
coaxial really is one sign question, and what the predicate's signature is. That
signature goes in this section before A, B and C start, because a lane inventing
an interface another lane must guess is how two lanes become one (§48.4).

| step | one deliverable | may write |
|---|---|---|
| **S the spike** | is coaxial cone-against-pipe one sign question? | scratchpad only, never merged |
| **E the oracles** | `rcad-fuzz`'s order and scale oracles, re-applied (§50.9) | `tests/fuzzing/mod.rs`, `tests/every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`, `fuzz/fuzz_targets/where_two_surfaces_cross.rs` |
| **F the soak** | a shared soak module, the `checked_mul` bug, the missing knife-edge arm (§50.9) | `tests/soak/mod.rs`, `tests/how_far_the_marcher_strays_from_a_crossing_that_is_a_circle.rs`, `tests/every_point_along_a_crossing_is_on_both_surfaces.rs`, `tests/the_same_pair_turned_names_the_same_kind_of_crossing.rs`, `tests/how_often_naming_a_crossing_refuses.rs`, `rcad-topo/tests/turning_a_fuse.rs` |
| **A the crossing** | name the coaxial pair; split `intersect.rs:181`'s arm | `rcad-geom`: `intersect.rs`, `error.rs`, `walk.rs`, its own tests |
| **B the bend** | third `HowItRuns` variant, the ordering call sites, and the 84 of 84 | `rcad-topo/src/fuse/arrange.rs` |
| **C the consumer** | `settle_on_the_lower_rounding`'s superseded key; split the two frustum families' pin apart, and re-word `assemble.rs:2118`'s comment to cite the circles rather than the refusal (§50.11) | `rcad-topo/src/fuse/assemble.rs`, `rcad-topo/tests/how_often_a_fuse_with_a_cone_refuses.rs` |
| **D the negative zero** | the second latent defect, which spans both files | `arrange.rs` **and** `assemble.rs`, after B and C merge |

**A, B and C run concurrently. D is serialized behind both.** D is one defect
owning two small regions in two files that other lanes are writing; splitting it
across B and C would put the failing test — which CLAUDE.md rule 2 requires
*first* — in a place where neither half can make it fail for the right reason.

**C is thin without the consumer.** `settle_on_the_lower_rounding` is one private
function with one caller (`assemble.rs:1774`). Giving C the `assemble.rs` side of
A's work is what stops A shipping a predicate with nothing calling it, which §13
says should not be built at all.

### 50.5 The five rules whose pass and fail are written before the lane runs

§47.6 and §49.5 both record that the brief which said what counted as *failing*
produced better work than the brief that said what to build. Three units running.

- **S.** **Pass:** every coaxial pair in the draw decides under a certified
  policy, zero undecided, at §48.10's bar of a comparable draw. **Fail:** one
  undecided pair on a well-separated configuration, or forty minutes gone. A
  non-zero count is **not** a unit failure — it is the spike working, and cone
  against pipe converts from code to a written answer under §50.6's rule.
- **A.** **Fail:** the coaxial case is named but `intersect.rs:181` still raises
  `ACrossingBetweenAConeAndAnotherCurvedSurface` for it; or the *"not named at
  all"* claim at `intersect.rs:110-114` is left standing; or anything beyond the
  coaxial pair is named (§50.2 item 4); or `error.rs:331-335`'s *"one sign
  question apiece"* is left standing, which §50.11 showed is wrong twice over —
  there is no shape question, and there are two coaxiality signs, not one. **Also
  a fail: reporting this work as moving the import rate.** It moves it by zero
  files and §50.11 measured that before the lane ran.
- **B.** **Fail:** the frustum family answers fewer than **84 of 84**; or any new
  call site of the ordering predicate carries the spike's derivation instead of
  its own. §49.6 published 84 of 84 on a spike's word and §15.6 says a published
  bound needs a test.
- **C.** **Fail:** the `assemble.rs:2121` refusal still fires for a coaxial pair
  that A now names.
- **D.** **Fail:** either latent-defect test passes against the unfixed code.

**At forty-five minutes a lane commits what is done, writes what is left, and
hands back** (CLAUDE.md). The handback point is named per lane so that "what is
left" is not "a tree that does not compile": for **B** the variant and its match
arms land first and the 84 of 84 is a second commit; for **A** the dispatch arm
splits first and the shape discriminant is second.

### 50.6 What stays out

| out | why |
|---|---|
| **cone against cone, cone against ball** | §50.2 item 4. Naming either makes `assemble.rs:2118` a wrong answer |
| **torus against anything curved** | six pairs unnamed; untouched by this unit |
| **`cuts.rs`, most of `three_surfaces.rs`** | §49.6 records them **unaudited**, and they stay recorded that way. §50.7 charge 5 covers two files, not four |
| **`ResultIsNotManifold` with `count` other than 1** | 864 of 31 104 and counted by nothing. Unmeasured is not this unit's spine |
| **the writer, fillets, B-splines** | unchanged. Fillets stay gated on §44.7's reason |
| **connecting `BSplineSurface`** | built, tested, and reachable from nothing. Left deliberately: §13 wants a named consumer and §24.1 already sent the free-form marcher away. Recorded here so it is a decision and not an omission |

### 50.7 The gate's five charges

Named, because §49.5 found that briefs naming failure beat briefs naming work, and
a gate is a brief like any other.

1. **Re-derive 84 of 84** on the frustum family, independently of B's test.
2. **Score §49.7's prediction.** Name every call site the ordering predicate
   reached, and check each carries its own derivation and not the spike's.
3. **Run both latent-defect tests against the unfixed code** and confirm they
   fail. A test that passes without the fix proves nothing.
4. **Read the audit's negative claims for §16.10 shape.** An audit's output is
   mostly *"this one is fine"*, and a negative claim needs a demonstration and not
   a paragraph.
5. **The §49.8 sweep**, moved here from a lane by §50.2 item 3: the 20 sites, after
   A, B, C and D have landed, so that the sweep sees the new call sites. If the
   class is found present, say so about `cuts.rs` and `three_surfaces.rs` too —
   they become *more likely dirty than before*, not unchanged.

### 50.8 The machinery this unit changes before it dispatches

None of this is a deliverable. All of it is done before the spike runs, because
each one is cheap now and expensive during a wave.

- **Worktrees move out of the repo.** They are created at
  `.claude/worktrees/<name>` inside the tree; 38 of them held **166 GB**, and
  `.gitignore` did not cover them, so `git add -A` at the root would stage the
  lot. The cause is exact and not neglect: the harness sweep keeps any worktree
  with unpushed commits, **this repo has no remote**, so every lane branch was
  unpushed and the sweep kept all 38 for ever. They move to a sibling
  `rcad-worktrees/`, which is where the 14 hand-made ones already live.
  `.claude/worktrees/` is gitignored anyway, as insurance against the default
  coming back.
- **The repo gets a remote.** 67 000 lines of source and 51 000 of tests exist in
  one place on one disk. It is also what makes the sweep work as designed.
- **`just clean-targets` and `just prune-worktrees`, kept separate.** Stripping
  `target/` is always safe — kache refills it. Removing a worktree is safe only
  once the work is merged, which is why it is the gate's to run. Collapsing them
  makes the safe one inherit the unsafe one's ceremony.
- **kache is given room.** It has been the `RUSTC_WRAPPER` since `mise.toml:24`
  and it is running over budget: **71.4 GiB in a 50 GiB store, 143%**, hit rate
  57%, misses eating 93.7% of wrapper time. `doctor` passes every check — the
  budget is simply too small. Raised to 150 GiB out of the space the worktrees
  give back, then `gc`, then the daemon installed as a service rather than left
  on demand, because four concurrent lanes are exactly what an on-demand daemon
  has not been tested for.
- **Every lane brief carries `RCAD_STEP_CORPUS`.** Without it four `rcad-step`
  tests fail — by §44.10's decision, so that green means the corpus ran — and a
  lane that runs `just default` without the variable reads a red suite as its own
  doing. It is not in the tree (375 MB of third-party files) and it is not in
  `mise.toml` (a machine-specific absolute path), so the only place left is the
  brief. Four lanes hitting the same unexplained failure is four debugging
  sessions for a variable.
- **The heredoc ban is enforced by a hook rather than recorded again.** §49.5
  found three of thirteen lanes patching `.rs` files with heredocs or `sed -i`
  and chose to record it; recording has now been tried and changed nothing. The
  stated risk is that *a pattern that does not match fails quietly*, and quiet
  failure is what a gate reading a diff is worst at catching.

  What it costs, measured over M5: the lanes patched files with Python heredocs
  **233 times at ~620 tokens a call — 145k tokens, a sixth of everything the run
  generated**, and roughly **70 minutes** spent retyping code that `Edit` takes
  two strings for. The heredoc re-emits its `def patch(path, pairs)` scaffold on
  every call; `Edit` takes an old string and a new one. CLAUDE.md carries the
  rule and cites this paragraph rather than repeating the arithmetic, because the
  rule is what every agent needs in context and the arithmetic is what the record
  is for.

### 50.9 What the two unmerged branches were worth, audited rather than assumed

§50.2 item 5 settled `m5-predicates-scaling`: already in main, byte for byte.
`rcad-fuzz` needed reading, because main fixed the same overflow by a **different
route** and a diff cannot tell a superseded fix from a missing one. Merge base
`67a6b7f`; `rcad-geom` has grown ~17 300 lines since.

**Dead, and the reason matters more than the verdict.** The branch's headline —
*"an axis whose direction was too big to square stored no direction"* — is fixed
on main **twice over**, and the branch's own fix would be unreachable code there.
`normalized_as` measures at a rescaled copy (`point.rs:363`, via
`power_of_two_that_keeps_the_squares_in_range` at `:267`), so its proposed
`if !length.is_finite()` arm can never fire; and `Axis::new` (`point.rs:513`) no
longer calls the raw path at all. Main's version also fixes the **underflow** end
that the branch declined — its doc said *"a direction too small for it is not"* —
so main is strictly better. Both regression seeds are dead too: the defect they
pin is fixed (`half_space.rs:171-175` names this caller by name), and the seeds
are ChaCha byte-strings that no longer replay the recorded pairs after the
generator drifted. **Every number the branch published is stale**, by 5× to 40×,
because M4 and §33 moved the refusal rates underneath them.

**Live, and one of it is a bug in main.**

1. **The order and scale oracles.** `tests/fuzzing/mod.rs` on main exports exactly
   one oracle. There is **no commutativity oracle anywhere in `rcad-geom`** over
   the fuzz corpus — a swapped-argument `intersect` appears twice in the crate,
   both hand-written. The branch adds two: `intersect(a,b)` against
   `intersect(b,a)`, and the same pair written at `2^±1`. That closes a hole the
   branch states plainly — a pair the fuzzer *refuses* to name produces no point
   to check, so every defect that shows up as a refusal was outside what any
   amount of fuzzing could see. It is also the oracle that found the branch's own
   fix, so its catch rate is demonstrated rather than argued.
2. **`MEASURED_CASES * factor` is a bare multiply, and in release it wraps.**
   `how_often_naming_a_crossing_refuses.rs:619`, compounded twenty-fold at
   `:309`, and again at `rcad-topo/tests/turning_a_fuse.rs:318-325`. A soak that
   overflows runs **fewer** cases than shipped while the operator reads the
   number as the wide one — a green run that says nothing, which is §48.2's
   whole subject. `checked_mul` is the fix.
3. **Three files in `rcad-geom` cannot be soaked at all.**
   `how_far_the_marcher_strays…` (96 draws), `every_point_along_a_crossing…`
   (512), `the_same_pair_turned…` (2048) are hard-seeded and hard-sized on main,
   so soaking them reruns the same draws for ever. The branch's defect
   description is **verbatim still true**. Meanwhile `rcad-topo` has grown to
   *three* private copies of the same two variable names, so the argument for one
   shared module is stronger now than when it was written.
4. **`is_on_a_knife_edge` has no `[Sphere, Cylinder]` arm**
   (`the_same_pair_turned…:212`). A ball tangent along a whole circle inside a
   pipe falls through to `_ => false`, so that property reads as covering more
   than it does. Re-derive the case by hand; do not re-add the dead seed. The
   oracle lane deliberately omits a *turn* oracle citing exactly this gap, so
   closing it unblocks a third.

**Two lanes, not one, and they are split by write set rather than by theme.**
Items 2, 3 and 4 all touch the fixed-seed test files, so they are one lane; the
oracles are another. **Lane E may not write
`how_often_naming_a_crossing_refuses.rs`** — that file is Lane F's, and it is the
one file on the branch that conflicts hard, because main rewrote its whole
constants block for M4/§33. Take main's constants wholesale and redo the
private-helper deletion by hand.

**What Lane E must re-derive rather than inherit**, per §50.5's rule applied to a
branch instead of a spike: the branch's `scaled()` and `kind_of()` predate
`Surface::Cone` and `Surface::Torus`, so without new arms they will **silently
skip every cone and torus pair** — a green oracle over a shrinking family, which
is the §48.2 failure again. And its three published floors were measured against
the old surface set; they move.

### 50.10 The prediction

**§49.7's prediction is carried into this unit and scored by charge 2** — a
correct fix that carries a rounded test into a place where it decides an answer.
It is not restated as this unit's own; a prediction scored by the gate that
inherits it is the point.

**The prediction for the unit after this one comes from what this planning session
did wrong twice.** Both times, a plan was made about the tree without reading the
part of the tree that already answered it: a producer and its consumer were put in
concurrent lanes with the dependency unnoticed until a second pass, and a build
cache was proposed as future work while the same tool sat configured in
`mise.toml` and had since 2026-09-02. §49.8 closed on the same shape from the
other end — an expression in a third copy, guarded rather than removed.

> **The next unit will build something the tree already has, somewhere else.**
> Score it by asking, at the next gate, of every new function: was there already
> one, and what search would have found it? The countermeasure is the cheap one —
> grep before writing, not after review.

### 50.11 The spike passed, and it moved the work it was written for

**Pass. 80 000 pairs, 0 undecided, 0 disagreements**, over four draws of 20 000
with slopes and radii spanning `1e-4 … 1e4` each, so the circle standoff
`r/(s·√W)` spanned sixteen decades. Every decision came from level 2 or level 3.

**The claim it was sent to demonstrate is wrong, and in a useful direction.**
`error.rs:331-335` says the coaxial cases are *"one sign question apiece"*. There
is **no shape question at all**: a `Cone` is double-napped (`cone.rs:20-34`) and a
`Cylinder` has a positive radius, so `slope·|h| = ρ` has exactly two roots
`h = ±ρ/slope`. It is **always exactly two circles** — never zero, never one, and
the sphere-and-pipe row's `Pos/Zero/Neg` table has no analogue here. The two signs
that *are* asked decide whether the pair is **coaxial**, and there are two, not
one: `‖w_cone × w_pipe‖² = 0` and `‖(apex − o_pipe) × w_pipe‖² = 0`. Both are sums
of squares, so both answer only `Pos` or `Zero`, and both are shapes
`where_two_pipes_cross:853` already asks. **Nothing new needs deriving**, which is
§50.10's prediction pointing at a lane before the lane ran.

**§50.2 item 4 was right to flag the hazard and wrong that it applies here.**
`assemble.rs:2118` asks *does this crossing hold a straight line*, and **both**
answers to the coaxiality sign give the same answer to it — a quartic holds no
line, and two circles hold no line either. The arm stays `Some(false)`, now under
a proof about geometry instead of about `intersect`'s refusal. So **Lane C's
`assemble.rs` work is a comment, not a door**, and no `Policy`-free export is
needed at all — unlike §49.8's pair. Lane C is thin again and takes its weight
from the pin below instead.

**A limit, total and deferred.** 20 000 of 20 000 apexes built as `origin + t·ŵ`
on a **non-coordinate** axis land off the stored line and are refused. Worst miss
`2.6e-14` on coordinates of order 100 — one or two ulps — and the exact predicate
is *right*: those surfaces genuinely are not coaxial. A limit, not a gap (§16.10).

**Then the corpus was asked whether any of this is worth doing, and the first
answer was no.** Across all 367 files, `ACrossingBetweenAConeAndAnotherCurvedSurface`
is raised **zero times**, and naming the coaxial pair moves the import rate by
**exactly zero files**. Never blended, per §15.10: kicad holds a cone in 9 of 323
files and a coaxial cone-pipe neighbour in **6 of 323 (1.9%)**; nist holds one in
**32 of 44 (73%)**, with **1046 of 1162** adjacent cone-pipe edges exactly
coaxial. All 38 of those files refuse for something else — 26 of nist's 32 die in
the *reader*, on `B_SPLINE_CURVE_WITH_KNOTS` and `VERTEX_LOOP`, before geometry is
asked at all.

Two findings inside that measurement are worth keeping. **Every coaxial pair in
both corpora — 24 of 24 kicad, 2190 of 2190 nist — sits on a coordinate
direction**, which is the spike's limit confirmed from the other side and makes
the count robust. And **nothing anywhere shares an `AXIS2_PLACEMENT_3D` between a
cone and a cylinder** — zero pairs, both corpora — so the expected "they qualify
because the exporter reused a placement" is simply false. About 5% of nist's real
countersinks sit on leaning axes and stay refused even with Lane A.

**But the importer was the wrong place to look, and §48.7 had already said so.**
`crates/rcad-topo/tests/how_often_a_fuse_with_a_cone_refuses.rs:366` pins
`AFrustumAgainstABar` and `TwoFrustumsOnOneAxis` at `answered == 0`, with the
reason written into the assertion: *"a cone against another curved surface is a
degree-four space curve nothing here can name."* `a_frustum` builds on
`Cone::new(Axis::z(), slope)` (`tests/cones/mod.rs:49`) and the bar is translated
along `z` alone (`:207-213`), so **`AFrustumAgainstABar` is a coaxial cone and
pipe on a coordinate axis — precisely the case the spike decided 20 000 times out
of 20 000.**

- **`AFrustumAgainstABar` is Lane A's named consumer**, and §13 is satisfied by it
  rather than by any file.
- **`TwoFrustumsOnOneAxis` is not.** It is cone against cone (`:216-219`), which
  §50.2 item 4 keeps out of scope, and it stays pinned at zero.
- **The pin at `:366` has to move, and no lane owned that file.** Lane A landing
  makes that assertion *fail*, which is the correct behaviour of a pin and the
  reason it was written. `how_often_a_fuse_with_a_cone_refuses.rs` is added to
  **Lane C's** write set, and splitting the two families' expectations apart is
  what Lane C is for now that its door turned into a comment.

**So Lane A stands, with its claim rewritten and its consumer named.** What it
must not claim is an import-rate gain: there is none, and §50.5's failure
criterion for Lane A is extended to say so — a lane that reports this work as
moving the corpus has reported a number the measurement above already refutes.

