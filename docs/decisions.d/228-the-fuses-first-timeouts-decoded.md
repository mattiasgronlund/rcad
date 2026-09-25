## 228. The fuse's first timeouts, decoded: two are one slow refusal, two do not reproduce — 2026-09-25

*Row T3 of `docs/test-plan.md`, from §226.3. Measured on `0c1346f9`, `main` at
the time, with `cargo run -q -p rcad-fuzz-explain -- <target> <file>`, which is
a debug build.*

  228.1   What the four inputs are
  228.2   How long each takes, and under what load
  228.3   The two slow ones are a slow refusal, not a hang
  228.4   The two fast ones: why the night stopped them is open
  228.5   What is left

### 228.1 What the four inputs are

The three fuse artifacts in `rcad-nightly-fuzz/fuzz/artifacts/two_solids_fused_by_a_stranger`
are all 4 763 bytes long and are three different inputs (`cmp` differs on each
pair). All three decode to the same shape: the first half does not decode, and
the second is one model of 2 solids, 7 faces, 13 edges and 10 vertices, split
into the two arguments of a **Difference** under the **tolerant** policy.

The cone-or-torus artifact is 63 bytes: a cone with its apex at the origin, its
axis along (0, √½, √½) and slope 1, against a plane with normal (0, 0, −1) and
offset −5.6·10⁻³⁰³. So the plane goes through the apex, but for a number far
below any rounding, and its normal is at 45° to the cone's axis. If slope 1 is a
45° half-angle, the plane holds one whole line of the cone; that reading of
`slope` was not checked against `Cone`.

| artifact | written | outcome on `main` | wall time |
|---|---|---|---|
| fuse `timeout-c0c1f6b0…` | 09-25 00:21 | refused: `CannotMeshAFace { why: BoundaryCrossesItself { at: 18250 } }`, face slot 8 | 508 s |
| fuse `timeout-0d5a11d5…` | 09-25 04:23 | the same refusal on the same face slot, `at: 17874` | 494 s |
| fuse `timeout-bb8aa340…` | 09-25 07:39 | succeeded: the Difference made one solid | 9 s |
| cone-or-torus `timeout-694123dc…` | 09-25 07:42 | succeeded: one curve; the oracles asked 1 000 132 points | 7 s |

### 228.2 How long each takes, and under what load

Each time is start to finish of the one command, to the second, and includes
cargo's start-up. The first run also rebuilt the crate for the new `HEAD`; the
other three reused that build. The machine has 24 cores. The load average was
0.29 when the first started, and 12.1 six minutes later as other sessions'
builds began. It was 3.56 when the other three started together. `ps` showed
each slow run at 99–100% of one core, so the eight minutes are the run's own
work, not waiting for a core.

### 228.3 The two slow ones are a slow refusal, not a hang

Both finish, in a little over eight minutes each. That is about fifty times the
night's ten-second limit. The outcome is a correct refusal, not a wrong answer.
The two inputs differ in their bytes but not in what they do: same shape, same
face, same kind of refusal.

This was not profiled, so where the time goes is not named. The lead, with what
it rests on: `its_sides_cross` in `crates/rcad-topo/src/validate.rs` counts
`BoundaryCrossesItself` as a refusal that a finer mesh may cure. The solid is
then meshed again with the deflection halved each time, down to a floor of half
the distance between its two closest corners (`how_close_two_corners_come`).
Three things are unmeasured: whether this fuse reaches that loop, how many
halvings it ran, and what the floor was for this solid. What the refusal's `at`
field counts was not looked up either.

### 228.4 The two fast ones: why the night stopped them is open

The third fuse input succeeds in 9 s and the cone-or-torus input in 7 s, both in
a debug build. cargo-fuzz builds with optimisation by default, keeping debug
assertions and adding the address sanitizer. So the night's binary should be
faster than this one, not slower. Yet the night stopped both at ten seconds.

This section does not settle why. Two causes fit and neither was measured. One
is the night's load: five targets, each run with `-fork=4`. The other is a
commit between the tip the night fuzzed and `0c1346f9` that made these inputs
faster. To settle it, run the fuzz binary on each file under the night's flags
— `cargo +nightly fuzz run <target> <file>` from `fuzz/`. This lane did not.

### 228.5 What is left

- Time the four inputs in the fuzz build. That settles 228.4.
- Profile the slow refusal, or count its halvings. That settles 228.3's lead.
  Eight minutes for a Difference of a 7-face solid is kernel work, not a test
  plan row. Whether it earns a lane is for whoever cuts the next batch.
- The inputs are named here by the hashes libFuzzer gave them, and those hashes
  are of their bytes. The copies used here were in this lane's scratchpad,
  which a reboot wipes. The live copies are in the nightly tree's artifacts
  directory.
