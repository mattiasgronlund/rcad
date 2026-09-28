## 329. Which way a torus face points: out of the tube, and the mesher is the one turned round — 2026-09-28

*Row Q339 (`lane-q339-which-way-a-torus-faces`, off `0a7142ff`). Diagnosis and
pins; no `src/` changed. Q103 (validate refuses a three-quarter bend) and Q102
(whose first lane turned `classify.rs`'s torus gradient round, §323.4) both
rest on which way a torus's own normal points. It points **out of the tube**:
the standard, the reader, `Model::validate` and `classify.rs` as `0a7142ff`
has it all say so, and so do three files in the corpus. The mesher alone says
into the tube, and the kernel's own bends are built turned round to suit it.*

  329.1   The four legs, checked
  329.2   The experiment
  329.3   What each configuration says
  329.4   The corpus, which the kernel did not make
  329.5   What must change, and what must not
  329.6   The pins

### 329.1 The four legs, checked

**(a) holds.** The torus arms of `validate.rs`'s `gradient_of`, `mesh.rs`'s
`gradient_of` and `classify.rs`'s `gradient_at` all compute
`(w·w)(x·x − R² − r²)x + 2R²(x·w)w`, which is `(x·x + R² − r² − 2R²)` written
the other way — the quartic's gradient over four. `Torus::side_of` is `Neg`
inside the tube (its doc works the factorisation `(s − R)² + a² − r²`). At
`(3, 0, 0)` on the ring of four and tube of one the gradient is `(−24, 0, 0)`,
towards the axis, which is out of the tube at the inner equator.

**(b) holds.** `check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind`
takes `gradient_of(surface) * orientation.sign()` as a face's outward, for the
face and for the face across. It skips a face whose every corner is on the
partner — so **a bend's caps are never asked**, since each cap's one corner is
on the wall. Only the wall's coedges are read.

**(c) holds.** `tests/rings/mod.rs` and `holds.rs`'s `a_piece_of_pipe` use the
wall `Reversed`. A cap circle is stored running the way `Torus::point_at`'s
first coordinate goes (`the_way_it_is_parametrised_about` crossed into the
gradient), which at `(3, 0, 0)` is `−z`. The near cap, facing `−y`, walks it
`Reversed`: `+z` there, which turns left-handed about `−y`. The wall walks it
`Forward`, `−z`, and `(−x) × (−z) = −y` points away from the wall's own
material. So every loop turns the wrong way about the solid's true outward
normal, caps included, and the wall is used turned round; the closed-edge rule
reads only the wall and sees two turns cancel.

In the chart `(tube angle, ring angle)`, `∂tube × ∂ring` at the outer equator
is `r·w × (R + r)(w × ê) = −r(R + r)ê`: **into** the tube. For every other
curved surface here `∂first × ∂second` agrees with the gradient — a ball's
`∂lon × ∂lat` and a pipe's `∂angle × ∂height` both point out. The torus is the
one whose chart was ordered tube-first (for its seam, `Torus::point_at`'s doc)
and turned its handedness round with it. `mesh.rs`'s `hand_over` turns a face's
triangles by `face.orientation()` alone, and the triangulator draws every
triangle anticlockwise in the chart, so a torus face used forwards meshes
facing into the tube.

**(d) holds, on the corpus rather than on the standard's own words.**
`brep.rs`'s `surface_of_the_face` reads `same_sense .T.` as `Forward` for every
curved surface. The standard's sentence on the sense could not be read here:
the ISO 10303-42 sample on `cdn.standards.iteh.ai` is a scanned image, and the
STEP Tools schema pages carry no maths. What was read is the parametrisation
`C + (R + r cos v)((cos u)x + (sin u)y) + r(sin v)z`, ring angle first, whose
`σ_u × σ_v` at `v = 0` is `(R + r)r·ê`, out of the tube; and Open CASCADE's
`Geom_ToroidalSurface` reference, verbatim: "The parametrization of the surface
in the U direction is defined such as the normal Vector (N = D1U ^ D1V) is
oriented towards the 'outside region' of the surface." 329.4 is the evidence
that does not lean on either.

### 329.2 The experiment

A quarter bend of ring four and tube one about `+z`, from a cap on `y = 0`
(facing `−y`) to one on `x = 0` (facing `−x`), encloses `2π² = 19.739` by
Pappus. Three descriptions:

- **the file as the standard says**: STEP text, wall `TOROIDAL_SURFACE`
  `same_sense .T.`, every loop right-handed about its face's outward normal;
- **the file the kernel's way**: the same text with the wall `.F.` and all four
  `ORIENTED_EDGE` flags turned over;
- **`rings::a_quarter_bend(4, 1)`**, the kernel's own builder, read into the
  test by `#[path]`.

Each was asked `Model::validate` (through `Import::read`, which runs it),
`measure` at deflection `0.01`, and `classify_point` at eight exact points plus
the tube's middle at `45°`, `(2√2, 2√2, 0)` rounded. Four configurations:
`0a7142ff`; `0a7142ff` with `hand_over`'s `inside_out` also true for a torus
face used forwards (one line, `!= matches!(stored, Surface::Torus(_))`); each of
those with `lane-q102-a-point-a-ray-can-leave` merged in. All four were run in a
throwaway detached tree under this lane's `target/`, never on the branch, with a
probe version of `tests/which_way_a_torus_faces_normal_points.rs` that printed
rather than asserted: `mise exec -- just t -p rcad-step --test
which_way_a_torus_faces_normal_points probe_all --no-capture`.

### 329.3 What each configuration says

"Past a cap" is `(−1, 3.5, 0)` and `(3.5, −1, 0)`, both outside the bend. The
other six exact points — `(4, 1, 0)`, `(3.5, 3.5, 0)`, `(1, 4, 0.5)` inside,
`(4, 4, 0)`, `(5.5, 1, 0)`, `(1, 1, 0)` outside — came back right in every row
that got as far as classifying.

| classify | mesh | file as the standard says | kernel's way (file and `rings`) |
|---|---|---|---|
| `0a7142ff` | `0a7142ff` | refused, `FacesDoNotAgreeWhichWayTheyFace` | validates, `19.706 ± 0.392` (file) / `19.707 ± 0.398` (`rings`); past a cap **Inside** |
| `0a7142ff` | turned | validates, `19.706 ± 0.392`; **all right** | refused, `FacesDoNotAgreeWhichWayTheyFace` |
| Q102 | `0a7142ff` | refused, `FacesDoNotAgreeWhichWayTheyFace` | validates, same volumes; all right |
| Q102 | turned | validates, `19.706 ± 0.392`; past a cap **Inside** | refused |

The middle of the tube at `45°` is `CannotFireARayAtAFacesSurface` without Q102
and `Inside` with it, whichever way the mesh turns — that is Q102's other
change, the ray fired from an exact point beside a box (§323.2), and it is
right in both.

So there are two consistent end states and one of them is wrong about files.
Turning the mesh and keeping `classify.rs`'s sign reads the standard's bend
right in every question asked; turning `classify.rs` and keeping the mesh reads
the kernel's bend right and refuses the standard's. The refusal in the first
row passed the closed-edge rule to get there: `check_closed_edges_are_walked_*`
runs before `check_shells_and_solids`, whose curved-volume sum is what refuses.

### 329.4 The corpus, which the kernel did not make

367 corpus files, 87 with a face on a `TOROIDAL_SURFACE` — 420 such faces
`.T.` and 301 `.F.` (a throwaway regex count over every `ADVANCED_FACE`,
`target/tori.py` in the lane's tree and not committed). Importing those 87 on `0a7142ff`: **none imports**. Three are refused
`FacesDoNotAgreeWhichWayTheyFace` — `Relay_1-Form-A_Schrack-RYII_RM5mm`,
`Relay_1-Form-B_Schrack-RYII_RM5mm` and `Relay_SPDT_Finder_34.51_Vertical`,
with 2, 2 and 4 torus faces, all `.T.` — and the other 84 by names that come
earlier and have nothing to do with a normal (42 `B_SPLINE_CURVE_WITH_KNOTS`,
26 corners off their curved face, and so on).

With the mesh turned and nothing else, **those three import** — so they
validate — and every other file is refused by exactly the name it was refused
by before. Two `nist_ftc_11` spellings are refused
`AClosedEdgeIsWalkedAgainstItsSurface` in both; that is before the mesh and was
not traced.

Three real files, written by someone else's kernel, agree with the standard's
reading, the reader's, `validate.rs`'s and `classify.rs`'s, and disagree only
with `mesh.rs`.

With the mesh turned alone, the whole workspace ran 2617 tests and failed 42
(`mise exec -- just t --workspace --no-fail-fast` in the throwaway tree). Nearly
all are bends built the kernel's way — `rings::a_quarter_bend` panics on
`FacesDoNotAgreeWhichWayTheyFace` — and the rest are corpus pins that move
because the three relays now import (329.5).

### 329.5 What must change, and what must not

**One row, because its halves cannot land apart.** Turning the mesh refuses
every kernel-built bend, and rebuilding the bends without turning the mesh
refuses every rebuilt bend.

1. **`mesh.rs`**: a torus face's triangles turned against its chart, so that a
   face used forwards faces out of the tube. `hand_over`'s `inside_out` is the
   one line of `mesh.rs` that reads `face.orientation()`, and turning it is
   what 329.3's "turned" rows ran.
2. **Every bend the kernel builds**, rebuilt with the wall `Forward` and every
   loop walked the other way — which is each closed-edge coedge turned over,
   caps and wall alike (reversing a wall's wire and turning its circles over
   are the same thing). `tests/rings/mod.rs` (`a_bend`,
   `the_top_half_of_a_ring`) and `holds.rs`'s test builders, and every test
   file that builds a torus face inline. No production code builds one: every
   `Torus::new` under `src/` outside the reader sits below its file's
   `#[cfg(test)]` (`holds.rs`, `mesh.rs`, `fuse/{select,arrange,assemble}.rs`,
   `rcad-mesh`'s `triangulate.rs`), and no sweep in `src/` makes a bend.
3. **`classify.rs`: no change on `0a7142ff`; undo Q102's sign if it lands
   first.** Its torus arm of `gradient_at` is right. Q102's beside-a-box route
   is independent and right and should be kept; only the negation of the torus
   arm (`5e856162`) goes.
4. **Not to change**: `validate.rs` (`gradient_of`, the closed-edge rule),
   `mesh.rs`'s own `gradient_of` (it names which way an edge is stored, the
   same as `validate.rs`'s), `rcad-geom`'s `Torus`, and `rcad-step`'s reader.
5. **Corpus pins move**: `how_often_a_real_file_imports.rs` and
   `a_real_file_written_out_and_read_back_is_the_same_solid.rs` count three
   more files imported; the writer fuzzer's seeds hold kernel-built bends
   (`every_model_the_writer_fuzzer_kept_is_written_again.rs`, two tests).
6. **Q103's premise has to be re-derived after the fix.** A three-quarter bend
   described the kernel's way is not a valid solid by this section, so
   "validate refuses a valid three-quarter bend" may have been validate
   refusing a bend turned round in one place and not the other. Q103's
   `a_bend_walked` helpers were not read.

### 329.6 The pins

`crates/rcad-step/tests/which_way_a_torus_faces_normal_points.rs`, three tests,
each passing on `0a7142ff` and each failing with 329.5's step 1 alone (checked
in the throwaway tree):

- `a_bend_written_as_the_standard_says_is_refused_by_the_mesh_today` — moves to
  "validates, encloses `2π²`, places every point right".
- `a_bend_built_the_kernels_way_validates_and_puts_a_point_past_its_cap_inside`
  — asserts both points past a cap `Inside`, which is wrong; with step 1 alone
  it panics in `rings::a_quarter_bend`, and with step 2 it is the standard's
  bend and moves to `Outside`.
- `three_relays_in_the_corpus_are_refused_by_the_mesh_today` — moves to
  "imports".
