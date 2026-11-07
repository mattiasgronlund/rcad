## 488. The STEP writer keeps writing a solid that touches itself along a line, and its docs will say which rules the file breaks — 2026-11-07

*Asked by Q3032, which §469.3 filed when it found that no `MANIFOLD_SOLID_BREP`
holds such a solid, however its line is written, and that rcad's writer writes
one all the same. Measured at `a025eef79` with a probe test that was not kept
(§488.6). Two lanes ran the row: the first read the standard and measured, and
handed over at its context budget before writing anything; the second ran the
probe and the script again, checked the quotes against the pages, and wrote
this. Nothing under `crates/` changes
here. The change this needs is a row of its own (§488.7).*

**Settled: the writer keeps writing such a solid as it does today, and its docs
say which of STEP's rules the file breaks.** Q3032 offered three answers, and
the second one does not exist. Among the forms of ISO 10303-42 read, no entity
that says *solid* holds a skin touching itself along a line (§488.2). Refusing
it by name in `CannotWrite` would make a valid answer of the kernel, one a
designer draws on purpose, impossible to write, and would take it out of the
round trip that is the writer's reason to exist (§488.3). And the writer's files
were never conforming STEP: every one breaks a rule of ISO 10303-43, and 21
files rcad reads and writes back break `face`'s IP2 (§488.4). What the writer
promises is that rcad's reader reads the file back as the same solid, and for
this shape it does (§488.1).

So the file says what the model holds, with nothing repaired: two `EDGE_CURVE`s
up the line between the same two `VERTEX_POINT`s, each on a `LINE` of its own
and each walked by two faces. The docs, and a test pinning that round trip, are
the row (§488.7).

### 488.1 What the written file holds along the line

The probe built §469.4's plate both ways its four walls pair. It also built the
fuse's own answer that touches itself (§39.4): the L plate less the prism lying
in its corner, with the coordinates of
`crates/rcad-topo/tests/a_prism_lying_on_one_edge.rs` lines 696 to 736, as a
`Difference` with the plate first. It wrote each model with `rcad_step::write`,
read the text back with `StepFile::read` and `Import::read`, and validated what
came back. A script then read the file's topology along the line: the
`VERTEX_POINT`s on it, the `EDGE_CURVE`s between two of them, and the faces
whose loops walk each one.

| solid | the line | edges on it | the faces along each, by which way they face | read back |
|---|---|---|---|---|
| plate, walls paired across the air | `#120` at `(2, 2, 0)` to `#33` at `(2, 2, 1)` | `#228` on `LINE #227`, `#308` on `LINE #307`, both from `#120` to `#33` | `#228`: `+y` and `+x`; `#308`: `-y` and `-x` | 1 solid, volume `11`, error `0`, 18 vertices, valid |
| plate, walls paired across the plate | the same | `#228`, and `#242` from `#33` to `#120` on `LINE #241` | `#228`: `+y` and `-x`; `#242`: `+x` and `-y` | the same |
| L plate less the prism | `#440` at `(1, 1, 0)` to `#554` at `(1, 1, 0.5)` | `#555` on `LINE #552`, `#653` on `LINE #652`, both from `#440` to `#554` | `#555`: `+y` and `+x`; `#653`: `-x` and `-y` | volume `2.5`, error `0`, among the 3 solids written |

Each plate is one `MANIFOLD_SOLID_BREP`, `#341`, of one `CLOSED_SHELL`, `#340`,
of 12 faces. The L file holds three solids because the fuse leaves its two
inputs in the model (`fuse/mod.rs`, lines 119 to 121) and `Model` has no way to
take a solid out. The answer is `#673`, of one `CLOSED_SHELL` of 12 faces.

So the writer writes the model's two edges as two edges and rcad's reader reads
back the shape that went out. What another system makes of the file was not
tried. The second reader in rcad's own tests, `step-io`, compares with this one
at Part 21's layer of entities and attributes
(`a_second_reader_reads_what_this_writer_wrote.rs`), and asks nothing about
topology.

### 488.2 No solid STEP allows holds it

The forms considered are every way ISO 10303-42:2021 offers to say *a solid*.
That is the `ONEOF` of `solid_model` (§6.4.1): `csg_solid`,
`manifold_solid_brep`, `swept_face_solid`, `swept_area_solid`,
`swept_disk_solid`, `solid_replica`, `brep_2d` and `trimmed_volume`. The two
subtypes of `manifold_solid_brep`, `brep_with_voids` (§6.4.3) and
`faceted_brep` (§6.4.4), inherit its rules. Beside them are the models of §6.2
that are not solids. For each, here is why it does not hold the shape.

**A skin of faces, `manifold_solid_brep` or either subtype.** Near any point of
the line, the skin is four half-planes meeting along it. Take the line out of a
small ball round the point, and the skin left inside the ball is four pieces,
where a 2-manifold leaves two. §6.4.2 bounds the solid by closed 2-manifolds,
whatever the line is written as:

- one edge with four faces along it breaks `closed_shell`'s IP1 and IP3
  (§469.3), and rcad refuses it on the way in (§469.5);
- two edges sharing their two vertices, as the writer writes it, lie on each
  other end to end, which IP5 (*"Distinct edges do not intersect, but may share
  vertices"*) and §6.4.2's *"do not intersect except at their boundaries"*
  forbid;
- two edges with vertices of their own at the same points still lie on each
  other, and now share no vertex either.

**The same skin with one pair of faces moved off the line by a hair.** That
writes a different shape. It is a repair, which §42.9 refuses on the reading
side for the same reason.

**Two `MANIFOLD_SOLID_BREP`s.** The plate cut at `x = 2` gives the box
`[2, 4] × [0, 2] × [0, 1]` and the rest, each a valid solid, touching each other.
But the file then says two solids where the model holds one. rcad's reader would
read back two, so the round trip could no longer say *the same solid*, and the
writer would have to make the faces along the cut, which the model does not
hold.

**CSG, swept and replica solids.** A `csg_solid` needs a tree of primitives, and
a swept solid a profile and a path. A fused answer in rcad holds neither, only
its faces, and rcad's reader reads none of them: `EXTRUDED_AREA_SOLID` and
`CSG_SOLID` appear nowhere in `crates/rcad-step/src`. The CSG primitives and
`half_space_solid` each say one fixed shape, or half of space, and make an L
plate only through a `csg_solid`'s tree. The plate is a sweep, of
the L's outline less the hole, a profile touching itself at the point `(2, 2)`.
Whether Part 42 lets a swept area's bounds touch at a point was not read.

**The models of §6.2 that are not solids.** §6.2 says that
`shell_based_surface_model`, `face_based_surface_model`, the two wireframe
models, `geometric_set` and `geometric_curve_set` *"can be used for the
communication of incomplete models or non-manifold objects"*. They are skins,
wires and sets, not solids, so such a file says *a surface* where the model
holds a solid. rcad's reader walks past `SHELL_BASED_SURFACE_MODEL`
(`SHAPES_THIS_READER_DOES_NOT_TAKE`, `import.rs`) and refuses a file with no
solid as `Refusal::TheFileHoldsNoSolid`, the refusal `CannotWrite::TheModelHasNoSolid`'s
doc cites as its reason.

**Not read:** `tessellated_solid`, which is outside `solid_model`'s `ONEOF`.
It is triangles, so it carries no curved face exactly, and whether its rules let
its skin touch itself was not read. For a solid whose faces are all flat, as
the plate's are, it is the one form left open.

**So it is a limit of STEP, not a gap in the writer** (§16.10): among the forms
read, the shape has no faithful form as one solid. The unread form is a reason
to reopen this (§488.5), not a fix: rcad's reader does not read
`tessellated_solid` either, so the round trip would be lost with it.

### 488.3 Why not refuse it by name

- **It is a valid answer, made on purpose.** The fuse writes it (§39.4),
  `SolidBuilder` takes it (§469.4), and a designer draws an L plate less a
  pocket in its corner. A refusal would mean rcad reads, builds and fuses a
  solid it cannot write.
- **It would take the shape out of the oracle.** The writer exists to be *an
  oracle the reader does not supply itself* (`write.rs`, the module doc, lines
  7 to 9). §471.2's round trip of the plate is how it was shown that such
  solids survive the trip, and a refusal ends that check for them.
- **The file says exactly what the model holds.** Nothing is moved, joined or
  dropped. §469 settled that rcad holds the shape as two edges on the line, and
  the file shows those two edges.
- **Telling it apart would be code no one has asked for.** Two straight edges
  of one shell between the same two vertices always lie on each other, so for
  a straight line the test is cheap. Two curved ones between the same two
  vertices may be the two halves of a circle, so for a curved line it needs
  geometry. No consumer of rcad's files has asked for files that conform
  (§13).

### 488.4 The writer's files were never conforming STEP

**Every file breaks a rule of ISO 10303-43.** `representation_item` (§4.4.16)
has `WR1: SIZEOF(using_representations(SELF)) > 0`, and its text says the item
*"shall participate in at least one"* representation. A `MANIFOLD_SOLID_BREP`
is a `representation_item`, through `solid_model` and
`geometric_representation_item`. The writer puts its solids in no
representation: the written plate holds no `PRODUCT`, `SHAPE_REPRESENTATION` or
`SHAPE_DEFINITION`, and its one line naming a `REPRESENTATION` is the context
`#5`. In the code, `Writing::new` and `Writing::units` (`write.rs`, lines 269 to
337) write the header, the units and that context, and `solid_from_shell`
writes the shell and the solid and nothing that holds them (lines 486 to 487).
So every file the writer writes breaks a formal rule, the cube of `rcad`'s
crate example included, and its header names `AUTOMOTIVE_DESIGN` all the same.

**21 files rcad reads break `face`'s IP2, and the writer writes them back.** In
the LED files of §469.6 a hole touches its face's outline at one vertex, which
IP2 forbids. rcad reads all 21 files, and writes all 21 back with their corners
in two fans (§471.3).

**What the writer answers to has always been rcad's reader.**
`CannotWrite::TheModelHasNoSolid` is a limit because `Import::read` would
refuse the file, and `ASolidWithMoreThanOneShell` writes no `BREP_WITH_VOIDS`
because rcad's reader cannot read one back (`write.rs`, lines 85 to 111). To
refuse the line while writing both of the above would be inconsistent, and to
refuse those too would refuse to write back files rcad reads.

**Not filed: a representation for the solid**, a `SHAPE_REPRESENTATION` or an
`ADVANCED_BREP_SHAPE_REPRESENTATION` over the solids, and the `PRODUCT`
structure above it. That is what another system needs to read rcad's files,
and no consumer that hands them to one has been named (§13). The docs row
(§488.7) has the writer's docs say so.

### 488.5 What would reopen this

- **A consumer that hands rcad's files to another system.** The writer would
  need §488.4's representation first, and the line would then be a question of
  what that system does with two edges on one line, which nobody has tried.
- **A reader that refuses such a file**, rcad's own included.
- **A reading of `tessellated_solid`'s rules that lets the skin touch itself**,
  for solids whose faces are all flat (§488.2).
- **A real file that writes two edges on one line.** Neither corpus holds one:
  §469.8's `edge_uses.py` found no line edges on one segment. That was not run
  again here.

### 488.6 What ran this

- The probe: `crates/rcad/tests/probe_q3032_the_line_written.rs`, which copies
  the plate builder of `crates/rcad/tests/what_rcad_brep_asks_through_the_front_door.rs`
  (lines 138 to 303), run with `mise exec -- just t -p rcad --test
  probe_q3032_the_line_written --no-capture` and removed. It wrote the three
  files to the session's scratchpad and printed, for each, the faces written,
  the vertices, the volume out and what came back, one line each:
  `written 12 faces, 18 vertices in the model; volume out (11.0, 0.0); back 1
  solids [(11.0, 0.0)], 18 vertices` for both plates, and `written 12 faces, 37
  vertices in the model; volume out (2.5, 0.0); back 3 solids [(3.0, 0.0),
  (1.5, 0.0), (2.5, 0.0)], 37 vertices` for the L. Both lanes ran it at
  `a025eef79`, and both printed these lines.
- `along_the_line.py`, in the same scratchpad and not kept, given a file and the
  line's `x` and `y`: the `VERTEX_POINT`s on the line, each `EDGE_CURVE` between
  two of them with its curve, and each face's `ORIENTED_EDGE` on one of those
  edges, with the way the face's plane faces. Every number in §488.1's table is
  its print.
- `grep -c "PRODUCT\|SHAPE_REPRESENTATION\|SHAPE_DEFINITION"` on the written
  plate printed `0`.
- The standard, read 2026-11-07 as steptools.com serves it: ISO 10303-43's
  `representation_item` in
  `https://www.steptools.com/stds/smrl/data/resource_docs/representation_structures/sys/4_schema.htm`,
  and ISO 10303-42:2021's §6.2, §6.4.1 to §6.4.4 in `6_schema.htm` under
  `https://www.steptools.com/stds/smrl/data/resource_docs/geometric_and_topological_representation/sys/`.
  `closed_shell`'s propositions are quoted from §469.3.

### 488.7 The rows

- **One row, filed by Q3032's lane**, tier 1: the writer's docs in `write.rs`
  say that a solid touching itself along a line is written as the model holds
  it, as two edges on the line, that such a file breaks `closed_shell`'s IP5 and
  `manifold_solid_brep`'s rule on edges, and that every file breaks
  `representation_item`'s WR1 (§488.4), with this section as the reason. The
  places are the module doc above line 33 and `solid_from_shell`'s doc, lines
  339 to 369. A test in `what_rcad_brep_asks_through_the_front_door.rs` pins the
  round trip of §488.1: both plates written and read back as one valid solid of
  volume `11` and 18 vertices, so a refusal added later has to change a test
  first.
