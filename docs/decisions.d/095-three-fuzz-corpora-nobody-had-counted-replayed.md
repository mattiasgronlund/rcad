## 95. Three fuzz corpora nobody had counted, replayed 2026-09-13

*§29.9 gave `where_two_surfaces_cross` a suite that replays its corpus on every
commit, and the other three targets never got one. Two of the three turn out to
have been nearly inert for months, and it took counting to see it.*

### 95.1 What the three corpora actually reach

Measured on this tree before anything was changed, by replaying each corpus
through its own target's harness and counting how far each seed got:

| corpus | seeds | decoded | operation reached | ok | refused | time |
|---|---|---|---|---|---|---|
| `no_step_file_makes_the_reader_panic` | 907 | 135 parse | **0 imports** | — | 135 | 15 ms |
| `no_model_makes_the_writer_panic` | 1180 | **16** models | 16 writes | — | — | 13 ms |
| `two_solids_fused_by_a_stranger` | 1740 | 910 of 3480 halves | 669 fuses | 255 | 414 | 0.79 s |

**The reader corpus never reached the importer.** All 135 seeds that parse fail
at the *first* line of `Import::read` — `what_the_file_declared`
(`crates/rcad-step/src/import.rs:73`) → `Refusal::TheFileDeclaresNoLengthUnit`.
Not one of 907 reaches
`nothing_is_placed_somewhere_this_reader_cannot_move_it`, the
`MANIFOLD_SOLID_BREP` walk, `solid_from`, or the `Model::validate` sweep the
harness ends with. The target's own doc comment said "**And the mapping is
fuzzed and not only the parse**", in the present tense, and over the checked-in
corpus that sentence was false.

It was false for a reason nothing was going to fix on its own. Byte mutation
was never going to build a `GEOMETRIC_REPRESENTATION_CONTEXT` with a
`LENGTH_UNIT` hanging off it out of nothing, and the corpus had no seed to
mutate *from*: the hand-written `a_small_whole_file` declares its `LENGTH_UNIT`
as a free-standing complex instance that no context references, so it refuses
too. A coverage-guided run keeps what reaches somewhere new, and there was
nowhere new to reach.

**The writer corpus decoded 16 models out of 1180, and all 16 were empty** — at
most one solid, no faces, no edges, no vertices. So `rcad_step::write` was
reached 16 times in 1180 and turned away every time at `TheModelHasNoSolid` or
`ASolidWithNoShell`, the two shallowest refusals it has, and
`every_number_the_door_promised_is_real` walked three empty iterators 1180
times over.

**The fuse corpus was the healthy one.** The one difference that is measured is
that the target splits its input in two and decodes a `Model` from each half, so
it gets two draws per seed — off the *same* `postcard` door that leaves the
writer's corpus at 16, so the door is not what separates them. Why two draws per
seed is worth 910 against 16 is not measured, and no paragraph here guesses at
it. Its models are real — 7 and 12 faces are the common shapes — and its
refusals are `ResultIsNotManifold` and `Undecidable` raised by *where three
surfaces meet*, *a point inside a piece of a face* and *where two planes cross*.

### 95.2 Feasibility was the question asked, and the answer was the whole of it

The proposal was to replay all three corpora, benchmark first, and split off
measured slow cases if the gate cost proved material. It does not: 0.8 s for
all three, against a suite where the `where_two_surfaces_cross` replay *alone*
was 63 s until §94. **The second half of the proposal is answered by the first
half's measurement**, and nothing is split.

The shape is §29.9's, copied exactly. Each target's pure harness moves to a
`tests/fuzzing/` module in the crate that owns the thing being fuzzed — reader
and writer in `rcad-step`, fuse in `rcad-topo` — the target reaches across with
`#[path]`, and the replay is an ordinary integration test beside it. A property
added to one of those files is explored by `cargo fuzz` and replayed on every
commit without either side being edited.

**One thing went the other way and shrank.** Both the writer's harness and the
fuse's carried their own copy of `every_number_the_door_promised_is_real`, the
§44.5 check on what `Deserialize for Model` promised. There is now one copy, in
`crates/rcad-topo/tests/fuzzing/the_door.rs` — `rcad-topo` owns the door — and
both harnesses reach it by `#[path]`. Two places to update when the door grows
a fourth promise is one place to forget, which is the argument CLAUDE.md makes
about the plugins and it applies inside a repo too.

**Each stage gets its own count, not one total.** A byte string that does not
parse, a model that does not validate and a fuse that refuses fail for entirely
different reasons, and a single number lets the deep stages fall to nothing
behind the shallow ones. That is §29.9's argument for three oracle counts
rather than one, and the reader corpus is the case that proves it: 135 of 907
parsed, which looks like a working corpus, and **none at all** imported.

### 95.3 Six control seeds, one either side of each operation

A floor of 0 asserts nothing, and a floor met by 16 empty models asserts almost
nothing. So the two inert corpora are repaired in the same change, and each of
the three gets a named seed either side of its operation:

| corpus | reaches it | is refused by it |
|---|---|---|
| reader | `a_block_that_imports` | `a_block_with_no_solid_in_it` |
| writer | `two_cubes_that_write` | `a_ball_whose_pole_the_writer_refuses` |
| fuse | `two_cubes_that_fuse` | `two_cubes_touching_along_one_edge` |

`a_block_that_imports` is `written_by_hand::a_block([2.0, 4.0, 8.0], ".MILLI.")`,
the same block three other test files measure, and it is the corpus's only seed
that reaches `solid_from` and the sweep. `a_block_with_no_solid_in_it` is that
file with its one `MANIFOLD_SOLID_BREP` line deleted, so it gets past the unit
and the placements and is then refused `TheFileHoldsNoSolid` — a refusal
*inside* the mapping, where the 135 that were already there all stop at the
door.

`two_cubes_that_write` is two cubes of side 2, at the origin and at `(1, 1, 1)`,
which writes 10 435 bytes of Part 21. `a_ball_whose_pole_the_writer_refuses` is
one ball of radius 2, refused `AWireThatStandsStillAtAPole` — a named gap deep
inside `solid_from_shell`, past where that corpus had ever been.

The two fuse seeds are `[header] ++ bytes ++ bytes`, so both halves decode to
the same model and the second header is the first rotated by four. Header `0x01`
intersects the overlapping pair and comes back with a model; header `0x02` on
two cubes meeting along one edge is refused `ResultIsNotManifold { count: 4 }`.

**Six became eight the same day**, when §95.4 moved the rule off the split and
added two more fuse seeds: one to hold up the thinnest floor that move creates,
and one to keep the single branch it would otherwise have dropped. Header `0x01`
draws a union on its first half now rather than an intersection on both.

**Four of the six are `postcard` bytes and stop decoding if `Model`'s
serialized shape changes.** That is the alarm working rather than a fragility: a
change to the door that silently reinterprets a stranger's saved file is
precisely what §44.5 is about, and the floors go red for it.

**The seeds are worth more to the fuzzer than to the replay.** An overnight run
now has something to mutate from on the reader and the writer, which is the half
of the problem no test could fix.

### 95.4 The split and the rule were the same number

Found while building the fuse control seeds, and it was a defect in the target
rather than in the kernel. `how_one_model_is_fused` read
`header % (solids.len() + 1)` for the split and `header % 3` for the rule. For a
model with **two** solids those are the same expression, so of the nine (split,
rule) pairs only three could be drawn:

| header % 3 | split | rule |
|---|---|---|
| 0 | `arguments` empty | `Union` |
| 1 | one against one | `Intersection` |
| 2 | `tools` empty | `Difference` |

**That a two-sided union and a two-sided difference could not be drawn at all is
a demonstration and not a paragraph** (§16.10). Every fused half of the corpus
was labelled with the `(sidedness, rule, outcome)` triple it landed on, `outcome`
being `Ok` or the name of the refusal variant. The corpus reached **23 distinct
triples, and not one of them was a two-sided union or a two-sided difference.**

It is fixed now: the rule comes off `(header >> 4) % 3`. The rest of this section
is the measurement that chose that over the obvious fix, because the obvious fix
is the wrong one.

#### A second header byte loses most of the corpus

Measured over all 1749 seeds, replaying each under each candidate:

| layout | halves | decoded | validated | fused | one-sided | ok | refused |
|---|---|---|---|---|---|---|---|
| `header % 3`, as it stood | 3498 | 922 | 678 | 678 | 569 | 260 | 418 |
| a rule byte in front of the halves | 3496 | 392 | 336 | 336 | 303 | 149 | 187 |
| a rule byte off the end of the input | 3496 | 211 | 80 | 80 | 70 | 26 | 54 |
| `(header >> 4) % 3`, same byte | 3498 | 922 | 678 | 678 | 569 | 262 | 416 |

**Both byte layouts lose most of the corpus, and taking the byte off the end —
which looked like the cheap option — is the worse of the two**: 678 fuses down to
80. A minimised seed is byte-tight, because the fuzzer shrank it until every byte
mattered, so shifting a half along by one byte or truncating it by one stops it
decoding. And the end is not a free place to take a byte from: the input is cut
at `rest.len() / 2`, so dropping the last byte moves that cut on every seed of
odd length as well as truncating the second half of every seed.

This is §29's answer reached from the other side. There the repack was declined
because 92 of 830 seeds would have decoded to a different *pair*; here the draw
survives and the *model* does not.

#### Which bits the rule comes off

Every payload-preserving choice leaves `fused` at 678 — the split is untouched,
so no seed loses its model whichever bits the rule reads. What separates them is
where the nine (split, rule) cells land, over the 612 halves holding two solids:

| rule read from | two-sided union / intersection / difference | smallest of the nine | triples gained | lost |
|---|---|---|---|---|
| `header % 3` | 0 / 109 / 0 | 0 | — | — |
| `(header >> 2) % 3` | 37 / 66 / 6 | 1 | 8 | 3 |
| `(header >> 3) % 3` | 92 / 10 / 7 | 1 | 11 | 6 |
| `(header >> 4) % 3` | 68 / 36 / 5 | 2 | 9 | 4 |
| `(header >> 5) % 3` | 14 / 67 / 28 | 0 | 11 | 6 |
| `(header >> 6) % 3` | 13 / 30 / 66 | 1 | 8 | 3 |

`(header >> 6) % 3` is disqualified outright, and not on a count: bit 7 is the
policy bit, so rule and policy would share two bits and **a tolerant
intersection could never be drawn**. That trades one coupling for another.

That leaves `>> 2` and `>> 4`, which net the same five triples. `>> 4` is taken
for three reasons. It gains **eight** two-sided triples where `>> 2` gains four,
and a two-sided draw is the scarce kind — 109 of 678. The smallest of its nine
cells is 2 rather than 1, so a floor can rest on it. And it is the nibble
`rotate_left(4)` already swaps, so the two halves read each other's nibble for
the rule and the layout takes one sentence to explain.

#### What it costs, and the seed that pays it back

Nine triples gained, four lost. The gains are the whole point: two-sided
difference reaching `Ok`, `Invalid`, `ResultIsNotManifold` and `Undecidable`, and
two-sided union reaching those three plus
`CannotTellTheCellsOfAFaceApart` — cases nothing in the corpus could draw before.

Three of the four losses are a pairing rather than a branch: the refusal variant
is still reached, under a different sidedness or rule. **One is a real loss.**
`CurvedFaceWithAHoleGoesTooFarRound` was reached by exactly one half in the
corpus — seed `4217bc51fd9dd4955482c024121a1b0b54b50762`, half 1, header `0xf4`,
a tolerant two-sided intersection — and `0xf4`'s high nibble draws a union
instead. Header `0xa0` draws the same tolerant two-sided intersection on the same
model and reaches the same refusal, so the branch is kept by name:
`a_curved_face_with_a_hole_that_goes_too_far_round`. Losing a branch a night's
fuzzing found, to a layout change, is the cost §29 declined its repack over; here
it is one seed to avoid.

`two_cubes_one_taken_out_of_the_other`, header `0x22` — its own nibble swap, so
both halves draw the same — is the second seed added, and it holds up the
thinnest floor in the replay: two-sided differences number 7, and 5 of those are
incidental.

That makes **eight** control seeds where §95.3 had six.

#### The edges the kernel actually runs, counted

§29 declined its repack partly because **it did not buy the coverage it was for**,
and that is a claim this one has to answer in the same currency. Replaying the
same 1751 files through the real `cargo-fuzz` binary with `-runs=0`:

| layout | edges (`cov`) | features (`ft`) |
|---|---|---|
| `header % 3` | 14 765, 14 766 | 69 734, 69 750 |
| `(header >> 4) % 3` | 14 788, 14 789, 14 793 | 69 253, 69 273, 69 277 |

Repeated because the numbers are not deterministic — the ranges are two and three
runs of the same binary. They do not overlap: **the same seeds reach about 25 more
edges of the kernel** under the new layout.

**The feature count falls by about 480, and that is not a contradiction.**
libFuzzer's features include hit-count buckets as well as edges, so a corpus that
stops running the same one-sided difference 282 times loses buckets while reaching
more code. `cov` is the number that answers "what did the kernel run", and it went
up.

#### `one_sided` did not move, and it is a different defect

The 569 is not the coupling, and the assertion that named it was wrong about what
it measured. It is `header % (solids.len() + 1)` having two degenerate outcomes
of three for a two-solid model, plus 66 one-solid halves that have no two-sided
draw at all. Moving the rule left it at exactly 569.

Biasing the split toward the interior was measured as well, off the low nibble so
as not to touch the bits the rule now reads:

| split | one-sided | two-sided | distinct triples | gained | lost | halves whose operation moves |
|---|---|---|---|---|---|---|
| `header % (n + 1)`, as it stands | 569 | 109 | 28 | 9 | 4 | 269 |
| never one-sided when `n >= 2` | 66 | 612 | 23 | 16 | 16 | 599 |
| interior half the time | 456 | 222 | 28 | 9 | 4 | 378 |

**Neither buys a branch.** Always-interior trades the entire one-sided union
family for the two-sided one and lands back on 23 triples — sixteen gained
against sixteen lost, for 599 halves reinterpreted. Interior-half-the-time
reaches exactly the same 28 triples as leaving the split alone, for 109 more
halves reinterpreted than the rule move alone costs. So the split is left alone,
on the measurement rather than on an argument.

`every_model_the_fuse_fuzzer_kept_is_fused_again.rs` still asserts the 569 *does
not drop*, written that way round on purpose, but it now says the split's modulus
rather than "the shared modulus": a drop means somebody changed the split and
this section is out of date. `two_sided_unions >= 50` and
`two_sided_differences >= 5` are the two assertions that hold the fix down, and
both were flatly zero before it.

### 95.5 The numbers

With the six seeds in, measured by the three replay tests:

| corpus | seeds | reached |
|---|---|---|
| reader | 909 | 137 parse, 547 instances, 1 imports, 136 refuse |
| writer | 1182 | 18 decode, 13 faces, 1 writes, 17 refuse |
| fuse | 1751 | 3502 halves, 926 decode, 682 validate, 682 fuse, 265 ok, 417 refuse |

**The fuse row includes the seven crash seeds §92 rescued**, which landed on
`main` while this was being written. Replaying them moves halves 3484 → 3498,
fuses 673 → 678, `ok` 257 → 260 and refusals 416 → 418, for no measurable time.
That is the arrangement paying for itself on its first day: seven seeds a night
found are now five more fuses a commit can break.

**The fuse row is §95.4's too**, which is why it reads 1751 and not 1749: the two
seeds it added bring four halves, and the rule coming off the high nibble moves
`ok` and refusals without moving a single decode. It also carries
`two_sided_unions: 69` and `two_sided_differences: 7`, the two counters that were
zero for every seed until that move.

22 ms, 19 ms and 0.75 s run on their own. Every floor is set at roughly three
quarters of what was measured, which is §29.9's rule and catches a stage falling
silent without failing on ordinary drift. The exceptions are the two counts that
come from one seed each — the reader's `imported` and the writer's `written` —
whose floor is 1, and the control seed is what holds it up.

**What they cost the gate is nothing measurable, and that is a stronger claim
than the sum.** `just gate` is green at **1863 tests**, six of them these, and
the six take 1.65 s of the run between them under full parallel load — the fuse
one 1.5 s of that, against 0.74 s alone, which is contention and not the test.
The gate's wall time is set by its slowest single test at **30.7 s**, so none of
the six is anywhere near the critical path. Do not read a wall-time total as the
cost of adding a test to a suite that runs in parallel; read the critical path.

All four targets build on nightly after the move, and all three corpora replay
clean through the real `cargo fuzz` binaries with `-runs=0`: 910, 1183 and 1743
runs, no crash.

### 95.6 The fuse target asks the answer three questions, not none

§95 stopped at "does it crash", which is what a corpus nobody had counted could
be asked for. With the counting done, the fuse target is where three promises
already written down can be asked continuously, and `let _ =` on the answer was
throwing all three away.

| checked | promised by | tested before this |
|---|---|---|
| a refusal leaves the model byte-identical | §17.4's atomicity row, and `Model::fuse`'s "Atomic" heading | **nothing** |
| every solid handed back resolves in the model that minted it | §17.4's history-map row — "deleted" is an input with no image, not a key that stopped resolving | nothing directly |
| an `Ok` answer passes the sweep under its own policy | `Model::fuse`, "What `Ok` promises, and under which policy" | §66.1, once, by a harness built for the day |

**The atomicity row is the one that had nothing.** Grepping the tree for it
finds `fusing_two_bars_side_by_side.rs`, which calls `Model::validate` after a
refusal — so it would pass a model that had been half written and was still
sound, which is exactly the half-applied boolean §4.1 would rather crash than
emit. The bytes are compared through `postcard`, the same door the seeds came
in through, so what is compared is what a saved file would hold rather than the
arena's own bookkeeping.

**The third is §66.1 stopping being a one-off.** That harness was the fuzz
target's own decode and split with the answer kept instead of dropped, run three
times by hand against `a6f03b8`; it is now the target itself, on every commit.
It carries none of §66.1's sixteen forward: those failed a *certified* sweep
after a *tolerant* fuse, which §68 and `Model::fuse` both say is correct, and
the sweep here is under the policy the fuse ran under — the 0 of §66.1's middle
row, made permanent.

#### The `Ok` sweep leans on the certified sweep above it, and that is not obvious

`Model::fuse` answering `Ok` does **not** mean the caller's model validates.
§87.3 leaves that open deliberately: inputs that fail `the_inputs_fit_in_f64`
stand `Assembly::check_dynamic_range` down, and `Model::validate`'s thirteenth
check — the coordinate range, the only one that folds over the whole model —
then refuses an answer the arithmetic reached exactly.
`a_frame_a_long_way_from_the_origin_encloses_what_a_frame_encloses` is such a
fuse and it passes on `main`. So the assertion added here is **false of
`Model::fuse` in general** and true of every fuse this harness can draw.

What makes it true here is the `Policy::certified()` sweep the harness runs
before fusing. `the_inputs_fit_in_f64` folds over the input solids alone and
takes chords alone, so its ratio is bounded by the whole-model ratio that sweep
has already accepted: no coordinate it sees is larger, no edge it sees is
shorter. A model that reaches the fuse fits, the range check runs, and a fuse
that would have needed the carve-out refuses instead of answering.

That sweep was put there to keep the target off `Model::fuse`'s documented panic
on unsound input. It is now load-bearing twice, and moving it to the fuse's own
policy — which reads like a tightening — would make the assertion fire on good
answers. The harness says so where the sweep is.

#### A check nothing reaches passes for free, so two counters say what was reached

`solids_made: 266` across 265 answers is how many keys the resolve check looked
at; 265 empty answers would have left it at zero and the check asserting
nothing. `ok_that_wrote_something: 249` is how many `Ok` fuses moved the bytes
— the control on the refusal side's comparison, because serialization blind to
what a fuse writes would let that comparison pass on a half-written model. The
16 that did not move them are the empty answers, which §17.4 says are real ones.

**The byte check was shown to catch the thing it is for, not argued to.** A
refusal made to leave one cube behind in `Model::fuse` takes the replay red on
the first refusal it reaches, in 8 ms:

```text
a refused fuse left the model changed: 3556 bytes where it had 2381
```

#### What it costs, and why the writer target did not get the same

0.82 s against 0.81 s over 682 fuses — one extra `postcard` round each, and the
gate's critical path is a 30.7 s test §95.5 already measured.

**The writer target was asked the same question and the answer was no.** A round
trip through `StepFile` and `Import` is feasible — both are public, and
`a_solid_written_out_and_read_back_is_the_same_solid.rs` already does exactly
it. But §95.5's writer row is `written: 1`: one model in a 1182-seed corpus
writes at all, and it is the hand-made control seed `two_cubes_that_write`. The
check would run once per gate, against a deterministic test that already
compares volume, area and centroid **bit for bit** and then fuses the result
against a third solid. That is the stronger oracle and it already runs.

So the writer target's problem is not missing checks, it is the 18-of-1182
decode rate that starves everything downstream of one. The fuse target takes 926
of 3502 off the same `postcard` door by splitting its input in two. Why that is
worth 926 against 18 is still not measured, and is its own piece of work rather
than a guess made here.

