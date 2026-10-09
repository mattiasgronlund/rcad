## 420. A lock does not keep what is in `target/` — 2026-10-09

*Mattias asked on 2026-10-09 what to do about
`dispatch-next-worktrees/wf_39298cae-c5a-1`, which `just lanes` showed as
`HELD`. Claimed 2026-10-09T06:28Z. Docs only, on `main`.*

### 420.1 Two trees locked to keep files that were already gone

Two lane trees stayed locked after their rows merged, so that later rows could
read what the lanes had left in `target/`:

| Tree | Branch | Lock reason | Rows it was kept for |
|---|---|---|---|
| `dispatch-next-worktrees/wf_39298cae-c5a-1` | `lane-q236-timing-compiled-out-2`, `ee8d4b08` | keeps Q236's fuzz builds and logs in target/q236/ for Q277-Q279; not a lane, nothing runs here | Q277, Q278, Q279, all `todo` |
| `dispatch-next-worktrees/wf_aa4db869-30f-1` | `lane-q163-bytes`, `1cd12e7e` | keeps §273's scripts and q163 binaries in target/ for Q321 and Q322 (and timing-computed-breakdown.py, §273.9's split); not a lane, nothing runs here | Q321, Q322, both `dropped` |

On 2026-10-09 neither tree had a `target/` or a `fuzz/target/`. Both trees'
top directories were last changed in the same minute, 2026-09-30 06:26Z
(`find … -newermt '2026-09-30 08:26' ! -newermt '2026-09-30 08:27'`, local
time, matched exactly these two). That is when the two `target/` directories
went.

Both were locked at the time. The first tree's lock file was last written
2026-09-27 19:40:45Z, and locking again would have rewritten it. The hourly
clean (§371) lists both trees as `kept (locked, an agent is using it)` at
06:00:34Z and again at 07:00:34Z that day.

### 420.2 Who deleted them

Mattias did, by hand in a terminal, while clearing disk space on the morning
of 2026-09-30. His shell history (`~/.bash_history`, lines 1901 and 1907) has:

    cd ~
    find . -name target -print0 | xargs -0 rm -rf

Run from the home folder, that removes every directory named `target` under
it, locked tree or not. Bash wrote no times, so the time comes from what the
command left behind. Directories in six other repos under `~/git` (probe-rs,
inari, three garagerobotics checkouts and one Tietoevry checkout) were last
changed between 06:26:38Z and 06:26:47Z, along with the two rcad trees at
06:26:41Z and 06:26:43Z. It also took a tracked source directory that happens
to be called `target`, `probe-rs/src/gdb_server/target/`, 13 files. The same
history removes rscene's `target/` folders twenty-six lines earlier (line
1881), and rscene's worktree directories were last changed at 06:16:02Z. The
order matches.

What did not do it:

- **Not `rcad-clean-targets`.** Its runs on either side of 06:26Z kept both
  trees, and each run finished in the same second it started (`journalctl
  --user -u rcad-clean-targets --since '2026-09-30 07:59' --until '2026-09-30
  10:05'`, local time). The recipe has skipped locked trees since before the
  timer existed.
- **No Claude session ran or suggested it.** No transcript holds a record
  between 2026-09-29T23:01Z and 2026-09-30T06:37Z. No assistant message or
  tool call before the deletion holds a `find -name target` that removes
  anything, except `clean-targets`'s own code, which skips locked trees. The
  commands sessions did hand Mattias around then also keep locked trees:
  `just clean-targets` on 2026-09-29 at 20:07Z, and on 2026-09-28
  `remove-unused-trees.sh`, which checks each tree's lock again before it
  removes it.

**The disk was not measured to be full that morning.** It was full the
evening before: "No space left on device" in five runs from 20:05Z to 20:24Z,
until `just clean-targets` brought `/` to 77%, 203 GB free. Nothing measured
it again until 06:38Z. That was after the delete, and after the kache store
had gone from 151 GB to 554 MB the same morning. At that point `/` was at
32%, 596 GB free.

### 420.3 What it cost

- Q277's lane, launched 2026-10-09 06:15Z, is rewriting `build.sh` and
  `pads.sh` from §305.2's and §305.4's prose and rebuilding the binaries. Q278
  and Q279 will have to do the same.
- The logs §305 quotes cannot be read again. Its figures now rest on the
  section and on what its verifier read on 2026-09-27.
- For nine days `just lanes` showed two `HELD` trees whose locks protected
  nothing and whose reasons named files that did not exist.

### 420.4 The rule

**Lock a tree to keep people out of it, never to keep files in it.** §92.8
said where a fuzz input's bytes belong: in two places, neither of them a lock.
The same holds for a measuring lane's scripts and logs. A lock only stops
`git worktree remove` and the cleaners that choose to honour it. A `find` from
the home folder that removes every `target` ignores it, and 420.2 is that
happening.

- **A script a section cites by path is committed.** It goes into the section
  as a code block, or into a tracked file the section names, in the same
  commit. §305 named five scripts (`build.sh`, `run.sh`, `pads.sh`,
  `replay2.sh`, `fnstat.sh`) and printed only the build line, which is why
  Q277's lane is writing them again.
- **A later row is briefed to rebuild, not to find.** Binaries come back from
  the build line the section prints. Logs come back by running again. The
  lines that matter are quoted in the section, as §305 did.

Nothing is built for this. A check in `just lanes` that flags a held tree
whose reason names a missing `target/` path would have caught these two, but
it has no consumer once no lock reason promises files.

### 420.5 The two trees, removed

Removed on 2026-10-09 at about 06:27Z, after these checks in each tree:

- `git status --porcelain --ignored` printed nothing, and `git ls-files
  --others` counted 0.
- `fuzz/artifacts/` held no file. This is §92.3's check: that directory is
  gitignored and can be the only copy of a crash.
- `git rev-list --count main..HEAD` was 0, and each branch tip is an ancestor
  of `origin/main` (`git merge-base --is-ancestor`).
- No running `claude` process had either tree as its working directory.

Then `git worktree unlock`, a plain `git worktree remove` without force, and
`git branch -d lane-q236-timing-compiled-out-2 lane-q163-bytes`.
