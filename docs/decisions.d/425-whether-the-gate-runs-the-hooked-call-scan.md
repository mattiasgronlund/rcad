## 425. Whether the gate runs the hooked-call scan — 2026-10-09

*Queue row Q324, SECOND run, re-filed from Q321. Branch
`lane-q324-the-hooked-call-scan-wired-into-the-gate`, off `afc8aed9`,
carrying forward the first run's one commit (`cd42d6db`, reserving
`check-timing-hooked-calls` in `gate_steps` and `gate_reusable_steps`) on
branch `lane-q324-whether-the-gate-runs-the-hooked-call-scan`, then this
run's own two commits — `b698c062`, which wrote the step and wired it in,
and `bb366863`, which takes it back out.*

### 425.1 The question

Should `just gate` run `fuzz/timing-hooked-calls.py`, the scan §419 built,
so a hooked libFuzzer intercept reachable from `rcad-fuzz-timing` fails a
lane's own gate rather than waiting for the fuzz night to find it?

**Does not fit.** The step was built, wired into `gate_steps`,
`gate_metered_steps`, `gate_reusable_steps` and the `gate` recipe's
`cargo_steps` chain, shown to catch a real hooked call, then taken back
out, because every cold measurement of it came in at roughly 2.5x the
brief's 60-second budget — a cost that would have landed on every lane's
gate several times a day.

### 425.2 The budget and what it measured against

The brief's rule: the step stays in the gate only if it adds at most 60s to
the wall time of a cold `just gate quiet fresh` and of a cold `gate-lane`
in a fresh tree — the two gates a lane actually runs, several times a day,
in a tree whose `fuzz/target` is always cold. Two measurements, both on the
wired-in commit `b698c062`:

- Standalone, zero gate contention, `fuzz/target` and `target` absent:
  `real 2m16.647s` for `mise exec -- just check-timing-hooked-calls` (on
  `90e5dccf`, an intermediate commit of this run with the same recipe
  body).
- The step's own printed line inside a genuinely cold `gate-lane fresh`
  (both `fuzz/target` and `target` wiped first): `check-timing-hooked-calls
  (149.1 s)`, read from `target/last-gate.log` after `mise exec -- just
  gate-lane fresh` on `b698c062`. The gate's own longest step in that run
  was `test` at 325.3s; the run ended `gate: PASS`.

Both numbers are close to each other (2m16.6s standalone, 2m29.1s inside
the gate run, accounting for the step's own two guard checks around the
timed build) and both are about 2.5x the 60s budget. Neither is close
enough to the line to be worth arguing over measurement noise.

No fresh, directly-measured "`gate-lane`, without the step, on this exact
base" number exists: the step sits on the gate's serial `cargo_steps`
chain, sharing `fuzz/target`'s cargo lock with `check-fuzz-builds`, so
removing it should cut a cold `gate-lane`'s wall time by close to its own
149.1s line — but that is read off the chain's structure, not a number
anyone watched happen, and is not offered as one. The nearest actual
"without" reference is the first run's own `just gate quiet fresh` (which
also runs `test-slow`, unlike `gate-lane`) at `7m8.7s` wall on `afc8aed9`,
an ancestor of this base with no gate-affecting commits between the two
other than `docs/queue.md` and `decisions.d` entries.

### 425.3 The step works; this is a cost decision, not a correctness one

Before measuring cost, the brief asked for a demonstration that the step
actually catches something. A scratch `#[inline(never)]` function was
added to `crates/rcad-fuzz-timing/src/lib.rs` (never committed; `git
status --short` confirmed clean afterward) doing a slice `==` on the
input bytes, called from the crate's own non-generic, non-inlined
`stop_the_clock`. Rebuilding and running the recipe found it:

```
calls from rcad_fuzz_timing to a hooked function: 2 (1 outside SlowRun::is)
  211ee3  call -> ['__interceptor_trampoline_bcmp', 'bcmp']  via GOT slot 3f20e8  reaches a hook outside SlowRun::is
error: Recipe `check-timing-hooked-calls` failed with exit code 1
```

reverted, with `git diff` on that file coming back empty afterward. The
scan is sound and the step's exit code is the real signal §419 built it to
be. What does not fit is running a 2+ minute cold build of five fuzz
targets, on `+nightly`, inside every lane's gate — not whether the check
is worth having at all.

### 425.4 What the status quo already gives up, and what it does not

`fuzz-overnight` already runs this exact build-and-scan on main's own tip
every night (`justfile`'s `fuzz-overnight` recipe calls the same `cargo
+nightly fuzz build` and then the script). Leaving the step out of the
gate costs at most a night's delay before a hooked call is found, never
silent non-detection — the gap is "a day late", not "unchecked". That is
why the 60-second-per-lane-gate budget, weighed against same-day
detection, comes down against keeping it: a lane's own gate runs many
times a day across many lanes, where the night runs once.

### 425.5 What would make it fit

A lead named in the brief, checked and confirmed true: `crates/rcad-fuzz-timing/Cargo.toml`
declares no dependencies at all, and the scan only walks code reachable
from that crate's own functions, so its answer should depend only on
`rcad-fuzz-timing` + `fuzz/`'s build files + the script + the installed
nightly/`cargo-fuzz` version — never on the kernel crates (`rcad-geom`,
`rcad-mesh`, `rcad-topo`, `rcad-step`, …) a lane actually changes.

Checked by adding a second scratch change (also never committed, also
confirmed clean after reverting): an inert `#[allow(dead_code)] pub fn` to
`crates/rcad-geom/src/lib.rs`. Rebuilding and re-running the scan, then
diffing its output against the pre-change baseline, found differences
*only* in the cargo build log lines — which crates recompiled, and for how
long (7.91s versus 1m34s, because `rcad-geom`'s dependents `rcad-mesh`,
`rcad-topo` and `rcad-step` recompiled too) — never in the scan's own
"calls from rcad_fuzz_timing ..." count lines. The lead holds: a kernel
crate's own content cannot move this step's answer.

So a cache keyed on just those few inputs — `rcad-fuzz-timing`'s own
source and `Cargo.toml`, `fuzz/`'s build files and `timing-hooked-calls.py`
itself, and the installed nightly and `cargo-fuzz` versions — rather than
the whole-tree match `gate_reusable_steps` uses today (`docs/decisions.md`
§337.3's "tree without `docs/` and `*.md`"), could serve this step on almost
every lane's gate without ever rebuilding it: a lane that touches only
kernel crates, which is most of them, would hit the cache every time. That
caching mechanism does not exist yet; building it is real, separate work,
not a one-line fix, and is left to a follow-up row rather than built here
on the strength of one lead.

### 425.6 What was reverted

`b698c062`'s wiring, reverted in `bb366863`: the recipe body in `justfile`
(between `check-timing-hooked-calls-selftest` and the `fuzz-watermark`
comment), the `check-timing-hooked-calls` entry in `gate_steps`,
`gate_metered_steps`, `gate_reusable_steps`, and the `cargo_steps` string
inside the `gate` recipe, `check-gate-runs-every-check`'s hardcoded count
(62 back to 61), and the skip-reasons paragraph added to
`.github/workflows/ci.yml`. `check-timing-hooked-calls-selftest` (Q323's,
pinning the script itself) is untouched and stays in the gate; only the
step that builds and runs the five fuzz targets on every lane came out.
`mise exec -- just check-gate-runs-every-check` and `mise exec -- just
pre-gate gate-lane` both pass on the commit that removes it.
