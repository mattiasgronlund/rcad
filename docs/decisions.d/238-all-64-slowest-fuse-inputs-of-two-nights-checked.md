## 238. All 64 slowest fuse inputs of two nights checked against the corpus, not three — 2026-09-25

*Finishes §234.3's corpus-membership check: it hashed 3 of 09-21's 64 slowest
fuse inputs and drew a reading from the one that was a genuine corpus member.
This checks all 64 of 09-21's and all 64 of 09-23's, plus the byte-identity
question §234.3 left untried.*

  238.1   What was run
  238.2   The counts
  238.3   What they show for §234.3's reading

### 238.1 What was run

Read-only throughout; nothing was written under `/home/mattias/rcad-fuzz-logs`,
`/home/mattias/rcad-fuzz-corpus` or `rcad-nightly-fuzz`. All times below are
UTC; the overnight timer fires at 22:10 UTC (00:10 CEST, UTC+2), so the night
named 09-22 runs from 2026-09-21T22:10:00Z and the night named 09-23 from
2026-09-22T22:10:00Z.

For each of the 64 files under
`/home/mattias/rcad-fuzz-logs/2026-09-21/timings/two_solids_fused_by_a_stranger/slowest/`
and the 64 under the same path for `2026-09-23`:

- `sha1sum <file>` — libFuzzer names a corpus file by the sha1 of its content.
- checked for a file of that name in both
  `/home/mattias/rcad-fuzz-corpus/two_solids_fused_by_a_stranger` (the live,
  growing corpus) and
  `/home/mattias/git/mattiasgronlund/rcad-nightly-fuzz/fuzz/corpus/two_solids_fused_by_a_stranger`
  (the tracked seed corpus, read-only).
- where present, `stat -c '%Y'` for its mtime, converted with `date -u -d
  @<epoch> +%FT%TZ`.

Every match found was in the live corpus directory only, never the tracked
seed one, so none of the mtimes below are a git-checkout time in disguise.

Each file was then classified by mtime against the two cutoffs
(`2026-09-21T22:10:00Z` and `2026-09-22T22:10:00Z`, epoch `1790028600` and
`1790115000`): **never kept** (in neither directory), **in 09-22's seed
pool** (present, mtime before 09-22 started), **added during 09-22** (mtime
in `[2026-09-21T22:10:00Z, 2026-09-22T22:10:00Z)`), or **added later** (mtime
at or after `2026-09-22T22:10:00Z`).

The same hash-and-check was run a third time on 09-22's own 64 slowest files,
to compare against itself, and the three nights' 64-hash lists were compared
pairwise with `comm -12` on sorted sha1 lists for byte-identical overlap.

### 238.2 The counts

| night | slowest files | never kept | in 09-22's seed pool | added during 09-22 | added later |
|-------|--------------:|-----------:|----------------------:|--------------------:|------------:|
| 09-21 | 64 | 52 | **12** | 0 | 0 |
| 09-23 | 64 | 57 | 0 | 0 | **7** |
| 09-22 (own list) | 64 | 63 | 1 (mtime 2026-09-13) | — | — |

12 of 09-21's 64 (18.8%) were already corpus members before 09-22's night
even began — an order of magnitude more than the one §234.3 hashed, spanning
mtimes from 2026-09-19T00:23:56Z to 2026-09-21T01:38:16Z, all short of the
2026-09-21T22:10:00Z cutoff. Zero of them were added during 09-22's own run
or afterward.

Of 09-23's 7 kept slow inputs, all 7 have mtimes at or after
2026-09-22T22:10:00Z (09-23's own start) — none pre-date 09-22 and none were
added during 09-22's run. Six fall inside 09-23's own active window
(2026-09-22T23:41:29Z to 2026-09-23T03:09:42Z); one
(`slow-000003204424us-c95269bc5dc2cc1c`, sha1 `412172101e74…`) has a mtime of
2026-09-23T22:22:22Z — inside the *following* night's (09-24's) window,
despite being one of 09-23's own 64 slowest inputs by execution time that
night. Its content was measured as slow during 09-23's run either way; only
its corpus copy's timestamp says it was not saved until a night later.

*Later, §240: that timestamp is a rewrite. libFuzzer's fork-mode merge writes
a kept input over any file of the same name without looking, so a later night
can re-date a file it did not find. 09-23's own log grows the corpus by one
within 90 seconds of this input being timed, so it very likely belongs in
"added during 09-23". §240.7 says which way every mtime-split count here can
move.*

09-22's own list, checked the same way, kept only 1 of 64 (1.6%) as a
persistent corpus member, and that one was already in the corpus since
2026-09-13 — far short of both neighbours' kept fractions (18.8% and 10.9%).

The byte-identity check found no overlap in any of the three pairs: 0 of
09-21's 64 sha1s match any of 09-22's 64; 0 of 09-23's 64 match any of
09-22's 64; and, checked as a free byproduct, 0 of 09-21's 64 match any of
09-23's 64.

### 238.3 What they show for §234.3's reading

**"In the corpus but not reached early" is the common case for 09-21, not a
lucky pick.** §234.3 hashed three names and got one hit; this hashes all 64
and gets twelve — close to a fifth of the list, not a one-off. The reading
stands on firmer ground than a single example, though the mechanism behind
it (an unpinned RNG seed and unlogged `-fork=4` job scheduling meaning a
present corpus member can sit unexplored for a whole run) is still not
confirmed by a direct rerun; that gap is §234.5's, unchanged by this row.

**The same story does not run in the other direction.** Asked directly: of
09-23's kept slow inputs, how many were already available to 09-22, and how
many did 09-22 itself add? **Zero and zero.** None of 09-23's 7 persisted
slow inputs existed before 09-22's night, and none were deposited into the
corpus during 09-22's own run — all 7 first appear no earlier than 09-23's
own overnight run, one not until the night after that. Whatever made 09-22
fast, it neither inherited a landmine from 09-21 that 09-23 later rediscovered
as its own, nor handed 09-23 one of its own making. The two nights differ in
kept-slow fraction too (09-22 kept 1.6% of its 64, against 09-21's 18.8% and
09-23's 10.9%), consistent with 09-22 spending less of its run producing
inputs odd enough to be worth keeping at all — not established as a general
pattern here, just observed on this trio.

This leaves what §234.5 already named open (the rerun-under-a-different-seed
check, and leg (c)'s code-diff replay) untouched, plus one small new loose
end: the one 09-23 slow input whose corpus copy is dated a night later than
the run that measured it as slow, not chased further here.
