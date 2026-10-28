## 470. Six walls round one line answer alike however they are paired, and the fuse runs three faces through each other there — 2026-10-28

*Answers Q2912, the second question §469.4 filed: whether a solid whose faces
round one line are paired so that the pairs cross is sound, and if not, what
should refuse it. Measured by Q2912's read-only lane at `1085ae50`, with two
probe test files in a scratch worktree, not kept (§470.6).*

**Settled, as far as it was run: a crossing pairing answers what the other two
pairings answer.** Two plates with six walls round one line, each built three
ways, all build, validate and measure right, and no point query and no fuse
tried answers differently for the crossing pairing (§470.1). Reading the fuse
says why: it never reads which faces a caller paired round a line of six
(§470.2). So no `SolidBuilder` check is filed, since no wrong answer was found
for one to stop.

That is a finding and not a proof. *No query can tell them apart* is not shown,
and one reader of *the face across an edge*, the fuse's rule for placing a
sliver, was worked by hand and not run, and by hand it does tell them apart
(§470.3). It is filed, so the question stays open there (§16.10).

**Found instead: the fuse writes three faces that run through each other.** A
plate less three ordinary prisms meeting at one line comes back with the walls
either side of the line joined into one face three times over, with no edge on
the line, and `Model::validate` passes it (§470.4). That is a wrong answer from
shapes anyone can make, class B. A plate whose six walls lie on six planes is
refused by every fuse that keeps the line (§470.5).

### 470.1 Two plates, three pairings each, one answer

Both plates are `[0, 6]²` from `z = 0` to `z = 1`, less three holes with a
corner at `(3, 3)`, so six walls meet along the line up through `(3, 3)`,
alternating up and down. Each was built through `SolidBuilder` with that line
as three edges, paired three ways: walls with plate between them, walls with air
between them, and crossing.

| plate | its holes | walls at | crossing pairs | volume |
|---|---|---|---|---|
| G1, §469.4's | triangles `(3,3) (4,4) (3,4)` and `(3,3) (2,3) (2,2)`, square `[3, 4] × [2, 3]` | 0°, 45°, 90°, 180°, 225°, 270° | each on one plane | `34`, error `0` |
| G2 | triangles `(3,3) (5,4) (3,4)`, `(3,3) (2,2) (1,4)`, `(3,3) (4,3) (3.5,1.5)` | 0°, 26.6°, 90°, 153.4°, 225°, 288.4° | no two on one plane | `32.75`, error `0` |

- All six build and pass `Model::validate`.
- `classify_point` at 868 points round the line, 144 angles by two radii by
  three heights and four on the line, gives the same answer for all three
  pairings, on each plate.
- 17 fuses with boxes and wedges, each run against all three pairings of each
  plate, give the same answers down to the faces along the line: across the
  line, through it, on it, touching it from a sector of air or of plate, and
  away from it. On G1 they answer, with the volumes due. On G2 every one that
  keeps the line refuses, alike for all three pairings (§470.5).

### 470.2 Why the fuse cannot tell the pairings apart

Read, not run. The fuse's arrangement makes one edge of the line, with six
darts along it. It reads an input edge's two faces only to pair a line of
exactly four darts (`the_pairs_round_two_input_edges` and `runs_along` in
`fuse/assemble.rs`). Round a line of four, the darts alternate in which way they
walk it, so the one pairing that would cross pairs two walking one way, which is
refused (§469.4). A line of six darts goes instead to
`part_each_line_more_faces_walk_into_pairs_on_one_plane`, which pairs by plane,
or to the refusal. So the caller's pairing is thrown away before anything could
read it.

### 470.3 The other readers of the face across an edge

Read, not run:

- `the_face_across` in `holds.rs` trims curved faces. A flat face with straight
  sides winds its own sides and does not ask.
- The callers in `validate.rs` and `mesh.rs` are on curved runs. Not all of them
  were read.
- `from_one_edge` and `beside_one_edge` in `fuse/select.rs` place a sliver as a
  last resort. Their doc reads the solid near an edge as the wedge between that
  edge's two faces, which holds only where the skin there is those two faces and
  nothing else. That is false on any line with four or more faces, whatever the
  pairing. Worked by hand on G2, with the sliver on the 0° wall and the point
  read at 180° in a hole: paired across the air the rule reads it inside, while
  paired across the plate or crossing it reads it outside, which is right. So
  here the pairing could matter, though the wrong reading is the air pairing's
  and not the crossing one's. Nothing has run it, so it has no class (§470.7).

### 470.4 Three prisms cut from a plate leave three faces running through each other

Take `[0, 6]² × [0, 1]` less G1's three holes as prisms from `z = -1` to
`z = 2`, in one difference or three in turn. Every pairing of the built G1 fuses
to the same answer as well.

- It comes back as one solid with 13 faces where 16 are due, and volume `34`.
- The walls on `y = 3`, `x = 3` and `y = x` each come back as one face running
  straight through the line `x = y = 3`, with no edge on it. The face on
  `y = 3` walks `(3,3,1) (2,3,1) (2,3,0) (3,3,0) (4,3,0) (4,3,1)`. So three
  faces cross each other along a line that is an edge of none of them.
- `Model::validate` passes it.
- Its volume, 868 point queries round the line and 9 fuses run on it after are
  right.
- After only the two triangles are cut, `y = x` is one face lying past the line,
  and that is right, because the plate there is a half-plane.

Where it goes wrong was read and not run.
`part_each_line_more_faces_walk_into_pairs_on_one_plane` pairs the opposite
walls, which lie on one plane and face one way, and `join_up_what_is_one_face`
then removes each pair's edge. Its doc gives the reason: two such faces lie
either side of the line, so the line is no boundary between them. That fails
where other faces walk the line too.

It is class B: a plate perforated in a checkerboard of triangles is a shape
people make (§440.3). Two rows are filed: the fuse, and `validate` passing what
the fuse wrote, the second after the first (§470.7).

### 470.5 Six walls on six planes are refused by every fuse that keeps the line

G2, in any of its three pairings, refuses:

- less the box `[5, 7]² × [0.5, 1.5]`, far from the line, and less
  `[2.5, 3.5]² × [0.5, 1.5]` across it: `CannotTellTheCellsOfAFaceApart`;
- united with the slab `[-1, 7]² × [1, 2]`: `ResultIsNotManifold`, count 6;
- touched by wedges: `Undecidable`.

G1, whose opposite walls lie on one plane, answers each of these. It is class C,
since only a hand-built solid has six walls on six planes round a line. Not run:
whether the plate less G2's holes as prisms is made by the fuse, which would make
it B.

### 470.6 What ran this

Two probe test files under `crates/rcad/tests/` of a scratch worktree made with
`git worktree add --detach` at `1085ae50`, both removed after. Copies are in the
dispatcher's scratchpad for this session, which a reboot wipes.

- `just t -p rcad --test q2912_probe --no-capture probe_q2912_build`: §470.1's
  table and the faces along the line for every pairing.
- `just t -p rcad --test q2912_probe --no-capture probe_q2912_points`: the 868
  points.
- `just t -p rcad --test q2912_probe --no-capture probe_q2912_fuses`: the 17
  fuses on both plates, and §470.5's refusals.
- `just t -p rcad --test q2912_probe_two --no-capture`: §470.4's plate made by
  the fuse, its faces and their walks, its points, and the 9 fuses after.

### 470.7 The rows

- **Q2918**, tier 1, wrong/B: the plate less three prisms comes back with three
  faces running through each other; trace it and keep the line as three edges
  (§470.4).
- **Q2919**, tier 1, wrong/B, after Q2918: `Model::validate` passes those
  faces; refuse them by name, or show what the check would cost (§470.4).
- **Q2920**, tier 5, refuses/C: six walls on six planes refused by every fuse
  that keeps the line (§470.5).
- **Q2921**, tier 1, no class: run the fuse's sliver rule along a line of six
  faces (§470.3).
- Q2912 is closed at `1085ae50`, the base its lane read.
