## 173. The wildcard ledger counted four fewer in a container, because `awk` there is not gawk

`just gate-fresh` (§172.4) found this on its first working run, which is the
whole reason it exists. Same commit, same recipe: green here, red in the
container, on a number that is supposed to be a property of the tree.

```
wildcard arms over our own enums: 83 (83 direct, 0 composite), expected 87
```

### 173.1 The direct half agreed; it was the composite half that vanished

87 here is 83 direct + 4 composite. The container's 83 is 83 direct + **0**
composite — so clippy, the compiler, `--all-targets`, the build cache and
`RUSTC_WRAPPER` are all in the clear. They were the first suspects and every one
of them is wrong. What disagreed is `composite_wildcards.awk`, a line-based text
scan that compiles nothing and reads only the files in front of it.

Run over this tree, with the same arguments `check-wildcards` passes:

```
gawk    crates/rcad-geom/src/intersect.rs:1696
        crates/rcad-geom/tests/the_same_pair_turned_names_…_crossing.rs:379
        COUNT=4
mawk    COUNT=0
```

### 173.2 `\<` is gawk's, and mawk reads it as a literal `<`

The scan opened a match on `if ($0 !~ /\<match\>/) next`. GNU awk's `\<` and
`\>` are word boundaries and are its own extension; mawk has no such thing and
takes the backslash off, so the pattern means `<match>` — which no line of Rust
contains. **The scan therefore never left its idle state, read nothing, and
printed `COUNT=0`.** Not an error, not a warning: a clean zero.

```
$ docker run --rm rust:1-bookworm readlink -f $(command -v awk)
/usr/bin/mawk
```

mawk is Debian's default `awk`, so this is every Debian and Ubuntu container,
including `rust:1-bookworm`, which is what `gate-fresh` and any runner image
would use. The fix is to spell the boundary out —
`/(^|[^A-Za-z0-9_])match([^A-Za-z0-9_]|$)/`, `_` excluded from the class because
gawk counts it a word character too. It is the only GNU-only construct in the
192 lines; with it gone, **gawk, mawk, nawk and busybox awk all print the same
two sites and `COUNT=4`.**

### 173.3 87 is right, and the ledger does not move

This matters more than the fix. The tempting repair when a gate says 83 and the
pin says 87 is to move the pin, and it would have been wrong: the four arms are
really there, `intersect.rs:1696` and the turned-pair test at 379, and four
different awks agree once the scan is portable. **Nothing about the tree
changed and `wildcard_arms` stays at 87.** §40's bookkeeping was never off. What
was off was the instrument, in one direction only — silently downwards, which is
the direction a ledger cannot notice, because a check that finds fewer wildcards
than the pin reads as good news about the code.

Nor is the defect this branch's: `git diff --name-only 7e9a7f55..HEAD -- '*.rs'`
is empty, and the `\<match\>` line predates §172 entirely. §172's green on
`11142675` stands.

### 173.4 The test is to run the fixtures under every awk on the machine

`check-wildcards-selftest` already pins the scan against fixtures whose answer is
known. It ran them under `awk`, whichever that is — and on this machine that is
gawk, which is exactly the one implementation that could not see the bug.

It now runs them under `awk`, `gawk`, `mawk`, `nawk` and `busybox awk`, every one
of them that is on `PATH`, and fails if any disagrees. All four others are
installed here, so **the test fails on this machine without the fix** — it does
not need a container to bite, which is the only reason it is a test and not a
ritual. It prints which implementations it ran under, because a bare container
has mawk alone and an arm one implementation wide should say so rather than look
like four.

A blacklist of gawk-isms was the other candidate and is weaker: it catches the
constructs somebody already knows about, which is the class of guard that was
just shown not to hold. Running the thing under the other interpreter catches the
next one too.

### 173.5 The instrument goes before the ledger

`gate_steps` ran `check-wildcards` and then `check-wildcards-selftest`, so the
first thing the container printed was *"a new one, or a new variant reaching an
old one — see docs/decisions.md §40"*: a sentence about the tree, when what had
moved was the scan. The selftest, one step later, would have said so plainly and
never got to run.

The two are swapped now. **Prove the instrument, then read it** — the selftest
takes no compilation and costs nothing, and it is the step that can tell a broken
scan from a changed tree.

### 173.6 What the class is

This is §77.6's third instance and the first one `gate-fresh` caught rather than
a red runner: a check passing for a property of the environment it ran in rather
than of the tree. The first two were `check-plugins` reading a directory CI does
not have, and `check-wildcards`'s own direct half counting colour escapes under
`CARGO_TERM_COLOR: always`. **That is twice now for this one recipe**, both times
in the half that is a text scan over somebody else's output format, and both
times the answer was silently *low* rather than an error.

Worth carrying forward: every one of the three was found by taking away what this
machine supplies and running the check again. None was found by reading it. The
recipe that does the taking-away is `just gate-fresh`, it costs no Actions
minutes, and it earned its keep on the first run that got as far as the tree.

### 173.7 jq is pinned, python is not, and the awk is not rewritten

Mattias asked, on reading the above, whether it was time to pin a python in mise
and replace the awk with it. Three answers, and the first one is not about
either.

**`jq` is pinned, because it was the next thing to fall over.** With §173 fixed,
`gate-fresh` got four steps further and failed in `check-hooks`: no path on
stdout, no worktree, no lock — three of five checks wrong. `worktree-create.sh`
parses its payload with `jq`, `rust:1-bookworm` has none, and nothing in this
repo ever installed one. It passed here and on the runner because both images
happen to ship it. **Fourth instance of the same class in one session**, and the
cheapest possible fix: `aqua:jqlang/jq` is one static binary, so the pin costs a
download. Six places use it — three hooks, three recipes.

**Python is not pinned, and the asymmetry with jq is the argument.** It is
already load-bearing and already unpinned: `check-spend-selftest` runs
`check-spend.py`, `just question` runs the parser, and a recipe carries an inline
`#!/usr/bin/env python3`. This machine has 3.12 and the container has 3.11.2 —
two versions, in the two environments the gate runs in, nobody the wiser, because
the container has never reached step 38. By §173.6's own logic that is the next
one to bite.

It was still not pinned, for a reason and not an oversight. `check-spend.py` was
run under the container's 3.11 before this was decided: **60 classifier cases,
all as expected.** So the divergence is hypothetical where awk's was measured,
and the fix is not a download but an interpreter build on every cold runner —
against §172, where runner minutes are the binding constraint and the month's
arithmetic already sits at ~1270 of 2000. §13 says not to build the part before
the consumer names itself, and no consumer has. **What pins it is the first
python failure that differs across the two, or a month where minutes stop being
the constraint** — and the evidence of the 3.11 run is written here so that
whoever pins it is not re-deriving this.

**The awk is not rewritten in python.** There is one awk *script*, 196 lines, and
it is the one this section already fixed and now proves under four
implementations. The other 65 uses in the justfile are one-liners of the shape
`{print $1}` and `/^worktree /{print $2}` — POSIX, and the same under every awk
there is. A rewrite would spend the proof to buy a property one line already
bought. The script's own header names the thing that actually retires it:
rustc's `non_exhaustive_omitted_patterns`, unstable on the pinned 1.96. Port it
if it needs substantial work before that lint stabilises; not before.

**A fifth, found while writing this.** `check-worktree-create.sh` tested its lock
reason for non-ASCII with `grep -qP`. `-P` is GNU grep's, and a grep without it
exits non-zero on the *option* — which that `if` reads as "found nothing" and
passes. Silently, in the safe-looking direction, exactly like `COUNT=0`. It is a
`tr -d '\000-\177'` now, verified to catch what the old one caught.

And a defect in the same file that cost a round trip: it ran the hook with
`2>/dev/null`. The missing `jq` announced itself on stderr every time and was
discarded every time, so a reader got three symptoms and no cause. Stderr is
kept now and printed when a check fails. **A selftest that hides why it failed is
worse than no selftest, because it is believed.**

### 173.8 The kache check read a text format, and mise changed it

With jq pinned, `gate-fresh` got one step further and failed in
`check-kache-off-on-ci`:

```
off CI, RUSTC_WRAPPER is '' and should be kache
```

which reads as the `mise.toml` template having broken, and is nothing of the
sort. The check sed'd `^export RUSTC_WRAPPER=` out of `mise env`. This machine
has mise **2026.7.5**, which prints that line. A container curls the latest,
**2026.9.11**, which does not print it in that shape at all:

```
=== mise env | grep RUSTC_WRAPPER ===   (no line matched)
=== what mise exec actually applies ===
on:  []        off: [kache]
```

So both halves extracted the empty string. The local half failed, naming the
wrong cause — and **the CI half passed vacuously**, which is the half whose
failure took CI red for ten pushes. A check with two assertions, one of which
had quietly stopped asserting anything.

It asks `mise exec -- sh -c 'echo "${RUSTC_WRAPPER-}"'` now: the value mise
actually applies to a process, which is the thing the gate relies on, rather than
a line in a format mise is free to change. Both mise versions agree on it.

The class is §173.6's again with a new supplier: not the machine's `awk` or its
`jq`, but a *version* of a tool whose human-readable output was being parsed as
an interface. **Anything parsed out of a tool's stdout is a version dependency**,
and the gate has more of them.

### 173.9 The night fuzzes what CI blessed, and CI can no longer bless anything

`fuzz-watermark` picks the commit the overnight fuzz stands on: the newest on
`main` that a CI run went green for. §172 spent the Actions allowance, so no push
can be green in that sense any more.

**And it fails by staying put**, deliberately — "a night that cannot work out
where to stand is a night that runs where it already is". That was right when the
only way to lose the answer was a network blip. Against a billing failure it
means the night keeps fuzzing the last green commit for as long as the account is
in arrears, saying nothing: `just status` had it **18 commits behind `main`** the
morning this was found, and the gap only grows. A fuzzer working on a tree a
fortnight old is not obviously broken from any log — it still finds things, and
they are things that may already be fixed.

**What was wrong is the supplier, not the condition.** §81.4 asked that nothing
fuzz a tree nobody has read end to end. A green CI run was evidence of that
reading; `just gate` on a clean tree *is* that reading, and it already runs on
every commit that lands. So `_record-green` writes the sha of a passing full gate
into `rcad-greens` in the common git dir — one list for every worktree, untracked,
because a commit's greenness is a fact about this machine and not about the tree
it describes. `fuzz-watermark` merges that list with `gh`'s and keeps its other
two conditions unchanged: on `main`, and forward only.

Three things that would each have made the record a lie, and are refused:

- **A trimmed gate does not count.** `gate-docs` re-enters `gate` with
  `gate_metered_steps` removed and arrives at the recorder as a pass having
  compiled nothing. `_record-green` requires every metered step in the list it
  was handed. This is the one that would have left no trace: the file would name
  a commit and say nothing about how little ran for it.
- **A dirty tree does not count**, because the sha is the entire record.
- **The recorder never fails the gate.** Every giving-up path says why on stdout
  and exits 0. Bookkeeping that can turn a green run red is worse than none.

Merging two lists cost the old picking loop its ordering: `gh` returns
newest-first and a de-duplicated union returns in sha order, so "the newest" now
has to be worked out with `--is-ancestor` between candidates rather than taken as
the first that qualifies. Arm nine pins it in both directions, because a loop that
took either source's side would pass a one-directional test.

**Arm five earned its place twice.** `grep -v '^$'` exits 1 when it filters every
line away, so under `pipefail` two silent sources arrived as a *failed recipe* —
the single thing this recipe must never do. The arm that says "a gh that cannot
answer made the whole night fail" caught it on the first run of the new code.

What this does not do is make a local green as good as an independent one. It is
the same machine, the same toolchain, the same environment whose properties §173
spent the day finding checks that depended on. `just gate-fresh` is the
independent reading, and the honest upgrade is to let it record too — deferred
under §13 until the night's greens are actually seen to be wrong, and written
down here so that whoever wants it knows the hook already exists.

### 173.10 The same staleness was in `status`, where more people read it

§173.9 fixed the night and left the morning alone. `fuzz-watermark` was not the
only reader asking GitHub whether a commit had been read end to end: `just
status`'s fifth row asks the same question, of the same spent allowance, and
`status` is the recipe CLAUDE.md sends a session to *first*.

The number it gave was true of CI and useless about the repo. On 2026-09-20 it
read **"main tip 2229c205, last green 146d0d6, 22 commit(s) on main not yet
green"** while a full `just gate` had passed on `2229c205` itself. A reader who
believed it would think twenty-two commits were unread; a reader who knew about
§172 would know the row could not move and stop reading it. Neither of those is
a row worth printing, and the second is the worse outcome — a survey people have
learnt to skip is how the other four rows stop being read too.

So the green row now reads both sources, exactly as `fuzz-watermark` does: `gh`
for CI, `rcad-greens` for local gates, normalised to full shas in this checkout,
furthest-forward-that-is-an-ancestor wins. It is `_status-green`, beside
`_status-branches` and for the same reason — the live repository's answer is
whatever happened to it this week, so the ways the row goes wrong can only be
pinned against a fixture with a stub `gh`.

**The row names which source answered, and that is the substance rather than a
nicety.** §173.9 ends by refusing to call a local green as good as an
independent one, and that refusal has to survive contact with the place the
number is actually read. A CI green is a different machine. A local green is
*this* machine — the one §173 spent a day finding four environment-dependent
faults in, three of which were green here and red in a container on the same
tree. Now that almost every green is local, a row that blurred the two would be
the more expensive error, so a green nothing independent has seen says so on a
second line and a CI green carries no such line. That is §173.9's owed caveat
paid where it is load-bearing, next to the number it qualifies.

**What is still not taught about `rcad-greens`: nothing, and that is checked
rather than assumed.** §173.9 left this reader behind precisely by assuming, so
the claim owes a demonstration (§16.10). `gh run ... --status success` is how a
recipe here asks whether a commit is green, and the tree holds exactly two such
query sites — `fuzz-watermark` at `justfile:3356` and `_status-green` at
`justfile:5913`. The three remaining `gh run` hits are prose in `ci.yml`'s
comments. Both query sites now read both sources, so the set of readers asking
only GitHub is empty.

**The row still counts shas, and will often say one or two are not green.**
Landing §173.10 itself left it reading "main tip f2494a21, last green d32cca01,
2 commit(s) on main not yet green" — where `26727b9d` is the merge, whose tree is
`d8eb9b50`, *byte for byte the tree that was gated*, and `f2494a21` is docs-only
and was read by `gate-docs`. Both had been read; neither had been read under its
own sha.

Counting trees instead would remove that, and it is deliberately not done. A sha
is the whole record (§173.9), and the number this row exists to give is a
conservative one — it should err towards saying work is unread. **What makes it
tolerable is that it heals and the CI version could not.** The old row asked an
allowance that can never answer again, so its number only ever grew; this one
goes to zero the next time anybody runs a full gate on the tip. A row that is
briefly pessimistic after a merge is a different thing from a row that is
permanently wrong, and only the second is worth code.

### 173.11 The fifth run found nothing, which is the first evidence the class is exhausted

`gate-fresh` had gone four for four: every run since it was written found a real
fault, each one a check answering a property of its environment rather than of
the tree — mawk's word boundary (§173), the missing `jq` (§173.7), mise's changed
output format (§173.8). Each fix bought one or two more steps, and the furthest
it had ever reached was **step 10 of 43**. The expectation handed to this session
was a fifth.

On 2026-09-20 it ran the whole gate: **43 steps, PASS, 2101 tests** on `5ca78b31`,
against the same tree's 2101 here. Thirty-three steps ran in a container for the
first time and none of them disagreed with this machine.

**That is evidence about the instrument, not a clean bill of health.** Four
findings in four runs meant the next red was the expected outcome and the tree
was never the suspect; one clean run inverts that. A red from `gate-fresh` is now
*news* — the prior that it is the check rather than the code has just lost the
only support it had, which was an unbroken streak. Whoever gets the next one
should weigh both, where §173 could say "assume the check" and be right four
times running.

Two things the run showed that no local gate can:

- **`check-fuzz-watermark-selftest` passes in a container**, so §173.9's two-source
  picking loop is exercised somewhere without a `gh`, an account or this machine's
  `rcad-greens`. That arm was written the same day it was first run anywhere.
- **The container's green evaporates, exactly as §173.9 left it.** The run ends
  `green recorded for 5ca78b31 in /rcad/.git/rcad-greens` — inside the throwaway
  clone, which the recipe deletes on exit. The independent reading that §173.9
  says would be the honest upgrade therefore still reaches nothing, and the hook
  being *present and inert* is why that stays a deferral rather than a bug.

`nextest` reported **5 leaky** in the container and none here. Leaky is a passing
test that outlived its own process handles, not a failure, and the difference is
process reaping under PID 1 in a container rather than anything about the code.
Recorded because it is exactly the kind of difference that looks like a finding
next time somebody reads this output cold.

