## 96. A second fuzz format that cannot avoid a cone or a torus, closed 2026-09-14

*§51.3 named the gap: `two_surfaces_from` reads two bits per surface and asks
`which % 3`, so no byte string it has ever produced named a cone or a torus.
Widening it was measured and rejected in
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` — a repack moves
92 of the 830 checked-in seeds to a different kind. This closes the gap with a
second format instead, whose decoder cannot hand back a pair without a cone or
a torus in it, reusing the point, order and scaling oracles §51.3's format
already had.*

### 96.1 The invariant is in the decoder, not a filter over what it drew

`two_surfaces_with_a_cone_or_a_torus_from` reads one bit for which of its two
slots is forced, one more bit for whether that slot is a cone or a ring, and
the free slot from the remaining six bits as `% 5` across all five kinds. The
forced slot is built by the cone arm or the ring arm and nothing else, so there
is no byte string this function hands back a pair without one — every one of
the 32 seeds that decode and all 94 060 of the soak's decoding draws are
forced-slot cone or torus, checked rather than assumed.

`(kinds >> 2) % 5` over six bits is not uniform — 64 is not a multiple of 5, so
a plane, a ball, a pipe or a cone lands 13 times in 64 against a ring's 12.
Measured, not guessed: the soak's kind counts are cone 106 667, plane 22 950,
ball 22 908, pipe 22 678, ring 12 917 of 200 000 draws. This is the same shape
of bias §51.3 records for the first format's `which % 3`, left alone for the
same reason: a coverage-guided fuzzer explores by feedback and not by assuming
its own bytes are uniform, so what matters is that the bias is written down
rather than hidden.

The worst case is a torus paired with a torus: `1 + 2 × 8 × 5 = 81` bytes,
against the first format's 71. `MOST_BYTES_A_CONE_OR_TORUS_PAIR_READS` is the
new ceiling, and `justfile`'s `fuzz-cones-and-tori` passes it to `-max_len`
rather than reusing `fuzz`'s 71 — and `fuzz_all` now lists five recipes, not
four, so the overnight run reaches it too.

### 96.2 Nothing about the first format moved

The old corpus and its decoder are untouched:
`every_seed_in_the_checked_in_corpus_names_its_crossing_honestly` in
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` still reports
exactly what it reported before this landed — 830 seeds, 749 decoding,
`Asked { points: 4235, orders: 549, scalings: 1097 }` — and the 136-pair hand
table is unchanged at `Asked { points: 696, orders: 39, scalings: 78 }`.
Reproduced here on the merged tree, not restated from before. The new format
lives beside both, in its own corpus directory
(`fuzz/corpus/where_a_cone_or_a_torus_crosses/`), its own fuzz target
(`where_a_cone_or_a_torus_crosses`), and its own replay test
(`every_cone_or_torus_pair_the_fuzzer_draws_is_named_honestly.rs`).

### 96.3 What the 36 hand-written seeds cover

Every replay counts its buckets and names the one that goes missing, rather
than checking only a total:

| What | Coverage |
|---|---|
| Kind pairs (all nine that have a cone or a torus in them) | ball+cone 3, ball+ring 4, cone+cone 2, cone+pipe 7, cone+plane 4, cone+ring 2, pipe+ring 4, plane+ring 4, ring+ring 2 |
| Exponent bands | tiny 4 (down to `2^-608`), ordinary 24, huge 4 (up to `2^632`) |
| Pairs sharing one axis line | 10 |

32 of the 36 decode; the other four are the decoder's own refusals, the same
as any byte string the first format also turns away — nothing here repairs a
number into an acceptable one.

### 96.4 The numbers

Measured on the merged tree by running
`every_cone_or_torus_pair_the_fuzzer_draws_is_named_honestly`'s two tests
directly — the dispatcher's own run, not the lane's report restated:

| Row | Corpus | Decoded | `Asked` |
|---|---|---|---|
| Corpus | 36 seeds | 32 | `{ points: 352, orders: 12, scalings: 24 }` |
| Soak | 200 000 draws | 94 060 | `{ points: 830, orders: 11 435, scalings: 22 770 }` |

Floors are set at roughly three quarters of these, the same convention the
first format's replay test documents for its own floors.

### 96.5 A saturating cast found while authoring the seeds

`bytes_for`, the encoder used to author the first format's seeds by hand, wrote
the wrong exponent for any number whose `log2` is `31 + 5k`: the whole number
landed on exactly `2^31`, and the cast to `i32` saturated one step low. A
radius asked for as `2.0` came back as `1.9999999990686774`, against the
function's own doc comment promising bit-for-bit round-tripping. Fixed by
sharing the number-writing loop with the new format's own encoder,
`bytes_for_a_cone_or_torus_pair`, and covered by a property that sweeps every
power of two from `2^-120` to `2^120` through it.

`bytes_for` has no callers anywhere in the tree — grepped, not assumed — so no
checked-in seed was ever produced by the broken path and none needed
re-minimising. The 830 seeds are bytes already on disk; nothing regenerates
them from this function.

