## 419. A jump table apart from a call — 2026-10-08

*Queue row Q1444, re-filed from Q321. Claimed 2026-10-08T21:41:27Z. Branch
`lane-q1444-the-split-verified-and-written-down`, off `374a44e4`, carrying
forward the first lane's four commits (`129a1d10`, `b3bc2adc`, `44e72485`,
`5b5541e7`) on branch `lane-q1444-a-jump-table-apart-from-a-call`. §273.9
found the gap this closes.*

### 419.1 What the split counts

`fuzz/timing-hooked-calls.py`'s `scan()` already tracked, per register, the
one chain that proves a `call *%reg` reaches a hooked GOT slot: a load of
that slot, matched later by the call. §273.9 found that its one remaining
bucket — "calls ... through a computed pointer, unscanned" — lumped two very
different things together: a genuine unresolved call or tail jump that might
reach anywhere, and the tail of an ordinary relative jump table, the
lowering of a `match` or `switch` under `-fPIE`, which cannot reach anywhere
but inside its own function.

This row adds a second, independent per-register chain for that jump
table's own shape: a `lea table(%rip),%base` (the table's fixed base), a
`movslq (%base,%idx,4),%off` (a 4-byte entry read off that base), and an
`add %base,%off` (the entry turned back into an absolute address by adding
the same base again). A tail `jmp *%reg` whose register was last set by that
third step — add of the base back into the offset — is reported apart, as
"staying inside their own function": the table holds offsets from an address
the function's own code computed, never from an imported symbol, so the sum
can only land somewhere this same function already contains.

**Why a jump the chain does not match stays on the can-leave side.** §273.9
also found, reading `objdump` by hand, that a `jmp *%reg` cannot be told
apart from a genuine cross-function indirect jump by mnemonic alone — only
the full three-step sequence, ending in the same base register added back
into itself, proves the register holds a table-relative sum rather than
some other computed value. So the match is deliberately narrow: a bare
`jmp *%reg` with no load or add in front of it, a jump through a register
that merely happens to be live at the same point (a near miss, built as one
of the fixture's four cases below), and every `call *%reg` without
exception — cautious-side, even where the same three-step sequence precedes
it, since nothing rules out a compiler reusing a jump table's registers for
an ordinary indirect call — all stay in "can leave", renamed from the old
"calls ... through a computed pointer, unscanned" to "calls or tail jumps
... that can leave the function, unscanned" to say what it now holds. A call
through a memory operand (a vtable slot, a stored function pointer) stays
there too, unchanged from §273.9.

### 419.2 The fixture

`check-timing-hooked-calls-selftest` assembles a hand-written x86-64 object
(`as`, not a compiler — no optimiser votes on whether a jump table exists)
with four functions, one chain each so no register state leaks between
them:

- `rcad_fuzz_timing_table_jump`: the real sequence, `lea
  jumptable(%rip),%rax; movslq (%rax,%r12,4),%rcx; add %rax,%rcx; jmp *%rcx`.
- `rcad_fuzz_timing_bare_jmp`: `jmp *%rax` alone, no load or add before it.
- `rcad_fuzz_timing_near_miss`: the same load and add as the first function,
  but the tail jump is through `%rdx`, a register the chain never touched.
- `rcad_fuzz_timing_call_star`: `call *%rax`, with no load or add at all.

None of the four calls a hooked function, so the script's exit code and its
first two counts are unchanged (0, clean); the selftest checks only the two
lines this split adds: "staying inside their own function: 1" (the first
function alone) and "can leave the function, unscanned: 3" (the other
three — the bare jump, the near miss, and the call).

Checked against `374a44e4`'s copy of the script (`git show
374a44e4:fuzz/timing-hooked-calls.py`, restored after with `git checkout --`),
the fixture's four functions still print as one lumped line, "calls in
rcad_fuzz_timing through a computed pointer, unscanned: 4" — the selftest's
`grep -qxF` for the split lines finds neither and exits 1, for the missing
line rather than a crash or an unrecognised flag. Restoring the tip's script
and re-running (`mise exec -- just check-timing-hooked-calls-selftest`)
exits 0.

### 419.3 The five targets, measured at this branch's tip

Built with the same flags `fuzz-overnight` uses (`justfile`'s
`fuzz-overnight` recipe), but into a target directory of this lane's own
rather than `fuzz/target/`, so the rlib `check-fuzz-overnight-selftest`'s
`find | head -n 1` guard reads (arm sixteen's fixture) is never shadowed by
a real one — that collision is Q288's and Q857's, not this row's:

```
$ cd fuzz && CI=1 RUSTC="<repo>/fuzz/rustc-stand-in.sh" \
    RUSTFLAGS="-C target-cpu=x86-64-v3" \
    cargo +nightly fuzz build --target-dir <repo>/target/q1444-fuzz-build
   Finished `release` profile [optimized + debuginfo] target(s) in 3m 31s
```

(cold, nothing built in this worktree before; §273.9's cold figure for a
different worktree was 1m47s and §273.8's for a third was 1m08s — the same
kind of run, scattered by each machine's own cache state, not a regression.)

The stand-in's own guard, same check `fuzz-overnight` runs, against the rlib
it actually produced here:

```
$ nm target/q1444-fuzz-build/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing/78aff9ae22e3a9bf/out/librcad_fuzz_timing-78aff9ae22e3a9bf.rlib \
    | grep -ic sancov
0
```

The script against all five binaries:

```
$ fuzz/timing-hooked-calls.py \
    target/q1444-fuzz-build/.../where_two_surfaces_cross \
    target/q1444-fuzz-build/.../where_a_cone_or_a_torus_crosses \
    target/q1444-fuzz-build/.../no_step_file_makes_the_reader_panic \
    target/q1444-fuzz-build/.../no_model_makes_the_writer_panic \
    target/q1444-fuzz-build/.../two_solids_fused_by_a_stranger
[every target:]
  calls from rcad_fuzz_timing to a hooked function: 1 (0 outside SlowRun::is)
  loads of a hooked slot in rcad_fuzz_timing with no call matched: 0
  calls or tail jumps in rcad_fuzz_timing through a computed pointer that can leave the function, unscanned: 4
  tail jumps in rcad_fuzz_timing through a relative jump table, staying inside their own function: 17
  clean
$ echo $?
0
```

4 and 17 sum to 21 on every target, matching §273.9's lumped count at
`5ebbee66` (21 per target, read out by mnemonic there as 4 `call` and 17
`jmp`) exactly: the split moves the same 17 instructions from "unscanned"
into their own line, and counts nothing new or fewer.

This 4 is the count **before** Q1452's walk out of the crate; §419.5 gives
the count after.

**`q163-before.bin` (the `3319f838` build) was not re-measured here.** It
lived only as a copy kept in the locked worktree `wf_aa4db869-30f-1`, and
that worktree's `target/` is gone — Q321, the row Q1444 was re-filed from,
names the loss. §273.9's own reading of it (16 lumped, split by mnemonic as
2 `call` and 14 `jmp`) is the only record left; this row has nothing to add
to or check it against.

### 419.4 Q322 and Q323, re-read against this split

Q322's "4 offenders" is a different count entirely — hooked-call hits
outside `SlowRun::is`, from the script's first printed line, not the
computed-pointer bucket this row splits — so nothing in it changes here.

Q323 already reads "The indirect calls left in rcad_fuzz_timing, 4 per
target at `5ebbee66`": that is already the can-leave bucket's own count (4,
not the lumped 21), named along with the two call sites behind it (`BTreeMap`
drop glue in `alloc`, `call_once_force` in `std::sync::Once`) that match
§273.9's by-mnemonic reading. Q323 was filed with this split already in mind
— it is `after:Q1444` for that reason — and needs no correction: its number
was the can-leave bucket's all along, never the lumped one.

### 419.5 Q1452: the count after the walk out of the crate, and the fifth offender on `3319f838` — 2026-10-09

*Branch `lane-q1452-a-computed-call-through-a-helper-pinned`, off `c7760e33`,
carrying forward `lane-q1452-the-scan-follows-calls-out-of-the-crate`'s one
commit (`251cbadd`, off `602ee000`).*

Built at this row's own tip with §419.3's own command, into a target
directory of this lane's own:

```
$ cd fuzz && CI=1 RUSTC="<repo>/fuzz/rustc-stand-in.sh" \
    RUSTFLAGS="-C target-cpu=x86-64-v3" \
    cargo +nightly fuzz build --target-dir <repo>/target/q1452-fuzz-build
    Finished `release` profile [optimized + debuginfo] target(s) in 1m 44s
$ nm <repo>/target/q1452-fuzz-build/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing/8472baab04ea5884/out/librcad_fuzz_timing-8472baab04ea5884.rlib \
    | grep -ic sancov
0
```

The script against all five binaries:

```
$ fuzz/timing-hooked-calls.py \
    target/q1452-fuzz-build/.../where_two_surfaces_cross \
    target/q1452-fuzz-build/.../where_a_cone_or_a_torus_crosses \
    target/q1452-fuzz-build/.../no_step_file_makes_the_reader_panic \
    target/q1452-fuzz-build/.../no_model_makes_the_writer_panic \
    target/q1452-fuzz-build/.../two_solids_fused_by_a_stranger
[every target:]
  calls from rcad_fuzz_timing to a hooked function: 1 (0 outside SlowRun::is)
  loads of a hooked slot in rcad_fuzz_timing with no call matched: 0
  calls or tail jumps in rcad_fuzz_timing through a computed pointer that can leave the function, unscanned: 71
  tail jumps in rcad_fuzz_timing through a relative jump table, staying inside their own function: 17
  calls from rcad_fuzz_timing reaching a hooked function through another function outside the crate: 0
  clean
$ echo $?
0
```

**71 replaces §419.3's 4** on every target; the 17 is unchanged, since
`timing_table_jumps` still filters on the function's own symbol rather than
on `reachable_from`. The walk out of the crate (the merged commit above)
widened `timing_computed`'s own filter from "the function's symbol names
`rcad_fuzz_timing`" to "the function is reachable from one that does" — and
every helper the crate calls directly has computed-pointer calls of its own
(drop glue, iterator machinery, collection internals in `alloc` and `core`)
that nothing ever counted before, because the old filter never looked past
the crate's own functions. The script still exits 0 on all five targets:
wider counting changes what two lines say, not whether anything already
treated as an offender newly is one.

**§273.3's fifth offender — `how_slow_the_name_says`'s `split_once("us-")`
reaching `bcmp` through `core`'s `StrSearcher::new` — is still not found by
this scan, but not for the reason this row's brief gave.** The brief said
the floating nightly inlines the whole chain into `Watch::merge_out`, so no
`StrSearcher::new` symbol would exist to reach. Checked, and false: building
`3319f838`'s fuzz crate with the floating nightly and the same
`-C target-cpu=x86-64-v3` `fuzz-overnight` uses finds the symbol, at the
same address §273.3 read it at:

```
$ cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
    cargo +nightly fuzz build --target-dir <repo>/target/q1452-3319f838-build
    Finished `release` profile [optimized + debuginfo] target(s) in 2m 53s
$ nm target/q1452-3319f838-build/x86_64-unknown-linux-gnu/release/where_two_surfaces_cross \
    | grep -i StrSearcher
00000000003847f0 T _RNvMsu_NtNtCsh61yYgEBR7J_4core3str7patternNtB5_11StrSearcher3new
```

The wall is one call earlier. `merge_out` reaches `StrSearcher::new` through
a GOT slot, not a literal `call <name>`:

```
$ objdump -d --no-show-raw-insn target/q1452-3319f838-build/.../where_two_surfaces_cross | grep -n '2050e9:'
  2050e9:	call   *0x192d51(%rip)        # 397e40 <_DYNAMIC+0x7c0>
$ readelf -rW target/q1452-3319f838-build/.../where_two_surfaces_cross | grep 397e40
0000000000397e40  0000000000000008 R_X86_64_RELATIVE                         3847f0
```

The relocation resolves slot `397e40` to `3847f0`, `StrSearcher::new`,
confirming by hand what §273.3 read the same way with a different tool.
`scan()` treats a `(%rip)` slot load as one of two things only: a hooked
target (counted as a hit) or nothing — the branch that counts a "computed
pointer, unscanned" call explicitly excludes anything with `"(%rip)" in
rest and comment`, on the assumption that such a call is already decided
one way or the other by the hooked-slot check. A slot that resolves to some
*other* named function falls through both branches, and `calls_out` is only
ever populated by the separate "direct `call <name>`" match a few lines
above — so `merge_out`'s own edge to `StrSearcher::new` never exists for
`reachable_from` to walk, and `StrSearcher::new`'s own `call bcmp` a few
instructions later is never visited. Running the script on this build
confirms it: the offender line still reads 4, and the new chain line reads
0 — the fifth offender is not among them.

```
$ fuzz/timing-hooked-calls.py target/q1452-3319f838-build/.../where_two_surfaces_cross
  calls from rcad_fuzz_timing to a hooked function: 5 (4 outside SlowRun::is)
  calls from rcad_fuzz_timing reaching a hooked function through another function outside the crate: 0
```

This is a gap and not a limit: the relocation names the target exactly, the
same table `hooked_slots` already reads for a hooked one, and nothing about
the arithmetic refuses — the scan simply never asks that table about a slot
that is not on the hooked list. It is a real finding and not today's
callers' problem: every edge this crate's own code makes into a helper
Q1452 added the walk for is a literal `call <name>`; this GOT-slot shape
has only ever been met by building this one old commit and tracing the
relocation by hand, the "found only by a probe" class. A follow-up row is
queued for it.

The second half of the brief's claim holds as stated. `nightly-2025-03-05`
is rustc 1.87 and cannot build `3319f838`'s `Cargo.lock`:

```
$ cd <3319f838 worktree>/fuzz && rustup run nightly-2025-03-05 cargo fuzz build
error: rustc 1.87.0-nightly is not supported by the following packages:
  nalgebra@0.35.0 requires rustc 1.89.0
  rcad-fuzz-timing@0.0.0 requires rustc 1.96
  safe_arch@1.2.0 requires rustc 1.89
  wide@1.7.0 requires rustc 1.89
```

So the demonstration the brief asked for is not there — same conclusion as
the first lane reached, a different cause: not an unavailable symbol, but
an edge this scan has never tracked, on a shape this row is the first to
meet.

### 419.6 Q1533: the GOT slot resolved through the relocation table — 2026-10-09

*Branch `lane-q1533-a-call-through-a-got-slot-followed`, off `4dc3cb88`.*

`scan()` now resolves a `call *SLOT(%rip)` slot through the same
`R_X86_64_RELATIVE` table `hooked_slots` already reads. A slot already known
to be hooked is unchanged. A slot that resolves to some other named function
now gets a `calls_out` edge, exactly like a direct `call <name>`, so
`reachable_from` and `reaches_hook` walk through it. A slot that resolves to
no known function start falls into the existing "computed pointer,
unscanned" bucket rather than being dropped — the same bucket a `call *%reg`
this scan never traced to a slot already lands in.

**The pin.** A fourth hand-assembled fixture in
`check-timing-hooked-calls-selftest`: `rcad_fuzz_timing_got_slot` calls
through a GOT-shaped slot — `call *slot(%rip)` — that a real link (`as`, then
`ld -pie`, so `readelf -rW` shows a genuine `R_X86_64_RELATIVE`, not a
hand-written comment standing in for one) resolves to
`outside_the_crate_through_a_slot`, which calls `bcmp`. Against
`4dc3cb88:fuzz/timing-hooked-calls.py` (the base script) this fixture
reports `clean` and exits 0 — confirmed by running it. Against the fixed
script it exits 1, with:

```
  calls from rcad_fuzz_timing reaching a hooked function through another function outside the crate: 1
    rcad_fuzz_timing_got_slot -> outside_the_crate_through_a_slot -> bcmp
```

**The demonstration §419.5 could not give.** `3319f838`'s fuzz crate built
again, the same way: `git archive 3319f838` into
`target/q1533-3319f838-src`, then

```
$ cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
    cargo +nightly fuzz build --target-dir <repo>/target/q1533-3319f838-build
    Finished `release` profile [optimized + debuginfo] target(s) in 1m 25s
```

The fixed script against `where_two_surfaces_cross`:

```
  calls from rcad_fuzz_timing to a hooked function: 5 (4 outside SlowRun::is)
  calls or tail jumps in rcad_fuzz_timing through a computed pointer that can leave the function, unscanned: 249
  tail jumps in rcad_fuzz_timing through a relative jump table, staying inside their own function: 14
  calls from rcad_fuzz_timing reaching a hooked function through another function outside the crate: 2
    rcad_fuzz_timing::stop_the_clock -> <rcad_fuzz_timing::Watch>::merge_out -> <core::str::pattern::StrSearcher>::new -> __interceptor_trampoline_bcmp -> bcmp
    <rcad_fuzz_timing::Watch>::merge_out -> <core::str::pattern::StrSearcher>::new -> __interceptor_trampoline_bcmp -> bcmp
```

The fifth offender now reads as **2**, not 0: `merge_out` reaches
`StrSearcher::new` through GOT slot `397908` (the same shape as §419.5's
`397e40`), and two different crate functions reach it along that edge —
`merge_out` itself and `stop_the_clock`, which calls `merge_out` — so
`main`'s per-function chain walk names it twice, once from each entry
point. §419.5's offender line (4, direct hits) is unchanged; this row's fix
only ever touches the chain-offender line.

`4dc3cb88`'s own (unfixed) script against the same binary reads the chain
line as 0, the offender line as 4 — the exact gap §419.5 found, reproduced
on a fresh build rather than read off the old one:

```
$ fuzz/timing-hooked-calls.py target/q1533-3319f838-build/.../where_two_surfaces_cross   # base script
  calls from rcad_fuzz_timing to a hooked function: 5 (4 outside SlowRun::is)
  calls from rcad_fuzz_timing reaching a hooked function through another function outside the crate: 0
```

The "computed, unscanned" count moves too, on this same binary: 69
(`4dc3cb88`'s script, already past Q1452's walk-out-of-crate widening) to
249 (this fix). Same cause as §419.5's own 4 → 71: resolving the GOT-slot
edges widens `reachable_from`'s reachable set, which pulls in more
functions a timing-crate function reaches only through one of these edges
— each with real, unrelated `call *%reg` sites of its own that were never
reachable before. Wider counting, not a new offender: the offender and
chain-offender lines are what changed meaning, and both moved the way this
row says.

**Today's five targets, at this branch's own tip,** built with §419.3's own
command into a target directory of this row's own:

```
$ cd fuzz && CI=1 RUSTC="<repo>/fuzz/rustc-stand-in.sh" \
    RUSTFLAGS="-C target-cpu=x86-64-v3" \
    cargo +nightly fuzz build --target-dir <repo>/target/q1533-fuzz-build
    Finished `release` profile [optimized + debuginfo] target(s) in 2m 38s
$ nm <repo>/target/q1533-fuzz-build/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing/1722c9cc30ea4823/out/librcad_fuzz_timing-1722c9cc30ea4823.rlib \
    | grep -ic sancov
0
```

The fixed script against all five binaries, every target reading the same:

```
  calls from rcad_fuzz_timing to a hooked function: 1 (0 outside SlowRun::is)
  calls or tail jumps in rcad_fuzz_timing through a computed pointer that can leave the function, unscanned: 251
  tail jumps in rcad_fuzz_timing through a relative jump table, staying inside their own function: 17
  calls from rcad_fuzz_timing reaching a hooked function through another function outside the crate: 0
  clean
$ echo $?
0
```

**Following GOT edges turns up no real hooked call in today's crate.** The
chain-offender line reads 0 on every one of the five targets, and every
target still exits 0. The "unscanned" line moves from 71 (`4dc3cb88`'s own
script, read in §419.3/§419.5) to 251 here, the same widening as the
`3319f838` build above and for the same reason — nothing to report as a
new offender, since the count that would carry one stayed at 0.

Q323, the indirect calls through a computed pointer, now starts counting at
251 unscanned "can leave" calls and 17 table jumps per target (today's tip),
rather than 71 and 17.
