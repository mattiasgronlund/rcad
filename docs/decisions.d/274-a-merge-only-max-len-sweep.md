## 274. A merge-only sweep: coverage keeps climbing past 65 536, and the 124 whole files already carry almost all of it — 2026-09-26

*Queue row Q158, from §264.8 items 3 and 4. Off `main` at `b3be45a9`, no code
changed. Four merge-only (`-merge=1`) passes of the built
`no_step_file_makes_the_reader_panic` binary, each starting from an empty
output directory in the scratchpad and merging the STEP corpus
(`RCAD_STEP_CORPUS`, 520 files, 392 005 383 bytes — the same count and byte
total as §264.1) into it, read-only. No fuzzing, no `-fork`, no
`-max_total_time`: each run is the merge alone, and its final `DONE` line is
the number quoted below. Every run set `RCAD_FUZZ_TIMINGS` to the same
directory, so the four are alike in it. That does not keep the timing crate's
own code out of the counts: the build change §267.1 says would do it is not
made (§267.6), so every `cov` and `ft` below includes whatever edges that
crate adds. Run at 12:41–12:47 UTC on 2026-09-26; the
one-minute load average read 27.41 just before the first run and 34.50 just
after the last, because Q148 was fuzzing on this machine throughout — a
merge's coverage does not depend on load, only its wall time does, and every
run here finished in a few seconds regardless.*

  274.1   What was run
  274.2   Coverage keeps rising past 65 536
  274.3   Almost all of it is the 124 whole files
  274.4   Whether §264.7's pin should move

### 274.1 What was run

Four merges, same binary, same corpus, differing only in `-max_len` and, for
the fourth, which files were offered:

| run | input | `-max_len` | new files kept | `cov` | `ft` |
|---|---|---|---|---|---|
| A | full corpus, 520 files | 32 768 | 205 | 8 718 | 15 990 |
| B | full corpus, 520 files | 65 536 | 236 | 10 266 | 30 159 |
| C | full corpus, 520 files | 131 072 | 281 | 10 761 | 40 623 |
| D | the 124 files that fit whole in 65 536 | 65 536 | 106 | 10 107 | 29 132 |

"New files kept" is libFuzzer's own `MERGE-OUTER` count, out of 520 (124 for
D). 46 files fit whole in 32 768 bytes, 124 in 65 536, and 219 in 131 072 —
checked directly against this machine's corpus (`find
/home/mattias/git/mattiasgronlund/rcad-step-corpus -type f -printf '%s\n' |
awk '$1<=N{c++} END{print c+0}'`), and 46/124 agree with §264.8's count. The
corpus's five largest files are 7.5, 8.6, 14.0, 15.9 and 60.3 MB, so even at
131 072 most files are still cut to their first 131 072 bytes rather than read
whole.

*(Added at merge.)* The verifier rebuilt the binary from the same commit and
reran all four merges on the same corpus. Its `cov` and `ft` came out 4% to 5%
below the table's, and two runs of A on its build differed by 8 edges and 30
features, so these integers do not repeat exactly. The shares §274.2 and
§274.3 rest on did, within two points: `cov` as a share of C 79.7% and 94.9%,
`ft` 38.6% and 74.6%, and D as a share of B 97.3% (`cov`) and 96.3% (`ft`).

### 274.2 Coverage keeps rising past 65 536

B is §264.7's pin. It is not close to what this sweep's ceiling (C, 131 072)
reaches:

| | A (32 768) | B (65 536) | C (131 072) |
|---|---|---|---|
| `cov` as a share of C | 81.0% | 95.4% | 100% |
| `ft` as a share of C | 39.4% | 74.2% | 100% |

Going from B to C buys 495 more edges (4.8% on top of B) and 10 464 more
features (34.7% on top of B) — proportionally a bigger jump in features than
32 768→65 536 bought: that step added 14 169 features, 88.6% on top of A's own
15 990. Features climb faster than edges at every step here: a larger
`-max_len` mostly re-exercises edges already found, along different byte
values past the old cutoff, which libFuzzer counts as a new feature rather
than a new edge. Nothing here marks 131 072 as a ceiling either — the corpus
holds files up to 60 MB, and this sweep did not run above 131 072.

### 274.3 Almost all of it is the 124 whole files

D, run over only the 124 files that already fit under 65 536 (so none of them
were cut, unlike in B), reaches:

| D as a share of B (full corpus, same `-max_len`) | |
|---|---|
| `cov` | 98.5% (10 107 / 10 266) |
| `ft` | 96.6% (29 132 / 30 159) |

The other 396 files, cut to their first 65 536 bytes in B, add only 159 edges
and 1 027 features on top of what the 124 whole ones already give alone.
§264.8 item 4's question has an answer at this `-max_len`: the 124 whole files
(24% of the corpus by count) are almost the entire benefit; the 396 truncated
ones are a small remainder.

### 274.4 Whether §264.7's pin should move

**§264.7's 65 536 is not the point where coverage stops growing.** §274.2
shows 131 072 still reaching 4.8% more edges and 34.7% more features on a
merge that took a few seconds either way. But §264.7 did not pick 65 536
because it sat at a coverage ceiling — it picked it against arm B's *rate*,
measured over 900 s of actual fuzzing with `-fork=4`, where the corpus at
65 536 already ran at a tenth of B's per-core rate (§264.2, §264.4), and the
arm that read every file whole ran at 0.3%–0.8% of that rate, with timeouts
and an exit 70 (§264.5). This lane's runs are merge-only and say nothing about
the rate a night of mutation would sustain once every input can grow to
128 KiB instead of 64 KiB.

So this lane does not move the pin, and recommends leaving §264.7's 65 536 as
it stands: the coverage gain at 131 072 is real (§274.2) but unpriced against
rate, and rate is what rejected the larger arm in §264.7 in the first place. A
Q144-style timed pair, run at 131 072 against B, would price it; this row did
not run that.
