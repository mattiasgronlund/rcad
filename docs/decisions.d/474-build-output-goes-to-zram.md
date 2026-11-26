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
474.10 adds a second way build output goes: with its tree, when
`prune-worktrees` removes it. 474.11 answers 474.5's question of which lock
cargo holds during a build, and says what `clean-targets` keeps.

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

**474.13 replaces the 30 minutes with 4 hours.**

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
which of the two directories, was for Q3039 to find before it relied on one.
474.11 has the answer: `.cargo-build-lock`, in the build directory itself, and
no lock at all while `cargo test` runs the tests it built.

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
  touches `/var/cache/zram/kache`, which is kache's store. (474.11: the timer
  and its unit change on `main` together with their install, in a row of their
  own.)
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
  still 474.8's reading to take. (Q3037 took it: it does. See the end of this
  section.)
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

- In a fresh tree they fail. This said "with 'rlib was not found'" at first,
  and that was wrong: Q3037's lane ran `check-timing-hooked-calls` on the base
  in a fresh tree, and it stopped with no message of its own, only `just`'s
  `failed with exit code 1`. `find` on the missing directory exits 1, and under
  `set -e` that ended the recipe before the line that prints the message. `fuzz-overnight` has the same line, so it wrote no
  `BUILD FAILED` either.
- `rcad-nightly-fuzz` still holds an rlib from 2026-10-21 at that path, so the
  night's sancov guard reads that old file instead of the one its own build
  made. The guard's promise, that a night can never pass while the current
  rlib carries a sancov reference, does not hold until Q3037 lands.
- A fuzz build was not run to show where the rlib lands now. The evidence is
  `cargo metadata`'s build directory, and 474.2's scratch crate. (Q3037's lane
  ran one. The five fuzz binaries are still copied into
  `fuzz/target/x86_64-unknown-linux-gnu/release/`. The rlib stays in the build
  directory, at
  `x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing/<hash>/out/`.)

Mattias chose to keep the setting and pin Q3037, not take build-dir back out.
Q3037 is reworded to that. Its `mise.toml` line and its zram unit are dropped:
the cargo config and zram-kache already do both. Q3039 no longer waits on it,
since the build directories are already on the disk.

**What Q3037 changed, 2026-11-01.** Every recipe that reads cargo's output was
read, and four things changed or were measured.

- **The sancov guard reads the build directory cargo reports.**
  `fuzz-overnight` and `check-timing-hooked-calls` now run one private recipe,
  `_fuzz-timing-rlib-is-clean`. It asks `cargo +nightly metadata` in `fuzz/`
  for `build_directory` and checks every `librcad_fuzz_timing-*.rlib` under
  it. With build-dir unset, that directory is `fuzz/target`, as before. A tree
  with no rlib now says so, and the night writes `BUILD FAILED`. A stale rlib
  left under `fuzz/target` is no longer read. `check-fuzz-overnight-selftest`'s
  stub cargo answers `metadata` too. Its arms Thirty-eight to Forty run the
  night with build-dir set and with it unset, and the two with it set failed on
  the old recipe.
- **The gate could serve `check-timing-hooked-calls` after a change to its
  own guard.** `just gate` serves that step from an older green when
  `_timing-hooked-calls-fingerprint` has not moved. The fingerprint hashed
  neither the recipe's own text nor `fuzz/rustc-stand-in.sh`, so a change to
  either was served and not run. It now hashes both recipes, as `just --show`
  prints them, and the stand-in. A case in `check-gate-verdicts-selftest`
  failed before this and passes after it.
- **`check-wildcards` had lost its own build directory.** §381 gave its clippy
  `CARGO_TARGET_DIR=target/wildcards`, so that `lint` and it stop taking turns
  in one cache. With build-dir set, both `cargo metadata` and
  `CARGO_TARGET_DIR=target/wildcards cargo metadata` named the same build
  directory, and `check-wildcards` run right after `lint` checked all 8
  workspace crates again. It now sets `CARGO_BUILD_BUILD_DIR` to `wildcards`
  under the build directory cargo reports, which is `target/wildcards` again
  when build-dir is unset. In a fresh tree, `lint`, `check-wildcards`, `lint`
  and `check-wildcards`, one after another, checked 79, 79, 0 and 0 crates.
  `check-wildcards-selftest` runs the recipe under a stub cargo with build-dir
  set and unset, and the case with it set failed on the old recipe.
- **`coverage` needs nothing.** cargo-llvm-cov 0.6.24 follows build-dir by
  itself. On a scratch crate, it built into `<build directory>/llvm-cov-target`,
  wrote its `.profraw` files under `target/llvm-cov-target`, and reported 12
  regions, 8 of them covered. That still shares nothing with the ordinary
  build. `provenance-cost`, which the row named, is a plain `cargo bench` and
  reads nothing under `target/`.

The rest read only the fuzz binaries, which are still copied into
`fuzz/target`. That is `fuzz-triage`, `check-fuzz-triage-selftest` and
`check-gate-fresh-cleanup-selftest`, so they are right as they stand.
`clean-targets` is Q3039's.

**A restore into a build directory reflinks.** Between 18:54Z and 18:58Z, a
fresh tree ran the four runs above and then `just t -p rcad-geom`. Then
`kache report --format json --since 20m --root` with that tree's build
directory gave 202 local hits, 0 misses, `reflinked_bytes` 249,787,538 and
`copied_bytes` 16,285, with `zero_copy_pct` 100.0. The bytes that were copied
are about 0.007% of what was restored.

### 474.10 A removed tree's build directories go with it

*Mattias, 2026-11-02, with the zram disk 79G full: "Should we not alos clear
build directories as part of pruning a workspace?"*

Yes. 474.5 left every build directory to one rule, cleared after 30 idle
minutes, and counted a finished lane among what that rule covers. The rule is
Q3039's and has not landed, so today nothing clears a build directory, and the
ones whose trees are gone are most of the disk. (474.11 is that rule.)

**What was on the disk at 2026-11-02 09:09Z.** `df` gave 79G used of 126G.
Every live worktree was asked for its two build directories, root and `fuzz/`,
with `cargo metadata`. Every directory under `/var/cache/zram/targets` that
none of them named was put down to the workspace its `deps/*.d` files name:

| Whose | Directories | `du` |
|---|---|---|
| a worktree that exists | 19 | 34.8 GiB |
| an rcad worktree that no longer exists | 41 | 69.9 GiB |
| a workspace under `/tmp` | 7 | 1.7 GiB |
| no rcad path and no `/tmp` path in its `.d` files | 21 | 6.4 GiB |

`du` counts a file kache restored by reflink once for every place it sits, so
69.9 GiB overstates the space. 25.5 GiB of it is `incremental*`, which rustc
writes itself and kache never holds, so at least that much is real. The 41 were
all lane trees under `rcad-worktrees/clean-up-dead-sessions-worktrees/`. Eight
of them were looked up by path, and none exists.

**So `prune-worktrees` clears a tree's build directories when it removes the
tree.** That is the one moment something knows for certain no build will use
them again, so it needs no answer to the lock question 474.5 leaves to Q3039.
The idle rule is still wanted. A tree the harness removes, a `git worktree
remove` by hand and a scratch workspace under `/tmp` never pass through
`prune-worktrees`; the last two rows of the table are those.

What the recipe needs:

- **Ask cargo before the tree goes.** `cargo metadata` reads the tree's
  manifest, so it cannot answer after. `build_directory` comes from `cargo
  metadata --no-deps --offline --format-version 1` at the root, and from `cargo
  +nightly metadata` in `fuzz/`, as `_fuzz-timing-rlib-is-clean` asks. Those
  are the only two workspaces: they are the two tracked `Cargo.toml` files with
  a `[workspace]` table. All 73 trees answered both at 09:09Z.
- **Clear only a build directory outside the tree.** With build-dir unset it
  is `<tree>/target`, which goes with the tree already.
- **Never touch `/var/cache/zram/kache`**, which is kache's store.
- **A tree whose cargo cannot answer still goes**, and the recipe prints which
  build directory it could not find. A tree is not worth keeping for its
  cache, and the idle rule reaches what is left.

The 41 already left behind are out of the recipe's reach: their trees are gone,
so cargo cannot name their build directories.

### 474.11 Which lock cargo holds, and what `clean-targets` keeps

474.5 left one question: which lock cargo 1.96 holds while it builds, and in
which directory. **It is `.cargo-build-lock`, in the build directory, one beside
each profile it builds. No lock is held while `cargo test` runs the tests it
built.** So `clean-targets` takes those locks, and also keeps a directory that a
program is running from.

**The measurement.** A scratch crate under cargo 1.96.0, with
`~/.cargo/config.toml`'s build-dir set (474.9) and a `build.rs` that sleeps
15 s. While the build script slept, `lslocks` showed cargo holding three
`flock`s:

- `FLOCK WRITE` on `<build directory>/debug/.cargo-build-lock`;
- `FLOCK READ` on `target/debug/.cargo-lock`;
- `FLOCK WRITE` on `target/debug/.cargo-artifact-lock`.

A `--target x86_64-unknown-linux-gnu` build held those three, and the same
three again under `x86_64-unknown-linux-gnu/debug/`. Once cargo exited, `flock
-n` took every one of them. While `cargo test` ran a test that sleeps 15 s, it
took every one of them too, and the test's process was running
`<build directory>/debug/deps/lockprobe-<hash>`. With the build directory's
`.cargo-build-lock` held from a shell, `cargo build` printed `Blocking waiting
for file lock on build directory` and waited.

The two locks in `target/` say the same about `target/`, which `clean-targets`
no longer touches.

**Where the locks sit on the machine.** At 2026-11-02 15:55Z, the
`.cargo-build-lock` files under `/var/cache/zram/targets` were at these places
inside their build directories:

| Where | How many |
|---|---|
| `debug/` | 52 |
| `release/` | 36 |
| `wildcards/debug/` | 15 |
| `x86_64-unknown-linux-gnu/release/` | 8 |
| `armv7-unknown-linux-gnueabihf/debug/` | 1 |

`wildcards/` is `check-wildcards`'s own build directory inside the tree's
(474.9). So the recipe looks for the lock at any depth, not at a list of
profiles.

**What `clean-targets` does now.** Each directory two levels under
`/var/cache/zram/targets` is one build directory, since `{workspace-path-hash}`
makes two levels. It is cleared only when all of these hold:

- Nothing anywhere in it changed in the last 30 minutes. Any file or directory
  in it decides, not the directory's own time, which moves only when its own
  entries do.
- Every `.cargo-build-lock` in it can be taken with `flock -n`.
- It has at least one, since a directory with none cannot say whether a cargo
  is building there; and `find` read all of it.
- No process is running a program from it, read from `/proc/<pid>/exe`.

It first renamed the directory to a `.clearing-*` name while holding the locks;
474.12 shows why that let a second cargo in and what it does now. A run killed
between the two steps left a `.clearing-*` directory, and a run still deletes
any it finds. A run takes a
lock on `/var/cache/zram/targets` first, and a second run that finds it taken
clears nothing and exits 0. It refuses a directory that holds kache's store or
sits inside it, and exits 1 when `/var/cache/zram/targets` is missing, which is
the disk not being mounted. The recipe's comment block says why for each, and
`check-clean-targets-selftest` builds each case in a miniature disk. The
selftest failed on the old recipe.

**What it would clear today.** The same checks with nothing deleted, at
2026-11-02 15:55Z: 65 build directories, 7 with something changed in the last
30 minutes, none without a lock file, none locked, none running a program, so
58 would be cleared. The pass took 3.9 s. One about ten minutes earlier counted
63 and 57, in 3.6 s. `df` gave the disk 77G used of 126G.

**The unit files are unchanged, and that is a row of its own.** The installed
`rcad-clean-targets.service` runs `just clean-targets` with no arguments in
`main`'s tree, and the recipe's defaults are the zram paths, so it runs the new
rule as soon as this reaches `main`. What is stale is the wording, which still
says it strips `target/`, and the hourly timer: a directory idle for 30
minutes can wait up to 90 before it goes. They are not changed here because
`check-systemd-units` fails any gate whose tracked pair differs from the
installed copy, and the pair is installed on this machine. A lane that changed
the pair would fail its own gate until the install, and every other tree's
gate would fail after it. So the pair changes on `main` together with its
install, as 3e770449f did for `rcad-fuzz.service`.

**Two gaps were left untried here, and 474.12 tries both.**

474.10 says why `prune-worktrees` needs no lock: it clears a build directory
only together with its tree. `clean-targets` never asks which tree a build
directory is for, so the two need no helper in common.

### 474.12 The two gaps in `clean-targets`, tried

Both showed, and both are closed (Q3072, from Q3039). Measured with cargo 1.96.0
and cargo-nextest 0.9.143 on a scratch crate with a `build.rs` that sleeps
12 s and two tests that sleep 8 s each, `CARGO_BUILD_BUILD_DIR` set to a
directory of its own, everything in it set four hours old.

**A nextest run between two tests.** `nextest run -j1`, sampled once a second:
`lslocks` listed no lock under the build directory at any sample, the
`cargo-nextest` process had no open file in it (its fds were the output file and
nothing else), and a program was running from it only while a test ran. Between
the two tests there was no sample-sized window with anything to see, and no
sign a run was in progress. So a `clean-targets` landing there clears the
directory and the next test fails to start. The recipe cannot tell which build
directory a nextest uses: `{workspace-path-hash}` is cargo's, and nextest's
process names only its own working directory. So it keeps every directory
while any process named `cargo-nextest` exists (`pgrep -x`), and says why. That
costs clearing nothing for as long as one run lasts, which is minutes, and the
hourly timer comes round again. This is wider than it needs to be and is the
narrowest thing the measurement allows; a way to read the directory off the
process would narrow it.

**A cargo waiting on the lock while the directory is renamed.** The recipe held
the lock, a `cargo build` started and printed `Blocking waiting for file lock on
build directory`, then the recipe renamed the directory to `.clearing-*` and
let go. `lslocks` then showed the waiting cargo holding `WRITE` on the file now
under `.clearing-*/debug/.cargo-build-lock (deleted)`, and a second `cargo
build` started 3 s later printed `Compiling` at once instead of waiting, so two
cargos built into the same directory. As a second try, the recipe's steps were
replaced by deleting everything but the `.cargo-build-lock` files in place while
holding them, then letting go. The waiting cargo then held `WRITE` on the same
file at its own path, and the second `cargo build` printed `Blocking waiting for
file lock on build directory` and was still waiting when `timeout` stopped it
at 6 s. So the recipe now deletes in place and keeps the lock files. What is
left of a cleared directory is its lock files, which the run reports as kept
(only its lock files are left), after the 30 minutes that deleting in place
itself restarts.

`check-clean-targets-selftest` pins both: a copy of `sleep` named `fx-nextest`
running (the recipe's third argument names the process to look for, so a real
nextest on the machine, such as the gate's own, cannot keep the selftest's
fixtures; the first run of the selftest failed that way), and the inode of a cleared directory's lock file
unchanged by the run. Both fail on the recipe before this change.

**Not tried.** Whether cargo reopens `.cargo-build-lock` if its file is deleted
under a held lock, since the recipe never does that now.

### 474.13 A build directory waits 4 hours, not 30 minutes

*Mattias, 2026-11-26: "Do we really need the scheduled script which remove the
target folder content if it has not been updated under the last 30 minutes, I
think that is managed by kache now". Shown what follows: "Please update the
timer to 4 hours."*

**474.5's reason for 30 minutes no longer holds.** It said a clear loses
"nothing but time, and the time is one kache refill", 11 to 16 s with kache
0.27 (§373.4). kache 2.3.1 compiles the crates being edited without storing
them: its report for a solidcraft edit on 2026-11-26 showed solidcore, engine,
cli and io with size 0 in the store. So those crates exist only in the build
directory. After a clear they compile from scratch, without rustc's incremental
state, and kache forgets which crates were being edited. In the 24 hours to
20:10 on 2026-11-26 the timer cleared one solidcraft session's build directory
four times (21:21, 09:00, 15:40 and 18:40), each time in a pause between two
edits.

**kache does not take over.** Its daemon does clear target directories on its
own, at most once an hour while no build runs (`src/target_cleanup.rs` in
2.3.1, and its configuration docs). With this machine's config it clears only
those whose workspace was deleted, a day after their last build. Idle ones go
only with `cache.auto_clean_idle_targets_days` set, which counts whole days
and is off by default. Below its free-space floor, 10 GiB by default, it evicts
cache entries, and it removes target directories only with
`cache.auto_recover_targets = true`, off by default, and then only ones idle
for a day. Over the same 24 hours the timer cleared 98G as `du` counts it, from
90 build directories, counting a directory again each time it was cleared. The
disk had 29G free at the end. A rule counted in days would not fit.

**So the rule is 4 hours.** Nothing else in `clean-targets` changes, and the
timer still runs every 10 minutes. The selftest's idle fixtures are now five
hours old, and a build directory changed an hour ago must be kept, which fails
on the 30-minute recipe.

**Not measured: what it costs in RAM.** The disk now holds every build
directory written in the last 4 hours instead of the last 30 minutes. 474.6
says what to read. If `df /var/cache/zram` or `zramctl`'s `TOTAL` comes near
what the machine can spare, the time goes down again.
