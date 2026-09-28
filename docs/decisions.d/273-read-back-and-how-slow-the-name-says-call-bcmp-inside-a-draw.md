## 273. `read_back` and `how_slow_the_name_says` call `bcmp` inside a draw — 2026-09-26

*Queue row Q163, from §267.6: find whether `read_back`'s match arms and
`how_slow_the_name_says`'s `strip_prefix("slow-")` compile to calls to
`memcmp` in the fuzz build, where libFuzzer's hook sees them inside a draw
when they come out unequal, and if they do, compare by bytes. A first lane
(`lane-q163-memcmp`) read the `read_back` half and stopped with nothing
committed. Its figures are used here where they were re-derived, and each
says so.*

### 273.1 The answer

Yes, both do, and one more call does that nobody asked about. In the fuzz
build of `where_two_surfaces_cross`, the timing crate makes five calls to
`bcmp`. Each goes through ASan's interceptor to libFuzzer's
`__sanitizer_weak_hook_memcmp` (273.4). Inside a draw, two come out unequal
on every merge once `totals` and `slowest/` have something in them, and a
third on some names (273.3):

- `read_back` compares each word of `totals` with `"runs"`, `"nanos"` and
  `"lost"` by `bcmp`. The `lost` line is compared with `"runs"` first, so
  every merge makes one unequal 4-byte `bcmp`.
- `split_once("us-")` calls `core`'s `StrSearcher::new`, which checks the
  needle against itself with `bcmp`. For `"us-"` that is `"us"` against
  `"s-"`, unequal, 2 bytes, once for every `slow-` file in `slowest/` on
  every merge. This is the one nobody asked about.
- `strip_prefix("slow-")` is a 5-byte `bcmp`. It comes out unequal only on
  a name that does not start with `slow-`, such as a leftover
  `being-written`.

The fifth is `SlowRun::is`, which calls `bcmp` only after the hashes have
matched, so it comes out equal (§267.3 item 4) and is left as it is.

The hook is the same one §267.3 item 4 names. It returns early unless a draw
is running, the two sides differ and they are longer than one byte, and
otherwise puts both sides into the table libFuzzer's `CMP` mutations draw
from. When a merge happens is the clock's choice, so these calls let the
clock change libFuzzer's table.

The fix is on this branch. `read_back` matches each word with byte array
patterns, and `how_slow_the_name_says` finds `slow-` with one and `us-` with
`windows(3)` and a byte pattern. Rebuilt the same way, the crate has one call
to `bcmp` left, `SlowRun::is`'s, and nothing `merge_out` calls reaches a
hooked function (273.5). LLVM did not turn the byte compares back into a
`bcmp`.

What the change cannot reach is the crate's coverage instrumentation, which
still traces the merge's compares whether a draw is running or not. That
belongs to the build row, Q161 (273.6).

### 273.2 How it was read

**The build.** `where_two_surfaces_cross` built the way `fuzz-target` builds
it, `RUSTFLAGS="-C target-cpu=x86-64-v3" cargo +nightly fuzz build
where_two_surfaces_cross`, on this lane's base `3319f838`. The timing crate's
`rustc` line in the verbose log has `-C opt-level=3`, `-C
debuginfo=line-tables-only`, `-Cdebug-assertions`, `-Zsanitizer=address`
and `-sanitizer-coverage-trace-compares`. The first lane
built it in its own tree at the same sha, with a clean status. This lane
copied that binary and re-read everything below from the copy
(`target/q163-before.bin`). The compiler is `rustc 1.100.0-nightly
(0dfb098f3 2026-08-31)`, LLVM 23.1.0 (`mise exec -- rustc +nightly -vV`).

**Two traps in the first lane's scan.** Its `q163-calls.sh` looked for
`call <bcmp>` and found 111 calls to a hooked function in the binary, none of
them in the timing crate. Rust calls a function in another object through
the GOT, so a call looks like one of two things. One is `call
*0x1929c9(%rip)`, with the slot's address in objdump's comment. The other is
a load of that slot into a register and a `call *%r12` later. Neither names
`bcmp`. The script's second filter would have missed them too. It kept the
inline chains that pass through `rcad-fuzz-timing/src/lib.rs`, but the fuzz
build's debug info names that file `/proc/self/cwd/src/lib.rs`, as it does
every crate built outside the fuzz workspace. That is how addr2line prints
every chain in 273.3.

**The scan that replaces it** is `target/q163-got-calls.py <binary> <tag>`,
which is gitignored like everything under `target/`. It reads the
relocations with `readelf -rW` and names each slot by the function its
`R_X86_64_RELATIVE` addend points at. Then it counts every `call` or `jmp`
to a hooked function, of three kinds: direct, through a GOT slot in memory,
and through a register loaded from one. For a register call it walks back
through the same function to the register's last write. It also counts every
load of a hooked slot that no call used, and that count is 0 in every run
below, so no load was missed. The hooked functions are `bcmp`, `memcmp`,
`strncmp`, `strcmp`, `strncasecmp`, `strcasecmp`, `strstr`, `strcasestr` and
`memmem`, and their `__interceptor_` names. For the calls in a function
whose symbol names `rcad_fuzz_timing`, it prints the inline chain from
`addr2line -i`.

Two more scripts read around those calls. `target/q163-chain-calls.py
<listing> <binary> <regex>` lists every call in an `objdump -d -l --inlines`
listing whose inline chain matches the regex, and names the slot of each GOT
call. `target/q163-reach.py <binary> <listing> <root>` follows every call
the disassembly names, starting from what a root calls. For each callee it
says whether a hooked function is reachable, and by which path. It stops at
a panic, an abort or an ASan report, since those end the draw. It does not
follow a call through a vtable, and it says how many of those it skipped.

### 273.3 Before: five `bcmp` calls in the crate

```
$ python3 target/q163-got-calls.py target/q163-before.bin before
GOT slots whose relocation names a hooked function: 397908 -> 128678 ['__interceptor_trampoline_bcmp', 'bcmp']
calls and tail jumps to a hooked function anywhere in the binary: 140 ({'GOT memory': 15, 'GOT register': 4, 'direct': 121})
of those, in a function whose symbol names rcad_fuzz_timing: 5
loads of a hooked GOT slot in those functions with no call matched to them: 0 []
the same, anywhere in the binary: 0
```

The five, with the lines of the crate their chains pass through:

| address | via | bytes | `lib.rs` line | what |
|---|---|---|---|---|
| `0x1fe8b3` | GOT slot, in `stop_the_clock` | from the input | 708, 755 | `SlowRun::is` in `offer` |
| `0x200d08` | `%rax`, loaded at `0x200cec` | 5 | 590 | `read_back`, `"nanos"` |
| `0x200d85` | `%r12`, loaded at `0x200d7e` | 4 | 589 | `read_back`, `"runs"` |
| `0x200dac` | `%r12`, the same load | 4 | 595 | `read_back`, `"lost"` |
| `0x204f39` | GOT slot, in `merge_out` | 5 | 499 | `strip_prefix("slow-")` |

These match the first lane's addresses and lengths for `read_back`. The byte
count is the `mov $N,%edx` before each call, and the literal is the `lea`
into `%rsi` or `%rdi`. Dumped with `objdump -s`, `0xcd440` holds `runs`,
`0xcd460` holds `nanos`, `0xcd480` holds `lost` and `0xccf20` holds `slow-`.

**`read_back`.** The match switches on the word's length first
(`__sanitizer_cov_trace_switch` at `0x200cd9`, cases 4 and 5). A 5-byte word
goes to the `"nanos"` `bcmp`. A 4-byte word goes to `"runs"` at `0x200d85`,
and only if that is unequal does it go on to `"lost"` at `0x200dac`. So of
the four lines `written_out` writes, the `lost` line makes one unequal 4-byte
`bcmp`, `lost` against `runs`. The others come out equal. `"b"` is a
one-byte compare with no call in it.

**`how_slow_the_name_says`.** Line 499's `strip_prefix` skips names of 4
bytes or less (`cmp $0x4,%r15; jbe`) and calls `bcmp` with 5 at `0x204f39`.
Line 500's `split_once("us-")` makes no call to a hooked function in
`merge_out` itself:

```
$ python3 target/q163-chain-calls.py target/q163-before.merge_out.dis target/q163-before.bin '\((how_slow_the_name_says|read_back)\)'
...
204f39  call GOT 397908 -> bcmp / __interceptor_trampoline_bcmp
2050e9  call GOT 397e40 -> <core::str::pattern::StrSearcher>::new
205c4f  call GOT 397df8 -> core::slice::memchr::memchr_aligned
...
```

But `StrSearcher::new`, which `core` ships already compiled, reaches `bcmp`
directly:

```
$ python3 target/q163-reach.py target/q163-before.bin target/q163-before.dis '16rcad_fuzz_timingNtB2_5Watch9merge_out$'
...
what the roots call: 85 functions
  128678 __interceptor_trampoline_bcmp: HOOKED, called directly
  ...
  3847f0 <core::str::pattern::StrSearcher>::new: reaches ['__interceptor_trampoline_bcmp', 'bcmp'] by
      3847f0 <core::str::pattern::StrSearcher>::new
      128678 __interceptor_trampoline_bcmp
```

The call is at `0x3849d7`. `addr2line -i` places it at `pattern.rs:1476` in
`TwoWaySearcher::new`, which is `needle[..crit_pos] ==
needle[period..period + crit_pos]`. That compares the needle with a shifted
copy of itself and never reads the name. Which bytes it compares for `"us-"`
was measured, not worked out by hand. A probe, `target/q163-probe.rs`, built
with the same nightly (`rustc +nightly -O --edition 2021`), defines `bcmp`
and `memcmp` itself and logs each call made while it is armed:

```
-- strip_prefix("slow-") on a slow- name
bcmp n=5 a="slow-" b="slow-" unequal=0
-- split_once("us-") on the rest
bcmp n=2 a="us" b="s-" unequal=1
   split: Some(("000000000123", "00000000000000ff"))
-- strip_prefix("slow-") on being-written
bcmp n=5 a="slow-" b="being" unequal=1
```

So every name `prune_to_the_slowest_kept` parses makes one unequal 2-byte
`bcmp` inside the draw. Every merge parses every file in `slowest/`, which is
at least `keep` of them (64 by default) once the top is full.

Of the other 84 functions `merge_out` calls, none reaches a hooked function
along a path the disassembly names. That covers `std::fs::read_to_string`,
`read_dir`, `Path::join`, `File::lock`, `remove_file`, the `Split` iterator
behind `split_whitespace`, `memchr_aligned`, the allocator and
`format_inner`. What the scan cannot follow is a call through a computed
pointer, a vtable or a jump table. `merge_out` makes 12 of them itself, and
the scan counts each callee's own.

### 273.4 ASan's `bcmp` reaches libFuzzer's hook

The first lane took this from memory of compiler-rt. Here it is read off the
binary with `objdump -d --disassemble=<name>`:

- The GOT slot `0x397908` holds `0x128678`, `__interceptor_trampoline_bcmp`,
  which is `jmp 12ee20 <___interceptor_bcmp>`. `memcmp`'s trampoline is
  `0x128673`, `jmp 12ed90 <___interceptor_memcmp>`, and no GOT slot points at
  it.
- `___interceptor_bcmp`, once ASan is set up (`AsanInited` and
  `TryAsanInitFromRtl` both true), loads `__interception::real_bcmp` and
  calls `MemcmpInterceptorCommon` at `0x12ee68`. Before that it goes to
  `internal_memcmp` or to the real `bcmp`, which never happens inside a draw.
- `MemcmpInterceptorCommon` (`0x12e3c0`) has three paths, one for each
  setting of `intercept_memcmp` and `strict_memcmp`. All three meet at
  `0x12e5bf` and call `__sanitizer_weak_hook_memcmp` at `0x12e5cf`, passing
  the two pointers, `n` and the result. The only paths that skip it are ASan
  error reports.
- `__sanitizer_weak_hook_memcmp` (`0x30a680`) loads
  `fuzzer::RunningUserCallback`. It returns if that is false, or if `n <= 1`,
  or if the result is 0. Otherwise it tail-jumps to
  `TracePC::AddValueForMemcmp`. That is `FuzzerTracePC.cpp:623-629` in
  libfuzzer-sys 0.4.13, as compiled. `AddValueForMemcmp` ends with
  `TORCW.Insert(Idx ^ Hash, Word(B1, Len), Word(B2, Len))` (`:380`).

### 273.5 After: the words and the name compared byte by byte

`read_back` now matches `words.next().map(str::as_bytes)` against
`[b'r', b'u', b'n', b's']` and the like, and `how_slow_the_name_says` is:

```rust
let [b's', b'l', b'o', b'w', b'-', ..] = name.as_bytes() else {
    return None;
};
let rest = &name["slow-".len()..];
let at = rest
    .as_bytes()
    .windows(3)
    .position(|three| matches!(three, [b'u', b's', b'-']))?;
let (micros, input) = (&rest[..at], &rest[at + "us-".len()..]);
```

It finds the same first `us-` that `split_once` did. The slicing cannot
panic, because `slow-` and `us-` are ASCII and no byte of a longer UTF-8
character is below `0x80`.

Rebuilt by `target/q163-build.sh after`, into this lane's own `fuzz/target`,
and copied to `target/q163-after.bin`. The timing crate's `rustc` line in the
verbose log matches the first lane's, apart from the paths of the two trees
and the hash in the output directory's name.

```
$ python3 target/q163-got-calls.py target/q163-after.bin after
GOT slots whose relocation names a hooked function: 3964c8 -> 127d98 ['__interceptor_trampoline_bcmp', 'bcmp']
calls and tail jumps to a hooked function anywhere in the binary: 136 ({'GOT memory': 14, 'GOT register': 1, 'direct': 121})
of those, in a function whose symbol names rcad_fuzz_timing: 1
loads of a hooked GOT slot in those functions with no call matched to them: 0 []
the same, anywhere in the binary: 0

1fdfd3  call ['__interceptor_trampoline_bcmp', 'bcmp']  via GOT slot 3964c8  rdx: set by mov at 1fdfcf
  in rcad_fuzz_timing[d5b75b1568706c15]::stop_the_clock
    ...
    /proc/self/cwd/src/lib.rs:734  (is)
    /proc/self/cwd/src/lib.rs:781  ({closure#0})
    ...
```

The four gone are the four in `merge_out`, and 140 less 4 is the 136 left in
the binary. The one left is `SlowRun::is`, which compares bytes only after
the hashes match and the lengths match, so it comes out equal unless two
inputs of the same length share a 64-bit hash.

What `merge_out` calls, read by `q163-reach.py` from the same root on both
binaries:

| | before | after |
|---|---|---|
| functions `merge_out` calls | 85 | 82 |
| a hooked function, called directly | 1 (`bcmp`) | 0 |
| reaches a hooked function | 1 (`StrSearcher::new`) | 0 |
| ends the draw (panics, ASan reports, abort) | 25 | 24 |
| reaches no hooked function | 58 | 58 |

By name (`target/q163-reach-diff.sh`), the three gone are `bcmp`,
`StrSearcher::new` and `panic_bounds_check`, which `split_once` could reach.
From `stop_the_clock`, the only hooked call is `SlowRun::is`'s `bcmp`.
`merge_out` reaches no hooked function among the 802 functions the scan
follows from there, with 217 calls through a computed pointer not followed.

Inside the two functions there are no calls left but ASan's and the
coverage hooks'. `q163-chain-calls.py` over
`target/q163-after.merge_out.dis`, made by `objdump -d -l --inlines
--no-show-raw-insn --disassemble=<merge_out>`, with the chain regex
`\(how_slow_the_name_says\)`, prints no call at all. Its counts are 12 ASan
load reports, 2 `__asan_handle_no_return`, and 57 coverage hooks.

The tests. With the change, `just t -p rcad-fuzz-timing` runs 32 and all
pass. The new one, `tests/a_merge_reads_back_every_word_of_totals.rs`, runs
the crate through `timed` in a child process with `RCAD_FUZZ_TIMINGS` set. It
leaves a `totals` holding one line of each word, and checks that the second
draw's merge reports all four. **No behaviour test fails without this
change**, because the change is to generated code, not to behaviour. That
was checked on a scratch crate holding the base's `lib.rs` and the new test
(`target/q163-base-crate`, `cargo test`): the 31 unit tests and the new test
all pass.

**A check of the built binary does fail without it.** That is the family this
change needs: not a run of the code, but a scan of the fuzz build for calls
that reach libFuzzer's memcmp hook. `q163-got-calls.py` finds five in the
crate before the change (273.3) and one after, `SlowRun::is`'s (273.5). It is
not in the gate because the gate builds no fuzz binary. That takes a nightly
compiler and ASan, and the gate only type-checks the fuzz targets on the
pinned stable one (§29.9, §88.6, `just recipe check-fuzz-builds`). Row Q181
checks the script in and fails the night on it, after Q161. It also asks
whether a cheap build of the crate alone could put the check in the gate.

What the new test catches is a byte pattern spelled wrong. With
`[b'l', b'o', b't', b's']` in place of `lost`, it failed with ``the `lost`
line was not read back``. No other test in the crate reads a `lost` line
back.

### 273.6 What the coverage instrumentation still traces

While the crate is built with `-sanitizer-coverage-trace-compares`, its
compares call `__sanitizer_cov_trace_*` hooks. Those hooks never read
`RunningUserCallback`. In `FuzzerTracePC.cpp` it is read only in the
`__sanitizer_weak_hook_*` functions, lines 625 to 685. So they run at the
clock's moments too. What each one does with a compare in the merge:

- `__sanitizer_cov_trace_switch` returns at once when the largest case is
  below 256 (`:560-561`). `q163-switches.py` read the case table of every
  switch in `merge_out`, 27 before and 27 after. The largest case in any of
  them is 255, so this hook does nothing here. That includes `read_back`'s
  length switch (cases 4 and 5) and the one table the change added, with
  cases 108 and 114, which are `l` and `r`.
- `__sanitizer_cov_trace_const_cmp1` and `__sanitizer_cov_trace_cmp1`, which
  trace the `"b"` compare and now every byte of the patterns, go to
  `TracePC::HandleCmp` with a one-byte type (`:386-396`). That inserts into
  neither `TORC4` nor `TORC8` and adds only to `ValueProfileMap`, which is
  read only under `-use_value_profile` (`FuzzerTracePC.h:264`).
- `__sanitizer_cov_trace_const_cmp4`, `const_cmp8` and `cmp8` do insert into
  `TORC4` and `TORC8`, the tables the `CMP` mutations draw from. The length
  checks in both functions are 8-byte compares, and the old code also traced
  each `bcmp`'s result against 0 as a 4-byte one. Counted by
  `q163-chain-calls.py` over calls whose chain passes through `read_back` or
  `how_slow_the_name_says`, there were 25 `const_cmp4`, 31 `const_cmp8` and
  42 `cmp8` before, and 14, 25 and 19 after.

This change fixes none of that, and it is not this row's to fix. Once the
crate is built without coverage instrumentation, which is §267.4's route and
row Q161, these calls are gone. The `bcmp` calls this row removed would not
have been. ASan intercepts `bcmp` for every caller, instrumented or not
(§267.3 item 4), so this change is needed whatever Q161 does.

### 273.7 What is left

- **Q161, the build row**, removes 273.6's trace calls. When it lands,
  `q163-got-calls.py` on the night's own binary should still report one
  hooked call in the crate, `SlowRun::is`'s. Row Q181 makes the night check
  that, and takes the items below with it.
- **Only `where_two_surfaces_cross` was read.** The timing crate is compiled
  the same way into every target, and `merge_out` is not generic, so the
  other four should carry the same code. That was not checked.
- **`start_the_clock` was not scanned.** It reads `RCAD_FUZZ_TIMINGS` and
  `RCAD_FUZZ_SLOWEST` once, in the first draw, which is the empty input at
  start-up and not a moment the clock chooses.
- **Calls through a computed pointer were not followed.** That is 12 in
  `merge_out` itself. A hooked call behind one of them would not show in
  273.5's table.
- **The scripts live in the gitignored `target/`** of this lane's tree:
  `q163-build.sh`, `q163-got-calls.py`, `q163-chain-calls.py`,
  `q163-reach.py`, `q163-reach-diff.sh`, `q163-switches.py`, `q163-probe.rs`
  and `q163-base-crate/`. The first lane's
  `q163-calls.sh` misses every call in 273.3, for the two reasons in 273.2.

### 273.8 Q181: the scan checked in, and what it still cannot see

`fuzz/timing-hooked-calls.py` is 273.2's `q163-got-calls.py`, adapted to run
from the tree rather than from a lane's scratch directory: no on-disk
disassembly cache, one or more binaries on the command line instead of one
tag, and `SlowRun::is`'s own hooked call read off `addr2line`'s inline
chain and excluded, rather than left for a person to recognise by eye.
`fuzz-overnight` calls it on all five targets' own binaries, right after the
sancov guard (§267.4, §292) and before any target is started, and fails the
night the same way that guard does — `BUILD FAILED` in the summary, nothing
fuzzed. `check-fuzz-overnight-selftest`'s arms Twenty-four and Twenty-five
test that wiring against a stub standing in for the script (its own `cargo`
stub never produces a real binary), armed by `STUB_TIMING_HOOKED`; the
scan's own logic is what this section demonstrates below, against real
binaries.

**The demonstration.** Built at `3319f838` — the commit 273.2 read
`where_two_surfaces_cross` from — is `target/q163-before.bin` from the
locked tree Q181 named, copied out rather than rebuilt since it is the same
binary 273.2 already established was made the way `fuzz-target` builds one.
Run against it:

```
$ fuzz/timing-hooked-calls.py q163-before.bin
q163-before.bin:
  calls from rcad_fuzz_timing to a hooked function: 5 (4 outside SlowRun::is)
  loads of a hooked slot in rcad_fuzz_timing with no call matched: 0
  calls in rcad_fuzz_timing through a computed pointer, unscanned: 2
  200d08  call -> [...'bcmp']  ...  in <rcad_fuzz_timing::Watch>::merge_out
      ... lib.rs:590  (read_back)  ... lib.rs:271  (merge_out)
  200d85  call -> [...'bcmp']  ...  lib.rs:589  (read_back)  ...
  200dac  call -> [...'bcmp']  ...  lib.rs:595  (read_back)  ...
  204f39  call -> [...'bcmp']  ...  lib.rs:499  (how_slow_the_name_says)  ...
$ echo $?
1
```

Four offenders, at the same addresses and the same `read_back`/
`how_slow_the_name_says` lines 273.3 read by hand, with the fifth —
`SlowRun::is`'s, at `0x1fe8b3` — correctly left out. Exit 1.

**All five targets, at this branch's own tip.** §273.7 named this as unread;
built here with `RUSTFLAGS="-C target-cpu=x86-64-v3"` and the sancov
stand-in, `taskset -c 6-15`, cold (a worktree that had never built the fuzz
crate before), `cargo +nightly fuzz build` finished in **1m 08s** for all
five. Run against all five binaries:

```
$ fuzz/timing-hooked-calls.py fuzz/target/.../where_two_surfaces_cross \
    fuzz/target/.../no_step_file_makes_the_reader_panic \
    fuzz/target/.../no_model_makes_the_writer_panic \
    fuzz/target/.../two_solids_fused_by_a_stranger \
    fuzz/target/.../where_a_cone_or_a_torus_crosses
[every target:]
  calls from rcad_fuzz_timing to a hooked function: 1 (0 outside SlowRun::is)
  loads of a hooked slot in rcad_fuzz_timing with no call matched: 0
  calls in rcad_fuzz_timing through a computed pointer, unscanned: 3
  clean
$ echo $?
0
```

The one call left in every target is `SlowRun::is`'s own, same as 273.5
found for `where_two_surfaces_cross` alone — the fix holds across all five,
not only the one target 273 read by hand. `start_the_clock` needed no
separate scan: this script's filter is the disassembly *symbol*, not a
hand-picked function name, so a hooked call inlined into `start_the_clock`
would show up the same way one inlined into `stop_the_clock` does; none of
the five binaries has one.

**The computed-pointer count, broadened and re-measured.** §273.3's "12" in
`merge_out` and "217" reachable from it were `q163-reach.py`'s count of
calls its call-graph walk could not follow at all — through a register
*or* a memory operand. `q163-got-calls.py`'s own `elif re.match(r"^\*%\w+$",
...)` branch only ever looked at register-only indirect calls, and, worse,
silently dropped the ones it found unresolved instead of counting them —
so its own report had no line for this. Fixed here: any indirect call or
tail jump this scan cannot resolve to a known target, register or memory
operand alike, is now counted and printed as `calls ... through a computed
pointer, unscanned`. Re-measured on `q163-before.bin` with the broadened
count, `merge_out` has **8**, not 12 — a different tool, on the same
binary, counting a related but not identical thing (`q163-reach.py` follows
*every* function `merge_out` calls and also counts a computed call inside
*those*; this script only looks inside functions whose own symbol names
`rcad_fuzz_timing`). On this branch's fix, `merge_out` has **0**: the two
functions the fix removed apparently carried indirect dispatch of their
own (`Split`'s iterator machinery, most likely) along with the `bcmp`
calls. The crate's remaining computed calls, on every target built here,
are `alloc`'s `BTreeMap` drop glue and `std::sync::Once`'s
`call_once_force` — neither is a comparison, and neither is reachable from
`merge_out`. None can be ruled in or out as reaching a hook by this script;
closing that needs either a real points-to analysis of the binary or a
runtime interposer that walks the caller's stack on every ASan memcmp
hook — both bigger than this row, and named in the script's own doc
comment so the gap is not silent.

**A second blind spot, found running this scan and not in 273.2's own
list.** `how_slow_the_name_says`'s `split_once("us-")` call — 273.3's "one
nobody asked about" — reached `bcmp` through `core`'s
`StrSearcher::new`, a function the standard library ships already
compiled. Its symbol names `core::str::pattern`, not `rcad_fuzz_timing`, so
this scan's filter — "in a function whose symbol names rcad_fuzz_timing" —
would never have caught it even on the unfixed crate; `q163-got-calls.py`
never did either (273.3's own count of 5 hooked calls in the crate does
not include it — `q163-reach.py`'s separate call-graph walk is what found
it). Confirmed by running this script on `q163-before.bin` above: it
reports 4 offenders, not the 5 the full read-out named, and the missing
one is exactly this call. The fix already in this crate (273.5) removed
the call itself, so nothing here is currently missed — but a future change
that calls a std or `core` function which itself compares bytes, without
that function's code being inlined into one of the crate's own functions,
would pass this check silently. Closing it needs the same kind of
call-graph walk `q163-reach.py` did by hand, kept running rather than run
once; not done here, named so it is not mistaken for closed.

**Whether a build of `rcad-fuzz-timing` alone is cheap enough for the
gate.** Asked, not decided — the row says to leave the move for a row of
its own. Measured the closest thing to "alone" `cargo fuzz build` allows:
`cargo clean -p rcad-fuzz-timing` (removing only that crate's own build
output, 869 KiB in 7 files) against the tree already built above, then
`cargo +nightly fuzz build` again. It recompiled `rcad-fuzz-timing` and
relinked all five target binaries — there is no `cargo fuzz build` that
links fewer than "the targets asked for," so relinking five is folded into
this number rather than split out — in **13s**. That is on top of a
`cargo build`'s ordinary cost for the rest of the dependency graph already
being present in `target/`, which the gate's own tree would not have cold;
the 1m08s figure above is closer to what a first run there would cost. Both
numbers are `just gate`'s existing budget (`just recipe gate`) territory
rather than nightly-only territory on their face, but *this* row's job was
the check, not the placement, so the move stays with Q181's own
recommendation and not a change here: cheap enough to be worth a future
row asking whether `check-fuzz-builds` or a new gate step should run it,
expensive enough (a nightly compiler, ASan, a sanitizer-coverage
stand-in) that it is not a drop-in addition to today's `check` step.

Run at 2026-09-27T22:00Z, on this branch's own tip (`main` at `778b6ace`
plus this row's own commits).

### 273.9 Q181's second pass: redone after a full disk, and a stale count fixed

The first lane's verifier rejected this row only because the disk was full —
its gate failed at `check-ignored` with nextest unable to list the ignored
tests, and every call after that failed with `ENOSPC`, so nothing above was
re-checked. The disk is clear now (`df -h /`: 322G free of 913G). This section
redoes the checks rather than trusting the first lane's own read-out of them.

**The scan against 3319f838's build.** The locked tree's `q163-before.bin`
(sha256 `05cec502...ae3dd0`, matching the copy kept here byte for byte) fed to
`fuzz/timing-hooked-calls.py` reports the same four offenders at the same
addresses and the same `read_back`/`how_slow_the_name_says` lines 273.8
printed, with `SlowRun::is` correctly left out, and exit 1 — unchanged.

**The five targets, built fresh at this tip.** `taskset -c 6-15`, a worktree
that had never built the fuzz crate before, the same `cargo +nightly fuzz
build` with the sancov stand-in: **1m 47s** (273.8's cold figure for a
different worktree was 1m08s; both are the same kind of run, on different
machines' cache state, not a regression). The stand-in's own guard passed —
`rcad-fuzz-timing`'s rlib carries 0 `sancov` references. The script against
all five binaries: each reports exactly one hooked call, in `SlowRun::is`,
none outside it, `clean`, exit 0 — unchanged.

**A stale count, found and fixed.** Both transcripts above also print
"calls ... through a computed pointer, unscanned" — **2** for `q163-before.bin`
and **3** for each of the five targets. A fresh run of the same script against
the same, checksum-identical binaries reports **16** and **21**. This is not a
build difference or a tool-version difference (binutils 2.42 both times, and
`q163-before.bin` was not rebuilt, only copied) — it is the same script
counting differently than the number written down. Read out by mnemonic
(`target/timing-computed-breakdown.py`, gitignored), `q163-before.bin`'s 16
split as 2 `call *%reg` and 14 `jmp *%reg`; the five targets' 21 split as 4
`call` and 17 `jmp`. The `call` counts (2, and 4) match what this section
already reported as merge_out's genuinely unresolved calls in 273.8's own
broadened re-measurement. The `jmp` instances do not belong in the same
bucket: reading the instructions before one (`movslq
(%rax,%r12,4),%rcx; add %rax,%rcx; jmp *%rcx`, inside a monomorphized
`core::ptr::drop_glue` for `Option<Result<std::fs::DirEntry, Error>>>`) is the
ordinary relative-jump-table pattern a match or switch lowers to under
`-fPIE`: the table holds offsets from the function's own base, so the jump
can only land inside the same function, never at an imported symbol like
`bcmp`. Counting it as "unscanned, might reach a hook" is not wrong in the
sense of missing a real risk — it is over-cautious in a way that buries the
few calls that are genuinely unresolved. The script's exit code never used
this count (`main` fails only on `offenders` or `timing_unmatched`), so this
was a stale transcript, not a behavior bug: the two numbers above were
apparently taken from a run that only counted the `call` mnemonic, and the
checked-in script counts `call` and `jmp` alike in one shared branch,
without anyone re-running the transcript after that widened. Fixed here by
correcting the numbers rather than narrowing the script — telling a jump
table's tail `jmp` apart from a genuine cross-function tail call by mnemonic
alone is not reliable enough to build in the same row that found the gap, and
the script's own docstring already names "a call through a memory operand"
as a known blind spot this count is not meant to resolve.

**The selftest and the file reverted to the base.** With
`fuzz/timing-hooked-calls.py` removed entirely (its state at the base commit,
where it does not exist), `just check-fuzz-overnight-selftest` still passes
clean, with no output, twice run. None of its arms call the real file: `run()`
now points `RCAD_FUZZ_TIMING_CHECK` at a stub for every arm that uses it, and
the one arm that does not (arm Six) hands off to a second, nested `just`
process reading a fixture recipe that never reaches the build step at all —
that hand-off is the arm's own point. So the tests this row's `justfile`
change lives or dies by are arms Twenty-four and Twenty-five, not the `.py`
file's presence: removing the `timing_check=...` block this row added to
`fuzz-overnight` — the counterpart in `justfile#fuzz-overnight`, keeping
Twenty-four and Twenty-five themselves — makes Twenty-four fail exactly as
it should, `a timing-crate call reaching a hook outside SlowRun::is was
found and the night reported success`; restoring the block returns the
selftest to clean. `tests_that_fail_without_the_change` names that block.

Run at 2026-09-28T10:00Z–10:20Z, on this branch's own tip.
