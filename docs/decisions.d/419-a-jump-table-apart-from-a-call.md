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
