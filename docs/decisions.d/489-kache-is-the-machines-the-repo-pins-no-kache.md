## 489. kache is the machine's: the repo pins no kache and sets no wrapper — 2026-11-26

*At Mattias's request: "For work in other repos, we need to uplift kache to the
latest 2.x version on this host. As kache must be in sync with the host version
of kache daemon, I think we should move kache config out of the repo, and
instead see that as a host config."*

**Settled: the machine says which kache runs, and whether it runs.** rcad's
`mise.toml` pins no kache and sets no `RUSTC_WRAPPER`, and `mise.lock` has no
kache in it. The machine's mise config, `~/.config/mise/config.toml`, pins kache
and sets the wrapper, still empty when `CI` is set. The daemon's unit runs the
same install that pin picks. This machine moved from 0.27.0 to 2.3.1 on the
same day. `check-kache-is-the-hosts` replaces `check-kache-off-on-ci` (§489.3).

This overrides §319.1's "the daemon now runs the newest kache installed on this
machine, whichever repo pinned it", and the reason a repo pinned its own kache
at all, which `mise.toml` gave as "the file which sets the variable is the file
which installs the tool". That rule still holds; the file is the machine's now.

### 489.1 Why a repo cannot own it

- **One daemon serves every repo on the machine.** It is the user service
  `~/.config/systemd/user/kache.service`. On 2026-11-26, five repos pinned
  `cargo:kache` 0.27 for it: rcad, rcad-brep, rscene, sketch_solve and
  tangency-labs.
- **The daemon ran whichever kache was newest.** Its unit ran
  `.../cargo-kache/latest/bin/kache`, and mise points `latest` at the newest
  version installed, whichever repo installed it (§319.1). So one repo trying
  2.x would have moved every repo's daemon, and the other four would have kept
  their 0.27 wrappers against it.
- **2.0 broke what made a mixed pair work.** §319.3 found that 0.16 wrappers
  worked against a 0.27 daemon by reading both sources, and said kache promised
  nothing about it. kache 2.0.0's notes, under *Breaking Changes*: "Restart the
  daemon after upgrading for the new execution protocol", and "Install matching
  `kache` and `kache-engine` binaries together." So the wrapper is no longer a
  thing a repo can choose apart from the daemon. It has to be the same release,
  and the machine is the only place that knows both.

### 489.2 What the machine holds now

None of this is tracked here. This section is its record, as §474.9 is for
zram-kache.

- **`~/.config/mise/config.toml`** pins `"cargo:kache" = "2"` and sets
  `RUSTC_WRAPPER` to the template rcad's `mise.toml` had, character for
  character, so it is still empty when `CI` is set. Its comments say why, and
  how to build without the cache for one run (§489.4).
- **`kache.service` runs `.../cargo-kache/2/bin/kache daemon run`**, not
  `latest`. mise keeps the `2` link on the newest 2.x installed, and a `"2"` pin
  resolves to that same version. After the install the links read `2 ->
  ./2.3.1`, `2.3 -> ./2.3.1` and `latest -> ./2.3.1`. `mise which kache` gave
  `.../cargo-kache/2/bin/kache` from `~` and from this tree, and `mise ls
  --current cargo:kache` gave `2.3.1 ~/.config/mise/config.toml 2`. So the
  wrapper and the daemon are one file, whatever a repo installs to try.
- **kache checks the pair itself.** `kache doctor`'s *Daemon service* line
  compares the file the unit runs with the running kache by inode (2.3.1's
  `service.rs`, `recorded_exe_problem`), and it passed.
- **`~/.config/kache/config.toml` is unchanged**, and 2.3.1 reads it. `kache
  config list` gives `cache.local_max_size = "62GiB"` and `cache.local_store =
  "/var/cache/zram/kache"`, and doctor says the daemon's settings match.
- **The install** was `env -u RUSTC_WRAPPER mise install cargo:kache@2.3.1`, run
  from a directory no repo config reaches, before the machine's config named
  kache. It took 368 s and put `kache`, `kache-engine`, `kache-cl` and
  `kache-clang-cl` in one directory. On a new machine, install it with `CI=1`
  set, because the `[env]` would ask cargo for kache in order to build kache.
  That is ci.yml's fault 2 again, for the whole machine.
- **`kache init` was not run.** The upgrade guide asks only "Keep your existing
  Cargo wrapper and Kache configuration", and a restart. `kache init --check`
  proposed four changes: configure `~/.cargo/config.toml`; put Cargo and
  compiler links into `~/.bashrc` and `~/.profile`; set `build.rustdoc` to a
  shim; and set `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER` for "test
  pacing". Each of them changes every cargo on the machine, mise or not, and
  the last one changes how tests run.

### 489.3 What the repo holds now, and what checks it

- `mise.toml` and `mise.lock` name no kache, and `mise.toml` sets no `[env]`.
  Its comment says why.
- **`check-kache-is-the-hosts` replaces `check-kache-off-on-ci`.** It fails when
  any mise config in the tree pins kache, under any backend. It asks `mise ls
  --current --json` where each kache in use was pinned, rather than reading
  `mise.toml`, because mise reads `.mise.toml`, `mise.local.toml` and
  `mise/config.toml` as well. With the 0.27 line put back into `mise.toml` it
  printed `.../mise.toml pins cargo:kache, and kache is the machine's` and
  exited 1.
- **It keeps the old check's CI half:** with `CI` set, `mise exec` gives an
  empty `RUSTC_WRAPPER`. Two things need that. A runner, where mise applies
  `[env]` while it installs `[tools]` (ci.yml's fault 2). And `fuzz-overnight`
  and `fuzz-target`, which set `CI=1` to keep kache out of the stand-in build
  (§292).
- **It drops the other half**, that off CI the wrapper is `kache`. Whether a
  machine caches is the machine's choice. A runner never did, and a machine
  without kache builds slower and still right.
- **`gate` never serves it from an earlier green.** §369.1 let the old check be
  served because "the tree's `mise.toml` sets that key in every case, and mise
  lets the nearest config win, so a config further out cannot change it." That
  is no longer true: the config further out is now the only one that sets it.
  So it left `gate_reusable_steps` and joined the steps
  `check-gate-verdicts-selftest` says must never be there.

**Found while writing it: with no config setting the key, `CI=1` does not clear
a wrapper the caller already has.** This session's shell had
`RUSTC_WRAPPER=kache` in its environment, put there by `mise activate` when it
started in rcad's tree under the old `mise.toml`. With the repo's `[env]` gone
and the machine's not yet written, `CI=true mise exec` passed that value
through, and the check failed with `with CI set, RUSTC_WRAPPER is 'kache'`.
With `env -u RUSTC_WRAPPER` it passed. Once the machine's config set the key
again, the check passed in that same shell, and `CI=true mise exec` printed
`[]` there. So the fuzz recipes' way
of keeping kache out now depends on the machine's config having the `CI`
template. The CI half is what says so when it does not.

### 489.4 What the upgrade did

- **The daemon moved at 10:42:37 CET.** No cargo or rustc was running, and no
  rcad lane was held. The 0.27 daemon drained and stopped cleanly, and 2.3.1
  listened on `/var/cache/zram/kache/daemon.sock`. `kache daemon status` gives
  `CLI v2.3.1` and `Daemon running v2.3.1`.
- **`kache doctor` passes every check but one, *Link layout*:** "Cargo output
  directory could not be resolved". kache's resolver gives up on purpose on a
  `build-dir` that holds `{workspace-path-hash}`: its own test in
  `target_seed/placement.rs` lists that config among the ones it cannot
  resolve. `~/.cargo/config.toml` sets exactly that (§474.9). The same resolver
  feeds 2.0's new worktree seeding, which also needs rustc 1.98, and rcad pins
  1.96. So nothing is lost today. Whether a restore still reflinks under 2.3.1
  was not measured; §474.9 measured it for 0.27.
- **`kache doctor --probe`** says "Rust compile, store and restore produced a
  local hit with identical bytes".
- **Nothing in the store is a hit for 2.3.1.** The store held 3,255 entries and
  43.6 GiB before any 2.x build. The keys changed twice since 0.27: 2.1.1's
  notes move them from 33 to 34, and 2.2.0's from 34 to 35. The probe below
  shows it on one crate. The old
  entries stay until eviction takes them by size and age, as in §319.1.
- **A 0.27 wrapper still works against the 2.3.1 daemon.** A dependency-free
  scratch crate was built four times with rustup's cargo, each time into its
  own target and build directory. Two builds through 0.27 were a miss and then
  a local hit, under key `c31e5efe…`. Two through 2.3.1 were a `dup` and then a
  local hit, under `714f391f…`. Every build exited 0, the daemon was still
  2.3.1 afterwards, and doctor's store checks passed afterwards. So a repo
  still on 0.27 keeps building, but its entries are under keys no 2.x wrapper
  reads. The files are shared (2.3.1 stored 3 duplicate blobs and 0 new) and
  the entries are not. Nothing bigger than one crate was tried.
- **`RUSTC_WRAPPER= just gate` never turned the cache off,** although
  `mise.toml` said it did from 2026-09-07 to this commit. mise sets the value
  back: `RUSTC_WRAPPER= mise exec -- sh -c 'echo "[$RUSTC_WRAPPER]"'` prints
  `[kache]`. tangency-labs' `mise.toml` said so first. `KACHE_DISABLED=1` gets
  through to kache, which reads it (`kache-config` 2.3.1, `config.rs`), and so
  does `CI=1`, through the template. The machine's config now says this.

### 489.5 What is left

- **Four other repos still pin `cargo:kache` 0.27** in their own `mise.toml`:
  rcad-brep, rscene, sketch_solve and tangency-labs. Each should drop the pin.
  Its wrapper is then the machine's, and its entries are the ones every other
  repo reads. rover pins `github:kunobi-ninja/kache` 0.9.0 behind a shim of its
  own and sets `KACHE_LOCAL_ONLY`. It was not looked at further.
  `rcad-nightly-fuzz` is an rcad tree on an old base, and gets this when it
  moves.
- **`Cargo.toml` and the three `tests/merged.rs` files say kache 0.27 restores
  test executables as private copies** (§373.2). That was not measured under
  2.3.1, and the comments name the release they measured.
