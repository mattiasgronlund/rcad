## 226. The test plan gets a file and a standing lane, so work on what the tests search is not starved — 2026-09-25

*Asked by Mattias on 2026-09-25: "how do we ensure that #4 and #5 plus 1..4 get
their own prioritized lanes, so they do not get stuck?" The numbers are the
2026-09-18 fuzz plan's items and the four next steps of a review of it the same
day. This section writes down the answer. It dispatches nothing.*

  226.1   What got stuck, and where each piece was written
  226.2   Why a ranking does not fix it, and a reserved lane does
  226.3   What the review of the nights measured
  226.4   What is built, and what is not
  226.5   Found on the way: §222.5's flaky selftest loses its cause to §186

### 226.1 What got stuck, and where each piece was written

On 2026-09-18 a session answered "how can we improve the nightly fuzz tests so
that they can identify more failures?" with five changes. The first three —
targets run at once, fork mode, a ten-second limit per input — landed that day
as §129, §134 and §135. The other two did not, and were never picked up:

- **Item 4, check answers and not only "did not panic"**: the writer reads back
  what it wrote; the fuse asks each pair both ways round. It is written down in
  §129.5 item 3, as "what is not done". Nothing reads a section's "what is left"
  as a list of work.
- **Item 5, a STEP keyword dictionary and the real STEP files as seeds** for the
  reader target. It is written **nowhere in the tree**: only in that session's
  transcript and in `/tmp/rcad-handoff-nightly-fuzz-yield-2026-09-18.md`, which a
  reboot has since wiped.

On 2026-09-20 §171.7 found the same hole from the other side: this kind of work
"has no record", and the only box for it, roadmap 11.3, was ticked. It moved
11.3 to `partly` and left the record's shape unsettled. Five days later there
was still no record.

### 226.2 Why a ranking does not fix it, and a reserved lane does

§171.5 sorts the work into three kinds. A is priced by the corpus files it frees
or the fuzz findings that still fail on `main`, and lanes are cut from it. C is
what changes the rate at which A appears — test strategy. A C item frees no file,
so in any ranking by what a lane frees it comes last, every time. Writing C down
is necessary and not enough; it would be a better-kept list that still never
wins.

So the rule is a **reserved lane, not a rank**. Whenever lanes are dispatched,
one of them takes the top `todo` row of `docs/test-plan.md`, whatever unit §0
names. §0 carries a line saying so, because `just status` prints §0 first to
every session, and that is where a dispatcher looks before cutting lanes.

Inside the file the order is the rank. A row names the section that argues for
it, and a section that ends by naming a change to what the tests search adds its
row in the same commit — which is the step §129.5 and the 2026-09-18 handoff
both skipped.

### 226.3 What the review of the nights measured

The per-night counters in `~/rcad-fuzz-logs/<night>/fuzz*.log` (the last
`#N: cov: … oom/timeout/crash:` line of each), comparing the last night that ran
the old way, 2026-09-18, with 2026-09-25:

| target | runs 09-18 | runs 09-25 |
|---|---|---|
| writer | 98.7 M | 2 957 M |
| reader | 45.6 M | 449 M |
| two surfaces crossing | 259 k | 677 k |
| a cone or a torus crossing | 29.5 M | 2.1 M |
| fuse | 2.97 M | 2.05 M |

The night went from about 8 core-hours to 160 (five targets, `-fork=4`, eight
hours). The extra went to the writer and the reader, which assert only that
nothing panicked. The cone-or-torus target made 160 M runs on the night of
09-20 and 2.3 M on 09-21 under the same flags, so something between the two
commits those nights fuzzed made it about 70× slower. That is row T1, unexplained
here.

What the nights found after the change: §140 on 09-19 and §181 on 09-20 (four
crashes, one bug), both fixed; on 09-21 one crash of §181's family already fixed
on `main` (§186.1); nothing on 09-22 to 09-25. In the same week the ordinary
suite's new families found the wrong answers — §191's five, §187's two mesher
bugs, §223's five cone associativity findings — and the `#[ignore]` count went
from 2 to 16.

The crossing target writes 550 to 750 new `timeout-` or `slow-unit-` files a
night; the live `rcad-nightly-fuzz/fuzz/artifacts` held 2653 and 1171 on
2026-09-25. §135.4 made them measurements rather than findings, expecting about
one a night and trusting the summary count to show a new one. The count is
cumulative and the line reads "5 targets, nothing found", so the fuse's first
timeouts ever, three on 2026-09-24/25, went unseen. Rows T3 and T4.

### 226.4 What is built, and what is not

Built: `docs/test-plan.md` with eight open rows and five done; the line in §0;
`docs/plan.md`'s 11.3 now names the file as its evidence.

Not built: a check that holds the file's rows to their shape. It is row T8, last,
because the reserved lane is what keeps the rows moving, and a check only keeps
them tidy.

### 226.5 Found on the way: §222.5's flaky selftest loses its cause to §186

§222.5 traced `check-fuzz-overnight-selftest` failing six times in nine to its
arms running the real `fuzz-watermark`, which walked every sha in the shared
`.git/rcad-greens`. §186, merged today as `a736e351`, takes that path away twice:
`fuzz-watermark` no longer reads `rcad-greens` at all, and the selftest's shared
`run()` sets `RCAD_FUZZ_ALREADY_ADVANCED=1` so no arm asks it. Shown by reading
the merged recipes (`just recipe fuzz-watermark`, `just recipe
check-fuzz-overnight-selftest`), not by a run under §222.5's load.
