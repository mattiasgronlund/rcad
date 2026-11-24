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

**The fuzz workspace's directory is linked too (Q3107).** The first gate in
`bdd-3` ran with `fuzz/target/x86_64-unknown-linux-gnu/release` linked, and
`check-fuzz-overnight-selftest` failed in arm `twenty_nine`: "none of 26 stale
rlib names sorted before the real one under find, on 8 different real names in
a row". That arm planted its fixtures under
`fuzz/target/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing` and needs
a directory made later to come first under `find`. ext4 lists a directory in
hash order, which gives that. xfs, the zram disk's filesystem, lists a small
directory in the order its entries were made, which never does. Run with the
arm's own pattern of names (`target/order.sh` in `bdd-3`), a later directory
came first in 0 of 8 attempts on xfs and in 7 of 8 on ext4. So Q3069 left the
link out, and Q3106 made the arm put a `find` of its own in front of the
night's `PATH` that lists in reverse name order, which does not depend on the
filesystem. With that done, `link-targets.sh` links the directory: it asks
`cargo +nightly metadata` in `fuzz/`, since the fuzz workspace builds on the
nightly toolchain, and keys the link by that workspace's own build directory,
so its destination is not the root's. `check-link-targets-to-zram-selftest`
checks the link is made, its stub cargo answering a key of its own for
`+nightly`, and that nothing is made where cargo fails or build-dir is unset.
The gate's own steps put no binary there: after the first gate the destination
held only the arm's fixtures. The five fuzz binaries come from `cargo fuzz
build`, in a night or a fuzz recipe.

**Nothing else the tooling writes goes under a linked directory.** A grep of
`.claude/` and `tooling/` found no writes under `target/debug` or
`target/release`. In the justfile, the only writes under them are cargo's own
copies, such as the ones `cargo run --profile test -p rcad-fuzz-explain` makes in
`target/debug`, and those copies are what the link is for. Under the fuzz
triple's directory, the justfile writes only the fuzz-overnight selftest's
fixtures, above, which now sit on the zram disk too. `bdd-2` found kache's `.kache-build-script-shims` in
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

**Installed, 2026-11-03.** Mattias ran all four, from the hub rather than main's
tree, at 7243e8238 (the recipe copies the hook into the common `.git/hooks`, so
the tree it runs from does not matter). Checked by the dispatcher afterwards:
the hook and `/usr/local/sbin/rcad-zram-artifact-dirs` are byte for byte the
tracked files, the second owned by root with mode 755; `systemctl show
zram-kache` lists all four `ExecStartPost` lines, with `NeedDaemonReload=no`.
`git worktree add --detach` of a scratch tree at 121def0d3 got `target/debug`
and `target/release` as links into `/var/cache/zram/artifacts/4a/e4454a24fdbaf8/`,
destinations made. With those two destinations removed, the installed boot step
run by hand as `mattias` with the unit's arguments printed `made 2
destinations, 1 already there, 0 links refused, 0 could not be made` in 2.7 s,
warm cache. The scratch tree and its destinations were removed after. The first
run at boot, with a cold cache, is still unmeasured.

### 476.6 What Q3070 does: idle trees get their links, and the copies go with their build directory

*Q3070's three lanes, 2026-11-03, in trees `wf_af1988e5-529-2`, `-3` and `-4`.
Each measurement says which tree it was made in.*

**Why it is needed.** In `529-2`, a survey of every rcad worktree found real
directories where the links belong, the largest being the dispatcher's own
tree, `clean-up-dead-sessions`, with a real `target/debug` of 20G and
`target/release` of 361M, both holding a whole build from before build-dir was
set (474.9). A tree made before the hook of 476.5 has them, and so does a tree
after a plain `cargo clean`.

**`clean-targets` replaces an idle tree's real `target/debug` and
`target/release` with their links,** in a pass before it clears the build
directories. It asks `git worktree list` for the trees, and `cargo metadata
--format-version 1 --no-deps --offline` in each tree for its build directory,
whose last two parts are the key of its links (476.3), as
`tooling/zram/link-targets.sh` reads it. The key is `{workspace-path-hash}`,
cargo's own hash of the workspace path, and the recipe asks cargo for it rather
than computing it, since a copy of cargo's hash here would go wrong without a
word the day cargo changes it. Reading it the other way round, from a build
directory's `.d` files, which name the tree, fails exactly where it is needed:
a cleared build directory has none left. So the recipe now needs git and
cargo. The unit's `PATH` has no cargo: in `529-2`, the unit's
environment under `mise exec` printed `cargo: command not found`. So when
`PATH` has no cargo, the recipe adds `${CARGO_HOME:-~/.cargo}/bin` to it. It
runs `cargo metadata` with a 20 s timeout and `RUSTUP_AUTO_INSTALL=0`, so a
missing toolchain is a refusal, not a download.

**A real directory is kept** for every reason a build directory is kept (474.11),
read over two directories: the real directory, with its `.cargo-lock`, and the
tree's build directory, with its `.cargo-build-lock`. In `529-2`, cargo 1.96
held `target/debug/.cargo-lock` while it built: a scratch crate whose `build.rs`
sleeps 6 s, probed at 3 s, printed `artifact .cargo-lock: HELD during build`.
The build directory is asked too because a test runs from there, and through
`CARGO_BIN_EXE_<name>` it may run the binary that sits in `target/debug`. A
build directory that does not exist is skipped, since a tree that has never
built has none. Three more reasons keep it:

- **A gate is running in the tree.** Between two of its steps no lock is held
  and no program runs, so nothing above shows it. The gate keeps
  `target/last-gate.log.run/pid`, and while it runs that pid is a bash whose
  command line has `<run>/script` as one argument. The recipe asks for both,
  because a pid alone may be another process once the pids wrap.
- **`cargo metadata` failed in the tree**, so its key is not known.
- **The build directory is not `<root>/<xx>/<hash>`, or the target directory
  is not `<tree>/target`**, so a link would point at no key or sit in the wrong
  place.

**How it replaces one.** It holds both directories' locks, makes the
destination, moves the real directory aside to `.<name>-replaced-by-a-link`,
makes the link, lets go of the locks, and only then deletes the old
directory, since 20G takes a while. If the link cannot be made and nothing has
taken the path, the directory is moved back. A run killed between the move and
the delete leaves the `.<name>-replaced-by-a-link` directory, and the next run
removes it. A cargo that opened the old `.cargo-lock` before the move gets it
once the locks are let go, and writes through the link. A cargo that starts
after the move opens the destination's `.cargo-lock`, a different file. The two
are kept apart by the build directory's `.cargo-build-lock`, which stays where
it is and which cargo 1.96 holds from a build's start to its exit (474.11).
This answers 476.3's question about `cargo
clean`: the link is made again once the tree is idle.

**The build loop clears a build directory's copies with it.** The build
directory `<root>/<xx>/<hash>` and its link destinations
`<artifacts>/<xx>/<hash>` are kept or cleared together. The destinations are
read as the second directory, with their `.cargo-lock`, which cargo holds
through the link while it builds. A fresh file in either one, a held lock in
either one, or a program running from either one keeps both. Under
`artifacts`, only the directories beside a `.cargo-lock`, two or more levels
down, are cleared, with what is under them. inari links its whole `target/`
under the same root, and in `529-2` one of its destinations held
`/var/cache/zram/artifacts/5a/fc54fb831019f4/.rustc_info.json`, with `doc/`,
`tmp/` and `package/` beside its profiles. Those are not cargo's copies, and
nothing says whether they are in use. A cleared destination keeps its lock
files and its own directory, since a link to a missing directory fails the next
build with `File exists (os error 17)` (476.5).

A key the tree pass has just converted is kept by the build loop until 30
minutes have passed, because making the destination is a change in it. A later
run clears it as usual.

**`prune-worktrees` removes a removed tree's `<artifacts>/<xx>/<hash>` whole,**
along with the build directory it pairs with (474.10). It does so even when that
build directory is gone already. Nothing else removes a destination: the boot
step makes one for each link, and `clean-targets` only clears one.

**The selftests.** `check-clean-targets-selftest` has a miniature `artifacts/`
beside its `targets/` and `kache/`, a git repo of its own with one worktree for
each reason a real directory is kept, and a stub `cargo` that answers only for
those worktrees, so no real tree is ever asked. Its idle tree has both
directories replaced, with the tooling's `last-gate.log` left in `target/`. It
pins the tallies: in the ordinary run, `2 real target directories replaced with
links; 7 kept` and 9 build directories kept, and in the run after it, `0 real
target directories replaced with links; 7 kept` and `0 cleared; 13 build
directories kept`. `check-prune-worktrees-selftest` gives each tree's stub key a
`<xx>/<hash>` shape and checks that a removed tree's destinations go, that a
kept tree's stay, and that a tree whose build directory was already gone loses
its destinations too. Both selftests fail on the base's recipes, which take
none of the new arguments.

**What this leaves.**

- **Destinations whose build directory is gone** were never cleared by
  `clean-targets`, because its build loop walks the build directories. In
  `529-4`, 13 keys under `/var/cache/zram/artifacts` had no build directory and
  held 344M between them, inari's `5a/fc54fb831019f4` among them. Most are from
  trees removed before this section. They cannot be removed whole, since a tree
  with its links that has not built since the boot has no build directory
  either, and removing its destinations breaks its next build. Q3118 clears
  their files, in a second loop after the build loop, under the idle rules of a
  build directory with the destination itself as the directory asked about: idle
  30 minutes, no `.cargo-lock` taken, no program running from it, and not the
  destination of a running nextest's tree. Directories and lock files stay, as
  in the build loop. It prints `cleared <size>: <destination>, a link
  destination with no build directory` for each and a tally line, `<size>
  cleared from link destinations with no build directory; <n> kept`. A
  destination with no `.cargo-lock` under it is kept, the same rule as any
  directory that cannot say whether a cargo is building. That keeps inari's
  `5a/fc54fb831019f4`, whose profiles are not cargo's copies, and anything else
  without a lock file. Clearing those would need a rule that tells a tree's
  link destination from a stray directory, and that is not decided here.
- **`tooling/systemd/rcad-clean-targets.service`** says the recipe needs neither
  git nor anything from `~/.cargo`. Both are false now. That is a row too, since
  the unit is outside Q3070's write set.

### 476.7 `target/ci` is linked too, for solidcraft's trees

*A session working in `~/git/tangency-labs`, 2026-11-24. Mattias, in it:
"Please make sure that target/ci is added to link-targets.sh."*

**Why.** Mattias links his solidcraft checkouts with this script by hand, since
solidcraft has no hook for it. solidcraft's root `Cargo.toml` has a
`[profile.ci]`, `cargo xtask fast` builds with `--profile ci`, and its xtask
runs `cargo run --profile ci -p solidcraft-cli` (`xtask/src/external_corpus.rs`,
`xtask/src/app_icons.rs`), so cargo copies `solidcraft-cli` into `target/ci`.
That is read from solidcraft's code at `ad3164473`, not measured: the checkout
had no `target/ci` to look at.

**In an rcad tree the link points at an empty directory.** rcad has no `ci`
profile. 476.5 left out `target/wildcards/debug` and `fuzz/target/debug`,
because cargo copies nothing into them. `target/ci` is linked anyway, as
`target/release` is (476.5): a build with that profile copies into it, and an
empty destination costs one directory.

**What changed.** `link-targets.sh` links `ci` beside `debug` and `release`.
`clean-targets` replaces a real `target/ci` in an idle tree as it does the other
two, and counts a running nextest's tree's `target/ci` as in use.
`check-link-targets-to-zram-selftest` checks that the `ci` link is made, and
that it is not made where cargo fails or build-dir is unset. In
`check-clean-targets-selftest` the idle tree has a real `target/ci` too, so the
tally of the ordinary run is now `3 real target directories replaced with links;
7 kept`. The hook and the boot step need no change: the hook runs the tree's
own script, and the boot step makes the destination of every link into
`artifacts`.

**What this leaves.**

- **solidcore's own workspace inside solidcraft** sets `target-dir = "bin"` in
  `solidcore/.cargo/config.toml`, so its test, goldens and diagnostics
  binaries are copied into `solidcore/bin/<profile>` on the SSD. The script
  links only `<tree>/target`.
- **The hub's checkout is at `7243e8238`**, older than 476.5's fuzz link and
  this section. A solidcraft tree must be linked with main's copy of the
  script, not the hub's.
