## 271. Selftests that nest an unpinned `just` call, and whether any reads the live tree

*Queue row Q152. §263.8's `check-queue-api-selftest` pins every nested `just`
call with `--justfile` and `--working-directory` so the scratch hub it builds,
not this repo, answers. This section sweeps the rest of the justfile for the
same pattern — a selftest that builds a scratch directory under `mktemp` or
`target/` and then calls `just` or `{{ just_executable() }}` inside it — and
checks each one against the four files a leak could hit: this repo's
justfile, `docs/queue.md`, `docs/decisions.d/`, `open-questions.md`, and the
queue claims file (`rcad-queue-claims`, in the git common directory, read
through `QUEUE_FILE`/`QUEUE_CLAIMS`).*

### 271.1 The two ways a nested call avoids the live tree

Neither is "pin every flag". A recipe under test either:

- **takes the scratch path as an explicit argument** and does everything
  through it (`git -C "$dir"`, or a bare `{{file}}`/`{{dir}}` parameter), so
  the nested `just` can run completely unpinned — it reads the real
  justfile, but that justfile's recipe body never touches a path the fixture
  did not hand it; or
- **defaults to a relative path** (`docs/queue.md`, `docs/decisions.d`,
  `open-questions.md`, the claims file's `git rev-parse --git-common-dir`),
  and then the nested call needs either `--working-directory` into the
  scratch tree (so those defaults resolve there), a `cd` into the scratch
  tree before the bare call, or a `--set` that overrides the variable
  directly (`decisions_dir`, `queue_file`, `questions_root`) with an absolute
  scratch path.

A selftest that mixes the second shape with neither fix is the bug this
section is looking for. None turned up.

### 271.2 The table

Every selftest below nests a `just`/`{{ just_executable() }}` call inside a
`mktemp -d` or `target/` scratch directory.

| selftest | pinned (`--justfile`+`--working-directory`) | if not, what makes it safe |
|---|---|---|
| `check-queue-api-selftest` (§263.8) | yes, on every call (`q()`/`check()` helpers) | — |
| `check-claim-section-selftest` | yes, on every call (`claim()` helper) | — |
| `check-src-selftest` | yes | — |
| `check-decisions-selftest` | no | `--set decisions_dir "$dir/…"`; `check-decisions` derives its pointer path as `${decisions_dir%.d}.md`, never a separate cwd-relative variable, so the one override redirects every file it reads (traced in `check-decisions`'s body) |
| `check-recipe-selftest` | `--justfile` only (deliberately the real one) | `--set recipes_file "$dir/fixture"`; `recipe` reads only that variable, and the real justfile is what its second half (the "sweep") is deliberately comparing itself against |
| `check-plugins`'s embedded `question` fixture (not a separate `-selftest` recipe, ~line 3139) | no | `--set questions_root "$dir"`; `question` reads `open-questions.md` only under that variable |
| `check-queue-selftest` | no (`j='{{ just_executable() }}'`, bare) | `--set queue_file "$dir/$1.md" --set queue_running 1`; the `queue_running` override skips the only code path (`queue.py running --count` / `elsewhere`) that reads the claims file, and this selftest drives only `next`/`ready`, never `take`/`add`/`done`/`drop`/`park`/`after`/`pick`/`show`/`running` — the verbs that reach `queue.py`'s `Claims` and would need `--working-directory` or `QUEUE_CLAIMS` to stay off the real one |
| `check-lanes-selftest` | no (bare `just lanes "$dir/main"`) | `lanes dir="."` does everything through `git -C "$repo"`/`git -C "$tree"`; traced its body, no relative path outside `{{dir}}` |
| `check-release-lanes-selftest` | no | `release-lanes pattern dir="."`, same shape, traced |
| `check-ff-main-selftest` | no | `ff-main dir="."`, same shape, traced |
| `check-hub-selftest` | no | `check-hub dir="."`, same shape, traced |
| `check-status-selftest` | no (bare `just _status-branches …`, `_status-green …`, `_where-the-work-is-check …`) | all three take the path/file as an argument and never read a second, implicit one; traced all three bodies |
| `check-clean-targets-selftest` | no | `clean-targets dir="."`, same shape, traced |
| `check-prune-worktrees-selftest` | no | `prune-worktrees dir="."`, same shape, traced |
| `check-fuzz-overnight-selftest` | mixed: most calls unpinned with no `cd` either | `fuzz-overnight` never reads any of the four files (grepped its whole body); isolation there comes from `RCAD_FUZZ_LOGS`/`RCAD_FUZZ_CORPUS`/a stubbed `cargo` on `PATH`, not from `--working-directory`. One case (the "hub is refused" fixture, ~line 4569) does `(cd "$hub" && … {{ just_executable() }} fuzz-overnight 4)` — the `cd` is the pin, into a hub carrying its own copied `justfile` |

### 271.3 What was checked by grep, not by full trace, and came up empty

`check-fuzz-watermark-selftest`, `check-gate-fresh-selftest`,
`check-gate-fresh-cleanup-selftest`, `check-fuzz-triage-selftest`,
`check-fuzz-unit-selftest`, `check-fuzz-yield-selftest`,
`check-coverage-selftest`, `check-t-selftest`, `check-corpus-selftest` and
`check-hub-align-selftest` each nest at least one unpinned `just`/
`{{ just_executable() }}` call inside a scratch directory. None of them
appears anywhere near `decisions_dir`, `queue_file`, `questions_root`,
`open-questions.md`, `QUEUE_CLAIMS`, or a queue verb — checked with one `grep`
of the whole justfile for each of those strings, not narrowed to the
recipe's own lines, so a dynamically-built path (e.g. one assembled from
`git rev-parse --show-toplevel` at runtime) would not show up this way. That
gap is real and is `remaining` below; everything in §271.2 above it was read
line by line.

`check-gate-summary-selftest` and `check-gate-docs-selftest` nest unpinned
calls too, but not against a scratch **repo**: they drive the real `gate`/
`gate-docs` recipes with `--set gate_steps`/`--set gate_log` against a
`mktemp` log directory, which is testing this repo's own gate machinery on
purpose, not a case of an intended scratch tree leaking into the live one.

### 271.4 The finding

**No selftest in this justfile appends to the live `docs/queue.md`, the live
`docs/decisions.d/`, the live `open-questions.md`, or the shared
`rcad-queue-claims` file.** §263.8's fix pinned the one case that used to.
Every other nested call either takes the scratch path as an explicit
argument and stays inside it, or overrides the one variable the recipe under
test actually reads.
