## 234. Why 09-22's fuse fuzzing ran 36.7 M times against its neighbours' 1.3–4.6 M — 2026-09-25

*Closes §227.5's second open item under the same rule: read-and-measure only,
nothing in the kernel or the night changed to produce this. Checks the
hypothesis §227.5 left untried — "a night that happened not to start among the
slow inputs" — leg by leg, against 09-21 and 09-23, the nights either side.*

  234.1   The three nights: same flags, same fork count, different seed counts
  234.2   Leg (a) holds again, on this target's own report
  234.3   Leg (b): present, not absent — a slow input was in both 09-21's and 09-22's corpus
  234.4   Leg (c): code changed every night; which files moved differs
  234.5   What is left

### 234.1 The three nights: same flags, same fork count, different seed counts

The three commits, read from the nightly tree's reflog without writing to it
(`git -C /home/mattias/git/mattiasgronlund/rcad-nightly-fuzz reflog
--date=iso`):

| night | commit |
|-------|--------|
| 2026-09-21 | `ae124a29` |
| 2026-09-22 | `79eedce7` |
| 2026-09-23 | `055aa4a9` |

The `Running` line of each `fuzz-fuse.log` is the same command on all three,
down to the flag list: `-max_total_time=28760/28744/28756 -fork=4 -timeout=10
-ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`, differing only by the few
seconds each night's build stole from the window. None of the three passes
`-seed=`, so libFuzzer's own `INFO: Seed:` line is drawn fresh from system
entropy each night — `2902698029` (09-21), `3656053136` (09-22), `3260516880`
(09-23) — three different, unpinned numbers.

The seed count is on the next line, `-fork=4: N seed inputs`, read from the
same three logs plus 09-20, 09-24 and 09-25 for the neighbourhood:

| night | seeds |
|-------|------:|
| 09-20 | 3 383 |
| 09-21 | 3 297 |
| **09-22** | **3 107** |
| 09-23 | 3 520 |
| 09-24 | 3 591 |
| 09-25 | 3 687 |

This is §227.5's own figure, read here independently rather than restated:
09-22's 3 107 is the low point of the week, and 3 297–3 687 is exactly the
range the nights on both sides of it cover.

### 234.2 Leg (a) holds again, on this target's own report

§227.3's table already found this shape across the whole week; read here
directly from each night's own
`timings/two_solids_fused_by_a_stranger/report`:

| night | runs | p99 |
|-------|-----:|----:|
| 09-21 | 1 303 505 | 1 006.633 ms |
| **09-22** | **36 707 613** | **48.234 ms** |
| 09-23 | 4 583 772 | 67.109 ms |

09-22 ran 28x the inputs 09-21 did, with a p99 twenty times lower, and still
runs at roughly half 09-23's p99. Both neighbours spend at least one input in
a hundred over a few tens of milliseconds and some past a second; 09-22 does
not. Leg (a) — the nights round it spent most of their time in a few slow
inputs — holds for this specific trio, the same shape §227.3 found for the
week as a whole.

### 234.3 Leg (b): present, not absent — a slow input was in both 09-21's and 09-22's corpus

§227.5 phrased leg (b) two ways: the slow inputs were missing from 09-22's
corpus, or they were not reached early. These are different claims, and only
one of them survives a check.

`timings/two_solids_fused_by_a_stranger/slowest/` keeps the raw bytes of each
night's 64 slowest inputs, named `slow-<microseconds>us-<16 hex>` — a name
libFuzzer does not use for corpus files, which are named by the sha1 of their
own content. Hashing three of 09-21's slowest files
(`sha1sum` in `timings/.../slowest/`) and searching
`/home/mattias/rcad-fuzz-corpus/two_solids_fused_by_a_stranger` for a file of
that name:

- `slow-000008528656us-5aabd14d01388c30` → sha1 `a27c1eff5e62a14278c5bd8beec25f4ffc7925ee`
  — **present** in the corpus, `mtime 2026-09-20 02:02:05`.
- `slow-000008520707us-a9f4c4cf0f060b07` → sha1 `206dd866ad54d363f0a13ae2d146ee999c68591d` — not present.
- `slow-000008520408us-09b0cbc07f6eee90` → sha1 `f3dce5ebed7c6d14c1153912e7959ebf207d6d04` — not present.

Two of three are one-off mutations that never entered the corpus — libFuzzer
only keeps an input that adds coverage, so most of a night's "slowest 64" are
never seen again by name. But the one that is a genuine, persistent corpus
member was added on 2026-09-20 (§227.2's night, in the newly-found slow
region) and, by mtime, was already sitting in the corpus before both 09-21's
00:10 checkout and 09-22's — nothing removed it in between, and it is still
there today. **So at least one of the slow inputs that made 09-21 slow was
equally present in 09-22's 3 107-file seed pool**, and 09-22 still ran fast.
"Lacked those slow inputs" is false for this one; "were not reached early"
is what is left standing.

That reading fits §234.1's flags: each night draws a fresh, unpinned libFuzzer
RNG seed, and `-fork=4` splits the corpus across four independent jobs whose
own mutation schedules are not logged. A corpus member sitting unexplored for
most of a 28 000-second run is exactly what a different draw of that RNG
produces — the same mechanism §227.2 demonstrated for the cone target, which
took three hours to first wander into its slow region under one seed. No
attempt was made here to rerun 09-22's corpus under 09-21's seed to confirm
this directly (see §234.5); the corpus-membership check is the evidence in
its place, and it was checked on three names rather than all 64.

### 234.4 Leg (c): code changed every night; which files moved differs

`git diff --stat` between the three commits, unrestricted (the restriction to
`crates/rcad-geom`/`rcad-predicates`/`rcad-fuzz-timing`/`fuzz/` that §227.2
used for the cone target misses the fuse's own crate):

**`ae124a29` (09-21) → `79eedce7` (09-22): 84 commits.** Touches
`crates/rcad-geom/src/{cone,cylinder,intersect,sphere,torus,walk,exact,half_space}.rs`,
`crates/rcad-mesh`, `crates/rcad-step`, and
`crates/rcad-topo/src/{builder,error,mesh,model,validate}.rs` — but **not**
`crates/rcad-topo/src/fuse/{arrange,assemble,bake,mod,select}.rs`, the fuse
operation's own files.

**`79eedce7` (09-22) → `055aa4a9` (09-23): 59 commits.** Touches
`crates/rcad-topo/src/fuse/arrange.rs` (+446/−), `assemble.rs` (+120/−),
`bake.rs` (+41/−), `mod.rs` (+16), `select.rs` (+141/−), plus `holds.rs`,
`mesh.rs` again, and `rcad-step/brep.rs` — this time the fuse's own files are
exactly what moved.

So "nothing else differed" is false for both night pairs — this repository
lands dozens of commits most nights, and §227.3 already found the fuse gains
coverage counters (86 179 to 104 953) across the week, so churn on any single
night is not by itself unusual. What is specific to this pair is that the
change feeding into 09-22 left the fuse's own code alone while the change
feeding into 09-23 did not touch it lightly. That is suggestive of a code
explanation sitting beside the corpus one, but it is not established as
causal here: §227.3's own week table shows a comparable swing (09-19 to
09-20, 6.3 ms to 30.1 ms mean per run, 4.49 M to 3.48 M runs) between nights
whose diffs were never checked for size. This leg is left seen, not
confirmed — the same status §227.5 gave the cone target's timing-record
gap.

**Machine load has no record to check.** `sar` is installed
(`/usr/bin/sar`), but `/var/log/sysstat/` holds only `sa17`, `sa18` and
`sa19` — no data for 09-20 through 09-25, the whole window this row and §227
cover. No other load-average log was found. Same fork count (`-fork=4` on
all three nights, implying the same `nproc`) is the only thing checked here;
whether something else was competing for cores that night is unmeasured,
exactly as §227.3 left it for the fuse generally.

### 234.5 What is left

- **Leg (c) is not run down.** §227.4's method — build both binaries, replay
  the same fixed input sets on each, swap order to cancel load — was not
  repeated for `ae124a29`/`79eedce7`/`055aa4a9`. That would say directly
  whether the fuse/*.rs changes between 09-22 and 09-23 made the same inputs
  slower, the way §227.4 did for the week as a whole.
- **Leg (b) was checked on 3 of 64 named slow inputs from one night.** A full
  sweep — hash all 64 from 09-21 and 09-23 each, check presence and mtime
  against 09-22's 00:10 cutoff — would say whether "present but unexplored" is
  the rule or this lane's one example was a favourable pick.
- **No direct rerun of 09-22's corpus under a different RNG seed.** The
  "not reached early" reading rests on the mechanism (unpinned seed, forked
  jobs) being sufficient, not on a rerun showing it happening.
- **Machine load is unrecoverable for this window**; `sar`'s own retention
  cut it before 09-20. If load matters here, only a live measurement on a
  future night would catch it.
