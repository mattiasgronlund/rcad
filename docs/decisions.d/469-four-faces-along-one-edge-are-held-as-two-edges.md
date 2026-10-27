## 469. Four faces along one line are held already, as two edges on it, and one edge with four faces stays refused — 2026-10-27

*Asked by a brief written on rcad-brep's side and pasted here by Mattias.
rcad-brep reads OCCT's `.brep` files. OCCT's checker, run through pythonocc,
calls two of its made plates valid, and rcad refuses both with
`ADraftEdgeIsNotWalkedByTwoCoedges`, count 4. That is the brief's report: the
two pins it names, `made-5c-touching-at-a-shared-vertex` and
`made-8b-two-holes-sharing-a-vertex` in rcad-brep's `tests/import.rs`, are on no
branch on this machine and were not read. rcad-brep's roadmap §10 row 3 records
Mattias's rule that rcad answers to STEP and to the kernel and never to
rcad-brep, so the brief asks whether STEP or rcad needs such solids, and asks
for a decision and not a build. Measured at `d01662b8`. The one thing added
under `crates/` is a test of what rcad-brep now leans on (§469.4).*

**Settled: rcad needs no change, because it already holds the shape.** A solid
that touches itself along a line is an answer the fuse writes (§39.4): the line
is two edges, each with two faces along it. `SolidBuilder` takes the same shape
from a caller who writes it that way. The brief's own L plate builds,
validates, measures 11 and is cut right, with either of the two ways of pairing
its four walls (§469.4). What rcad refuses is one way of **writing** the shape:
one edge with four faces along it. That refusal stays, and §17.4's first row
stands as written. It would be a second way of writing a shape rcad already
holds (§469.5), STEP forbids it inside a solid (§469.3), and of 431 real solids
two have such an edge and neither would import if it were allowed (§469.2).

So rcad-brep's answer is to split such an edge into two (`docs/consumers.md`).
The brief's candidate, *an even number of faces per edge, paired round it*, is
not needed: the pair it would store on each coedge is what the second edge
already is.

Two correctness questions turned up, and both are filed rather than reserved,
since §13 defers features and never correctness. In 21 files that import today,
two pieces of skin touch at a single corner (§469.6). And six faces round one
line, paired so that the pairs cross, pass `Model::validate` (§469.4).

### 469.1 The shape, and what refuses it today

An L-shaped plate with a square hole whose corner sits exactly on the L's inner
corner, pushed up into a solid. Going round the vertical line through that
corner, the quarters alternate material, air, material, air, and four faces
meet along the line: two walls of the notch and two of the hole. It has a
closed skin and a well-defined inside. Two holes touching at a corner, a hole
touching the outline at a point, two blocks united while touching along an
edge, and two bars lying side by side touching along a line are the same shape.

`SolidBuilder` refuses it in `check_every_edge_is_walked_once_each_way`
(`builder.rs`) when the line is written as one edge. An edge walked by any
number of coedges other than two is `ADraftEdgeIsNotWalkedByTwoCoedges {
count }`, and an edge two coedges walk the same way is
`ADraftEdgeIsWalkedTheSameWayTwice`.

The fuse does not always refuse it. Where four faces meet along one line of an
answer, `part_each_line_two_faces_rest_against_each_other_along`
(`fuse/assemble.rs`) splits the line into two edges on it, two faces each, where
it can pair them. The answer may then be one solid touching itself along the
line (§39.4, `a_prism_lying_on_one_edge.rs`). Where it cannot pair them it
refuses `ResultIsNotManifold`, by §17.4's first row. Two cubes touching along an
edge are refused that way, and whether two touching solids may ever be a fuse's
answer is §80.9's reservation.

### 469.2 Two real solids in 431, and neither would import without the rule

**What the corpus test already says.** `how_often_a_real_file_imports` does
count this refusal: it groups a builder refusal by its message, and on
`d01662b8` one file of 367 is stopped by it,
`kicad/Fuseholder_Blade_Mini_Keystone_3568.step` — *the edge described at
position 288 is walked by 4 coedges*. It counts only the refusal a file meets
first (§45.2), so it cannot say how many files hold such an edge behind another
refusal.

**What the files hold.** A script in the scratchpad (§469.8) reads each solid's
topology and nothing else: shells to faces to loops to oriented edges, counting
how many loop uses each `EDGE_CURVE` has inside one solid. It evaluates no
geometry, so no curve or surface this kernel does not hold can hide a file from
it. It walked 430 of the corpus's 431 `MANIFOLD_SOLID_BREP`s and
`BREP_WITH_VOIDS`: 387 of 388 KiCad solids in 323 files and all 43 NIST
solids in 44 files. The one it could not walk is
`Fuseholder_Schurter_0031.7701.xx.step` `#3981`, whose loop `#4025` has two
oriented edges that do not meet end to start.

| corpus | edges | used twice | used four times | used any other number of times |
|---|---|---|---|---|
| NIST | 25,451 | 25,451 | 0 | 0 |
| KiCad | 128,754 | 128,750 | 4 | 0 |

The four edges are two in each of two solids, and every face along them is flat:

| file | edges | what is there | refused first today by | behind that |
|---|---|---|---|---|
| `Fuseholder_Blade_Mini_Keystone_3568.step` | `#2972`, `#3601`, lines 3.685 long | two wedges of material touching tip to tip along the line, with air either side, so round it air, material, air, material — the brief's shape | this rule | at `z = 0`, faces `#123` and `#839` both face down and both walk edges `#250` and `#258` forwards. The reader does not turn a loop round for `same_sense` (`brep.rs`, the doc above line 261), so the builder refuses `ADraftEdgeIsWalkedTheSameWayTwice` there, however the line is written |
| `Nidec_Copal_SH-7010C.step` | `#981`, `#1888`, lines 0.2 long | two blocks touching along an edge: along `#981`, material at `x < 1.31, z < 3.55` and at `x > 1.31, z > 3.55`, read off the four faces' corners and which way each faces | `B_SPLINE_CURVE_WITH_KNOTS` | 137 such curves, a kind of curve this kernel holds no form of |

**So: 2 solids in 431, in 2 files of 367, and no NIST part.** No edge anywhere
is used once, three times, or more than four. Reading such an edge as two, as
rcad-brep now will, would import **no** file of either corpus. The fuse holder's
skin is also walked the same way twice, which breaks STEP's own rule (§469.3)
and which the builder refuses whatever is done with the line, and the switch
holds curves rcad does not.

### 469.3 STEP forbids it inside a solid

ISO 10303-42 is not on this machine and was not read. Its wording is quoted
here from buildingSMART's IFC topology resource, whose `IfcClosedShell` and
`IfcManifoldSolidBrep` say they are *"adapted from closed_shell [and
manifold_solid_brep] defined in ISO 10303-42"* and carry the text *"according
to ISO/CD 10303-42:1992"*. The informal propositions of `closed_shell`
include:

> 1. Every edge shall be referenced exactly twice by the loops of the face.
> 2. Each oriented edge shall be unique.
> 3. No edge shall be referenced by more than two faces.
> ...
> 8. The closed shell shall be an oriented arcwise connected 2-manifold.

and `manifold_solid_brep` is *"a finite, arcwise connected volume bounded by
one or more surfaces, each of which is a connected, oriented, finite, closed
2-manifold"*, where *"each edge along the boundary of a face is shared by at
most one other face in the assemblage."*

**That largely settles the STEP side.** A `MANIFOLD_SOLID_BREP` with an edge
four faces share is not valid STEP, by propositions 1, 3 and 8 at once. The two
files of §469.2 break the standard, and rcad refusing them is the standard's
rule and not a gap in rcad. Reading them anyway, by splitting the edge as
rcad-brep will, would mean repairing a file to mean something it does not say.
That is §42.9's healing question, and the reader refuses rather than repairs.
Neither corpus uses any of STEP's non-solid representations for such a shape:
the keyword scan in §469.8 finds no `NON_MANIFOLD_SURFACE_SHAPE_REPRESENTATION`
anywhere.

What the standard says of the other way of writing it was not settled here. Two
edges on one line, each referenced by two faces, is what propositions 1 and 3
ask for; whether proposition 8 takes a skin that touches itself along those two
edges was not read, and rcad's STEP writer was not run on such a solid.

### 469.4 The same shape, written as two edges

**`crates/rcad/tests/what_rcad_brep_asks_through_the_front_door.rs` builds the
brief's plate through the `rcad` facade alone**: the outline `(0, 0) (4, 0) (4,
2) (2, 2) (2, 4) (0, 4)`, the hole from `(1, 1)` to `(2, 2)`, from `z = 0` to
`z = 1`, so the four walls meet along the line up through `(2, 2)`.

| how the line up `(2, 2)` is written | `SolidBuilder::build` | `Model::validate` | volume | less the box `[1.5, 2.5]² × [0.5, 1.5]` |
|---|---|---|---|---|
| one edge, four walls along it | `ADraftEdgeIsNotWalkedByTwoCoedges { count: 4 }` | | | |
| two edges: the notch's two walls, the hole's two | builds | passes | `11`, error `0` | `10.75`, error `0` |
| two edges: the two walls either side of each quarter of plate | builds | passes | `11`, error `0` | `10.75`, error `0` |

The box holds the two half-unit cubes of plate either side of the line at the
top, so cutting it out crosses the line where the plate touches itself.

**Why two pairings, and why neither needs geometry.** Going round a line in a
solid, the faces along it alternate in which way they walk it, as
`part_each_line_two_faces_rest_against_each_other_along`'s doc says. So round a
line with four faces, two walk it up and two down. Pairing each one going up
with one going down can be done two ways, and those are the two rows above: one
pairs walls with air between them, the other walls with plate between them. The
third way to pair four faces would pass two faces through each other, and it
pairs two faces walking the same way, which the builder refuses as
`ADraftEdgeIsWalkedTheSameWayTwice`. So with four faces, which way each face
walks the edge is all a reader needs to split it, and either split is sound.
The fuse writes both: a bar lying on a wall keeps the wall's two pieces as one
pair, with the wall's material between them (`the_two_pairs_along`), and an L
plate less a prism lying in its corner keeps each input edge's two faces as a
pair, with air between them (`pair_each_line_one_solid_touches_itself_along`).

**With six or more faces round one line, a pairing by direction alone can
cross, and nothing refuses it.** A probe, not kept, built `[0, 6]²` less three
holes with a corner at `(3, 3)`, from `z = 0` to `z = 1`: a triangle over 45° to
90°, one over 180° to 225°, and the square `[3, 4] × [2, 3]`. Six walls meet
along the line, alternating up and down. It paired them three ways: walls with
plate between them, walls with air between them, and the walls at 0° and 180°,
45° and 225°, 90° and 270°, whose pairs cross. **All three build, pass
`Model::validate` and measure 34**, and a box cut off its far corner and one cut
across the line came back `33.75` for each.

Whether a crossing pairing is a sound solid is not known. Its edges' two faces
are not next to each other round the line, and anything that reads *the face
across an edge* as the next face round it would be misled. No real file has six
faces along an edge (§469.2), but `SolidBuilder` is public, so that is no
answer (§16.10). It is filed (§469.9).

### 469.5 Why one edge with four faces stays refused

It would be a second way of writing a shape rcad already holds, and every place
that asks which face is on the other side of an edge would have to learn which
of three it means. `the_face_across` in `holds.rs` takes the first face along
the edge that is not the one it was asked about, which with four faces is the
first of three. `stored_surfaces_either_side` in `view.rs` refuses three or more
faces by name, `AnEdgeWithoutTwoFacesAlongItCannotBeWalked`. And a saved model
is checked at the door for two coedges walking each edge opposite ways
(`check_shape_is_possible`, `validate.rs`). That door takes no `Policy` (§44.5),
so it could not work the pairs out from geometry: the saved form would need each
coedge to name its partner. That is what §17.4's first row means by changing
what a `Model` is, everywhere. Written as two edges, the edge **is** the pair,
and none of that moves.

The brief also asked for the call sites to be counted. An audit of them ran in
this session and stopped at its context budget; its counts were rough and were
not kept, so none is given here. The three places above were read at
`d01662b8`.

### 469.6 rcad is looser than STEP at corners, and nothing has asked whether that is safe

The brief's last point is that a corner has the same problem: two blocks
touching at one point. The script of §469.2 also counts, for every vertex, the
corners of faces at it, joined wherever two of them share an edge that ends
there. One group is one fan of faces round the vertex. Two groups is a corner
where two pieces of skin touch at one point, which proposition 8 forbids.

**48 KiCad solids have such a corner, in 48 files, and 21 of those files import
today**, every one of them pinned in `KICAD_FILES_THAT_IMPORT`. In
`LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step`, vertex `#74` at `(-1.23, -1.27,
3.0)` is a corner of face `#59`'s outline, and the round hole in the same face
(loop `#87`) passes through the same `VERTEX_POINT`: a hole touching the outline
at one point. The reader makes one corner of one `VERTEX_POINT`
(`describe_the_vertex`, `brep.rs`), so the imported `Model` holds one vertex
with two fans of faces round it.

`Model::validate` asks of a vertex only that its point is finite and that some
edge ends there (`check_vertex`, `validate.rs`). The plate of §469.4 shows the
rest of the sweep does not ask either. Paired across the air, at each end of the
line the top face's outline corner joins the two notch walls by the edges they
share there, and its hole corner joins the two hole walls, which is two groups.
Paired across the plate it is one. Both pass.

**This is not an argument for one edge with four faces,** and it does not change
§469's verdict. It is a correctness question with no answer: whether anything
rcad computes on such a solid assumes one fan per corner — a fuse, the mesh,
which side of the skin a point is on, the volume, the STEP writer. 21 files of
the corpus reach every one of those today. §13 defers features and never
correctness, so it is filed (§469.9) rather than reserved.

### 469.7 What would reopen this

A STEP file cannot, by §469.3: one that holds such an edge is not valid STEP. A
consumer that needs the shape does not either, since rcad holds it. A consumer
that needs **one edge** with four faces, and cannot write two, would; none has
been named. The questions of §469.4 and §469.6 may narrow what rcad takes, but
they are about solids rcad already holds, not about this refusal.

Whether a fuse may answer with two solids touching along a line is §80.9's
reservation, and nothing here moves it. If a consumer for that comes, Manifold's
way — every touching case decided one way or the other by a fixed tie-break,
never held — is the other answer to weigh. OCCT and ACIS hold one edge with four
faces, Parasolid as a general body and not a solid, and CGAL's Nef polyhedra
hold anything. None of that was checked here, and none of it is a reason: §1.7's
criteria are about answers.

### 469.8 What ran this

- `mise exec -- just t how_often_a_real_file_imports --no-capture`, on
  `d01662b8` with the corpus at `../rcad-step-corpus`: the tally and the
  per-file lines quoted in §469.2.
- `edge_uses.py`, in the first half of this session's scratchpad and not kept:
  every solid's edge uses, its corners in two fans, its vertices at one point
  and its line edges on one segment (none of the last two in either corpus), and
  every representation keyword. `faces_on_edge.py` and `corners_at.py` beside it
  printed the faces along `#2972`, `#3601`, `#981`, `#1888`, `#250` and `#258`,
  and the corners at `#74`, which is where every row of the two tables of
  §469.2 comes from.
- The 21 files: the 48 names the script flagged, intersected with the 165
  names in `KICAD_FILES_THAT_IMPORT`.
- The STEP wording: buildingSMART's `IFC4.x-development` repository on GitHub,
  `IfcClosedShell.md` and `IfcManifoldSolidBrep.md`, read 2026-10-27.
- `mise exec -- just t -p rcad --test what_rcad_brep_asks_through_the_front_door`:
  the table of §469.4, three tests.
- The six-face probe: a test file added to `rcad-topo`'s `merged` target, run
  with `just t -p rcad-topo -E 'test(/probe_four_faces_on_one_line/)'
  --no-capture`, and removed. It printed the build, `validate`, the volume and
  both cuts for each pairing, which is where §469.4's last paragraphs come from.

### 469.9 The rows

- **Q2911**, tier 1: whether anything rcad computes assumes one fan of faces
  round each corner, run on the 21 importing files' shape and on §469.4's plate
  paired across the air (§469.6).
- **Q2912**, tier 1: whether a crossing pairing round a line of six faces is a
  sound solid, and if not, what should refuse it (§469.4).
