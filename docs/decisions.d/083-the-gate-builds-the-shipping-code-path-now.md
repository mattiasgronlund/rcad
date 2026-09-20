## 83. The gate builds the shipping code path now, recorded 2026-09-10

`just test` runs 1821 tests and every one of them has `debug_assertions` on.
`Cargo.toml` has no `[profile.release]` section at all, and `[profile.test]`
sets only `opt-level = 2`, so nothing in the gate had ever built the profile a
user gets — not `check`, which is `cargo check --workspace --all-targets` under
the dev profile, not `lint`, and not `test`.

This is not a question about speed. §82.7 re-measured the two profiles against
each other on the same sweep workload, five model sizes each, and they landed
within 3% at every size. What differs is not how fast the two builds are but
what is in them: every `debug_assert!` and every `cfg!(debug_assertions)`
branch is present in all 1821 gated tests and gone from what ships.

What went in is one filtered arm, `check-release-build` — one crate, one test
binary, `--cargo-profile release` — placed in `gate` after `test`, so the
ordinary suite still reports first. The arm is small on purpose and the
interesting part of it is not the arm but what stops it from lying about
itself.

### 83.1 `ModelId` was a different type in the two builds

**Everything in this subsection is the tree as it stood when the arm went in.
§84 closed it on this same branch**, so `key.rs` defines one `ModelId` now and
the paragraphs below describe what the arm was built to be able to see.

`crates/rcad-topo/src/key.rs` defined `ModelId` twice. Under
`#[cfg(debug_assertions)]` it was a `u64` handed out by an atomic counter, so
two models never shared one. Under `#[cfg(not(debug_assertions))]` it was
`pub(crate) struct ModelId;` — a zero-sized type with one inhabitant, so every
model had the same one.

Every key carries a `ModelId`, and `slot_in` opens with a plain `assert!`, not
a `debug_assert!`. The assert is compiled into both builds. What changes is
what it is comparing: in release the two sides are values of a type with one
value, so the comparison is always true and the assert can never fire. A key
made by another live model then resolves to whatever now sits in that slot —
the wrong entity, no crash, which is the case §1.3 wrote the `ModelId` for in
the first place.

`Model::fuse`'s `# Panics` section says it panics "if a key came from another
model", without qualifying the build. In a release build it does not. The key
types' own doc comment is the honest one — "in debug builds a key also
remembers which model made it" — and nothing that gates had ever run the other
half.

**That is not fixed here, and this section does not claim it is.** The fix is a
later unit, and it is §84. What this one does is make the place its test can
live exist: a test for release-build `ModelId` written anywhere else in
`rcad-topo` passes under `just gate` for the wrong reason, because the gate
would run it in the build where the counter is real.

### 83.2 One test binary in release, not a second gate

The alternative was a release arm over the whole gate — `test` again with
`--cargo-profile release`. `test` is the largest single thing in `gate`; the
implementing lane's runs put its nextest summary at 25.2 s to 27.5 s of a
roughly 70 s warm gate. Running it twice buys the handful of tests this section
is about at the price of running every other test a second time in a build that
is not what those tests are asking about.

So the recipe names one crate and one binary:

    cargo nextest run --cargo-profile release -p rcad-topo \
        --test what_only_a_release_build_does --no-tests fail

It is the twenty-second `check-*` recipe, and `check-gate-runs-every-check`'s
pinned count moved from 21 to 22 in the same commit.

### 83.3 An empty binary is the mechanism, and `--no-tests fail` is the guard

`crates/rcad-topo/tests/what_only_a_release_build_does.rs` carries
`#![cfg(not(debug_assertions))]` at file level. Under the dev profile the whole
file strips to an empty binary, so `just test` never runs its tests and never
has to know they are there.

That same emptiness is what makes the arm checkable. If the profile flag is
ever lost, or the filter stops matching what it names, nextest selects nothing
— and a run of no tests otherwise reports itself as a pass. `--no-tests fail`
turns it into a failure instead.

Both directions were run here, on `2076a74`, when the file held one test, and
again on `690c81f`, where §84's two joined it:

| Command | Selected | Exit |
|---|---|---|
| `just check-release-build` | 1 test on `2076a74`, 3 on `690c81f`, all passed | 0 |
| the same filter, no `--cargo-profile release` | 0 tests, both times | 4, `no tests to run` |

The file's own selftest asserts `!cfg!(debug_assertions)`, which is redundant
with the file-level `cfg` on the day it was written. That is its job. The `cfg`
is what keeps the file out of the dev suite, and deleting it is the change that
would turn this arm into a second copy of the dev suite wearing a release
name. With the assertion there, deleting the `cfg` is a red `just test` — the
test gets selected by the ordinary run and fails in it.

### 83.4 The failure this is shaped against

§77.6 collects three checks that had passed for a property of the machine they
ran on rather than a property of the tree, and states the technique that found
all three: take away what the environment supplies and run it again.

A release arm belongs on that list before it is ever written. Everything it
claims rides on one flag on one command line, and the failure mode of losing
that flag is not a red run — it is a green one, over nothing, under a recipe
whose name says the shipping code path was built. `--no-tests fail` is §77.6's
technique applied in advance instead of after the incident: the empty selection
is arranged to be the loud case rather than the quiet one.

`check-ignored` pins a count for the same reason and says so in its own comment
— a run that selects nothing is exactly what a passing run looks like.

### 83.5 What it costs, and the number nobody has

Measured by the implementing lane on this machine, and recorded in `6c00bc2`:

- **0.24 s** for the arm alone, warm.
- **17.0 s and 17.1 s** cold with the compile cache off, which is what a runner
  has: `check-kache-off-on-ci` keeps `RUSTC_WRAPPER` empty when `CI` is set.
  A first cold run read 35.1 s and did not repeat.
- Warm `just gate` **70.73 s before and 71.58 s after** on the committed tree.

**The gate-level figure is noise and should not be quoted as a cost.** Two more
runs of that same tree read 77.87 s and 81.85 s, and the suite's own nextest
summary moved 25.2 s to 27.5 s between runs — both larger than the 0.85 s
difference. The cold ~17 s is the number that means anything, because CI builds
without a cache.

**What the added cost is as a share of a CI gate is unmeasured.** Nobody has
timed the whole gate cold without a compile cache, so there is no denominator
to put the 17 s over. That is written down as unmeasured rather than divided
out of the warm 71 s, which would be dividing a cold numerator by a warm
denominator.

For what it is worth against the 0.24 s: this branch's worktree, which had
never built release before, ran `just check-release-build` in 3.6 s wall, of
which 3.2 s was an incremental release build of `rcad-topo` and everything
under it through a warm `kache`. Warm and cold are the two ends the lane
measured; that is a third case in between and it is a one-off.

### 83.6 Where the next such test goes

Write it in `crates/rcad-topo/tests/what_only_a_release_build_does.rs`. The
recipe names that file and that crate, so nothing else changes.

A test that has to live in another crate takes one more `cargo nextest run`
line in the recipe, naming that crate and a file of the same name, behind the
same file-level `cfg`.

**`--workspace` is the thing not to reach for**, and not only for the build
time. Without a `--test` filter it would pull in every other test binary, none
of which is behind a `cfg`, so they would all run in release — which is the
doubled suite §83.2 declined, arriving by accident. It would also satisfy
`--no-tests fail` on the strength of tests that say nothing about the shipping
code path, which is the guard in §83.3 defeated.

