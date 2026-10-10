## 87. The fuse builds its answer twice, recorded 2026-09-10

§66.7's first item, closed. `Assembly::commit` builds the whole answer in a
throwaway `Model` of its own, sweeps that with **twelve of `Model::validate`'s
thirteen checks**, and writes into the caller's model only if the sweep is
clean. A broken answer is now `Error::TheFuseWroteABrokenAnswer` and the
caller's model is byte-identical to what it was.

The shape is `SolidBuilder::build`'s, which §66.4's table already named as the
one row that was right. Nothing here is new engineering; what took the work was
deciding which of the sweep's questions the throwaway should be allowed to ask,
and who is to blame for each answer.

**Twelve and not thirteen, and the first cut of this unit got that wrong.** It
swept the throwaway with the whole of `Model::validate`, which ends in a fold of
the coordinate range over everything in the model. Over a model holding only the
answer that fold *is* the answer's own range — which is exactly the question
`Assembly::check_dynamic_range` is kept behind a guard so as not to ask. §87.3
is that correction; it used to say the sweep asked the question and let the
answer through unwrapped, and it now says the sweep does not ask.

**It also deletes `Model::check_what_was_just_written`**, which §82 had built
five days earlier. That is not a reversal of §82 and §87.6 says why: §82 cut
eleven of the sweep's thirteen checks out by hand to make the sweep local to
what the fuse wrote, and a model holding only the answer is local for free.

### 87.1 What was wrong with the old shape

```rust
let written = self.write_into(model, &self.solids, &self.runs);
let worst = self.tag_everything_it_built(model, &written, inherited);
model.assert_approximation_did_not_grow(approximation_before.max(worst));

model
    .check_what_was_just_written(&written, policy, &mut ProvenanceLog::new())
    .expect("a fuse writes a valid answer or does not write at all");
```

The `expect` is the last line of the operation and the write is the first. A
fuse that built something broken had already put it in the caller's arena when
the panic fired, and §66.4 established that nothing in this crate can take a
write back. The panic was therefore the *only* thing standing between the caller
and a model that no longer validates — which is precisely §4.1's "crash >
corrupt" doing its job, and precisely the case §66.4 wanted a scratch model for
so that the crash would not be needed.

The second thing wrong with it was quieter: the sweep's records went into a
`ProvenanceLog::new()` that was dropped on the next line. `SolidBuilder::build`
drains its scratch sweep's records into the caller's log; `commit` threw them
away.

### 87.2 Thirteen decisions, in the order they bite

1. `commit` builds into `Model::new(model.scale())`, tags that throwaway, sweeps
   it with `Model::check_everything_but_the_coordinate_range`, and only then
   calls `write_into` a second time into the caller's model.
2. **Write twice.** Not a copy and not a graft. `Assembly::write_into` is a pure
   function of `&self`, the lumps and the runs: it reads nothing out of the model
   it writes into and names no entity that was already there. That is what makes
   a second call available, and it is the same reason `SolidBuilder::build`
   calls its own `write_into` twice rather than copying.
3. The throwaway is checked with **twelve of the sweep's thirteen** checks, not
   with §82's hand-cut local subset. It holds the answer and nothing else, so a
   whole-model sweep of it is already local — and it gets back the keys-resolve
   pass, which is one of the two the local one left out. The other, the
   coordinate range, is left out here too and for a different reason: §87.3.
   The check is **skipped, not filtered**. `Model::validate` returns on its first
   finding, so dropping a `DynamicRangeTooWide` out of what it returned would let
   a range failure mask a real one — the answer would come back `Ok` with a
   broken shape in it. `Model::check_everything_but_the_coordinate_range` is
   twelve calls that never reach the thirteenth.
4. `commit` returns `Result<Written>`. It no longer panics on a broken answer in
   **either** build. There is no `cfg!(debug_assertions)` split; §4.1's
   two-door wording was written when the check did not run in release at all.
5. The `Err` is a new `Error` variant, `TheFuseWroteABrokenAnswer`, wrapping the
   `Invalid` the sweep produced. `Error::Invalid` already means "your description
   was bad" where `SolidBuilder::build` returns it, and overloading it would make
   "we have a bug" and "you gave us a bad shape" the same value.
6. **The blame is blanket**: every `Invalid` the sweep finds is wrapped. It was
   not, in the first cut — `Assembly::whose_fault_a_swept_answer_is` let
   `Invalid::DynamicRangeTooWide` past as itself, because the sweep asking the
   range question meant a caller's own too-wide geometry could come back that
   way. With the question gone from the sweep there was nothing left to except,
   and the function is deleted. §87.3 walks the other twenty-four `Invalid`
   variants and says why none of them wants excepting.
7. `commit` takes a `&mut ProvenanceLog` so the throwaway sweep's records reach
   the caller's log, drained through a local log the way `SolidBuilder::build`
   drains `mine` into `log`.
8. After the real write, `check_touched_solid` runs on the solids just written.
   `SolidBuilder::build` does exactly this. It is not redundancy: nothing else
   looks at what the caller actually ends up with, and it is what guards the
   second write.
9. `assert_approximation_did_not_grow` stays where it is, after the real write,
   and stays a panic. It compares against the caller's model's worst
   approximation *before* the write, so a fresh throwaway has nothing honest to
   compare. §87.4 draws the line this leaves.
10. `Model::check_what_was_just_written` is deleted, with its two unit tests and
    `check_approximation_ordering_of` — §87.6.
11. Two new tests, both of which fail without the change — §87.5.
12. `a_fuse_is_not_blamed_for_a_solid_it_never_touched.rs` passes untouched. Its
    module doc now says the property holds *structurally* rather than naming the
    deleted function as what is asked instead.
13. `build.rs`'s five `add_*` builders keep their post-write panic. They are the
    same change and §66.4's "until" still applies to them; §87.8.

### 87.3 The one question the throwaway sweep must not ask

**The thirteenth check is the model-wide coordinate range, and the throwaway
sweep leaves it out.** `Model::check_everything_but_the_coordinate_range` is
`Model::validate`'s first twelve calls; `validate` is now that, and then
`check_dynamic_range`.

`Assembly::check_dynamic_range` asks the answer's own range before either write,
but only behind `if the_inputs_fit_in_f64(self, arguments, tools)?`
(`crates/rcad-topo/src/fuse/mod.rs`). **That guard is the whole point**: a fuse
is answerable for what it builds, not for what it was handed, so when the
caller's own solids are already further apart than `f64` can resolve, the
question is not asked at all and the fuse hands back the answer the arithmetic
reached. A whole-model fold over a model holding only the answer *is* the
answer's own range, so putting `Model::validate` on the throwaway put the
question straight back and made the guard do nothing.

**Two fuses stopped being answered, and this is how they read.** Both are tests
that pass on `main`, both fuse inputs that already fail `the_inputs_fit_in_f64`,
and the first cut of this unit refused both:

```
a_frame_a_long_way_from_the_origin_encloses_what_a_frame_encloses
  the model is broken: coordinates span a range of 1.099511627779e12,
  which is too wide to compute in
a_pair_of_towers_a_hundred_powers_of_ten_shorter_is_settled_and_measured
  two overlapping towers are a union f64 can prove:
  Invalid(DynamicRangeTooWide { ratio: 2e200 })
```

**Those are gaps and not limits**, which is the §16.10 distinction and the
reason this is a bug rather than a tightening. The answer exists in both cases
and the arithmetic reached it exactly: the second test's own doc calls its pair
one "which a certified fuse settles and used to refuse", and the first's says
that being outside `DynamicRange::LARGEST_SUPPORTED_RATIO` "is a separate
statement from the volume, which stays exact". A frame at `2^40` united with a
copy of itself is its own volume to the last bit; the range says the *model* is
uncomfortable to compute in, which is a true statement about where the caller
put their solids and not a reason to withhold a proved answer. Deferring them
would have undone deliberate earlier work — the guard was put there for exactly
these two.

Contrast the range question `Assembly::check_dynamic_range` *does* ask, which is
a limit and is documented as one: the sliver where two nearly-flush faces meet
is a feature neither input had, its ratio is a length over a length so no
scaling helps, and there is no answer to hand back.

**With the question gone, the blame is blanket.** Every `Invalid` the sweep
returns becomes `Error::TheFuseWroteABrokenAnswer`, and
`Assembly::whose_fault_a_swept_answer_is` is deleted. Walking the other
twenty-four `Invalid` variants for one that is the caller's fault rather than
this crate's finds none, and the reason is structural rather than a list: the
throwaway is `Model::new` and then one `Assembly::write_into`, so every entity
the twelve checks look at was put there by this crate on this call. A dangling
key, a non-finite corner, a wire that does not close, a shell enclosing negative
space, an approximation tag out of order — each names something `write_into` or
`tag_everything_it_built` wrote. `DynamicRangeTooWide` was the exception only
because it was never about an entity: it folded over the model as a whole, and
"as a whole" is where the caller's placement leaks in. Nothing else in the
sweep folds.

`Invalid::EdgeHasNoLength` is the one worth naming, because it is the closest
thing to a caller's-geometry finding left — two corners of the answer with
different names landing on one `f64` triple, which is what happens when the
inputs are far enough apart. It is wrapped, and continuity is one argument for
that — §82's hand-cut local check already ran `check_edges`, so this variant was
already the fuse's to answer for — but continuity would say the same of a
variant that ought to have moved, so it is not what decides.

**What decides it is §16.10's limit/gap distinction, and it puts
`EdgeHasNoLength` on the opposite side from `DynamicRangeTooWide`.** When two
named corners of the answer land on one `f64` triple there is **no answer to
hand back**: the edge between them is a feature of the answer that has no
length, the two names are genuinely one point in the only arithmetic there is,
and no rescaling of the inputs recovers the distinction because the corners are
the fuse's own output and not the caller's coordinates. That is a limit, so
refusing is right and the only arguable part is the label — whether "the fuse
wrote something broken" is a fair name for arithmetic that ran out. It is,
because the fuse is what named two corners it could not keep apart.
`DynamicRangeTooWide` is the exact opposite case: the answer existed, exactly,
and was being withheld over a statement about where the caller put their solids
(§16.10 calls that a gap, and the two tests above are what it cost). A gap
gets fixed; a limit gets refused and documented. This one is refused.

Refusals that are not about the model at all travel unchanged: `Error::Undecidable`
where a predicate gave up on the answer, because "the kernel wrote something
broken" is not what that says. `Error::Geometry` from a range that could not be
measured no longer comes out of the sweep at all — it was the range check's, and
the range check is not run there.

> **2026-10-10 note (Q1614): the range is asked before the shells are
> nested.** `Model::fused_under` (`fuse/mod.rs`) now asks
> `Assembly::check_dynamic_range` between `assemble`, which ends at
> `group_into_shells`, and `Assembly::nest_the_shells`, which it calls itself.
> The check is still behind the same `the_inputs_fit_in_f64` guard, and still
> asked in that one place. It was moved, not copied: a copy would log a second
> warning under a tolerant policy. The reason is that nesting measures each
> shell's volume. A sliver a rounding thin, whose `f64` corners all round onto
> one plane, measures as a shell enclosing nothing, or less than nothing, and
> was refused `Invalid::VolumeIsNotPositive` before the range was asked. That
> refusal hid the limit this section names behind one that reads as a
> mistake. Q1612's two skewed combs were the case
> (`two_skewed_combs_touching_along_one_line.rs`). Their certified
> intersection, a prism of `3.08e−32`, was refused `VolumeIsNotPositive {
> volume: 0.0 }`. It is now refused `DynamicRangeTooWide`, as their union and
> differences already were.
>
> The check now also runs ahead of the wobble check. A tolerant policy's
> warning is held in a log of its own and moved into the caller's only after
> the wobble check. So that check judges the same records as before, and the
> caller's log ends up with the same records, in the same order. Under a
> certified policy, a fuse the wobble check would refuse `Undecidable` whose
> answer is also too wide would now be refused for the range instead.
> Q1614's first relay ran every fusing crate's tests in the old order beside
> the new, with a toggle since taken out, and found no such fuse. Of the fuses
> those tests ask, 24 moved, all in `rcad-topo`, all certified, and all
> refused by nesting before. None moved in `rcad-step`, `rcad-fuzz-explain`,
> `rcad`, `rcad-mesh` or `rcad-fuzz-timing`. The pins that moved say so:
> the combs, the turned cube less the block (§417.3's note), both block edges
> grazing a ball, and two wedge differences of the narrow lune.
> `Assembly::commit` asserts that the shells were nested before it writes
> anything.

### 87.4 What is still a panic, and why the operation is not wholly atomic

Two things fire after the real write and both stay panics:

- `assert_approximation_did_not_grow`, for the reason in decision 9.
- `check_touched_solid` on each written solid.

Reaching either means the answer that swept clean did not survive being written
a second time — which is this crate corrupting its own arena, not a broken
answer. §4.1's middle row is the right response to that and always was.

So `Model::fuse`'s `# Atomic` promise holds up to those two and no further, and
`commit`'s `# Panics` says so in as many words. A doc comment that claimed
whole-operation atomicity would be the kind of sentence §16.10 exists to stop:
the claim is *nearly* true, and the honest form names the two cases where it is
not rather than rounding them off.

### 87.5 The tests, and what each demonstrates

`an_answer_whose_wire_does_not_close_is_refused_and_nothing_is_written` builds
an `Assembly` by hand — a triangle whose second face walks all three edges
forwards instead of backwards, so every edge is still used by two coedges
running opposite ways and the wire is the one thing wrong with it — hands it to
`commit`, and asks for both halves: the new `Err`, and a model whose seven
arenas are exactly the length they were.

`Assembly`, `commit` and `write_into` are all `pub(crate)`, so this has to be a
unit test inside `src/` rather than an integration test.

Its teeth were measured rather than assumed. Made to write before sweeping, it
reports the seven arena lengths as `[11, 15, 30, 8, 8, 2, 2]` against
`[8, 12, 24, 6, 6, 1, 1]` — the three corners, three edges, six coedges, two
wires, two faces, shell and solid of the broken answer, left behind in a model
that started as one plain cube.

**Two more tests, neither of them new, are what the §87.3 correction is checked
against**: `a_frame_a_long_way_from_the_origin_encloses_what_a_frame_encloses`
and `a_pair_of_towers_a_hundred_powers_of_ten_shorter_is_settled_and_measured`.
Passing *untouched* is the whole of what they demonstrate — they measure fuses
of inputs too wide for `f64`, they passed before §87, the first cut of §87
refused both, and the fix is judged by their going green again with not a line
edited. A test that has to be adjusted to accept a new refusal is not evidence
that the refusal was right.

`the_same_answer_is_written_into_an_empty_model_and_into_a_busy_one` is the
demonstration decision 2's claim needs. CLAUDE.md's §16.10 rule says a claim
that something cannot happen needs a demonstration and not a paragraph, and
"the second write writes the same answer as the first" is that kind of claim —
everything in §87 rests on it. One union of two cubes is committed into an empty
model and into the model that holds both inputs, and the two answers are
compared by position within `Written`, since the slots cannot match.

### 87.6 Why deleting §82's check is not undoing §82

§82.4's argument was that asking eleven of the sweep's thirteen checks of the
written entities asks the same questions as asking all thirteen of the whole
model, and it was a good argument. §87 does not contradict it. It makes it
**moot**: a throwaway model holding only the answer *is* the set of entities
§82.4 argued about, so a sweep of it asks exactly the questions the hand-cut
eleven asked, with no hand-cutting.

One of the two that were cut comes back for free and for the right reason:

- `check_keys_resolve` was cut because there was nothing in a just-written
  subgraph for it to find. That is still true, and running it costs a pass over
  arenas that hold only the answer.

`check_dynamic_range` is the one that does **not** come back, and the first cut
of this unit brought it back by mistake. §82.5 cut it because folding over every
solid in the model blames a fuse for where somebody else's solid sits, and it is
true that folded over a model holding only the answer it *is* the answer's own
range — but the answer's own range is a question the fuse has already decided
not to ask for these inputs. §87.3.

So the count is twelve, not eleven and not thirteen: §82's eleven, plus
`check_keys_resolve`.

What §82 built therefore had one caller left after this change — its own unit
test — which made it dead in everything that ships. `check_approximation_ordering_of`
went with it, and `Written` lost the three arenas nobody reads any more: a solid
names its shells, which name their faces, which name their wires, which name
their coedges, so listing them again was a second copy of what the arena says.

`Model::check_touched_solid` also moved **out** of `Assembly::write_into`, where
it had been running on each solid as it was built. Left there it put a panic in
front of all three of `write_into`'s calls — including the throwaway one whose
whole job is to turn a broken answer into an `Err`. `commit` asks it of the real
write instead, which is where it guards something.

### 87.7 What it costs, measured

**Re-measured twice: once after the twelve-check correction, and again after a
gate audit could not reproduce two of the numbers.** Release, best of 50 calls
after 10 warmups. The harness was a temporary `#[ignore]`d unit test in
`fuse/assemble.rs` that timed `Model::fuse` end to end on a freshly built model,
and separately timed `Model::new` + `write_into` + `tag_everything_it_built` +
`check_everything_but_the_coordinate_range` over an `Assembly` built once —
which is exactly the block `commit` added. It was deleted after the run. It was
run as:

```
mise exec -- just t --no-capture --release --run-ignored all \
    temporary_measurement_of_what_the_throwaway_costs
```

**What the shapes are, exactly, because the first cut did not say and a row did
not survive that.** Every solid is `Scale::millimetres(100.0)`, every fuse is
`Rule::Union` with one argument and the rest as tools:

- **Two cubes.** Two `Model::add_cube(2.0, …)`, so each runs from the origin to
  `(2, 2, 2)`; the second moved by `place_solid` with the translation
  `(1, 1, 1)`. Argument: the unmoved one. 12 faces.
- **Three cubes.** Three of those same cubes, the second moved by `(1, 1, 1)`
  and the third by `(2, 2, 2)`. Argument: the unmoved one; tools: the other two.
  18 faces.
- **Cube and bar.** One such cube, and `add_round_bar(0.5, 4.0, …)` moved to
  `(1, 1, -1)` so it stands right through the cube. Argument: the cube. 10
  faces, two of them curved.

Best of the four runs, all on one machine — an AMD Ryzen 9 3900X, the same one
`just gate` runs on here:

| Answer | Whole fuse | Throwaway build + tag + sweep | Per face |
|---|---|---|---|
| 12 flat faces (two cubes) | 1.42 ms | 34.2 µs, **+2.5%** | 2 849 ns |
| 18 flat faces (three cubes) | 2.94 ms | 55.0 µs, **+1.9%** | 3 055 ns |
| 10 faces, two of them curved (cube and bar) | 0.861 ms | 125.7 µs, **+17.1%** | 12 572 ns |

**The percentage is the block over the fuse *without* it — what this change
added — and not the block's share of the new total.** The "whole fuse" column
times the shipped code, so the block is inside it: `125.7 / (861 − 125.7)` is
17.1% where `125.7 / 861` is 14.6%. Both are arithmetic on the same two
timings and they answer different questions; a reader asking what §87 cost is
asking the first. The two flat rows cannot tell the conventions apart — 2.5%
against 2.4%, and 1.9% either way — so the curved row is the only place in this
table where the choice is visible, which is why it is easy to get wrong here and
why this paragraph exists. The `add_extrusion` figure below uses the same
convention: "six times its present cost" is measured against the cost before it,
and two conventions in one section would be worse than either.

**The block held to three figures over four runs; two of the millisecond figures
it is divided by did not.** The spread underneath: whole fuse 1.422–1.463 /
2.940–3.030 / 0.861–0.882 ms, block 34.2–35.7 / 55.0–58.2 / 125.7–127.9 µs.
Those spreads bound the three shares at 2.4–2.6%, 1.8–2.0% and 16.6–17.5% —
a bound over the worst pairing of the two columns, not four recomputed runs,
which is what figures reported as ranges will honestly carry.

**The 18-face row used to say 4.75 ms and a 1.2% share, and neither reproduces.**
An independent gate audit re-measuring from the old description alone got the
face count and the block right (56.4–61.0 µs) and the fuse at 2.84–3.01 ms — a
share near 2.0%. Re-run here with the arrangement pinned above it is 2.94 ms and
1.9%, which is the audit's answer and not the published one. The old row is
withdrawn: the only thing that could have made it 4.75 ms is a different
three-cube arrangement, and the section did not say which, so there is nothing
to defend. §82.7 is the precedent — a ratio belongs to a machine *and* to a
workload, and "three cubes, two moved" pins neither.

**The curved row's original 17.3% was right, and was briefly withdrawn as an
arithmetic slip.** `128.8 / 873` is 14.8%, which is not 17.3%, and on that
reading the row was corrected to a share of the new total. But
`128.8 / (873 − 128.8)` is 17.31%, so the old figure was never a slip: it was
this section's convention, stated nowhere, visible on no other row. It is
restored, with the paragraph above it saying which denominator it is. The
lesson is the one §82.7 keeps learning: a ratio needs its workload *and* its
two ends written down, and the row where a convention is invisible is the row
that will be corrected in the wrong direction.

**The per-face constant is about 3 µs on a flat answer and four times that on
a curved one.** The first cut's table had only flat answers in it and read the
agreement between two of them as the sweep being linear in faces full stop; the
third row says it is not. A curved solid's volume check meshes it
(`SolidView::mesh`, §66.3's "coarsest mesh there is"), and meshing is where the
time goes — 10 faces cost more than 18 flat ones. An answer's face count is not
enough to price this: its curvature is the other half.

Flat-answer numbers are a shade cheaper than the pre-correction table's, and one
check fewer is why: 34.2 µs against 45.1 µs on the same 12-face shape.

Two things this table does not say, and neither should be read into it. It is
**pure addition** — it does not net off `check_what_was_just_written`, which
this unit deleted and which asked eleven of the same questions of the same
entities. The real cost of §87 against the code it replaced is therefore smaller
than these numbers, and by an amount nobody has measured. And the percentages are
a share of *this* fuse, whose cost grows much faster than its answer does: the
18-face row's fuse is 2.1× the 12-face row's while its answer is 1.5×. Quote the
microseconds a face; the percentage is about which shape you picked.

**And this is what the same shape would cost `add_extrusion`**, which §66.7's
fourth item wanted a real figure for instead of §66.2's +7.9% proxy. Measured
the same way, from the same harness, with no change to `build.rs` at all: the
rehearsal a throwaway `add_extrusion` would add is `Model::new` + the build +
the twelve-check sweep, and all three are callable from outside.

**Both halves against the same starting condition, which is a fresh `Model`.**
That is the condition a rehearsal is stuck with — it makes its own throwaway
every call — so it is the one the operation has to be timed under too. The
profile is a 4 × 4 square from `ProfileBuilder`, extruded 4 with
`Isometry::identity()`, six faces:

| `add_extrusion`, a 4 × 4 × 4 box, 6 faces | Time |
|---|---|
| Into a fresh `Model`, one call each | **2.79–2.90 µs** |
| What a rehearsal adds — fresh `Model` + the same build + the twelve-check sweep | **13.8–14.2 µs** |
| Repeated into one `Model` whose arenas have already grown | 1.57–1.60 µs |

**About 14 µs added to a ~3 µs operation: the rehearsal costs roughly five times
the build it rehearses, and would leave `add_extrusion` at about six times its
present cost.** Read it that way and not as a percentage to two figures — the
denominator is the part that moves.

**The +640% this section used to publish came from the third row, and the third
row is not like for like.** 14.1 µs over 2.2 µs is +640%, but 2.2 µs is
`add_extrusion` into a model that has already grown, and a rehearsal never gets
one: it always starts from `Model::new`. An independent gate audit reproduced the
14 µs within 5% on five different harness designs and could not reproduce the
2.2 µs on any of them, getting 3.21–3.40 µs from a fresh model and 1.6–3.2 µs,
very noisy, from a grown one — four consecutive rounds inside one process gave
it +143%, +270%, +255% and +400%. Against a fresh model here it is +490%, and
the honest thing to say is the sentence above rather than any of those numbers.

The direction the proxy got wrong is unchanged, and it is the point: §66.2's
+7.9% was out by nearly two orders of magnitude. The reason is the one the fuse
rows already show from the other end — the sweep's cost is arithmetic over the
answer, and it does not care how cheap the operation that produced the answer
was. A fuse pays 34 µs against 1.42 ms; an extrusion pays 14 µs against 3 µs.
§66.4's "Panic, until it gets a scratch" rows for the five `add_*` builders now
have a number to weigh, and it is a much worse trade than the roadmap assumed.

Subtraction on the whole fuse was tried by the first cut and abandoned: two runs
of the same shipped code differed by 1.4 ms on a 23-face shape, which is fourteen
times the effect being measured. Timing the block directly, as here, is why the
shares in the first table hold to two figures where the raw milliseconds move by
a few per cent.

### 87.8 What this closes, and what it leaves

**Closed: §66.7's first item.** "Nobody has measured the copy" is no longer
true, and there is no copy — there are two writes. §66.4's costing sentence is
amended in place to say so.

**Not closed: the five `add_*` builders in `build.rs`.** They are the same
change and the same shape, and §66.4's rows still say "Panic, until it gets a
scratch". The reason they wait is now a measured **~14 µs added to a ~3 µs
operation** on `add_extrusion` — about six times its present cost — rather than
§66.2's +7.9% proxy; §87.7's second table, which closes §66.7's fourth item. A
fuse pays this cost against milliseconds and an extrusion pays it against
microseconds, and the gap is two orders of magnitude wider than the proxy said.
§13's rule stands: the consumer that would weigh that trade is the importer, and
it does not exist yet.

**Not closed: `place_solid`.** It writes in place and has nothing to write into
a throwaway, so §66.7's second item — whether it should roll back with the
inverse isometry — is untouched by this and stays open.

**Not closed: whether "valid" should depend on a policy at all.** §66.7's third
item. §87.3 makes it slightly sharper by settling that the fuse asks the range
question in exactly one place, under the caller's policy, but it does not answer
it.

