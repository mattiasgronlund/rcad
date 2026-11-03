## 476. Cargo's copies of binaries into `target/` go to zram too, and `target/` itself stays on disk — 2026-11-02

*Session `mattias-3d`, 2026-11-02, wrote to this session that Mattias had asked
it to describe the problem below for this session to build. Those are its
words, not his. His, in this session the same day: "The worktree should stay
but target/ should move so that kache may use reflinks..." Asked whether the
tooling's own files in `target/` should move with it, he chose "Link cargo's
dirs": `target/` stays a real directory on disk, and the directories cargo
copies its output into become links into `/var/cache/zram`. Amends 474.2.*

**In short.** 474.2 left one thing unmeasured: how many bytes cargo copies
back into `target/` for a binary. It is tens of megabytes per tree on every
link, written to the SSD. Cargo copies by hard link where it can, and it cannot
across two filesystems. So the directories it copies into go onto the zram
disk too, and the hard link works. `target/` itself, and everything the tooling
writes at its top level, stays on disk, so 474.2's rule still holds: nothing on
the zram disk is ever the only copy of anything.

### 476.1 What cargo copies, and where

After a build, cargo puts each finished binary, example and some libraries
into `target/<profile>/` (`debug/`, `debug/examples/`, `release/`), from the
build directory 474.9 set. Across two filesystems that is a full copy.

**Read in this session,** at 17:00-17:01 CET in Q3039's lane tree
(`wf_c2796423-42a-3`): 8 files over 2 MB under `target/debug`, about 92 MB,
each with a link count of 1. The largest were `rcad-fuzz-explain` (19 MB),
`examples/blocker_depth_probe` (13 MB), `examples/near_parallel_which_pair`
and `examples/near_parallel_census` (13 MB each), and
`librcad_fuzz_explain.rlib` (7 MB).

**Reported by `mattias-3d`, not run here:** 60-140 MiB of these per lane tree,
sometimes with `librcad_topo.rlib` (76 MiB), written again on every link. Since
zram-kache went on (2026-11-01 18:42), 173 GiB of writes went to zram and 88
GiB to the NVMe, and kache restored 95.8 GiB by reflink with no copies. Its
tests, on a scratch crate under cargo 1.96:

1. `build.target-dir` does not expand `{workspace-path-hash}`; the braces stay
   in the path. One fixed target-dir for all trees would have them overwrite
   each other's binaries.
2. With `target` a link to a directory on the zram disk, the copy becomes a
   hard link: same inode, link count 2, nothing written.
3. With the link's destination missing, which is the state after every boot,
   the build fails with `Not a directory (os error 20)`.
4. `cargo clean` deletes the link, and the next build makes a real `target/`
   on the SSD without a word.
5. A `post-checkout` hook in the common `.git/hooks` runs on every `git
   worktree add`, with and without `--detach`, inside the new tree, with `$1`
   the all-zero sha. This repo has no hooks and no `core.hooksPath` today.

It tested `target` linked whole. This section links the directories inside it
instead, which it did not test (476.3).

### 476.2 Why not `target/` whole

`target/` is where the tooling writes what it keeps: the justfile names
`target/` 139 times, among them `last-gate.log`, `land-steps.txt`,
`verify-lane.txt` and the fuzz recipes' `crash-`, `timeout-` and `slow-unit-`
files. `CLAUDE.md` tells every lane to write its scripts there, and lanes keep
their measurements there for their verifier and merger (§371.5). This tree's
own `target/` holds probe scripts and saved evidence such as `q985/tree.tar`.
On the zram disk a reboot loses all of it, which breaks Mattias's condition in
474: nothing to save before a reboot. Moving those files to another directory
would mean changing 139 paths and every agent file that says `target/`, to save
nothing that linking cargo's own directories does not already save.

### 476.3 What the rows must show

- **That linking `target/debug` works as linking `target/` did.** The copy
  must come out a hard link (link count 2), and a build must still pass. If
  cargo refuses a link one level down, say so with the error and take the
  next level that works.
- **Which directories cargo copies into.** At least `target/debug` and
  `target/release` in the root workspace, the five fuzz binaries in
  `fuzz/target/x86_64-unknown-linux-gnu/release/` (474.9), and whatever
  `CARGO_TARGET_DIR=target/wildcards` in `check-wildcards` gets. Read them off
  a build, not this list.
- **Nothing the tooling writes goes under a linked directory.** Grep the
  justfile, `.claude/` and `tooling/` for writes under `target/debug`,
  `target/release` and the fuzz triple's directory, and move any you find.
- **A link whose destination is missing is made again, not failed on.** After
  every boot the destinations are gone (test 3). The boot step is a script run
  from `zram-kache.service`'s `ExecStartPost`, in the drop-in
  `/etc/systemd/system/zram-kache.service.d/subdirs.conf` that already makes
  `kache/` and `targets/`. It finds links under `~/git` that point into
  `/var/cache/zram` and makes their destinations, owned by `mattias`. It also
  covers inari's trees, which `mattias-3d` gave inari the same brief for. No
  session can `sudo`, so the row writes the script, keeps it tracked, and gives
  Mattias the exact commands to install it and run `systemctl daemon-reload`.
  Never restart `zram-kache`: a restart empties the disk.
- **A new tree gets its links when it is made,** by a `post-checkout` hook
  (test 5) that runs only when `$1` is the all-zero sha, is fast, and always
  exits 0, so it can never fail a checkout. It is tracked in this repo and
  installed into the common `.git/hooks` by a recipe that Mattias runs, not
  a session: on 2026-11-02 auto mode blocked the inari session from installing
  its own hook into its `.git/hooks`. So the recipe is tested on a scratch repo
  only, and no gate step fails because the hook is missing. A tree made before
  it is installed is the second row's to convert.
- **The destination is named after the tree, the way inari names it.** inari's
  session, 2026-11-02: each of its two workspaces, `target` and `bench/target`,
  links whole to `/var/cache/zram/artifacts/<xx>/<hash>`, the last two parts of
  the workspace's `build_directory` from `cargo metadata --format-version 1
  --no-deps --offline`, so that it pairs with `/var/cache/zram/targets/<xx>/<hash>`.
  rcad keys the same way and links the directories inside `target/` under it.
  One boot script then serves both: it makes the destination of every link
  that points into `/var/cache/zram/artifacts`, as `mattias` with mode 755,
  looking under `~/git` and under `/tmp/claude-1000`, where inari keeps
  scratch trees.
- **What `cargo clean` does to a link is answered** (test 4): either something
  that runs often makes a link again where a real directory has appeared, or
  the reason that is not needed is written here.

### 476.4 The rows

- **Links, the hook and the boot step:** everything in 476.3. It comes first.
- **Converting the trees there are, and clearing the destinations:** after the
  first and after Q3039, which is rewriting `clean-targets` as this is written.
  An idle tree whose `target/debug` is a real directory has it replaced by the
  link, under the rules `clean-targets` already keeps for idle and locked trees:
  never a tree a build or gate is running in. `prune-worktrees` (474.10, Q3053)
  removes a removed tree's link destinations with its build directories. Q3039's
  idle rule clears them with the build directory too. While the build directory
  is there the two share their bytes through the hard links, but once it is
  cleared a destination alone holds tens of megabytes of RAM.

### 476.5 What Q3069 found

*Q3069's two lanes, 2026-11-03. The first worked in tree `wf_84a56c29-bdd-2`
and the second in `wf_84a56c29-bdd-3`; each finding says which.*

**Linking `target/debug` one level down works as linking `target/` did.** In
`bdd-2`, with `target/debug` a link to
`/var/cache/zram/artifacts/71/3fd20140d51477/debug`, `cargo build -p
rcad-fuzz-explain --bins --lib` passed in 13.07 s, and
`target/debug/rcad-fuzz-explain` (36,170,904 bytes) was inode 140240753 with
link count 2, the same file as the build directory's `deps/` copy. In `bdd-3`,
after `just pre-gate gate-lane`, all 8 files over 1 MB in the linked
`target/debug` had link count 2: `rcad-fuzz-explain`,
`librcad_fuzz_explain.rlib` and six examples, 92,521,353 bytes. With
`target/debug` on the SSD, each would have been a full copy (476.1).

**A missing destination fails differently one level down.** In `bdd-2`, a build
of a scratch crate whose `target/debug` linked to a missing directory failed
with `File exists (os error 17)`, not 476.1's `Not a directory (os error 20)`
for `target` linked whole. Either way the boot step must make it first.

**Which directories cargo copies into, read off a gate.** In `bdd-3`, with
`target/debug` and `target/release` linked, `just pre-gate gate-lane` and then
`just check-release-build` left no file over 1 MB with link count 1 under
`target/` or `fuzz/target/` (`find target fuzz/target -xdev -type f -size +1M
-links 1` printed nothing). Cargo made two more real directories:
`target/wildcards/debug`, for `check-wildcards`' clippy, and `fuzz/target/debug`.
Each holds only a `.cargo-lock`, so neither is linked. `target/release` got no
binary either, since a release test binary stays in the build directory. It is
linked anyway, because any `cargo build --release` copies into it.

**The fuzz workspace's directory is not linked yet.** The first gate in `bdd-3`
ran with `fuzz/target/x86_64-unknown-linux-gnu/release` linked too, and
`check-fuzz-overnight-selftest` failed in arm `twenty_nine`: "none of 26 stale
rlib names sorted before the real one under find, on 8 different real names in
a row". That arm plants its fixtures under
`fuzz/target/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing` and needs
a directory made later to come first under `find`. ext4 lists a directory in
hash order, which gives that. xfs, the zram disk's filesystem, lists a small
directory in the order its entries were made, which never does. Run with the
arm's own pattern of names (`target/order.sh` in `bdd-3`), a later directory
came first in 0 of 8 attempts on xfs and in 7 of 8 on ext4. With that link, the
arm fails in every tree. The arm is outside Q3069's write set, so
`link-targets.sh` leaves the fuzz directory alone, and
`check-link-targets-to-zram-selftest` fails if anything is made under
`fuzz/target`. A row moves the arm's fixtures, and a row after it adds the link.
The gate's own steps put no binary there: after the first gate the destination
held only the arm's fixtures. The five fuzz binaries come from `cargo fuzz
build`, in a night or a fuzz recipe.

**Nothing else the tooling writes goes under a linked directory.** A grep of
`.claude/` and `tooling/` found no writes under `target/debug` or
`target/release`. In the justfile, the only writes under them are cargo's own
copies, such as the ones `cargo run --profile test -p rcad-fuzz-explain` makes in
`target/debug`, and those copies are what the link is for. Under the fuzz
triple's directory, the justfile writes only the fuzz-overnight selftest's
fixtures, above. `bdd-2` found kache's `.kache-build-script-shims` in
`target/debug` in trees made before build-dir was set. It was not there after
`bdd-3`'s gate.

**What `cargo clean` does to a link** (476.3), in `bdd-2` on a scratch crate
whose `target/debug` was a link:

- Plain `cargo clean` removes `target/`, which takes the link itself but not its
  destination, and it removes the build directory too. The destination keeps
  the binary, now with link count 1, still taking RAM. The next build makes a
  real `target/debug` on the SSD (inode 52299735, link count 1).
- `cargo clean --profile dev` also deletes the link and leaves the
  destination's files.
- `cargo clean -p <package>` keeps the link and deletes that package's files in
  it.

Nothing in the justfile, `.claude/` or `tooling/` runs `cargo clean`. So
nothing yet makes a link again where a real directory has come back. Q3070 will:
it replaces a real `target/debug` in an idle tree. Until then,
`just link-targets-to-zram`, run after a `cargo clean` and before the next
build, makes the link again.

**The hook and the boot step.** `tooling/zram/post-checkout` acts only when its
`$1` is the all-zero sha, runs the new tree's own `tooling/zram/link-targets.sh`,
and always exits 0. On a tree already linked, that script ran in 0.097 s in
`bdd-3`. `tooling/zram/make-artifact-dirs.sh` is the boot step. Its search of
`~/git` and `/tmp/claude-1000` for links into `/var/cache/zram/artifacts` took
2.4 s with the disk cache warm. It leaves out `.git` and anything more than two
levels inside a `target`. Pruned that way, it walked 999,735 entries in `bdd-2`'s
count, against 1,479,358 unpruned. The time with a cold cache, as at boot, was
not measured. `zram-kache start` hands the new disk's root to its owner, so the
step runs as `mattias`, and every directory it makes is his.

**What Mattias runs** to install both. No session can: auto mode refused the
inari session its own hook (476.3), and the boot step needs `sudo`.

1. From main's tree: `just link-targets-to-zram install-hook`.
2. `sudo install -m 0755 -o root -g root tooling/zram/make-artifact-dirs.sh
   /usr/local/sbin/rcad-zram-artifact-dirs`. It is a copy owned by root, because
   a unit started as root must not run a file that any session can edit. A
   change to the tracked script reaches the copy only when this is run again.
3. Add two lines to `/etc/systemd/system/zram-kache.service.d/subdirs.conf`,
   after the two that are there:

   ```
   ExecStartPost=/usr/bin/install -d -m 0755 -o mattias -g mattias /var/cache/zram/artifacts
   ExecStartPost=-/usr/sbin/runuser -u mattias -- /usr/local/sbin/rcad-zram-artifact-dirs /var/cache/zram/artifacts /home/mattias/git /tmp/claude-1000
   ```

4. `sudo systemctl daemon-reload`, and never a restart of `zram-kache`: a
   restart empties the disk. The lines take effect at the next boot.

A tree made before the hook is installed has no links, and a gate there passes
just the same. Converting those trees is Q3070's.
