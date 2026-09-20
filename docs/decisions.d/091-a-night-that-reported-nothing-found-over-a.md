## 91. A night that reported nothing found over a crash, recorded 2026-09-13

`~/rcad-fuzz-logs/latest` read

```text
2026-09-13 08:10  6e71fe6 (1 days old, 0 behind main)  4 targets, nothing found
```

and it was wrong. The night before, on **the same commit**, `where_two_surfaces_cross`
had exited 77 on a real bug, and nothing was fixed in between: the only commit
since was `b7bda36`, which touches a test, this file and one corpus seed, and no
`src/` at all.

§90 gave the night a watermark so it would stop re-finding what was already
fixed. This is the opposite failure and it was hiding underneath: the night lost
what was **not** fixed.

### 91.1 Why the next night could not re-find it

Two properties of `fuzz/artifacts/`, and either one alone is enough to lose a
finding.

**libFuzzer never reads that directory back as a seed.** It writes a crashing
input there and moves on; the corpus it loads next time does not contain it. So
the 2026-09-13 night started from a corpus without the crash in it, rolled a
fresh seed — 3315978241 against the previous night's 2764877876 — and did not
come across the same input again. Confirmed by hand: the crash
`crash-cba396f88aa1063b9d879100bded7b9fcd0d5f95` is in neither
`fuzz/corpus/where_two_surfaces_cross/` nor `~/rcad-fuzz-corpus/`, and
`git log --all` finds it on no branch. The two *slow* units from the same nights
are in the shared corpus, because a slow input is kept as ordinary coverage and a
crashing one is not.

**It is gitignored and it lives in a worktree.** §89's slow unit sat there as its
only copy for three days in a tree that had been unlocked and was due to be
reclaimed, which is what `b7bda36` was written about eight hours before this
section. The crash above was in the same state: one copy, in
`rcad-nightly-fuzz`, a tree a cleanup sweep asked about the same morning.

So the night now copies `fuzz/artifacts/` into its own log directory after **each
target**, and names the count in the summary line. After each target rather than
once at the end, so a night killed from outside keeps what the targets that did
run found. Named in the summary because a copy nobody is told about is very
nearly as lost as the original — the 2026-09-13 night wrote three slow units and
said "nothing found", and both halves of that sentence were true.

What is deliberately **not** done is landing a new artifact into the corpus
automatically. A crashing input inside the corpus makes every later night die
while loading it, so a crash can only become a seed once it is fixed. §85 and §88
did that by hand and were right to.

### 91.2 The bug the night found, which is a gap and not a limit

Two parallel pipes, axes along +X, the thin one's axis lying exactly on the fat
one's skin. `intersect(fat, thin)` returns `CannotNameWhichPieceOfACrossingThisIs`;
`intersect(thin, fat)` returns `Curves` with two runs. It reproduces at `b7bda36`,
`main`'s tip — the artifact was re-run, not reasoned about.

That is §16.10's **gap**: the answer exists, and one order did not reach it. The
oracle that caught it says so in its own doc comment, which is
`naming_a_pair_does_not_depend_on_which_way_round_it_is_asked` in
`crates/rcad-geom/tests/fuzzing/mod.rs`.

**The 5.04e153 coordinate in the failing input is not the cause**, which is worth
writing down because it is the first thing the input makes you think. That
coordinate sits along the axis direction, so it names the same line as any other
point on it, and `x = 0` refuses identically. What decides it is **the radius
ratio alone**, and the threshold is scale-free: fat-first stops deciding at
`r1 ≈ 2^26 · r2` — 2^27 at `r2 = 1`, 2^26 at `r2 = 1.984375` and again at
`r2 = 1000` — while thin-first answers at every ratio tried up to 2^40. Moving
the thin axis one ulp either side of the fat skin changes nothing, so this is not
a knife-edge either.

2^26 is a 67 m pipe against a 1 m one. That is reachable from the public API and
not only from a fuzzer, which is what makes it a gap to fix rather than a limit
to defer. **It is not fixed here.** This section records it, the seed is not
landed (see §91.1 — it would kill the night), and the fix is a lane's.

### 91.3 A seed that costs 63 seconds, landed anyway

The same night's timings table is the other half of what was missed, and nothing
in the night could have flagged it: p50 0.010 ms, p99 5.5 ms, p100 **146 029 ms**,
and libFuzzer's `-timeout` defaults to 1200 s, so a 146-second input is not a
timeout and the night exits 0 over it. Only §90's own per-target timings saw it
at all. Re-run at `main`'s tip: 152.2 s, exit 0. Sixty-four inputs ate 1774.8 s,
25.4% of that target's measured 6996.8 s.

That input is now `fuzz/corpus/where_two_surfaces_cross/slow-unit-b9068e9ed0f147c54aec5b6b35af2f152303b761`,
and **landing it is a real cost, argued rather than waved through.** It takes the
corpus row of `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly` from
0.072 s to 63.2 s — measured by adding it and taking it away again — so every
`just gate` and every CI run pays a minute for it until the gap behind it closes.

It is landed anyway, for two reasons. The input was **one sweep from being
destroyed**: in `fuzz/artifacts/` and last night's `timings/*/slowest/` and
nowhere else, in a locked tree somebody was asking to reclaim that morning.
And 91.1's copy-out protects the *next* one, not this one, which was written
before the recipe changed. The alternative — keep it only in the logs and land it
with the fix — is what §89 did, and §89 is the section that then published a
26 000× timing with no input behind it for three days.

Two near-perpendicular pipes, radius 2147482112 against radius
`4.3601508761683463e-106`, a ratio near 5e114. After 63 seconds it produces **no
points at all**, so it brought the corpus row one order and two scalings and not
a single point. Producing nothing slowly is §89's shape exactly, at ten times the
cost, and that is the next thing to chase.

A second orphan landed with it, and this one took no arguing:
`slow-unit-5a19686d61280c78efada98b5ede9c3b925f5898`, written 2026-09-09, found by
a peer session sweeping dead worktrees and within minutes of being deleted with
the tree it sat in. Measured the same way, it costs about 0.1 s and brings **18
points**, one order and two scalings. That is the case §89 and this section are
both making, in its easy form: a seed that buys coverage for a tenth of a second
needs no justification beyond having nearly been lost.

**Two orphans of this shape were found on one day, by two sessions that were not
looking for the same thing.** That is the measurement that justifies 91.1 rather
than any single incident: the gitignored artifacts directory is not an occasional
trap, it is where findings go by default, and it reads as clutter to every test a
sweep can apply — unlocked tree, clean `git status`, commit an ancestor of `main`.

### 91.4 What is checked

`check-fuzz-overnight-selftest` gains two arms, and the stub `cargo` now writes
an artifact where `cargo-fuzz` would — its cwd is the `fuzz/` directory the
recipe cd's into, and `-artifact_prefix` is not something the environment can
move, so the fixture has to write into the real `fuzz/artifacts/` and take it
away again on the way out. The tag is unique per run so two lanes can gate at
once.

The arms are that **every one of the four targets has its artifact in the night's
own directory** afterwards, and that the summary line says how many were kept.
Both were confirmed by breaking the recipe and watching them fail — the first
reports `where_two_surfaces_cross's artifact was left in the worktree`, which is
the sentence the morning of 2026-09-13 needed and did not have.

The count is asserted as **at least four and not exactly four**. The stub writes
into the real artifacts directory, so a tree where somebody has actually fuzzed
has more in it than the fixture put there, and pinning the number would make the
arm pass or fail on a property of the machine rather than of the recipe. That is
§77.6's mistake, and this is the third time it has been available to make.

### 91.5 The morning tool called two live crashes clean, recorded 2026-09-17

`just fuzz-triage` read `rcad-fuzz-explain`'s **exit code** as its verdict. That
crate exits 0 "when the bytes decoded to a case and the case replayed, whatever
the case then did" — `crates/rcad-fuzz-explain/src/main.rs` says so in the
sentence above `fn main`, deliberately, because a decoder that died on a
crashing input could not report one. So a replay that panicked exited 0 exactly
like one that answered, and the recipe printed both of these on the same entry:

```text
outcome: panicked — … writing this pair at 2^-1 changed what it crosses in.
still fails on main: no — it decoded and replayed clean
```

and then totalled `0 of 8 input(s) did not come back clean on main`. The answer
was **two of eight**, both of them the cone-against-plane scaling crash of §91.2's
family, on `fbab36ff`, which was `main`'s tip that morning. Confirmed by hand
before anything was changed: the same command on
`crash-89994c6f26f3142366aee9b02b26d32baca87c38` prints `outcome: panicked` and
exits `0`.

The contract was never ambiguous. `Outcome` is three variants and its doc comment
says "the three arms are the three `just fuzz-triage` was written against" — so
the recipe had an answer written for it on stdout and read a different question's
answer instead.

**Why four green arms did not see it.** Every fixture in
`check-fuzz-triage-selftest` is bytes that do not decode (`not a real crash
input`, `nor is this one`), which exit non-zero. The panicking-replay path — the
one the recipe exists for — had no arm at all, and arm one asserts only that the
string `still fails on HEAD` appears somewhere, not what follows it. This is
§77.6's shape once more: every arm was passing for a property of its fixture
rather than of the recipe.

**The fix splits deciding from producing.** `fuzz-triage-verdict` takes the
decoder's exit code and its stdout and prints the verdict, exiting non-zero when
the input counts against the tally; `fuzz-triage` calls it. Deciding needs only
text, so the gate can put all five cases in front of it — panicked, succeeded,
refused, never-replayed, and a fourth outcome nobody has written yet — without a
workspace, a nightly build, or an input that panics.

**No arm owns an input that panics, and that is the point.** An artifact that
crashes for ever is the one thing this kernel is trying not to have, so an arm
built on one goes red on the day the bug is fixed. Canned text has no such
expiry.

A `refused` replay counts as **gone, not failing**. Giving up by name is an
answer (§17.4), and a crash artifact that now refuses is a crash that no longer
reproduces. A fourth outcome counts against the tally and says the contract has
moved, rather than passing as clean on a name this recipe has never seen.

