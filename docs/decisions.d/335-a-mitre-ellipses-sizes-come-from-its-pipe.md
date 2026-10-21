## 335. A mitre ellipse's sizes come from its pipe, and the file's are held to twelve digits — 2026-09-28

*Queue row Q38, from §229.5. Lane `lane-q38-sizes-from-the-pipe`, off
`96da3252`. `every_face_holds_the_curve` in `crates/rcad-step/src/brep.rs` now
works an `ELLIPSE`'s two sizes out from the first face's pipe and holds that
one ellipse against every face; the file's own sizes are still checked, to
what writing them in twelve digits costs. 273 of the corpus's 636 ellipses
between two pipes are now named by their plane. No file starts or stops
importing. **A gap** (§16.10): the answer was in the file's other numbers.*

### 335.1 What stopped the mitre, and it was not the axis miss

§229.5 said the Fujitsu `FTR-F1` relays' mitre ellipses fail the ellipse
check, and pointed at the two pipe axes missing each other by about `1.9e-14`.
That miss is not the cause. On ellipse `#691` of
`kicad/Relay_DPDT_Fujitsu_FTR-F1C.step`, the ellipse the level pipe gives is
**5.26** roundings off it, where the allowance is 16. A rounding here is
`f64::EPSILON` times `16.15`, the largest number involved.

The cause is the size the file writes. `#692 = ELLIPSE('',#693,0.141421356237,0.1)`
writes `0.1·√2` in twelve digits, which is `3.1e-13` short. With that size the
file's ellipse is **66.3** roundings off the level pipe and **61.0** off the
upright one. All 9 ellipses in each of the two relays fail as written. Worked
out from the pipe, all 18 pass, and the worst is **14.37** (`#2668`).

Measured with a probe printed from inside `every_face_holds_the_curve`, not
committed, run on this lane's tree over both corpora with
`mise exec -- just t -p rcad-step --test <probe> --no-capture`. The probe was
a copy of the first Q38 lane's corpus walker.

### 335.2 Why the sizes come from the pipe

A plane cuts a pipe of radius `R` about the unit axis `d` in an ellipse. Its
size along each of its two ways `w` is `R / |w × d|`. The model never reads the
file's sizes: the run names the plane, the walk draws the plane crossing the
pipe, and `the_arc_between` reads only the placement's axis and the corners. So
the question the check has to answer is not whether the file's numbers are on
the pipe. It is whether one ellipse, the one the plane cuts, lies on every face.

That is what the ELLIPSE arm now asks. It works the two sizes out from the
first face, which must be a pipe, and holds that one ellipse against every
face. Holding it against the second face also checks that pipe's radius and
lean against the first's. `a_mitre_between_pipes_a_hair_apart_is_named_by_its_pipes`
shows why that part matters. Its level pipe is `1e-13` wider and `1e-13`
higher, so every corner and both seams stay on it. Checked only against the
file's sizes, that elbow **imports**, named by its plane, and
`Model::validate` does not notice.

Each size is worked out along its own way, not taken to be `R / |n·d|` along
the reference direction. The schema lets the first size be the short one, and
`a_mitre_whose_reference_runs_along_its_short_axis_is_named_by_its_plane` is a
file that does that. A plane along the pipe's axis gives an infinite size, and
an infinite size makes the allowance infinite too, since `largest_of` takes the
sizes in. So the arm holds nothing when a size, drawn or written, is not
finite. `Axis::direction` is length one to within rounding (its constructor
scales it, and `Axis::transformed` only turns it), so `|w × d|` is a sine.

### 335.3 Why the file's sizes are still checked, and to twelve digits

If only the drawn ellipse were checked, a file whose sizes are plainly wrong
would be read as the curve the plane cuts without a word. Its faces agree with
each other, but its curve is not on them. The file's ellipse is therefore
still held against every face, to the same allowance **plus five parts in
`10¹²` of the larger drawn size**. That is the most a size written in twelve
significant digits can be off: half a unit in the twelfth digit, relative to a
number whose first digit is 1. A file that fails this is named by its faces, as
every ellipse was before §222.2.

Twelve comes from what the corpus's writers write. KiCad's exporter writes an
ellipse's sizes in twelve significant digits and trims trailing zeros (`0.11313708499` is
`0.08·√2` rounded to `0.113137084990`). The measurement is what makes it a
floor rather than a guess. Over both
corpora, **no ellipse that the drawn check passes fails the written one**, so
the allowance costs nothing a file needs. An eleven-digit size fails it, and
`a_mitre_whose_size_is_written_in_eleven_digits_is_named_by_its_pipes` holds
that edge (§15.6).

This is not a tolerance on the model. The curve the model is built from is held
to the unchanged sixteen roundings. The twelve digits only decide whether the
file's own word about the curve agrees with its faces, and a no sends the run
to its faces, which is the safe direction. It does not use the file's declared
uncertainty (§42.9 option A stays).

### 335.4 What moved

| over both corpora, per distinct ellipse edge the reader reaches | count |
|---|---|
| passed before, pass now | 359 |
| failed before, pass now | 273 |
| fail the drawn check | 4 |
| pass drawn, fail written | 0 |
| passed before, fail now | 0 |

The 273 are in 29 files. The 4 are all in
`Fuseholder_Cylinder-5x20mm_Schurter_0031_8201_Horizontal_Open.step` (`#338`,
`#2313`, `#2331`, `#3043`), and row Q381 takes them.

`A_PLANE_BETWEEN_CURVED_FACES` in `how_often_a_real_file_imports.rs` was
**688** planes between two curved faces in imported files on the base, 334 of
them ellipses on pipes. Now it is **848**, with 494, and the floor moves from
516 to 636, three quarters as before. `KICAD_FILES_THAT_IMPORT` did not move:
nist 1 of 44, KiCad 161 of 323, before and after. The files the check now
passes that already imported, the Hongfa HF115F, Omron G2RL, G6K-2P and
G6K-2P-Y, CUI SR5, Finder 32.21 and 36.11, the SANYOU SRDs, Omron G5NB and
G5LE-1 among them, all still import. The pinned set asserts that.

Of the 162 KiCad refusals, one changed its name.
`Relay_SPDT_Finder_34.51_Horizontal.step` moved from
`CannotNameWhichPieceOfACrossingThisIs` to "a torus crossing another curved
surface is a space curve of degree up to eight".

> **2026-10-10 note (Q558, from Q381): the four are limits, not gaps.** Each
> of `#338`, `#2313`, `#2331` and `#3043` is a 22.5° mitre between an upright
> pipe and a 45° pipe, both of radius `0.4`, written
> `ELLIPSE(…,0.432956880117,0.4)`. The file puts each centre
> `(±5e-13, 0, −5e-13)` off the leaning pipe's axis. It writes the centre's
> two offsets from that axis's origin as `0.117157287526` and `0.117157287525`
> for `#338` and `#2313`, and as `1.168556998188` and `1.168556998189` for
> `#2331` and `#3043`, where a 45° axis needs them equal. Worked out in exact
> decimals that is `√2·5e-13` off, **193.34** roundings of `16.471428571429`
> and **283.90** of `11.217157287525`. The kernel's `f64` gives **193.70**
> (+0.36) and **283.67** (−0.23). The allowance is 16. Their four siblings,
> `#1769`, `#2216`, `#2523` and `#2960`, whose centre's `x` is a short decimal
> (`22.1` or `0.4`), are `0` off, exactly and in `f64`.
>
> Each of the four edges runs from a corner where both seams meet,
> `(x, ±4.8, 10.048600289336)`, to one across the pipe: `#33`, `#870`, `#926`
> and `#1520`. The file puts that second corner `1.68305e-13` **inside** the
> leaning pipe, **47.16** and **74.21** spacings of its largest number worked
> out exactly. The kernel certifies it `Neg`, `1.6706e-13` off at **46.81**
> spacings and `1.6894e-13` at **74.49**. The siblings' corner across the
> pipe is outside it instead, certified `Pos` at `5.376e-13`, **107.61**
> spacings, and `5.395e-13`, **237.89**. That is the file writing twelve
> decimals, and corners off their faces are Q559's ground, not this note's.
>
> **Why they are limits** (§16.10). The reader can change one of three
> things: the curve it names the run by, the allowance it holds that curve
> to, or the file's numbers.
>
> - *The curve.* Two pipes of one radius whose axes meet cross only in the two
>   planes through the meeting point that halve the angle between the axes. A
>   point the same distance from both axes is as far along one as along the
>   other, so it is on one of those planes. Here the axes meet at
>   `z = 10.048600289337`, `1e-12` above the file's centre, and neither plane
>   runs through the centre. A plane taken through the meeting point gives an
>   ellipse on both pipes, but it misses both corners. The seam corner is
>   `3.84e-13` off the nearer plane for `#338` and `#2313`, **104.91**
>   spacings, and `3.82e-13` for `#2331` and `#3043`, **171.17**. The
>   siblings' is on it. So no curve on both pipes starts where the file
>   starts the edge, whatever plane is taken, and the corner across the pipe
>   is off the leaning pipe besides.
> - *The allowance.* Widening it past 193 or 284 roundings is a tolerance on
>   the model, which §335.3 rules out. With the drawn check's `16` replaced by
>   `200` in `a_surface_holds_the_ellipse`, `#338`'s elbow imports, named by
>   its plane.
> - *The file's numbers.* Moving a centre or a corner onto the crossing is
>   healing, which §42.9 leaves to the user.
>
> Pinned in `crates/rcad-step/tests/a_mitre_whose_centre_the_file_writes_off_its_pipe.rs`,
> from a STEP text holding one bend in the file's own decimals.
> `a_mitre_whose_centre_is_off_its_leaning_pipe_is_named_by_its_pipes` holds
> that `#338`'s elbow is refused `TheSolidTheFileDescribesIsNotOne`, with its
> closed mitre walked against its surface (why the run named by two pipes
> ends there is not traced), and the allowance at 200 fails it.
> `a_mitre_whose_centre_is_on_both_pipes_is_named_by_its_plane` holds that
> `#1769`'s elbow is named by its plane, validates, and meshes onto both
> pipes, and the allowance at `0` fails it, refused
> `CannotNameWhichPieceOfACrossingThisIs`.
> `how_far_the_file_writes_each_mitre_off_its_leaning_pipe` holds every kernel
> figure above for all eight. The exact figures came from a scratch script in
> exact decimals, not committed.

### 335.5 Where the two Fujitsu relays stop now

Both are still refused `TheSolidTheFileDescribesIsNotOne { why:
Geometry(CannotNameWhichPieceOfACrossingThisIs) }`, but not at a mitre. All 9
mitres in each are now named by their plane. The one run still named by a face
is a quarter arc of a `CIRCLE`: edge `#2708` in `FTR-F1C` and `#2384` in
`FTR-F1A`. (This said a closed `CIRCLE` until Q561, which read the edges: each
runs a quarter of its circle.) The circle is of radius `0.1` round
`(10.05, 1.15, 16.4)`, turning about
`(3.663735981263E-015, 1., 0.)`, where a pin's pipe meets the ball of its own
radius that rounds its end. The ball holds it exactly. The pipe's axis starts at
`(10.05, -19.75, 16.4)` and leans the same `3.7e-15`, and `20.9` along it the
pipe's bound puts the circle `7.64e-14` off. That is **17.42** roundings, just
past the 16. The run
is named by the ball, and a ball and a pipe that only touch leave the walk two
pieces it cannot tell apart. This is the circle arm's row, not this one.

> **2026-10-10 note (Q561, from Q382): the two circles are limits, not gaps.**
> Edge `#2708` of `FTR-F1C` runs a quarter of circle `#2709`, from
> `#2404 = (10.05, 1.15, 16.5)` to `#1030 = (10.15, 1.15, 16.4)`, between the
> pipe `#2723` and the ball `#3414`. `FTR-F1A`'s `#2384` runs `#2385` between
> the same two points (`#2080`, `#962`) on the pipe `#2399` and the ball
> `#2998`, all in the same decimals. The pipe's axis starts at
> `(10.05, -19.75, 16.4)` and leans `3.663735981263e-15` in `x` per unit of
> `y`, so `20.9` along it, beside the ball's centre, it is off that centre by
> `7.6572082008e-14` in exact decimals, **17.46** roundings of `19.75`. The
> kernel's `f64` gives `7.6395e-14`, **17.42** (−0.04). The file centres the
> circle on the ball, so that is the circle's miss off the pipe.
>
> The miss is the centre's and nothing else's. The circle, the ball and the
> pipe are all written radius `0.1`, so the radius is `0` off on both faces,
> and the circle turns about the pipe's own direction bit for bit, so the
> pipe's bound gets `0` from the tilt as well.
>
> In exact arithmetic this ball and this pipe do not cross in a circle at all.
> Take `u` along `x` and `v` along `y` from the ball's centre and call the miss
> `q`; along the loop below, the lean moves the axis by under `1e-21`, so take
> it as running along `y` there. Taking the pipe's equation from the ball's leaves
> `v² = q·(q − 2u)`, so the two meet only where `u ≤ q/2`, in one thin loop
> over the half of the ball away from `#1030`, running `√(q·(2r − q))` either
> way along the axis: **1.2375e-7** exactly, **1.2361e-7** from the kernel's
> `q`. The file's arc runs from `u = 0` at `#2404` to `u = r` at `#1030`, so
> past its first `q/2` it runs where the faces do not meet at all. The kernel
> certifies `#1030`
> inside the pipe, `Neg`, `7.6395e-14` off, **20.98** spacings of `16.4`
> (**21.03** exactly).
>
> **Why they are limits** (§16.10). The ways out, each with the figure that
> rules it out:
>
> - *The radius from a face* rather than the circle's. It is `0.1` on both,
>   `0` off, so all of the `17.42` is the centre's.
> - *The centre moved onto the pipe's axis.* The circle is then on the pipe and
>   off the ball by as much: **20.98** roundings of `16.4` in `f64`, **21.03**
>   exactly, past the 16 from the other side.
> - *The centre moved halfway.* Half of each miss, about `8.7` and `10.5`
>   roundings (halved from the two figures above, not run through the check),
>   would pass both faces. But a run between two curved faces is named by the
>   circle's plane, and the plane through the halfway centre with the file's
>   normal is the file's plane to the last bit: both offsets are
>   `1.1500000000000368`, since the move is `3.8e-14` along `x` and the
>   normal's `x` is `3.7e-15`. The walk pairs that plane with the ball (the doc
>   of `the_surface_that_names_the_curve` in `brep.rs`), which draws the file's
>   circle, `17.42` off the pipe; paired with the pipe it would draw the circle
>   `20.98` off the ball. So moving the centre is the allowance widened under
>   another name.
> - *The ball and the pipe themselves*, which is what the reader does now.
>   Their crossing is the thin loop above, not a circle, and it does not reach
>   `#1030`, which is inside the pipe. The walk cannot say which piece of it
>   the arc is, and the solid is refused
>   `CannotNameWhichPieceOfACrossingThisIs`. No curve on both faces ends at
>   `#1030`, whatever surface the run is named by.
> - *The allowance.* Widening it past `17.42` roundings is a tolerance on the
>   model, which §335.3 rules out. With `A_CIRCLE_MAY_MISS_ITS_FACE_BY` at
>   `18.0` in `brep.rs` the pin end below imports.
> - *The file's numbers.* Moving the centre or `#1030` onto the crossing is
>   healing, which §42.9 leaves to the user.
>
> Pinned in
> `crates/rcad-step/tests/a_circle_the_file_puts_off_its_pipe_past_the_allowance.rs`,
> from a STEP text holding one pin end, the ball, the leaning pipe and a far
> disc, in the file's own decimals. Its rim is cut in two arcs, at `#2404` and
> at `(10.05, 1.15, 16.3)` across from it, which are both on both faces;
> `#1030` is not used, since it is off the pipe.
> `a_circle_centred_off_its_leaning_pipe_is_named_by_the_ball` holds that this
> pin end is refused `TheSolidTheFileDescribesIsNotOne { why:
> Geometry(CannotNameWhichPieceOfACrossingThisIs) }`, as the two files are,
> and the allowance at `18.0` fails it ("… was read as on the pipe").
> `a_circle_centred_on_its_upright_pipe_is_named_by_its_plane` is the control
> from the same file: the same ball on the upright pipe `#2699` with circle
> `#2692` (`#2368` in `FTR-F1A`), centred on that pipe's axis, imports with
> its four arc coedges named by the plane `z = 16.4`, and validates.
> `how_far_the_file_puts_the_circle_off_its_pipe` holds every kernel figure
> above. The exact figures came from 40-digit decimals in a one-line
> `python3` script, not committed. Q382's count of the circles where curved
> faces meet that fail with the radius as written and with the radius from a
> face, over both corpora, was not reproduced, and is left out.
>
> The rim written **closed** goes another way, and no file here writes it so.
> The reader names it `RightRoundWhereItsFacesCross` by the ball and the pipe,
> the pin end imports and `Model::validate` passes it, and `measure` then
> refuses `ACrossingAlongThisEdgeCouldNotBeFollowed { why:
> CannotFollowACrossingBecauseNoStepStaysOnIt }`, `shortenings: 50`. That
> crossing is the thin loop, not the rim, so the model holds a curve the file
> did not draw. Filed as a row of its own.

> **2026-10-11 note (Q1659, `walk.rs`, `7151e6e5`): both relays now refuse a
> different way.** Since Q1659 picked an open run's loop by its two corners and
> settles each corner onto both surfaces before the march, the quarter arc
> from `#2404` to `#1030` is refused there rather than reaching the ball/pipe
> tie this section and the fourth way out of the 2026-10-10 note above
> describe. `#1030` is where the ball and the pipe are tangent, so settling it
> refuses `CannotFollowACrossingWhereItsSurfacesAreTangent { at:
> SettlingAPointOntoBothSurfaces }` at every deflection
> (`a_quarter_circle_ending_where_a_ball_and_its_own_pipe_do_not_cross_is_refused`
> in `walk.rs`), not `CannotNameWhichPieceOfACrossingThisIs` as the section's
> first sentence and that note say. `mise exec -- just t -p rcad-step -E
> 'binary(merged) & test(how_often_a_real_file_imports)' --no-capture` now
> prints "the builder or the
> sweep: geometry: this crossing runs where its two surfaces are too nearly
> parallel to follow it, asked while settling a point onto both of them" for
> both `Relay_DPDT_Fujitsu_FTR-F1C.step` and `Relay_DPST_Fujitsu_FTR-F1A.step`.
> The pin end in
> `a_circle_the_file_puts_off_its_pipe_past_the_allowance.rs` still passes as
> refused `CannotNameWhichPieceOfACrossingThisIs`, since it never uses `#1030`.

> **2026-10-13 note (Q1889, `march.rs`, `walk.rs`): the pin end is refused at
> its corners, and it is still a limit.** Q1889's first fix (`1983b8c8`) stops
> `march`'s `one_step` keeping a trial that came back short behind its start.
> With it the coarse walk of the pin end's rim no longer stopped at the gap
> between the two strands, and the pin end imported and validated. The arc on
> the `x > 10.05` side of the rim was walked at `f64::MAX` as 7 points with
> `x − 10.05` from `0.044` to `0.1`, each certified outside the ball, under a
> claimed deviation of `7.9e-7`. Every point where the two cross has
> `x − 10.05` at most `q/2 = 3.8e-14`, so each was at least `0.044` off the
> crossing. A wrong answer, worse than the refusal it replaced. The coarse mesh
> measured the volume `0.631 ± 4.27` against `0.789` by hand, so the volume
> could not tell.
>
> What let it through is the bound `march.rs`'s `how_far_off_the_crossing`
> puts on a settled point, `(d₁ + d₂)/s`. It is the widest the region two flat
> slabs leave, so it holds only where neither surface bends away within its
> own reach. At the rim's two corners, where the thin loop's two strands meet,
> the ball and the pipe lean `s = 7.6e-13` apart, and that reach is far wider
> than the gap they rest on. Q1889's second fix bounds an open run's two
> corners by Kantorovich's theorem with the two radii counted
> (`settled_with_a_bound_where_they_bend`): it refuses
> `CannotFollowACrossingWhereItsSurfacesAreTangent { at:
> BoundingHowFarOffTheCrossingAPointIs }` where `h = √2·L·η/s` is over a
> half, with `L = Σ 1/(rᵢ − |d| − 2η)`. At these corners `h` is far over a
> half, so both halves of the rim are refused before the march, at every
> deflection and in either order of the surfaces
> (`both_halves_of_a_rim_where_a_ball_and_a_pipe_off_its_centre_turn_tangent_are_refused`
> in `walk.rs`). The pin end is refused `ThisEntityOfTheSolidWasRefused` at
> its first arc `#32` with that error
> (`a_circle_centred_off_its_leaning_pipe_is_refused_where_its_ball_and_pipe_turn_tangent`).
> The quarter arc to `#1030` is refused the same way at `#2404`, which is now
> asked first, and `how_often_a_real_file_imports` prints "… asked while
> bounding how far off a point sits" for both relay files.
>
> It is still a limit (§16.10). No curve on both faces runs along the rim's
> `x > 10.05` side, so that arc has no answer. The `x < 10.05` side holds two
> strands, `1.2e-7` either side of the rim, and the file says which neither
> is. Elsewhere the flat bound is left as it was: an open run's corners are
> the only points the bend is counted at. Counting it at every marched point
> is row Q1678.
