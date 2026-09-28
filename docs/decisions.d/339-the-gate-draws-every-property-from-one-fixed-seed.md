## 339. The gate draws every property from one fixed seed — 2026-09-28

*Mattias, reading the handoff for a gate verdict cache: "we should not 'draw a
fresh random seed' on every run, as that is not deterministic." This reverses
§17.7's "the properties beside them run from a fresh seed every time" for every
run that does not name a seed itself.*

### 339.1 What was random, and the one line that fixes it

A test that builds its proptest runner from `Config::default()` got a new seed
from the operating system on every run: `TestRunner::new` hands
`Config::default().rng_seed`, which is `RngSeed::Random`, to
`TestRng::default_rng`, and that calls `ChaChaRng::from_os_rng`
(proptest 1.11, `src/test_runner/rng.rs`, `default_rng`). That covered:

- the 34 `over()` call sites in `fuses_that_must_agree.rs`, and that file's
  `every_frustum_of_the_family_has_skin_inside_the_block_and_skin_outside_it`;
- `over()` in `fuses_mean_what_their_rule_says.rs`;
- every `proptest!` block, in 17 files across `rcad`, `rcad-geom`, `rcad-mesh`,
  `rcad-predicates` and `rcad-topo`.

The runners that already take `RCAD_SOAK_SEED` (`measured()`, `soak::seeded()`,
`the_seed()`, §290.1) were already fixed, and are unchanged.

The fix is `PROPTEST_RNG_SEED = "0"` under `[env]` in `.cargo/config.toml`.
`Config::default()` reads that variable, so each of those runners now starts
from `ChaChaRng::seed_from_u64(0)`. That is one line instead of nineteen files,
and a `proptest!` block written tomorrow gets the same seed without anyone
remembering to give it one. Cargo passes `[env]` to `cargo test` and to
`cargo nextest` alike: the test in §339.3 ran through `just t`, which is
nextest. The line is not `force`d, so a value set in the shell wins over it. A
soak still sets `PROPTEST_RNG_SEED=<seed>` for each chunk, the way §290.3's
chunks did.

The seed is **0** because soak chunks count up from 1 (§290.3, and
`fuses_that_must_agree.rs`'s "`PROPTEST_RNG_SEED` 1 through 20"), so no soak
chunk repeats the gate's draw.

### 339.2 Why

- **A property that goes red or green by the draw cannot say which commit
  broke it.** With a fixed seed, a red gate is caused by the code, and
  `just t <name>` reproduces it. `turning_a_fuse.rs` switched to a fixed seed
  for this reason, and §41.13 is the run where it went red on a defect four
  lanes had already shipped past.
- **The verdict cache needs it.** The cache serves the gate's verdict for code
  that has already gated green. That is only honest if running the gate again
  would give the same verdict, and with a fresh seed it would not: each run
  drew new cases.

§17.7's reason for fresh seeds still holds: "their job is to find inputs nobody
has thought of and a frozen seed would stop them finding any". Finding those
inputs is the soak's job (§290: `RCAD_SOAK_SEED`, `RCAD_SOAK_FACTOR`, and
`PROPTEST_RNG_SEED` for each chunk). **What this gives up** is the searching
the gate did by accident. `.git/rcad-greens` holds 249 green gates from
2026-09-26 18:14 to 09-28 16:27 UTC, and each of them drew a new set of cases
for those properties. Nothing runs a soak on a schedule, since the nightly job runs
`cargo fuzz` and no proptests. So after this, those properties search only when
someone runs a soak.

### 339.3 The demonstration

`two_runs_of_a_property_draw_the_same_values`, in `fuses_that_must_agree.rs`,
builds two runners the way `over` does and compares what they draw: 27 values
each, of which 19 are that file's saved failures, replayed first, and 8 are new
draws. With the line, the two runs are equal. With the line taken out of
`.cargo/config.toml`, the first 19 agree and the last 8 differ:

```text
left:  [..., 11066388549689811005, 5040018514056854778, 2583728676035403651, ...]
right: [..., 11066388549689811005, 12186318465754232957, 8731997634182331956, ...]
```

Every property passes on seed 0's draw. `just gate quiet` on `5942c253` with
this change: `gate: PASS`, 2668 tests run, 2668 passed, 11 skipped, and 25 doc
tests passed. A fixed seed whose draw failed would have been a finding, not a
reason to pick another seed.

### 339.4 What this does not cover

A grep of `crates/` for `thread_rng`, `rand::rng()`, `from_entropy`,
`from_os_rng`, `OsRng` and `getrandom` finds none of them in a test or in the
kernel. `RandomState` appears only in `rcad-fuzz-timing`, which §267 covers.
**Not audited:** `std` `HashMap` and `HashSet` iteration order, which is seeded
per process. Eleven files under `crates/*/src` use one. Whether any fuse,
mesh, STEP read or STEP write depends on that order is not measured here. If
one does, the gate is not deterministic even with the seed fixed.
