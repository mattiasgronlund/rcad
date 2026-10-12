## 392. Validate reads which way a three-quarter bend walks its circles — 2026-10-03

*Queue row Q427, which was Q103 (refuses/B, §243.3, §243.6, §343.3), off
`2ad40445`. `Model::validate` refused a sound three-quarter bend
`AClosedEdgeIsWalkedAgainstItsSurface` and passed one walked the wrong way
round. At a circle round a ring's tube it no longer reads a corner. This
section is what was run, why reading the side off the wire checks nothing,
and what the sweep reads now.*

### 392.1 What was run on the base

Every solid here is a piece of the ring `R = 4`, `r = 1` about `+z`, with a
flat cap on a plane through the axis at each end and the wall used forwards,
built straight into the arena by `holds.rs`'s `a_bend_walked` and
`a_piece_of_pipe_walked`. "Seamed" walls are one wire with a seam along the
inner equator; "seamless" walls are two wires, one circle each. "Turned over"
is every coedge on a circle turned over with the caps and seam left as they
were. On `2ad40445`, `Model::validate` gave:

| wall | far corner | walk | caps face | the solid is | base gave |
|---|---|---|---|---|---|
| seamed | `(0, 3, 0)` | as built | out | a quarter bend | `Ok` |
| seamed | `(0, 3, 0)` | turned over | out | caps walked against their faces | `AClosedEdge…` |
| seamed | `(0, −3, 0)` | as built | out | a three-quarter bend | `AClosedEdge…` |
| seamed | `(0, −3, 0)` | turned over | out | **caps walked against their faces** | **`Ok`** |
| seamless | `(0, 3, 0)` | as built | out | a quarter bend | `Ok` |
| seamless | `(0, 3, 0)` | as built | in | caps facing in | `FacesDoNotAgree…` |
| seamless | `(0, 3, 0)` | turned over | out | caps facing in | `AClosedEdge…` |
| seamless | `(0, 3, 0)` | turned over | in | **the other three quarters** | **`AClosedEdge…`** |
| seamless | `(0, −3, 0)` | as built | out | **a three-quarter bend** | **`AClosedEdge…`** |
| seamless | `(0, −3, 0)` | as built | in | caps facing in | `AClosedEdge…` |
| seamless | `(0, −3, 0)` | turned over | out | caps facing in | `FacesDoNotAgree…` |
| seamless | `(0, −3, 0)` | turned over | in | the other quarter | `Ok` |

The four bold rows are wrong: three sound solids refused, and the
wrong-way three-quarter bend, which Q103 never ran, passed. A seamless wall
turned over is the other band between the same circles, so whether it is
sound turns on the caps. A seamed wall turned over walks its circles as the
other band while its seam lies in this one. The wrong-way three-quarter bend
got past the mesh's check that faces agree which way they face as well as the
closed-edge rule; why the mesh did not see it was not traced.

### 392.2 Reading the side off the wire checks nothing

Q103's finding (e) holds on this base, though §361 and Q667 turned the
torus's orientation and the bends' walks round since. The closed-edge rule
says which way a face walks a closed edge, given which side of the partner its
material lies on next to the edge. `holds.rs` runs it backwards, reading the
side off the walk. Handing that side back into the rule gives back the walk it
was read from, whatever the walk is, so no walk is ever refused. The rule is
an identity between a face's walk, its normal and where it lies, and the
partner's normal cancels out of it (worked by hand from §36.6's own
`(∇S × ∇T) × ∇S · ∇T` argument). So what has to be read is where the face
lies, from something other than the walk.

**On a ring between two circles round its tube, the corners cannot say it.**
Both bands between two such circles have the same corners. The corner read
takes the band to be the short one, which is why it refused every band past
half a turn. Only the walks say which band it is, and for a seamed wall the
seam says it too.

### 392.3 What the sweep reads now

At a closed edge that is a circle round a ring's tube, that is between a ring
and a plane holding its axis, asked exactly as stored (`holds_the_axis_of`):

- **The ring's face is not asked the rule.** It is held instead to walking
  every other circle round the tube the other way along its edge. Each such
  circle is stored running the way the tube angle grows, and a band runs on
  round the ring from one of its circles and back towards the other, so with
  its material on its left it walks them opposite ways. Two circles walked the
  same way bound no band.
- **A flat face whose outer wire is that circle alone lies inside the tube**
  next to it: the disc a circle round the tube bounds is the tube's
  cross-section. That is fact rather than a read, so the rule now holds each
  such cap's walk to the way it faces. Before, a cap had no corner off the
  wall and was skipped, and only the mesh could catch it, which it does not
  when the wall has a seam (392.1).

Everywhere else the corner is read as before.

After the change, all twelve rows of 392.1 give what the solid is: the six
sound ones `Ok`, the six others refused, every one of them
`AClosedEdgeIsWalkedAgainstItsSurface`. Through `SolidBuilder`,
`two_loops_side_by_side_on_a_torus.rs`'s three-quarter band now builds,
validates and measures `6π²` within its stated error at deflections `0.1`,
`0.03` and `0.01`. A wall walking both its circles the same way round the tube,
with each cap facing the way its own circle is walked, is refused
`AClosedEdgeIsWalkedAgainstItsSurface` both ways round the ring. With the
check on the other circles switched off in place, the anticlockwise one is
still refused, by the mesh, `FacesDoNotAgreeWhichWayTheyFace`. So that check
names the fault earlier rather than being the only thing that catches it.

The tests that fail without the change, with `validate.rs` alone put back to
`2ad40445`: `holds.rs`'s
`a_three_quarter_bends_wall_is_refused_rather_than_read_as_a_quarter`, which
now asserts the bend validates and the wrong-way bend is refused, and
`the_three_quarter_band_between_the_same_two_circles_measures_what_pappus_says`.
`a_wall_walking_both_its_circles_the_same_way_round_the_tube_is_refused`
passes on the base as well, where the corner read refuses it.

**One pin moved, outside this row's files.** `rcad-step`'s
`which_way_a_torus_faces_normal_points.rs`
`a_bend_written_with_every_turn_turned_over_is_refused` asserts
`FacesDoNotAgreeWhichWayTheyFace` and now gets
`AClosedEdgeIsWalkedAgainstItsSurface`. That bend's wall faces into its
material and every coedge is turned over, so each cap is walked against the
way it faces, and the new name is true. It is still refused; the cap is now
caught by the sweep before the mesh. The other 2905 of the workspace's 2906
tests passed.

### 392.4 What this leaves

- `which_way_a_torus_faces_normal_points.rs`'s pin wants
  `AClosedEdgeIsWalkedAgainstItsSurface` (392.3). It was outside this row's
  files.
- `holds_the_axis_of` asks the questions `holds.rs`'s `how_the_axis_lies_on`
  asks, in a second spelling, because that one is private to a file outside
  this row. Make one `pub(crate)` and drop the other.
- A circle round a ring that is **not** round its tube is still read off a
  corner. A plane square to the axis cuts a ring in two such circles, and a
  band running from one round the tube past the other would come back to the
  plane's other side, as a three-quarter bend does. Reasoned and not run, so
  not classed.

  Run on 2026-10-12, by Q855 on `6e27ef88`, and it holds. The lane built the
  ring band that one plane square to the axis cuts (ring 4, tube 1.25 about
  +z, the plane at z = +0.75 and at z = -0.75, facing +z and facing -z), and
  tried all 8 walks of each. In every case exactly one walk builds and
  validates, with its volume near what Pappus's theorem gives, and every
  other walk is refused. Its verifier ran the same probe on the base and got
  the same lines, so Q855 was closed done at `6e27ef88` with nothing merged
  (§433.8, `26aedaaf`).
