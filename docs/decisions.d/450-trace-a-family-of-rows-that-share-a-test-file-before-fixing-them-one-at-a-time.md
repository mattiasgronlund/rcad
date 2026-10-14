## 450. Trace a family of rows that share a test file before fixing them one at a time — 2026-10-14

*Mattias, 2026-10-14, after he was shown why the day had closed more tier 1
rows than it added: "How do we make sure we repeat the pattern?" The pattern
is 446.8's. One trace over four rows found one cause, and one fix closed those
four rows and two more. The answer below was decided by session
`dispatch-2026-10-03-a8` under `CLAUDE.md`'s "Decide it yourself", and written
by the session after it.*

**In short.** Three or more open rows that name one test file in their write
sets are a **family**. When the fill would send a member of a family that has
had no trace, the dispatcher files a trace row for the family and sends that
row instead, and the other members wait on it. The trace says which members
share a cause. It files one fix row for each cause, and names the members that
fix should close. Every fix lane also runs the shapes of the open rows that
share its test file, and the merge closes the ones it now answers. Today that
was done by hand. `just queue families` will list the families once Q2003
lands. Until then the line in 450.2 lists them.

### 450.1 What happened on 10-14

Counted along `origin/main`'s first-parent history of `docs/queue.md`, by
Stockholm day, over the rows open in tier 1 and not parked. Each commit's rows,
read with `tooling/queue/rows.py`'s `Queue`, are compared with the commit
before. A row new in tier 1 counts as added. A row that leaves open tier 1 as
done or dropped counts as closed.

| up to 19:45 | 10-12 | 10-13 | 10-14 |
|---|---|---|---|
| added | 34 | 37 | 21 |
| closed | 38 | 39 | 39 |
| …of them done, class A or B | 11 | 23 | 15 |

Whole days: on 10-12, 52 rows were added and 49 closed, and on 10-13, 54
added and 56 closed. On 10-14 by 20:08, 22 were added and 40 closed, and open
tier 1 fell from 55 to 37. So rows closed at the usual pace, and the fall came
from fewer rows being filed. Why fewer were filed is not established.

**The fix that closed six.** One merge, `b982307a`, landed Q1965. Then
`17532d10` closed Q1793, Q1896, Q1897, Q1929 and Q1991 by hand, "answered by
Q1965". The chain:

- On 10-13, after the Q1930 sweep (§446), Mattias pinned Q1953: trace the
  two-thin-bars family, Q1793, Q1896, Q1897 and Q1929. All four name
  `two_equal_bars_leaning_at_each_other.rs`.
- The trace (446.8) found one test behind 76 of the family's 85 refusals.
  `assemble` asked whether two pipes touch at a corner by an exact `f64` nought
  at the corner's rounded point. It filed Q1965 for that cause, and Q1966 and
  Q1967 for two others.
- Q1965 records on each corner that it is a touch point of two pipes
  (`b36fc5e0`).
- Its lane ran every row of the family on its branch. Q1896's second bar less
  the first answered, which its row expected to stay refused and 446.8 had
  given a cause of its own. Q1991, filed from Q1967, answered too.
- Q1966 and Q1967 closed as well. Q1968, a cause the trace found at other
  angles, is in a lane.

§448 is the other side. It traced nine rows picked by kind of refusal, from
four test files. For most of them the cause it found needs a feature, which
448.7 deferred. So its rows Q1962 and Q1963 were dropped, and they closed
nothing.

What to repeat, then, is two things. First, trace the rows that share a shape
before any one of them is fixed. Second, have each fix lane check the rows
beside its own.

### 450.2 Finding a family

A row names the test file that pins its shape, so rows on one shape share a
test file in their write sets. This line lists the families on a queue. It
strips a `#item` off each entry and leaves out closed and parked rows:

```
git show <sha>:docs/queue.md \
  | awk '/^```/{f=!f; next} f && /^Q[0-9]+ / && $3 !~ /^(done|dropped|parked):/' \
  | awk '{n=split($5, w, ","); for (i=1;i<=n;i++) {p=w[i]; sub(/#.*/, "", p); if (p ~ /tests\/.*\.rs$/) print p, $1"("$2")"}}' \
  | sort | awk '{c[$1]++; r[$1]=r[$1]" "$2} END {for (k in c) if (c[k]>=3) print c[k], k, r[k]}' | sort -rn
```

At `16eab121`, which was main when Q1953 was filed, it prints 26 families. One
of them is `two_equal_bars_leaning_at_each_other.rs` with Q1793(1), Q1896(1),
Q1897(1) and Q1929(1), which is exactly the family Q1953 traced. Over tier 1
alone, with two rows enough, it comes second.

At `6f3ac702` it prints 23. Eight of them have a member in tier 1:

| Rows | Test file | Members, with their tiers |
|---|---|---|
| 10 | `a_piece_no_ray_can_place_names_its_refusal.rs` | Q1948 and Q1949 in 1, 8 in 5 |
| 4 | `degenerate_curved_fuses.rs` | Q1600 and Q1983 in 1, Q1087 and Q885 in 5 |
| 4 | `a_walk_neither_winding_nor_the_axis_places.rs` | Q1988 in 1, 3 in 5 |
| 4 | `a_plane_cut_along_a_cones_other_nappe.rs` | Q1905 in 1, Q1570 in 2.5, Q493 and Q861 in 5 |
| 3 | `meshing_a_plane_cut_along_a_cones_other_nappe.rs` | Q1905 in 1, Q493 and Q861 in 5 |
| 3 | `a_prism_lying_on_one_edge.rs` | Q1923 in 1, Q1783 in 2, Q1784 in 5 |
| 3 | `a_cone_cap_whose_rim_runs_through_its_apex.rs` | Q1873 in 1, Q1631 and Q1987 in 2 |
| 3 | `a_bar_lying_along_a_plane_or_a_bar.rs` | Q1765, Q1903 and Q1983, all in 1 |

Five more have a member in tier 2. Ten have members in tier 5 only. The largest
of those are `a_solid_written_out_and_read_back_is_the_same_solid.rs` with 6
rows, and `a_turned_plane_exactly_proportional_to_another.rs`,
`an_edge_between_two_planes_its_curve_leaves.rs` and
`cutting_a_patch_into_triangles.rs` with 5 each. The biggest family of all,
`a_joined_flat_face_whose_rings_enclose_no_area.rs` with 10 rows, has 9 in tier
5 and Q1225 in tier 2.

**What it misses.** A family is a place to trace. It is not a finding.

- **A family whose rows name different test files.** Q1765, Q1842, Q1936 and
  Q1839, §448's rows of one kind, name four files.
- **Different causes in one file.** A shared file does not mean a shared cause.
  `degenerate_curved_fuses.rs` holds many unrelated shapes.
- **Grouping by the function that refuses splits a family.** 446.8 found one
  test behind five different errors raised in different places, so grouping by
  446.5's table of raising functions would have split the two-thin-bars family.
  A shared shape is the better mark.

### 450.3 The rule

1. **A family is three or more open rows, not parked, that name one
   `tests/*.rs` file in their write sets.** Three is the bar 448.4 set for one
   cause being worth a fix row of its own. A family that has had a trace is not
   a family again until three rows that no trace named share its file.
2. **The trace is filed when the fill reaches the family, not before.** When
   `just queue next` takes a row that is in a family with no trace, the
   dispatcher files the trace, puts the other members after it, and takes the
   trace in that slot. All of that is one `queue apply`:

   ```
   just queue apply add <tier> opus - "from/- Trace the family on <test file>: <ids>. …§450" + after <member> <what it already waits on>,@1 + … + take @1 <branch>
   ```

   So no row is moved ahead of where the tier order already had it. The fill
   reaches a family's first member when it is due, and only then does the trace
   take that member's slot. Filing all 23 traces now would have added 23 rows
   in one go. Most of them would sit in tier 5, where the fill does not reach
   for a long time.
3. **The trace row.**
   - Its write set is `-`, as a diagnosis row's is (§230.5), since the trace
     changes nothing in `src/`.
   - It sits in the tier of the member the fill reaches first. Its fix rows do
     the same with the members each fix closes. `after` refuses to make a row
     with a class wait on a row in a later tier (435.6). With this rule no
     member is ever in a tier ahead of the row it waits on, so it never
     refuses.
   - Its text names the test file and every member, and asks what Q1953's
     asked. Run each member's shapes under `Policy::certified()` at the base.
     Name the first step that goes wrong, not only the function that gives up.
     Say which members share a cause, and whether each cause is a gap or a
     limit (§16.10).
   - It ends as Q1953 did. It files one fix row per cause, each naming the
     members it should close and the pin of each. Each of those members is
     set to wait on its fix row with `["after", "<member>", "@n"]`, which
     `just land` already takes as a step. A member that stands alone keeps
     waiting on the trace, and the trace is done, so the fill can take it
     again.
   - A member in a running lane, and a pinned member, are left alone. `after`
     refuses a pinned row anyway (§430.4). The trace still reads them.
4. **A fix lane runs its family.** Prep lists, as a fact, the open rows that
   share a test file with the lane's row. At the end, the lane runs each of
   those rows' shapes on its branch and reports the ones that now answer, with
   the runs. The verifier runs those again. The merge closes each one it
   confirms with the lane's row, "answered by", in the same `queue apply`.
   That is what `17532d10` did by hand. Q2004 builds it.

### 450.4 The tier 5 families are Mattias's call

The largest families are class C, in tier 5. Under this rule they are traced
when the fill reaches them, which is after every tier above. One exception is a
family with a member higher up: its trace sits in that member's tier and reads
the tier 5 members with it. `a_piece_no_ray_can_place_names_its_refusal.rs`
will be traced in tier 1 when Q1948 comes up. Whether to trace the tier 5-only
families sooner is a change to §230.1's tier order. That is Mattias's to make,
and this section does not make it.

### 450.5 The rows filed

- **Q2003**, tier 2.2, `just queue families`. It lists the families as 450.3's
  rule 1 defines them. It leaves out a family that has had a trace: a row whose
  text starts "Trace the family on <file>" names its members. The count goes in
  `just queue`'s plain output, and the list goes on the queue page.
  `dispatch-lanes` step 4 then names the recipe in place of 450.2's line. Its
  test is a copy of `16eab121`'s queue, which gives the two-thin-bars family as
  above.
- **Q2004**, tier 2.2, rule 4. Prep's fact, the lane's report field, the
  verifier's check, and the merge closing what answers.

Until Q2003 lands, `dispatch-lanes` step 4 points at 450.2's line, and the
dispatcher runs it at each fill.
