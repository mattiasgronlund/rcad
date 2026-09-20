## 181. A pipe on a fat pipe's skin refuses one way round when the pair sits far from the origin, found 2026-09-20

*The night of 2026-09-20 wrote four crashes that are not carry-over and still
fail on `main`. All four are §140's geometry — a pipe lying on a fat pipe's skin
— **moved away from the origin**, which is the one thing §140's pair was not.
Asked one way the pair names two straight runs, asked the other it refuses with
`CannotNameWhichPieceOfACrossingThisIs`. **Not fixed, and not a lane yet.** This
section is the reproducer, the mechanism, two fixes that were written and
reverted because they are provably no-ops here, and the decision the next
session has to take first.*

### 181.1 The four are last night's, and they are one case

`crashed:4` in `~/rcad-fuzz-logs/latest` is four libFuzzer deadly signals in
`fuzz.log`, and the `crash:` counter stepping `1 → 2 → 3 → 4` through the night
is where the number comes from. Dating is off the **live** worktree
`rcad-nightly-fuzz/fuzz/artifacts`, never off the night's copy, which restamps
everything it carries over (§91):

| artifact | target | first seen |
|---|---|---|
| `crash-cba396f8` | `where_two_surfaces_cross` | 2026-09-12 |
| `crash-d04e0041` | `where_two_surfaces_cross` | 2026-09-14 |
| `crash-30debd29` | `where_a_cone_or_a_torus_crosses` | 2026-09-16 |
| `crash-89994c6f` | `where_a_cone_or_a_torus_crosses` | 2026-09-17 |
| `crash-6cf9b428` | `where_two_surfaces_cross` | 2026-09-19 |
| **`crash-0f06237e`** | `where_two_surfaces_cross` | **2026-09-20 00:17** |
| **`crash-477e4468`** | `where_two_surfaces_cross` | **2026-09-20 02:08** |
| **`crash-87e70e7f`** | `where_two_surfaces_cross` | **2026-09-20 02:31** |
| **`crash-b76d3903`** | `where_two_surfaces_cross` | **2026-09-20 04:47** |

Replayed against `main` at `ae124a29`, **the five older ones all succeed** —
including `crash-6cf9b428`, which is §140's own pair, closed by §143.9. The four
new ones all panic in
`naming_a_pair_does_not_depend_on_which_way_round_it_is_asked`.

The other 766 artifacts of the night's 775 are `timeout-*` and `slow-unit-*`,
not crashes. `just fuzz-triage` decodes every file under the night, which is 775
`cargo run`s; decoding the nine `crash-*` by name is one call each and is what
this section was written from.

### 181.2 The pair, and the reproducer

All four are two cylinders with **parallel axes** along `[1, 0, 0]`, radius
`5.047645337156084e153` against `1.984375`, and the thin axis offset from the
fat one by exactly the fat radius in `y`. `1.984375` is `127/64`, so `r₂²` is
exact and §143.9's `Exact::times` floor is not in play.

```rust
every_oracle_holds_of(two_surfaces_from(&bytes_for([2, 2], &[
    100663296.0, 0.0, 3.705489715806406e21, 127.0, 0.0, 0.0,
    5.047645337156084e153,
    5.047645337156084e153, 5.047645337156084e153, 41581179.0,
    167772415.0, 0.0, 0.0, 1.984375,
])).unwrap())
```

**What separates the four is one number, and it is not a radius.** Each is
§140's pair with a large displacement written into an axis origin, and nothing
else moves:

| artifact | fat origin `z` | thin origin `z` |
|---|---|---|
| `crash-0f06237e` | `3.705489715806406e21` | `41581179.0` |
| `crash-477e4468` | `7.47717270846756e66` | `41581179.0` |
| `crash-b76d3903` | `-5.115950638227784e18` | `41581179.0` |
| `crash-87e70e7f` | `-1048576.0` | `1.5065044164928034e59` |

§140's pair has both at `(0,0,0)` and `(R₁, R₁, Z)`. That is the whole
difference, and §181.4 is why it is the whole difference.

### 181.3 The answer exists, and the refusal is not a knife edge

The true axis separation is `d = √(R₁² + Δz²) = R₁ + Δz²/(2R₁)`. For the largest
`Δz` of the four that excess is `1.4e-111`, against `r₂ = 1.984375`. So `d − R₁`
is smaller than `r₂` by 111 decades and the thin pipe cuts the fat pipe's skin
in two endless straight lines — not tangentially, not marginally. `Exact` holds
`E = A − W₁(r₁² + r₂²)` as `1.3730654033946731e43`, which is `Δz² − r₂²` and is
the same in both orders.

So this is §16.10's **gap** and not a limit, and unlike §140.5 that does not rest
on the inference "the other order answered, so the answer exists". It rests on
the geometry being decided by 111 decades in numbers `Exact` holds without
cancelling.

### 181.4 Where it is actually lost, measured

Instrumented in `two_lines_told_apart`, the two orders build **the same plane**
— same offset magnitude, normals negated — and differ only in what they can read
back off it:

```
fat first:  normal=[-0.0, 3.705489715806365e21, 5.047645337156084e153]
            from=[100663296.0, 0.0, 3.705489715806406e21]
            offset=1.870399788587003e175   off=[-0.0, 0.0]
            leaning=[-0.0, 0.0]            either=[6.703903964971294e153, inf]

thin first: normal=[0.0, -3.705489715806365e21, -5.047645337156084e153]
            from=[5.047645337156084e153, 5.047645337156084e153, 41581179.0]
            offset=-1.870399788587003e175  off=[-8.198738891782513e158, ...]
            leaning=[-0.0, 0.0]            either=[6.703903964971294e153, inf]
```

`off_the_picker` is `normal · from − offset`. Asked the fat pipe first that dot
product has **one** non-zero term, because `from.y` is `0` and `normal.x` is
`-0`, so it lands on an `f64` exactly and the residue is zero. Asked the thin
pipe first it has **two**, `-1.87e175` and `-2.10e161`, and the smaller one does
not survive the sum: what is left is `-8.198738891782513e158`, about half an ulp
of the larger. The check needs the two lines either side of the plane, and they
stand `6.703903964971294e153` apart in these units — **122 000 times finer than
the residue**. So the refusal is right about the plane it was given.

**In distance:** the plane sits `|offset| / ‖normal‖ = 3.705e21` from the origin.
`ulp(1.870399788587003e175)` is `2⁵³⁰ ≈ 3.5e159`, so the plane can only be placed
on a grid of `3.5e159 / 5.05e153 ≈ 6.9e5`, while the two lines it must separate
stand `2·r₂ = 3.97` apart. §140's pair escapes this entirely: its plane passes
**through the origin** — `normal · anchor = −Z·R₁ + R₁·Z = 0` exactly — so the
offset is `0`, which has no rounding to lose.

That is the finding. **This family is not about radius ratio, which §140 and
§143 are; it is about how far from the origin the pair sits.** A ratio sweep
will not find it and did not.

### 181.5 Two fixes written and reverted, and why that is a result

The obvious readings both turn out to be wrong here, and both cost a build, so
they are written down rather than left for the next session to re-derive.

**`across`'s numerator cancels, and it does not matter.** `across = (δ² + r₁² −
r₂²) / 2δ` is computed in plain interval arithmetic with no exact arm, and asked
the thin pipe first the numerator comes back `[-0.0, 1.4968802321510399e292]`
where the true value is `1.373e43` — 249 decades too wide, with `how_far`
straddling zero. Giving it an exact arm at `A`'s scale, in §143.9's
`map_or_else(…, Exact::enclosure)` shape, fixes it exactly:
`[1.3601048723524991e-111, 1.3601048723525e-111]`. **The pair still refuses.**

**`between_the_two_lines`'s anchor cancels too, and that does not matter
either.** It computes `(δ² + r₁² − r₂²)/(2δ²)` in plain `f64`, where `δ²` and
`r₂²` are two independently rounded numbers at `2.5479e307`; their difference is
±1 ulp ≈ `4e291` instead of `1.373e43`, throwing the fraction to `±7.8e-17` where
the truth is `2.69e-265`. Feeding it the exact numerator gives
`2.694533354672661e-265`. **The pair still refuses.**

**Both are no-ops for this geometry, and the reason is one line.** `leaning` is
`normal · sideways`, and the normal is exactly square to `towards` here, so
`leaning` is `[-0.0, 0.0]`. `middle = off_the_picker + leaning · how_far` then
never reads `how_far` at all, and the anchor only ever moves along `towards`,
which does not move the plane. Every `f64` of `across` is discarded before it can
matter. So neither change has a test that fails without it and passes with it,
and under CLAUDE.md's second principle neither lands. They are still real
cancellations, and a pair with a **leaning** direction — where
`HalfSpace::through_a_point_square_to` takes a fallback normal and `leaning` is a
real number rather than a rounding — would feel them. Nobody has drawn one.

**One claim made along the way was wrong and is corrected here rather than
quietly dropped.** Building the two lines as `f64` triples and reading the
picker at them gave two same-sign readings, which looks like the answering order
certifying a separation it has not got — §140.5's worse branch. It is not
evidence: the kernel never materialises those points, and the `towards · how_far`
product rounds away `3e137` of `y`, which is what produced the readings. Taken
the way the kernel takes it — `±out · (normal · up)` — fat-first reads
`±1.0e154`, opposite signs, genuinely separated. §140.5's question stays open.

### 181.6 Limit or gap, and what the next session decides first

**It is a gap, because one family of fixes is not excluded.** §16.10 wants the
family named before anything is called a limit, so:

- **Rescale the normal.** Offset and its ulp scale together; the ratio that
  matters is invariant. Excluded.
- **Move the anchor along `along` or `towards`.** The normal is exactly square to
  both — `Exact::dot_of` returns `Zero` for the first and `[-0.0, 0.0]` for the
  second — so the offset does not change at all. Excluded.
- **Move the anchor along `up`, inside the slab.** The whole ±`r₂` of slack moves
  `normal · anchor` by `1.0e154`, about `3e-6` of one ulp of the offset. It
  cannot move the stored offset by a single step. Excluded.
- **Anchor the half-space at a point rather than at the origin** — carry the
  point it was built through and read `normal · (x − p)`, so a point near `p`
  never forms `1.87e175` at all. **Not excluded, and it would work.**

So the answer exists and the arithmetic could reach it with a representation that
this kernel does not currently have. That is a gap deferred for its size, not a
limit, and §13 does not apply — CLAUDE.md says in terms that §13 governs
features and never correctness gaps, which is §140.7's reasoning and it carries
over unchanged.

**What the lane decides before it writes anything:** making both orders *answer*
is not obviously the safe direction. The returned `Picker` carries that same
coarse offset downstream, so a fix that only teaches `off_the_picker` to read
`normal · (from − anchor)` would make both orders answer while leaving every
consumer the plane that cannot resolve the slab. Making both orders *refuse* is
safe under "crash > corrupt" and is the smaller change. Which of those is right
turns on whether any consumer of a `Picker` at these coordinates can tell the two
runs apart, and **that is a question about `walk.rs` and the fuse, not about
`intersect.rs`** — nothing in this section measured it.

### 181.7 What this owes

A lane, on §140.7's reasoning and not a fresh one. §0 needs no edit: this is a
defect against the current unit, not a roadmap item, and §45.2 governs the count.

If §39.4 gains a row for it, the row is a **gap**, `CannotNameWhichPieceOfA`
`CrossingThisIs` from `two_lines_told_apart`, demonstrated by §181.2's seed, and
distinguished from §140's row by the pair's distance from the origin rather than
by its radius ratio.

Commands this section was written from: `just decision 140`, `140.5`, `140.7`,
`143.9`, `93`, `150`; `just src two_lines_between_two_pipes`,
`how_deeply_two_side_by_side_pipes_overlap_enclosing`, `how_far_off_an_axis_exactly`,
`between_the_two_lines`, `two_lines_told_apart`; `cargo run -p rcad-fuzz-explain`
on each of the nine `crash-*` artifacts against `ae124a29`; and temporary
`eprintln!` instrumentation in `intersect.rs`, reverted.
