## 82. A fuse that was never asked what it wrote, recorded 2026-09-09

§66.3 settled it in a paragraph: the four touched checks and `Assembly::commit`'s
post-write check run in every build. §4.1's middle row and §4.3's first bullet
were amended the same day to say so, and both still carry the amendment. The code
was never changed. The four `if !cfg!(debug_assertions) { return; }` guards stayed
exactly where §66.1 found them, and `commit`'s `was_sound` kept its
`cfg!(debug_assertions) &&`.

Two days later §78.10 walked into that same `was_sound` line from the other
direction, did not connect it to §66, and wrote the question down again as open
— "a real decision with a cost on both sides — a full `validate` per commit is
the quadratic sweep §4.3 exists to avoid". That reads §4.3 as it stood *before*
§66 amended it, and it re-opened a question that had been answered.

Both halves of that sentence needed work, and only one of them was simply wrong.
The question was not open. But the thing standing in the code really was the
quadratic sweep, so "§66.3 said run it, so run it" would have shipped a
whole-model `Model::validate` after every fuse — and this unit measured that
choice and did not make it. What went in is a **local** check over the entities
the fuse wrote, which is a different function from the one §66.3 named, arrived
at by measurement rather than by merging §66's own words.

> **Amended 2026-09-10, §87.** `Model::check_what_was_just_written` is deleted.
> §87 has the fuse build its answer in a throwaway model and sweep *that* with
> twelve of `Model::validate`'s thirteen checks, which left this function with
> one caller: its own unit test.
>
> **§82.4's argument is moot, not wrong.** It argued that eleven of the sweep's
> thirteen checks asked of the written entities ask the same questions as
> thirteen asked of the whole model, and that argument holds. A throwaway model
> holding only the answer simply *is* the set of entities §82.4 argued about, so
> a sweep of it asks those questions without anything being cut by hand — and
> `check_keys_resolve`, one of the two §82 cut, comes back for free (§87.6).
>
> **§82.5's objection to the coordinate range survives, and §87 nearly lost
> it.** §87's first cut swept the throwaway with the whole of `Model::validate`,
> reasoning that a fold over a model holding only the answer is not a fold over
> the caller's other solids. True, and beside the point: the fuse has already
> decided, in `the_inputs_fit_in_f64`, not to ask the answer's range when the
> caller's inputs were too wide to resolve, and the fold asks it anyway. Two
> passing fuses started refusing. §87.3.
>
> §82.7's measurements stand and are what §87.7 is quoted against. What §82
> chose and §87 changes is the *other* half — §82 put the check in and left the
> `expect` where it was, so a broken answer still crashed after being written.

### 82.1 What §66.3 decided, and what the code did instead

§66.3 is fourteen lines and it decides two things. The four touched checks lose
their `debug_assertions` gates, because they cost 50–65 ns a face of *one solid*
and the O(n²) §4.3 gated them for is the full sweep's, not theirs. And
`assemble.rs`'s post-write sweep runs in every build too, "one `Model::validate`
of a model the fuse has already spent tens of milliseconds on".

Neither happened. What a caller got instead is §66.1's answer, unchanged: in a
release build a broken invariant produced neither of §4.1's two responses — no
panic, no `Err`, and a solid that does not validate handed back as `Ok`.

The second gate was worse than a gate. `was_sound` was

    cfg!(debug_assertions) && model.validate(policy, &mut ProvenanceLog::new()).is_ok()

and the check below it ran only `if was_sound`. Read as a check that is what it
looks like; read as a **silencer** it is what it did. A model that did not
already validate — for any reason, including one this fuse had nothing to do
with — made `commit` skip its own post-check rather than complain about
anything. So the one case where a fuse is most likely to be handed something
odd is the case where nobody looked at what it wrote.

The doc comment above it explained, at length and correctly, *why the check
stands down when the model was not sound before the write*: a whole-model sweep
would report somebody else's broken solid as this fuse's bug. That reason is
real, it survives into what replaced it, and §82.5 is the test for it. What the
comment never said is that the whole check was debug-only as well.

### 82.2 Why the gate could never have seen it

`Cargo.toml` has a `[profile.test]` and it sets `opt-level = 2` and nothing
else, with a comment saying so deliberately: "`debug-assertions` and
`overflow-checks` are separate settings and stay on, so every `debug_assert!`
the kernel checks itself with still fires and no test verifies less than it
did." There is no `[profile.release]` section at all.

Which is exactly right for what that comment is about, and it means
`cargo nextest run` **never builds the code path that ships**. Every
`cfg!(debug_assertions)` in this crate reads `true` under the suite, in the
optimised profile as much as the unoptimised one. A check gated on it is
untested by construction, and so is its absence.

The demonstration, and it is the reason this subsection exists rather than
being an observation: the implementing lane deleted the fuse's self-check
**entirely** — not the gate, the whole `if was_sound { … }` block — and the gate
passed 1816 of 1816. That count is the suite as it stood before this unit's own
tests went in; the tree this section is recorded on runs 1820, all passing.
Nothing in the suite was holding that check in place. A
gate cannot notice a check it never compiles, and this is what §16.10's
sufficiency species looks like when the thing being trusted is the test suite
itself.

### 82.3 The two halves, as built

**The touched checks.** The four early returns in `crates/rcad-topo/src/model.rs`
are gone and `check_touched_solid`, `check_touched_face`, `check_touched_wire`
and `check_touched_edge` run in every build, which is §66.3's first half taken
literally because §66.2 had already priced it. Two unit tests in that file were
`#[cfg(debug_assertions)]` for the sole reason that the check they exercise was;
both lost the attribute and now run in whatever profile the suite is built in.

**The fuse's post-write check.** `was_sound` is gone, and with it both its gate
and its silence. In its place `Assembly::commit` calls a new
`Model::check_what_was_just_written`, in every build, and `expect`s it:

    model
        .check_what_was_just_written(&written, policy, &mut ProvenanceLog::new())
        .expect("a fuse writes a valid answer or does not write at all");

`Written` grew the three lists it did not carry — coedges, wires and shells —
because `write_into` was already minting them and only threw the slots away.

The `expect` is deliberate and it is the branch Mattias chose. §66.5's
`Error::TheKernelIsWrong` is not built here and §66.4 says why: `commit` writes
into the caller's arena before anything checks it, so an `Err` would hand back
an error *and* leave the orphans behind. Turning the panic into a return needs
the scratch model §66.7's first item is about, and that is still unpriced. This
unit made the check happen; it did not change what happens when it fails.

### 82.4 Why asking only the new entities asks the same questions

`Model::validate` runs thirteen checks. `check_what_was_just_written` runs
eleven of them, over the written entities and nothing else, and the argument
that this asks the same questions is structural rather than a hope.

`Assembly::write_into` builds **every** entity fresh — vertices, edges, coedges,
wires, faces, shells, solids — and links them by indices into its own local
vectors. Nothing new points at anything old, and nothing old points at anything
new. The written entities are therefore a piece of the arena on their own,
closed under "refers to".

That is what makes the sweep's *counting* loops local despite reading global.
"Is this face used by exactly one shell" is a count over every shell in the
model — but an untouched face's shells cannot have changed, and a new face can
only be named by a new shell. The same argument carries the orphan check, and
the used-once counts for coedges, wires, faces and shells alike.

**The two that are left out, and why each is left out for its own reason.**

`check_dynamic_range` is genuinely global and cannot be made local by this
argument: it folds the largest coordinate and the smallest feature over every
solid there is, so two solids that are each fine can be too far apart to compute
between. Asked after the write it fails a fuse for where somebody else's solid
sits, which is §82.5. The local form of the same question is asked, and asked
*before* the write, by `Assembly::check_dynamic_range` behind
`the_inputs_fit_in_f64` — the answer's own coordinates against the answer's own
shortest edge.

`check_keys_resolve` is left out because there is nothing here for it to find.
Every key these checks follow was minted by `write_into` microseconds earlier,
and where the local check reaches into an arena it indexes it directly, so a key
that did not resolve panics rather than coming back as `Invalid::DanglingKey`.
Under §4.1 that is the right response to this crate having broken its own arena,
and it is not a question a caller's input can raise.

### 82.5 The check that fired for somebody else's solid

The whole-model sweep at commit is not merely expensive. It can **fail for a
solid the fuse never touched**, and that is now a test in the tree rather than a
paragraph: `crates/rcad-topo/tests/a_fuse_is_not_blamed_for_a_solid_it_never_touched.rs`.

Three blocks, each 2 mm on a side. One sits at `1024`. Two sit at the origin,
the second offset so it overlaps the first in one corner by exactly `2^-30`, so
the union carries edges that short. Every number is exact, so
`Policy::certified()` decides everything and a refusal would be a bug rather
than the arithmetic giving up.

The arithmetic, against `DynamicRange::LARGEST_SUPPORTED_RATIO` of `1e11`:

| Asked of | Largest coordinate | Shortest feature | Ratio |
|---|---|---|---|
| The far block alone | 1026 | 2 | 513 |
| The answer alone | just under 4 | `2^-30` | 4.3e9 |
| The whole model | 1026 | `2^-30` | 1.1e12 |

So the fuse's own pre-write range check passes, correctly, and a whole-model
range check after the write does not. On the unchanged tree that is a panic:

    a fuse writes a valid model or does not write at all:
      Invalid(DynamicRangeTooWide { ratio: 1101659111424.0 })

for a step cut near the origin and a block a thousand millimetres away that this
fuse never looked at. `1026 × 2^30` is `1 101 659 111 424` exactly, which is the
ratio in the message — the `1026` and not `1024` because the far block spans
`1024` to `1026`.

**The test is not a tautology, and the file says how it avoids being one.** A
fuse that quietly failed to make a `2^-30` step would pass a test that only
asserted `Ok`. So it asserts instead that the model afterwards is one
`Model::validate` *refuses*, by name and with a ratio past the limit — the
whole-model check firing, which is precisely what the commit-time sweep would
have panicked on — with the fuse having answered and committed anyway. A second
test runs the same fuse with nothing else in the model and requires both the
answer and a full `validate` to pass, which is the control that says the refusal
above is the far block's doing and not the step's.

### 82.6 The touched checks were guarding something real

`model.rs` carries two tests named `a_mutation_notices_…`: one swaps a wire's
steps so the wire no longer closes, the other grows an edge a third coedge, and
both then call `place_solid` and expect a panic. Both were `#[cfg(debug_assertions)]`,
with a doc comment explaining that the check they exercise is not there in a
release profile.

With the gates still in place and the two attributes taken off,

    mise exec -- just t -p rcad-topo --cargo-profile release -E 'test(a_mutation_notices)'

fails with "test did not panic as expected". A release build walked straight
past a wire whose steps had been swapped, reached through the public
`place_solid`. That is the guard §66.1 said was missing, demonstrated on this
tree rather than argued.

The cost of putting it back was re-derived here at **48.0 ns per face** of the
solid handed in. §66.2 measured 50–65 ns on its own shapes and the committed
comment in `model.rs` still says 50–65; the implementing lane's own run gave
42–53. Three measurements of the same quantity, agreeing to within the spread,
and none of them large enough to argue about: `place_solid` on a 202-face solid
pays about ten microseconds.

### 82.7 What the sweep costs, and what went in instead

This is the half §66.3 got from a single workload, and the half where a
different workload gives a different answer.

**The sweep is linear in the whole model, not in the answer** — 1.94 µs a face
when this section was first written, and `commit` swept twice, once for
`was_sound` and once for the post-check. §66.2 measured 3.4–4.4 µs a face for
the same sweep on its own shapes. The two are not in conflict about the shape of
the cost, which is what matters: linear in the model, whatever the constant.

**Re-measured, because the two tables below did not fit each other.** The sweep
is public, so this needs no source edit at all: `Model::validate` over a model of
plain blocks, best of twenty runs after three warm ones, `--cargo-profile
release`, on models of 240, 492, 1452, 4332 and 5040 faces, giving 0.487, 0.991,
2.915, 8.661 and 10.049 ms. Least squares over those five: **1.99 µs a face,
with a fitted fixed cost of 13 µs**. Per-face it never leaves 1.99–2.03, so the
line goes through the origin for any purpose here — *there is no fixed
per-sweep overhead*, which matters because a fixed overhead is the only thing
that could have let a small model cost what the row below used to claim. The
same run under the default test profile gives 0.491 / 0.979 / 2.833 / 8.435 /
9.849 ms, within 3% of the release figures at every size. That the profile
barely moves the sweep is worth writing down where the profile is the whole
subject: `Cargo.toml` sets `opt-level = 2` under `[profile.test]` and carries no
`[profile.release]` at all, and the sweep has no `debug_assertions` gate left in
it — which is the difference that does remain between the two.

**Where the fuse built the model itself, §66.3's reading holds.** A 60-union
chain growing to 8046 faces came out *nominally faster* with the sweep on than
off — the difference is under a percent and inside the noise, and the reason it
can go either way is that the fuse's own cost grows as about O(n^1.7) against
the sweep's O(n). At that scale the sweep really is free, exactly as §66.3 says.

**Where the model holds geometry the fuse never touched, it does not.** The
fuse's own cost is then flat, because it is working on the same two small solids
however much else is lying about, and the sweep is not. Both halves re-measured
here, same profile and same machine as the rate above: a union of two blocks
overlapping by half, in a model grown around them, best of twelve rebuilds at
each size, comes to 1.435 ms at 12 faces and 1.452 ms at 5052 — **flat to about
1.2% over a 420× model**, which is what the local check bought. Against that
1.44 ms, two sweeps cost:

| Faces in the model | Two sweeps | Against the fuse's own flat 1.44 ms |
|---|---|---|
| 240 | 0.97 ms | 0.7× |
| 5040 | 20.10 ms | 14× |

**The 240 row used to say 1.3×, and that was wrong.** It is the one number in
this section no linear rate can produce: 1.3 × 1.34 ms is 1.74 ms, which would
have 240 faces sweeping for longer than the 492-face row of the before-and-after
table below, whose gap says the same two sweeps take 1.72 ms there. Twice the
model, the same cost, under a heading that says the sweep is linear in the whole
model. Only a fixed per-sweep overhead of
about 0.4 ms could hold both, and the fit above puts that overhead at 13 µs — so
the row went, not the arithmetic around it. What it should have said all along
is that at 240 faces the sweep is the same order as the fuse it follows, not a
multiple of it.

**Both columns of that table are ratios, and only the sweep half reproduces.** A
third machine re-measuring from this description alone got the sweep to within
3% — 0.490 ms at 240, 10.346 at 5040 — and the fuse at 0.816 ms, not 1.44. So
its own ratios are 1.2× and 25×, on either side of the two above. The sweep is
the reproducible half because the workload is stated exactly and blocks are
blocks; the fuse's cost depends on the pair being united, which "two blocks
overlapping by half" does not pin down. Read 0.7× and 14× as *this* machine's
ratios. What survives every run is the shape: the sweep grows with the model and
the fuse does not, so somewhere between 240 faces and 5040 the sweep passes the
fuse and keeps going.

The 5040 row survives, and the 15× it used to carry is not disturbed by the 14×
above: that is one machine's fuse against another's, 1.94 µs a face against a
1.34 ms fuse being 14.6× where 1.99 against 1.44 is 13.9× and 2.05 against 0.82
is 25×. §66.3 and `Model::check_what_was_just_written`'s doc comment both quote
15 times. 14.6 rounds to it, 13.9 rounds to 14 and 25 rounds to neither: the
figure is one machine's and the doc comment is the one that says whose, quoting
the 1.94 µs a face it came from. §66.3's bare "15 times the fuse (§82.7)" is the
loose one, and it is loose about the fuse and not about the sweep.

That is the case §66.2's headline figure does not cover, and it is why what went
in is `check_what_was_just_written` and not `Model::validate`. §66.3's phrase "a
model the fuse has already spent tens of milliseconds on" describes the first
workload and quietly assumes it. The corrected 240 row does not weaken that: a
sweep can be the price of one fuse and still be the wrong shape, because it is
the model and not the answer that it grows with, and the 5040 row is where that
shows.

**The cost, before and after**, per fuse of two fixed blocks in a model that
grows around them:

| Faces in the model | Per fuse, before | Per fuse, after |
|---|---|---|
| 492 | 3.79 ms | 2.07 ms |
| 1452 | 7.15 ms | 1.70 ms |
| 4332 | 18.07 ms | 1.72 ms |

Before, 4.8× over an 8.8× model — the fuse's cost was mostly not the fuse.
After, flat, which is what "local" is supposed to mean. Those are the gate
agent's numbers; the implementing lane's own run gave 2.68 / 6.37 / 17.50 before
and 1.71 / 1.65 / 1.86 after, the same shape and the same conclusion.

**That table checks out against the rate, and it cannot say how many times
`commit` swept.** Its own row gaps are the sweeping that went away: 1.72, 5.45
and 16.35 ms, which over two sweeps is 1.75, 1.88 and 1.89 µs a face — against
the 1.99 measured directly above, on a machine that is not the one those rows
came from. Read as *one* sweep the same gaps are 3.50, 3.75 and 3.77 µs a face,
and that reading is not absurd: §66.2 clocked this same sweep at **3.4–4.4 µs a
face** on its own shapes — this section's opening paragraph says so, and
`Model::check_what_was_just_written`'s doc comment repeats it — so a
once-sweeping `commit` on *that* machine lands inside the range *that* machine
published. The gaps fit
"twice" at this machine's rate and "once" at the other's, and a rate that varies
by two between machines is exactly what the pair of measurements shows. So the
gaps corroborate the rate and settle nothing about the count.

**The count is read off the source, and only off the source.** `git show
6b94c13^:crates/rcad-topo/src/fuse/assemble.rs` has `commit` calling
`Model::validate` twice — once into `was_sound` before the write, once after it,
guarded by that first result. This was a gate agent's correction: the paragraph
above it once claimed the arithmetic said "twice" on its own.

The comparison is not quite like for like and it is worth saying which way it
leans: "before" is the debug-gated code with its sweep forced on, so it includes
the `was_sound` sweep the release build never paid for. What the release build
paid for before this unit was nothing at all, because it looked at nothing.

### 82.8 The tolerance decision inside the decision

`check_edges` and `check_faces` are local walks, but their tolerances are not
local: `Model::allowed_wobble`, `Model::what_writing_a_plane_down_costs` and
`Model::what_writing_a_point_down_costs` all read `Model::largest_coordinate`,
an uncached fold over every vertex in the model, measured at 3.4 ns a vertex.

So a local check has a choice, and it is a choice about *answers* and not about
speed. Fold over the answer only and the tolerances get tighter, because the
answer's largest coordinate is no larger than the model's — which would make the
commit check **stricter** than `Model::validate`, and a fuse could then panic on
an answer the public sweep is perfectly happy with. Cheaper, and it changes what
is valid.

The other way was taken. The three helpers now take the largest coordinate as an
argument, and each caller folds once and hands it down. `check_what_was_just_written`
therefore judges a face by exactly the number the full sweep would judge it by:
**an answer it accepts is one `Model::validate` accepts, but for the coordinate
range.** That equality is the whole reason the local check can stand in for the
sweep, and it is worth more than the microseconds.

The full sweep got cheaper as a side effect, because it had been folding more
than once: `check_edges` went from two folds over every vertex to one, and
`check_faces` from three to one.

### 82.9 Three sentences that were false in every build

Two doc comments and one reachability argument, all three corrected in code by
this unit.

**`Model::fuse` promised a panic nothing raises.** Its `# Panics` said it panics
"if an input solid is broken in a way `Model::validate` rejects". Nothing has
ever asked that question — not in debug, not in release. What a broken input
actually does is trip an assert somewhere further in, under whatever name that
site gives it, which is a different promise and the one now written down.

**`was_sound` read as a check and behaved as a silencer**, which §82.1 has.

**The doc comment on `check_no_flat_face_is_used_reversed` claimed the
turned-round wall was reachable through the public API** — `validate.rs:669`
before this unit moved it, `:819` now — and that claim was doing real work: it
is what made the `AFlatFaceIsUsedReversed` check §16.11's kind rather than a
hardening exercise.
It is no longer true. `SolidBuilder::build` refuses that shape by name, and
`building_a_solid_out_of_faces.rs` has asserted so all along — the test was
there while the paragraph beside the check said the opposite. This is §16.10's
unreachability species with the sign flipped: a claim that something *is*
reachable, gone stale, and stale in the direction that makes an absence look
better justified than it is.

### 82.10 What this closes, what it does not, and what it opened

**Closed.** §66.3 is implemented on both halves, with the local check standing
in for the full sweep for the reason §82.7 measures. §78.10's second open item
is closed by this section, and the paragraph there is left standing the way
§78.10's first item's was, because what it got wrong is the useful part.

**Still open, unchanged.** §66.7's first item — `Assembly::commit` needs a
scratch model before the fuse's panic can become an `Err` — is not touched here.
Mattias chose the branch where a failure still panics, and the scratch-and-copy
is still unpriced. §66.7's other three items are untouched too: `place_solid`'s
rollback, whether "valid" should depend on a policy, and the +7.9% on
`add_extrusion` waiting for an importer to weigh it against.

**Opened, and it is a correctness hole with a demonstration.** `key.rs`'s
`ModelId` is `struct ModelId(u64)` fed by an atomic counter under
`#[cfg(debug_assertions)]`, and `struct ModelId;` — a zero-sized type, every
model carrying the same one — under `#[cfg(not(debug_assertions))]`. The comment
says it plainly: "the check costs nothing and proves nothing". So in a release
build a key from another live model resolves silently against whatever now sits
in that slot. Three lines through the public API: `large.solid(one)` returns the
wrong solid, and `large.fuse(&[one], …)` returns **`Ok`** with a union of two of
`large`'s own solids, volume 1500 where the caller asked about a 1 mm³ cube.

The reason it was a zero-sized type was cost, and the cost is not there. Making
the id real grows the arena by **exactly zero** — nothing stored in a `Model`
holds a key; every payload refers by slot — with peak RSS up 0.1% on a 231
195-entity STEP import and about 2% on the paths that mint keys.

It is with Mattias as its own unit, and the open question is not whether to fix
it. It is **how a release-only behaviour gets a test at all**, since §82.2 is the
answer for this one too: the gate runs the dev profile and cannot see it.

**Closed 2026-09-10 by §84.** `ModelId` is one `u64` in both builds now, and
the two tests that say so are in the release arm §83 built. The three
paragraphs above are left standing, because two of their numbers are the useful
part:

- **"volume 1500" is wrong.** The fuse returns `Ok` naming **no solids at
  all** — the model's own block fused against itself, the caller's cube never
  looked at, volume `-0`. §84.2 has the four measured lines. A reader hunting
  for a 1500 would have found nothing wrong and stopped.
- **"peak RSS up 0.1% … and about 2%" is an estimate that did not survive
  measuring.** Five release runs a side put the whole after-range for peak
  memory inside the before-range, and the control column that mints no key
  moved as far as the furthest of the ones that do (§84.4).

`large.solid(one)` returning the wrong solid is the line that was right, and it
is the one that mattered.

### 82.11 What this section re-derived, and what it took on a lane's word

§78.11's habit, kept, and §80.10's columns.

**Re-derived from this tree while writing this section.** The thirteen checks in
`Model::validate` and the eleven in `Model::check_what_was_just_written`, by
reading both bodies at `crates/rcad-topo/src/validate.rs:92` and the function
below it — the two left out are `check_keys_resolve` and `check_dynamic_range`,
which is what §82.4 says. `Cargo.toml`'s `[profile.test]` setting `opt-level = 2`
and nothing else, and the file having no `[profile.release]` at all. The four
early returns gone from `model.rs` and the two `#[cfg(debug_assertions)]`
attributes gone from the `a_mutation_notices_…` tests. `was_sound`'s exact old
text and the `expect` that replaced it. The three tolerance helpers taking
`largest_coordinate` as an argument, and `check_edges` folding once where it
folded twice. `ModelId`'s two definitions at `crates/rcad-topo/src/key.rs:20`
and `:38`.

**Re-derived as arithmetic.** §82.5's whole table. `1026 × 2^30 =
1 101 659 111 424`, which is the ratio in the panic message to the digit; `1026 /
2 = 513` for the far block alone; `4 × 2^30 ≈ 4.3e9` for the answer alone. The
test file asserts the shape of that refusal on every `just gate`, so the
committed part of it is not one lane's measurement on one day.

**Taken on a lane's or the gate agent's word, needing a build the suite does not
make or instrumentation that is not checked in:**

- The gate passing **1816/1816 with the fuse's self-check deleted entirely**
  (the implementing lane). This is the load-bearing one in §82.2 and it is a
  counterfactual build, so nothing in the tree can hold it.
- The **panic on the unchanged tree** in §82.5 — the exact message and the exact
  ratio, produced by reverting `assemble.rs` and `validate.rs` in a scratch copy
  and running the new test against them (the gate agent). Those are the two
  files and no others: `6b94c13` never touched `model.rs`, and reverting
  `assemble.rs` alone does not compile, because the new `validate.rs` reads the
  `coedges`, `wires` and `shells` that `Written` grew. Reproduced a second time
  on 2026-09-09. The ratio is re-derivable as arithmetic; that this is what the
  old code printed is not.
- **48.0 ns a face** for the touched checks (the gate agent), against 42–53 from
  the implementing lane and §66.2's 50–65. The `release`-profile run of the two
  un-gated `a_mutation_notices_…` tests failing with "test did not panic as
  expected" is likewise a build nothing keeps.
- The **60-union chain** to 8046 faces coming out nominally faster with the
  sweep on, and the fuse's own cost growing as about O(n^1.7).
- The **sweep's per-face rate** and the **untouched-geometry row**. Both were
  re-measured on a second machine on 2026-09-09 — 1.99 µs a face over five model
  sizes with a fitted fixed cost of 13 µs, and the fuse flat at 1.435–1.452 ms
  from 12 faces to 5052 — which is how the **1.3× at 240 faces was found to be
  wrong** and replaced by 0.7×. A third run, a gate agent's, rebuilt the harness
  from that description alone and got 0.490 / 1.014 / 2.999 / 8.852 / 10.346 ms
  over the same five sizes — 2.05 µs a face, fitted fixed cost 7 µs, per-face
  never leaving 2.04–2.07 — and the fuse flat at 0.816 ms on 12 faces against
  0.818 on 5052, 0.2% over the same 420× model. Three machines now agree that
  the rate is flat per face, that a fixed per-sweep cost is tens of microseconds
  and not the 0.4 ms the old 240 row needed, and that the fuse does not notice
  geometry it never touched. They do not agree on the fuse's absolute cost —
  1.34, 1.44 and 0.82 ms — so the 240 row's 0.7× is the ratio on one of the
  three and the 5040 row's 14× the ratio on that same one; the third machine's
  own numbers give 1.2× and 25×. It is still a timing harness nothing keeps: the
  workload is `Model::validate` and `Model::fuse` on plain blocks, which is
  cheap to write again, and §82.7 says exactly how — that description was enough
  to reproduce it, which is the thing being claimed.
- §82.7's **before-and-after table**, 3.79 / 7.15 / 18.07 → 2.07 / 1.70 / 1.72
  ms (the gate agent), with the implementing lane's 2.68 / 6.37 / 17.50 → 1.71 /
  1.65 / 1.86 beside it. Two runs on two machines agreeing about the shape is
  the claim; neither is a benchmark in the tree.
- **3.4 ns a vertex** for `Model::largest_coordinate`.
- Everything in §82.10's last part: the three-line release-build demonstration
  and its volume of 1500, the zero arena growth, the +0.1% peak RSS on a
  231 195-entity STEP import and the ~2% on key-making paths.

**What nobody established, so nothing above should be read as saying it.** No
measurement here was taken on a model whose untouched geometry is *curved* —
§82.7's flat-fuse rows are planar blocks, as §66.2's were. And the local
check's equality with the sweep in §82.8 is an argument from the tolerances
being the same numbers, plus §82.4's structural claim about `write_into`; it is
not a differential run of both checks over a corpus. Either would be worth
having if this is ever reopened.

---

