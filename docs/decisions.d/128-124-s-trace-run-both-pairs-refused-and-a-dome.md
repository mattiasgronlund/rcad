## 128. §124's trace, run — both pairs refused, and a dome on a shaft has no untangent version, built 2026-09-18

*§124.4 booked one piece of debt: every number in §124 was traced by reading
source, and no `SolidBuilder` model was ever built and fed both `Orientation`
pairs. Built, on `lane-led-repro` off `d1e573a4`, as
`crates/rcad-topo/tests/a_dome_on_a_shaft_of_the_same_radius.rs`. **§124 is
right, on the machine, first try.** And it understated itself: the shape it
calls one unlucky file is the only shape of its kind there is.*

### 128.1 What was built

Three faces, four edges, three corners — the file's `#17` and `#44` with a flat
floor under them instead of the flange and lead frame `LED_D5.0mm-3` actually
carries. The floor is there because the rule reads each face's material side off
that face's own corner **away** from the shared edge, and the shaft needs one as
much as the dome does.

The two surfaces are the file's own numbers, and so are two of the three
corners: `#23` at `(5.04, -5.051514762044E-015, 9.1)` and `#50` at
`(5.04, -2.22044604925E-015, 4.0)`. The third, the dome's pole, is `#32` put
back **exactly** on the sphere at `(2.54, -5.051514762044E-015, 11.6)`, so that
the corner check has nothing to say about it and only the seam is ever in
question. The dome is cut open up one meridian to that pole, which is the
arrangement the file uses (`#19` walks the seam circle, then `#30` out to the
pole and straight back) and is the whole of why the dome has a corner off the
shaft at all.

`SolidBuilder`, `DraftStep`, `EdgeRun`, `Picker`, `HalfSpace` and every surface
constructor were enough, exactly as §124.4 said they would be. Nothing in
`crates/*/src/` was touched.

### 128.2 The three answers, and they are §124's

| what was described | what came back |
| --- | --- |
| dome Forward, shaft Reversed | `Invalid::AClosedEdgeIsWalkedAgainstItsSurface` |
| dome Reversed, shaft Forward | `Invalid::AClosedEdgeIsWalkedAgainstItsSurface` |
| dome Forward, shaft Forward | `Error::ADraftEdgeIsWalkedTheSameWayTwice` |

The first two are §124.2's claim that flipping the reader's guess only moves the
failure. The third is the half that makes the requirement **unsatisfiable**
rather than merely unmet, and it is worth its own line: the pair the rule asks
for does not get refused *by* the rule, it is refused one door earlier by
`SolidBuilder::check_every_edge_is_walked_once_each_way` and never reaches the
rule at all.

§124.1's two distant corners were re-measured off the same model rather than
believed: the dome's pole sits `1.19e-15` off the shaft's axis against a radius
of `2.5` (`Decision::Neg`, and nowhere near a close call), and the shaft's floor
corner sits `5.67978872846517` from the dome's centre, `√(2.5² + 5.1²)`, against
a radius of `2.5` (`Decision::Pos`, likewise). The test asserts bounds —
`< 1e-14` and `√(2.5² + 5.1²)` to `1e-12` — rather than those digits, which came
off a scratchpad `python3` over the same numbers; what is being pinned is that
neither is a close call, and a digit of the distance is not that fact.

Neither half of `material_side` is where
the arithmetic is thin, which is what leaves the tangency as the only thing that
can have gone wrong.

### 128.3 What §124 understated: one seam and tangent are the same condition

§124 reads as a finding about `LED_D5.0mm-3`, and §124.4's "this is one file"
says how little is known about how common the shape is. One line of arithmetic
settles that, and it was not in §124:

A cylinder of radius `r` about an axis, and a sphere of radius `R` centred **on**
that axis, cross where `(z − h)² = R² − r²`. That is **two** circles whenever
`R > r`, **none** whenever `R < r`, and **exactly one** only at `R = r` — which
is the case where the two surfaces are tangent along it.

So a dome that sits square on a shaft shares **one closed edge** with it *if and
only if* the two are the same radius, which is to say *if and only if* they are
tangent. Every such shape lands in §124's gap by construction; there is no
transversal member of the family. `a_dome_on_a_narrower_shaft_has_two_seams_and_not_one`
asserts both circles are on both surfaces, which is that statement as a test.

This is not a claim about how often the shape appears in files — that is still
§124.4's open item. It is a claim that when it does appear, it is always this
case and never a near miss.

### 128.4 The control, and what it cannot be

A reproduction whose every outcome is a refusal proves nothing on its own: both
refusals could be one mistake in the description. §128.3 is why the obvious
control does not exist — there is no transversal dome-on-a-shaft to build the
same description out of — so the control that is left is narrower and is
labelled as such in the test.

Pull the shaft in to `1.5` under a dome of `2.5` and the corner is a real
crossing. The closed-edge rule then takes **one** of the two pairs and refuses
the other, which is what a closed edge is supposed to get. So the rule is
satisfiable, the description reaches it, and the double refusal above is about
the tangency and not about how this file describes a solid.

That near shape is **not a sound solid and is not asserted to be**: its seam
edge claims `RightRoundWhereItsFacesCross` where §128.3 says there are two
loops. Measured over five radius pairs, the pair the rule lets through goes on to
be refused later — `AWireWillNotLieFlatOnItsFace` at `2.5/2.0` and `2.5/0.5`,
`CannotFollowACrossingWhereItsSurfacesAreTangent` at
`SettlingAPointOntoBothSurfaces` for `2.5/1.5`, `5/3` and `10/6`. Those are the
mis-named run being walked, not a new defect, and the test asserts only where the
refusals fall either side of the closed-edge rule.

### 128.5 What is still not measured

- **Whether either fix in §124.4 makes this solid build.** The test asserts
  today's answers and says in its own doc comment that a fix makes it fail. It
  does not say the solid is sound once the rule stops refusing — the mesh has
  never seen it.
- **The mis-named two-loop run's own refusal.** §128.4's five measurements were
  taken to place the control and nothing more; whether
  `AWireWillNotLieFlatOnItsFace` and the marcher's tangency refusal are the
  right answers to a run that names one loop where there are two was not traced.
  Worth its own lane if the vocabulary question in `EdgeRun`'s doc comment is
  ever reopened.
- **How often the shape appears.** §124.4's item stands untouched. §128.3
  narrows what would be counted — every coaxial equal-radius dome-and-shaft pair,
  and nothing else — but counts nothing.

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-led-repro d1e573a4`; `mise exec -- just decision 124`, `124.1`–`124.4`;
`mise exec -- just src a_hemisphere_sitting_on_a_pipe_of_the_same_radius_barely_crosses_it_at_all`,
`SolidBuilder`, `add_round_bar`, `add_ball`, `EdgeRun`,
`a_closed_crossing_is_walked_the_way_it_is_stored`,
`which_side_of_it_this_face_lies_on`, `which_way_a_closed_crossing_turns`;
direct `Read` of `builder.rs`, `validate.rs`, `march.rs`, `along_a_crossing.rs`,
`mesh.rs`, `building_a_solid_out_of_faces.rs`, and of
`kicad/LED_D5.0mm-3.step`'s `#10`–`#59`; `mise exec -- just t -E
'test(/^a_dome_on_a_shaft_of_the_same_radius::/)' --no-fail-fast`; a scratchpad
`python3` for the two distances in §128.2; `mise exec -- just test` (1980 tests,
1975 before this lane); `mise exec -- just gate`.

