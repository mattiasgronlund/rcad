## 172. The gate on a metered runner: a docs arm, cancelled supersedes, and a fresh-environment gate at home

CI stopped on 2026-09-20. Six pushes to `main` produced jobs with **zero steps**
that failed in two seconds, and the annotation on run `35501873094`, job
`106054944816`, says why:

> The job was not started because recent account payments have failed or your
> spending limit needs to be increased.

Nothing in the tree was wrong and no check failed. The runner never started. The
first thing this cost was half an hour of looking for a regression, because a
red X on a push to `main` has meant a regression every previous time.

`rcad` is private, so GitHub-hosted minutes are metered. The account is on
**GitHub Free** — 2000 minutes a month — which is not read off a billing page
but derived: `gh api /repos/mattiasgronlund/rcad/branches/main/protection`
answers *"Upgrade to GitHub Pro or make this repository public to enable this
feature"*, and branch protection on a private repo is a Pro feature. Between
2026-09-04 and 09-20, 171 runs spent **2271 wall-clock minutes**, 490 of them on
09-19 alone across 42 runs. The allowance was gone before anyone looked.

### 172.1 What was actually being bought

The bill is one number: **`just gate` is 9m46s** and everything else in the job
is under a minute. Step 11 of run `35459633208` took 586 s; `rustup show` 13 s,
`jdx/mise-action@v2` 3 s, `Swatinem/rust-cache@v2` 16 s, the corpus cache 2 s,
`apt-get install locales` 5 s, the two checkouts and the summary 5 s between
them. Anyone optimising the setup steps would have been optimising 40 seconds.

Against that, what the runs contained:

| of `main`'s last 150 commits | count |
| --- | --- |
| touching only `docs/` and `*.md` | 43 |
| merge commits | 32 |
| touching source | 75 |

42 of the 43 prose commits touched `docs/decisions.md`. Each one bought a full
compile, a 2093-test suite and a release build, of a tree whose source had not
moved — 13 minutes to find out that prose is still prose.

And separately, **40% of the runs were obsolete before they finished**: 68 of
171 had a later push land on the same branch inside their own 13 minutes.

### 172.2 Three cuts, none of which changes what a run checks

**`concurrency` with `cancel-in-progress`**, in both workflows. This is the 40%,
and it is free.

What it costs is a sentence this repo says often. A superseded commit now never
gets a green of its own, so "CI is green on `<sha>`" can only be said of a tip.
That is a real loss of record and it is accepted, because it is the weaker half
of a rule already kept here — "gate the tree you merged, not the one you tested"
— and the tip is the tree that is on `main`. A commit whose green was cancelled
was, by construction, not the tree anyone was going to keep.

**A docs arm**, `.github/workflows/ci-docs.yml`, selected by `paths`. `ci.yml`
gains the complementary `paths-ignore: ['docs/**', '**.md']`. The arm runs
`just gate-docs`, measured at **1m07s** against 9m46s.

This is sound only because no prose file is a compile input here. There is no
`#![doc = include_str!("../README.md")]` anywhere in the workspace —
`grep -rn 'include_str!' --include=*.rs crates/` finds nothing pointing at
`docs/` or an `.md` file, only prose *references* to section numbers inside
doc comments. That idiom is common enough that it could arrive in a single
commit and silently make a README a build input, so the check belongs in the
header of `ci.yml` where someone widening the filter will read it, and it is
there.

**`just gate-fresh`**, which needs no runner at all. See §172.4.

### 172.3 The docs arm is an exclusion, and that is the whole design

`gate-docs` runs `gate_steps` **minus** `gate_metered_steps`. It was tempting to
write the list the other way round — name the cheap steps the arm should run —
and that would have been the bug. A new `check-*` recipe would then join `gate`,
be enforced by `check-gate-runs-every-check` on the full arm, and quietly never
run on a prose commit. Excluding instead means a new check joins the docs arm by
default, which is the same direction §77's `check-gate-runs-every-check` already
guards, for the same reason.

**The failure that matters is not an expensive arm, it is a cheap one that
checks nothing about docs.** If `check-decisions` ever landed in the metered
list, a `docs/decisions.md` commit would be gated by nothing that reads
`docs/decisions.md` — and it would pass, loudly, printing the same summary
block a real gate prints. So `check-gate-docs-selftest` names
`check-decisions`, `check-decisions-selftest`, `check-claim-section-selftest`,
`check-plan`, `check-plan-selftest` and `check-where-the-work-is` one at a time
and fails if any of them leaves the docs arm. Named rather than counted: a count
survives the swap that matters.

It holds four more things, each mutation-tested rather than confirmed by a green
run — the lesson of §77.6, applied to the recipe written to serve it:

- every name in `gate_metered_steps` is really a step of the gate, because a
  renamed step leaves a name behind that excludes nothing and says so nowhere;
- both lists parsed to something, because both are grepped out of one file and
  an extraction that stops matching compares two empty sets and passes;
- the expensive steps really are gone, or the arm buys nothing;
- an arm that excluded *everything* refuses, because running no steps exits 0
  through `gate`'s loop and prints a passing summary having checked nothing.

**`check-corpus` is in the metered list, and it is the trap.** It compiles
nothing and runs in 0.16 s, so it reads exactly like a step the cheap arm should
keep. It *fails* when `RCAD_STEP_CORPUS` is not a directory — by design, §44.10
— and putting a corpus there is a 119 MiB fetch. A docs arm that ran it would
have gone red on every prose commit until somebody repaired it by adding that
fetch to the cheap job, at which point the cheap job would not have been cheap.
This was found by running `RCAD_STEP_CORPUS=/nonexistent/corpus just gate-docs`,
not by reading the list. `check-corpus-selftest` stays in the docs arm, because
it builds its own fixture and never opens the real corpus, and the selftest pins
it there too.

A push touching prose *and* source matches neither exclusion and runs **both**
arms, about a minute of overlap. `paths` and `paths-ignore` cannot both be given
for one event, so the alternative was a third filter guessing from a diff — and
the arm that would have to guess is the expensive one. The overlap is cheaper
than the guess.

### 172.4 `gate-fresh`: the manoeuvre from §77.6, as a recipe

The cuts above make the next month fit. They do nothing for the days until the
allowance resets, and a repo with no second opinion is where this was in August.

`just gate-fresh` runs the whole gate in a container on a fresh `--no-local`
clone, with `HOME` pointed at a scratch directory, no `CLAUDE_PLUGINS_ROOT`, no
sibling hub and no warm `target/`. That is precisely the manoeuvre §77.6 records
as the only way the three red runs of 2026-09 were found — take away what the
environment supplies and run it again — which until now was done by hand each
time it was needed.

**It is not the runner and does not claim to be.** Same kernel, same docker
daemon, same disk, so it cannot catch what only another machine would. What it
catches is the class the runner was bought for, at no minutes.

Two things it needs that a bare image lacks, both learnt from `ci.yml` rather
than from a failure: `locale-gen en_US.UTF-8`, because `check-corpus-selftest`
**refuses** without a dictionary-collating locale and a container has only
`C.UTF-8`, which is the wrong one (§79); and the corpus, mounted read-only.
`check-plugins`, the plugin arm of `check-hooks` and `check-questions-selftest`
all skip inside it for want of a config directory, which is §77 working rather
than breaking.

**It refuses a dirty tree, and that guard is the interesting part of the
recipe.** A clone carries commits, so the first version of this gated `HEAD`
while the changes it was written to check sat uncommitted three directories
away — and it would have reported a green, in the same words, for a tree nobody
had asked about. That is the §77.6 shape again, one level up: a check passing for
a property of something other than its subject. Copying the working tree in was
the wrong fix, because then it is not a fresh clone and the recipe has no
purpose; so it names the dirty files and exits 2. A WIP commit is enough, and
the branch is still on disk.

### 172.5 The cheapest cut is not in this section's tooling

Half the prose commits did not need to exist. The pattern on `main` is three
pushes for one unit of work:

```
2ebe8554  fix(topo): §169 — a pie slice of a ball says which points are inside it
55e628a8  docs: §169 — a pie slice is where its arc leaves the corner it shares…
08119094  docs: §0 — §168.4's pie slice closed by §169, fused arcs next
```

**5 of the 6 `§0` pointer commits sit directly beside the section commit they
announce**, and across `main` 27 of 44 referenced sections cost two or more
commits — 117 commits carrying 27 units of work.

The `§0` line says "§168.4's pie slice closed by §169", a sentence that is only
true *because* §169 exists. Same fact, same author, same minute, two pushes.

The cost argument is the smaller one. **Splitting them manufactures exactly the
staleness §102 built `status` to warn about**: §0 is edited by hand,
`check-where-the-work-is` holds its shape but cannot tell anyone it is stale, and
landing the pointer one commit after the section guarantees `main` carries a
wrong first row for the length of a gate, every time, by construction.

So: **the `§0` update goes in the same commit as the section that closes it.**
The code commit landing separately is honest — it comes from a lane, and the
section and pointer are written later by whoever merged — but those two are the
same post-merge act by the same session.

No check is proposed for this, and §158's arithmetic is why: the candidates are
every docs commit on `main`, the real hits are the ones where a `§0` edit and a
section edit were split, and a gate arm that guessed would fire on every
legitimate `§0`-only correction. It is a habit, written down where habits are
written down, and CLAUDE.md points at it.

Where it drifted from is worth naming, because it was a rule working as
designed: §8.3's branch-per-change requirement is about test claims, so a
docs-only change may go straight to `main`. "May land on its own" became "does
land on its own", nothing enforced otherwise, and until 2026-09-20 nothing
billed for it.

### 172.6 What is still owed

- **The arithmetic fits the average, not the peak.** ~10 runs a day → ~6 after
  cancelling → three code arms at 13 minutes and three docs arms at one is about
  42 minutes a day, ~1270 a month, inside 2000. A day like 09-19 — 42 runs — is
  ~169 minutes even after both cuts, and sustained at that rate nothing here is
  enough. The next lever then is not a fourth cut but batching `main`: PRs and a
  schedule rather than a run per push.
- **`check-release-build` was not moved to a scheduled arm**, though it is the
  obvious next cut and was in the plan this section was written from. It wants
  its own decision about how often a release build has to be green, and §13
  says not to build the part before the consumer.
- **Nothing measures the saving yet.** Every number above is from before the
  change. The honest check is the same query a month from now: `gh run list
  --repo mattiasgronlund/rcad --limit 300 --json createdAt,updatedAt` and sum
  the durations by day.

