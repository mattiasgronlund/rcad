## 115. A cap circle is walked by this kernel's rule and not by the file's flag, fixed 2026-09-17

**`Orientation` means two different things and a STEP file answers only one of
them.** On an edge with two corners it means *from the start corner towards the
end one*, and `ORIENTED_EDGE.orientation` — taken against the bound's own
orientation — says exactly that. On a **closed** edge there are no two corners
to run between, so that meaning is empty, and §30.8 item 10 gives the word its
other meaning there: the way the curved surface either side of the edge winds.

`rcad-step`'s reader was handing the file's flag straight through for both. On a
cap circle the two questions are unrelated, so the answer was right by luck, and
the luck ran about even.

### 115.1 What the rule is, and that the reader is only reading it

`validate.rs`'s `a_closed_crossing_is_walked_the_way_it_is_stored` is the rule,
and `Model::validate` has read it back since §30.8 item 10 became a check:
`Invalid::AClosedEdgeIsWalkedAgainstItsSurface` is a face walking a closed edge
the other way. The reader now asks the same question before it builds:

- the way the edge is **stored** running is the curved surface's own longitude
  at the corner, which `Surface::the_way_its_first_coordinate_runs_at` gives;
- the way this use of the edge **travels** is the tangent to the `CIRCLE` or
  `ELLIPSE` under it, `w × (corner − centre)`, turned round by
  `EDGE_CURVE.same_sense` and by the way the loop uses the edge;
- `Orientation::Forward` is the two agreeing in sign.

**Nothing here is a second spelling of the rule**, which §23.8 would forbid.
The longitude has been one public method in `rcad-geom` since it was written for
exactly this, and the reader calls it rather than working it out again.

### 115.2 Why it takes a pass over the whole shell first

The rule needs the surfaces on **both** sides of the edge, and a face being
described knows only its own. So `Walk::find_the_closed_edges` walks the
`CLOSED_SHELL` first and records, for every `EDGE_CURVE` whose two
`VERTEX_POINT`s are the same entity, the stored surface of each face that uses
it. A face the pre-pass cannot read is passed over rather than refused: the walk
proper is about to refuse it by name, and a second copy of that decision could
only come to disagree with the first.

Which of the two surfaces names the direction is `the_curved_one_of`, and it has
to be the one `rcad-topo` picks or a file is walked against the very rule it is
read by. `Surface::how_it_sorts_against` is that order. It was narrowed to
`pub(crate)` by `c570205b` "until it has a caller", with a doc comment that
claimed `rcad-step` already asked and a grep that said otherwise; this is the
caller, and the widening is in the commit that brings it.

### 115.3 The sign is a hint, which is what makes plain `f64` allowed

The dot product is compared against zero with no certified predicate, and that
is the same standing `Walk::area_swept_by`'s winding sweep already has (§44.2).
`Model::validate` asks the certified question of the same edge moments later, so
a hint that came out wrong is a refusal **by name** and never a solid with a
seam running backwards. Crash before corrupt, and here not even a crash.

The one place the hint is a guess is a travel that dots to exactly zero against
the longitude. `which_way_a_closed_crossing_turns` reads the surface's latitude
there and this has no gradient to build one from, so `Forward` is taken and that
named refusal is the whole of the cost.

### 115.4 Measured, and the count is of files and not of edges

`mise exec -- just t how_often_a_real_file_imports --no-capture`, against the
323-file KiCad corpus, with `crates/rcad-step/src/brep.rs` at `44c59a7e` and
then at this change:

| | files importing | rate |
|---|---|---|
| before | 97 / 323 | 30.0 % |
| after | 110 / 323 | 34.1 % |

**Thirteen files gained and none lost** — the test pins the set by name rather
than by rate for exactly this reason (§44.10), so "thirteen" is a set difference
it printed and not a subtraction anyone did. They are the nine
`LED_D1.8mm_*_Horizontal`, `Relay_DPDT_Finder_40.52`, `Relay_SPDT_Finder_40.31`,
`Relay_SPDT_Finder_40.51` and `SW_NKK_BB15AH`. The NIST table did not move: 1 of
44 either way.

**Sixteen files were stopped first by the closed-edge reason and thirteen came
through**, which is §44.11 item 5 from both sides at once. Two met a later
blocker the same run. One, `LED_D5.0mm-3`, is still refused as *a face walks a
closed edge against the way its surface winds* — the reader's hint and the
certified rule disagree about that file and **which of them is right is not
measured**. It is booked here as the open end of this section rather than left
for someone to find in a table.

### 115.5 The test names a property, not a file

`tests/which_way_a_cap_circle_is_walked.rs` writes a round bar by hand — two
discs and a cylindrical wall, two cap circles, one `VERTEX_POINT` each — and
writes it **twice**, changing only which way round the `CYLINDRICAL_SURFACE`'s
own axis is spelled. A cylinder is the same surface either way: the same point
set, the same radial outward normal, the same material inside, so `same_sense`
stays `.T.` and not one other character of the file moves. Both spellings have
to import and both have to enclose `πr²h`.

What makes that a test and not a tautology is that the kernel's rule turns round
with that axis while every flag in the file stays put. A reader that hands the
flag through gives both spellings the same answer and so must be wrong about
one: with `brep.rs` at `44c59a7e` the bar with its wall axis written downwards
is refused, and the change is what makes both come through.

It names no corpus file and no count, because §113's fixture was rescued out
from under its own test half a day after it was written.

**The two volumes are asked to agree within the measurement's own error rather
than bit for bit.** They came out two ulps apart — the mesher summing one bar's
faces in the order one spelling puts them in and the other's in the other. Two
ulps is not a fact about the bar, and a test that pinned them would be measuring
the summation order.

