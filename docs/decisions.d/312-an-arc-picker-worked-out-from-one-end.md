## 312. An arc's picker is worked out at one end, whichever face named it — 2026-09-27

*Q170's lane. The fuse fuzzer's both-ways check (§129.5 item 3) panicked on 84
of 2026-09-26's crash files: swapping the two sides of an intersection changed
the answer.*

### 312.1 The 84 are one fault

The night's triage key writes every number as `N`, and three files printed 5,
4 and 4 faces, so the 84 were checked rather than assumed. Replayed through
`tests/fuzzing/fuse.rs` on the base (`4ac75a74`), the full messages split
1 + 62 + 21 by face count (3, 4 and 5 faces). Every half that disagrees, on
both nights, is the same pair and the same difference:

| night | halves that disagree | inputs | rule | what differs |
|---|---|---|---|---|
| 2026-09-26 | 84 | a six-faced block and a ball | intersection, 79 certified and 5 tolerant | only the pickers |
| 2026-09-27 | 107 | the same | intersection, 102 certified and 5 tolerant | only the pickers |

"Only the pickers" means the two answers are the same bit for bit once each
`PartOfWhereItCrosses` has its half-spaces taken out: the same corners, the
same surfaces, the same way round, the same runs. What is left differs in the
last bits — `[3.0010877488956558, −0.6709234432247422, 1.9649986854431174]`
offset `2.98445841377732` one way, `[3.001087748895655, …, 1.964998685443117]`
offset `2.984458413777319` the other, on the sample
`crash-0687348c…`.

### 312.2 Neither answer is wrong

Both are the same solid. A picker is the half-space through an arc's chord,
square to the plane the arc lies in, and it only has to hold the one arc and
not the other. For a circle the two ends give the same normal: `v(c·c) −
c(c·v)` is the part of the tangent square to the chord, times `|c|²`, and the
tangent makes the same angle with the chord at both ends. So the two pickers
are one plane rounded two ways, both hold both ends of the arc, and both put
the circle's centre on the same side. The fault is not a wrong arc; it is that
the numbers written down depend on the path taken, which is exactly what
`fuses_that_must_agree.rs` and the fuzzer hold the fuse to.

### 312.3 Where the path got in

`Edges::of` names an arc the first time any face reaches its edge, from the
corner that face walks away from. In the forward order the block's pieces come
first and name each arc from the plane's side; in the swapped order the ball's
pieces come first, and they walk the same edge the other way. A probe in
`Edge::how_it_runs` on the sample showed all six arcs named from opposite ends
in the two orders, with the facing vectors differing only in their last bits.

### 312.4 The fix, and the family it was picked from

`Edge::the_way_across_the_chord_into_the_arc` works the facing out at **the end
whose stored point comes first, compared as numbers**, from the crossing's
direction there, and takes only a sign from the named direction: the old
facing, squared off against its chord the old way, points to the arc's side,
and the new one is turned to agree with it. Their dot is `±|a||b|` for two
vectors square to one chord, so it is far from zero wherever the old facing
was usable, and exactly zero where the old facing was — which the old code
refused, and this refuses too.

The family is "make the facing a function of the edge and not of which face
named it". Two members were weighed:

- **Add the facings from both ends.** Symmetric with no tie-break at all, but
  for a crossing that is not flat — a ball against a pipe — the two ends'
  facings lie in different planes and their sum is a vector no order of the
  old code ever produced. Its correctness would need an argument of its own.
- **Pick one end by something both orders agree on.** Every picker it makes is
  one the old code made in one of the two orders, so it adds no new geometric
  claim. Corner indices will not do — they follow the walk — and the stored
  points do: both orders already agree on them bit for bit, or the two answers
  would differ anyway. Taken.

This is not the "do not compare coordinates" trap. Nothing is joined on the
comparison; it picks which of two known corners to start from. `-0.0` and
`0.0` compare equal, so they choose alike, and two ends equal as numbers have
no chord, which the old code refused and this refuses too.

### 312.5 What pins it

- `fuzz/corpus/two_solids_fused_by_a_stranger/09968a30…`, the smallest of the
  84 (3047 bytes, the four-face group). On the base its replay through
  `two_models_split_and_fused` panicked at `fuse.rs:490`; with the fix it
  replays clean.
- `tests/a_turned_block_and_a_ball_intersected_either_way_round.rs`: a cube of
  side 2 turned one radian about `(1, 1, 1)` and a ball of radius 1 poking in
  at a corner, intersected both ways and held to `same_answer`.
  The lane's run of it against the base was refused by the permission
  classifier, so the lane only argued from §312.3 that it fails without the
  fix. Q170's verifier ran it: with the base's `crates/rcad-topo/src` and the
  new tests kept, both this test and the seed's replay fail — this one with
  "putting the ball first changed the answer to a Intersection", the seed at
  `fuse.rs:490` — and both pass again on the branch.

### 312.6 The first version read the sign off the tangent, and two suites caught it

The first version of the fix took the sign as `towards · (∇S × ∇T)` at the
named corner, the reading `walks_a_closed_up_run_the_way_it_is_stored` makes,
and turned the crossing's direction at the chosen end by it. It cleared all 84
files and failed two things in the gate:

- **`fuse_laws_over_a_cone.rs`**, two tests: `(block ∪ frustum) ∪ cube` went
  from answered to `CannotMeshAFace`. A probe showed one circle's two arcs
  between `(3.5, 6.41, 0)` and `(3.5, 3.59, 0)` both given the `+x` side. The
  second arc's `towards` came from `arc_of_a_joined_run`, which is the step
  to a corner the joining dissolved, and a point far enough round an arc
  leans back against the tangent — so its dot with the crossing's direction
  had the wrong sign. The old facing from it was `(−4, 0, 0)`, the right side.
- **`a_narrow_lune_from_a_pole_corner.rs`**, the 10 degree wedge at radius 1,
  as an intersection: `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` became
  `TwoPatchesOfTheMeshDrewTheSameSide`. The arc behind it runs from the pole
  `(0, 0, 1)` to `(0, −1.28e−15, 1)`. Named from the pole the old facing was
  exactly zero, and the first version, reading only a sign, went on from the
  other end, where the facing is `(0, 0, 2.09e−45)` — the right side, and the
  size an arc that short has — and was refused by the mesh one stage later.

Taking the side from the old facing itself fixes both: it is right wherever
the old code was, and it is zero exactly where the old code refused. Both
suites pass unchanged, and the 84 and the 107 still replay clean.

After the fix, all 102 of 2026-09-26's files and all 152 of 2026-09-27's replay
without this panic. What still panics on both nights is a different fault, the
`EdgeRun::RightRoundWhereItCrosses` rim `comparison.rs` has no shape for (11
and 13 files, §216).
