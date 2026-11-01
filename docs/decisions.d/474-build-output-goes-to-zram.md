## 474. Build output goes to zram; the worktrees and `target/` stay on disk — 2026-11-01

*Mattias, 2026-11-01: "I really would like all worktrees to be stored in
volatile memory (i.e. only in RAM). But I would also like the work here to
survive a reboot. Currently I try to ask for all relevant information to be
stashed away, but sometime the host will reboot without my knowledge, so there
should no be any information that needs to be stashed away during the reboot."
Then, after 474.1: "So you are right it would be enough to move /target to the
ZRAM backed disk. And using xfs would help kcache more build output." And: "And
in that case we could also clear /target for worktrees which are waiting for
other rows to be completed before any work can continue on them to same ZRAM
space."*

So: cargo's build output goes to a zram disk, and nothing else moves. A reboot
empties that disk and loses only what kache can rebuild. The worktrees, their
`target/` directories and everything the tooling writes there stay on disk.
This amends §371.5 and §373.4. The rows are at 474.7.

### 474.1 Why not whole worktrees in RAM

It can be done, but it would cost far more than it buys. A reboot that deletes a
worktree's directory takes four things a branch does not hold:

- **Commits only a detached `HEAD` reaches.** `prune-worktrees` runs `git
  worktree prune` first, and that deletes git's record of every unlocked tree
  whose directory is gone, `HEAD` included. In a throwaway repo, an unlocked
  detached tree's commit was gone after the tree was deleted and the repo
  pruned and `gc --prune=now`. A locked tree's commit stayed, because prune
  keeps a locked tree's record. 24 of the 60 trees were detached that day.
- **Uncommitted edits and untracked files.**
- **What the tooling writes to `target/`:** scripts, measurement CSVs, gate
  logs, `target/land/`. That is §371.2's list.
- **Crash inputs in `fuzz/artifacts/`** (§92.3).

All four could be kept with a snapshot per tree: a commit whose parent is the
tree's `HEAD` and whose content is its working files, written through a second
index file under a ref of its own. The same throwaway repo showed it working:
the tree's own index and `HEAD` untouched, and `git restore --source=<ref>
--worktree -- .` bringing the edits and untracked files back. Taking one of
this tree, 5899 files and nothing changed, took 16 to 20 ms.

**It buys nothing, because the bytes are not in the source.** This tree without
`target/` or `fuzz/target/` was 55 MB (`du -sh --exclude=target
--exclude=fuzz/target`). All of `rcad-worktrees/` was 80 GB. So a tree's
source is under 1% of its size, and the SSD wear and RAM argued in §373 are
the build output's. Moving the source would add the snapshots and a restore at
boot, and gain only that dead trees vanish at a reboot. `prune-worktrees`
already does that.

### 474.2 Cargo puts the build where `build-dir` says, and `target/` keeps the rest

Cargo 1.96, the version `rust-toolchain.toml` pins, has `build.build-dir`.
§373.4 found that it moves every rlib and all incremental state as well as the
test binaries. Here that is exactly what is wanted. `{workspace-path-hash}` in
the value gives each tree a directory of its own, with nothing to set per tree.

**A scratch crate**, built with
`CARGO_BUILD_BUILD_DIR='/dev/shm/build-dir-probe/{workspace-path-hash}'`:

- cargo reported the test binary at
  `/dev/shm/build-dir-probe/9a/c5614f85196351/debug/deps/t-d6568c40f1585dd7`;
- the crate's own `target/` held only `CACHEDIR.TAG`, `debug/.cargo-lock`,
  `debug/.cargo-artifact-lock` and an empty `debug/examples`.

**This repo:** `CARGO_BUILD_BUILD_DIR='/dev/shm/rcad-build-probe/{workspace-path-hash}'
mise exec -- just t -p rcad-geom` ran 1302 tests, all passed, in 21 s.

- The build wrote 450 MB to the RAM directory.
- The only file it added under the tree's `target/` was `last-test.log`.
- It went through kache without an error. `kache report --since 10m` counted
  133 local hits, 2 misses and 8 passthroughs. The daemon is shared, so some of
  those may be another session's.

So everything the tooling writes into `target/` stays on disk, and nothing on
the zram disk is ever the only copy of anything. That is how Mattias's first
condition, nothing to save before a reboot, holds without a snapshot.

The scratch crate had no `[[bin]]` target. How many bytes cargo copies back
into `target/` for a binary target is unmeasured.

### 474.3 kache's store stays on disk, so XFS buys kache nothing here

A reflink, like a hardlink, cannot cross filesystems. XFS helps kache only if
the store sits on the same filesystem as the build directories, which here would
put it on the zram disk too. That does not fit, and it would not last:

- **It does not fit.** `du -sh ~/.cache/kache` said 205 GB, against
  `local_max_size = "150GiB"` in `~/.config/kache/config.toml`. The comment there
  says the cap was raised from 50 GiB because the hit rate had dropped to 57%.
- **It barely compresses.** 400 store files picked at random, 1592 MiB,
  came to 939 MiB at `zstd -3`: 1.69 to one. At the cap that is about 90 GB of
  RAM out of 125.
- **A reboot would empty it,** and then every tree starts cold.

So the store stays on disk, and every build copies out of it, as test binaries
already always do (§319). Inside the zram disk, ext4 and XFS make no difference
to kache, and ext4 is the one with nothing to install (`xfsprogs` is not
installed). The question of moving `/home` to XFS so reflinks work on disk is
separate, and this settles nothing about it.

After this, the SSD takes the store's writes for new compiles and the small
files in `target/`. What that comes to a day is for 474.6.

### 474.4 How big

A lane's `target/` (`status-page-queue-tables`, 6575 MiB) came to 4199 MiB at
`zstd -3`: 1.56 to one. So zram holds about half as much again as a plain RAM
disk of the same memory, not several times as much.

What has to fit is the trees that are building at that moment, since 474.5
clears the rest. That is three slots (§447.7), each a lane, then its verifier,
then its lander, plus Mattias's own sessions: roughly six to ten builds of
about 6.6 GB, which is 25 to 42 GB of RAM at 1.56 to one. **That is an
estimate, not a measurement.**

The disk starts at 80 GB of build output. That is about twelve builds of 6.6
GB, and at most about 51 GB of RAM at 1.56 to one. 474.6 says what to read to
resize it.

When the disk is full, a build fails as it did with a full `/` (§315). So does a
build when the disk is not mounted: `/mnt/rcad-build` is root's when nothing is
mounted on it, so cargo cannot create a directory there and stops. That is the
failure wanted. A build that quietly falls back to writing on the SSD would undo
this section without anyone noticing.

**Setting it up, as root, once per boot until Q3037's unit does it:**

```
sudo modprobe zram
sudo zramctl --find --size 80G --algorithm zstd
sudo mkfs.ext4 -q -m 0 /dev/zram0
sudo mkdir -p /mnt/rcad-build
sudo mount -o discard,noatime /dev/zram0 /mnt/rcad-build
sudo chown mattias:mattias /mnt/rcad-build
```

`zramctl --find` prints the device it took. Use that device in place of
`/dev/zram0` if it names another. **`discard` is what gives memory back.**
According to the kernel's zram documentation, a zram device frees the memory
behind a block when the filesystem discards it. Without that option, a deleted
file keeps its RAM. Not yet tried here.

### 474.5 A tree's build output is cleared once it is idle, whatever its row is doing

Mattias asked for the build output of trees waiting on other rows to be cleared.
Once build output and `target/` are apart, the rule can be wider than that, and
simpler.

§371.5 waits three hours before sweeping a `target/` only because a measuring
lane's evidence lives there (§371.2). Build output on the zram disk is never
evidence. So clearing it can lose nothing but time, and the time is one kache
refill: §373.4 measured a warm restore of the whole workspace's test binaries
at 11 to 16 s.

**So any build directory is cleared once nothing in it has changed for 30
minutes and no cargo is building in it.** That covers a waiting row's tree, an
idle session, a dead one and a finished lane alike, and the rule needs no queue.
If 30 minutes proves too short, it costs a refill.

**`target/` itself is no longer swept.** What is left in it is the evidence
§371.2 protected, which is small. It goes when `prune-worktrees` removes the
tree.

The one guard is not clearing a directory under a build that is running. The
scratch crate left `.cargo-lock` and `.cargo-artifact-lock` in `target/debug`,
not in the build directory. Which lock cargo 1.96 holds during a build, and in
which of the two directories, is for Q3039 to find before it relies on one.

### 474.6 What to read once this has run a week

- **SSD writes a day.** Read the drive's `total_data_written`, as the
  disk-wear notes from 2026-10-31 did: 56 TB in 31 days then, about 1.8 TB a
  day. Compare at the same number of compiles a day, never the raw figure.
- **The zram disk's peak:** `zramctl` shows `DATA` (build output stored) and
  `TOTAL` (the RAM it takes). Read them at a moment when all three slots are
  building. If `TOTAL` comes near the RAM the machine can spare, or a build
  failed for space, change the 80 GB.

### 474.7 The rows

- **Q3037: send cargo's build output to the zram disk.**
  - Set `CARGO_BUILD_BUILD_DIR` to `/mnt/rcad-build/{workspace-path-hash}` in
    `mise.toml`'s `[env]`, off on CI as `RUSTC_WRAPPER` is.
  - Keep the fuzz recipes working. They read build-script output under
    `fuzz/target/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing`,
    which the setting would move.
  - Check every other recipe that reads cargo's layout under `target/`
    (`provenance-cost`'s `target/llvm-cov-target` among them).
  - Add a selftest showing that a build leaves no build output in `target/`.
  - Write the unit that sets up the zram disk at boot, for Mattias to install.
  - The lane first checks `findmnt /mnt/rcad-build`, and stops if it is not
    the zram disk.
- **Q3039, after Q3037: clear idle build directories (474.5)** in place of
  sweeping `target/`, in `clean-targets` and its timer and selftest.
- **Q3038: make the scratchpads, `open-questions.md` and handoffs survive a
  reboot.**
  - Today they are in `/tmp`, which `systemd-tmpfiles` empties at every boot.
    Copying them out first is what `~/tmp-saved-2026-10-31/save.sh` was for, and
    it is the save step Mattias's first condition rules out.
  - The Claude Code docs say `CLAUDE_CODE_TMPDIR` moves the scratchpad root.
    Not yet tried; show a new session's scratchpad landing there before relying
    on it.
  - `questions_root` in the `justfile` defaults to `$TMPDIR/claude-<uid>`.
  - Once off `/tmp`, nothing empties the directory, and on 2026-10-17 it was
    61 GB, most of it old lane probe trees. So the row also says what clears it.
