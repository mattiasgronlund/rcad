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

Since §337.9, `check-hooks`, `check-questions-selftest`, `check-plugins` and
`deny` are on the list after all, each served off a key of its own.

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

### 337.9 The steps that check the machine, served on keys of their own

*Q693, 2026-10-21: stop running on every gate the steps that check the machine
rather than the tree. Mattias said they "check things that should be validated
when the main agent starts, or when the test is updated". The row recommended
serving each green on a key of what it reads, or running them from a
SessionStart hook. This takes the first.*

**Four of the eight steps are served now**: `check-hooks`,
`check-questions-selftest`, `check-plugins` and `deny`. Each has a key of its
own, a sha256 of three things: the step's name, the code's fingerprint from
§337.3, and what the step reads outside the tree. Its record is
`<git common dir>/rcad-verdicts-machine/<key>`, beside `rcad-verdicts/` and
§425.7's `rcad-verdicts-timing-hooked-calls/`. §337.4's rules hold for it as
they are: served for a day at most, run again under `fresh` and the audit, and
recorded only by a run that ran it and passed. A failure over a record of the
same key is §337.6's violation, written down in the same file.

**What they cost.** Each step alone, once, by `target/q693-time-steps.sh`. The
first column is this lane's tree with the load average at 50. The second is
the first Q693 lane's run on the base, `bb1f44fe`, on a quieter machine.

| step | ms, load 50 | ms, on the base |
| --- | --- | --- |
| `check-hooks` | 32232 | 7969 |
| `check-questions-selftest` | 15466 | 3719 |
| `check-plugins` | 4939 | 1282 |
| `deny` | 2421 | 855 |
| `check-corpus` | 387 | 148 |
| `check-hub` | 257 | 69 |
| `check-systemd-units` | 233 | 66 |
| `check-queue-merge-driver` | 214 | 49 |

In a served `just gate quiet` on the base, `check-hooks` took 8.2 s. The step
that ran longest was `check-lanes-workflow`, at 29.5 s, and it still decides
how long a served gate takes.

**What they read outside the tree.** The first lane ran `strace -f -e
trace=%file` over each step, and `target/q693-opened2.sh` lists what each
opened. It leaves out what every recipe opens: the hub's `.git`,
`~/.gitconfig`, `~/.config/git/ignore`, `~/mise.toml`, `~/mise.lock`,
`~/.config/mise/config.toml`, and cargo's own caches.

- `check-hooks` and `check-questions-selftest` open the plugins' `bin/` and
  `hooks/` under `~/git/mattiasgronlund/claude-questions/plugins`.
- `check-plugins` opens those, and also `~/.claude/settings.json` and
  `~/.claude/plugins/known_marketplaces.json`. Its `check-declared.sh` names
  two more files that this trace did not open: `.claude/settings.local.json`,
  which is ignored and so outside the fingerprint, and
  `~/.claude/plugins/installed_plugins.json`. The key reads them anyway.
- `deny` opens the advisory database's checkout under `~/.cargo/advisory-dbs/`
  and cargo's package-cache locks.
- All four looked for `~/.claude/plugins/marketplaces/mattiasgronlund-local/plugins`
  and did not find it. That path is `marketplace_clone`, and where it exists,
  `plugins_root` points there instead.

So the key adds these to the step's name and the code's fingerprint:

- For the three plugin steps: the path of `claude_config`, and whether it is a
  directory. Then, for each of the three plugin directories, its path and every
  file's type, mode, path and sha256, or `absent`. 34 files take 90 ms
  (`target/q693-keys.sh`).
- For the three plugin steps too, each of the four config files above, as
  `jq -c` prints it with every `lastUpdated` dropped, or its sha256 where `jq`
  cannot read it, or `absent`.
- For `deny`: the path and HEAD commit of each checkout under
  `${CARGO_HOME:-~/.cargo}/advisory-dbs/`.

**Every `lastUpdated` is dropped from the config files.** At first the key
hashed each file whole, and the lane gate at `33ef2659` did not record
`check-plugins`. It printed `· verdicts: check-plugins not recorded, because
what it reads changed while the gate ran`. That gate started at
2026-10-21T19:30:54Z, and during it Claude Code wrote `"lastUpdated":
"2026-10-21T19:38:13.627Z"` into `~/.claude/plugins/known_marketplaces.json`.
It stamps that field whenever it refreshes a marketplace, on its own clock.
`installed_plugins.json` carries a `lastUpdated` in 43 entries. Hashed whole,
`check-plugins` would seldom be served. `check-declared.sh` never reads
`lastUpdated`. It reads `has($m)` and `.[$m].source` of
`known_marketplaces.json`, `.plugins | has($k)` of `installed_plugins.json`,
and `.enabledPlugins` and `.extraKnownMarketplaces` of the settings files. Any
other change to these files still moves the key. A projection onto just the
fields it reads was not taken: when `check-declared.sh` starts to read another
field, the plugin's digest moves once, but the projection would then miss every
later change to that field. Dropping `lastUpdated` has the same blind spot for
that one field. A `check-declared.sh` that starts to read `lastUpdated` needs
the drop taken out.

**The config files are in all three plugin steps' keys, though only
`check-plugins` reads them.** The selftest cannot drive the real
`check-plugins`, since it runs `just question` against the real questions
parser, which a fixture plugin does not have. It drives
`check-questions-selftest` instead, so the config files have to be in that
step's key for the selftest to reach the `lastUpdated` drop. The other way was
a private recipe for the key, which the selftest could call with a fixture
config directory. That recipe was outside this row's write set. Having more in
the key than the step reads is still sound. It costs one more run of
`check-hooks` and `check-questions-selftest` when a settings file or a plugin
install changes.

**`deny` fetches the advisory database on every run.** Over one `just deny`,
`target/q693-deny-fetch.sh` saw `FETCH_HEAD`'s mtime move from 1792610807 to
1792610815, while HEAD stayed `4b587f00`. The first relay saw it move by 504 s.
So what `deny` reads can change while it runs. The gate takes each key again
after its step has ended. A pass whose key moved records nothing, and says
`· verdicts: <step> not recorded, because what it reads changed while the gate
ran`. A failure whose key moved is no violation, since no record passed what it
read. The next gate runs `deny` again, on whatever the fetch left, and records
it.

**Why only four.** `check-corpus`, `check-hub`, `check-systemd-units` and
`check-queue-merge-driver` take a fifth to two fifths of a second each, even at
load 50. A sound key for each would have to read what the step reads: the
corpus's 119 MiB, the hub's status, the installed units, or git's config. It
would cost about as much as the step. They stay on the selftest's list of
steps that are never served, which now names eleven.

**Why the key holds the whole fingerprint, and not the recipe's text.** The
row suggested a key of each recipe's own text and what it reads outside the
tree. But each step reads the tree as well. `check-hooks` runs
`.claude/hooks/*.sh` and `.claude/patching-cases.local.json`, `check-plugins`
runs this justfile's `question` recipe, and `deny` reads `Cargo.lock` and
`deny.toml`. The fingerprint already covers the recipe, every tracked file a
step reads, and the environment variables §337.3 names. A narrower key would
need a trace of what each step reads in the tree, as §425.7 made for
`check-timing-hooked-calls`. None of these four costs enough to pay for that.
The price is that a code commit runs all four again.

**`check-corpus` cannot be keyed on the manifest alone.** The row suggested
it. But `corpus/manifest.sha256` is tracked, so it is in the fingerprint
already, and what `check-corpus` checks is that the corpus on disk still
matches it. A key of the manifest alone would serve a pass over a corpus that
has been changed. So `check-corpus` still runs on every gate, and §337.3's
argument for leaving `RCAD_STEP_CORPUS` out of the fingerprint still stands: a
changed corpus fails the gate, whatever the fingerprint says.

**What CI runs.** `ci.yml` runs `just gate`. A runner's fresh clone has no
store, so it serves nothing. With no Claude Code config, the three plugin
steps print `skipped, not passed`. Their key then says `no config`, so a skip
is never served to a machine that has Claude Code. `ci.yml` has been off since
2026-09-20 (§337.7).

**Why not a SessionStart hook.** It would need a change to
`.claude/settings.json`, which this row does not hold. CI has no session
start. And a hook runs at every session start, whether anything changed or
not, and does not run when a plugin is pulled in the middle of a session. The
key covers both of Mattias's cases. When an agent starts on a machine whose
plugins or config have changed, the next gate's key has moved, and the step
runs. When the test is updated, in a plugin or in the tree, the key moves too.

**What the key cannot see**, beyond §337.5:

- **A hand edit inside the advisory database's checkout.** The key reads its
  commit only, since a fetch leaves 67 entries in that checkout's `git status`
  (`target/q693-db-status.sh`).
- **The cargo registry index**, which `deny.toml`'s `yanked = "deny"` reads.
- **An advisory published, or a crate yanked, since the record.** A served
  `deny` fetches nothing, so it sees one only when its record expires, within
  a day, or sooner when a code change runs it.
- **`jq` and `python3`**, which `check-plugins` and the plugins' scripts run.
  These are §337.5's system tools.
- **`~/mise.toml`, `~/mise.lock` and `~/.config/mise/config.toml`**, which
  every step reads, served or not.

**`__pycache__` is left out of the key.** Python writes bytecode beside a
script it runs, and `questions/bin/__pycache__` is there now. It follows from
the script, and a step that wrote it would move its own key while it ran.
`target/q693-keys.sh` ran each plugin step once, and the key's digest of the
plugins did not move.

**The demonstration.** `check-gate-verdicts-selftest` fails unless the four are
on `gate_reusable_steps`. It drives the real `check-questions-selftest` against
a fixture plugin, through these cases:

- the first gate records it under its own key, and not in the whole-tree
  store;
- the next gate is served it, and its summary does not say that a whole-tree
  record is gone;
- a new `lastUpdated` in the fixture's `known_marketplaces.json` is served it,
  and a marketplace moved to another repository runs it and moves its key;
- a commit that touched only `docs/` is served it;
- `fresh`, an audit, and a record two days old each run it, and say why;
- a failure over its record is a violation, and the record is deleted;
- a code commit runs it, and so does a machine with no Claude Code;
- a pass whose plugin changed while it ran records nothing, and says so;
- the next gate runs it, and records it under a new key;
- a failure whose plugin changed while it ran is no violation, and the record
  of the plugin before is kept.

Three mutants of `gate` were each caught. With the check of whether the key
moved taken out, the selftest failed on "did not say why it recorded nothing".
With the four put back into the whole-tree list, it failed on "was recorded in
the whole-tree store, off the code's fingerprint alone". With the config files
read by `jq -c '.'`, so that `lastUpdated` stays, it failed on "a new
lastUpdated stamp in known_marketplaces.json moved check-questions-selftest's
key".

**The real runs.** `just pre-gate gate-lane` on `a31350d3`, the commit that
drops `lastUpdated`, passed with 4440 tests of 4440. The load average was 15
when it started. It ran all four, `check-hooks` in 20.3 s, `check-questions-selftest` in 13.0 s,
`check-plugins` in 3.8 s and `deny` in 1.7 s. It recorded each on its own key:
`f6446181eab2`, `e0bfa98e535a`, `d29584258d1b` and `8cd83bca2b81`. Unlike the
gate at `33ef2659`, it recorded `check-plugins` too.

Then `aa40b10a` changed only this section, and the same command was served
all four, each on a line of its own such as `· served: check-plugins from
a31350d3's gate at 2026-10-21T20:00:27Z`, beside the whole tree's served line.
None of the four ran. The gate started at 20:01:05Z and wrote its exit at
20:02:03Z, 58 s. The step that ran longest was `check-lanes-workflow`, at 57.0
s, so it still decides how long a served gate takes, as it did on the base.

### 337.10 `check-queue-api-selftest`, served on a key narrower than the tree

*Q3074, 2026-11-02: `check-queue-api-selftest` was keyed on the whole-tree
fingerprint, like every step in `gate_reusable_steps` other than the five
above. So a `docs/queue.md` push served it, but every change to the Rust
kernel — the usual kind of commit — ran it again, for no reason tied to what
it actually reads.*

**What it costs.** Run alone at `51da3d6e` on 2026-11-02, it took 441 s wall,
182 s user and 185 s sys. In the three gate logs found where it ran, the
serial cargo chain beside it (`test`, mostly) took 753 to 1009 s, so
`check-queue-api-selftest` is not what sets a gate's wall time — but it is
about 6 CPU minutes of a gate's total that a narrower key can give back. Of
the 370 commits on `main` from 2026-10-26 to 2026-11-02 that touched anything
outside `docs/` and `*.md`, 310 touched nothing in `justfile`, `tooling/`,
`.gitattributes` or `mise.toml` either — so 310 of 370 non-prose commits were
paying for a run the whole-tree key could not serve, on inputs that had not
moved.

**What it reads.** `check-queue-api-selftest` copies only `tooling/queue/*.py`
and `.gitattributes` into a scratch hub (`justfile:18446-18448` at the time
this was measured) and drives `just queue` against that copy — it reads no
part of the kernel crates at all. A 90 s `openat`-only `strace -f` of a real
run, extended by a second trace into the recipe's own loop of cases before it
was cut off by its own timeout, found nothing opened inside the tree outside
`justfile`, `tooling/`, `.gitattributes`, `mise.toml` and `mise.lock` — and
nothing under `docs/` outside the selftest's own scratch repository. `mise.lock`
is in the key though none of `_timing-hooked-calls-fingerprint`'s four paths
need anything like it: every `python3` the recipe runs resolves through mise's
own shim first, and that shim reads the project's `mise.toml` and `mise.lock`
to pick a version before it ever reaches `tooling/queue/queue.py`.

**The key**, `_queue-api-selftest-fingerprint`: `git ls-tree -r --full-tree
HEAD` restricted to `justfile`, `tooling/`, `.gitattributes`, `mise.toml` and
`mise.lock`, plus the same environment block `_code-fingerprint` takes
(§337.3). Same shape as `_code-fingerprint`, so the two stay easy to compare by
eye; unlike `_timing-hooked-calls-fingerprint`, there is no `--show` of a
recipe to add, since `justfile` is itself one of the paths and a change to
either recipe's own text already moves the key. The record is
`<git common dir>/rcad-verdicts-queue-api-selftest/<fingerprint>`, beside
`rcad-verdicts/` and §425.7's `rcad-verdicts-timing-hooked-calls/`. `gate`'s
own body excludes `check-queue-api-selftest` from the whole-tree `$reusable`
loop the same way it excludes `check-timing-hooked-calls`, and gives it a
lookup block and a write-back block shaped like that step's own, against this
narrower fingerprint and this record directory instead.

**The demonstration.** `check-gate-verdicts-selftest` drives the whole `gate`
recipe against a disposable git repository, through a copy of this justfile
committed into that repository as its own `justfile` — `set
allow-duplicate-recipes` lets a trailing stub `check-queue-api-selftest:` win
over the real, slow recipe, so the test exercises `gate`'s serve and
write-back logic for real without paying for the real recipe's minutes. It
asserts: the first gate runs the step and records it under its own store, not
the whole-tree one; a commit that touches only `crates/a.rs`, outside the
narrow set, leaves the fingerprint where it was and is served the step's
green, without running it again; a commit to `tooling/queue/rows.py`, inside
the narrow set, moves the fingerprint and the next gate runs the step again.

**The real run.** `just pre-gate gate-lane` on this section's own commit
passed, 4689 of 4689 tests (7 slow, 79 skipped), and recorded
`check-queue-api-selftest`'s green under its own store. The step itself ran
once, cold, at 335.0 s — there was no record yet for this exact fingerprint —
beside `test`'s own 844.6 s. A later gate on a commit that moves nothing under
`justfile`, `tooling/`, `.gitattributes` or `mise.toml` is served this step's
green instead of running it again.

### 337.11 The five cargo steps, served on a key of what they read

*Q3076, 2026-11-02: `check`, `check-ignored`, `test`, `test-slow` and
`check-release-build` were keyed on the whole-tree fingerprint, which holds the
justfile and `tooling/`. So Q3053, whose one commit `99cb631e7` changed only
the justfile, ran `test` fresh for 842.3 s in its lane gate, and its land gate
ran it again.*

**What it costs.** `test` took a median of 804 s over the 19 fresh gates of
2026-11-02, as the row counted them. It is the step a gate waits on longest.
§337.9 kept the machine steps on the whole-tree key because none of them cost
enough to pay for a trace of what it reads in the tree. `test` does.

**What they read.** The first lane on the row traced each of the five once,
with `strace -f --seccomp-bpf -qq -s 256 -e trace=%file -e signal=none -o
<file> mise exec -- just <step>`. Each exited 0. `test` took 558.8 s (4694
tests, 79 skipped), `check` 20.1 s, `check-ignored` 54.1 s, `test-slow` 111.0 s
and `check-release-build` 3.1 s. Inside the tree, outside `target/`:

- All five opened only `.cargo/config.toml`, `Cargo.toml`, `Cargo.lock`,
  `crates/`, `rust-toolchain.toml`, `mise.toml`, `mise.lock`, the justfile,
  `tooling/hub-align` and `.git`.
- `test` also read 4736 files under `fuzz/corpus`. Tests in
  `crates/rcad-topo/tests/` and in
  `crates/rcad-step/tests/a_solid_the_writer_harness_judges_alone.rs` reach
  them as `crates/<crate>/../../fuzz/corpus/...`, so the first count of the
  paths filed them under `crates/`. A second count, which resolves the `..`,
  found them.
- `fuzz/step.dict` is read by an `include_str!` in
  `crates/rcad-step/src/names.rs`. The trace saw only a probe of it, because
  kache served the compile.
- They looked for `.config/nextest.toml`, `.config/mise*`, `.mise*`,
  `.tool-versions`, `.kache.toml`, `rust-toolchain` and `crates/*/README.md`,
  and found none of them. These names are in the key, so a commit that adds one
  moves it.
- Nothing under `docs/`, `.github/`, `.claude/`, `corpus/`, `deny.toml`, or
  `tooling/` beyond `hub-align`.

`tooling/hub-align` is read by `hub_aligned := shell(... || true)` at the top
of the justfile, which every `just` runs as it parses the file. No recipe reads
the value, and the `|| true` swallows a failure, so it cannot change what a
step answers. It is left out of the key. No `.md` file is read, since cargo's
`README.md` probes feed no build, so the key drops `*.md` as §337.3 does.

**The key**, `_code-fingerprint rust`. It hashes:

- `git ls-tree -r --full-tree HEAD` of the cargo, toolchain, mise and kache
  names above, found or not, and of `crates`, `fuzz` and `corpus`, all without
  `*.md`. `corpus/` is in for `corpus/manifest.sha256`, which pins what
  `RCAD_STEP_CORPUS` points at (§337.3);
- of the justfile, only what the five run: `just --show` of their recipes and
  of `_slow-tests-filter`, the `set` and `export` lines every recipe gets, and
  the values of `slow_tests` and `ignored_tests`;
- the same environment block as the whole-tree key.

The `--show`s run before the hash, so a recipe that cannot be shown stops the
recipe with no hash printed, rather than giving a hash that left it out. With no
argument `_code-fingerprint` hashes the same bytes it did before, so the
whole-tree records written before this change still serve. Any other argument
exits 2.

The record is `<git common dir>/rcad-verdicts-rust/<fingerprint>`, beside
`rcad-verdicts/` and the stores of §337.9 and §337.10. It has the whole-tree
record's shape, with the steps line that a partial run adds to and that keeps
its date (§397.4). It keeps §337.4's rules: a day, `fresh`, and the audit of a
sha starting with `0`. A failure over a record is a violation, and the record
is deleted (§337.6). `gate` leaves the five out of the whole-tree `$reusable`
loop, and writes both records back through the same two functions,
`record_green` and `record_violation`.

**The counts.** A record holds the counts its steps printed, and one run can
now write two records. So `counts_of` reads only the logs of the record's own
steps, where the whole-tree record used to read the whole log. Only `test`,
`test-slow` and `check-release-build` print counts: a read of three real gate
logs found no other step that does, and the first lane gate on the new key
agrees. Its cargo record, `15b8a2a94404`, holds `test`'s summary,
`check-release-build`'s 3 tests and the doc tests' 30, and its whole-tree record
holds no counts.

`_gate-summary` takes one served record. When the cargo steps were served, the
summary takes theirs, since that is where the counts are. A run served both
records notes each count of the whole-tree record that it left out, as `·
verdicts: not in the summary, which shows the cargo steps' counts: <line>
(served from <sha>)`. No whole-tree step prints a count today, so the note is
for one that starts to. Taking more than one record has no consumer, so it is
not built (§13).

**An old whole-tree record with `test`'s counts is never served beside a run
of `test`.** Before this change the whole-tree record held `test`'s counts,
and a gate that served it while running `test` would show them twice. But the
whole-tree key holds the justfile, and `just gate` runs the tree's own
justfile. So every whole-tree key the new gate takes holds the new justfile,
which no old gate recorded.

**Left on the whole-tree key**: `check-fuzz-builds`, `check-wildcards`,
`lint`, `check-fmt` and `check-t-selftest`. In the first lane gate on the new
key they took 7.4 s, 1.8 s, 0.3 s, 4.9 s and 0.7 s. None was traced, and on
§337.9's argument none costs enough to pay for a trace. `check-fuzz-builds`
builds `fuzz/` with its own `Cargo.lock`, outside the workspace. `lint` is
`cargo clippy`, which reads a `clippy.toml` when there is one, and its 0.3 s
came after `pre-gate` had run it. `check-t-selftest` drives justfile recipes.

**The demonstration.** `check-gate-verdicts-selftest` drives the whole `gate`
against a scratch repository, through a copy of this justfile with fast stubs
for the five under `set allow-duplicate-recipes`. `test` and `test-slow` print
a nextest summary, so the counts can be followed into a record and out of it.
`check-release-build` fails while `RUST_SELFTEST_FAIL` is set, which nothing in
the key reads. It asserts:

- `_code-fingerprint rusty` exits 2 and names what it refused;
- the first gate records the five under their own store and not the whole-tree
  one, with the steps line and `test`'s counts;
- a commit that adds a comment to the justfile moves the whole-tree key and
  not this one, is served all five, and shows `test`'s counts as served;
- a commit to `crates/a.rs` runs them, and so does one that changes `test`'s
  recipe;
- `Cargo.lock`, `.cargo/config.toml`, `.config/nextest.toml`,
  `rust-toolchain.toml`, `mise.toml`, `fuzz/step.dict`, a file under
  `fuzz/corpus`, `corpus/manifest.sha256` and `crates/b/src/lib.rs` each move
  the key, and `tooling/a.py`, `docs/notes.txt`, `crates/x/NOTES.md`,
  `.github/workflows/ci.yml` and `deny.toml` each leave it where it was;
- `RCAD_SOAK_SEED` moves it, and `RCAD_STEP_CORPUS` does not;
- `fresh`, a record two days old and a sha starting with `0` each run them, and
  say why;
- a run that leaves out `test-slow` records the four it ran, and the next run
  is served those four, runs `test-slow`, shows both counts, and adds it to the
  record's steps line;
- a failure of `check-release-build` over its record is a violation: the record
  is deleted and the violation is written down;
- one run that writes both records puts in each only its own steps' counts,
  and the next run, served both, notes the whole-tree count it left out of the
  summary.

The selftest failed before each part it checks was there. At `f993d78fd`, with
the cases and no key, it failed on ``Justfile does not contain recipe `rust` ``.
At `92a0187fc`, with the both-records case and no note, it failed on "a gate
served both records left the whole-tree record's counts out without saying
so".

The selftest's fixture commits each hold a copy of the justfile, and their
dates are pinned, so their shas follow from the justfile's bytes. About one
justfile in sixteen gives a sha starting with `0`, and a case that needs a step
served then gets an audit instead. The lane gate on `d1c36af40`, which changed
only comments, failed that way, on "the cargo steps' fixture commit starts with
0". So `commit_served` amends the message of each commit whose case needs
serving until its sha does not start with `0`.

**The real run.** Three `just pre-gate gate-lane` runs on the row's branch, on
2026-11-02:

- On `15e1bfe71`, the first gate on the new key, with no record for it yet.
  `test` ran in 417.5 s (4696 tests, 13 slow, 79 skipped), `check` in 19.2 s,
  `check-ignored` in 48.2 s and `check-release-build` in 1.2 s. It passed and
  recorded the four as `15b8a2a94404`. `gate-lane` leaves `test-slow` to the
  land gate (§397.4), so the steps line names four.
- `d1c36af40` changed only comments in the justfile, outside the five's
  recipes. `_code-fingerprint rust` gave the same `15b8a2a9440436fe…`, and the
  whole-tree key moved. Its gate printed `· served: check check-ignored test
  check-release-build from 15e1bfe7's gate at 2026-11-02T22:56:30Z`, and failed
  only on the selftest's sha above.
- On `1776e22e1`, which changed only the selftest, the gate was served the same
  four and passed in 286 s, from 23:04:22Z to 23:09:08Z, at a load of 3.03 to
  7.31.

On a commit like these two, the longest step is now `check-queue-api-selftest`,
at 290.6 s and 284.1 s, since §337.10's key holds the whole justfile.

**What this leaves.** One row: key `check-queue-api-selftest` on the justfile
recipes it runs rather than the whole justfile, as `_code-fingerprint rust`
does for the five, so that a commit that changes only the justfile is served
it too.

### 337.12 `check-queue-api-selftest`, keyed on the part of the justfile it runs

*Q3086, 2026-11-03: §337.10 keyed `check-queue-api-selftest` on the whole
justfile, beside `tooling/`, `.gitattributes`, `mise.toml` and `mise.lock`. So
once §337.11 served the cargo steps to a commit that changes only the
justfile, this step was the longest of such a gate: 290.6 s and 284.1 s in the
two lane gates of 2026-11-02 that §337.11's real run names.*

**What it runs.** The first lane on the row traced one real run on the base
`219aed7c5`, with `strace -f --seccomp-bpf -qq -s 512 -e trace=execve -e
signal=none -o <file> mise exec -- just check-queue-api-selftest`. It took
about ten minutes under the trace and exited clean. Its 2216 `just` processes
ran ten recipes and nothing else: `queue` 551 times, `_queue-rows` 465,
`queue-page` 261, `check-queue` 220, `_queue-check` 220, `decisions-all` 220,
`_triage-refusal` 8, `_status-triage` 8, `queue-merge-driver` once, and
`check-queue-api-selftest` itself once. The other 261 were `just --evaluate
queue_bundle`, which `tooling/queue-page/render.py` runs. A word scan of the
justfile for the recipes each one calls was tried first and is not the list:
it took `check`, `status`, `fuzz` and `t` from shell names such as `check()`
and `status=$?`.

**What it reads of the justfile.** `just --show` prints a recipe's comments,
attributes and body, and a body prints `{{ queue_file }}`, not what
`queue_file` holds. The ten bodies read eleven assignments, found in the
expression trees `just --dump --dump-format json` gives of them:
`decisions_dir` in `decisions-all`, and `fuzz_log_root`, `queue_file`,
`queue_slots`, `queue_bundle`, `queue_shared_files`, `queue_running`,
`queue_now`, `queue_threads`, `queue_page_dir` and `queue_merge_driver` in
`queue`, `queue-page`, `check-queue`, `_queue-check` and `queue-merge-driver`.
None is built from another assignment. The justfile reaches the step two other
ways:

- Every recipe gets the settings, and the exported assignments in its
  environment, and the Python the step runs reads its environment.
- `just` 1.42.4 evaluates every assignment before it runs any recipe. A probe
  justfile holding `` x := `false` `` and a recipe that never reads `x` fails
  with "Backtick failed with exit code 1". The step's own `just` calls run
  with `--working-directory` set to its scratch hub, so an assignment no
  recipe reads can stop the step there and not in the tree: a backtick, a
  `shell()`, or a function that looks for a file where `just` runs. A probe
  justfile holding only `x := read("here.txt")` ran its recipe from the
  directory holding `here.txt`, and from another failed with "Call to function
  `read` failed: I/O error reading `here.txt`".

**The key**, `_queue-api-selftest-fingerprint`. It hashes:

- `git ls-tree -r --full-tree HEAD` of `tooling`, `.gitattributes`,
  `mise.toml` and `mise.lock`: §337.10's paths without the justfile;
- of the justfile, `just --show` of the ten recipes, then from `just --dump
  --dump-format json` the settings, the unexports, the definitions of the
  eleven, and every assignment that is exported or that calls a function or
  runs a backtick anywhere in its expression, and every assignment at all
  under `set export`;
- the same environment block as before.

A definition, not a value: `queue_page_dir` and `fuzz_log_root` name
`RCAD_QUEUE_PAGE_DIR` and `RCAD_FUZZ_LOGS`, which the step sets to its scratch
directory, so the value the gate's own `just` gives them is not the one the
step reads, and renaming the variable leaves that value the same. The JSON
dump and not the text one, because the text dump prints a string as it is
written, so a definition written over two lines takes two lines of it, and the
line that names it holds only the first.
Today the rule takes twelve assignments, two of them among the eleven:
`hub_aligned`'s `shell()`, `questions_root`'s `` `id -u` ``,
`RCAD_STEP_CORPUS`'s backtick, and nine that call `env_var`,
`env_var_or_default`, `home_directory` or `path_exists`.

The `--show`s and the dump run before the hash, as in §337.11, so a recipe
that cannot be shown, or one of the eleven that is no longer defined, stops the
recipe with no hash printed. This narrows §337.10's key, which held the
justfile whole. A commit that changes another recipe, a comment outside the
ten, or a plain assignment none of the ten reads leaves the key where it was,
and the step is served.

**The limit.** Nothing checks the two lists. A recipe one of the ten comes to
call, or an assignment one comes to read, has to be added by hand, as with
`_code-fingerprint rust`'s list, and a trace like the one above is how to find
it.

**The demonstration.** `check-gate-verdicts-selftest`'s block for this step,
on §337.10's fixture, now asserts:

- a second `decisions-toc` after the stub leaves the key where it was and is
  served the step;
- a second `queue-merge-driver` moves the key and runs the step;
- a change to `queue_bundle`, a rename of the variable `queue_page_dir` reads
  with its value the same, and a change to only the second line of a
  `queue_shared_files` written over two, each move the key;
- an unread assignment of a plain string leaves it; one with a backtick, one
  that calls `read()`, an exported one, and a new setting each move it.

Each fingerprint a case takes must be a hash, so one that errors cannot pass a
`!=` as an empty string.

The selftest failed before each part it checks was there. At `2e0e493bd`, with
the first cases and §337.10's key, it failed on "a commit that changed only
decisions-toc, which check-queue-api-selftest never runs, moved its
fingerprint". The key at `7a9e7c8d2` took each definition as a line of the text
dump and an unread assignment only for a backtick or `shell()`; at
`e64e18b12`, with the rest of the cases, it failed on "a commit that changed
only the second line of queue_shared_files, which queue reads, left
check-queue-api-selftest's fingerprint where it was". Five mutations, each
reverted, failed it on the case they break: on `7a9e7c8d2`, dropping
`queue-merge-driver` from the ten, and dropping the eleven definitions; on
`07b195173`, counting only `shell()` among the calls, leaving exported
assignments out, and leaving the settings out.

**The real run.** `just pre-gate gate-lane` on `11a76a317`, the row's commit
of this section, passed. Its branch changed only the justfile and this file
since `219aed7c`, so it was served `check`, `check-ignored`, `test` and
`check-release-build` from `219aed7c`'s gate on §337.11's key, with `test`'s
4772 tests. `check-queue-api-selftest` ran, at 276.1 s, since nothing was yet
recorded under the new key, and its green was recorded as `c264e8773686`. It
was still the longest step of the gate; the next was
`check-gate-verdicts-selftest`, at 66.0 s. On a scratch commit over
`11a76a317` that added one line to `decisions-toc`'s body, the whole-tree key
moved from the gate's `37a08eb0e766` to `a0778dc4dc83`, and this one stayed
`c264e8773686`, so a gate there would be served the step.

**What this leaves.** The comment above `gate_reusable_steps` still says this
step answers from `justfile`, `tooling/`, `.gitattributes` and `mise.toml` and
is keyed on that tree. It is in an item Q3080 held while this row ran, so a row
says to bring it to this section.
