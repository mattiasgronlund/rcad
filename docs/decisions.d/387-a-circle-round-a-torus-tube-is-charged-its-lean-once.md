## 387. A circle round a torus's tube is charged its lean once — 2026-10-03

*Queue row Q563 (refuses/A, §229.5, §15.6), off `c82e221f`. One factor in
`how_far_off_the_circle_can_be`'s torus arm moved from `2` to `√2`; this
section is the proof §15.6 asks of a published bound, the search behind it,
and what the change moved.*

### 387.1 The defect, and the file it refused

The reader holds a file's `CIRCLE` against each face along its edge
(§229.4). For a circle round a torus's tube it bounds how far the circle is
from the tube circle it is nearest — centre `m` on the ring, axis `t` the
way round the ring there, radius `ρ_t` — as `|c − m| + |r − ρ_t| +
2·r·sin φ`, with `sin φ = |w × t|`. The last term is what a turn from `w`
onto `t` costs a point, and a turn by `φ` moves a unit vector at most
`2·sin(φ/2)`, which for a small turn is `sin φ`, not `2·sin φ`: the lean was
counted twice, as Q382's probe read it.

`kicad/Relay_1-Form-C_Schrack-RYII_RM3.2mm.step` closes each of its two pin
bends with such a circle: circle `#1375` of edge `#1374` on torus `#1558`,
and circle `#1568` of edge `#1567` on torus `#1588`, each of the tube's
radius `0.25`, each turned `1.04e-13` off its tube circle. At `2` they read
`19.43` and `19.31` roundings of `12.05` against the sixteen allowed, and the
file was refused `what #15 describes is not a solid this kernel can hold`. At
`√2` they read `13.74` and `13.65`, and the file imports: Q667 had already
fixed the turned-over torus of §229.5, which Q382's probe found it meeting
next, and nothing else stops it. It also round-trips: `165 / 165 imported KiCad files
round-tripped to the same solid`.

### 387.2 The proof, for every angle the arm can be handed

Write `X = c + r·e` for a point of the circle, `e` a unit vector square to
`w`. Pick the sign `s` with `s·(w·t) ≥ 0` and let `Q` be the turn about
`w × t` by the angle `φ` from `w` to `s·t`; `Q` is nothing when `w × t` is
nothing, since then `w = s·t`. `Q·e` is a unit vector square to `t`, so
`Y = m + ρ_t·Q·e` is on the tube circle, which lies on the torus, and

    |X − Y| ≤ |c − m| + r·|e − Q·e| + |r − ρ_t|.

A turn by `φ` moves a unit vector by `2·sin(φ/2)` times the length of its
part square to the turn's axis, so `|e − Q·e| ≤ 2·sin(φ/2)`. Since
`0 ≤ φ ≤ 90°`, `cos(φ/2) ≥ 1/√2`, and

    2·sin(φ/2) = sin φ / cos(φ/2) ≤ √2·sin φ,   sin φ = |w × t|.

**Past a right angle.** The arm reads `|w × t|`, which is the same for a turn
`ψ` and for `180° − ψ`. The proof never meets `ψ > 90°`, because a circle is
the same points whichever way its axis points and so is the tube circle: it
turns `w` onto `−t` instead. For the directed angle the inequality is false
— at `ψ` near `180°`, `2·sin(ψ/2)` is near `2` and `√2·sin ψ` near `0` — so
the choice of `s` is the whole of the argument there. Nothing upstream rules
`w·t < 0` out: `t` is `up × outward`, fixed by the torus's axis, and `w` is
the file's direction as written. **Both circles of 387.1 are handed in past
a right angle**: `#1375` turns about `(−1, −1.04e-13, 0)` where `t = (1, 0, 0)`,
and `#1568` about `(1.03e-13, −1, 0)` where `t = (0, 1, 0)`. That file is the
demonstration that the arm is asked there, and its import is the
demonstration that the bound holds there.

**Why `√2` and nothing smaller.** At a right angle the inequality is an
equality, and the triangle it bounds is reached: with `c = m`, `r = ρ_t` and
`w` turned a right angle about the torus's axis, the point `m + r·t` of the
circle is `√2·r` from every point of the tube circle.

### 387.3 The search (§15.6)

`a_circle_round_the_tube_turned_by_any_angle_is_within_its_bound`, in
`brep.rs`: the tube circle at the outside of a torus of ring `5` and tube
`1.5`, turned by every `5°` from `0°` to `180°` about the torus's axis, about
a direction square to `t`, and about one with a part along `t`; 111 circles,
720 points each, every point's distance from the tube circle and from the
torus held against the bound plus four roundings. Of each axis's 37 turns,
18 are past a right angle. Its control asks that some point come within `1e-9` of
`√2·r·sin φ` off the tube circle, and one does, at the right angle.

Run with the factor at `1.414`, about one part in 6 600 under `√2`, it fails at
that point: `a circle turned 1.5707963267948966 about … has a point
2.1213203435596424 off the tube circle, past its bound 2.121`. Command:
`mise exec -- just t -p rcad-step --lib -E 'test(/tube/)'`.

`the_relays_two_circles_round_their_tubes_are_inside_the_allowance` holds the
relay's two circles from the file's decimals at `13.74` and `13.65`, and
`tests/a_circle_leaning_off_a_torus_counted_once.rs` imports the file and
checks it still carries those decimals; on `c82e221f` it is refused.

### 387.4 What it moved

- **KiCad imports `164` → `165`.** Two pins outside this lane's write set
  moved with it, and the merge re-pinned both: `how_often_a_real_file_imports`
  (`KICAD_FILES_THAT_IMPORT` names the relay) and
  `every_kicad_file_that_imports_round_trips_to_the_same_solid` (its count
  pin reads `165`, and its pinned list names the relay).
- **`A_CIRCLE_MAY_MISS_ITS_FACE_BY` is closer to the corpus than it said.**
  Its doc comment called `1.20` roundings, measured on `b3701c4c` when only a
  ball and a pipe had a bound, the furthest any imported file needs. The
  relay now imports needing `13.74`, so sixteen clears a real file by less
  than a sixth, and an allowance under `13.74` would refuse it. The comment
  says so now. §195's "the worst circle in any file that imports is 1.20" is
  a dated record and stays as written.

### 387.5 What this leaves

- Measure the furthest any circle sits off its face, in these units, over
  every file that imports, now that the cone and the torus have bounds: the
  only figure for the whole corpus is `b3701c4c`'s, from before them.

**2026-10-12, Q812:** Measured.
`the_furthest_a_circle_sits_off_its_face_over_the_whole_corpus` in `brep.rs`
(`mise exec -- just t the_furthest_a_circle_sits_off_its_face_over_the_whole_corpus`)
sweeps every `CIRCLE` that bounds an `ADVANCED_FACE`, read straight off the
file rather than through `A_CIRCLE_MAY_MISS_ITS_FACE_BY`, over the 166 of 367
corpus files that import, and prints `29.43` roundings:
`nist_ctc_03_asme1_ap242-e2.stp` circle `#1681` on plane `#1291` by edge
`#2757`. The file still imports — this is a figure about the file, not about
a refusal.

That is because `A_CIRCLE_MAY_MISS_ITS_FACE_BY` only enters a decision
through `Walk::every_face_holds_the_curve`'s two call sites (`round_run`,
`describe_the_halves_of_a_circle`) and `Walk::the_surface_that_names_the_curve`,
and each of those, read directly rather than from the doc comment, only ever
picks which `EdgeRun` or `Option` comes back on `false` — never `Err`. So a
circle the allowance does not cover, however far off, is named by its faces
instead of refusing the import. The one other direct read of this constant,
`Walk::a_whole_ball_off_its_chart` (§404), does feed a refusal when missed —
but that branch has never fired on a real corpus file (§404.4: twice across
all of `rcad-step`'s tests, both the new slit tests, never the corpus), so
this measurement does not reach it either way.
