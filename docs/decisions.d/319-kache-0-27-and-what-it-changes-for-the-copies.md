## 319. kache 0.27, and what it changes for the copies — 2026-09-28

*At Mattias's request: "Upgrade to kache 0.27 as there has been improvements in
this area." rcad-18, straight after §315.*

### 319.1 What moved

- `mise.toml` pins `cargo:kache` at 0.27, and `mise.lock` says 0.27.0.
- **One daemon serves rcad, rscene and sketch_solve.** It is the user service
  `~/.config/systemd/user/kache.service`, and it ran
  `.../cargo-kache/0.16.0/bin/kache` by that versioned path. `kache daemon
  install` under 0.27 records mise's `latest` alias instead,
  `.../cargo-kache/latest/bin/kache`. So the daemon now runs **the newest kache
  installed on this machine**, whichever repo pinned it. The next upgrade needs
  no reinstall, and installing a newer kache just to try it moves the daemon
  too. `kache doctor` passes every check.
- **The cache went cold once.** 0.27 builds its keys a new way: its entries
  carry `key_schema` 32, and the 43 925 entries 0.16 wrote carry 27. So nothing
  0.16 stored is a hit for 0.27. The first build of the test binaries under 0.27
  took 227 s; the same build from the cache takes 12 s. The old entries stay
  until eviction reaches them by size and age.

### 319.2 What it does for the copies: nothing

The disk problem is §203.1's and §315's: every tree holds its own copy of every
test binary. How it was measured: build the test binaries (`cargo nextest run
--workspace --no-run`) into a fresh `CARGO_TARGET_DIR`, then count, **by inode**,
the bytes that share their inode with something outside that directory. Cargo's
own links inside one tree (`deps/x-<hash>` to `x`) do not count. Each row is
one run, made while rcad-0e's lanes were building.

| tree, in order built | kache | from cache | shared outside the tree | the tree's own |
|---|---|---|---|---|
| k16a | 0.16 | 40 s | 0 | 5.74 GB, 5.16 GB executables |
| k16b | 0.16 | 35 s | 0 | 5.74 GB |
| k27a | 0.27 | cold, 227 s | 0.03 GB | 5.74 GB |
| k27b | 0.27 | 12 s | 0.16 GB, 26 files | 5.61 GB, 5.03 GB executables |
| k27d, beside k27c | 0.27 | 12 s | 0.03 GB | 5.74 GB, 5.17 GB executables |

- **Test executables are still private copies**, by design. 0.27's
  `wrapper.rs:4865` says: "User-facing binaries, tests, examples and other
  dylibs stay private copies, because a post-build `strip` rewrites them in
  place."
- **What 0.27 does share** is proc-macro `.so` files and build-script binaries
  (kache #1187). That is the 0.16 GB in k27b.
- **But a blob still links into one tree at a time** (#794), and that covers
  those 0.16 GB too. k27b got the links. k27c was built after k27b was deleted,
  so it got them again. k27d was built while k27c still held them, and got
  copies: `libserde_derive` has link count 2 in k27c and 1 in k27d. With a dozen
  lane trees on disk, that is the usual case.
- The 0.03 GB that every 0.27 tree shares is a copy of the `kache` binary itself,
  under `.kache-build-script-shims/`.
- No rlib shared its inode in any 0.27 tree, not even in k27b, which was the
  first tree to restore them. Those are 0.39 GB a tree. Why was not looked into.

So a tree costs what it cost under 0.16, and removing trees (§315) is still the
fix for the disk. What 0.27 gained is speed: a build from the cache took 12 s,
against 29–40 s under 0.16. That speed-up is not from linking, because k27d
copied as much as k16b did.

### 319.3 Old wrappers against the new daemon

Every lane on a base from before this commit runs a 0.16 wrapper, and so do
rscene and sketch_solve until they bump their own pins. Those wrappers talk to
the 0.27 daemon and use the index it migrated. kache promises nothing about this
mix, so here is what makes it work:

- **The index.** 0.27 migrates `index.db` to generation 7 when it first opens it
  (`kache-store` 0.27, `store.rs`, `initialize_db`). Every step adds a table or
  a column, except two. It rebuilds `file_hashes` WITHOUT ROWID, and it drops
  `source_env_runtime_uses`. 0.16 never reads `rowid` or `user_version`. Its
  own `initialize_db` runs on every open and creates `source_env_runtime_uses`
  again (`kache-0.16.0`, `cache_key.rs:3165`).
- **The daemon.** A client restarts a daemon only when the client's binary is
  newer by mtime (0.16's `daemon.rs:269`, `client_epoch_is_newer`). The 0.16
  binary dates from 2026-09-02, so it never restarts the 0.27 daemon.
- **Shown, not only read.** The daemon's first cleanup pass logged "rebuilt
  file_hashes without rowid" at 08:40:14Z. After that pass, a 0.16 wrapper built
  the test binaries into a fresh tree in 29 s from its own entries, with no
  error. The daemon still reported 0.27.0 afterwards. The index then read
  `user_version` 7, `file_hashes` WITHOUT ROWID, and `source_env_runtime_uses`
  back, with 31 rows.

So the mix is safe to leave running while old lanes finish. rscene and
sketch_solve were told, and they bump their own pins.

### 319.4 Not taken

`[cache] shared_hardlink_restores` (#1072) lets every tree link the same blob.
kache's own doc comment says to turn it on only when "target directories are
single-use". The reason is that a restore stamps the mtime on the shared inode,
which can make another tree's dependents look stale and rebuild. Lane trees are
used once, but this tree and the dispatchers' trees are not. It also leaves
test and example executables private (`wrapper.rs:4865` above), and those are
5.03 GB of k27d's 5.77. So at most the other 0.74 GB could ever be shared.
