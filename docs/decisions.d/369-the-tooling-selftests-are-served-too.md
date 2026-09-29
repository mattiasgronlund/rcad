## 369. The tooling selftests are served too — 2026-09-29

*Q687, done by session rcad-cf in its own tree, not as a lane. Every figure
below was run in this session. The traces are the ones §367.7 describes, run
again at `a5b666b9` because the first set was deleted when the disk filled.*

In short: 33 more steps go into `gate_reusable_steps`, 42 in all. They are the
30 that §367.7 found reading the tree and nothing else, plus three more:
`check-prune-worktrees-selftest`, and the two whose fixtures leaned on the real
`docs/`, which now do not. A served gate still runs 14 steps. Eight check the
machine and are Q693's. Six read `docs/`, and those have to run.

### 369.1 What the traces could not see

§337.2's three checks were an strace, a grep of the recipe, and a grep of the
Rust. §367.7 ran the strace. Three more things can change a step's answer
without showing up there:

- **Files inside the tree that git ignores.** These are in no fingerprint.
  Every path inside the tree that each of the 35 steps touched, outside
  `target/`, went through `git check-ignore`. The only hits are `__pycache__/`
  under `tooling/queue/` and `tooling/session-spend/`, from three steps. Python
  checks a `.pyc` against its source's modification time and size before it
  uses it, so what the step runs is still the tracked source.
- **The HEAD sha**, which a docs-only commit moves. Three steps read `git
  rev-parse HEAD`. `check-gate-summary-selftest` and `check-gate-docs-selftest`
  run a nested gate whose steps are `list` and `no-such-step`. Neither of those
  is reusable, and the nested gate uses HEAD only to serve, record or audit,
  which it does only when a reusable step is in its list.
  `check-fuzz-triage-selftest` runs `fuzz-triage` against `HEAD`, and
  `fuzz-triage` takes that to mean the tree it stands in: it borrows a worktree
  only when `against` names another commit. It prints the short sha, and the
  selftest never checks what it printed.
- **Environment variables.** The recipe text of each step, and of every recipe
  it calls, was searched for `$NAME` and `env_var(...)`. `RCAD_STEP_CORPUS` and
  `RCAD_NIGHTLY_FUZZ_WORKTREE` start with `RCAD_`, and the fingerprint already
  covers those (§337.3). `GH_GREEN`, `DOCKER_ARGV`, `DOCKER_STATUS`,
  `DOCKER_CLEANUP_STATUS` and `REAL_RM` are set by the selftests themselves.
  `check-kache-off-on-ci` reads `RUSTC_WRAPPER` through `mise exec`. The tree's
  `mise.toml` sets that key in every case, and mise lets the nearest config win,
  so a config further out cannot change it. Neither `~/mise.toml` nor
  `~/.config/mise/config.toml` names it.

### 369.2 The three added to the 30

- **`check-prune-worktrees-selftest`** lists `/proc`, and §367.7 had not
  checked what it does with the listing. `prune-worktrees` keeps a tree when
  some process's working directory is that tree or inside it. Every tree the
  selftest builds is under a directory `mktemp -d` has just made, so the only
  process that can match is the selftest's own `sleep`. Whether `/proc` exists
  depends on the machine, and a verdict is served only on the machine that
  recorded it.
- **`check-skills-selftest`**'s fixture for a path that does not exist was
  `docs/no-such-file.md`. It passed only while no such file was there. Now it is
  `crates/no-such-file.rs`, and adding that file would move the fingerprint.
  Shown: with the old path present beside the justfile it runs from, `HEAD`'s
  version failed and the new one passed.
- **`check-queue-selftest`**'s shape half ran `_queue-check` in the real tree,
  where it asks `git ls-files` which paths are tracked. It also ran
  `check-queue` once, against the real sections. Now all of it runs with
  `--working-directory` in a scratch repository. That repository has four files
  staged and a `docs/decisions.d/` holding the five sections the fixtures cite.
  `decisions_dir` is the relative path `docs/decisions.d`, so `check-queue`
  still goes through `decisions-all` and reads the scratch copy. Shown: with
  `docs/decisions.d/013-*.md` moved out of the tree, `HEAD`'s version failed and
  the new one passed. With `nowhere/new.md` added to the index, the same. The
  fill half already set `queue_running`, so it never counted the machine.

### 369.3 What still runs

`check-plugins`, `check-hooks`, `check-hub`, `check-corpus`,
`check-fuzz-unit`, `deny`, `check-questions-selftest` and
`check-queue-merge-driver` check the machine. They are Q693's.
`check-decisions`, `check-plan`, `check-queue`, `check-where-the-work-is`,
`check-skills` and `check-lanes-workflow` read `docs/`.

Those 14 are exactly the steps `check-gate-verdicts-selftest` refuses to serve.
It gained `check-queue-merge-driver`, which reads git's config.

Why `check-queue-selftest` took 13.0 s in one timing and 5.5 s in the next
(§367.7) was still not looked into.
