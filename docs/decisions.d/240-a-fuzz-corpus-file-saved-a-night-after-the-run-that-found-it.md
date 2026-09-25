## 240. A fuzz corpus file saved a night after the run that found it — 2026-09-25

*§238.2's loose end, chased: one of 09-23's 64 slowest fuse inputs has a
corpus copy dated twelve minutes into the *next* night. Read-only throughout;
no fuzz binary was run for this row.*

  240.1   The file, and the two mtimes that disagree
  240.2   Nothing in the recipes touches the corpus; libFuzzer's merge is the
          only writer
  240.3   The write has no existence check, so it can rewrite unchanged content
  240.4   The 22:22:22Z write, traced to a real merge line in that night's log
  240.5   The same input, very likely already merged the night before
  240.6   A cluster, not a one-off: the whole corpus shows the same pattern
  240.7   The plain answer, and which counts elsewhere could move

### 240.1 The file, and the two mtimes that disagree

Two copies of the same bytes, checked with `sha1sum` on both and found
byte-identical (`412172101e74cf7bf4c1bbeea31e503a333f1991`):

- `/home/mattias/rcad-fuzz-logs/2026-09-23/timings/two_solids_fused_by_a_stranger/slowest/slow-000003204424us-c95269bc5dc2cc1c`
  — mtime `2026-09-23T04:45:33Z` (`stat -c '%Y %y'`).
- `/home/mattias/rcad-fuzz-corpus/two_solids_fused_by_a_stranger/412172101e74cf7bf4c1bbeea31e503a333f1991`
  — mtime `2026-09-23T22:22:22Z`.

The first is written by this repo's own code
(`crates/rcad-fuzz-timing/src/lib.rs`, read in full), the second by libFuzzer.
They are not the same file with two names for the same event — they are two
independent writers, so the fact that one mtime sits eighteen hours after the
other is a real disagreement to explain, not a rounding artefact.

### 240.2 Nothing in the recipes touches the corpus; libFuzzer's merge is the only writer

`mise exec -- just recipe fuzz-overnight` and `mise exec -- just recipe
fuzz-target` (both read in full, justfile:3500 and justfile:3946). Neither
merges, minimises, copies or touches the corpus directory. `fuzz-overnight`'s
`copy_out` (justfile, "the input a target found is kept beside the log")
copies `fuzz/artifacts/` — a different, gitignored directory for crash-class
finds — never the corpus. `fuzz-target` only `mkdir -p`s the corpus directory
before handing it to libFuzzer as its first positional argument
(justfile:3946-3973); after that, libFuzzer owns it.

So whatever wrote `22:22:22Z` is libFuzzer itself. Under `-fork` (this repo
always runs with `-fork` at night, `fuzz-target`'s `fork_flags`), the one place
that writes a new file into the main corpus directory is
`GlobalEnv::RunOneMergeJob` in
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.13/libfuzzer/FuzzerFork.cpp`,
lines 238-241:

```
auto U = FileToVector(Path);
auto NewPath = DirPlusFile(MainCorpusDir, Hash(U));
WriteToFile(U, NewPath);
```

This runs once per completed fork job, for every file in that job's own
temporary corpus that the job's `CrashResistantMerge` found to carry features
not yet in `Env.Features` (the run's own running tally, built up since that
run's own start — FuzzerFork.cpp:229-260).

### 240.3 The write has no existence check, so it can rewrite unchanged content

`WriteToFile` (`FuzzerIO.cpp:80-86`, read in full):

```
void WriteToFile(const uint8_t *Data, size_t Size, const std::string &Path) {
  FILE *Out = fopen(Path.c_str(), "wb");
  if (!Out) return;
  fwrite(Data, sizeof(Data[0]), Size, Out);
  fclose(Out);
}
```

`fopen(..., "wb")` truncates and rewrites whatever is already at `Path`. There
is no `stat`, no "does this hash already exist" check, anywhere between
`RunOneMergeJob` and this call. Since `NewPath` is `MainCorpusDir/Hash(U)`,
writing the same bytes a second time writes to the exact same path — same
name, same content, new mtime.

Each night is a fresh process with a fresh `Env.Features`, built by replaying
every file currently in both corpus directories through **that night's own,
freshly rebuilt** binary (`FuzzWithFork`'s startup `CrashResistantMerge`,
FuzzerFork.cpp:342-354; the binary itself is rebuilt once per night by
`fuzz-overnight`'s `cargo +nightly fuzz build`, before any target starts). If
a job's own subset of files ever registers a piece of already-present content
as carrying a feature that run's own tally has not yet recorded — plausible
across a rebuilt binary, and unconditionally true for a file this run has not
yet replayed at all when the job's merge runs — that content gets written
again, at whatever moment this run happens to touch it, however old the bytes
actually are.

### 240.4 The 22:22:22Z write, traced to a real merge line in that night's log

`/home/mattias/rcad-fuzz-logs/2026-09-24/fuzz-fuse.log` is the night that
starts `2026-09-23T22:10:00Z` (the timer's 22:10 UTC / 00:10 CEST fires into
a directory named for the *next* calendar date, per §238.1's convention).
Its own `.started`/`build.log` mtimes and its "Running" line agree the process
started `2026-09-23T22:11:25Z`. Its log carries:

```
#40688: … corp: 3611 … time: 654s job: 33 …
#41928: … corp: 3612 … time: 656s job: 32 …
```

`22:11:25Z + 656s = 22:22:21Z`, one second off the corpus file's own
`22:22:22Z` — inside the resolution of a `stat` mtime versus a status line
that only prints whole seconds. `corp` genuinely goes up by one right there
(3611 → 3612), so this is not a stray restat of an idle file: that night's own
process ran a real merge at that exact moment and wrote one file. Given
§240.1 and §240.3, there is nothing that distinguishes "this run just
discovered this content for the first time" from "this run just rewrote
content that was already sitting there" — the log cannot tell them apart, and
the file on disk cannot either, because `fopen(..., "wb")` leaves no trace of
what it overwrote.

### 240.5 The same input, very likely already merged the night before

`/home/mattias/rcad-fuzz-logs/2026-09-23/fuzz-fuse.log` is 09-23's own run,
started `2026-09-22T22:11:08Z`. The timing sample that clocked this content
as slow, `2026-09-23T04:45:33Z`, falls `23665s` after that start — squarely
inside a two-line gap in that night's own log:

```
#4110032: … corp: 3864 … time: 23635s job: 441 …
#4114430: … corp: 3865 … time: 23763s job: 442 …
```

`corp` goes up by one between `t=23635s` and `t=23763s` (job 442's merge, at
process-start-plus-23763s ≈ `2026-09-23T04:47:11Z`) — a real "one file added"
event, about a minute and a half after the exact second this content was
executed and timed. The log does not print which file a merge added, so this
is not identity proof, but it places a genuine corpus-growth event for this
target within ninety seconds of this content's own execution, on the night
before the mtime we actually observe. The straightforward reading is that this
input was already a corpus member roughly eighteen hours before its recorded
mtime, and night 09-24 (§240.4) rewrote it rather than discovered it.

### 240.6 A cluster, not a one-off: the whole corpus shows the same pattern

If §240.3's mechanism fires most nights, mtimes across the whole corpus
should cluster in the first few minutes after each night starts — the window
where a fresh run's `Env.Features` is still catching up with what earlier
nights already put on disk. `find /home/mattias/rcad-fuzz-corpus/two_solids_fused_by_a_stranger
-maxdepth 1 -type f -printf '%TH:%TM\n'` over all 10 038 files currently
there, bucketed by clock minute: the 21-minute window `00:09`-`00:30` local
(the first twenty minutes after the timer's `00:10` start) holds **371**
files. Spread evenly over the corpus's history that window would hold on the
order of a dozen (10 038 files over roughly two weeks of clock time is well
under one file a minute on average). Thirty times the flat rate, landing
exactly where §240.3 predicts a rewrite burst, is not proof each of the 371
is a rewrite rather than a true first find — the file's own name still says
nothing about which — but it says the mechanism traced on one file is not a
one-off coincidence.

### 240.7 The plain answer, and which counts elsewhere could move

**A corpus file's mtime is not always the time its content was first kept.
It is the time some run's merge step last decided that content was new to
it — which is usually the same moment, but not always.** libFuzzer's
fork-mode merge (`RunOneMergeJob`, FuzzerFork.cpp:238-241, via `WriteToFile`,
FuzzerIO.cpp:80-86) truncates and rewrites its target path with no check for
whether a file of that name already exists, and every night starts that
merge's feature tally from zero. The named exception is exactly this: a file
already in the corpus, replayed by a later night's freshly built binary or
simply not yet re-seen by that night's own tally when its merge runs, can be
written again — same name, same bytes, later mtime.

**The move is one-directional.** A rewrite only ever pushes an mtime later;
nothing in this path moves one earlier, and a file whose features are already
known to a run's own tally is never touched. So wherever a downstream count
splits corpus files into "before a cutoff" and "added during/after it" by
`find -newermt`-style mtime comparisons, the "before" bucket is a possible
undercount and the "added during/after" bucket a possible overcount — never
the reverse.

Two counts split their input sets exactly that way:

- **§227.4** splits by `-newermt <night's start>`: 1 525 cone-or-torus inputs
  "before 09-20" against 163 "added during 09-20", and 8 951 fuse inputs
  "before 09-18" against 2 675 "added from 09-18 to 09-25". Any of those
  "added" counts could include content that is really older than its bucket
  says, carried forward by a rewrite. Leg (b)'s conclusion — that a night's
  newly-arrived inputs are far slower than its old ones (about 5 000× for the
  cone, 36× for the fuse) — would, if anything, be a floor rather than a
  ceiling: old, unremarkable content wrongly counted as "added" would be fast,
  not slow, so its presence pulls the reported "added" averages (962 ms,
  295 ms/160 ms) down, not up. Not recomputed here.
- **§238.2**'s four-way split (never kept / in the prior night's seed pool /
  added during / added later) rests on the same mtime cutoffs, and this row's
  own file is one of its rows: the 7th of 09-23's "added later" inputs is the
  file traced in §240.4-§240.5. §240.5's evidence points to it having really
  been a same-night (09-23) addition, which would move it from "added later"
  into "added during 09-23," not change 09-21's or 09-22's counts. That is
  one file moving one column in one night's row — not a re-run of §238.2's
  counts, and not evidence that any of its other 63+63+63 classifications are
  wrong.
