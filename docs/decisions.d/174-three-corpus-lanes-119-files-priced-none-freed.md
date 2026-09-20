## 174. Three corpus lanes, 119 files priced, none freed — the top rows are one queue

*§171 priced the corpus and §171.7 said to cut lanes from the top of its table.
Three ran on 2026-09-20: the corner off a curved face (75 files), both coedges
the same way round (26), and triangulating a corner on a departing side (18).
Two found real defects and fixed them. The corpus imported **115 files before
and 115 after**. Every file freed of its named blocker met another one standing
directly behind it, and in the largest case that second blocker is a limit. This
section records the three findings, the measurement that ties them together, and
what it does to §171's pricing — which was never wrong, but was read as a
forecast by the session that wrote it, this one included.*

### 174.1 What ran

Four lanes, all branched from `16217f40`, all reporting both the sha they landed
on and the sha they checked out (§-the-delegating-skill's rule, and it cost
nothing here because every one of them landed right).

| lane | §171.2 row | verdict | code changed |
|---|---|---|---|
| corner off a curved face | 75 | **limit** | none |
| both coedges the same way round | 26 | **gap**, fixed | `rcad-step/src/brep.rs`, 31 lines |
| triangulating, loops touch | 18 | **deferred feature**, closed | `rcad-mesh/src/triangulate.rs`, 119 lines |
| the unheld-geometry family | — | measured | `examples/blocker_depth_probe.rs`, +121 |

The write-set table was built first, by a read-only lookup, and it earned its
keep twice. It found that `crates/rcad-step/tests/how_often_a_real_file_imports.rs`
is far more than the `KICAD_FILES_THAT_IMPORT` pin §171.7 forbids: it also holds
a `grouped()` function that is a **second, independent copy** of `import.rs`'s
`kind_of` match over every `Refusal` variant, the census test, and four
unrelated tests. Forbidding the file outright — rather than the pin — is what
kept three lanes off one another. It also found that `brep.rs` was wanted by two
lanes at once, which is why each brief names the *functions* it owns rather than
the file; in the event the collision never happened, because the lane that would
have shared it changed no code at all.

### 174.2 The measurement: 115 before, 115 after

Re-derived on the merged tree, not taken from any lane's report:

```
RCAD_STEP_CORPUS=… cargo run -q --release --example blocker_depth_probe
```

| row of §171.2's table | then | now |
|---|---|---|
| **files that import** | **115** | **115** |
| a corner is not on the curved face the file says it is on | 75 | **98** |
| both coedges of the edge at `<N>` run the same way round | 26 | **1** |
| triangulating: the corner at `<N>` sits on the side leaving `<N>` | 18 | **0** |
| triangulating: the boundary crosses itself | 0 | **18** |
| a face walks a closed edge against the way its surface winds | 1 | **3** |

Two fixes landed, one refusal vanished from the table entirely, 25 files left
the row they were in — and the number of files this kernel can read did not
move by one.

### 174.3 The coedge fix: a tie-break that was wrong every time it fired

`Walk::which_way_round_a_closed_edge_is_stored` ended in

```rust
Ok(Some(travel.dot(winding) >= 0.0))
```

`travel` is this use of a closed edge's direction in space; `winding` is the
carrying surface's first coordinate direction. **The two uses of one closed edge
travel exact opposite ways**, so the dot normally separates them by itself. When
it is exactly zero it does not — `-0.0 >= 0.0` is also true — so both uses answer
`Orientation::Forward`, and `SolidBuilder` refuses the whole solid by
`ADraftEdgeIsWalkedTheSameWayTwice`. Since one of two uses *has* to be the other
way, that tie-break is provably wrong whenever it fires.

It fires on a bend of pipe capped by a plane, and not by accident: a torus's
first coordinate runs round its **tube** while the cap circle runs round its
**ring**. At `SW_PUSH_6mm.step`'s corner the two are perpendicular by
construction —

```
corner - centre = (1.65, 0, 0.1)   radial 1.65 = R,  0.1 = r  -> top of the tube
winding  = (1, 0, 0)               round the tube is radial there
travel   = (0, ±1.65, 0)           round the ring
dot      = 0.0 exactly, for both uses
```

Zero now returns `None`, and the reader falls back to `ORIENTED_EDGE`'s own
flag, which answers a different question (§115) but does at least tell the two
uses apart; `Model::validate` asks the certified question moments later and
refuses by `AClosedEdgeIsWalkedAgainstItsSurface` when that luck runs out. The
invariant in `check_every_edge_is_walked_once_each_way` is untouched and
unweakened — **the check was right, the input to it was wrong.**

The brief for this lane carried a hypothesis, that the bug was in reading
`ORIENTED_EDGE`'s sense flag. The lane walked the file's entities by hand and
**disconfirmed it**: the flag is read correctly and the override downstream is
what loses it. Recorded because a lane that reports back the brief's own guess,
confirmed, is the thing §-core-principle-5 exists to prevent, and this one did
the opposite.

### 174.4 The corner off a curved face is a limit, drawn from the bytes

The corpus's largest blocker cannot be fixed. `kicad/LED_D3.0mm.step` puts the
corner `#118` at `(-0.23, -1.16619037897, 3.)` on `#140`, a cylinder of radius
`1.9` about `(1.27, 0., 3.)` whose axis is exactly `-z`. The corner is exactly
`1.5` from that axis, so the `y` the cylinder asks for is `√(1.9² − 1.5²)`.
Exact decimal arithmetic on the file's own text:

```
radial distance from the file's own text : 1.90000000000057689954234201768…
radius from the file's own text          : 1.9
miss                                     : 5.768995e-13
y the cylinder asks for                  : 1.166190378969060094174830…
y the file actually wrote                : 1.16619037897        (twelve digits)
```

The kernel's refusal reports `off_by = 5.768986e-13`; the file's own text says
`5.7689954e-13`. They agree to six digits. **The exporter truncated to twelve
significant figures.** The furthest a corner of this cylinder written as three
`f64` can land off it is `NEAREST_F64_POINT_ROUNDOFF · L = 6.7e-16`, and
`how_far_off_a_curved_surface_the_nearest_f64_can_sit.rs` is the search that
says so, so this corner is **866 times further off than any rounding could put
it**. The answer is not among the file's numbers. No filter, no exact level and
no rescale reaches a corner that was never written down — the family of fixes
was enumerated and each ruled out individually, per §16.10, including the one
that matters: the axis needs no normalising, so "we built the surface
inaccurately" is not available either.

It is the family and not an anecdote: of the 76 rows measured, **75 are 56
coordinate spacings or more off, out to 20 777**, where one spacing is the whole
of what a rounding can cost.

The only thing that would move these files is tagging an imported corner with
slop — §42.9's option C — and that remains one decision about what an imported
corner's slop *is*, not a tolerance to be reached for in a lane. It is not taken
here.

### 174.5 What is behind each wall

Both fixes moved their files onto another blocker, and the two successors are
different in kind.

**The 23 from the coedge fix** now stop at the corner off a curved face — the
row §174.4 has just shown to be a limit. **This section does not conclude that
those 23 are therefore dead**, and neither should the next reader. §174.4's
measurement covers the 75 files whose *first* refusal was already this one;
these 23 were hidden behind the coedge bug and have never been measured. Whether
their misses are short-written decimals or genuine roundings is **open**, and it
is now a cheap question, because the probe reaches them. Assuming it inherits is
exactly the unearned assumption the core principles forbid.

**The 18 from the triangulation fix** stop at "the boundary crosses itself",
which is not a triangulation defect at all. The face is a circle of radius `1.5`
walked into **three corners** — 180°, 270°, 0° — and two rectangular pin holes
reach past the resulting chord triangle, so the boundary genuinely crosses
itself *in the walked domain* while the real circle contains both holes
comfortably. A sagitta of 0.75 mm on that circle is the whole of it. That is the
edge walk being far too coarse, in `walk_a_wire_round`
(`crates/rcad-topo/src/mesh.rs`), and it is a deliverable of its own. It now
prices at 18 files, having inherited this lane's entire slice.

The triangulation refusal itself was neither a limit nor an arithmetic gap but a
**deferred feature whose deferral expired**: `refuse_loops_that_touch`'s own doc
comment said it was "stricter than it strictly has to be" and that "nothing in
this milestone needs a hole that touches". The consumer turned up. The line now
sits at "a corner both loops name", and it sits no further for a reason that was
demonstrated rather than argued — allowing a corner in the *middle* of another
loop's side made `check_everything` report a boundary side that is no edge of
any triangle, because the triangulator splits that side at the foreign vertex. A
silent split there is a crack in the neighbouring face, which is §25.1.

### 174.6 A published factor that did not survive its own bound

The corner-off-a-curved-face lane shipped a test pinning §174.4's demonstration,
and it computed its rounding budget as

```rust
let a_rounding_of_that_coordinate = asked_for * NEAREST_F64_POINT_ROUNDOFF;
assert!(short_by > 2000.0 * a_rounding_of_that_coordinate, …);
```

`asked_for` is the `y`, `1.166`. But that constant's own doc comment defines it
*per unit of the point's largest coordinate*, and this point is
`(-0.23, -1.16619…, 3.)` — the largest is **3.0**. Taking the `y` understates
the budget by 2.6×, and the assertion's 2000× margin holds only against the
understated one: at the honest `6.7e-16` the `y` is 1411× out, and the assertion
would have **failed**. The finding is untouched and still overwhelming; only the
published factor moves, to 866× on the quantity the refusal actually reports.

**A green gate would never have caught this, because the test passes.** It was
caught by the dispatcher re-deriving the lane's numbers before the merge, which
is the one step in the delegating skill that exists for exactly this and is the
one most easily skipped. It is the same shape as §77.6's three red runs: a check
passing for a property of the thing it was handed rather than of the thing it
claims about.

A second, smaller instance from the same lane, not fixed and recorded here so it
is not rediscovered: `how_far_off` **over-reports at the bottom end**. On
`SW_Slide_SPDT_Angled_…`'s corner the true radial miss is `5.55e-17` and
`how_far_off` returns `8.9e-16`, sixteen times larger — the chart round-trip's
own rounding at that magnitude. Harmless for the 75, whose misses are three to
five orders above the noise floor, but it is the number `refusal.rs` publishes
as fact and the number §44.6 uses to split a gap from a limit, **so that split
is unreliable below about one coordinate spacing.**

### 174.7 The unheld-geometry family, priced

§171.3 recorded that an earlier draft had published "68 files, one lane" and that
the number did not exist, because co-occurrence is pairwise and is not a union,
and that anyone wanting it must ask the probe. Asked, on `16217f40`:

| set | kicad | nist | combined |
|---|---|---|---|
| S1 = `B_SPLINE_CURVE_WITH_KNOTS` | 9 | 8 | **17** |
| S2 = S1 + `B_SPLINE_SURFACE_WITH_KNOTS` | 23 | 14 | **37** |
| S3 = S2 + `SURFACE_OF_REVOLUTION` | 47 | 14 | **61** |
| S4 = S3 + `BOUNDED_SURFACE` | 49 | 18 | **67** |
| S5 = S4 + `VERTEX_LOOP` | 49 | 28 | **77** |
| S6 = the whole family | 53 | 29 | **82** |

A file counts only when **every** reason and fatal it holds falls inside the set
— a union, not a co-occurrence. The instrument is calibrated: the probe's
deliverables 1–5 are untouched and reproduce §171.2's published table exactly,
and the arithmetic closes, 82 + 169 + 1 = 252 = 367 − 115.

Three reason strings appear that §171.2 never listed, because they never appear
*alone*: `BOUNDED_CURVE`, `SEAM_CURVE`, `SURFACE_OF_LINEAR_EXTRUSION`. That is
why S6 exceeds "S5 plus what §171.2 named".

**S2 is the shape worth having**, at 37 files for two new geometry types —
larger than the coedge row this batch just cleared. S3–S6 buy another 45 for
five more type implementations. And 82 is a **ceiling**: 170 files still hold
something outside the family after all of it is done. Given §174.2, a ceiling is
what these numbers should be read as and nothing more.

### 174.8 What this does to §171's pricing

§171.2 said, in its own words, that it "prices a fix from above and is not a
forecast", and §45.2 has said the same thing for longer. Neither was wrong. What
was missing is **how far above**, and for the top three rows the answer turns out
to be *all the way*: 119 files priced, 0 freed.

So the correction is not to the arithmetic but to the reading. §171.4's rule —
two items are concurrent when no file holds both — is a statement about what can
be worked on at once, and this session took it for a statement about what can be
*delivered* at once. It is not. Two reasons that never share a file today can
still be the same queue, one standing behind the other, and the probe cannot see
that: it reports the first refusal, so a second wall in the same file is
invisible until the first comes down. The 18 triangulation files were being
blocked twice under **one reason string**, in two different faces.

What follows from that, and is the practical half of this section:

- **A row's file count is an upper bound on what clearing it buys, and the bound
  is not tight.** Quote it as a ceiling or not at all.
- **The only number that settles it is the import count**, before and after, from
  the same command. Three lanes reported honestly that they had freed nothing;
  a lane reporting its row had gone from 26 to 1 would have been telling the
  truth and saying nothing.
- **Running the top of the table in parallel is still right** — it is how we
  learned this in one afternoon rather than three — but the portfolio's expected
  yield is not the sum of its rows, and nothing should be planned as though it
  were.

### 174.9 What is left

- **The 23 files now sitting on the corner-off-a-curved-face row have not been
  measured.** §174.5. One probe answers it.
- **The coarse edge walk**, `walk_a_wire_round`, 18 files. §174.5.
- **S2 of the unheld-geometry family**, 37 files, two geometry types. §174.7.
- **`how_far_off` over-reports below one spacing**, which makes §44.6's gap/limit
  split unreliable there. §174.6. No consumer today; recorded rather than fixed.
- **One file still refuses same-way-round.**
  `Relay_SPST_Panasonic_ADW11.step`'s two uses of `#586` compose to the same
  direction in the file itself. It may be a malformed export; that was not
  proved and is not claimed.
- **Two NIST files moved to "a face walks a closed edge against the way its
  surface winds."** Doing better needs `which_way_a_closed_crossing_turns` with
  the two faces' `outward` gradients, which are `pub(crate)` in `rcad-topo`.
  §23.8 forbids spelling them a second time in `rcad-step`, so the work is to
  export the rule from `rcad-geom`, not to copy it.
- **A soak.** Triangulation changed, and the property suites search randomly, so
  one green gate is weak evidence for the mesh and fuse invariants.

Write set: this section, `docs/decisions.md` §0, and the merge of the four lane
branches.

