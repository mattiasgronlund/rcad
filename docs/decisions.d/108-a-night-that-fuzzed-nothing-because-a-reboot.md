## 108. A night that fuzzed nothing because a reboot took its PATH, fixed 2026-09-15

The night of 2026-09-15 started at 00:10:21, advanced to `1fe32c5b`, and was
over at 00:10:23. `~/rcad-fuzz-logs/latest` read

```
2026-09-15 00:10  BUILD FAILED  /home/mattias/rcad-fuzz-logs/2026-09-15
```

and `2026-09-15/build.log` was one line: `cargo: command not found`. Eight hours
of fuzzing, none of it spent. §81 put the night on a timer so nobody has to
remember it; the cost of that is that nobody notices when it stops, and this
stopped for a day before anyone asked.

### 108.1 It was the machine, not the tree

The first guess is always the commit the night advanced onto, because §90 makes
the night move and moving is the thing that changed. It was not that. The build
line — `justfile`'s `(cd fuzz && RUSTFLAGS=… cargo +nightly fuzz build)` — is
**byte-identical** in `9b82e9e8`, which the night of the 14th ran for eight
hours, and in `1fe32c5b`, which failed in two seconds. Nothing in the diff
between them touches `mise.toml` or `rust-toolchain.toml`.

What is between the two runs is a reboot. `journalctl --user -u
rcad-fuzz.service` prints it plainly:

```
Sep 14 08:10:02  rcad-fuzz.service: Consumed 7h 59min 45.618s CPU time.
-- Boot ea84b2d91538474a9674565329d6c75a --
Sep 15 00:10:23  the fuzz targets did not build, so nothing was fuzzed
```

and the user manager's PID goes 3546 → 2113 across it.

**The way to find a fault like this is to take away what the environment
supplies and run it again** — §77.6's rule, and the third time it has been the
answer here. Reproducing the manager's environment exactly and running the
night's own build line in it gives `cargo: command not found`, exit 127, the
same line as `build.log`; prepending `~/.cargo/bin` and changing nothing else
gives `Finished \`release\` profile … in 1m 03s`, exit 0.

### 108.2 Inherited is not set

`systemctl --user show-environment` carries no `~/.cargo/bin`. `cargo` here is
rustup's shim in that directory, and the only files on this machine that add it
are `~/.bashrc` and `~/.profile`; there is no `~/.config/environment.d/`.
systemd `--user` reads none of those. mise supplies `just`, `cargo-fuzz` and
`kache` from its install dirs and pins no rust, so `mise exec --` does not
supply `cargo` either — which is checkable and was checked.

So the honest statement of the fault is: **the unit never had a durable source
of `cargo`, and for five nights it did not need one.** Why it did not is an
inference and is marked as one — the pre-reboot manager's environment is not
recoverable, so "the PATH was inherited from a login session" is the only
explanation consistent with what *is* measurable (no durable source exists now,
none existed in any file then, and the boundary is a boot) rather than something
this section watched happen.

The fix is one line in `tooling/systemd/rcad-fuzz.service`:

```
Environment=PATH=%h/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin
```

The unit already named `%h/.local/bin/mise` absolutely, with a comment saying
systemd starts without a profile. The reasoning was right and was carried one
binary short of where it needed to go.

Verified end to end and not only at the build line: a real `fuzz-overnight`,
75 seconds wide, run by systemd `--user` with exactly the `Environment=` the
installed unit now carries, reported `5 targets, nothing found` and exited 0.

**`fuzz-cones-and-tori` had never run a night.** It entered `fuzz_all` in
`1fe32c5b` — the commit this very run advanced onto — so the one night it would
first have run is the night that did not run. That short verification run is the
first time the cone and torus target has been fuzzed by the timer at all.

### 108.3 Two copies of the unit, and nothing made them agree

`tooling/systemd/rcad-fuzz.service` is tracked here; `~/.config/systemd/user/`
holds a **copy**, not a symlink, and systemd runs the copy. Nothing compared
them. That is §60's shape — a second record beside the thing itself — and it had
a price on the day: the fix above lands in a directory systemd will never read,
so writing it changed nothing about tonight until somebody copied it over.

`just check-fuzz-unit` is now in `gate_steps`. **An installed copy must match
the tracked one; a machine with no installed copy is not checked**, because
almost nowhere runs the night and a runner has no `~/.config/systemd/user` at
all, so absence is the ordinary case and not a finding. Half an install — one of
the pair on disk — is a failure, because that is somebody copying by hand and
stopping, which is this same mistake one file earlier.

The gap is named rather than claimed away: **this cannot tell a machine that
never installed the timer from one where the installed file was removed.** Both
look identical from here and only the second is a problem. What it catches is
the case that cost the night — installed, and behind the tree.

A symlink would delete the second record outright and was rejected on its own
evidence: the tree that would hold it is `rcad-nightly-fuzz`, which
`git checkout --detach`es itself every night under §90, so the unit would change
under the running service.

**Every tree on an older base fails this check until the fix lands**, because it
has the old unit tracked and the new one installed. That is the check working
rather than misfiring, and it is nobody's business to fix from inside such a
lane. `check-corpus` has had the same shape since §79 — a tracked manifest
against a thing outside the tree — so the repo already takes this trade, and the
remedy is the same one: land it and rebase. It is a reason to merge promptly, not
a reason to soften the check.

### 108.4 What the morning found anyway, and the two nights that found it twice

`just fuzz-triage /home/mattias/rcad-fuzz-logs/2026-09-14 main` against
`a2a93b9a`: **0 of 6 inputs did not come back clean.** Both crashes replay clean
and were already seeded — `crash-d04e0041…` was fixed by `9c29cae0`, the scaling
oracle tilting the axis it scaled. All four slow units decode, replay clean, and
land on the same outcome: `SurfaceMeet::Curves, 2 of them`, harness at
`Asked { points: 0, orders: 1, scalings: 2 }`.

Only one of the four was in the corpus. The other three were in
`~/rcad-fuzz-logs/artifacts-rescued-2026-09-14/` and in the gitignored
`fuzz/artifacts/` of a worktree, which is exactly the arrangement §91 was
written about — and §91's cost is now measured rather than predicted:
`262c17e4…`, found 2026-09-13 00:12, and `7e62ebe6…`, found 2026-09-14 00:11,
**are the same case.** Same second cylinder — origin `[0, 32, 1]`, axis `+z`,
radius `1.375` — and a first cylinder differing only in its z-origin,
`822083584.0` against `805306368.0`, same radius `167774208.0`. Two nights, one
shape, neither seeded, so the fuzzer paid for the same discovery twice.

All three are now in `fuzz/corpus/where_two_surfaces_cross/`, renamed the way
§93 renamed its crash — a name that says what it is, with the hash kept so it
still points back at the night:

```
slow-unit-a-huge-pipe-crossing-a-small-one-262c17e4…
slow-unit-the-same-pair-with-the-big-pipe-moved-7e62ebe6…
slow-unit-a-huge-pipe-and-a-hair-thin-one-5dddf2a9…
```

**All four slow units are one huge pipe against one small one** — radii
`167774208.0`, `2147482112.0`, `50397183.0` against `1.375`, `1.0` and
`4.3e-19`. That reads as one question about scale and not as four findings, and
what it costs is open: `262c17e4…` alone took 3m47s to replay in a debug build.
Left as a question rather than answered here, because the answer is measurement
this section did not do.

### 108.5 The one sentence the night said to a person was the wrong one

`build.log` had `cargo: command not found` in it the whole time. What the night
printed to the journal, under that, was

> nightly is not pinned by this repo (docs/decisions.md §29.9);
> `rustup toolchain install nightly` is the usual answer

which is good advice about a thing that was not wrong. A log nobody opens and a
sentence that misdirects the person who does is worse than the sentence not
being there: it spends the reader's first guess.

The hint now branches on `command -v cargo`, and says, when there is none, that
the toolchain is not the question, prints the PATH it had, and names
`check-fuzz-unit` as the thing that answers whether systemd is running the unit
in the tree.

`check-fuzz-overnight-selftest` gained an arm for it. **Its PATH is built by
dropping every entry that holds a `cargo` and keeping the rest** — not by
listing the tools the night needs into an empty directory, which was tried and
is not a list anybody finishes: the first attempt died on `id`, which this
justfile runs in a top-level backtick before any recipe body starts. Borrowing
`/usr/bin:/bin` instead would have been worse and quieter, passing here where
cargo lives under `~/.cargo` and testing nothing wherever cargo ships in a
distribution package — §77.6 again, which is the fourth time that shape has come
up in this repo.

Where cargo *does* ship beside the coreutils the arm cannot run, and it says so
by name rather than failing further down in a way that reads like the night's
fault.

The mutation is the proof and it was run: with the branch forced to the nightly
sentence, the arm fails, and the output it fails on is
`cargo: command not found` followed by the advice about rustup — the night of
2026-09-15, reproduced on demand.

