## 114. The same oracle defect, one arm along, fixed 2026-09-17

§98 is the night of 2026-09-14 that crashed on its own oracle. This is that
night again, in the arm §98 did not touch. `fuzz-cones-and-tori` died on the
night of 2026-09-16 and again on the night of 2026-09-17, on two different
inputs that are one bug, and the kernel was right every time.

Three things are recorded here: the bug, why a fix made three days earlier did
not reach it, and what the guard is that would have caught both. The fourth
thing that went wrong that morning — the triage tool reporting both live crashes
as clean — is §91.5 and is only pointed at from here.

### 114.1 The pair, and why both answers were right

A cone of slope `7.8e28` with its apex at `3.6e103`, against a plane square to
its axis. At its own scale `intersect` names **one circle**. At `2^-1` it names
**a hyperbola in two branches**, and
`writing_a_pair_at_another_power_of_two_does_not_change_it` panicked.

Both answers are right, because the two questions were about two different
planes. The harness did not write the plane at half size; it rebuilt it:

```rust
let [a, b, c, d] = flat.coefficients();
Plane::from_coefficients([a, b, c, exactly_scaled(d, factor)?])
```

`Plane::from_coefficients` reduces and normalizes the vector it is handed, and
the vector it was handed was an already-normalized stored normal. That leaned
the normal **`2.8e-17`** off the cone's axis. A cone's own side leans `1/slope`
— `1.3e-29` — away from square, so the rebuilt plane was off by two thousand
billion times the tilt that separates one circle from two branches. It really
was the steeper of the two and it really did cut both nappes.

**A footgun and not a broken promise**, word for word as §98.2 says of
`Axis::new`. `from_coefficients` claims that two *exactly* proportional
coefficient sets give the same plane bit for bit, and that claim holds. A stored
normal is the rounded normalizing of whatever built it, so it lies exactly along
nothing, and the claim has nothing to say about it.

### 114.2 §98 fixed the type that crashed, not the pattern

Both arms of the scaling oracle were born in one commit — `9698f948`,
2026-09-04 — doing the same thing. §98's fix, `9c29cae0` on 2026-09-14, changed
`scaled_axis` and did not touch `scaled`'s plane arm. `git log -L` over the two
function bodies is the evidence: `scaled_axis`'s history reads `9c29cae0` then
`9698f948`; `scaled`'s reads `3c0ff09e`, a rustfmt, then `9698f948`, with no
`9c29cae0` in it at all.

§98.2's sentence was general and its patch was not: *a stored direction is the
rounded normalizing of whatever built it, so it lies exactly along nothing.*
That is true of a stored normal without changing a word. Three days and two
nights were spent on the difference between a sentence that generalizes and a
patch that does not.

So §98.3's closing line — **"It now compares the pair against the pair"** — was
true of the surfaces that carry an axis and false of planes on the day it was
written. It is corrected here rather than there.

### 114.3 The check §98.3 named and nobody built

§98.3 has the diagnostic in one line: *"`scaled(pair, 2⁰)` — a factor of one, no
scaling at all — already turned `Curves` into `Nothing`. That is the whole
diagnosis in one call."* It was used to find the axis bug by hand and never
turned into a test, so it was not standing there to find the plane one.

`scaling_any_surface_by_one_hands_back_the_bits_it_was_given` is that line as a
test, over all five arms of `scaled`, comparing `f64::to_bits` and not values.
Put back the old plane arm and it fails on the slanted plane it builds: offset
`2.22` comes back `2.2199999999999998`, one unit in the last place. That is the
same rebuild landing on the offset rather than on the normal — which of the two
moves is a property of the direction, and
`rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal` is where the
normal's share is counted.

**The oracle it sits beside could not have caught either bug.**
`writing_a_pair_at_another_power_of_two_does_not_change_it` reports that a
scaling changed the answer and has no way to say whether the scaling or the
rebuild changed it. That is why both nights read as kernel crashes, and why each
of them cost a session to find out otherwise.

### 114.4 `Plane::moved_to`, and what the corpus replays now

`Plane::moved_to(offset)` is the plane's half of `Axis::moved_to`: the stored
normal handed straight back, the offset scaled exactly, nothing left to round.
Multiplying a model by `2^k` multiplies a plane's offset and leaves its normal
alone, because a normal is a direction and not a length.

Its doc comment publishes **24 733 of 100 000** — how many random directions
come back with other bits when a plane is rebuilt from its own coefficients —
and `rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal` is the test
§15.6 wants beside a published number. Re-measured on this tree on 2026-09-17:
**24 733**, and `moved_to` moved **0** of the same 100 000.

Both crash artifacts are checked in under names rather than hashes:

```text
fuzz/corpus/where_a_cone_or_a_torus_crosses/a_plane_square_to_a_huge_cones_axis
fuzz/corpus/where_a_cone_or_a_torus_crosses/a_plane_square_to_a_flatter_huge_cones_axis
```

`every_seed_in_the_checked_in_corpus_names_its_crossing_honestly` replays them
through the same three oracles the fuzz target runs, so there is no bespoke test
beside the fixtures. **They are the first keepers a coverage-guided run has sent
back to this target**; every other seed in that directory was hand-written. The
file's corpus row moves with them, re-measured on this tree on 2026-09-17:

| row | pairs | points | orders | scalings |
|---|---|---|---|---|
| corpus, before | 32 of 36 seeds | 352 | 12 | 24 |
| corpus, after | **34 of 38 seeds** | **377** | **14** | **28** |

### 114.5 Nothing in `src/` does this, and one test does it on purpose

§98.2 said the same of axes, and the plane half is checked here rather than
assumed. Every `from_coefficients` call in the tree was read. All but one take
four numbers a caller wrote down: literals in a doc test or an assertion, or an
`[f64; 4]` parameter — `arrange.rs:4477` (`cut_by`, over a `&[[f64; 4]]`),
`point_in_solid.rs:517` (`meet_of_three_planes`) and
`a_piece_of_a_face_bounded_by_two_straight_sides.rs:281` are the three that take
theirs from a parameter, and not one of them is holding a `Plane` to get it.
`plane.rs:767` is the `TryFrom<[f64; 4]>` impl forwarding.

The one call that does rebuild a plane from its own coefficients is
`the_spellings_a_round_surface_still_has.rs:135`, which is the test that counts
how often that moves the normal. That is the intended and only instance.

### 114.6 The night that lost an hour and a half, left alone on purpose

`fuzz-cones-and-tori` runs last in `fuzz_all` on a 5744-second slice. It died 15
seconds into it on the 16th (`#129569`) and about 2 seconds into it on the 17th
(`#16678`) — **roughly 1h36m a night** of fuzzing bought and not spent, two
nights running.

Asked whether the night should survive its own crash, Mattias said no: fix the
bug, not the night. A target that carries on past a crash is a target whose
morning line has to be read twice, and the crash is the finding. With the bug
fixed, the next night is the measurement — if `fuzz-cones-and-tori` still loses
its slice, it is losing it to something else, and that is worth knowing on its
own rather than hidden behind a swallowed crash.

