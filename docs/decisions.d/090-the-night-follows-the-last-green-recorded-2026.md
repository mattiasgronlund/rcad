## 90. The night follows the last green, recorded 2026-09-11

§81 bought the fuzzer hours that the gate cannot give it. §81.4 then refused to
let the night advance its own worktree, on the grounds that an unattended
`git checkout` fuzzes a write nobody reviewed, and accepted in exchange a timer
that could sit on one commit for a month.

The exchange was a bad one, and it took three nights to show it. **The night of
2026-09-11 spent itself re-finding §85**, on a base from 2026-09-09 that was
sixty-eight commits behind `main`, twelve hours after the fix for it went green.

So the night now advances itself, to **the newest commit on `main` that CI has
gone green on**, and only ever forward.

### 90.1 What the refusal cost, and what it was buying

The 00:10 run of 2026-09-11 wrote
`fuzz/artifacts/where_two_surfaces_cross/crash-e8acf510…`, and it reproduces
§85's bug: a point the walk put on a crossing coming back certified `Pos`
against one of the two surfaces that named it. `3ff8327` fixed that on
2026-09-10 at 11:44 and CI was green on it by 11:55 — **twelve and a half hours
before the run started**. The night ran on `879bd9f`, dated 2026-09-09, which is
sixty-eight commits behind the `main` of that morning.

That is §62's shape, which §81.4 named and then walked into: a run that
completes and was settled before it started.

What the refusal was buying is the case where `main` holds something that should
not be fuzzed — which in this repository means a commit that gets backed out.
**There has been one revert in 1003 commits.** Against it, one night in three
spent proving something already known. A rule should be priced against what
happens, and this one was priced against what might.

### 90.2 Green, not new — why an offset is the wrong watermark

The obvious cheap fix is to stand a few commits back from the tip — `main~5`,
say — on the theory that what is five commits old is settled. It is not, because
commits do not arrive one at a time: a lane merges its whole branch at once, so
five commits back can be five minutes back.

Measured over the last 115 positions on `main`, the wall-clock distance from a
commit to the one five before it:

| | minutes |
|---|---|
| min | 2 |
| p25 | 18 |
| median | 44 |
| p75 | 245 |
| max | 1046 |

**Under fifteen minutes in 21 of 115 cases, and over seventeen hours in the
worst.** An offset is noise in both directions, and it is at its worst on the
busy days — the days when unreviewed code is most likely to be there to catch.

A fixed delay ("nothing younger than four hours") bounds the staleness but still
measures the wrong thing: time is not review either, and four quiet hours say
nothing at all.

What does mean reviewed is the gate. §77 put `just gate` on every push to `main`
and it answers in nine to twelve minutes:

    71c7474  18:09:28Z -> 18:18:39Z        f3c13e9  18:02:46Z -> 18:14:34Z
    beca957  12:02:01Z -> 12:12:28Z        3ff8327  09:46:59Z -> 09:55:30Z

So "the last green" is both fresher than any offset would dare to be and the
only one of the three that answers §81.4's actual objection. §81.4 is not
overturned so much as finally given the thing it was asking for.

### 90.3 What moves the night, and what leaves it where it is

`just fuzz-watermark` decides and prints; `just fuzz-overnight` checks out. They
are separate because the second one's selftest runs the first for real, in
whatever worktree the gate was run from, and a checkout there would move
somebody's tree out from under them.

Three conditions, and each is a way a night could otherwise go wrong:

- **green** — nothing else is evidence that anything read the tree;
- **on `main`** — `--branch main` filters on the name a run was made under, so
  the candidate is checked against `origin/main` itself;
- **forward only** — a tree somebody advanced by hand is never dragged back, and
  §81.4's `git -C ../rcad-nightly-fuzz checkout --detach main` still works
  exactly as it did.

It **never exits non-zero**, and it stays put and says why when the tree is
dirty, when `gh` is absent or cannot answer, when `origin` cannot be reached, or
when no green commit is ahead of where it stands. A night that cannot work out
where to stand runs where it already is: trading a stale run for no run is the
worse of the two, and the summary line says which happened.

It asks `gh` before it fetches, so that a gate running this recipe needs no
network to pass — §77.6's shape, a check answering a question about the machine
it ran on.

### 90.4 The indicator that could not see the failure

§81.4 did not leave the staleness unwatched. It printed the commit and its age,
and `latest` on the morning of 2026-09-11 read:

    2026-09-11 08:10  879bd9f (1 days old)  4 targets, 1 TO LOOK AT (fuzz:crash)

Everything in that line is true, and the crash it names was reported. **A commit
one day old was sixty-eight commits behind.** Age answers "when was this
written", and the question the night needs answered is "what has landed since" —
a quantity §81.4 never printed, so a reader checking for staleness saw a small
number and moved on.

The summary line now carries the distance beside the age, which is what says the
watermark is stuck when it is stuck: a night that could not advance reports a
number that grows.

### 90.5 What is checked

`check-fuzz-watermark-selftest` builds a fixture repository with a stub `gh` —
no network, no account, and nothing that depends on the state of this tree — and
asks six questions of it: that it takes the newest green on `main` and not
merely the first candidate; that a green commit which is not on `main` loses to
one that is; that a tree already on the last green stays and says so; that it
never offers to move backwards; that a `gh` which cannot answer leaves the night
standing and still exits 0; and that uncommitted work is never checked out from
under.

The forward-only arm was confirmed by deleting the ancestor test and watching
arm four fail, which is the only one of the six whose absence a passing suite
would otherwise hide.

`check-fuzz-overnight-selftest` gains two: the summary line must carry the
distance as well as the age, and **the tree the gate ran in must be where it
was**. The second is the one that matters to whoever runs `just gate` — the
night is now the only recipe in this file that can move a worktree, and a stub
`gh` that answers nothing is what keeps it from doing so on somebody's machine.

