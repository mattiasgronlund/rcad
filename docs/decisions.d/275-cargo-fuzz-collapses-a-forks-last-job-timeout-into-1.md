## 275. `cargo-fuzz` collapses a `-fork` run's last-job timeout (70) into a flat 1, so `fuzz-overnight` never sees the 70 that `FuzzerFork.cpp` gives

*Q159. The row: "A -fork run exits with its last job's code, so arm C of §264
exited 70 with -ignore_timeouts=1 set (FuzzerFork.cpp:488-491). Find out
whether cargo-fuzz passes that 70 on, and what fuzz-target under set -euo
pipefail and fuzz-overnight then do with it: whether a night that timed out
last is reported as a failure or stops the targets after it." §264.5 §264.8.*

  275.1   `cargo fuzz run` always exits 1 on any target failure, never the child's own code
  275.2   `fuzz-target`'s own exit code is exactly cargo-fuzz's, unchanged by `set -euo pipefail`
  275.3   What reaches `fuzz-overnight`'s `wait`, and what it does with it
  275.4   Verdict: §264.5's "not checked" is now checked, and it changes the picture

### 275.1 `cargo fuzz run` always exits 1 on any target failure, never the child's own code

`cargo-fuzz`'s `exec_fuzz` (installed version 0.13.2,
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cargo-fuzz-0.13.2/src/project.rs:538-598`)
spawns the built fuzz binary as a child, `wait()`s on it, and when
`!status.success()` calls `bail!("Fuzz target exited with {}", status)` at
line 598 — an `Err`, not a passthrough of `status.code()`. `main()`
(`src/main.rs:169-171`) is a plain `fn main() -> Result<()>`, so any `Err`
becomes Rust's default `Termination` for `Result`: the message is printed and
the process exits **1**, whatever `status` held.

Run, not read: built `where_two_surfaces_cross`
(`mise exec -- cargo +nightly fuzz build where_two_surfaces_cross`), then, with
a scratch corpus root so nothing tracked was touched:

```
cargo +nightly fuzz run where_two_surfaces_cross <scratch-corpus>/where_two_surfaces_cross corpus/where_two_surfaces_cross -- \
    -max_total_time=30 -fork=1 -timeout=1 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1
```

`-fork=1` makes its one job trivially the "last" job. Fuzzing found several
inputs over the 1-second `-timeout`; the log shows libFuzzer's own line
`INFO: exiting: 70 time: 35s` — the exact `FuzzerFork.cpp:488-491` exit —
followed by cargo-fuzz's `Error: Fuzz target exited with exit status: 70`.
The whole `cargo fuzz run` *process* nonetheless exited **1**, confirmed by
echoing `$?` right after the redirected run: `EXIT_STATUS=1`.

### 275.2 `fuzz-target`'s own exit code is exactly cargo-fuzz's, unchanged by `set -euo pipefail`

`fuzz-target` (`justfile:4144`) ends with that same
`cargo +nightly fuzz run ...` invocation as its last statement — nothing
after it, no pipe — so under `set -euo pipefail` the recipe's own exit status
is simply whatever that last command returned; `-e` has nothing left to act
on. Confirmed generically with a throwaway recipe of the same shape
(`target/exit-code-check.justfile`, script body `set -euo pipefail; exit
{{code}}`): `mise exec -- just -f target/exit-code-check.justfile
last-stmt-fails 70` returns `$? = 70` through `just` unchanged. Combined with
§275.1: whenever the built target fails for any reason libFuzzer can raise —
crash, oom, or a `-fork` run whose last job timed out — `fuzz-target`'s own
exit code is always cargo-fuzz's flat **1**, never the specific 70 (or any
other libFuzzer code) the child gave.

### 275.3 What reaches `fuzz-overnight`'s `wait`, and what it does with it

`fuzz-overnight`'s per-target `case "$status" in 124|137) ...; *) ...esac`
(the loop after `recipes=({{fuzz_all}})`) is commented as if a real 70 can
land in that `*` arm alongside other values; §275.2 says it never can —
`wait "${pids[$i]}"` observes `just <recipe>`'s own exit code, which is
cargo-fuzz's flat 1 (or 0) for every one of those cases, indistinguishably.
That does not matter to the arm's own logic, which does not switch on the
number at all: it greps the target's log for libFuzzer's own trailer line,
`grep -q '^INFO: exiting:'`. That line is written by the *child* process
before cargo-fuzz decides its own exit code, so it is present exactly when
the fork run finished on its own (cleanly, or via an ignored last-job
timeout) and absent when the target's launch failed outright — which is what
the arm is actually distinguishing.

Verified with a stand-in rather than a real night, so as not to touch
`rcad-nightly-fuzz` or spend a night's window (`target/fuzz-overnight-standin.sh`
and `target/fake-fuzz-recipe.sh`, both scratch scripts under `target/`,
not part of this row's write set and not committed). It copies
`fuzz-overnight`'s own start-loop / wait-loop / case-statement verbatim, with
three fake "recipes" in place of the real five: `timeout-70` (exits 70 having
printed `INFO: exiting: 70`, modelling the last-job-timed-out case), `clean`
(exits 0), and `silent-crash` (exits 1, printing nothing of the sort,
modelling a launch failure). Running `./target/fuzz-overnight-standin.sh`
gives:

```
--- all 3 stand-in targets started before any wait: timeout-70 clean silent-crash
--- timeout-70 finished with raw exit status 70
--- clean finished with raw exit status 0
--- silent-crash finished with raw exit status 1
silent-crash: never got fuzzing (exited 1) — see ./target/standin-night/silent-crash.log
=== to_look_at: silent-crash:no-run
```

Only `silent-crash` — the one with no `INFO: exiting:` line — lands in
`to_look_at`. `timeout-70` does not, and would not either if its status were
1 instead of 70: both values fall into the same `*` arm, and the grep never
inspects the number. And because all three are started in the same `for`
loop, before any `wait` runs, one target's exit code cannot stop the targets
after it — `fuzz-overnight` already launches all five before it waits on the
first.

### 275.4 Verdict: §264.5's "not checked" is now checked, and it changes the picture

§264.5's and §264.8's open question — "whether cargo-fuzz passes the 70 on" —
is answered: **no**. `cargo fuzz run` never returns the libFuzzer child's own
code; `exec_fuzz`'s `bail!` on any non-zero child status turns every failure
into an `anyhow::Error`, and `fn main() -> Result<()>` turns every `Err` into
a flat process exit of 1. So the `FuzzerFork.cpp` "exit with the last job's
own code" fact that made arm C's *child* process exit 70 is real and
reproducible (§275.1), but invisible one layer up: `fuzz-target`, and hence
`fuzz-overnight`, see cargo-fuzz's own 1, never 70, for that case, for a
crash, or for an oom alike.

This does not misreport a night that timed out last. `fuzz-overnight`'s `*`
arm never actually keyed on the number 70 — only on `grep -q '^INFO:
exiting:'` — and that line survives cargo-fuzz's translation unchanged
(§275.3), because it is the child's own stdout, written before cargo-fuzz
decides its own exit code. A night whose last `-fork` job timed out is
**not** reported as a failure, and the targets after it are unaffected —
they already started before any `wait` ran.

What is inaccurate is the *reasoning* written down, not the *behaviour*:
`fuzz-overnight`'s own comment above the case statement ("70 is a timeout in
the last job of forty thousand ... **Under `-fork` the exit status says
nothing about the night**") and §264.5's framing both describe a value — the
literal 70 — reaching a layer of the pipeline where it can never actually
arrive. The classification the code performs is still the right one; the
comment justifying it names a number `$status` cannot hold at that point.

Left for whoever next touches `fuzz-overnight` (Q148, while it runs, per this
row's brief — nothing here was fixed): reword that comment to say `$status`
there is cargo-fuzz's own 0/1, not libFuzzer's fork exit code, so a future
reader does not go looking for a 70 that cannot land in that variable.
