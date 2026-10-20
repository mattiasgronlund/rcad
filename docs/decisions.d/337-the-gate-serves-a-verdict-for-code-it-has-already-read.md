## 337. The gate serves a verdict for code it has already read — 2026-09-28

*Mattias pasted another agent's design for a faster build and CI, at
`~/optimized_build_and_ci.txt`, and said "I think it should be feasible for this
repo also". The design has three parts: work out which components a change
touches, keep a verdict cache keyed on a hash of each component's inputs, and
carry those verdicts to CI over a ref, so CI does not redo what a developer
already did. This section builds the second part, for one component: all the
code. It defers the third, and says why in §337.7.*

The first part is not built either. The repeats measured in §337.1 are whole
repeats of the code, and one component catches every one of them. Whether a
split by crate would catch more was not measured, and nothing needs it yet
(§13).

### 337.1 What repeats, and what a hit saves

`.git/rcad-greens` held 247 green gates, from 2026-09-26 18:14 to 09-28 15:51.
None of them gated the same whole tree twice. But **117 of them, 47%, gated
code that had already gated green**: the same tree once `docs/**` and `*.md`
are taken out. Most are mergers gating again after `main` moved by a
`docs/queue.md` push.

The method, since the scripts that ran it lived in a gitignored `target/`:
replay the file in order, name each sha's code as

```sh
git ls-tree -r <sha> | awk -F'\t' '$2 !~ /^docs\// && $2 !~ /\.md$/' | sha256sum
```

and count a green as a repeat when an earlier green had the same name. The
repeats came a median **19 minutes** after the first green of that code. The
90th percentile was 100 minutes and the longest 589. 99 of the 117 came within
an hour, and **all 117 within a day**. So a day is how long a verdict is
served.

What a hit saves comes from timing each gate step once, alone, in a fresh
worktree on 2026-09-28. kache already serves every compile, so the compiling
steps are cheap. The gate took 256.7 s, and the nine steps §337.2 allows to be
served took **213.1 s** of it:

| step | seconds |
| --- | --- |
| `check-fuzz-overnight-selftest` | 87.8 |
| `test` | 69.2 |
| `check-queue-api-selftest` | 29.5 |
| `check-ignored` | 7.8 |
| `lint` | 4.7 |
| `check-wildcards` | 4.7 |
| `check` | 4.2 |
| `check-fuzz-builds` | 3.7 |
| `check-release-build` | 1.5 |

The two slow selftests are most of it. Serving only the compiling steps in
`gate_metered_steps` would have saved about 88 s.

### 337.2 Which steps may be served

**A step may be served only if nothing it reads is prose, or outside the tree.**
Anything else it reads would be an input the fingerprint in §337.3 cannot see.
The steps that may be served are listed in `gate_reusable_steps`. A step that is
not on the list runs every time, so a new step is never served until someone
adds it.

Three checks settled the list:

- **An strace of every gate step**, `strace -f -e trace=%file`, one run each on
  2026-09-28, looking for files opened under `docs/`, any `.md`, and anything
  under `$HOME`.
- **A grep of each candidate's recipe text** for `docs/` and `.md`. A read
  through git, such as `git show HEAD:docs/...`, opens `.git/objects` and does
  not show up in strace.
- **A grep of the Rust under `crates`, `tooling` and `fuzz`** for `.md` and
  `docs/` outside comments, and for every file read.

For the nine steps on the list:

- Cargo stats `crates/*/README.md`, because a package may name a README. No
  crate has one: `ls crates/*/README.md` finds nothing. Cargo hands rustc a
  README's path, never its text.
- `lint` runs `taplo`, which walks the tree calling `stat`. `strace -e
  trace=openat,open,read taplo fmt --check` opens no `.md` file, only the 14
  `.toml` files.
- `check-queue-api-selftest` writes and reads `docs/queue.md` and
  `docs/decisions.d/`, but only in the scratch repository it builds under
  `target/`.
- `check-wildcards` names `docs/decisions.md` in a message and nowhere else.
- The Rust names `docs/decisions.md` in seven messages that it prints or
  panics with, and opens no prose. The files it opens are the STEP corpus (§337.3) and the
  seeds under `fuzz/corpus/`. Those are tracked and not ignored, so a new seed
  there makes the tree dirty, and a dirty tree is never served.

Left off the list, and why:

- **`check-hooks`** runs `{{guardrails_plugin}}/bin/selftest.sh`, which is
  outside the tree. That costs 6.2 s a gate.
- **`deny`** fetches the advisory database.
- **`check-corpus`** reads the STEP corpus, which is outside the tree. It also
  has to run every time for §337.3's exclusion of `RCAD_STEP_CORPUS` to be
  sound.
- **`check-fuzz-unit`** reads the installed systemd unit.
- **`check-plugins`** reads the plugins installed on the machine, and
  **`check-hub`** reads the hub checkout.
- **Every step that reads `docs/` or a `.md`**, such as `check-decisions`,
  `check-plan`, `check-queue`, `check-where-the-work-is`, `check-skills` and
  `check-lanes-workflow`.

`check-gate-verdicts-selftest` fails if the list names anything that is not a
gate step, and if it names any of thirteen of the steps above.

### 337.3 What the fingerprint covers

`_code-fingerprint` prints a sha256 of two things.

**HEAD's tree without `docs/` and `*.md`**, from `git ls-tree -r --full-tree
HEAD`. That lists every other file's mode and blob, so a change of content or
of the executable bit moves it. The `justfile`, `mise.toml`,
`rust-toolchain.toml`, `.cargo/config.toml` and `Cargo.lock` are all in it, so
a changed recipe, tool version or lock moves it too. The two patterns are
exactly `ci.yml`'s `paths-ignore: ['docs/**', '**.md']`, where `docs/**` means
the `docs` at the root only. That is the claim §172.3 has made for every docs
push since 2026-09-20: prose is not an input to a build or a test. The
selftest fails if either workflow's list changes, so the two claims cannot
come apart.

**Some environment variables**, as sorted `NAME=value` pairs:

- `RCAD_*`: every variable the tests read starts with `RCAD_`. A grep of
  `env::var` across `crates` and `tooling` finds `RCAD_SEARCH_SAMPLES`,
  `RCAD_SOAK_FACTOR`, `RCAD_SOAK_SEED`, `RCAD_FUZZ_TIMINGS`,
  `RCAD_FUZZ_SLOWEST` and `RCAD_STEP_CORPUS`.
- `PROPTEST_*`: since §339, `PROPTEST_RNG_SEED` decides what every property
  draws. A soak's shell sets it, and a gate run from that shell must not be
  served a verdict for seed 0.
- `NEXTEST_*`: `NEXTEST_RETRIES` can turn a red test green.
- `RUSTC` and `RUSTUP_TOOLCHAIN`, which override `rust-toolchain.toml`.
- `RUSTFLAGS`, `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `CARGO_BUILD_*` and
  `CARGO_PROFILE_*`, which change what is compiled.
- `RUST_MIN_STACK`, which moves where a deep recursion overflows.

The first plan named only the first two and the flags. The rest were added
while writing it, for the reasons given against each.

**`RCAD_STEP_CORPUS` is left out.** It says where the corpus is, not what is in
it. What is in it is pinned by `corpus/manifest.sha256`, which is in the tree,
and `check-corpus` fails on a missing, altered or extra `.stp` or `.step` file.
`check-corpus` is never served, so a changed corpus fails the gate whatever the
fingerprint says.

**A tree with anything uncommitted or untracked gets no fingerprint.** The name
is read from HEAD, and an untracked `.rs` file under `crates/*/tests/` gets
compiled anyway. `git status --porcelain` does not list ignored files. The
ignored paths are `target/`, `corpora/`, `*.profraw`, `.claude/worktrees/`,
`__pycache__/`, and `fuzz/artifacts/`, `fuzz/coverage/` and `fuzz/target/`. Of
those, a served step reads only `fuzz/artifacts/` (§337.5).

### 337.4 Serving, recording and the audit

**The store** is one file per fingerprint, in
`<git common dir>/rcad-verdicts/<fingerprint>`. The common dir is the hub's
`.git`, so every worktree on the machine shares it. That is the point: the
repeat is usually a merger's tree gating code that a lane's tree already gated.
A record has four kinds of line: `commit`, `date` (UTC), `host`, and the
summary's `tests:` and `doc tests:` lines. It is written to a temp file and
moved into place. Records older than a week are removed whenever a new one is
written.

**Serving.** At the start, if the run has any reusable step and a record for
this code exists, the gate copies the record next to its log and reads the
copy. It reads a copy because a violation in another worktree can delete the
record before this run's summary reads it. It then serves nothing, and says
why, when any of these holds:

- the record cannot be read;
- the second word is `fresh`;
- the record is older than a day;
- HEAD's sha starts with `0`. That is the audit.

Otherwise it skips the reusable steps and prints `· served: <steps> from
<sha>'s gate at <date>`. Every other step still runs. The summary prints the
record's `tests:` and `doc tests:` lines with `(served from <sha>)` after them.
It must not print `tests: not reached`, because that describes a gate that
died early.

**Recording.** A run writes a record only when all of these hold:

- the gate passed;
- nothing was served;
- every step in `gate_reusable_steps` was in this run's list, so `gate-docs`,
  which re-enters `gate` with the compiling steps taken out, never records;
- the fingerprint is the same at the end as at the start, so the tree did not
  change while the gate ran.

A served run records nothing. So a record's date is always the date of a run
that actually ran the steps. Nothing is ever served on the strength of a run
more than a day old.

**The audit** runs everything on a commit whose sha starts with `0`, which is
one commit in sixteen. It uses the sha rather than a random draw for §339's
reason: the same commit gated twice should do the same thing. A commit that is
audited is audited every time. An audit that passes writes the record again
with a new date.

**`_record-green` is unchanged.** It is handed the configured `gate_steps`, not
the steps that ran, and a served gate is a real green for its sha. Its code
passed every reusable step on this machine within the day, in the same
environment, and every other step ran now.

### 337.5 What the fingerprint cannot see

- **The toolchain and the mise tools**, which are pinned by version, not by
  content. A tool reinstalled at the same version is taken to be the same tool.
- **The system's `bash`, `awk`, `sed`, `python3`, `git` and coreutils.**
- **Config outside the tree**: `~/.gitconfig`, `~/.config/kache/config.toml`,
  and whatever cargo, rustfmt, clippy and nextest look for in `$HOME` and in
  the tree's parent directories. On 2026-09-28 `ls` of
  `~/.cargo/config.toml`, `~/git/.cargo`, `~/git/mattiasgronlund/.cargo`,
  `rcad-worktrees/.cargo`, `~/.config/nextest`, `~/.config/rustfmt`,
  `~/.rustfmt.toml`, `~/.clippy.toml`, `~/.kache.toml` and `~/git/.kache.toml`
  found none of them.
- **kache's cache.** A wrong compile served by kache is already trusted by
  every gate, fresh or not. The verdict cache adds nothing to that risk.
- **The clock.** `check-fuzz-overnight-selftest` drives a stub with sleeps and
  names directories by `date`.
- **The ignored `fuzz/artifacts/`.** The same selftest writes there, and is
  written to accept whatever is already there ("at least five, and not exactly
  five").
- **`HashMap` and `HashSet` order**, which §339.4 leaves unaudited.

The store is per machine, which holds most of these still between a record and
the gate that is served it. The audit catches the rest, one commit in sixteen.

### 337.6 A violation

A violation is a reusable step failing on code that has a record, when nothing
was served: an audit, a `fresh` run, or a run past the record's day. **Same
code, same environment, different answer.** Since §339 the seed is fixed, so
the cause is one of two things. Either an input is missing from the
fingerprint, and §337.5 is where to look first. Or the step is not
deterministic: a timing-dependent test, a race, or §339.4's hash order.

The gate prints three lines starting `!!! verdict violated`, which cite this
section. It appends one line to `<git common dir>/rcad-verdict-violations`:

```text
<date> <fingerprint> <failed step> <this commit> passed-at=<the recorded commit> <host>
```

and deletes the record, so nothing serves it again.

**It is a finding, not noise.** Do not re-run the gate until it goes green.
The recorded commit and this one have the same code, so run the failing step
on both, and look for what differs between the two runs. A full disk or a
killed process shows up here too, and the log's tail says so. When the cause
is found, write it down in this section. Then either add the input to
`_code-fingerprint`, or take the step off `gate_reusable_steps`.

**2026-10-20, `check-land-selftest`: a race, not a missing input.** Q2395's
lander gate at `05ba225e` failed the step 38 minutes after `efbbf546`'s gate
recorded a pass for the same code. The arm `dead-entry` printed `land: still
running: a holder has the land lock`, so `wait-for does not name the run that
waits to land lane-y`. That is the race Q1886 measured: the arm sends SIGSTOP
to the run waiting on the land lock and frees the lock at once, and a waiter
that has not stopped yet takes the lock first. With every process on one CPU
the stopped waiter held the lock in 145 of 200 rounds. The step stays on
`gate_reusable_steps` all the same. Served, it runs only when the code
changes; taken off, it would run the race in every gate. Q1886 makes the arm
wait until the waiter has stopped.

### 337.7 The CI half, deferred

The design's third part would carry a laptop's verdict to CI, so that CI skips
what was already checked. It is not built, because it would not bring CI back.

**`ci.yml` has been off since 2026-09-20 11:58 UTC**, and nothing recorded
that until now. `gh api repos/mattiasgronlund/rcad/actions/workflows` gives
its state as `disabled_manually`, with `updated_at` 2026-09-20T13:58:37+02:00.
That is 26 minutes after its last run, at 11:32 UTC, and the same day §172
found the allowance spent. `ci-docs.yml` is still on. `gh run list --created
'>=2026-09-21'` shows 291 runs, all `CI (docs)`, and all failing in 3–7 s
because no job can start.

**Per-push CI does not fit the allowance at this rate, however short the job.**
There were 86 pushes on 09-26, 88 on 09-27, and 63 by 16:00 on 09-28, against
5–23 a day on 09-21 to 09-25. GitHub bills each job rounded up to a whole
minute, so the docs arm's 1m07s bills 2. At 86 pushes a day, even a job of
under a minute bills 86 × 30 = 2580 minutes a month, against 2000. Making each
run shorter is all the transport does, so it cannot fix this. Only fewer runs
can (a schedule, or §172.6's batching), or minutes that cost nothing (a public
repository, or a self-hosted runner), or paying.

**A nightly fresh gate fits without any transport.** §172.1 measured a run at
about 13 minutes, which is about 390 minutes a month. That figure is from
2026-09-19 and was not measured again. The suite has grown since.

**Serving a laptop's verdict on CI would throw away what a runner is for.** A
runner is a fresh machine. §77.6 and `gate-fresh` exist because a check can
pass for a reason in the environment and not in the tree. A verdict made on
the laptop carries the laptop's environment with it.

There is **no signature** in this design, and there would be none in the CI
half. This is one private account, so anything that can write a ref can also
edit `ci.yml`, and a signature would add nothing.

**The shape, if the CI half is ever built:** a `pre-push` hook pushes the
record as `refs/rcad-verdicts/<fingerprint>`, and a CI step fetches it into
`.git/rcad-verdicts/`. The gate then serves it unchanged. It should be keyed on
the fingerprint, not the tree id, so that a push that only moves
`docs/queue.md` still hits. A runner's corpus need not sit where the laptop's
does, which is one more reason `RCAD_STEP_CORPUS` is left out of the
fingerprint.

Whether CI comes back, and how, is Mattias's call, because it turns on money or
on making the repository public. For the record: of the 61 first-parent
commits on `main` since 09-26 18:15 that touched code, 51 had that code gated
green on this machine first. Most of the 10 that did not were in the middle of
a push, not at its tip.

### 337.8 The demonstration

**`check-gate-verdicts-selftest`** holds all of the above in a scratch
repository. It points `GIT_DIR` and `GIT_WORK_TREE` there, so the store it
writes is the scratch repository's and never the real one. Its commit dates are
fixed, so every sha in it is the same on every run, and none of its serving
cases turns into an audit by chance. It checks that:

- a commit touching only `docs/` and `.md` files keeps the fingerprint, and one
  touching `crates/a.rs`, a root `docs.rs`, or `crates/docs/b.rs` moves it;
- `RCAD_SOAK_SEED` moves it, and `RCAD_STEP_CORPUS` does not;
- the first gate records, the second is served, and a prose commit on top is
  served the green of the code under it;
- `fresh`, an untracked `.rs` file, a record two days old, and a sha starting
  with `0` each run everything, and say why;
- a failing step that is not reusable leaves the record alone;
- a reusable step failing on recorded code is reported, written down, and its
  record deleted, and the summary block is still the last thing printed;
- a run that left out a reusable step records nothing;
- a served summary carries the recorded counts, marked `(served from <sha>)`,
  and says so if the record it was served from is gone.

`check-gate-runs-every-check` expects 52 `check-*` recipes, up from 51.

**Every mutant was killed: 19 of 19.** Each mutant changed one thing in the
`justfile`, and a selftest failed on every one:

- the `docs/` filter taken out, unanchored, or the `.md` filter taken out;
- `RCAD_STEP_CORPUS` put back into the environment, or the environment dropped;
- the dirty-tree check taken out;
- `fresh` ignored, the day's expiry ignored, or the audit taken out;
- the skip of served steps taken out;
- a violation keeping its record, or any failed step counting as a violation;
- a run with a reusable step left out recording anyway;
- the served summary printing `not reached`, dropping its `(served from …)`
  mark, or printing nothing when its record is gone;
- `check-hooks` put on the reusable list, or a name that is no step put on it;
- `gate-docs` not passing its `gate_log` on (`check-gate-docs-selftest`).

**The real runs**, on this machine on 2026-09-28, with a load average between
32 and 37 from other sessions' lanes:

1. `just gate quiet` on `44d54b8e`, the commit that added this section, ran
   every step: `gate: PASS`, 2675 tests run, 2675 passed, 11 skipped, and 25
   doc tests passed. It printed `· verdicts: this code's green is recorded as
   6cd9f337e69d`.
2. `just gate quiet` on `dd531c7e`, a commit on top that touched only this
   file, was served. It took 1m08s. **But its summary had no counts**, only two
   errors saying `target/last-gate.log.served` could not be read.

The cause was older than the cache. `check-gate-docs-selftest` runs
`gate-docs` with `--set gate_log` pointing into its scratch directory, but
`gate-docs` re-entered `gate` with only `--set gate_steps`. So the nested run
wrote to the default `target/last-gate.log`, which is the real gate's log. It
emptied that log, and now also deleted the copy of the record next to it.
After run 2, `target/last-gate.log` began with the nested run's ``Error:
Justfile does not contain recipe `no-such-step` ``. Every real gate's log has
been cut at that step since `11142675` added it, on 2026-09-20. The counts the
summary reads are written after that step, which is why nobody noticed.

The fix is one `--set gate_log` in `gate-docs`. `check-gate-docs-selftest` now
reads the log it gave the nested run, and `_gate-summary` prints `tests:
served, but the record it was served from is gone` instead of printing
nothing. Both new checks fail without their fix, as the mutants above show.

3. `just gate quiet` on `b2c83780`, the fix merged with `origin/main`, ran
   every step in **5m50s**, with the load average at 33: `gate: PASS`, 2681
   tests run, 2681 passed, 11 skipped, and 25 doc tests passed. It recorded
   `c7091797ee87`. Its log now begins with `check-plugins`' own output.
4. `just gate quiet` on `d7143ba7`, a commit on top that touched only this
   file, was served from `b2c83780` in **51 s**. Its summary carried run 3's
   counts, each line marked `(served from b2c83780)`. The 43 steps it ran all
   passed. It took 7 s more than §337.1's 44 s, which left out the new
   selftest and was timed on a quieter machine.
5. The landing gate, on `e214ddc7`, the branch merged with an `origin/main`
   that had moved a test file, ran every step in **5m34s**: `gate: PASS`, 2682
   tests run, 2682 passed, 11 skipped, and 25 doc tests passed.
6. While it ran, `main` moved again, by a merge that touched only
   `docs/decisions.d/250-*.md` and `docs/queue.md`. So the push was refused as
   not a fast-forward, which is the case §337.1 counted 117 times. The branch
   merged `main` again as `3c92df7b`, and its gate was **served from
   `e214ddc7` in 47.5 s**. That was pushed to `main`.
