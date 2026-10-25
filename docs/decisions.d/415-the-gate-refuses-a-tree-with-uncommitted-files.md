## 415. The gate refuses a tree with uncommitted or untracked files — 2026-10-08

*Designed with Mattias on 2026-10-08, and built by Q1413. Where the design
left a choice open, the choice and its reason are in §415.2.*

In short: `just gate` used to run every step on a tree with uncommitted or
untracked files, and could pass. That pass was worth less than it looked:
`_code-fingerprint` gives such a tree no name, so nothing was served and
nothing was recorded, and the verifier after the lane ran the whole gate
again. Now `gate` refuses such a tree before any step starts, names the files,
says to commit first, and exits 2. `gate-lane` and `gate-docs` re-enter `gate`,
so they refuse too.

### 415.1 Why

A lane that gates work it has not committed hands its verifier a gate that
cannot be served (§337.3): no sha names what it read. Over the `lanes` runs
since 2026-10-01, re-counted while building this:

```
python3 /tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-main/d82cdb29-95f8-4f99-a902-30ba28d53418/scratchpad/last_gate.py
```

prints `347 lane-verifier pairs in lanes runs since 2026-10-01`, and on its
row `lane gated green, then committed code` a total of 23: 23 lanes committed
code after their last green gate. The same script writes `last_gate.json`
beside itself, and in it 12 of those 23 last gates said `uncommitted or
untracked` on their `· verdicts:` line, so they visibly gated uncommitted work.
17 lanes in all had a last gate that said it. The handoff
`/tmp/rcad-handoff-lanes-js-merge-on-a-red-gate-2026-10-08.md`, under "Other
findings", had 23 of 343 and 12 a few hours earlier. None of them came back red.
What they cost is time. The same print gives a verifier's `verify-lane` a median
of 2.6 min when its gate was served green and 6.7 min when it ran green.

### 415.2 What it does, and the choices the design left open

**The check is `git status --porcelain --untracked-files=normal`**, in `gate`,
after its argument checks and before anything else. Untracked files count,
because cargo compiles an untracked `.rs` under `crates/*/tests/` all the same
(§337.3). The flag is named because `status.showUntrackedFiles=no` in a user's
git config would hide that file from a bare `--porcelain`, and
`_code-fingerprint` now passes the flag too. Ignored files do not count;
§337.3 lists them.

`gate-fresh` refuses only tracked changes, and that stays. It gates a clone,
and a clone cannot see an untracked file, so the gate it runs is of HEAD.

**It is a refusal and not a step.** The steps start together and run to their
end. A step would only say "commit first" once the whole gate had been paid
for.

**It prints `git status --short`, then "commit first — a WIP commit is enough,
and the branch is still on disk", and exits 2**, as the refusal of a misspelt
word does, in `gate-fresh`'s words (§172.4). There is no word that overrides
it. Lanes would be the first to use one.

**It prints no summary block.** The block describes the steps that ran, and none
did. A block would have to say `gate: FAIL`, and the readers that count
`gate: FAIL` would then count a refusal as a red gate: `last_gate.py` above,
and `land.py`'s `gate()`, which stops on that line. Neither reader can meet a
refusal today. `verify_lane.py` refuses a dirty tree itself
(`refuse_unless_a_clean_linked_tree`) and then checks out a commit, so its tree
is clean when it gates. If one ever were refused, it would report `the gate
exited 2 and printed no summary block; its output is in the log above`, with
the refusal in that log, which is true. `land.py` stops on a dirty tree before
it gates (its `tree` stop). If it ever met a refusal, its fallback is the last
line, which says to commit first.

**The refusal comes before `target/last-gate.log` is emptied**, so a refused run
leaves the last real gate's log in place.

`pre-gate` does not refuse. A lane's `just pre-gate gate-lane` on a dirty tree
runs `pre-gate`'s few seconds first, and then `gate-lane` refuses.

### 415.3 Who runs the gate, and whether their tree can be dirty

- **A lane** (`lane.md`) now commits first. Its paragraph on gating says so.
- **A verifier**: `verify_lane.py`, as above. Its tree is clean when it gates.
- **A merger** commits the merge in `merger.md` step 3.7, where `git status
  --short` must print nothing, and gates in step 4.
- **`just land`** stops on a dirty tree before it gates.
- **A top-level session or Mattias at a terminal** now needs a WIP commit to
  gate an edit. That is the cost (§415.5).
- **CI** (`ci.yml`): after the checkout, its steps write `GITHUB_ENV`, run
  `rustup show`, `mise-action` (`mise install`), `rust-cache` (which restores
  `target/`, an ignored path), and the corpus into `RUNNER_TEMP`, which its
  comment keeps outside the checkout so that nothing reads it as untracked.
  Then `check-corpus`, which reads, and the locales, which are system files.
  `ci-docs.yml` runs the checkout, `mise-action` and `gate-docs`.
- **`gate-fresh`** clones HEAD with `--no-local`, so the clone starts clean.
  In the container, before the gate: `apt-get` and `locale-gen` write under
  `/etc` and `/usr`, `git config --global` and the `mise.run` install write
  under `/tmp/home`, and `rustup show` writes in the image's rustup home. No
  cargo command runs in the clone before the gate, so nothing can write
  `Cargo.lock` there. `mise install` is the one command that can write into the
  clone: `mise.toml` sets `lockfile = true`, and `mise.lock` is tracked. On this
  machine, mise 2026.9.13 ran `mise install` in this lane's tree and `git status
  --porcelain` stayed empty. The container installs whatever `mise.run` serves,
  and 2026.10.4 is out. A mise that writes `mise.lock` again would make the gate
  in there refuse and name the file, so the failure would be loud. CI's
  `mise-action` is in the same position. `gate-fresh` was not run for this:
  docker and a cold tree are ten minutes. The recipe now carries this
  paragraph's point beside `mise install`, and a row runs it once.

  **Q1420 ran it, on main at `11b049277`.** The container selected mise
  2026.10.19 (`mise: selected 2026.10.19 (minimum release age: 24h)`), newer
  than the host's 2026.9.13 and newer than the 2026.10.4 this paragraph named
  when it was written. `mise install` finished its tool installs and then
  printed its own lockfile check inline, ending `mise lock ✓ 63 targets
  checked`, with no `Updating` or write line before it — a check against the
  tracked `mise.lock`, not a rewrite of it. `just gate quiet`'s first step,
  `check-plugins`, started on the very next line, which is only reachable past
  this same recipe's own dirty-tree refusal (§415.2): the refusal did not fire,
  so the clone was still clean after `mise install`. The hypothesis this
  paragraph named is answered: **no, the newer mise did not write `mise.lock`,**
  and `gate-fresh` and `ci.yml`'s `mise-action` need no change for it.

  The run's gate still failed, first on `check-status-selftest`, also on
  `check-queue-selftest`, `check-clean-targets-selftest` and
  `check-timing-hooked-calls` — none of them about `mise.lock` or a dirty tree,
  and the run crossed 2026-10-25 into 2026-10-26 local time while `test` alone
  ran 931.7s, so a date-boundary flake is one live hypothesis and an unrelated
  regression is another; neither is checked here. Filed as a follow-up.
- **The gate's own selftests run nested gates.** `check-gate-verdicts-selftest`
  already ran its gates in a scratch repository, with `GIT_DIR` and
  `GIT_WORK_TREE`. `check-gate-summary-selftest` now does the same. Its gates
  asked git about the real tree, and the real tree is dirty whenever someone
  runs the selftest by hand mid-edit, or when a failing proptest writes its
  seed (§415.4) while the real gate runs.
  `check-gate-docs-selftest` runs `gate-docs` and `gate-lane` in the real tree
  as well, and it was outside Q1413's write set. On this lane's tree, with
  `justfile` and `lane.md` edited and not yet committed, `just
  check-gate-docs-selftest` failed on the refusal, while `just
  check-gate-summary-selftest` and `just check-gate-verdicts-selftest` passed.
  A row moves it into a scratch repository too. Until then, a red gate whose
  failing proptest wrote its seed before `check-gate-docs-selftest` ran shows
  that step on its `also failed:` line as well. That needs a seed written
  while the selftest runs. It is the thirteenth step that builds nothing, so
  on a machine with fourteen cores or more it starts at once; in this lane's
  gate it took 4.5 s, while `test` waited behind five cargo steps before it
  started.
- **No other recipe runs `gate`, `gate-lane` or `gate-docs`** through `$j` or
  `just_executable()`. A grep of the justfile found only the three selftests
  above, and `gate-lane` and `gate-docs` themselves. `check-land-selftest` and
  `check-verify-lane-selftest` put a stub in place of `just`, through
  `LAND_JUST` and `VERIFY_LANE_JUST`.

### 415.4 What a gate writes into the tree

**A passing gate writes no tracked or unignored file**, as far as this lane
could see. A script polled `git --no-optional-locks status --porcelain
--untracked-files=all` about every 0.2 s while `just pre-gate gate-lane` and
then `just test-slow` ran on `a0b0a3a1`, with every step passing but
`check-decisions`, which failed only because this file was not yet written. It
polled 1201 times and every answer was empty. The records agree: a gate
records a verdict only when `_code-fingerprint` finds the tree clean at its
end, and 236 verifier gates in §415.1's window were served from one.

**A failing proptest does write one.** Ten test files set
`FileFailurePersistence::WithSource`, among them
`crates/rcad-topo/tests/turning_a_fuse.rs` and `fuses_that_touch.rs`, and the
seed goes into the `.proptest-regressions` file beside the test, which is
tracked. The next gate then refuses until the seed is committed or reverted.
That is right: proptest replays the file's seeds first, so the file is part of
what the gate reads.

### 415.5 The sentences for a dirty tree that stay

- `_code-fingerprint`'s "the tree has uncommitted or untracked files". `gate`
  calls it at its start, where the refusal has just found the tree clean, and
  at the end of a pass, where the answer decides whether the pass is recorded.
  The second call still meets the sentence when the tree changes while the gate
  runs. `check-gate-verdicts-selftest` now drives that: it writes a file while a
  two-second step runs and checks that the pass says `not recorded, because the
  tree changed while the gate ran` and leaves no record. Kept.
- `_record-green`'s check that the tree is clean (§173.9), which looks at
  tracked files only. `gate` reaches it
  when a passing run's tree changed, and `gate-fresh` reaches it when the host
  tree changed while the container ran. `check-fuzz-watermark-selftest` pins it
  in its case seven. Kept.
- `gate`'s own line `· verdicts: …, so nothing is served or recorded` at its
  start. It also prints "there is no commit here" and a fingerprint call that
  printed nothing. Kept.

### 415.6 What it costs, and what it does not catch

**An edit, gate, look loop at a terminal now needs a WIP commit** before each
gate. **A seed that a failing gate writes blocks the next gate** until it is
committed or reverted. Both are the rule working.

**It does not catch a lane that gates a clean tree and then commits more
code.** The other 11 of the 23 lanes above show no sign of a dirty tree at
their last gate, and a refusal at gate time cannot see a later commit. The verifier's own gate is what catches it, and that
is why the verifier keeps its gate.

### 415.7 What this leaves

- `check-gate-docs-selftest` runs its nested `gate-docs` and `gate-lane` in a
  scratch repository, as the two selftests beside it do (§415.3). Filed.
- One run of `just gate-fresh` after this lands, to see whether the container's
  newer mise writes `mise.lock` (§415.3). Done, by Q1420: it does not.
- Whether `check-status-selftest`, `check-queue-selftest`,
  `check-clean-targets-selftest` and `check-timing-hooked-calls` failing
  together in Q1420's `gate-fresh` run is a date-boundary flake (the run
  crossed local midnight) or a real regression on `11b049277` (§415.3). Filed.
