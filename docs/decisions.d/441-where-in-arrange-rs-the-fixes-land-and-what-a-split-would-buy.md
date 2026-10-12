## 441. Where in arrange.rs the fixes land, and what splitting it would buy — 2026-10-12

*Mattias asked on 2026-10-12, reading "Meets a running lane's write set" on the
queue page: would it not be a good idea to split
`crates/rcad-topo/src/fuse/arrange.rs`, now 15 344 lines, and can a split be
found that cuts the contention in a meaningful way? §424.5 had left the split
as the next step, and said it pays only if the new files' borders fall where
fixes land, which nobody had measured. This measures it.*

In short: the contention is real. At 11:39 UTC, Q1660's lane held back 78
ready rows, 31 of them tier 1; 58 of those met it on `arrange.rs`, the rest on
`assemble.rs`. Fixes land all over the file, with no hot spot. A split into
ten files, one per stage that `Arrangement::of` runs, would have let 57% of the
past rows' pairs run side by side, where today none can. But only if each row
names its file before the work. For only 22 of 43 landed rows did the row's own
text name every file its fix changed. And 27 of the 68 open rows hold the whole `fuse/` directory, which no
split narrows. So the split is not done now, and §424's order stands: Q1530 and
Q1531 first, because they need no row to guess. The ten files are written down
here, so the split is ready if §424.4's bars send it back.

  441.1   What was measured
  441.2   Where the fixes land
  441.3   The ten files
  441.4   What the split buys, if every row names its file
  441.5   Why it does not buy that now
  441.6   What is decided

### 441.1 What was measured

**The rows.** Every merge reachable from `origin/main` (at `944bc593`) since
2026-09-20 whose subject starts `Merge lane-`, `Merge merge-` or `Merge branch
'lane-`, and whose diff from its first parent changes `arrange.rs`. Its first
parent was `main` when it landed, so `git diff -U0 M^1 M` is that row's change.
A row landed twice, once inside a merger branch and once on `main`, keeps the
later landing. That gives 67 rows, landed 2026-09-26 to 2026-10-12. Q1661,
Q1766 and their merger branch were one change, and are counted once. 64 of the
rows changed code; 3 changed only the in-file tests.

`git log --first-parent -- arrange.rs` is the wrong tool for this. A merger
branch that merged `origin/main` and was then fast-forwarded puts that merge on
the first-parent line. Its diff from its first parent is the *other* rows'
change, and the merges it brought in are off the line and never seen.

**The items.** The top-level items of today's file, read off rustfmt's layout
the way `just src` reads them: a header at column 0, the closing `}` at column
0, and methods inside `impl` and items inside `mod` at four spaces. A hunk
touches the items its old lines overlap in `M^1` and the items its new lines
overlap in `M`. An item since renamed or removed counts as the nearest item
above it that today's file still has.

**The open rows** are read from `docs/queue.md` at `944bc593`. A row's text
"names" a function when a word in it is the name of a function in
`arrange.rs`. Names of six letters or fewer, and `exactly` and `direction`,
match too much and are left out.

### 441.2 Where the fixes land

Cut today's code, lines 1 to 10 409, into 500-line blocks, and count one
change for each row that changed an item in a block. Every block took between
8 and 34 changes, and none stood out. The items most often changed were
`crossing_for` (11 rows), the `use` block (10), and `place_the_meets`,
`how_a_crossing_runs`, `a_pair_that_crosses_in_a_conic` and
`a_corner_is_this_place` (8 each). No item was changed by more than 11 of the
67 rows.

So no single small file would take the heat off. The rows are spread out, which
is what a split needs, if each row stays inside one piece.

### 441.3 The ten files

Each stage is one or more of the calls `Arrangement::of` makes, in its order
(lines 1125 to 1139). A free function goes to the file whose methods call it
most. Line and row counts are for code only; the six test modules are 4 816
lines more.

| file | what it holds | lines | rows that changed it |
|---|---|---|---|
| `arrange/mod.rs` | module doc, the types (`Arrangement`, `Crossing`, `Node`, `Arc`), `of`, `pieces`, `piece_from` to the end of the impl | 706 | 18 |
| `names.rs` | one name per cut surface, lines 160 to 683 (`canonical_cut_surfaces`, `name_of_surface_bits`, `three_planes_faced_the_one_way`) | 509 | 1 |
| `draw.rs` | `the_whole_surface` to `picks_the_whole_of` | 882 | 14 |
| `place.rs` | corners, meets and touches (`place_the_corners` to `the_faces_across_split_it_no_more`) | 1 268 | 27 |
| `runs.rs` | how each crossing runs: `work_out_how_each_crossing_runs`, `how_a_crossing_runs`, the curve each pair of surfaces cuts | 1 638 | 18 |
| `order.rs` | the nodes along a crossing (`merge_points_that_are_one_point` to `one_straight_line`, `the_one_the_start_is`) | 1 057 | 17 |
| `arcs.rs` | `cut_the_crossings_into_arcs` to `a_touch_is_at` | 621 | 16 |
| `turn.rs` | the walk round a cell, and the turn at a node (`walk_the_cycles` to `is_an_end_of_the_halves`) | 982 | 11 |
| `cells.rs` | cells, lumps and windings (`group_the_cycles_into_cells` to `lump_of_each_cycle`) | 1 666 | 12 |
| `side.rs` | which side of the cuts a piece is on (`which_side_of_everything` to `stretches_inside`) | 763 | 11 |

Each file is a child module of `arrange` with its own `impl Arrangement`
block. A child module sees the struct's private fields; a method called from
another file needs `pub(super)`.

`mod.rs` is changed by 18 rows, because a fix that adds a field to `Crossing`
or `Node` changes the type as well as its stage. That is the floor under every
split along these lines.

### 441.4 What the split buys, if every row names its file

| split | pairs of rows that share a file | pairs landed within 6 h |
|---|---|---|
| none (today) | 100% | 100% |
| five stage files | 60% (1 207 of 2 016) | 65% |
| the ten files of 441.3 | 43% (868 of 2 016) | 46% (48 of 105) |

Going past ten gains little. Half the rows change more than one stage: 31 of 64
changed one file, 14 two, 6 three, and 13 four or more.

### 441.5 Why it does not buy that now

**A row would have to name its file before the work, and the rows' text gets it
right half the time.** 43 of the 64 landed rows named an `arrange.rs` function
in their queue text or as `#item`. 14 changed exactly the files those names fall
in, 22 stayed within them, and 21 changed a file their text never pointed at.
A write set built from the text would have been too narrow for 21 lanes, each
reporting `out_of_write_set_needs` and sent again. That is §424.5's case
against naming items before the trace, and it holds at file level too.

**A third of the open rows hold the directory.** Of 68 open rows that hold
`arrange.rs` or a directory over it, 27 hold all of `fuse/` (15 of them tier
1), 38 hold `arrange.rs` whole, and 3 hold items. A row holding `fuse/` meets
every file in it, however many there are. Only 22 of the 38 whole-file rows
name a function, which is what a row would need to be pointed at one file.

**The slots cap what any finer hold can win.** There are four slots, verifiers
counted (§397.7). Q1531 lets two lanes into `arrange.rs` with no row rewritten.
A split could let a third or fourth in, but only once every other slot is free.

**The split has its own cost, once.** Git does not follow one file into ten, so
it conflicts with every branch in flight that touches `arrange.rs`. The 41 open
rows naming it would each be dropped and added again, since rows have no edit
verb, and every `after:` that waits on one goes with it.

The in-file tests are a third of the file, 4 816 lines. Moving them out is not
a fix for contention: 3 of 67 rows changed only those tests.

### 441.6 What is decided

- **No split now.** Q1530 (a lane narrows its hold) and Q1531 (two lanes in
  `arrange.rs`) go first, as §424.6 has them. Neither needs a row to guess
  where its fix lands.
- **If Q1532's count sends §424.4 back, 441.3's ten files are the split.**
  Rows are then written to name a file only after a trace has found it
  (§230.5). A row with no trace behind it names `arrange/` whole.
- **What would change this:** most fix rows being written after a trace, so
  that the text-to-file match of 441.5 goes well past half. Measure that the
  same way before splitting.
