## 79. A gate arm that died without saying so, recorded 2026-09-09

`check-corpus` (§44.10's companion, added when `corpus/fetch.sh` was) passed the
first time it ran, on a corpus that was right. `rscene` runs a copy of it, and
theirs was green while one of its three arms could not name a file at all. The
recipe here is not exposed to that bug, and the selftest is here anyway; the
difference between those two sentences is most of this section.

### 79.1 What the arm was doing instead of working

`rscene`'s copy sorted its two path lists `LC_ALL=C` and left the `comm` that
compares them in the ambient locale. `comm` then reads its input as disordered,
says so on stderr, and exits 1 — and under the recipe's own `set -euo pipefail`
an `extra=$(comm …)` assignment that exits non-zero takes the whole recipe down.
Their mutant run therefore named **nothing**, which read as three dead arms and
was one abort before any arm could print. That misreading is worth more than the
bug: an exit code standing in for an assertion is the layer the defect hid
inside, and a suite that stops at the first fault hides the two behind it.

Our copy has no `LC_ALL` on either sort or on the `comm`, so all three agree by
construction and the arm works. That is a property of the recipe and not of the
machine, which is the form of the claim that matters — "it depends on your
locale" was how both of us first put it, and it is wrong.

I also claimed the mixed configuration would produce a *silently* wrong answer
at exit 0, reasoning that `comm`'s order check is a heuristic. That is reasoning
and not a demonstration (§16.10). The search: all 256 subsets of eight names
whose C and dictionary collations disagree, both sides C-sorted, every answer
compared against the set difference computed independently. 255 warned; none was
quiet and wrong; the one silent case was silent and correct. Withdrawn. What is
demonstrated is that the mixed arm is *loud* and wrong — it emits names present
in both lists — and that the recipe aborts rather than passing.

### 79.2 Four mutations, each failing a different case

`check-corpus-selftest` builds a synthetic six-file corpus and runs
`check-corpus` against four copies of it. It touches no real corpus, so it runs
on a machine that has never fetched one, and sits in `gate` beside
`check-corpus` rather than behind it.

Verified by mutating the production recipe, one mutation at a time, reverting
between:

| Mutation | What failed |
|---|---|
| `LC_ALL=C` added to both sorts, `comm` left ambient | the invented-file case, on the missing **name**, at exit code 1 |
| the missing-file arm stops trimming `: FAILED open or read` | nothing, until the fifth guard — §79.3 |
| the mismatched arm stops trimming `: FAILED` | the same, and the same guard catches it |
| the invented-file arm's `ok=0` deleted | the same case, on the **exit code**, output naming the file and then saying `corpus matches … 5 files` |
| the `.stp`/`.step` filter dropped from the walk | the `notes.txt` guard, in four places at once |

The first and third are why both assertions exist. A mutation that leaves the
exit code right and the name missing passes an exit-code check; a mutation that
leaves the name right and the exit code wrong passes a name check. Neither
assertion alone catches both, and this recipe now has one mutation for each
direction on record rather than an argument that it might.

The first mutation is the one that differs from `rscene`. There the selftest
holds a fix; here there is no fix to delete, so proving the case is not vacuous
means *adding* the bug. That asymmetry is worth stating rather than leaving a
reader to assume the two repos' selftests verify the same thing.

### 79.3 The two mutations the ported suite did not catch

Dropping the `sed` that trims `: FAILED open or read` off the missing-file arm's
names left all four cases green. The arm then prints
`nested/deep one.step: FAILED open or read`, and the case asserts the substrings
`not on disk` and `nested/deep one.step` — both of which are still there. A
needle is a substring, so no case built this way can tell a trimmed name from an
untrimmed one.

That is a mutation surviving a suite written to catch exactly this class, found
by running it rather than by reading it — the argument for verifying by mutation,
restated one level up. The fix is a fifth guard beside the `notes.txt` one: no
arm may leave `sha256sum`'s own wording in what it prints,
`grep -qE 'FAILED( open or read)?$'` over every case's output.

**`check-corpus` trims two suffixes, and only one of them had been run.**
`: FAILED open or read` for a file that is gone, plain `: FAILED` for one whose
bytes changed. The guard's regex covers both by construction, but covered and
verified are different claims and this section asserted the second one from a
run of the first. `rscene [3568f9]` said so, and it was right. Deleting the
mismatched arm's trim: all four cases green, and with the guard,
`FAIL an arm printed sha256sum's wording rather than a bare name: aB.stp: FAILED`.
So both trims are now demonstrated rather than one demonstrated and one argued.

The guard is safe only while no case exercises `check-corpus`'s "failed in a way
this recipe does not parse" arm, which prints `sha256sum`'s output raw and means
to. A future case that reaches that arm must assert on it rather than be excused
from the guard.

`rscene`'s copy had the same hole in both trims. They ran the two mutations in
their own tree and reported both green; that half is their word and not a run of
ours, which is the distinction this repo owes on any claim about a tree it does
not build. Their §12 bullet had said "the missing-file arm's `sed` ... so it
prints its heading and not the file", which is the *printing* `sed` and is
genuinely caught; the commit message on `9b8256f` said only "the missing-file
arm's `sed`", and that arm has two. The ambiguity was theirs and the misreading
mine, which is why the fifth mutation exists at all.

They closed it on 2026-09-09 in `rscene`'s `d712f23`, with the same guard: same
regex, same placement after the four cases, same caveat about the unparsed arm.
That much was read off their tree rather than taken from their report — the
`justfile` diff, not the commit message.

`rscene [3568f9]` above is a **session**, not a commit; `git cat-file -t` on it
fails in both repos, as it should, and so does `a57470`. Sessions are how a finding like this one arrives
— it was said in conversation hours before either repo had a commit to point at
— and git cannot attribute one, since every commit in both repos is authored by
Mattias. So the file names the session, spends the ref knowing it dies with it,
and puts the durable half in the argument beside it. `rcad [a57470]` appears the
same way twice elsewhere. `rscene` reached the same rule from the other side
after reading this one as a sha and running `git cat-file` on it: their §12
spends `rcad-15` three times and now says, at `ca470be`, what kind of name that
is.

### 79.4 Why it refuses instead of skipping

The cases run under a locale that collates by dictionary order, found by asking
`sort` rather than by naming `en_US.UTF-8`, and the recipe **exits 1** when no
such locale exists. Under `C` or `C.UTF-8` — a runner's ambient locale — the
collation is already the one the bug would force, so the case that catches it
cannot fail. Three of four would still pass, which is worse than running none:
a green that has quietly stopped asking the question is what §45 is about, and
what §77 settled for a different gate arm on a different machine. `ci.yml` runs
`locale-gen en_US.UTF-8` so the runner has one.

### 79.5 What this does not cover

The synthetic corpus is six files and the real one is 367. Nothing here checks
that `corpus/manifest.sha256` describes the corpus anyone actually has — that is
`check-corpus`'s own job, and this only checks that it can say so. Nothing
compares this repo's manifest with `rscene`'s, either; §12 there records that
the two were compared by hand, once, and found identical at 367 lines.

