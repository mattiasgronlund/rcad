## 124. The rule asks both faces of one closed edge to be forward, and only one of them can be, measured 2026-09-18

*§115.4 booked one open end: `LED_D5.0mm-3` still refuses on its one closed
edge, the reader's hint and the certified rule disagree about it, and which of
them is right was not measured. Traced through both formulas with the file's
own numbers, on `main` at `19c6f3d1` (pushed, green on CI run
`35315093500`). Neither side is wrong. The certified rule asks for something no
edge can be given.*

### 124.1 The one closed edge in the file, and what each side does with it

`kicad/LED_D5.0mm-3.step` has exactly one closed edge —
`grep -oE "EDGE_CURVE\('',#[0-9]+,#[0-9]+" | awk` over the file finds one line
where the two vertex references match, `#21 = EDGE_CURVE('',#22,#22,#24,.T.)` —
shared by `#17`, a `SPHERICAL_SURFACE` dome, and `#44`, a `CYLINDRICAL_SURFACE`
wall, both of the same radius `2.5` and (up to the file's own femtometre
rounding) the same axis. This is the exact shape §109.7 and §111 already dug
into — a dome sitting on a body of its own radius, tangent along the seam —
and §111.1's numbers are reused below rather than retyped, checked instead
against `kicad/LED_D5.0mm-3.step` directly:

```
#39 = SPHERICAL_SURFACE('',#40,2.5);      #41 = (2.54, -5.051514762044E-015, 9.1)
#73 = CYLINDRICAL_SURFACE('',#74,2.5);    #75 = (2.54, -3.030908857227E-015, 9.1)
#76 = DIRECTION('',(-0.E+000, 3.330669073875E-016, -1.))
#22 = VERTEX_POINT('',#23);               #23 = (5.04, -5.051514762044E-015, 9.1)
```

Both `#17` and `#44` carry `same_sense = .T.`, so `Walk::surface_of_the_face`
(`crates/rcad-step/src/brep.rs:268`) gives both faces `Orientation::Forward` —
neither surface is stored turned round.

**The reader's hint**, `which_way_round_a_closed_edge_is_stored`
(`brep.rs:610`), works out a travel vector per face and dots it against the
sphere's longitude `ẑ × (corner − centre) = (0, 2.5, 8.3e-16)` — exact, because
the sphere's centre and the corner share the same stored `y` bit for bit. That
last term is not zero, as this section first wrote: the sphere's axis carries a
`3.33e-16` `y` component of its own. It changes no sign below. For
`#17` the travel comes to `(2.0e-15, 2.5, 8.3e-16)`, dot `+6.25`, `Forward`.
For `#44` it is the exact negative, `(-2.0e-15, -2.5, -8.3e-16)`, dot `-6.25`,
`Reversed`. Both dot products are nowhere near zero — this half of the
question has no numerical difficulty at all, and the two answers are forced
to be opposite of each other by construction: the two faces' boundary loops
run this shared edge in exactly opposite spatial directions (`travel_44 =
-travel_17`, computed, not assumed), so whichever way the winding vector
points, one face agrees with it and the other cannot.

**The certified rule**, `a_closed_crossing_is_walked_the_way_it_is_stored`
(`crates/rcad-topo/src/validate.rs:1556`), is run once per face, each time
against the *other* face as `across`. §111.1 already proved its `turning`
term — `(∇S × ∇T) · (ẑ × ∇S)`, `4.206230998539495e-30` — is not a coin toss
for this file: it is an exact value of the stored numbers with nothing
cancelling, and §111.2's boxed enclosure of it still proves the same sign.
(This section first cited the enclosure's lower edge as though it were the
value itself, and called the enclosure's whole width the width "either side"
of it — as if the box were twice as wide as it was. Both figures are now
struck from §111.2 for a separate reason, that they do not reproduce; this
one is a wording bug, independent of that. It is the fourth time this file
has been wrong about a number quoted from itself rather than re-run, after
§111.3, §115.4 and §120.5.) What §111 did not
trace is the other half the rule needs, `material_side`
(`which_side_of_it_this_face_lies_on`, `validate.rs:1608`), which reads the
side of one corner **not** on the shared edge:

- for face `#17`, the only such corner is `#31`, the sphere's pole at
  `(2.54, -6.44e-15, 11.6)` — on the cylinder's own axis, radius `0` against
  the cylinder's `2.5`, `Decision::Neg`, nowhere near degenerate;
- for face `#44`, it is `#49` at `(5.04, -2.22e-15, 4.0)` — `√(2.5² + 5.1²) ≈
  5.68` from the sphere's centre against its radius `2.5`, `Decision::Pos`,
  just as far from degenerate.

`turning` flips sign between the two calls (`outward[0]` and `outward[1]`
swap, and cross product anti-commutes — `which_way_a_closed_crossing_turns`'s
own test `a_crossing_with_room_in_it_is_read_the_same_both_ways_round`
demonstrates exactly this swap), so `runs_along_stored` is `true` for `#17`
and `false` for `#44`. Folding in `material_side` through
`walks_it_forward = runs_along_stored XOR (material_side == Pos)`:

| face | `runs_along_stored` | `material_side` | `walks_it_forward` |
|---|---|---|---|
| `#17` (sphere) | true | Neg | **true** |
| `#44` (cylinder) | false | Pos | **true** |

Both come out **Forward**.

### 124.2 Why "both Forward" cannot be given to any reader

`SolidBuilder::build` (`crates/rcad-topo/src/builder.rs:324`) calls
`check_every_edge_is_walked_once_each_way` before it ever reaches
`Model::validate`, and its own doc names the refusal for getting this wrong:
`Error::ADraftEdgeIsWalkedTheSameWayTwice`. Running
`RCAD_STEP_CORPUS=<corpus> mise exec -- just t how_often_a_real_file_imports
--no-capture` today (`kicad: 114 / 323 = 35.3%`, `nist: 1 / 44 = 2.3%` — kicad
moved from §115.4's stale `110 / 323`, nist did not move and §115.4 says so
itself, `1 of 44` either way, and neither is quoted from anywhere but this run)
turns up **26 refusals across 25 files** carrying exactly that shape, `"both
coedges of the edge described at position N run the same way round"`, on a
family of switch
files (`SW_PUSH_6mm*`, `SW_TH_Tactile_Omron*`, …) that this section does not
otherwise touch — named here only to show the invariant is real and load-bearing
elsewhere in the tree, not a theoretical possibility.

So no possible assignment of `Orientation` to the two coedges over `#21` can
ever be *both* the pair the builder demands (one Forward, one Reversed) *and*
the pair the certified rule asks for here (both Forward). Whichever way the
reader guesses — and its own guess, `#17` Forward / `#44` Reversed, is the one
required by the builder's own invariant and matches what §111.1's `∇S`, `∇T`
say about which way is physically outward at the corner — exactly one of the
two per-face checks in
`check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind`
(`validate.rs:629`) must fail. The observed refusal is `#44`'s: reader says
`Reversed`, rule wants `walks_it_forward = true` i.e. `Forward`, mismatch,
`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`. Flipping the reader's guess
only moves the failure to `#17` — `walks_it_forward = true` there too, so
swapping the pair does not help either.

### 124.3 The verdict

**Neither the file nor the reader's hint is wrong.** `LED_D5.0mm-3` is an
ordinary dome-on-a-shaft, tangent along its seam at the radius both surfaces
share — not a degenerate or malformed solid — and the reader computes the one
`Orientation` pair the topology can accept. The defect is in the certified
rule's own arithmetic: `a_closed_crossing_is_walked_the_way_it_is_stored`
combines a **local** signal (`turning`, evaluated at the shared corner) with a
**distant** one (`material_side`, sampled at a corner the file happens to put
somewhere else on each face) that the doc comment's own derivation — "the
face's material is inside its partner" — only holds apart cleanly when the two
surfaces genuinely cross, transversally, at the edge. `which_way_a_closed_crossing_turns`'s
own doc comment says as much for the case it does catch: "`Decision::Zero` out
of both leaves the wire with nothing to be but the zero vector, which is two
surfaces touching rather than crossing and has no direction to name"
(`which_way_a_closed_crossing_turns.rs:39`). The shorter sentence this section
first quoted here — "two faces whose gradients point the same way are not
crossing" — is the doc of a *test*,
`two_surfaces_that_touch_rather_than_cross_have_no_direction`, not of the
function. The claim it was cited for survives the swap. But that guard only
fires on an *exact* zero
cross product, and this file's own femtometre-scale axis rounding is enough to
give `∇S × ∇T` a real, provable-nonzero, but still line-thin value (§111's
whole finding), so the file's near-tangency slips past the one check meant to
say "this is a touch, not a crossing" and lands in logic that assumes a
crossing.

**This is §16.10's gap, not a limit, and not a bad file.** The answer exists —
this file should import, and the topology to give it already exists in the
reader's own guess — the certified rule's combining formula cannot reach it
for a tangent pair, not because the arithmetic runs out of precision (§111
already showed it does not) but because `material_side` and `turning` are
computed at two different points that need not agree once "crossing" degrades
into "touching along the whole seam."

### 124.4 The family of fixes, and what is not measured

Two families were open, and only one fits what §124.1–124.2 measured:

- **Detect the near-tangency and treat it as `Decision::Zero`** the way an
  exact tangency already is, using the same angle `a_hemisphere_sitting_on_a_pipe_of_the_same_radius_barely_crosses_it_at_all`
  already computes (`squared_sine` of the angle between `∇S` and `∇T`) as the
  test for "this is a touch". This changes what counts as `Zero` rather than
  what the sign means, and would make `check_closed_edges_are_walked_the_way_their_surfaces_wind`
  skip this edge (`Ok(None)`, per the existing `continue` arm) rather than
  assert an unsatisfiable pair.
- **Make `material_side` and `turning` agree on where they look** — sample
  `material_side` at (or infinitesimally near) the same corner `turning`
  already uses, rather than at whichever other corner each face happens to
  carry, so the two terms are answering the same local question instead of a
  local one and a global one. This is the more invasive fix, though one level
  further down than this section first said: `which_side_of_it_this_face_lies_on`
  (`validate.rs:1608`) has exactly one caller, the closed-edge check itself
  (`validate.rs:651`), so narrowing *it* risks nothing. What is shared is
  `the_side_this_face_lies_on` (`holds.rs:533`), which it calls and whose doc
  carries the sentence "This is the only place the model says which side"
  (`holds.rs:515`). That one has three callers — `holds.rs:373`,
  `validate.rs:563` and `validate.rs:1622` — so the risk is real, but it is a
  risk to the function underneath rather than to this one.

Neither is built here, on purpose — this lane owns a measurement and not
`crates/`. What is **not** measured:

- **This is one file.** `LED_D5.0mm-3` is the only file in the 367-file corpus
  (`kicad` + `nist`) carrying this exact refusal text — checked directly against
  today's `just t how_often_a_real_file_imports --no-capture` output, not
  assumed — so nothing here says how common an exact-radius, near-coincident-axis
  dome-on-cylinder seam is outside this corpus. The 25-file "both coedges run the
  same way round" family above looks related by its error text but was not
  traced; it may be the same contradiction arrived at through the reader
  producing the *wrong* pair outright (a plain reader bug, no topology
  question), a different tangency, or something else again — this section says
  only that it exists and is out of scope, not what it is.
- **No end-to-end reproduction was built.** Every number above was traced
  through the actual source (`validate.rs`, `which_way_a_closed_crossing_turns.rs`,
  `holds.rs`, `sphere.rs`, `cylinder.rs`, `brep.rs`) and cross-checked against
  the file and against the numbers `a_hemisphere_sitting_on_a_pipe_of_the_same_radius_barely_crosses_it_at_all`
  already asserts, but no throwaway `SolidBuilder` model was actually built and
  fed both possible `Orientation` pairs to watch both fail. `SolidBuilder`,
  `DraftVertex`/`DraftEdge`/`DraftCoedge`/`DraftFace`/`DraftWire`,
  `which_way_a_closed_crossing_turns` and every `Surface` constructor used here
  are public, so that reproduction is buildable by whoever picks up either fix
  above, without needing anything this lane could not already reach.
- **Whether `material_side`'s "distant corner" choice is wrong in general**, or
  only wrong once the two surfaces are this close to tangent, was not settled —
  the doc comment's own Cauchy–Schwarz argument (`validate.rs:1498`) reads as
  though it holds for any crossing, and finding the exact angle at which it
  stops holding (if it is a threshold at all, rather than a discontinuity right
  at tangency) is a numerical-refusal question this lane did not have the
  budget for.

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-hint-vs-rule 19c6f3d1`; `mise exec -- just decision 115`, `115.1`–`115.5`,
`109`, `109.7`, `111`, `111.1`–`111.4`, `112`, `116`, `117`; `mise exec -- just
decisions-find 'LED_D5.0mm-3'`, `'closed edge'`,
`'which_way_a_closed_crossing_turns'`, `'CannotTellWhichWayAClosedEdgeRuns'`;
`mise exec -- just src the_curved_one_of`, `how_it_sorts_against`,
`the_way_its_first_coordinate_runs_at`, `which_way_a_closed_crossing_turns`,
`a_closed_crossing_is_walked_the_way_it_is_stored`, `which_side_of_it_this_face_lies_on`,
`turned_by`, `the_face_across`, `gradient_of`, `the_side_this_face_lies_on`,
`side_of`; direct `Read` of `kicad/LED_D5.0mm-3.step`, `brep.rs`,
`validate.rs`, `which_way_a_closed_crossing_turns.rs`, `holds.rs`,
`sphere.rs`, `cylinder.rs`, `builder.rs`; `git log --oneline --follow -- crates/rcad-geom/src/which_way_a_closed_crossing_turns.rs`;
`RCAD_STEP_CORPUS=<corpus> mise exec -- cargo run -p rcad-step --release
--example blocker_depth_probe`; `RCAD_STEP_CORPUS=<corpus> mise exec -- just t
how_often_a_real_file_imports --no-capture`.

### 124.5 What an audit of the above moved, and what it did not

§124 was written from source read rather than source run, and §124.4 says so
itself. A read-only lane re-derived every number and citation in it against
`d1e573a4` on 2026-09-18. **The finding holds.** The one-file claim, both corpus
rates, the 367-file corpus, every STEP entity quoted in §124.1, both travel
vectors, the XOR in `walks_it_forward`, the both-Forward table and the
local-versus-distant chain all reproduce. Nothing below changes the verdict.

Six supporting facts did not reproduce, and they are corrected in place above
rather than left standing with a note. Two of them are worth naming:

- **The family is 25 files, not 37.** §124.2 counted `grep` lines and §124.4
  reported them as files; 11 of the 37 are the test's own histogram summary, and
  the 26 refusal lines fall on 25 distinct files, one file refusing twice.
  A lane sizing that family off the old number would have started 48% over.
  Re-derived here, not taken from the lane: `grep 'run the same way round' |
  grep '  refused  ' | awk '{print $3}' | sort -u | wc -l` over a fresh
  `how_often_a_real_file_imports --no-capture`.
- **`which_side_of_it_this_face_lies_on` is not shared.** §124.4's second fix
  family rested on it having three callers and carrying the "only place the
  model says which side" doc. It has **one** caller and a different doc; the
  sentence and the three callers belong to `the_side_this_face_lies_on` in
  `holds.rs`, which it calls. The invasiveness argument survives one level
  down, and §124.4 now says which function it is about.

The other four are cosmetic and corrected above: two line numbers off by one,
the Cauchy–Schwarz line 57 lines out, a winding vector whose `z` term is
`8.3e-16` rather than `0`, and a sentence about NIST moving when §115.4 says it
did not.

**One claim could not be checked, and the remedy named for it was wrong.**
§124.1 quoted §111.2's enclosure edge as if it were the `turning` value —
corrected in §124.1 itself, to §111.1's `4.206230998539495e-30`. When this
section was written, neither figure was asserted anywhere but in this file, so
the debt it flagged was real. But `blocker_depth_probe` does not pay it. Its own
doc comment says what it does: "turns what it finds into the number that
actually prices a fix: not how many files a reason stops first, but how many
files have **only** that reason and nothing else behind it" — a census over the
whole STEP corpus, one row per file, of which import blocker sticks after the
others are cleared. There is no path in it anywhere near a single edge's
`turning` arithmetic, so running it would reproduce nothing here.

**The debt is paid now, and by a test rather than by that probe.**
`the_dome_and_pipe_from_the_led_file_turn_by_a_real_amount`
(`crates/rcad-geom/tests/`) recomputes the five-line chain from §111.1's own
four points and one direction, written in as literals, and asserts both the
cross product and `4.206230998539495e-30`. It needs no corpus, because §111.1
names the whole input. Its opening draws the distinction §124.1 lost — that this
is the value and **not** §111.2's enclosure — and it reaches the same verdict on
those edges that §111.2 above now records: the `nextafter`-slack procedure was
never checked in, so there is nothing to call and no way to reproduce them bit
for bit.

The lesson is not that §124 was careless: it was explicit that it had run
nothing, and it was right about the thing it was for. It is that a section which
says "traced by reading" has a debt, and the debt falls due in the citations
before it falls due in the conclusion.

