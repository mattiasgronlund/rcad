## 277. The ten selftests §271.3 only grepped, read line by line

*Queue row Q176. §271.3 named ten selftests it checked by one whole-justfile
`grep` for `decisions_dir`, `queue_file`, `questions_root`,
`open-questions.md`, `QUEUE_CLAIMS` and a queue verb, and said that grep would
miss a path built at run time. This reads all ten line by line, follows every
script or recipe each one calls, and for six of them runs the selftest under
`strace -f -e trace=openat,rename,unlink` to confirm no open, rename or
unlink ever names `docs/queue.md`, `docs/decisions.d`, `open-questions.md` or
`rcad-queue-claims`.*

### 277.1 Two of §271.3's own premises do not hold

Before the table: two of the ten are not what §271.3 said they were.

**`check-hub-align-selftest` nests no `just`/`{{ just_executable() }}` call at
all.** It builds a scratch git repository under `mktemp -d` and calls
`bash "$align" "$dir/main"` directly, where `$align` is
`tooling/hub-align/align.sh`. That script never shells out to `just`; it does
everything through `git -C "$hub"` on the path it is handed. §271.3 grouped
it with the nine that do nest a `just` call — this section keeps it in the
table for completeness, but the finding for it is "there is no nested `just`
call to pin or leak from," not "it is pinned" or "it is safe by some other
route."

**`check-fuzz-watermark-selftest` is fully pinned, not unpinned.** Its two
nested calls, `picked()` and `record()`, both read:

```
{{just_executable()}} --justfile "{{justfile()}}" \
    --working-directory "$repo" fuzz-watermark
```

and the same for `_record-green`. `$repo` is the scratch fixture repository
built earlier in the same recipe. This is exactly §271.1's first shape
(`--justfile` + `--working-directory`), the shape §271.2's table already
credits fourteen other selftests with. Why the earlier grep called it
"unpinned": the grep in §271.3 was for `decisions_dir`, `queue_file`,
`questions_root`, `open-questions.md`, `QUEUE_CLAIMS` and a queue verb — none
of which say anything about whether a nested call is pinned — so it never
checked pinning for these ten at all; it only checked whether the four
strings appeared anywhere in the justfile near them, which is a different
question answered by coincidence, not by design.

`check-gate-fresh-selftest` and `check-gate-fresh-cleanup-selftest` are the
same shape: both nested calls carry `--justfile "{{ justfile() }}"
--working-directory "$repo"` against a scratch repo built in the same recipe.
All three are §271.1's first shape and none of the three touches a `docs/`
path or `RCAD_STEP_CORPUS`-style variable at all — `fuzz-watermark`,
`_record-green` and `gate-fresh` deal in commits and a `.git/rcad-greens`
file, not decisions, queue rows or questions.

### 277.2 The table

| selftest | nested call(s) | directory it runs in | paths built at run time | reaches the live tree? |
|---|---|---|---|---|
| `check-fuzz-watermark-selftest` | `fuzz-watermark`, `_record-green` — both `--justfile "{{justfile()}}" --working-directory "$repo"` | scratch git repo `$repo` under `mktemp -d` | none outside `$repo`; both recipes only touch commits and `$repo/.git/rcad-greens` | no — pinned (§277.1), and neither recipe reads or writes any of the four files |
| `check-gate-fresh-selftest` | `gate-fresh` — `--justfile "{{ justfile() }}" --working-directory "$repo"` | scratch git repo `$repo` under `mktemp -d`, plus a stubbed `docker` on `PATH` | `$repo/.git/rcad-greens`, a stub-recorded clone path under a second `mktemp -d` | no — pinned; `gate-fresh` touches only the greens file and the docker mount, never the four |
| `check-gate-fresh-cleanup-selftest` | `gate-fresh` — same pin as above | same shape, plus a fake `rm`/`docker` on `PATH` | same as above, plus a `.root-owned` marker file under the cloned scratch tree | no — pinned; same recipe, same reach |
| `check-fuzz-triage-selftest` | `fuzz-triage "$1" HEAD`, `fuzz-triage-verdict main "$1" "$dir/said"` — bare, no `--justfile`/`--working-directory` | wherever the selftest itself runs (this repo's root; no `cd`) | `root='{{ fuzz_log_root }}'` resolves through `RCAD_FUZZ_LOGS="$logs"` (scratch); `against="HEAD"` resolves via `git rev-parse HEAD` in the real repo, only to pick a commit to replay — when it equals the real `HEAD` no worktree is borrowed and `work="."`; verdicts are written to `$night/fuzz-triage-verdicts.tsv` under the scratch `$root` | no — read the whole body of both recipes; neither references `docs/queue.md`, `docs/decisions.d`, `open-questions.md`, `QUEUE_CLAIMS` or any queue verb. `fuzz-promote`, called later in the same selftest, is separately pinned with `--justfile`/`--working-directory` |
| `check-fuzz-unit-selftest` | `check-fuzz-unit "$tracked"` — bare | wherever the selftest runs | `tracked` is the explicit scratch argument; `{{systemd_user_dir}}` resolves through `RCAD_SYSTEMD_USER="$installed"` (scratch) | no (confirmed by strace, §277.3) — `check-fuzz-unit` reads only `{{tracked}}/$unit` and `{{systemd_user_dir}}/$unit` |
| `check-fuzz-yield-selftest` | `fuzz-yield` — bare, `RCAD_FUZZ_LOGS="$logs"` | wherever the selftest runs | `root='{{ fuzz_log_root }}'` via the env var (scratch); `stats='{{ justfile_directory() }}/tooling/fuzz-yield/log_stats.py'` is a real, read-only tracked script, not one of the four | no (confirmed by strace, §277.3) |
| `check-coverage-selftest` | `coverage-report "$work/$name.json"` — bare | wherever the selftest runs | `{{report}}` is the explicit scratch JSON path; the Python recipe reads only that file and the source paths named inside it, all under `$work` | no (confirmed by strace, §277.3) |
| `check-t-selftest` | `_test-digest "$dir/pass.log" 0` (and two more fixture logs) — bare; also, for real, `t -E 'test(/a_cube_comes_back_sound_and_measuring_the_same/)'` | wherever the selftest runs | `_test-digest` reads only the given `$log` path; `t` writes `target/last-test.log` (gitignored, real repo, not one of the four) and runs the real suite on purpose | no — read both recipe bodies; neither one has ever heard of `docs/queue.md`, `docs/decisions.d`, `open-questions.md` or the claims file |
| `check-corpus-selftest` | `check-corpus "$manifest"` — bare, `RCAD_STEP_CORPUS="$root"` | wherever the selftest runs | `root="$RCAD_STEP_CORPUS"` (scratch); `manifest` is the explicit scratch path | no (confirmed by strace, §277.3) |
| `check-hub-align-selftest` | none — nests `bash "$align" "$dir/main"`, not `just` (§277.1) | scratch git repo `$dir/main` under `mktemp -d` | `align.sh` does everything through `git -C "$hub"`, where `$hub` is the argument it was handed; it never reads a relative path | no (confirmed by strace, §277.3) — and there is no nested `just` call for a leak to hide in |

### 277.3 What was run, and what was not

Six of the ten cost nothing to run for real — no `cargo` build, plain bash or
one small Python script — so each ran once under
`strace -f -e trace=openat,rename,unlink`, with the trace grepped for
`docs/queue\.md`, `decisions\.d`, `open-questions\.md` and `rcad-queue-claims`:
`check-fuzz-watermark-selftest`, `check-gate-fresh-selftest`,
`check-gate-fresh-cleanup-selftest`, `check-fuzz-unit-selftest`,
`check-fuzz-yield-selftest`, `check-coverage-selftest`,
`check-corpus-selftest` and `check-hub-align-selftest` — eight, not six; all
eight passed and every trace came back with no match. Command, run once per
recipe from this repository's root:

```
strace -f -e trace=openat,rename,unlink -o target/strace-out/<name>.trace \
    mise exec -- just <name>
```

`check-fuzz-triage-selftest` and `check-t-selftest` were not run: this
worktree's `target/` was cold (3.8 MB, no build artifacts), and the first
`cargo run`/`cargo nextest run` either one triggers is a from-scratch compile
— the "cold tree" case `docs/decisions.d`'s `just gate` notes can run to the
full nine minutes, which does not fit this lane's budget against two
selftests whose bodies were already read in full with no reference anywhere
to the four paths, the three redirect variables, or a queue verb. Reading
settled these two; running them would only have re-confirmed a body with no
dynamic path construction left unaccounted for.

### 277.4 The finding

**None of the ten reaches the live `docs/queue.md`, `docs/decisions.d/`,
`open-questions.md` or `rcad-queue-claims`.** Three of the ten
(`check-fuzz-watermark-selftest`, `check-gate-fresh-selftest`,
`check-gate-fresh-cleanup-selftest`) are pinned with `--justfile` and
`--working-directory` exactly like §271.2's fourteen, and were miscounted as
unpinned only because §271.3's grep tested for the wrong strings, not for
pinning. One (`check-hub-align-selftest`) nests no `just` call at all. The
remaining six nest a genuinely bare `just` call but redirect the one variable
the recipe under test reads through an explicit scratch argument or an
`RCAD_*` environment variable, the same shape §271.2 already credited to
`check-lanes-selftest` and its siblings — and for eight of those nine
(excluding the two left unrun, §277.3) that redirection was confirmed by
running the selftest under `strace` rather than by reading alone.

§271's finding now covers the whole justfile: no selftest of either the
fourteen in §271.2 or the ten here writes into, or reads a decision from, the
live queue, decisions, questions or claims files that another session is
using at the same time.
