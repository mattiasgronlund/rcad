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

### 335.5 Where the two Fujitsu relays stop now

Both are still refused `TheSolidTheFileDescribesIsNotOne { why:
Geometry(CannotNameWhichPieceOfACrossingThisIs) }`, but not at a mitre. All 9
mitres in each are now named by their plane. The one run still named by a face
is a **closed `CIRCLE`**: `#2708` in `FTR-F1C` and `#2384` in `FTR-F1A`. It is a
circle of radius `0.1` round `(10.05, 1.15, 16.4)`, turning about
`(3.663735981263E-015, 1., 0.)`, where a pin's pipe meets the ball of its own
radius that rounds its end. The ball holds it exactly. The pipe's axis starts at
`(10.05, -19.75, 16.4)` and leans the same `3.7e-15`, and `20.9` along it the
pipe's bound puts the circle `7.64e-14` off. That is **17.42** roundings, just
past the 16. The run
is named by the ball, and a ball and a pipe that only touch leave the walk two
pieces it cannot tell apart. This is the circle arm's row, not this one.
