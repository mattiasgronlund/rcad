## 484. What five fuzz targets at four forks cost by themselves — 2026-11-07

*Q12 asked how much of a night's slowdown the load alone costs: five targets
at `-fork=4`, 20 processes on 24 threads (§227.5). §233.3 had left that load
as the only cause standing for two ten-second timeouts, by elimination, and
§233.4 said a live night would close the question if one of the two inputs
crossed ten seconds and reopen it if neither did. Neither did. This is what
was measured, and why the question is left open rather than chased.*

### 484.1 What was measured

From about 23:15 UTC on 2026-11-06, with the night fuzz running five targets
at four forks (load average 20 to 24 on 24 threads, 14 to 52 tasks runnable),
Q12's lane replayed §233's two inputs fifteen times each on the night's own
fuzz binaries, `rcad-nightly-fuzz/fuzz/target/x86_64-unknown-linux-gnu/release`,
built at 23:13 UTC from the night's tree at `eea76408`:

| input | alone at `6664fef8` (§233.2) | under the night's load, today's code |
|---|---|---|
| fuse `bb8aa340…` | 1 664 ms | 57 to 130 ms |
| cone-or-torus `694123dc…` | 3 271 ms | 3 to 9 ms |

Each time is libFuzzer's `Executed ... in N ms` line. Wall time over CPU time
was at most about 1.4 for the fuse input and about 1 for the cone-or-torus
one. Two writing lanes and a Gappa run were on the machine too, so the load
was not the fuzz's alone. The lane read the two files from
`rcad-fuzz-logs/2026-11-06/artifacts/`; they are hard links carried from
night to night (24 of them, last written 2026-10-12), so the night of
2026-11-06 did not time out on them again.

### 484.2 What it says

On these two inputs the night's load cost at most about 1.4 times. At that
rate §233.2's 1.7 s and 3.3 s come to about 2.3 s and 4.6 s, well short of
the night's `-timeout=10`. So load alone does not account for the two
timeouts, unless a slow input suffers far more from load than these fast
runs did, through memory bandwidth for instance, which nobody measured.
§233.3's reading is not confirmed, and why the two inputs timed out is not
known.

### 484.3 Why it is left open

Finding out needs the binary they were slow on: a fuzz build of `6664fef8`,
the night of 2026-09-25, timed under a live night. Q12's lane proposed that
row. It is not filed. Both inputs now run in milliseconds on today's code, so
they cannot time out again, and nothing open needs to know why they did
(§13). A new timeout on a current night would bring the question back, and
its own replay on the night's binary is cheaper than rebuilding a binary six
weeks old.
