## 97. What the suite never runs, measured 2026-09-14

*The question asked was whether measuring what remains unseen is feasible and
worth it, over three proposals: line coverage with a ratchet later, scheduled
mutation testing, and an implementation order. Two thirds of the order had
already landed — §95 is the corpus replay and the result postconditions, §96 is
the cone and torus fuzzing — so what was left was the measuring. Both tools were
run. Coverage is cheap and found a dead path nothing had named; mutation testing
is affordable and found a survivor in its first eleven mutants. Neither gets a
gate and neither gets a timer, and §97.6 is the arithmetic for the second of
those.*

### 97.1 The easy coverage number is 95.7%, and it is wrong twice over

`cargo llvm-cov nextest --workspace` costs **577 s cold** — about one `just
gate` — in a `target/llvm-cov-target` of its own that shares nothing with the
ordinary build. All 1866 tests pass instrumented. The cost is not the build:
the suite goes to **347 s against the 30.7 s critical path §95.5 measures**,
with one `rcad-step` test alone at 342 s. `llvm-tools` is now in
`rust-toolchain.toml` for this and for nothing else, the pin being
`profile = "minimal"`.

Its printed answer is 95.7% of lines and of regions. **The real answer is
94.0% of kernel regions, with 2 115 never run**, and the gap between those two
sentences is two defects in the naive read, one of which is silent.

**A generic is exported once per instantiation** and its regions are counted
again each time. The records have to be collapsed onto their spans:
107 032 records over 61 110 spans on this tree. Nothing about the
un-collapsed number looks wrong, which is the problem with it.

**An inline `#[cfg(test)] mod tests` is ordinary lines of the file it sits in**,
and it is covered by construction — a test that does not execute is not a test.
62 source files carry one, and they hold **25 742 of the 61 110 regions, 42%**.
Cutting them moves the answer 95.7% → 94.0%: the padding is large and the
inflation is small, which is why a raw number is fine for a verdict and wrong
for anything pinned.

**The obvious filter for that fails silently.** Names in llvm-cov's export are
v0-mangled, so testing a function's name for `::tests::` matches nothing, keeps
every test function, and prints a confident "0 dropped" beside a denominator
three times too large. That is what the first draft of this did. The cut that
works is geometric — the region's own file and start line against that file's
first `#[cfg(test)]`.

`crates/*/tests/` needs no filter at all: llvm-cov maps only `src` into its
file summary. Measured rather than assumed — the printed and src-only totals
agree to the digit in all seven crates.

**So the reading checks itself against the summary before it says anything.**
The collapsed span count must equal what llvm-cov's own per-file summary reports
for `crates/*/src`, and 35 368 kept + 25 742 cut = 61 110 is that identity
holding. A parse that drifts from it refuses rather than printing a plausible
number, which is the only thing that would catch the next version of the
mangling bug.

### 97.2 A dead path in the marcher that nothing said was dead

`rcad-geom/src/march.rs` is the worst-covered file at **58.2%**, 245 regions of
586 never run, and three whole functions in it are never entered:
`how_far_off_a_cone_enclosing`, `how_far_off_a_torus_enclosing` and
`how_far_off_a_plane_enclosing`.

The first two are unreachable on purpose and say so. `how_far_off_and_which_way`
carries a paragraph on each arm naming the refusals that stop a cone or a torus
before a run exists to follow — `ACrossingBetweenAConeAndAnotherCurvedSurface`,
`ACrossingBetweenAPlaneAndATorusThatIsAQuartic`,
`ACrossingBetweenATorusAndAnotherCurvedSurface` — and explaining that the arm
does the arithmetic rather than panicking because §40 forbids the wildcard that
would have hidden it.

**The plane arm carries no such sentence and is equally dead.** Demonstrated
rather than argued, which is what §16.10 asks for: `march` has exactly one
caller, `follow_the_quartic` (`crates/rcad-geom/src/walk.rs:756`), reached only
from `WhatARunLiesOn::QuarticPickedUpAt` (`walk.rs:344`); a plane against a
quadric is a conic and never a quartic. So `how_far_off_enclosing`'s plane arm
and the function under it cannot be reached through the public API, not merely
through today's callers. The two neighbouring arms had that written down and
this one did not, and a coverage run is what made the asymmetry visible.

This is recorded and **not fixed here**. Whether it becomes a comment like its
neighbours or a demonstration in a test is a decision, not a cleanup.

### 97.3 What the missing 6% is, which is the part worth having

94.0% is not a number anybody can act on. Classified by the line each of the
2 115 unreached regions sits on:

| Kind | Count |
|---|---|
| an invariant `panic!` / `assert!` | 195 |
| a named `Error` / `Refusal` / `Invalid` | 162 |
| a `Decision::Undecidable` arm | 53 |
| a `None` / `Nowhere` / empty answer | 59 |
| everything else | 1 646 |

The last row is mostly continuation lines of the same constructs, and 244 of it
is `march.rs`. The 195 are unreached because §4.1 wants them unreached — they
are the crash-over-corrupt guards, and a suite that reached them would be
reporting a corrupt solid.

**What is left is the refusal half of this kernel.** CLAUDE.md's rule is that a
predicate returns `Decision` and every call site matches all four cases; 53 of
those arms have never run at the call site that matches them. Same for 162
named refusals. That is a more useful sentence than any percentage, and it is
why the recipe prints places.

So `coverage` and `coverage-report` print a list and no ratio. Every other
check in the `justfile` pins a count of things — wildcard arms, ignored tests,
corpus stages — and a ratio is the one number that moves for reasons nobody
reads.

### 97.4 Moving the inline tests out was considered and is the wrong trade

The tidiest fix for §97.1's denominator is to put the unit tests in
`crates/<crate>/tests/` and leave `src` holding only kernel code. Then the
denominator is right because of where the files sit rather than because code
implements a rule, llvm-cov's default output is already correct, and the whole
reading in §97.1 deletes itself. Given that the reading was wrong on its first
try, that is a serious argument.

**It fails on visibility, and not on diff size.** A `#[cfg(test)] mod tests` is
inside the module and can call private and `pub(crate)` items; a file under
`crates/<crate>/tests/` is a separate crate that sees only `pub`. Counting
declarations above the test module in those 62 files: **833 private `fn`, 199
`pub(crate)`/`pub(super)` items, 362 `pub fn`** — three quarters of what those
tests sit beside is invisible from `tests/`.

`march.rs:181` is the case that settles it. `march` is `pub(crate)`, and its
inline tests call it directly at lines 645, 669, 688, 711 and 723; not one of
the five compiles from `tests/`. The move would either make the marcher public
so a coverage number is easier to compute, or rewrite those five to reach it
through `walk` and test something else. So the measurement accommodates the
code, which is the right way round.

### 97.5 Four mutations of the reading, each failing a different case

`check-coverage-selftest` builds a two-file tree under `mktemp`, writes five
synthetic exports, and asserts on what `coverage-report` says about each. It
needs no coverage run, so it sits in `gate` beside `check-corpus-selftest`
rather than behind a ten-minute recipe. `coverage` and `coverage-report` are out
of `gate` and named without a `check-` prefix for the reason
`check-gate-runs-every-check` gives: a check nobody runs is not a check, so a
`check-` name would be paid on every commit.

Verified by mutating the recipe, one at a time, reverting between — §79's rule
applied to a reader rather than a shell recipe:

| Mutation | What failed |
|---|---|
| the collapse keeps the last count rather than the widest | the reversed-generic case **only**; the forward one still passed |
| `where_the_tests_start` always returns `None` | the cut case, on a line below the attribute being listed |
| the `"/src/"` test dropped from the record walk | the summary cross-check, before any output |
| the cross-check's `sys.exit` deleted | the disagreement case, which then printed a plausible table |

**The first is §79.2's lesson arriving again.** One order of the two records
passed with the bug in place and the other did not, so a suite carrying only
the forward case would have called that mutation caught. Both orders are cases
for that reason.

**The third is why the cross-check is load-bearing and not a nicety.** Dropping
the `src` filter is the same shape as the mangling bug that already happened
once, and the only thing that notices is the identity against llvm-cov's own
summary.

### 97.6 Mutation testing gets no timer, and the cheap rotation would be the worst one

`cargo mutants --list` sees **9 158 mutants**: rcad-geom 3 821, rcad-topo
3 543, rcad-step 766, rcad-predicates 594, rcad-mesh 342, rcad-fuzz-timing 90,
rcad 2. Heaviest files `fuse/arrange.rs` 711, `mesh.rs` 698, `walk.rs` 692,
`intersect.rs` 536.

**One shard of eleven, in `fuse/select.rs`, found a survivor.** `replace + with
*` at `crates/rcad-topo/src/fuse/select.rs:903` turns `&corners[index + 1..]`
into `&corners[index..]`, so `candidate_points` pairs every corner with itself
and pushes that corner — a point *on the boundary* — into the candidate set.
Nothing in rcad-topo's suite notices. It is probably harmless in effect, since
a boundary point fails the inside test it is drawn for; what it demonstrates is
that nothing asserts what that set contains. Unfixed here.

**The cost, measured clean.** Same eleven mutants, nothing else on the machine,
back to back:

| Jobs | Wall |
|---|---|
| `-j1` | 81 s |
| `-j4` | **44 s** |
| `-j8` | 46 s |
| `-j16` | 52 s |

It flattens at four and gets worse past eight: one `cargo` build already
spreads over all 24 cores, so jobs divide the same machine rather than adding
to it. About 4 s a mutant at the best setting, which puts all 9 158 at **eight
to fifteen hours** — one or two nights of the window `fuzz-overnight` already
uses. An extrapolation from eleven mutants of one file, so an order of magnitude
and not a schedule; the first sweep also pays a cold `kache` that later runs do
not.

**Three numbers published earlier in the session are withdrawn**: 18 s a
mutant, "`-j4` buys nothing", and "all 9 158 is two days". Each was measured
beside a 48 MB JSON parse and four selftest runs, and the 198 s run that
anchored them was also the first on that file. This is the §77.6 shape one more
time — a measurement of the environment it ran in rather than of the thing —
and the fix was the same: take away what the environment was supplying and run
it again.

**Scope: the mutated package's own tests, which is cargo-mutants' default.**
Running the same ten rcad-predicates mutants with `--test-workspace` cost
**222 s against 21 s, 10.6×, for the same verdict** — 8 caught, 2 unviable,
nothing the package's own tests had missed. One shard is evidence for keeping
the default, not proof of it.

**So there is no rotation, and no timer.** Three reasons, in the order they
bind.

A full sweep is one or two nights, so there is nothing to spread across nights.
A rotation would be machinery for a scheduling problem the arithmetic does not
have.

A rotation built the cheap way — `--shard k/n` against a date-derived index —
would report on its own numbering. `--shard` partitions **tonight's** mutant
list, and that list moves as the tree does: one added function reshuffles which
mutants fall in which shard. "Every part has now been tested" would then be a
true statement about shard indices visited and a false one about mutants
tested. A sound rotation has to key on file names, which are stable, plus a
gate arm that fails when a new `src` file is in no bucket — `check-gate-runs-every-check`'s
shape, and real work for a problem the paragraph above dissolves.

And the binding constraint is triage. One shard of eleven gave one survivor
against seven caught and three unviable, 1 in 8 viable; at anywhere near that
rate the full sweep is **hundreds** of survivors, each needing the read
`select.rs:903` needed before it could be called probably-harmless. The
roadmap's §11.4 says it outright — tooling finds bugs and does not fix them,
and the bottleneck has moved to repair. A nightly cycle manufactures findings
far above the rate anyone can judge them, and a list nobody works through is a
list that gets ignored.

What is settled instead: `cargo mutants --in-diff` on a branch's own diff
before a merge, and one deliberate full sweep on a day somebody can triage it.
§13 is why the sweep is not scheduled here — it has no named consumer until
that day exists.

### 97.7 The first scope: ten survivors, and only two of them mean anything

Q4's first deliberate scope, run at `190d672`: `interval.rs`, `cascade.rs`,
`policy.rs`, `decision.rs` — the 117 mutants where "replaced a `Decision`
variant" would live. **117 mutants in 59 s**, 10 missed, 78 caught, 29
unviable, `--output` into the scratchpad so nothing lands in the tree.

Triaged, because a survivor is a question and not a finding:

**Two are provably equivalent mutants.** `Policy::symbolic_perturbation`
replaced with `false` is identical to the real function — its doc comment says
"Always `false`" and §1.5 says why there is deliberately no way to turn it on.
And `+` to `-` at `interval.rs:94` computes the same interval: the operand is
`[-reach, reach]`, so `[c,c] - [-r,r]` and `[c,c] + [-r,r]` are both
`[c-r, c+r]`. Neither is a test to write.

**Five are the error payload of a branch that cannot observe them.** Four
mutants across `interval.rs:90` and `:91` change `lo` and `hi` inside
`Error::NotAnInterval`, on the path taken only when `centre` is not finite —
where `inf - r`, `inf + r`, `inf / r` and `inf * r` agree for every positive
`r`. They are distinguishable only for a *negative* `reach` with a non-finite
`centre`, which nothing asks for. The fifth is the `Debug` impl. Diagnostics,
not arithmetic.

**Two are a real gap, and it is the one worth having.** `cascade.rs:119` and
`:122` are `value > error_bound` and `-value > error_bound`, the float filter's
own certification test, and `>` to `>=` survives both. At exact equality the
true value can be zero, so the mutant certifies a sign the bound has not
earned — and no input in the suite lands on `value == error_bound`. That is the
boundary the whole level-1 argument rests on, untested, which is §15.6's rule
about a published bound wanting a test applied to the comparison that enforces
it.

**One is unsettled and is not being guessed at.** `delete -` in `impl Neg for
Interval` survives, and the two instruments disagree about it in a way that is
worth writing down: **coverage says line 197 runs, and mutation says nothing
depends on what it computes.** Breaking the impl on purpose and reading the
compiler's errors — rather than grepping for a unary minus, which finds nothing
— names eight call sites. Four are inside code §97.2 already showed is dead
(`march.rs:568` in `how_far_off_a_cone_enclosing`, and
`three_surfaces.rs:1264`, `:1603`, `:1648`). **Four run**: `intersect.rs:1027`
and `three_surfaces.rs:1688`, `:1694`, `:1706`, the last three all inside
`the_two_places_along_it`, which is `(-b ± √D) / 2a` in interval arithmetic
with `the_tighter` picking the narrower of two enclosures of the same root.

Making `neg` the identity makes both of those enclosures wrong, and the suite
still passes. Either the wrong enclosures reliably straddle zero and the
cascade falls through to level 3 and answers correctly anyway, or a wrong
level-2 answer is being certified and nothing notices. **Reading cannot settle
which, and this section is not going to assert one** — §97.8 carries it as
open. What is already certain is narrower and still useful: nothing pins the
*numbers* `the_two_places_along_it` returns, only the decisions downstream of
them.

**So the survivor rate is 10 in 88 viable, and the useful rate is 2 or 3.**
§97.6's triage argument was built on 1 in 8 and holds at that ratio; what it
did not predict is how much of the 1 in 8 is equivalent mutants and
diagnostics. A whole sweep at this rate would be roughly 900 survivors and
perhaps 200 worth reading, which sharpens the argument rather than softening
it: the cost is not the compute, it is that somebody has to do what this
subsection did, nine hundred times.

### 97.8 What this does not close

**`march.rs`'s plane arm still says nothing.** §97.2 demonstrates it is
unreachable and leaves the code untouched. Either it gets the paragraph its two
neighbours have, or it gets a test that reaches it, and nobody has decided
which.

**`select.rs:903`'s survivor is unfixed and untested.** The mutant is recorded,
the reasoning that it is probably harmless is recorded, and neither is an
assertion in the suite. Whatever fixes it should assert what `candidate_points`
returns rather than delete the mutant.

**The 94.0% is not held by anything and will go stale.** Deliberately: nothing
pins it, no `check-` recipe runs `coverage`, and the number in §97.1 is true of
this tree on this date. The list is the product, and it is re-read by running
the recipe rather than by trusting this section.

**Two of §97.7's ten survivors are unanswered, and they are the two that
matter.** Nothing lands on `value == error_bound`, so `cascade.rs:119` and
`:122` have no test at the boundary the float filter's certification rests on;
constructing an input where a predicate's value equals its own forward error
bound exactly is the work, and it is constructible because both are `f64` and
the bound is a deterministic function of the inputs. And whether `neg` becoming
the identity is an equivalent mutation or a wrong level-2 answer nothing
notices is **unsettled** — the experiment is to apply it and check what
`the_two_places_along_it` hands back at the four reached call sites against a
closed-form root, not to read the function again.

**The full sweep is unrun**, and the survivor rate now has two points that do
not agree: 1 in 8 viable on `fuse/select.rs`, 10 in 88 on this scope, of which
2 or 3 were worth reading. §97.6's triage argument survives both, but if a
whole sweep comes in far under them the argument against a periodic sweep
weakens and should be re-made rather than cited.

**"Semantic STEP goldens next" was the third item of the proposal, and it is
mostly already built.** Asked in a session of its own while this was being
written — `rcad [6f62db]`, a session and not a commit, named the way §79.3
names one — and the answer came back that the checks exist, scattered across
`crates/rcad-step/tests/` by purpose rather than gathered under one name: units
and uncertainty in `a_solid_written_by_hand_becomes_a_model.rs`, solid counts
pinned by name in `KICAD_FILES_THAT_IMPORT` and `KICAD_FILES_THAT_ROUND_TRIP`,
dimensions for `kicad/` in `a_real_file_measures_what_its_own_name_says`
(`how_often_a_real_file_imports.rs:191`) which reads pin-header spacing and
DIP-switch pitch off the filenames.

**The one open gap is narrow**: `nist/`'s 44 files get parse, import, sweep and
refusal accounting and no check against an independently known dimension, where
`kicad/` gets one from its filenames. NIST publishes nominal geometry, so the
same pattern could extend — undecided, and nobody has committed to it.

Two of those claims were verified here rather than taken on that session's
word, which is the distinction §79.3 says this file owes about a tree it did
not build. The two test functions and the two constants are where it says they
are. And **a placement golden would test a feature that does not exist**:
`nothing_is_placed_somewhere_this_reader_cannot_move_it` in `import.rs` refuses
`NEXT_ASSEMBLY_USAGE_OCCURRENCE` outright and refuses any
`ITEM_DEFINED_TRANSFORMATION` or shape-placing `MAPPED_ITEM` whose ends are not
both the identity, so this reader imports one unmoved shape per file. The
scattering, and that shell counts are moot because a multi-shell solid is
refused at both doors, are that session's reading and not this one's.

What §97 has to offer it is the reader's coverage map — `geometry.rs` 81.9%,
`brep.rs` 87.7%, `import.rs` 78.8%, `units.rs` 86.8%, `write.rs` 89.6%, the
unreached lines overwhelmingly named refusals for entity kinds the corpus never
reaches, which is §43.10's 85.8% seen from the other side. That is context and
not an answer: the gap found above is about which corpus has an independent
fact to check against, which no line count can see.

