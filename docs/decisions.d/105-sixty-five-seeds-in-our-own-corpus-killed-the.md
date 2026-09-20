## 105. Sixty-five seeds in our own corpus killed the tool that explains them, fixed 2026-09-14

`rcad-fuzz-explain` prints a `bytes_for` line so a fuzz case can be pasted
straight into a test. On `b013ac4` it died on **65 of the 4709 seeds already
checked into `fuzz/corpus`** — including the one `just fuzz-triage` opened
every morning. The number it refused to write down was representable, and the
writer had a second spelling for it that fits; that makes it a **gap and not a
limit** (§16.10), so it was fixed rather than deferred. `75d954e` is the test
and the fix, `36b8d63` the doc comment, merged as `6939147`.

The arithmetic is the small part. The part worth the record is that a
diagnostic tool had no test of its own while the thing it diagnoses had 4709
inputs — and that a green sweep of all 4709 would still have missed the same
fault at the other end of the range.

### 105.1 What died, and on how many

```text
mise exec -- cargo run -q -p rcad-fuzz-explain -- where_two_surfaces_cross \
  fuzz/corpus/where_two_surfaces_cross/slow-unit-b9068e9ed0f147c54aec5b6b35af2f152303b761

thread 'main' panicked at crates/rcad-geom/tests/fuzzing/mod.rs:726:9:
9.332114885078756e-187 is past what one byte of exponent reaches
```

Exit 101. §101.5 met this one seed and read it as a known limit of the repro
writer. Sweeping the whole corpus on `b013ac4` says it was not one seed and it
was not a limit: **4709 seeds, 2361 exit 0, 2283 exit 1, 65 exit 101.** All 65
are in `where_two_surfaces_cross` and every other target had zero. The 65 carry
**21 distinct numbers**, smallest magnitude `-2.191809349008403e-193`, largest
`-1.4708981359843996e-185`. The 2283 that exit 1 are graceful decode refusals
of deliberate garbage, which is the tool working.

The same sweep after the merge: **4709 seeds, 2426 exit 0, 2283 exit 1, zero
panics.** 2361 + 65 = 2426, so exactly the 65 converted and nothing else moved.
`just gate` on the merge: **PASS, exit 0, 1892 tests** — 1877 before, plus five
new tests in each of the three test binaries that pull in the fuzzing module.

Those four counts are Mattias's own sweeps, run by his own script, and this
lane publishes them as given. Everything else below this lane derived itself
and says so where it does.

### 105.2 Wanting the largest whole number is a preference, never a requirement

A number goes into the seed as a whole number of thirty-one bits and one byte
of exponent, and the byte counts in steps of five — each step is a factor of
thirty-two. `written_as_numbers` picked the step that made the whole number as
large as it could, and then asserted that step fit in the byte:

```text
steps = ceil((|n|.log2() - 31) / 5)      # the largest whole number
if |n / 2^(5·steps)| >= 2^31 { steps += 1 }
assert!((-128..=127).contains(&steps))   # the old check
```

Wanting the largest whole number is a preference and never a requirement.
Dropping one step divides the whole number by thirty-two and names the
**identical** value, so wherever the preferred spelling ran off the end of the
byte, a smaller one did not. `9.332114885078756e-187` wants a step of `-129`
for a whole number of `136247104`; at `-128`, the smallest step there is, it is
a whole `4257722` and reads back bit for bit. The number was representable and
the writer could not spell it. Clamping the step into the byte is the whole of
the fix.

That is the shape §16.10 names a gap: the answer exists and the arithmetic
could not reach it. A limit gets deferred; a gap gets fixed.

### 105.3 Where the test was missing, which is the part worth the record

Nothing in the suite ever reached this writer with a number the decoder had
produced. The corpus replays run the other way — bytes in, geometry out — so
they exercise `a_number` and never `written_as_numbers`. §101.5 checked exactly
that and found it reassuring: the gate-covered regression test does not call
the helper, so `just gate` was unaffected. It was the wrong thing to be
reassured by. The gate being unaffected is the same fact as the helper having
no test.

So a seed that killed `just fuzz-triage` every morning sat in `fuzz/corpus`
with no red test anywhere in the tree. The tool that exists to explain a fuzz
failure was the one piece of the fuzzing code with nothing fuzzing it, and the
corpus it reads from had 4709 inputs.

The fix's test closes it the direct way: take every number the decoder can hand
back, write it down, read it back, and require the same bits. That is seven
whole numbers against all 256 exponent bytes, plus the two ends that must still
panic.

### 105.4 The blind spot, which a green sweep does not cover

`fuzz/corpus` holds only small magnitudes here, so the sweep of 4709 proves
nothing about the other end of the range. **The same fault exists at the top,
and no sweep of this corpus could have found it.** `i32::MIN` at exponent byte
`127` is `-3.061802069160839e+200`; it wants a step of `128`, and one byte
stops at `127`, where it is spelled as exactly `i32::MIN`. It took reasoning
about the encoding, not evidence from the night, and it is covered by a
constructed test rather than by a seed.

This lane re-derived the scale of both ends rather than taking it on the fix
lane's word — transcribing both versions of the arithmetic above into a
standalone `rustc` program under the scratchpad and running the grid `n = whole
· 2^(5·byte)` over `whole ∈ {1, -1, 2^31-1, -2^31, 3, 12345}` and all 256
exponent bytes:

```text
maximising step outside the byte: 21 of 1536
  1 at bytes -128..-123 (6)      -1 at bytes -128..-123 (6)
  3 at bytes -128..-124 (5)   12345 at bytes -128..-126 (3)
  -2147483648 at byte 127 (1)
```

Twenty at the bottom of the range and exactly one at the top — and that one is
`i32::MIN` alone, because `i32::MAX` is a hair under `2^31` and its step lands
inside the byte. A corpus of small numbers finds the twenty and can never find
the one.

That is §16.10's rule turned around. The section makes a claim of impossibility
show a demonstration; a measurement's blind spot deserves the same treatment,
because "4709 seeds, zero panics" reads as a clean bill and is not one. Name
what the measurement could not see.

### 105.5 The check that survived is stricter, not looser

A clamp that silently truncates would trade a panic for a wrong answer, which
is the wrong direction for this kernel. The new assert moves off the step and
onto the whole number the clamped step leaves, and tests it at **both** ends:

```rust
let whole = (number / 2f64.powi(steps * FIVES_PER_STEP)).round();
let spellable =
    (-2f64.powi(31)..2f64.powi(31)).contains(&whole) && (whole != 0.0) == (*number != 0.0);
assert!(spellable, "{number:e} is past what five bytes reach");
```

Too wide and the whole number will not fit in thirty-one bits; too fine and it
rounds away. The `(whole != 0.0) == (*number != 0.0)` half is the one that
earns its place: a number below half of `2^-640` clamps to step `-128`, rounds
to a whole `0`, and passes the range test — without that half the clamp turns a
panic into a number written down as zero. Transcribing the old and new writers
side by side, a third of `2^-640` panicked under the old assert (step `-134`,
outside the byte) and panics under the new one with `whole = 0`; with the clamp
and without the zero test it would have been written out as `0` and exited
cleanly.

NaN moved the same way, and it moved from silence to a crash. The old writer
cast NaN through `as i32`, which saturates to `0`, so a NaN went into the repro
line as the number zero and nothing said a word; the new check fails the range
test and panics. Infinity panicked under both writers — the fix lane's note
said it was new, and it was not — but under the old one it panicked on
`i32::MAX · 5` overflowing before the clamp ever ran, which is the arithmetic
and not the number, and a build with the overflow checks off wrapped instead and
blamed a `NaN`. Clamping the wanted step before the bump check multiplies it
sends infinity through the very check above: the step is `127`, `inf / 2^635` is
still `inf`, and that is not inside thirty-one bits — so every build now refuses
infinity for being too wide to spell, and says so naming the number. Crash over
corrupt, in the one place that had been quietly corrupting.

### 105.6 This does not overturn §101

§101 replayed 350 rescued artifacts against `04f6085` and promoted no seed,
because none of them was a live kernel bug. **That stands.** Every one of these
65 is the repro-line writer in `rcad-fuzz-explain`, not the kernel: the seeds
replay fine, and what died was the printing of the paste-into-a-test line
afterwards. `fuzz/corpus/` is still unchanged by either section — nothing here
was promoted, because all 65 were already checked in.

What §101 got wrong is narrower and is in §101.5: it read one panicking seed as
a known limit of the diagnostic tool and stopped. It was 65 seeds, and it was a
gap. The lesson is the one in §105.3 — "the gate does not cover it" is a reason
to look harder, not a reason to relax.

