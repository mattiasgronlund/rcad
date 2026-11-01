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
empties that disk and loses only what a build can make again. The worktrees,
their `target/` directories and everything the tooling writes there stay on
disk. This amends §371.5 and §373.4. The rows are at 474.7.

474.8 is Mattias's layout: the disk is XFS at `/var/cache/zram`, and kache's
store moves onto it too. Where 474.3 and 474.4 disagree with 474.8, 474.8 wins.
474.9 is what was installed, and where it differs from 474.7 or 474.8, it wins.

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

**474.8 overturns this.** Mattias put the store on the zram disk too, and 474.8
shrinks it to fit.

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

**474.8 replaces this section's sizes and its setup.** The disk now holds kache's
store as well, and is XFS.

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
build when the disk is not mounted: `/var/cache/zram` is root's when nothing is
mounted on it, so cargo cannot create a directory there and stops. 474.8 shows
it stopping. That is the failure wanted. A build that quietly falls back to
writing on the SSD would undo this section without anyone noticing.

The setup is in 474.8. **`discard` is what gives memory back.**
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
- **The zram disk's peak:** `zramctl` shows `DATA` (what is stored) and
  `TOTAL` (the RAM it takes). Read them at a moment when all three slots are
  building. If `TOTAL` comes near the RAM the machine can spare, or a build
  failed for space, change the 80 GB. 474.8 adds three more readings.

### 474.7 The rows

- **Q3037: send cargo's build output to the zram disk.** 474.9 cut this row
  down to the recipes, the rest being done.
  - Set `CARGO_BUILD_BUILD_DIR` to
    `/var/cache/zram/targets/{workspace-path-hash}` in `mise.toml`'s `[env]`,
    off on CI as `RUSTC_WRAPPER` is.
  - Keep the fuzz recipes working. They read build-script output under
    `fuzz/target/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing`,
    which the setting would move.
  - Check every other recipe that reads cargo's layout under `target/`
    (`provenance-cost`'s `target/llvm-cov-target` among them).
  - Add a selftest showing that a build leaves no build output in `target/`.
  - Write the unit that sets up the zram disk at boot as 474.8 does by hand,
    for Mattias to install. It makes `kache/` and `targets/` only once
    `mountpoint -q /var/cache/zram` says the disk is mounted: made on the bare
    mountpoint, they would let builds write to the SSD with no error. The disk
    has to be up before Mattias's kache daemon starts.
  - kache's config is Mattias's file and outside the repo, so the lane hands
    him 474.8's two lines and does not edit it.
  - The lane first checks `findmnt /var/cache/zram`, and stops if it is not
    XFS on a zram device. It records whether a restore reflinks (474.8).
- **Q3039, after Q3037: clear idle build directories (474.5)** in place of
  sweeping `target/`, in `clean-targets` and its timer and selftest. It never
  touches `/var/cache/zram/kache`, which is kache's store.
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

### 474.8 Mattias's layout: one XFS zram disk at `/var/cache/zram`, kache's store on it too

*Mattias, 2026-11-01: "I expect the filesystem layout to be: ZRAM backed XFS
filesystem is mounted on /var/cache/zram and that there are: -
/var/cache/zram/kache for kache:s files. - /var/cache/zram/targets for target
folders."*

So the zram disk is XFS, mounted at `/var/cache/zram`. kache's store goes in
`/var/cache/zram/kache`, and cargo's build directories in
`/var/cache/zram/targets/{workspace-path-hash}`. "Target folders" means 474.2's
build directories, not each tree's `target/`. That stays on disk, because the
tooling's evidence lives there. This overturns 474.3, and replaces 474.4's
sizes and setup.

**What it buys: reflinks, and builds off the SSD.**

- A reflink cannot cross filesystems (474.3). With the store and the build
  directories on one XFS, kache can reflink a restored file instead of copying
  it.
- kache tries to. `kache doctor` (0.27.0) printed `FICLONE unsupported
  (expected on ext4)`, and FICLONE is the call that makes a reflink. Among the
  strings in its binary are the build-record fields `reflinked_bytes` and
  `store_reflinked_bytes`. That a restore here does reflink is not tried.
- A reflinked file shares its blocks with the store, so a tree restored from
  kache costs little RAM beyond what the store already takes. What a tree costs
  of its own (incremental state, build-script output, what kache did not
  restore) is unknown until it runs.
- The store's writes leave the SSD too, so a build writes next to nothing
  there. 474.6's SSD reading says how near.

**The store has to shrink to fit in RAM.** Read on 2026-11-01 from kache's
index, opened read-only:

- The store holds 159.1 GiB of distinct files, against `local_max_size =
  "150GiB"`. `du -sh ~/.cache/kache` said 205G, and the index beside the store
  is 9.8 GB of that.
- Store files that some entry used in the last 3, 7 and 14 days: 55.0, 75.5
  and 100.6 GiB. Of those, the files of entries that had been hit at least
  once: 20.1, 36.6 and 51.9 GiB.
- kache evicted 88,968 entries between 2026-10-18 and 2026-10-30, all for
  space. 333 of them, 0.4%, were asked for again afterwards.
- Over the last 7 days, which is as far back as kache's events go (`kache stats
  --since` 7d, 14d and 30d gave the same counts): 23,099 hits, 1,182 misses,
  1,151 duplicates, a 90.8% hit rate.

So the cap becomes **40 GiB**. That holds the 36.6 GiB used in the last week by
entries that had been hit before, but not everything used that week. kache
does not evict by age alone: 66,550 of its evictions were of entries idle for
under a day, and 274 of those were wanted again. So what 40 GiB costs in hit
rate is not known. The comment in `config.toml` says the rate had fallen to 57%
at a 50 GiB cap; that is the risk, and 474.6's hit-rate reading is what moves
the cap. Every 10 GiB more of it costs about 7.7 GiB of RAM at 1.3 to one.

**How well it compresses, one page at a time.** zram compresses each 4 KiB page
on its own, which does worse than compressing whole files. 400 files picked from
each, run through `zstd -b3`, whole and with `-B4096`:

| | whole files | 4 KiB blocks |
|---|---|---|
| kache's store | 1.451 | 1.386 |
| this tree's `target/debug` | 1.410 | 1.325 |

474.3's 1.69 and 474.4's 1.56 came from other samples, with more of their bytes
in big files. The planning figure is 1.3 to one, the worst of the four. That the
kernel's zram compresses at zstd level 3 by default is not checked.

**The disk stays at 80 GiB, sized to fill before RAM runs out.**

- A write zram cannot store, for want of RAM, is an I/O error to XFS. For its
  own records, the kernel's XFS documentation (`docs.kernel.org/admin-guide/xfs`,
  its error settings under `/sys/fs/xfs/<dev>/error/`) says XFS retries or
  fails depending on the error, and that failing "may cause a shut down". What
  it does by default for this error, it does not say. For file data, whether
  cargo or kache ever see the error is unknown. None of it was tried. A build
  that stops is better than one that reads back a file that was never written,
  so the disk must fill and refuse writes while there is still RAM.
- Full, at 1.3 to one, 80 GiB takes about 62 GiB of RAM, which leaves about 63
  of the 125 GiB.
- `sar -r`'s `kbmemused`, in 10-minute samples, peaked at 19.0 to 27.0 GiB a
  day from 2026-10-23 to 2026-10-31, except on 2026-10-25, when it reached
  74.3 GiB. A day like that beside a full disk does not fit, and which gives
  first, the OOM killer or zram, is not known.
- Inside the 80 GiB: the store at its 40 GiB cap; its index, which was 9.8 GB
  after a month and starts empty after a reboot; and about 30 GiB for build
  directories. That is four or five trees at 474.4's 6.6 GB if they share
  nothing with the store, and more with reflinks.

**A reboot empties the store,** so the first build after one is a cold build.
§373.8 has a cold build of rcad-topo alone at 116 s with no kache. The trees
after it restore from what that build stored.

**The kache daemon is shared** with rscene and sketch_solve
(`~/.config/systemd/user/kache.service`), so moving the store moves it for them
too. Their build output stays on the SSD, on another filesystem from the store,
so kache has to copy into them where today it can hardlink, and their entries
share the 40 GiB. Giving them the same `CARGO_BUILD_BUILD_DIR` would put them on
reflinks too. That is outside this repo, and Mattias's to decide.

**A missing disk fails loudly, as 474.4 wants.** `/var/cache` is root's (mode
755), and `/var/cache/zram` does not exist yet. As mattias, `mkdir
/var/cache/zram` said `Permission denied`. A scratch crate built with
`CARGO_BUILD_BUILD_DIR='/var/cache/zram/targets/{workspace-path-hash}'` stopped
with exit 101: `failed to create directory /var/cache/zram/targets/29`, caused
by `Permission denied (os error 13)`. The mountpoint, once made, is root's too,
so the same holds while the disk is not mounted. What kache does when it cannot
make its store is not tried.

**Setting it up, as root, once per boot until Q3037's unit does it.** None of
this was tried:

```
sudo apt install xfsprogs
sudo modprobe zram
sudo zramctl --find --size 80G --algorithm zstd
sudo mkfs.xfs -q -m reflink=1 /dev/zram0
sudo mkdir -p /var/cache/zram
sudo mount -o discard,noatime /dev/zram0 /var/cache/zram
sudo mkdir /var/cache/zram/kache /var/cache/zram/targets
sudo chown mattias:mattias /var/cache/zram/kache /var/cache/zram/targets
xfs_info /var/cache/zram
```

- `xfsprogs` is not installed today: `dpkg -l xfsprogs` says `un`.
- `zramctl --find` prints the device it took. Use that device in place of
  `/dev/zram0` if it names another.
- `xfs_info` has to print `reflink=1`. `-m reflink=1` asks for it rather than
  trusting the default.
- Then, in `~/.config/kache/config.toml`, under `[cache]`:

  ```
  local_store = "/var/cache/zram/kache"
  local_max_size = "40GiB"
  ```

  The key comes from kache 0.27.0's binary, not from its docs. Its strings pair
  `cache.local_store` with `KACHE_CACHE_DIR`, and `kache doctor`'s advice says
  to move "`cache.local_store`/`KACHE_CACHE_DIR`". The file's own comment says
  the daemon restarts itself when the file changes. `kache doctor` should then
  print `Cache dir /var/cache/zram/kache`.
- The old store in `~/.cache/kache` is then read by nothing. When to delete it
  is Mattias's call. Keeping it until 474.6's week has run costs only disk.

**Four more readings for 474.6:**

- `xfs_info /var/cache/zram` says `reflink=1`, and `cp --reflink=always` from
  `/var/cache/zram/kache` into `/var/cache/zram/targets` works.
- `reflinked_bytes` against `copied_bytes` in `kache report --format json`,
  for a restore into a build directory: whether kache reflinks here at all.
- `zramctl`'s `DATA` over `COMPR` is the real ratio, in place of 1.3. With
  `sar -r`'s peak beside `TOTAL`, it says whether the 80 and the 40 GiB can
  grow.
- `kache stats --json --since 7d`'s hit rate, against 90.8%.

### 474.9 What was installed: zram-kache, with build-dir set for every build

*Mattias, 2026-11-01, after running his own `setup-zram-kache.sh`, when asked
whether to take build-dir back out of `~/.cargo/config.toml` until Q3037
lands: "Keep it, we lets pin Q3037."*

Mattias installed the layout with his own tool, zram-kache
(`~/setup-zram-kache.sh`, `/usr/local/sbin/zram-kache`, `/etc/zram-kache.conf`).
Read at 2026-11-01 17:55Z, it differs from 474.8 in three places. Where they
differ, this section is what is on the machine.

- **The disk is the size of RAM, and spills to a file on disk.** `zramctl`
  shows `/dev/zram1` at 125.7G. Behind it is a backing file,
  `/var/lib/zram-kache/backing.img` (134,981,136,384 bytes), and
  `zram-kache-psi.service` writes pages out to it when memory runs short. That
  replaces 474.8's rule of a disk small enough to fill before RAM runs out.
  Whether writing out keeps up with builds that fill RAM fast is not tried. If
  it does not, zram fails the write as 474.8 says. The backing file took `/` to
  84% full, 148G free.
- **kache's cap is 62 GiB**, the script's default of half the disk, not 40.
- **build-dir is set for every cargo build on the machine**, in
  `~/.cargo/config.toml`: `build-dir =
  "/var/cache/zram/targets/{workspace-path-hash}"`. 474.7 had planned it in
  rcad's `mise.toml`. `cargo metadata` now gives this tree's build directory
  as `/var/cache/zram/targets/92/c34bc3ea8c8c4c`, and nightly cargo
  (1.100.0-nightly) gives the fuzz workspace's as `.../2b/d0e823affcbaef`.
  rscene and sketch_solve get it too, so their restores can reflink as well.
  CI has no such file.

**What works.**

- `xfs_info` says `reflink=1`. `cp --reflink=always` of an 8 MiB file inside
  `/var/cache/zram/targets` worked, and `cmp` found the two the same.
- `kache doctor` says `Cache dir /var/cache/zram/kache` and `Cache FS xfs
  (local)`. Its `Link layout` check fails when run from a worktree, with
  `EXDEV: build tree is on mount /`, because it judges from the directory it
  runs in and not from cargo's build directory. Whether a restore reflinks is
  still 474.8's reading to take.
- A missing disk still fails builds. zram-kache makes the bare mountpoint as
  root (`mkdir -p`) and hands Mattias only the mounted filesystem (`chown` after
  `mount`). `kache/` and `targets/` come from the unit's `ExecStartPost`, which
  runs only after a start that worked. So 474.8's demonstration still holds.
- Mattias's user manager starts after `zram-kache.service`, through a drop-in
  for `user@1000.service`.

**What broke: two recipes read cargo's old layout.** `fuzz-overnight` and
`check-timing-hooked-calls` look for `rcad-fuzz-timing`'s rlib under
`fuzz/target/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing`, and
builds no longer write there.

- In a fresh tree they fail with "rlib was not found".
- `rcad-nightly-fuzz` still holds an rlib from 2026-10-21 at that path, so the
  night's sancov guard reads that old file instead of the one its own build
  made. The guard's promise, that a night can never pass while the current
  rlib carries a sancov reference, does not hold until Q3037 lands.
- A fuzz build was not run to show where the rlib lands now. The evidence is
  `cargo metadata`'s build directory, and 474.2's scratch crate.

Mattias chose to keep the setting and pin Q3037, not take build-dir back out.
Q3037 is reworded to that. Its `mise.toml` line and its zram unit are dropped:
the cargo config and zram-kache already do both. Q3039 no longer waits on it,
since the build directories are already on the disk.
