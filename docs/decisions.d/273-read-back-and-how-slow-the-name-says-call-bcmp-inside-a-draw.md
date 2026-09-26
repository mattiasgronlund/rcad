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
`__sanitizer_weak_hook_memcmp` (273.4). Three of them come out unequal
inside a draw in normal running (273.3):

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

### 273.2 How it was read

**The build.** `where_two_surfaces_cross` built the way `fuzz-target` builds
it, `RUSTFLAGS="-C target-cpu=x86-64-v3" cargo +nightly fuzz build
where_two_surfaces_cross`, on this lane's base `3319f838`. That is release,
optimised, with debug info, `-Cdebug-assertions` and ASan. The first lane
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
`format_inner`. The paths not followed are calls through a vtable, such as
`fmt`'s writer. `merge_out` makes 12 of them itself, and each callee's line
counts its own.

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
